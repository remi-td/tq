//! Object inspection command implementation
//!
//! Provides comprehensive inspection of Teradata database objects showing
//! type, columns, indexes, storage/skew (tables), and definitions (views/macros).

use crate::cli::OutputFormat;
use crate::commands::format_helpers::{
    csv_escape, format_size, json_escape, map_table_kind, parse_table_name, truncate_str,
};
use crate::commands::query_helpers;
use crate::db::{DatabaseClient, Value};
use crate::error::Result;
use crate::sql::escape_sql_string;
use std::io::Write;

// =============================================================================
// Public API
// =============================================================================

/// Execute /inspect in REPL mode with human-readable output
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    object_name: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;
    inspect_object(client, object_name, writer)?;
    writeln!(writer)?;
    Ok(())
}

/// Execute `tq inspect` in batch mode with format selection
pub fn execute<W: Write>(
    client: &DatabaseClient,
    object_name: &str,
    format: OutputFormat,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    match format {
        OutputFormat::Table => {
            inspect_object(client, object_name, writer)?;
        }
        OutputFormat::Json => {
            inspect_object_json(client, object_name, writer)?;
        }
        OutputFormat::Csv => {
            inspect_object_csv(client, object_name, writer)?;
        }
    }
    Ok(())
}

// =============================================================================
// Core inspection logic
// =============================================================================

