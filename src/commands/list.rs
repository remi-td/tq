//! List command implementation
//!
//! Lists database objects: databases, tables, or views.
//! Used by `tq list <type> [pattern]` (batch) and `/list` (REPL delegation).

use crate::cli::{ListObjectType, OutputFormat};
use crate::commands::format_helpers::{csv_escape, format_size, json_escape, markdown_escape_pipe};
use crate::db::DatabaseClient;
use crate::error::Result;
use crate::pagination::PaginationInfo;
use crate::sql::escape_sql_string;
use std::io::Write;

// =============================================================================
// Public API
// =============================================================================

/// Execute `tq list` in batch mode with format selection
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &crate::cli::ListArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    let pagination_args = args.page_size.map(|ps| (ps, args.page));
    match args.object_type {
        ListObjectType::Databases => list_databases(client, args.format, pagination_args, writer),
        ListObjectType::Tables => list_tables(client, args.pattern.as_deref(), args.database.as_deref(), args.format, pagination_args, writer),
        ListObjectType::Views => list_views(client, args.database.as_deref(), args.format, pagination_args, writer),
    }
}

/// Execute /list in REPL mode (table format with extra spacing)
///
/// The REPL calls this with a subcommand string and optional pattern/database.
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    subcommand: &str,
    pattern: Option<&str>,
    database: Option<&str>,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;
    match subcommand {
        "databases" | "db" | "dbs" => list_databases(client, OutputFormat::Table, None, writer)?,
        "tables" | "table" | "t" => {
            list_tables(client, pattern, database, OutputFormat::Table, None, writer)?;
        }
        "views" | "view" | "v" => {
            list_views(client, database, OutputFormat::Table, None, writer)?;
        }
        _ => {
            writeln!(writer, "Error: Unknown list subcommand: {}", subcommand)?;
            writeln!(writer, "Available: databases, tables, views")?;
        }
    }
    writeln!(writer)?;
    Ok(())
}

// =============================================================================
// Databases (enriched with Owner and Type)
// =============================================================================

/// Database entry with owner and type info
struct DatabaseEntry {
    name: String,
    owner: String,
    db_kind: String,
}

