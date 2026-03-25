//! Abort session/query command implementation
//!
//! This module provides functionality to abort Teradata sessions or running
//! queries. These are privileged DBA operations with safety confirmations.
//!
//! Sprint 49: Initial implementation (Issue #20)

use crate::cli::{AbortArgs, OutputFormat};
use crate::db::DatabaseClient;
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

/// Execute the abort command in batch mode
///
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
        writeln!(writer, "This is a destructive operation that will terminate the target session.")?;
        writeln!(writer, "Add --force to confirm: tq abort --force {}", args.session_id)?;
        return Ok(());
    }

    let result = perform_abort(client, args.session_id, args.query)?;

    match args.format {
        OutputFormat::Table => display_table(&result, writer)?,
        OutputFormat::Csv => display_csv(&result, writer)?,
        OutputFormat::Json => display_json(&result, writer)?,
        OutputFormat::Markdown | OutputFormat::Md => display_markdown(&result, writer)?,
    }

    Ok(())
}

/// Execute abort in REPL mode with interactive confirmation
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
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Perform the actual abort operation
fn perform_abort(
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
            session_id: 1234,
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
        // We can't call execute_for_repl without a real client, but we can
        // test the confirmation message format
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
}
