//! Skew analysis command implementation
//!
//! This module provides functionality to analyze AMP-level resource
//! distribution for Teradata sessions, helping identify hot AMPs
//! and data skew issues.
//!
//! Sprint 50: Initial implementation (Issue #24)

use crate::cli::{OutputFormat, SkewArgs};
use crate::commands::format_helpers::markdown_escape_pipe;
use crate::db::{DatabaseClient, Value};
use crate::error::Result;
use super::monitoring_utils::{escape_csv, extract_decimal, extract_integer, extract_trimmed_string};
use super::severity::MonitoringContext;
use std::io::Write;

/// SQL to retrieve AMP-level metrics for a specific session
fn build_skew_sql(session_id: i64) -> String {
    format!(
        r#"SELECT
    SessionNo,
    UserName,
    AMPCPUSec,
    AMPIO,
    ReqSpool,
    AvgAmpCPUSec,
    HotAmp1CPU,
    HotAmp1IOCnt AS HotAmp1IO,
    AvgAmpIOCnt,
    TotalIOCount,
    MaxAmpCPUSec,
    MinAmpCPUSec,
    MaxAmpIO,
    MinAmpIO
FROM TABLE (MonitorSession(-1, '*', 0)) AS t1
WHERE SessionNo = {}
"#,
        session_id
    )
}

/// SQL to retrieve top sessions by skew
const TOP_SKEW_SQL: &str = r#"
SELECT TOP 10
    SessionNo,
    UserName,
    AMPCPUSec,
    AMPIO,
    ReqSpool,
    AvgAmpCPUSec,
    HotAmp1CPU,
    HotAmp1IOCnt AS HotAmp1IO,
    AvgAmpIOCnt,
    TotalIOCount,
    MaxAmpCPUSec,
    MinAmpCPUSec,
    MaxAmpIO,
    MinAmpIO
FROM TABLE (MonitorSession(-1, '*', 0)) AS t1
WHERE AvgAmpCPUSec > 0 OR AvgAmpIOCnt > 0
ORDER BY CASE WHEN HotAmp1CPU > 0 THEN (1 - AvgAmpCPUSec / HotAmp1CPU) ELSE 0 END DESC
"#;

/// Skew information for a session
#[derive(Debug, Clone)]
pub struct SkewInfo {
    /// Session ID
    pub session_no: i64,
    /// Username
    pub user_name: String,
    /// Total AMP CPU seconds
    pub amp_cpu_sec: f64,
    /// Total AMP I/O count
    pub amp_io: i64,
    /// Spool space used
    pub req_spool: i64,
    /// CPU skew percentage
    pub cpu_skew: Option<f64>,
    /// IO skew percentage
    pub io_skew: Option<f64>,
    /// Max AMP CPU
    pub max_amp_cpu: f64,
    /// Min AMP CPU
    pub min_amp_cpu: f64,
    /// Max AMP IO
    pub max_amp_io: i64,
    /// Min AMP IO
    pub min_amp_io: i64,
}

impl SkewInfo {
    /// Create SkewInfo from a query result row
    pub fn from_row(row: &[Value]) -> Option<Self> {
        if row.len() < 14 {
            return None;
        }

        let session_no = extract_integer(&row[0])?;
        let user_name = extract_trimmed_string(&row[1], "[NULL]");
        let amp_cpu_sec = extract_decimal(&row[2]).unwrap_or(0.0);
        let amp_io = extract_integer(&row[3]).unwrap_or(0);
        let req_spool = extract_integer(&row[4]).unwrap_or(0);
        let avg_amp_cpu = extract_decimal(&row[5]).unwrap_or(0.0);
        let hot_amp1_cpu = extract_decimal(&row[6]).unwrap_or(0.0);
        let hot_amp1_io = extract_decimal(&row[7]).unwrap_or(0.0);
        let avg_amp_io = extract_decimal(&row[8]).unwrap_or(0.0);
        let _total_io = extract_integer(&row[9]).unwrap_or(0);
        let max_amp_cpu = extract_decimal(&row[10]).unwrap_or(0.0);
        let min_amp_cpu = extract_decimal(&row[11]).unwrap_or(0.0);
        let max_amp_io = extract_integer(&row[12]).unwrap_or(0);
        let min_amp_io = extract_integer(&row[13]).unwrap_or(0);

        let cpu_skew = calculate_skew(avg_amp_cpu, hot_amp1_cpu);
        let io_skew = calculate_skew(avg_amp_io, hot_amp1_io);

        Some(Self {
            session_no,
            user_name,
            amp_cpu_sec,
            amp_io,
            req_spool,
            cpu_skew,
            io_skew,
            max_amp_cpu,
            min_amp_cpu,
            max_amp_io,
            min_amp_io,
        })
    }
}

