//! Search command implementation
//!
//! Searches for database objects across all accessible databases.
//! Used by `tq search <type> <keyword>` (batch) and `/search` (REPL delegation).

use crate::cli::{OutputFormat, SearchObjectType};
use crate::commands::format_helpers::{csv_escape, format_size, markdown_escape_pipe};
use crate::db::DatabaseClient;
use crate::error::Result;
use crate::pagination::PaginationInfo;
use crate::sql::escape_sql_string;
use serde::Serialize;
use std::io::Write;

/// Maximum number of rows to fetch when paginating client-side.
/// Used as a SQL TOP limit to avoid unbounded result sets.
const MAX_SEARCH_FETCH: usize = 100_000;

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
    // When page_size is set, don't apply SQL TOP limit - fetch all then paginate client-side
    let effective_limit = if args.page_size.is_some() {
        None
    } else {
        args.limit
    };

    let pagination_args = args.page_size.map(|ps| (ps, args.page));

    match args.object_type {
        SearchObjectType::Tables => search_tables(
            client,
            &args.keyword,
            args.database.as_deref(),
            args.format,
            effective_limit,
            pagination_args,
            writer,
        ),
        SearchObjectType::Columns => search_columns(
            client,
            &args.keyword,
            args.database.as_deref(),
            args.format,
            effective_limit,
            pagination_args,
            writer,
        ),
        SearchObjectType::Views => search_views(
            client,
            &args.keyword,
            args.database.as_deref(),
            args.format,
            effective_limit,
            pagination_args,
            writer,
        ),
        SearchObjectType::Procedures => search_procedures(
            client,
            &args.keyword,
            args.database.as_deref(),
            args.format,
            effective_limit,
            pagination_args,
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
            search_tables(client, keyword, database, OutputFormat::Table, None, None, writer)?;
        }
        "columns" | "column" | "col" | "c" => {
            search_columns(client, keyword, database, OutputFormat::Table, None, None, writer)?;
        }
        "views" | "view" | "v" => {
            search_views(client, keyword, database, OutputFormat::Table, None, None, writer)?;
        }
        "procedures" | "procs" | "proc" | "p" => {
            search_procedures(client, keyword, database, OutputFormat::Table, None, None, writer)?;
        }
        _ => {
            writeln!(writer, "Error: Unknown search subcommand: {}", subcommand)?;
            writeln!(writer, "Available: tables, columns, views, procedures")?;
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
    pagination_args: Option<(usize, usize)>,
    writer: &mut W,
) -> Result<()> {
    let escaped_keyword = escape_sql_string(keyword);

    let db_filter = if let Some(db) = database {
        format!("AND t.DatabaseName = '{}'", escape_sql_string(db))
    } else {
        String::new()
    };

    // When paginating, fetch all rows (use large limit); otherwise use specified limit
    let row_limit = if pagination_args.is_some() {
        MAX_SEARCH_FETCH
    } else {
        limit.unwrap_or(100)
    };

    // DBC.TableSizeV is not available on all systems; fall back to a simpler
    // query (no size/row-count columns) when it is inaccessible.
    let sql_with_size = format!(
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
    let db_filter_simple = if let Some(db) = database {
        format!("AND DatabaseName = '{}'", escape_sql_string(db))
    } else {
        String::new()
    };
    let sql_no_size = format!(
        "SELECT TOP {limit} TRIM(DatabaseName) AS db_name, \
         TRIM(TableName) AS table_name, TableKind, \
         '' AS RowCount, '' AS CurrentPerm, \
         TRIM(CreatorName) AS Owner \
         FROM DBC.TablesV \
         WHERE UPPER(TableName) LIKE UPPER('%{keyword}%') \
         AND TableKind IN ('T', 'O') \
         {db_filter} \
         ORDER BY DatabaseName, TableName",
        limit = row_limit,
        keyword = escaped_keyword,
        db_filter = db_filter_simple
    );

    let result = match client.execute(&sql_with_size) {
        Ok(r) => r,
        Err(_) => client.execute(&sql_no_size)?,
    };

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

    // Apply pagination if requested
    let pagination = pagination_args.map(|(page_size, page)| {
        PaginationInfo::new(page, page_size, tables.len())
    });

    let display_tables = if let Some(ref pg) = pagination {
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
            render_table_search_table(display_tables, keyword, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Json => render_table_search_json_with_pagination(display_tables, pagination.as_ref(), writer)?,
        OutputFormat::Csv => {
            render_table_search_csv(display_tables, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Markdown | OutputFormat::Md => {
            render_table_search_markdown(display_tables, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
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

/// Serde-serializable envelope for table search JSON output
#[derive(Serialize)]
struct TableSearchJsonEnvelope<'a> {
    ok: bool,
    row_count: usize,
    data: Vec<TableSearchJsonRow<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination: Option<PaginationJson>,
}

/// Single row in table search JSON output
#[derive(Serialize)]
struct TableSearchJsonRow<'a> {
    database: &'a str,
    table_name: &'a str,
    #[serde(rename = "type")]
    kind: &'a str,
    estimated_rows: Option<i64>,
    size_bytes: Option<i64>,
    owner: &'a str,
}

/// Pagination metadata for JSON output (shared across search types)
#[derive(Serialize)]
struct PaginationJson {
    page: usize,
    page_size: usize,
    total_rows: usize,
    total_pages: usize,
    has_more: bool,
}

impl PaginationJson {
    fn from_info(pg: &PaginationInfo) -> Self {
        PaginationJson {
            page: pg.page,
            page_size: pg.page_size,
            total_rows: pg.total_rows,
            total_pages: pg.total_pages(),
            has_more: pg.has_more(),
        }
    }
}

fn render_table_search_json_with_pagination<W: Write>(
    tables: &[TableSearchResult],
    pagination: Option<&PaginationInfo>,
    writer: &mut W,
) -> Result<()> {
    let envelope = TableSearchJsonEnvelope {
        ok: true,
        row_count: tables.len(),
        data: tables
            .iter()
            .map(|t| TableSearchJsonRow {
                database: &t.database,
                table_name: &t.table_name,
                kind: &t.kind,
                estimated_rows: t.row_count_raw,
                size_bytes: t.size_bytes,
                owner: &t.owner,
            })
            .collect(),
        pagination: pagination.map(PaginationJson::from_info),
    };
    serde_json::to_writer(&mut *writer, &envelope)?;
    writeln!(writer)?;
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
    writeln!(
        writer,
        "| Database | Name | Type | Rows (Est.) | Size | Owner |"
    )?;
    writeln!(writer, "| :--- | :--- | :--- | ---: | ---: | :--- |")?;
    for t in tables {
        writeln!(
            writer,
            "| {} | {} | {} | {} | {} | {} |",
            markdown_escape_pipe(&t.database),
            markdown_escape_pipe(&t.table_name),
            markdown_escape_pipe(&t.kind),
            markdown_escape_pipe(&t.row_count_display),
            markdown_escape_pipe(&t.size_display),
            markdown_escape_pipe(&t.owner)
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
    pagination_args: Option<(usize, usize)>,
    writer: &mut W,
) -> Result<()> {
    let escaped_keyword = escape_sql_string(keyword);

    let db_filter = if let Some(db) = database {
        format!("AND c.DatabaseName = '{}'", escape_sql_string(db))
    } else {
        String::new()
    };

    let row_limit = if pagination_args.is_some() {
        MAX_SEARCH_FETCH
    } else {
        limit.unwrap_or(100)
    };

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

    // Apply pagination if requested
    let pagination = pagination_args.map(|(page_size, page)| {
        PaginationInfo::new(page, page_size, columns.len())
    });

    let display_columns = if let Some(ref pg) = pagination {
        let (start, end) = pg.row_range();
        if start < columns.len() {
            &columns[start..end.min(columns.len())]
        } else {
            &columns[0..0]
        }
    } else {
        &columns[..]
    };

    match format {
        OutputFormat::Table => {
            render_column_search_table(display_columns, keyword, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Json => render_column_search_json_with_pagination(display_columns, pagination.as_ref(), writer)?,
        OutputFormat::Csv => {
            render_column_search_csv(display_columns, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Markdown | OutputFormat::Md => {
            render_column_search_markdown(display_columns, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
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

/// Serde-serializable envelope for column search JSON output
#[derive(Serialize)]
struct ColumnSearchJsonEnvelope<'a> {
    ok: bool,
    row_count: usize,
    data: Vec<ColumnSearchJsonRow<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination: Option<PaginationJson>,
}

/// Single row in column search JSON output
#[derive(Serialize)]
struct ColumnSearchJsonRow<'a> {
    database: &'a str,
    table_name: &'a str,
    column_name: &'a str,
    column_type: &'a str,
    nullable: &'a str,
}

fn render_column_search_json_with_pagination<W: Write>(
    columns: &[ColumnSearchResult],
    pagination: Option<&PaginationInfo>,
    writer: &mut W,
) -> Result<()> {
    let envelope = ColumnSearchJsonEnvelope {
        ok: true,
        row_count: columns.len(),
        data: columns
            .iter()
            .map(|c| ColumnSearchJsonRow {
                database: &c.database,
                table_name: &c.table_name,
                column_name: &c.column_name,
                column_type: &c.column_type,
                nullable: &c.nullable,
            })
            .collect(),
        pagination: pagination.map(PaginationJson::from_info),
    };
    serde_json::to_writer(&mut *writer, &envelope)?;
    writeln!(writer)?;
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
    writeln!(
        writer,
        "| Database | Table | Column | Type | Nullable |"
    )?;
    writeln!(writer, "| :--- | :--- | :--- | :--- | :--- |")?;
    for c in columns {
        writeln!(
            writer,
            "| {} | {} | {} | {} | {} |",
            markdown_escape_pipe(&c.database),
            markdown_escape_pipe(&c.table_name),
            markdown_escape_pipe(&c.column_name),
            markdown_escape_pipe(&c.column_type),
            markdown_escape_pipe(&c.nullable)
        )?;
    }
    Ok(())
}

// =============================================================================
// View Search
// =============================================================================

/// View search result entry
struct ViewSearchResult {
    database: String,
    view_name: String,
    owner: String,
}

fn search_views<W: Write>(
    client: &DatabaseClient,
    keyword: &str,
    database: Option<&str>,
    format: OutputFormat,
    limit: Option<usize>,
    pagination_args: Option<(usize, usize)>,
    writer: &mut W,
) -> Result<()> {
    let escaped_keyword = escape_sql_string(keyword);

    let db_filter = if let Some(db) = database {
        format!("AND t.DatabaseName = '{}'", escape_sql_string(db))
    } else {
        String::new()
    };

    let row_limit = if pagination_args.is_some() {
        MAX_SEARCH_FETCH
    } else {
        limit.unwrap_or(100)
    };

    let sql = format!(
        "SELECT TOP {limit} TRIM(t.DatabaseName) AS db_name, \
         TRIM(t.TableName) AS view_name, \
         TRIM(t.CreatorName) AS Owner \
         FROM DBC.TablesV t \
         WHERE UPPER(t.TableName) LIKE UPPER('%{keyword}%') \
         AND t.TableKind = 'V' \
         {db_filter} \
         ORDER BY t.DatabaseName, t.TableName",
        limit = row_limit,
        keyword = escaped_keyword,
        db_filter = db_filter
    );

    let result = client.execute(&sql)?;

    let views: Vec<ViewSearchResult> = result
        .rows
        .iter()
        .filter_map(|row| {
            let database = row.first().map(|v| v.display())?;
            let view_name = row.get(1).map(|v| v.display())?;

            if database == "[NULL]" || view_name == "[NULL]" {
                return None;
            }

            let owner = row
                .get(2)
                .map(|v| {
                    let s = v.display().trim().to_string();
                    if s == "[NULL]" {
                        String::new()
                    } else {
                        s
                    }
                })
                .unwrap_or_default();

            Some(ViewSearchResult {
                database: database.trim().to_string(),
                view_name: view_name.trim().to_string(),
                owner,
            })
        })
        .collect();

    // Apply pagination if requested
    let pagination = pagination_args.map(|(page_size, page)| {
        PaginationInfo::new(page, page_size, views.len())
    });

    let display_views = if let Some(ref pg) = pagination {
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
            render_view_search_table(display_views, keyword, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Json => render_view_search_json_with_pagination(display_views, pagination.as_ref(), writer)?,
        OutputFormat::Csv => {
            render_view_search_csv(display_views, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Markdown | OutputFormat::Md => {
            render_view_search_markdown(display_views, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
    }

    Ok(())
}

fn render_view_search_table<W: Write>(
    views: &[ViewSearchResult],
    keyword: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(
        writer,
        "Views matching '{}' ({}):",
        keyword,
        views.len()
    )?;
    writeln!(
        writer,
        "{:<20} {:<30} {:<15}",
        "Database", "Name", "Owner"
    )?;
    writeln!(writer, "{}", "-".repeat(67))?;

    if views.is_empty() {
        writeln!(writer, "(no views found)")?;
    } else {
        for v in views {
            writeln!(
                writer,
                "{:<20} {:<30} {:<15}",
                v.database, v.view_name, v.owner
            )?;
        }
    }

    writeln!(writer)?;
    writeln!(writer, "{} view(s)", views.len())?;
    Ok(())
}

/// Serde-serializable envelope for view search JSON output
#[derive(Serialize)]
struct ViewSearchJsonEnvelope<'a> {
    ok: bool,
    row_count: usize,
    data: Vec<ViewSearchJsonRow<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination: Option<PaginationJson>,
}

/// Single row in view search JSON output
#[derive(Serialize)]
struct ViewSearchJsonRow<'a> {
    database: &'a str,
    view_name: &'a str,
    owner: &'a str,
}

fn render_view_search_json_with_pagination<W: Write>(
    views: &[ViewSearchResult],
    pagination: Option<&PaginationInfo>,
    writer: &mut W,
) -> Result<()> {
    let envelope = ViewSearchJsonEnvelope {
        ok: true,
        row_count: views.len(),
        data: views
            .iter()
            .map(|v| ViewSearchJsonRow {
                database: &v.database,
                view_name: &v.view_name,
                owner: &v.owner,
            })
            .collect(),
        pagination: pagination.map(PaginationJson::from_info),
    };
    serde_json::to_writer(&mut *writer, &envelope)?;
    writeln!(writer)?;
    Ok(())
}

fn render_view_search_csv<W: Write>(
    views: &[ViewSearchResult],
    writer: &mut W,
) -> Result<()> {
    writeln!(writer, "Database,ViewName,Owner")?;
    for v in views {
        writeln!(
            writer,
            "{},{},{}",
            csv_escape(&v.database),
            csv_escape(&v.view_name),
            csv_escape(&v.owner)
        )?;
    }
    Ok(())
}

fn render_view_search_markdown<W: Write>(
    views: &[ViewSearchResult],
    writer: &mut W,
) -> Result<()> {
    writeln!(writer, "| Database | Name | Owner |")?;
    writeln!(writer, "| :--- | :--- | :--- |")?;
    for v in views {
        writeln!(
            writer,
            "| {} | {} | {} |",
            markdown_escape_pipe(&v.database),
            markdown_escape_pipe(&v.view_name),
            markdown_escape_pipe(&v.owner)
        )?;
    }
    Ok(())
}

// =============================================================================
// Procedure Search
// =============================================================================

/// Procedure search result entry
struct ProcedureSearchResult {
    database: String,
    procedure_name: String,
    owner: String,
}

fn search_procedures<W: Write>(
    client: &DatabaseClient,
    keyword: &str,
    database: Option<&str>,
    format: OutputFormat,
    limit: Option<usize>,
    pagination_args: Option<(usize, usize)>,
    writer: &mut W,
) -> Result<()> {
    let escaped_keyword = escape_sql_string(keyword);

    let db_filter = if let Some(db) = database {
        format!("AND t.DatabaseName = '{}'", escape_sql_string(db))
    } else {
        String::new()
    };

    let row_limit = if pagination_args.is_some() {
        MAX_SEARCH_FETCH
    } else {
        limit.unwrap_or(100)
    };

    let sql = format!(
        "SELECT TOP {limit} TRIM(t.DatabaseName) AS db_name, \
         TRIM(t.TableName) AS proc_name, \
         TRIM(t.CreatorName) AS Owner \
         FROM DBC.TablesV t \
         WHERE UPPER(t.TableName) LIKE UPPER('%{keyword}%') \
         AND t.TableKind = 'P' \
         {db_filter} \
         ORDER BY t.DatabaseName, t.TableName",
        limit = row_limit,
        keyword = escaped_keyword,
        db_filter = db_filter
    );

    let result = client.execute(&sql)?;

    let procedures: Vec<ProcedureSearchResult> = result
        .rows
        .iter()
        .filter_map(|row| {
            let database = row.first().map(|v| v.display())?;
            let procedure_name = row.get(1).map(|v| v.display())?;

            if database == "[NULL]" || procedure_name == "[NULL]" {
                return None;
            }

            let owner = row
                .get(2)
                .map(|v| {
                    let s = v.display().trim().to_string();
                    if s == "[NULL]" {
                        String::new()
                    } else {
                        s
                    }
                })
                .unwrap_or_default();

            Some(ProcedureSearchResult {
                database: database.trim().to_string(),
                procedure_name: procedure_name.trim().to_string(),
                owner,
            })
        })
        .collect();

    // Apply pagination if requested
    let pagination = pagination_args.map(|(page_size, page)| {
        PaginationInfo::new(page, page_size, procedures.len())
    });

    let display_procs = if let Some(ref pg) = pagination {
        let (start, end) = pg.row_range();
        if start < procedures.len() {
            &procedures[start..end.min(procedures.len())]
        } else {
            &procedures[0..0]
        }
    } else {
        &procedures[..]
    };

    match format {
        OutputFormat::Table => {
            render_procedure_search_table(display_procs, keyword, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Json => render_procedure_search_json_with_pagination(display_procs, pagination.as_ref(), writer)?,
        OutputFormat::Csv => {
            render_procedure_search_csv(display_procs, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
        OutputFormat::Markdown | OutputFormat::Md => {
            render_procedure_search_markdown(display_procs, writer)?;
            if let Some(ref pg) = pagination {
                pg.write_footer(writer)?;
            }
        }
    }

    Ok(())
}

fn render_procedure_search_table<W: Write>(
    procedures: &[ProcedureSearchResult],
    keyword: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(
        writer,
        "Procedures matching '{}' ({}):",
        keyword,
        procedures.len()
    )?;
    writeln!(
        writer,
        "{:<20} {:<30} {:<15}",
        "Database", "Name", "Owner"
    )?;
    writeln!(writer, "{}", "-".repeat(67))?;

    if procedures.is_empty() {
        writeln!(writer, "(no procedures found)")?;
    } else {
        for p in procedures {
            writeln!(
                writer,
                "{:<20} {:<30} {:<15}",
                p.database, p.procedure_name, p.owner
            )?;
        }
    }

    writeln!(writer)?;
    writeln!(writer, "{} procedure(s)", procedures.len())?;
    Ok(())
}

/// Serde-serializable envelope for procedure search JSON output
#[derive(Serialize)]
struct ProcedureSearchJsonEnvelope<'a> {
    ok: bool,
    row_count: usize,
    data: Vec<ProcedureSearchJsonRow<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination: Option<PaginationJson>,
}

/// Single row in procedure search JSON output
#[derive(Serialize)]
struct ProcedureSearchJsonRow<'a> {
    database: &'a str,
    procedure_name: &'a str,
    owner: &'a str,
}

fn render_procedure_search_json_with_pagination<W: Write>(
    procedures: &[ProcedureSearchResult],
    pagination: Option<&PaginationInfo>,
    writer: &mut W,
) -> Result<()> {
    let envelope = ProcedureSearchJsonEnvelope {
        ok: true,
        row_count: procedures.len(),
        data: procedures
            .iter()
            .map(|p| ProcedureSearchJsonRow {
                database: &p.database,
                procedure_name: &p.procedure_name,
                owner: &p.owner,
            })
            .collect(),
        pagination: pagination.map(PaginationJson::from_info),
    };
    serde_json::to_writer(&mut *writer, &envelope)?;
    writeln!(writer)?;
    Ok(())
}

fn render_procedure_search_csv<W: Write>(
    procedures: &[ProcedureSearchResult],
    writer: &mut W,
) -> Result<()> {
    writeln!(writer, "Database,ProcedureName,Owner")?;
    for p in procedures {
        writeln!(
            writer,
            "{},{},{}",
            csv_escape(&p.database),
            csv_escape(&p.procedure_name),
            csv_escape(&p.owner)
        )?;
    }
    Ok(())
}

fn render_procedure_search_markdown<W: Write>(
    procedures: &[ProcedureSearchResult],
    writer: &mut W,
) -> Result<()> {
    writeln!(writer, "| Database | Name | Owner |")?;
    writeln!(writer, "| :--- | :--- | :--- |")?;
    for p in procedures {
        writeln!(
            writer,
            "| {} | {} | {} |",
            markdown_escape_pipe(&p.database),
            markdown_escape_pipe(&p.procedure_name),
            markdown_escape_pipe(&p.owner)
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
        render_table_search_json_with_pagination(&tables, None, &mut buf).unwrap();
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
        render_table_search_json_with_pagination(&tables, None, &mut buf).unwrap();
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
        render_table_search_json_with_pagination(&tables, None, &mut buf).unwrap();
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
        render_column_search_json_with_pagination(&columns, None, &mut buf).unwrap();
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
        render_column_search_json_with_pagination(&columns, None, &mut buf).unwrap();
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
        render_table_search_json_with_pagination(&tables, None, &mut buf).unwrap();
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
        render_column_search_json_with_pagination(&columns, None, &mut buf2).unwrap();
        let output2 = String::from_utf8(buf2).unwrap();
        assert!(output2.contains("\"ok\":true"));
        assert!(output2.contains("\"row_count\":1"));
        assert!(output2.contains("\"data\":[{"));
    }

    // =========================================================================
    // Pagination tests for search renderers
    // =========================================================================

    #[test]
    fn test_table_search_json_with_pagination() {
        let tables = vec![
            TableSearchResult {
                database: "a".to_string(),
                table_name: "t1".to_string(),
                kind: "TABLE".to_string(),
                row_count_display: "10".to_string(),
                row_count_raw: Some(10),
                size_display: "1 KB".to_string(),
                size_bytes: Some(1024),
                owner: "admin".to_string(),
            },
        ];
        let pg = PaginationInfo::new(1, 5, 10);
        let mut buf = Vec::new();
        render_table_search_json_with_pagination(&tables, Some(&pg), &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("\"pagination\":{"));
        assert!(output.contains("\"page\":1"));
        assert!(output.contains("\"page_size\":5"));
        assert!(output.contains("\"total_rows\":10"));
        assert!(output.contains("\"total_pages\":2"));
        assert!(output.contains("\"has_more\":true"));
    }

    #[test]
    fn test_column_search_json_with_pagination() {
        let columns = vec![
            ColumnSearchResult {
                database: "db".to_string(),
                table_name: "tbl".to_string(),
                column_name: "col".to_string(),
                column_type: "INTEGER".to_string(),
                nullable: "N".to_string(),
            },
        ];
        let pg = PaginationInfo::new(2, 5, 7);
        let mut buf = Vec::new();
        render_column_search_json_with_pagination(&columns, Some(&pg), &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("\"pagination\":{"));
        assert!(output.contains("\"page\":2"));
        assert!(output.contains("\"has_more\":false"));
    }

    #[test]
    fn test_table_search_json_no_pagination() {
        let tables: Vec<TableSearchResult> = vec![];
        let mut buf = Vec::new();
        render_table_search_json_with_pagination(&tables, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.contains("\"pagination\""));
    }

    // =========================================================================
    // Sprint 57: Serde JSON edge case tests
    // =========================================================================

    #[test]
    fn test_table_search_json_special_chars() {
        let tables = vec![TableSearchResult {
            database: "my\"db".to_string(),
            table_name: "emp\\table".to_string(),
            kind: "TABLE".to_string(),
            row_count_display: "10".to_string(),
            row_count_raw: Some(10),
            size_display: "1 KB".to_string(),
            size_bytes: Some(1024),
            owner: "admin".to_string(),
        }];
        let mut buf = Vec::new();
        render_table_search_json_with_pagination(&tables, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Must parse as valid JSON
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["data"][0]["database"], "my\"db");
        assert_eq!(v["data"][0]["table_name"], "emp\\table");
    }

    #[test]
    fn test_column_search_json_special_chars() {
        let columns = vec![ColumnSearchResult {
            database: "db".to_string(),
            table_name: "tbl".to_string(),
            column_name: "col\"name".to_string(),
            column_type: "VARCHAR(100)".to_string(),
            nullable: "Y".to_string(),
        }];
        let mut buf = Vec::new();
        render_column_search_json_with_pagination(&columns, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["data"][0]["column_name"], "col\"name");
    }

    #[test]
    fn test_table_search_json_multi_row_count() {
        let tables = vec![
            TableSearchResult {
                database: "a".to_string(),
                table_name: "t1".to_string(),
                kind: "TABLE".to_string(),
                row_count_display: "1".to_string(),
                row_count_raw: Some(1),
                size_display: "1 B".to_string(),
                size_bytes: Some(1),
                owner: "x".to_string(),
            },
            TableSearchResult {
                database: "b".to_string(),
                table_name: "t2".to_string(),
                kind: "NoPI".to_string(),
                row_count_display: "-".to_string(),
                row_count_raw: None,
                size_display: "-".to_string(),
                size_bytes: None,
                owner: "y".to_string(),
            },
            TableSearchResult {
                database: "c".to_string(),
                table_name: "t3".to_string(),
                kind: "TABLE".to_string(),
                row_count_display: "99".to_string(),
                row_count_raw: Some(99),
                size_display: "5 KB".to_string(),
                size_bytes: Some(5120),
                owner: "z".to_string(),
            },
        ];
        let mut buf = Vec::new();
        render_table_search_json_with_pagination(&tables, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["row_count"], 3);
        assert_eq!(v["data"].as_array().unwrap().len(), 3);
        assert!(v["data"][1]["estimated_rows"].is_null());
        assert!(v["data"][1]["size_bytes"].is_null());
    }

    // =========================================================================
    // Sprint 57: View search tests
    // =========================================================================

    #[test]
    fn test_view_search_result_structure() {
        let v = ViewSearchResult {
            database: "mydb".to_string(),
            view_name: "emp_summary".to_string(),
            owner: "admin".to_string(),
        };
        assert_eq!(v.database, "mydb");
        assert_eq!(v.view_name, "emp_summary");
        assert_eq!(v.owner, "admin");
    }

    #[test]
    fn test_render_view_search_table_format() {
        let views = vec![
            ViewSearchResult {
                database: "hr".to_string(),
                view_name: "emp_summary".to_string(),
                owner: "alice".to_string(),
            },
            ViewSearchResult {
                database: "reporting".to_string(),
                view_name: "emp_report".to_string(),
                owner: "bob".to_string(),
            },
        ];
        let mut buf = Vec::new();
        render_view_search_table(&views, "emp", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Views matching 'emp' (2):"));
        assert!(output.contains("Database"));
        assert!(output.contains("Name"));
        assert!(output.contains("Owner"));
        assert!(output.contains("hr"));
        assert!(output.contains("emp_summary"));
        assert!(output.contains("reporting"));
        assert!(output.contains("emp_report"));
        assert!(output.contains("2 view(s)"));
    }

    #[test]
    fn test_render_view_search_table_empty() {
        let views: Vec<ViewSearchResult> = vec![];
        let mut buf = Vec::new();
        render_view_search_table(&views, "xyz", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Views matching 'xyz' (0):"));
        assert!(output.contains("(no views found)"));
        assert!(output.contains("0 view(s)"));
    }

    #[test]
    fn test_render_view_search_json() {
        let views = vec![ViewSearchResult {
            database: "hr".to_string(),
            view_name: "emp_summary".to_string(),
            owner: "alice".to_string(),
        }];
        let mut buf = Vec::new();
        render_view_search_json_with_pagination(&views, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["row_count"], 1);
        assert_eq!(v["data"][0]["database"], "hr");
        assert_eq!(v["data"][0]["view_name"], "emp_summary");
        assert_eq!(v["data"][0]["owner"], "alice");
        assert!(v.get("pagination").is_none());
    }

    #[test]
    fn test_render_view_search_json_empty() {
        let views: Vec<ViewSearchResult> = vec![];
        let mut buf = Vec::new();
        render_view_search_json_with_pagination(&views, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["row_count"], 0);
        assert_eq!(v["data"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_render_view_search_json_with_pagination() {
        let views = vec![ViewSearchResult {
            database: "hr".to_string(),
            view_name: "v1".to_string(),
            owner: "admin".to_string(),
        }];
        let pg = PaginationInfo::new(1, 5, 10);
        let mut buf = Vec::new();
        render_view_search_json_with_pagination(&views, Some(&pg), &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["pagination"]["page"], 1);
        assert_eq!(v["pagination"]["page_size"], 5);
        assert_eq!(v["pagination"]["total_rows"], 10);
        assert_eq!(v["pagination"]["total_pages"], 2);
        assert_eq!(v["pagination"]["has_more"], true);
    }

    #[test]
    fn test_render_view_search_json_special_chars() {
        let views = vec![ViewSearchResult {
            database: "my\"db".to_string(),
            view_name: "view\\name".to_string(),
            owner: "admin".to_string(),
        }];
        let mut buf = Vec::new();
        render_view_search_json_with_pagination(&views, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["data"][0]["database"], "my\"db");
        assert_eq!(v["data"][0]["view_name"], "view\\name");
    }

    #[test]
    fn test_render_view_search_csv() {
        let views = vec![ViewSearchResult {
            database: "hr".to_string(),
            view_name: "emp_summary".to_string(),
            owner: "alice".to_string(),
        }];
        let mut buf = Vec::new();
        render_view_search_csv(&views, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Database,ViewName,Owner"));
        assert!(output.contains("hr,emp_summary,alice"));
    }

    #[test]
    fn test_render_view_search_markdown() {
        let views = vec![ViewSearchResult {
            database: "hr".to_string(),
            view_name: "emp_summary".to_string(),
            owner: "alice".to_string(),
        }];
        let mut buf = Vec::new();
        render_view_search_markdown(&views, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("| Database | Name | Owner |"));
        assert!(output.contains("| :--- | :--- | :--- |"));
        assert!(output.contains("| hr | emp_summary | alice |"));
    }

    // =========================================================================
    // Procedure search tests
    // =========================================================================

    #[test]
    fn test_procedure_search_result_structure() {
        let p = ProcedureSearchResult {
            database: "mydb".to_string(),
            procedure_name: "update_salary".to_string(),
            owner: "admin".to_string(),
        };
        assert_eq!(p.database, "mydb");
        assert_eq!(p.procedure_name, "update_salary");
        assert_eq!(p.owner, "admin");
    }

    #[test]
    fn test_render_procedure_search_table_format() {
        let procs = vec![
            ProcedureSearchResult {
                database: "hr".to_string(),
                procedure_name: "update_salary".to_string(),
                owner: "alice".to_string(),
            },
            ProcedureSearchResult {
                database: "payroll".to_string(),
                procedure_name: "calc_bonus".to_string(),
                owner: "bob".to_string(),
            },
        ];
        let mut buf = Vec::new();
        render_procedure_search_table(&procs, "sal", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Procedures matching 'sal' (2):"));
        assert!(output.contains("Database"));
        assert!(output.contains("Name"));
        assert!(output.contains("Owner"));
        assert!(output.contains("hr"));
        assert!(output.contains("update_salary"));
        assert!(output.contains("payroll"));
        assert!(output.contains("calc_bonus"));
        assert!(output.contains("2 procedure(s)"));
    }

    #[test]
    fn test_render_procedure_search_table_empty() {
        let procs: Vec<ProcedureSearchResult> = vec![];
        let mut buf = Vec::new();
        render_procedure_search_table(&procs, "xyz", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Procedures matching 'xyz' (0):"));
        assert!(output.contains("(no procedures found)"));
        assert!(output.contains("0 procedure(s)"));
    }

    #[test]
    fn test_render_procedure_search_json() {
        let procs = vec![ProcedureSearchResult {
            database: "hr".to_string(),
            procedure_name: "update_salary".to_string(),
            owner: "alice".to_string(),
        }];
        let mut buf = Vec::new();
        render_procedure_search_json_with_pagination(&procs, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["row_count"], 1);
        assert_eq!(v["data"][0]["database"], "hr");
        assert_eq!(v["data"][0]["procedure_name"], "update_salary");
        assert_eq!(v["data"][0]["owner"], "alice");
        assert!(v.get("pagination").is_none());
    }

    #[test]
    fn test_render_procedure_search_json_empty() {
        let procs: Vec<ProcedureSearchResult> = vec![];
        let mut buf = Vec::new();
        render_procedure_search_json_with_pagination(&procs, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["row_count"], 0);
        assert_eq!(v["data"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_render_procedure_search_json_with_pagination() {
        let procs = vec![ProcedureSearchResult {
            database: "hr".to_string(),
            procedure_name: "sp1".to_string(),
            owner: "admin".to_string(),
        }];
        let pg = PaginationInfo::new(1, 5, 10);
        let mut buf = Vec::new();
        render_procedure_search_json_with_pagination(&procs, Some(&pg), &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["pagination"]["page"], 1);
        assert_eq!(v["pagination"]["page_size"], 5);
        assert_eq!(v["pagination"]["total_rows"], 10);
        assert_eq!(v["pagination"]["total_pages"], 2);
        assert_eq!(v["pagination"]["has_more"], true);
    }

    #[test]
    fn test_render_procedure_search_json_special_chars() {
        let procs = vec![ProcedureSearchResult {
            database: "my\"db".to_string(),
            procedure_name: "proc\\name".to_string(),
            owner: "admin".to_string(),
        }];
        let mut buf = Vec::new();
        render_procedure_search_json_with_pagination(&procs, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["data"][0]["database"], "my\"db");
        assert_eq!(v["data"][0]["procedure_name"], "proc\\name");
    }

    #[test]
    fn test_render_procedure_search_csv() {
        let procs = vec![ProcedureSearchResult {
            database: "hr".to_string(),
            procedure_name: "update_salary".to_string(),
            owner: "alice".to_string(),
        }];
        let mut buf = Vec::new();
        render_procedure_search_csv(&procs, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Database,ProcedureName,Owner"));
        assert!(output.contains("hr,update_salary,alice"));
    }

    #[test]
    fn test_render_procedure_search_markdown() {
        let procs = vec![ProcedureSearchResult {
            database: "hr".to_string(),
            procedure_name: "update_salary".to_string(),
            owner: "alice".to_string(),
        }];
        let mut buf = Vec::new();
        render_procedure_search_markdown(&procs, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("| Database | Name | Owner |"));
        assert!(output.contains("| :--- | :--- | :--- |"));
        assert!(output.contains("| hr | update_salary | alice |"));
    }
}
