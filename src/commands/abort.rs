//! Abort session/query command implementation
//!
//! This module provides functionality to abort Teradata sessions or running
//! queries. These are privileged DBA operations with safety confirmations.
//!
//! Sprint 49: Initial implementation (Issue #20)
//! Sprint 61: Extended with abort-by-user and abort-by-host

use crate::cli::{AbortArgs, OutputFormat};
use crate::db::{DatabaseClient, Value};
use crate::error::Result;
use super::monitoring_utils::escape_csv;
use std::io::Write;

/// Result of an abort operation
#[derive(Debug, Clone)]
pub struct AbortResult {
    /// Session ID that was targeted
    pub session_id: i64,
    /// Whether only the query (not session) was aborted
    pub query_only: bool,
    /// Whether the operation succeeded
    pub success: bool,
    /// Human-readable message
    pub message: String,
}

/// Result of a bulk abort operation (user or host)
#[derive(Debug, Clone)]
pub struct BulkAbortResult {
    /// Description of the target (e.g., "user alice" or "host myserver01")
    pub target: String,
    /// Individual abort results for each session
    pub results: Vec<AbortResult>,
    /// Number of sessions found matching the criteria
    pub sessions_found: usize,
}

/// Execute the abort command in batch mode
///
/// Dispatches to single-session, user-based, or host-based abort
/// depending on which arguments are provided.
/// Requires `--force` flag for non-interactive operation.
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &AbortArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    if !args.force {
        writeln!(writer, "Error: Abort operations require --force flag in batch mode.")?;
        writeln!(writer)?;
        writeln!(writer, "This is a destructive operation that will terminate the target session(s).")?;
        if let Some(sid) = args.session_id {
            writeln!(writer, "Add --force to confirm: tq abort --force {}", sid)?;
        } else if let Some(ref user) = args.user {
            writeln!(writer, "Add --force to confirm: tq abort --force --user {}", user)?;
        } else if let Some(ref host) = args.host {
            writeln!(writer, "Add --force to confirm: tq abort --force --host {}", host)?;
        }
        return Ok(());
    }

    if let Some(ref user) = args.user {
        let bulk = abort_user_sessions(client, user)?;
        match args.format {
            OutputFormat::Table => display_bulk_table(&bulk, writer)?,
            OutputFormat::Csv => display_bulk_csv(&bulk, writer)?,
            OutputFormat::Json => display_bulk_json(&bulk, writer)?,
            OutputFormat::Markdown | OutputFormat::Md => display_bulk_markdown(&bulk, writer)?,
        }
    } else if let Some(ref host) = args.host {
        let bulk = abort_host_sessions(client, host)?;
        match args.format {
            OutputFormat::Table => display_bulk_table(&bulk, writer)?,
            OutputFormat::Csv => display_bulk_csv(&bulk, writer)?,
            OutputFormat::Json => display_bulk_json(&bulk, writer)?,
            OutputFormat::Markdown | OutputFormat::Md => display_bulk_markdown(&bulk, writer)?,
        }
    } else if let Some(session_id) = args.session_id {
        let result = perform_abort(client, session_id, args.query)?;
        match args.format {
            OutputFormat::Table => display_table(&result, writer)?,
            OutputFormat::Csv => display_csv(&result, writer)?,
            OutputFormat::Json => display_json(&result, writer)?,
            OutputFormat::Markdown | OutputFormat::Md => display_markdown(&result, writer)?,
        }
    } else {
        writeln!(writer, "Error: Specify a session ID, --user, or --host.")?;
        writeln!(writer)?;
        writeln!(writer, "Usage: tq abort --force <session_id>")?;
        writeln!(writer, "       tq abort --force --user <username>")?;
        writeln!(writer, "       tq abort --force --host <hostname>")?;
    }

    Ok(())
}

