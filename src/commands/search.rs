//! Search command implementation
//!
//! Searches for database objects across all accessible databases.
//! Used by `tq search <type> <keyword>` (batch) and `/search` (REPL delegation).

use crate::cli::{OutputFormat, SearchObjectType};
use crate::commands::format_helpers::{csv_escape, format_size, json_escape};
use crate::db::DatabaseClient;
use crate::error::Result;
use crate::sql::escape_sql_string;
use std::io::Write;

// =============================================================================
// Public API
// =============================================================================

/// Execute `tq search` in batch mode with format selection
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &crate::cli::SearchArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    match args.object_type {
        SearchObjectType::Tables => search_tables(
            client,
            &args.keyword,
            args.database.as_deref(),
            args.format,
            args.limit,
            writer,
        ),
        SearchObjectType::Columns => search_columns(
            client,
            &args.keyword,
            args.database.as_deref(),
            args.format,
            args.limit,
            writer,
        ),
    }
}

/// Execute /search in REPL mode (table format with extra spacing)
///
/// The REPL calls this with a subcommand string and keyword.
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    subcommand: &str,
    keyword: &str,
    database: Option<&str>,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;
    match subcommand {
        "tables" | "table" | "t" => {
            search_tables(client, keyword, database, OutputFormat::Table, None, writer)?;
        }
        "columns" | "column" | "col" | "c" => {
            search_columns(client, keyword, database, OutputFormat::Table, None, writer)?;
        }
        _ => {
            writeln!(writer, "Error: Unknown search subcommand: {}", subcommand)?;
            writeln!(writer, "Available: tables, columns")?;
        }
    }
    writeln!(writer)?;
    Ok(())
}

// =============================================================================
// Table Search
// =============================================================================

/// Table search result entry
struct TableSearchResult {
    database: String,
    table_name: String,
    kind: String,
    row_count_display: String,
    row_count_raw: Option<i64>,
    size_display: String,
    size_bytes: Option<i64>,
    owner: String,
}

