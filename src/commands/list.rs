//! List command implementation
//!
//! Lists database objects: databases, tables, or views.
//! Used by `tq list <type> [pattern]` (batch) and `/list` (REPL delegation).

use crate::cli::{ListObjectType, OutputFormat};
use crate::commands::format_helpers::{csv_escape, json_escape};
use crate::db::DatabaseClient;
use crate::error::Result;
use crate::sql::escape_sql_string;
use std::io::Write;

// =============================================================================
// Public API
// =============================================================================

/// Execute `tq list` in batch mode with format selection
pub fn execute<W: Write>(
    client: &DatabaseClient,
    object_type: ListObjectType,
    pattern: Option<&str>,
    database: Option<&str>,
    format: OutputFormat,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    match object_type {
        ListObjectType::Databases => list_databases(client, format, writer),
        ListObjectType::Tables => list_tables(client, pattern, database, format, writer),
        ListObjectType::Views => list_views(client, database, format, writer),
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
        "databases" | "db" | "dbs" => list_databases(client, OutputFormat::Table, writer)?,
        "tables" | "table" | "t" => {
            list_tables(client, pattern, database, OutputFormat::Table, writer)?;
        }
        "views" | "view" | "v" => {
            list_views(client, database, OutputFormat::Table, writer)?;
        }
        _ => {
            writeln!(writer, "Unknown list subcommand: {}", subcommand)?;
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
    writer: &mut W,
) -> Result<()> {
    let sql = r#"
        SELECT TRIM(DatabaseName) AS database_name,
               TRIM(OwnerName) AS owner_name,
               CASE DBKind
                   WHEN 'D' THEN 'Database'
                   WHEN 'U' THEN 'User'
                   ELSE DBKind
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

    match format {
        OutputFormat::Table => {
            writeln!(writer, "Databases ({}):", databases.len())?;
            writeln!(
                writer,
                "{:<30} {:<20} {:<10}",
                "Name", "Owner", "Type"
            )?;
            writeln!(writer, "{}", "-".repeat(60))?;

            for db in &databases {
                writeln!(
                    writer,
                    "{:<30} {:<20} {:<10}",
                    db.name, db.owner, db.db_kind
                )?;
            }

            writeln!(writer)?;
            writeln!(writer, "{} database(s)", databases.len())?;
        }
        OutputFormat::Json => {
            write!(writer, "[")?;
            for (i, db) in databases.iter().enumerate() {
                if i > 0 {
                    write!(writer, ",")?;
                }
                write!(
                    writer,
                    "{{\"name\":\"{}\",\"owner\":\"{}\",\"type\":\"{}\"}}",
                    json_escape(&db.name),
                    json_escape(&db.owner),
                    json_escape(&db.db_kind)
                )?;
            }
            writeln!(writer, "]")?;
        }
        OutputFormat::Csv => {
            writeln!(writer, "DatabaseName,Owner,Type")?;
            for db in &databases {
                writeln!(
                    writer,
                    "{},{},{}",
                    csv_escape(&db.name),
                    csv_escape(&db.owner),
                    csv_escape(&db.db_kind)
                )?;
            }
        }
    }

    Ok(())
}

// =============================================================================
// Tables (enriched with row count estimate and size)
// =============================================================================

/// Table entry with metadata
struct TableEntry {
    name: String,
    kind: String,
    row_count: String,
    size: String,
}

fn list_tables<W: Write>(
    client: &DatabaseClient,
    pattern: Option<&str>,
    database: Option<&str>,
    format: OutputFormat,
    writer: &mut W,
) -> Result<()> {
    let db_clause = if let Some(db) = database {
        format!("t.DatabaseName = '{}'", escape_sql_string(db))
    } else {
        "t.DatabaseName = DATABASE".to_string()
    };

    // Query with size from TableSizeV (LEFT JOIN for tables without size info)
    let sql = format!(
        "SELECT TRIM(t.TableName) AS table_name, t.TableKind, \
         COALESCE(CAST(s.RowCount AS VARCHAR(20)), '') AS RowCount, \
         COALESCE(CAST(s.CurrentPerm AS VARCHAR(20)), '') AS CurrentPerm \
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

            let row_count = row
                .get(2)
                .map(|v| {
                    let s = v.display().trim().to_string();
                    if s.is_empty() || s == "[NULL]" {
                        "-".to_string()
                    } else {
                        s
                    }
                })
                .unwrap_or_else(|| "-".to_string());

            let size_bytes = row
                .get(3)
                .map(|v| {
                    let s = v.display().trim().to_string();
                    if s.is_empty() || s == "[NULL]" {
                        return "-".to_string();
                    }
                    match s.parse::<i64>() {
                        Ok(b) => format_size_short(b),
                        Err(_) => s,
                    }
                })
                .unwrap_or_else(|| "-".to_string());

            Some(TableEntry {
                name: name.trim().to_string(),
                kind: kind_str,
                row_count,
                size: size_bytes,
            })
        })
        .collect();

    let db_label = database.unwrap_or("(current)");
    let pattern_str = pattern
        .map(|p| format!(" matching '{}'", p))
        .unwrap_or_default();

    match format {
        OutputFormat::Table => {
            writeln!(writer, "Tables in {}{}:", db_label, pattern_str)?;
            writeln!(
                writer,
                "{:<35} {:<8} {:<12} {:<10}",
                "Name", "Type", "Rows (Est.)", "Size"
            )?;
            writeln!(writer, "{}", "-".repeat(65))?;

            for t in &tables {
                writeln!(
                    writer,
                    "{:<35} {:<8} {:>12} {:>10}",
                    t.name, t.kind, t.row_count, t.size
                )?;
            }

            writeln!(writer)?;
            writeln!(writer, "{} table(s)", tables.len())?;
        }
        OutputFormat::Json => {
            write!(writer, "[")?;
            for (i, t) in tables.iter().enumerate() {
                if i > 0 {
                    write!(writer, ",")?;
                }
                write!(
                    writer,
                    "{{\"name\":\"{}\",\"type\":\"{}\",\"rows_est\":\"{}\",\"size\":\"{}\"}}",
                    json_escape(&t.name),
                    json_escape(&t.kind),
                    json_escape(&t.row_count),
                    json_escape(&t.size)
                )?;
            }
            writeln!(writer, "]")?;
        }
        OutputFormat::Csv => {
            writeln!(writer, "TableName,Type,RowsEst,Size")?;
            for t in &tables {
                writeln!(
                    writer,
                    "{},{},{},{}",
                    csv_escape(&t.name),
                    csv_escape(&t.kind),
                    csv_escape(&t.row_count),
                    csv_escape(&t.size)
                )?;
            }
        }
    }

    Ok(())
}

// =============================================================================
// Views
// =============================================================================

fn list_views<W: Write>(
    client: &DatabaseClient,
    database: Option<&str>,
    format: OutputFormat,
    writer: &mut W,
) -> Result<()> {
    let db_clause = if let Some(db) = database {
        format!("DatabaseName = '{}'", escape_sql_string(db))
    } else {
        "DatabaseName = DATABASE".to_string()
    };

    let sql = format!(
        "SELECT TRIM(TableName) AS view_name \
         FROM DBC.TablesV \
         WHERE {} AND TableKind = 'V' \
         ORDER BY TableName",
        db_clause
    );

    let result = client.execute(&sql)?;

    let views: Vec<String> = result
        .rows
        .iter()
        .filter_map(|row| {
            let name = row.first().map(|v| v.display())?;
            if name == "[NULL]" {
                None
            } else {
                Some(name.trim().to_string())
            }
        })
        .collect();

    let db_label = database.unwrap_or("(current)");

    match format {
        OutputFormat::Table => {
            writeln!(writer, "Views in {}:", db_label)?;
            writeln!(writer, "{}", "-".repeat(40))?;

            if views.is_empty() {
                writeln!(writer, "(no views found)")?;
            } else {
                for view in &views {
                    writeln!(writer, "  {}", view)?;
                }
            }

            writeln!(writer)?;
            writeln!(writer, "{} view(s)", views.len())?;
        }
        OutputFormat::Json => {
            write!(writer, "[")?;
            for (i, view) in views.iter().enumerate() {
                if i > 0 {
                    write!(writer, ",")?;
                }
                write!(writer, "\"{}\"", json_escape(view))?;
            }
            writeln!(writer, "]")?;
        }
        OutputFormat::Csv => {
            writeln!(writer, "ViewName")?;
            for view in &views {
                writeln!(writer, "{}", csv_escape(view))?;
            }
        }
    }

    Ok(())
}

// =============================================================================
// Helpers
// =============================================================================

/// Format byte count as compact human-readable size
fn format_size_short(bytes: i64) -> String {
    if bytes < 0 {
        return format!("{} B", bytes);
    }

    const KB: i64 = 1024;
    const MB: i64 = 1024 * KB;
    const GB: i64 = 1024 * MB;
    const TB: i64 = 1024 * GB;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

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
        assert_eq!(format_size_short(0), "0 B");
        assert_eq!(format_size_short(512), "512 B");
        assert_eq!(format_size_short(1024), "1.0 KB");
        assert_eq!(format_size_short(1048576), "1.0 MB");
        assert_eq!(format_size_short(1073741824), "1.0 GB");
        assert_eq!(format_size_short(1099511627776), "1.0 TB");
        assert_eq!(format_size_short(-100), "-100 B");
    }

    #[test]
    fn test_database_entry_structure() {
        let db = DatabaseEntry {
            name: "mydb".to_string(),
            owner: "DBC".to_string(),
            db_kind: "Database".to_string(),
        };
        assert_eq!(db.name, "mydb");
        assert_eq!(db.db_kind, "Database");
    }

    #[test]
    fn test_table_entry_structure() {
        let t = TableEntry {
            name: "employees".to_string(),
            kind: "TABLE".to_string(),
            row_count: "1000".to_string(),
            size: "2.5 MB".to_string(),
        };
        assert_eq!(t.name, "employees");
        assert_eq!(t.row_count, "1000");
    }
}
