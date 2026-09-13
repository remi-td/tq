//! TOON (Token-Oriented Object Notation) output formatting
//!
//! TOON is designed specifically for Large Language Model context injection.
//! It combines YAML-style metadata comments with dense, unquoted or minimally-quoted
//! comma-delimited data rows to eliminate border, whitespace, and key repetition overhead.
//!
//! Example:
//! ```text
//! # rows: 2
//! [columns: order_id, customer_id, total_amount]
//! 1001, 452, 149.50
//! 1002, 891, 1200.00
//! ```

use crate::db::{QueryResult, Value};
use crate::error::Result;
use crate::format::token_budget::{self, TokenOptions};
use crate::pagination::PaginationInfo;
use std::io::Write;

/// Options for TOON formatting
#[derive(Debug, Clone, Default)]
pub struct ToonOptions {
    /// Token compression options
    pub token_opts: TokenOptions,
}

/// Escape and format a single TOON cell value.
///
/// Primitives (numbers, dates, simple strings) are left unquoted for token density.
/// Quotes are added only if the string contains commas, quotes, newlines, or tabs.
pub fn format_toon_cell(value: &Value, token_opts: &TokenOptions) -> String {
    match value {
        Value::Null => {
            if token_opts.compress {
                "~".to_string()
            } else {
                "NULL".to_string()
            }
        }
        Value::Integer(i) => i.to_string(),
        Value::Decimal(d) => {
            let s = d.to_string();
            if token_opts.compress {
                token_budget::compress_float_str(&s, token_opts.max_float_decimals)
            } else {
                s
            }
        }
        Value::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Date(d) => d.to_string(),
        Value::Time(t) => t.to_string(),
        Value::Timestamp(ts) => ts.to_string(),
        Value::String(s) => {
            let text = if token_opts.compress {
                token_budget::truncate_string(s, token_opts.max_cell_chars)
            } else {
                s.clone()
            };

            // Needs quoting if contains delimiter, quote, newline, or tab
            if text.contains(',') || text.contains('"') || text.contains('\n') || text.contains('\t') || text.starts_with(' ') || text.ends_with(' ') {
                format!("\"{}\"", text.replace('"', "\"\""))
            } else if text.is_empty() {
                "\"\"".to_string()
            } else if text == "~" || text == "NULL" {
                format!("\"{}\"", text)
            } else {
                text
            }
        }
        Value::Bytes(bytes) => {
            let hex: String = bytes.iter().map(|b| format!("{:02X}", b)).collect();
            format!("0x{}", hex)
        }
    }
}

/// Write query results as TOON format
pub fn write<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &ToonOptions,
) -> Result<()> {
    write_internal(result, writer, options, None, None)
}

/// Write query results with timing in TOON format
pub fn write_with_timing<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &ToonOptions,
) -> Result<()> {
    let timing_str = format!("{:.3}s", result.execution_time.as_secs_f64());
    write_internal(result, writer, options, Some(&timing_str), None)
}

/// Write query results with pagination in TOON format
pub fn write_with_pagination<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &ToonOptions,
    pagination: Option<&PaginationInfo>,
) -> Result<()> {
    write_internal(result, writer, options, None, pagination)
}

/// Internal TOON writer handling headers, timing, pagination, and token budgeting
fn write_internal<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &ToonOptions,
    timing: Option<&str>,
    pagination: Option<&PaginationInfo>,
) -> Result<()> {
    let mut buffer = Vec::new();

    // 1. Build Metadata Comments
    let mut row_count_line = format!("# rows: {}", result.row_count);
    if let Some(t) = timing {
        row_count_line.push_str(&format!(" | elapsed: {}", t));
    }
    writeln!(&mut buffer, "{}", row_count_line)?;

    if let Some(pg) = pagination {
        writeln!(
            &mut buffer,
            "# page: {}/{} (total rows: {})",
            pg.page, pg.total_pages(), pg.total_rows
        )?;
    }

    // 2. Build Column Specification
    let col_names: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
    writeln!(&mut buffer, "[columns: {}]", col_names.join(", "))?;

    // 3. Render Data Rows with Dynamic Token Budgeting
    let mut current_tokens = token_budget::estimate_tokens(&String::from_utf8_lossy(&buffer));
    let mut included_rows = 0;
    let mut truncated = false;

    for row in &result.rows {
        let row_values: Vec<String> = row
            .iter()
            .map(|val| format_toon_cell(val, &options.token_opts))
            .collect();
        let row_line = format!("{}\n", row_values.join(", "));

        let row_tokens = token_budget::estimate_tokens(&row_line);

        if let Some(budget) = options.token_opts.token_budget {
            if current_tokens + row_tokens > budget && included_rows > 0 {
                truncated = true;
                break;
            }
        }

        buffer.extend_from_slice(row_line.as_bytes());
        current_tokens += row_tokens;
        included_rows += 1;
    }

    // 4. Append Truncation Notice if budget was reached
    if truncated {
        let trunc_msg = format!(
            "# Truncated: Showing {} of {} rows to fit {} token budget.\n",
            included_rows,
            result.row_count,
            options.token_opts.token_budget.unwrap()
        );
        buffer.extend_from_slice(trunc_msg.as_bytes());
        current_tokens = token_budget::estimate_tokens(&String::from_utf8_lossy(&buffer));
    }

    // 5. Prepend or append token count if requested
    if options.token_opts.show_tokens {
        writeln!(writer, "# tokens: ~{}", current_tokens)?;
    }

    writer.write_all(&buffer)?;
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
            ColumnMetadata::new("amount", TeradataType::Decimal, true),
        ];
        let rows = vec![
            vec![
                Value::Integer(1),
                Value::String("Alice".into()),
                Value::Decimal(149.50),
            ],
            vec![
                Value::Integer(2),
                Value::String("Bob, Jr.".into()), // contains comma
                Value::Null,
            ],
        ];
        QueryResult::new(columns, rows, Duration::from_millis(15))
    }

    #[test]
    fn test_toon_formatting_basic() {
        let res = make_test_result();
        let mut opts = ToonOptions::default();
        opts.token_opts.compress = true;

        let mut output = Vec::new();
        write(&res, &mut output, &opts).unwrap();
        let s = String::from_utf8(output).unwrap();

        assert!(s.contains("# rows: 2"));
        assert!(s.contains("[columns: id, name, amount]"));
        assert!(s.contains("1, Alice, 149.5"));
        // Bob, Jr. should be quoted because it contains comma
        assert!(s.contains("2, \"Bob, Jr.\", ~"));
    }

    #[test]
    fn test_toon_token_budget_truncation() {
        let res = make_test_result();
        let mut opts = ToonOptions::default();
        opts.token_opts.token_budget = Some(15); // Very small budget

        let mut output = Vec::new();
        write(&res, &mut output, &opts).unwrap();
        let s = String::from_utf8(output).unwrap();

        assert!(s.contains("# Truncated: Showing 1 of 2 rows to fit 15 token budget."));
    }

    #[test]
    fn test_toon_show_tokens() {
        let res = make_test_result();
        let mut opts = ToonOptions::default();
        opts.token_opts.show_tokens = true;

        let mut output = Vec::new();
        write(&res, &mut output, &opts).unwrap();
        let s = String::from_utf8(output).unwrap();

        assert!(s.contains("# tokens: ~"));
    }
}
