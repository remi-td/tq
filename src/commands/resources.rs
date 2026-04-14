//! Resources command implementation
//!
//! This module provides functionality to display Teradata system resource usage
//! metrics from ResUsage tables. Supports two modes:
//!
//! - **Virtual (default)**: Per-VPROC metrics from DBC.ResUsageSVPR
//! - **Physical**: Per-node metrics from DBC.ResUsageSPMA
//!
//! Each mode shows CPU, I/O, and memory metrics from the most recent sample
//! period, plus a summary footer with skew calculations.

use crate::cli::{OutputFormat, ResourcesArgs};
use crate::commands::format_helpers::markdown_escape_pipe;
use crate::db::{DatabaseClient, QueryResult, Value};
use crate::error::Result;
use super::monitoring_utils::{escape_csv, extract_decimal, extract_integer};
use std::io::Write;

/// SQL query to retrieve per-VPROC resource usage from DBC.ResUsageSVPR
///
/// Returns the most recent sample period's metrics for each virtual processor,
/// including CPU busy percentages, I/O throughput in KB, and memory usage.
const VIRTUAL_SQL: &str = r#"
SELECT
    VprocNo,
    AvgCPUBusy,
    PeakCPUBusy,
    AvgDiskReadKB + AvgDiskWriteKB AS AvgIOKB,
    PeakDiskReadKB + PeakDiskWriteKB AS PeakIOKB,
    MemCtxtPageReads,
    MemTotalUsed
FROM DBC.ResUsageSVPR
WHERE TheDate = (SELECT MAX(TheDate) FROM DBC.ResUsageSVPR)
  AND TheTime = (SELECT MAX(TheTime) FROM DBC.ResUsageSVPR WHERE TheDate = (SELECT MAX(TheDate) FROM DBC.ResUsageSVPR))
ORDER BY VprocNo
"#;

/// SQL query to retrieve per-node resource usage from DBC.ResUsageSPMA
///
/// Returns the most recent sample period's metrics for each physical node,
/// including CPU busy percentages, I/O counts, and memory allocation.
const PHYSICAL_SQL: &str = r#"
SELECT
    NodeID,
    AvgCPUBusy,
    PeakCPUBusy,
    AvgDiskReads + AvgDiskWrites AS AvgIOCnt,
    PeakDiskReads + PeakDiskWrites AS PeakIOCnt,
    MemSize,
    MemFreeKB
FROM DBC.ResUsageSPMA
WHERE TheDate = (SELECT MAX(TheDate) FROM DBC.ResUsageSPMA)
  AND TheTime = (SELECT MAX(TheTime) FROM DBC.ResUsageSPMA WHERE TheDate = (SELECT MAX(TheDate) FROM DBC.ResUsageSPMA))
ORDER BY NodeID
"#;

/// Resource information extracted from a ResUsage result row
///
/// This struct is used for both virtual (per-VPROC) and physical (per-node)
/// modes. The `id` field represents the VprocNo or NodeID depending on mode.
/// The `io_value` and `peak_io_value` fields represent KB (virtual) or
/// count (physical) depending on mode.
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    /// VPROC number or Node ID
    pub id: i64,
    /// Average CPU busy percentage
    pub avg_cpu: f64,
    /// Peak CPU busy percentage
    pub peak_cpu: f64,
    /// Average I/O (KB for virtual, count for physical)
    pub avg_io: f64,
    /// Peak I/O (KB for virtual, count for physical)
    pub peak_io: f64,
    /// Memory metric 1: MemCtxtPageReads (virtual) or MemSize (physical)
    pub mem_metric1: f64,
    /// Memory metric 2: MemTotalUsed (virtual) or MemFreeKB (physical)
    pub mem_metric2: f64,
}

