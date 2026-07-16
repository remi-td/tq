//! Markdown output formatting
//!
//! Provides GitHub-Flavored Markdown (GFM) table output with:
//! - Pipe-delimited table format
//! - Type-based column alignment (left for text, right for numeric)
//! - Pipe character escaping in values
//! - Optional header row
//! - Metadata footer with row count and timing

use crate::db::{Alignment, QueryResult};
use crate::error::Result;
use std::io::Write;

/// Markdown formatting options
#[derive(Debug, Clone)]
pub struct MarkdownOptions {
    /// Include column headers
    pub show_header: bool,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self { show_header: true }
    }
}

/// Escape a value for use inside a markdown table cell.
///
/// The pipe character `|` must be escaped as `\|` to avoid breaking
/// the table structure. NULL values render as empty strings.
pub fn escape_value(value: &str) -> String {
    value.replace('|', "\\|")
}

/// Build the alignment separator row based on column types.
///
/// Returns a row like `|:---|---:|` where numeric columns are
/// right-aligned and all others are left-aligned.
fn alignment_separator(result: &QueryResult) -> String {
    let cells: Vec<String> = result
        .columns
        .iter()
        .map(|col| match col.data_type.alignment() {
            Alignment::Right => "---:".to_string(),
            Alignment::Center => ":---:".to_string(),
            Alignment::Left => ":---".to_string(),
        })
        .collect();
    format!("| {} |", cells.join(" | "))
}

/// Write query results as a GitHub-Flavored Markdown table
pub fn write<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &MarkdownOptions,
) -> Result<()> {
    if crate::format::is_show_query_result(result) {
        if let Some(row) = result.rows.first() {
            if let Some(val) = row.first() {
                let ddl = val.display();
                writeln!(writer, "```sql")?;
                write!(writer, "{}", ddl)?;
                if !ddl.ends_with('\n') {
                    writeln!(writer)?;
                }
                writeln!(writer, "```")?;
                return Ok(());
            }
        }
    }

    // Write header
    if options.show_header && !result.columns.is_empty() {
        let headers: Vec<String> = result
            .columns
            .iter()
            .map(|c| escape_value(&c.name))
            .collect();
        writeln!(writer, "| {} |", headers.join(" | "))?;
        writeln!(writer, "{}", alignment_separator(result))?;
    }

    // Write data rows
    for row in &result.rows {
        let cells: Vec<String> = row.iter().map(|v| escape_value(&v.to_csv_string())).collect();
        writeln!(writer, "| {} |", cells.join(" | "))?;
    }

    Ok(())
}

/// Write query results as markdown with a metadata footer line.
///
/// Appends a line after the table showing row count and execution time:
/// `_N row(s), 0.050s_`
pub fn write_with_metadata<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &MarkdownOptions,
) -> Result<()> {
    write(result, writer, options)?;

    writeln!(writer)?;
    writeln!(
        writer,
        "_{} row(s), {:.3}s_",
        result.row_count,
        result.execution_time.as_secs_f64()
    )?;

    Ok(())
}

