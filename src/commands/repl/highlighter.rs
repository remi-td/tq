//! SQL Syntax Highlighting for the REPL
//!
//! Provides real-time syntax highlighting for SQL input using nu-ansi-term.
//! Colors are customizable and follow the specification:
//! - Keywords (SELECT, FROM, WHERE): Cyan bold
//! - Strings ('text'): Green
//! - Numbers (123, 45.67): Yellow
//! - Comments (-- comment, /* */): Gray italic
//! - Functions (COUNT, SUM): Magenta
//! - Operators (=, !=, AND, OR): White

use nu_ansi_term::{Color, Style};
use reedline::{Highlighter, StyledText};

/// SQL keywords that should be highlighted
const SQL_KEYWORDS: &[&str] = &[
    // Data Query Language (DQL)
    "SELECT",
    "SEL",
    "FROM",
    "WHERE",
    "AND",
    "OR",
    "NOT",
    "IN",
    "EXISTS",
    "BETWEEN",
    "LIKE",
    "IS",
    "NULL",
    "TRUE",
    "FALSE",
    "ORDER",
    "BY",
    "ASC",
    "DESC",
    "NULLS",
    "FIRST",
    "LAST",
    "GROUP",
    "HAVING",
    "DISTINCT",
    "ALL",
    "AS",
    "ON",
    "USING",
    "JOIN",
    "INNER",
    "LEFT",
    "RIGHT",
    "FULL",
    "OUTER",
    "CROSS",
    "NATURAL",
    "UNION",
    "INTERSECT",
    "EXCEPT",
    "MINUS",
    "LIMIT",
    "TOP",
    "SAMPLE",
    "OFFSET",
    "FETCH",
    "NEXT",
    "ROWS",
    "ONLY",
    "WITH",
    "RECURSIVE",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    // Data Definition Language (DDL)
    "CREATE",
    "ALTER",
    "DROP",
    "TRUNCATE",
    "RENAME",
    "COMMENT",
    "TABLE",
    "VIEW",
    "INDEX",
    "DATABASE",
    "SCHEMA",
    "SEQUENCE",
    "PRIMARY",
    "KEY",
    "FOREIGN",
    "REFERENCES",
    "UNIQUE",
    "CHECK",
    "DEFAULT",
    "CONSTRAINT",
    "CASCADE",
    "RESTRICT",
    "SET",
    // Data Manipulation Language (DML)
    "INSERT",
    "INTO",
    "VALUES",
    "UPDATE",
    "DELETE",
    "MERGE",
    "UPSERT",
    "REPLACE",
    // Data Control Language (DCL)
    "GRANT",
    "REVOKE",
    "DENY",
    // Transaction Control
    "BEGIN",
    "TRANSACTION",
    "COMMIT",
    "ROLLBACK",
    "SAVEPOINT",
    // Teradata-specific
    "QUALIFY",
    "PARTITION",
    "OVER",
    "VOLATILE",
    "MULTISET",
    "NORMALIZE",
    "PERIOD",
    "OVERLAPS",
    "CONTAINS",
];

