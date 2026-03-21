//! Shared utility functions for monitoring commands
//!
//! This module provides common extraction and formatting functions used across
//! all PMON monitoring commands (sessions, sysconfig, locks, query_inspect).
//!
//! Sprint 39: Extracted from duplicated implementations in sessions.rs,
//! sysconfig.rs, locks.rs, and sample.rs.

use crate::db::Value;

/// Extract an integer value from a Teradata row value
///
/// Handles Integer, Decimal (truncated to i64), and Null variants.
/// Returns None for Null or unrecognized types.
pub fn extract_integer(value: &Value) -> Option<i64> {
    match value {
        Value::Integer(v) => Some(*v),
        Value::Decimal(v) => Some(*v as i64),
        Value::Null => None,
        _ => None,
    }
}

/// Extract a decimal/float value from a Teradata row value
///
/// Handles Decimal, Integer (promoted to f64), and Null variants.
/// Returns None for Null or unrecognized types.
pub fn extract_decimal(value: &Value) -> Option<f64> {
    match value {
        Value::Decimal(v) => Some(*v),
        Value::Integer(v) => Some(*v as f64),
        Value::Null => None,
        _ => None,
    }
}

/// Extract a trimmed string value with configurable null display text
///
/// For String values, trims leading/trailing whitespace.
/// For Null, returns the specified `null_display` text.
/// For other types, uses the Value's display() method and trims.
///
/// # Arguments
/// * `value` - The database value to extract
/// * `null_display` - Text to display for NULL values (e.g., "[NULL]", "[unavailable]")
pub fn extract_trimmed_string(value: &Value, null_display: &str) -> String {
    match value {
        Value::String(s) => s.trim().to_string(),
        Value::Null => null_display.to_string(),
        other => other.display().trim().to_string(),
    }
}

/// Escape a string for CSV output
///
/// Wraps the string in double quotes if it contains a comma, double quote,
/// or newline character. Internal double quotes are escaped by doubling them.
pub fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // extract_integer tests
    // =========================================================================

    #[test]
    fn test_extract_integer_from_integer() {
        let value = Value::Integer(42);
        assert_eq!(extract_integer(&value), Some(42));
    }

    #[test]
    fn test_extract_integer_from_decimal() {
        let value = Value::Decimal(42.5);
        assert_eq!(extract_integer(&value), Some(42));
    }

    #[test]
    fn test_extract_integer_from_null() {
        let value = Value::Null;
        assert_eq!(extract_integer(&value), None);
    }

    #[test]
    fn test_extract_integer_from_string() {
        let value = Value::String("42".to_string());
        assert_eq!(extract_integer(&value), None);
    }

    #[test]
    fn test_extract_integer_from_boolean() {
        let value = Value::Boolean(true);
        assert_eq!(extract_integer(&value), None);
    }

    #[test]
    fn test_extract_integer_negative() {
        let value = Value::Integer(-100);
        assert_eq!(extract_integer(&value), Some(-100));
    }

    #[test]
    fn test_extract_integer_zero() {
        let value = Value::Integer(0);
        assert_eq!(extract_integer(&value), Some(0));
    }

    // =========================================================================
    // extract_decimal tests
    // =========================================================================

    #[test]
    fn test_extract_decimal_from_decimal() {
        let value = Value::Decimal(42.5);
        let result = extract_decimal(&value);
        assert!(result.is_some());
        assert!((result.unwrap() - 42.5).abs() < 0.001);
    }

    #[test]
    fn test_extract_decimal_from_integer() {
        let value = Value::Integer(42);
        let result = extract_decimal(&value);
        assert!(result.is_some());
        assert!((result.unwrap() - 42.0).abs() < 0.001);
    }

    #[test]
    fn test_extract_decimal_from_null() {
        let value = Value::Null;
        assert_eq!(extract_decimal(&value), None);
    }

    #[test]
    fn test_extract_decimal_from_string() {
        let value = Value::String("42.5".to_string());
        assert_eq!(extract_decimal(&value), None);
    }

    #[test]
    fn test_extract_decimal_negative() {
        #[allow(clippy::approx_constant)]
        let neg_val = -3.14;
        let value = Value::Decimal(neg_val);
        let result = extract_decimal(&value);
        assert!(result.is_some());
        assert!((result.unwrap() - neg_val).abs() < 0.001);
    }

    // =========================================================================
    // extract_trimmed_string tests
    // =========================================================================

    #[test]
    fn test_extract_trimmed_string_from_string() {
        let value = Value::String("  hello  ".to_string());
        assert_eq!(extract_trimmed_string(&value, "[NULL]"), "hello");
    }

    #[test]
    fn test_extract_trimmed_string_from_null_with_null_display() {
        let value = Value::Null;
        assert_eq!(extract_trimmed_string(&value, "[NULL]"), "[NULL]");
    }

    #[test]
    fn test_extract_trimmed_string_from_null_with_unavailable_display() {
        let value = Value::Null;
        assert_eq!(
            extract_trimmed_string(&value, "[unavailable]"),
            "[unavailable]"
        );
    }

    #[test]
    fn test_extract_trimmed_string_from_integer() {
        let value = Value::Integer(42);
        assert_eq!(extract_trimmed_string(&value, "[NULL]"), "42");
    }

    #[test]
    fn test_extract_trimmed_string_from_boolean() {
        let value = Value::Boolean(true);
        assert_eq!(extract_trimmed_string(&value, "[NULL]"), "true");
    }

    #[test]
    fn test_extract_trimmed_string_no_trim_needed() {
        let value = Value::String("hello".to_string());
        assert_eq!(extract_trimmed_string(&value, "[NULL]"), "hello");
    }

    #[test]
    fn test_extract_trimmed_string_empty() {
        let value = Value::String("   ".to_string());
        assert_eq!(extract_trimmed_string(&value, "[NULL]"), "");
    }

    // =========================================================================
    // escape_csv tests
    // =========================================================================

    #[test]
    fn test_escape_csv_simple() {
        assert_eq!(escape_csv("hello"), "hello");
    }

    #[test]
    fn test_escape_csv_with_comma() {
        assert_eq!(escape_csv("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn test_escape_csv_with_quotes() {
        assert_eq!(escape_csv("say \"hello\""), "\"say \"\"hello\"\"\"");
    }

    #[test]
    fn test_escape_csv_with_newline() {
        assert_eq!(escape_csv("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn test_escape_csv_empty() {
        assert_eq!(escape_csv(""), "");
    }

    #[test]
    fn test_escape_csv_with_all_special_chars() {
        assert_eq!(
            escape_csv("a,b\"c\nd"),
            "\"a,b\"\"c\nd\""
        );
    }

    #[test]
    fn test_escape_csv_no_special_chars() {
        let val = "17.20.00.17 (Released: 2024-01-15)";
        assert_eq!(escape_csv(val), val);
    }

    #[test]
    fn test_escape_csv_with_comma_in_parentheses() {
        let val = "17.20.00.17 (Released: January 15, 2024)";
        assert_eq!(
            escape_csv(val),
            "\"17.20.00.17 (Released: January 15, 2024)\""
        );
    }
}
