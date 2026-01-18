//! SQL statement parsing for batch mode execution
//!
//! This module provides simple semicolon-based statement splitting for batch execution.
//! It follows the MVP approach documented in the Sprint 10 architecture.
//!
//! # Design Decisions
//!
//! - **Simple splitting**: Split on `;` without full SQL grammar parsing
//! - **Comments preserved**: Pass through to Teradata (handles them correctly)
//! - **Line tracking**: Track line numbers for error messages
//! - **Empty handling**: Skip whitespace-only statements
//!
//! # Known Limitations
//!
//! - Semicolons inside string literals may cause incorrect splits (rare in practice)
//! - Full SQL parsing deferred to future sprints if needed

/// A parsed SQL statement with metadata for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStatement {
    /// The SQL statement text (trimmed)
    pub sql: String,
    /// 1-based statement number for user-facing messages
    pub statement_number: usize,
    /// Line number where statement starts (1-based)
    pub start_line: usize,
}

impl ParsedStatement {
    /// Create a new ParsedStatement
    pub fn new(sql: String, statement_number: usize, start_line: usize) -> Self {
        Self {
            sql,
            statement_number,
            start_line,
        }
    }

    /// Get a preview of the statement for error messages (truncated to max length)
    pub fn preview(&self, max_len: usize) -> String {
        let normalized: String = self.sql.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.len() <= max_len {
            normalized
        } else {
            format!("{}...", &normalized[..max_len.saturating_sub(3)])
        }
    }
}

/// Parse SQL text into individual statements
///
/// Splits the input on semicolons and returns a vector of `ParsedStatement`
/// structs with statement numbering and line tracking.
///
/// # Arguments
/// * `sql` - The SQL text to parse (may contain multiple statements)
///
/// # Returns
/// A vector of parsed statements. Empty statements (whitespace-only) are skipped.
///
/// # Example
/// ```
/// use tq::sql::parser::parse_statements;
///
/// let sql = "SELECT 1;\nSELECT 2;\n\n-- Comment\nSELECT 3;";
/// let statements = parse_statements(sql);
///
/// assert_eq!(statements.len(), 3);
/// assert_eq!(statements[0].sql, "SELECT 1");
/// assert_eq!(statements[0].statement_number, 1);
/// assert_eq!(statements[0].start_line, 1);
/// ```
pub fn parse_statements(sql: &str) -> Vec<ParsedStatement> {
    let mut statements = Vec::new();
    let mut statement_number = 0;

    // Track current byte position in input
    let mut byte_offset = 0;

    // Split on semicolons
    for segment in sql.split(';') {
        let trimmed = segment.trim();

        if !trimmed.is_empty() {
            // Calculate line number based on newlines before this segment's first non-whitespace char
            // Count newlines from start to current byte position plus leading whitespace
            let segment_start = sql[..byte_offset].len();
            let segment_with_leading = &sql[segment_start..segment_start + segment.len()];

            // Find where content starts (skip leading whitespace)
            let content_offset = segment_with_leading
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(0);

            // Count newlines from start of input to start of content
            let start_line = 1 + sql[..segment_start + content_offset]
                .chars()
                .filter(|&c| c == '\n')
                .count();

            statement_number += 1;
            statements.push(ParsedStatement::new(
                trimmed.to_string(),
                statement_number,
                start_line,
            ));
        }

        // Move byte offset past this segment and the semicolon
        byte_offset += segment.len() + 1; // +1 for the semicolon
    }

    statements
}

