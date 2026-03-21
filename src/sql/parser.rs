//! SQL statement parsing for batch mode execution
//!
//! This module provides a quote-aware, comment-aware SQL statement parser for batch execution.
//! It uses a single-pass character lexer with an explicit state machine to correctly handle:
//!
//! - Single-quoted string literals (including escaped quotes `''`)
//! - Line comments (`-- ...`)
//! - Block comments (`/* ... */`)
//! - Semicolons as statement boundaries (only in Normal state)
//!
//! # Design Decisions
//!
//! - **State-machine lexer**: Scans character-by-character with a four-state enum
//! - **Comments stripped**: Removed before statement assembly to prevent contamination
//! - **Quoted strings preserved**: Including escaped quotes (`''`)
//! - **Line tracking**: Incremented on every `\n` regardless of state
//! - **Empty handling**: Skip whitespace-only or comment-only statements
//!
//! See `docs/design/batch-mode.md` for the full design rationale.

/// Lexer state for SQL parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexState {
    /// Normal SQL text -- semicolons are statement separators here
    Normal,
    /// Inside a single-quoted string literal ('...')
    InSingleQuotedString,
    /// Inside a line comment (-- ... \n)
    InLineComment,
    /// Inside a block comment (/* ... */)
    InBlockComment,
}

/// A parsed SQL statement with metadata for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStatement {
    /// The SQL statement text (trimmed, comments stripped)
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

/// Push `ch` to `buf` and record the start line on first non-whitespace character.
#[inline]
fn record_content(ch: char, buf: &mut String, start_line: &mut Option<usize>, current_line: usize) {
    if start_line.is_none() && !ch.is_whitespace() {
        *start_line = Some(current_line);
    }
    buf.push(ch);
}

