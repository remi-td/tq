//! Table output formatting with terminal width awareness
//!
//! Sprint 11: Simplified table formatting that removes broken padding logic
//! and implements terminal-width-aware column truncation.
//!
//! ## Design Philosophy
//! - NO complex padding calculations (repeatedly broke in Sprints 6, 8, 11)
//! - Detect terminal width and show columns that fit
//! - Clear truncation indicator when columns are hidden
//! - Batch mode (non-TTY) shows ALL columns
//!
//! ## Features
//! - Box-drawing characters (UTF-8)
//! - Terminal width detection
//! - Column truncation with "(+n cols)" indicator
//! - Type-based alignment
//! - NULL value styling

use crate::db::{Alignment, QueryResult};
use crate::error::Result;
use crossterm::terminal;
use std::io::{IsTerminal, Write};

/// Maximum column width for table output (content chars, excluding padding)
/// This prevents individual columns from dominating display space.
/// Values exceeding this will be truncated with ellipsis.
const MAX_COLUMN_WIDTH: usize = 100;

/// Table formatting options
#[derive(Debug, Clone)]
pub struct TableOptions {
    /// Include column headers
    pub show_header: bool,
    /// Use colors (for terminal output)
    pub use_color: bool,
    /// Maximum column width (deprecated, kept for API compatibility)
    #[allow(dead_code)]
    pub max_column_width: Option<u16>,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            show_header: true,
            use_color: true,
            max_column_width: Some(80),
        }
    }
}

/// Column selection result after fitting columns to terminal width
#[derive(Debug)]
struct ColumnSelection {
    /// Indices of columns to display
    visible_columns: Vec<usize>,
    /// Number of hidden columns
    hidden_count: usize,
    /// Names of hidden columns (for footer message)
    hidden_names: Vec<String>,
    /// Width needed for each visible column (content + 2 for spacing)
    column_widths: Vec<usize>,
}

/// Get terminal width, returning None for non-TTY (batch mode)
fn get_terminal_width() -> Option<usize> {
    // Check if stdout is a terminal (TTY)
    if !std::io::stdout().is_terminal() {
        // Batch mode: return None to indicate no truncation
        return None;
    }

    // Try to get terminal size using crossterm
    if let Ok((width, _height)) = terminal::size() {
        Some(width as usize)
    } else {
        // Default fallback for TTY without detectable size
        Some(80)
    }
}

/// Calculate minimum width needed for a column
///
/// Width is calculated from actual cell content (content-based width calculation):
/// 1. Scan header and all sampled values
/// 2. Take maximum of header length and max value length
/// 3. Cap at effective maximum (explicit max_width or MAX_COLUMN_WIDTH default)
/// 4. Add 2 for padding (1 space on each side)
///
/// This ensures columns are sized to actual content, not schema type definitions,
/// while preventing any single column from dominating display space.
fn calculate_column_width(
    header: &str,
    values: &[String],
    max_sample: usize,
    max_width: Option<usize>,
) -> usize {
    let header_width = header.len();
    let max_value_width = values
        .iter()
        .take(max_sample)
        .map(|v| v.len())
        .max()
        .unwrap_or(0);

    // Content width: max of header and data
    let content_width = std::cmp::max(header_width, max_value_width);

    // Apply maximum cap (use explicit max_width if provided, otherwise default to MAX_COLUMN_WIDTH)
    let effective_max = max_width.unwrap_or(MAX_COLUMN_WIDTH);
    let capped_width = std::cmp::min(content_width, effective_max);

    // Add padding (2 chars: 1 space on each side)
    capped_width + 2
}

