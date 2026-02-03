//! Visual Validator - Dimensional Assertion Utilities
//!
//! Sprint 30: Provides assertion functions for validating that table output
//! respects terminal width constraints. These utilities address the Sprint 29
//! testing gap where tests passed but output was garbled at real terminal widths.
//!
//! # Overview
//!
//! This module provides three core assertion functions:
//!
//! 1. [`assert_no_overflow`] - Validates that no line exceeds terminal width
//! 2. [`assert_column_widths_within_terminal`] - Validates column width calculations
//! 3. [`assert_truncation_markers_present`] - Validates truncation indicator placement
//!
//! # Design Philosophy
//!
//! These assertions are designed to be:
//! - **Detailed**: Panic messages include line numbers, actual vs expected widths,
//!   and the offending content for easy debugging
//! - **Unicode-aware**: Uses Unicode width calculations, not byte length
//! - **Robust**: Handles edge cases like empty output, single lines, and Unicode
//!
//! # Example
//!
//! ```ignore
//! use tests::tools::visual_validator::{assert_no_overflow, assert_column_widths_within_terminal};
//!
//! let table_output = "| col1 | col2 |\n| ---- | ---- |\n| val  | val  |";
//!
//! // Ensure no line exceeds 80 characters
//! assert_no_overflow(&table_output, 80);
//!
//! // Ensure column widths fit within 80-character terminal
//! assert_column_widths_within_terminal(&table_output, 80);
//! ```
//!
//! # Dependencies
//!
//! This module uses only the standard library plus `unicode-width` for
//! accurate Unicode character width calculations (already a dependency
//! of the main crate).

use unicode_width::UnicodeWidthStr;

/// Validates that no line in the output exceeds max_width.
///
/// This is the primary dimensional constraint check - if any line exceeds
/// the terminal width, the table will wrap and become garbled.
///
/// # Arguments
///
/// * `output` - The table output string to validate (may be multi-line)
/// * `max_width` - Maximum allowed display width for any line
///
/// # Panics
///
/// Panics with a detailed message if any line exceeds `max_width`, including:
/// - Line number (1-indexed)
/// - Actual width vs maximum allowed
/// - The content of the offending line (truncated if very long)
///
/// # Edge Cases
///
/// - Empty output: Passes (no lines to check)
/// - Single line: Validates that single line
/// - Unicode characters: Uses display width, not byte count
/// - ANSI escape codes: Currently included in width calculation
///   (TODO: Consider stripping ANSI codes for pure content width)
///
/// # Example
///
/// ```ignore
/// // Passes - all lines under 80 characters
/// assert_no_overflow("short line\nanother short line", 80);
///
/// // Panics - second line too wide
/// let wide = "ok\n".to_string() + &"x".repeat(100);
/// assert_no_overflow(&wide, 80); // panics!
/// ```
pub fn assert_no_overflow(output: &str, max_width: usize) {
    for (line_idx, line) in output.lines().enumerate() {
        let line_width = UnicodeWidthStr::width(line);
        if line_width > max_width {
            // Build detailed error message
            let display_content = if line.len() > 100 {
                format!("{}...", &line[..100])
            } else {
                line.to_string()
            };

            panic!(
                "\n\
                 =========================================\n\
                 OVERFLOW DETECTED - Line exceeds terminal width\n\
                 =========================================\n\
                 Line number: {} (1-indexed)\n\
                 Actual width: {} characters\n\
                 Maximum allowed: {} characters\n\
                 Overflow by: {} characters\n\
                 -----------------------------------------\n\
                 Line content:\n\
                 {}\n\
                 =========================================\n",
                line_idx + 1,
                line_width,
                max_width,
                line_width - max_width,
                display_content
            );
        }
    }
}

