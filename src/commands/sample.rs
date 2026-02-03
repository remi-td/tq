//! Sample and Peek command implementations
//!
//! This module provides data exploration commands for quick table inspection:
//! - Sample: Random sampling using Teradata's SAMPLE clause
//! - Peek: First N rows plus column metadata using TOP clause
//!
//! Sprint 33: Initial implementation

use crate::cli::{OutputFormat, PeekArgs, SampleArgs};
use crate::db::{ColumnInfo, DatabaseClient, QueryResult};
use crate::error::{Result, TqError};
use std::io::Write;

/// Default sample size when not specified
pub const DEFAULT_SAMPLE_SIZE: usize = 10;

/// Maximum sample size to prevent runaway queries
pub const MAX_SAMPLE_SIZE: usize = 1000;

/// Default peek row count
pub const DEFAULT_PEEK_ROWS: usize = 5;

/// Execute the sample command
///
/// Retrieves a random sample of rows from the specified table using
/// Teradata's SAMPLE clause for efficient sampling.
///
/// # Arguments
/// * `client` - Database client
/// * `args` - Sample command arguments
/// * `writer` - Output writer
/// * `use_color` - Whether to use color output
pub fn execute_sample<W: Write>(
    client: &DatabaseClient,
    args: &SampleArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    // Validate and clamp sample size
    let sample_size = args.count.min(MAX_SAMPLE_SIZE);

    // Parse table name (may be qualified or unqualified)
    let (database, table_name) = parse_table_name(&args.table, client.config().database.as_str());

    // Build the sample query using SAMPLE clause
    let sql = format!(
        "SELECT * FROM {}.{} SAMPLE {}",
        database, table_name, sample_size
    );

    log::debug!("Executing sample query: {}", sql);

    // Execute query
    let result = client.execute(&sql).map_err(|e| {
        handle_sample_error(e, &args.table, "sample")
    })?;

    // Display results
    match args.format {
        OutputFormat::Table => display_table_result(&result, writer, sample_size, &args.table)?,
        OutputFormat::Csv => display_csv_result(&result, writer)?,
        OutputFormat::Json => display_json_result(&result, writer)?,
    }

    Ok(())
}

/// Execute the peek command
///
/// Displays the first N rows of a table along with column metadata.
/// Uses TOP clause for efficient row limiting.
///
/// # Arguments
/// * `client` - Database client
/// * `args` - Peek command arguments
/// * `writer` - Output writer
/// * `use_color` - Whether to use color output
pub fn execute_peek<W: Write>(
    client: &DatabaseClient,
    args: &PeekArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    // Parse table name
    let (database, table_name) = parse_table_name(&args.table, client.config().database.as_str());

    // First, get column metadata
    let columns = get_column_metadata(client, database, table_name)?;

    // Build query for first N rows
    let sql = format!(
        "SELECT TOP {} * FROM {}.{}",
        args.count, database, table_name
    );

    log::debug!("Executing peek query: {}", sql);

    // Execute query
    let result = client.execute(&sql).map_err(|e| {
        handle_sample_error(e, &args.table, "peek")
    })?;

    // Display results
    match args.format {
        OutputFormat::Table => display_peek_table(&columns, &result, writer, &args.table)?,
        OutputFormat::Csv => display_peek_csv(&columns, &result, writer)?,
        OutputFormat::Json => display_peek_json(&columns, &result, writer)?,
    }

    Ok(())
}

/// Parse table name into database and table components
///
/// Handles both qualified (database.table) and unqualified (table) names.
fn parse_table_name<'a>(table: &'a str, default_database: &'a str) -> (&'a str, &'a str) {
    if let Some(dot_pos) = table.find('.') {
        let database = &table[..dot_pos];
        let table_name = &table[dot_pos + 1..];
        (database, table_name)
    } else {
        (default_database, table)
    }
}