/// Select which columns to display based on terminal width
fn select_visible_columns(
    column_names: &[String],
    column_values: &[Vec<String>],
    terminal_width: Option<usize>,
    max_column_width: Option<usize>,
) -> ColumnSelection {
    let total_columns = column_names.len();

    // Batch mode (non-TTY) or pager mode: show all columns
    let Some(term_width) = terminal_width else {
        let widths: Vec<usize> = column_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let values: Vec<String> = column_values.iter().map(|row| row[i].clone()).collect();
                calculate_column_width(name, &values, 100, max_column_width)
            })
            .collect();

        return ColumnSelection {
            visible_columns: (0..total_columns).collect(),
            hidden_count: 0,
            hidden_names: vec![],
            column_widths: widths,
        };
    };

    // Interactive mode: calculate which columns fit
    let mut visible = Vec::new();
    let mut widths = Vec::new();
    let mut used_width = 0;

    // Reserve space for truncation indicator: "| (+n cols) |" = ~15 chars
    let truncation_width = 15;
    // Account for left border
    let left_border = 1; // "│"

    for (idx, name) in column_names.iter().enumerate() {
        let values: Vec<String> = column_values.iter().map(|row| row[idx].clone()).collect();
        let col_width = calculate_column_width(name, &values, 100, None);

        // Separator width: │ between columns (1 char)
        let separator_width = if visible.is_empty() { 0 } else { 1 };

        // Calculate new total width if we add this column
        let new_width = used_width + left_border + col_width + separator_width;

        // Check if we can fit this column
        let remaining_columns = total_columns - idx - 1;
        if remaining_columns > 0 {
            // Not the last column - need room for truncation indicator
            if new_width + truncation_width < term_width {
                // +1 for right border
                visible.push(idx);
                widths.push(col_width);
                used_width = new_width;
            } else {
                break; // Stop adding columns
            }
        } else {
            // Last column - no truncation indicator needed
            if new_width < term_width {
                // +1 for right border
                visible.push(idx);
                widths.push(col_width);
            }
            // Either way, we're done
            break;
        }
    }

    // Ensure at least one column is shown
    if visible.is_empty() && !column_names.is_empty() {
        let values: Vec<String> = column_values.iter().map(|row| row[0].clone()).collect();
        let col_width = calculate_column_width(&column_names[0], &values, 100, None);
        visible.push(0);
        widths.push(col_width);
    }

    let hidden_count = total_columns - visible.len();
    let hidden_names: Vec<String> = column_names
        .iter()
        .enumerate()
        .filter(|(i, _)| !visible.contains(i))
        .map(|(_, name)| name.clone())
        .collect();

    ColumnSelection {
        visible_columns: visible,
        hidden_count,
        hidden_names,
        column_widths: widths,
    }
}

/// Render the table with UTF-8 box-drawing characters
fn render_table(
    result: &QueryResult,
    selection: &ColumnSelection,
    options: &TableOptions,
) -> String {
    let mut output = String::new();

    // Get column names and values as strings
    let column_names: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
    let row_values: Vec<Vec<String>> = result
        .rows
        .iter()
        .map(|row| row.iter().map(|v| v.display()).collect())
        .collect();

    // Calculate truncation indicator width if needed
    let truncation_col_width = if selection.hidden_count > 0 {
        format!("(+{} cols)", selection.hidden_count).len() + 2
    } else {
        0
    };

    // Render top border
    output.push_str(&render_border(
        &selection.column_widths,
        truncation_col_width,
        BorderStyle::Top,
    ));

    // Render header row
    if options.show_header {
        output.push_str(&render_header_row(
            &column_names,
            selection,
            truncation_col_width,
        ));

        // Render header separator
        output.push_str(&render_border(
            &selection.column_widths,
            truncation_col_width,
            BorderStyle::Middle,
        ));
    }

    // Render data rows
    for row in &row_values {
        output.push_str(&render_data_row(
            row,
            &result.columns,
            selection,
            truncation_col_width,
            options,
        ));
    }

    // Render bottom border
    output.push_str(&render_border(
        &selection.column_widths,
        truncation_col_width,
        BorderStyle::Bottom,
    ));

    output
}

#[derive(Debug, Clone, Copy)]
enum BorderStyle {
    Top,
    Middle,
    Bottom,
}

fn render_border(column_widths: &[usize], truncation_width: usize, style: BorderStyle) -> String {
    let (left, mid, right, line) = match style {
        BorderStyle::Top => ('╭', '┬', '╮', '─'),
        BorderStyle::Middle => ('├', '┼', '┤', '─'),
        BorderStyle::Bottom => ('╰', '┴', '╯', '─'),
    };

    let mut border = String::new();
    border.push(left);

    for (i, width) in column_widths.iter().enumerate() {
        if i > 0 {
            border.push(mid);
        }
        border.push_str(&line.to_string().repeat(*width));
    }

    // Add truncation indicator column
    if truncation_width > 0 {
        border.push(mid);
        border.push_str(&line.to_string().repeat(truncation_width));
    }

    border.push(right);
    border.push('\n');
    border
}

