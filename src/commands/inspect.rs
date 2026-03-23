//! Object inspection command implementation
//!
//! Provides comprehensive inspection of Teradata database objects showing
//! type, columns, indexes, storage/skew (tables), and definitions (views/macros).
//!
//! Sprint 45: Initial implementation (Issue #33)

use crate::cli::OutputFormat;
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
    let (db_part, obj_part) = parse_object_name(object_name);

    // Resolve database name
    let database = match resolve_object_database(client, db_part) {
        Ok(db) => db,
        Err(e) => {
            writeln!(writer, "Error resolving database: {}", e)?;
            return Ok(());
        }
    };

    // Section 1: Object Info (required — if this fails, the object doesn't exist)
    let obj_info = match query_object_type(client, &database, obj_part) {
        Ok(Some(info)) => info,
        Ok(None) => {
            writeln!(writer, "Object '{}' not found.", object_name)?;
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
    writeln!(writer, "=== Object Info ===")?;
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

    // Section 2: Columns
    match query_columns(client, &database, obj_part) {
        Ok(columns) => {
            if !columns.is_empty() {
                writeln!(writer, "=== Columns ({}) ===", columns.len())?;
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
                    writeln!(
                        writer,
                        "  {:<24} {:<20} {:<10} {}",
                        truncate_str(&col.name, 22),
                        truncate_str(&col.col_type, 18),
                        &col.nullable,
                        col.default.as_deref().unwrap_or("")
                    )?;
                }
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

    // Section 3: Indexes (only for tables)
    if obj_info.table_kind == "T" || obj_info.table_kind == "O" {
        match query_indexes(client, &database, obj_part) {
            Ok(indexes) => {
                if !indexes.is_empty() {
                    writeln!(writer, "=== Indexes ===")?;
                    for idx in &indexes {
                        let uniqueness = if idx.is_unique { "U" } else { "NU" };
                        let columns_str = idx.columns.join(", ");
                        if let Some(ref name) = idx.name {
                            writeln!(
                                writer,
                                "  {} ({}{}) \"{}\": {}",
                                idx.index_type, uniqueness, idx.kind_suffix, name, columns_str
                            )?;
                        } else {
                            writeln!(
                                writer,
                                "  {} ({}{}): {}",
                                idx.index_type, uniqueness, idx.kind_suffix, columns_str
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
                writeln!(writer, "=== Storage ===")?;
                writeln!(
                    writer,
                    "  Current Size:  {}",
                    format_size(storage.total_size)
                )?;
                writeln!(
                    writer,
                    "  Peak Size:     {}",
                    format_size(storage.peak_size)
                )?;
                let skew = calculate_skew(storage.max_amp_size, storage.avg_amp_size);
                writeln!(writer, "  Skew Factor:   {:.1}%", skew)?;
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
                writeln!(writer, "=== Definition ===")?;
                for line in definition.lines() {
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

    Ok(())
}

/// JSON output for batch mode
fn inspect_object_json<W: Write>(
    client: &DatabaseClient,
    object_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (db_part, obj_part) = parse_object_name(object_name);
    let database = resolve_object_database(client, db_part)?;

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

    // Columns
    if let Ok(columns) = query_columns(client, &database, obj_part) {
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
            if let Some(ref def) = col.default {
                write!(writer, ",\"default\":\"{}\"", json_escape(def))?;
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
    let (db_part, obj_part) = parse_object_name(object_name);
    let database = resolve_object_database(client, db_part)?;

    let obj_info = match query_object_type(client, &database, obj_part)? {
        Some(info) => info,
        None => {
            writeln!(writer, "error,Object '{}' not found", object_name)?;
            return Ok(());
        }
    };

    // Output columns as CSV (most useful tabular representation)
    writeln!(writer, "Column,Type,Nullable,Default")?;
    if let Ok(columns) = query_columns(client, &database, obj_part) {
        for col in &columns {
            writeln!(
                writer,
                "{},{},{},{}",
                csv_escape(&col.name),
                csv_escape(&col.col_type),
                csv_escape(&col.nullable),
                csv_escape(col.default.as_deref().unwrap_or(""))
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
                format_size(storage.total_size),
                format_size(storage.peak_size),
                skew,
                storage.amp_count
            )?;
        }
    }

    Ok(())
}

// =============================================================================
// Data structures
// =============================================================================

/// Metadata about a database object from DBC.TablesV
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

/// Column metadata
struct ColumnInfo {
    name: String,
    col_type: String,
    nullable: String,
    default: Option<String>,
}

/// Index metadata (grouped by index)
struct IndexInfo {
    name: Option<String>,
    index_type: String,
    kind_suffix: String,
    is_unique: bool,
    columns: Vec<String>,
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
// Query helpers
// =============================================================================

/// Parse an object name into optional database and object parts
///
/// Handles both `database.object` and `object` forms.
fn parse_object_name(name: &str) -> (Option<&str>, &str) {
    if let Some(dot_pos) = name.find('.') {
        let db = &name[..dot_pos];
        let obj = &name[dot_pos + 1..];
        (Some(db), obj)
    } else {
        (None, name)
    }
}

/// Resolve the database name, querying `SELECT DATABASE` if not specified
fn resolve_object_database(
    client: &DatabaseClient,
    db: Option<&str>,
) -> Result<String> {
    if let Some(db_name) = db {
        return Ok(db_name.to_string());
    }

    let result = client.execute("SELECT DATABASE")?;
    if let Some(row) = result.rows.first() {
        if let Some(val) = row.first() {
            let db_name = val.display().trim().to_string();
            if !db_name.is_empty() && db_name != "[NULL]" {
                return Ok(db_name);
            }
        }
    }

    Ok(client.config().database.clone())
}

/// Query DBC.TablesV for object type and metadata
fn query_object_type(
    client: &DatabaseClient,
    db: &str,
    obj: &str,
) -> Result<Option<ObjectInfo>> {
    let sql = format!(
        "SELECT TRIM(TableKind) AS TableKind, \
         CAST(CreateTimeStamp AS VARCHAR(26)) AS Created, \
         COALESCE(TRIM(CommentString), '') AS Comment \
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

/// Query DBC.ColumnsV for column metadata (same SQL pattern as /describe)
fn query_columns(
    client: &DatabaseClient,
    db: &str,
    obj: &str,
) -> Result<Vec<ColumnInfo>> {
    let sql = format!(
        "SELECT TRIM(ColumnName), ColumnType, Nullable, DefaultValue \
         FROM DBC.ColumnsV \
         WHERE DatabaseName = '{}' AND TableName = '{}' \
         ORDER BY ColumnId",
        escape_sql_string(db),
        escape_sql_string(obj)
    );

    let result = client.execute(&sql)?;
    let mut columns = Vec::with_capacity(result.rows.len());

    for row in &result.rows {
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
        let default = row.get(3).and_then(|v| {
            let s = v.display();
            if s == "[NULL]" {
                None
            } else {
                Some(s.trim().to_string())
            }
        });

        columns.push(ColumnInfo {
            name,
            col_type,
            nullable,
            default,
        });
    }

    Ok(columns)
}

/// Query DBC.IndicesV for index information (same SQL pattern as /show indexes)
fn query_indexes(
    client: &DatabaseClient,
    db: &str,
    obj: &str,
) -> Result<Vec<IndexInfo>> {
    let sql = format!(
        "SELECT TRIM(IndexName) AS IndexName, \
         IndexType, \
         UniqueFlag, \
         TRIM(ColumnName) AS ColumnName, \
         IndexNumber, \
         ColumnPosition \
         FROM DBC.IndicesV \
         WHERE DatabaseName = '{}' AND TableName = '{}' \
         ORDER BY IndexNumber, ColumnPosition",
        escape_sql_string(db),
        escape_sql_string(obj)
    );

    let result = client.execute(&sql)?;

    // Group rows by IndexNumber — build IndexInfo entries directly
    let mut indexes: Vec<IndexInfo> = Vec::new();
    let mut index_numbers: Vec<i64> = Vec::new();

    for row in &result.rows {
        let index_name = row.first().and_then(|v| {
            let s = v.display().trim().to_string();
            if s.is_empty() || s == "[NULL]" {
                None
            } else {
                Some(s)
            }
        });
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

        let (index_type_label, kind_suffix) = map_index_type(&index_type_raw);
        let is_unique = unique_flag == "Y" || unique_flag == "U";

        // Find or create entry for this index number
        if let Some(pos) = index_numbers.iter().position(|n| *n == index_number) {
            indexes[pos].columns.push(column_name);
        } else {
            index_numbers.push(index_number);
            indexes.push(IndexInfo {
                name: index_name,
                index_type: index_type_label.to_string(),
                kind_suffix: kind_suffix.to_string(),
                is_unique,
                columns: vec![column_name],
            });
        }
    }

    Ok(indexes)
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
        let total_size = extract_i64(&row[0]);
        let peak_size = extract_i64(&row[1]);
        let max_amp_size = extract_i64(&row[2]);
        let avg_amp_size = extract_i64(&row[3]);
        let amp_count = extract_i64(&row[4]);

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

// =============================================================================
// Formatting helpers
// =============================================================================

/// Map TableKind character to human-readable label
fn map_table_kind(kind: &str) -> String {
    match kind {
        "T" | "O" => "Table".to_string(),
        "V" => "View".to_string(),
        "M" => "Macro".to_string(),
        "P" => "Stored Procedure".to_string(),
        "G" => "Trigger".to_string(),
        "A" => "Aggregate".to_string(),
        "E" => "External SP".to_string(),
        "N" => "Hash Index".to_string(),
        "I" => "Join Index".to_string(),
        other => format!("Unknown ({})", other),
    }
}

/// Map index type character to label and suffix
fn map_index_type(raw: &str) -> (&'static str, &'static str) {
    match raw.trim() {
        "P" => ("Primary Index", "PI"),
        "S" => ("Secondary Index", "SI"),
        "Q" => ("Partitioned Primary Index", "PPI"),
        "J" => ("Join Index", "JI"),
        "K" => ("Primary Key", "PK"),
        "U" => ("Unique Index", "UI"),
        "V" => ("Value-Ordered Index", "VOSI"),
        "H" => ("Hash Index", "HI"),
        _ => ("Index", ""),
    }
}

/// Format nullable indicator
fn format_nullable(s: &str) -> String {
    match s.trim().to_uppercase().as_str() {
        "Y" | "YES" | "TRUE" | "1" => "YES".to_string(),
        "N" | "NO" | "FALSE" | "0" => "NO".to_string(),
        _ => s.to_string(),
    }
}

/// Format byte count as human-readable size
fn format_size(bytes: i64) -> String {
    if bytes < 0 {
        return format!("{} B", bytes);
    }

    const KB: i64 = 1024;
    const MB: i64 = 1024 * KB;
    const GB: i64 = 1024 * MB;
    const TB: i64 = 1024 * GB;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

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

/// Truncate a string to a maximum length with ellipsis
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        ".".repeat(max_len)
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Summarize an error message for inline display
fn summarize_error(e: &crate::error::TqError) -> String {
    let msg = e.to_string();
    if msg.len() > 80 {
        format!("{}...", &msg[..77])
    } else {
        msg
    }
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

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_object_name_unqualified() {
        let (db, obj) = parse_object_name("mytable");
        assert!(db.is_none());
        assert_eq!(obj, "mytable");
    }

    #[test]
    fn test_parse_object_name_qualified() {
        let (db, obj) = parse_object_name("mydb.mytable");
        assert_eq!(db, Some("mydb"));
        assert_eq!(obj, "mytable");
    }

    #[test]
    fn test_parse_object_name_multiple_dots() {
        // First dot is the separator
        let (db, obj) = parse_object_name("a.b.c");
        assert_eq!(db, Some("a"));
        assert_eq!(obj, "b.c");
    }

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1572864), "1.50 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1073741824), "1.00 GB");
        assert_eq!(format_size(1319413964), "1.23 GB");
    }

    #[test]
    fn test_format_size_terabytes() {
        assert_eq!(format_size(1099511627776), "1.00 TB");
    }

    #[test]
    fn test_format_size_negative() {
        assert_eq!(format_size(-100), "-100 B");
    }

    #[test]
    fn test_calculate_skew_normal() {
        // max = 105, avg = 100 -> 5%
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
        // max = 101, avg = 100 -> 1%
        assert!((calculate_skew(101, 100) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_map_table_kind_all_known() {
        assert_eq!(map_table_kind("T"), "Table");
        assert_eq!(map_table_kind("O"), "Table");
        assert_eq!(map_table_kind("V"), "View");
        assert_eq!(map_table_kind("M"), "Macro");
        assert_eq!(map_table_kind("P"), "Stored Procedure");
        assert_eq!(map_table_kind("G"), "Trigger");
        assert_eq!(map_table_kind("A"), "Aggregate");
        assert_eq!(map_table_kind("E"), "External SP");
        assert_eq!(map_table_kind("N"), "Hash Index");
        assert_eq!(map_table_kind("I"), "Join Index");
    }

    #[test]
    fn test_map_table_kind_unknown() {
        assert_eq!(map_table_kind("X"), "Unknown (X)");
        assert_eq!(map_table_kind("Z"), "Unknown (Z)");
    }

    #[test]
    fn test_format_nullable() {
        assert_eq!(format_nullable("Y"), "YES");
        assert_eq!(format_nullable("N"), "NO");
        assert_eq!(format_nullable("YES"), "YES");
        assert_eq!(format_nullable("NO"), "NO");
        assert_eq!(format_nullable("1"), "YES");
        assert_eq!(format_nullable("0"), "NO");
    }

    #[test]
    fn test_json_escape() {
        assert_eq!(json_escape("hello"), "hello");
        assert_eq!(json_escape("he\"llo"), "he\\\"llo");
        assert_eq!(json_escape("line\nnew"), "line\\nnew");
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(csv_escape("hello"), "hello");
        assert_eq!(csv_escape("hello,world"), "\"hello,world\"");
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("short", 10), "short");
        assert_eq!(truncate_str("exactly10c", 10), "exactly10c");
        assert_eq!(truncate_str("this is a long string", 10), "this is...");
    }

    #[test]
    fn test_map_index_type() {
        assert_eq!(map_index_type("P"), ("Primary Index", "PI"));
        assert_eq!(map_index_type("S"), ("Secondary Index", "SI"));
        assert_eq!(map_index_type("Q"), ("Partitioned Primary Index", "PPI"));
        assert_eq!(map_index_type("K"), ("Primary Key", "PK"));
        assert_eq!(map_index_type("X"), ("Index", ""));
    }
}
