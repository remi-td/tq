//! Database value types and query result models
//!
//! This module provides type-safe representations for database values and query results.
//! The Value enum preserves type information from Teradata for proper formatting.

use serde::Serialize;
use std::time::Duration;

/// Database value with proper type handling
///
/// Represents values retrieved from Teradata with appropriate Rust types.
/// Supports serialization to JSON with proper type preservation.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Value {
    /// NULL value
    Null,
    /// Boolean value
    Boolean(bool),
    /// Integer value (32-bit and smaller)
    Integer(i64),
    /// Decimal/float value
    Decimal(f64),
    /// String value (VARCHAR, CHAR)
    String(String),
    /// Date value (ISO 8601 format: YYYY-MM-DD)
    Date(String),
    /// Timestamp value (ISO 8601 format)
    Timestamp(String),
    /// Time value (HH:MM:SS)
    Time(String),
    /// Binary data (BLOB, BYTE)
    Bytes(Vec<u8>),
}

impl Value {
    /// Convert to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Value::Null => serde_json::Value::Null,
            Value::Boolean(b) => serde_json::Value::Bool(*b),
            Value::Integer(i) => serde_json::Value::Number((*i).into()),
            Value::Decimal(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Date(s) | Value::Timestamp(s) | Value::Time(s) => {
                serde_json::Value::String(s.clone())
            }
            Value::Bytes(b) => {
                // Encode bytes as base64 for JSON
                use base64::{engine::general_purpose::STANDARD, Engine};
                serde_json::Value::String(STANDARD.encode(b))
            }
        }
    }

    /// Format value for display (handles NULL specially)
    pub fn display(&self) -> String {
        match self {
            Value::Null => "[NULL]".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Decimal(f) => {
                // Format with reasonable precision
                if f.fract() == 0.0 {
                    format!("{:.1}", f)
                } else {
                    format!("{}", f)
                }
            }
            Value::String(s) => s.clone(),
            Value::Date(s) | Value::Timestamp(s) | Value::Time(s) => s.clone(),
            Value::Bytes(b) => format!("<{} bytes>", b.len()),
        }
    }

    /// Format value for CSV (empty string for NULL)
    pub fn to_csv_string(&self) -> String {
        match self {
            Value::Null => String::new(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Decimal(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Date(s) | Value::Timestamp(s) | Value::Time(s) => s.clone(),
            Value::Bytes(b) => {
                use base64::{engine::general_purpose::STANDARD, Engine};
                STANDARD.encode(b)
            }
        }
    }

    /// Check if value is NULL
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Check if value is numeric (for alignment purposes)
    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Integer(_) | Value::Decimal(_))
    }
}

/// Teradata SQL data types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeradataType {
    /// BYTEINT, SMALLINT, INTEGER
    Integer,
    /// BIGINT
    BigInt,
    /// SMALLINT
    SmallInt,
    /// DECIMAL, NUMERIC
    Decimal,
    /// FLOAT, REAL, DOUBLE PRECISION
    Float,
    /// CHAR
    Char,
    /// VARCHAR
    Varchar,
    /// DATE
    Date,
    /// TIME
    Time,
    /// TIMESTAMP
    Timestamp,
    /// BOOLEAN (Teradata extension)
    Boolean,
    /// BLOB
    Blob,
    /// CLOB
    Clob,
    /// BYTE, VARBYTE
    Byte,
    /// Unknown or unmapped type
    Unknown,
}

impl TeradataType {
    /// Get alignment for table formatting
    pub fn alignment(&self) -> Alignment {
        match self {
            TeradataType::Integer
            | TeradataType::BigInt
            | TeradataType::SmallInt
            | TeradataType::Decimal
            | TeradataType::Float => Alignment::Right,

            TeradataType::Boolean => Alignment::Center,

            _ => Alignment::Left,
        }
    }

    /// Map from JDBC-style type code to TeradataType
    pub fn from_type_code(code: i32) -> Self {
        match code {
            -6 | -5 => TeradataType::BigInt, // TINYINT, BIGINT
            4 => TeradataType::Integer,      // INTEGER
            5 => TeradataType::SmallInt,     // SMALLINT
            2 | 3 => TeradataType::Decimal,  // NUMERIC, DECIMAL
            6..=8 => TeradataType::Float,    // FLOAT, REAL, DOUBLE
            1 => TeradataType::Char,         // CHAR
            12 => TeradataType::Varchar,     // VARCHAR
            91 => TeradataType::Date,        // DATE
            92 => TeradataType::Time,        // TIME
            93 => TeradataType::Timestamp,   // TIMESTAMP
            16 => TeradataType::Boolean,     // BOOLEAN
            2004 => TeradataType::Blob,      // BLOB
            2005 => TeradataType::Clob,      // CLOB
            -2 | -3 => TeradataType::Byte,   // BINARY, VARBINARY
            _ => TeradataType::Unknown,
        }
    }
}