/// Format results as a markdown string
pub fn format_string(result: &QueryResult, options: &MarkdownOptions) -> Result<String> {
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
        QueryResult::new(columns, rows, Duration::from_millis(50))
    }

    #[test]
    fn test_write_markdown() {
        let result = create_test_result();
        let options = MarkdownOptions::default();

        let output = format_string(&result, &options).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        assert_eq!(lines.len(), 4); // header + separator + 2 data rows
        assert_eq!(lines[0], "| id | name | score |");
        assert_eq!(lines[1], "| ---: | :--- | ---: |");
        assert_eq!(lines[2], "| 1 | Alice | 95.5 |");
        assert_eq!(lines[3], "| 2 | Bob | 87.3 |");
    }

    #[test]
    fn test_write_markdown_no_header() {
        let result = create_test_result();
        let options = MarkdownOptions { show_header: false };

        let output = format_string(&result, &options).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        assert_eq!(lines.len(), 2); // data rows only
        assert_eq!(lines[0], "| 1 | Alice | 95.5 |");
        assert_eq!(lines[1], "| 2 | Bob | 87.3 |");
    }

    #[test]
    fn test_write_with_metadata() {
        let result = create_test_result();
        let options = MarkdownOptions::default();

        let mut buffer = Vec::new();
        write_with_metadata(&result, &mut buffer, &options).unwrap();
        let output = String::from_utf8_lossy(&buffer);
        let lines: Vec<&str> = output.trim().lines().collect();

        // Last line should be the metadata footer
        let last = lines.last().unwrap();
        assert_eq!(*last, "_2 row(s), 0.050s_");
    }

    #[test]
    fn test_markdown_null_values() {
        let columns = vec![
            ColumnMetadata::new("id", TeradataType::Integer, false),
            ColumnMetadata::new("name", TeradataType::Varchar, true),
        ];
        let rows = vec![vec![Value::Integer(1), Value::Null]];
        let result = QueryResult::new(columns, rows, Duration::ZERO);

        let output = format_string(&result, &MarkdownOptions::default()).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        // NULL renders as empty string (like CSV)
        assert_eq!(lines[2], "| 1 |  |");
    }

    #[test]
    fn test_markdown_special_characters() {
        let columns = vec![ColumnMetadata::new("data", TeradataType::Varchar, false)];
        let rows = vec![
            vec![Value::String("normal".into())],
            vec![Value::String("has | pipe".into())],
            vec![Value::String("a|b|c".into())],
        ];
        let result = QueryResult::new(columns, rows, Duration::ZERO);

        let output = format_string(&result, &MarkdownOptions::default()).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        assert_eq!(lines[2], "| normal |");
        assert_eq!(lines[3], "| has \\| pipe |");
        assert_eq!(lines[4], "| a\\|b\\|c |");
    }

    #[test]
    fn test_markdown_empty_result() {
        let columns = vec![
            ColumnMetadata::new("id", TeradataType::Integer, false),
            ColumnMetadata::new("name", TeradataType::Varchar, true),
        ];
        let rows: Vec<Vec<Value>> = vec![];
        let result = QueryResult::new(columns, rows, Duration::ZERO);

        let output = format_string(&result, &MarkdownOptions::default()).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        // Header and separator only, no data rows
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "| id | name |");
        assert_eq!(lines[1], "| ---: | :--- |");
    }

    #[test]
    fn test_type_alignment() {
        let columns = vec![
            ColumnMetadata::new("int_col", TeradataType::Integer, false),
            ColumnMetadata::new("str_col", TeradataType::Varchar, false),
            ColumnMetadata::new("dec_col", TeradataType::Decimal, false),
            ColumnMetadata::new("bool_col", TeradataType::Boolean, false),
            ColumnMetadata::new("date_col", TeradataType::Date, false),
        ];
        let rows: Vec<Vec<Value>> = vec![];
        let result = QueryResult::new(columns, rows, Duration::ZERO);

        let output = format_string(&result, &MarkdownOptions::default()).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        // Separator line encodes alignment
        let sep = lines[1];
        // Integer -> right, Varchar -> left, Decimal -> right, Boolean -> center, Date -> left
        assert_eq!(sep, "| ---: | :--- | ---: | :---: | :--- |");
    }

    #[test]
    fn test_write_markdown_show_query() {
        use crate::db::{ColumnMetadata, TeradataType, Value, QueryResult};
        use std::time::Duration;

        let columns = vec![ColumnMetadata::new("Request Text", TeradataType::Varchar, false)];
        let ddl = "CREATE SET TABLE db.table ,\n     NO FLASHBACK\n     (id INTEGER);";
        let rows = vec![vec![Value::String(ddl.to_string())]];
        let result = QueryResult::new(columns, rows, Duration::ZERO);

        let options = MarkdownOptions::default();
        let mut buffer = Vec::new();
        write(&result, &mut buffer, &options).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Should output the DDL wrapped in sql block
        let expected = format!("```sql\n{}\n```\n", ddl);
        assert_eq!(output, expected);
    }
}

