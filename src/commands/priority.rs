//! Priority change command implementation
//!
//! This module provides functionality to change the priority of Teradata
//! sessions. Priority changes are non-destructive and do not require
//! confirmation prompts.
//!
//! Sprint 49: Initial implementation (Issue #20)

use crate::cli::{OutputFormat, PriorityArgs};
use crate::db::DatabaseClient;
use crate::error::Result;
use super::monitoring_utils::escape_csv;
use std::io::Write;

/// Valid priority levels for Teradata sessions
const VALID_PRIORITIES: &[&str] = &["RUSH", "MEDIUM", "LOW"];

/// Result of a priority change operation
#[derive(Debug, Clone)]
pub struct PriorityResult {
    /// Session ID that was targeted
    pub session_id: i64,
    /// Priority level set
    pub priority: String,
    /// Whether the operation succeeded
    pub success: bool,
    /// Human-readable message
    pub message: String,
}

/// Validate and normalize a priority level string
///
/// Returns the uppercase priority if valid, or an error message.
pub fn validate_priority(level: &str) -> std::result::Result<String, String> {
    let upper = level.to_uppercase();
    if VALID_PRIORITIES.contains(&upper.as_str()) {
        Ok(upper)
    } else {
        Err(format!(
            "Invalid priority '{}'. Valid levels: {}",
            level,
            VALID_PRIORITIES.join(", ")
        ))
    }
}

/// Execute the priority command in batch mode
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &PriorityArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    // Validate priority level
    let priority = match validate_priority(&args.level) {
        Ok(p) => p,
        Err(msg) => {
            writeln!(writer, "Error: {}", msg)?;
            return Ok(());
        }
    };

    let result = perform_priority_change(client, args.session_id, &priority)?;

    match args.format {
        OutputFormat::Table => display_table(&result, writer)?,
        OutputFormat::Csv => display_csv(&result, writer)?,
        OutputFormat::Json => display_json(&result, writer)?,
    }

    Ok(())
}