impl ResourceInfo {
    /// Create ResourceInfo from a query result row
    ///
    /// Expects 7 columns: id, avg_cpu, peak_cpu, avg_io, peak_io, mem1, mem2.
    /// Returns None if required fields are missing or cannot be parsed.
    pub fn from_row(row: &[Value]) -> Option<Self> {
        if row.len() < 7 {
            return None;
        }

        let id = extract_integer(&row[0])?;
        let avg_cpu = extract_decimal(&row[1]).unwrap_or(0.0);
        let peak_cpu = extract_decimal(&row[2]).unwrap_or(0.0);
        let avg_io = extract_decimal(&row[3]).unwrap_or(0.0);
        let peak_io = extract_decimal(&row[4]).unwrap_or(0.0);
        let mem_metric1 = extract_decimal(&row[5]).unwrap_or(0.0);
        let mem_metric2 = extract_decimal(&row[6]).unwrap_or(0.0);

        Some(Self {
            id,
            avg_cpu,
            peak_cpu,
            avg_io,
            peak_io,
            mem_metric1,
            mem_metric2,
        })
    }
}

/// Calculate skew percentage across a set of values
///
/// Skew measures how unevenly a metric is distributed across VPROCs or nodes.
/// Formula: `100 * (1 - (avg / max))`
///
/// Returns None if there are no values or the maximum is zero.
pub fn calculate_skew(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if max <= 0.0 {
        return None;
    }
    let avg = values.iter().sum::<f64>() / values.len() as f64;
    Some(100.0 * (1.0 - (avg / max)))
}

/// Execute the resources command and write results
///
/// This is the main entry point for batch mode. Selects the appropriate
/// SQL query based on the `--physical` flag.
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &ResourcesArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    let sql = if args.physical { PHYSICAL_SQL } else { VIRTUAL_SQL };
    let result = client.execute(sql)?;
    let physical = args.physical;

    match args.format {
        OutputFormat::Table => display_table(&result, writer, physical)?,
        OutputFormat::Csv => display_csv(&result, writer, physical)?,
        OutputFormat::Json => display_json(&result, writer, physical)?,
        OutputFormat::Markdown | OutputFormat::Md => {
            display_markdown(&result, writer, physical)?;
        }
    }

    Ok(())
}

