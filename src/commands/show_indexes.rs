//! Show-indexes command implementation
//!
//! Displays index information for a table from DBC.IndicesV.
//! Used by `tq show-indexes <table>` (batch) and `/show indexes` (REPL delegation).

use crate::cli::OutputFormat;
use crate::db::DatabaseClient;
use crate::error::Result;
use crate::sql::escape_sql_string;
use std::io::Write;

// =============================================================================
// Public API
// =============================================================================

/// Execute `tq show-indexes` in batch mode with format selection
pub fn execute<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    format: OutputFormat,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    match format {
        OutputFormat::Table => show_indexes_table(client, table_name, writer),
        OutputFormat::Json => show_indexes_json(client, table_name, writer),
        OutputFormat::Csv => show_indexes_csv(client, table_name, writer),
    }
}

/// Execute /show indexes in REPL mode (delegates to table format with spacing)
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;
    show_indexes_table(client, table_name, writer)?;
    writeln!(writer)?;
    Ok(())
}

// =============================================================================
// Data structures
// =============================================================================

struct IndexRow {
    index_name: String,
    index_type: String,
    column_name: String,
    position: String,
}

// =============================================================================
// Query helper
// =============================================================================

fn parse_table_name(name: &str) -> (Option<&str>, &str) {
    if let Some(dot_pos) = name.find('.') {
        (Some(&name[..dot_pos]), &name[dot_pos + 1..])
    } else {
        (None, name)
    }
}

fn query_indexes(
    client: &DatabaseClient,
    table_name: &str,
) -> Result<(Vec<IndexRow>, String)> {
    let (database, table) = parse_table_name(table_name);

    let sql = if let Some(db) = database {
        format!(
            "SELECT TRIM(IndexName) AS IndexName, \
             CASE IndexType \
                 WHEN 'P' THEN 'Primary' \
                 WHEN 'S' THEN 'Secondary' \
                 WHEN 'Q' THEN 'PPI' \
                 WHEN 'J' THEN 'Join' \
                 WHEN 'K' THEN 'Primary Key' \
                 WHEN 'U' THEN 'Unique' \
                 WHEN 'V' THEN 'Value-Ordered' \
                 WHEN 'H' THEN 'Hash' \
                 ELSE IndexType \
             END AS IndexType, \
             TRIM(ColumnName) AS ColumnName, \
             ColumnPosition \
             FROM DBC.IndicesV \
             WHERE DatabaseName = '{}' AND TableName = '{}' \
             ORDER BY IndexNumber, ColumnPosition",
            escape_sql_string(db),
            escape_sql_string(table)
        )
    } else {
        format!(
            "SELECT TRIM(IndexName) AS IndexName, \
             CASE IndexType \
                 WHEN 'P' THEN 'Primary' \
                 WHEN 'S' THEN 'Secondary' \
                 WHEN 'Q' THEN 'PPI' \
                 WHEN 'J' THEN 'Join' \
                 WHEN 'K' THEN 'Primary Key' \
                 WHEN 'U' THEN 'Unique' \
                 WHEN 'V' THEN 'Value-Ordered' \
                 WHEN 'H' THEN 'Hash' \
                 ELSE IndexType \
             END AS IndexType, \
             TRIM(ColumnName) AS ColumnName, \
             ColumnPosition \
             FROM DBC.IndicesV \
             WHERE TableName = '{}' AND DatabaseName = DATABASE \
             ORDER BY IndexNumber, ColumnPosition",
            escape_sql_string(table)
        )
    };

    let result = client.execute(&sql)?;

    let qualified = if let Some(db) = database {
        format!("{}.{}", db, table)
    } else {
        table.to_string()
    };

    let rows: Vec<IndexRow> = result
        .rows
        .iter()
        .map(|row| {
            let index_name = row
                .first()
                .map(|v| {
                    let s = v.display();
                    if s == "[NULL]" {
                        "(unnamed)".to_string()
                    } else {
                        s.trim().to_string()
                    }
                })
                .unwrap_or_default();
            let index_type = row
                .get(1)
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();
            let column_name = row
                .get(2)
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();
            let position = row
                .get(3)
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();

            IndexRow {
                index_name,
                index_type,
                column_name,
                position,
            }
        })
        .collect();

    Ok((rows, qualified))
}

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

fn show_indexes_table<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (rows, qualified) = query_indexes(client, table_name)?;

    if rows.is_empty() {
        writeln!(writer, "No indexes found for table '{}'.", table_name)?;
        writeln!(writer)?;
        writeln!(writer, "Suggestions:")?;
        writeln!(writer, "  - Check the table name spelling")?;
        writeln!(
            writer,
            "  - Try using qualified name: show-indexes database.table"
        )?;
        writeln!(
            writer,
            "  - Verify you have SELECT permission on DBC.IndicesV"
        )?;
        return Ok(());
    }

    writeln!(writer, "Indexes on {}:", qualified)?;
    writeln!(writer)?;
    writeln!(
        writer,
        "{:<30} {:<15} {:<25} {:<10}",
        "IndexName", "IndexType", "ColumnName", "Position"
    )?;
    writeln!(writer, "{}", "-".repeat(80))?;

    for row in &rows {
        writeln!(
            writer,
            "{:<30} {:<15} {:<25} {:<10}",
            truncate_str(&row.index_name, 29),
            truncate_str(&row.index_type, 14),
            truncate_str(&row.column_name, 24),
            row.position
        )?;
    }

    writeln!(writer)?;
    writeln!(writer, "{} index column(s)", rows.len())?;
    Ok(())
}

fn show_indexes_json<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (rows, _qualified) = query_indexes(client, table_name)?;

    write!(writer, "[")?;
    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            write!(writer, ",")?;
        }
        write!(
            writer,
            "{{\"index_name\":\"{}\",\"index_type\":\"{}\",\"column_name\":\"{}\",\"position\":\"{}\"}}",
            json_escape(&row.index_name),
            json_escape(&row.index_type),
            json_escape(&row.column_name),
            json_escape(&row.position)
        )?;
    }
    writeln!(writer, "]")?;
    Ok(())
}

fn show_indexes_csv<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (rows, _qualified) = query_indexes(client, table_name)?;

    writeln!(writer, "IndexName,IndexType,ColumnName,Position")?;
    for row in &rows {
        writeln!(
            writer,
            "{},{},{},{}",
            csv_escape(&row.index_name),
            csv_escape(&row.index_type),
            csv_escape(&row.column_name),
            csv_escape(&row.position)
        )?;
    }
    Ok(())
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
