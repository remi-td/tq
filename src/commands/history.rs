//! Session history command implementation
//!
//! This module provides functionality to view historical session activity
//! from DBC.LogOnOffV, helping DBAs analyze logon/logoff patterns,
//! identify peak usage periods, and plan capacity.
//!
//! Sprint 51: Initial implementation (Issue #19)

use crate::cli::{HistoryArgs, OutputFormat};
use crate::db::{DatabaseClient, Value};
use crate::error::Result;
use super::monitoring_utils::{escape_csv, extract_integer, extract_trimmed_string};
use std::io::Write;

/// Default time range: 1 hour
const DEFAULT_DURATION: &str = "1h";

/// A session history event from DBC.LogOnOffV
#[derive(Debug, Clone)]
pub struct HistoryEvent {
    /// Session ID
    pub session_id: i64,
    /// Username
    pub user_name: String,
    /// Event type: Logon or Logoff
    pub event_type: String,
    /// Event timestamp
    pub event_time: String,
    /// Client IP/host address
    pub client_addr: String,
}

impl HistoryEvent {
    /// Create HistoryEvent from a query result row
    pub fn from_row(row: &[Value]) -> Option<Self> {
        if row.len() < 5 {
            return None;
        }

        let session_id = extract_integer(&row[0])?;
        let user_name = extract_trimmed_string(&row[1], "[NULL]");
        let event_type = extract_trimmed_string(&row[2], "[NULL]");
        let event_time = extract_trimmed_string(&row[3], "[unknown]");
        let client_addr = extract_trimmed_string(&row[4], "[unknown]");

        Some(Self {
            session_id,
            user_name,
            event_type,
            event_time,
            client_addr,
        })
    }
}

/// Parse a duration string like "1h", "24h", "7d", "30m" into a SQL interval
///
/// Returns the Teradata INTERVAL clause as a string.
pub fn parse_duration_to_interval(duration: &str) -> std::result::Result<String, String> {
    let duration = duration.trim().to_lowercase();

    if duration.is_empty() {
        return parse_duration_to_interval(DEFAULT_DURATION);
    }

    // Parse number + unit
    let (num_str, unit) = if duration.ends_with('d') {
        (&duration[..duration.len() - 1], "DAY")
    } else if duration.ends_with('h') {
        (&duration[..duration.len() - 1], "HOUR")
    } else if duration.ends_with('m') {
        (&duration[..duration.len() - 1], "MINUTE")
    } else {
        return Err(format!(
            "Invalid duration '{}'. Use format: 1h, 24h, 7d, 30m",
            duration
        ));
    };

    let num: u32 = num_str.parse().map_err(|_| {
        format!(
            "Invalid duration '{}'. Number part '{}' is not a valid integer.",
            duration, num_str
        )
    })?;

    if num == 0 {
        return Err("Duration must be greater than zero.".to_string());
    }

    // Teradata interval limit safeguard
    if unit == "DAY" && num > 365 {
        return Err("Duration cannot exceed 365 days.".to_string());
    }
    if unit == "HOUR" && num > 8760 {
        return Err("Duration cannot exceed 8760 hours (365 days).".to_string());
    }

    Ok(format!("INTERVAL '{}' {}", num, unit))
}

/// Build the SQL query for session history
fn build_history_sql(interval: &str, user_filter: Option<&str>) -> String {
    let user_clause = if let Some(user) = user_filter {
        // Sanitize by removing single quotes to prevent injection
        let sanitized = user.replace('\'', "");
        format!(" AND TRIM(UserName) = '{}'", sanitized)
    } else {
        String::new()
    };

    format!(
        r#"SELECT
    SessionNo,
    TRIM(UserName) AS UserName,
    CASE Event
        WHEN 'L' THEN 'Logon'
        WHEN 'O' THEN 'Logoff'
        WHEN 'A' THEN 'Auth Fail'
        ELSE TRIM(Event)
    END AS EventType,
    CAST(LogTS AS VARCHAR(30)) AS EventTime,
    COALESCE(TRIM(ClientAddr), '[unknown]') AS ClientAddr
FROM DBC.LogOnOffV
WHERE LogTS >= CURRENT_TIMESTAMP - {interval}{user_clause}
ORDER BY LogTS DESC"#,
        interval = interval,
        user_clause = user_clause
    )
}