/// Check if a SQL string contains multiple statements
///
/// This is useful for determining whether to use single-statement or batch execution.
///
/// # Arguments
/// * `sql` - The SQL text to check
///
/// # Returns
/// `true` if the SQL contains more than one statement
pub fn has_multiple_statements(sql: &str) -> bool {
    parse_statements(sql).len() > 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_statement() {
        let statements = parse_statements("SELECT 1");
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[0].statement_number, 1);
        assert_eq!(statements[0].start_line, 1);
    }

    #[test]
    fn test_parse_single_statement_with_semicolon() {
        let statements = parse_statements("SELECT 1;");
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 1");
    }

    #[test]
    fn test_parse_multiple_statements() {
        let statements = parse_statements("SELECT 1; SELECT 2; SELECT 3;");
        assert_eq!(statements.len(), 3);
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[1].sql, "SELECT 2");
        assert_eq!(statements[2].sql, "SELECT 3");

        // Statement numbering
        assert_eq!(statements[0].statement_number, 1);
        assert_eq!(statements[1].statement_number, 2);
        assert_eq!(statements[2].statement_number, 3);
    }

    #[test]
    fn test_parse_multiline_statements() {
        let sql = "SELECT 1;\nSELECT 2;\n\nSELECT 3;";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 3);

        // Line tracking
        assert_eq!(statements[0].start_line, 1);
        assert_eq!(statements[1].start_line, 2);
        assert_eq!(statements[2].start_line, 4); // Skips blank line
    }

    #[test]
    fn test_parse_empty_input() {
        let statements = parse_statements("");
        assert!(statements.is_empty());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let statements = parse_statements("   \n\n   ");
        assert!(statements.is_empty());
    }

    #[test]
    fn test_parse_empty_statements_skipped() {
        let statements = parse_statements("SELECT 1;;; SELECT 2;");
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[1].sql, "SELECT 2");
    }

    #[test]
    fn test_parse_preserves_comments() {
        // SQL comments should be preserved and passed to Teradata
        let sql = "-- This is a comment\nSELECT 1;";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 1);
        assert!(statements[0].sql.contains("-- This is a comment"));
    }

    #[test]
    fn test_parse_multiline_comment() {
        let sql = "/* Multi-line\n   comment */\nSELECT 1;";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 1);
        assert!(statements[0].sql.contains("/* Multi-line"));
    }

    #[test]
    fn test_parse_complex_script() {
        let sql = r#"
-- Setup script
CREATE TABLE temp_data (id INT, value VARCHAR(100));

INSERT INTO temp_data VALUES (1, 'test');
INSERT INTO temp_data VALUES (2, 'test2');

/* Query the data */
SELECT * FROM temp_data;

-- Cleanup
DROP TABLE temp_data;
"#;
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 5);

        // Verify first statement includes comment
        assert!(statements[0].sql.contains("CREATE TABLE"));

        // Verify statement ordering
        assert!(statements[1].sql.contains("INSERT"));
        assert!(statements[2].sql.contains("INSERT"));
        assert!(statements[3].sql.contains("SELECT"));
        assert!(statements[4].sql.contains("DROP"));
    }

    #[test]
    fn test_parse_statement_with_newlines() {
        let sql = "SELECT\n  a,\n  b,\n  c\nFROM\n  table_name;";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 1);
        assert!(statements[0].sql.contains("SELECT"));
        assert!(statements[0].sql.contains("FROM"));
    }

    #[test]
    fn test_parse_trailing_semicolons() {
        let statements = parse_statements("SELECT 1;;;");
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 1");
    }

    #[test]
    fn test_parse_leading_whitespace() {
        let statements = parse_statements("  \n  SELECT 1;  \n  SELECT 2;  ");
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[1].sql, "SELECT 2");
    }

    #[test]
    fn test_statement_preview_short() {
        let stmt = ParsedStatement::new("SELECT 1".to_string(), 1, 1);
        assert_eq!(stmt.preview(80), "SELECT 1");
    }

    #[test]
    fn test_statement_preview_long() {
        let stmt = ParsedStatement::new(
            "SELECT a, b, c, d, e, f FROM very_long_table_name WHERE condition = 'something very long'"
                .to_string(),
            1,
            1,
        );
        let preview = stmt.preview(50);
        assert!(preview.len() <= 50);
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn test_statement_preview_normalizes_whitespace() {
        let stmt = ParsedStatement::new("SELECT\n    a,\n    b\nFROM\n    t".to_string(), 1, 1);
        let preview = stmt.preview(100);
        assert_eq!(preview, "SELECT a, b FROM t");
    }

    #[test]
    fn test_has_multiple_statements() {
        assert!(!has_multiple_statements("SELECT 1"));
        assert!(!has_multiple_statements("SELECT 1;"));
        assert!(has_multiple_statements("SELECT 1; SELECT 2"));
        assert!(has_multiple_statements("SELECT 1; SELECT 2; SELECT 3"));
    }

    #[test]
    fn test_has_multiple_statements_empty() {
        assert!(!has_multiple_statements(""));
        assert!(!has_multiple_statements(";;;"));
    }

    #[test]
    fn test_line_tracking_accuracy() {
        let sql = "SELECT 1;\n\n\nSELECT 2;";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].start_line, 1);
        assert_eq!(statements[1].start_line, 4); // After 3 newlines
    }

    #[test]
    fn test_windows_line_endings() {
        let sql = "SELECT 1;\r\nSELECT 2;\r\n";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 2);
        // Line tracking counts \n, so \r\n counts as one line
        assert_eq!(statements[0].start_line, 1);
    }
}
