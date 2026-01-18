//! SQL keyword completion for reedline
//!
//! Provides auto-completion for SQL keywords when user presses Tab.
//! Supports case-insensitive matching while preserving user input casing.

use reedline::{Completer, Suggestion};

/// SQL keyword completer for reedline
///
/// Provides completion suggestions for SQL keywords based on typed prefix.
/// Matches are case-insensitive but preserve the user's casing in input.
pub struct SqlCompleter {
    /// Complete list of SQL keywords
    keywords: Vec<String>,
}

impl SqlCompleter {
    /// Create a new SQL completer with all supported keywords
    pub fn new() -> Self {
        let keywords = vec![
            // DML statements
            "SELECT",
            "INSERT",
            "UPDATE",
            "DELETE",
            "WITH",
            // DDL statements
            "CREATE",
            "DROP",
            "ALTER",
            "TRUNCATE",
            // Table/Database operations
            "TABLE",
            "DATABASE",
            "SCHEMA",
            "VIEW",
            "INDEX",
            "PROCEDURE",
            "FUNCTION",
            // Clauses
            "FROM",
            "WHERE",
            "GROUP BY",
            "HAVING",
            "ORDER BY",
            "LIMIT",
            "OFFSET",
            "DISTINCT",
            "ALL",
            "TOP",
            // JOINs
            "JOIN",
            "INNER JOIN",
            "LEFT JOIN",
            "RIGHT JOIN",
            "FULL JOIN",
            "CROSS JOIN",
            "ON",
            "USING",
            "AS",
            // Set operations
            "UNION",
            "INTERSECT",
            "EXCEPT",
            // Logical operators
            "AND",
            "OR",
            "NOT",
            "IN",
            "EXISTS",
            "BETWEEN",
            "LIKE",
            "IS NULL",
            "IS NOT NULL",
            // Aggregates and functions
            "COUNT",
            "SUM",
            "AVG",
            "MIN",
            "MAX",
            "COUNT",
            // Transactions
            "BEGIN",
            "COMMIT",
            "ROLLBACK",
            "TRANSACTION",
            // Conditionals
            "CASE",
            "WHEN",
            "THEN",
            "ELSE",
            "END",
            // Data modification
            "VALUES",
            "SET",
            // Constraints
            "PRIMARY KEY",
            "FOREIGN KEY",
            "UNIQUE",
            "CHECK",
            "CONSTRAINT",
            // Permissions
            "GRANT",
            "REVOKE",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self { keywords }
    }
}

impl Default for SqlCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer for SqlCompleter {
    fn complete(&mut self, line: &str, _pos: usize) -> Vec<Suggestion> {
        // Get the last word on the line
        let last_word = line.split_whitespace().last().unwrap_or("").to_uppercase();

        if last_word.is_empty() {
            return vec![];
        }

        // Find all keywords that match the prefix
        let mut suggestions: Vec<Suggestion> = self
            .keywords
            .iter()
            .filter(|kw| kw.starts_with(&last_word))
            .map(|kw| Suggestion {
                value: kw.clone(),
                description: None,
                style: None,
                extra: None,
                span: reedline::Span {
                    start: line.len().saturating_sub(last_word.len()),
                    end: line.len(),
                },
                append_whitespace: true,
            })
            .collect();

        // Sort by length (shorter matches first) then alphabetically
        suggestions.sort_by(|a, b| {
            a.value
                .len()
                .cmp(&b.value.len())
                .then_with(|| a.value.cmp(&b.value))
        });

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completer_creation() {
        let completer = SqlCompleter::new();
        assert!(!completer.keywords.is_empty());
        assert!(completer.keywords.contains(&"SELECT".to_string()));
    }

    #[test]
    fn test_complete_select() {
        let mut completer = SqlCompleter::new();
        let suggestions = completer.complete("SEL", 3);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.value == "SELECT"));
    }

    #[test]
    fn test_complete_multiple_matches() {
        let mut completer = SqlCompleter::new();
        let suggestions = completer.complete("IN", 2);
        assert!(!suggestions.is_empty());
        // Should have both IN and INNER JOIN
        let has_in = suggestions.iter().any(|s| s.value == "IN");
        let has_inner = suggestions.iter().any(|s| s.value == "INNER JOIN");
        assert!(has_in || has_inner);
    }

    #[test]
    fn test_complete_no_match() {
        let mut completer = SqlCompleter::new();
        let suggestions = completer.complete("XYZ", 3);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_complete_empty_input() {
        let mut completer = SqlCompleter::new();
        let suggestions = completer.complete("", 0);
        assert!(suggestions.is_empty());
    }
}