/// Execute resources query and return results for REPL mode
///
/// Used by the REPL metacommand handler. Includes error handling for
/// privilege and compatibility errors.
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    physical: bool,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    let sql = if physical { PHYSICAL_SQL } else { VIRTUAL_SQL };
    let table_name = if physical {
        "DBC.ResUsageSPMA"
    } else {
        "DBC.ResUsageSVPR"
    };

    match client.execute(sql) {
        Ok(result) => {
            let resources: Vec<ResourceInfo> = result
                .rows
                .iter()
                .filter_map(|row| ResourceInfo::from_row(row))
                .collect();

            if resources.is_empty() {
                let mode = if physical { "Physical" } else { "Virtual" };
                writeln!(writer, "Resources ({mode}):")?;
                writeln!(writer, "(no resource usage data found)")?;
                writeln!(writer)?;
                writeln!(writer, "0 entries")?;
            } else {
                display_resources_table(&resources, writer, physical)?;
                writeln!(writer)?;
                write_skew_summary(&resources, writer)?;
                writeln!(writer)?;
                let label = if physical { "node(s)" } else { "VPROC(s)" };
                writeln!(
                    writer,
                    "{} {} (Query time: {:.3}s)",
                    resources.len(),
                    label,
                    result.execution_time.as_secs_f64()
                )?;
            }
        }
        Err(e) => {
            let error_str = e.to_string().to_lowercase();

            if error_str.contains("privilege")
                || error_str.contains("access")
                || error_str.contains("permission")
                || error_str.contains("3523")
            {
                writeln!(writer, "Error: Insufficient privileges to query resource usage.")?;
                writeln!(writer)?;
                writeln!(writer, "Required: SELECT privilege on {table_name}")?;
                writeln!(writer)?;
                writeln!(writer, "To grant access, a DBA can run:")?;
                writeln!(writer, "  GRANT SELECT ON {table_name} TO <username>;")?;
            } else if error_str.contains("resusage")
                && (error_str.contains("syntax")
                    || error_str.contains("not found")
                    || error_str.contains("does not exist"))
            {
                writeln!(writer, "Error: ResUsage tables not available.")?;
                writeln!(writer)?;
                writeln!(
                    writer,
                    "This feature requires Teradata ResUsage logging to be enabled."
                )?;
                writeln!(
                    writer,
                    "The required table ({table_name}) may not exist on this system."
                )?;
            } else {
                writeln!(writer, "Error listing resource usage: {e}")?;
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Display resources in table format
fn display_table<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    physical: bool,
) -> Result<()> {
    let resources: Vec<ResourceInfo> = result
        .rows
        .iter()
        .filter_map(|row| ResourceInfo::from_row(row))
        .collect();

    let mode = if physical { "Physical" } else { "Virtual" };

    if resources.is_empty() {
        writeln!(writer, "Resources ({mode}):")?;
        writeln!(writer, "(no resource usage data found)")?;
        writeln!(writer)?;
        writeln!(writer, "0 entries")?;
        return Ok(());
    }

    display_resources_table(&resources, writer, physical)?;
    writeln!(writer)?;
    write_skew_summary(&resources, writer)?;
    writeln!(writer)?;
    let label = if physical { "node(s)" } else { "VPROC(s)" };
    writeln!(
        writer,
        "{} {} (Query time: {:.3}s)",
        resources.len(),
        label,
        result.execution_time.as_secs_f64()
    )?;

    Ok(())
}

/// Display resources using comfy_table
fn display_resources_table<W: Write>(
    resources: &[ResourceInfo],
    writer: &mut W,
    physical: bool,
) -> Result<()> {
    use comfy_table::{presets, Cell, CellAlignment, ContentArrangement, Table};

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    let (id_hdr, io_avg_hdr, io_peak_hdr, mem1_hdr, mem2_hdr) = if physical {
        ("NodeID", "AvgIOCnt", "PeakIOCnt", "MemSize", "MemFreeKB")
    } else {
        (
            "VprocNo",
            "AvgIOKB",
            "PeakIOKB",
            "MemCtxtPageReads",
            "MemTotalUsed",
        )
    };

    table.set_header(vec![
        id_hdr,
        "AvgCPUBusy",
        "PeakCPUBusy",
        io_avg_hdr,
        io_peak_hdr,
        mem1_hdr,
        mem2_hdr,
    ]);

    for r in resources {
        table.add_row(vec![
            Cell::new(r.id).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.2}", r.avg_cpu)).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.2}", r.peak_cpu)).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.2}", r.avg_io)).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.2}", r.peak_io)).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.2}", r.mem_metric1)).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.2}", r.mem_metric2)).set_alignment(CellAlignment::Right),
        ]);
    }

    let mode = if physical { "Physical" } else { "Virtual" };
    writeln!(writer, "Resources ({mode}):")?;
    writeln!(writer, "{table}")?;

    Ok(())
}

/// Write the skew summary footer showing CPU and I/O skew across all entries
fn write_skew_summary<W: Write>(resources: &[ResourceInfo], writer: &mut W) -> Result<()> {
    let cpu_values: Vec<f64> = resources.iter().map(|r| r.avg_cpu).collect();
    let io_values: Vec<f64> = resources.iter().map(|r| r.avg_io).collect();

    let cpu_skew = calculate_skew(&cpu_values);
    let io_skew = calculate_skew(&io_values);

    let cpu_str = cpu_skew
        .map(|v| format!("{v:.2}%"))
        .unwrap_or_else(|| "[--]".to_string());
    let io_str = io_skew
        .map(|v| format!("{v:.2}%"))
        .unwrap_or_else(|| "[--]".to_string());

    writeln!(writer, "Skew: CPU {cpu_str}, I/O {io_str}")?;

    Ok(())
}

/// Display resources in CSV format
fn display_csv<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    physical: bool,
) -> Result<()> {
    let (id_hdr, io_avg_hdr, io_peak_hdr, mem1_hdr, mem2_hdr) = if physical {
        ("NodeID", "AvgIOCnt", "PeakIOCnt", "MemSize", "MemFreeKB")
    } else {
        (
            "VprocNo",
            "AvgIOKB",
            "PeakIOKB",
            "MemCtxtPageReads",
            "MemTotalUsed",
        )
    };

    writeln!(
        writer,
        "{id_hdr},AvgCPUBusy,PeakCPUBusy,{io_avg_hdr},{io_peak_hdr},{mem1_hdr},{mem2_hdr}"
    )?;

    for row in &result.rows {
        if let Some(r) = ResourceInfo::from_row(row) {
            writeln!(
                writer,
                "{},{},{},{},{},{},{}",
                r.id,
                escape_csv(&format!("{:.2}", r.avg_cpu)),
                escape_csv(&format!("{:.2}", r.peak_cpu)),
                escape_csv(&format!("{:.2}", r.avg_io)),
                escape_csv(&format!("{:.2}", r.peak_io)),
                escape_csv(&format!("{:.2}", r.mem_metric1)),
                escape_csv(&format!("{:.2}", r.mem_metric2)),
            )?;
        }
    }

    Ok(())
}