/// Text alignment for table formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

/// Column metadata from query result
#[derive(Debug, Clone)]
pub struct ColumnMetadata {
    /// Column name
    pub name: String,
    /// Teradata data type
    pub data_type: TeradataType,
    /// Whether the column can contain NULL
    pub nullable: bool,
}

impl ColumnMetadata {
    /// Create new column metadata
    pub fn new(name: impl Into<String>, data_type: TeradataType, nullable: bool) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable,
        }
    }
}

/// A row of query results
pub type Row = Vec<Value>;

/// Complete query result set with metadata
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Column metadata
    pub columns: Vec<ColumnMetadata>,
    /// Data rows
    pub rows: Vec<Row>,
    /// Total row count
    pub row_count: usize,
    /// Query execution time
    pub execution_time: Duration,
}

impl QueryResult {
    /// Create a new query result
    pub fn new(columns: Vec<ColumnMetadata>, rows: Vec<Row>, execution_time: Duration) -> Self {
        let row_count = rows.len();
        Self {
            columns,
            rows,
            row_count,
            execution_time,
        }
    }

    /// Create an empty query result
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: 0,
            execution_time: Duration::ZERO,
        }
    }

    /// Check if result is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get column names
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }

    /// Iterate over rows
    pub fn iter(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter()
    }
}

impl IntoIterator for QueryResult {
    type Item = Row;
    type IntoIter = std::vec::IntoIter<Row>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

impl<'a> IntoIterator for &'a QueryResult {
    type Item = &'a Row;
    type IntoIter = std::slice::Iter<'a, Row>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Null.display(), "[NULL]");
        assert_eq!(Value::Boolean(true).display(), "true");
        assert_eq!(Value::Integer(42).display(), "42");
        assert_eq!(Value::Decimal(3.15).display(), "3.15"); // Avoid clippy::approx_constant
        assert_eq!(Value::String("hello".into()).display(), "hello");
        assert_eq!(Value::Date("2024-01-15".into()).display(), "2024-01-15");
    }

    #[test]
    fn test_value_to_json() {
        assert_eq!(Value::Null.to_json(), serde_json::Value::Null);
        assert_eq!(Value::Boolean(true).to_json(), serde_json::json!(true));
        assert_eq!(Value::Integer(42).to_json(), serde_json::json!(42));
        assert_eq!(
            Value::String("test".into()).to_json(),
            serde_json::json!("test")
        );
    }

    #[test]
    fn test_value_to_csv() {
        assert_eq!(Value::Null.to_csv_string(), "");
        assert_eq!(Value::Integer(42).to_csv_string(), "42");
        assert_eq!(Value::String("hello".into()).to_csv_string(), "hello");
    }

    #[test]
    fn test_value_is_numeric() {
        assert!(Value::Integer(42).is_numeric());
        assert!(Value::Decimal(3.15).is_numeric()); // Avoid clippy::approx_constant
        assert!(!Value::String("42".into()).is_numeric());
        assert!(!Value::Null.is_numeric());
    }

    #[test]
    fn test_teradata_type_alignment() {
        assert_eq!(TeradataType::Integer.alignment(), Alignment::Right);
        assert_eq!(TeradataType::Decimal.alignment(), Alignment::Right);
        assert_eq!(TeradataType::Varchar.alignment(), Alignment::Left);
        assert_eq!(TeradataType::Boolean.alignment(), Alignment::Center);
    }

    #[test]
    fn test_query_result() {
        let columns = vec![
            ColumnMetadata::new("id", TeradataType::Integer, false),
            ColumnMetadata::new("name", TeradataType::Varchar, true),
        ];
        let rows = vec![
            vec![Value::Integer(1), Value::String("Alice".into())],
            vec![Value::Integer(2), Value::String("Bob".into())],
        ];
        let result = QueryResult::new(columns, rows, Duration::from_millis(100));

        assert_eq!(result.row_count, 2);
        assert!(!result.is_empty());
        assert_eq!(result.column_names(), vec!["id", "name"]);
        assert_eq!(result.execution_time, Duration::from_millis(100));
    }
}
