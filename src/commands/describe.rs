//! Describe command implementation
//!
//! Shows table/view structure: object header, columns (name, type, nullable,
//! default, comment), and indexes. Used by both `tq describe <table>` (batch)
//! and `/describe` (REPL delegation).

use crate::cli::OutputFormat;
use crate::commands::format_helpers::{
    classify_index, column_type_case_sql, csv_escape, format_nullable, json_escape,
    map_table_kind, parse_table_name, truncate_str,
};
use crate::db::{DatabaseClient, Value};
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
// Data structures
// =============================================================================

/// Object metadata from DBC.TablesV
struct ObjectHeader {
    database: String,
    name: String,
    table_kind: String,
    kind_label: String,
}

/// Column information from DBC.ColumnsV
struct ColumnRow {
    name: String,
    col_type: String,
    nullable: String,
    default: String,
    comment: String,
}

/// Index information grouped by index number
struct IndexGroup {
    name: String,
    index_type_label: String,
    short_label: String,
    columns: Vec<String>,
}

// =============================================================================
// Query helpers
// =============================================================================

/// Resolve the database for an unqualified name by querying SELECT DATABASE
fn resolve_database(client: &DatabaseClient) -> Result<String> {
    let result = client.execute("SELECT DATABASE")?;
    if let Some(row) = result.rows.first() {
        if let Some(val) = row.first() {
            let db = val.display().trim().to_string();
            if !db.is_empty() && db != "[NULL]" {
                return Ok(db);
            }
        }
    }
    Ok(client.config().database.clone())
}

/// Query DBC.TablesV for object header metadata
fn query_object_header(
    client: &DatabaseClient,
    db: &str,
    table: &str,
) -> Result<Option<ObjectHeader>> {
    let sql = format!(
        "SELECT TRIM(DatabaseName), TRIM(TableName), TRIM(TableKind) \
         FROM DBC.TablesV \
         WHERE DatabaseName = '{}' AND TableName = '{}'",
        escape_sql_string(db),
        escape_sql_string(table)
    );

    let result = client.execute(&sql)?;
    if let Some(row) = result.rows.first() {
        let database = row
            .first()
            .map(|v| v.display().trim().to_string())
            .unwrap_or_default();
        let name = row
            .get(1)
            .map(|v| v.display().trim().to_string())
            .unwrap_or_default();
        let table_kind = row
            .get(2)
            .map(|v| v.display().trim().to_string())
            .unwrap_or_default();
        let kind_label = map_table_kind(&table_kind);

        Ok(Some(ObjectHeader {
            database,
            name,
            table_kind,
            kind_label,
        }))
    } else {
        Ok(None)
    }
}

/// Query DBC.ColumnsV for column metadata with proper type translation
fn query_columns(
    client: &DatabaseClient,
    db: &str,
    table: &str,
) -> Result<Vec<ColumnRow>> {
    let type_expr = column_type_case_sql();
    let sql = format!(
        "SELECT TRIM(ColumnName), \
         {} AS ColType, \
         Nullable, DefaultValue, \
         COALESCE(TRIM(CommentString), '') AS ColComment \
         FROM DBC.ColumnsV \
         WHERE DatabaseName = '{}' AND TableName = '{}' \
         ORDER BY ColumnId",
        type_expr,
        escape_sql_string(db),
        escape_sql_string(table)
    );

    let result = client.execute(&sql)?;

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
                .map(|v| {
                    let s = v.display().trim().to_string();
                    if s == "[NULL]" {
                        String::new()
                    } else {
                        s
                    }
                })
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
            let comment = row
                .get(4)
                .map(|v| {
                    let s = v.display().trim().to_string();
                    if s == "[NULL]" {
                        String::new()
                    } else {
                        s
                    }
                })
                .unwrap_or_default();

            ColumnRow {
                name,
                col_type,
                nullable,
                default,
                comment,
            }
        })
        .collect();

    Ok(columns)
}

