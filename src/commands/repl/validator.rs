//! SQL statement validator for multi-line history support
//!
//! This module provides a validator that detects when SQL statements are complete,
//! enabling reedline to store multi-line commands as single history entries.
//!
//! When the validator returns `Incomplete`, reedline:
//! - Does NOT save partial input to history
//! - Continues accepting input on new lines
//! - Accumulates all lines into a single buffer
//!
//! When the validator returns `Complete`, the ENTIRE accumulated buffer is saved
//! as one history entry, achieving the multi-line history behavior.
//!
//! Sprint 24: Implements REQ-HIST-001 through REQ-HIST-007 from specs/repl.md

use reedline::{ValidationResult, Validator};

/// Validates SQL statement completion for multi-line history support
///
/// Returns `Incomplete` until a semicolon terminator is found at the end
/// of the input, causing reedline to accumulate multi-line input as a
/// single history entry.
///
/// # Examples
///
/// ```ignore
/// let validator = SqlStatementValidator;
///
/// // Empty input is complete (allows pressing Enter on empty line)
/// assert_eq!(validator.validate(""), ValidationResult::Complete);
///
/// // Metacommands are always complete (single line)
/// assert_eq!(validator.validate("/help"), ValidationResult::Complete);
/// assert_eq!(validator.validate("\\q"), ValidationResult::Complete);
///
/// // SQL without semicolon is incomplete
/// assert_eq!(validator.validate("SELECT * FROM users"), ValidationResult::Incomplete);
///
/// // SQL with trailing semicolon is complete
/// assert_eq!(validator.validate("SELECT * FROM users;"), ValidationResult::Complete);
/// ```
#[derive(Debug, Clone, Default)]
pub struct SqlStatementValidator;

impl SqlStatementValidator {
    /// Create a new SQL statement validator
    pub fn new() -> Self {
        Self
    }
}

impl Validator for SqlStatementValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        let trimmed = line.trim();

        // Empty input is complete (allows pressing Enter on empty line)
        if trimmed.is_empty() {
            return ValidationResult::Complete;
        }

        // Metacommands are always complete (single line)
        // They start with '/' or '\' and are executed immediately
        if trimmed.starts_with('/') || trimmed.starts_with('\\') {
            return ValidationResult::Complete;
        }

        // SQL statements are complete when ending with semicolon
        //
        // Note: We use simple trailing semicolon detection for performance.
        // Edge cases (semicolons inside strings/comments) are rare in
        // interactive use and can be handled by users adjusting their input
        // (e.g., adding a space after the closing quote before continuing).
        //
        // Alternative considered: Full SQL lexer to detect semicolons in context
        // Rejected because: Performance overhead per keystroke, complexity,
        // and edge cases are rare enough that the simple approach is pragmatic.
        if trimmed.ends_with(';') {
            ValidationResult::Complete
        } else {
            ValidationResult::Incomplete
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input_complete() {
        let validator = SqlStatementValidator::new();
        assert!(matches!(validator.validate(""), ValidationResult::Complete));
        assert!(matches!(
            validator.validate("   "),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("\n"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("  \t  "),
            ValidationResult::Complete
        ));
    }

    #[test]
    fn test_metacommand_complete() {
        let validator = SqlStatementValidator::new();

        // Forward slash metacommands
        assert!(matches!(
            validator.validate("/help"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("/quit"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("/describe users"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("/list tables"),
            ValidationResult::Complete
        ));

        // Backslash metacommands (psql-style)
        assert!(matches!(
            validator.validate("\\q"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("\\dt"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("\\d users"),
            ValidationResult::Complete
        ));
    }

    #[test]
    fn test_sql_with_semicolon_complete() {
        let validator = SqlStatementValidator::new();

        // Single line SQL with semicolon
        assert!(matches!(
            validator.validate("SELECT 1;"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("SELECT * FROM users;"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("SELECT * FROM users WHERE id = 1;"),
            ValidationResult::Complete
        ));

        // Multi-line SQL with trailing semicolon
        assert!(matches!(
            validator.validate("SELECT *\nFROM users;"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("SELECT\n  col1,\n  col2\nFROM t;"),
            ValidationResult::Complete
        ));

        // With trailing whitespace (should still be complete)
        assert!(matches!(
            validator.validate("SELECT 1;  "),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("SELECT 1;\n"),
            ValidationResult::Complete
        ));
    }

    #[test]
    fn test_sql_without_semicolon_incomplete() {
        let validator = SqlStatementValidator::new();

        // Single line SQL without semicolon
        assert!(matches!(
            validator.validate("SELECT 1"),
            ValidationResult::Incomplete
        ));
        assert!(matches!(
            validator.validate("SELECT * FROM users"),
            ValidationResult::Incomplete
        ));
        assert!(matches!(
            validator.validate("SELECT * FROM users WHERE id = 1"),
            ValidationResult::Incomplete
        ));

        // Multi-line SQL without semicolon
        assert!(matches!(
            validator.validate("SELECT *\nFROM users"),
            ValidationResult::Incomplete
        ));
        assert!(matches!(
            validator.validate("SELECT\n  col1,\n  col2\nFROM t"),
            ValidationResult::Incomplete
        ));
    }

    #[test]
    fn test_semicolon_in_middle_incomplete() {
        let validator = SqlStatementValidator::new();

        // Semicolon in middle but not at end - still incomplete
        // (User needs to add final semicolon)
        assert!(matches!(
            validator.validate("SELECT ; FROM users"),
            ValidationResult::Incomplete
        ));

        // Note: This is a limitation - semicolons in strings will cause early
        // termination. This is documented as a known edge case that users can
        // work around by adjusting their input.
    }

    #[test]
    fn test_complex_sql_statements() {
        let validator = SqlStatementValidator::new();

        // INSERT statement
        assert!(matches!(
            validator.validate("INSERT INTO users (id, name) VALUES (1, 'test');"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("INSERT INTO users (id, name) VALUES (1, 'test')"),
            ValidationResult::Incomplete
        ));

        // UPDATE statement
        assert!(matches!(
            validator.validate("UPDATE users SET name = 'new' WHERE id = 1;"),
            ValidationResult::Complete
        ));

        // DELETE statement
        assert!(matches!(
            validator.validate("DELETE FROM users WHERE id = 1;"),
            ValidationResult::Complete
        ));

        // DDL statements
        assert!(matches!(
            validator.validate("CREATE TABLE t (id INT);"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("DROP TABLE t;"),
            ValidationResult::Complete
        ));
    }

    #[test]
    fn test_leading_whitespace_preserved() {
        let validator = SqlStatementValidator::new();

        // Leading whitespace should not affect completion
        assert!(matches!(
            validator.validate("   SELECT 1;"),
            ValidationResult::Complete
        ));
        assert!(matches!(
            validator.validate("   SELECT 1"),
            ValidationResult::Incomplete
        ));
    }

    #[test]
    fn test_validator_is_cloneable() {
        let validator = SqlStatementValidator::new();
        let _cloned = validator.clone();
        // Validator should be cloneable for use with reedline
    }

    #[test]
    fn test_validator_default() {
        let validator = SqlStatementValidator;
        assert!(matches!(
            validator.validate("SELECT 1;"),
            ValidationResult::Complete
        ));
    }
}
