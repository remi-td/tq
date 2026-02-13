//! SQL identifier quoting and escaping utilities
//!
//! This module provides utilities for safely handling SQL identifiers and
//! string literals in Teradata SQL queries.
//!
//! # Security
//!
//! Proper quoting of SQL identifiers is critical for preventing SQL injection
//! attacks. Always use these utilities when incorporating user-provided table
//! or column names into SQL queries.
//!
//! # Teradata Identifier Rules
//!
//! Teradata follows ANSI SQL identifier quoting rules:
//! - Quoted identifiers are enclosed in double quotes `"identifier"`
//! - Embedded double quotes are escaped by doubling: `"my""table"`
//! - Quoted identifiers preserve case and allow special characters
//!
//! # Example
//!
//! ```
//! use tq::sql::identifiers::{quote_identifier, quote_qualified_name, escape_sql_string};
//!
//! // Quote a simple identifier
//! assert_eq!(quote_identifier("employees"), "\"employees\"");
//!
//! // Quote an identifier with special characters
//! assert_eq!(quote_identifier("my table"), "\"my table\"");
//!
//! // Quote an identifier with embedded quotes
//! assert_eq!(quote_identifier("user\"name"), "\"user\"\"name\"");
//!
//! // Quote a qualified name (database.table)
//! assert_eq!(quote_qualified_name("prod", "employees"), "\"prod\".\"employees\"");
//!
//! // Escape a string literal for use in WHERE clause
//! assert_eq!(escape_sql_string("O'Brien"), "O''Brien");
//! ```

