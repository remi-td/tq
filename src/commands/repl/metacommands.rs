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

use super::metadata_completer::CompletionState;
use super::state::ReplState;
use crate::cli::LogonMechanism;
use crate::db::{ConnectionConfig, DatabaseClient};
use crate::error::Result;
use std::io::Write;
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
    // Normalize: remove leading / or \ and lowercase for command matching
    let trimmed = input.trim();
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

        // Describe command (Sprint 4)
        "describe" | "d" => {
            if args.is_empty() {
                writeln!(writer, "Usage: /describe <table_name>")?;
                writeln!(writer, "       /describe <database>.<table_name>")?;
            } else {
                execute_describe(client, args[0], writer)?;
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
    // Normalize: remove leading / or \ and lowercase for command matching
    let trimmed = input.trim();
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

        // Describe command (Sprint 4)
        "describe" | "d" => {
            if args.is_empty() {
                writeln!(writer, "Usage: /describe <table_name>")?;
                writeln!(writer, "       /describe <database>.<table_name>")?;
            } else {
                execute_describe(completion_state.client(), args[0], writer)?;
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
        "sessions" | "s" => {
            crate::commands::sessions::execute_for_repl(completion_state.client(), writer)?;
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
        "  /list databases        List all accessible databases"
    )?;
    writeln!(
        writer,
        "  /list tables [pattern] List tables (optional glob pattern)"
    )?;
    writeln!(writer, "  /list views            List views in current database")?;
    writeln!(writer, "  /dt                    Shortcut for /list tables")?;
    writeln!(writer, "  /dv                    Shortcut for /list views")?;
    writeln!(writer)?;
    writeln!(writer, "System Monitoring:")?;
    writeln!(
        writer,
        "  /sessions, /s          List active sessions with performance metrics"
    )?;
    writeln!(writer)?;
    writeln!(writer, "SQL Execution:")?;
    writeln!(writer, "  Enter SQL statements ending with semicolon (;)")?;
    writeln!(writer, "  Multi-line statements are supported")?;
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

/// Execute the /list metacommand (Sprint 22)
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

    match subcommand.as_str() {
        "databases" | "db" | "dbs" => {
            execute_list_databases(completion_state, writer)?;
        }
        "tables" | "table" | "t" => {
            execute_list_tables(completion_state, pattern, writer)?;
        }
        "views" | "view" | "v" => {
            execute_list_views(completion_state, writer)?;
        }
        _ => {
            writeln!(writer)?;
            writeln!(writer, "Unknown list subcommand: {}", subcommand)?;
            writeln!(writer, "Available: databases, tables, views")?;
            writeln!(writer)?;
        }
    }

    Ok(())
}

/// Execute /list databases
///
/// Lists all accessible databases from DBC.DatabasesV.
fn execute_list_databases<W: Write>(
    completion_state: &mut CompletionState,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    // First try to use cached databases (loaded at startup)
    if completion_state.ensure_databases_loaded() {
        let cache = completion_state.cache();
        if let Some(databases) = cache.get_cached_databases() {
            writeln!(writer, "Databases ({}):", databases.len())?;
            writeln!(writer, "{}", "-".repeat(40))?;

            // Display in columns for better readability
            let col_width = 25;
            let cols = 3;

            for chunk in databases.chunks(cols) {
                let line: Vec<String> = chunk
                    .iter()
                    .map(|db| format!("{:<width$}", db, width = col_width))
                    .collect();
                writeln!(writer, "{}", line.join(""))?;
            }

            writeln!(writer)?;
            writeln!(writer, "{} database(s)", databases.len())?;
            writeln!(writer)?;
            return Ok(());
        }
    }

    // Fallback: query directly
    let client = completion_state.client();
    let sql = r#"
        SELECT TRIM(DatabaseName) AS database_name
        FROM DBC.DatabasesV
        WHERE DatabaseName NOT IN ('All', 'Console', 'Crashdumps',
                                   'dbcmngr', 'Default', 'External_AP',
                                   'EXTUSER', 'LockLogShredder', 'PUBLIC',
                                   'SQLJ', 'Sys_Calendar', 'SysAdmin',
                                   'SYSBAR', 'SYSJDBC', 'SYSLIB', 'SYSSPATIAL',
                                   'SystemFe', 'SYSUDTLIB', 'TD_SERVER_DB',
                                   'TD_SYSFNLIB', 'TD_SYSGPL', 'TD_SYSXML',
                                   'TDMaps', 'TDPUSER', 'TDQCD', 'TDStats',
                                   'tdwm', 'VIEWPOINT')
        ORDER BY DatabaseName
    "#;

    match client.execute(sql) {
        Ok(result) => {
            writeln!(writer, "Databases ({}):", result.row_count)?;
            writeln!(writer, "{}", "-".repeat(40))?;

            let col_width = 25;
            let cols = 3;
            let databases: Vec<String> = result
                .rows
                .iter()
                .filter_map(|row| {
                    row.first().map(|v| {
                        let s = v.display();
                        if s == "[NULL]" {
                            None
                        } else {
                            Some(s)
                        }
                    })
                })
                .flatten()
                .collect();

            for chunk in databases.chunks(cols) {
                let line: Vec<String> = chunk
                    .iter()
                    .map(|db| format!("{:<width$}", db, width = col_width))
                    .collect();
                writeln!(writer, "{}", line.join(""))?;
            }

            writeln!(writer)?;
            writeln!(writer, "{} database(s)", databases.len())?;
        }
        Err(e) => {
            writeln!(writer, "Error listing databases: {}", e)?;
            writeln!(writer)?;
            writeln!(writer, "You may not have permission to query DBC.DatabasesV")?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Execute /list tables [pattern]
///
/// Lists tables in the current database, optionally filtered by a glob pattern.
fn execute_list_tables<W: Write>(
    completion_state: &mut CompletionState,
    pattern: Option<&str>,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    let current_db = completion_state.current_database().to_string();

    // Ensure tables are loaded for current database
    if !completion_state.ensure_tables_loaded() {
        writeln!(writer, "Warning: Could not load table metadata from cache")?;
    }

    // Query tables from DBC.TablesV
    let client = completion_state.client();
    let sql = format!(
        r#"
        SELECT TRIM(TableName) AS table_name,
               TableKind
        FROM DBC.TablesV
        WHERE DatabaseName = '{}'
          AND TableKind IN ('T', 'O')
        ORDER BY TableName
        "#,
        escape_sql_string(&current_db)
    );

    match client.execute(&sql) {
        Ok(result) => {
            // Apply pattern filter if provided
            let tables: Vec<(String, &str)> = result
                .rows
                .iter()
                .filter_map(|row| {
                    let name = row.first().map(|v| v.display())?;
                    let kind = row.get(1).map(|v| v.display()).unwrap_or_default();
                    if name == "[NULL]" {
                        return None;
                    }

                    // Apply glob pattern if provided
                    if let Some(pat) = pattern {
                        if !matches_glob(&name, pat) {
                            return None;
                        }
                    }

                    let kind_str = match kind.as_str() {
                        "T" => "TABLE",
                        "O" => "OBJECT",
                        _ => "TABLE",
                    };
                    Some((name, kind_str))
                })
                .collect();

            let pattern_str = pattern.map(|p| format!(" matching '{}'", p)).unwrap_or_default();
            writeln!(
                writer,
                "Tables in {}{}:",
                current_db,
                pattern_str
            )?;
            writeln!(writer, "{:<40} {:<10}", "Name", "Type")?;
            writeln!(writer, "{}", "-".repeat(50))?;

            for (name, kind) in &tables {
                writeln!(writer, "{:<40} {:<10}", name, kind)?;
            }

            writeln!(writer)?;
            writeln!(writer, "{} table(s)", tables.len())?;
        }
        Err(e) => {
            writeln!(writer, "Error listing tables: {}", e)?;
            writeln!(writer)?;
            writeln!(
                writer,
                "Verify you have SELECT permission on DBC.TablesV"
            )?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Execute /list views
///
/// Lists views in the current database.
fn execute_list_views<W: Write>(
    completion_state: &mut CompletionState,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    let current_db = completion_state.current_database().to_string();
    let client = completion_state.client();

    let sql = format!(
        r#"
        SELECT TRIM(TableName) AS view_name
        FROM DBC.TablesV
        WHERE DatabaseName = '{}'
          AND TableKind = 'V'
        ORDER BY TableName
        "#,
        escape_sql_string(&current_db)
    );

    match client.execute(&sql) {
        Ok(result) => {
            let views: Vec<String> = result
                .rows
                .iter()
                .filter_map(|row| {
                    let name = row.first().map(|v| v.display())?;
                    if name == "[NULL]" {
                        None
                    } else {
                        Some(name)
                    }
                })
                .collect();

            writeln!(writer, "Views in {}:", current_db)?;
            writeln!(writer, "{}", "-".repeat(40))?;

            if views.is_empty() {
                writeln!(writer, "(no views found)")?;
            } else {
                for view in &views {
                    writeln!(writer, "  {}", view)?;
                }
            }

            writeln!(writer)?;
            writeln!(writer, "{} view(s)", views.len())?;
        }
        Err(e) => {
            writeln!(writer, "Error listing views: {}", e)?;
            writeln!(writer)?;
            writeln!(
                writer,
                "Verify you have SELECT permission on DBC.TablesV"
            )?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Simple glob pattern matching
///
/// Supports:
/// - `*` matches any sequence of characters
/// - `?` matches any single character
/// - Case-insensitive matching
fn matches_glob(text: &str, pattern: &str) -> bool {
    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    // Convert glob pattern to regex-like matching
    let mut pattern_chars = pattern_lower.chars().peekable();
    let mut text_chars = text_lower.chars().peekable();

    fn match_recursive(
        pattern: &mut std::iter::Peekable<std::str::Chars>,
        text: &mut std::iter::Peekable<std::str::Chars>,
    ) -> bool {
        loop {
            match (pattern.peek().copied(), text.peek().copied()) {
                (None, None) => return true,
                (None, Some(_)) => return false,
                (Some('*'), _) => {
                    pattern.next();
                    // Try matching rest of pattern at each position
                    if pattern.peek().is_none() {
                        return true; // Trailing * matches everything
                    }
                    // Try matching at current position and all subsequent positions
                    let mut text_clone = text.clone();
                    loop {
                        let mut pattern_clone = pattern.clone();
                        let mut text_try = text_clone.clone();
                        if match_recursive(&mut pattern_clone, &mut text_try) {
                            return true;
                        }
                        if text_clone.next().is_none() {
                            return false;
                        }
                    }
                }
                (Some('?'), Some(_)) => {
                    pattern.next();
                    text.next();
                }
                (Some(p), Some(t)) if p == t => {
                    pattern.next();
                    text.next();
                }
                _ => return false,
            }
        }
    }

    match_recursive(&mut pattern_chars, &mut text_chars)
}

/// Print help text
fn print_help<W: Write>(writer: &mut W) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "tq REPL Commands:")?;
    writeln!(writer, "  /help, /?              Show this help message")?;
    writeln!(writer, "  /quit, /q              Exit the REPL")?;
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
    writeln!(writer)?;
    writeln!(writer, "SQL Execution:")?;
    writeln!(writer, "  Enter SQL statements ending with semicolon (;)")?;
    writeln!(writer, "  Multi-line statements are supported")?;
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

/// Execute the /describe metacommand
///
/// Shows table structure including columns, types, and nullable status.
fn execute_describe<W: Write>(
    client: &DatabaseClient,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    // Parse table name - may be qualified (database.table) or unqualified
    let (database, table) = if let Some(dot_pos) = table_name.find('.') {
        let db = &table_name[..dot_pos];
        let tbl = &table_name[dot_pos + 1..];
        (Some(db), tbl)
    } else {
        (None, table_name)
    };

    // Build the query to fetch column information from DBC.ColumnsV
    let sql = if let Some(db) = database {
        format!(
            r#"SELECT ColumnName, ColumnType, Nullable, DefaultValue, CommentString
               FROM DBC.ColumnsV
               WHERE DatabaseName = '{}'
                 AND TableName = '{}'
               ORDER BY ColumnId"#,
            escape_sql_string(db),
            escape_sql_string(table)
        )
    } else {
        format!(
            r#"SELECT ColumnName, ColumnType, Nullable, DefaultValue, CommentString
               FROM DBC.ColumnsV
               WHERE TableName = '{}'
                 AND DatabaseName = DATABASE
               ORDER BY ColumnId"#,
            escape_sql_string(table)
        )
    };

    // Execute the query
    match client.execute(&sql) {
        Ok(result) => {
            if result.row_count == 0 {
                writeln!(
                    writer,
                    "Table '{}' not found or no columns available.",
                    table_name
                )?;
                writeln!(writer)?;
                writeln!(writer, "Suggestions:")?;
                writeln!(writer, "  - Check the table name spelling")?;
                writeln!(
                    writer,
                    "  - Try using qualified name: /describe database.table"
                )?;
                writeln!(
                    writer,
                    "  - Verify you have SELECT permission on DBC.ColumnsV"
                )?;
            } else {
                // Display table header
                let qualified_name = if let Some(db) = database {
                    format!("{}.{}", db, table)
                } else {
                    table.to_string()
                };
                writeln!(writer, "Table: {}", qualified_name)?;
                writeln!(writer)?;

                // Display columns header
                writeln!(writer, "Columns:")?;
                writeln!(
                    writer,
                    "{:<25} {:<20} {:<10} {:<15}",
                    "Column", "Type", "Nullable", "Default"
                )?;
                writeln!(writer, "{}", "-".repeat(70))?;

                // Display each column
                for row in &result.rows {
                    let col_name = row.first().map(|v| v.display()).unwrap_or_default();
                    let col_type = row.get(1).map(|v| v.display()).unwrap_or_default();
                    let nullable = row
                        .get(2)
                        .map(|v| format_nullable(&v.display()))
                        .unwrap_or_else(|| "YES".to_string());
                    let default = row
                        .get(3)
                        .map(|v| {
                            let s = v.display();
                            if s == "[NULL]" {
                                "-".to_string()
                            } else {
                                s
                            }
                        })
                        .unwrap_or_else(|| "-".to_string());

                    writeln!(
                        writer,
                        "{:<25} {:<20} {:<10} {:<15}",
                        truncate_string(&col_name, 24),
                        truncate_string(&col_type, 19),
                        nullable,
                        truncate_string(&default, 14)
                    )?;
                }

                writeln!(writer)?;
                writeln!(writer, "{} column(s)", result.row_count)?;
            }
        }
        Err(e) => {
            writeln!(writer, "Error describing table '{}': {}", table_name, e)?;
            writeln!(writer)?;
            writeln!(writer, "Suggestions:")?;
            writeln!(writer, "  - Check table name spelling")?;
            writeln!(
                writer,
                "  - Verify you have permission to access DBC.ColumnsV"
            )?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Escape single quotes in SQL strings to prevent injection
fn escape_sql_string(s: &str) -> String {
    s.replace('\'', "''")
}

/// Format nullable indicator
fn format_nullable(s: &str) -> String {
    match s.trim().to_uppercase().as_str() {
        "Y" | "YES" | "TRUE" | "1" => "YES".to_string(),
        "N" | "NO" | "FALSE" | "0" => "NO".to_string(),
        _ => s.to_string(),
    }
}

/// Truncate string to a maximum length with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        ".".repeat(max_len)
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

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
    if args.iter().any(|&arg| arg == "--append") {
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

/// Format result as table string
fn format_as_table(result: &crate::db::QueryResult) -> Result<String> {
    use comfy_table::{presets, ContentArrangement, Table};

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Add header
    let headers: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
    table.set_header(&headers);

    // Add rows
    for row in &result.rows {
        let values: Vec<String> = row.iter().map(|v| v.display().to_string()).collect();
        table.add_row(values);
    }

    Ok(table.to_string())
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

    #[test]
    fn test_escape_sql_string() {
        assert_eq!(escape_sql_string("test"), "test");
        assert_eq!(escape_sql_string("test's"), "test''s");
        assert_eq!(escape_sql_string("it's a 'test'"), "it''s a ''test''");
    }

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
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("exactly10c", 10), "exactly10c");
        assert_eq!(truncate_string("this is a long string", 10), "this is...");
        assert_eq!(truncate_string("test", 3), "...");
        assert_eq!(truncate_string("ab", 2), "ab");
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

    // Sprint 22: Tests for glob pattern matching

    #[test]
    fn test_matches_glob_exact() {
        assert!(matches_glob("orders", "orders"));
        assert!(!matches_glob("orders", "customers"));
    }

    #[test]
    fn test_matches_glob_case_insensitive() {
        assert!(matches_glob("ORDERS", "orders"));
        assert!(matches_glob("orders", "ORDERS"));
        assert!(matches_glob("Orders", "orders"));
    }

    #[test]
    fn test_matches_glob_star_prefix() {
        assert!(matches_glob("order_items", "*items"));
        assert!(matches_glob("line_items", "*items"));
        assert!(!matches_glob("items_archive", "*items"));
    }

    #[test]
    fn test_matches_glob_star_suffix() {
        assert!(matches_glob("order_items", "order*"));
        assert!(matches_glob("orders", "order*"));
        assert!(!matches_glob("new_orders", "order*"));
    }

    #[test]
    fn test_matches_glob_star_middle() {
        assert!(matches_glob("order_items", "order*items"));
        assert!(matches_glob("order_line_items", "order*items"));
        assert!(!matches_glob("orders_archive", "order*items"));
    }

    #[test]
    fn test_matches_glob_star_only() {
        assert!(matches_glob("anything", "*"));
        assert!(matches_glob("", "*"));
    }

    #[test]
    fn test_matches_glob_question_mark() {
        assert!(matches_glob("order1", "order?"));
        assert!(matches_glob("orders", "order?"));
        assert!(!matches_glob("orders123", "order?"));
    }

    #[test]
    fn test_matches_glob_complex() {
        assert!(matches_glob("order_items_2024", "order*items*"));
        assert!(matches_glob("customer_orders", "*order*"));
        assert!(matches_glob("t1", "t?"));
    }

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
}
