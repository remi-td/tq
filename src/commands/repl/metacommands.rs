//! Metacommand handling for the REPL
//!
//! Metacommands start with '/' or '\' and provide non-SQL functionality
//! like session management, help, and REPL control.
//!
//! Sprint 4 additions:
//! - /describe <table> - Show table structure (columns, types, nullable)
//! - /ping - Test connection within REPL with latency display
//!
//! Sprint 7 additions:
//! - /logon <connection_string> - Switch to a different database connection
//!
//! Sprint 22 additions:
//! - /list databases - List all accessible databases
//! - /list tables [pattern] - List tables with optional glob pattern
//! - /list views - List views in current database
//!
//! Sprint 26 additions:
//! - /sessions - List active database sessions with performance metrics
//!
//! Sprint 34 refactoring:
//! - Use shared sql::escape_sql_string and sql::quote_qualified_name utilities

use super::executor::execute_sql_with_state;
use super::metadata_completer::CompletionState;
use super::state::ReplState;
use crate::cli::LogonMechanism;
use crate::commands::format_helpers::{format_nullable, truncate_str};
use crate::db::{ConnectionConfig, DatabaseClient};
use crate::error::Result;
use crate::sql::{escape_sql_string, quote_qualified_name};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

/// Handle a metacommand
///
/// Returns Ok(true) to continue the REPL, Ok(false) to exit.
pub fn handle_metacommand<W: Write>(
    input: &str,
    state: &mut ReplState,
    client: &DatabaseClient,
    writer: &mut W,
) -> Result<bool> {
    // Normalize: remove leading / or \, strip trailing semicolons (users type them
    // out of SQL habit), and lowercase for command matching
    let trimmed = input.trim().trim_end_matches(';').trim();
    let without_prefix = trimmed.trim_start_matches('/').trim_start_matches('\\');

    // Split into command and arguments (preserve case for arguments)
    let mut parts = without_prefix.split_whitespace();
    let command = parts.next().unwrap_or("").to_lowercase();
    let args: Vec<&str> = parts.collect();

    match command.as_str() {
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

        // Ping command (Sprint 4)
        "ping" => {
            execute_ping(client, state, writer)?;
        }

        // Describe command — delegates to batch describe module
        "describe" | "d" => {
            if args.is_empty() {
                writeln!(writer, "Usage: /describe <table_name>")?;
                writeln!(writer, "       /describe <database>.<table_name>")?;
            } else {
                crate::commands::describe::execute_for_repl(client, args[0], writer)?;
            }
        }

        // Export command (Sprint 6, Sprint 12, Sprint 13: simplified syntax)
        // Note: Old handler - no full dataset export (client not available for re-execution)
        "export" => {
            if args.is_empty() {
                writeln!(writer, "Usage: /export <format> [destination]")?;
                writeln!(writer)?;
                writeln!(writer, "Formats: table, csv, json, sql")?;
                writeln!(
                    writer,
                    "Destination: file path or 'clipboard' (optional, defaults to stdout)"
                )?;
                writeln!(writer)?;
                writeln!(writer, "Examples:")?;
                writeln!(writer, "  /export csv results.csv  Export to file")?;
                writeln!(writer, "  /export json clipboard   Copy JSON to clipboard")?;
                writeln!(writer, "  /export table            Print table to stdout")?;
            } else {
                execute_export(state, None, writer, &args)?;
            }
        }

        // Pager control command (Sprint 6)
        "pager" => {
            if args.is_empty() {
                // Show current setting
                let status = if state.is_pager_enabled() {
                    "on"
                } else {
                    "off"
                };
                writeln!(writer, "Pager: {}", status)?;
            } else {
                match args[0].to_lowercase().as_str() {
                    "on" => {
                        state.set_pager(true);
                        writeln!(writer, "Result paging enabled")?;
                    }
                    "off" => {
                        state.set_pager(false);
                        writeln!(writer, "Result paging disabled")?;
                    }
                    _ => {
                        writeln!(
                            writer,
                            "Invalid pager setting '{}'. Use 'on' or 'off'.",
                            args[0]
                        )?;
                    }
                }
            }
        }

        // Colors control command (Sprint 6)
        "colors" => {
            if args.is_empty() {
                // Show current setting
                let status = if state.are_colors_enabled() {
                    "on"
                } else {
                    "off"
                };
                writeln!(writer, "Colors: {}", status)?;
            } else {
                match args[0].to_lowercase().as_str() {
                    "on" => {
                        state.set_colors(true);
                        writeln!(writer, "Syntax highlighting enabled")?;
                    }
                    "off" => {
                        state.set_colors(false);
                        writeln!(writer, "Syntax highlighting disabled")?;
                    }
                    _ => {
                        writeln!(
                            writer,
                            "Invalid color setting '{}'. Use 'on' or 'off'.",
                            args[0]
                        )?;
                    }
                }
            }
        }

        // Sprint 36: Repeat last query (basic handler - no client available)
        "repeat" | "r" => {
            match state.last_sql() {
                Some(sql) => {
                    writeln!(writer, "Repeating: {}", sql)?;
                    writeln!(
                        writer,
                        "Note: /repeat requires full REPL mode for execution."
                    )?;
                }
                None => {
                    writeln!(writer, "No previous query to repeat.")?;
                }
            }
        }

        // Sprint 37: Edit last query (basic handler - no client available)
        "edit" | "e" => {
            writeln!(
                writer,
                "The /edit command requires full REPL mode with database connection."
            )?;
        }

        // Sprint 36: Show indexes (basic handler - no client available for query)
        "show" => {
            writeln!(
                writer,
                "The /show command requires full REPL mode with database connection."
            )?;
        }
        "di" => {
            writeln!(
                writer,
                "The /di command requires full REPL mode with database connection."
            )?;
        }

        // Sprint 38: Sysconfig and locks (basic handler - no client available)
        "sysconfig" | "sc" => {
            writeln!(
                writer,
                "The /sysconfig command requires full REPL mode with database connection."
            )?;
        }
        "locks" | "lk" => {
            writeln!(
                writer,
                "The /locks command requires full REPL mode with database connection."
            )?;
        }
        // Sprint 39: Query inspection (basic handler - no client available)
        "query" | "qi" => {
            writeln!(
                writer,
                "The /query command requires full REPL mode with database connection."
            )?;
        }

        // Sprint 45: Inspect command (basic handler - no client available)
        "inspect" | "i" => {
            writeln!(
                writer,
                "The /inspect command requires full REPL mode with database connection."
            )?;
        }

        // Sprint 50: Explain and skew (basic handler - no client available)
        "explain" => {
            writeln!(
                writer,
                "The /explain command requires full REPL mode with database connection."
            )?;
        }
        "skew" => {
            writeln!(
                writer,
                "The /skew command requires full REPL mode with database connection."
            )?;
        }

        // Sprint 49: Session control (basic handler - no client available)
        "abort" => {
            writeln!(
                writer,
                "The /abort command requires full REPL mode with database connection."
            )?;
        }
        "priority" => {
            writeln!(
                writer,
                "The /priority command requires full REPL mode with database connection."
            )?;
        }

        // Sprint 40: Params command (basic handler)
        "params" | "p" => {
            handle_params_basic(&args, state, writer)?;
        }

        // Unknown command
        _ => {
            writeln!(writer, "Unknown command: /{}", command)?;
            writeln!(writer, "Type /help for available commands.")?;
        }
    }

    Ok(true)
}

