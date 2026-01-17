//! JSON output formatting
//!
//! Provides JSON formatting with:
//! - Array of objects format (default)
//! - Proper type preservation (numbers, booleans, null)
//! - Pretty printing for human readability

use crate::db::QueryResult;
use crate::error::Result;
use serde_json::{Map, Value as JsonValue};
use std::io::Write;

/// JSON formatting options
#[derive(Debug, Clone)]
pub struct JsonOptions {
    /// Pretty print with indentation
    pub pretty: bool,
}

impl Default for JsonOptions {
    fn default() -> Self {
        Self { pretty: true }
    }
}

/// Write query results as JSON array of objects
pub fn write<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &JsonOptions,
) -> Result<()> {
    let rows: Vec<JsonValue> = result
        .rows
        .iter()
        .map(|row| {
            let mut obj = Map::new();
            for (value, col) in row.iter().zip(&result.columns) {
                obj.insert(col.name.clone(), value.to_json());
            }
            JsonValue::Object(obj)
        })
        .collect();

    let json = JsonValue::Array(rows);

    if options.pretty {
        serde_json::to_writer_pretty(&mut *writer, &json)?;
    } else {
        serde_json::to_writer(&mut *writer, &json)?;
    }
    writeln!(writer)?;

    Ok(())
}

/// Write query results as JSONL (one JSON object per line)
///
/// This format is better for streaming and processing large datasets.
pub fn write_jsonl<W: Write>(result: &QueryResult, writer: &mut W) -> Result<()> {
    for row in &result.rows {
        let mut obj = Map::new();
        for (value, col) in row.iter().zip(&result.columns) {
            obj.insert(col.name.clone(), value.to_json());
        }
        serde_json::to_writer(&mut *writer, &JsonValue::Object(obj))?;
        writeln!(writer)?;
    }

    Ok(())
}

/// Write query results with metadata wrapper
///
/// Format:
/// {
///   "row_count": N,
///   "execution_time_ms": M,
///   "columns": [...],
///   "rows": [...]
/// }
pub fn write_with_metadata<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &JsonOptions,
) -> Result<()> {
    let mut wrapper = Map::new();

    // Add metadata
    wrapper.insert("row_count".to_string(), JsonValue::Number(result.row_count.into()));
    wrapper.insert(
        "execution_time_ms".to_string(),
        JsonValue::Number(serde_json::Number::from_f64(result.execution_time.as_secs_f64() * 1000.0).unwrap_or_else(|| 0.into())),
    );

    // Add column info
    let columns: Vec<JsonValue> = result
        .columns
        .iter()
        .map(|col| {
            let mut obj = Map::new();
            obj.insert("name".to_string(), JsonValue::String(col.name.clone()));
            obj.insert(
                "type".to_string(),
                JsonValue::String(format!("{:?}", col.data_type)),
            );
            obj.insert("nullable".to_string(), JsonValue::Bool(col.nullable));
            JsonValue::Object(obj)
        })
        .collect();
    wrapper.insert("columns".to_string(), JsonValue::Array(columns));

    // Add rows
    let rows: Vec<JsonValue> = result
        .rows
        .iter()
        .map(|row| {
            let mut obj = Map::new();
            for (value, col) in row.iter().zip(&result.columns) {
                obj.insert(col.name.clone(), value.to_json());
            }
            JsonValue::Object(obj)
        })
        .collect();
    wrapper.insert("rows".to_string(), JsonValue::Array(rows));

    let json = JsonValue::Object(wrapper);

    if options.pretty {
        serde_json::to_writer_pretty(&mut *writer, &json)?;
    } else {
        serde_json::to_writer(&mut *writer, &json)?;
    }
    writeln!(writer)?;

    Ok(())
}

/// Format results as a JSON string
pub fn format_string(result: &QueryResult, options: &JsonOptions) -> Result<String> {
    let mut buffer = Vec::new();
    write(result, &mut buffer, options)?;
    Ok(String::from_utf8_lossy(&buffer).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{ColumnMetadata, TeradataType, Value};
    use std::time::Duration;

    fn create_test_result() -> QueryResult {
        let columns = vec![
            ColumnMetadata::new("id", TeradataType::Integer, false),
            ColumnMetadata::new("name", TeradataType::Varchar, true),
            ColumnMetadata::new("active", TeradataType::Boolean, false),
            ColumnMetadata::new("score", TeradataType::Decimal, true),
        ];
        let rows = vec![
            vec![
                Value::Integer(1),
                Value::String("Alice".into()),
                Value::Boolean(true),
                Value::Decimal(95.5),
            ],
            vec![
                Value::Integer(2),
                Value::Null,
                Value::Boolean(false),
                Value::Decimal(87.3),
            ],
        ];
        QueryResult::new(columns, rows, Duration::from_millis(50))
    }

    #[test]
    fn test_write_json() {
        let result = create_test_result();
        let options = JsonOptions { pretty: false };

        let mut buffer = Vec::new();
        write(&result, &mut buffer, &options).unwrap();
        let output = String::from_utf8_lossy(&buffer);

        // Parse back to verify
        let parsed: Vec<serde_json::Value> = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed.len(), 2);

        // Check first row
        assert_eq!(parsed[0]["id"], 1);
        assert_eq!(parsed[0]["name"], "Alice");
        assert_eq!(parsed[0]["active"], true);
        assert_eq!(parsed[0]["score"], 95.5);

        // Check null handling
        assert!(parsed[1]["name"].is_null());
    }

    #[test]
    fn test_write_json_pretty() {
        let result = create_test_result();
        let options = JsonOptions { pretty: true };

        let output = format_string(&result, &options).unwrap();

        // Pretty output should contain newlines and indentation
        assert!(output.contains('\n'));
        assert!(output.contains("  "));
    }

    #[test]
    fn test_write_jsonl() {
        let result = create_test_result();

        let mut buffer = Vec::new();
        write_jsonl(&result, &mut buffer).unwrap();
        let output = String::from_utf8_lossy(&buffer);

        // Should have one JSON object per line
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines.len(), 2);

        // Each line should parse independently
        let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(first["name"], "Alice");

        let second: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert!(second["name"].is_null());
    }

    #[test]
    fn test_write_with_metadata() {
        let result = create_test_result();
        let options = JsonOptions { pretty: false };

        let mut buffer = Vec::new();
        write_with_metadata(&result, &mut buffer, &options).unwrap();
        let output = String::from_utf8_lossy(&buffer);

        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();

        assert_eq!(parsed["row_count"], 2);
        assert!(parsed["execution_time_ms"].as_f64().unwrap() > 0.0);
        assert_eq!(parsed["columns"].as_array().unwrap().len(), 4);
        assert_eq!(parsed["rows"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_type_preservation() {
        let columns = vec![
            ColumnMetadata::new("num", TeradataType::Integer, false),
            ColumnMetadata::new("dec", TeradataType::Decimal, false),
            ColumnMetadata::new("bool", TeradataType::Boolean, false),
            ColumnMetadata::new("str", TeradataType::Varchar, false),
        ];
        let rows = vec![vec![
            Value::Integer(42),
            Value::Decimal(3.14),
            Value::Boolean(true),
            Value::String("hello".into()),
        ]];
        let result = QueryResult::new(columns, rows, Duration::ZERO);

        let output = format_string(&result, &JsonOptions { pretty: false }).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(output.trim()).unwrap();

        // Verify types are preserved
        assert!(parsed[0]["num"].is_i64());
        assert!(parsed[0]["dec"].is_f64());
        assert!(parsed[0]["bool"].is_boolean());
        assert!(parsed[0]["str"].is_string());
    }
}