/// Display resources in JSON format
fn display_json<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    physical: bool,
) -> Result<()> {
    let (id_key, io_avg_key, io_peak_key, mem1_key, mem2_key) = if physical {
        ("NodeID", "AvgIOCnt", "PeakIOCnt", "MemSize", "MemFreeKB")
    } else {
        (
            "VprocNo",
            "AvgIOKB",
            "PeakIOKB",
            "MemCtxtPageReads",
            "MemTotalUsed",
        )
    };

    let entries: Vec<serde_json::Value> = result
        .rows
        .iter()
        .filter_map(|row| {
            ResourceInfo::from_row(row).map(|r| {
                serde_json::json!({
                    id_key: r.id,
                    "AvgCPUBusy": r.avg_cpu,
                    "PeakCPUBusy": r.peak_cpu,
                    io_avg_key: r.avg_io,
                    io_peak_key: r.peak_io,
                    mem1_key: r.mem_metric1,
                    mem2_key: r.mem_metric2,
                })
            })
        })
        .collect();

    // Calculate skew for JSON output
    let resources: Vec<ResourceInfo> = result
        .rows
        .iter()
        .filter_map(|row| ResourceInfo::from_row(row))
        .collect();

    let cpu_values: Vec<f64> = resources.iter().map(|r| r.avg_cpu).collect();
    let io_values: Vec<f64> = resources.iter().map(|r| r.avg_io).collect();

    let json_output = serde_json::json!({
        "ok": true,
        "mode": if physical { "physical" } else { "virtual" },
        "row_count": entries.len(),
        "skew": {
            "cpu": calculate_skew(&cpu_values),
            "io": calculate_skew(&io_values),
        },
        "data": entries,
    });
    let json_str = serde_json::to_string_pretty(&json_output)?;
    writeln!(writer, "{json_str}")?;

    Ok(())
}