/// Build SQL for session history summary statistics
fn build_summary_sql(interval: &str, user_filter: Option<&str>) -> String {
    let user_clause = if let Some(user) = user_filter {
        let sanitized = user.replace('\'', "");
        format!(" AND TRIM(UserName) = '{}'", sanitized)
    } else {
        String::new()
    };

    format!(
        r#"SELECT
    SUM(CASE WHEN Event = 'L' THEN 1 ELSE 0 END) AS Logons,
    SUM(CASE WHEN Event = 'O' THEN 1 ELSE 0 END) AS Logoffs,
    SUM(CASE WHEN Event = 'A' THEN 1 ELSE 0 END) AS AuthFails,
    COUNT(DISTINCT UserName) AS UniqueUsers,
    COUNT(*) AS TotalEvents
FROM DBC.LogOnOffV
WHERE LogTS >= CURRENT_TIMESTAMP - {interval}{user_clause}"#,
        interval = interval,
        user_clause = user_clause
    )
}

/// Summary statistics for session history
#[derive(Debug, Clone, Default)]
pub struct HistorySummary {
    pub logons: i64,
    pub logoffs: i64,
    pub auth_fails: i64,
    pub unique_users: i64,
    pub total_events: i64,
}

impl HistorySummary {
    fn from_row(row: &[Value]) -> Option<Self> {
        if row.len() < 5 {
            return None;
        }
        Some(Self {
            logons: extract_integer(&row[0]).unwrap_or(0),
            logoffs: extract_integer(&row[1]).unwrap_or(0),
            auth_fails: extract_integer(&row[2]).unwrap_or(0),
            unique_users: extract_integer(&row[3]).unwrap_or(0),
            total_events: extract_integer(&row[4]).unwrap_or(0),
        })
    }
}

/// Execute the history command in batch mode
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &HistoryArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    let duration_str = args.last.as_deref().unwrap_or(DEFAULT_DURATION);
    let interval = match parse_duration_to_interval(duration_str) {
        Ok(i) => i,
        Err(msg) => {
            writeln!(writer, "Error: {}", msg)?;
            return Ok(());
        }
    };

    let events = query_history(client, &interval, args.user.as_deref())?;
    let summary = query_summary(client, &interval, args.user.as_deref())?;

    match args.format {
        OutputFormat::Table => display_table(&events, &summary, duration_str, writer)?,
        OutputFormat::Csv => display_csv(&events, writer)?,
        OutputFormat::Json => display_json(&events, &summary, duration_str, writer)?,
        OutputFormat::Markdown | OutputFormat::Md => {
            display_markdown(&events, &summary, duration_str, writer)?
        }
    }

    Ok(())
}