/// Validates that column widths fit within terminal_width.
///
/// This function parses the table structure and validates that the calculated
/// total column width (including borders and padding) does not exceed the
/// terminal width.
///
/// # Arguments
///
/// * `output` - The table output string (must be a formatted table with borders)
/// * `terminal_width` - The terminal width to validate against
///
/// # Panics
///
/// Panics with a detailed message if:
/// - Total table width exceeds terminal width
/// - Unable to parse column structure (with suggestion to check table format)
///
/// The panic message includes:
/// - Number of columns detected
/// - Individual column widths
/// - Total table width vs terminal width
///
/// # Table Format Expected
///
/// Expects tables with pipe borders (`|` or `│`) separating columns:
/// ```text
/// │ col1 │ col2 │ col3 │
/// │ val1 │ val2 │ val3 │
/// ```
///
/// # Edge Cases
///
/// - Empty output: Passes (no table to check)
/// - Non-table output: Passes with warning (no columns detected)
/// - Single column: Validates single column width
///
/// # Example
///
/// ```ignore
/// let table = "│ id │ name │\n│ 1  │ foo  │";
/// assert_column_widths_within_terminal(&table, 80); // passes
/// assert_column_widths_within_terminal(&table, 10); // panics - too narrow!
/// ```
pub fn assert_column_widths_within_terminal(output: &str, terminal_width: usize) {
    // First, check basic overflow (every line must fit)
    assert_no_overflow(output, terminal_width);

    // Parse column structure from output
    let column_analysis = analyze_table_structure(output);

    if let Some(analysis) = column_analysis {
        // Verify total width fits
        if analysis.total_width > terminal_width {
            panic!(
                "\n\
                 =========================================\n\
                 COLUMN WIDTH VIOLATION - Table too wide for terminal\n\
                 =========================================\n\
                 Terminal width: {} characters\n\
                 Table width: {} characters\n\
                 Overflow by: {} characters\n\
                 -----------------------------------------\n\
                 Column count: {}\n\
                 Column widths: {:?}\n\
                 -----------------------------------------\n\
                 Suggestion: Reduce column widths or enable truncation\n\
                 =========================================\n",
                terminal_width,
                analysis.total_width,
                analysis.total_width - terminal_width,
                analysis.column_widths.len(),
                analysis.column_widths
            );
        }
    }
    // If no table structure detected, the basic overflow check is sufficient
}

/// Validates that truncation markers appear at expected column indices.
///
/// Truncation markers (`...` or `…`) indicate that cell content was truncated
/// to fit within column width constraints. This assertion validates that
/// truncation markers appear in the expected columns.
///
/// # Arguments
///
/// * `output` - The table output string
/// * `expected_columns` - Slice of 0-indexed column indices that should contain
///   at least one truncation marker
///
/// # Panics
///
/// Panics if any expected column does not contain a truncation marker.
/// The panic message includes:
/// - Which column indices were expected to have truncation
/// - Which columns actually have truncation markers
/// - Sample content from the column
///
/// # Truncation Markers Detected
///
/// - `…` (Unicode ellipsis, U+2026)
/// - `...` (ASCII triple dot)
///
/// # Edge Cases
///
/// - Empty expected slice: Always passes (no expectations)
/// - Column index out of range: Panics with clear error
/// - Multiple truncations in same column: Counts as satisfied
///
/// # Example
///
/// ```ignore
/// let table = "│ id │ long_na… │ desc │\n│ 1  │ truncat… │ ok   │";
///
/// // Expect truncation in column 1 (0-indexed)
/// assert_truncation_markers_present(&table, &[1]); // passes
///
/// // Expect truncation in column 2 - but there's none!
/// assert_truncation_markers_present(&table, &[2]); // panics!
/// ```
pub fn assert_truncation_markers_present(output: &str, expected_columns: &[usize]) {
    if expected_columns.is_empty() {
        return; // No expectations, pass
    }

    let truncation_analysis = analyze_truncation_markers(output);

    for &expected_col in expected_columns {
        if !truncation_analysis.columns_with_truncation.contains(&expected_col) {
            // Find sample content from that column for debugging
            let sample = get_column_sample(output, expected_col);

            panic!(
                "\n\
                 =========================================\n\
                 TRUNCATION MARKER MISSING\n\
                 =========================================\n\
                 Expected truncation in column: {} (0-indexed)\n\
                 Columns with truncation found: {:?}\n\
                 -----------------------------------------\n\
                 Sample content from column {}:\n\
                 {}\n\
                 -----------------------------------------\n\
                 Suggestion: Check if content is actually being truncated\n\
                 or if truncation markers are using different characters\n\
                 =========================================\n",
                expected_col,
                truncation_analysis.columns_with_truncation,
                expected_col,
                sample.unwrap_or_else(|| "[column not found]".to_string())
            );
        }
    }
}

// ============================================================================
// Internal Analysis Structures and Functions
// ============================================================================

/// Result of analyzing table column structure
#[derive(Debug)]
struct TableStructureAnalysis {
    /// Width of each column (content width, not including borders)
    column_widths: Vec<usize>,
    /// Total width of the table (including all borders and padding)
    total_width: usize,
}

