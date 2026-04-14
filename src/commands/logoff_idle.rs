//! Logoff idle sessions command implementation
//!
//! This module provides functionality to detect and terminate idle Teradata
//! sessions that have been connected longer than a configurable threshold.
//! Useful for cleaning up stale connections that consume resources.
//!
//! Sprint 61: Initial implementation

use crate::cli::{LogoffIdleArgs, OutputFormat};
use crate::commands::abort;
use crate::db::{parse_duration, DatabaseClient, Value};
use crate::error::Result;
use chrono::TimeZone;
use super::monitoring_utils::escape_csv;
use std::io::Write;
use std::time::Duration;

/// Default idle threshold: 1 hour
const DEFAULT_OLDER_THAN: &str = "1h";

/// SQL to find idle sessions with their logon timestamps
const IDLE_SESSIONS_SQL: &str = r#"
SELECT
    SessionNo,
    UserName,
    LogonTime
FROM TABLE (MonitorSession(-1, '*', 0)) AS t1
WHERE PEState = 'IDLE'
"#;

/// An idle session found by the monitoring query
#[derive(Debug, Clone)]
pub struct IdleSession {
    /// Session ID
    pub session_id: i64,
    /// Username
    pub user_name: String,
    /// Logon timestamp string (from Teradata)
    pub logon_time: String,
    /// How long the session has been connected (estimated)
    pub idle_duration_desc: String,
}

/// Result of a logoff-idle operation
#[derive(Debug, Clone)]
pub struct LogoffIdleResult {
    /// The threshold duration used for filtering
    pub threshold_desc: String,
    /// All idle sessions found (before threshold filtering)
    pub total_idle: usize,
    /// Sessions that matched the age threshold
    pub eligible: Vec<IdleSession>,
    /// Abort results for each terminated session
    pub abort_results: Vec<abort::AbortResult>,
}

/// Execute the logoff-idle command in batch mode
///
/// Requires `--force` flag for non-interactive operation.
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &LogoffIdleArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    if !args.force {
        writeln!(writer, "Error: Logoff-idle requires --force flag in batch mode.")?;
        writeln!(writer)?;
        writeln!(
            writer,
            "This is a destructive operation that will terminate idle sessions."
        )?;
        writeln!(
            writer,
            "Add --force to confirm: tq logoff-idle --force --older-than {}",
            args.older_than
        )?;
        return Ok(());
    }

    let threshold = parse_duration(&args.older_than)?;
    let result = find_and_abort_idle_sessions(client, threshold)?;

    match args.format {
        OutputFormat::Table => display_table(&result, writer)?,
        OutputFormat::Csv => display_csv(&result, writer)?,
        OutputFormat::Json => display_json(&result, writer)?,
        OutputFormat::Markdown | OutputFormat::Md => display_markdown(&result, writer)?,
    }

    Ok(())
}