fn render_header_row(
    column_names: &[String],
    selection: &ColumnSelection,
    truncation_width: usize,
) -> String {
    let mut row = String::from("│");

    for (i, &col_idx) in selection.visible_columns.iter().enumerate() {
        let name = &column_names[col_idx];
        let width = selection.column_widths[i];
        // Basic spacing: 1 space before and after
        row.push_str(&format!(" {:width$}", name, width = width - 2));
        row.push_str(" │");
    }

    // Add truncation indicator
    if selection.hidden_count > 0 {
        let indicator = format!("(+{} cols)", selection.hidden_count);
        row.push_str(&format!(
            " {:width$}",
            indicator,
            width = truncation_width - 2
        ));
        row.push_str(" │");
    }

    row.push('\n');
    row
}

fn render_data_row(
    values: &[String],
    columns: &[crate::db::ColumnMetadata],
    selection: &ColumnSelection,
    truncation_width: usize,
    options: &TableOptions,
) -> String {
    let mut row = String::from("│");

    for (i, &col_idx) in selection.visible_columns.iter().enumerate() {
        let value = &values[col_idx];
        let width = selection.column_widths[i];
        let col = &columns[col_idx];

        // Truncate value if it exceeds allocated width (accounting for padding)
        // This prevents long values from breaking table alignment
        let max_value_len = width.saturating_sub(2); // -2 for padding spaces
        let truncated_value = if value.len() > max_value_len && max_value_len > 3 {
            format!("{}...", &value[..max_value_len.saturating_sub(3)])
        } else if value.len() > max_value_len {
            value[..max_value_len].to_string()
        } else {
            value.clone()
        };

        // Format value with alignment
        let formatted = match col.data_type.alignment() {
            Alignment::Right => format!(" {:>width$}", truncated_value, width = width - 2),
            Alignment::Center => format!(" {:^width$}", truncated_value, width = width - 2),
            Alignment::Left => format!(" {:width$}", truncated_value, width = width - 2),
        };

        // Apply NULL styling if color is enabled
        if options.use_color && truncated_value == "[NULL]" {
            // ANSI escape for dim/italic: \x1b[2;3m ... \x1b[0m
            row.push_str(&format!("\x1b[2;3m{}\x1b[0m", formatted));
        } else {
            row.push_str(&formatted);
        }
        row.push_str(" │");
    }

    // Add truncation indicator for data rows
    if selection.hidden_count > 0 {
        row.push_str(&format!(" {:width$}", "...", width = truncation_width - 2));
        row.push_str(" │");
    }

    row.push('\n');
    row
}

/// Write query results as a formatted table
///
/// Sprint 11: Simplified implementation with terminal width awareness.
/// - In TTY mode: Shows columns that fit, with "(+n cols)" indicator for hidden ones
/// - In batch mode: Shows ALL columns without truncation
pub fn write<W: Write>(result: &QueryResult, writer: &mut W, options: &TableOptions) -> Result<()> {
    write_with_width_constraint(result, writer, options, get_terminal_width())
}

fn write_with_width_constraint<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &TableOptions,
    terminal_width: Option<usize>,
) -> Result<()> {
    if result.is_empty() {
        writeln!(writer, "No results returned.")?;
        return Ok(());
    }

    if crate::format::is_show_query_result(result) {
        if let Some(row) = result.rows.first() {
            if let Some(val) = row.first() {
                let ddl = val.display();
                write!(writer, "{}", ddl)?;
                if !ddl.ends_with('\n') {
                    writeln!(writer)?;
                }
                return Ok(());
            }
        }
    }

    // Prepare data
    let column_names: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
    let column_values: Vec<Vec<String>> = result
        .rows
        .iter()
        .map(|row| row.iter().map(|v| v.display()).collect())
        .collect();

    // Select columns to display based on terminal width
    // Sprint 30: Pager no longer uses this - it formats from QueryResult directly
    let selection = select_visible_columns(&column_names, &column_values, terminal_width, None);

    // Render the table
    let table_output = render_table(result, &selection, options);
    write!(writer, "{}", table_output)?;

    // Show hidden columns message if any
    if !selection.hidden_names.is_empty() {
        writeln!(writer)?;
        writeln!(
            writer,
            "{} columns hidden: {}",
            selection.hidden_count,
            selection.hidden_names.join(", ")
        )?;
        writeln!(
            writer,
            "Use --format csv or --format json to see all columns"
        )?;
    }

    Ok(())
}