/// Parse SQL text into individual statements
///
/// Splits the input using a state-machine lexer that correctly handles quoted strings,
/// line comments, and block comments. Comments are stripped from statement text.
///
/// # Arguments
/// * `sql` - The SQL text to parse (may contain multiple statements)
///
/// # Returns
/// A vector of parsed statements. Empty statements (whitespace-only or comment-only)
/// are skipped.
///
/// # Example
/// ```
/// use tq::sql::parser::parse_statements;
///
/// let sql = "SELECT 1;\nSELECT 2;\n\nSELECT 3;";
/// let statements = parse_statements(sql);
///
/// assert_eq!(statements.len(), 3);
/// assert_eq!(statements[0].sql, "SELECT 1");
/// assert_eq!(statements[0].statement_number, 1);
/// assert_eq!(statements[0].start_line, 1);
/// ```
pub fn parse_statements(sql: &str) -> Vec<ParsedStatement> {
    let mut statements: Vec<ParsedStatement> = Vec::new();
    let mut state = LexState::Normal;

    // Buffer for the current statement's content (comments excluded)
    let mut current = String::new();
    // Line number of the first content character in `current`
    let mut stmt_start_line: Option<usize> = None;
    let mut current_line: usize = 1;
    let mut statement_number: usize = 0;

    let mut chars = sql.chars().peekable();

    while let Some(ch) = chars.next() {
        // Line tracking applies in every state
        if ch == '\n' {
            // Count the newline before processing state transitions
            // (except InLineComment where \n triggers transition AND is counted)
            match state {
                LexState::InLineComment => {
                    // Transition back to Normal. Add a space to prevent token merging.
                    current_line += 1;
                    current.push(' ');
                    state = LexState::Normal;
                }
                LexState::InBlockComment => {
                    current_line += 1;
                    // Discard newlines inside block comments (already counted)
                }
                LexState::Normal => {
                    current_line += 1;
                    // Newlines in normal text go into the buffer
                    record_content(ch, &mut current, &mut stmt_start_line, current_line);
                }
                LexState::InSingleQuotedString => {
                    current_line += 1;
                    // Newlines inside strings are preserved
                    current.push(ch);
                }
            }
            continue;
        }

        match state {
            LexState::Normal => match ch {
                '\'' => {
                    record_content(ch, &mut current, &mut stmt_start_line, current_line);
                    state = LexState::InSingleQuotedString;
                }
                '-' if chars.peek() == Some(&'-') => {
                    chars.next(); // consume second '-'
                    state = LexState::InLineComment;
                }
                '/' if chars.peek() == Some(&'*') => {
                    chars.next(); // consume '*'
                    state = LexState::InBlockComment;
                }
                ';' => {
                    // Statement boundary -- emit if non-empty
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        statement_number += 1;
                        statements.push(ParsedStatement::new(
                            trimmed,
                            statement_number,
                            stmt_start_line.unwrap_or(current_line),
                        ));
                    }
                    current.clear();
                    stmt_start_line = None;
                }
                other => {
                    record_content(other, &mut current, &mut stmt_start_line, current_line);
                }
            },

            LexState::InSingleQuotedString => match ch {
                '\'' if chars.peek() == Some(&'\'') => {
                    // Escaped quote -- consume both, append both to preserve literal
                    let next = chars.next().unwrap();
                    current.push(ch);
                    current.push(next);
                }
                '\'' => {
                    current.push(ch);
                    state = LexState::Normal;
                }
                other => current.push(other),
            },

            LexState::InLineComment => {
                // Non-newline characters in a line comment are discarded.
                // Newline handling is done above in the \n branch.
            }

            LexState::InBlockComment => {
                if ch == '*' && chars.peek() == Some(&'/') {
                    chars.next(); // consume '/'
                    // Add a space to prevent token merging
                    current.push(' ');
                    state = LexState::Normal;
                }
                // Block comment content discarded.
            }
        }
    }

    // Flush trailing statement (no terminating semicolon)
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        statement_number += 1;
        statements.push(ParsedStatement::new(
            trimmed,
            statement_number,
            stmt_start_line.unwrap_or(current_line),
        ));
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
    fn test_parse_strips_line_comments() {
        // Line comments are stripped from statement output
        let sql = "-- This is a comment\nSELECT 1;";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 1");
    }

    #[test]
    fn test_parse_strips_block_comments() {
        // Block comments are stripped from statement output
        let sql = "/* Multi-line\n   comment */\nSELECT 1;";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 1");
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

        // Comments are stripped; statements contain only SQL
        assert!(statements[0].sql.contains("CREATE TABLE"));
        assert!(!statements[0].sql.contains("Setup script"));

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

    // --- Bug #28: Semicolons inside quoted strings ---

    #[test]
    fn test_semicolon_in_string_literal_not_a_boundary() {
        let sql = "INSERT INTO t (id, desc) VALUES (1, 'a; b');";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 1);
        assert_eq!(
            statements[0].sql,
            "INSERT INTO t (id, desc) VALUES (1, 'a; b')"
        );
    }

    #[test]
    fn test_escaped_quote_with_semicolon_in_string() {
        let sql = "INSERT INTO t VALUES ('it''s; complex');";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 1);
        assert_eq!(
            statements[0].sql,
            "INSERT INTO t VALUES ('it''s; complex')"
        );
    }

    // --- Bug #29: Multi-line INSERT works correctly ---

    #[test]
    fn test_multi_line_statement_is_single_statement() {
        let sql = "INSERT INTO employees (\n  id,\n  name,\n  salary\n) VALUES (\n  1,\n  'Alice',\n  50000\n);";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 1);
        assert!(statements[0].sql.contains("INSERT INTO employees"));
        assert!(statements[0].sql.contains("'Alice'"));
        assert_eq!(statements[0].start_line, 1);
    }

    // --- Bug #30: Comments between statements ---

    #[test]
    fn test_block_comment_between_statements_does_not_contaminate() {
        let sql = "SELECT 1; /* this is a comment */ SELECT 2;";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[1].sql, "SELECT 2");
        // No comment text in either statement
        assert!(!statements[0].sql.contains("comment"));
        assert!(!statements[1].sql.contains("comment"));
    }

    #[test]
    fn test_line_comment_between_statements_does_not_contaminate() {
        let sql = "SELECT 1;\n-- this is a comment\nSELECT 2;";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[1].sql, "SELECT 2");
        assert!(!statements[1].sql.contains("comment"));
    }

    // --- Comment stripping ---

    #[test]
    fn test_comments_are_stripped_from_output() {
        // Inline comment after SQL
        let sql = "SELECT 1 -- get one\n;";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 1");
    }

    #[test]
    fn test_comment_only_segment_is_skipped() {
        // A segment that is only a comment should not produce a statement
        let sql = "SELECT 1; -- just a comment\n; SELECT 2;";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[1].sql, "SELECT 2");
    }

    // --- Mixed scenario ---

    #[test]
    fn test_mixed_multiline_comments_and_quoted_semicolons() {
        let sql = r#"
-- Create table
CREATE TABLE t (id INT, name VARCHAR(50));

/* Insert data with semicolons in strings */
INSERT INTO t VALUES (1, 'hello; world');
INSERT INTO t VALUES (2, 'it''s; a test');

-- Query
SELECT * FROM t WHERE name = 'hello; world';
"#;
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 4);
        assert!(statements[0].sql.starts_with("CREATE TABLE"));
        assert_eq!(
            statements[1].sql,
            "INSERT INTO t VALUES (1, 'hello; world')"
        );
        assert_eq!(
            statements[2].sql,
            "INSERT INTO t VALUES (2, 'it''s; a test')"
        );
        assert!(statements[3].sql.contains("'hello; world'"));
    }

    #[test]
    fn test_has_multiple_statements_with_quoted_semicolons() {
        // A semicolon inside a string should NOT count as a statement boundary
        assert!(!has_multiple_statements("SELECT 'a;b'"));
        assert!(has_multiple_statements("SELECT 'a;b'; SELECT 2"));
    }

    #[test]
    fn test_block_comment_spanning_multiple_lines() {
        let sql = "SELECT 1;\n/*\nThis is\na multi-line\ncomment\n*/\nSELECT 2;";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[1].sql, "SELECT 2");
    }

    #[test]
    fn test_string_with_newline() {
        // Newlines inside strings are preserved
        let sql = "SELECT 'line1\nline2';";
        let statements = parse_statements(sql);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 'line1\nline2'");
    }
}