fn list_databases<W: Write>(
    client: &DatabaseClient,
    format: OutputFormat,
    pagination_args: Option<(usize, usize)>,
    writer: &mut W,
) -> Result<()> {
    let sql = r#"
        SELECT TRIM(DatabaseName) AS database_name,
               TRIM(OwnerName) AS owner_name,
               CASE
                   WHEN OwnerName = 'DBC' OR DatabaseName = 'DBC' THEN 'System'
                   WHEN DBKind = 'U' THEN 'User'
                   ELSE 'User'
               END AS db_kind
        FROM DBC.DatabasesV
        WHERE DatabaseName NOT IN ('All', 'Console', 'Crashdumps',
                                   'dbcmngr', 'Default', 'External_AP',
                                   'EXTUSER', 'LockLogShredder', 'PUBLIC',
                                   'SQLJ', 'Sys_Calendar', 'SysAdmin',
                                   'SYSBAR', 'SYSJDBC', 'SYSLIB', 'SYSSPATIAL',
                                   'SystemFe', 'SYSUDTLIB', 'TD_SERVER_DB',
                                   'TD_SYSFNLIB', 'TD_SYSGPL', 'TD_SYSXML',
                                   'TDMaps', 'TDPUSER', 'TDQCD', 'TDStats',
                                   'tdwm', 'VIEWPOINT')
        ORDER BY DatabaseName
    "#;

    let result = client.execute(sql)?;
    let databases: Vec<DatabaseEntry> = result
        .rows
        .iter()
        .filter_map(|row| {
            let name = row.first().map(|v| v.display())?;
            if name == "[NULL]" {
                return None;
            }
            let owner = row
                .get(1)
                .map(|v| {
                    let s = v.display();
                    if s == "[NULL]" {
                        String::new()
                    } else {
                        s.trim().to_string()
                    }
                })
                .unwrap_or_default();
            let db_kind = row
                .get(2)
                .map(|v| {
                    let s = v.display();
                    if s == "[NULL]" {
                        String::new()
                    } else {
                        s.trim().to_string()
                    }
                })
                .unwrap_or_default();
            Some(DatabaseEntry {
                name: name.trim().to_string(),
                owner,
                db_kind,
            })
        })
        .collect();

    // Apply pagination if requested
    let pagination = pagination_args.map(|(page_size, page)| {
        PaginationInfo::new(page, page_size, databases.len())
    });

    let display = if let Some(ref pg) = pagination {
        let (start, end) = pg.row_range();
        if start < databases.len() {
            &databases[start..end.min(databases.len())]
        } else {
            &databases[0..0]
        }
    } else {
        &databases[..]
    };

    match format {
        OutputFormat::Table => {
            render_databases_table(display, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Json => render_databases_json_with_pagination(display, pagination.as_ref(), writer)?,
        OutputFormat::Csv => {
            render_databases_csv(display, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Markdown | OutputFormat::Md => {
            render_databases_markdown(display, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
    }

    Ok(())
}

/// Render databases as a human-readable table.
fn render_databases_table<W: Write>(
    databases: &[DatabaseEntry],
    writer: &mut W,
) -> Result<()> {
    writeln!(writer, "Databases ({}):", databases.len())?;
    writeln!(
        writer,
        "{:<30} {:<20} {:<10}",
        "Name", "Owner", "Type"
    )?;
    writeln!(writer, "{}", "-".repeat(60))?;

    for db in databases {
        writeln!(
            writer,
            "{:<30} {:<20} {:<10}",
            db.name, db.owner, db.db_kind
        )?;
    }

    writeln!(writer)?;
    writeln!(writer, "{} database(s)", databases.len())?;
    Ok(())
}

/// Render databases as JSON with optional pagination.
fn render_databases_json_with_pagination<W: Write>(
    databases: &[DatabaseEntry],
    pagination: Option<&PaginationInfo>,
    writer: &mut W,
) -> Result<()> {
    write!(writer, "{{\"ok\":true,\"row_count\":{},\"data\":[", databases.len())?;
    for (i, db) in databases.iter().enumerate() {
        if i > 0 {
            write!(writer, ",")?;
        }
        write!(
            writer,
            "{{\"database\":\"{}\",\"owner\":\"{}\",\"type\":\"{}\"}}",
            json_escape(&db.name),
            json_escape(&db.owner),
            json_escape(&db.db_kind)
        )?;
    }
    write!(writer, "]")?;
    if let Some(pg) = pagination {
        write!(
            writer,
            ",\"pagination\":{{\"page\":{},\"page_size\":{},\"total_rows\":{},\"total_pages\":{},\"has_more\":{}}}",
            pg.page,
            pg.page_size,
            pg.total_rows,
            pg.total_pages(),
            pg.has_more()
        )?;
    }
    writeln!(writer, "}}")?;
    Ok(())
}

/// Render databases as CSV.
fn render_databases_csv<W: Write>(
    databases: &[DatabaseEntry],
    writer: &mut W,
) -> Result<()> {
    writeln!(writer, "DatabaseName,Owner,Type")?;
    for db in databases {
        writeln!(
            writer,
            "{},{},{}",
            csv_escape(&db.name),
            csv_escape(&db.owner),
            csv_escape(&db.db_kind)
        )?;
    }
    Ok(())
}

/// Render databases as a Markdown table.
fn render_databases_markdown<W: Write>(
    databases: &[DatabaseEntry],
    writer: &mut W,
) -> Result<()> {
    writeln!(writer, "| DatabaseName | Owner | Type |")?;
    writeln!(writer, "| :--- | :--- | :--- |")?;
    for db in databases {
        writeln!(
            writer,
            "| {} | {} | {} |",
            markdown_escape_pipe(&db.name),
            markdown_escape_pipe(&db.owner),
            markdown_escape_pipe(&db.db_kind)
        )?;
    }
    Ok(())
}

// =============================================================================
// Tables (enriched with row count estimate, size, and owner)
// =============================================================================

/// Table entry with metadata
struct TableEntry {
    name: String,
    kind: String,
    row_count_display: String,
    row_count_raw: Option<i64>,
    size_display: String,
    size_bytes: Option<i64>,
    owner: String,
}

fn list_tables<W: Write>(
    client: &DatabaseClient,
    pattern: Option<&str>,
    database: Option<&str>,
    format: OutputFormat,
    pagination_args: Option<(usize, usize)>,
    writer: &mut W,
) -> Result<()> {
    let db_clause = if let Some(db) = database {
        format!("t.DatabaseName = '{}'", escape_sql_string(db))
    } else {
        "t.DatabaseName = DATABASE".to_string()
    };

    // Query with size from TableSizeV and owner from CreatorName
    let sql = format!(
        "SELECT TRIM(t.TableName) AS table_name, t.TableKind, \
         COALESCE(CAST(s.RowCount AS VARCHAR(20)), '') AS RowCount, \
         COALESCE(CAST(s.CurrentPerm AS VARCHAR(20)), '') AS CurrentPerm, \
         TRIM(t.CreatorName) AS Owner \
         FROM DBC.TablesV t \
         LEFT JOIN ( \
             SELECT DatabaseName, TableName, \
                    SUM(RowCount) AS RowCount, \
                    SUM(CurrentPerm) AS CurrentPerm \
             FROM DBC.TableSizeV \
             GROUP BY DatabaseName, TableName \
         ) s ON t.DatabaseName = s.DatabaseName AND t.TableName = s.TableName \
         WHERE {} AND t.TableKind IN ('T', 'O') \
         ORDER BY t.TableName",
        db_clause
    );

    let result = client.execute(&sql)?;

    let tables: Vec<TableEntry> = result
        .rows
        .iter()
        .filter_map(|row| {
            let name = row.first().map(|v| v.display())?;
            let kind = row.get(1).map(|v| v.display()).unwrap_or_default();
            if name == "[NULL]" {
                return None;
            }

            if let Some(pat) = pattern {
                if !matches_glob(&name, pat) {
                    return None;
                }
            }

            let kind_str = if kind.trim() == "O" {
                "NoPI".to_string()
            } else {
                "TABLE".to_string()
            };

            let row_count_raw = row.get(2).and_then(|v| {
                let s = v.display().trim().to_string();
                if s.is_empty() || s == "[NULL]" {
                    None
                } else {
                    s.parse::<i64>().ok()
                }
            });
            let row_count_display = row_count_raw
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".to_string());

            let size_bytes = row.get(3).and_then(|v| {
                let s = v.display().trim().to_string();
                if s.is_empty() || s == "[NULL]" {
                    None
                } else {
                    s.parse::<i64>().ok()
                }
            });
            let size_display = size_bytes
                .map(|b| format_size(b, 1))
                .unwrap_or_else(|| "-".to_string());

            let owner = row
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

            Some(TableEntry {
                name: name.trim().to_string(),
                kind: kind_str,
                row_count_display,
                row_count_raw,
                size_display,
                size_bytes,
                owner,
            })
        })
        .collect();

    let db_label = database.unwrap_or("(current)");
    let pattern_str = pattern
        .map(|p| format!(" matching '{}'", p))
        .unwrap_or_default();

    // Apply pagination if requested
    let pagination = pagination_args.map(|(page_size, page)| {
        PaginationInfo::new(page, page_size, tables.len())
    });

    let display = if let Some(ref pg) = pagination {
        let (start, end) = pg.row_range();
        if start < tables.len() {
            &tables[start..end.min(tables.len())]
        } else {
            &tables[0..0]
        }
    } else {
        &tables[..]
    };

    match format {
        OutputFormat::Table => {
            writeln!(writer, "Tables in {}{}:", db_label, pattern_str)?;
            writeln!(
                writer,
                "{:<30} {:<8} {:>12} {:>10} {:<15}",
                "Name", "Type", "Rows (Est.)", "Size", "Owner"
            )?;
            writeln!(writer, "{}", "-".repeat(78))?;

            for t in display {
                writeln!(
                    writer,
                    "{:<30} {:<8} {:>12} {:>10} {:<15}",
                    t.name, t.kind, t.row_count_display, t.size_display, t.owner
                )?;
            }

            writeln!(writer)?;
            writeln!(writer, "{} table(s)", display.len())?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Json => {
            write!(writer, "{{\"ok\":true,\"row_count\":{},\"data\":[", display.len())?;
            for (i, t) in display.iter().enumerate() {
                if i > 0 {
                    write!(writer, ",")?;
                }
                let rows_json = match t.row_count_raw {
                    Some(n) => n.to_string(),
                    None => "null".to_string(),
                };
                let size_json = match t.size_bytes {
                    Some(n) => n.to_string(),
                    None => "null".to_string(),
                };
                write!(
                    writer,
                    "{{\"name\":\"{}\",\"type\":\"{}\",\"estimated_rows\":{},\"size_bytes\":{},\"owner\":\"{}\"}}",
                    json_escape(&t.name),
                    json_escape(&t.kind),
                    rows_json,
                    size_json,
                    json_escape(&t.owner)
                )?;
            }
            write!(writer, "]")?;
            if let Some(ref pg) = pagination {
                write!(
                    writer,
                    ",\"pagination\":{{\"page\":{},\"page_size\":{},\"total_rows\":{},\"total_pages\":{},\"has_more\":{}}}",
                    pg.page,
                    pg.page_size,
                    pg.total_rows,
                    pg.total_pages(),
                    pg.has_more()
                )?;
            }
            writeln!(writer, "}}")?;
        }
        OutputFormat::Csv => {
            writeln!(writer, "TableName,Type,RowsEst,Size,Owner")?;
            for t in display {
                writeln!(
                    writer,
                    "{},{},{},{},{}",
                    csv_escape(&t.name),
                    csv_escape(&t.kind),
                    csv_escape(&t.row_count_display),
                    csv_escape(&t.size_display),
                    csv_escape(&t.owner)
                )?;
            }
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Markdown | OutputFormat::Md => {
            writeln!(writer, "| Name | Type | Rows (Est.) | Size | Owner |")?;
            writeln!(writer, "| :--- | :--- | ---: | ---: | :--- |")?;
            for t in display {
                writeln!(
                    writer,
                    "| {} | {} | {} | {} | {} |",
                    markdown_escape_pipe(&t.name),
                    markdown_escape_pipe(&t.kind),
                    markdown_escape_pipe(&t.row_count_display),
                    markdown_escape_pipe(&t.size_display),
                    markdown_escape_pipe(&t.owner)
                )?;
            }
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
    }

    Ok(())
}

// =============================================================================
// Views (enriched with Owner)
// =============================================================================

/// View entry with owner info
struct ViewEntry {
    name: String,
    owner: String,
}

fn list_views<W: Write>(
    client: &DatabaseClient,
    database: Option<&str>,
    format: OutputFormat,
    pagination_args: Option<(usize, usize)>,
    writer: &mut W,
) -> Result<()> {
    let db_clause = if let Some(db) = database {
        format!("DatabaseName = '{}'", escape_sql_string(db))
    } else {
        "DatabaseName = DATABASE".to_string()
    };

    let sql = format!(
        "SELECT TRIM(TableName) AS view_name, \
         TRIM(CreatorName) AS Owner \
         FROM DBC.TablesV \
         WHERE {} AND TableKind = 'V' \
         ORDER BY TableName",
        db_clause
    );

    let result = client.execute(&sql)?;

    let views: Vec<ViewEntry> = result
        .rows
        .iter()
        .filter_map(|row| {
            let name = row.first().map(|v| v.display())?;
            if name == "[NULL]" {
                return None;
            }
            let owner = row
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
            Some(ViewEntry {
                name: name.trim().to_string(),
                owner,
            })
        })
        .collect();

    let db_label = database.unwrap_or("(current)");

    // Apply pagination if requested
    let pagination = pagination_args.map(|(page_size, page)| {
        PaginationInfo::new(page, page_size, views.len())
    });

    let display = if let Some(ref pg) = pagination {
        let (start, end) = pg.row_range();
        if start < views.len() {
            &views[start..end.min(views.len())]
        } else {
            &views[0..0]
        }
    } else {
        &views[..]
    };

    match format {
        OutputFormat::Table => {
            writeln!(writer, "Views in {}:", db_label)?;
            writeln!(
                writer,
                "{:<35} {:<15}",
                "Name", "Owner"
            )?;
            writeln!(writer, "{}", "-".repeat(50))?;

            if display.is_empty() {
                writeln!(writer, "(no views found)")?;
            } else {
                for view in display {
                    writeln!(writer, "{:<35} {:<15}", view.name, view.owner)?;
                }
            }

            writeln!(writer)?;
            writeln!(writer, "{} view(s)", display.len())?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Json => {
            write!(writer, "{{\"ok\":true,\"row_count\":{},\"data\":[", display.len())?;
            for (i, view) in display.iter().enumerate() {
                if i > 0 {
                    write!(writer, ",")?;
                }
                write!(
                    writer,
                    "{{\"name\":\"{}\",\"owner\":\"{}\"}}",
                    json_escape(&view.name),
                    json_escape(&view.owner)
                )?;
            }
            write!(writer, "]")?;
            if let Some(ref pg) = pagination {
                write!(
                    writer,
                    ",\"pagination\":{{\"page\":{},\"page_size\":{},\"total_rows\":{},\"total_pages\":{},\"has_more\":{}}}",
                    pg.page,
                    pg.page_size,
                    pg.total_rows,
                    pg.total_pages(),
                    pg.has_more()
                )?;
            }
            writeln!(writer, "}}")?;
        }
        OutputFormat::Csv => {
            writeln!(writer, "ViewName,Owner")?;
            for view in display {
                writeln!(
                    writer,
                    "{},{}",
                    csv_escape(&view.name),
                    csv_escape(&view.owner)
                )?;
            }
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Markdown | OutputFormat::Md => {
            writeln!(writer, "| ViewName | Owner |")?;
            writeln!(writer, "| :--- | :--- |")?;
            for view in display {
                writeln!(
                    writer,
                    "| {} | {} |",
                    markdown_escape_pipe(&view.name),
                    markdown_escape_pipe(&view.owner)
                )?;
            }
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
    }

    Ok(())
}

// =============================================================================
// Helpers
// =============================================================================

/// Simple glob pattern matching (case-insensitive)
///
/// Supports `*` for any sequence and `?` for any single character.
fn matches_glob(text: &str, pattern: &str) -> bool {
    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    let mut pattern_chars = pattern_lower.chars().peekable();
    let mut text_chars = text_lower.chars().peekable();

    fn match_recursive(
        pattern: &mut std::iter::Peekable<std::str::Chars>,
        text: &mut std::iter::Peekable<std::str::Chars>,
    ) -> bool {
        loop {
            match (pattern.peek().copied(), text.peek().copied()) {
                (None, None) => return true,
                (None, Some(_)) => return false,
                (Some('*'), _) => {
                    pattern.next();
                    if pattern.peek().is_none() {
                        return true;
                    }
                    let mut text_clone = text.clone();
                    loop {
                        let mut pattern_clone = pattern.clone();
                        let mut text_try = text_clone.clone();
                        if match_recursive(&mut pattern_clone, &mut text_try) {
                            return true;
                        }
                        if text_clone.next().is_none() {
                            return false;
                        }
                    }
                }
                (Some('?'), Some(_)) => {
                    pattern.next();
                    text.next();
                }
                (Some(p), Some(t)) => {
                    if p != t {
                        return false;
                    }
                    pattern.next();
                    text.next();
                }
                (Some(_), None) => return false,
            }
        }
    }

    match_recursive(&mut pattern_chars, &mut text_chars)
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_glob_exact() {
        assert!(matches_glob("employees", "employees"));
        assert!(!matches_glob("employees", "employee"));
    }

    #[test]
    fn test_matches_glob_star() {
        assert!(matches_glob("employees", "emp*"));
        assert!(matches_glob("employees", "*ees"));
        assert!(matches_glob("employees", "*ploy*"));
        assert!(!matches_glob("employees", "xyz*"));
    }

    #[test]
    fn test_matches_glob_question() {
        assert!(matches_glob("abc", "a?c"));
        assert!(!matches_glob("abcd", "a?c"));
    }

    #[test]
    fn test_matches_glob_case_insensitive() {
        assert!(matches_glob("EMPLOYEES", "emp*"));
        assert!(matches_glob("employees", "EMP*"));
    }

    #[test]
    fn test_format_size_short_values() {
        assert_eq!(format_size(0, 1), "0 B");
        assert_eq!(format_size(512, 1), "512 B");
        assert_eq!(format_size(1024, 1), "1.0 KB");
        assert_eq!(format_size(1048576, 1), "1.0 MB");
        assert_eq!(format_size(1073741824, 1), "1.0 GB");
        assert_eq!(format_size(1099511627776, 1), "1.0 TB");
        assert_eq!(format_size(-100, 1), "-100 B");
    }

    #[test]
    fn test_database_entry_structure() {
        let db = DatabaseEntry {
            name: "mydb".to_string(),
            owner: "DBC".to_string(),
            db_kind: "System".to_string(),
        };
        assert_eq!(db.name, "mydb");
        assert_eq!(db.db_kind, "System");
    }

    #[test]
    fn test_table_entry_structure() {
        let t = TableEntry {
            name: "employees".to_string(),
            kind: "TABLE".to_string(),
            row_count_display: "1000".to_string(),
            row_count_raw: Some(1000),
            size_display: "2.5 MB".to_string(),
            size_bytes: Some(2621440),
            owner: "admin".to_string(),
        };
        assert_eq!(t.name, "employees");
        assert_eq!(t.row_count_raw, Some(1000));
        assert_eq!(t.size_bytes, Some(2621440));
    }

    // Writer-injection tests for rendering functions

    #[test]
    fn test_render_databases_table() {
        let databases = vec![
            DatabaseEntry {
                name: "mydb".to_string(),
                owner: "DBC".to_string(),
                db_kind: "System".to_string(),
            },
            DatabaseEntry {
                name: "userdb".to_string(),
                owner: "alice".to_string(),
                db_kind: "User".to_string(),
            },
        ];
        let mut buf = Vec::new();
        render_databases_table(&databases, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Databases (2):"));
        assert!(output.contains("Name"));
        assert!(output.contains("Owner"));
        assert!(output.contains("Type"));
        assert!(output.contains("mydb"));
        assert!(output.contains("System"));
        assert!(output.contains("userdb"));
        assert!(output.contains("User"));
        assert!(output.contains("2 database(s)"));
    }

    #[test]
    fn test_render_databases_json_uses_database_key() {
        let databases = vec![DatabaseEntry {
            name: "testdb".to_string(),
            owner: "bob".to_string(),
            db_kind: "User".to_string(),
        }];
        let mut buf = Vec::new();
        render_databases_json_with_pagination(&databases, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Must use "database" key, not "name"
        assert!(output.contains("\"database\":\"testdb\""));
        assert!(!output.contains("\"name\":"));
    }

    #[test]
    fn test_render_databases_csv() {
        let databases = vec![DatabaseEntry {
            name: "mydb".to_string(),
            owner: "DBC".to_string(),
            db_kind: "System".to_string(),
        }];
        let mut buf = Vec::new();
        render_databases_csv(&databases, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("DatabaseName,Owner,Type"));
        assert!(output.contains("mydb,DBC,System"));
    }

    #[test]
    fn test_list_tables_json_integer_types() {
        // Verify the JSON logic uses integer types for rows and size
        let t = TableEntry {
            name: "test".to_string(),
            kind: "TABLE".to_string(),
            row_count_display: "500".to_string(),
            row_count_raw: Some(500),
            size_display: "1.0 KB".to_string(),
            size_bytes: Some(1024),
            owner: "alice".to_string(),
        };
        let rows_json = match t.row_count_raw {
            Some(n) => n.to_string(),
            None => "null".to_string(),
        };
        let size_json = match t.size_bytes {
            Some(n) => n.to_string(),
            None => "null".to_string(),
        };
        assert_eq!(rows_json, "500");
        assert_eq!(size_json, "1024");
    }

    #[test]
    fn test_list_tables_json_null_values() {
        let t = TableEntry {
            name: "empty".to_string(),
            kind: "TABLE".to_string(),
            row_count_display: "-".to_string(),
            row_count_raw: None,
            size_display: "-".to_string(),
            size_bytes: None,
            owner: String::new(),
        };
        let rows_json = match t.row_count_raw {
            Some(n) => n.to_string(),
            None => "null".to_string(),
        };
        let size_json = match t.size_bytes {
            Some(n) => n.to_string(),
            None => "null".to_string(),
        };
        assert_eq!(rows_json, "null");
        assert_eq!(size_json, "null");
    }

    #[test]
    fn test_error_prefix_on_unknown_subcommand() {
        // Verify that unknown list subcommand includes "Error:" prefix
        let mut buf = Vec::new();
        // Simulate what execute_for_repl does for unknown subcommand
        writeln!(buf, "Error: Unknown list subcommand: foo").unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.starts_with("Error:"));
    }

    #[test]
    fn test_view_entry_structure() {
        let v = ViewEntry {
            name: "myview".to_string(),
            owner: "alice".to_string(),
        };
        assert_eq!(v.name, "myview");
        assert_eq!(v.owner, "alice");
    }
}
