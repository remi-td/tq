//! Terminal Simulator - Configurable Terminal Width Simulation
//!
//! Sprint 30: Provides configurable terminal width simulation for testing
//! table output formatting. This addresses the Sprint 29 gap where tests
//! couldn't validate output against specific terminal dimensions.
//!
//! # Overview
//!
//! The [`TerminalSimulator`] struct simulates a terminal with specified
//! dimensions, allowing tests to validate that output fits within specific
//! terminal constraints without requiring an actual terminal.
//!
//! # Design Philosophy
//!
//! This simulator is designed to be:
//! - **Simple**: Focused API for width/height validation
//! - **Flexible**: Supports various terminal dimensions for edge case testing
//! - **Integrated**: Works with visual_validator assertions
//! - **Zero-dependency**: Uses only standard library + unicode-width
//!
//! # Example
//!
//! ```ignore
//! use tests::tools::terminal_simulator::TerminalSimulator;
//!
//! // Simulate a narrow terminal (80x24)
//! let term = TerminalSimulator::new(80, 24);
//!
//! // Validate output fits
//! let output = render_table(&data);
//! term.validate_output(&output).expect("output should fit");
//!
//! // Check specific constraints
//! assert!(term.width_fits("short line"));
//! assert!(term.height_fits("line1\nline2\nline3"));
//! ```
//!
//! # Common Terminal Sizes
//!
//! The module provides constants for common terminal sizes:
//! - `TERMINAL_80X24` - Classic VT100 terminal (80x24)
//! - `TERMINAL_120X40` - Common modern default (120x40)
//! - `TERMINAL_NARROW` - Very narrow terminal for edge case testing (40x24)
//! - `TERMINAL_WIDE` - Wide terminal (200x50)

use unicode_width::UnicodeWidthStr;

// ============================================================================
// Common Terminal Size Constants
// ============================================================================

/// Classic VT100 terminal dimensions (80 columns x 24 rows)
pub const TERMINAL_80X24: (usize, usize) = (80, 24);

/// Common modern terminal default (120 columns x 40 rows)
pub const TERMINAL_120X40: (usize, usize) = (120, 40);

/// Narrow terminal for edge case testing (40 columns x 24 rows)
pub const TERMINAL_NARROW: (usize, usize) = (40, 24);

/// Wide terminal (200 columns x 50 rows)
pub const TERMINAL_WIDE: (usize, usize) = (200, 50);

/// User-reported terminal size from Sprint 29 issue (117 columns)
pub const TERMINAL_117: (usize, usize) = (117, 40);

// ============================================================================
// TerminalSimulator
// ============================================================================

/// Simulates a terminal with specified width and height.
///
/// This struct represents a virtual terminal with fixed dimensions,
/// allowing validation of output against specific terminal constraints.
///
/// # Example
///
/// ```ignore
/// let term = TerminalSimulator::new(80, 24);
/// assert_eq!(term.size(), (80, 24));
///
/// // Check if output fits
/// let result = term.validate_output("short line\nanother line");
/// assert!(result.is_ok());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSimulator {
    width: usize,
    height: usize,
}

