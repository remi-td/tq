//! Teradata type formatting utilities
//!
//! This module provides utilities for formatting Teradata column types
//! from their internal type codes to human-readable names.
//!
//! # Type Codes
//!
//! Teradata uses single or two-character codes to represent data types
//! in system views like DBC.ColumnsV. This module translates these codes
//! to standard SQL type names.
//!
//! # Example
//!
//! ```
//! use tq::sql::types::format_column_type;
//!
//! // VARCHAR(100)
//! let varchar = format_column_type("CV", Some(100), None, None);
//! assert_eq!(varchar, "VARCHAR(100)");
//!
//! // DECIMAL(10,2)
//! let decimal = format_column_type("D", None, Some(10), Some(2));
//! assert_eq!(decimal, "DECIMAL(10,2)");
//!
//! // INTEGER
//! let integer = format_column_type("I", None, None, None);
//! assert_eq!(integer, "INTEGER");
//! ```

/// Format a Teradata column type code to human-readable SQL type name
///
/// Translates Teradata's internal type codes (from DBC.ColumnsV.ColumnType)
/// to standard SQL type names with appropriate length/precision/scale modifiers.
///
/// # Arguments
///
/// * `type_code` - The Teradata type code (e.g., "CV", "I", "D", "DA")
/// * `length` - Optional column length (for character/binary types)
/// * `precision` - Optional total digits (for numeric types)
/// * `scale` - Optional fractional digits (for decimal types)
///
/// # Returns
///
/// A human-readable SQL type name (e.g., "VARCHAR(100)", "DECIMAL(10,2)", "DATE")
///
/// # Type Code Reference
///
/// | Code | SQL Type |
/// |------|----------|
/// | CV | VARCHAR |
/// | CF | CHAR |
/// | I | INTEGER |
/// | I1 | BYTEINT |
/// | I2 | SMALLINT |
/// | I8 | BIGINT |
/// | D | DECIMAL |
/// | F | FLOAT |
/// | DA | DATE |
/// | TS | TIMESTAMP |
/// | TZ | TIMESTAMP WITH TIME ZONE |
/// | AT | TIME |
/// | BV | VARBYTE |
/// | BF | BYTE |
/// | CO | CLOB |
/// | BO | BLOB |
/// | JN | JSON |
pub fn format_column_type(
    type_code: &str,
    length: Option<i32>,
    precision: Option<i32>,
    scale: Option<i32>,
) -> String {
    match type_code.trim() {
        // Character types
        "CV" => format!("VARCHAR({})", length.unwrap_or(0)),
        "CF" => format!("CHAR({})", length.unwrap_or(0)),

        // Integer types
        "I" => "INTEGER".to_string(),
        "I1" => "BYTEINT".to_string(),
        "I2" => "SMALLINT".to_string(),
        "I8" => "BIGINT".to_string(),

        // Numeric types
        "D" => {
            if let (Some(p), Some(s)) = (precision, scale) {
                format!("DECIMAL({},{})", p, s)
            } else {
                "DECIMAL".to_string()
            }
        }
        "F" => "FLOAT".to_string(),

        // Date/time types
        "DA" => "DATE".to_string(),
        "TS" => "TIMESTAMP".to_string(),
        "TZ" => "TIMESTAMP WITH TIME ZONE".to_string(),
        "AT" => "TIME".to_string(),

        // Binary types
        "BV" => format!("VARBYTE({})", length.unwrap_or(0)),
        "BF" => format!("BYTE({})", length.unwrap_or(0)),

        // Large object types
        "CO" => "CLOB".to_string(),
        "BO" => "BLOB".to_string(),

        // JSON type
        "JN" => "JSON".to_string(),

        // Unknown type - return as-is
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Character types
    #[test]
    fn test_format_column_type_varchar() {
        let result = format_column_type("CV", Some(100), None, None);
        assert_eq!(result, "VARCHAR(100)");
    }

    #[test]
    fn test_format_column_type_varchar_zero_length() {
        let result = format_column_type("CV", None, None, None);
        assert_eq!(result, "VARCHAR(0)");
    }

    #[test]
    fn test_format_column_type_char() {
        let result = format_column_type("CF", Some(10), None, None);
        assert_eq!(result, "CHAR(10)");
    }

    #[test]
    fn test_format_column_type_char_with_whitespace() {
        // Type codes may have trailing whitespace from database
        let result = format_column_type("CF  ", Some(10), None, None);
        assert_eq!(result, "CHAR(10)");
    }

    // Integer types
    #[test]
    fn test_format_column_type_integer() {
        let result = format_column_type("I", None, None, None);
        assert_eq!(result, "INTEGER");
    }

    #[test]
    fn test_format_column_type_byteint() {
        let result = format_column_type("I1", None, None, None);
        assert_eq!(result, "BYTEINT");
    }

    #[test]
    fn test_format_column_type_smallint() {
        let result = format_column_type("I2", None, None, None);
        assert_eq!(result, "SMALLINT");
    }

    #[test]
    fn test_format_column_type_bigint() {
        let result = format_column_type("I8", None, None, None);
        assert_eq!(result, "BIGINT");
    }

    // Numeric types
    #[test]
    fn test_format_column_type_decimal_with_precision_scale() {
        let result = format_column_type("D", None, Some(10), Some(2));
        assert_eq!(result, "DECIMAL(10,2)");
    }

    #[test]
    fn test_format_column_type_decimal_no_precision() {
        let result = format_column_type("D", None, None, None);
        assert_eq!(result, "DECIMAL");
    }

    #[test]
    fn test_format_column_type_decimal_partial_precision() {
        // Only precision, no scale
        let result = format_column_type("D", None, Some(18), None);
        assert_eq!(result, "DECIMAL");
    }

    #[test]
    fn test_format_column_type_float() {
        let result = format_column_type("F", None, None, None);
        assert_eq!(result, "FLOAT");
    }

    // Date/time types
    #[test]
    fn test_format_column_type_date() {
        let result = format_column_type("DA", None, None, None);
        assert_eq!(result, "DATE");
    }

    #[test]
    fn test_format_column_type_timestamp() {
        let result = format_column_type("TS", None, None, None);
        assert_eq!(result, "TIMESTAMP");
    }

    #[test]
    fn test_format_column_type_timestamp_with_timezone() {
        let result = format_column_type("TZ", None, None, None);
        assert_eq!(result, "TIMESTAMP WITH TIME ZONE");
    }

    #[test]
    fn test_format_column_type_time() {
        let result = format_column_type("AT", None, None, None);
        assert_eq!(result, "TIME");
    }

    // Binary types
    #[test]
    fn test_format_column_type_varbyte() {
        let result = format_column_type("BV", Some(256), None, None);
        assert_eq!(result, "VARBYTE(256)");
    }

    #[test]
    fn test_format_column_type_byte() {
        let result = format_column_type("BF", Some(8), None, None);
        assert_eq!(result, "BYTE(8)");
    }

    // Large object types
    #[test]
    fn test_format_column_type_clob() {
        let result = format_column_type("CO", None, None, None);
        assert_eq!(result, "CLOB");
    }

    #[test]
    fn test_format_column_type_blob() {
        let result = format_column_type("BO", None, None, None);
        assert_eq!(result, "BLOB");
    }

    // JSON type
    #[test]
    fn test_format_column_type_json() {
        let result = format_column_type("JN", None, None, None);
        assert_eq!(result, "JSON");
    }

    // Unknown types
    #[test]
    fn test_format_column_type_unknown() {
        let result = format_column_type("XX", None, None, None);
        assert_eq!(result, "XX");
    }

    #[test]
    fn test_format_column_type_unknown_with_whitespace() {
        let result = format_column_type("  XX  ", None, None, None);
        assert_eq!(result, "XX");
    }

    // Edge cases
    #[test]
    fn test_format_column_type_empty() {
        let result = format_column_type("", None, None, None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_column_type_whitespace_only() {
        let result = format_column_type("   ", None, None, None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_column_type_case_sensitivity() {
        // Type codes are typically uppercase from Teradata
        // Lowercase should not match
        let result = format_column_type("cv", None, None, None);
        assert_eq!(result, "cv");
    }
}
