//! Shared query functions for database object metadata
//!
//! Consolidates duplicated query logic from inspect.rs, describe.rs, and
//! show_indexes.rs into a single module. Each function encapsulates one
//! DBC system view query pattern.

use crate::commands::format_helpers::{
    classify_index, column_type_case_sql, format_nullable, map_table_kind,
};
use crate::db::{DatabaseClient, Value};
use crate::error::Result;
use crate::sql::escape_sql_string;

// =============================================================================
// Shared types
// =============================================================================

/// Column metadata from DBC.ColumnsV
pub struct ColumnInfo {
    pub name: String,
    pub col_type: String,
    pub nullable: String,
    pub default_val: String,
    pub comment: String,
}

/// Index metadata grouped by index number from DBC.IndicesV
pub struct IndexGroup {
    pub name: Option<String>,
    pub index_type_label: String,
    pub short_label: String,
    pub columns: Vec<String>,
    pub is_primary: bool,
}

/// Object header metadata from DBC.TablesV
pub struct ObjectHeader {
    pub database: String,
    pub name: String,
    pub object_type: String,
    pub kind_label: String,
    pub table_kind: String,
    pub row_count: Option<i64>,
}

// =============================================================================
// Query functions
// =============================================================================

/// Resolve the database name, falling back to `SELECT DATABASE` if not specified.
pub fn resolve_database(
    client: &DatabaseClient,
    explicit_db: Option<&str>,
) -> Result<String> {
    if let Some(db_name) = explicit_db {
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

/// Query DBC.TablesV for object header metadata.
///
/// Returns `None` if the object does not exist.
pub fn query_object_header(
    client: &DatabaseClient,
    database: &str,
    name: &str,
) -> Result<Option<ObjectHeader>> {
    let sql = format!(
        "SELECT TRIM(DatabaseName), TRIM(TableName), TRIM(TableKind), \
         COALESCE(CAST( \
             (SELECT SUM(RowCount) FROM DBC.TableSizeV s \
              WHERE s.DatabaseName = t.DatabaseName \
                AND s.TableName = t.TableName) \
         AS BIGINT), NULL) AS RowCount \
         FROM DBC.TablesV t \
         WHERE DatabaseName = '{}' AND TableName = '{}'",
        escape_sql_string(database),
        escape_sql_string(name)
    );

    let result = client.execute(&sql)?;
    if let Some(row) = result.rows.first() {
        let db = row
            .first()
            .map(|v| v.display().trim().to_string())
            .unwrap_or_default();
        let obj_name = row
            .get(1)
            .map(|v| v.display().trim().to_string())
            .unwrap_or_default();
        let table_kind = row
            .get(2)
            .map(|v| v.display().trim().to_string())
            .unwrap_or_default();
        let kind_label = map_table_kind(&table_kind);
        let row_count = row.get(3).and_then(|v| match v {
            Value::Integer(n) => Some(*n),
            Value::Decimal(f) => Some(*f as i64),
            _ => {
                let s = v.display().trim().to_string();
                if s.is_empty() || s == "[NULL]" {
                    None
                } else {
                    s.parse::<i64>().ok()
                }
            }
        });

        Ok(Some(ObjectHeader {
            database: db,
            name: obj_name,
            object_type: table_kind.clone(),
            kind_label,
            table_kind,
            row_count,
        }))
    } else {
        Ok(None)
    }
}

/// Query DBC.ColumnsV for column metadata with proper type translation.
///
/// Always selects CommentString for the comment field.
pub fn query_columns(
    client: &DatabaseClient,
    database: &str,
    object_name: &str,
) -> Result<Vec<ColumnInfo>> {
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
        escape_sql_string(database),
        escape_sql_string(object_name)
    );

    let result = client.execute(&sql)?;

    let columns: Vec<ColumnInfo> = result
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
            let default_val = row
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

            ColumnInfo {
                name,
                col_type,
                nullable,
                default_val,
                comment,
            }
        })
        .collect();

    Ok(columns)
}

/// Query DBC.IndicesV for index information, grouped by index number.
///
/// Takes explicit database and table name parameters.
pub fn query_indexes(
    client: &DatabaseClient,
    database: &str,
    table_name: &str,
) -> Result<Vec<IndexGroup>> {
    let sql = format!(
        "SELECT TRIM(IndexName) AS IndexName, \
         IndexType, UniqueFlag, \
         TRIM(ColumnName) AS ColumnName, \
         IndexNumber, ColumnPosition \
         FROM DBC.IndicesV \
         WHERE DatabaseName = '{}' AND TableName = '{}' \
         ORDER BY IndexNumber, ColumnPosition",
        escape_sql_string(database),
        escape_sql_string(table_name)
    );

    let result = client.execute(&sql)?;

    let mut groups: Vec<IndexGroup> = Vec::new();
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

        let is_unique = unique_flag == "Y" || unique_flag == "U";
        let (type_label, short_label) = classify_index(&index_type_raw, is_unique);
        let is_primary = index_type_raw.trim() == "P"
            || index_type_raw.trim() == "Q"
            || index_type_raw.trim() == "K";

        if let Some(pos) = index_numbers.iter().position(|n| *n == index_number) {
            groups[pos].columns.push(column_name);
        } else {
            index_numbers.push(index_number);
            groups.push(IndexGroup {
                name: index_name,
                index_type_label: type_label.to_string(),
                short_label: short_label.to_string(),
                columns: vec![column_name],
                is_primary,
            });
        }
    }

    Ok(groups)
}

/// Query DBC.IndicesV using a potentially qualified name (database.table).
///
/// If unqualified, uses `DATABASE` keyword in WHERE clause. Returns
/// the index groups and a qualified display name.
pub fn query_indexes_qualified(
    client: &DatabaseClient,
    table_name: &str,
) -> Result<(Vec<IndexGroup>, String)> {
    let (database, table) =
        crate::commands::format_helpers::parse_table_name(table_name);

    let sql = if let Some(db) = database {
        format!(
            "SELECT TRIM(IndexName) AS IndexName, \
             IndexType, UniqueFlag, \
             TRIM(ColumnName) AS ColumnName, \
             IndexNumber, ColumnPosition \
             FROM DBC.IndicesV \
             WHERE DatabaseName = '{}' AND TableName = '{}' \
             ORDER BY IndexNumber, ColumnPosition",
            escape_sql_string(db),
            escape_sql_string(table)
        )
    } else {
        format!(
            "SELECT TRIM(IndexName) AS IndexName, \
             IndexType, UniqueFlag, \
             TRIM(ColumnName) AS ColumnName, \
             IndexNumber, ColumnPosition \
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

    let mut groups: Vec<IndexGroup> = Vec::new();
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

        let is_unique = unique_flag == "Y" || unique_flag == "U";
        let (type_label, short_label) = classify_index(&index_type_raw, is_unique);
        let is_primary = index_type_raw.trim() == "P"
            || index_type_raw.trim() == "Q"
            || index_type_raw.trim() == "K";

        if let Some(pos) = index_numbers.iter().position(|n| *n == index_number) {
            groups[pos].columns.push(column_name);
        } else {
            index_numbers.push(index_number);
            groups.push(IndexGroup {
                name: index_name,
                index_type_label: type_label.to_string(),
                short_label: short_label.to_string(),
                columns: vec![column_name],
                is_primary,
            });
        }
    }

    Ok((groups, qualified))
}