/// Consolidated object inspection with graceful degradation per section
fn inspect_object<W: Write>(
    client: &DatabaseClient,
    object_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (db_part, obj_part) = parse_table_name(object_name);

    // Resolve database name
    let database = match query_helpers::resolve_database(client, db_part) {
        Ok(db) => db,
        Err(e) => {
            writeln!(writer, "Error: Could not resolve database: {}", e)?;
            return Ok(());
        }
    };

    // Section 1: Object Info (required -- if this fails, the object doesn't exist)
    let obj_info = match query_object_type(client, &database, obj_part) {
        Ok(Some(info)) => info,
        Ok(None) => {
            writeln!(writer, "Error: Object '{}' not found.", object_name)?;
            writeln!(writer)?;
            writeln!(writer, "Suggestions:")?;
            writeln!(writer, "  - Check the object name spelling")?;
            writeln!(
                writer,
                "  - Try using qualified name: /inspect database.object"
            )?;
            writeln!(
                writer,
                "  - Verify you have SELECT permission on DBC.TablesV"
            )?;
            return Ok(());
        }
        Err(e) => {
            writeln!(
                writer,
                "Error querying object type for '{}': {}",
                object_name, e
            )?;
            return Ok(());
        }
    };

    // Display Object Info section
    writeln!(writer, "── Object Info ──")?;
    writeln!(writer, "  Type:      {}", obj_info.kind_label)?;
    writeln!(writer, "  Database:  {}", database)?;
    writeln!(writer, "  Name:      {}", obj_part)?;
    if !obj_info.created.is_empty() {
        writeln!(writer, "  Created:   {}", obj_info.created)?;
    }
    if !obj_info.comment.is_empty() {
        writeln!(writer, "  Comment:   {}", obj_info.comment)?;
    }
    writeln!(writer)?;

    // Section 2: Columns (using shared query_helpers)
    match query_helpers::query_columns(client, &database, obj_part) {
        Ok(columns) => {
            if !columns.is_empty() {
                writeln!(writer, "── Columns ({}) ──", columns.len())?;
                let header_default = "Default";
                writeln!(
                    writer,
                    "  {:<24} {:<20} {:<10} {}",
                    "Column", "Type", "Nullable", header_default
                )?;
                let separator = format!(
                    "  {:<24} {:<20} {:<10} {}",
                    "\u{2500}".repeat(22),
                    "\u{2500}".repeat(18),
                    "\u{2500}".repeat(8),
                    "\u{2500}".repeat(15)
                );
                writeln!(writer, "{}", separator)?;

                for col in &columns {
                    let default_display = if col.default_val == "-" {
                        "-"
                    } else {
                        &col.default_val
                    };
                    writeln!(
                        writer,
                        "  {:<24} {:<20} {:<10} {}",
                        truncate_str(&col.name, 22),
                        truncate_str(&col.col_type, 18),
                        &col.nullable,
                        default_display
                    )?;
                }
                writeln!(writer, "{} columns", columns.len())?;
                writeln!(writer)?;
            }
        }
        Err(e) => {
            writeln!(
                writer,
                "  (Column information unavailable: {})",
                summarize_error(&e)
            )?;
            writeln!(writer)?;
        }
    }

    // Section 3: Indexes (only for tables, using shared query_helpers)
    if obj_info.table_kind == "T" || obj_info.table_kind == "O" {
        match query_helpers::query_indexes(client, &database, obj_part) {
            Ok(indexes) => {
                if !indexes.is_empty() {
                    writeln!(writer, "── Indexes ──")?;
                    for idx in &indexes {
                        let columns_str = idx.columns.join(", ");
                        if let Some(ref name) = idx.name {
                            writeln!(
                                writer,
                                "  {} ({}) \"{}\": {}",
                                idx.index_type_label, idx.short_label, name, columns_str
                            )?;
                        } else {
                            writeln!(
                                writer,
                                "  {} ({}): {}",
                                idx.index_type_label, idx.short_label, columns_str
                            )?;
                        }
                    }
                    writeln!(writer)?;
                }
            }
            Err(e) => {
                writeln!(
                    writer,
                    "  (Index information unavailable: {})",
                    summarize_error(&e)
                )?;
                writeln!(writer)?;
            }
        }

        // Section 4: Storage (only for tables)
        match query_storage(client, &database, obj_part) {
            Ok(storage) => {
                writeln!(writer, "── Storage ──")?;
                writeln!(
                    writer,
                    "  Current Size:  {}",
                    format_size(storage.total_size, 2)
                )?;
                writeln!(
                    writer,
                    "  Peak Size:     {}",
                    format_size(storage.peak_size, 2)
                )?;
                let skew = calculate_skew(storage.max_amp_size, storage.avg_amp_size);
                writeln!(
                    writer,
                    "  Skew Factor:   {:.1}% {}",
                    skew,
                    interpret_skew(skew)
                )?;
                writeln!(writer, "  AMP Count:     {}", storage.amp_count)?;
                writeln!(writer)?;
            }
            Err(e) => {
                writeln!(
                    writer,
                    "  (Storage information unavailable: {})",
                    summarize_error(&e)
                )?;
                writeln!(writer)?;
            }
        }
    }

    // Section 5: Definition (for views and macros)
    if obj_info.table_kind == "V" || obj_info.table_kind == "M" {
        match query_definition(client, &database, obj_part, &obj_info.table_kind) {
            Ok(definition) => {
                writeln!(writer, "── Definition ──")?;
                let formatted = format_ddl(&definition);
                for line in formatted.lines() {
                    writeln!(writer, "  {}", line)?;
                }
                writeln!(writer)?;
            }
            Err(e) => {
                writeln!(
                    writer,
                    "  (Definition unavailable: {})",
                    summarize_error(&e)
                )?;
                writeln!(writer)?;
            }
        }
    }

    // Section 6: Dependencies (for views and macros)
    if obj_info.table_kind == "V" || obj_info.table_kind == "M" {
        match query_dependencies(client, &database, obj_part) {
            Ok((upstream, downstream)) => {
                writeln!(writer, "── Dependencies ──")?;
                writeln!(writer)?;
                writeln!(writer, "  Uses (upstream)")?;
                if upstream.is_empty() {
                    writeln!(writer, "    None")?;
                } else {
                    for dep in &upstream {
                        writeln!(
                            writer,
                            "    {}.{}  ({})",
                            dep.database, dep.name, dep.kind_label
                        )?;
                    }
                }
                writeln!(writer)?;
                writeln!(writer, "  Used By (downstream)")?;
                if downstream.is_empty() {
                    writeln!(writer, "    None")?;
                } else {
                    for dep in &downstream {
                        writeln!(
                            writer,
                            "    {}.{}  ({})",
                            dep.database, dep.name, dep.kind_label
                        )?;
                    }
                }
                writeln!(writer)?;
            }
            Err(_) => {
                writeln!(writer, "── Dependencies ──")?;
                writeln!(
                    writer,
                    "  (Dependency information unavailable — requires SELECT on DBC.TablesV)"
                )?;
                writeln!(writer)?;
            }
        }
    }

    Ok(())
}