/// Get column metadata for a table
fn get_column_metadata(
    client: &DatabaseClient,
    database: &str,
    table_name: &str,
) -> Result<Vec<ColumnInfo>> {
    // Query column information from DBC.ColumnsV
    let sql = format!(
        r#"SELECT
            ColumnName,
            ColumnType,
            Nullable,
            ColumnLength,
            DecimalTotalDigits,
            DecimalFractionalDigits
        FROM DBC.ColumnsV
        WHERE DatabaseName = '{}'
          AND TableName = '{}'
        ORDER BY ColumnId"#,
        database.to_uppercase(),
        table_name.to_uppercase()
    );

    let result = client.execute(&sql)?;

    let columns = result
        .rows
        .iter()
        .map(|row| {
            let name = row
                .first()
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();
            let type_code = row
                .get(1)
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();
            let nullable = row
                .get(2)
                .map(|v| v.display().trim() == "Y")
                .unwrap_or(true);
            let length = row
                .get(3)
                .and_then(|v| v.display().parse::<i32>().ok());
            let precision = row
                .get(4)
                .and_then(|v| v.display().parse::<i32>().ok());
            let scale = row
                .get(5)
                .and_then(|v| v.display().parse::<i32>().ok());

            ColumnInfo {
                name,
                data_type: format_column_type(&type_code, length, precision, scale),
                nullable,
            }
        })
        .collect();

    Ok(columns)
}

/// Format column type from Teradata type code
fn format_column_type(
    type_code: &str,
    length: Option<i32>,
    precision: Option<i32>,
    scale: Option<i32>,
) -> String {
    match type_code.trim() {
        "CV" => format!("VARCHAR({})", length.unwrap_or(0)),
        "CF" => format!("CHAR({})", length.unwrap_or(0)),
        "I" => "INTEGER".to_string(),
        "I1" => "BYTEINT".to_string(),
        "I2" => "SMALLINT".to_string(),
        "I8" => "BIGINT".to_string(),
        "D" => {
            if let (Some(p), Some(s)) = (precision, scale) {
                format!("DECIMAL({},{})", p, s)
            } else {
                "DECIMAL".to_string()
            }
        }
        "F" => "FLOAT".to_string(),
        "DA" => "DATE".to_string(),
        "TS" => "TIMESTAMP".to_string(),
        "TZ" => "TIMESTAMP WITH TIME ZONE".to_string(),
        "AT" => "TIME".to_string(),
        "BV" => format!("VARBYTE({})", length.unwrap_or(0)),
        "BF" => format!("BYTE({})", length.unwrap_or(0)),
        "CO" => "CLOB".to_string(),
        "BO" => "BLOB".to_string(),
        "JN" => "JSON".to_string(),
        other => other.to_string(),
    }
}

/// Handle sample/peek query errors with user-friendly messages
fn handle_sample_error(error: TqError, table: &str, command: &str) -> TqError {
    let error_str = error.to_string().to_lowercase();

    if error_str.contains("3807") || error_str.contains("does not exist") {
        TqError::QueryExecution(format!(
            "Table '{}' not found.\n\n\
             Verify the table name and check your current database.\n\
             Use qualified names (database.table) for tables in other databases.",
            table
        ))
    } else if error_str.contains("3523") || error_str.contains("privilege") {
        TqError::QueryExecution(format!(
            "Insufficient privileges to {} table '{}'.\n\n\
             You need SELECT privilege on this table.",
            command, table
        ))
    } else {
        error
    }
}

/// Extract column names from QueryResult
fn get_column_names(result: &QueryResult) -> Vec<String> {
    result.columns.iter().map(|c| c.name.clone()).collect()
}

/// Display sample result as table
fn display_table_result<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    sample_size: usize,
    table: &str,
) -> Result<()> {
    use comfy_table::{presets, ContentArrangement, Table};

    let mut table_display = Table::new();
    table_display.load_preset(presets::UTF8_FULL);
    table_display.set_content_arrangement(ContentArrangement::Dynamic);

    // Set headers from column names
    let column_names = get_column_names(result);
    table_display.set_header(&column_names);

    // Add rows
    for row in &result.rows {
        let row_values: Vec<String> = row.iter().map(|v| v.display()).collect();
        table_display.add_row(row_values);
    }

    writeln!(writer)?;
    writeln!(writer, "Sample from {} ({} rows):", table, sample_size)?;
    writeln!(writer, "{}", table_display)?;
    writeln!(writer)?;
    writeln!(
        writer,
        "{} row(s) returned (Query time: {:.3}s)",
        result.rows.len(),
        result.execution_time.as_secs_f64()
    )?;

    Ok(())
}

