//! Describe command implementation
//!
//! Shows table structure: column names, types, nullable flags, and defaults.
//! Used by both `tq describe <table>` (batch) and `/describe` (REPL delegation).

use crate::cli::OutputFormat;
use crate::db::DatabaseClient;
use crate::error::Result;
use crate::sql::escape_sql_string;
use std::io::Write;

// =============================================================================
// Public API
// =============================================================================

/// Execute `tq describe` in batch mode with format selection
pub fn execute<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    format: OutputFormat,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    match format {
        OutputFormat::Table => describe_table(client, table_name, writer),
        OutputFormat::Json => describe_json(client, table_name, writer),
        OutputFormat::Csv => describe_csv(client, table_name, writer),
    }
}

/// Execute /describe in REPL mode (delegates to table format with extra spacing)
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;
    describe_table(client, table_name, writer)?;
    writeln!(writer)?;
    Ok(())
}

// =============================================================================
// Internal helpers
// =============================================================================

/// Column information from DBC.ColumnsV
struct ColumnRow {
    name: String,
    col_type: String,
    nullable: String,
    default: String,
}

/// Parse qualified table name into (optional database, table)
fn parse_table_name(name: &str) -> (Option<&str>, &str) {
    if let Some(dot_pos) = name.find('.') {
        (Some(&name[..dot_pos]), &name[dot_pos + 1..])
    } else {
        (None, name)
    }
}

/// Query DBC.ColumnsV for column metadata
fn query_columns(
    client: &DatabaseClient,
    table_name: &str,
) -> Result<(Vec<ColumnRow>, String)> {
    let (database, table) = parse_table_name(table_name);

    let sql = if let Some(db) = database {
        format!(
            "SELECT TRIM(ColumnName), ColumnType, Nullable, DefaultValue \
             FROM DBC.ColumnsV \
             WHERE DatabaseName = '{}' AND TableName = '{}' \
             ORDER BY ColumnId",
            escape_sql_string(db),
            escape_sql_string(table)
        )
    } else {
        format!(
            "SELECT TRIM(ColumnName), ColumnType, Nullable, DefaultValue \
             FROM DBC.ColumnsV \
             WHERE TableName = '{}' AND DatabaseName = DATABASE \
             ORDER BY ColumnId",
            escape_sql_string(table)
        )
    };

    let result = client.execute(&sql)?;

    let qualified = if let Some(db) = database {
        format!("{}.{}", db, table)
    } else {
        table.to_string()
    };

    let columns: Vec<ColumnRow> = result
        .rows
        .iter()
        .map(|row| {
            let name = row
                .first()
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();
            let col_type = row
                .get(1)
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();
            let nullable = row
                .get(2)
                .map(|v| format_nullable(&v.display()))
                .unwrap_or_else(|| "YES".to_string());
            let default = row
                .get(3)
                .map(|v| {
                    let s = v.display();
                    if s == "[NULL]" {
                        "-".to_string()
                    } else {
                        s.trim().to_string()
                    }
                })
                .unwrap_or_else(|| "-".to_string());

            ColumnRow {
                name,
                col_type,
                nullable,
                default,
            }
        })
        .collect();

    Ok((columns, qualified))
}

/// Format nullable indicator consistently
fn format_nullable(s: &str) -> String {
    match s.trim().to_uppercase().as_str() {
        "Y" | "YES" | "TRUE" | "1" => "YES".to_string(),
        "N" | "NO" | "FALSE" | "0" => "NO".to_string(),
        _ => s.to_string(),
    }
}

/// Truncate string to max length with ellipsis
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        ".".repeat(max_len)
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

// =============================================================================
// Output formats
// =============================================================================

fn describe_table<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (columns, qualified) = query_columns(client, table_name)?;

    if columns.is_empty() {
        writeln!(
            writer,
            "Table '{}' not found or no columns available.",
            table_name
        )?;
        writeln!(writer)?;
        writeln!(writer, "Suggestions:")?;
        writeln!(writer, "  - Check the table name spelling")?;
        writeln!(writer, "  - Try using qualified name: describe database.table")?;
        writeln!(
            writer,
            "  - Verify you have SELECT permission on DBC.ColumnsV"
        )?;
        return Ok(());
    }

    writeln!(writer, "Table: {}", qualified)?;
    writeln!(writer)?;
    writeln!(writer, "Columns:")?;
    writeln!(
        writer,
        "{:<25} {:<20} {:<10} {:<15}",
        "Column", "Type", "Nullable", "Default"
    )?;
    writeln!(writer, "{}", "-".repeat(70))?;

    for col in &columns {
        writeln!(
            writer,
            "{:<25} {:<20} {:<10} {:<15}",
            truncate_str(&col.name, 24),
            truncate_str(&col.col_type, 19),
            &col.nullable,
            truncate_str(&col.default, 14)
        )?;
    }

    writeln!(writer)?;
    writeln!(writer, "{} column(s)", columns.len())?;
    Ok(())
}

fn describe_json<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (columns, _qualified) = query_columns(client, table_name)?;

    if columns.is_empty() {
        writeln!(
            writer,
            "{{\"error\": \"Table '{}' not found\"}}",
            json_escape(table_name)
        )?;
        return Ok(());
    }

    write!(writer, "[")?;
    for (i, col) in columns.iter().enumerate() {
        if i > 0 {
            write!(writer, ",")?;
        }
        write!(
            writer,
            "{{\"name\":\"{}\",\"type\":\"{}\",\"nullable\":\"{}\",\"default\":\"{}\"}}",
            json_escape(&col.name),
            json_escape(&col.col_type),
            json_escape(&col.nullable),
            json_escape(&col.default)
        )?;
    }
    writeln!(writer, "]")?;
    Ok(())
}

fn describe_csv<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (columns, _qualified) = query_columns(client, table_name)?;

    writeln!(writer, "Column,Type,Nullable,Default")?;
    for col in &columns {
        writeln!(
            writer,
            "{},{},{},{}",
            csv_escape(&col.name),
            csv_escape(&col.col_type),
            csv_escape(&col.nullable),
            csv_escape(&col.default)
        )?;
    }
    Ok(())
}

/// Escape a string for JSON output
fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Escape a string for CSV output
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