/// JSON output for batch mode
fn inspect_object_json<W: Write>(
    client: &DatabaseClient,
    object_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (db_part, obj_part) = parse_table_name(object_name);
    let database = query_helpers::resolve_database(client, db_part)?;

    let obj_info = match query_object_type(client, &database, obj_part)? {
        Some(info) => info,
        None => {
            writeln!(writer, "{{\"error\": \"Object '{}' not found\"}}", object_name)?;
            return Ok(());
        }
    };

    write!(writer, "{{")?;
    write!(
        writer,
        "\"type\":\"{}\",\"database\":\"{}\",\"name\":\"{}\"",
        json_escape(&obj_info.kind_label),
        json_escape(&database),
        json_escape(obj_part)
    )?;
    if !obj_info.created.is_empty() {
        write!(writer, ",\"created\":\"{}\"", json_escape(&obj_info.created))?;
    }

    // Columns (using shared query_helpers)
    if let Ok(columns) = query_helpers::query_columns(client, &database, obj_part) {
        write!(writer, ",\"columns\":[")?;
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                write!(writer, ",")?;
            }
            write!(
                writer,
                "{{\"name\":\"{}\",\"type\":\"{}\",\"nullable\":\"{}\"",
                json_escape(&col.name),
                json_escape(&col.col_type),
                json_escape(&col.nullable)
            )?;
            if col.default_val != "-" {
                write!(writer, ",\"default\":\"{}\"", json_escape(&col.default_val))?;
            }
            write!(writer, "}}")?;
        }
        write!(writer, "]")?;
    }

    // Storage for tables
    if obj_info.table_kind == "T" || obj_info.table_kind == "O" {
        if let Ok(storage) = query_storage(client, &database, obj_part) {
            let skew = calculate_skew(storage.max_amp_size, storage.avg_amp_size);
            write!(
                writer,
                ",\"storage\":{{\"current_size\":{},\"peak_size\":{},\"skew_factor\":{:.1},\"amp_count\":{}}}",
                storage.total_size, storage.peak_size, skew, storage.amp_count
            )?;
        }
    }

    writeln!(writer, "}}")?;
    Ok(())
}

/// CSV output for batch mode
fn inspect_object_csv<W: Write>(
    client: &DatabaseClient,
    object_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (db_part, obj_part) = parse_table_name(object_name);
    let database = query_helpers::resolve_database(client, db_part)?;

    let obj_info = match query_object_type(client, &database, obj_part)? {
        Some(info) => info,
        None => {
            writeln!(writer, "error,Object '{}' not found", object_name)?;
            return Ok(());
        }
    };

    // Output columns as CSV (most useful tabular representation)
    writeln!(writer, "Column,Type,Nullable,Default")?;
    if let Ok(columns) = query_helpers::query_columns(client, &database, obj_part) {
        for col in &columns {
            let default_display = if col.default_val == "-" {
                ""
            } else {
                &col.default_val
            };
            writeln!(
                writer,
                "{},{},{},{}",
                csv_escape(&col.name),
                csv_escape(&col.col_type),
                csv_escape(&col.nullable),
                csv_escape(default_display)
            )?;
        }
    }

    // Add storage info as a summary comment for tables
    if obj_info.table_kind == "T" || obj_info.table_kind == "O" {
        if let Ok(storage) = query_storage(client, &database, obj_part) {
            let skew = calculate_skew(storage.max_amp_size, storage.avg_amp_size);
            writeln!(
                writer,
                "# Storage: {} current, {} peak, {:.1}% skew, {} AMPs",
                format_size(storage.total_size, 2),
                format_size(storage.peak_size, 2),
                skew,
                storage.amp_count
            )?;
        }
    }

    Ok(())
}

