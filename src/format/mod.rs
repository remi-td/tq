//! Output formatting for query results
//!
//! This module provides formatters for different output types:
//! - `table`: Human-readable UTF8 tables with comfy-table
//! - `json`: JSON output with type preservation
//! - `csv`: RFC 4180 compliant CSV output

pub mod csv;
pub mod json;
pub mod table;

use crate::cli::OutputFormat;
use crate::db::QueryResult;
use crate::error::Result;
use std::io::Write;

/// Format options combining all format-specific options
#[derive(Debug, Clone, Default)]
pub struct FormatOptions {
    /// Table formatting options
    pub table: table::TableOptions,
    /// JSON formatting options
    pub json: json::JsonOptions,
    /// CSV formatting options
    pub csv: csv::CsvOptions,
}

impl FormatOptions {
    /// Create options with header display setting
    pub fn with_header(mut self, show_header: bool) -> Self {
        self.table.show_header = show_header;
        self.csv.show_header = show_header;
        self
    }

    /// Create options with color setting
    pub fn with_color(mut self, use_color: bool) -> Self {
        self.table.use_color = use_color;
        self
    }

    /// Create options with pretty print setting for JSON
    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.json.pretty = pretty;
        self
    }
}

/// Write query results in the specified format
pub fn write_output<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    format: OutputFormat,
    options: &FormatOptions,
) -> Result<()> {
    match format {
        OutputFormat::Table => table::write(result, writer, &options.table),
        OutputFormat::Json => json::write(result, writer, &options.json),
        OutputFormat::Csv => csv::write(result, writer, &options.csv),
    }
}

/// Write query results with timing information (table format only)
pub fn write_output_with_timing<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    format: OutputFormat,
    options: &FormatOptions,
    show_timing: bool,
) -> Result<()> {
    match format {
        OutputFormat::Table => {
            if show_timing {
                table::write_with_timing(result, writer, &options.table)
            } else {
                table::write(result, writer, &options.table)
            }
        }
        OutputFormat::Json => {
            if show_timing {
                json::write_with_metadata(result, writer, &options.json)
            } else {
                json::write(result, writer, &options.json)
            }
        }
        OutputFormat::Csv => {
            // CSV doesn't typically include timing, just write data
            csv::write(result, writer, &options.csv)
        }
    }
}

/// Helper to format to string
pub fn format_to_string(
    result: &QueryResult,
    format: OutputFormat,
    options: &FormatOptions,
) -> Result<String> {
    let mut buffer = Vec::new();
    write_output(result, &mut buffer, format, options)?;
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
        ];
        let rows = vec![
            vec![Value::Integer(1), Value::String("Alice".into())],
            vec![Value::Integer(2), Value::String("Bob".into())],
        ];
        QueryResult::new(columns, rows, Duration::from_millis(50))
    }

    #[test]
    fn test_format_to_string_table() {
        let result = create_test_result();
        let options = FormatOptions::default().with_color(false);
        let output = format_to_string(&result, OutputFormat::Table, &options).unwrap();

        assert!(output.contains("id"));
        assert!(output.contains("name"));
        assert!(output.contains("Alice"));
    }

    #[test]
    fn test_format_to_string_json() {
        let result = create_test_result();
        let options = FormatOptions::default();
        let output = format_to_string(&result, OutputFormat::Json, &options).unwrap();

        let parsed: Vec<serde_json::Value> = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["name"], "Alice");
    }

    #[test]
    fn test_format_to_string_csv() {
        let result = create_test_result();
        let options = FormatOptions::default();
        let output = format_to_string(&result, OutputFormat::Csv, &options).unwrap();

        assert!(output.contains("id,name"));
        assert!(output.contains("1,Alice"));
    }

    #[test]
    fn test_format_options_builder() {
        let options = FormatOptions::default()
            .with_header(false)
            .with_color(false)
            .with_pretty(false);

        assert!(!options.table.show_header);
        assert!(!options.csv.show_header);
        assert!(!options.table.use_color);
        assert!(!options.json.pretty);
    }
}