/// Handle a metacommand with mutable CompletionState (Sprint 7)
///
/// This version supports /logon for connection switching and uses the
/// shared completion state for database operations.
///
/// Returns Ok(true) to continue the REPL, Ok(false) to exit.
pub fn handle_metacommand_with_state<W: Write>(
    input: &str,
    state: &mut ReplState,
    completion_state: &mut CompletionState,
    writer: &mut W,
) -> Result<bool> {
    // Normalize: remove leading / or \, strip trailing semicolons (users type them
    // out of SQL habit), and lowercase for command matching
    let trimmed = input.trim().trim_end_matches(';').trim();
    let without_prefix = trimmed.trim_start_matches('/').trim_start_matches('\\');

    // Split into command and arguments (preserve case for arguments)
    let mut parts = without_prefix.split_whitespace();
    let command = parts.next().unwrap_or("").to_lowercase();
    let args: Vec<&str> = parts.collect();

    match command.as_str() {
        // Exit commands
        "quit" | "q" | "exit" => {
            return Ok(false);
        }

        // Help command
        "help" | "?" => {
            print_help_extended(writer)?;
        }

        // Session info command
        "session" => {
            print_session_info(state, writer)?;
        }

        // Ping command (Sprint 4)
        "ping" => {
            execute_ping(completion_state.client(), state, writer)?;
        }

        // Describe command — delegates to batch describe module
        "describe" | "d" => {
            if args.is_empty() {
                writeln!(writer, "Usage: /describe <table_name>")?;
                writeln!(writer, "       /describe <database>.<table_name>")?;
            } else {
                crate::commands::describe::execute_for_repl(
                    completion_state.client(),
                    args[0],
                    writer,
                )?;
            }
        }

        // Export command (Sprint 6, Sprint 12, Sprint 13: simplified syntax)
        "export" => {
            if args.is_empty() {
                writeln!(writer, "Usage: /export <format> [file|clipboard]")?;
                writeln!(writer)?;
                writeln!(writer, "Formats: table, csv, json, sql")?;
                writeln!(writer)?;
                writeln!(writer, "Examples:")?;
                writeln!(writer, "  /export csv results.csv    Export to file")?;
                writeln!(writer, "  /export json clipboard     Copy JSON to clipboard")?;
                writeln!(writer, "  /export table              Print table to stdout")?;
                writeln!(writer)?;
                writeln!(writer, "Note: File exports include ALL rows (no limit),")?;
                writeln!(
                    writer,
                    "      clipboard/stdout exports use currently displayed rows."
                )?;
            } else {
                execute_export(state, Some(completion_state.client()), writer, &args)?;
            }
        }

        // Pager control command (Sprint 6)
        "pager" => {
            if args.is_empty() {
                // Show current setting
                let status = if state.is_pager_enabled() {
                    "on"
                } else {
                    "off"
                };
                writeln!(writer, "Pager: {}", status)?;
            } else {
                match args[0].to_lowercase().as_str() {
                    "on" => {
                        state.set_pager(true);
                        writeln!(writer, "Result paging enabled")?;
                    }
                    "off" => {
                        state.set_pager(false);
                        writeln!(writer, "Result paging disabled")?;
                    }
                    _ => {
                        writeln!(
                            writer,
                            "Invalid pager setting '{}'. Use 'on' or 'off'.",
                            args[0]
                        )?;
                    }
                }
            }
        }

        // Colors control command (Sprint 6)
        "colors" => {
            if args.is_empty() {
                // Show current setting
                let status = if state.are_colors_enabled() {
                    "on"
                } else {
                    "off"
                };
                writeln!(writer, "Colors: {}", status)?;
            } else {
                match args[0].to_lowercase().as_str() {
                    "on" => {
                        state.set_colors(true);
                        writeln!(writer, "Syntax highlighting enabled")?;
                    }
                    "off" => {
                        state.set_colors(false);
                        writeln!(writer, "Syntax highlighting disabled")?;
                    }
                    _ => {
                        writeln!(
                            writer,
                            "Invalid color setting '{}'. Use 'on' or 'off'.",
                            args[0]
                        )?;
                    }
                }
            }
        }

        // Logon command (Sprint 7)
        "logon" => {
            if args.is_empty() {
                writeln!(writer)?;
                writeln!(writer, "Usage: /logon <connection_string>")?;
                writeln!(writer)?;
                writeln!(writer, "Format: user:password@host:port/database")?;
                writeln!(
                    writer,
                    "        user@host:port/database  (password from env/file)"
                )?;
                writeln!(writer)?;
                writeln!(writer, "Examples:")?;
                writeln!(writer, "  /logon alice:secret@dbhost:1025/prod")?;
                writeln!(writer, "  /logon bob@192.168.1.100:1025/staging")?;
                writeln!(writer)?;
            } else {
                execute_logon(args[0], state, completion_state, writer)?;
            }
        }

        // Sprint 22: List command for schema inspection
        "list" | "l" => {
            if args.is_empty() {
                writeln!(writer)?;
                writeln!(writer, "Usage: /list <subcommand> [options]")?;
                writeln!(writer)?;
                writeln!(writer, "Subcommands:")?;
                writeln!(
                    writer,
                    "  databases           List all accessible databases"
                )?;
                writeln!(
                    writer,
                    "  tables [pattern]    List tables (optional glob pattern)"
                )?;
                writeln!(writer, "  views               List views in current database")?;
                writeln!(writer)?;
                writeln!(writer, "Examples:")?;
                writeln!(writer, "  /list databases")?;
                writeln!(writer, "  /list tables")?;
                writeln!(writer, "  /list tables order*")?;
                writeln!(writer, "  /list views")?;
                writeln!(writer)?;
                writeln!(writer, "Aliases: /l (short for /list)")?;
                writeln!(writer)?;
            } else {
                execute_list(completion_state, &args, writer)?;
            }
        }

        // Sprint 22: Direct aliases for list subcommands
        "dt" => {
            // /dt is alias for /list tables
            execute_list(completion_state, &["tables"], writer)?;
        }
        "dv" => {
            // /dv is alias for /list views
            execute_list(completion_state, &["views"], writer)?;
        }

        // Sprint 26: Sessions command
        "sessions" => {
            crate::commands::sessions::execute_for_repl(completion_state.client(), writer)?;
        }

        // Sprint 33: Data sampling commands
        "sample" => {
            let args_str = args.join(" ");
            execute_sample(completion_state, &args_str, writer)?;
        }
        "peek" => {
            let args_str = args.join(" ");
            execute_peek(completion_state, &args_str, writer)?;
        }

        // Sprint 36: Repeat last query
        "repeat" | "r" => {
            execute_repeat(state, completion_state, writer)?;
        }

        // Sprint 37: Edit last query in external editor
        "edit" | "e" => {
            execute_edit(state, completion_state, writer)?;
        }

        // Sprint 36: Show indexes command
        "show" => {
            if args.is_empty() {
                writeln!(writer)?;
                writeln!(writer, "Usage: /show <subcommand> [options]")?;
                writeln!(writer)?;
                writeln!(writer, "Subcommands:")?;
                writeln!(
                    writer,
                    "  indexes <table>    Show index information for a table"
                )?;
                writeln!(writer)?;
                writeln!(writer, "Examples:")?;
                writeln!(writer, "  /show indexes employees")?;
                writeln!(writer, "  /show indexes prod.orders")?;
                writeln!(writer)?;
            } else {
                let subcommand = args[0].to_lowercase();
                match subcommand.as_str() {
                    "indexes" | "index" => {
                        if args.len() < 2 {
                            writeln!(writer, "Usage: /show indexes <table_name>")?;
                            writeln!(
                                writer,
                                "       /show indexes <database>.<table_name>"
                            )?;
                        } else {
                            crate::commands::show_indexes::execute_for_repl(
                                completion_state.client(),
                                args[1],
                                writer,
                            )?;
                        }
                    }
                    _ => {
                        writeln!(writer)?;
                        writeln!(writer, "Unknown show subcommand: {}", subcommand)?;
                        writeln!(writer, "Available: indexes")?;
                        writeln!(writer)?;
                    }
                }
            }
        }
        // Sprint 36: Direct alias for /show indexes
        "di" => {
            if args.is_empty() {
                writeln!(writer, "Usage: /di <table_name>")?;
                writeln!(writer, "       /di <database>.<table_name>")?;
            } else {
                crate::commands::show_indexes::execute_for_repl(
                    completion_state.client(),
                    args[0],
                    writer,
                )?;
            }
        }

        // Sprint 38: System configuration command
        "sysconfig" | "sc" => {
            crate::commands::sysconfig::execute_for_repl(completion_state.client(), writer)?;
        }

        // Sprint 38: Lock information command
        "locks" | "lk" => {
            crate::commands::locks::execute_for_repl(completion_state.client(), writer)?;
        }

        // Sprint 39: Query inspection command
        "query" | "qi" => {
            if args.is_empty() {
                writeln!(writer, "Usage: /query <session_id>")?;
                writeln!(writer, "       /qi <session_id>")?;
                writeln!(writer)?;
                writeln!(
                    writer,
                    "Show recent SQL queries for a given session."
                )?;
                writeln!(writer)?;
                writeln!(writer, "Examples:")?;
                writeln!(writer, "  /query 1234")?;
                writeln!(writer, "  /qi 1234")?;
                writeln!(writer)?;
                writeln!(writer, "Use /sessions to list active session IDs.")?;
            } else {
                match args[0].parse::<i64>() {
                    Ok(session_id) => {
                        crate::commands::query_inspect::execute_for_repl(
                            completion_state.client(),
                            session_id,
                            writer,
                        )?;
                    }
                    Err(_) => {
                        writeln!(
                            writer,
                            "Error: '{}' is not a valid session ID. Expected a number.",
                            args[0]
                        )?;
                    }
                }
            }
        }

        // Sprint 45: Inspect command (full handler)
        "inspect" | "i" => {
            if args.is_empty() {
                writeln!(writer)?;
                writeln!(writer, "Usage: /inspect <table_or_view>")?;
                writeln!(writer, "       /inspect <database>.<object>")?;
                writeln!(writer)?;
                writeln!(writer, "Examples:")?;
                writeln!(writer, "  /inspect employees")?;
                writeln!(writer, "  /inspect mydb.orders")?;
                writeln!(writer, "  /inspect DBC.TablesV")?;
                writeln!(writer)?;
            } else {
                let object_name = args.join(" ");
                crate::commands::inspect::execute_for_repl(
                    completion_state.client(),
                    &object_name,
                    writer,
                )?;
            }
        }

        // Sprint 49: Abort session/query command
        "abort" => {
            if args.is_empty() {
                writeln!(writer)?;
                writeln!(writer, "Usage: /abort <session_id> [yes]")?;
                writeln!(writer, "       /abort query <session_id> [yes]")?;
                writeln!(writer)?;
                writeln!(writer, "Abort a session or its running query.")?;
                writeln!(writer, "Append 'yes' to confirm the operation.")?;
                writeln!(writer)?;
                writeln!(writer, "Examples:")?;
                writeln!(writer, "  /abort 1234 yes       Abort session 1234")?;
                writeln!(writer, "  /abort query 1234 yes Abort running query on session 1234")?;
                writeln!(writer)?;
                writeln!(writer, "Use /sessions to list active session IDs.")?;
                writeln!(writer)?;
            } else if args[0].eq_ignore_ascii_case("query") {
                // /abort query <session_id> [yes]
                if args.len() < 2 {
                    writeln!(writer, "Usage: /abort query <session_id> [yes]")?;
                } else {
                    match args[1].parse::<i64>() {
                        Ok(session_id) => {
                            let confirmed = args.len() > 2 && args[2].eq_ignore_ascii_case("yes");
                            crate::commands::abort::execute_for_repl(
                                completion_state.client(),
                                session_id,
                                true,
                                confirmed,
                                writer,
                            )?;
                        }
                        Err(_) => {
                            writeln!(
                                writer,
                                "Error: '{}' is not a valid session ID. Expected a number.",
                                args[1]
                            )?;
                        }
                    }
                }
            } else {
                // /abort <session_id> [yes]
                match args[0].parse::<i64>() {
                    Ok(session_id) => {
                        let confirmed = args.len() > 1 && args[1].eq_ignore_ascii_case("yes");
                        crate::commands::abort::execute_for_repl(
                            completion_state.client(),
                            session_id,
                            false,
                            confirmed,
                            writer,
                        )?;
                    }
                    Err(_) => {
                        writeln!(
                            writer,
                            "Error: '{}' is not a valid session ID. Expected a number.",
                            args[0]
                        )?;
                    }
                }
            }
        }

        // Sprint 50: Explain plan command
        "explain" => {
            let sql = args.join(" ");
            crate::commands::explain::execute_for_repl(
                completion_state.client(),
                &sql,
                writer,
            )?;
        }

        // Sprint 50: Skew analysis command
        "skew" => {
            if args.is_empty() {
                // Show top sessions by skew
                crate::commands::skew::execute_for_repl(
                    completion_state.client(),
                    None,
                    writer,
                )?;
            } else {
                match args[0].parse::<i64>() {
                    Ok(session_id) => {
                        crate::commands::skew::execute_for_repl(
                            completion_state.client(),
                            Some(session_id),
                            writer,
                        )?;
                    }
                    Err(_) => {
                        writeln!(
                            writer,
                            "Error: '{}' is not a valid session ID. Expected a number.",
                            args[0]
                        )?;
                        writeln!(writer)?;
                        writeln!(writer, "Usage: /skew [session_id]")?;
                        writeln!(writer, "       /skew           Show top sessions by skew")?;
                        writeln!(writer, "       /skew 1234      Analyze specific session")?;
                    }
                }
            }
        }

        // Sprint 49: Priority change command
        "priority" => {
            if args.len() < 2 {
                writeln!(writer)?;
                writeln!(writer, "Usage: /priority <session_id> <level>")?;
                writeln!(writer)?;
                writeln!(writer, "Change the priority of a Teradata session.")?;
                writeln!(writer, "Valid levels: RUSH, MEDIUM, LOW")?;
                writeln!(writer)?;
                writeln!(writer, "Examples:")?;
                writeln!(writer, "  /priority 1234 rush")?;
                writeln!(writer, "  /priority 1234 low")?;
                writeln!(writer)?;
                writeln!(writer, "Use /sessions to list active session IDs.")?;
                writeln!(writer)?;
            } else {
                match args[0].parse::<i64>() {
                    Ok(session_id) => {
                        crate::commands::priority::execute_for_repl(
                            completion_state.client(),
                            session_id,
                            args[1],
                            writer,
                        )?;
                    }
                    Err(_) => {
                        writeln!(
                            writer,
                            "Error: '{}' is not a valid session ID. Expected a number.",
                            args[0]
                        )?;
                    }
                }
            }
        }

        // Sprint 40: Params command (full handler)
        "params" | "p" => {
            handle_params_basic(&args, state, writer)?;
        }

        // Unknown command
        _ => {
            writeln!(writer, "Unknown command: /{}", command)?;
            writeln!(writer, "Type /help for available commands.")?;
        }
    }

    Ok(true)
}