/// Calculate skew percentage
fn calculate_skew(avg: f64, hot: f64) -> Option<f64> {
    if hot > 0.0 {
        Some(100.0 * (1.0 - (avg / hot)))
    } else {
        None
    }
}

/// Render a skew percentage, colored by the configured `skew` thresholds
///
/// Severity is a separate axis from the four-word interpretation ladder below:
/// the words describe the distribution, the color reflects site policy.
/// A `None` skew has no measurement and is therefore never colored.
fn skew_cell(skew: Option<f64>, idle_text: &str, ctx: &MonitoringContext) -> String {
    match skew {
        Some(v) => ctx
            .styler
            .paint(ctx.thresholds.skew(v), &format!("{:.1}", v)),
        None => idle_text.to_string(),
    }
}

/// Format skew with interpretation hint
fn format_skew_with_hint(skew: Option<f64>) -> String {
    match skew {
        Some(v) if v < 10.0 => format!("{:.1}% (good)", v),
        Some(v) if v < 30.0 => format!("{:.1}% (moderate)", v),
        Some(v) if v < 60.0 => format!("{:.1}% (high)", v),
        Some(v) => format!("{:.1}% (severe)", v),
        None => "[idle]".to_string(),
    }
}

/// Execute the skew command in batch mode
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &SkewArgs,
    writer: &mut W,
    ctx: &MonitoringContext,
) -> Result<()> {
    let infos = if let Some(session_id) = args.session_id {
        query_session_skew(client, session_id)?
    } else {
        query_top_skew(client)?
    };

    match args.format {
        OutputFormat::Table => display_table(&infos, args.session_id, writer, ctx)?,
        OutputFormat::Csv => display_csv(&infos, writer)?,
        OutputFormat::Json => display_json(&infos, writer)?,
        OutputFormat::Markdown | OutputFormat::Md => display_markdown(&infos, writer, ctx)?,
    }

    Ok(())
}

/// Execute skew analysis in REPL mode
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    session_id: Option<i64>,
    writer: &mut W,
    ctx: &MonitoringContext,
) -> Result<()> {
    writeln!(writer)?;

    let infos = if let Some(sid) = session_id {
        match query_session_skew(client, sid) {
            Ok(i) => i,
            Err(e) => {
                let error_str = e.to_string().to_lowercase();
                if error_str.contains("privilege")
                    || error_str.contains("access")
                    || error_str.contains("3523")
                {
                    writeln!(writer, "Error: Insufficient privileges to query session metrics.")?;
                    writeln!(writer)?;
                    writeln!(writer, "Required: SELECT privilege on DBC.MonitorSession")?;
                } else {
                    writeln!(writer, "Error: {}", e)?;
                }
                writeln!(writer)?;
                return Ok(());
            }
        }
    } else {
        match query_top_skew(client) {
            Ok(i) => i,
            Err(e) => {
                writeln!(writer, "Error: {}", e)?;
                writeln!(writer)?;
                return Ok(());
            }
        }
    };

    if infos.is_empty() {
        if let Some(sid) = session_id {
            writeln!(writer, "Session {} not found or has no activity.", sid)?;
        } else {
            writeln!(writer, "No active sessions with measurable resource usage.")?;
        }
    } else if session_id.is_some() {
        // Single session detail view
        let info = &infos[0];
        writeln!(writer, "Skew Analysis for Session {}", info.session_no)?;
        writeln!(writer, "{}", "─".repeat(50))?;
        writeln!(writer, "  User:       {}", info.user_name)?;
        writeln!(writer, "  AMP CPU:    {:.3}s", info.amp_cpu_sec)?;
        writeln!(writer, "  AMP I/O:    {}", info.amp_io)?;
        writeln!(writer, "  Spool:      {} bytes", info.req_spool)?;
        writeln!(writer)?;
        writeln!(writer, "  CPU Skew:   {}", format_skew_with_hint(info.cpu_skew))?;
        writeln!(writer, "  I/O Skew:   {}", format_skew_with_hint(info.io_skew))?;
        writeln!(writer)?;
        writeln!(writer, "  AMP CPU Range: {:.3}s - {:.3}s", info.min_amp_cpu, info.max_amp_cpu)?;
        writeln!(writer, "  AMP I/O Range: {} - {}", info.min_amp_io, info.max_amp_io)?;
    } else {
        // Top skew summary view
        writeln!(writer, "Top Sessions by Skew (active only)")?;
        writeln!(writer, "{}", "─".repeat(80))?;

        use comfy_table::{presets, Cell, CellAlignment, ContentArrangement, Table};
        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL);
        table.set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec![
            "SessionNo", "UserName", "AMP CPU", "AMP I/O",
            "CPU Skew %", "I/O Skew %", "Interpretation",
        ]);

        for info in &infos {
            let cpu_skew_str = skew_cell(info.cpu_skew, "[idle]", ctx);
            let io_skew_str = skew_cell(info.io_skew, "[idle]", ctx);
            let interpretation = match info.cpu_skew {
                Some(v) if v < 10.0 => "good",
                Some(v) if v < 30.0 => "moderate",
                Some(v) if v < 60.0 => "high",
                Some(_) => "severe",
                None => "idle",
            };

            table.add_row(vec![
                Cell::new(info.session_no).set_alignment(CellAlignment::Right),
                Cell::new(&info.user_name),
                Cell::new(format!("{:.3}", info.amp_cpu_sec)).set_alignment(CellAlignment::Right),
                Cell::new(info.amp_io).set_alignment(CellAlignment::Right),
                Cell::new(&cpu_skew_str).set_alignment(CellAlignment::Right),
                Cell::new(&io_skew_str).set_alignment(CellAlignment::Right),
                Cell::new(interpretation),
            ]);
        }

        writeln!(writer, "{}", table)?;
        writeln!(writer)?;
        writeln!(writer, "{} session(s) shown", infos.len())?;
    }

    writeln!(writer)?;
    Ok(())
}