/// Display sample result as CSV
fn display_csv_result<W: Write>(result: &QueryResult, writer: &mut W) -> Result<()> {
    // Write header
    let column_names = get_column_names(result);
    writeln!(writer, "{}", column_names.join(","))?;

    // Write rows
    for row in &result.rows {
        let row_values: Vec<String> = row.iter().map(|v| escape_csv(&v.display())).collect();
        writeln!(writer, "{}", row_values.join(","))?;
    }

    Ok(())
}

/// Display sample result as JSON
fn display_json_result<W: Write>(result: &QueryResult, writer: &mut W) -> Result<()> {
    let column_names = get_column_names(result);

    let rows: Vec<serde_json::Value> = result
        .rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, col_name) in column_names.iter().enumerate() {
                let value = row
                    .get(i)
                    .map(|v| {
                        use crate::db::Value;
                        match v {
                            Value::Null => serde_json::Value::Null,
                            Value::Integer(n) => serde_json::Value::Number((*n).into()),
                            Value::Decimal(n) => serde_json::Number::from_f64(*n)
                                .map(serde_json::Value::Number)
                                .unwrap_or(serde_json::Value::Null),
                            Value::Boolean(b) => serde_json::Value::Bool(*b),
                            _ => serde_json::Value::String(v.display()),
                        }
                    })
                    .unwrap_or(serde_json::Value::Null);
                obj.insert(col_name.clone(), value);
            }
            serde_json::Value::Object(obj)
        })
        .collect();

    let json_output = serde_json::to_string_pretty(&rows)?;
    writeln!(writer, "{}", json_output)?;

    Ok(())
}

/// Display peek result as table with column metadata
fn display_peek_table<W: Write>(
    columns: &[ColumnInfo],
    result: &QueryResult,
    writer: &mut W,
    table: &str,
) -> Result<()> {
    use comfy_table::{presets, Cell, CellAlignment, ContentArrangement, Table};

    // Display column metadata
    writeln!(writer)?;
    writeln!(writer, "Columns in {}:", table)?;

    let mut meta_table = Table::new();
    meta_table.load_preset(presets::UTF8_FULL);
    meta_table.set_content_arrangement(ContentArrangement::Dynamic);
    meta_table.set_header(vec!["Column", "Type", "Nullable"]);

    for col in columns {
        meta_table.add_row(vec![
            Cell::new(&col.name),
            Cell::new(&col.data_type),
            Cell::new(if col.nullable { "YES" } else { "NO" }).set_alignment(CellAlignment::Center),
        ]);
    }

    writeln!(writer, "{}", meta_table)?;
    writeln!(writer)?;

    // Display data preview
    if result.rows.is_empty() {
        writeln!(writer, "(table is empty)")?;
    } else {
        writeln!(writer, "First {} row(s):", result.rows.len())?;

        let mut data_table = Table::new();
        data_table.load_preset(presets::UTF8_FULL);
        data_table.set_content_arrangement(ContentArrangement::Dynamic);
        let column_names = get_column_names(result);
        data_table.set_header(&column_names);

        for row in &result.rows {
            let row_values: Vec<String> = row.iter().map(|v| v.display()).collect();
            data_table.add_row(row_values);
        }

        writeln!(writer, "{}", data_table)?;
    }

    writeln!(writer)?;
    writeln!(
        writer,
        "{} column(s), {} row(s) (Query time: {:.3}s)",
        columns.len(),
        result.rows.len(),
        result.execution_time.as_secs_f64()
    )?;

    Ok(())
}

/// Display peek result as CSV with column metadata section
fn display_peek_csv<W: Write>(
    columns: &[ColumnInfo],
    result: &QueryResult,
    writer: &mut W,
) -> Result<()> {
    // Column metadata section
    writeln!(writer, "# Columns")?;
    writeln!(writer, "Column,Type,Nullable")?;
    for col in columns {
        writeln!(
            writer,
            "{},{},{}",
            escape_csv(&col.name),
            escape_csv(&col.data_type),
            if col.nullable { "YES" } else { "NO" }
        )?;
    }

    writeln!(writer)?;
    writeln!(writer, "# Data")?;

    // Data section
    let column_names = get_column_names(result);
    writeln!(writer, "{}", column_names.join(","))?;
    for row in &result.rows {
        let row_values: Vec<String> = row.iter().map(|v| escape_csv(&v.display())).collect();
        writeln!(writer, "{}", row_values.join(","))?;
    }

    Ok(())
}