/// Print help text (extended version with /logon and /list)
fn print_help_extended<W: Write>(writer: &mut W) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "tq REPL Commands:")?;
    writeln!(writer, "  /help, /?              Show this help message")?;
    writeln!(writer, "  /quit, /q              Exit the REPL")?;
    writeln!(
        writer,
        "  /edit, /e              Edit last query in $EDITOR"
    )?;
    writeln!(
        writer,
        "  /repeat, /r            Re-execute last query"
    )?;
    writeln!(
        writer,
        "  /session               Show current session information"
    )?;
    writeln!(writer, "  /ping                  Test database connection")?;
    writeln!(writer, "  /describe <table>, /d  Show table structure")?;
    writeln!(
        writer,
        "  /export <fmt> [file|clipboard]  Export result (csv, json, table, sql)"
    )?;
    writeln!(
        writer,
        "  /pager on|off          Enable/disable result paging"
    )?;
    writeln!(
        writer,
        "  /colors on|off         Enable/disable syntax highlighting"
    )?;
    writeln!(
        writer,
        "  /logon <conn_str>      Switch to a different connection"
    )?;
    writeln!(writer)?;
    writeln!(writer, "Schema Inspection:")?;
    writeln!(
        writer,
        "  /inspect <obj>, /i     Inspect object (type, columns, indexes, size)"
    )?;
    writeln!(
        writer,
        "  /list databases        List all accessible databases"
    )?;
    writeln!(
        writer,
        "  /list tables [pattern] List tables (optional glob pattern)"
    )?;
    writeln!(writer, "  /list views            List views in current database")?;
    writeln!(
        writer,
        "  /show indexes <table>  Show index information"
    )?;
    writeln!(writer, "  /dt                    Shortcut for /list tables")?;
    writeln!(writer, "  /dv                    Shortcut for /list views")?;
    writeln!(writer, "  /di <table>            Shortcut for /show indexes")?;
    writeln!(writer)?;
    writeln!(writer, "Data Exploration:")?;
    writeln!(
        writer,
        "  /sample <table> [n]    Show random sample (default: 10, max: 1000)"
    )?;
    writeln!(
        writer,
        "  /peek <table>          Show first 5 rows with column info"
    )?;
    writeln!(writer)?;
    writeln!(writer, "System Monitoring:")?;
    writeln!(
        writer,
        "  /sessions              List active sessions with performance metrics"
    )?;
    writeln!(
        writer,
        "  /sysconfig, /sc        Display system configuration (version and AMP count)"
    )?;
    writeln!(
        writer,
        "  /locks, /lk            Display current lock contention and blocking chains"
    )?;
    writeln!(
        writer,
        "  /query <id>, /qi <id>  Show recent SQL queries for a session"
    )?;
    writeln!(
        writer,
        "  /abort <id> [yes]      Abort a session (append 'yes' to confirm)"
    )?;
    writeln!(
        writer,
        "  /abort query <id> [yes] Abort running query on a session"
    )?;
    writeln!(
        writer,
        "  /priority <id> <level> Change session priority (RUSH/MEDIUM/LOW)"
    )?;
    writeln!(
        writer,
        "  /explain <sql>         Show execution plan for a SQL statement"
    )?;
    writeln!(
        writer,
        "  /skew [session_id]     Analyze AMP-level resource skew"
    )?;
    writeln!(writer)?;
    writeln!(writer, "Variable Substitution:")?;
    writeln!(
        writer,
        "  /params load <file>    Load a YAML parameter file"
    )?;
    writeln!(
        writer,
        "  /params unload         Clear all loaded parameters"
    )?;
    writeln!(
        writer,
        "  /params show           Show loaded parameters and variables"
    )?;
    writeln!(
        writer,
        "  /p                     Shortcut for /params"
    )?;
    writeln!(writer)?;
    writeln!(writer, "SQL Execution:")?;
    writeln!(writer, "  Enter SQL statements ending with semicolon (;)")?;
    writeln!(writer, "  Multi-line statements are supported")?;
    writeln!(writer, "  /edit opens the last query in your $EDITOR")?;
    writeln!(writer, "  /repeat re-executes the last SQL statement")?;
    writeln!(writer)?;
    writeln!(writer, "Tab Completion:")?;
    writeln!(writer, "  Tab after /            Complete metacommands")?;
    writeln!(writer, "  Tab after FROM/JOIN    Complete table names")?;
    writeln!(writer, "  Tab after SELECT/WHERE Complete column names")?;
    writeln!(writer)?;
    writeln!(writer, "Keyboard Shortcuts:")?;
    writeln!(writer, "  Up/Down        Navigate command history")?;
    writeln!(
        writer,
        "  Tab            Auto-complete (commands, tables, columns)"
    )?;
    writeln!(writer, "  Ctrl-C         Cancel current input")?;
    writeln!(writer, "  Ctrl-D         Exit REPL (when input is empty)")?;
    writeln!(writer, "  Ctrl-R         Search command history")?;
    writeln!(writer)?;
    writeln!(writer, "Result Paging:")?;
    writeln!(
        writer,
        "  When result sets are large, an interactive pager activates automatically."
    )?;
    writeln!(
        writer,
        "  j/k or Up/Down   Scroll rows up/down"
    )?;
    writeln!(
        writer,
        "  Space/b          Page down/up"
    )?;
    writeln!(
        writer,
        "  Left/Right       Scroll columns (for wide tables)"
    )?;
    writeln!(
        writer,
        "  g/G              Jump to first/last row"
    )?;
    writeln!(
        writer,
        "  q or Esc         Exit pager, return to prompt"
    )?;
    writeln!(
        writer,
        "  Column indicators (+N cols) show when columns are hidden."
    )?;
    writeln!(writer)?;

    Ok(())
}