/// Execute history in REPL mode
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    args: &[&str],
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    // Parse args: /history [--last <duration>] [--user <username>]
    let mut duration_str = DEFAULT_DURATION;
    let mut user_filter: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--last" | "-l" => {
                if i + 1 < args.len() {
                    duration_str = args[i + 1];
                    i += 2;
                } else {
                    writeln!(writer, "Error: --last requires a duration value (e.g., 1h, 24h, 7d)")?;
                    writeln!(writer)?;
                    return Ok(());
                }
            }
            "--user" | "-u" => {
                if i + 1 < args.len() {
                    user_filter = Some(args[i + 1]);
                    i += 2;
                } else {
                    writeln!(writer, "Error: --user requires a username")?;
                    writeln!(writer)?;
                    return Ok(());
                }
            }
            _ => {
                // Try to interpret as duration if it looks like one
                if args[i].ends_with('h') || args[i].ends_with('d') || args[i].ends_with('m') {
                    duration_str = args[i];
                    i += 1;
                } else {
                    writeln!(writer, "Unknown argument: {}", args[i])?;
                    writeln!(writer)?;
                    writeln!(writer, "Usage: /history [--last <duration>] [--user <username>]")?;
                    writeln!(writer)?;
                    writeln!(writer, "Duration formats: 30m, 1h, 24h, 7d")?;
                    writeln!(writer)?;
                    return Ok(());
                }
            }
        }
    }

    let interval = match parse_duration_to_interval(duration_str) {
        Ok(i) => i,
        Err(msg) => {
            writeln!(writer, "Error: {}", msg)?;
            writeln!(writer)?;
            return Ok(());
        }
    };

    match query_history_and_summary(client, &interval, user_filter) {
        Ok((events, summary)) => {
            if events.is_empty() {
                writeln!(writer, "Session History (last {}):", duration_str)?;
                writeln!(writer, "(no session events found)")?;
            } else {
                display_repl_output(&events, &summary, duration_str, user_filter, writer)?;
            }
        }
        Err(e) => {
            let error_str = e.to_string().to_lowercase();
            if error_str.contains("privilege")
                || error_str.contains("access")
                || error_str.contains("3523")
            {
                writeln!(writer, "Error: Insufficient privileges to query session history.")?;
                writeln!(writer)?;
                writeln!(writer, "Required: SELECT privilege on DBC.LogOnOffV")?;
                writeln!(writer)?;
                writeln!(writer, "To grant access, a DBA can run:")?;
                writeln!(writer, "  GRANT SELECT ON DBC.LogOnOffV TO <username>;")?;
            } else if error_str.contains("logonoff") && error_str.contains("not found") {
                writeln!(writer, "Error: DBC.LogOnOffV not available.")?;
                writeln!(writer)?;
                writeln!(writer, "Session logging may not be enabled on this system.")?;
            } else {
                writeln!(writer, "Error: {}", e)?;
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Query both history events and summary
fn query_history_and_summary(
    client: &DatabaseClient,
    interval: &str,
    user_filter: Option<&str>,
) -> Result<(Vec<HistoryEvent>, HistorySummary)> {
    let events = query_history(client, interval, user_filter)?;
    let summary = query_summary(client, interval, user_filter)?;
    Ok((events, summary))
}

/// Query session history events
fn query_history(
    client: &DatabaseClient,
    interval: &str,
    user_filter: Option<&str>,
) -> Result<Vec<HistoryEvent>> {
    let sql = build_history_sql(interval, user_filter);
    let result = client.execute(&sql)?;

    Ok(result
        .rows
        .iter()
        .filter_map(|row| HistoryEvent::from_row(row))
        .collect())
}

/// Query summary statistics
fn query_summary(
    client: &DatabaseClient,
    interval: &str,
    user_filter: Option<&str>,
) -> Result<HistorySummary> {
    let sql = build_summary_sql(interval, user_filter);
    let result = client.execute(&sql)?;

    Ok(result
        .rows
        .first()
        .and_then(|row| HistorySummary::from_row(row))
        .unwrap_or_default())
}

/// Display REPL output with summary header and event table
fn display_repl_output<W: Write>(
    events: &[HistoryEvent],
    summary: &HistorySummary,
    duration: &str,
    user_filter: Option<&str>,
    writer: &mut W,
) -> Result<()> {
    let title = if let Some(user) = user_filter {
        format!("Session History (last {}, user: {})", duration, user)
    } else {
        format!("Session History (last {})", duration)
    };
    writeln!(writer, "{}", title)?;
    writeln!(writer, "{}", "─".repeat(60))?;
    writeln!(
        writer,
        "  Logons: {}  |  Logoffs: {}  |  Auth Failures: {}  |  Unique Users: {}",
        summary.logons, summary.logoffs, summary.auth_fails, summary.unique_users
    )?;
    writeln!(writer, "{}", "─".repeat(60))?;
    writeln!(writer)?;

    use comfy_table::{presets, Cell, CellAlignment, ContentArrangement, Table};
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        "SessionNo", "UserName", "Event", "Time", "Client",
    ]);

    // Show at most 50 events in REPL
    let max_display = 50;
    for event in events.iter().take(max_display) {
        table.add_row(vec![
            Cell::new(event.session_id).set_alignment(CellAlignment::Right),
            Cell::new(&event.user_name),
            Cell::new(&event.event_type),
            Cell::new(&event.event_time),
            Cell::new(&event.client_addr),
        ]);
    }

    writeln!(writer, "{}", table)?;
    writeln!(writer)?;

    if events.len() > max_display {
        writeln!(
            writer,
            "Showing {} of {} events (use batch mode for full output)",
            max_display,
            events.len()
        )?;
    } else {
        writeln!(writer, "{} event(s)", events.len())?;
    }

    Ok(())
}

/// Display history in table format (batch mode)
fn display_table<W: Write>(
    events: &[HistoryEvent],
    summary: &HistorySummary,
    duration: &str,
    writer: &mut W,
) -> Result<()> {
    if events.is_empty() {
        writeln!(writer, "Session History (last {}):", duration)?;
        writeln!(writer, "(no session events found)")?;
        return Ok(());
    }

    use comfy_table::{presets, Cell, CellAlignment, ContentArrangement, Table};

    // Summary header
    writeln!(writer, "Session History (last {}):", duration)?;
    writeln!(writer, "{}", "─".repeat(60))?;
    writeln!(
        writer,
        "  Logons: {}  |  Logoffs: {}  |  Auth Failures: {}  |  Unique Users: {}",
        summary.logons, summary.logoffs, summary.auth_fails, summary.unique_users
    )?;
    writeln!(writer, "{}", "─".repeat(60))?;
    writeln!(writer)?;

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        "SessionNo", "UserName", "Event", "Time", "Client",
    ]);

    for event in events {
        table.add_row(vec![
            Cell::new(event.session_id).set_alignment(CellAlignment::Right),
            Cell::new(&event.user_name),
            Cell::new(&event.event_type),
            Cell::new(&event.event_time),
            Cell::new(&event.client_addr),
        ]);
    }

    writeln!(writer, "{}", table)?;
    writeln!(writer)?;
    writeln!(writer, "{} event(s)", events.len())?;

    Ok(())
}