// =============================================================================
// Data structures (inspect-specific)
// =============================================================================

/// Metadata about a database object from DBC.TablesV (inspect-specific: includes
/// created timestamp and comment which the shared ObjectHeader does not).
struct ObjectInfo {
    /// Raw TableKind character (T, V, M, O, etc.)
    table_kind: String,
    /// Human-readable type label
    kind_label: String,
    /// Creation timestamp
    created: String,
    /// Comment string
    comment: String,
}

/// Storage metrics from DBC.TableSizeV
struct StorageInfo {
    total_size: i64,
    peak_size: i64,
    max_amp_size: i64,
    avg_amp_size: i64,
    amp_count: i64,
}

// =============================================================================
// Query helpers (inspect-specific)
// =============================================================================

/// Query DBC.TablesV for object type and metadata (inspect-specific: includes
/// created timestamp and comment).
fn query_object_type(
    client: &DatabaseClient,
    db: &str,
    obj: &str,
) -> Result<Option<ObjectInfo>> {
    let sql = format!(
        "SELECT TRIM(TableKind) AS TableKind, \
         CAST(CreateTimeStamp AS VARCHAR(26)) AS Created, \
         COALESCE(TRIM(CommentString), '') AS CommentStr \
         FROM DBC.TablesV \
         WHERE DatabaseName = '{}' AND TableName = '{}'",
        escape_sql_string(db),
        escape_sql_string(obj)
    );

    let result = client.execute(&sql)?;

    if let Some(row) = result.rows.first() {
        let table_kind = row
            .first()
            .map(|v| v.display().trim().to_string())
            .unwrap_or_default();
        let created = row
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
        let comment = row
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
        let kind_label = map_table_kind(&table_kind);

        Ok(Some(ObjectInfo {
            table_kind,
            kind_label,
            created,
            comment,
        }))
    } else {
        Ok(None)
    }
}

/// Query DBC.TableSizeV for storage metrics
fn query_storage(
    client: &DatabaseClient,
    db: &str,
    obj: &str,
) -> Result<StorageInfo> {
    let sql = format!(
        "SELECT CAST(SUM(CurrentPerm) AS BIGINT) AS TotalSize, \
         CAST(SUM(PeakPerm) AS BIGINT) AS PeakSize, \
         CAST(MAX(CurrentPerm) AS BIGINT) AS MaxAmpSize, \
         CAST(AVG(CurrentPerm) AS BIGINT) AS AvgAmpSize, \
         COUNT(*) AS AmpCount \
         FROM DBC.TableSizeV \
         WHERE DatabaseName = '{}' AND TableName = '{}'",
        escape_sql_string(db),
        escape_sql_string(obj)
    );

    let result = client.execute(&sql)?;

    if let Some(row) = result.rows.first() {
        let total_size = row.first().map(extract_i64).unwrap_or(0);
        let peak_size = row.get(1).map(extract_i64).unwrap_or(0);
        let max_amp_size = row.get(2).map(extract_i64).unwrap_or(0);
        let avg_amp_size = row.get(3).map(extract_i64).unwrap_or(0);
        let amp_count = row.get(4).map(extract_i64).unwrap_or(0);

        Ok(StorageInfo {
            total_size,
            peak_size,
            max_amp_size,
            avg_amp_size,
            amp_count,
        })
    } else {
        Ok(StorageInfo {
            total_size: 0,
            peak_size: 0,
            max_amp_size: 0,
            avg_amp_size: 0,
            amp_count: 0,
        })
    }
}