/// Execute the /logon metacommand (Sprint 7)
///
/// Switches to a new database connection, clearing the metadata cache.
fn execute_logon<W: Write>(
    connection_string: &str,
    state: &mut ReplState,
    completion_state: &mut CompletionState,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "Connecting...")?;

    let start = Instant::now();

    // Parse connection string
    // Use TD2 as default logmech and 30s timeout
    let config = match ConnectionConfig::from_connection_string(
        connection_string,
        LogonMechanism::Td2,
        Duration::from_secs(30),
        None,
    ) {
        Ok(mut cfg) => {
            // Try to resolve password from environment/file
            if let Err(e) = cfg.resolve_password(None) {
                writeln!(writer, "Error: {}", e)?;
                writeln!(writer)?;
                return Ok(());
            }
            cfg
        }
        Err(e) => {
            writeln!(writer, "Error: Invalid connection string")?;
            writeln!(writer, "{}", e)?;
            writeln!(writer)?;
            writeln!(writer, "Format: user:password@host:port/database")?;
            writeln!(writer)?;
            return Ok(());
        }
    };

    // Create new database client
    let new_client = match DatabaseClient::new(config.clone(), None) {
        Ok(client) => client,
        Err(e) => {
            writeln!(writer, "Error: Failed to create client")?;
            writeln!(writer, "{}", e)?;
            writeln!(writer)?;
            return Ok(());
        }
    };

    // Test connection with ping
    match new_client.ping() {
        Ok(latency) => {
            let elapsed = start.elapsed();

            // Update completion state with new client
            completion_state.update_client(new_client, &config.database);

            // Update REPL state
            state.update_connection(config.clone(), Some(connection_string.to_string()));

            writeln!(writer)?;
            writeln!(writer, "Connected! ({}ms)", elapsed.as_millis())?;
            writeln!(writer, "  Host:     {}:{}", config.host, config.port)?;
            writeln!(writer, "  Database: {}", config.database)?;
            writeln!(writer, "  User:     {}", config.user)?;
            writeln!(writer, "  Latency:  {}ms", latency.as_millis())?;
            writeln!(writer)?;
            writeln!(
                writer,
                "Note: Tab completion cache cleared for new connection."
            )?;
        }
        Err(e) => {
            let elapsed = start.elapsed();
            writeln!(writer)?;
            writeln!(
                writer,
                "Connection FAILED (after {}ms)",
                elapsed.as_millis()
            )?;
            writeln!(writer)?;
            writeln!(writer, "Error: {}", e)?;
            writeln!(writer)?;
            writeln!(writer, "The previous connection remains active.")?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Execute the /list metacommand — delegates to batch list module
///
/// Provides schema inspection commands:
/// - /list databases - List all accessible databases
/// - /list tables [pattern] - List tables with optional glob pattern
/// - /list views - List views in current database
fn execute_list<W: Write>(
    completion_state: &mut CompletionState,
    args: &[&str],
    writer: &mut W,
) -> Result<()> {
    if args.is_empty() {
        writeln!(writer, "Error: Missing subcommand.")?;
        writeln!(writer, "Usage: /list <databases|tables|views>")?;
        return Ok(());
    }

    let subcommand = args[0].to_lowercase();
    let pattern = args.get(1).copied();

    crate::commands::list::execute_for_repl(
        completion_state.client(),
        &subcommand,
        pattern,
        None,
        writer,
    )?;

    Ok(())
}

// Old execute_list_databases, execute_list_tables, execute_list_views, matches_glob
// removed — REPL now delegates to crate::commands::list::execute_for_repl.

/// Print help text
fn print_help<W: Write>(writer: &mut W) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "tq REPL Commands:")?;
    writeln!(writer, "  /help, /?              Show this help message")?;
    writeln!(writer, "  /quit, /q              Exit the REPL")?;
    writeln!(
        writer,
        "  /edit, /e              Edit last query in $EDITOR"
    )?;
    writeln!(
        writer,
        "  /repeat, /r            Re-execute last query"
    )?;
    writeln!(
        writer,
        "  /session               Show current session information"
    )?;
    writeln!(writer, "  /ping                  Test database connection")?;
    writeln!(writer, "  /describe <table>, /d  Show table structure")?;
    writeln!(
        writer,
        "  /show indexes <table>  Show index information"
    )?;
    writeln!(
        writer,
        "  /export <fmt> [file|clipboard]  Export result (csv, json, table, sql)"
    )?;
    writeln!(
        writer,
        "  /pager on|off          Enable/disable result paging"
    )?;
    writeln!(
        writer,
        "  /colors on|off         Enable/disable syntax highlighting"
    )?;
    writeln!(writer)?;
    writeln!(writer, "SQL Execution:")?;
    writeln!(writer, "  Enter SQL statements ending with semicolon (;)")?;
    writeln!(writer, "  Multi-line statements are supported")?;
    writeln!(writer, "  /edit opens the last query in your $EDITOR")?;
    writeln!(writer, "  /repeat re-executes the last SQL statement")?;
    writeln!(writer)?;
    writeln!(writer, "Keyboard Shortcuts:")?;
    writeln!(writer, "  Up/Down        Navigate command history")?;
    writeln!(writer, "  Tab            Auto-complete SQL keywords")?;
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

/// Execute the /ping metacommand
///
/// Tests database connectivity and displays latency information.
fn execute_ping<W: Write>(
    client: &DatabaseClient,
    state: &ReplState,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    let start = Instant::now();
    match client.ping() {
        Ok(latency) => {
            let config = state.connection_info();
            let session_duration = format_duration(state.session_duration());

            writeln!(writer, "Connection OK ({}ms)", latency.as_millis())?;
            writeln!(writer, "Host: {}:{}", config.host, config.port)?;
            writeln!(writer, "Database: {}", config.database)?;
            writeln!(writer, "User: {}", config.user)?;
            writeln!(writer, "Session active for: {}", session_duration)?;
        }
        Err(e) => {
            let elapsed = start.elapsed();
            let config = state.connection_info();

            writeln!(
                writer,
                "Connection FAILED (after {}ms)",
                elapsed.as_millis()
            )?;
            writeln!(writer)?;
            writeln!(writer, "Error: {}", e)?;
            writeln!(writer)?;
            writeln!(writer, "Host: {}:{}", config.host, config.port)?;
            writeln!(writer)?;
            writeln!(writer, "Suggestions:")?;
            writeln!(writer, "  - Check network connectivity")?;
            writeln!(writer, "  - Database may be overloaded")?;
            writeln!(writer, "  - Session may have timed out")?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

// Old execute_describe, format_nullable, truncate_string removed —
// REPL /describe now delegates to crate::commands::describe::execute_for_repl.

/// Execute the /export metacommand (Sprint 6, Sprint 12, Sprint 13)
///
/// Exports the last query result to a file or clipboard in various formats.
/// When exporting to a file and the result was limited, re-executes the query
/// to get the full dataset.
///
/// Sprint 13: Simplified syntax
///   /export <format> [destination]
///
/// Where:
///   format: table, csv, json, sql (REQUIRED)
///   destination: file path or 'clipboard' (OPTIONAL, defaults to stdout)
///
/// Deprecated (still works with warnings):
///   /export clipboard <format>  -> Use /export <format> clipboard
///   /export <format> --append   -> Append mode removed
fn execute_export<W: Write>(
    state: &ReplState,
    client: Option<&DatabaseClient>,
    writer: &mut W,
    args: &[&str],
) -> Result<()> {
    // Check if we have a result to export
    let result = match state.last_result() {
        Some(r) => r,
        None => {
            writeln!(writer)?;
            writeln!(writer, "Error: No query results to export.")?;
            writeln!(writer, "  Run a SELECT query first, then use /export")?;
            writeln!(writer)?;
            return Ok(());
        }
    };

    // Parse arguments to determine: format, destination (file/clipboard), append flag
    let (format, destination, append, deprecation) = parse_export_args(args);

    // Show deprecation warnings (Sprint 13)
    match deprecation {
        DeprecationWarning::ClipboardFirst => {
            writeln!(writer)?;
            writeln!(
                writer,
                "Warning: Deprecated syntax: Use '/export {} clipboard' instead",
                format
            )?;
        }
        DeprecationWarning::AppendMode => {
            writeln!(writer)?;
            writeln!(
                writer,
                "Warning: --append mode is deprecated and will be removed in a future version"
            )?;
        }
        DeprecationWarning::None => {}
    }

    // Validate format
    let format_lower = format.to_lowercase();
    if !["table", "csv", "json", "sql"].contains(&format_lower.as_str()) {
        writeln!(writer)?;
        writeln!(
            writer,
            "Error: Unknown format '{}'. Supported formats: table, csv, json, sql",
            format
        )?;
        writeln!(writer)?;
        return Ok(());
    }

    // For file exports, check if we need to re-execute to get full dataset (Sprint 12)
    let result_to_export = match destination {
        ExportDestination::File(_) if state.was_last_result_limited() => {
            // Need to re-execute query without limit to get full dataset
            match (client, state.last_sql()) {
                (Some(db_client), Some(sql)) => {
                    writeln!(writer)?;
                    writeln!(writer, "Re-executing query to export full dataset...")?;

                    match db_client.execute(sql) {
                        Ok(full_result) => {
                            writeln!(
                                writer,
                                "Retrieved {} rows (full dataset)",
                                full_result.row_count
                            )?;
                            Box::new(full_result)
                        }
                        Err(e) => {
                            writeln!(writer)?;
                            writeln!(writer, "Error re-executing query: {}", e)?;
                            writeln!(writer, "Falling back to limited results in memory.")?;
                            Box::new(result.clone())
                        }
                    }
                }
                _ => {
                    // No client available or no SQL stored - use what we have
                    writeln!(writer)?;
                    writeln!(writer, "Warning: Cannot re-execute query for full dataset.")?;
                    writeln!(
                        writer,
                        "Exporting limited results ({} rows).",
                        result.row_count
                    )?;
                    Box::new(result.clone())
                }
            }
        }
        _ => {
            // Clipboard or not limited - use current result
            Box::new(result.clone())
        }
    };

    // Export based on destination
    match destination {
        ExportDestination::Clipboard => {
            export_to_clipboard(&result_to_export, &format_lower, writer)?;
        }
        ExportDestination::File(filepath) => {
            // Export based on format
            match format_lower.as_str() {
                "table" => {
                    export_table(&result_to_export, Some(filepath), append, writer)?;
                }
                "csv" => {
                    export_csv(&result_to_export, Some(filepath), append, writer)?;
                }
                "json" => {
                    export_json(&result_to_export, Some(filepath), append, writer)?;
                }
                "sql" => {
                    export_sql(&result_to_export, Some(filepath), append, writer)?;
                }
                _ => unreachable!(),
            }
        }
        ExportDestination::Stdout => {
            // Export to stdout (no file specified)
            match format_lower.as_str() {
                "table" => {
                    export_table(&result_to_export, None, false, writer)?;
                }
                "csv" => {
                    export_csv(&result_to_export, None, false, writer)?;
                }
                "json" => {
                    export_json(&result_to_export, None, false, writer)?;
                }
                "sql" => {
                    export_sql(&result_to_export, None, false, writer)?;
                }
                _ => unreachable!(),
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Export destination
enum ExportDestination<'a> {
    Clipboard,
    File(&'a str),
    Stdout,
}

/// Deprecation warning for export syntax
enum DeprecationWarning {
    None,
    ClipboardFirst,
    AppendMode,
}

/// Parse export command arguments
///
/// Sprint 13: Simplified syntax per export-syntax-simplification-design.md
///
/// New unified syntax: /export <format> [destination]
/// - format: table, csv, json, sql (REQUIRED)
/// - destination: file path or 'clipboard' (OPTIONAL, defaults to stdout)
///
/// Deprecated syntax (still supported with warnings):
/// - /export clipboard <format>  -> Use /export <format> clipboard instead
/// - /export <format> --append   -> Append mode removed
///
/// Returns (format, destination, append_flag, deprecation_warning)
fn parse_export_args<'a>(
    args: &'a [&str],
) -> (String, ExportDestination<'a>, bool, DeprecationWarning) {
    let mut format = "table".to_string(); // default format
    let mut destination = ExportDestination::Stdout;
    let mut append = false;
    let mut deprecation = DeprecationWarning::None;

    // Check if "--append" appears in args (deprecated)
    if args.contains(&"--append") {
        append = true;
        deprecation = DeprecationWarning::AppendMode;
    }

    // Filter out flags to get positional args only
    let positional: Vec<&str> = args
        .iter()
        .filter(|&&arg| !arg.starts_with("--"))
        .copied()
        .collect();

    // Check if first positional is "clipboard" (deprecated syntax: /export clipboard <format>)
    if !positional.is_empty() && positional[0].eq_ignore_ascii_case("clipboard") {
        destination = ExportDestination::Clipboard;
        deprecation = DeprecationWarning::ClipboardFirst;

        // Second positional (if present) is the format
        if positional.len() > 1 {
            format = positional[1].to_string();
        }
    } else {
        // New syntax: /export <format> [destination]
        // First positional is format
        if !positional.is_empty() {
            format = positional[0].to_string();
        }

        // Second positional (if present) is destination
        if positional.len() > 1 {
            let dest_arg = positional[1];
            if dest_arg.eq_ignore_ascii_case("clipboard") {
                destination = ExportDestination::Clipboard;
            } else {
                destination = ExportDestination::File(dest_arg);
            }
        }
    }

    (format, destination, append, deprecation)
}

/// Export results to clipboard (Sprint 12)
///
/// Copies the formatted result to the system clipboard.
fn export_to_clipboard<W: Write>(
    result: &crate::db::QueryResult,
    format: &str,
    writer: &mut W,
) -> Result<()> {
    use arboard::Clipboard;

    // Format data as string
    let content = match format {
        "table" => format_as_table(result)?,
        "csv" => format_as_csv(result)?,
        "json" => format_as_json(result)?,
        "sql" => format_as_sql(result)?,
        _ => {
            return Err(crate::error::TqError::InvalidConfig(format!(
                "Unsupported format for clipboard: {}",
                format
            )))
        }
    };

    // Copy to clipboard
    match Clipboard::new() {
        Ok(mut clipboard) => match clipboard.set_text(&content) {
            Ok(_) => {
                writeln!(writer)?;
                writeln!(
                    writer,
                    "Exported {} rows to clipboard ({})",
                    result.row_count, format
                )?;
            }
            Err(e) => {
                writeln!(writer)?;
                writeln!(writer, "Error: Failed to copy to clipboard: {}", e)?;
                writeln!(writer)?;
                writeln!(writer, "The clipboard may not be available on this system.")?;
            }
        },
        Err(e) => {
            writeln!(writer)?;
            writeln!(writer, "Error: Clipboard not available: {}", e)?;
            writeln!(writer)?;
            writeln!(writer, "Possible reasons:")?;
            writeln!(writer, "  - Running in a headless environment")?;
            writeln!(writer, "  - Missing clipboard support on this platform")?;
        }
    }

    Ok(())
}

/// Format result as table string using terminal-aware column selection
///
/// Uses the format::table module which calculates content-based column widths,
/// detects terminal width, and hides columns that don't fit (with a "+N cols" indicator).
/// This prevents wide tables from rendering unreadable 2-char columns.
fn format_as_table(result: &crate::db::QueryResult) -> Result<String> {
    let options = crate::format::table::TableOptions {
        show_header: true,
        use_color: true,
        max_column_width: None,
    };
    crate::format::table::format_string(result, &options)
}

/// Format result as CSV string
fn format_as_csv(result: &crate::db::QueryResult) -> Result<String> {
    let mut output = String::new();

    // Add headers
    let headers: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
    output.push_str(&headers.join(","));
    output.push('\n');

    // Add rows
    for row in &result.rows {
        let values: Vec<String> = row
            .iter()
            .map(|v| {
                let s = v.display().to_string();
                // Escape quotes and wrap in quotes if needed
                if s.contains(',') || s.contains('"') || s.contains('\n') {
                    format!("\"{}\"", s.replace("\"", "\"\""))
                } else {
                    s
                }
            })
            .collect();
        output.push_str(&values.join(","));
        output.push('\n');
    }

    Ok(output)
}

/// Format result as JSON string
fn format_as_json(result: &crate::db::QueryResult) -> Result<String> {
    let mut rows = Vec::new();

    for row in &result.rows {
        let mut obj = serde_json::json!({});
        for (i, col) in result.columns.iter().enumerate() {
            let value = &row[i];
            let json_value = match value {
                crate::db::Value::Null => serde_json::Value::Null,
                crate::db::Value::String(s) => serde_json::Value::String(s.clone()),
                crate::db::Value::Integer(n) => serde_json::Value::Number((*n).into()),
                crate::db::Value::Decimal(f) => serde_json::Number::from_f64(*f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null),
                crate::db::Value::Boolean(b) => serde_json::Value::Bool(*b),
                crate::db::Value::Date(d) => serde_json::Value::String(d.clone()),
                crate::db::Value::Timestamp(ts) => serde_json::Value::String(ts.clone()),
                crate::db::Value::Time(t) => serde_json::Value::String(t.clone()),
                crate::db::Value::Bytes(_) => {
                    serde_json::Value::String(value.display().to_string())
                }
            };
            obj[&col.name] = json_value;
        }
        rows.push(obj);
    }

    let json_array = serde_json::Value::Array(rows);
    Ok(serde_json::to_string_pretty(&json_array)?)
}

/// Format result as SQL INSERT statements string
fn format_as_sql(result: &crate::db::QueryResult) -> Result<String> {
    let table_name = "exported_data";
    let mut output = String::new();

    // Add header comment
    output.push_str("-- Exported data\n");
    output.push_str(&format!("-- CREATE TABLE {} (\n", table_name));
    for col in &result.columns {
        output.push_str(&format!("--   {} VARCHAR(255),\n", col.name));
    }
    output.push_str("-- );\n\n");

    // Add INSERT statements
    for row in &result.rows {
        let mut values = Vec::new();
        for value in row {
            let sql_value = match value {
                crate::db::Value::Null => "NULL".to_string(),
                crate::db::Value::String(s) => format!("'{}'", s.replace("'", "''")),
                crate::db::Value::Integer(n) => n.to_string(),
                crate::db::Value::Decimal(f) => f.to_string(),
                crate::db::Value::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
                _ => format!("'{}'", value.display().to_string().replace("'", "''")),
            };
            values.push(sql_value);
        }

        let col_names: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
        output.push_str(&format!(
            "INSERT INTO {} ({}) VALUES ({});\n",
            table_name,
            col_names.join(", "),
            values.join(", ")
        ));
    }

    Ok(output)
}

/// Export results as table to file or stdout
fn export_table<W: Write>(
    result: &crate::db::QueryResult,
    file: Option<&str>,
    append: bool,
    writer: &mut W,
) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::BufWriter;

    let content = format_as_table(result)?;

    if let Some(filepath) = file {
        let file_handle = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(filepath);

        match file_handle {
            Ok(f) => {
                use std::io::Write as _;
                let mut file_writer = BufWriter::new(f);
                file_writer.write_all(content.as_bytes())?;
                writeln!(writer, "Exported {} rows to {}", result.row_count, filepath)?;
            }
            Err(e) => {
                writeln!(writer, "Error: Cannot write to {}: {}", filepath, e)?;
            }
        }
    } else {
        // Output to stdout
        write!(writer, "{}", content)?;
    }

    Ok(())
}

/// Export results as CSV
fn export_csv<W: Write>(
    result: &crate::db::QueryResult,
    file: Option<&str>,
    append: bool,
    writer: &mut W,
) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::BufWriter;

    let csv_content = format_as_csv(result)?;

    // Write to file or stdout
    if let Some(filepath) = file {
        let file_handle = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(filepath);

        match file_handle {
            Ok(f) => {
                use std::io::Write as _;
                let mut file_writer = BufWriter::new(f);
                file_writer.write_all(csv_content.as_bytes())?;
                writeln!(writer, "Exported {} rows to {}", result.row_count, filepath)?;
            }
            Err(e) => {
                writeln!(writer, "Error: Cannot write to {}: {}", filepath, e)?;
            }
        }
    } else {
        // Output to stdout
        write!(writer, "{}", csv_content)?;
    }

    Ok(())
}

/// Export results as JSON
fn export_json<W: Write>(
    result: &crate::db::QueryResult,
    file: Option<&str>,
    append: bool,
    writer: &mut W,
) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::BufWriter;

    let json_str = format_as_json(result)?;

    if let Some(filepath) = file {
        let file_handle = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(filepath);

        match file_handle {
            Ok(f) => {
                use std::io::Write as _;
                let mut file_writer = BufWriter::new(f);
                file_writer.write_all(json_str.as_bytes())?;
                writeln!(writer, "Exported {} rows to {}", result.row_count, filepath)?;
            }
            Err(e) => {
                writeln!(writer, "Error: Cannot write to {}: {}", filepath, e)?;
            }
        }
    } else {
        // Output to stdout
        writeln!(writer, "{}", json_str)?;
    }

    Ok(())
}

/// Export results as SQL INSERT statements
fn export_sql<W: Write>(
    result: &crate::db::QueryResult,
    file: Option<&str>,
    append: bool,
    writer: &mut W,
) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::BufWriter;

    let sql_content = format_as_sql(result)?;

    if let Some(filepath) = file {
        let file_handle = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(filepath);

        match file_handle {
            Ok(f) => {
                use std::io::Write as _;
                let mut file_writer = BufWriter::new(f);
                file_writer.write_all(sql_content.as_bytes())?;
                writeln!(writer, "Exported {} rows to {}", result.row_count, filepath)?;
            }
            Err(e) => {
                writeln!(writer, "Error: Cannot write to {}: {}", filepath, e)?;
            }
        }
    } else {
        // Output to stdout
        writeln!(writer, "{}", sql_content)?;
    }

    Ok(())
}

// =============================================================================
// Sprint 33: Data Sampling Commands
// =============================================================================

/// Default sample size for /sample command
const DEFAULT_SAMPLE_SIZE: usize = 10;

/// Maximum sample size for /sample command (prevent accidental large queries)
const MAX_SAMPLE_SIZE: usize = 1000;

/// Execute the /sample metacommand (Sprint 33)
///
/// Shows a random sample of rows from the specified table using Teradata SAMPLE clause.
///
/// Usage: /sample <table> [n]
///   - table: Table name (unqualified or database.table)
///   - n: Number of rows to sample (default: 10, max: 1000)
fn execute_sample<W: Write>(
    completion_state: &mut CompletionState,
    args: &str,
    writer: &mut W,
) -> Result<()> {
    let args_trimmed = args.trim();
    if args_trimmed.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "Usage: /sample <table> [n]")?;
        writeln!(writer)?;
        writeln!(writer, "Show random sample of rows from a table.")?;
        writeln!(writer)?;
        writeln!(writer, "Arguments:")?;
        writeln!(writer, "  table   Table name (or database.table)")?;
        writeln!(writer, "  n       Number of rows (default: {}, max: {})", DEFAULT_SAMPLE_SIZE, MAX_SAMPLE_SIZE)?;
        writeln!(writer)?;
        writeln!(writer, "Examples:")?;
        writeln!(writer, "  /sample employees        Sample 10 rows from employees")?;
        writeln!(writer, "  /sample orders 50        Sample 50 rows from orders")?;
        writeln!(writer, "  /sample prod.items 100   Sample from different database")?;
        writeln!(writer)?;
        return Ok(());
    }

    // Parse arguments: table_name [sample_size]
    let parts: Vec<&str> = args_trimmed.split_whitespace().collect();
    let table_name = parts[0];

    // Parse sample size
    let sample_size: usize = if parts.len() > 1 {
        match parts[1].parse::<usize>() {
            Ok(n) => n,
            Err(_) => {
                writeln!(writer)?;
                writeln!(writer, "Error: Invalid sample size '{}'", parts[1])?;
                writeln!(writer, "Sample size must be a positive integer between 1 and {}", MAX_SAMPLE_SIZE)?;
                writeln!(writer)?;
                writeln!(writer, "Example: /sample {} 50", table_name)?;
                writeln!(writer)?;
                return Ok(());
            }
        }
    } else {
        DEFAULT_SAMPLE_SIZE
    };

    // Validate sample size
    if sample_size == 0 {
        writeln!(writer)?;
        writeln!(writer, "Error: Sample size must be at least 1")?;
        writeln!(writer)?;
        writeln!(writer, "Example: /sample {} {}", table_name, DEFAULT_SAMPLE_SIZE)?;
        writeln!(writer)?;
        return Ok(());
    }

    if sample_size > MAX_SAMPLE_SIZE {
        writeln!(writer)?;
        writeln!(writer, "Error: Sample size {} exceeds maximum ({})", sample_size, MAX_SAMPLE_SIZE)?;
        writeln!(writer)?;
        writeln!(writer, "For larger samples, use SQL directly:")?;
        writeln!(writer, "  SELECT * FROM {} SAMPLE {};", table_name, sample_size)?;
        writeln!(writer)?;
        return Ok(());
    }

    // Resolve qualified table name
    let qualified_name = resolve_table_name(table_name, completion_state);

    // Generate SQL using Teradata SAMPLE clause with properly quoted identifier
    let quoted_name = quote_table_reference(&qualified_name);
    let sql = format!("SELECT * FROM {} SAMPLE {}", quoted_name, sample_size);

    writeln!(writer)?;

    // Execute query
    let client = completion_state.client();
    let start = Instant::now();

    match client.execute(&sql) {
        Ok(result) => {
            let elapsed = start.elapsed();

            // Display header
            writeln!(writer, "Random sample from {} ({} rows):", qualified_name, result.row_count)?;
            writeln!(writer)?;

            // Format and display result
            if result.row_count > 0 {
                let formatted = format_as_table(&result)?;
                writeln!(writer, "{}", formatted)?;
            }

            // Footer with timing
            writeln!(writer)?;
            writeln!(
                writer,
                "{} rows sampled from {} (Query time: {:.3}s)",
                result.row_count,
                table_name,
                elapsed.as_secs_f64()
            )?;
        }
        Err(e) => {
            let error_msg = e.to_string();
            handle_sample_error(&error_msg, &qualified_name, writer)?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Execute the /peek metacommand (Sprint 33)
///
/// Shows first 5 rows and column metadata for quick table inspection.
///
/// Usage: /peek <table>
fn execute_peek<W: Write>(
    completion_state: &mut CompletionState,
    args: &str,
    writer: &mut W,
) -> Result<()> {
    let table_name = args.trim();
    if table_name.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "Usage: /peek <table>")?;
        writeln!(writer)?;
        writeln!(writer, "Show first 5 rows and column info for quick table inspection.")?;
        writeln!(writer)?;
        writeln!(writer, "Examples:")?;
        writeln!(writer, "  /peek employees        Preview employees table")?;
        writeln!(writer, "  /peek prod.items       Preview table in different database")?;
        writeln!(writer)?;
        return Ok(());
    }

    // Resolve qualified table name
    let qualified_name = resolve_table_name(table_name, completion_state);

    writeln!(writer)?;
    writeln!(writer, "Table: {}", qualified_name)?;
    writeln!(writer)?;

    let client = completion_state.client();

    // First, get column metadata using the same approach as /describe
    let (database, table) = parse_qualified_name(&qualified_name);

    let columns_sql = if let Some(db) = database {
        format!(
            r#"SELECT TRIM(ColumnName) AS column_name,
                      TRIM(ColumnType) AS column_type,
                      Nullable,
                      ColumnLength,
                      DecimalTotalDigits,
                      DecimalFractionalDigits
               FROM DBC.ColumnsV
               WHERE DatabaseName = '{}'
                 AND TableName = '{}'
               ORDER BY ColumnId"#,
            escape_sql_string(db),
            escape_sql_string(table)
        )
    } else {
        format!(
            r#"SELECT TRIM(ColumnName) AS column_name,
                      TRIM(ColumnType) AS column_type,
                      Nullable,
                      ColumnLength,
                      DecimalTotalDigits,
                      DecimalFractionalDigits
               FROM DBC.ColumnsV
               WHERE TableName = '{}'
                 AND DatabaseName = DATABASE
               ORDER BY ColumnId"#,
            escape_sql_string(table)
        )
    };

    // Display column information
    writeln!(writer, "Column Information:")?;
    match client.execute(&columns_sql) {
        Ok(columns_result) => {
            if columns_result.row_count == 0 {
                writeln!(writer, "  (no column information available)")?;
            } else {
                writeln!(writer, "{:<25} {:<20} {:<10} {:<15}", "Column", "Type", "Nullable", "Size")?;
                writeln!(writer, "{}", "-".repeat(70))?;

                for row in &columns_result.rows {
                    let col_name = row.first().map(|v| v.display()).unwrap_or_default();
                    let col_type = row.get(1).map(|v| v.display()).unwrap_or_default();
                    let nullable = row.get(2)
                        .map(|v| format_nullable(&v.display()))
                        .unwrap_or_else(|| "YES".to_string());
                    let col_length = row.get(3).map(|v| v.display()).unwrap_or_default();
                    let size_str = if col_length == "[NULL]" || col_length.is_empty() {
                        "-".to_string()
                    } else {
                        col_length
                    };

                    writeln!(
                        writer,
                        "{:<25} {:<20} {:<10} {:<15}",
                        truncate_str(&col_name, 24),
                        truncate_str(&col_type, 19),
                        nullable,
                        truncate_str(&size_str, 14)
                    )?;
                }
            }
        }
        Err(e) => {
            writeln!(writer, "  (error loading column info: {})", e)?;
        }
    }

    writeln!(writer)?;

    // Now fetch first 5 rows using TOP with properly quoted identifier
    let quoted_name = quote_table_reference(&qualified_name);
    let data_sql = format!("SELECT TOP 5 * FROM {}", quoted_name);
    let start = Instant::now();

    match client.execute(&data_sql) {
        Ok(result) => {
            let elapsed = start.elapsed();

            if result.row_count == 0 {
                writeln!(writer, "Table is empty (0 rows)")?;
            } else {
                writeln!(writer, "First {} rows:", result.row_count)?;
                writeln!(writer)?;
                let formatted = format_as_table(&result)?;
                writeln!(writer, "{}", formatted)?;
            }

            writeln!(writer)?;
            writeln!(writer, "(Query time: {:.3}s)", elapsed.as_secs_f64())?;
        }
        Err(e) => {
            let error_msg = e.to_string();
            handle_sample_error(&error_msg, &qualified_name, writer)?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

// =============================================================================
// Sprint 36: /repeat Command
// =============================================================================

/// Execute the /repeat metacommand (Sprint 36)
///
/// Re-executes the last SQL statement. Uses the same execution path as normal
/// SQL execution, including default row limiting for SELECT queries.
fn execute_repeat<W: Write>(
    state: &mut ReplState,
    completion_state: &mut CompletionState,
    writer: &mut W,
) -> Result<()> {
    let last_sql = match state.last_sql() {
        Some(sql) => sql.to_string(),
        None => {
            writeln!(writer, "No previous query to repeat.")?;
            return Ok(());
        }
    };

    writeln!(writer)?;
    writeln!(writer, "Repeating: {}", &last_sql)?;

    let default_limit = state.default_limit();
    let client = completion_state.client();

    // Re-execute through the same path as normal SQL execution
    match execute_sql_with_state(client, state, &last_sql, writer, default_limit) {
        Ok(row_count) => {
            state.record_query(row_count);
        }
        Err(e) => {
            writeln!(writer, "\nError: {}", e)?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

// =============================================================================
// Sprint 37: /edit Command - External Editor Integration
// =============================================================================

/// Resolve the editor to use for the /edit command
///
/// Priority: $VISUAL -> $EDITOR -> "vi" (fallback)
///
/// Returns the editor command string. The fallback to "vi" ensures the command
/// works on all UNIX-like systems without explicit configuration.
fn resolve_editor() -> std::result::Result<String, String> {
    if let Ok(visual) = std::env::var("VISUAL") {
        if !visual.trim().is_empty() {
            return Ok(visual);
        }
    }

    if let Ok(editor) = std::env::var("EDITOR") {
        if !editor.trim().is_empty() {
            return Ok(editor);
        }
    }

    // Fallback to vi (available on all UNIX-like systems)
    Ok("vi".to_string())
}

/// Create a temporary file with .sql extension for editor integration
///
/// Uses the `tempfile` crate to create a secure temporary file with a
/// descriptive prefix and `.sql` extension for proper syntax highlighting.
fn create_temp_sql_file(content: &str) -> Result<(tempfile::NamedTempFile, std::path::PathBuf)> {
    let temp_file = tempfile::Builder::new()
        .prefix("tq_edit_")
        .suffix(".sql")
        .tempfile()
        .map_err(|e| {
            std::io::Error::other(format!("Failed to create temp file: {}", e))
        })?;

    let path = temp_file.path().to_path_buf();
    std::fs::write(&path, content).map_err(|e| {
        std::io::Error::other(format!("Failed to write temp file: {}", e))
    })?;

    Ok((temp_file, path))
}

/// Launch an editor as a blocking subprocess and wait for exit
///
/// Returns Ok(()) if the editor exits successfully (exit code 0).
/// Returns an error message string if the editor fails to launch or exits
/// with a non-zero status.
fn launch_editor(editor: &str, file_path: &Path) -> std::result::Result<(), String> {
    let status = Command::new(editor)
        .arg(file_path)
        .status()
        .map_err(|e| format!("Failed to launch editor '{}': {}", editor, e))?;

    if !status.success() {
        return Err(format!(
            "Editor '{}' exited with non-zero status: {}",
            editor,
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

/// Check if edited content differs from original
///
/// Comparison is performed on trimmed content so that trailing whitespace
/// or newlines added by the editor do not count as a change.
fn content_changed(original: &str, edited: &str) -> bool {
    original.trim() != edited.trim()
}

/// Execute the /edit metacommand (Sprint 37)
///
/// Opens the last SQL query in an external editor, then executes
/// the modified query if changes were made. Follows the same execution
/// path as `/repeat` for consistency.
fn execute_edit<W: Write>(
    state: &mut ReplState,
    completion_state: &mut CompletionState,
    writer: &mut W,
) -> Result<()> {
    // 1. Check if there's a previous query
    let original_sql = match state.last_sql() {
        Some(s) => s.to_string(),
        None => {
            writeln!(writer, "No previous query to edit.")?;
            return Ok(());
        }
    };

    // 2. Resolve editor
    let editor = match resolve_editor() {
        Ok(e) => e,
        Err(e) => {
            writeln!(writer, "Error: {}", e)?;
            writeln!(writer, "Set $EDITOR or $VISUAL environment variable.")?;
            return Ok(());
        }
    };

    // 3. Create temp file with original SQL
    let (_temp_file, temp_path) = create_temp_sql_file(&original_sql)?;

    // 4. Launch editor
    writeln!(writer, "Opening editor: {}", editor)?;
    if let Err(e) = launch_editor(&editor, &temp_path) {
        writeln!(writer, "Error: {}", e)?;
        return Ok(());
    }

    // 5. Read edited content
    let edited_sql = std::fs::read_to_string(&temp_path).map_err(|e| {
        std::io::Error::other(format!("Failed to read edited file: {}", e))
    })?;

    // 6. Check if content changed
    if !content_changed(&original_sql, &edited_sql) {
        writeln!(writer, "No changes made. Query not executed.")?;
        return Ok(());
    }

    // 7. Check if result is empty
    let trimmed = edited_sql.trim();
    if trimmed.is_empty() {
        writeln!(writer, "No changes made. Query not executed.")?;
        return Ok(());
    }

    // 8. Execute the edited query
    writeln!(writer)?;
    writeln!(writer, "Executing edited query:")?;
    writeln!(writer, "{}", trimmed)?;
    writeln!(writer)?;

    let default_limit = state.default_limit();
    let client = completion_state.client();

    match execute_sql_with_state(client, state, trimmed, writer, default_limit) {
        Ok(row_count) => {
            // Store edited query as new last_sql (enables /repeat after /edit)
            state.set_last_query(trimmed.to_string(), default_limit > 0);
            state.record_query(row_count);
        }
        Err(e) => {
            writeln!(writer, "\nError: {}", e)?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

// Old build_show_indexes_sql and execute_show_indexes removed —
// REPL /show indexes now delegates to crate::commands::show_indexes::execute_for_repl.

/// Resolve a table name to fully qualified form (database.table)
///
/// If the table name already contains a dot, it's returned as-is.
/// Otherwise, uses the current database from connection state.
/// This returns the unquoted name for display purposes.
fn resolve_table_name(name: &str, state: &CompletionState) -> String {
    if name.contains('.') {
        // Already qualified
        name.to_string()
    } else {
        // Use current database
        let current_db = state.current_database();
        if current_db.is_empty() {
            // No current database, return unqualified (query will use session default)
            name.to_string()
        } else {
            format!("{}.{}", current_db, name)
        }
    }
}

/// Quote a table reference for safe use in SQL queries
///
/// Handles both qualified (database.table) and unqualified (table) names.
/// Uses double-quote quoting following ANSI SQL standards.
///
/// Sprint 34: Added for security hardening of data sampling commands
fn quote_table_reference(qualified_name: &str) -> String {
    if let Some(dot_pos) = qualified_name.find('.') {
        let db = &qualified_name[..dot_pos];
        let table = &qualified_name[dot_pos + 1..];
        quote_qualified_name(db, table)
    } else {
        // Unqualified name - just quote the table
        crate::sql::quote_identifier(qualified_name)
    }
}

/// Parse a qualified name into (database, table) parts
fn parse_qualified_name(name: &str) -> (Option<&str>, &str) {
    if let Some(dot_pos) = name.find('.') {
        let db = &name[..dot_pos];
        let tbl = &name[dot_pos + 1..];
        (Some(db), tbl)
    } else {
        (None, name)
    }
}

/// Handle errors from sample/peek commands with user-friendly messages
fn handle_sample_error<W: Write>(
    error_msg: &str,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let error_upper = error_msg.to_uppercase();

    if error_upper.contains("3807") || error_upper.contains("OBJECT") && error_upper.contains("NOT EXIST") {
        // Table not found error
        writeln!(writer, "Error: Table '{}' not found.", table_name)?;
        writeln!(writer)?;
        writeln!(writer, "Suggestions:")?;
        writeln!(writer, "  - Check the table name spelling")?;
        writeln!(writer, "  - Use /list tables to see available tables")?;
        writeln!(writer, "  - Try using qualified name: /sample database.{}",
                 table_name.split('.').next_back().unwrap_or(table_name))?;
    } else if error_upper.contains("3523") || error_upper.contains("PRIVILEGE") {
        // Permission denied error
        writeln!(writer, "Error: Permission denied on table '{}'.", table_name)?;
        writeln!(writer)?;
        writeln!(writer, "You need SELECT privilege on this table.")?;
        writeln!(writer, "Contact your DBA or use:")?;
        writeln!(writer, "  GRANT SELECT ON {} TO <your_username>;", table_name)?;
    } else {
        // Generic error
        writeln!(writer, "Error: {}", error_msg)?;
    }

    Ok(())
}

/// Handle the /params metacommand (Sprint 40)
///
/// Provides runtime parameter file management in the REPL:
/// - `/params load <file>` - Load a YAML parameter file
/// - `/params unload` - Clear all loaded parameters
/// - `/params show` - Display currently loaded parameters and variables
/// - `/params` (no args) - Show usage help
fn handle_params_basic<W: Write>(
    args: &[&str],
    state: &mut ReplState,
    writer: &mut W,
) -> Result<()> {
    match args.first().copied() {
        Some("load") => {
            let path_str = match args.get(1) {
                Some(p) => *p,
                None => {
                    writeln!(writer, "Usage: /params load <file.yaml>")?;
                    return Ok(());
                }
            };
            let path = Path::new(path_str);
            match state.params.load_file(path) {
                Ok(()) => {
                    writeln!(writer, "Loaded parameters from '{}'", path_str)?;
                    let paths = state.params.list_available_paths();
                    if !paths.is_empty() {
                        writeln!(writer, "Variables available: {}", paths.len())?;
                    }
                }
                Err(e) => {
                    writeln!(writer, "{}", e)?;
                }
            }
        }
        Some("unload") => {
            state.params.clear();
            writeln!(writer, "All parameters cleared.")?;
        }
        Some("show") => {
            if state.params.is_empty() {
                writeln!(writer, "No parameters loaded.")?;
            } else {
                writeln!(writer, "Loaded files:")?;
                for f in state.params.loaded_files() {
                    writeln!(writer, "  {}", f.display())?;
                }
                writeln!(writer)?;
                let vars = state.params.list_variables();
                if vars.is_empty() {
                    writeln!(writer, "No variables defined (files may be empty).")?;
                } else {
                    writeln!(writer, "Available variables:")?;
                    for (path, value) in &vars {
                        // Truncate long values for display (UTF-8 safe)
                        let display_value = if value.len() > 60 {
                            let truncated: String = value.chars().take(57).collect();
                            format!("{}...", truncated)
                        } else {
                            value.clone()
                        };
                        writeln!(writer, "  {{{{{}}}}} = {}", path, display_value)?;
                    }
                }
            }
        }
        Some(other) => {
            writeln!(writer, "Unknown /params subcommand: {}", other)?;
            writeln!(writer, "Usage: /params [load <file> | unload | show]")?;
        }
        None => {
            writeln!(writer)?;
            writeln!(writer, "Usage: /params <subcommand>")?;
            writeln!(writer)?;
            writeln!(writer, "Subcommands:")?;
            writeln!(
                writer,
                "  load <file>    Load a YAML parameter file"
            )?;
            writeln!(
                writer,
                "  unload         Clear all loaded parameters"
            )?;
            writeln!(
                writer,
                "  show           Show currently loaded parameters"
            )?;
            writeln!(writer)?;
            writeln!(writer, "Examples:")?;
            writeln!(writer, "  /params load params.yaml")?;
            writeln!(writer, "  /params show")?;
            writeln!(writer, "  /params unload")?;
            writeln!(writer)?;
            writeln!(
                writer,
                "SQL variables use {{{{key}}}} syntax. See 'tq help params' for details."
            )?;
            writeln!(writer)?;
        }
    }
    Ok(())
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

    #[test]
    fn test_help_includes_new_commands() {
        let mut output = Vec::new();
        print_help(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // Verify Sprint 4 commands are documented
        assert!(output_str.contains("/ping"));
        assert!(output_str.contains("/describe"));
    }

    // Note: escape_sql_string tests are in src/sql/identifiers.rs

    #[test]
    fn test_format_nullable() {
        assert_eq!(format_nullable("Y"), "YES");
        assert_eq!(format_nullable("N"), "NO");
        assert_eq!(format_nullable("YES"), "YES");
        assert_eq!(format_nullable("NO"), "NO");
        assert_eq!(format_nullable("TRUE"), "YES");
        assert_eq!(format_nullable("FALSE"), "NO");
        assert_eq!(format_nullable("1"), "YES");
        assert_eq!(format_nullable("0"), "NO");
        // Edge case - unknown values pass through
        assert_eq!(format_nullable("UNKNOWN"), "UNKNOWN");
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("short", 10), "short");
        assert_eq!(truncate_str("exactly10c", 10), "exactly10c");
        assert_eq!(truncate_str("this is a long string", 10), "this is...");
        assert_eq!(truncate_str("test", 3), "...");
        assert_eq!(truncate_str("ab", 2), "ab");
    }

    // Sprint 34: Quote table reference tests (updated Sprint 46 for uppercase)
    #[test]
    fn test_quote_table_reference_simple() {
        assert_eq!(quote_table_reference("employees"), "\"EMPLOYEES\"");
    }

    #[test]
    fn test_quote_table_reference_qualified() {
        assert_eq!(
            quote_table_reference("prod.employees"),
            "\"PROD\".\"EMPLOYEES\""
        );
    }

    #[test]
    fn test_quote_table_reference_with_spaces() {
        assert_eq!(
            quote_table_reference("my database.my table"),
            "\"MY DATABASE\".\"MY TABLE\""
        );
    }

    #[test]
    fn test_quote_table_reference_with_quotes() {
        // Edge case: embedded quotes should be escaped
        assert_eq!(
            quote_table_reference("db\"x.tbl\"y"),
            "\"DB\"\"X\".\"TBL\"\"Y\""
        );
    }

    // Sprint 12/13: Export argument parsing tests
    #[test]
    fn test_parse_export_args_clipboard_first_deprecated() {
        // Sprint 13: Old syntax (deprecated but still supported)
        let args = vec!["clipboard", "csv"];
        let (format, dest, append, deprecation) = parse_export_args(&args);
        assert_eq!(format, "csv");
        assert!(matches!(dest, ExportDestination::Clipboard));
        assert!(!append);
        assert!(matches!(deprecation, DeprecationWarning::ClipboardFirst));
    }

    #[test]
    fn test_parse_export_args_clipboard_last_new_syntax() {
        // Sprint 13: New syntax (preferred)
        let args = vec!["json", "clipboard"];
        let (format, dest, append, deprecation) = parse_export_args(&args);
        assert_eq!(format, "json");
        assert!(matches!(dest, ExportDestination::Clipboard));
        assert!(!append);
        assert!(matches!(deprecation, DeprecationWarning::None));
    }

    #[test]
    fn test_parse_export_args_file() {
        let args = vec!["csv", "output.csv"];
        let (format, dest, append, deprecation) = parse_export_args(&args);
        assert_eq!(format, "csv");
        match dest {
            ExportDestination::File(path) => assert_eq!(path, "output.csv"),
            _ => panic!("Expected File destination"),
        }
        assert!(!append);
        assert!(matches!(deprecation, DeprecationWarning::None));
    }

    #[test]
    fn test_parse_export_args_file_with_append_deprecated() {
        // Sprint 13: Append mode is deprecated
        let args = vec!["csv", "--append", "output.csv"];
        let (format, dest, append, deprecation) = parse_export_args(&args);
        assert_eq!(format, "csv");
        match dest {
            ExportDestination::File(path) => assert_eq!(path, "output.csv"),
            _ => panic!("Expected File destination"),
        }
        assert!(append);
        assert!(matches!(deprecation, DeprecationWarning::AppendMode));
    }

    #[test]
    fn test_parse_export_args_default_format_clipboard_deprecated() {
        // Sprint 13: "clipboard" first is deprecated
        let args = vec!["clipboard"];
        let (format, dest, append, deprecation) = parse_export_args(&args);
        assert_eq!(format, "table"); // Default format when only "clipboard" specified
        assert!(matches!(dest, ExportDestination::Clipboard));
        assert!(!append);
        assert!(matches!(deprecation, DeprecationWarning::ClipboardFirst));
    }

    #[test]
    fn test_parse_export_args_format_only() {
        let args = vec!["json"];
        let (format, dest, append, deprecation) = parse_export_args(&args);
        assert_eq!(format, "json");
        assert!(matches!(dest, ExportDestination::Stdout));
        assert!(!append);
        assert!(matches!(deprecation, DeprecationWarning::None));
    }

    #[test]
    fn test_format_as_csv() {
        use crate::db::{ColumnMetadata, QueryResult, TeradataType, Value};

        let result = QueryResult {
            columns: vec![
                ColumnMetadata {
                    name: "id".to_string(),
                    data_type: TeradataType::Integer,
                    nullable: false,
                },
                ColumnMetadata {
                    name: "name".to_string(),
                    data_type: TeradataType::Varchar,
                    nullable: true,
                },
            ],
            rows: vec![
                vec![Value::Integer(1), Value::String("Alice".to_string())],
                vec![Value::Integer(2), Value::String("Bob, Jr.".to_string())],
            ],
            row_count: 2,
            execution_time: Duration::from_millis(10),
        };

        let csv = format_as_csv(&result).unwrap();
        assert!(csv.contains("id,name"));
        assert!(csv.contains("1,Alice"));
        assert!(csv.contains("2,\"Bob, Jr.\""));
    }

    #[test]
    fn test_format_as_json() {
        use crate::db::{ColumnMetadata, QueryResult, TeradataType, Value};

        let result = QueryResult {
            columns: vec![ColumnMetadata {
                name: "id".to_string(),
                data_type: TeradataType::Integer,
                nullable: false,
            }],
            rows: vec![vec![Value::Integer(42)]],
            row_count: 1,
            execution_time: Duration::from_millis(10),
        };

        let json = format_as_json(&result).unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("42"));
    }

    // matches_glob tests moved to list.rs tests

    #[test]
    fn test_help_extended_includes_list_commands() {
        let mut output = Vec::new();
        print_help_extended(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // Verify Sprint 22 /list commands are documented
        assert!(output_str.contains("/list databases"));
        assert!(output_str.contains("/list tables"));
        assert!(output_str.contains("/list views"));
        assert!(output_str.contains("/dt"));
        assert!(output_str.contains("/dv"));
    }

    #[test]
    fn test_help_extended_includes_metacommand_tab_completion() {
        let mut output = Vec::new();
        print_help_extended(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // Verify tab completion for metacommands is documented
        assert!(output_str.contains("Tab after /"));
        assert!(output_str.contains("metacommands"));
    }

    #[test]
    fn test_help_extended_includes_sessions_command() {
        let mut output = Vec::new();
        print_help_extended(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // Verify Sprint 26 /sessions command is documented
        assert!(output_str.contains("/sessions"));
        assert!(output_str.contains("/s"));
        assert!(output_str.contains("System Monitoring"));
        assert!(output_str.contains("active sessions"));
    }

    #[test]
    fn test_help_extended_includes_result_paging_section() {
        let mut output = Vec::new();
        print_help_extended(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // Verify Sprint 28 Result Paging section is documented
        assert!(output_str.contains("Result Paging"));
        assert!(output_str.contains("j/k"));
        assert!(output_str.contains("Space/b"));
        assert!(output_str.contains("Left/Right"));
        assert!(output_str.contains("g/G"));
        assert!(output_str.contains("q or Esc"));
        assert!(output_str.contains("Column indicators"));
    }

    // =========================================================================
    // Sprint 36: /repeat command tests
    // =========================================================================

    #[test]
    fn test_repeat_no_previous_query() {
        let config = create_test_config();
        let state = ReplState::new(config);

        // With no last SQL, repeat should print message
        assert!(state.last_sql().is_none());
    }

    #[test]
    fn test_repeat_has_previous_query() {
        let config = create_test_config();
        let mut state = ReplState::new(config);

        state.set_last_query("SELECT * FROM employees".to_string(), false);
        assert_eq!(state.last_sql(), Some("SELECT * FROM employees"));
    }

    #[test]
    fn test_repeat_via_basic_handler_no_query() {
        let config = create_test_config();
        let mut state = ReplState::new(config);
        let client = DatabaseClient::mock();

        let mut output = Vec::new();
        let result = handle_metacommand("/repeat", &mut state, &client, &mut output);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should continue REPL

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("No previous query to repeat"));
    }

    #[test]
    fn test_repeat_alias_r_via_basic_handler() {
        let config = create_test_config();
        let mut state = ReplState::new(config);
        let client = DatabaseClient::mock();

        let mut output = Vec::new();
        let result = handle_metacommand("/r", &mut state, &client, &mut output);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should continue REPL

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("No previous query to repeat"));
    }

    #[test]
    fn test_repeat_backslash_alias() {
        let config = create_test_config();
        let mut state = ReplState::new(config);
        let client = DatabaseClient::mock();

        let mut output = Vec::new();
        let result = handle_metacommand("\\r", &mut state, &client, &mut output);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("No previous query to repeat"));
    }

    #[test]
    fn test_help_includes_repeat_command() {
        let mut output = Vec::new();
        print_help(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("/repeat"));
        assert!(output_str.contains("/r"));
        assert!(output_str.contains("Re-execute last query"));
    }

    #[test]
    fn test_help_extended_includes_repeat_command() {
        let mut output = Vec::new();
        print_help_extended(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("/repeat"));
        assert!(output_str.contains("/r"));
        assert!(output_str.contains("Re-execute last query"));
    }

    #[test]
    fn test_default_limit_stored_in_state() {
        let config = create_test_config();
        let mut state = ReplState::new(config);

        // Default should be 0 (no limit)
        assert_eq!(state.default_limit(), 0);

        state.set_default_limit(500);
        assert_eq!(state.default_limit(), 500);
    }

    // build_show_indexes_sql tests removed — show_indexes now in batch module

    #[test]
    fn test_parse_qualified_name_simple() {
        let (db, table) = parse_qualified_name("employees");
        assert!(db.is_none());
        assert_eq!(table, "employees");
    }

    #[test]
    fn test_parse_qualified_name_with_database() {
        let (db, table) = parse_qualified_name("prod.orders");
        assert_eq!(db, Some("prod"));
        assert_eq!(table, "orders");
    }

    #[test]
    fn test_help_includes_show_indexes() {
        let mut output = Vec::new();
        print_help(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("/show indexes"));
    }

    #[test]
    fn test_help_extended_includes_show_indexes() {
        let mut output = Vec::new();
        print_help_extended(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("/show indexes"));
        assert!(output_str.contains("/di"));
    }

    #[test]
    fn test_show_via_basic_handler() {
        let config = create_test_config();
        let mut state = ReplState::new(config);
        let client = DatabaseClient::mock();

        let mut output = Vec::new();
        let result = handle_metacommand("/show", &mut state, &client, &mut output);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("full REPL mode"));
    }

    #[test]
    fn test_di_via_basic_handler() {
        let config = create_test_config();
        let mut state = ReplState::new(config);
        let client = DatabaseClient::mock();

        let mut output = Vec::new();
        let result = handle_metacommand("/di", &mut state, &client, &mut output);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("full REPL mode"));
    }

    // =========================================================================
    // Sprint 37: /edit command tests
    // =========================================================================

    // Mutex to serialize tests that modify VISUAL/EDITOR environment variables
    static EDITOR_ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Helper to run a test that modifies VISUAL/EDITOR env vars safely
    fn with_editor_env<F, T>(test_fn: F) -> T
    where
        F: FnOnce() -> T,
    {
        let _lock = EDITOR_ENV_MUTEX.lock().unwrap();
        let orig_visual = std::env::var("VISUAL").ok();
        let orig_editor = std::env::var("EDITOR").ok();

        let result = test_fn();

        // Restore original values
        match orig_visual {
            Some(v) => std::env::set_var("VISUAL", v),
            None => std::env::remove_var("VISUAL"),
        }
        match orig_editor {
            Some(v) => std::env::set_var("EDITOR", v),
            None => std::env::remove_var("EDITOR"),
        }
        result
    }

    #[test]
    fn test_resolve_editor_visual_set() {
        with_editor_env(|| {
            std::env::set_var("VISUAL", "code");
            std::env::set_var("EDITOR", "vim");

            let result = resolve_editor();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "code");
        });
    }

    #[test]
    fn test_resolve_editor_editor_set() {
        with_editor_env(|| {
            std::env::remove_var("VISUAL");
            std::env::set_var("EDITOR", "nano");

            let result = resolve_editor();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "nano");
        });
    }

    #[test]
    fn test_resolve_editor_fallback() {
        with_editor_env(|| {
            std::env::remove_var("VISUAL");
            std::env::remove_var("EDITOR");

            let result = resolve_editor();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "vi");
        });
    }

    #[test]
    fn test_resolve_editor_empty_visual_falls_through() {
        with_editor_env(|| {
            std::env::set_var("VISUAL", "  ");
            std::env::set_var("EDITOR", "emacs");

            let result = resolve_editor();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "emacs");
        });
    }

    #[test]
    fn test_execute_edit_no_previous_query() {
        let config = create_test_config();
        let mut state = ReplState::new(config);
        let client = DatabaseClient::mock();

        let mut output = Vec::new();
        let result = handle_metacommand("/edit", &mut state, &client, &mut output);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("full REPL mode"));
    }

    #[test]
    fn test_execute_edit_basic_mode_error() {
        let config = create_test_config();
        let mut state = ReplState::new(config);
        let client = DatabaseClient::mock();

        let mut output = Vec::new();
        let result = handle_metacommand("/edit", &mut state, &client, &mut output);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let output_str = String::from_utf8(output).unwrap();
        assert!(
            output_str.contains("full REPL mode"),
            "Basic handler should indicate full REPL mode required"
        );
    }

    #[test]
    fn test_edit_alias_e_via_basic_handler() {
        let config = create_test_config();
        let mut state = ReplState::new(config);
        let client = DatabaseClient::mock();

        let mut output = Vec::new();
        let result = handle_metacommand("/e", &mut state, &client, &mut output);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("full REPL mode"));
    }

    #[test]
    fn test_edit_backslash_alias() {
        let config = create_test_config();
        let mut state = ReplState::new(config);
        let client = DatabaseClient::mock();

        let mut output = Vec::new();
        let result = handle_metacommand("\\e", &mut state, &client, &mut output);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("full REPL mode"));
    }

    #[test]
    fn test_edit_command_in_help_text() {
        let mut output = Vec::new();
        print_help(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(
            output_str.contains("/edit"),
            "Help text should include /edit command"
        );
        assert!(
            output_str.contains("/e"),
            "Help text should include /e alias"
        );
    }

    #[test]
    fn test_edit_command_in_help_extended() {
        let mut output = Vec::new();
        print_help_extended(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(
            output_str.contains("/edit"),
            "Extended help text should include /edit command"
        );
        assert!(
            output_str.contains("/e"),
            "Extended help text should include /e alias"
        );
        assert!(
            output_str.contains("$EDITOR"),
            "Extended help text should mention $EDITOR"
        );
    }

    #[test]
    fn test_content_changed_identical() {
        assert!(!content_changed("SELECT 1", "SELECT 1"));
    }

    #[test]
    fn test_content_changed_whitespace_only() {
        assert!(!content_changed("SELECT 1", "  SELECT 1  \n"));
    }

    #[test]
    fn test_content_changed_different() {
        assert!(content_changed("SELECT 1", "SELECT 2"));
    }

    #[test]
    fn test_content_changed_empty_vs_nonempty() {
        assert!(content_changed("SELECT 1", ""));
    }

    #[test]
    fn test_create_temp_sql_file() {
        let content = "SELECT * FROM employees;";
        let result = create_temp_sql_file(content);
        assert!(result.is_ok());

        let (_temp, path) = result.unwrap();
        assert!(path.extension().is_some_and(|ext| ext == "sql"));

        let read_back = std::fs::read_to_string(&path).unwrap();
        assert_eq!(read_back, content);
    }

    #[test]
    fn test_edit_alias_in_metacommands() {
        // Verify that the METACOMMANDS array in metadata_completer.rs
        // includes the edit command - we test this indirectly by checking
        // that the tab completion system recognizes it
        use crate::commands::repl::metadata_completer::complete_metacommands_for_test;

        // This test verifies the edit entry exists in METACOMMANDS
        let suggestions = complete_metacommands_for_test("edit");
        assert!(
            !suggestions.is_empty(),
            "Tab completion should include /edit"
        );

        let suggestions = complete_metacommands_for_test("e");
        let has_edit = suggestions.iter().any(|s| s.contains("edit"));
        assert!(has_edit, "Tab completion for 'e' should include edit");
    }

    // Sprint 45: Semicolon stripping tests (Bug #32)

    #[test]
    fn test_semicolon_stripping_describe() {
        // /describe tablename; should strip the semicolon
        let input = "/describe tablename;";
        let trimmed = input.trim().trim_end_matches(';').trim();
        let without_prefix = trimmed.trim_start_matches('/').trim_start_matches('\\');
        let mut parts = without_prefix.split_whitespace();
        let command = parts.next().unwrap_or("").to_lowercase();
        let args: Vec<&str> = parts.collect();
        assert_eq!(command, "describe");
        assert_eq!(args, vec!["tablename"]);
    }

    #[test]
    fn test_semicolon_stripping_list_tables() {
        // /list tables; should strip the semicolon
        let input = "/list tables;";
        let trimmed = input.trim().trim_end_matches(';').trim();
        let without_prefix = trimmed.trim_start_matches('/').trim_start_matches('\\');
        let mut parts = without_prefix.split_whitespace();
        let command = parts.next().unwrap_or("").to_lowercase();
        let args: Vec<&str> = parts.collect();
        assert_eq!(command, "list");
        assert_eq!(args, vec!["tables"]);
    }

    #[test]
    fn test_semicolon_stripping_no_semicolon() {
        // /describe tablename (no semicolon) should still work
        let input = "/describe tablename";
        let trimmed = input.trim().trim_end_matches(';').trim();
        let without_prefix = trimmed.trim_start_matches('/').trim_start_matches('\\');
        let mut parts = without_prefix.split_whitespace();
        let command = parts.next().unwrap_or("").to_lowercase();
        let args: Vec<&str> = parts.collect();
        assert_eq!(command, "describe");
        assert_eq!(args, vec!["tablename"]);
    }

    #[test]
    fn test_semicolon_stripping_double_semicolons() {
        // /describe a;; should strip both semicolons
        let input = "/describe a;;";
        let trimmed = input.trim().trim_end_matches(';').trim();
        let without_prefix = trimmed.trim_start_matches('/').trim_start_matches('\\');
        let mut parts = without_prefix.split_whitespace();
        let command = parts.next().unwrap_or("").to_lowercase();
        let args: Vec<&str> = parts.collect();
        assert_eq!(command, "describe");
        assert_eq!(args, vec!["a"]);
    }

    #[test]
    fn test_semicolon_stripping_show_indexes() {
        // /show indexes tablename; should strip the semicolon
        let input = "/show indexes tablename;";
        let trimmed = input.trim().trim_end_matches(';').trim();
        let without_prefix = trimmed.trim_start_matches('/').trim_start_matches('\\');
        let mut parts = without_prefix.split_whitespace();
        let command = parts.next().unwrap_or("").to_lowercase();
        let args: Vec<&str> = parts.collect();
        assert_eq!(command, "show");
        assert_eq!(args, vec!["indexes", "tablename"]);
    }

    #[test]
    fn test_semicolon_stripping_sample() {
        // /sample dbc.tables; should strip the semicolon
        let input = "/sample dbc.tables;";
        let trimmed = input.trim().trim_end_matches(';').trim();
        let without_prefix = trimmed.trim_start_matches('/').trim_start_matches('\\');
        let mut parts = without_prefix.split_whitespace();
        let command = parts.next().unwrap_or("").to_lowercase();
        let args: Vec<&str> = parts.collect();
        assert_eq!(command, "sample");
        assert_eq!(args, vec!["dbc.tables"]);
    }

    // Sprint 45: Inspect tab completion test

    #[test]
    fn test_inspect_in_metacommands() {
        use crate::commands::repl::metadata_completer::complete_metacommands_for_test;

        let suggestions = complete_metacommands_for_test("inspect");
        assert!(
            !suggestions.is_empty(),
            "Tab completion should include /inspect"
        );

        let suggestions = complete_metacommands_for_test("i");
        let has_inspect = suggestions.iter().any(|s| s.contains("inspect"));
        assert!(has_inspect, "Tab completion for 'i' should include inspect");
    }
}