/// Display history in CSV format
fn display_csv<W: Write>(events: &[HistoryEvent], writer: &mut W) -> Result<()> {
    writeln!(writer, "SessionNo,UserName,Event,Time,Client")?;
    for event in events {
        writeln!(
            writer,
            "{},{},{},{},{}",
            event.session_id,
            escape_csv(&event.user_name),
            escape_csv(&event.event_type),
            escape_csv(&event.event_time),
            escape_csv(&event.client_addr)
        )?;
    }
    Ok(())
}

/// Display history in JSON format
fn display_json<W: Write>(
    events: &[HistoryEvent],
    summary: &HistorySummary,
    duration: &str,
    writer: &mut W,
) -> Result<()> {
    let json_events: Vec<serde_json::Value> = events
        .iter()
        .map(|event| {
            serde_json::json!({
                "SessionNo": event.session_id,
                "UserName": event.user_name,
                "Event": event.event_type,
                "Time": event.event_time,
                "Client": event.client_addr
            })
        })
        .collect();

    let json = serde_json::json!({
        "Duration": duration,
        "Summary": {
            "Logons": summary.logons,
            "Logoffs": summary.logoffs,
            "AuthFailures": summary.auth_fails,
            "UniqueUsers": summary.unique_users,
            "TotalEvents": summary.total_events
        },
        "Events": json_events
    });
    let output = serde_json::to_string_pretty(&json)?;
    writeln!(writer, "{}", output)?;
    Ok(())
}

