//! Columnar JSON (Compact) output formatting
//!
//! Provides dense, valid JSON formatting without the "key tax" of repeating
//! column names on every single row. Column names are declared once in `cols`,
//! followed by pure value tuples in `rows`.
//!
//! Example:
//! ```json
//! {"ok":true,"row_count":2,"cols":["id","name"],"rows":[[1,"Alice"],[2,"Bob"]]}
//! ```

use crate::db::{QueryResult, Value};
use crate::error::Result;
use crate::format::token_budget::{self, TokenOptions};
use crate::pagination::PaginationInfo;
use serde_json::{Map, Value as JsonValue};
use std::io::Write;

/// Options for Compact JSON formatting
#[derive(Debug, Clone, Default)]
pub struct CompactJsonOptions {
    /// Token compression options
    pub token_opts: TokenOptions,
}

/// Convert a DB Value to a JSON Value with optional token compression
fn value_to_compact_json(val: &Value, token_opts: &TokenOptions) -> JsonValue {
    if !token_opts.compress {
        return val.to_json();
    }

    match val {
        Value::Null => JsonValue::Null,
        Value::Decimal(d) => {
            if let Ok(capped) = token_budget::compress_float_str(&d.to_string(), token_opts.max_float_decimals).parse::<f64>() {
                serde_json::Number::from_f64(capped)
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null)
            } else {
                val.to_json()
            }
        }
        Value::String(s) => {
            let truncated = token_budget::truncate_string(s, token_opts.max_cell_chars);
            JsonValue::String(truncated)
        }
        _ => val.to_json(),
    }
}

/// Write query results as Compact Columnar JSON
pub fn write<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &CompactJsonOptions,
) -> Result<()> {
    write_internal(result, writer, options, None, None)
}

/// Write query results with timing in Compact Columnar JSON
pub fn write_with_timing<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &CompactJsonOptions,
) -> Result<()> {
    let timing_ms = result.execution_time.as_millis();
    write_internal(result, writer, options, Some(timing_ms), None)
}

/// Write query results with pagination in Compact Columnar JSON
pub fn write_with_pagination<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &CompactJsonOptions,
    pagination: Option<&PaginationInfo>,
) -> Result<()> {
    write_internal(result, writer, options, None, pagination)
}

/// Write query results with metadata and pagination in Compact Columnar JSON
pub fn write_with_metadata_and_pagination<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &CompactJsonOptions,
    pagination: Option<&PaginationInfo>,
) -> Result<()> {
    let timing_ms = result.execution_time.as_millis();
    write_internal(result, writer, options, Some(timing_ms), pagination)
}

/// Internal Columnar JSON writer handling timing, pagination, and token budgeting
fn write_internal<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &CompactJsonOptions,
    timing_ms: Option<u128>,
    pagination: Option<&PaginationInfo>,
) -> Result<()> {
    let mut envelope = Map::new();
    envelope.insert("ok".to_string(), JsonValue::Bool(true));

    let col_names: Vec<JsonValue> = result
        .columns
        .iter()
        .map(|c| JsonValue::String(c.name.clone()))
        .collect();
    envelope.insert("cols".to_string(), JsonValue::Array(col_names));

    if let Some(ms) = timing_ms {
        envelope.insert(
            "elapsed_ms".to_string(),
            JsonValue::Number(serde_json::Number::from(ms as u64)),
        );
    }

    if let Some(pg) = pagination {
        let mut pg_map = Map::new();
        pg_map.insert("page".to_string(), JsonValue::Number(pg.page.into()));
        pg_map.insert(
            "page_size".to_string(),
            JsonValue::Number(pg.page_size.into()),
        );
        pg_map.insert(
            "total_pages".to_string(),
            JsonValue::Number(pg.total_pages().into()),
        );
        pg_map.insert(
            "total_rows".to_string(),
            JsonValue::Number(pg.total_rows.into()),
        );
        envelope.insert("pagination".to_string(), JsonValue::Object(pg_map));
    }

    // Build rows with token budget estimation
    let mut rows_json: Vec<JsonValue> = Vec::new();
    let mut truncated = false;

    // Baseline envelope tokens
    let mut current_tokens = 20;

    for row in &result.rows {
        let cells: Vec<JsonValue> = row
            .iter()
            .map(|val| value_to_compact_json(val, &options.token_opts))
            .collect();
        let row_val = JsonValue::Array(cells);

        let row_str = serde_json::to_string(&row_val).unwrap_or_default();
        let row_tokens = token_budget::estimate_tokens(&row_str);

        if let Some(budget) = options.token_opts.token_budget {
            if current_tokens + row_tokens > budget && !rows_json.is_empty() {
                truncated = true;
                break;
            }
        }

        rows_json.push(row_val);
        current_tokens += row_tokens;
    }

    envelope.insert(
        "row_count".to_string(),
        JsonValue::Number(rows_json.len().into()),
    );

    if truncated {
        envelope.insert("truncated".to_string(), JsonValue::Bool(true));
        envelope.insert(
            "total_rows".to_string(),
            JsonValue::Number(result.row_count.into()),
        );
        if let Some(b) = options.token_opts.token_budget {
            envelope.insert(
                "token_budget".to_string(),
                JsonValue::Number(serde_json::Number::from(b as u64)),
            );
        }
    }

    envelope.insert("rows".to_string(), JsonValue::Array(rows_json));

    // Calculate final tokens if show_tokens is set
    if options.token_opts.show_tokens {
        let serialized_preview = serde_json::to_string(&envelope).unwrap_or_default();
        let total_tokens = token_budget::estimate_tokens(&serialized_preview) + 3; // accounting for key
        envelope.insert(
            "tokens_est".to_string(),
            JsonValue::Number(serde_json::Number::from(total_tokens as u64)),
        );
    }

    // Write minified JSON
    serde_json::to_writer(&mut *writer, &JsonValue::Object(envelope))?;
    writeln!(writer)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{ColumnMetadata, TeradataType};
    use std::time::Duration;

    fn make_test_result() -> QueryResult {
        let columns = vec![
            ColumnMetadata::new("id", TeradataType::Integer, false),
            ColumnMetadata::new("name", TeradataType::Varchar, true),
        ];
        let rows = vec![
            vec![Value::Integer(1), Value::String("Alice".into())],
            vec![Value::Integer(2), Value::String("Bob".into())],
        ];
        QueryResult::new(columns, rows, Duration::from_millis(15))
    }

    #[test]
    fn test_compact_json_basic() {
        let res = make_test_result();
        let opts = CompactJsonOptions::default();

        let mut output = Vec::new();
        write(&res, &mut output, &opts).unwrap();
        let s = String::from_utf8(output).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(s.trim()).unwrap();
        assert_eq!(parsed["ok"], true);
        assert_eq!(parsed["row_count"], 2);
        assert_eq!(parsed["cols"], serde_json::json!(["id", "name"]));
        assert_eq!(
            parsed["rows"],
            serde_json::json!([[1, "Alice"], [2, "Bob"]])
        );
    }

    #[test]
    fn test_compact_json_show_tokens() {
        let res = make_test_result();
        let mut opts = CompactJsonOptions::default();
        opts.token_opts.show_tokens = true;

        let mut output = Vec::new();
        write(&res, &mut output, &opts).unwrap();
        let s = String::from_utf8(output).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(s.trim()).unwrap();
        assert!(parsed["tokens_est"].as_u64().unwrap() > 0);
    }
}