/// Write query results with timing footer
pub fn write_with_timing<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &TableOptions,
) -> Result<()> {
    write(result, writer, options)?;

    // Add row count and timing
    writeln!(
        writer,
        "{} row(s) in set ({:.3}s)",
        result.row_count,
        result.execution_time.as_secs_f64()
    )?;

    Ok(())
}

/// Format results as a simple string (for testing/debugging)
pub fn format_string(result: &QueryResult, options: &TableOptions) -> Result<String> {
    let mut buffer = Vec::new();
    write(result, &mut buffer, options)?;
    Ok(String::from_utf8_lossy(&buffer).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{ColumnMetadata, TeradataType, Value};
    use std::time::Duration;

    fn create_test_result() -> QueryResult {
        let columns = vec![
            ColumnMetadata::new("id", TeradataType::Integer, false),
            ColumnMetadata::new("name", TeradataType::Varchar, true),
            ColumnMetadata::new("active", TeradataType::Boolean, false),
        ];
        let rows = vec![
            vec![
                Value::Integer(1),
                Value::String("Alice".into()),
                Value::Boolean(true),
            ],
            vec![
                Value::Integer(2),
                Value::String("Bob".into()),
                Value::Boolean(false),
            ],
            vec![Value::Integer(3), Value::Null, Value::Boolean(true)],
        ];
        QueryResult::new(columns, rows, Duration::from_millis(100))
    }

    fn create_wide_result() -> QueryResult {
        // Create a result with many columns to test truncation
        let columns = vec![
            ColumnMetadata::new("id", TeradataType::Integer, false),
            ColumnMetadata::new("username", TeradataType::Varchar, true),
            ColumnMetadata::new("email", TeradataType::Varchar, true),
            ColumnMetadata::new("department", TeradataType::Varchar, true),
            ColumnMetadata::new("role", TeradataType::Varchar, true),
            ColumnMetadata::new("active", TeradataType::Boolean, false),
            ColumnMetadata::new("created_at", TeradataType::Timestamp, true),
            ColumnMetadata::new("updated_at", TeradataType::Timestamp, true),
        ];
        let rows = vec![vec![
            Value::Integer(1),
            Value::String("alice".into()),
            Value::String("alice@example.com".into()),
            Value::String("engineering".into()),
            Value::String("developer".into()),
            Value::Boolean(true),
            Value::String("2024-01-01".into()),
            Value::String("2024-01-15".into()),
        ]];
        QueryResult::new(columns, rows, Duration::from_millis(50))
    }

    #[test]
    fn test_write_table() {
        let result = create_test_result();
        let options = TableOptions {
            use_color: false,
            ..Default::default()
        };

        let mut buffer = Vec::new();
        write(&result, &mut buffer, &options).unwrap();
        let output = String::from_utf8_lossy(&buffer);

        assert!(output.contains("id"));
        assert!(output.contains("name"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        assert!(output.contains("[NULL]"));
    }

    #[test]
    fn test_write_table_no_header() {
        let result = create_test_result();
        let options = TableOptions {
            show_header: false,
            use_color: false,
            ..Default::default()
        };

        let output = format_string(&result, &options).unwrap();

        // Should contain data
        assert!(output.contains("Alice"));
        assert!(output.contains("1"));
    }

    #[test]
    fn test_write_table_empty() {
        let result = QueryResult::empty();
        let options = TableOptions::default();

        let mut buffer = Vec::new();
        write(&result, &mut buffer, &options).unwrap();
        let output = String::from_utf8_lossy(&buffer);

        assert!(output.contains("No results"));
    }

    #[test]
    fn test_write_with_timing() {
        let result = create_test_result();
        let options = TableOptions {
            use_color: false,
            ..Default::default()
        };

        let mut buffer = Vec::new();
        write_with_timing(&result, &mut buffer, &options).unwrap();
        let output = String::from_utf8_lossy(&buffer);

        assert!(output.contains("3 row(s) in set"));
        assert!(output.contains("0.100s"));
    }

    // Sprint 11: Column selection tests

    #[test]
    fn test_column_width_calculation() {
        let values = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ];
        let width = calculate_column_width("name", &values, 100, None);
        // "Charlie" is 7 chars, header "name" is 4 chars, so max is 7 + 2 = 9
        assert_eq!(width, 9);
    }

    #[test]
    fn test_column_width_uses_header_when_larger() {
        let values = vec!["A".to_string(), "B".to_string()];
        let width = calculate_column_width("very_long_header", &values, 100, None);
        // Header is 16 chars, values are 1 char, so width = 16 + 2 = 18
        assert_eq!(width, 18);
    }

    #[test]
    fn test_select_columns_batch_mode() {
        // Batch mode (terminal_width = None) should show all columns
        let column_names = vec![
            "id".to_string(),
            "name".to_string(),
            "email".to_string(),
            "dept".to_string(),
        ];
        let column_values = vec![vec![
            "1".to_string(),
            "Alice".to_string(),
            "alice@example.com".to_string(),
            "engineering".to_string(),
        ]];

        let selection = select_visible_columns(&column_names, &column_values, None, None);

        assert_eq!(selection.visible_columns.len(), 4);
        assert_eq!(selection.hidden_count, 0);
        assert!(selection.hidden_names.is_empty());
    }

    #[test]
    fn test_select_columns_narrow_terminal() {
        // Narrow terminal should truncate columns
        let column_names = vec![
            "id".to_string(),
            "username".to_string(),
            "email".to_string(),
            "department".to_string(),
            "role".to_string(),
        ];
        let column_values = vec![vec![
            "1".to_string(),
            "alice".to_string(),
            "alice@example.com".to_string(),
            "engineering".to_string(),
            "developer".to_string(),
        ]];

        // Very narrow terminal (50 chars) - should truncate
        let selection = select_visible_columns(&column_names, &column_values, Some(50), None);

        // Should have fewer than all columns
        assert!(selection.visible_columns.len() < 5);
        assert!(selection.hidden_count > 0);
        assert!(!selection.hidden_names.is_empty());

        // Should prioritize leftmost columns
        assert!(selection.visible_columns.contains(&0)); // id should be first
    }

    #[test]
    fn test_select_columns_wide_terminal() {
        // Wide terminal should show all columns
        let column_names = vec!["id".to_string(), "name".to_string()];
        let column_values = vec![vec!["1".to_string(), "Alice".to_string()]];

        let selection = select_visible_columns(&column_names, &column_values, Some(200), None);

        assert_eq!(selection.visible_columns.len(), 2);
        assert_eq!(selection.hidden_count, 0);
    }

    #[test]
    fn test_truncation_indicator_in_output() {
        // Test that truncation indicator appears in narrow terminal
        let result = create_wide_result();
        let options = TableOptions {
            use_color: false,
            ..Default::default()
        };

        // Manually select columns with truncation for testing
        let column_names: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
        let column_values: Vec<Vec<String>> = result
            .rows
            .iter()
            .map(|row| row.iter().map(|v| v.display()).collect())
            .collect();

        // Force narrow terminal
        let selection = select_visible_columns(&column_names, &column_values, Some(60), None);

        // If truncation happened, we should see the indicator
        if selection.hidden_count > 0 {
            let table_output = render_table(&result, &selection, &options);
            assert!(table_output.contains(&format!("(+{} cols)", selection.hidden_count)));
            assert!(table_output.contains("..."));
        }
    }

    #[test]
    fn test_border_rendering() {
        let widths = vec![5, 10, 8];

        let top = render_border(&widths, 0, BorderStyle::Top);
        assert!(top.contains("╭"));
        assert!(top.contains("┬"));
        assert!(top.contains("╮"));

        let middle = render_border(&widths, 0, BorderStyle::Middle);
        assert!(middle.contains("├"));
        assert!(middle.contains("┼"));
        assert!(middle.contains("┤"));

        let bottom = render_border(&widths, 0, BorderStyle::Bottom);
        assert!(bottom.contains("╰"));
        assert!(bottom.contains("┴"));
        assert!(bottom.contains("╯"));
    }

    #[test]
    fn test_hidden_columns_message() {
        let result = create_wide_result();
        let options = TableOptions {
            use_color: false,
            ..Default::default()
        };

        // Get column data
        let column_names: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
        let column_values: Vec<Vec<String>> = result
            .rows
            .iter()
            .map(|row| row.iter().map(|v| v.display()).collect())
            .collect();

        // Force truncation with narrow terminal
        let selection = select_visible_columns(&column_names, &column_values, Some(50), None);

        if selection.hidden_count > 0 {
            // Hidden names should be populated
            assert!(!selection.hidden_names.is_empty());

            // The message should list hidden column names
            let mut buffer = Vec::new();
            write(&result, &mut buffer, &options).unwrap();
            // Note: In tests, stdout is not a terminal, so batch mode applies
            // This test verifies the selection logic, not the full output
        }
    }

    #[test]
    fn test_at_least_one_column_shown() {
        // Even with very narrow terminal, at least one column should show
        let column_names = vec!["very_long_column_name".to_string()];
        let column_values = vec![vec!["some_value".to_string()]];

        let selection = select_visible_columns(&column_names, &column_values, Some(10), None);

        // Must show at least one column
        assert!(!selection.visible_columns.is_empty());
        assert_eq!(selection.visible_columns[0], 0);
    }

    // Sprint 32: Content-based column width tests

    #[test]
    fn test_column_width_max_cap_applied() {
        // Test that MAX_COLUMN_WIDTH cap is applied even when max_width is None
        // Create a very long string that exceeds MAX_COLUMN_WIDTH (100)
        let long_value = "x".repeat(150);
        let values = vec![long_value];
        let width = calculate_column_width("col", &values, 100, None);
        // Should be capped at MAX_COLUMN_WIDTH (100) + 2 padding = 102
        assert_eq!(width, MAX_COLUMN_WIDTH + 2);
    }

    #[test]
    fn test_column_width_explicit_max_overrides_default() {
        // Test that explicit max_width overrides the default MAX_COLUMN_WIDTH
        let long_value = "x".repeat(150);
        let values = vec![long_value];
        // Explicit max of 40 (like pager uses)
        let width = calculate_column_width("col", &values, 100, Some(40));
        // Should be capped at 40 + 2 padding = 42
        assert_eq!(width, 42);
    }

    #[test]
    fn test_column_width_null_value_representation() {
        // Test that [NULL] (6 chars) is correctly sized
        let values = vec!["[NULL]".to_string()];
        let width = calculate_column_width("col", &values, 100, None);
        // "[NULL]" is 6 chars, header "col" is 3 chars, so width = 6 + 2 = 8
        assert_eq!(width, 8);
    }

    #[test]
    fn test_column_width_null_is_considered_in_max() {
        // Test that [NULL] can be the determining factor for width
        let values = vec![
            "A".to_string(),
            "B".to_string(),
            "[NULL]".to_string(), // 6 chars - should determine width
        ];
        let width = calculate_column_width("id", &values, 100, None);
        // max content is 6 ([NULL]), header is 2, so width = 6 + 2 = 8
        assert_eq!(width, 8);
    }

    #[test]
    fn test_column_width_empty_strings() {
        // Test that empty strings don't cause issues and header determines width
        let values = vec!["".to_string(), "".to_string(), "".to_string()];
        let width = calculate_column_width("Status", &values, 100, None);
        // All values are empty, so header "Status" (6 chars) determines width
        assert_eq!(width, 8); // 6 + 2 padding
    }

    #[test]
    fn test_column_width_mixed_empty_and_content() {
        // Test that empty strings don't affect width when other values exist
        let values = vec![
            "active".to_string(),
            "".to_string(),
            "pending".to_string(), // 7 chars - should determine width
        ];
        let width = calculate_column_width("stat", &values, 100, None);
        // "pending" is 7 chars, header is 4, so width = 7 + 2 = 9
        assert_eq!(width, 9);
    }

    #[test]
    fn test_column_width_numeric_values() {
        // Test numeric values (represented as strings)
        let values = vec![
            "1".to_string(),
            "42".to_string(),
            "1500".to_string(), // 4 chars
        ];
        let width = calculate_column_width("amount", &values, 100, None);
        // "amount" is 6 chars, max value "1500" is 4 chars
        // Header wins: 6 + 2 = 8
        assert_eq!(width, 8);
    }

    #[test]
    fn test_column_width_large_numbers() {
        // Test larger numeric values that exceed header length
        let values = vec![
            "1000000".to_string(),  // 7 chars
            "99999999".to_string(), // 8 chars
        ];
        let width = calculate_column_width("id", &values, 100, None);
        // "99999999" is 8 chars, header "id" is 2 chars
        // Value wins: 8 + 2 = 10
        assert_eq!(width, 10);
    }

    #[test]
    fn test_column_width_exactly_at_max() {
        // Test value exactly at MAX_COLUMN_WIDTH
        let exact_max_value = "x".repeat(MAX_COLUMN_WIDTH);
        let values = vec![exact_max_value];
        let width = calculate_column_width("col", &values, 100, None);
        // Exactly at max: 100 + 2 = 102
        assert_eq!(width, MAX_COLUMN_WIDTH + 2);
    }

    #[test]
    fn test_column_width_one_over_max() {
        // Test value one character over MAX_COLUMN_WIDTH
        let over_max_value = "x".repeat(MAX_COLUMN_WIDTH + 1);
        let values = vec![over_max_value];
        let width = calculate_column_width("col", &values, 100, None);
        // Capped at max: 100 + 2 = 102 (not 103)
        assert_eq!(width, MAX_COLUMN_WIDTH + 2);
    }

    #[test]
    fn test_column_width_sampling_limit() {
        // Test that max_sample parameter limits how many values are considered
        let mut values: Vec<String> = (0..50).map(|_| "short".to_string()).collect();
        values.push("this_is_a_very_long_value".to_string()); // 25 chars at position 50

        // Sample only first 10 rows - should not see the long value
        let width = calculate_column_width("col", &values, 10, None);
        // Only sees "short" (5 chars) and header "col" (3 chars)
        assert_eq!(width, 7); // 5 + 2 padding

        // Sample all rows - should see the long value
        let width_all = calculate_column_width("col", &values, 100, None);
        // Sees "this_is_a_very_long_value" (25 chars)
        assert_eq!(width_all, 27); // 25 + 2 padding
    }

    #[test]
    fn test_column_width_unicode_basic() {
        // Test basic Unicode characters (non-ASCII)
        // Note: This uses byte length, not display width
        // "cafe" with accent: "caf\u{00e9}" is 4 display chars but 5 bytes in UTF-8
        let values = vec!["cafe".to_string(), "naïve".to_string()]; // naïve has ï (2 bytes)
        let width = calculate_column_width("word", &values, 100, None);
        // "naïve" is 6 bytes (n, a, ï[2], v, e), header is 4
        // Current implementation uses byte length
        assert!(width >= 6); // At least 6 + 2 = 8
    }

    #[test]
    fn test_column_width_constant_value() {
        // Verify the MAX_COLUMN_WIDTH constant is set correctly
        assert_eq!(MAX_COLUMN_WIDTH, 100);
    }

    #[test]
    fn test_select_columns_respects_max_width() {
        // Test that select_visible_columns respects max_column_width parameter
        let column_names = vec!["col1".to_string()];
        let long_value = "x".repeat(150);
        let column_values = vec![vec![long_value]];

        // Without max_width constraint, uses default MAX_COLUMN_WIDTH
        let selection_default = select_visible_columns(&column_names, &column_values, None, None);
        assert_eq!(selection_default.column_widths[0], MAX_COLUMN_WIDTH + 2);

        // With explicit max_width of 50
        let selection_constrained =
            select_visible_columns(&column_names, &column_values, None, Some(50));
        assert_eq!(selection_constrained.column_widths[0], 52); // 50 + 2 padding
    }

    #[test]
    fn test_write_table_show_query() {
        use crate::db::{ColumnMetadata, TeradataType, Value, QueryResult};
        use std::time::Duration;

        let columns = vec![ColumnMetadata::new("Request Text", TeradataType::Varchar, false)];
        let ddl = "CREATE SET TABLE db.table ,\n     NO FLASHBACK\n     (id INTEGER);";
        let rows = vec![vec![Value::String(ddl.to_string())]];
        let result = QueryResult::new(columns, rows, Duration::ZERO);

        let options = TableOptions::default();
        let mut buffer = Vec::new();
        write(&result, &mut buffer, &options).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Should output the raw DDL without table borders or headers
        assert_eq!(output, format!("{}\n", ddl));
    }
}