/// Display history in Markdown format
fn display_markdown<W: Write>(
    events: &[HistoryEvent],
    summary: &HistorySummary,
    duration: &str,
    writer: &mut W,
) -> Result<()> {
    fn esc(s: &str) -> String {
        s.replace('|', "\\|")
    }
    writeln!(writer, "## Session History (last {})", duration)?;
    writeln!(writer)?;
    writeln!(
        writer,
        "Logons: {} | Logoffs: {} | Auth Failures: {} | Unique Users: {}",
        summary.logons, summary.logoffs, summary.auth_fails, summary.unique_users
    )?;
    writeln!(writer)?;
    writeln!(writer, "| SessionNo | UserName | Event | Time | Client |")?;
    writeln!(writer, "| ---: | :--- | :--- | :--- | :--- |")?;
    for event in events {
        writeln!(
            writer,
            "| {} | {} | {} | {} | {} |",
            event.session_id,
            esc(&event.user_name),
            esc(&event.event_type),
            esc(&event.event_time),
            esc(&event.client_addr)
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_hours() {
        assert_eq!(
            parse_duration_to_interval("1h").unwrap(),
            "INTERVAL '1' HOUR"
        );
        assert_eq!(
            parse_duration_to_interval("24h").unwrap(),
            "INTERVAL '24' HOUR"
        );
    }

    #[test]
    fn test_parse_duration_days() {
        assert_eq!(
            parse_duration_to_interval("7d").unwrap(),
            "INTERVAL '7' DAY"
        );
        assert_eq!(
            parse_duration_to_interval("30d").unwrap(),
            "INTERVAL '30' DAY"
        );
    }

    #[test]
    fn test_parse_duration_minutes() {
        assert_eq!(
            parse_duration_to_interval("30m").unwrap(),
            "INTERVAL '30' MINUTE"
        );
    }

    #[test]
    fn test_parse_duration_uppercase() {
        assert_eq!(
            parse_duration_to_interval("1H").unwrap(),
            "INTERVAL '1' HOUR"
        );
        assert_eq!(
            parse_duration_to_interval("7D").unwrap(),
            "INTERVAL '7' DAY"
        );
    }

    #[test]
    fn test_parse_duration_invalid_unit() {
        assert!(parse_duration_to_interval("1x").is_err());
        assert!(parse_duration_to_interval("abc").is_err());
    }

    #[test]
    fn test_parse_duration_zero() {
        assert!(parse_duration_to_interval("0h").is_err());
    }

    #[test]
    fn test_parse_duration_empty() {
        // Empty defaults to 1h
        assert_eq!(
            parse_duration_to_interval("").unwrap(),
            "INTERVAL '1' HOUR"
        );
    }

    #[test]
    fn test_parse_duration_too_large_days() {
        assert!(parse_duration_to_interval("366d").is_err());
    }

    #[test]
    fn test_parse_duration_too_large_hours() {
        assert!(parse_duration_to_interval("8761h").is_err());
    }

    #[test]
    fn test_parse_duration_invalid_number() {
        assert!(parse_duration_to_interval("abch").is_err());
    }

    #[test]
    fn test_history_event_from_row() {
        let row = vec![
            Value::Integer(1234),
            Value::String("testuser".to_string()),
            Value::String("Logon".to_string()),
            Value::String("2026-03-23 10:00:00".to_string()),
            Value::String("192.168.1.1".to_string()),
        ];
        let event = HistoryEvent::from_row(&row);
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.session_id, 1234);
        assert_eq!(event.user_name, "testuser");
        assert_eq!(event.event_type, "Logon");
    }

    #[test]
    fn test_history_event_from_row_insufficient() {
        let row = vec![Value::Integer(1234), Value::String("user".to_string())];
        assert!(HistoryEvent::from_row(&row).is_none());
    }

    #[test]
    fn test_history_event_from_row_nulls() {
        let row = vec![
            Value::Integer(1234),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
        ];
        let event = HistoryEvent::from_row(&row);
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.user_name, "[NULL]");
        assert_eq!(event.event_type, "[NULL]");
    }

    #[test]
    fn test_history_summary_from_row() {
        let row = vec![
            Value::Integer(100),
            Value::Integer(95),
            Value::Integer(3),
            Value::Integer(15),
            Value::Integer(198),
        ];
        let summary = HistorySummary::from_row(&row);
        assert!(summary.is_some());
        let summary = summary.unwrap();
        assert_eq!(summary.logons, 100);
        assert_eq!(summary.logoffs, 95);
        assert_eq!(summary.auth_fails, 3);
        assert_eq!(summary.unique_users, 15);
        assert_eq!(summary.total_events, 198);
    }

    #[test]
    fn test_history_summary_default() {
        let summary = HistorySummary::default();
        assert_eq!(summary.logons, 0);
        assert_eq!(summary.logoffs, 0);
        assert_eq!(summary.total_events, 0);
    }

    #[test]
    fn test_build_history_sql_no_filter() {
        let sql = build_history_sql("INTERVAL '1' HOUR", None);
        assert!(sql.contains("DBC.LogOnOffV"));
        assert!(sql.contains("INTERVAL '1' HOUR"));
        assert!(!sql.contains("UserName ="));
    }

    #[test]
    fn test_build_history_sql_with_user() {
        let sql = build_history_sql("INTERVAL '24' HOUR", Some("testuser"));
        assert!(sql.contains("TRIM(UserName) = 'testuser'"));
    }

    #[test]
    fn test_build_history_sql_sanitizes_quotes() {
        let sql = build_history_sql("INTERVAL '1' HOUR", Some("user'name"));
        // Single quotes should be removed to prevent injection
        assert!(sql.contains("username"));
        assert!(!sql.contains("user'name"));
    }

    #[test]
    fn test_build_summary_sql() {
        let sql = build_summary_sql("INTERVAL '7' DAY", None);
        assert!(sql.contains("SUM(CASE WHEN Event = 'L' THEN 1"));
        assert!(sql.contains("DBC.LogOnOffV"));
        assert!(sql.contains("INTERVAL '7' DAY"));
    }

    #[test]
    fn test_display_csv() {
        let events = vec![
            HistoryEvent {
                session_id: 1234,
                user_name: "alice".to_string(),
                event_type: "Logon".to_string(),
                event_time: "2026-03-23 10:00:00".to_string(),
                client_addr: "192.168.1.1".to_string(),
            },
            HistoryEvent {
                session_id: 1234,
                user_name: "alice".to_string(),
                event_type: "Logoff".to_string(),
                event_time: "2026-03-23 11:00:00".to_string(),
                client_addr: "192.168.1.1".to_string(),
            },
        ];
        let mut output = Vec::new();
        display_csv(&events, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("SessionNo,UserName,Event,Time,Client"));
        assert!(s.contains("1234,alice,Logon,"));
        assert!(s.contains("1234,alice,Logoff,"));
    }

    #[test]
    fn test_display_json() {
        let events = vec![HistoryEvent {
            session_id: 1234,
            user_name: "bob".to_string(),
            event_type: "Logon".to_string(),
            event_time: "2026-03-23 10:00:00".to_string(),
            client_addr: "10.0.0.1".to_string(),
        }];
        let summary = HistorySummary {
            logons: 1,
            logoffs: 0,
            auth_fails: 0,
            unique_users: 1,
            total_events: 1,
        };
        let mut output = Vec::new();
        display_json(&events, &summary, "1h", &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(json["Duration"], "1h");
        assert_eq!(json["Summary"]["Logons"], 1);
        assert_eq!(json["Events"][0]["SessionNo"], 1234);
        assert_eq!(json["Events"][0]["UserName"], "bob");
    }

    #[test]
    fn test_display_table_empty() {
        let summary = HistorySummary::default();
        let mut output = Vec::new();
        display_table(&[], &summary, "1h", &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("no session events found"));
    }

    #[test]
    fn test_display_csv_special_chars() {
        let events = vec![HistoryEvent {
            session_id: 1234,
            user_name: "user, special".to_string(),
            event_type: "Logon".to_string(),
            event_time: "2026-03-23 10:00:00".to_string(),
            client_addr: "host".to_string(),
        }];
        let mut output = Vec::new();
        display_csv(&events, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("\"user, special\""));
    }

    #[test]
    fn test_display_json_empty() {
        let summary = HistorySummary::default();
        let mut output = Vec::new();
        display_json(&[], &summary, "24h", &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(json["Duration"], "24h");
        assert!(json["Events"].as_array().unwrap().is_empty());
        assert_eq!(json["Summary"]["TotalEvents"], 0);
    }
}
