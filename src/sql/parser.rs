//! SQL statement parsing for batch mode execution
//!
//! This module provides a quote-aware, comment-aware SQL statement parser for batch execution.
//! It uses a single-pass character lexer with an explicit state machine to correctly handle:
//!
//! - Single-quoted string literals (including escaped quotes `''`)
//! - Line comments (`-- ...`)
//! - Block comments (`/* ... */`)
//! - Semicolons as statement boundaries (only in Normal state)
//! - `BEGIN ... END` bodies inside stored-procedure, trigger, and macro definitions
//!
//! # Design Decisions
//!
//! - **State-machine lexer**: Scans character-by-character with a four-state enum
//! - **Comments stripped**: Removed before statement assembly to prevent contamination
//! - **Quoted strings preserve**: Including escaped quotes (`''`)
//! - **Line tracking**: Incremented on every `\n` regardless of state
//! - **Column tracking**: Incremented per character, reset to 1 after `\n`
//! - **Empty handling**: Skip whitespace-only or comment-only statements
//! - **Error reporting**: Unterminated strings and block comments return `ParseError`
//!   with line and column of the opening delimiter
//! - **BEGIN/END depth**: A counter inhibits `;` as a statement terminator while
//!   inside a procedure/trigger/macro body. The body is entered on the first
//!   `BEGIN` that follows a `(CREATE|REPLACE) ... (PROCEDURE|TRIGGER|MACRO|FUNCTION)`
//!   header, and exited when the matching bare `END` brings the depth back to 0.
//!
//! See `docs/design/batch-mode.md` for the full design rationale.

use regex::Regex;
use std::fmt;
use std::sync::OnceLock;

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

/// Error returned when SQL input cannot be parsed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Human-readable description of the error
    pub message: String,
    /// 1-based line number where the error originates
    pub line: usize,
    /// 1-based column number where the error originates
    pub column: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {}, column {}",
            self.message, self.line, self.column
        )
    }
}

impl std::error::Error for ParseError {}

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

/// Returns `true` if `ch` is part of an ASCII word (letter, digit, or underscore).
///
/// Used as the word-boundary test for BEGIN/END keyword detection. An identifier
/// such as `BEGIN_DATE` must NOT be recognised as the keyword `BEGIN`, so we
/// reject the token if the character immediately before or after it satisfies
/// this predicate.
#[inline]
fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

/// Lazily-compiled regex that matches a `(CREATE|REPLACE) ... (PROCEDURE|TRIGGER|MACRO|FUNCTION)`
/// sequence in the current statement buffer.
///
/// Case-insensitive. Word-boundaries are enforced on both ends. The lazy `[\s\S]*?`
/// bound is safe because the buffer is cleared on every top-level `;` emission.
fn procedure_header_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:CREATE|REPLACE)\b[\s\S]*?\b(?:PROCEDURE|TRIGGER|MACRO|FUNCTION)\b")
            .expect("valid procedure-header regex")
    })
}

/// Returns `true` if `buf` contains a procedure/trigger/macro/function header
/// preceding the current position.
#[inline]
fn is_procedure_header(buf: &str) -> bool {
    procedure_header_regex().is_match(buf)
}