/// Query skew metrics for a specific session
fn query_session_skew(client: &DatabaseClient, session_id: i64) -> Result<Vec<SkewInfo>> {
    let sql = build_skew_sql(session_id);
    let result = client.execute(&sql)?;

    Ok(result
        .rows
        .iter()
        .filter_map(|row| SkewInfo::from_row(row))
        .collect())
}

/// Query top sessions by skew
fn query_top_skew(client: &DatabaseClient) -> Result<Vec<SkewInfo>> {
    let result = client.execute(TOP_SKEW_SQL)?;

    Ok(result
        .rows
        .iter()
        .filter_map(|row| SkewInfo::from_row(row))
        .collect())
}

/// Display skew info in table format
fn display_table<W: Write>(
    infos: &[SkewInfo],
    session_id: Option<i64>,
    writer: &mut W,
    ctx: &MonitoringContext,
) -> Result<()> {
    if infos.is_empty() {
        if let Some(sid) = session_id {
            writeln!(writer, "Session {} not found or has no activity.", sid)?;
        } else {
            writeln!(writer, "No active sessions with measurable resource usage.")?;
        }
        return Ok(());
    }

    use comfy_table::{presets, Cell, CellAlignment, ContentArrangement, Table};
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        "SessionNo", "UserName", "AMP CPU (s)", "AMP I/O",
        "CPU Skew %", "I/O Skew %", "Max CPU", "Min CPU",
        "Max I/O", "Min I/O",
    ]);

    for info in infos {
        let cpu_skew_str = skew_cell(info.cpu_skew, "[--]", ctx);
        let io_skew_str = skew_cell(info.io_skew, "[--]", ctx);

        table.add_row(vec![
            Cell::new(info.session_no).set_alignment(CellAlignment::Right),
            Cell::new(&info.user_name),
            Cell::new(format!("{:.3}", info.amp_cpu_sec)).set_alignment(CellAlignment::Right),
            Cell::new(info.amp_io).set_alignment(CellAlignment::Right),
            Cell::new(&cpu_skew_str).set_alignment(CellAlignment::Right),
            Cell::new(&io_skew_str).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.3}", info.max_amp_cpu)).set_alignment(CellAlignment::Right),
            Cell::new(format!("{:.3}", info.min_amp_cpu)).set_alignment(CellAlignment::Right),
            Cell::new(info.max_amp_io).set_alignment(CellAlignment::Right),
            Cell::new(info.min_amp_io).set_alignment(CellAlignment::Right),
        ]);
    }

    writeln!(writer, "Skew Analysis:")?;
    writeln!(writer, "{}", table)?;
    writeln!(writer)?;
    writeln!(writer, "{} session(s)", infos.len())?;

    Ok(())
}

