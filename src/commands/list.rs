//! List command implementation
//!
//! Lists database objects: databases, tables, or views.
//! Used by `tq list <type> [pattern]` (batch) and `/list` (REPL delegation).

use crate::cli::{ListObjectType, OutputFormat};
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

// =============================================================================
// Databases
// =============================================================================

fn list_databases<W: Write>(
    client: &DatabaseClient,
    format: OutputFormat,
    writer: &mut W,
) -> Result<()> {
    let sql = r#"
        SELECT TRIM(DatabaseName) AS database_name
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
    let databases: Vec<String> = result
        .rows
        .iter()
        .filter_map(|row| {
            let name = row.first().map(|v| v.display())?;
            if name == "[NULL]" {
                None
            } else {
                Some(name)
            }
        })
        .collect();

    match format {
        OutputFormat::Table => {
            writeln!(writer, "Databases ({}):", databases.len())?;
            writeln!(writer, "{}", "-".repeat(40))?;

            let col_width = 25;
            let cols = 3;
            for chunk in databases.chunks(cols) {
                let line: Vec<String> = chunk
                    .iter()
                    .map(|db| format!("{:<width$}", db, width = col_width))
                    .collect();
                writeln!(writer, "{}", line.join(""))?;
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
                write!(writer, "\"{}\"", json_escape(db))?;
            }
            writeln!(writer, "]")?;
        }
        OutputFormat::Csv => {
            writeln!(writer, "DatabaseName")?;
            for db in &databases {
                writeln!(writer, "{}", csv_escape(db))?;
            }
        }
    }

    Ok(())
}

// =============================================================================
// Tables
// =============================================================================

fn list_tables<W: Write>(
    client: &DatabaseClient,
    pattern: Option<&str>,
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
        "SELECT TRIM(TableName) AS table_name, TableKind \
         FROM DBC.TablesV \
         WHERE {} AND TableKind IN ('T', 'O') \
         ORDER BY TableName",
        db_clause
    );

    let result = client.execute(&sql)?;

    let tables: Vec<(String, &str)> = result
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

            let kind_str = if kind.trim() == "O" { "NoPI" } else { "TABLE" };
            Some((name, kind_str))
        })
        .collect();

    let db_label = database.unwrap_or("(current)");
    let pattern_str = pattern
        .map(|p| format!(" matching '{}'", p))
        .unwrap_or_default();

    match format {
        OutputFormat::Table => {
            writeln!(
                writer,
                "Tables in {}{}:",
                db_label, pattern_str
            )?;
            writeln!(writer, "{:<40} {:<10}", "Name", "Type")?;
            writeln!(writer, "{}", "-".repeat(50))?;

            for (name, kind) in &tables {
                writeln!(writer, "{:<40} {:<10}", name, kind)?;
            }

            writeln!(writer)?;
            writeln!(writer, "{} table(s)", tables.len())?;
        }
        OutputFormat::Json => {
            write!(writer, "[")?;
            for (i, (name, kind)) in tables.iter().enumerate() {
                if i > 0 {
                    write!(writer, ",")?;
                }
                write!(
                    writer,
                    "{{\"name\":\"{}\",\"type\":\"{}\"}}",
                    json_escape(name),
                    kind
                )?;
            }
            writeln!(writer, "]")?;
        }
        OutputFormat::Csv => {
            writeln!(writer, "TableName,Type")?;
            for (name, kind) in &tables {
                writeln!(writer, "{},{}", csv_escape(name), kind)?;
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
                Some(name)
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
}