/// Result of analyzing truncation markers
#[derive(Debug)]
struct TruncationAnalysis {
    /// 0-indexed columns that contain at least one truncation marker
    columns_with_truncation: Vec<usize>,
}

/// Analyze the table structure to extract column widths
///
/// Parses table rows looking for pipe-delimited columns and calculates
/// the width of each column.
fn analyze_table_structure(output: &str) -> Option<TableStructureAnalysis> {
    // Find a data row (non-border row with pipe characters)
    let data_row = output.lines().find(|line| {
        let has_pipes = line.contains('│') || line.contains('|');
        let not_border = !line
            .chars()
            .all(|c| is_border_char(c) || c.is_whitespace());
        has_pipes && not_border
    })?;

    // Parse cells from the row
    let cells = parse_table_row(data_row);
    if cells.is_empty() {
        return None;
    }

    // Calculate column widths from cell content
    // Each column width = content width (cells already trimmed in parse)
    let column_widths: Vec<usize> = cells
        .iter()
        .map(|cell| UnicodeWidthStr::width(cell.as_str()))
        .collect();

    // Calculate total table width:
    // - 1 char for leading border
    // - For each column: space + content + space + border = width + 3
    let total_width = 1 + column_widths.iter().map(|w| w + 3).sum::<usize>();

    Some(TableStructureAnalysis {
        column_widths,
        total_width,
    })
}

/// Analyze output for truncation markers
///
/// Scans all data rows and identifies which columns contain truncation markers.
fn analyze_truncation_markers(output: &str) -> TruncationAnalysis {
    let mut columns_with_truncation: Vec<usize> = Vec::new();

    for line in output.lines() {
        // Skip border lines
        if line.chars().all(|c| is_border_char(c) || c.is_whitespace()) {
            continue;
        }

        let cells = parse_table_row(line);
        for (col_idx, cell) in cells.iter().enumerate() {
            if contains_truncation_marker(cell) && !columns_with_truncation.contains(&col_idx) {
                columns_with_truncation.push(col_idx);
            }
        }
    }

    columns_with_truncation.sort_unstable();
    TruncationAnalysis {
        columns_with_truncation,
    }
}

/// Parse a table row into cell contents
///
/// Splits on pipe characters and trims whitespace from each cell.
fn parse_table_row(line: &str) -> Vec<String> {
    // Handle both Unicode and ASCII pipes
    let normalized = line.replace('│', "|");

    let parts: Vec<&str> = normalized.split('|').collect();

    // Skip first and last parts (outside the table borders)
    if parts.len() <= 2 {
        return vec![];
    }

    parts[1..parts.len() - 1]
        .iter()
        .map(|s| s.trim().to_string())
        .collect()
}

/// Check if a character is a table border character
fn is_border_char(c: char) -> bool {
    matches!(
        c,
        '─' | '│'
            | '┌'
            | '┐'
            | '├'
            | '┤'
            | '└'
            | '┘'
            | '╭'
            | '╮'
            | '╰'
            | '╯'
            | '┼'
            | '┬'
            | '┴'
            | '|'
            | '-'
            | '+'
    )
}

/// Check if a string contains a truncation marker
fn contains_truncation_marker(s: &str) -> bool {
    s.contains('…') || s.contains("...")
}

/// Get a sample of content from a specific column for error reporting
fn get_column_sample(output: &str, column_idx: usize) -> Option<String> {
    let mut samples: Vec<String> = Vec::new();

    for line in output.lines() {
        // Skip border lines
        if line.chars().all(|c| is_border_char(c) || c.is_whitespace()) {
            continue;
        }

        let cells = parse_table_row(line);
        if let Some(cell) = cells.get(column_idx) {
            if samples.len() < 3 {
                // Collect up to 3 samples
                samples.push(cell.clone());
            }
        }
    }

    if samples.is_empty() {
        None
    } else {
        Some(samples.join("\n"))
    }
}

// ============================================================================
// Additional Utility Functions
// ============================================================================

/// Measure the display width of a string (Unicode-aware)
///
/// This is a convenience wrapper around `unicode_width::UnicodeWidthStr::width()`
/// for use in tests.
///
/// # Example
///
/// ```ignore
/// assert_eq!(display_width("hello"), 5);
/// assert_eq!(display_width(""), 2); // Full-width CJK character
/// ```
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Get the maximum line width from multi-line output
///
/// Useful for checking the widest line in table output.
///
/// # Example
///
/// ```ignore
/// let output = "short\nvery long line here\nmedium";
/// assert_eq!(max_line_width(&output), 19);
/// ```
pub fn max_line_width(output: &str) -> usize {
    output
        .lines()
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0)
}