/// Execute logoff-idle in REPL mode
///
/// Parses REPL-style arguments: /logoff idle [--older-than <duration>] [yes]
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    args: &[&str],
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    // Parse args: /logoff idle [--older-than <duration>] [yes]
    let mut duration_str = DEFAULT_OLDER_THAN;
    let mut confirmed = false;
    let mut i = 0;

    while i < args.len() {
        match args[i] {
            "--older-than" | "-t" => {
                if i + 1 < args.len() {
                    duration_str = args[i + 1];
                    i += 2;
                } else {
                    writeln!(
                        writer,
                        "Error: --older-than requires a duration value (e.g., 1h, 2h, 30m)"
                    )?;
                    writeln!(writer)?;
                    return Ok(());
                }
            }
            arg if arg.eq_ignore_ascii_case("yes") => {
                confirmed = true;
                i += 1;
            }
            _ => {
                // Try to interpret as duration
                if args[i].ends_with('h')
                    || args[i].ends_with('m')
                    || args[i].ends_with('d')
                    || args[i].ends_with('s')
                {
                    duration_str = args[i];
                    i += 1;
                } else {
                    writeln!(writer, "Unknown argument: {}", args[i])?;
                    writeln!(writer)?;
                    writeln!(
                        writer,
                        "Usage: /logoff idle [--older-than <duration>] [yes]"
                    )?;
                    writeln!(writer)?;
                    writeln!(writer, "Duration formats: 30m, 1h, 2h, 24h")?;
                    writeln!(writer)?;
                    return Ok(());
                }
            }
        }
    }

    let threshold = match parse_duration(duration_str) {
        Ok(d) => d,
        Err(e) => {
            writeln!(writer, "Error: {}", e)?;
            writeln!(writer)?;
            return Ok(());
        }
    };

    let idle_sessions = find_idle_sessions(client, threshold)?;

    if idle_sessions.is_empty() {
        writeln!(
            writer,
            "No idle sessions older than {} found.",
            format_duration(threshold)
        )?;
        writeln!(writer)?;
        return Ok(());
    }

    if !confirmed {
        writeln!(
            writer,
            "Found {} idle session(s) older than {}:",
            idle_sessions.len(),
            format_duration(threshold)
        )?;
        writeln!(writer)?;
        writeln!(writer, "  {:>10}  {:16}  LogonTime", "SessionNo", "UserName")?;
        writeln!(writer, "  {:>10}  {:16}  -------------------", "----------", "----------------")?;
        for sess in &idle_sessions {
            writeln!(
                writer,
                "  {:>10}  {:16}  {}",
                sess.session_id, sess.user_name, sess.logon_time
            )?;
        }
        writeln!(writer)?;
        writeln!(
            writer,
            "Abort all? (Use '/logoff idle --older-than {} yes' to confirm)",
            duration_str
        )?;
        writeln!(writer)?;
        return Ok(());
    }

    // Proceed with abort
    writeln!(
        writer,
        "Aborting {} idle session(s) older than {}...",
        idle_sessions.len(),
        format_duration(threshold)
    )?;

    let mut succeeded = 0;
    let mut failed = 0;
    for sess in &idle_sessions {
        match abort::perform_abort(client, sess.session_id, false) {
            Ok(result) => {
                writeln!(writer, "  {}", result.message)?;
                if result.success {
                    succeeded += 1;
                } else {
                    failed += 1;
                }
            }
            Err(e) => {
                writeln!(writer, "  Error aborting session {}: {}", sess.session_id, e)?;
                failed += 1;
            }
        }
    }

    writeln!(writer)?;
    writeln!(writer, "Done: {} succeeded, {} failed.", succeeded, failed)?;
    writeln!(writer)?;
    Ok(())
}

// =========================================================================
// Internal helpers
// =========================================================================

/// Find idle sessions older than the given threshold
fn find_idle_sessions(
    client: &DatabaseClient,
    threshold: Duration,
) -> Result<Vec<IdleSession>> {
    let result = client.execute(IDLE_SESSIONS_SQL)?;
    let threshold_secs = threshold.as_secs() as f64;
    let mut sessions = Vec::new();

    for row in &result.rows {
        if row.len() < 3 {
            continue;
        }

        let session_id = match &row[0] {
            Value::Integer(v) => *v,
            Value::Decimal(v) => *v as i64,
            _ => continue,
        };

        let user_name = match &row[1] {
            Value::String(s) => s.trim().to_string(),
            Value::Null => "[NULL]".to_string(),
            other => other.display().trim().to_string(),
        };

        let logon_time_str = match &row[2] {
            Value::Timestamp(s) | Value::Date(s) | Value::String(s) => s.trim().to_string(),
            Value::Null => continue,
            other => other.display().trim().to_string(),
        };

        // Estimate age from logon_time by parsing the timestamp
        let idle_secs = estimate_session_age_secs(&logon_time_str);

        // Only include sessions older than the threshold
        if idle_secs >= threshold_secs {
            sessions.push(IdleSession {
                session_id,
                user_name,
                logon_time: logon_time_str,
                idle_duration_desc: format_seconds(idle_secs),
            });
        }
    }

    Ok(sessions)
}

/// Find idle sessions and abort them (batch mode)
fn find_and_abort_idle_sessions(
    client: &DatabaseClient,
    threshold: Duration,
) -> Result<LogoffIdleResult> {
    // First find all idle sessions
    let all_idle_result = client.execute(IDLE_SESSIONS_SQL)?;
    let total_idle = all_idle_result.rows.len();

    let eligible = find_idle_sessions(client, threshold)?;
    let eligible_count = eligible.len();

    let mut abort_results = Vec::with_capacity(eligible_count);
    for sess in &eligible {
        abort_results.push(abort::perform_abort(client, sess.session_id, false)?);
    }

    Ok(LogoffIdleResult {
        threshold_desc: format_duration(threshold),
        total_idle,
        eligible,
        abort_results,
    })
}