/// Display skew info in CSV format
fn display_csv<W: Write>(infos: &[SkewInfo], writer: &mut W) -> Result<()> {
    writeln!(writer, "SessionNo,UserName,AMPCPUSec,AMPIO,CPUSkew,IOSkew,MaxCPU,MinCPU,MaxIO,MinIO")?;
    for info in infos {
        let cpu_skew_str = info.cpu_skew.map(|v| format!("{:.1}", v)).unwrap_or_default();
        let io_skew_str = info.io_skew.map(|v| format!("{:.1}", v)).unwrap_or_default();
        writeln!(
            writer,
            "{},{},{:.3},{},{},{},{:.3},{:.3},{},{}",
            info.session_no,
            escape_csv(&info.user_name),
            info.amp_cpu_sec,
            info.amp_io,
            cpu_skew_str,
            io_skew_str,
            info.max_amp_cpu,
            info.min_amp_cpu,
            info.max_amp_io,
            info.min_amp_io
        )?;
    }
    Ok(())
}

/// Display skew info in JSON format
fn display_json<W: Write>(infos: &[SkewInfo], writer: &mut W) -> Result<()> {
    let sessions: Vec<serde_json::Value> = infos
        .iter()
        .map(|info| {
            serde_json::json!({
                "SessionNo": info.session_no,
                "UserName": info.user_name,
                "AMPCPUSec": info.amp_cpu_sec,
                "AMPIO": info.amp_io,
                "ReqSpool": info.req_spool,
                "CPUSkew": info.cpu_skew,
                "IOSkew": info.io_skew,
                "MaxAmpCPU": info.max_amp_cpu,
                "MinAmpCPU": info.min_amp_cpu,
                "MaxAmpIO": info.max_amp_io,
                "MinAmpIO": info.min_amp_io,
            })
        })
        .collect();

    let output = serde_json::json!({
        "ok": true,
        "row_count": sessions.len(),
        "data": sessions
    });
    let json_str = serde_json::to_string_pretty(&output)?;
    writeln!(writer, "{}", json_str)?;
    Ok(())
}

