//! Show-indexes command implementation
//!
//! Displays index information for a table from DBC.IndicesV with two-section
//! layout (Primary Index, Secondary Indexes) and UPI/NUPI/USI/NUSI labels.
//! Used by `tq show-indexes <object>` (batch) and `/show indexes` (REPL delegation).

use crate::cli::OutputFormat;
use crate::commands::format_helpers::{csv_escape, json_escape};
use crate::commands::query_helpers::{self, IndexGroup};
use crate::db::DatabaseClient;
use crate::error::Result;
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
        OutputFormat::Markdown | OutputFormat::Md => {
            show_indexes_markdown(client, table_name, writer)
        }
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
// Output formats
// =============================================================================

fn show_indexes_table<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (groups, qualified) = query_helpers::query_indexes_qualified(client, table_name)?;

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

    render_indexes_sections(&groups, writer)?;

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

/// Render the primary and secondary index sections.
fn render_indexes_sections<W: Write>(
    groups: &[IndexGroup],
    writer: &mut W,
) -> Result<()> {
    let primary: Vec<&IndexGroup> = groups.iter().filter(|g| g.is_primary).collect();
    let secondary: Vec<&IndexGroup> = groups.iter().filter(|g| !g.is_primary).collect();

    if !primary.is_empty() {
        writeln!(writer, "── Primary Index ──")?;
        for idx in &primary {
            render_index_line(idx, writer)?;
        }
        writeln!(writer)?;
    } else {
        writeln!(writer, "No Primary Index (NoPI)")?;
        writeln!(writer)?;
    }

    if !secondary.is_empty() {
        writeln!(writer, "── Secondary Indexes ──")?;
        for idx in &secondary {
            render_index_line(idx, writer)?;
        }
        writeln!(writer)?;
    } else {
        writeln!(writer, "No secondary indexes.")?;
        writeln!(writer)?;
    }

    Ok(())
}

/// Render a single index line with optional name.
fn render_index_line<W: Write>(idx: &IndexGroup, writer: &mut W) -> Result<()> {
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
    Ok(())
}

fn show_indexes_json<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (groups, qualified) = query_helpers::query_indexes_qualified(client, table_name)?;

    // Structured JSON: {ok, object, primary_index, secondary_indexes}
    write!(writer, "{{\"ok\":true,\"object\":\"{}\"", json_escape(&qualified))?;

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

    writeln!(writer, "}}")?;
    Ok(())
}

fn show_indexes_csv<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (groups, _qualified) = query_helpers::query_indexes_qualified(client, table_name)?;

    writeln!(writer, "IndexName,IndexType,ShortType,IsPrimary,Columns")?;
    for idx in &groups {
        let cols = idx.columns.join(", ");
        let name_display = idx.name.as_deref().unwrap_or("(unnamed)");
        writeln!(
            writer,
            "{},{},{},{},{}",
            csv_escape(name_display),
            csv_escape(&idx.index_type_label),
            csv_escape(&idx.short_label),
            if idx.is_primary { "Yes" } else { "No" },
            csv_escape(&cols)
        )?;
    }
    Ok(())
}

fn show_indexes_markdown<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let (groups, _qualified) = query_helpers::query_indexes_qualified(client, table_name)?;

    fn esc(s: &str) -> String {
        s.replace('|', "\\|")
    }
    writeln!(writer, "| IndexName | IndexType | ShortType | IsPrimary | Columns |")?;
    writeln!(writer, "| :--- | :--- | :--- | :--- | :--- |")?;
    for idx in &groups {
        let cols = idx.columns.join(", ");
        let name_display = idx.name.as_deref().unwrap_or("(unnamed)");
        writeln!(
            writer,
            "| {} | {} | {} | {} | {} |",
            esc(name_display),
            esc(&idx.index_type_label),
            esc(&idx.short_label),
            if idx.is_primary { "Yes" } else { "No" },
            esc(&cols)
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
            name: None,
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
            name: Some("idx_name".to_string()),
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
            name: Some("pk_composite".to_string()),
            index_type_label: "Primary Index".to_string(),
            short_label: "NUPI".to_string(),
            is_primary: true,
            columns: vec!["col_a".to_string(), "col_b".to_string(), "col_c".to_string()],
        };
        assert_eq!(idx.columns.join(", "), "col_a, col_b, col_c");
    }

    // Writer-injection tests for rendering functions

    #[test]
    fn test_render_indexes_sections_with_both() {
        let groups = vec![
            IndexGroup {
                name: None,
                index_type_label: "Primary Index".to_string(),
                short_label: "UPI".to_string(),
                is_primary: true,
                columns: vec!["emp_id".to_string()],
            },
            IndexGroup {
                name: Some("idx_last".to_string()),
                index_type_label: "Secondary Index".to_string(),
                short_label: "NUSI".to_string(),
                is_primary: false,
                columns: vec!["last_name".to_string()],
            },
        ];
        let mut buf = Vec::new();
        render_indexes_sections(&groups, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("── Primary Index ──"));
        assert!(output.contains("Primary Index (UPI): emp_id"));
        assert!(output.contains("── Secondary Indexes ──"));
        assert!(output.contains("Secondary Index (NUSI) \"idx_last\": last_name"));
    }

    #[test]
    fn test_render_indexes_sections_no_primary() {
        let groups = vec![IndexGroup {
            name: Some("idx_col".to_string()),
            index_type_label: "Secondary Index".to_string(),
            short_label: "NUSI".to_string(),
            is_primary: false,
            columns: vec!["col_a".to_string()],
        }];
        let mut buf = Vec::new();
        render_indexes_sections(&groups, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("No Primary Index (NoPI)"));
        assert!(output.contains("── Secondary Indexes ──"));
    }

    #[test]
    fn test_render_indexes_sections_no_secondary() {
        let groups = vec![IndexGroup {
            name: None,
            index_type_label: "Primary Index".to_string(),
            short_label: "UPI".to_string(),
            is_primary: true,
            columns: vec!["id".to_string()],
        }];
        let mut buf = Vec::new();
        render_indexes_sections(&groups, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("── Primary Index ──"));
        assert!(output.contains("No secondary indexes."));
    }

    #[test]
    fn test_render_index_line_with_name() {
        let idx = IndexGroup {
            name: Some("my_idx".to_string()),
            index_type_label: "Secondary Index".to_string(),
            short_label: "USI".to_string(),
            is_primary: false,
            columns: vec!["email".to_string()],
        };
        let mut buf = Vec::new();
        render_index_line(&idx, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Secondary Index (USI) \"my_idx\": email"));
    }

    #[test]
    fn test_render_index_line_without_name() {
        let idx = IndexGroup {
            name: None,
            index_type_label: "Primary Index".to_string(),
            short_label: "NUPI".to_string(),
            is_primary: true,
            columns: vec!["a".to_string(), "b".to_string()],
        };
        let mut buf = Vec::new();
        render_index_line(&idx, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Primary Index (NUPI): a, b"));
    }
}