/// Estimate the age of a session in seconds from its logon timestamp string
///
/// Parses common Teradata timestamp formats. If parsing fails, returns 0.0
/// (session will not be eligible for logoff, which is the safe default).
fn estimate_session_age_secs(logon_time: &str) -> f64 {
    // Teradata timestamps are typically: "2024-01-15 10:30:45" or similar
    // We parse with chrono for robustness
    let now = chrono::Local::now();

    // Try common formats
    let formats = [
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
    ];

    for fmt in &formats {
        if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(logon_time.trim(), fmt) {
            // Assume the timestamp is in the server's local timezone
            if let Some(local) = now.timezone().from_local_datetime(&parsed).single() {
                let age = now.signed_duration_since(local);
                return age.num_seconds().max(0) as f64;
            }
        }
    }

    // Parsing failed; return 0 so the session is not erroneously included
    0.0
}

/// Format a Duration as a human-readable string (e.g., "1h", "30m", "2h 15m")
fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;

    if hours > 0 && minutes > 0 {
        format!("{}h {}m", hours, minutes)
    } else if hours > 0 {
        format!("{}h", hours)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", total_secs)
    }
}

/// Format seconds into a human-readable duration string
fn format_seconds(secs: f64) -> String {
    let total = secs as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", total)
    }
}

// =========================================================================
// Display formatters
// =========================================================================

/// Display logoff-idle results in table format
fn display_table<W: Write>(result: &LogoffIdleResult, writer: &mut W) -> Result<()> {
    if result.eligible.is_empty() {
        writeln!(
            writer,
            "No idle sessions older than {} found ({} idle total).",
            result.threshold_desc, result.total_idle
        )?;
        return Ok(());
    }

    let succeeded = result.abort_results.iter().filter(|r| r.success).count();
    let failed = result.abort_results.len() - succeeded;

    writeln!(
        writer,
        "Idle sessions older than {} ({} of {} idle):",
        result.threshold_desc,
        result.eligible.len(),
        result.total_idle
    )?;
    writeln!(writer)?;

    for (sess, res) in result.eligible.iter().zip(result.abort_results.iter()) {
        let status = if res.success { "OK" } else { "FAIL" };
        writeln!(
            writer,
            "  [{}] Session {} ({}), logged on {}",
            status, sess.session_id, sess.user_name, sess.logon_time
        )?;
    }

    writeln!(writer)?;
    writeln!(writer, "Done: {} succeeded, {} failed.", succeeded, failed)?;
    Ok(())
}

/// Display logoff-idle results in CSV format
fn display_csv<W: Write>(result: &LogoffIdleResult, writer: &mut W) -> Result<()> {
    writeln!(writer, "SessionId,UserName,LogonTime,IdleDuration,Success,Message")?;
    for (sess, res) in result.eligible.iter().zip(result.abort_results.iter()) {
        writeln!(
            writer,
            "{},{},{},{},{},{}",
            sess.session_id,
            escape_csv(&sess.user_name),
            escape_csv(&sess.logon_time),
            escape_csv(&sess.idle_duration_desc),
            res.success,
            escape_csv(&res.message)
        )?;
    }
    Ok(())
}

/// Display logoff-idle results in JSON format
fn display_json<W: Write>(result: &LogoffIdleResult, writer: &mut W) -> Result<()> {
    let sessions: Vec<serde_json::Value> = result
        .eligible
        .iter()
        .zip(result.abort_results.iter())
        .map(|(sess, res)| {
            serde_json::json!({
                "SessionId": sess.session_id,
                "UserName": sess.user_name,
                "LogonTime": sess.logon_time,
                "IdleDuration": sess.idle_duration_desc,
                "Success": res.success,
                "Message": res.message
            })
        })
        .collect();

    let succeeded = result.abort_results.iter().filter(|r| r.success).count();
    let failed = result.abort_results.len() - succeeded;

    let json = serde_json::json!({
        "ok": true,
        "Threshold": result.threshold_desc,
        "TotalIdle": result.total_idle,
        "Eligible": result.eligible.len(),
        "Succeeded": succeeded,
        "Failed": failed,
        "Sessions": sessions
    });

    let output = serde_json::to_string_pretty(&json)?;
    writeln!(writer, "{}", output)?;
    Ok(())
}