/// Quote a SQL identifier with double quotes
///
/// Wraps the identifier in double quotes and escapes any embedded double quotes
/// by doubling them, following ANSI SQL standards.
///
/// This should be used for table names, column names, database names, and other
/// SQL identifiers that may contain special characters or reserved words.
///
/// # Arguments
///
/// * `identifier` - The identifier to quote
///
/// # Returns
///
/// The quoted identifier, safe for use in SQL queries
///
/// # Examples
///
/// ```
/// use tq::sql::identifiers::quote_identifier;
///
/// // Simple identifier
/// assert_eq!(quote_identifier("employees"), "\"employees\"");
///
/// // Identifier with space
/// assert_eq!(quote_identifier("my table"), "\"my table\"");
///
/// // Identifier with embedded quote
/// assert_eq!(quote_identifier("user\"name"), "\"user\"\"name\"");
/// ```
pub fn quote_identifier(identifier: &str) -> String {
    // Escape embedded double quotes by doubling them
    let escaped = identifier.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

/// Quote a qualified SQL name (database.table or schema.table)
///
/// Quotes both the database/schema name and table name with double quotes,
/// properly escaping any embedded double quotes.
///
/// # Arguments
///
/// * `database` - The database or schema name
/// * `table` - The table name
///
/// # Returns
///
/// The quoted qualified name in format `"database"."table"`
///
/// # Examples
///
/// ```
/// use tq::sql::identifiers::quote_qualified_name;
///
/// // Simple qualified name
/// assert_eq!(quote_qualified_name("prod", "employees"), "\"prod\".\"employees\"");
///
/// // Database with special characters
/// assert_eq!(quote_qualified_name("my db", "my table"), "\"my db\".\"my table\"");
///
/// // Names with embedded quotes
/// assert_eq!(quote_qualified_name("db\"1", "tbl\"2"), "\"db\"\"1\".\"tbl\"\"2\"");
/// ```
pub fn quote_qualified_name(database: &str, table: &str) -> String {
    format!("{}.{}", quote_identifier(database), quote_identifier(table))
}

/// Escape single quotes in SQL string literals
///
/// Escapes single quotes by doubling them, as required for SQL string literals.
/// This should be used for values in WHERE clauses, INSERT statements, etc.
///
/// **Note**: This is for string literals inside single quotes, NOT for identifiers.
/// Use `quote_identifier()` for table/column names.
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// The escaped string, safe for use as a SQL string literal value
///
/// # Examples
///
/// ```
/// use tq::sql::identifiers::escape_sql_string;
///
/// // Simple string
/// assert_eq!(escape_sql_string("hello"), "hello");
///
/// // String with single quote
/// assert_eq!(escape_sql_string("O'Brien"), "O''Brien");
///
/// // Multiple quotes
/// assert_eq!(escape_sql_string("it's a 'test'"), "it''s a ''test''");
/// ```
pub fn escape_sql_string(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // quote_identifier tests
    // =========================================================================

    #[test]
    fn test_quote_identifier_simple() {
        assert_eq!(quote_identifier("employees"), "\"employees\"");
    }

    #[test]
    fn test_quote_identifier_with_space() {
        assert_eq!(quote_identifier("my table"), "\"my table\"");
    }

    #[test]
    fn test_quote_identifier_with_multiple_spaces() {
        assert_eq!(quote_identifier("my  big  table"), "\"my  big  table\"");
    }

    #[test]
    fn test_quote_identifier_with_embedded_quote() {
        assert_eq!(quote_identifier("user\"name"), "\"user\"\"name\"");
    }

    #[test]
    fn test_quote_identifier_with_multiple_quotes() {
        assert_eq!(quote_identifier("a\"b\"c"), "\"a\"\"b\"\"c\"");
    }

    #[test]
    fn test_quote_identifier_with_special_characters() {
        assert_eq!(quote_identifier("table-name"), "\"table-name\"");
        assert_eq!(quote_identifier("table.name"), "\"table.name\"");
        assert_eq!(quote_identifier("table@name"), "\"table@name\"");
        assert_eq!(quote_identifier("table#name"), "\"table#name\"");
        assert_eq!(quote_identifier("table$name"), "\"table$name\"");
    }

    #[test]
    fn test_quote_identifier_with_leading_number() {
        assert_eq!(quote_identifier("123table"), "\"123table\"");
    }

    #[test]
    fn test_quote_identifier_all_numbers() {
        assert_eq!(quote_identifier("12345"), "\"12345\"");
    }

    #[test]
    fn test_quote_identifier_reserved_word() {
        assert_eq!(quote_identifier("SELECT"), "\"SELECT\"");
        assert_eq!(quote_identifier("FROM"), "\"FROM\"");
        assert_eq!(quote_identifier("TABLE"), "\"TABLE\"");
    }

    #[test]
    fn test_quote_identifier_empty() {
        assert_eq!(quote_identifier(""), "\"\"");
    }

    #[test]
    fn test_quote_identifier_whitespace_only() {
        assert_eq!(quote_identifier("   "), "\"   \"");
    }

    #[test]
    fn test_quote_identifier_unicode() {
        assert_eq!(quote_identifier("tabl_"), "\"tabl_\"");
        // Note: Teradata may have limitations on Unicode characters
    }

    #[test]
    fn test_quote_identifier_unicode_actual() {
        // Test Chinese characters
        assert_eq!(quote_identifier("表名"), "\"表名\"");
        assert_eq!(quote_identifier("用户数据"), "\"用户数据\"");

        // Test Arabic characters
        assert_eq!(quote_identifier("جدول"), "\"جدول\"");

        // Test Japanese characters (Hiragana, Katakana, Kanji)
        assert_eq!(quote_identifier("テーブル"), "\"テーブル\"");
        assert_eq!(quote_identifier("顧客名簿"), "\"顧客名簿\"");

        // Test Cyrillic characters
        assert_eq!(quote_identifier("таблица"), "\"таблица\"");

        // Test emoji
        assert_eq!(quote_identifier("data_📊"), "\"data_📊\"");
        assert_eq!(quote_identifier("users_👤"), "\"users_👤\"");

        // Test Unicode with embedded double quotes (must be escaped)
        assert_eq!(quote_identifier("表\"名"), "\"表\"\"名\"");
        assert_eq!(quote_identifier("данные\"база"), "\"данные\"\"база\"");

        // Test mixed ASCII and Unicode
        assert_eq!(quote_identifier("user_数据_table"), "\"user_数据_table\"");
        assert_eq!(quote_identifier("col1_表_col2"), "\"col1_表_col2\"");

        // Test accented Latin characters
        assert_eq!(quote_identifier("café"), "\"café\"");
        assert_eq!(quote_identifier("naïve"), "\"naïve\"");
        assert_eq!(quote_identifier("résumé"), "\"résumé\"");

        // Test Greek characters
        assert_eq!(quote_identifier("πίνακας"), "\"πίνακας\"");

        // Test Hebrew characters
        assert_eq!(quote_identifier("טבלה"), "\"טבלה\"");
    }

    #[test]
    fn test_quote_identifier_tab_and_newline() {
        // These are unusual but should be handled
        assert_eq!(quote_identifier("col\ttab"), "\"col\ttab\"");
        assert_eq!(quote_identifier("col\nnewline"), "\"col\nnewline\"");
    }

    #[test]
    fn test_quote_identifier_consecutive_quotes() {
        assert_eq!(quote_identifier("a\"\"b"), "\"a\"\"\"\"b\"");
    }

    #[test]
    fn test_quote_identifier_starts_with_quote() {
        assert_eq!(quote_identifier("\"table"), "\"\"\"table\"");
    }

    #[test]
    fn test_quote_identifier_ends_with_quote() {
        assert_eq!(quote_identifier("table\""), "\"table\"\"\"");
    }

    // =========================================================================
    // quote_qualified_name tests
    // =========================================================================

    #[test]
    fn test_quote_qualified_name_simple() {
        assert_eq!(
            quote_qualified_name("prod", "employees"),
            "\"prod\".\"employees\""
        );
    }

    #[test]
    fn test_quote_qualified_name_with_spaces() {
        assert_eq!(
            quote_qualified_name("my database", "my table"),
            "\"my database\".\"my table\""
        );
    }

    #[test]
    fn test_quote_qualified_name_with_quotes() {
        assert_eq!(
            quote_qualified_name("db\"1", "tbl\"2"),
            "\"db\"\"1\".\"tbl\"\"2\""
        );
    }

    #[test]
    fn test_quote_qualified_name_special_characters() {
        assert_eq!(
            quote_qualified_name("db-test", "table@prod"),
            "\"db-test\".\"table@prod\""
        );
    }

    #[test]
    fn test_quote_qualified_name_reserved_words() {
        assert_eq!(
            quote_qualified_name("SELECT", "FROM"),
            "\"SELECT\".\"FROM\""
        );
    }

    #[test]
    fn test_quote_qualified_name_empty_parts() {
        assert_eq!(quote_qualified_name("", ""), "\"\".\"\"");
    }

    #[test]
    fn test_quote_qualified_name_mixed_complexity() {
        assert_eq!(
            quote_qualified_name("simple", "complex \"name"),
            "\"simple\".\"complex \"\"name\""
        );
    }

    // =========================================================================
    // escape_sql_string tests
    // =========================================================================

    #[test]
    fn test_escape_sql_string_simple() {
        assert_eq!(escape_sql_string("test"), "test");
    }

    #[test]
    fn test_escape_sql_string_with_single_quote() {
        assert_eq!(escape_sql_string("O'Brien"), "O''Brien");
    }

    #[test]
    fn test_escape_sql_string_with_multiple_quotes() {
        assert_eq!(escape_sql_string("it's a 'test'"), "it''s a ''test''");
    }

    #[test]
    fn test_escape_sql_string_consecutive_quotes() {
        assert_eq!(escape_sql_string("a''b"), "a''''b");
    }

    #[test]
    fn test_escape_sql_string_starts_with_quote() {
        assert_eq!(escape_sql_string("'hello"), "''hello");
    }

    #[test]
    fn test_escape_sql_string_ends_with_quote() {
        assert_eq!(escape_sql_string("hello'"), "hello''");
    }

    #[test]
    fn test_escape_sql_string_only_quotes() {
        assert_eq!(escape_sql_string("'''"), "''''''");
    }

    #[test]
    fn test_escape_sql_string_empty() {
        assert_eq!(escape_sql_string(""), "");
    }

    #[test]
    fn test_escape_sql_string_no_quotes() {
        assert_eq!(escape_sql_string("hello world 123"), "hello world 123");
    }

    #[test]
    fn test_escape_sql_string_double_quotes_unchanged() {
        // Double quotes are not escaped in string literals
        assert_eq!(escape_sql_string("say \"hello\""), "say \"hello\"");
    }

    #[test]
    fn test_escape_sql_string_mixed_quotes() {
        // Only single quotes should be escaped
        assert_eq!(
            escape_sql_string("it's \"quoted\""),
            "it''s \"quoted\""
        );
    }

    #[test]
    fn test_escape_sql_string_with_special_chars() {
        // Other special characters should pass through unchanged
        assert_eq!(
            escape_sql_string("tab\there\nnewline"),
            "tab\there\nnewline"
        );
    }

    #[test]
    fn test_escape_sql_string_backslash() {
        // Backslashes are not special in standard SQL
        assert_eq!(escape_sql_string("path\\to\\file"), "path\\to\\file");
    }

    // =========================================================================
    // Integration tests - using both functions together
    // =========================================================================

    #[test]
    fn test_sql_injection_prevention_identifier() {
        // Attempt to inject SQL via identifier
        let malicious = "employees; DROP TABLE users; --";
        let quoted = quote_identifier(malicious);
        // The result should be a safely quoted identifier, not executable SQL
        assert_eq!(quoted, "\"employees; DROP TABLE users; --\"");
    }

    #[test]
    fn test_sql_injection_prevention_string() {
        // Attempt to inject SQL via string value
        let malicious = "'; DROP TABLE users; --";
        let escaped = escape_sql_string(malicious);
        // The result should be an escaped string, not executable SQL
        assert_eq!(escaped, "''; DROP TABLE users; --");
    }

    #[test]
    fn test_sql_injection_prevention_qualified_name() {
        // Attempt to inject SQL via qualified name
        let malicious_db = "db\".\"evil";
        let malicious_table = "table\"; DROP TABLE users; --";
        let quoted = quote_qualified_name(malicious_db, malicious_table);
        // Embedded quotes should be escaped
        assert_eq!(
            quoted,
            "\"db\"\".\"\"evil\".\"table\"\"; DROP TABLE users; --\""
        );
    }
}