fn search_tables<W: Write>(
    client: &DatabaseClient,
    keyword: &str,
    database: Option<&str>,
    format: OutputFormat,
    limit: Option<usize>,
    writer: &mut W,
) -> Result<()> {
    let escaped_keyword = escape_sql_string(keyword);

    let db_filter = if let Some(db) = database {
        format!("AND t.DatabaseName = '{}'", escape_sql_string(db))
    } else {
        String::new()
    };

    let row_limit = limit.unwrap_or(100);

    let sql = format!(
        "SELECT TOP {limit} TRIM(t.DatabaseName) AS db_name, \
         TRIM(t.TableName) AS table_name, t.TableKind, \
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
         WHERE UPPER(t.TableName) LIKE UPPER('%{keyword}%') \
         AND t.TableKind IN ('T', 'O') \
         {db_filter} \
         ORDER BY t.DatabaseName, t.TableName",
        limit = row_limit,
        keyword = escaped_keyword,
        db_filter = db_filter
    );

    let result = client.execute(&sql)?;

    let tables: Vec<TableSearchResult> = result
        .rows
        .iter()
        .filter_map(|row| {
            let database = row.first().map(|v| v.display())?;
            let table_name = row.get(1).map(|v| v.display())?;
            let kind = row.get(2).map(|v| v.display()).unwrap_or_default();

            if database == "[NULL]" || table_name == "[NULL]" {
                return None;
            }

            let kind_str = if kind.trim() == "O" {
                "NoPI".to_string()
            } else {
                "TABLE".to_string()
            };

            let row_count_raw = row.get(3).and_then(|v| {
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

            let size_bytes = row.get(4).and_then(|v| {
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
                .get(5)
                .map(|v| {
                    let s = v.display().trim().to_string();
                    if s == "[NULL]" {
                        String::new()
                    } else {
                        s
                    }
                })
                .unwrap_or_default();

            Some(TableSearchResult {
                database: database.trim().to_string(),
                table_name: table_name.trim().to_string(),
                kind: kind_str,
                row_count_display,
                row_count_raw,
                size_display,
                size_bytes,
                owner,
            })
        })
        .collect();

    match format {
        OutputFormat::Table => render_table_search_table(&tables, keyword, writer)?,
        OutputFormat::Json => render_table_search_json(&tables, writer)?,
        OutputFormat::Csv => render_table_search_csv(&tables, writer)?,
        OutputFormat::Markdown | OutputFormat::Md => {
            render_table_search_markdown(&tables, writer)?;
        }
    }

    Ok(())
}

fn render_table_search_table<W: Write>(
    tables: &[TableSearchResult],
    keyword: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(
        writer,
        "Tables matching '{}' ({}):",
        keyword,
        tables.len()
    )?;
    writeln!(
        writer,
        "{:<20} {:<30} {:<8} {:>12} {:>10} {:<15}",
        "Database", "Name", "Type", "Rows (Est.)", "Size", "Owner"
    )?;
    writeln!(writer, "{}", "-".repeat(98))?;

    if tables.is_empty() {
        writeln!(writer, "(no tables found)")?;
    } else {
        for t in tables {
            writeln!(
                writer,
                "{:<20} {:<30} {:<8} {:>12} {:>10} {:<15}",
                t.database, t.table_name, t.kind, t.row_count_display, t.size_display, t.owner
            )?;
        }
    }

    writeln!(writer)?;
    writeln!(writer, "{} table(s)", tables.len())?;
    Ok(())
}

fn render_table_search_json<W: Write>(
    tables: &[TableSearchResult],
    writer: &mut W,
) -> Result<()> {
    write!(
        writer,
        "{{\"ok\":true,\"row_count\":{},\"data\":[",
        tables.len()
    )?;
    for (i, t) in tables.iter().enumerate() {
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
            "{{\"database\":\"{}\",\"table_name\":\"{}\",\"type\":\"{}\",\"estimated_rows\":{},\"size_bytes\":{},\"owner\":\"{}\"}}",
            json_escape(&t.database),
            json_escape(&t.table_name),
            json_escape(&t.kind),
            rows_json,
            size_json,
            json_escape(&t.owner)
        )?;
    }
    writeln!(writer, "]}}")?;
    Ok(())
}

fn render_table_search_csv<W: Write>(
    tables: &[TableSearchResult],
    writer: &mut W,
) -> Result<()> {
    writeln!(writer, "Database,TableName,Type,RowsEst,Size,Owner")?;
    for t in tables {
        writeln!(
            writer,
            "{},{},{},{},{},{}",
            csv_escape(&t.database),
            csv_escape(&t.table_name),
            csv_escape(&t.kind),
            csv_escape(&t.row_count_display),
            csv_escape(&t.size_display),
            csv_escape(&t.owner)
        )?;
    }
    Ok(())
}

fn render_table_search_markdown<W: Write>(
    tables: &[TableSearchResult],
    writer: &mut W,
) -> Result<()> {
    fn esc(s: &str) -> String {
        s.replace('|', "\\|")
    }
    writeln!(
        writer,
        "| Database | Name | Type | Rows (Est.) | Size | Owner |"
    )?;
    writeln!(writer, "| :--- | :--- | :--- | ---: | ---: | :--- |")?;
    for t in tables {
        writeln!(
            writer,
            "| {} | {} | {} | {} | {} | {} |",
            esc(&t.database),
            esc(&t.table_name),
            esc(&t.kind),
            esc(&t.row_count_display),
            esc(&t.size_display),
            esc(&t.owner)
        )?;
    }
    Ok(())
}

// =============================================================================
// Column Search
// =============================================================================

/// Column search result entry
struct ColumnSearchResult {
    database: String,
    table_name: String,
    column_name: String,
    column_type: String,
    nullable: String,
}

fn search_columns<W: Write>(
    client: &DatabaseClient,
    keyword: &str,
    database: Option<&str>,
    format: OutputFormat,
    limit: Option<usize>,
    writer: &mut W,
) -> Result<()> {
    let escaped_keyword = escape_sql_string(keyword);

    let db_filter = if let Some(db) = database {
        format!("AND c.DatabaseName = '{}'", escape_sql_string(db))
    } else {
        String::new()
    };

    let row_limit = limit.unwrap_or(100);

    let sql = format!(
        "SELECT TOP {limit} TRIM(c.DatabaseName) AS db_name, \
         TRIM(c.TableName) AS table_name, \
         TRIM(c.ColumnName) AS column_name, \
         TRIM(c.ColumnType) AS col_type, \
         c.Nullable \
         FROM DBC.ColumnsV c \
         WHERE UPPER(c.ColumnName) LIKE UPPER('%{keyword}%') \
         {db_filter} \
         ORDER BY c.DatabaseName, c.TableName, c.ColumnName",
        limit = row_limit,
        keyword = escaped_keyword,
        db_filter = db_filter
    );

    let result = client.execute(&sql)?;

    let columns: Vec<ColumnSearchResult> = result
        .rows
        .iter()
        .filter_map(|row| {
            let database = row.first().map(|v| v.display())?;
            let table_name = row.get(1).map(|v| v.display())?;
            let column_name = row.get(2).map(|v| v.display())?;
            let col_type = row.get(3).map(|v| v.display()).unwrap_or_default();
            let nullable_raw = row.get(4).map(|v| v.display()).unwrap_or_default();

            if database == "[NULL]" || table_name == "[NULL]" || column_name == "[NULL]" {
                return None;
            }

            let nullable = match nullable_raw.trim().to_uppercase().as_str() {
                "Y" | "YES" => "Y".to_string(),
                _ => "N".to_string(),
            };

            Some(ColumnSearchResult {
                database: database.trim().to_string(),
                table_name: table_name.trim().to_string(),
                column_name: column_name.trim().to_string(),
                column_type: col_type.trim().to_string(),
                nullable,
            })
        })
        .collect();

    match format {
        OutputFormat::Table => render_column_search_table(&columns, keyword, writer)?,
        OutputFormat::Json => render_column_search_json(&columns, writer)?,
        OutputFormat::Csv => render_column_search_csv(&columns, writer)?,
        OutputFormat::Markdown | OutputFormat::Md => {
            render_column_search_markdown(&columns, writer)?;
        }
    }

    Ok(())
}

fn render_column_search_table<W: Write>(
    columns: &[ColumnSearchResult],
    keyword: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(
        writer,
        "Columns matching '{}' ({}):",
        keyword,
        columns.len()
    )?;
    writeln!(
        writer,
        "{:<20} {:<25} {:<25} {:<15} {:<8}",
        "Database", "Table", "Column", "Type", "Nullable"
    )?;
    writeln!(writer, "{}", "-".repeat(95))?;

    if columns.is_empty() {
        writeln!(writer, "(no columns found)")?;
    } else {
        for c in columns {
            writeln!(
                writer,
                "{:<20} {:<25} {:<25} {:<15} {:<8}",
                c.database, c.table_name, c.column_name, c.column_type, c.nullable
            )?;
        }
    }

    writeln!(writer)?;
    writeln!(writer, "{} column(s)", columns.len())?;
    Ok(())
}

fn render_column_search_json<W: Write>(
    columns: &[ColumnSearchResult],
    writer: &mut W,
) -> Result<()> {
    write!(
        writer,
        "{{\"ok\":true,\"row_count\":{},\"data\":[",
        columns.len()
    )?;
    for (i, c) in columns.iter().enumerate() {
        if i > 0 {
            write!(writer, ",")?;
        }
        write!(
            writer,
            "{{\"database\":\"{}\",\"table_name\":\"{}\",\"column_name\":\"{}\",\"column_type\":\"{}\",\"nullable\":\"{}\"}}",
            json_escape(&c.database),
            json_escape(&c.table_name),
            json_escape(&c.column_name),
            json_escape(&c.column_type),
            json_escape(&c.nullable)
        )?;
    }
    writeln!(writer, "]}}")?;
    Ok(())
}

fn render_column_search_csv<W: Write>(
    columns: &[ColumnSearchResult],
    writer: &mut W,
) -> Result<()> {
    writeln!(writer, "Database,TableName,ColumnName,ColumnType,Nullable")?;
    for c in columns {
        writeln!(
            writer,
            "{},{},{},{},{}",
            csv_escape(&c.database),
            csv_escape(&c.table_name),
            csv_escape(&c.column_name),
            csv_escape(&c.column_type),
            csv_escape(&c.nullable)
        )?;
    }
    Ok(())
}

fn render_column_search_markdown<W: Write>(
    columns: &[ColumnSearchResult],
    writer: &mut W,
) -> Result<()> {
    fn esc(s: &str) -> String {
        s.replace('|', "\\|")
    }
    writeln!(
        writer,
        "| Database | Table | Column | Type | Nullable |"
    )?;
    writeln!(writer, "| :--- | :--- | :--- | :--- | :--- |")?;
    for c in columns {
        writeln!(
            writer,
            "| {} | {} | {} | {} | {} |",
            esc(&c.database),
            esc(&c.table_name),
            esc(&c.column_name),
            esc(&c.column_type),
            esc(&c.nullable)
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
    fn test_table_search_result_structure() {
        let t = TableSearchResult {
            database: "mydb".to_string(),
            table_name: "employees".to_string(),
            kind: "TABLE".to_string(),
            row_count_display: "1000".to_string(),
            row_count_raw: Some(1000),
            size_display: "2.5 MB".to_string(),
            size_bytes: Some(2621440),
            owner: "admin".to_string(),
        };
        assert_eq!(t.database, "mydb");
        assert_eq!(t.table_name, "employees");
        assert_eq!(t.kind, "TABLE");
        assert_eq!(t.row_count_raw, Some(1000));
        assert_eq!(t.size_bytes, Some(2621440));
        assert_eq!(t.owner, "admin");
    }

    #[test]
    fn test_column_search_result_structure() {
        let c = ColumnSearchResult {
            database: "mydb".to_string(),
            table_name: "employees".to_string(),
            column_name: "salary".to_string(),
            column_type: "DECIMAL(10,2)".to_string(),
            nullable: "Y".to_string(),
        };
        assert_eq!(c.database, "mydb");
        assert_eq!(c.table_name, "employees");
        assert_eq!(c.column_name, "salary");
        assert_eq!(c.column_type, "DECIMAL(10,2)");
        assert_eq!(c.nullable, "Y");
    }

    #[test]
    fn test_render_table_search_table_format() {
        let tables = vec![
            TableSearchResult {
                database: "hr".to_string(),
                table_name: "employees".to_string(),
                kind: "TABLE".to_string(),
                row_count_display: "500".to_string(),
                row_count_raw: Some(500),
                size_display: "1.0 KB".to_string(),
                size_bytes: Some(1024),
                owner: "alice".to_string(),
            },
            TableSearchResult {
                database: "sales".to_string(),
                table_name: "emp_targets".to_string(),
                kind: "NoPI".to_string(),
                row_count_display: "-".to_string(),
                row_count_raw: None,
                size_display: "-".to_string(),
                size_bytes: None,
                owner: "bob".to_string(),
            },
        ];
        let mut buf = Vec::new();
        render_table_search_table(&tables, "emp", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Tables matching 'emp' (2):"));
        assert!(output.contains("Database"));
        assert!(output.contains("Name"));
        assert!(output.contains("Type"));
        assert!(output.contains("Rows (Est.)"));
        assert!(output.contains("Size"));
        assert!(output.contains("Owner"));
        assert!(output.contains("hr"));
        assert!(output.contains("employees"));
        assert!(output.contains("TABLE"));
        assert!(output.contains("sales"));
        assert!(output.contains("emp_targets"));
        assert!(output.contains("NoPI"));
        assert!(output.contains("2 table(s)"));
    }

    #[test]
    fn test_render_table_search_table_empty() {
        let tables: Vec<TableSearchResult> = vec![];
        let mut buf = Vec::new();
        render_table_search_table(&tables, "xyz", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Tables matching 'xyz' (0):"));
        assert!(output.contains("(no tables found)"));
        assert!(output.contains("0 table(s)"));
    }

    #[test]
    fn test_render_table_search_json() {
        let tables = vec![TableSearchResult {
            database: "hr".to_string(),
            table_name: "employees".to_string(),
            kind: "TABLE".to_string(),
            row_count_display: "500".to_string(),
            row_count_raw: Some(500),
            size_display: "1.0 KB".to_string(),
            size_bytes: Some(1024),
            owner: "alice".to_string(),
        }];
        let mut buf = Vec::new();
        render_table_search_json(&tables, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.starts_with("{\"ok\":true,\"row_count\":1,\"data\":["));
        assert!(output.contains("\"database\":\"hr\""));
        assert!(output.contains("\"table_name\":\"employees\""));
        assert!(output.contains("\"type\":\"TABLE\""));
        assert!(output.contains("\"estimated_rows\":500"));
        assert!(output.contains("\"size_bytes\":1024"));
        assert!(output.contains("\"owner\":\"alice\""));
        assert!(output.ends_with("]}\n"));
    }

    #[test]
    fn test_render_table_search_json_empty() {
        let tables: Vec<TableSearchResult> = vec![];
        let mut buf = Vec::new();
        render_table_search_json(&tables, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output, "{\"ok\":true,\"row_count\":0,\"data\":[]}\n");
    }

    #[test]
    fn test_render_table_search_json_null_values() {
        let tables = vec![TableSearchResult {
            database: "db".to_string(),
            table_name: "tbl".to_string(),
            kind: "TABLE".to_string(),
            row_count_display: "-".to_string(),
            row_count_raw: None,
            size_display: "-".to_string(),
            size_bytes: None,
            owner: String::new(),
        }];
        let mut buf = Vec::new();
        render_table_search_json(&tables, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("\"estimated_rows\":null"));
        assert!(output.contains("\"size_bytes\":null"));
    }

    #[test]
    fn test_render_table_search_csv() {
        let tables = vec![TableSearchResult {
            database: "hr".to_string(),
            table_name: "employees".to_string(),
            kind: "TABLE".to_string(),
            row_count_display: "500".to_string(),
            row_count_raw: Some(500),
            size_display: "1.0 KB".to_string(),
            size_bytes: Some(1024),
            owner: "alice".to_string(),
        }];
        let mut buf = Vec::new();
        render_table_search_csv(&tables, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Database,TableName,Type,RowsEst,Size,Owner"));
        assert!(output.contains("hr,employees,TABLE,500,1.0 KB,alice"));
    }

    #[test]
    fn test_render_table_search_markdown() {
        let tables = vec![TableSearchResult {
            database: "hr".to_string(),
            table_name: "employees".to_string(),
            kind: "TABLE".to_string(),
            row_count_display: "500".to_string(),
            row_count_raw: Some(500),
            size_display: "1.0 KB".to_string(),
            size_bytes: Some(1024),
            owner: "alice".to_string(),
        }];
        let mut buf = Vec::new();
        render_table_search_markdown(&tables, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("| Database | Name | Type | Rows (Est.) | Size | Owner |"));
        assert!(output.contains("| :--- | :--- | :--- | ---: | ---: | :--- |"));
        assert!(output.contains("| hr | employees | TABLE | 500 | 1.0 KB | alice |"));
    }

    #[test]
    fn test_render_column_search_table_format() {
        let columns = vec![
            ColumnSearchResult {
                database: "hr".to_string(),
                table_name: "employees".to_string(),
                column_name: "salary".to_string(),
                column_type: "DECIMAL(10,2)".to_string(),
                nullable: "Y".to_string(),
            },
            ColumnSearchResult {
                database: "hr".to_string(),
                table_name: "employees".to_string(),
                column_name: "base_salary".to_string(),
                column_type: "DECIMAL(8,2)".to_string(),
                nullable: "N".to_string(),
            },
        ];
        let mut buf = Vec::new();
        render_column_search_table(&columns, "salary", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Columns matching 'salary' (2):"));
        assert!(output.contains("Database"));
        assert!(output.contains("Table"));
        assert!(output.contains("Column"));
        assert!(output.contains("Type"));
        assert!(output.contains("Nullable"));
        assert!(output.contains("salary"));
        assert!(output.contains("base_salary"));
        assert!(output.contains("2 column(s)"));
    }

    #[test]
    fn test_render_column_search_table_empty() {
        let columns: Vec<ColumnSearchResult> = vec![];
        let mut buf = Vec::new();
        render_column_search_table(&columns, "xyz", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Columns matching 'xyz' (0):"));
        assert!(output.contains("(no columns found)"));
        assert!(output.contains("0 column(s)"));
    }

    #[test]
    fn test_render_column_search_json() {
        let columns = vec![ColumnSearchResult {
            database: "hr".to_string(),
            table_name: "employees".to_string(),
            column_name: "salary".to_string(),
            column_type: "DECIMAL(10,2)".to_string(),
            nullable: "Y".to_string(),
        }];
        let mut buf = Vec::new();
        render_column_search_json(&columns, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.starts_with("{\"ok\":true,\"row_count\":1,\"data\":["));
        assert!(output.contains("\"database\":\"hr\""));
        assert!(output.contains("\"table_name\":\"employees\""));
        assert!(output.contains("\"column_name\":\"salary\""));
        assert!(output.contains("\"column_type\":\"DECIMAL(10,2)\""));
        assert!(output.contains("\"nullable\":\"Y\""));
        assert!(output.ends_with("]}\n"));
    }

    #[test]
    fn test_render_column_search_json_empty() {
        let columns: Vec<ColumnSearchResult> = vec![];
        let mut buf = Vec::new();
        render_column_search_json(&columns, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output, "{\"ok\":true,\"row_count\":0,\"data\":[]}\n");
    }

    #[test]
    fn test_render_column_search_csv() {
        let columns = vec![ColumnSearchResult {
            database: "hr".to_string(),
            table_name: "employees".to_string(),
            column_name: "salary".to_string(),
            column_type: "DECIMAL(10,2)".to_string(),
            nullable: "Y".to_string(),
        }];
        let mut buf = Vec::new();
        render_column_search_csv(&columns, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Database,TableName,ColumnName,ColumnType,Nullable"));
        assert!(output.contains("hr,employees,salary,\"DECIMAL(10,2)\",Y"));
    }

    #[test]
    fn test_render_column_search_markdown() {
        let columns = vec![ColumnSearchResult {
            database: "hr".to_string(),
            table_name: "employees".to_string(),
            column_name: "salary".to_string(),
            column_type: "DECIMAL(10,2)".to_string(),
            nullable: "Y".to_string(),
        }];
        let mut buf = Vec::new();
        render_column_search_markdown(&columns, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("| Database | Table | Column | Type | Nullable |"));
        assert!(output.contains("| :--- | :--- | :--- | :--- | :--- |"));
        assert!(output.contains("| hr | employees | salary | DECIMAL(10,2) | Y |"));
    }

    #[test]
    fn test_json_envelope_structure() {
        // Verify table search JSON has correct envelope structure
        let tables = vec![
            TableSearchResult {
                database: "a".to_string(),
                table_name: "b".to_string(),
                kind: "TABLE".to_string(),
                row_count_display: "1".to_string(),
                row_count_raw: Some(1),
                size_display: "1 B".to_string(),
                size_bytes: Some(1),
                owner: "c".to_string(),
            },
            TableSearchResult {
                database: "d".to_string(),
                table_name: "e".to_string(),
                kind: "NoPI".to_string(),
                row_count_display: "-".to_string(),
                row_count_raw: None,
                size_display: "-".to_string(),
                size_bytes: None,
                owner: String::new(),
            },
        ];
        let mut buf = Vec::new();
        render_table_search_json(&tables, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // ok is true
        assert!(output.contains("\"ok\":true"));
        // row_count matches data array length
        assert!(output.contains("\"row_count\":2"));
        // data is an array
        assert!(output.contains("\"data\":[{"));

        // Verify column search JSON has correct envelope structure
        let columns = vec![ColumnSearchResult {
            database: "x".to_string(),
            table_name: "y".to_string(),
            column_name: "z".to_string(),
            column_type: "INTEGER".to_string(),
            nullable: "N".to_string(),
        }];
        let mut buf2 = Vec::new();
        render_column_search_json(&columns, &mut buf2).unwrap();
        let output2 = String::from_utf8(buf2).unwrap();
        assert!(output2.contains("\"ok\":true"));
        assert!(output2.contains("\"row_count\":1"));
        assert!(output2.contains("\"data\":[{"));
    }
}
