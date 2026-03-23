//! Describe command implementation
//!
//! Shows table/view structure: object header, columns (name, type, nullable,
//! default, comment), and indexes. Used by both `tq describe <object>` (batch)
//! and `/describe` (REPL delegation).

use crate::cli::OutputFormat;
use crate::commands::format_helpers::{csv_escape, json_escape, parse_table_name, truncate_str};
use crate::commands::query_helpers::{self, ColumnInfo, IndexGroup, ObjectHeader};
use crate::db::DatabaseClient;
use crate::error::Result;
use std::io::Write;

// =============================================================================
// Public API
// =============================================================================

/// Execute `tq describe` in batch mode with format selection
pub fn execute<W: Write>(
    client: &DatabaseClient,
    object_name: &str,
    format: OutputFormat,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    match format {
        OutputFormat::Table => describe_table(client, object_name, writer),
        OutputFormat::Json => describe_json(client, object_name, writer),
        OutputFormat::Csv => describe_csv(client, object_name, writer),
    }
}

/// Execute /describe in REPL mode (delegates to table format with extra spacing)
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    object_name: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;
    describe_table(client, object_name, writer)?;
    writeln!(writer)?;
    Ok(())
}

// =============================================================================
// Output formats
// =============================================================================

fn describe_table<W: Write>(
    client: &DatabaseClient,
    object_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (db_part, table) = parse_table_name(object_name);
    let database = query_helpers::resolve_database(client, db_part)?;

    let header = query_helpers::query_object_header(client, &database, table)?;
    if header.is_none() {
        writeln!(
            writer,
            "Error: Object '{}' not found or no columns available.",
            object_name
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

    render_object_header(&header, writer)?;

    let columns = query_helpers::query_columns(client, &database, table)?;
    if columns.is_empty() {
        writeln!(writer, "No columns found.")?;
        return Ok(());
    }

    render_columns_table(&columns, writer)?;

    // Indexes (for tables)
    if header.table_kind == "T" || header.table_kind == "O" {
        if let Ok(indexes) = query_helpers::query_indexes(client, &database, table) {
            render_indexes_section(&indexes, writer)?;
        }
    }

    Ok(())
}

/// Render the object header block for table output.
fn render_object_header<W: Write>(header: &ObjectHeader, writer: &mut W) -> Result<()> {
    writeln!(writer, "── Object ──")?;
    writeln!(writer, "  Type:      {}", header.kind_label)?;
    writeln!(writer, "  Database:  {}", header.database)?;
    writeln!(writer, "  Name:      {}", header.name)?;
    if let Some(row_count) = header.row_count {
        if header.table_kind == "T" || header.table_kind == "O" {
            writeln!(writer, "  Rows (Est.): {}", row_count)?;
        }
    }
    writeln!(writer)?;
    Ok(())
}

/// Render the columns section for table output.
fn render_columns_table<W: Write>(columns: &[ColumnInfo], writer: &mut W) -> Result<()> {
    let has_comments = columns.iter().any(|c| !c.comment.is_empty());

    writeln!(writer, "── Columns ({}) ──", columns.len())?;
    if has_comments {
        writeln!(
            writer,
            "  {:<24} {:<20} {:<10} {:<15} Comment",
            "Column", "Type", "Nullable", "Default"
        )?;
        writeln!(writer, "  {}", "-".repeat(90))?;
        for col in columns {
            writeln!(
                writer,
                "  {:<24} {:<20} {:<10} {:<15} {}",
                truncate_str(&col.name, 22),
                truncate_str(&col.col_type, 18),
                &col.nullable,
                truncate_str(&col.default_val, 14),
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
        for col in columns {
            writeln!(
                writer,
                "  {:<24} {:<20} {:<10} {:<15}",
                truncate_str(&col.name, 22),
                truncate_str(&col.col_type, 18),
                &col.nullable,
                truncate_str(&col.default_val, 14)
            )?;
        }
    }
    writeln!(writer, "  {} column(s)", columns.len())?;
    writeln!(writer)?;
    Ok(())
}

/// Render the indexes section for table output.
fn render_indexes_section<W: Write>(indexes: &[IndexGroup], writer: &mut W) -> Result<()> {
    if indexes.is_empty() {
        writeln!(writer, "No indexes defined.")?;
        writeln!(writer)?;
        return Ok(());
    }

    writeln!(writer, "── Indexes ──")?;
    for idx in indexes {
        let cols = idx.columns.join(", ");
        if let Some(ref name) = idx.name {
            writeln!(
                writer,
                "  {} ({}) \"{}\": {}",
                idx.index_type_label, idx.short_label, name, cols
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
    Ok(())
}

fn describe_json<W: Write>(
    client: &DatabaseClient,
    object_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (db_part, table) = parse_table_name(object_name);
    let database = query_helpers::resolve_database(client, db_part)?;

    let header = query_helpers::query_object_header(client, &database, table)?;
    if header.is_none() {
        writeln!(
            writer,
            "{{\"error\":\"Object '{}' not found\"}}",
            json_escape(object_name)
        )?;
        return Ok(());
    }
    let header = header.unwrap();

    let columns = query_helpers::query_columns(client, &database, table)?;

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

    // Columns section — nullable as boolean, default as null when "-"
    write!(writer, ",\"columns\":[")?;
    for (i, col) in columns.iter().enumerate() {
        if i > 0 {
            write!(writer, ",")?;
        }
        let nullable_bool = col.nullable == "YES";
        let default_json = if col.default_val == "-" {
            "null".to_string()
        } else {
            format!("\"{}\"", json_escape(&col.default_val))
        };
        write!(
            writer,
            "{{\"name\":\"{}\",\"type\":\"{}\",\"nullable\":{},\"default\":{}",
            json_escape(&col.name),
            json_escape(&col.col_type),
            nullable_bool,
            default_json
        )?;
        if !col.comment.is_empty() {
            write!(writer, ",\"comment\":\"{}\"", json_escape(&col.comment))?;
        }
        write!(writer, "}}")?;
    }
    write!(writer, "]")?;

    // Indexes section (for tables)
    if header.table_kind == "T" || header.table_kind == "O" {
        if let Ok(indexes) = query_helpers::query_indexes(client, &database, table) {
            write!(writer, ",\"indexes\":[")?;
            for (i, idx) in indexes.iter().enumerate() {
                if i > 0 {
                    write!(writer, ",")?;
                }
                let name_json = match idx.name {
                    Some(ref n) => format!("\"{}\"", json_escape(n)),
                    None => "null".to_string(),
                };
                write!(
                    writer,
                    "{{\"name\":{},\"type\":\"{}\",\"columns\":[",
                    name_json,
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
    object_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (db_part, table) = parse_table_name(object_name);
    let database = query_helpers::resolve_database(client, db_part)?;

    let header = query_helpers::query_object_header(client, &database, table)?;
    if let Some(ref h) = header {
        writeln!(
            writer,
            "# Object: {}.{} ({})",
            h.database, h.name, h.kind_label
        )?;
    }

    let columns = query_helpers::query_columns(client, &database, table)?;

    writeln!(writer, "Column,Type,Nullable,Default,Comment")?;
    for col in &columns {
        writeln!(
            writer,
            "{},{},{},{},{}",
            csv_escape(&col.name),
            csv_escape(&col.col_type),
            csv_escape(&col.nullable),
            csv_escape(&col.default_val),
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

    #[test]
    fn test_column_info_via_query_helpers() {
        let col = ColumnInfo {
            name: "id".to_string(),
            col_type: "INTEGER".to_string(),
            nullable: "NO".to_string(),
            default_val: "-".to_string(),
            comment: "Primary key".to_string(),
        };
        assert_eq!(col.name, "id");
        assert_eq!(col.comment, "Primary key");
    }

    #[test]
    fn test_index_group_via_query_helpers() {
        let idx = IndexGroup {
            name: Some("pk_emp".to_string()),
            index_type_label: "Primary Index".to_string(),
            short_label: "UPI".to_string(),
            columns: vec!["emp_id".to_string()],
            is_primary: true,
        };
        assert_eq!(idx.short_label, "UPI");
        assert_eq!(idx.columns.len(), 1);
    }

    #[test]
    fn test_object_header_via_query_helpers() {
        let header = ObjectHeader {
            database: "mydb".to_string(),
            name: "employees".to_string(),
            object_type: "T".to_string(),
            kind_label: "Table".to_string(),
            table_kind: "T".to_string(),
            row_count: Some(1000),
        };
        assert_eq!(header.kind_label, "Table");
        assert_eq!(header.row_count, Some(1000));
    }

    // Writer-injection tests for rendering functions

    #[test]
    fn test_render_object_header_table() {
        let header = ObjectHeader {
            database: "testdb".to_string(),
            name: "employees".to_string(),
            object_type: "T".to_string(),
            kind_label: "Table".to_string(),
            table_kind: "T".to_string(),
            row_count: Some(42),
        };
        let mut buf = Vec::new();
        render_object_header(&header, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("── Object ──"));
        assert!(output.contains("Type:      Table"));
        assert!(output.contains("Database:  testdb"));
        assert!(output.contains("Name:      employees"));
        assert!(output.contains("Rows (Est.): 42"));
    }

    #[test]
    fn test_render_object_header_view_no_rows() {
        let header = ObjectHeader {
            database: "testdb".to_string(),
            name: "myview".to_string(),
            object_type: "V".to_string(),
            kind_label: "View".to_string(),
            table_kind: "V".to_string(),
            row_count: None,
        };
        let mut buf = Vec::new();
        render_object_header(&header, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Type:      View"));
        assert!(!output.contains("Rows (Est.)"));
    }

    #[test]
    fn test_render_columns_table_without_comments() {
        let columns = vec![
            ColumnInfo {
                name: "id".to_string(),
                col_type: "INTEGER".to_string(),
                nullable: "NO".to_string(),
                default_val: "-".to_string(),
                comment: String::new(),
            },
            ColumnInfo {
                name: "name".to_string(),
                col_type: "VARCHAR(100)".to_string(),
                nullable: "YES".to_string(),
                default_val: "-".to_string(),
                comment: String::new(),
            },
        ];
        let mut buf = Vec::new();
        render_columns_table(&columns, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("── Columns (2) ──"));
        assert!(output.contains("Column"));
        assert!(output.contains("Type"));
        assert!(output.contains("Nullable"));
        assert!(output.contains("id"));
        assert!(output.contains("INTEGER"));
        assert!(output.contains("2 column(s)"));
        // No "Comment" header when no comments
        assert!(!output.contains("Comment"));
    }

    #[test]
    fn test_render_columns_table_with_comments() {
        let columns = vec![ColumnInfo {
            name: "id".to_string(),
            col_type: "INTEGER".to_string(),
            nullable: "NO".to_string(),
            default_val: "-".to_string(),
            comment: "Primary key".to_string(),
        }];
        let mut buf = Vec::new();
        render_columns_table(&columns, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Comment"));
        assert!(output.contains("Primary key"));
    }

    #[test]
    fn test_render_indexes_section_with_indexes() {
        let indexes = vec![
            IndexGroup {
                name: None,
                index_type_label: "Primary Index".to_string(),
                short_label: "UPI".to_string(),
                columns: vec!["emp_id".to_string()],
                is_primary: true,
            },
            IndexGroup {
                name: Some("idx_name".to_string()),
                index_type_label: "Secondary Index".to_string(),
                short_label: "NUSI".to_string(),
                columns: vec!["last_name".to_string(), "first_name".to_string()],
                is_primary: false,
            },
        ];
        let mut buf = Vec::new();
        render_indexes_section(&indexes, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("── Indexes ──"));
        assert!(output.contains("Primary Index (UPI): emp_id"));
        assert!(output.contains("Secondary Index (NUSI) \"idx_name\": last_name, first_name"));
    }

    #[test]
    fn test_render_indexes_section_empty() {
        let indexes: Vec<IndexGroup> = vec![];
        let mut buf = Vec::new();
        render_indexes_section(&indexes, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("No indexes defined."));
    }

    #[test]
    fn test_describe_json_nullable_boolean() {
        // Verify JSON format uses boolean for nullable and null for default
        // by testing the rendering logic directly
        let col = ColumnInfo {
            name: "id".to_string(),
            col_type: "INTEGER".to_string(),
            nullable: "NO".to_string(),
            default_val: "-".to_string(),
            comment: String::new(),
        };
        // Simulate the JSON rendering logic
        let nullable_bool = col.nullable == "YES";
        assert!(!nullable_bool);

        let default_json = if col.default_val == "-" {
            "null".to_string()
        } else {
            format!("\"{}\"", json_escape(&col.default_val))
        };
        assert_eq!(default_json, "null");
    }

    #[test]
    fn test_describe_json_nullable_yes() {
        let col = ColumnInfo {
            name: "name".to_string(),
            col_type: "VARCHAR(100)".to_string(),
            nullable: "YES".to_string(),
            default_val: "N/A".to_string(),
            comment: String::new(),
        };
        let nullable_bool = col.nullable == "YES";
        assert!(nullable_bool);

        let default_json = if col.default_val == "-" {
            "null".to_string()
        } else {
            format!("\"{}\"", json_escape(&col.default_val))
        };
        assert_eq!(default_json, "\"N/A\"");
    }
}
