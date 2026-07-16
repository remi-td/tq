//! Output formatting for query results
//!
//! This module provides formatters for different output types:
//! - `table`: Human-readable UTF8 tables with comfy-table
//! - `json`: JSON output with type preservation
//! - `csv`: RFC 4180 compliant CSV output

pub mod csv;
pub mod json;
pub mod markdown;
pub mod table;

use crate::cli::OutputFormat;
use crate::db::QueryResult;
use crate::error::Result;
use crate::pagination::PaginationInfo;
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
    /// Markdown formatting options
    pub markdown: markdown::MarkdownOptions,
}

impl FormatOptions {
    /// Create options with header display setting
    pub fn with_header(mut self, show_header: bool) -> Self {
        self.table.show_header = show_header;
        self.csv.show_header = show_header;
        self.markdown.show_header = show_header;
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

/// Check if the query result represents a Teradata `SHOW` command.
///
/// Teradata `SHOW` statements return exactly 1 column named `"Request Text"`
/// (or `"RequestText"`) and 1 row of DDL.
pub fn is_show_query_result(result: &QueryResult) -> bool {
    if result.columns.len() == 1 {
        let col_name = result.columns[0].name.to_lowercase();
        if col_name == "request text" || col_name == "requesttext" {
            return true;
        }
    }
    false
}

/// Write query results in the specified format
pub fn write_output<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    format: OutputFormat,
    options: &FormatOptions,
) -> Result<()> {
    match format.canonical() {
        OutputFormat::Table => table::write(result, writer, &options.table),
        OutputFormat::Json => json::write(result, writer, &options.json),
        OutputFormat::Csv => csv::write(result, writer, &options.csv),
        OutputFormat::Markdown => markdown::write(result, writer, &options.markdown),
        _ => unreachable!(),
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
    match format.canonical() {
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
        OutputFormat::Csv => csv::write(result, writer, &options.csv),
        OutputFormat::Markdown => {
            if show_timing {
                markdown::write_with_metadata(result, writer, &options.markdown)
            } else {
                markdown::write(result, writer, &options.markdown)
            }
        }
        _ => unreachable!(),
    }
}

/// Write query results with optional pagination metadata
///
/// When pagination is provided:
/// - JSON format includes a "pagination" object in the envelope
/// - Other formats append a "Page X of Y (N total rows)" footer
pub fn write_output_with_pagination<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    format: OutputFormat,
    options: &FormatOptions,
    show_timing: bool,
    pagination: Option<&PaginationInfo>,
) -> Result<()> {
    match format.canonical() {
        OutputFormat::Json => {
            if show_timing {
                // With timing, use metadata writer; add pagination separately
                json::write_with_metadata_and_pagination(result, writer, &options.json, pagination)?;
            } else {
                json::write_with_pagination(result, writer, &options.json, pagination)?;
            }
        }
        _ => {
            // Render using standard formatters
            write_output_with_timing(result, writer, format, options, show_timing)?;
            // Append pagination footer for non-JSON formats
            if let Some(pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
    }
    Ok(())
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

        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed["ok"], true);
        assert_eq!(parsed["row_count"], 2);
        let data = parsed["data"].as_array().unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0]["name"], "Alice");
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
        assert!(!options.markdown.show_header);
        assert!(!options.table.use_color);
        assert!(!options.json.pretty);
    }

    #[test]
    fn test_format_to_string_markdown() {
        let result = create_test_result();
        let options = FormatOptions::default();
        let output = format_to_string(&result, OutputFormat::Markdown, &options).unwrap();

        assert!(output.contains("| id | name |"));
        assert!(output.contains("| 1 | Alice |"));
    }

    #[test]
    fn test_format_md_alias() {
        let result = create_test_result();
        let options = FormatOptions::default();
        let output = format_to_string(&result, OutputFormat::Md, &options).unwrap();

        assert!(output.contains("| id | name |"));
        assert!(output.contains("| 2 | Bob |"));
    }
}