/// Query DBC.IndicesV for index information, grouped by index number
fn query_indexes(
    client: &DatabaseClient,
    db: &str,
    table: &str,
) -> Result<Vec<IndexGroup>> {
    let sql = format!(
        "SELECT TRIM(IndexName) AS IndexName, \
         IndexType, UniqueFlag, \
         TRIM(ColumnName) AS ColumnName, \
         IndexNumber, ColumnPosition \
         FROM DBC.IndicesV \
         WHERE DatabaseName = '{}' AND TableName = '{}' \
         ORDER BY IndexNumber, ColumnPosition",
        escape_sql_string(db),
        escape_sql_string(table)
    );

    let result = client.execute(&sql)?;

    let mut groups: Vec<IndexGroup> = Vec::new();
    let mut index_numbers: Vec<i64> = Vec::new();

    for row in &result.rows {
        let index_name = row
            .first()
            .map(|v| {
                let s = v.display().trim().to_string();
                if s.is_empty() || s == "[NULL]" {
                    "(unnamed)".to_string()
                } else {
                    s
                }
            })
            .unwrap_or_else(|| "(unnamed)".to_string());
        let index_type_raw = row
            .get(1)
            .map(|v| v.display().trim().to_string())
            .unwrap_or_default();
        let unique_flag = row
            .get(2)
            .map(|v| v.display().trim().to_string())
            .unwrap_or_default();
        let column_name = row
            .get(3)
            .map(|v| v.display().trim().to_string())
            .unwrap_or_default();
        let index_number = match row.get(4) {
            Some(Value::Integer(n)) => *n,
            Some(v) => v.display().trim().parse::<i64>().unwrap_or(0),
            None => 0,
        };

        let is_unique = unique_flag == "Y" || unique_flag == "U";
        let (type_label, short_label) = classify_index(&index_type_raw, is_unique);

        if let Some(pos) = index_numbers.iter().position(|n| *n == index_number) {
            groups[pos].columns.push(column_name);
        } else {
            index_numbers.push(index_number);
            groups.push(IndexGroup {
                name: index_name,
                index_type_label: type_label.to_string(),
                short_label: short_label.to_string(),
                columns: vec![column_name],
            });
        }
    }

    Ok(groups)
}

// =============================================================================
// Output formats
// =============================================================================

fn describe_table<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (db_part, table) = parse_table_name(table_name);

    // Resolve database
    let database = if let Some(db) = db_part {
        db.to_string()
    } else {
        resolve_database(client)?
    };

    // Query object header
    let header = query_object_header(client, &database, table)?;
    if header.is_none() {
        writeln!(
            writer,
            "Error: Object '{}' not found or no columns available.",
            table_name
        )?;
        writeln!(writer)?;
        writeln!(writer, "Suggestions:")?;
        writeln!(writer, "  - Check the object name spelling")?;
        writeln!(
            writer,
            "  - Try using qualified name: describe database.object"
        )?;
        writeln!(
            writer,
            "  - Verify you have SELECT permission on DBC.ColumnsV"
        )?;
        return Ok(());
    }
    let header = header.unwrap();

    // Object header block
    writeln!(writer, "── Object ──")?;
    writeln!(writer, "  Type:      {}", header.kind_label)?;
    writeln!(writer, "  Database:  {}", header.database)?;
    writeln!(writer, "  Name:      {}", header.name)?;
    writeln!(writer)?;

    // Columns
    let columns = query_columns(client, &database, table)?;
    if columns.is_empty() {
        writeln!(writer, "No columns found.")?;
        return Ok(());
    }

    let has_comments = columns.iter().any(|c| !c.comment.is_empty());

    writeln!(writer, "── Columns ({}) ──", columns.len())?;
    if has_comments {
        writeln!(
            writer,
            "  {:<24} {:<20} {:<10} {:<15} Comment",
            "Column", "Type", "Nullable", "Default"
        )?;
        writeln!(writer, "  {}", "-".repeat(90))?;
        for col in &columns {
            writeln!(
                writer,
                "  {:<24} {:<20} {:<10} {:<15} {}",
                truncate_str(&col.name, 22),
                truncate_str(&col.col_type, 18),
                &col.nullable,
                truncate_str(&col.default, 14),
                truncate_str(&col.comment, 30)
            )?;
        }
    } else {
        writeln!(
            writer,
            "  {:<24} {:<20} {:<10} {:<15}",
            "Column", "Type", "Nullable", "Default"
        )?;
        writeln!(writer, "  {}", "-".repeat(70))?;
        for col in &columns {
            writeln!(
                writer,
                "  {:<24} {:<20} {:<10} {:<15}",
                truncate_str(&col.name, 22),
                truncate_str(&col.col_type, 18),
                &col.nullable,
                truncate_str(&col.default, 14)
            )?;
        }
    }
    writeln!(writer, "  {} column(s)", columns.len())?;
    writeln!(writer)?;

    // Indexes (for tables)
    if header.table_kind == "T" || header.table_kind == "O" {
        if let Ok(indexes) = query_indexes(client, &database, table) {
            if !indexes.is_empty() {
                writeln!(writer, "── Indexes ──")?;
                for idx in &indexes {
                    let cols = idx.columns.join(", ");
                    if idx.name != "(unnamed)" {
                        writeln!(
                            writer,
                            "  {} ({}) \"{}\": {}",
                            idx.index_type_label, idx.short_label, idx.name, cols
                        )?;
                    } else {
                        writeln!(
                            writer,
                            "  {} ({}): {}",
                            idx.index_type_label, idx.short_label, cols
                        )?;
                    }
                }
                writeln!(writer)?;
            }
        }
    }

    Ok(())
}