/// Peek ahead (without consuming) to determine whether an `END` keyword just
/// matched is actually one of the compound SPL forms `END IF`, `END LOOP`,
/// `END WHILE`, `END CASE`, or `END FOR`. These close an inner block, not the
/// procedure body, so they MUST NOT decrement the BEGIN/END depth counter.
///
/// `remainder` is a slice of the remaining input (the characters the iterator
/// has not yet yielded). We skip leading whitespace and compare the next word
/// against the set of inner-block keywords.
fn is_compound_end(remainder: &str) -> bool {
    // Skip leading whitespace AND SQL comments (line `--` and block `/* */`)
    // so that forms like `END -- note\n IF` and `END /* x */ IF` classify
    // correctly. Without comment-skip we would mis-read a compound-END as a
    // bare END and decrement the body-depth counter prematurely.
    let mut s = remainder;
    loop {
        let before = s.len();
        s = s.trim_start();
        if let Some(rest) = s.strip_prefix("--") {
            // Line comment runs to end-of-line (or end-of-input)
            s = match rest.find('\n') {
                Some(nl) => &rest[nl + 1..],
                None => "",
            };
        } else if let Some(rest) = s.strip_prefix("/*") {
            // Block comment runs to `*/` (or end-of-input — unterminated,
            // in which case there is nothing further to classify)
            s = match rest.find("*/") {
                Some(end) => &rest[end + 2..],
                None => "",
            };
        }
        if s.len() == before {
            break;
        }
    }

    // Extract the next word (run of ASCII alphanumeric + underscore)
    let word_end = s.find(|c: char| !is_word_char(c)).unwrap_or(s.len());
    if word_end == 0 {
        return false;
    }

    let next_word = &s[..word_end];
    matches!(
        next_word.to_ascii_uppercase().as_str(),
        "IF" | "LOOP" | "WHILE" | "CASE" | "FOR"
    )
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
/// `Ok(Vec<ParsedStatement>)` on success. Empty statements (whitespace-only or
/// comment-only) are skipped.
///
/// `Err(ParseError)` if the input contains an unterminated string literal or
/// block comment, with line/column pointing to the opening delimiter.
///
/// # Example
/// ```
/// use tq::sql::parser::parse_statements;
///
/// let sql = "SELECT 1;\nSELECT 2;\n\nSELECT 3;";
/// let statements = parse_statements(sql).unwrap();
///
/// assert_eq!(statements.len(), 3);
/// assert_eq!(statements[0].sql, "SELECT 1");
/// assert_eq!(statements[0].statement_number, 1);
/// assert_eq!(statements[0].start_line, 1);
/// ```
pub fn parse_statements(sql: &str) -> Result<Vec<ParsedStatement>, ParseError> {
    let mut statements: Vec<ParsedStatement> = Vec::new();
    let mut state = LexState::Normal;

    // Buffer for the current statement's content (comments excluded)
    let mut current = String::new();
    // Line number of the first content character in `current`
    let mut stmt_start_line: Option<usize> = None;
    let mut current_line: usize = 1;
    let mut current_col: usize = 1;
    let mut statement_number: usize = 0;

    // Track opening position for unterminated string/comment detection
    let mut string_start_line: usize = 0;
    let mut string_start_col: usize = 0;
    let mut comment_start_line: usize = 0;
    let mut comment_start_col: usize = 0;

    // BEGIN/END depth counter for SPL bodies. While > 0, semicolons in the
    // Normal state are NOT treated as top-level statement terminators.
    let mut begin_end_depth: u32 = 0;
    // Line where the outermost SPL body `BEGIN` was opened, for error reporting
    // at end-of-input (REQ-BATCH-SPL-007).
    let mut body_open_line: usize = 0;

    // Byte offset into `sql`. Used to cheaply obtain the remaining-input slice
    // for the `is_compound_end` lookahead without reallocating. Incremented
    // by `ch.len_utf8()` after each character we accept from the iterator.
    let mut byte_offset: usize = 0;

    let mut chars = sql.chars().peekable();

    while let Some(ch) = chars.next() {
        // Advance our byte offset as soon as we consume a character. This
        // keeps `sql[byte_offset..]` pointing at the characters the iterator
        // has not yet yielded, which we need for compound-`END` lookahead.
        let ch_bytes = ch.len_utf8();
        byte_offset += ch_bytes;
        // Line tracking applies in every state
        if ch == '\n' {
            // Count the newline before processing state transitions
            // (except InLineComment where \n triggers transition AND is counted)
            match state {
                LexState::InLineComment => {
                    // Transition back to Normal. Add a space to prevent token merging.
                    current_line += 1;
                    current_col = 1;
                    current.push(' ');
                    state = LexState::Normal;
                }
                LexState::InBlockComment => {
                    current_line += 1;
                    current_col = 1;
                    // Discard newlines inside block comments (already counted)
                }
                LexState::Normal => {
                    current_line += 1;
                    current_col = 1;
                    // Newlines in normal text go into the buffer
                    record_content(ch, &mut current, &mut stmt_start_line, current_line);
                }
                LexState::InSingleQuotedString => {
                    current_line += 1;
                    current_col = 1;
                    // Newlines inside strings are preserved
                    current.push(ch);
                }
            }
            continue;
        }

        match state {
            LexState::Normal => match ch {
                '\'' => {
                    string_start_line = current_line;
                    string_start_col = current_col;
                    record_content(ch, &mut current, &mut stmt_start_line, current_line);
                    state = LexState::InSingleQuotedString;
                    current_col += 1;
                }
                '-' if chars.peek() == Some(&'-') => {
                    chars.next(); // consume second '-'
                    byte_offset += 1; // the second '-' is one ASCII byte
                    state = LexState::InLineComment;
                    current_col += 2;
                }
                '/' if chars.peek() == Some(&'*') => {
                    comment_start_line = current_line;
                    comment_start_col = current_col;
                    chars.next(); // consume '*'
                    byte_offset += 1; // the '*' is one ASCII byte
                    state = LexState::InBlockComment;
                    current_col += 2;
                }
                ';' => {
                    if begin_end_depth > 0 {
                        // Inside a procedure/trigger/macro body -- the semicolon
                        // terminates an inner SPL statement, not the top-level
                        // statement. Preserve it in the buffer verbatim.
                        record_content(ch, &mut current, &mut stmt_start_line, current_line);
                        current_col += 1;
                    } else {
                        // Top-level statement boundary -- emit if non-empty
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
                        current_col += 1;
                    }
                }
                c if c.is_ascii_alphabetic() => {
                    // Potential keyword start. Enforce a left word boundary:
                    // if the preceding character is part of a word (e.g. the
                    // `_` or digit in `BEGIN_DATE`), this is an identifier
                    // continuation, not a keyword.
                    let left_boundary = current
                        .chars()
                        .next_back()
                        .is_none_or(|prev| !is_word_char(prev));

                    if left_boundary {
                        // Gather the full word by consuming continuation chars
                        // from the iterator.
                        let mut word = String::new();
                        word.push(c);
                        while let Some(&next) = chars.peek() {
                            if is_word_char(next) {
                                word.push(next);
                                chars.next();
                                byte_offset += next.len_utf8();
                            } else {
                                break;
                            }
                        }

                        // Update column tracking for every character in the word
                        current_col += word.chars().count();

                        // Classify the word
                        let upper = word.to_ascii_uppercase();
                        match upper.as_str() {
                            "BEGIN" => {
                                // Append first so the buffer (used by the
                                // procedure-header regex) contains the word.
                                // Use record_content for the first char to set
                                // start_line correctly.
                                let mut word_chars = word.chars();
                                let first = word_chars.next().expect("word has >=1 char");
                                record_content(
                                    first,
                                    &mut current,
                                    &mut stmt_start_line,
                                    current_line,
                                );
                                for rest in word_chars {
                                    current.push(rest);
                                }

                                // Open (or nest) a BEGIN/END body if:
                                //  - we are already inside a body (nesting), OR
                                //  - the statement buffer shows a procedure/trigger/
                                //    macro/function header.
                                // A naked `BEGIN TRANSACTION` at top level does NOT
                                // match the header regex and is therefore ignored.
                                if begin_end_depth > 0 || is_procedure_header(&current) {
                                    if begin_end_depth == 0 {
                                        body_open_line = current_line;
                                    }
                                    begin_end_depth = begin_end_depth.saturating_add(1);
                                }
                            }
                            "END" => {
                                // Append the word first.
                                let mut word_chars = word.chars();
                                let first = word_chars.next().expect("word has >=1 char");
                                record_content(
                                    first,
                                    &mut current,
                                    &mut stmt_start_line,
                                    current_line,
                                );
                                for rest in word_chars {
                                    current.push(rest);
                                }

                                // `END IF`, `END LOOP`, `END WHILE`, `END CASE`,
                                // `END FOR` close an inner block and must NOT
                                // decrement the body-depth counter. We peek the
                                // remaining input (without consuming) to classify.
                                if begin_end_depth > 0 {
                                    let remainder = &sql[byte_offset..];
                                    if !is_compound_end(remainder) {
                                        begin_end_depth -= 1;
                                    }
                                }
                            }
                            _ => {
                                // Not a tracked keyword -- just append as plain text.
                                let mut word_chars = word.chars();
                                let first = word_chars.next().expect("word has >=1 char");
                                record_content(
                                    first,
                                    &mut current,
                                    &mut stmt_start_line,
                                    current_line,
                                );
                                for rest in word_chars {
                                    current.push(rest);
                                }
                            }
                        }
                    } else {
                        // Identifier continuation -- just push this single char.
                        record_content(c, &mut current, &mut stmt_start_line, current_line);
                        current_col += 1;
                    }
                }
                other => {
                    record_content(other, &mut current, &mut stmt_start_line, current_line);
                    current_col += 1;
                }
            },

            LexState::InSingleQuotedString => match ch {
                '\'' if chars.peek() == Some(&'\'') => {
                    // Escaped quote -- consume both, append both to preserve literal.
                    // Safety: unwrap is safe here because peek() confirmed the next char exists.
                    let next = chars.next().unwrap();
                    byte_offset += next.len_utf8();
                    current.push(ch);
                    current.push(next);
                    current_col += 2;
                }
                '\'' => {
                    current.push(ch);
                    state = LexState::Normal;
                    current_col += 1;
                }
                other => {
                    current.push(other);
                    current_col += 1;
                }
            },

            LexState::InLineComment => {
                // Non-newline characters in a line comment are discarded.
                // Newline handling is done above in the \n branch.
                current_col += 1;
            }

            LexState::InBlockComment => {
                if ch == '*' && chars.peek() == Some(&'/') {
                    chars.next(); // consume '/'
                    byte_offset += 1; // the '/' is one ASCII byte
                    // Add a space to prevent token merging
                    current.push(' ');
                    state = LexState::Normal;
                    current_col += 2;
                } else {
                    // Block comment content discarded.
                    current_col += 1;
                }
            }
        }
    }

    // Check for unterminated constructs at end of input
    match state {
        LexState::InSingleQuotedString => {
            return Err(ParseError {
                message: "Unterminated string literal".to_string(),
                line: string_start_line,
                column: string_start_col,
            });
        }
        LexState::InBlockComment => {
            return Err(ParseError {
                message: "Unterminated block comment".to_string(),
                line: comment_start_line,
                column: comment_start_col,
            });
        }
        _ => {}
    }

    // If we reach end-of-input while still inside an SPL body, the BEGIN was
    // never closed by a matching END. Report it with the opening line so the
    // user can locate the unterminated block (REQ-BATCH-SPL-007).
    if begin_end_depth > 0 {
        return Err(ParseError {
            message: "Unterminated procedure/trigger/macro body".to_string(),
            line: body_open_line,
            column: 1,
        });
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

    Ok(statements)
}

/// Check if a SQL string contains multiple statements
///
/// This is useful for determining whether to use single-statement or batch execution.
/// If parsing fails (e.g. unterminated string), returns `false` as a safe default.
///
/// # Arguments
/// * `sql` - The SQL text to check
///
/// # Returns
/// `true` if the SQL contains more than one statement
pub fn has_multiple_statements(sql: &str) -> bool {
    parse_statements(sql)
        .ok()
        .is_some_and(|stmts| stmts.len() > 1)
}

// ============================================================================
// Significant token stream (shared lexical primitive)
// ============================================================================

/// A significant SQL token produced by [`significant_tokens`].
///
/// Whitespace, line comments (`-- ...`) and block comments (`/* ... */`) are
/// skipped by the iterator and never surfaced as tokens. The remaining tokens
/// are classified into the four cases below. This reuses the same quote/comment
/// state transitions as [`parse_statements`] so the two cannot diverge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlToken {
    /// An ASCII word run (letters, digits, underscore). Callers uppercase for
    /// keyword tests. The first character is always a letter or underscore
    /// because a leading digit run is yielded as a sequence of [`SqlToken::Other`].
    Word(String),
    /// A punctuation character relevant to CTE / LOCKING scanning: `(`, `)`,
    /// `,`, or `.`.
    Punct(char),
    /// A single-quoted string literal. The content is opaque and irrelevant to
    /// classification, so it is collapsed to this marker.
    StringLiteral,
    /// Any other single character (operators, other punctuation, digits, etc.).
    Other(char),
}

/// Iterate the significant tokens of `sql`, skipping arbitrary interleaved
/// whitespace, line comments (`-- ...`), and block comments (`/* ... */`).
///
/// The scanner mirrors the comment/quote handling of [`parse_statements`]:
///
/// - `--` starts a line comment that runs to the next `\n` (or end of input).
/// - `/*` starts a block comment that runs to the next `*/` (or end of input).
/// - `'` starts a single-quoted string; `''` is an embedded escaped quote. The
///   whole literal is yielded as a single [`SqlToken::StringLiteral`].
/// - A run of ASCII word characters becomes one [`SqlToken::Word`].
/// - `(`, `)`, `,`, `.` become [`SqlToken::Punct`]; anything else is
///   [`SqlToken::Other`].
///
/// An unterminated string or block comment simply ends the stream (the
/// classifier treats a truncated stream as unclassifiable / read-only-safe per
/// its own rules); no error is raised here because callers only need a best
/// effort prefix.
pub fn significant_tokens(sql: &str) -> impl Iterator<Item = SqlToken> + '_ {
    SignificantTokens {
        chars: sql.chars().peekable(),
    }
}