/// Display logoff-idle results in Markdown format
fn display_markdown<W: Write>(result: &LogoffIdleResult, writer: &mut W) -> Result<()> {
    if result.eligible.is_empty() {
        writeln!(
            writer,
            "No idle sessions older than {} found.",
            result.threshold_desc
        )?;
        return Ok(());
    }

    let succeeded = result.abort_results.iter().filter(|r| r.success).count();
    let failed = result.abort_results.len() - succeeded;

    writeln!(
        writer,
        "**Logoff Idle**: {} eligible of {} idle (threshold: {}), {} succeeded, {} failed",
        result.eligible.len(),
        result.total_idle,
        result.threshold_desc,
        succeeded,
        failed
    )?;
    writeln!(writer)?;
    writeln!(
        writer,
        "| SessionId | UserName | LogonTime | IdleDuration | Success | Message |"
    )?;
    writeln!(
        writer,
        "| ---: | :--- | :--- | :--- | :--- | :--- |"
    )?;
    for (sess, res) in result.eligible.iter().zip(result.abort_results.iter()) {
        writeln!(
            writer,
            "| {} | {} | {} | {} | {} | {} |",
            sess.session_id,
            sess.user_name.replace('|', "\\|"),
            sess.logon_time.replace('|', "\\|"),
            sess.idle_duration_desc,
            res.success,
            res.message.replace('|', "\\|")
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Duration formatting tests
    // =========================================================================

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h");
        assert_eq!(format_duration(Duration::from_secs(7200)), "2h");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(Duration::from_secs(1800)), "30m");
        assert_eq!(format_duration(Duration::from_secs(300)), "5m");
    }

    #[test]
    fn test_format_duration_mixed() {
        assert_eq!(format_duration(Duration::from_secs(5400)), "1h 30m");
        assert_eq!(format_duration(Duration::from_secs(8100)), "2h 15m");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(Duration::from_secs(45)), "45s");
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
    }

    #[test]
    fn test_format_seconds() {
        assert_eq!(format_seconds(3600.0), "1h 0m");
        assert_eq!(format_seconds(5400.0), "1h 30m");
        assert_eq!(format_seconds(300.0), "5m");
        assert_eq!(format_seconds(45.0), "45s");
    }

    // =========================================================================
    // Session age estimation tests
    // =========================================================================

    #[test]
    fn test_estimate_session_age_invalid() {
        // Invalid timestamp should return 0 (safe default)
        assert_eq!(estimate_session_age_secs("not-a-date"), 0.0);
        assert_eq!(estimate_session_age_secs(""), 0.0);
    }

    #[test]
    fn test_estimate_session_age_recent() {
        // A session that just started should have a small age
        let now = chrono::Local::now();
        let ts = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let age = estimate_session_age_secs(&ts);
        // Should be very close to 0 (within a few seconds)
        assert!(age < 5.0, "Expected age < 5s, got {}", age);
    }

    #[test]
    fn test_estimate_session_age_old() {
        // A session from 2 hours ago
        let two_hours_ago = chrono::Local::now() - chrono::Duration::hours(2);
        let ts = two_hours_ago.format("%Y-%m-%d %H:%M:%S").to_string();
        let age = estimate_session_age_secs(&ts);
        // Should be approximately 7200 seconds (allow some tolerance)
        assert!(age > 7100.0 && age < 7300.0, "Expected ~7200s, got {}", age);
    }

    // =========================================================================
    // Display format tests
    // =========================================================================

    #[test]
    fn test_display_table_empty() {
        let result = LogoffIdleResult {
            threshold_desc: "1h".to_string(),
            total_idle: 5,
            eligible: vec![],
            abort_results: vec![],
        };
        let mut output = Vec::new();
        display_table(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("No idle sessions older than 1h found (5 idle total)."));
    }

    #[test]
    fn test_display_table_with_results() {
        let result = LogoffIdleResult {
            threshold_desc: "2h".to_string(),
            total_idle: 10,
            eligible: vec![
                IdleSession {
                    session_id: 100,
                    user_name: "alice".to_string(),
                    logon_time: "2024-01-15 08:00:00".to_string(),
                    idle_duration_desc: "3h 15m".to_string(),
                },
            ],
            abort_results: vec![
                abort::AbortResult {
                    session_id: 100,
                    query_only: false,
                    success: true,
                    message: "Session 100 aborted.".to_string(),
                },
            ],
        };
        let mut output = Vec::new();
        display_table(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Idle sessions older than 2h (1 of 10 idle):"));
        assert!(s.contains("[OK] Session 100 (alice)"));
        assert!(s.contains("Done: 1 succeeded, 0 failed."));
    }

    #[test]
    fn test_display_csv_format() {
        let result = LogoffIdleResult {
            threshold_desc: "1h".to_string(),
            total_idle: 3,
            eligible: vec![
                IdleSession {
                    session_id: 200,
                    user_name: "bob".to_string(),
                    logon_time: "2024-01-15 09:00:00".to_string(),
                    idle_duration_desc: "2h".to_string(),
                },
            ],
            abort_results: vec![
                abort::AbortResult {
                    session_id: 200,
                    query_only: false,
                    success: true,
                    message: "Session 200 aborted.".to_string(),
                },
            ],
        };
        let mut output = Vec::new();
        display_csv(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("SessionId,UserName,LogonTime,IdleDuration,Success,Message"));
        assert!(s.contains("200,bob,2024-01-15 09:00:00,2h,true,Session 200 aborted."));
    }

    #[test]
    fn test_display_json_format() {
        let result = LogoffIdleResult {
            threshold_desc: "1h".to_string(),
            total_idle: 5,
            eligible: vec![
                IdleSession {
                    session_id: 300,
                    user_name: "carol".to_string(),
                    logon_time: "2024-01-15 07:00:00".to_string(),
                    idle_duration_desc: "4h".to_string(),
                },
            ],
            abort_results: vec![
                abort::AbortResult {
                    session_id: 300,
                    query_only: false,
                    success: true,
                    message: "Session 300 aborted.".to_string(),
                },
            ],
        };
        let mut output = Vec::new();
        display_json(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(json["Threshold"], "1h");
        assert_eq!(json["TotalIdle"], 5);
        assert_eq!(json["Eligible"], 1);
        assert_eq!(json["Succeeded"], 1);
        assert_eq!(json["Failed"], 0);
        assert_eq!(json["Sessions"][0]["SessionId"], 300);
        assert_eq!(json["Sessions"][0]["UserName"], "carol");
    }

    #[test]
    fn test_display_markdown_empty() {
        let result = LogoffIdleResult {
            threshold_desc: "1h".to_string(),
            total_idle: 0,
            eligible: vec![],
            abort_results: vec![],
        };
        let mut output = Vec::new();
        display_markdown(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("No idle sessions older than 1h found."));
    }

    #[test]
    fn test_display_markdown_with_results() {
        let result = LogoffIdleResult {
            threshold_desc: "2h".to_string(),
            total_idle: 8,
            eligible: vec![
                IdleSession {
                    session_id: 400,
                    user_name: "dave".to_string(),
                    logon_time: "2024-01-15 06:00:00".to_string(),
                    idle_duration_desc: "5h".to_string(),
                },
            ],
            abort_results: vec![
                abort::AbortResult {
                    session_id: 400,
                    query_only: false,
                    success: false,
                    message: "Error: Session 400 not found.".to_string(),
                },
            ],
        };
        let mut output = Vec::new();
        display_markdown(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("**Logoff Idle**"));
        assert!(s.contains("| SessionId | UserName | LogonTime | IdleDuration | Success | Message |"));
        assert!(s.contains("| 400 |"));
    }

    // =========================================================================
    // Idle session struct tests
    // =========================================================================

    #[test]
    fn test_idle_session_fields() {
        let sess = IdleSession {
            session_id: 999,
            user_name: "testuser".to_string(),
            logon_time: "2024-01-15 10:00:00".to_string(),
            idle_duration_desc: "1h 30m".to_string(),
        };
        assert_eq!(sess.session_id, 999);
        assert_eq!(sess.user_name, "testuser");
        assert_eq!(sess.idle_duration_desc, "1h 30m");
    }

    #[test]
    fn test_logoff_idle_result_fields() {
        let result = LogoffIdleResult {
            threshold_desc: "1h".to_string(),
            total_idle: 10,
            eligible: vec![],
            abort_results: vec![],
        };
        assert_eq!(result.threshold_desc, "1h");
        assert_eq!(result.total_idle, 10);
        assert!(result.eligible.is_empty());
    }
}
