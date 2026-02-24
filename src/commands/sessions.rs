//! Sessions command implementation
//!
//! This module provides functionality to list active Teradata sessions
//! with performance metrics including CPU/IO skew percentages.
//!
//! Sprint 26: Initial implementation

use crate::cli::{OutputFormat, SessionsArgs};
use crate::db::{DatabaseClient, QueryResult, Value};
use crate::error::Result;
use super::monitoring_utils::{extract_decimal, extract_integer, escape_csv};
use std::io::Write;

/// SQL query to retrieve session information from MonitorSession table function
///
/// Returns raw columns including AvgAmpCPUSec, HotAmp1CPU, AvgAmpIOCnt, HotAmp1IO
/// which are used to calculate skew percentages in Rust.
const SESSIONS_SQL: &str = r#"
SELECT
    SessionNo,
    UserName,
    LogonTime,
    PEState,
    AMPState,
    AMPCPUSec,
    AMPIO,
    ReqSpool,
    AvgAmpCPUSec,
    HotAmp1CPU,
    AvgAmpIOCnt,
    HotAmp1IO
FROM TABLE (MonitorSession(-1, '*', 0)) AS t1
ORDER BY SessionNo
"#;

/// Session information extracted from MonitorSession result
///
/// Contains the display columns plus calculated skew percentages.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Session identifier
    pub session_no: i64,
    /// Logged-in user name
    pub user_name: String,
    /// Session logon timestamp (formatted)
    pub logon_time: String,
    /// Parsing Engine state (IDLE/DISPATCHING/ACTIVE)
    pub pe_state: String,
    /// AMP state (IDLE/ACTIVE)
    pub amp_state: String,
    /// Total AMP CPU seconds consumed
    pub amp_cpu_sec: f64,
    /// Total AMP I/O count
    pub amp_io: i64,
    /// Requested spool space in bytes
    pub req_spool: i64,
    /// CPU skew percentage (None for idle sessions)
    pub cpu_skew: Option<f64>,
    /// IO skew percentage (None for idle sessions)
    pub io_skew: Option<f64>,
}

impl SessionInfo {
    /// Create SessionInfo from a query result row
    ///
    /// Extracts values from the 12 columns returned by SESSIONS_SQL and
    /// calculates skew percentages from the raw metrics.
    ///
    /// Returns None if required fields are missing or cannot be parsed.
    pub fn from_row(row: &[Value]) -> Option<Self> {
        if row.len() < 12 {
            return None;
        }

        let session_no = match &row[0] {
            Value::Integer(v) => *v,
            _ => return None,
        };

        let user_name = match &row[1] {
            Value::String(s) => s.trim().to_string(),
            Value::Null => "[NULL]".to_string(),
            _ => return None,
        };

        let logon_time = match &row[2] {
            Value::Timestamp(s) | Value::Date(s) | Value::String(s) => format_logon_time(s),
            Value::Null => "[NULL]".to_string(),
            _ => return None,
        };

        let pe_state = match &row[3] {
            Value::String(s) => s.trim().to_string(),
            Value::Null => "[NULL]".to_string(),
            other => other.display(),
        };

        let amp_state = match &row[4] {
            Value::String(s) => s.trim().to_string(),
            Value::Null => "[NULL]".to_string(),
            other => other.display(),
        };

        let amp_cpu_sec = extract_decimal(&row[5]).unwrap_or(0.0);
        let amp_io = extract_integer(&row[6]).unwrap_or(0);
        let req_spool = extract_integer(&row[7]).unwrap_or(0);

        // Extract raw values for skew calculation
        let avg_amp_cpu = extract_decimal(&row[8]).unwrap_or(0.0);
        let hot_amp1_cpu = extract_decimal(&row[9]).unwrap_or(0.0);
        let avg_amp_io = extract_decimal(&row[10]).unwrap_or(0.0);
        let hot_amp1_io = extract_decimal(&row[11]).unwrap_or(0.0);

        // Calculate skew percentages
        let cpu_skew = calculate_skew(avg_amp_cpu, hot_amp1_cpu);
        let io_skew = calculate_skew(avg_amp_io, hot_amp1_io);

        Some(Self {
            session_no,
            user_name,
            logon_time,
            pe_state,
            amp_state,
            amp_cpu_sec,
            amp_io,
            req_spool,
            cpu_skew,
            io_skew,
        })
    }
}