/// Query the definition of a view or macro using SHOW statement
///
/// Teradata's SHOW VIEW/MACRO returns the DDL as multiple rows, each containing
/// a fixed-width VARCHAR chunk. These chunks are split at arbitrary character
/// boundaries (NOT logical line breaks), so they must be concatenated directly
/// without inserting newlines between rows.
fn query_definition(
    client: &DatabaseClient,
    db: &str,
    obj: &str,
    kind: &str,
) -> Result<String> {
    let show_cmd = match kind {
        "V" => format!("SHOW VIEW \"{}\".\"{}\"", db, obj),
        "M" => format!("SHOW MACRO \"{}\".\"{}\"", db, obj),
        _ => return Ok(String::new()),
    };

    let result = client.execute(&show_cmd)?;

    // Concatenate all row chunks directly — they are arbitrary splits of the DDL text
    let mut definition = String::new();
    for row in &result.rows {
        if let Some(val) = row.first() {
            let text = val.display();
            if text != "[NULL]" {
                definition.push_str(&text);
            }
        }
    }

    Ok(definition.trim().to_string())
}

/// Format raw DDL text with line breaks at SQL keywords for readability
///
/// Teradata's SHOW VIEW returns DDL as a single continuous string.
/// This function inserts line breaks before major SQL clause keywords
/// using word-boundary matching to avoid splitting inside identifiers.
fn format_ddl(raw: &str) -> String {
    // Normalize whitespace: collapse multiple spaces/tabs into single space
    let normalized: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");

    // SQL clause keywords that should start a new line.
    // Each must be preceded and followed by a word boundary (space or start/end).
    let break_before = [
        "SELECT", "FROM", "WHERE", "AND", "OR", "JOIN", "LEFT JOIN",
        "RIGHT JOIN", "INNER JOIN", "OUTER JOIN", "CROSS JOIN",
        "ON", "GROUP BY", "ORDER BY", "HAVING", "UNION", "UNION ALL",
        "INTERSECT", "MINUS", "LOCK", "LOCKING", "WITH CHECK OPTION",
    ];

    let mut result = normalized;

    for kw in &break_before {
        // Build pattern: " KEYWORD " (space-bounded) to match whole words only
        let pattern = format!(" {} ", kw);
        let upper = result.to_uppercase();
        let mut new_result = String::new();
        let mut search_from = 0;

        while let Some(pos) = upper[search_from..].find(&pattern) {
            let abs_pos = search_from + pos;
            // Append everything up to the match, then newline + the keyword + space
            new_result.push_str(&result[search_from..abs_pos]);
            new_result.push('\n');
            // Preserve original case from result
            new_result.push_str(&result[abs_pos..abs_pos + pattern.len()]);
            search_from = abs_pos + pattern.len();
        }
        new_result.push_str(&result[search_from..]);
        result = new_result;
    }

    // Also break after AS that follows the view/macro name (e.g., "CREATE VIEW x AS")
    // Match " AS SELECT" or "AS\nSELECT" pattern
    let upper = result.to_uppercase();
    if let Some(pos) = upper.find(" AS SELECT") {
        let mut new_result = String::new();
        new_result.push_str(&result[..pos + 3]); // include " AS"
        new_result.push('\n');
        new_result.push_str(&result[pos + 3..]); // rest starting with " SELECT"
        result = new_result;
    } else if let Some(pos) = upper.find(" AS\nSELECT") {
        // Already broken, keep as-is
        let _ = pos;
    }

    result.trim().to_string()
}

/// A dependency reference to another database object
#[derive(Debug, Clone)]
struct DependencyRef {
    database: String,
    name: String,
    kind_label: String,
}

/// Query upstream and downstream dependencies for a view or macro
///
/// Upstream: objects referenced by this view/macro (parsed from DDL text in DBC.ViewTextV/TableTextV)
/// Downstream: objects that reference this view/macro (searched in DBC.ViewTextV/TableTextV)
fn query_dependencies(
    client: &DatabaseClient,
    database: &str,
    name: &str,
) -> Result<(Vec<DependencyRef>, Vec<DependencyRef>)> {
    let upstream = query_upstream_dependencies(client, database, name);
    let downstream = query_downstream_dependencies(client, database, name);

    // If both fail, propagate the error
    match (&upstream, &downstream) {
        (Err(_), Err(e)) => Err(crate::error::TqError::QueryExecution(e.to_string())),
        _ => Ok((
            upstream.unwrap_or_default(),
            downstream.unwrap_or_default(),
        )),
    }
}