fn describe_json<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (db_part, table) = parse_table_name(table_name);
    let database = if let Some(db) = db_part {
        db.to_string()
    } else {
        resolve_database(client)?
    };

    let header = query_object_header(client, &database, table)?;
    if header.is_none() {
        writeln!(
            writer,
            "{{\"error\":\"Object '{}' not found\"}}",
            json_escape(table_name)
        )?;
        return Ok(());
    }
    let header = header.unwrap();

    let columns = query_columns(client, &database, table)?;

    // Structured JSON: {object, columns, indexes}
    write!(writer, "{{")?;

    // Object section
    write!(
        writer,
        "\"object\":{{\"database\":\"{}\",\"name\":\"{}\",\"type\":\"{}\"}}",
        json_escape(&header.database),
        json_escape(&header.name),
        json_escape(&header.kind_label)
    )?;

    // Columns section
    write!(writer, ",\"columns\":[")?;
    for (i, col) in columns.iter().enumerate() {
        if i > 0 {
            write!(writer, ",")?;
        }
        write!(
            writer,
            "{{\"name\":\"{}\",\"type\":\"{}\",\"nullable\":\"{}\",\"default\":\"{}\"",
            json_escape(&col.name),
            json_escape(&col.col_type),
            json_escape(&col.nullable),
            json_escape(&col.default)
        )?;
        if !col.comment.is_empty() {
            write!(writer, ",\"comment\":\"{}\"", json_escape(&col.comment))?;
        }
        write!(writer, "}}")?;
    }
    write!(writer, "]")?;

    // Indexes section (for tables)
    if header.table_kind == "T" || header.table_kind == "O" {
        if let Ok(indexes) = query_indexes(client, &database, table) {
            write!(writer, ",\"indexes\":[")?;
            for (i, idx) in indexes.iter().enumerate() {
                if i > 0 {
                    write!(writer, ",")?;
                }
                write!(
                    writer,
                    "{{\"name\":\"{}\",\"type\":\"{}\",\"columns\":[",
                    json_escape(&idx.name),
                    json_escape(&idx.short_label)
                )?;
                for (j, col) in idx.columns.iter().enumerate() {
                    if j > 0 {
                        write!(writer, ",")?;
                    }
                    write!(writer, "\"{}\"", json_escape(col))?;
                }
                write!(writer, "]}}")?;
            }
            write!(writer, "]")?;
        }
    }

    writeln!(writer, "}}")?;
    Ok(())
}

fn describe_csv<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (db_part, table) = parse_table_name(table_name);
    let database = if let Some(db) = db_part {
        db.to_string()
    } else {
        resolve_database(client)?
    };

    let header = query_object_header(client, &database, table)?;
    if let Some(ref h) = header {
        writeln!(
            writer,
            "# Object: {}.{} ({})",
            h.database, h.name, h.kind_label
        )?;
    }

    let columns = query_columns(client, &database, table)?;

    writeln!(writer, "Column,Type,Nullable,Default,Comment")?;
    for col in &columns {
        writeln!(
            writer,
            "{},{},{},{},{}",
            csv_escape(&col.name),
            csv_escape(&col.col_type),
            csv_escape(&col.nullable),
            csv_escape(&col.default),
            csv_escape(&col.comment)
        )?;
    }
    Ok(())
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // format_helpers functions are tested in format_helpers::tests

    #[test]
    fn test_column_row_structure() {
        let col = ColumnRow {
            name: "id".to_string(),
            col_type: "INTEGER".to_string(),
            nullable: "NO".to_string(),
            default: "-".to_string(),
            comment: "Primary key".to_string(),
        };
        assert_eq!(col.name, "id");
        assert_eq!(col.comment, "Primary key");
    }

    #[test]
    fn test_index_group_structure() {
        let idx = IndexGroup {
            name: "pk_emp".to_string(),
            index_type_label: "Primary Index".to_string(),
            short_label: "UPI".to_string(),
            columns: vec!["emp_id".to_string()],
        };
        assert_eq!(idx.short_label, "UPI");
        assert_eq!(idx.columns.len(), 1);
    }

    #[test]
    fn test_object_header_structure() {
        let header = ObjectHeader {
            database: "mydb".to_string(),
            name: "employees".to_string(),
            table_kind: "T".to_string(),
            kind_label: "Table".to_string(),
        };
        assert_eq!(header.kind_label, "Table");
    }
}
