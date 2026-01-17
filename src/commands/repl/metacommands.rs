//! Metacommand handling for the REPL
//!
//! Metacommands start with '/' or '\' and provide non-SQL functionality
//! like session management, help, and REPL control.

use super::state::ReplState;
use crate::db::DatabaseClient;
use crate::error::Result;
use std::io::Write;

/// Handle a metacommand
///
/// Returns Ok(true) to continue the REPL, Ok(false) to exit.
pub fn handle_metacommand<W: Write>(
    input: &str,
    state: &ReplState,
    _client: &DatabaseClient,
    writer: &mut W,
) -> Result<bool> {
    // Normalize: remove leading / or \ and lowercase
    let normalized = input
        .trim()
        .trim_start_matches('/')
        .trim_start_matches('\\')
        .to_lowercase();

    // Split into command and arguments
    let mut parts = normalized.split_whitespace();
    let command = parts.next().unwrap_or("");
    let _args: Vec<&str> = parts.collect();

    match command {
        // Exit commands
        "quit" | "q" | "exit" => {
            return Ok(false);
        }

        // Help command
        "help" | "?" => {
            print_help(writer)?;
        }

        // Session info command
        "session" => {
            print_session_info(state, writer)?;
        }

        // Unknown command
        _ => {
            writeln!(writer, "Unknown command: /{}", command)?;
            writeln!(writer, "Type /help for available commands.")?;
        }
    }

    Ok(true)
}

/// Print help text
fn print_help<W: Write>(writer: &mut W) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "tq REPL Commands:")?;
    writeln!(writer, "  /help, /?      Show this help message")?;
    writeln!(writer, "  /quit, /q      Exit the REPL")?;
    writeln!(writer, "  /session       Show current session information")?;
    writeln!(writer)?;
    writeln!(writer, "SQL Execution:")?;
    writeln!(writer, "  Enter SQL statements ending with semicolon (;)")?;
    writeln!(writer, "  Multi-line statements are supported")?;
    writeln!(writer)?;
    writeln!(writer, "Keyboard Shortcuts:")?;
    writeln!(writer, "  Up/Down        Navigate command history")?;
    writeln!(writer, "  Ctrl-C         Cancel current input")?;
    writeln!(writer, "  Ctrl-D         Exit REPL (when input is empty)")?;
    writeln!(writer, "  Ctrl-R         Search command history")?;
    writeln!(writer)?;

    Ok(())
}

/// Print session information
fn print_session_info<W: Write>(state: &ReplState, writer: &mut W) -> Result<()> {
    let config = state.connection_info();
    let duration = state.session_duration();

    // Format duration nicely
    let duration_str = format_duration(duration);

    writeln!(writer)?;
    writeln!(writer, "Session Information:")?;
    writeln!(writer, "  Host:            {}:{}", config.host, config.port)?;
    writeln!(writer, "  Database:        {}", config.database)?;
    writeln!(writer, "  User:            {}", config.user)?;
    writeln!(writer, "  Logon Mechanism: {}", config.logmech)?;
    writeln!(writer, "  Session Start:   {}", state.session_start_time())?;
    writeln!(writer, "  Duration:        {}", duration_str)?;
    writeln!(writer, "  Queries Run:     {}", state.queries_executed())?;
    writeln!(writer, "  Rows Returned:   {}", state.total_rows())?;
    writeln!(writer)?;

    Ok(())
}

/// Format a duration nicely (e.g., "5m 23s" or "1h 30m 15s")
fn format_duration(duration: std::time::Duration) -> String {
    let total_secs = duration.as_secs();

    if total_secs < 60 {
        format!("{}s", total_secs)
    } else if total_secs < 3600 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        if secs == 0 {
            format!("{}m", mins)
        } else {
            format!("{}m {}s", mins, secs)
        }
    } else {
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;
        if secs == 0 && mins == 0 {
            format!("{}h", hours)
        } else if secs == 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h {}m {}s", hours, mins, secs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::LogonMechanism;
    use crate::db::ConnectionConfig;
    use std::time::Duration;

    fn create_test_config() -> ConnectionConfig {
        ConnectionConfig {
            host: "testhost".to_string(),
            port: 1025,
            database: "testdb".to_string(),
            user: "testuser".to_string(),
            password: None,
            logmech: LogonMechanism::Td2,
            timeout: Duration::from_secs(30),
        }
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(Duration::from_secs(60)), "1m");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(300)), "5m");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h");
        assert_eq!(format_duration(Duration::from_secs(3660)), "1h 1m");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    }

    #[test]
    fn test_help_output() {
        let mut output = Vec::new();
        print_help(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("/help"));
        assert!(output_str.contains("/quit"));
        assert!(output_str.contains("/session"));
        assert!(output_str.contains("Ctrl-C"));
    }

    #[test]
    fn test_session_info_output() {
        let config = create_test_config();
        let state = ReplState::new(config);

        let mut output = Vec::new();
        print_session_info(&state, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("testhost:1025"));
        assert!(output_str.contains("testdb"));
        assert!(output_str.contains("testuser"));
    }
}