/// Execute priority change in REPL mode
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    session_id: i64,
    level: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    // Validate priority level
    let priority = match validate_priority(level) {
        Ok(p) => p,
        Err(msg) => {
            writeln!(writer, "Error: {}", msg)?;
            writeln!(writer)?;
            return Ok(());
        }
    };

    match perform_priority_change(client, session_id, &priority) {
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
                writeln!(writer, "Error: Insufficient privileges to change session priority.")?;
                writeln!(writer)?;
                writeln!(writer, "Required: EXECUTE privilege on MonitorSetResource")?;
            } else {
                writeln!(writer, "Error: {}", e)?;
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Perform the actual priority change operation
fn perform_priority_change(
    client: &DatabaseClient,
    session_id: i64,
    priority: &str,
) -> Result<PriorityResult> {
    // Map priority name to Teradata priority code
    // MonitorSetResource takes: SessionNo, ResourceType, NewValue
    // ResourceType 1 = priority, NewValue: R=Rush, M=Medium, L=Low
    let priority_code = match priority {
        "RUSH" => "R",
        "MEDIUM" => "M",
        "LOW" => "L",
        _ => unreachable!("Priority already validated"),
    };

    let sql = format!(
        "SELECT * FROM TABLE (MonitorSetResource({}, 1, '{}')) AS t1",
        session_id, priority_code
    );

    match client.execute(&sql) {
        Ok(_) => {
            let message = format!(
                "Session {} priority changed to {}.",
                session_id, priority
            );
            Ok(PriorityResult {
                session_id,
                priority: priority.to_string(),
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
            Ok(PriorityResult {
                session_id,
                priority: priority.to_string(),
                success: false,
                message,
            })
        }
    }
}

/// Display priority result in table format
fn display_table<W: Write>(result: &PriorityResult, writer: &mut W) -> Result<()> {
    writeln!(writer, "{}", result.message)?;
    Ok(())
}

/// Display priority result in CSV format
fn display_csv<W: Write>(result: &PriorityResult, writer: &mut W) -> Result<()> {
    writeln!(writer, "SessionId,Priority,Success,Message")?;
    writeln!(
        writer,
        "{},{},{},{}",
        result.session_id,
        result.priority,
        result.success,
        escape_csv(&result.message)
    )?;
    Ok(())
}

/// Display priority result in JSON format
fn display_json<W: Write>(result: &PriorityResult, writer: &mut W) -> Result<()> {
    let json = serde_json::json!({
        "SessionId": result.session_id,
        "Priority": result.priority,
        "Success": result.success,
        "Message": result.message
    });
    let output = serde_json::to_string_pretty(&json)?;
    writeln!(writer, "{}", output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_priority_rush() {
        assert_eq!(validate_priority("rush"), Ok("RUSH".to_string()));
        assert_eq!(validate_priority("RUSH"), Ok("RUSH".to_string()));
        assert_eq!(validate_priority("Rush"), Ok("RUSH".to_string()));
    }

    #[test]
    fn test_validate_priority_medium() {
        assert_eq!(validate_priority("medium"), Ok("MEDIUM".to_string()));
        assert_eq!(validate_priority("MEDIUM"), Ok("MEDIUM".to_string()));
    }

    #[test]
    fn test_validate_priority_low() {
        assert_eq!(validate_priority("low"), Ok("LOW".to_string()));
        assert_eq!(validate_priority("LOW"), Ok("LOW".to_string()));
    }

    #[test]
    fn test_validate_priority_invalid() {
        let result = validate_priority("high");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("Invalid priority 'high'"));
        assert!(msg.contains("RUSH, MEDIUM, LOW"));
    }

    #[test]
    fn test_validate_priority_empty() {
        let result = validate_priority("");
        assert!(result.is_err());
    }

    #[test]
    fn test_priority_result_success() {
        let result = PriorityResult {
            session_id: 1234,
            priority: "RUSH".to_string(),
            success: true,
            message: "Session 1234 priority changed to RUSH.".to_string(),
        };
        assert!(result.success);
        assert_eq!(result.priority, "RUSH");
    }

    #[test]
    fn test_display_table_priority() {
        let result = PriorityResult {
            session_id: 1234,
            priority: "RUSH".to_string(),
            success: true,
            message: "Session 1234 priority changed to RUSH.".to_string(),
        };
        let mut output = Vec::new();
        display_table(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Session 1234 priority changed to RUSH."));
    }

    #[test]
    fn test_display_csv_priority() {
        let result = PriorityResult {
            session_id: 1234,
            priority: "MEDIUM".to_string(),
            success: true,
            message: "Session 1234 priority changed to MEDIUM.".to_string(),
        };
        let mut output = Vec::new();
        display_csv(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("SessionId,Priority,Success,Message"));
        assert!(s.contains("1234,MEDIUM,true,"));
    }

    #[test]
    fn test_display_json_priority() {
        let result = PriorityResult {
            session_id: 1234,
            priority: "LOW".to_string(),
            success: true,
            message: "Session 1234 priority changed to LOW.".to_string(),
        };
        let mut output = Vec::new();
        display_json(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(json["SessionId"], 1234);
        assert_eq!(json["Priority"], "LOW");
        assert_eq!(json["Success"], true);
    }

    #[test]
    fn test_display_json_failure() {
        let result = PriorityResult {
            session_id: 9999,
            priority: "RUSH".to_string(),
            success: false,
            message: "Error: Session 9999 not found or already terminated.".to_string(),
        };
        let mut output = Vec::new();
        display_json(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(json["Success"], false);
    }

    #[test]
    fn test_display_csv_message_with_special_chars() {
        let result = PriorityResult {
            session_id: 1234,
            priority: "RUSH".to_string(),
            success: false,
            message: "Error: Session 1234, permission denied".to_string(),
        };
        let mut output = Vec::new();
        display_csv(&result, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("\"Error: Session 1234, permission denied\""));
    }
}