impl TerminalSimulator {
    /// Create a new terminal simulator with specified dimensions.
    ///
    /// # Arguments
    ///
    /// * `width` - Terminal width in columns (characters)
    /// * `height` - Terminal height in rows (lines)
    ///
    /// # Panics
    ///
    /// Panics if width or height is 0 (invalid terminal dimensions).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let term = TerminalSimulator::new(80, 24);
    /// ```
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0, "Terminal width must be greater than 0");
        assert!(height > 0, "Terminal height must be greater than 0");
        Self { width, height }
    }

    /// Create a terminal simulator from a predefined size tuple.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use tests::tools::terminal_simulator::{TerminalSimulator, TERMINAL_80X24};
    ///
    /// let term = TerminalSimulator::from_tuple(TERMINAL_80X24);
    /// assert_eq!(term.width(), 80);
    /// ```
    pub fn from_tuple((width, height): (usize, usize)) -> Self {
        Self::new(width, height)
    }

    /// Returns the terminal dimensions as a tuple (width, height).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let term = TerminalSimulator::new(80, 24);
    /// let (w, h) = term.size();
    /// assert_eq!(w, 80);
    /// assert_eq!(h, 24);
    /// ```
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Returns the terminal width in columns.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the terminal height in rows.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Validates that output fits within terminal dimensions.
    ///
    /// This method checks both width (no line exceeds terminal width) and
    /// height (total lines fit within terminal height).
    ///
    /// # Arguments
    ///
    /// * `output` - The output string to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Output fits within terminal dimensions
    /// * `Err(String)` - Detailed error message explaining the violation
    ///
    /// # Example
    ///
    /// ```ignore
    /// let term = TerminalSimulator::new(80, 24);
    ///
    /// // Passes - fits within dimensions
    /// term.validate_output("short line").expect("should fit");
    ///
    /// // Fails - line too wide
    /// let wide_line = "x".repeat(100);
    /// let result = term.validate_output(&wide_line);
    /// assert!(result.is_err());
    /// ```
    pub fn validate_output(&self, output: &str) -> Result<(), String> {
        // Check width constraints
        for (line_idx, line) in output.lines().enumerate() {
            let line_width = UnicodeWidthStr::width(line);
            if line_width > self.width {
                return Err(format!(
                    "Line {} exceeds terminal width: {} > {} (content: '{}')",
                    line_idx + 1,
                    line_width,
                    self.width,
                    truncate_for_display(line, 50)
                ));
            }
        }

        // Check height constraints
        let line_count = if output.is_empty() {
            0
        } else {
            output.lines().count()
        };

        if line_count > self.height {
            return Err(format!(
                "Output exceeds terminal height: {} lines > {} rows",
                line_count, self.height
            ));
        }

        Ok(())
    }

    /// Validates only width constraints (ignores height).
    ///
    /// Useful for paged output where vertical scrolling is available
    /// but horizontal overflow is still problematic.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All lines fit within terminal width
    /// * `Err(String)` - Detailed error about width violation
    pub fn validate_width(&self, output: &str) -> Result<(), String> {
        for (line_idx, line) in output.lines().enumerate() {
            let line_width = UnicodeWidthStr::width(line);
            if line_width > self.width {
                return Err(format!(
                    "Line {} exceeds terminal width: {} > {} (content: '{}')",
                    line_idx + 1,
                    line_width,
                    self.width,
                    truncate_for_display(line, 50)
                ));
            }
        }
        Ok(())
    }

    /// Check if a single line fits within terminal width.
    ///
    /// # Arguments
    ///
    /// * `line` - A single line of text (should not contain newlines)
    ///
    /// # Returns
    ///
    /// `true` if the line's display width is <= terminal width
    ///
    /// # Example
    ///
    /// ```ignore
    /// let term = TerminalSimulator::new(80, 24);
    /// assert!(term.width_fits("short line"));
    /// assert!(!term.width_fits(&"x".repeat(100)));
    /// ```
    pub fn width_fits(&self, line: &str) -> bool {
        UnicodeWidthStr::width(line) <= self.width
    }

    /// Check if output fits within terminal height.
    ///
    /// # Arguments
    ///
    /// * `output` - Multi-line output string
    ///
    /// # Returns
    ///
    /// `true` if the number of lines is <= terminal height
    pub fn height_fits(&self, output: &str) -> bool {
        let line_count = if output.is_empty() {
            0
        } else {
            output.lines().count()
        };
        line_count <= self.height
    }

    /// Calculate how much an output would overflow the terminal width.
    ///
    /// Returns the maximum overflow amount (widest line - terminal width).
    /// Returns 0 if no overflow.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let term = TerminalSimulator::new(80, 24);
    /// let overflow = term.width_overflow("x".repeat(90).as_str());
    /// assert_eq!(overflow, 10);
    /// ```
    pub fn width_overflow(&self, output: &str) -> usize {
        output
            .lines()
            .map(|line| {
                let width = UnicodeWidthStr::width(line);
                width.saturating_sub(self.width)
            })
            .max()
            .unwrap_or(0)
    }

    /// Calculate how much an output would overflow the terminal height.
    ///
    /// Returns (line_count - terminal height) if overflow, 0 otherwise.
    pub fn height_overflow(&self, output: &str) -> usize {
        let line_count = if output.is_empty() {
            0
        } else {
            output.lines().count()
        };
        line_count.saturating_sub(self.height)
    }

    /// Get a detailed validation report for output.
    ///
    /// Returns a [`ValidationReport`] with comprehensive information about
    /// how the output fits (or doesn't fit) the terminal dimensions.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let term = TerminalSimulator::new(80, 24);
    /// let report = term.detailed_report("some output");
    /// println!("Max line width: {}", report.max_line_width);
    /// println!("Fits: {}", report.fits());
    /// ```
    pub fn detailed_report(&self, output: &str) -> ValidationReport {
        let line_count = if output.is_empty() {
            0
        } else {
            output.lines().count()
        };

        let max_line_width = output
            .lines()
            .map(UnicodeWidthStr::width)
            .max()
            .unwrap_or(0);

        let mut lines_over_width = Vec::new();
        for (idx, line) in output.lines().enumerate() {
            let width = UnicodeWidthStr::width(line);
            if width > self.width {
                lines_over_width.push((idx, width));
            }
        }

        ValidationReport {
            terminal_width: self.width,
            terminal_height: self.height,
            output_line_count: line_count,
            max_line_width,
            width_overflow: max_line_width.saturating_sub(self.width),
            height_overflow: line_count.saturating_sub(self.height),
            lines_over_width,
        }
    }
}

