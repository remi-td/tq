//! Show-indexes command implementation
//!
//! Displays index information for a table from DBC.IndicesV with two-section
//! layout (Primary Index, Secondary Indexes) and UPI/NUPI/USI/NUSI labels.
//! Used by `tq show-indexes <table>` (batch) and `/show indexes` (REPL delegation).

use crate::cli::OutputFormat;
use crate::commands::format_helpers::{
    classify_index, csv_escape, json_escape, parse_table_name,
};
use crate::db::{DatabaseClient, Value};
use crate::error::Result;
use crate::sql::escape_sql_string;
use std::io::Write;

// =============================================================================
// Public API
// =============================================================================

/// Execute `tq show-indexes` in batch mode with format selection
pub fn execute<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    format: OutputFormat,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    match format {
        OutputFormat::Table => show_indexes_table(client, table_name, writer),
        OutputFormat::Json => show_indexes_json(client, table_name, writer),
        OutputFormat::Csv => show_indexes_csv(client, table_name, writer),
    }
}

/// Execute /show indexes in REPL mode (delegates to table format with spacing)
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;
    show_indexes_table(client, table_name, writer)?;
    writeln!(writer)?;
    Ok(())
}

// =============================================================================
// Data structures
// =============================================================================

/// An index group with all its columns
struct IndexGroup {
    name: String,
    index_type_label: String,
    short_label: String,
    is_primary: bool,
    columns: Vec<String>,
}

// =============================================================================
// Query helper
// =============================================================================

fn query_indexes(
    client: &DatabaseClient,
    table_name: &str,
) -> Result<(Vec<IndexGroup>, String)> {
    let (database, table) = parse_table_name(table_name);

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
                is_primary,
                columns: vec![column_name],
            });
        }
    }

    Ok((groups, qualified))
}

// =============================================================================
// Output formats
// =============================================================================

fn show_indexes_table<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (groups, qualified) = query_indexes(client, table_name)?;

    if groups.is_empty() {
        writeln!(
            writer,
            "Error: No indexes found for table '{}'.",
            table_name
        )?;
        writeln!(writer)?;
        writeln!(writer, "Suggestions:")?;
        writeln!(writer, "  - Check the table name spelling")?;
        writeln!(
            writer,
            "  - Try using qualified name: show-indexes database.table"
        )?;
        writeln!(
            writer,
            "  - Verify you have SELECT permission on DBC.IndicesV"
        )?;
        return Ok(());
    }

    writeln!(writer, "Indexes on {}:", qualified)?;
    writeln!(writer)?;

    // Primary Index section
    let primary: Vec<&IndexGroup> = groups.iter().filter(|g| g.is_primary).collect();
    if !primary.is_empty() {
        writeln!(writer, "── Primary Index ──")?;
        for idx in &primary {
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

    // Secondary Indexes section
    let secondary: Vec<&IndexGroup> = groups.iter().filter(|g| !g.is_primary).collect();
    if !secondary.is_empty() {
        writeln!(writer, "── Secondary Indexes ──")?;
        for idx in &secondary {
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

    // Summary
    let total_cols: usize = groups.iter().map(|g| g.columns.len()).sum();
    writeln!(
        writer,
        "{} index(es), {} index column(s)",
        groups.len(),
        total_cols
    )?;
    Ok(())
}

fn show_indexes_json<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (groups, qualified) = query_indexes(client, table_name)?;

    // Structured JSON: {object, primary_index, secondary_indexes}
    write!(writer, "{{\"object\":\"{}\"", json_escape(&qualified))?;

    // Primary index
    let primary: Vec<&IndexGroup> = groups.iter().filter(|g| g.is_primary).collect();
    if let Some(pi) = primary.first() {
        write!(
            writer,
            ",\"primary_index\":{{\"type\":\"{}\",\"columns\":[",
            json_escape(&pi.short_label)
        )?;
        for (j, col) in pi.columns.iter().enumerate() {
            if j > 0 {
                write!(writer, ",")?;
            }
            write!(writer, "\"{}\"", json_escape(col))?;
        }
        write!(writer, "]}}")?;
    } else {
        write!(writer, ",\"primary_index\":null")?;
    }

    // Secondary indexes
    let secondary: Vec<&IndexGroup> = groups.iter().filter(|g| !g.is_primary).collect();
    write!(writer, ",\"secondary_indexes\":[")?;
    for (i, idx) in secondary.iter().enumerate() {
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

    writeln!(writer, "}}")?;
    Ok(())
}

fn show_indexes_csv<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (groups, _qualified) = query_indexes(client, table_name)?;

    writeln!(writer, "IndexName,IndexType,ShortType,IsPrimary,Columns")?;
    for idx in &groups {
        let cols = idx.columns.join(", ");
        writeln!(
            writer,
            "{},{},{},{},{}",
            csv_escape(&idx.name),
            csv_escape(&idx.index_type_label),
            csv_escape(&idx.short_label),
            if idx.is_primary { "Yes" } else { "No" },
            csv_escape(&cols)
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
    fn test_index_group_primary() {
        let idx = IndexGroup {
            name: "(unnamed)".to_string(),
            index_type_label: "Primary Index".to_string(),
            short_label: "UPI".to_string(),
            is_primary: true,
            columns: vec!["emp_id".to_string()],
        };
        assert!(idx.is_primary);
        assert_eq!(idx.short_label, "UPI");
    }

    #[test]
    fn test_index_group_secondary() {
        let idx = IndexGroup {
            name: "idx_name".to_string(),
            index_type_label: "Secondary Index".to_string(),
            short_label: "NUSI".to_string(),
            is_primary: false,
            columns: vec!["last_name".to_string(), "first_name".to_string()],
        };
        assert!(!idx.is_primary);
        assert_eq!(idx.short_label, "NUSI");
        assert_eq!(idx.columns.len(), 2);
    }

    #[test]
    fn test_index_group_composite() {
        let idx = IndexGroup {
            name: "pk_composite".to_string(),
            index_type_label: "Primary Index".to_string(),
            short_label: "NUPI".to_string(),
            is_primary: true,
            columns: vec!["col_a".to_string(), "col_b".to_string(), "col_c".to_string()],
        };
        assert_eq!(idx.columns.join(", "), "col_a, col_b, col_c");
    }
}