/// Display skew info in Markdown format
fn display_markdown<W: Write>(
    infos: &[SkewInfo],
    writer: &mut W,
    ctx: &MonitoringContext,
) -> Result<()> {
    writeln!(
        writer,
        "| SessionNo | UserName | AMP CPU (s) | AMP I/O | CPU Skew % | I/O Skew % | Max CPU | Min CPU | Max I/O | Min I/O |"
    )?;
    writeln!(
        writer,
        "| ---: | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
    )?;
    for info in infos {
        let cpu_skew_str = skew_cell(info.cpu_skew, "[--]", ctx);
        let io_skew_str = skew_cell(info.io_skew, "[--]", ctx);
        writeln!(
            writer,
            "| {} | {} | {:.3} | {} | {} | {} | {:.3} | {:.3} | {} | {} |",
            info.session_no,
            markdown_escape_pipe(&info.user_name),
            info.amp_cpu_sec,
            info.amp_io,
            cpu_skew_str,
            io_skew_str,
            info.max_amp_cpu,
            info.min_amp_cpu,
            info.max_amp_io,
            info.min_amp_io
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_skew_active() {
        let skew = calculate_skew(80.0, 100.0);
        assert!(skew.is_some());
        assert!((skew.unwrap() - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_skew_idle() {
        assert!(calculate_skew(0.0, 0.0).is_none());
    }

    #[test]
    fn test_calculate_skew_perfect() {
        let skew = calculate_skew(100.0, 100.0);
        assert!(skew.unwrap().abs() < 0.01);
    }

    #[test]
    fn test_format_skew_good() {
        assert!(format_skew_with_hint(Some(5.0)).contains("good"));
    }

    #[test]
    fn test_format_skew_moderate() {
        assert!(format_skew_with_hint(Some(20.0)).contains("moderate"));
    }

    #[test]
    fn test_format_skew_high() {
        assert!(format_skew_with_hint(Some(45.0)).contains("high"));
    }

    #[test]
    fn test_format_skew_severe() {
        assert!(format_skew_with_hint(Some(75.0)).contains("severe"));
    }

    #[test]
    fn test_format_skew_idle() {
        assert!(format_skew_with_hint(None).contains("idle"));
    }

    #[test]
    fn test_skew_info_from_row() {
        let row = vec![
            Value::Integer(1234),
            Value::String("DBC".to_string()),
            Value::Decimal(100.5),
            Value::Integer(5000),
            Value::Integer(1024000),
            Value::Decimal(25.0),  // avg_amp_cpu
            Value::Decimal(50.0),  // hot_amp1_cpu
            Value::Decimal(40.0),  // hot_amp1_io
            Value::Decimal(30.0),  // avg_amp_io
            Value::Integer(10000), // total_io
            Value::Decimal(50.0),  // max_amp_cpu
            Value::Decimal(10.0),  // min_amp_cpu
            Value::Integer(100),   // max_amp_io
            Value::Integer(5),     // min_amp_io
        ];

        let info = SkewInfo::from_row(&row);
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.session_no, 1234);
        assert_eq!(info.user_name, "DBC");
        assert!((info.amp_cpu_sec - 100.5).abs() < 0.01);
        assert_eq!(info.amp_io, 5000);

        // CPU skew: 100 * (1 - 25/50) = 50%
        assert!(info.cpu_skew.is_some());
        assert!((info.cpu_skew.unwrap() - 50.0).abs() < 0.01);

        // IO skew: 100 * (1 - 30/40) = 25%
        assert!(info.io_skew.is_some());
        assert!((info.io_skew.unwrap() - 25.0).abs() < 0.01);

        assert!((info.max_amp_cpu - 50.0).abs() < 0.01);
        assert!((info.min_amp_cpu - 10.0).abs() < 0.01);
        assert_eq!(info.max_amp_io, 100);
        assert_eq!(info.min_amp_io, 5);
    }

    #[test]
    fn test_skew_info_from_row_insufficient_cols() {
        let row = vec![Value::Integer(1234), Value::String("DBC".to_string())];
        assert!(SkewInfo::from_row(&row).is_none());
    }

    #[test]
    fn test_skew_info_from_row_null_session() {
        let row = vec![
            Value::Null,
            Value::String("DBC".to_string()),
            Value::Decimal(0.0),
            Value::Integer(0),
            Value::Integer(0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
            Value::Integer(0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
            Value::Integer(0),
            Value::Integer(0),
        ];
        assert!(SkewInfo::from_row(&row).is_none());
    }

    #[test]
    fn test_display_table_empty() {
        let mut output = Vec::new();
        display_table(&[], Some(1234), &mut output, &MonitoringContext::default()).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Session 1234 not found"));
    }

    #[test]
    fn test_display_table_empty_no_session() {
        let mut output = Vec::new();
        display_table(&[], None, &mut output, &MonitoringContext::default()).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("No active sessions"));
    }

    #[test]
    fn test_display_csv() {
        let infos = vec![SkewInfo {
            session_no: 1234,
            user_name: "testuser".to_string(),
            amp_cpu_sec: 100.5,
            amp_io: 5000,
            req_spool: 1024000,
            cpu_skew: Some(50.0),
            io_skew: Some(25.0),
            max_amp_cpu: 50.0,
            min_amp_cpu: 10.0,
            max_amp_io: 100,
            min_amp_io: 5,
        }];
        let mut output = Vec::new();
        display_csv(&infos, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("SessionNo,UserName"));
        assert!(s.contains("1234,testuser,100.500,5000,50.0,25.0"));
    }

    #[test]
    fn test_display_json() {
        let infos = vec![SkewInfo {
            session_no: 1234,
            user_name: "testuser".to_string(),
            amp_cpu_sec: 100.5,
            amp_io: 5000,
            req_spool: 1024000,
            cpu_skew: Some(50.0),
            io_skew: Some(25.0),
            max_amp_cpu: 50.0,
            min_amp_cpu: 10.0,
            max_amp_io: 100,
            min_amp_io: 5,
        }];
        let mut output = Vec::new();
        display_json(&infos, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["row_count"], 1);
        let data = json["data"].as_array().unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0]["SessionNo"], 1234);
        assert_eq!(data[0]["CPUSkew"], 50.0);
        assert_eq!(data[0]["IOSkew"], 25.0);
    }

    #[test]
    fn test_display_csv_null_skew() {
        let infos = vec![SkewInfo {
            session_no: 9999,
            user_name: "idle_user".to_string(),
            amp_cpu_sec: 0.0,
            amp_io: 0,
            req_spool: 0,
            cpu_skew: None,
            io_skew: None,
            max_amp_cpu: 0.0,
            min_amp_cpu: 0.0,
            max_amp_io: 0,
            min_amp_io: 0,
        }];
        let mut output = Vec::new();
        display_csv(&infos, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        // Empty skew values for idle sessions
        assert!(s.contains("9999,idle_user,0.000,0,,,"));
    }

    #[test]
    fn test_build_skew_sql() {
        let sql = build_skew_sql(1234);
        assert!(sql.contains("WHERE SessionNo = 1234"));
        assert!(sql.contains("MonitorSession"));
    }
}
