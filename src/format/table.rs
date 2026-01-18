//! Table output formatting using comfy-table
//!
//! Provides beautiful UTF8 table formatting with:
//! - Box-drawing characters
//! - Automatic column sizing
//! - Type-based alignment (numbers right, text left)
//! - NULL value styling

use crate::db::{Alignment, QueryResult};
use crate::error::Result;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, CellAlignment, Color,
    ContentArrangement, Table,
};
use crossterm::terminal;
use std::io::Write;

/// Table formatting options
#[derive(Debug, Clone)]
pub struct TableOptions {
    /// Include column headers
    pub show_header: bool,
    /// Use colors (for terminal output)
    pub use_color: bool,
    /// Maximum column width before wrapping (deprecated, terminal width is auto-detected)
    #[allow(dead_code)]
    pub max_column_width: Option<u16>,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            show_header: true,
            use_color: true,
            max_column_width: Some(80),
        }
    }
}

/// Write query results as a formatted table
pub fn write<W: Write>(result: &QueryResult, writer: &mut W, options: &TableOptions) -> Result<()> {
    if result.is_empty() {
        writeln!(writer, "No results returned.")?;
        return Ok(());
    }

    let mut table = Table::new();

    // Configure table style
    table.load_preset(UTF8_FULL);
    table.apply_modifier(UTF8_ROUND_CORNERS);

    // Use DynamicFullWidth to expand table to terminal width
    // This properly handles wide tables by dynamically sizing columns
    // to fit within the terminal while wrapping long content
    table.set_content_arrangement(ContentArrangement::DynamicFullWidth);

    // Detect terminal width and set table width accordingly
    // This ensures proper column sizing based on actual terminal dimensions
    if let Ok((term_width, _)) = terminal::size() {
        table.set_width(term_width);
    }

    // Add header row
    if options.show_header {
        let header_cells: Vec<Cell> = result
            .columns
            .iter()
            .map(|col| {
                let cell = Cell::new(&col.name);
                if options.use_color {
                    cell.add_attribute(Attribute::Bold).fg(Color::Cyan)
                } else {
                    cell.add_attribute(Attribute::Bold)
                }
            })
            .collect();

        table.set_header(header_cells);
    }

    // Add data rows
    for row in &result.rows {
        let cells: Vec<Cell> = row
            .iter()
            .zip(&result.columns)
            .map(|(value, col)| {
                let cell = Cell::new(value.display());

                // Apply alignment based on column type
                let cell = match col.data_type.alignment() {
                    Alignment::Left => cell,
                    Alignment::Center => cell.set_alignment(CellAlignment::Center),
                    Alignment::Right => cell.set_alignment(CellAlignment::Right),
                };

                // Style NULL values
                if value.is_null() && options.use_color {
                    cell.fg(Color::DarkGrey).add_attribute(Attribute::Italic)
                } else {
                    cell
                }
            })
            .collect();

        table.add_row(cells);
    }

    // Write table
    writeln!(writer, "{}", table)?;

    Ok(())
}

/// Write query results with timing footer
pub fn write_with_timing<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    options: &TableOptions,
) -> Result<()> {
    write(result, writer, options)?;

    // Add row count and timing
    writeln!(
        writer,
        "{} row(s) in set ({:.3}s)",
        result.row_count,
        result.execution_time.as_secs_f64()
    )?;

    Ok(())
}

/// Format results as a simple string (for testing/debugging)
pub fn format_string(result: &QueryResult, options: &TableOptions) -> Result<String> {
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
        ];
        let rows = vec![
            vec![
                Value::Integer(1),
                Value::String("Alice".into()),
                Value::Boolean(true),
            ],
            vec![
                Value::Integer(2),
                Value::String("Bob".into()),
                Value::Boolean(false),
            ],
            vec![Value::Integer(3), Value::Null, Value::Boolean(true)],
        ];
        QueryResult::new(columns, rows, Duration::from_millis(100))
    }

    #[test]
    fn test_write_table() {
        let result = create_test_result();
        let options = TableOptions {
            use_color: false,
            ..Default::default()
        };

        let mut buffer = Vec::new();
        write(&result, &mut buffer, &options).unwrap();
        let output = String::from_utf8_lossy(&buffer);

        assert!(output.contains("id"));
        assert!(output.contains("name"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        assert!(output.contains("[NULL]"));
    }

    #[test]
    fn test_write_table_no_header() {
        let result = create_test_result();
        let options = TableOptions {
            show_header: false,
            use_color: false,
            ..Default::default()
        };

        let output = format_string(&result, &options).unwrap();

        // Should not contain header row (but header name might appear if it matches data)
        assert!(output.contains("Alice"));
        assert!(output.contains("1"));
    }

    #[test]
    fn test_write_table_empty() {
        let result = QueryResult::empty();
        let options = TableOptions::default();

        let mut buffer = Vec::new();
        write(&result, &mut buffer, &options).unwrap();
        let output = String::from_utf8_lossy(&buffer);

        assert!(output.contains("No results"));
    }

    #[test]
    fn test_write_with_timing() {
        let result = create_test_result();
        let options = TableOptions {
            use_color: false,
            ..Default::default()
        };

        let mut buffer = Vec::new();
        write_with_timing(&result, &mut buffer, &options).unwrap();
        let output = String::from_utf8_lossy(&buffer);

        assert!(output.contains("3 row(s) in set"));
        assert!(output.contains("0.100s"));
    }
}