impl Default for TerminalSimulator {
    /// Default terminal is 120x40 (common modern default).
    fn default() -> Self {
        Self::from_tuple(TERMINAL_120X40)
    }
}

// ============================================================================
// ValidationReport
// ============================================================================

/// Detailed report of output validation against terminal dimensions.
///
/// Provides comprehensive information for debugging output that doesn't
/// fit terminal constraints.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Terminal width used for validation
    pub terminal_width: usize,
    /// Terminal height used for validation
    pub terminal_height: usize,
    /// Number of lines in the output
    pub output_line_count: usize,
    /// Width of the widest line
    pub max_line_width: usize,
    /// How many characters the widest line exceeds terminal width (0 if fits)
    pub width_overflow: usize,
    /// How many lines exceed terminal height (0 if fits)
    pub height_overflow: usize,
    /// Lines that exceed terminal width: (line_index, actual_width)
    pub lines_over_width: Vec<(usize, usize)>,
}

impl ValidationReport {
    /// Check if output fits within terminal dimensions.
    pub fn fits(&self) -> bool {
        self.width_overflow == 0 && self.height_overflow == 0
    }

    /// Check if output fits within terminal width (ignores height).
    pub fn width_fits(&self) -> bool {
        self.width_overflow == 0
    }

    /// Check if output fits within terminal height (ignores width).
    pub fn height_fits(&self) -> bool {
        self.height_overflow == 0
    }

    /// Generate a human-readable summary of the validation.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!(
            "Terminal: {}x{}",
            self.terminal_width, self.terminal_height
        ));

        parts.push(format!(
            "Output: {} lines, max width {}",
            self.output_line_count, self.max_line_width
        ));

        if self.width_overflow > 0 {
            parts.push(format!(
                "Width overflow: {} chars ({} lines too wide)",
                self.width_overflow,
                self.lines_over_width.len()
            ));
        }

        if self.height_overflow > 0 {
            parts.push(format!("Height overflow: {} lines", self.height_overflow));
        }

        if self.fits() {
            parts.push("Status: FITS".to_string());
        } else {
            parts.push("Status: OVERFLOW".to_string());
        }

        parts.join(" | ")
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Truncate a string for display in error messages.
fn truncate_for_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