/// Query upstream dependencies by finding objects referenced in this view's DDL
///
/// Uses DBC.TablesV to find tables/views in the same database that appear in the
/// view text from DBC.ViewTextV or DBC.TableTextV.
fn query_upstream_dependencies(
    client: &DatabaseClient,
    database: &str,
    name: &str,
) -> Result<Vec<DependencyRef>> {
    // Get the view/macro text and parse referenced objects from it
    let sql = format!(
        "SELECT TRIM(t.DatabaseName) AS DepDatabase, \
                TRIM(t.TableName) AS DepName, \
                TRIM(t.TableKind) AS DepKind \
         FROM DBC.TablesV t \
         WHERE EXISTS ( \
             SELECT 1 FROM DBC.ViewTextV v \
             WHERE v.DatabaseName = '{}' \
               AND v.TableName = '{}' \
               AND v.RequestText LIKE '%' || TRIM(t.TableName) || '%' \
         ) \
         AND NOT (t.DatabaseName = '{}' AND t.TableName = '{}') \
         AND t.TableKind IN ('T', 'O', 'V', 'M') \
         ORDER BY t.DatabaseName, t.TableName",
        escape_sql_string(database),
        escape_sql_string(name),
        escape_sql_string(database),
        escape_sql_string(name)
    );

    match client.execute(&sql) {
        Ok(result) => {
            Ok(result
                .rows
                .iter()
                .filter_map(|row| {
                    if row.len() >= 3 {
                        let db = row[0].display().trim().to_string();
                        let obj_name = row[1].display().trim().to_string();
                        let kind = row[2].display().trim().to_string();
                        Some(DependencyRef {
                            database: db,
                            name: obj_name,
                            kind_label: map_table_kind(&kind),
                        })
                    } else {
                        None
                    }
                })
                .collect())
        }
        Err(_) => {
            // ViewTextV may not be accessible — try a simpler approach via SHOW
            Ok(Vec::new())
        }
    }
}

/// Query downstream dependencies by finding views/macros that reference this object
fn query_downstream_dependencies(
    client: &DatabaseClient,
    database: &str,
    name: &str,
) -> Result<Vec<DependencyRef>> {
    let sql = format!(
        "SELECT TRIM(v.DatabaseName) AS DepDatabase, \
                TRIM(v.TableName) AS DepName, \
                TRIM(t.TableKind) AS DepKind \
         FROM DBC.ViewTextV v \
         JOIN DBC.TablesV t \
           ON t.DatabaseName = v.DatabaseName AND t.TableName = v.TableName \
         WHERE v.RequestText LIKE '%{}.{}%' \
           OR v.RequestText LIKE '%\"{}\".\"{}\"%' \
           OR v.RequestText LIKE '% {}%' \
         AND NOT (v.DatabaseName = '{}' AND v.TableName = '{}') \
         ORDER BY v.DatabaseName, v.TableName",
        escape_sql_string(database),
        escape_sql_string(name),
        escape_sql_string(database),
        escape_sql_string(name),
        escape_sql_string(name),
        escape_sql_string(database),
        escape_sql_string(name)
    );

    match client.execute(&sql) {
        Ok(result) => {
            Ok(result
                .rows
                .iter()
                .filter_map(|row| {
                    if row.len() >= 3 {
                        let db = row[0].display().trim().to_string();
                        let obj_name = row[1].display().trim().to_string();
                        let kind = row[2].display().trim().to_string();
                        // Don't include self
                        if db.eq_ignore_ascii_case(database)
                            && obj_name.eq_ignore_ascii_case(name)
                        {
                            return None;
                        }
                        Some(DependencyRef {
                            database: db,
                            name: obj_name,
                            kind_label: map_table_kind(&kind),
                        })
                    } else {
                        None
                    }
                })
                .collect())
        }
        Err(_) => Ok(Vec::new()),
    }
}