/// SQL aggregate and scalar functions
const SQL_FUNCTIONS: &[&str] = &[
    // Aggregate functions
    "COUNT",
    "SUM",
    "AVG",
    "MIN",
    "MAX",
    "STDDEV",
    "VARIANCE",
    "FIRST_VALUE",
    "LAST_VALUE",
    "NTH_VALUE",
    "ROW_NUMBER",
    "RANK",
    "DENSE_RANK",
    "NTILE",
    "LAG",
    "LEAD",
    "CSUM",
    "MSUM",
    "MAVG", // Teradata-specific aggregates
    // String functions
    "CONCAT",
    "SUBSTR",
    "SUBSTRING",
    "TRIM",
    "LTRIM",
    "RTRIM",
    "UPPER",
    "LOWER",
    "LENGTH",
    "CHAR_LENGTH",
    "POSITION",
    "REPLACE",
    "TRANSLATE",
    "COALESCE",
    "NULLIF",
    "LPAD",
    "RPAD",
    "REVERSE",
    "INITCAP",
    // Numeric functions
    "ABS",
    "CEIL",
    "CEILING",
    "FLOOR",
    "ROUND",
    "TRUNC",
    "TRUNCATE",
    "MOD",
    "POWER",
    "SQRT",
    "EXP",
    "LOG",
    "LN",
    "SIGN",
    "RANDOM",
    "GREATEST",
    "LEAST",
    // Date/time functions
    "CURRENT_DATE",
    "CURRENT_TIME",
    "CURRENT_TIMESTAMP",
    "DATE",
    "TIME",
    "TIMESTAMP",
    "INTERVAL",
    "EXTRACT",
    "DATE_TRUNC",
    "ADD_MONTHS",
    "MONTHS_BETWEEN",
    "YEAR",
    "MONTH",
    "DAY",
    "HOUR",
    "MINUTE",
    "SECOND",
    // Type conversion
    "CAST",
    "CONVERT",
    "TO_CHAR",
    "TO_DATE",
    "TO_NUMBER",
    "TRYCAST",
    // Conditional
    "IIF",
    "NULLIFZERO",
    "ZEROIFNULL",
    "NVL",
    "NVL2",
    "DECODE",
];

/// SQL operators that get special highlighting
const SQL_OPERATORS: &[&str] = &["AND", "OR", "NOT", "IN", "EXISTS", "BETWEEN", "LIKE", "IS"];

/// SQL syntax highlighter implementing reedline's Highlighter trait
#[derive(Clone)]
pub struct SqlHighlighter {
    /// Style for SQL keywords (SELECT, FROM, etc.)
    keyword_style: Style,
    /// Style for string literals
    string_style: Style,
    /// Style for numeric literals
    number_style: Style,
    /// Style for comments
    comment_style: Style,
    /// Style for function names
    function_style: Style,
    /// Style for operators
    operator_style: Style,
    /// Style for normal text
    default_style: Style,
    /// Whether highlighting is enabled
    enabled: bool,
}

impl SqlHighlighter {
    /// Create a new SQL highlighter with default colors
    pub fn new() -> Self {
        Self {
            keyword_style: Style::new().fg(Color::Cyan).bold(),
            string_style: Style::new().fg(Color::Green),
            number_style: Style::new().fg(Color::Yellow),
            comment_style: Style::new().fg(Color::DarkGray).italic(),
            function_style: Style::new().fg(Color::Magenta),
            operator_style: Style::new().fg(Color::White),
            default_style: Style::default(),
            enabled: true,
        }
    }