/// Calculate skew percentage from average and hot (maximum) values
///
/// Formula: skew = 100 * (1 - (avg / hot))
///
/// Returns None if hot value is zero (for idle sessions).
///
/// # Arguments
/// * `avg` - Average value across all AMPs
/// * `hot` - Maximum value (hottest AMP)
///
/// # Returns
/// * `Some(percentage)` - Skew percentage (0-100)
/// * `None` - If hot value is zero (session is idle)
pub fn calculate_skew(avg: f64, hot: f64) -> Option<f64> {
    if hot > 0.0 {
        Some(100.0 * (1.0 - (avg / hot)))
    } else {
        None
    }
}

/// Format logon time from Teradata timestamp format to user-friendly format
///
/// Converts "2026-01-27 15:33:26.00" to "2026/01/27 15:33:26.00"
fn format_logon_time(ts: &str) -> String {
    ts.replace('-', "/")
}

/// Format skew percentage for display
///
/// Returns "[--]" for None (idle sessions) or formatted percentage.
fn format_skew(skew: Option<f64>) -> String {
    match skew {
        Some(v) => format!("{:.2}", v),
        None => "[--]".to_string(),
    }
}

/// Execute the sessions command and write results
///
/// This is the main entry point for both batch mode and REPL mode.
///
/// # Arguments
/// * `client` - Database client for executing queries
/// * `args` - Command arguments (format, output file)
/// * `writer` - Output writer
/// * `use_color` - Whether to use color output
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(e)` - Database or I/O error
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &SessionsArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    // Execute the sessions query
    let result = client.execute(SESSIONS_SQL)?;

    // Process results based on output format
    match args.format {
        OutputFormat::Table => display_table(&result, writer)?,
        OutputFormat::Csv => display_csv(&result, writer)?,
        OutputFormat::Json => display_json(&result, writer)?,
    }

    Ok(())
}

/// Execute sessions query and return processed session information
///
/// This is used by the REPL metacommand handler.
pub fn execute_for_repl<W: Write>(client: &DatabaseClient, writer: &mut W) -> Result<()> {
    writeln!(writer)?;

    match client.execute(SESSIONS_SQL) {
        Ok(result) => {
            // Parse sessions from result
            let sessions: Vec<SessionInfo> = result
                .rows
                .iter()
                .filter_map(|row| SessionInfo::from_row(row))
                .collect();

            if sessions.is_empty() {
                writeln!(writer, "Sessions:")?;
                writeln!(writer, "(no active sessions found)")?;
                writeln!(writer)?;
                writeln!(writer, "0 active session(s)")?;
            } else {
                display_sessions_table(&sessions, writer)?;
                writeln!(writer)?;
                writeln!(
                    writer,
                    "{} active session(s) (Query time: {:.3}s)",
                    sessions.len(),
                    result.execution_time.as_secs_f64()
                )?;
            }
        }
        Err(e) => {
            let error_str = e.to_string().to_lowercase();

            // Check for privilege errors
            if error_str.contains("privilege")
                || error_str.contains("access")
                || error_str.contains("permission")
                || error_str.contains("3523")
            {
                writeln!(writer, "Error: Insufficient privileges to query sessions.")?;
                writeln!(writer)?;
                writeln!(writer, "Required: SELECT privilege on DBC.MonitorSession")?;
                writeln!(writer)?;
                writeln!(writer, "To grant access, a DBA can run:")?;
                writeln!(
                    writer,
                    "  GRANT SELECT ON DBC.MonitorSession TO <username>;"
                )?;
            } else if error_str.contains("monitorsession")
                && (error_str.contains("syntax") || error_str.contains("not found"))
            {
                // Version compatibility error
                writeln!(writer, "Error: MonitorSession function not available.")?;
                writeln!(writer)?;
                writeln!(writer, "This feature requires Teradata 14.10 or later.")?;
                writeln!(writer, "Your system may be running an earlier version.")?;
            } else {
                writeln!(writer, "Error listing sessions: {}", e)?;
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Display sessions in table format
fn display_table<W: Write>(result: &QueryResult, writer: &mut W) -> Result<()> {
    // Parse sessions
    let sessions: Vec<SessionInfo> = result
        .rows
        .iter()
        .filter_map(|row| SessionInfo::from_row(row))
        .collect();

    if sessions.is_empty() {
        writeln!(writer, "Sessions:")?;
        writeln!(writer, "(no active sessions found)")?;
        writeln!(writer)?;
        writeln!(writer, "0 active session(s)")?;
        return Ok(());
    }

    display_sessions_table(&sessions, writer)?;
    writeln!(writer)?;
    writeln!(
        writer,
        "{} active session(s) (Query time: {:.3}s)",
        sessions.len(),
        result.execution_time.as_secs_f64()
    )?;

    Ok(())
}

/// Display sessions using comfy_table
fn display_sessions_table<W: Write>(sessions: &[SessionInfo], writer: &mut W) -> Result<()> {
    use comfy_table::{presets, Cell, CellAlignment, ContentArrangement, Table};

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Set headers
    table.set_header(vec![
        "SessionNo",
        "UserName",
        "LogonTime",
        "PEState",
        "AMPState",
        "AMPCPUSec",
        "AMPIO",
        "ReqSpool",
        "Amp CPU Skew %",
        "Amp IO Skew %",
    ]);

    // Add rows
    for session in sessions {
        table.add_row(vec![
            Cell::new(session.session_no).set_alignment(CellAlignment::Right),
            Cell::new(&session.user_name),
            Cell::new(&session.logon_time),
            Cell::new(&session.pe_state),
            Cell::new(&session.amp_state),
            Cell::new(format!("{:.3}", session.amp_cpu_sec)).set_alignment(CellAlignment::Right),
            Cell::new(session.amp_io).set_alignment(CellAlignment::Right),
            Cell::new(format_spool(session.req_spool)).set_alignment(CellAlignment::Right),
            Cell::new(format_skew(session.cpu_skew)).set_alignment(CellAlignment::Right),
            Cell::new(format_skew(session.io_skew)).set_alignment(CellAlignment::Right),
        ]);
    }

    writeln!(writer, "Sessions:")?;
    writeln!(writer, "{}", table)?;

    Ok(())
}

/// Format spool value with thousand separators for large numbers
fn format_spool(spool: i64) -> String {
    if spool == 0 {
        return "0".to_string();
    }

    // Format with thousand separators
    let s = spool.abs().to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    if spool < 0 {
        format!("-{}", result)
    } else {
        result
    }
}

/// Display sessions in CSV format
fn display_csv<W: Write>(result: &QueryResult, writer: &mut W) -> Result<()> {
    // Write header
    writeln!(
        writer,
        "SessionNo,UserName,LogonTime,PEState,AMPState,AMPCPUSec,AMPIO,ReqSpool,Amp CPU Skew %,Amp IO Skew %"
    )?;

    // Parse and write sessions
    for row in &result.rows {
        if let Some(session) = SessionInfo::from_row(row) {
            let cpu_skew_str = session.cpu_skew.map(|v| format!("{:.2}", v)).unwrap_or_default();
            let io_skew_str = session.io_skew.map(|v| format!("{:.2}", v)).unwrap_or_default();

            writeln!(
                writer,
                "{},{},{},{},{},{:.3},{},{},{},{}",
                session.session_no,
                escape_csv(&session.user_name),
                escape_csv(&session.logon_time),
                escape_csv(&session.pe_state),
                escape_csv(&session.amp_state),
                session.amp_cpu_sec,
                session.amp_io,
                session.req_spool,
                cpu_skew_str,
                io_skew_str
            )?;
        }
    }

    Ok(())
}

/// Display sessions in JSON format
fn display_json<W: Write>(result: &QueryResult, writer: &mut W) -> Result<()> {
    let sessions: Vec<serde_json::Value> = result
        .rows
        .iter()
        .filter_map(|row| {
            SessionInfo::from_row(row).map(|session| {
                serde_json::json!({
                    "SessionNo": session.session_no,
                    "UserName": session.user_name,
                    "LogonTime": session.logon_time,
                    "PEState": session.pe_state,
                    "AMPState": session.amp_state,
                    "AMPCPUSec": session.amp_cpu_sec,
                    "AMPIO": session.amp_io,
                    "ReqSpool": session.req_spool,
                    "AmpCPUSkew": session.cpu_skew,
                    "AmpIOSkew": session.io_skew
                })
            })
        })
        .collect();

    let json_output = serde_json::to_string_pretty(&sessions)?;
    writeln!(writer, "{}", json_output)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_skew_active_session() {
        // Non-zero hot value should return skew percentage
        let skew = calculate_skew(80.0, 100.0);
        assert!(skew.is_some());
        let skew_value = skew.unwrap();
        assert!((skew_value - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_skew_idle_session() {
        // Zero hot value (idle session) should return None
        let skew = calculate_skew(0.0, 0.0);
        assert!(skew.is_none());
    }

    #[test]
    fn test_calculate_skew_perfect_balance() {
        // Average equals hot means 0% skew (perfect balance)
        let skew = calculate_skew(100.0, 100.0);
        assert!(skew.is_some());
        let skew_value = skew.unwrap();
        assert!(skew_value.abs() < 0.001);
    }

    #[test]
    fn test_calculate_skew_extreme_skew() {
        // Very small average with large hot means high skew
        let skew = calculate_skew(1.0, 100.0);
        assert!(skew.is_some());
        let skew_value = skew.unwrap();
        assert!((skew_value - 99.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_skew_null_handling() {
        // This tests that zero hot value returns None (NULL-like behavior)
        let skew = calculate_skew(50.0, 0.0);
        assert!(skew.is_none());
    }

    #[test]
    fn test_format_logon_time() {
        let result = format_logon_time("2026-01-27 15:33:26.00");
        assert_eq!(result, "2026/01/27 15:33:26.00");
    }

    #[test]
    fn test_format_logon_time_already_formatted() {
        let result = format_logon_time("2026/01/27 15:33:26.00");
        assert_eq!(result, "2026/01/27 15:33:26.00");
    }

    #[test]
    fn test_format_skew_some() {
        let result = format_skew(Some(12.345));
        assert_eq!(result, "12.35");
    }

    #[test]
    fn test_format_skew_none() {
        let result = format_skew(None);
        assert_eq!(result, "[--]");
    }

    #[test]
    fn test_format_spool_zero() {
        assert_eq!(format_spool(0), "0");
    }

    #[test]
    fn test_format_spool_small() {
        assert_eq!(format_spool(123), "123");
    }

    #[test]
    fn test_format_spool_large() {
        assert_eq!(format_spool(26753187840), "26,753,187,840");
    }

    #[test]
    fn test_format_spool_thousands() {
        assert_eq!(format_spool(1234567), "1,234,567");
    }

    #[test]
    fn test_escape_csv_simple() {
        assert_eq!(escape_csv("hello"), "hello");
    }

    #[test]
    fn test_escape_csv_with_comma() {
        assert_eq!(escape_csv("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn test_escape_csv_with_quotes() {
        assert_eq!(escape_csv("say \"hello\""), "\"say \"\"hello\"\"\"");
    }

    #[test]
    fn test_session_info_from_row() {
        let row = vec![
            Value::Integer(1076),
            Value::String("DBC".to_string()),
            Value::Timestamp("2026-01-27 15:33:26.00".to_string()),
            Value::String("IDLE".to_string()),
            Value::String("IDLE".to_string()),
            Value::Decimal(0.0),
            Value::Integer(6),
            Value::Integer(0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
        ];

        let session = SessionInfo::from_row(&row);
        assert!(session.is_some());

        let session = session.unwrap();
        assert_eq!(session.session_no, 1076);
        assert_eq!(session.user_name, "DBC");
        assert_eq!(session.logon_time, "2026/01/27 15:33:26.00");
        assert_eq!(session.pe_state, "IDLE");
        assert_eq!(session.amp_state, "IDLE");
        assert!(session.cpu_skew.is_none());
        assert!(session.io_skew.is_none());
    }

    #[test]
    fn test_session_info_from_row_active() {
        let row = vec![
            Value::Integer(1078),
            Value::String("DBC".to_string()),
            Value::Timestamp("2026-01-27 15:33:28.00".to_string()),
            Value::String("DISPATCHING".to_string()),
            Value::String("ACTIVE".to_string()),
            Value::Decimal(366.736),
            Value::Integer(75335),
            Value::Integer(26753187840),
            Value::Decimal(97.0),   // avg_amp_cpu
            Value::Decimal(100.0),  // hot_amp1_cpu
            Value::Decimal(96.22),  // avg_amp_io
            Value::Decimal(100.0),  // hot_amp1_io
        ];

        let session = SessionInfo::from_row(&row);
        assert!(session.is_some());

        let session = session.unwrap();
        assert_eq!(session.session_no, 1078);
        assert!(session.cpu_skew.is_some());
        assert!(session.io_skew.is_some());

        // CPU skew: 100 * (1 - 97/100) = 3%
        let cpu_skew = session.cpu_skew.unwrap();
        assert!((cpu_skew - 3.0).abs() < 0.01);

        // IO skew: 100 * (1 - 96.22/100) = 3.78%
        let io_skew = session.io_skew.unwrap();
        assert!((io_skew - 3.78).abs() < 0.01);
    }

    #[test]
    fn test_session_info_from_row_insufficient_columns() {
        let row = vec![
            Value::Integer(1076),
            Value::String("DBC".to_string()),
        ];

        let session = SessionInfo::from_row(&row);
        assert!(session.is_none());
    }

    #[test]
    fn test_session_info_from_row_with_nulls() {
        let row = vec![
            Value::Integer(1076),
            Value::Null,  // NULL user name
            Value::Timestamp("2026-01-27 15:33:26.00".to_string()),
            Value::Null,  // NULL PE state
            Value::String("IDLE".to_string()),
            Value::Null,  // NULL CPU sec
            Value::Null,  // NULL IO
            Value::Null,  // NULL spool
            Value::Null,  // NULL avg_amp_cpu
            Value::Null,  // NULL hot_amp1_cpu
            Value::Null,  // NULL avg_amp_io
            Value::Null,  // NULL hot_amp1_io
        ];

        let session = SessionInfo::from_row(&row);
        assert!(session.is_some());

        let session = session.unwrap();
        assert_eq!(session.user_name, "[NULL]");
        assert_eq!(session.pe_state, "[NULL]");
        assert_eq!(session.amp_cpu_sec, 0.0);
        assert_eq!(session.amp_io, 0);
        assert!(session.cpu_skew.is_none());
        assert!(session.io_skew.is_none());
    }

    #[test]
    fn test_extract_integer_from_integer() {
        let value = Value::Integer(42);
        assert_eq!(extract_integer(&value), Some(42));
    }

    #[test]
    fn test_extract_integer_from_decimal() {
        let value = Value::Decimal(42.5);
        assert_eq!(extract_integer(&value), Some(42));
    }

    #[test]
    fn test_extract_integer_from_null() {
        let value = Value::Null;
        assert_eq!(extract_integer(&value), None);
    }

    #[test]
    fn test_extract_decimal_from_decimal() {
        let value = Value::Decimal(42.5);
        let result = extract_decimal(&value);
        assert!(result.is_some());
        assert!((result.unwrap() - 42.5).abs() < 0.001);
    }

    #[test]
    fn test_extract_decimal_from_integer() {
        let value = Value::Integer(42);
        let result = extract_decimal(&value);
        assert!(result.is_some());
        assert!((result.unwrap() - 42.0).abs() < 0.001);
    }

    #[test]
    fn test_extract_decimal_from_null() {
        let value = Value::Null;
        assert_eq!(extract_decimal(&value), None);
    }

    #[test]
    fn test_session_info_from_row_with_non_string_state() {
        // Test that sessions with non-String pe_state/amp_state are NOT dropped
        // This is a regression test for Sprint 27 bug fix (issue #10)
        // Some Teradata versions may return Integer codes instead of String values
        let row = vec![
            Value::Integer(1079),
            Value::String("DBC".to_string()),
            Value::Timestamp("2026-01-27 15:33:29.00".to_string()),
            Value::Integer(1),  // Non-string PE state (was causing row to be dropped)
            Value::Integer(0),  // Non-string AMP state (was causing row to be dropped)
            Value::Decimal(0.0),
            Value::Integer(0),
            Value::Integer(0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
        ];

        let session = SessionInfo::from_row(&row);
        // Before fix: session would be None (row silently dropped)
        // After fix: session is Some with pe_state="1", amp_state="0"
        assert!(session.is_some(), "Session with non-String state values should NOT be dropped");

        let session = session.unwrap();
        assert_eq!(session.session_no, 1079);
        assert_eq!(session.pe_state, "1"); // Integer displayed as string
        assert_eq!(session.amp_state, "0"); // Integer displayed as string
    }

    #[test]
    fn test_session_info_from_row_with_boolean_state() {
        // Test that Boolean state values are also handled gracefully
        let row = vec![
            Value::Integer(1080),
            Value::String("DBC".to_string()),
            Value::Timestamp("2026-01-27 15:33:30.00".to_string()),
            Value::Boolean(true),   // Boolean PE state
            Value::Boolean(false),  // Boolean AMP state
            Value::Decimal(0.0),
            Value::Integer(0),
            Value::Integer(0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
            Value::Decimal(0.0),
        ];

        let session = SessionInfo::from_row(&row);
        assert!(session.is_some(), "Session with Boolean state values should NOT be dropped");

        let session = session.unwrap();
        assert_eq!(session.pe_state, "true");
        assert_eq!(session.amp_state, "false");
    }
}