/// Create a test table with specified dimensions for testing.
///
/// Generates a simple table with the given number of columns and rows,
/// where each cell contains predictable content.
///
/// # Arguments
///
/// * `cols` - Number of columns
/// * `rows` - Number of data rows (not including header)
/// * `cell_width` - Width of each cell's content
///
/// # Example
///
/// ```ignore
/// let table = create_test_table(3, 5, 10);
/// // Creates a 3-column, 5-row table with 10-char cells
/// ```
pub fn create_test_table(cols: usize, rows: usize, cell_width: usize) -> String {
    let mut lines = Vec::new();

    // Header
    let header: Vec<String> = (0..cols)
        .map(|i| format!("{:width$}", format!("col{}", i), width = cell_width))
        .collect();
    lines.push(format!("│ {} │", header.join(" │ ")));

    // Data rows
    for row in 0..rows {
        let cells: Vec<String> = (0..cols)
            .map(|col| format!("{:width$}", format!("r{}c{}", row, col), width = cell_width))
            .collect();
        lines.push(format!("│ {} │", cells.join(" │ ")));
    }

    lines.join("\n")
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // TerminalSimulator construction tests
    // ========================================================================

    #[test]
    fn test_new_terminal() {
        let term = TerminalSimulator::new(80, 24);
        assert_eq!(term.width(), 80);
        assert_eq!(term.height(), 24);
    }

    #[test]
    fn test_size_method() {
        let term = TerminalSimulator::new(120, 40);
        assert_eq!(term.size(), (120, 40));
    }

    #[test]
    fn test_from_tuple() {
        let term = TerminalSimulator::from_tuple(TERMINAL_80X24);
        assert_eq!(term.size(), (80, 24));
    }

    #[test]
    fn test_default() {
        let term = TerminalSimulator::default();
        assert_eq!(term.size(), TERMINAL_120X40);
    }

    #[test]
    #[should_panic(expected = "width must be greater than 0")]
    fn test_zero_width_panics() {
        TerminalSimulator::new(0, 24);
    }

    #[test]
    #[should_panic(expected = "height must be greater than 0")]
    fn test_zero_height_panics() {
        TerminalSimulator::new(80, 0);
    }

    // ========================================================================
    // Validation tests
    // ========================================================================

    #[test]
    fn test_validate_output_fits() {
        let term = TerminalSimulator::new(80, 24);
        let output = "short line\nanother line\nthird line";
        assert!(term.validate_output(output).is_ok());
    }

    #[test]
    fn test_validate_output_width_overflow() {
        let term = TerminalSimulator::new(80, 24);
        let wide_line = "x".repeat(100);
        let result = term.validate_output(&wide_line);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds terminal width"));
    }

    #[test]
    fn test_validate_output_height_overflow() {
        let term = TerminalSimulator::new(80, 5);
        let many_lines = "line\n".repeat(10);
        let result = term.validate_output(&many_lines);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds terminal height"));
    }

    #[test]
    fn test_validate_output_empty() {
        let term = TerminalSimulator::new(80, 24);
        assert!(term.validate_output("").is_ok());
    }

    #[test]
    fn test_validate_width_only() {
        let term = TerminalSimulator::new(80, 5);
        // Many lines but all short - width validation should pass
        let output = "short\n".repeat(100);
        assert!(term.validate_width(&output).is_ok());
    }

    // ========================================================================
    // width_fits and height_fits tests
    // ========================================================================

    #[test]
    fn test_width_fits_true() {
        let term = TerminalSimulator::new(80, 24);
        assert!(term.width_fits("short line"));
    }

    #[test]
    fn test_width_fits_exact() {
        let term = TerminalSimulator::new(10, 24);
        assert!(term.width_fits(&"x".repeat(10)));
    }

    #[test]
    fn test_width_fits_false() {
        let term = TerminalSimulator::new(10, 24);
        assert!(!term.width_fits(&"x".repeat(11)));
    }

    #[test]
    fn test_height_fits_true() {
        let term = TerminalSimulator::new(80, 10);
        assert!(term.height_fits("line1\nline2\nline3"));
    }

    #[test]
    fn test_height_fits_false() {
        let term = TerminalSimulator::new(80, 2);
        assert!(!term.height_fits("line1\nline2\nline3"));
    }

    #[test]
    fn test_height_fits_empty() {
        let term = TerminalSimulator::new(80, 24);
        assert!(term.height_fits(""));
    }

    // ========================================================================
    // Overflow calculation tests
    // ========================================================================

    #[test]
    fn test_width_overflow_none() {
        let term = TerminalSimulator::new(80, 24);
        assert_eq!(term.width_overflow("short line"), 0);
    }

    #[test]
    fn test_width_overflow_some() {
        let term = TerminalSimulator::new(80, 24);
        let line = "x".repeat(90);
        assert_eq!(term.width_overflow(&line), 10);
    }

    #[test]
    fn test_width_overflow_multi_line() {
        let term = TerminalSimulator::new(80, 24);
        let output = format!("{}\n{}\n{}", "x".repeat(85), "y".repeat(100), "z".repeat(90));
        // Maximum overflow is from the 100-char line
        assert_eq!(term.width_overflow(&output), 20);
    }

    #[test]
    fn test_height_overflow_none() {
        let term = TerminalSimulator::new(80, 10);
        assert_eq!(term.height_overflow("line1\nline2\nline3"), 0);
    }

    #[test]
    fn test_height_overflow_some() {
        let term = TerminalSimulator::new(80, 5);
        let output = "line\n".repeat(10);
        assert_eq!(term.height_overflow(&output), 5);
    }

    // ========================================================================
    // ValidationReport tests
    // ========================================================================

    #[test]
    fn test_detailed_report_fits() {
        let term = TerminalSimulator::new(80, 24);
        let output = "short\nlines\nhere";
        let report = term.detailed_report(output);

        assert!(report.fits());
        assert_eq!(report.output_line_count, 3);
        assert_eq!(report.width_overflow, 0);
        assert_eq!(report.height_overflow, 0);
        assert!(report.lines_over_width.is_empty());
    }

    #[test]
    fn test_detailed_report_width_overflow() {
        let term = TerminalSimulator::new(50, 24);
        let output = format!("{}\nshort\n{}", "x".repeat(60), "y".repeat(70));
        let report = term.detailed_report(&output);

        assert!(!report.fits());
        assert!(!report.width_fits());
        assert!(report.height_fits());
        assert_eq!(report.lines_over_width.len(), 2);
        // First line (index 0) is 60 chars, third line (index 2) is 70 chars
        assert!(report.lines_over_width.contains(&(0, 60)));
        assert!(report.lines_over_width.contains(&(2, 70)));
    }

    #[test]
    fn test_detailed_report_summary() {
        let term = TerminalSimulator::new(80, 24);
        let report = term.detailed_report("test output");
        let summary = report.summary();

        assert!(summary.contains("Terminal: 80x24"));
        assert!(summary.contains("Status: FITS"));
    }

    // ========================================================================
    // Test table generation tests
    // ========================================================================

    #[test]
    fn test_create_test_table_structure() {
        let table = create_test_table(3, 2, 5);
        let lines: Vec<&str> = table.lines().collect();

        // Should have 1 header + 2 data rows = 3 lines
        assert_eq!(lines.len(), 3);

        // Each line should have 3 columns
        for line in &lines {
            assert!(line.starts_with('│'));
            assert!(line.ends_with('│'));
        }
    }

    #[test]
    fn test_create_test_table_cell_content() {
        let table = create_test_table(2, 1, 8);
        // Should contain header and data cell markers
        assert!(table.contains("col0"));
        assert!(table.contains("col1"));
        assert!(table.contains("r0c0"));
        assert!(table.contains("r0c1"));
    }

    // ========================================================================
    // Constant tests
    // ========================================================================

    #[test]
    fn test_terminal_constants() {
        assert_eq!(TERMINAL_80X24, (80, 24));
        assert_eq!(TERMINAL_120X40, (120, 40));
        assert_eq!(TERMINAL_NARROW, (40, 24));
        assert_eq!(TERMINAL_WIDE, (200, 50));
        assert_eq!(TERMINAL_117, (117, 40));
    }

    // ========================================================================
    // Edge case tests
    // ========================================================================

    #[test]
    fn test_terminal_1x1() {
        let term = TerminalSimulator::new(1, 1);
        assert!(term.validate_output("x").is_ok());
        assert!(term.validate_output("xx").is_err());
        assert!(term.validate_output("x\nx").is_err());
    }

    #[test]
    fn test_unicode_width_handling() {
        let term = TerminalSimulator::new(10, 24);
        // ASCII characters
        assert!(term.width_fits(&"x".repeat(10)));
        // Ellipsis is 1 display width
        assert!(term.width_fits("hello wor…"));
    }

    #[test]
    fn test_clone_and_copy() {
        let term1 = TerminalSimulator::new(80, 24);
        let term2 = term1; // Copy
        #[allow(clippy::clone_on_copy)] // Testing clone trait implementation
        let term3 = term1.clone();

        assert_eq!(term1.size(), term2.size());
        assert_eq!(term1.size(), term3.size());
    }

    #[test]
    fn test_equality() {
        let term1 = TerminalSimulator::new(80, 24);
        let term2 = TerminalSimulator::new(80, 24);
        let term3 = TerminalSimulator::new(120, 40);

        assert_eq!(term1, term2);
        assert_ne!(term1, term3);
    }
}