    /// Create a disabled highlighter (no colors)
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::new()
        }
    }

    /// Enable or disable highlighting
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if a word is a SQL keyword
    fn is_keyword(word: &str) -> bool {
        let upper = word.to_uppercase();
        SQL_KEYWORDS.contains(&upper.as_str())
    }

    /// Check if a word is a SQL function
    fn is_function(word: &str) -> bool {
        let upper = word.to_uppercase();
        SQL_FUNCTIONS.contains(&upper.as_str())
    }

    /// Check if a word is a SQL operator keyword
    fn is_operator_keyword(word: &str) -> bool {
        let upper = word.to_uppercase();
        SQL_OPERATORS.contains(&upper.as_str())
    }

    /// Tokenize and highlight SQL text
    fn highlight_sql(&self, input: &str) -> StyledText {
        let mut styled = StyledText::new();

        if !self.enabled || input.is_empty() {
            styled.push((self.default_style, input.to_string()));
            return styled;
        }

        let mut chars = input.chars().peekable();
        let mut buffer = String::new();

        while let Some(c) = chars.next() {
            match c {
                // String literals (single quotes)
                '\'' => {
                    // Flush buffer first
                    if !buffer.is_empty() {
                        self.highlight_word(&buffer, &mut styled);
                        buffer.clear();
                    }

                    let mut string_literal = String::from("'");
                    loop {
                        match chars.next() {
                            Some('\'') => {
                                string_literal.push('\'');
                                // Check for escaped quote ('')
                                if chars.peek() == Some(&'\'') {
                                    string_literal.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                            Some(ch) => string_literal.push(ch),
                            None => break,
                        }
                    }
                    styled.push((self.string_style, string_literal));
                }

                // Single-line comment (--)
                '-' if chars.peek() == Some(&'-') => {
                    if !buffer.is_empty() {
                        self.highlight_word(&buffer, &mut styled);
                        buffer.clear();
                    }

                    let mut comment = String::from("-");
                    comment.push(chars.next().unwrap()); // second '-'

                    // Read until end of line
                    while let Some(&ch) = chars.peek() {
                        if ch == '\n' {
                            break;
                        }
                        comment.push(chars.next().unwrap());
                    }
                    styled.push((self.comment_style, comment));
                }

                // Multi-line comment (/* */)
                '/' if chars.peek() == Some(&'*') => {
                    if !buffer.is_empty() {
                        self.highlight_word(&buffer, &mut styled);
                        buffer.clear();
                    }

                    let mut comment = String::from("/");
                    comment.push(chars.next().unwrap()); // '*'

                    loop {
                        match chars.next() {
                            Some('*') if chars.peek() == Some(&'/') => {
                                comment.push('*');
                                comment.push(chars.next().unwrap());
                                break;
                            }
                            Some(ch) => comment.push(ch),
                            None => break,
                        }
                    }
                    styled.push((self.comment_style, comment));
                }

                // Whitespace - flush buffer
                c if c.is_whitespace() => {
                    if !buffer.is_empty() {
                        self.highlight_word(&buffer, &mut styled);
                        buffer.clear();
                    }
                    styled.push((self.default_style, c.to_string()));
                }

                // Punctuation and operators
                '(' | ')' | ',' | ';' | '+' | '*' | '/' | '%' | '=' | '<' | '>' | '!' | '.'
                | ':' | '[' | ']' | '{' | '}' => {
                    if !buffer.is_empty() {
                        self.highlight_word(&buffer, &mut styled);
                        buffer.clear();
                    }
                    styled.push((self.operator_style, c.to_string()));
                }

                // Other characters - add to buffer
                _ => {
                    buffer.push(c);
                }
            }
        }

        // Flush remaining buffer
        if !buffer.is_empty() {
            self.highlight_word(&buffer, &mut styled);
        }

        styled
    }

    /// Highlight a single word (identifier, keyword, function, or number)
    fn highlight_word(&self, word: &str, styled: &mut StyledText) {
        // Check if it's a number
        if self.is_number(word) {
            styled.push((self.number_style, word.to_string()));
        }
        // Check if it's a keyword
        else if Self::is_keyword(word) {
            // Operator keywords get different treatment
            if Self::is_operator_keyword(word) {
                styled.push((self.keyword_style, word.to_string()));
            } else {
                styled.push((self.keyword_style, word.to_string()));
            }
        }
        // Check if it's a function (word followed by parenthesis would be checked
        // at call site, but we highlight known functions anyway)
        else if Self::is_function(word) {
            styled.push((self.function_style, word.to_string()));
        }
        // Default style for identifiers
        else {
            styled.push((self.default_style, word.to_string()));
        }
    }

    /// Check if a string represents a number
    fn is_number(&self, s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        // Allow leading minus for negative numbers
        let s = s.strip_prefix('-').unwrap_or(s);

        if s.is_empty() {
            return false;
        }

        // Check for hex numbers
        if s.starts_with("0x") || s.starts_with("0X") {
            return s[2..].chars().all(|c| c.is_ascii_hexdigit());
        }

        // Check for decimal numbers (including floats with optional exponent)
        let mut has_dot = false;
        let mut has_exp = false;

        for (i, c) in s.chars().enumerate() {
            match c {
                '0'..='9' => continue,
                '.' if !has_dot && !has_exp => has_dot = true,
                'e' | 'E' if !has_exp && i > 0 => has_exp = true,
                '+' | '-'
                    if has_exp
                        && (s.chars().nth(i - 1) == Some('e')
                            || s.chars().nth(i - 1) == Some('E')) =>
                {
                    continue
                }
                _ => return false,
            }
        }

        true
    }
}

impl Default for SqlHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter for SqlHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        self.highlight_sql(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_keyword() {
        assert!(SqlHighlighter::is_keyword("SELECT"));
        assert!(SqlHighlighter::is_keyword("select"));
        assert!(SqlHighlighter::is_keyword("Select"));
        assert!(SqlHighlighter::is_keyword("FROM"));
        assert!(SqlHighlighter::is_keyword("WHERE"));
        assert!(!SqlHighlighter::is_keyword("employees"));
        assert!(!SqlHighlighter::is_keyword("foo"));
    }

    #[test]
    fn test_is_function() {
        assert!(SqlHighlighter::is_function("COUNT"));
        assert!(SqlHighlighter::is_function("count"));
        assert!(SqlHighlighter::is_function("SUM"));
        assert!(SqlHighlighter::is_function("AVG"));
        assert!(!SqlHighlighter::is_function("SELECT"));
        assert!(!SqlHighlighter::is_function("mytable"));
    }

    #[test]
    fn test_is_number() {
        let hl = SqlHighlighter::new();
        assert!(hl.is_number("123"));
        assert!(hl.is_number("45.67"));
        assert!(hl.is_number("-123"));
        assert!(hl.is_number("1.5e10"));
        assert!(hl.is_number("0x1F"));
        assert!(!hl.is_number("abc"));
        assert!(!hl.is_number("12abc"));
    }

    #[test]
    fn test_highlight_simple_select() {
        let hl = SqlHighlighter::new();
        let result = hl.highlight("SELECT * FROM employees", 0);

        // The result should contain styled segments
        // We can't easily test the exact styles, but we can verify it produces output
        assert!(!result.buffer.is_empty());
    }

    #[test]
    fn test_highlight_with_string() {
        let hl = SqlHighlighter::new();
        let result = hl.highlight("SELECT * FROM employees WHERE name = 'Alice'", 0);
        assert!(!result.buffer.is_empty());
    }

    #[test]
    fn test_highlight_with_comment() {
        let hl = SqlHighlighter::new();
        let result = hl.highlight("SELECT 1 -- this is a comment", 0);
        assert!(!result.buffer.is_empty());
    }

    #[test]
    fn test_highlight_with_numbers() {
        let hl = SqlHighlighter::new();
        let result = hl.highlight("SELECT 123, 45.67 FROM t", 0);
        assert!(!result.buffer.is_empty());
    }

    #[test]
    fn test_highlight_disabled() {
        let hl = SqlHighlighter::disabled();
        let result = hl.highlight("SELECT * FROM employees", 0);
        // When disabled, should return single segment with default style
        assert_eq!(result.buffer.len(), 1);
    }

    #[test]
    fn test_highlight_multiline_comment() {
        let hl = SqlHighlighter::new();
        let result = hl.highlight("SELECT /* comment */ 1", 0);
        assert!(!result.buffer.is_empty());
    }

    #[test]
    fn test_highlight_escaped_string() {
        let hl = SqlHighlighter::new();
        let result = hl.highlight("SELECT 'it''s a test'", 0);
        assert!(!result.buffer.is_empty());
    }

    #[test]
    fn test_teradata_keywords() {
        assert!(SqlHighlighter::is_keyword("QUALIFY"));
        assert!(SqlHighlighter::is_keyword("SEL")); // Teradata abbreviation
        assert!(SqlHighlighter::is_keyword("TOP"));
        assert!(SqlHighlighter::is_keyword("SAMPLE"));
    }

    #[test]
    fn test_teradata_functions() {
        assert!(SqlHighlighter::is_function("CSUM"));
        assert!(SqlHighlighter::is_function("MSUM"));
        assert!(SqlHighlighter::is_function("MAVG"));
    }
}