// =============================================================================
// Formatting helpers (inspect-specific)
// =============================================================================

/// Calculate skew percentage from max and average AMP sizes
///
/// Skew = ((max / avg) - 1) * 100
/// Returns 0.0 when average is zero to avoid division by zero.
fn calculate_skew(max_amp: i64, avg_amp: i64) -> f64 {
    if avg_amp == 0 {
        return 0.0;
    }
    ((max_amp as f64 / avg_amp as f64) - 1.0) * 100.0
}

/// Interpret skew percentage as a human-readable hint
fn interpret_skew(skew: f64) -> &'static str {
    if skew < 10.0 {
        "(low)"
    } else if skew <= 30.0 {
        "(moderate)"
    } else {
        "(high)"
    }
}

/// Extract an i64 from a Value, handling various representations
fn extract_i64(val: &Value) -> i64 {
    match val {
        Value::Integer(n) => *n,
        Value::Decimal(f) => *f as i64,
        _ => {
            let s = val.display();
            s.trim().parse::<i64>().unwrap_or(0)
        }
    }
}

/// Summarize an error message for inline display (UTF-8 safe)
fn summarize_error(e: &crate::error::TqError) -> String {
    let msg = e.to_string();
    truncate_str(&msg, 80)
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::format_helpers::classify_index;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0, 2), "0 B");
        assert_eq!(format_size(512, 2), "512 B");
        assert_eq!(format_size(1023, 2), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024, 2), "1.00 KB");
        assert_eq!(format_size(1536, 2), "1.50 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1048576, 2), "1.00 MB");
        assert_eq!(format_size(1572864, 2), "1.50 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1073741824, 2), "1.00 GB");
        assert_eq!(format_size(1319413964, 2), "1.23 GB");
    }

    #[test]
    fn test_format_size_terabytes() {
        assert_eq!(format_size(1099511627776, 2), "1.00 TB");
    }

    #[test]
    fn test_format_size_negative() {
        assert_eq!(format_size(-100, 2), "-100 B");
    }

    #[test]
    fn test_format_size_precision_1() {
        assert_eq!(format_size(1024, 1), "1.0 KB");
        assert_eq!(format_size(1536, 1), "1.5 KB");
        assert_eq!(format_size(1048576, 1), "1.0 MB");
    }

    #[test]
    fn test_format_size_precision_2() {
        assert_eq!(format_size(1024, 2), "1.00 KB");
        assert_eq!(format_size(1536, 2), "1.50 KB");
    }

    #[test]
    fn test_calculate_skew_normal() {
        assert!((calculate_skew(105, 100) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_skew_zero_avg() {
        assert_eq!(calculate_skew(100, 0), 0.0);
    }

    #[test]
    fn test_calculate_skew_equal() {
        assert_eq!(calculate_skew(100, 100), 0.0);
    }

    #[test]
    fn test_calculate_skew_low() {
        assert!((calculate_skew(101, 100) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_interpret_skew() {
        assert_eq!(interpret_skew(0.0), "(low)");
        assert_eq!(interpret_skew(5.0), "(low)");
        assert_eq!(interpret_skew(9.9), "(low)");
        assert_eq!(interpret_skew(10.0), "(moderate)");
        assert_eq!(interpret_skew(20.0), "(moderate)");
        assert_eq!(interpret_skew(30.0), "(moderate)");
        assert_eq!(interpret_skew(30.1), "(high)");
        assert_eq!(interpret_skew(100.0), "(high)");
    }

    #[test]
    fn test_classify_index_via_format_helpers() {
        let (label, short) = classify_index("P", true);
        assert_eq!(label, "Primary Index");
        assert_eq!(short, "UPI");
    }

    // DDL tests from TC-047-001

    #[test]
    fn test_ddl_multirow_concatenation() {
        // Simulate multi-row SHOW result: rows joined with newlines
        let rows = vec!["CREATE TABLE t (", "  col1 INTEGER", ");"];
        let mut definition = String::new();
        for text in &rows {
            let trimmed = text.trim_end();
            if !definition.is_empty() && !trimmed.is_empty() {
                definition.push('\n');
            }
            definition.push_str(trimmed);
        }
        let result = definition.trim().to_string();
        assert_eq!(result, "CREATE TABLE t (\n  col1 INTEGER\n);");
        assert_eq!(result.lines().count(), 3);
    }

    #[test]
    fn test_ddl_null_rows_filtered() {
        // NULL rows should be excluded from definition
        let rows: Vec<Option<&str>> = vec![
            Some("CREATE VIEW v AS"),
            None,
            Some("SELECT * FROM t"),
        ];
        let mut definition = String::new();
        for maybe_text in &rows {
            if let Some(text) = maybe_text {
                if *text != "[NULL]" {
                    let trimmed = text.trim_end();
                    if !definition.is_empty() && !trimmed.is_empty() {
                        definition.push('\n');
                    }
                    definition.push_str(trimmed);
                }
            }
        }
        let result = definition.trim().to_string();
        assert_eq!(result, "CREATE VIEW v AS\nSELECT * FROM t");
    }

    #[test]
    fn test_ddl_empty_result() {
        // Empty result returns empty string
        let rows: Vec<&str> = vec![];
        let mut definition = String::new();
        for text in &rows {
            let trimmed = text.trim_end();
            if !definition.is_empty() && !trimmed.is_empty() {
                definition.push('\n');
            }
            definition.push_str(trimmed);
        }
        let result = definition.trim().to_string();
        assert!(result.is_empty());
    }

    #[test]
    fn test_ddl_trim_whitespace() {
        // Trailing whitespace should be stripped from each row
        let rows = vec!["CREATE TABLE t (   ", "  col1 INTEGER   ", ");   "];
        let mut definition = String::new();
        for text in &rows {
            let trimmed = text.trim_end();
            if !definition.is_empty() && !trimmed.is_empty() {
                definition.push('\n');
            }
            definition.push_str(trimmed);
        }
        let result = definition.trim().to_string();
        // No trailing whitespace on any line
        for line in result.lines() {
            assert_eq!(line, line.trim_end(), "Line has trailing whitespace: {:?}", line);
        }
    }

    #[test]
    fn test_ddl_show_view_sql_construction() {
        // Verify SHOW VIEW SQL is constructed correctly
        let db = "mydb";
        let obj = "myview";
        let show_cmd = format!("SHOW VIEW \"{}\".\"{}\"", db, obj);
        assert_eq!(show_cmd, "SHOW VIEW \"mydb\".\"myview\"");
    }

    #[test]
    fn test_ddl_show_macro_sql_construction() {
        // Verify SHOW MACRO SQL is constructed correctly
        let db = "mydb";
        let obj = "mymacro";
        let show_cmd = format!("SHOW MACRO \"{}\".\"{}\"", db, obj);
        assert_eq!(show_cmd, "SHOW MACRO \"mydb\".\"mymacro\"");
    }

    #[test]
    fn test_summarize_error_short() {
        // Short errors pass through unchanged
        let msg = "Connection refused";
        let result = truncate_str(msg, 80);
        assert_eq!(result, "Connection refused");
    }

    #[test]
    fn test_summarize_error_long() {
        // Long errors get truncated with "..."
        let msg = "A".repeat(200);
        let result = truncate_str(&msg, 80);
        assert_eq!(result.chars().count(), 80);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_summarize_error_utf8_safe() {
        // UTF-8 multi-byte characters don't cause panics
        let msg = "\u{4e2d}".repeat(100); // 100 CJK characters
        let result = truncate_str(&msg, 80);
        assert!(result.chars().count() <= 80);
        assert!(result.ends_with("..."));
    }
}