/// Display resources in Markdown format
fn display_markdown<W: Write>(
    result: &QueryResult,
    writer: &mut W,
    physical: bool,
) -> Result<()> {
    let resources: Vec<ResourceInfo> = result
        .rows
        .iter()
        .filter_map(|row| ResourceInfo::from_row(row))
        .collect();

    let (id_hdr, io_avg_hdr, io_peak_hdr, mem1_hdr, mem2_hdr) = if physical {
        ("NodeID", "AvgIOCnt", "PeakIOCnt", "MemSize", "MemFreeKB")
    } else {
        (
            "VprocNo",
            "AvgIOKB",
            "PeakIOKB",
            "MemCtxtPageReads",
            "MemTotalUsed",
        )
    };

    writeln!(
        writer,
        "| {} | AvgCPUBusy | PeakCPUBusy | {} | {} | {} | {} |",
        markdown_escape_pipe(id_hdr),
        markdown_escape_pipe(io_avg_hdr),
        markdown_escape_pipe(io_peak_hdr),
        markdown_escape_pipe(mem1_hdr),
        markdown_escape_pipe(mem2_hdr),
    )?;
    writeln!(
        writer,
        "| ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
    )?;

    for r in &resources {
        writeln!(
            writer,
            "| {} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} |",
            r.id, r.avg_cpu, r.peak_cpu, r.avg_io, r.peak_io, r.mem_metric1, r.mem_metric2,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Helper to build a valid virtual-mode row with 7 columns
    fn make_virtual_row(
        vproc: i64,
        avg_cpu: f64,
        peak_cpu: f64,
        avg_io: f64,
        peak_io: f64,
        mem_page_reads: f64,
        mem_total_used: f64,
    ) -> Vec<Value> {
        vec![
            Value::Integer(vproc),
            Value::Decimal(avg_cpu),
            Value::Decimal(peak_cpu),
            Value::Decimal(avg_io),
            Value::Decimal(peak_io),
            Value::Decimal(mem_page_reads),
            Value::Decimal(mem_total_used),
        ]
    }

    /// Helper to build a QueryResult from rows
    fn make_result(rows: Vec<Vec<Value>>) -> QueryResult {
        let row_count = rows.len();
        QueryResult {
            columns: vec![],
            rows,
            row_count,
            execution_time: Duration::from_millis(123),
        }
    }

    // =========================================================================
    // ResourceInfo::from_row tests
    // =========================================================================

    #[test]
    fn test_from_row_valid() {
        let row = make_virtual_row(0, 45.5, 78.3, 1024.0, 2048.0, 10.0, 500.0);
        let info = ResourceInfo::from_row(&row);
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.id, 0);
        assert!((info.avg_cpu - 45.5).abs() < 0.001);
        assert!((info.peak_cpu - 78.3).abs() < 0.001);
        assert!((info.avg_io - 1024.0).abs() < 0.001);
        assert!((info.peak_io - 2048.0).abs() < 0.001);
        assert!((info.mem_metric1 - 10.0).abs() < 0.001);
        assert!((info.mem_metric2 - 500.0).abs() < 0.001);
    }

    #[test]
    fn test_from_row_insufficient_columns() {
        let row = vec![Value::Integer(0), Value::Decimal(45.5)];
        assert!(ResourceInfo::from_row(&row).is_none());
    }

    #[test]
    fn test_from_row_empty() {
        let row: Vec<Value> = vec![];
        assert!(ResourceInfo::from_row(&row).is_none());
    }

    #[test]
    fn test_from_row_null_id() {
        let row = vec![
            Value::Null,
            Value::Decimal(45.5),
            Value::Decimal(78.3),
            Value::Decimal(1024.0),
            Value::Decimal(2048.0),
            Value::Decimal(10.0),
            Value::Decimal(500.0),
        ];
        // ID is required, so None should be returned
        assert!(ResourceInfo::from_row(&row).is_none());
    }

    #[test]
    fn test_from_row_null_metrics() {
        let row = vec![
            Value::Integer(1),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
        ];
        let info = ResourceInfo::from_row(&row);
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.id, 1);
        assert_eq!(info.avg_cpu, 0.0);
        assert_eq!(info.peak_cpu, 0.0);
        assert_eq!(info.avg_io, 0.0);
        assert_eq!(info.peak_io, 0.0);
        assert_eq!(info.mem_metric1, 0.0);
        assert_eq!(info.mem_metric2, 0.0);
    }

    #[test]
    fn test_from_row_integer_metrics() {
        // Teradata may return integers instead of decimals
        let row = vec![
            Value::Integer(2),
            Value::Integer(50),
            Value::Integer(80),
            Value::Integer(1024),
            Value::Integer(2048),
            Value::Integer(10),
            Value::Integer(500),
        ];
        let info = ResourceInfo::from_row(&row);
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.id, 2);
        assert!((info.avg_cpu - 50.0).abs() < 0.001);
        assert!((info.peak_cpu - 80.0).abs() < 0.001);
    }

    // =========================================================================
    // calculate_skew tests
    // =========================================================================

    #[test]
    fn test_calculate_skew_empty() {
        assert!(calculate_skew(&[]).is_none());
    }

    #[test]
    fn test_calculate_skew_all_zero() {
        assert!(calculate_skew(&[0.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn test_calculate_skew_perfect_balance() {
        let skew = calculate_skew(&[50.0, 50.0, 50.0, 50.0]);
        assert!(skew.is_some());
        assert!(skew.unwrap().abs() < 0.001);
    }

    #[test]
    fn test_calculate_skew_extreme() {
        // One high, rest zero: skew should be 75% for 4 values [100, 0, 0, 0]
        // avg = 25, max = 100, skew = 100*(1 - 25/100) = 75%
        let skew = calculate_skew(&[100.0, 0.0, 0.0, 0.0]);
        assert!(skew.is_some());
        assert!((skew.unwrap() - 75.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_skew_moderate() {
        // Values: [80, 90, 100, 70] -> avg=85, max=100, skew=15%
        let skew = calculate_skew(&[80.0, 90.0, 100.0, 70.0]);
        assert!(skew.is_some());
        assert!((skew.unwrap() - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_skew_single_value() {
        let skew = calculate_skew(&[42.0]);
        assert!(skew.is_some());
        assert!(skew.unwrap().abs() < 0.001);
    }

    // =========================================================================
    // display_table tests
    // =========================================================================

    #[test]
    fn test_display_table_virtual_empty() {
        let result = make_result(vec![]);
        let mut output = Vec::new();
        display_table(&result, &mut output, false).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Resources (Virtual)"));
        assert!(text.contains("no resource usage data found"));
        assert!(text.contains("0 entries"));
    }

    #[test]
    fn test_display_table_physical_empty() {
        let result = make_result(vec![]);
        let mut output = Vec::new();
        display_table(&result, &mut output, true).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Resources (Physical)"));
        assert!(text.contains("no resource usage data found"));
    }

    #[test]
    fn test_display_table_virtual_with_data() {
        let rows = vec![
            make_virtual_row(0, 45.5, 78.3, 1024.0, 2048.0, 10.0, 500.0),
            make_virtual_row(1, 55.0, 82.1, 900.0, 1800.0, 12.0, 600.0),
        ];
        let result = make_result(rows);
        let mut output = Vec::new();
        display_table(&result, &mut output, false).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Resources (Virtual)"));
        assert!(text.contains("VprocNo"));
        assert!(text.contains("AvgCPUBusy"));
        assert!(text.contains("MemTotalUsed"));
        assert!(text.contains("2 VPROC(s)"));
        assert!(text.contains("Skew:"));
    }

    #[test]
    fn test_display_table_physical_with_data() {
        let rows = vec![
            make_virtual_row(1, 60.0, 90.0, 500.0, 1000.0, 8192.0, 4096.0),
        ];
        let result = make_result(rows);
        let mut output = Vec::new();
        display_table(&result, &mut output, true).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Resources (Physical)"));
        assert!(text.contains("NodeID"));
        assert!(text.contains("MemFreeKB"));
        assert!(text.contains("1 node(s)"));
    }

    // =========================================================================
    // display_csv tests
    // =========================================================================

    #[test]
    fn test_display_csv_virtual() {
        let rows = vec![
            make_virtual_row(0, 45.5, 78.3, 1024.0, 2048.0, 10.0, 500.0),
        ];
        let result = make_result(rows);
        let mut output = Vec::new();
        display_csv(&result, &mut output, false).unwrap();
        let text = String::from_utf8(output).unwrap();

        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("VprocNo"));
        assert!(lines[0].contains("AvgIOKB"));
        assert!(lines[0].contains("MemCtxtPageReads"));
        assert!(lines[1].starts_with("0,"));
    }

    #[test]
    fn test_display_csv_physical() {
        let rows = vec![
            make_virtual_row(1, 60.0, 90.0, 500.0, 1000.0, 8192.0, 4096.0),
        ];
        let result = make_result(rows);
        let mut output = Vec::new();
        display_csv(&result, &mut output, true).unwrap();
        let text = String::from_utf8(output).unwrap();

        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[0].contains("NodeID"));
        assert!(lines[0].contains("AvgIOCnt"));
        assert!(lines[0].contains("MemFreeKB"));
    }

    #[test]
    fn test_display_csv_empty() {
        let result = make_result(vec![]);
        let mut output = Vec::new();
        display_csv(&result, &mut output, false).unwrap();
        let text = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        // Only the header line
        assert_eq!(lines.len(), 1);
    }

    // =========================================================================
    // display_json tests
    // =========================================================================

    #[test]
    fn test_display_json_virtual() {
        let rows = vec![
            make_virtual_row(0, 45.5, 78.3, 1024.0, 2048.0, 10.0, 500.0),
            make_virtual_row(1, 55.0, 82.1, 900.0, 1800.0, 12.0, 600.0),
        ];
        let result = make_result(rows);
        let mut output = Vec::new();
        display_json(&result, &mut output, false).unwrap();
        let text = String::from_utf8(output).unwrap();

        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["mode"], "virtual");
        assert_eq!(json["row_count"], 2);
        assert!(json["skew"]["cpu"].is_number());
        assert!(json["skew"]["io"].is_number());
        assert_eq!(json["data"].as_array().unwrap().len(), 2);
        assert!(json["data"][0]["VprocNo"].is_number());
    }

    #[test]
    fn test_display_json_physical() {
        let rows = vec![
            make_virtual_row(1, 60.0, 90.0, 500.0, 1000.0, 8192.0, 4096.0),
        ];
        let result = make_result(rows);
        let mut output = Vec::new();
        display_json(&result, &mut output, true).unwrap();
        let text = String::from_utf8(output).unwrap();

        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["mode"], "physical");
        assert!(json["data"][0]["NodeID"].is_number());
        assert!(json["data"][0]["MemFreeKB"].is_number());
    }

    #[test]
    fn test_display_json_empty() {
        let result = make_result(vec![]);
        let mut output = Vec::new();
        display_json(&result, &mut output, false).unwrap();
        let text = String::from_utf8(output).unwrap();

        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["row_count"], 0);
        assert!(json["data"].as_array().unwrap().is_empty());
        assert!(json["skew"]["cpu"].is_null());
        assert!(json["skew"]["io"].is_null());
    }

    // =========================================================================
    // display_markdown tests
    // =========================================================================

    #[test]
    fn test_display_markdown_virtual() {
        let rows = vec![
            make_virtual_row(0, 45.5, 78.3, 1024.0, 2048.0, 10.0, 500.0),
        ];
        let result = make_result(rows);
        let mut output = Vec::new();
        display_markdown(&result, &mut output, false).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("VprocNo"));
        assert!(text.contains("AvgIOKB"));
        assert!(text.contains("MemTotalUsed"));
        assert!(text.contains("| 0 |"));
        assert!(text.contains("45.50"));
    }

    #[test]
    fn test_display_markdown_physical() {
        let rows = vec![
            make_virtual_row(1, 60.0, 90.0, 500.0, 1000.0, 8192.0, 4096.0),
        ];
        let result = make_result(rows);
        let mut output = Vec::new();
        display_markdown(&result, &mut output, true).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("NodeID"));
        assert!(text.contains("MemFreeKB"));
    }

    #[test]
    fn test_display_markdown_empty() {
        let result = make_result(vec![]);
        let mut output = Vec::new();
        display_markdown(&result, &mut output, false).unwrap();
        let text = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        // Header + separator, no data rows
        assert_eq!(lines.len(), 2);
    }

    // =========================================================================
    // Skew summary tests
    // =========================================================================

    #[test]
    fn test_write_skew_summary_with_data() {
        let resources = vec![
            ResourceInfo {
                id: 0,
                avg_cpu: 80.0,
                peak_cpu: 90.0,
                avg_io: 100.0,
                peak_io: 200.0,
                mem_metric1: 10.0,
                mem_metric2: 500.0,
            },
            ResourceInfo {
                id: 1,
                avg_cpu: 100.0,
                peak_cpu: 100.0,
                avg_io: 50.0,
                peak_io: 150.0,
                mem_metric1: 12.0,
                mem_metric2: 600.0,
            },
        ];

        let mut output = Vec::new();
        write_skew_summary(&resources, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Skew:"));
        assert!(text.contains("CPU"));
        assert!(text.contains("I/O"));
        // CPU: avg=90, max=100, skew=10%
        assert!(text.contains("10.00%"));
        // IO: avg=75, max=100, skew=25%
        assert!(text.contains("25.00%"));
    }

    #[test]
    fn test_write_skew_summary_all_zero() {
        let resources = vec![
            ResourceInfo {
                id: 0,
                avg_cpu: 0.0,
                peak_cpu: 0.0,
                avg_io: 0.0,
                peak_io: 0.0,
                mem_metric1: 0.0,
                mem_metric2: 0.0,
            },
        ];

        let mut output = Vec::new();
        write_skew_summary(&resources, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("[--]"));
    }
}
