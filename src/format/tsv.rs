//! TSV (Tab-Separated Values) output formatting
//!
//! TSV offers the single lowest token consumption for pure tabular data sets
//! due to BPE tokenizer mechanics where tab transitions merge efficiently
//! with adjacent alphanumerics without quotes or borders.

use crate::db::{QueryResult, Value};
use crate::error::Result;
use crate::format::token_budget::{self, TokenOptions};
use std::io::Write;

/// Options for TSV formatting
#[derive(Debug, Clone)]
pub struct TsvOptions {
    /// Whether to display header row
    pub show_header: bool,
    /// Token compression options
    pub token_opts: TokenOptions,
}

impl Default for TsvOptions {
    fn default() -> Self {
        Self {
            show_header: true,
            token_opts: TokenOptions::default(),
        }
    }
}

impl TsvOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Escape cell text for TSV output: replace tabs and newlines with spaces or escape sequences
pub fn escape_tsv_cell(value: &Value, token_opts: &TokenOptions) -> String {
    match value {
        Value::Null => {
            if token_opts.compress {
                "~".to_string()
            } else {
                "".to_string()
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
            text.replace('\t', " ")
                .replace('\r', "")
                .replace('\n', "\\n")
        }
        Value::Bytes(bytes) => {
            let hex: String = bytes.iter().map(|b| format!("{:02X}", b)).collect();
            format!("0x{}", hex)
        }
    }
}

/// Write query results as TSV format
pub fn write<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &TsvOptions,
) -> Result<()> {
    write_internal(result, writer, options, None)
}

/// Write query results with timing in TSV format
pub fn write_with_timing<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &TsvOptions,
) -> Result<()> {
    let timing_str = format!("{:.3}s", result.execution_time.as_secs_f64());
    write_internal(result, writer, options, Some(&timing_str))
}

/// Internal TSV writer
fn write_internal<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &TsvOptions,
    timing: Option<&str>,
) -> Result<()> {
    let mut buffer = Vec::new();

    if let Some(t) = timing {
        writeln!(&mut buffer, "# elapsed: {}", t)?;
    }

    // Header row
    if options.show_header {
        let headers: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
        writeln!(&mut buffer, "{}", headers.join("\t"))?;
    }

    let mut current_tokens = token_budget::estimate_tokens(&String::from_utf8_lossy(&buffer));
    let mut included_rows = 0;
    let mut truncated = false;

    for row in &result.rows {
        let row_values: Vec<String> = row
            .iter()
            .map(|val| escape_tsv_cell(val, &options.token_opts))
            .collect();
        let row_line = format!("{}\n", row_values.join("\t"));
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
        ];
        let rows = vec![
            vec![Value::Integer(1), Value::String("Alice".into())],
            vec![Value::Integer(2), Value::String("Bob with\ttab".into())],
        ];
        QueryResult::new(columns, rows, Duration::from_millis(15))
    }

    #[test]
    fn test_tsv_formatting() {
        let res = make_test_result();
        let mut opts = TsvOptions::new();
        opts.token_opts.compress = true;

        let mut output = Vec::new();
        write(&res, &mut output, &opts).unwrap();
        let s = String::from_utf8(output).unwrap();

        assert!(s.contains("id\tname"));
        assert!(s.contains("1\tAlice"));
        assert!(s.contains("2\tBob with tab")); // tab was replaced with space
    }
}