struct SignificantTokens<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl Iterator for SignificantTokens<'_> {
    type Item = SqlToken;

    fn next(&mut self) -> Option<SqlToken> {
        loop {
            let &ch = self.chars.peek()?;

            // Skip whitespace
            if ch.is_whitespace() {
                self.chars.next();
                continue;
            }

            // Line comment: -- ... \n
            if ch == '-' {
                self.chars.next();
                if self.chars.peek() == Some(&'-') {
                    self.chars.next();
                    // Consume to end of line (or input)
                    for c in self.chars.by_ref() {
                        if c == '\n' {
                            break;
                        }
                    }
                    continue;
                }
                // A lone '-' is an operator token.
                return Some(SqlToken::Other('-'));
            }

            // Block comment: /* ... */
            if ch == '/' {
                self.chars.next();
                if self.chars.peek() == Some(&'*') {
                    self.chars.next();
                    // Consume to closing */
                    let mut prev = '\0';
                    let mut closed = false;
                    for c in self.chars.by_ref() {
                        if prev == '*' && c == '/' {
                            closed = true;
                            break;
                        }
                        prev = c;
                    }
                    if !closed {
                        // Unterminated block comment: end the stream.
                        return None;
                    }
                    continue;
                }
                return Some(SqlToken::Other('/'));
            }

            // Single-quoted string literal
            if ch == '\'' {
                self.chars.next();
                loop {
                    match self.chars.next() {
                        Some('\'') => {
                            // Embedded escaped quote ('') stays inside the literal.
                            if self.chars.peek() == Some(&'\'') {
                                self.chars.next();
                                continue;
                            }
                            break;
                        }
                        Some(_) => continue,
                        // Unterminated string: end the stream.
                        None => return None,
                    }
                }
                return Some(SqlToken::StringLiteral);
            }

            // ASCII word run (letter or underscore start)
            if ch.is_ascii_alphabetic() || ch == '_' {
                let mut word = String::new();
                while let Some(&c) = self.chars.peek() {
                    if is_word_char(c) {
                        word.push(c);
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                return Some(SqlToken::Word(word));
            }

            // Relevant punctuation
            if matches!(ch, '(' | ')' | ',' | '.') {
                self.chars.next();
                return Some(SqlToken::Punct(ch));
            }

            // Anything else (operators, digits, etc.)
            self.chars.next();
            return Some(SqlToken::Other(ch));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_statement() {
        let statements = parse_statements("SELECT 1").unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[0].statement_number, 1);
        assert_eq!(statements[0].start_line, 1);
    }

    #[test]
    fn test_parse_single_statement_with_semicolon() {
        let statements = parse_statements("SELECT 1;").unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 1");
    }

    #[test]
    fn test_parse_multiple_statements() {
        let statements = parse_statements("SELECT 1; SELECT 2; SELECT 3;").unwrap();
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
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 3);

        // Line tracking
        assert_eq!(statements[0].start_line, 1);
        assert_eq!(statements[1].start_line, 2);
        assert_eq!(statements[2].start_line, 4); // Skips blank line
    }

    #[test]
    fn test_parse_empty_input() {
        let statements = parse_statements("").unwrap();
        assert!(statements.is_empty());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let statements = parse_statements("   \n\n   ").unwrap();
        assert!(statements.is_empty());
    }

    #[test]
    fn test_parse_empty_statements_skipped() {
        let statements = parse_statements("SELECT 1;;; SELECT 2;").unwrap();
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[1].sql, "SELECT 2");
    }

    #[test]
    fn test_parse_strips_line_comments() {
        // Line comments are stripped from statement output
        let sql = "-- This is a comment\nSELECT 1;";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 1");
    }

    #[test]
    fn test_parse_strips_block_comments() {
        // Block comments are stripped from statement output
        let sql = "/* Multi-line\n   comment */\nSELECT 1;";
        let statements = parse_statements(sql).unwrap();
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
        let statements = parse_statements(sql).unwrap();
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
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 1);
        assert!(statements[0].sql.contains("SELECT"));
        assert!(statements[0].sql.contains("FROM"));
    }

    #[test]
    fn test_parse_trailing_semicolons() {
        let statements = parse_statements("SELECT 1;;;").unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 1");
    }

    #[test]
    fn test_parse_leading_whitespace() {
        let statements = parse_statements("  \n  SELECT 1;  \n  SELECT 2;  ").unwrap();
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
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].start_line, 1);
        assert_eq!(statements[1].start_line, 4); // After 3 newlines
    }

    #[test]
    fn test_windows_line_endings() {
        let sql = "SELECT 1;\r\nSELECT 2;\r\n";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 2);
        // Line tracking counts \n, so \r\n counts as one line
        assert_eq!(statements[0].start_line, 1);
    }

    // --- Bug #28: Semicolons inside quoted strings ---

    #[test]
    fn test_semicolon_in_string_literal_not_a_boundary() {
        let sql = "INSERT INTO t (id, desc) VALUES (1, 'a; b');";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(
            statements[0].sql,
            "INSERT INTO t (id, desc) VALUES (1, 'a; b')"
        );
    }

    #[test]
    fn test_escaped_quote_with_semicolon_in_string() {
        let sql = "INSERT INTO t VALUES ('it''s; complex');";
        let statements = parse_statements(sql).unwrap();
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
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 1);
        assert!(statements[0].sql.contains("INSERT INTO employees"));
        assert!(statements[0].sql.contains("'Alice'"));
        assert_eq!(statements[0].start_line, 1);
    }

    // --- Bug #30: Comments between statements ---

    #[test]
    fn test_block_comment_between_statements_does_not_contaminate() {
        let sql = "SELECT 1; /* this is a comment */ SELECT 2;";
        let statements = parse_statements(sql).unwrap();
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
        let statements = parse_statements(sql).unwrap();
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
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 1");
    }

    #[test]
    fn test_comment_only_segment_is_skipped() {
        // A segment that is only a comment should not produce a statement
        let sql = "SELECT 1; -- just a comment\n; SELECT 2;";
        let statements = parse_statements(sql).unwrap();
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
        let statements = parse_statements(sql).unwrap();
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
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[1].sql, "SELECT 2");
    }

    #[test]
    fn test_string_with_newline() {
        // Newlines inside strings are preserved
        let sql = "SELECT 'line1\nline2';";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT 'line1\nline2'");
    }

    // --- Sprint 43: Unterminated string literal error ---

    #[test]
    fn test_unterminated_string_literal_returns_error() {
        let result = parse_statements("SELECT 'unterminated");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Unterminated string literal");
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 8); // The opening quote position
    }

    #[test]
    fn test_unterminated_string_on_second_line() {
        let result = parse_statements("SELECT 1;\nSELECT 'oops");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Unterminated string literal");
        assert_eq!(err.line, 2);
        assert_eq!(err.column, 8);
    }

    // --- Sprint 43: Unterminated block comment error ---

    #[test]
    fn test_unterminated_block_comment_returns_error() {
        let result = parse_statements("SELECT 1; /* never closed");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Unterminated block comment");
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 11); // The opening /* position
    }

    #[test]
    fn test_unterminated_block_comment_multiline() {
        let result = parse_statements("SELECT 1;\n/* comment starts here\nbut never ends");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Unterminated block comment");
        assert_eq!(err.line, 2);
        assert_eq!(err.column, 1);
    }

    // --- Sprint 43: Comment marker inside string is not a comment ---

    #[test]
    fn test_comment_marker_inside_string_is_not_comment() {
        // -- inside a string should NOT start a line comment
        let sql = "SELECT '-- not a comment';";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].sql, "SELECT '-- not a comment'");

        // /* inside a string should NOT start a block comment
        let sql2 = "SELECT '/* not a comment */';";
        let statements2 = parse_statements(sql2).unwrap();
        assert_eq!(statements2.len(), 1);
        assert_eq!(statements2[0].sql, "SELECT '/* not a comment */'");
    }

    // --- Sprint 43: ParseError Display ---

    #[test]
    fn test_parse_error_display() {
        let err = ParseError {
            message: "Unterminated string literal".to_string(),
            line: 3,
            column: 15,
        };
        assert_eq!(
            err.to_string(),
            "Unterminated string literal at line 3, column 15"
        );
    }

    // --- Sprint 43: has_multiple_statements returns false on parse error ---

    #[test]
    fn test_has_multiple_statements_returns_false_on_parse_error() {
        // Unterminated string should not panic, just return false
        assert!(!has_multiple_statements("SELECT 'unterminated"));
        assert!(!has_multiple_statements("/* unterminated comment"));
    }


    // =============================================================================
    // Bug #42: BEGIN/END depth tracking in statement splitter
    // TC094-A through TC094-I
    // =============================================================================

    // TC094-A: Single procedure — exact issue #42 repro

    #[test]
    fn test_procedure_body_is_single_statement() {
        let sql = "\
REPLACE PROCEDURE demo_user.sp_tq_repro()
BEGIN
    DECLARE v INTEGER;
    SET v = 1;
    IF v = 1 THEN
        SET v = 2;
    END IF;
END;";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 1, "procedure body must be a single statement");
        assert!(
            statements[0].sql.contains("DECLARE v INTEGER"),
            "body must contain DECLARE"
        );
        assert!(
            statements[0].sql.contains("END IF"),
            "body must contain END IF"
        );
        assert!(
            statements[0].sql.contains("SET v = 2"),
            "body must contain inner SET"
        );
    }

    // TC094-B: Nested BEGIN/END blocks

    #[test]
    fn test_nested_begin_end_blocks() {
        let sql = "\
REPLACE PROCEDURE test_user.sp_nested()
BEGIN
    DECLARE i INTEGER DEFAULT 0;
    lp: LOOP
        SET i = i + 1;
        IF i >= 3 THEN
            LEAVE lp;
        END IF;
    END LOOP lp;
END;";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(statements.len(), 1, "nested blocks must stay as one statement");
        assert!(
            statements[0].sql.contains("END LOOP lp"),
            "END LOOP must be in body"
        );
        assert!(statements[0].sql.contains("END IF"), "END IF must be in body");
        assert!(
            statements[0].sql.contains("DECLARE i INTEGER"),
            "body content preserved"
        );
    }

    // TC094-C: BEGIN inside a string literal does not open a block

    #[test]
    fn test_begin_in_string_literal_does_not_affect_depth() {
        let sql = "SELECT 'BEGIN' AS kw; SELECT 'END' AS kw2;";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(
            statements.len(),
            2,
            "BEGIN/END in string literals must not affect statement splitting"
        );
        assert_eq!(statements[0].sql, "SELECT 'BEGIN' AS kw");
        assert_eq!(statements[1].sql, "SELECT 'END' AS kw2");
    }

    #[test]
    fn test_begin_in_string_inside_procedure_body_does_not_add_depth() {
        let sql = "\
REPLACE PROCEDURE test_user.sp_str()
BEGIN
    SET v = 'BEGIN middle END';
END;
SELECT 1;";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(
            statements.len(),
            2,
            "procedure + trailing SELECT must be 2 statements"
        );
        assert!(
            statements[0].sql.contains("'BEGIN middle END'"),
            "string literal preserved in body"
        );
        assert_eq!(statements[1].sql, "SELECT 1");
    }

    // TC094-D: BEGIN/END inside comments do not affect block depth

    #[test]
    fn test_begin_in_line_comment_does_not_affect_depth() {
        let sql = "\
-- BEGIN: this is just a comment header
REPLACE PROCEDURE test_user.sp_comment()
BEGIN
    SET v = 1;
END;";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(
            statements.len(),
            1,
            "line comment with BEGIN must not affect procedure detection"
        );
        assert!(statements[0].sql.contains("SET v = 1"));
    }

    #[test]
    fn test_begin_in_block_comment_does_not_affect_depth() {
        let sql = "\
/* BEGIN setup block */
SELECT 1;
/* END setup block */
SELECT 2;";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(
            statements.len(),
            2,
            "block comment with BEGIN/END must not affect statement count"
        );
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[1].sql, "SELECT 2");
    }

    // TC094-E: Multi-procedure script — two procedures yield two statements

    #[test]
    fn test_multi_procedure_script() {
        let sql = "\
REPLACE PROCEDURE test_user.sp_first()
BEGIN
    SET v = 1;
END;
REPLACE PROCEDURE test_user.sp_second()
BEGIN
    SET w = 2;
END;";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(
            statements.len(),
            2,
            "two procedures must produce exactly two statements"
        );
        assert!(
            statements[0].sql.contains("sp_first"),
            "first statement is sp_first"
        );
        assert!(
            statements[1].sql.contains("sp_second"),
            "second statement is sp_second"
        );
    }

    // TC094-F: Mixed SPL + regular statements

    #[test]
    fn test_mixed_spl_and_regular_statements() {
        let sql = "\
REPLACE PROCEDURE test_user.sp_mixed()
BEGIN
    SET v = 1;
END;
SELECT 1;
INSERT INTO t VALUES (1);";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(
            statements.len(),
            3,
            "procedure + 2 regular statements must be 3 total"
        );
        assert!(
            statements[0].sql.contains("sp_mixed"),
            "first is the procedure"
        );
        assert_eq!(statements[1].sql, "SELECT 1");
        assert_eq!(statements[2].sql, "INSERT INTO t VALUES (1)");
    }

    // TC094-G: Case-insensitive header detection (PROCEDURE, TRIGGER)

    #[test]
    fn test_spl_headers_case_insensitive() {
        let sql_lower = "\
replace procedure test_user.sp_lower()
begin
    set v = 1;
end;";
        let stmts_lower = parse_statements(sql_lower).unwrap();
        assert_eq!(
            stmts_lower.len(),
            1,
            "lowercase procedure header must be detected"
        );

        let sql_trigger = "\
Create Trigger test_user.trg_mixed
After Insert On test_user.t
For Each Row
Begin
    Set v = 1;
End;";
        let stmts_trigger = parse_statements(sql_trigger).unwrap();
        assert_eq!(
            stmts_trigger.len(),
            1,
            "mixed-case CREATE TRIGGER must be detected"
        );
    }

    // TC094-H: CREATE vs REPLACE — both trigger body tracking

    #[test]
    fn test_create_procedure_also_tracked() {
        let sql = "\
CREATE PROCEDURE test_user.sp_create()
BEGIN
    DECLARE x INTEGER;
    SET x = 42;
END;";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(
            statements.len(),
            1,
            "CREATE PROCEDURE (not REPLACE) must also be tracked as single statement"
        );
        assert!(statements[0].sql.contains("DECLARE x INTEGER"));
    }

    // TC094-I: Regression — plain multi-statement scripts unaffected

    #[test]
    fn test_plain_multi_statement_regression() {
        let sql = "SELECT 1; SELECT 2; SELECT 3;";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(
            statements.len(),
            3,
            "plain multi-statement regression: must still split at semicolons"
        );
        assert_eq!(statements[0].sql, "SELECT 1");
        assert_eq!(statements[1].sql, "SELECT 2");
        assert_eq!(statements[2].sql, "SELECT 3");
    }

    // Sprint 64 review follow-ups
    // - Compound-END must skip intervening comments (`END -- x\n IF`, `END /* x */ IF`)
    // - Unterminated body at EOF must raise REQ-BATCH-SPL-007 error

    #[test]
    fn test_compound_end_with_line_comment_between() {
        let sql = "\
REPLACE PROCEDURE demo.sp_comment_between()
BEGIN
    IF 1 = 1 THEN
        SET x = 1;
    END -- closes the IF
    IF;
END;";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(
            statements.len(),
            1,
            "`END -- comment\\n IF` must still be recognised as compound END"
        );
    }

    #[test]
    fn test_compound_end_with_block_comment_between() {
        let sql = "\
REPLACE PROCEDURE demo.sp_block_between()
BEGIN
    IF 1 = 1 THEN
        SET x = 1;
    END /* closes the IF */ IF;
END;";
        let statements = parse_statements(sql).unwrap();
        assert_eq!(
            statements.len(),
            1,
            "`END /* comment */ IF` must still be recognised as compound END"
        );
    }

    #[test]
    fn test_unterminated_procedure_body_errors() {
        let sql = "\
REPLACE PROCEDURE demo.sp_bad()
BEGIN
    DECLARE v INTEGER;
    SET v = 1;
";
        let err = parse_statements(sql).expect_err("unterminated body must error");
        assert!(
            err.message.contains("Unterminated procedure"),
            "error message must identify unterminated procedure body, got: {}",
            err.message
        );
        assert_eq!(
            err.line, 2,
            "error line must point at the opening BEGIN line"
        );
    }

    #[test]
    fn test_unterminated_nested_body_errors() {
        let sql = "\
REPLACE PROCEDURE demo.sp_nested_bad()
BEGIN
    BEGIN
        SET x = 1;
    END;
";
        let err =
            parse_statements(sql).expect_err("outer BEGIN unterminated must error");
        assert!(err.message.contains("Unterminated"));
        assert_eq!(
            err.line, 2,
            "error should point at the OUTER (first-opened) BEGIN"
        );
    }

    // =============================================================================
    // Sprint 71: significant_tokens (shared lexical primitive)
    // =============================================================================

    fn words(sql: &str) -> Vec<String> {
        significant_tokens(sql)
            .filter_map(|t| match t {
                SqlToken::Word(w) => Some(w.to_ascii_uppercase()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn test_significant_tokens_skips_whitespace() {
        let toks: Vec<_> = significant_tokens("  SELECT   1 ").collect();
        assert_eq!(
            toks,
            vec![SqlToken::Word("SELECT".to_string()), SqlToken::Other('1')]
        );
    }

    #[test]
    fn test_significant_tokens_skips_line_comment() {
        assert_eq!(words("-- a comment\nSELECT 1"), vec!["SELECT"]);
    }

    #[test]
    fn test_significant_tokens_skips_block_comment() {
        assert_eq!(words("/* a */ /* b */ SELECT 1"), vec!["SELECT"]);
    }

    #[test]
    fn test_significant_tokens_skips_interleaved_comments() {
        assert_eq!(
            words("-- one\n/* two */\n-- three\nUPDATE t"),
            vec!["UPDATE", "T"]
        );
    }

    #[test]
    fn test_significant_tokens_collapses_string_literal() {
        let toks: Vec<_> = significant_tokens("WHERE x = 'a; -- b /* c */'").collect();
        // The string literal is opaque; its `;`, comment markers do not leak.
        assert!(toks.contains(&SqlToken::StringLiteral));
        assert_eq!(
            toks.iter()
                .filter(|t| matches!(t, SqlToken::StringLiteral))
                .count(),
            1
        );
    }

    #[test]
    fn test_significant_tokens_escaped_quote_in_string() {
        let toks: Vec<_> = significant_tokens("'it''s' END").collect();
        assert_eq!(
            toks,
            vec![SqlToken::StringLiteral, SqlToken::Word("END".to_string())]
        );
    }

    #[test]
    fn test_significant_tokens_punctuation() {
        let toks: Vec<_> = significant_tokens("a.b, (c)").collect();
        assert_eq!(
            toks,
            vec![
                SqlToken::Word("a".to_string()),
                SqlToken::Punct('.'),
                SqlToken::Word("b".to_string()),
                SqlToken::Punct(','),
                SqlToken::Punct('('),
                SqlToken::Word("c".to_string()),
                SqlToken::Punct(')'),
            ]
        );
    }

    #[test]
    fn test_significant_tokens_unterminated_string_ends_stream() {
        // No panic, stream simply ends at the unterminated literal.
        let toks: Vec<_> = significant_tokens("SELECT 'oops").collect();
        assert_eq!(toks, vec![SqlToken::Word("SELECT".to_string())]);
    }

    #[test]
    fn test_significant_tokens_unterminated_block_comment_ends_stream() {
        let toks: Vec<_> = significant_tokens("SELECT /* oops").collect();
        assert_eq!(toks, vec![SqlToken::Word("SELECT".to_string())]);
    }
}
