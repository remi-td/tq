//! Metacommand handling for the REPL
//!
//! Metacommands start with '/' or '\' and provide non-SQL functionality
//! like session management, help, and REPL control.
//!
//! Sprint 4 additions:
//! - /describe <table> - Show table structure (columns, types, nullable)
//! - /ping - Test connection within REPL with latency display

use super::state::ReplState;
use crate::db::DatabaseClient;
use crate::error::Result;
use std::io::Write;
use std::time::Instant;

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
    let without_prefix = trimmed
        .trim_start_matches('/')
        .trim_start_matches('\\');

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
                execute_describe(client, &args[0], writer)?;
            }
        }

        // Export command (Sprint 6)
        "export" => {
            if args.is_empty() {
                writeln!(writer, "Usage: /export <format> [file]")?;
                writeln!(writer, "       /export <format> --append [file]")?;
                writeln!(writer)?;
                writeln!(writer, "Formats: csv, json, sql")?;
                writeln!(writer, "Example: /export csv results.csv")?;
            } else {
                let format = args[0];
                let file = if args.len() > 1 {
                    Some(args[1])
                } else {
                    None
                };
                let append = args.contains(&"--append");
                execute_export(state, writer, format, file, append)?;
            }
        }

        // Pager control command (Sprint 6)
        "pager" => {
            if args.is_empty() {
                // Show current setting
                let status = if state.is_pager_enabled() { "on" } else { "off" };
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
                let status = if state.are_colors_enabled() { "on" } else { "off" };
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

/// Print help text
fn print_help<W: Write>(writer: &mut W) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "tq REPL Commands:")?;
    writeln!(writer, "  /help, /?              Show this help message")?;
    writeln!(writer, "  /quit, /q              Exit the REPL")?;
    writeln!(writer, "  /session               Show current session information")?;
    writeln!(writer, "  /ping                  Test database connection")?;
    writeln!(writer, "  /describe <table>, /d  Show table structure")?;
    writeln!(writer, "  /export <fmt> [file]   Export last result (csv, json, sql)")?;
    writeln!(writer, "  /pager on|off          Enable/disable result paging")?;
    writeln!(writer, "  /colors on|off         Enable/disable syntax highlighting")?;
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
fn execute_ping<W: Write>(client: &DatabaseClient, state: &ReplState, writer: &mut W) -> Result<()> {
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
fn execute_describe<W: Write>(client: &DatabaseClient, table_name: &str, writer: &mut W) -> Result<()> {
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
                writeln!(writer, "Table '{}' not found or no columns available.", table_name)?;
                writeln!(writer)?;
                writeln!(writer, "Suggestions:")?;
                writeln!(writer, "  - Check the table name spelling")?;
                writeln!(writer, "  - Try using qualified name: /describe database.table")?;
                writeln!(writer, "  - Verify you have SELECT permission on DBC.ColumnsV")?;
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
                    let col_name = row.get(0).map(|v| v.display()).unwrap_or_default();
                    let col_type = row.get(1).map(|v| v.display()).unwrap_or_default();
                    let nullable = row.get(2).map(|v| format_nullable(&v.display())).unwrap_or_else(|| "YES".to_string());
                    let default = row.get(3).map(|v| {
                        let s = v.display();
                        if s == "[NULL]" { "-".to_string() } else { s }
                    }).unwrap_or_else(|| "-".to_string());

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
            writeln!(writer, "  - Verify you have permission to access DBC.ColumnsV")?;
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

/// Execute the /export metacommand (Sprint 6)
///
/// Exports the last query result to a file in various formats (CSV, JSON, SQL).
fn execute_export<W: Write>(
    state: &ReplState,
    writer: &mut W,
    format: &str,
    file: Option<&str>,
    _append: bool,
) -> Result<()> {
    // Check if we have a result to export
    let result = match state.last_result() {
        Some(r) => r,
        None => {
            writeln!(writer)?;
            writeln!(writer, "Error: No query results to export.")?;
            writeln!(writer, "Execute a query first, then use /export to save the results.")?;
            writeln!(writer)?;
            return Ok(());
        }
    };

    // Validate format
    let format_lower = format.to_lowercase();
    if !["csv", "json", "sql"].contains(&format_lower.as_str()) {
        writeln!(writer)?;
        writeln!(
            writer,
            "Error: Unknown format '{}'. Supported formats: csv, json, sql",
            format
        )?;
        writeln!(writer)?;
        return Ok(());
    }

    // Export based on format
    match format_lower.as_str() {
        "csv" => {
            export_csv(result, file, writer)?;
        }
        "json" => {
            export_json(result, file, writer)?;
        }
        "sql" => {
            export_sql(result, file, writer)?;
        }
        _ => unreachable!(),
    }

    writeln!(writer)?;
    Ok(())
}

/// Export results as CSV
fn export_csv<W: Write>(
    result: &crate::db::QueryResult,
    file: Option<&str>,
    writer: &mut W,
) -> Result<()> {
    use std::fs::File;
    use std::io::BufWriter;

    // Build CSV content
    let mut csv_content = String::new();

    // Add headers
    let headers: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
    csv_content.push_str(&headers.join(","));
    csv_content.push('\n');

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
        csv_content.push_str(&values.join(","));
        csv_content.push('\n');
    }

    // Write to file or stdout
    if let Some(filepath) = file {
        match File::create(filepath) {
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
    writer: &mut W,
) -> Result<()> {
    use std::fs::File;
    use std::io::BufWriter;

    // Build JSON array
    let mut rows = Vec::new();

    for row in &result.rows {
        let mut obj = serde_json::json!({});
        for (i, col) in result.columns.iter().enumerate() {
            let value = &row[i];
            let json_value = match value {
                crate::db::Value::Null => serde_json::Value::Null,
                crate::db::Value::String(s) => serde_json::Value::String(s.clone()),
                crate::db::Value::Integer(n) => serde_json::Value::Number((*n).into()),
                crate::db::Value::Decimal(f) => {
                    serde_json::Number::from_f64(*f)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null)
                }
                crate::db::Value::Boolean(b) => serde_json::Value::Bool(*b),
                crate::db::Value::Date(d) => serde_json::Value::String(d.clone()),
                crate::db::Value::Timestamp(ts) => serde_json::Value::String(ts.clone()),
                crate::db::Value::Time(t) => serde_json::Value::String(t.clone()),
                crate::db::Value::Bytes(_) => serde_json::Value::String(value.display().to_string()),
            };
            obj[&col.name] = json_value;
        }
        rows.push(obj);
    }

    let json_array = serde_json::Value::Array(rows);
    let json_str = serde_json::to_string_pretty(&json_array)?;

    if let Some(filepath) = file {
        match File::create(filepath) {
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
    writer: &mut W,
) -> Result<()> {
    use std::fs::File;
    use std::io::BufWriter;

    // For now, use a generic table name
    // In the future, we could parse this from the last query
    let table_name = "exported_data";

    let mut sql_content = String::new();

    // Add CREATE TABLE statement
    sql_content.push_str(&format!("-- Exported data\n"));
    sql_content.push_str(&format!(
        "-- CREATE TABLE {} (\n",
        table_name
    ));
    for col in &result.columns {
        sql_content.push_str(&format!("--   {} VARCHAR(255),\n", col.name));
    }
    sql_content.push_str(&format!("-- );\n\n"));

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
        sql_content.push_str(&format!(
            "INSERT INTO {} ({}) VALUES ({});\n",
            table_name,
            col_names.join(", "),
            values.join(", ")
        ));
    }

    if let Some(filepath) = file {
        match File::create(filepath) {
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
}