/// Count the number of lines in output
///
/// Handles empty strings correctly (returns 0, not 1).
pub fn line_count(output: &str) -> usize {
    if output.is_empty() {
        0
    } else {
        output.lines().count()
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // assert_no_overflow tests
    // ========================================================================

    #[test]
    fn test_no_overflow_empty_output() {
        // Empty output should pass
        assert_no_overflow("", 80);
    }

    #[test]
    fn test_no_overflow_single_line_under() {
        assert_no_overflow("short line", 80);
    }

    #[test]
    fn test_no_overflow_single_line_exact() {
        let line = "x".repeat(80);
        assert_no_overflow(&line, 80);
    }

    #[test]
    #[should_panic(expected = "OVERFLOW DETECTED")]
    fn test_no_overflow_single_line_over() {
        let line = "x".repeat(81);
        assert_no_overflow(&line, 80);
    }

    #[test]
    fn test_no_overflow_multi_line_all_under() {
        let output = "line one\nline two\nline three";
        assert_no_overflow(output, 80);
    }

    #[test]
    #[should_panic(expected = "Line number: 2")]
    fn test_no_overflow_multi_line_second_over() {
        let line2 = "x".repeat(100);
        let output = format!("short\n{}\nshort", line2);
        assert_no_overflow(&output, 80);
    }

    #[test]
    fn test_no_overflow_unicode_characters() {
        // Unicode characters have different display widths
        // CJK characters are typically 2 display columns wide
        let output = "hello"; // 5 ASCII chars = 5 width
        assert_no_overflow(output, 10);
    }

    #[test]
    fn test_no_overflow_narrow_terminal() {
        assert_no_overflow("abc", 3);
    }

    #[test]
    #[should_panic(expected = "Overflow by: 1")]
    fn test_no_overflow_narrow_terminal_overflow() {
        assert_no_overflow("abcd", 3);
    }

    // ========================================================================
    // assert_column_widths_within_terminal tests
    // ========================================================================

    #[test]
    fn test_column_widths_empty_output() {
        assert_column_widths_within_terminal("", 80);
    }

    #[test]
    fn test_column_widths_simple_table() {
        let table = "│ id │ name │\n│ 1  │ foo  │";
        assert_column_widths_within_terminal(table, 80);
    }

    #[test]
    fn test_column_widths_exact_fit() {
        // Build a table that exactly fits 20 chars:
        // │ + space + "xx" + space + │ + space + "yy" + space + │ = 1+1+2+1+1+1+2+1+1 = 11
        let table = "│ xx │ yy │";
        assert_column_widths_within_terminal(table, 11);
    }

    #[test]
    #[should_panic(expected = "OVERFLOW DETECTED")]
    fn test_column_widths_too_narrow() {
        let table = "│ id │ name │";
        assert_column_widths_within_terminal(table, 5); // Way too narrow
    }

    #[test]
    fn test_column_widths_non_table_output() {
        // Non-table output should pass (falls back to basic overflow check)
        let output = "This is not a table\nJust regular text";
        assert_column_widths_within_terminal(output, 80);
    }

    #[test]
    fn test_column_widths_with_borders() {
        let table = "╭──────┬──────╮\n│ col1 │ col2 │\n├──────┼──────┤\n│ val1 │ val2 │\n╰──────┴──────╯";
        assert_column_widths_within_terminal(table, 80);
    }

    // ========================================================================
    // assert_truncation_markers_present tests
    // ========================================================================

    #[test]
    fn test_truncation_empty_expectations() {
        // No expectations = always pass
        assert_truncation_markers_present("any output", &[]);
    }

    #[test]
    fn test_truncation_ellipsis_unicode() {
        let table = "│ id │ long_na… │\n│ 1  │ truncat… │";
        assert_truncation_markers_present(table, &[1]);
    }

    #[test]
    fn test_truncation_ellipsis_ascii() {
        let table = "│ id │ long_na... │\n│ 1  │ truncat... │";
        assert_truncation_markers_present(table, &[1]);
    }

    #[test]
    fn test_truncation_multiple_columns() {
        let table = "│ lo… │ lon… │ ok │\n│ tr… │ tru… │ ok │";
        assert_truncation_markers_present(table, &[0, 1]);
    }

    #[test]
    #[should_panic(expected = "TRUNCATION MARKER MISSING")]
    fn test_truncation_missing() {
        let table = "│ id │ name │\n│ 1  │ foo  │";
        assert_truncation_markers_present(table, &[1]); // No truncation in column 1
    }

    #[test]
    #[should_panic(expected = "Expected truncation in column: 2")]
    fn test_truncation_wrong_column() {
        let table = "│ id │ nam… │ desc │\n│ 1  │ fo…  │ ok   │";
        // Truncation is in column 1, not column 2
        assert_truncation_markers_present(table, &[2]);
    }

    // ========================================================================
    // Utility function tests
    // ========================================================================

    #[test]
    fn test_display_width_ascii() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width(""), 0);
        assert_eq!(display_width(" "), 1);
    }

    #[test]
    fn test_display_width_unicode() {
        // Ellipsis is 1 character wide
        assert_eq!(display_width("…"), 1);
        // Em dash is 1 character wide
        assert_eq!(display_width("—"), 1);
    }

    #[test]
    fn test_max_line_width_empty() {
        assert_eq!(max_line_width(""), 0);
    }

    #[test]
    fn test_max_line_width_single_line() {
        assert_eq!(max_line_width("hello world"), 11);
    }

    #[test]
    fn test_max_line_width_multi_line() {
        let output = "short\nvery long line here\nmedium";
        assert_eq!(max_line_width(output), 19);
    }

    #[test]
    fn test_line_count_empty() {
        assert_eq!(line_count(""), 0);
    }

    #[test]
    fn test_line_count_single() {
        assert_eq!(line_count("one line"), 1);
    }

    #[test]
    fn test_line_count_multiple() {
        assert_eq!(line_count("one\ntwo\nthree"), 3);
    }

    // ========================================================================
    // Internal function tests
    // ========================================================================

    #[test]
    fn test_parse_table_row_simple() {
        let row = "│ col1 │ col2 │";
        let cells = parse_table_row(row);
        assert_eq!(cells, vec!["col1", "col2"]);
    }

    #[test]
    fn test_parse_table_row_ascii_pipes() {
        let row = "| col1 | col2 |";
        let cells = parse_table_row(row);
        assert_eq!(cells, vec!["col1", "col2"]);
    }

    #[test]
    fn test_parse_table_row_empty() {
        assert!(parse_table_row("no pipes here").is_empty());
    }

    #[test]
    fn test_is_border_char() {
        assert!(is_border_char('─'));
        assert!(is_border_char('│'));
        assert!(is_border_char('┼'));
        assert!(is_border_char('-'));
        assert!(is_border_char('|'));
        assert!(!is_border_char('a'));
        assert!(!is_border_char(' '));
    }

    #[test]
    fn test_contains_truncation_marker_unicode() {
        assert!(contains_truncation_marker("hello…"));
        assert!(contains_truncation_marker("…"));
    }

    #[test]
    fn test_contains_truncation_marker_ascii() {
        assert!(contains_truncation_marker("hello..."));
        assert!(contains_truncation_marker("..."));
    }

    #[test]
    fn test_contains_truncation_marker_none() {
        assert!(!contains_truncation_marker("hello"));
        assert!(!contains_truncation_marker(".."));
        assert!(!contains_truncation_marker("."));
    }

    #[test]
    fn test_analyze_table_structure() {
        let table = "│ id │ name │\n│ 1  │ foo  │";
        let analysis = analyze_table_structure(table);
        assert!(analysis.is_some());
        let a = analysis.unwrap();
        assert_eq!(a.column_widths.len(), 2);
    }

    #[test]
    fn test_analyze_table_structure_no_table() {
        let text = "This is just text\nNo table here";
        let analysis = analyze_table_structure(text);
        assert!(analysis.is_none());
    }

    #[test]
    fn test_get_column_sample() {
        let table = "│ id │ name │\n│ 1  │ foo  │\n│ 2  │ bar  │";
        let sample = get_column_sample(table, 1);
        assert!(sample.is_some());
        let s = sample.unwrap();
        assert!(s.contains("name") || s.contains("foo") || s.contains("bar"));
    }

    #[test]
    fn test_get_column_sample_invalid_column() {
        let table = "│ id │ name │";
        let sample = get_column_sample(table, 10);
        assert!(sample.is_none());
    }
}