/// Display peek result as JSON with columns and data sections
fn display_peek_json<W: Write>(
    columns: &[ColumnInfo],
    result: &QueryResult,
    writer: &mut W,
) -> Result<()> {
    let columns_json: Vec<serde_json::Value> = columns
        .iter()
        .map(|col| {
            serde_json::json!({
                "name": col.name,
                "type": col.data_type,
                "nullable": col.nullable
            })
        })
        .collect();

    let column_names = get_column_names(result);
    let rows: Vec<serde_json::Value> = result
        .rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, col_name) in column_names.iter().enumerate() {
                let value = row
                    .get(i)
                    .map(|v| {
                        use crate::db::Value;
                        match v {
                            Value::Null => serde_json::Value::Null,
                            Value::Integer(n) => serde_json::Value::Number((*n).into()),
                            Value::Decimal(n) => serde_json::Number::from_f64(*n)
                                .map(serde_json::Value::Number)
                                .unwrap_or(serde_json::Value::Null),
                            Value::Boolean(b) => serde_json::Value::Bool(*b),
                            _ => serde_json::Value::String(v.display()),
                        }
                    })
                    .unwrap_or(serde_json::Value::Null);
                obj.insert(col_name.clone(), value);
            }
            serde_json::Value::Object(obj)
        })
        .collect();

    let output = serde_json::json!({
        "columns": columns_json,
        "rows": rows
    });

    let json_output = serde_json::to_string_pretty(&output)?;
    writeln!(writer, "{}", json_output)?;

    Ok(())
}

/// Escape a string for CSV output
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_table_name_unqualified() {
        let (db, table) = parse_table_name("employees", "default_db");
        assert_eq!(db, "default_db");
        assert_eq!(table, "employees");
    }

    #[test]
    fn test_parse_table_name_qualified() {
        let (db, table) = parse_table_name("hr_db.employees", "default_db");
        assert_eq!(db, "hr_db");
        assert_eq!(table, "employees");
    }

    #[test]
    fn test_format_column_type_varchar() {
        let result = format_column_type("CV", Some(100), None, None);
        assert_eq!(result, "VARCHAR(100)");
    }

    #[test]
    fn test_format_column_type_char() {
        let result = format_column_type("CF", Some(10), None, None);
        assert_eq!(result, "CHAR(10)");
    }

    #[test]
    fn test_format_column_type_integer() {
        let result = format_column_type("I", None, None, None);
        assert_eq!(result, "INTEGER");
    }

    #[test]
    fn test_format_column_type_decimal() {
        let result = format_column_type("D", None, Some(10), Some(2));
        assert_eq!(result, "DECIMAL(10,2)");
    }

    #[test]
    fn test_format_column_type_decimal_no_precision() {
        let result = format_column_type("D", None, None, None);
        assert_eq!(result, "DECIMAL");
    }

    #[test]
    fn test_format_column_type_date() {
        let result = format_column_type("DA", None, None, None);
        assert_eq!(result, "DATE");
    }

    #[test]
    fn test_format_column_type_timestamp() {
        let result = format_column_type("TS", None, None, None);
        assert_eq!(result, "TIMESTAMP");
    }

    #[test]
    fn test_format_column_type_unknown() {
        let result = format_column_type("XX", None, None, None);
        assert_eq!(result, "XX");
    }

    #[test]
    fn test_escape_csv_simple() {
        assert_eq!(escape_csv("hello"), "hello");
    }

    #[test]
    fn test_escape_csv_with_comma() {
        assert_eq!(escape_csv("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn test_escape_csv_with_quotes() {
        assert_eq!(escape_csv("say \"hello\""), "\"say \"\"hello\"\"\"");
    }

    #[test]
    fn test_escape_csv_with_newline() {
        assert_eq!(escape_csv("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_SAMPLE_SIZE, 10);
        assert_eq!(MAX_SAMPLE_SIZE, 1000);
        assert_eq!(DEFAULT_PEEK_ROWS, 5);
    }
}
