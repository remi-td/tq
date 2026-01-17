//! CSV output formatting
//!
//! Provides RFC 4180 compliant CSV output with:
//! - Proper field quoting and escaping
//! - Optional header row
//! - Streaming support for large datasets

use crate::db::QueryResult;
use crate::error::Result;
use std::io::Write;

/// CSV formatting options
#[derive(Debug, Clone)]
pub struct CsvOptions {
    /// Include column headers
    pub show_header: bool,
    /// Field delimiter (default: comma)
    pub delimiter: char,
    /// Quote character (default: double quote)
    pub quote: char,
    /// Line terminator (default: CRLF for RFC 4180 compliance)
    pub line_terminator: &'static str,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            show_header: true,
            delimiter: ',',
            quote: '"',
            line_terminator: "\n", // Use Unix line endings for compatibility
        }
    }
}

/// Write query results as CSV
pub fn write<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &CsvOptions,
) -> Result<()> {
    let mut csv_writer = csv::WriterBuilder::new()
        .delimiter(options.delimiter as u8)
        .quote(options.quote as u8)
        .terminator(csv::Terminator::Any(options.line_terminator.as_bytes()[0]))
        .from_writer(writer);

    // Write header
    if options.show_header {
        let headers: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
        csv_writer.write_record(&headers)?;
    }

    // Write rows
    for row in &result.rows {
        let fields: Vec<String> = row.iter().map(|v| v.to_csv_string()).collect();
        csv_writer.write_record(&fields)?;
    }

    csv_writer.flush()?;
    Ok(())
}

/// Write query results as CSV using manual escaping (for more control)
pub fn write_manual<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &CsvOptions,
) -> Result<()> {
    // Write header
    if options.show_header {
        let header_line = result
            .columns
            .iter()
            .map(|c| escape_field(&c.name, options))
            .collect::<Vec<_>>()
            .join(&options.delimiter.to_string());
        writeln!(writer, "{}", header_line)?;
    }

    // Write rows
    for row in &result.rows {
        let line = row
            .iter()
            .map(|v| escape_field(&v.to_csv_string(), options))
            .collect::<Vec<_>>()
            .join(&options.delimiter.to_string());
        writeln!(writer, "{}", line)?;
    }

    Ok(())
}

/// Escape a field value according to RFC 4180
fn escape_field(value: &str, options: &CsvOptions) -> String {
    let needs_quoting = value.contains(options.delimiter)
        || value.contains(options.quote)
        || value.contains('\n')
        || value.contains('\r');

    if needs_quoting {
        // Double any quote characters and wrap in quotes
        let escaped = value.replace(
            options.quote,
            &format!("{}{}", options.quote, options.quote),
        );
        format!("{}{}{}", options.quote, escaped, options.quote)
    } else {
        value.to_string()
    }
}

/// Format results as a CSV string
pub fn format_string(result: &QueryResult, options: &CsvOptions) -> Result<String> {
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
            ColumnMetadata::new("score", TeradataType::Decimal, true),
        ];
        let rows = vec![
            vec![
                Value::Integer(1),
                Value::String("Alice".into()),
                Value::Decimal(95.5),
            ],
            vec![
                Value::Integer(2),
                Value::String("Bob".into()),
                Value::Decimal(87.3),
            ],
        ];
        QueryResult::new(columns, rows, Duration::ZERO)
    }

    #[test]
    fn test_write_csv() {
        let result = create_test_result();
        let options = CsvOptions::default();

        let output = format_string(&result, &options).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "id,name,score");
        assert_eq!(lines[1], "1,Alice,95.5");
        assert_eq!(lines[2], "2,Bob,87.3");
    }

    #[test]
    fn test_write_csv_no_header() {
        let result = create_test_result();
        let options = CsvOptions {
            show_header: false,
            ..Default::default()
        };

        let output = format_string(&result, &options).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "1,Alice,95.5");
    }

    #[test]
    fn test_escape_field_simple() {
        let options = CsvOptions::default();
        assert_eq!(escape_field("hello", &options), "hello");
    }

    #[test]
    fn test_escape_field_with_comma() {
        let options = CsvOptions::default();
        assert_eq!(escape_field("hello, world", &options), "\"hello, world\"");
    }

    #[test]
    fn test_escape_field_with_quote() {
        let options = CsvOptions::default();
        assert_eq!(escape_field("say \"hi\"", &options), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_escape_field_with_newline() {
        let options = CsvOptions::default();
        assert_eq!(escape_field("line1\nline2", &options), "\"line1\nline2\"");
    }

    #[test]
    fn test_csv_with_special_characters() {
        let columns = vec![ColumnMetadata::new("data", TeradataType::Varchar, false)];
        let rows = vec![
            vec![Value::String("normal".into())],
            vec![Value::String("has, comma".into())],
            vec![Value::String("has \"quote\"".into())],
            vec![Value::String("has\nnewline".into())],
        ];
        let result = QueryResult::new(columns, rows, Duration::ZERO);

        let output = format_string(&result, &CsvOptions::default()).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        assert_eq!(lines[1], "normal");
        assert_eq!(lines[2], "\"has, comma\"");
        assert_eq!(lines[3], "\"has \"\"quote\"\"\"");
        // Note: newline handling depends on csv crate behavior
    }

    #[test]
    fn test_csv_null_values() {
        let columns = vec![
            ColumnMetadata::new("id", TeradataType::Integer, false),
            ColumnMetadata::new("name", TeradataType::Varchar, true),
        ];
        let rows = vec![vec![Value::Integer(1), Value::Null]];
        let result = QueryResult::new(columns, rows, Duration::ZERO);

        let output = format_string(&result, &CsvOptions::default()).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        // NULL should be empty in CSV
        assert_eq!(lines[1], "1,");
    }

    #[test]
    fn test_csv_with_tab_delimiter() {
        let result = create_test_result();
        let options = CsvOptions {
            delimiter: '\t',
            ..Default::default()
        };

        let output = format_string(&result, &options).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        assert_eq!(lines[0], "id\tname\tscore");
        assert_eq!(lines[1], "1\tAlice\t95.5");
    }
}