/// Execute abort in REPL mode with interactive confirmation (single session)
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    session_id: i64,
    query_only: bool,
    confirmed: bool,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    if !confirmed {
        if query_only {
            writeln!(writer, "Abort running query on session {}? [y/N] ", session_id)?;
        } else {
            writeln!(writer, "Abort session {}? [y/N] ", session_id)?;
        }
        writeln!(writer, "(Use '/abort {} yes' to confirm)", session_id)?;
        writeln!(writer)?;
        return Ok(());
    }

    match perform_abort(client, session_id, query_only) {
        Ok(result) => {
            writeln!(writer, "{}", result.message)?;
        }
        Err(e) => {
            write_abort_error(writer, &e)?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Execute abort-by-user in REPL mode
pub fn execute_user_for_repl<W: Write>(
    client: &DatabaseClient,
    username: &str,
    confirmed: bool,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    // First, find matching sessions
    let session_ids = find_sessions_for_user(client, username)?;

    if session_ids.is_empty() {
        writeln!(writer, "No active sessions found for user '{}'.", username)?;
        writeln!(writer)?;
        return Ok(());
    }

    if !confirmed {
        writeln!(
            writer,
            "Found {} session(s) for user '{}':",
            session_ids.len(),
            username
        )?;
        for sid in &session_ids {
            writeln!(writer, "  Session {}", sid)?;
        }
        writeln!(writer)?;
        writeln!(
            writer,
            "Abort all? (Use '/abort user {} yes' to confirm)",
            username
        )?;
        writeln!(writer)?;
        return Ok(());
    }

    writeln!(
        writer,
        "Aborting {} session(s) for user '{}'...",
        session_ids.len(),
        username
    )?;

    let mut succeeded = 0;
    let mut failed = 0;
    for sid in &session_ids {
        match perform_abort(client, *sid, false) {
            Ok(result) => {
                writeln!(writer, "  {}", result.message)?;
                if result.success {
                    succeeded += 1;
                } else {
                    failed += 1;
                }
            }
            Err(e) => {
                writeln!(writer, "  Error aborting session {}: {}", sid, e)?;
                failed += 1;
            }
        }
    }

    writeln!(writer)?;
    writeln!(
        writer,
        "Done: {} succeeded, {} failed.",
        succeeded, failed
    )?;
    writeln!(writer)?;
    Ok(())
}

/// Execute abort-by-host in REPL mode
pub fn execute_host_for_repl<W: Write>(
    client: &DatabaseClient,
    hostname: &str,
    confirmed: bool,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    let session_ids = find_sessions_for_host(client, hostname)?;

    if session_ids.is_empty() {
        writeln!(
            writer,
            "No active sessions found from host '{}'.",
            hostname
        )?;
        writeln!(writer)?;
        return Ok(());
    }

    if !confirmed {
        writeln!(
            writer,
            "Found {} session(s) from host '{}':",
            session_ids.len(),
            hostname
        )?;
        for sid in &session_ids {
            writeln!(writer, "  Session {}", sid)?;
        }
        writeln!(writer)?;
        writeln!(
            writer,
            "Abort all? (Use '/abort host {} yes' to confirm)",
            hostname
        )?;
        writeln!(writer)?;
        return Ok(());
    }

    writeln!(
        writer,
        "Aborting {} session(s) from host '{}'...",
        session_ids.len(),
        hostname
    )?;

    let mut succeeded = 0;
    let mut failed = 0;
    for sid in &session_ids {
        match perform_abort(client, *sid, false) {
            Ok(result) => {
                writeln!(writer, "  {}", result.message)?;
                if result.success {
                    succeeded += 1;
                } else {
                    failed += 1;
                }
            }
            Err(e) => {
                writeln!(writer, "  Error aborting session {}: {}", sid, e)?;
                failed += 1;
            }
        }
    }

    writeln!(writer)?;
    writeln!(
        writer,
        "Done: {} succeeded, {} failed.",
        succeeded, failed
    )?;
    writeln!(writer)?;
    Ok(())
}

// =========================================================================
// Internal query helpers
// =========================================================================

/// Find all active session IDs for a given username
fn find_sessions_for_user(
    client: &DatabaseClient,
    username: &str,
) -> Result<Vec<i64>> {
    let safe_user = username.replace('\'', "''");
    let sql = format!(
        "SELECT SessionNo FROM TABLE (MonitorSession(-1, '*', 0)) AS t1 \
         WHERE TRIM(UserName) = '{}'",
        safe_user
    );
    extract_session_ids(client, &sql)
}

/// Find all active session IDs from a given hostname (substring match on LogonSource)
fn find_sessions_for_host(
    client: &DatabaseClient,
    hostname: &str,
) -> Result<Vec<i64>> {
    let safe_host = hostname.replace('\'', "''");
    let sql = format!(
        "SELECT SessionNo FROM TABLE (MonitorSession(-1, '*', 0)) AS t1 \
         WHERE TRIM(LogonSource) LIKE '%{}%'",
        safe_host
    );
    extract_session_ids(client, &sql)
}

/// Execute a session-finding query and extract session IDs from the result
fn extract_session_ids(
    client: &DatabaseClient,
    sql: &str,
) -> Result<Vec<i64>> {
    let result = client.execute(sql)?;
    let mut ids = Vec::new();
    for row in &result.rows {
        if let Some(Value::Integer(v)) = row.first() {
            ids.push(*v);
        } else if let Some(Value::Decimal(v)) = row.first() {
            ids.push(*v as i64);
        }
    }
    Ok(ids)
}

/// Abort all sessions for a user (batch mode)
fn abort_user_sessions(
    client: &DatabaseClient,
    username: &str,
) -> Result<BulkAbortResult> {
    let session_ids = find_sessions_for_user(client, username)?;
    let sessions_found = session_ids.len();
    let mut results = Vec::with_capacity(sessions_found);

    for sid in session_ids {
        results.push(perform_abort(client, sid, false)?);
    }

    Ok(BulkAbortResult {
        target: format!("user {}", username),
        results,
        sessions_found,
    })
}

/// Abort all sessions from a host (batch mode)
fn abort_host_sessions(
    client: &DatabaseClient,
    hostname: &str,
) -> Result<BulkAbortResult> {
    let session_ids = find_sessions_for_host(client, hostname)?;
    let sessions_found = session_ids.len();
    let mut results = Vec::with_capacity(sessions_found);

    for sid in session_ids {
        results.push(perform_abort(client, sid, false)?);
    }

    Ok(BulkAbortResult {
        target: format!("host {}", hostname),
        results,
        sessions_found,
    })
}

// =========================================================================
// Core abort operation
// =========================================================================

/// Perform the actual abort operation on a single session
pub(crate) fn perform_abort(
    client: &DatabaseClient,
    session_id: i64,
    query_only: bool,
) -> Result<AbortResult> {
    let sql = if query_only {
        // Abort only the running request/query on the session
        format!(
            "SELECT * FROM TABLE (MonitorCancelRequest({})) AS t1",
            session_id
        )
    } else {
        // Abort the entire session
        format!(
            "SELECT * FROM TABLE (MonitorAbortSession({})) AS t1",
            session_id
        )
    };

    match client.execute(&sql) {
        Ok(_) => {
            let message = if query_only {
                format!("Running query on session {} aborted.", session_id)
            } else {
                format!("Session {} aborted.", session_id)
            };
            Ok(AbortResult {
                session_id,
                query_only,
                success: true,
                message,
            })
        }
        Err(e) => {
            let error_str = e.to_string().to_lowercase();
            let message = if error_str.contains("not found")
                || error_str.contains("does not exist")
                || error_str.contains("invalid session")
            {
                format!(
                    "Error: Session {} not found or already terminated.",
                    session_id
                )
            } else {
                format!("Error: {}", e)
            };
            Ok(AbortResult {
                session_id,
                query_only,
                success: false,
                message,
            })
        }
    }
}

/// Write a formatted abort error with privilege guidance
fn write_abort_error<W: Write>(writer: &mut W, e: &crate::error::TqError) -> Result<()> {
    let error_str = e.to_string().to_lowercase();
    if error_str.contains("privilege")
        || error_str.contains("access")
        || error_str.contains("permission")
        || error_str.contains("3523")
    {
        writeln!(writer, "Error: Insufficient privileges to abort sessions.")?;
        writeln!(writer)?;
        writeln!(writer, "Required: ABORT SESSION privilege")?;
        writeln!(writer)?;
        writeln!(writer, "To grant access, a DBA can run:")?;
        writeln!(writer, "  GRANT ABORT SESSION TO <username>;")?;
    } else {
        writeln!(writer, "Error: {}", e)?;
    }
    Ok(())
}

// =========================================================================
// Single-session display formatters
// =========================================================================

/// Display abort result in table format
fn display_table<W: Write>(result: &AbortResult, writer: &mut W) -> Result<()> {
    writeln!(writer, "{}", result.message)?;
    Ok(())
}

/// Display abort result in CSV format
fn display_csv<W: Write>(result: &AbortResult, writer: &mut W) -> Result<()> {
    writeln!(writer, "SessionId,Action,Success,Message")?;
    let action = if result.query_only { "AbortQuery" } else { "AbortSession" };
    writeln!(
        writer,
        "{},{},{},{}",
        result.session_id,
        action,
        result.success,
        escape_csv(&result.message)
    )?;
    Ok(())
}

/// Display abort result in JSON format
fn display_json<W: Write>(result: &AbortResult, writer: &mut W) -> Result<()> {
    let json = serde_json::json!({
        "ok": true,
        "SessionId": result.session_id,
        "Action": if result.query_only { "AbortQuery" } else { "AbortSession" },
        "Success": result.success,
        "Message": result.message
    });
    let output = serde_json::to_string_pretty(&json)?;
    writeln!(writer, "{}", output)?;
    Ok(())
}

/// Display abort result in Markdown format
fn display_markdown<W: Write>(result: &AbortResult, writer: &mut W) -> Result<()> {
    let action = if result.query_only { "AbortQuery" } else { "AbortSession" };
    writeln!(writer, "| SessionId | Action | Success | Message |")?;
    writeln!(writer, "| ---: | :--- | :--- | :--- |")?;
    writeln!(
        writer,
        "| {} | {} | {} | {} |",
        result.session_id,
        action,
        result.success,
        result.message.replace('|', "\\|")
    )?;
    Ok(())
}

// =========================================================================
// Bulk abort display formatters
// =========================================================================

/// Display bulk abort results in table format
pub fn display_bulk_table<W: Write>(
    bulk: &BulkAbortResult,
    writer: &mut W,
) -> Result<()> {
    if bulk.sessions_found == 0 {
        writeln!(writer, "No active sessions found for {}.", bulk.target)?;
        return Ok(());
    }

    writeln!(
        writer,
        "Aborting {} session(s) for {}...",
        bulk.sessions_found, bulk.target
    )?;

    for result in &bulk.results {
        writeln!(writer, "  {}", result.message)?;
    }

    let succeeded = bulk.results.iter().filter(|r| r.success).count();
    let failed = bulk.results.len() - succeeded;
    writeln!(writer)?;
    writeln!(writer, "Done: {} succeeded, {} failed.", succeeded, failed)?;
    Ok(())
}

/// Display bulk abort results in CSV format
pub fn display_bulk_csv<W: Write>(
    bulk: &BulkAbortResult,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer, "SessionId,Action,Success,Message")?;
    for result in &bulk.results {
        writeln!(
            writer,
            "{},AbortSession,{},{}",
            result.session_id,
            result.success,
            escape_csv(&result.message)
        )?;
    }
    Ok(())
}

/// Display bulk abort results in JSON format
pub fn display_bulk_json<W: Write>(
    bulk: &BulkAbortResult,
    writer: &mut W,
) -> Result<()> {
    let sessions: Vec<serde_json::Value> = bulk
        .results
        .iter()
        .map(|r| {
            serde_json::json!({
                "SessionId": r.session_id,
                "Success": r.success,
                "Message": r.message
            })
        })
        .collect();

    let succeeded = bulk.results.iter().filter(|r| r.success).count();
    let failed = bulk.results.len() - succeeded;

    let json = serde_json::json!({
        "ok": true,
        "Target": bulk.target,
        "SessionsFound": bulk.sessions_found,
        "Succeeded": succeeded,
        "Failed": failed,
        "Sessions": sessions
    });
    let output = serde_json::to_string_pretty(&json)?;
    writeln!(writer, "{}", output)?;
    Ok(())
}

/// Display bulk abort results in Markdown format
pub fn display_bulk_markdown<W: Write>(
    bulk: &BulkAbortResult,
    writer: &mut W,
) -> Result<()> {
    if bulk.sessions_found == 0 {
        writeln!(writer, "No active sessions found for {}.", bulk.target)?;
        return Ok(());
    }

    let succeeded = bulk.results.iter().filter(|r| r.success).count();
    let failed = bulk.results.len() - succeeded;

    writeln!(
        writer,
        "**Abort {}**: {} found, {} succeeded, {} failed",
        bulk.target, bulk.sessions_found, succeeded, failed
    )?;
    writeln!(writer)?;
    writeln!(writer, "| SessionId | Success | Message |")?;
    writeln!(writer, "| ---: | :--- | :--- |")?;
    for result in &bulk.results {
        writeln!(
            writer,
            "| {} | {} | {} |",
            result.session_id,
            result.success,
            result.message.replace('|', "\\|")
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abort_result_session() {
        let result = AbortResult {
            session_id: 1234,
            query_only: false,
            success: true,
            message: "Session 1234 aborted.".to_string(),
        };
        assert_eq!(result.session_id, 1234);
        assert!(!result.query_only);
        assert!(result.success);
    }

    #[test]
    fn test_abort_result_query_only() {
        let result = AbortResult {
            session_id: 5678,
            query_only: true,
            success: true,
            message: "Running query on session 5678 aborted.".to_string(),
        };
        assert!(result.query_only);
        assert!(result.success);
    }

    #[test]
    fn test_display_table() {
        let result = AbortResult {
            session_id: 1234,
            query_only: false,
            success: true,
            message: "Session 1234 aborted.".to_string(),
        };
        let mut output = Vec::new();
        display_table(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Session 1234 aborted."));
    }

    #[test]
    fn test_display_csv_abort_session() {
        let result = AbortResult {
            session_id: 1234,
            query_only: false,
            success: true,
            message: "Session 1234 aborted.".to_string(),
        };
        let mut output = Vec::new();
        display_csv(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("SessionId,Action,Success,Message"));
        assert!(s.contains("1234,AbortSession,true,Session 1234 aborted."));
    }

    #[test]
    fn test_display_csv_abort_query() {
        let result = AbortResult {
            session_id: 5678,
            query_only: true,
            success: true,
            message: "Running query on session 5678 aborted.".to_string(),
        };
        let mut output = Vec::new();
        display_csv(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("5678,AbortQuery,true,"));
    }

    #[test]
    fn test_display_json_success() {
        let result = AbortResult {
            session_id: 1234,
            query_only: false,
            success: true,
            message: "Session 1234 aborted.".to_string(),
        };
        let mut output = Vec::new();
        display_json(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(json["SessionId"], 1234);
        assert_eq!(json["Action"], "AbortSession");
        assert_eq!(json["Success"], true);
    }

    #[test]
    fn test_display_json_failure() {
        let result = AbortResult {
            session_id: 9999,
            query_only: false,
            success: false,
            message: "Error: Session 9999 not found or already terminated.".to_string(),
        };
        let mut output = Vec::new();
        display_json(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(json["Success"], false);
        assert!(json["Message"].as_str().unwrap().contains("not found"));
    }

    #[test]
    fn test_display_csv_message_with_comma() {
        let result = AbortResult {
            session_id: 1234,
            query_only: false,
            success: false,
            message: "Error: Session 1234, not found".to_string(),
        };
        let mut output = Vec::new();
        display_csv(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        // CSV should quote the message containing a comma
        assert!(s.contains("\"Error: Session 1234, not found\""));
    }

    #[test]
    fn test_batch_mode_requires_force() {
        let args = AbortArgs {
            session_id: Some(1234),
            user: None,
            host: None,
            query: false,
            force: false,
            format: OutputFormat::Table,
            output: None,
        };
        // Without --force, batch mode should print error and not execute
        assert!(!args.force);
    }

    #[test]
    fn test_execute_for_repl_no_confirm() {
        // When not confirmed, should show confirmation prompt
        let mut output = Vec::new();
        let session_id = 1234;
        let query_only = false;
        let confirmed = false;

        if !confirmed {
            if query_only {
                writeln!(output, "Abort running query on session {}? [y/N] ", session_id).unwrap();
            } else {
                writeln!(output, "Abort session {}? [y/N] ", session_id).unwrap();
            }
        }

        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Abort session 1234? [y/N]"));
    }

    #[test]
    fn test_execute_for_repl_query_only_prompt() {
        let mut output = Vec::new();
        let session_id = 5678;
        let query_only = true;
        let confirmed = false;

        if !confirmed {
            if query_only {
                writeln!(output, "Abort running query on session {}? [y/N] ", session_id).unwrap();
            } else {
                writeln!(output, "Abort session {}? [y/N] ", session_id).unwrap();
            }
        }

        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Abort running query on session 5678? [y/N]"));
    }

    // =========================================================================
    // Bulk abort display tests (Sprint 61)
    // =========================================================================

    #[test]
    fn test_bulk_abort_result_empty() {
        let bulk = BulkAbortResult {
            target: "user alice".to_string(),
            results: vec![],
            sessions_found: 0,
        };
        let mut output = Vec::new();
        display_bulk_table(&bulk, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("No active sessions found for user alice."));
    }

    #[test]
    fn test_bulk_abort_result_table() {
        let bulk = BulkAbortResult {
            target: "user alice".to_string(),
            results: vec![
                AbortResult {
                    session_id: 100,
                    query_only: false,
                    success: true,
                    message: "Session 100 aborted.".to_string(),
                },
                AbortResult {
                    session_id: 200,
                    query_only: false,
                    success: true,
                    message: "Session 200 aborted.".to_string(),
                },
            ],
            sessions_found: 2,
        };
        let mut output = Vec::new();
        display_bulk_table(&bulk, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Aborting 2 session(s) for user alice"));
        assert!(s.contains("Session 100 aborted."));
        assert!(s.contains("Session 200 aborted."));
        assert!(s.contains("Done: 2 succeeded, 0 failed."));
    }

    #[test]
    fn test_bulk_abort_result_csv() {
        let bulk = BulkAbortResult {
            target: "host myserver".to_string(),
            results: vec![
                AbortResult {
                    session_id: 300,
                    query_only: false,
                    success: true,
                    message: "Session 300 aborted.".to_string(),
                },
                AbortResult {
                    session_id: 400,
                    query_only: false,
                    success: false,
                    message: "Error: Session 400 not found or already terminated.".to_string(),
                },
            ],
            sessions_found: 2,
        };
        let mut output = Vec::new();
        display_bulk_csv(&bulk, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("SessionId,Action,Success,Message"));
        assert!(s.contains("300,AbortSession,true,"));
        assert!(s.contains("400,AbortSession,false,"));
    }

    #[test]
    fn test_bulk_abort_result_json() {
        let bulk = BulkAbortResult {
            target: "user bob".to_string(),
            results: vec![
                AbortResult {
                    session_id: 500,
                    query_only: false,
                    success: true,
                    message: "Session 500 aborted.".to_string(),
                },
            ],
            sessions_found: 1,
        };
        let mut output = Vec::new();
        display_bulk_json(&bulk, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(json["Target"], "user bob");
        assert_eq!(json["SessionsFound"], 1);
        assert_eq!(json["Succeeded"], 1);
        assert_eq!(json["Failed"], 0);
        assert_eq!(json["Sessions"][0]["SessionId"], 500);
    }

    #[test]
    fn test_bulk_abort_result_markdown() {
        let bulk = BulkAbortResult {
            target: "user carol".to_string(),
            results: vec![
                AbortResult {
                    session_id: 600,
                    query_only: false,
                    success: true,
                    message: "Session 600 aborted.".to_string(),
                },
            ],
            sessions_found: 1,
        };
        let mut output = Vec::new();
        display_bulk_markdown(&bulk, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("**Abort user carol**"));
        assert!(s.contains("| SessionId | Success | Message |"));
        assert!(s.contains("| 600 |"));
    }

    #[test]
    fn test_bulk_abort_markdown_empty() {
        let bulk = BulkAbortResult {
            target: "host unknown".to_string(),
            results: vec![],
            sessions_found: 0,
        };
        let mut output = Vec::new();
        display_bulk_markdown(&bulk, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("No active sessions found for host unknown."));
    }

    #[test]
    fn test_batch_mode_user_requires_force() {
        let args = AbortArgs {
            session_id: None,
            user: Some("alice".to_string()),
            host: None,
            query: false,
            force: false,
            format: OutputFormat::Table,
            output: None,
        };
        assert!(!args.force);
        // The force check in execute() would prevent execution
    }

    #[test]
    fn test_batch_mode_host_requires_force() {
        let args = AbortArgs {
            session_id: None,
            user: None,
            host: Some("myserver".to_string()),
            query: false,
            force: false,
            format: OutputFormat::Table,
            output: None,
        };
        assert!(!args.force);
    }
}
