//! Query inspection command implementation
//!
//! This module provides functionality to inspect recent SQL queries for a
//! given session by querying DBC.QryLogV (DBQL).
//!
//! Sprint 39: Initial implementation

use crate::cli::{OutputFormat, QueryInspectArgs};
use crate::commands::monitoring_utils::{escape_csv, extract_integer, extract_trimmed_string};
use crate::db::{DatabaseClient, Value};
use crate::error::Result;
use std::io::Write;

/// Maximum number of recent queries to display
const MAX_QUERIES: usize = 5;

/// Maximum SQL text length for table display (truncated with ellipsis)
const TABLE_SQL_MAX_LEN: usize = 200;

/// SQL query template to retrieve recent queries from DBC.QryLogV
///
/// Retrieves the most recent queries for a given session ID,
/// including SQL text, timing, and status information.
fn build_query_sql(session_id: i64) -> String {
    format!(
        r#"SELECT TOP {max}
    SessionID,
    CAST(QueryText AS VARCHAR(10000)) AS QueryText,
    CAST(StartTime AS VARCHAR(30)) AS StartTime,
    CAST(TotalElapsedTime AS VARCHAR(30)) AS TotalElapsedTime,
    CASE
        WHEN AbortFlag = 'Y' THEN 'Aborted'
        WHEN ErrorCode <> 0 THEN 'Error'
        ELSE 'Complete'
    END AS QueryStatus
FROM DBC.QryLogV
WHERE SessionID = {session_id}
  AND CollectTimeStamp >= CURRENT_TIMESTAMP - INTERVAL '1' DAY
ORDER BY CollectTimeStamp DESC"#,
        max = MAX_QUERIES,
        session_id = session_id
    )
}

/// Query information extracted from DBC.QryLogV
#[derive(Debug, Clone)]
pub struct QueryInfo {
    /// Session ID that ran the query
    pub session_id: i64,
    /// SQL text of the query
    pub query_text: String,
    /// Query start time (formatted)
    pub start_time: String,
    /// Total elapsed time (formatted)
    pub total_elapsed: String,
    /// Query status (Complete, Active, Aborted, Error)
    pub status: String,
}

impl QueryInfo {
    /// Create QueryInfo from a query result row
    ///
    /// Returns None if required fields are missing or cannot be parsed.
    pub fn from_row(row: &[Value]) -> Option<Self> {
        if row.len() < 5 {
            return None;
        }

        let session_id = extract_integer(&row[0])?;
        let query_text = extract_trimmed_string(&row[1], "");
        let start_time = extract_trimmed_string(&row[2], "[unknown]");
        let total_elapsed = extract_trimmed_string(&row[3], "[unknown]");
        let status = extract_trimmed_string(&row[4], "Unknown");

        Some(Self {
            session_id,
            query_text,
            start_time,
            total_elapsed,
            status,
        })
    }
}

/// Execute the query-inspect command and write results (batch mode)
///
/// # Arguments
/// * `client` - Database client for executing queries
/// * `args` - Command arguments (session_id, format, output file)
/// * `writer` - Output writer
/// * `_use_color` - Whether to use color output
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &QueryInspectArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    let sql = build_query_sql(args.session_id);
    let result = client.execute(&sql)?;

    let queries: Vec<QueryInfo> = result
        .rows
        .iter()
        .filter_map(|row| QueryInfo::from_row(row))
        .collect();

    if queries.is_empty() {
        writeln!(
            writer,
            "No queries found for session {}.",
            args.session_id
        )?;
        return Ok(());
    }

    match args.format {
        OutputFormat::Table => display_table(&queries, args.session_id, writer)?,
        OutputFormat::Csv => display_csv(&queries, writer)?,
        OutputFormat::Json => display_json(&queries, writer)?,
    }

    Ok(())
}

/// Execute query inspection and display for REPL mode
///
/// Displays recent queries for the given session ID with error handling
/// for privilege errors and DBQL unavailability.
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    session_id: i64,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer)?;

    let sql = build_query_sql(session_id);

    match client.execute(&sql) {
        Ok(result) => {
            let queries: Vec<QueryInfo> = result
                .rows
                .iter()
                .filter_map(|row| QueryInfo::from_row(row))
                .collect();

            if queries.is_empty() {
                writeln!(writer, "No queries found for session {}.", session_id)?;
                writeln!(writer)?;
                writeln!(
                    writer,
                    "(Query time: {:.3}s)",
                    result.execution_time.as_secs_f64()
                )?;
            } else {
                display_repl_table(&queries, session_id, writer)?;
                writeln!(writer)?;
                writeln!(
                    writer,
                    "{} recent query(ies) for session {} (Query time: {:.3}s)",
                    queries.len(),
                    session_id,
                    result.execution_time.as_secs_f64()
                )?;
            }
        }
        Err(e) => {
            let error_str = e.to_string().to_lowercase();

            if error_str.contains("privilege")
                || error_str.contains("access")
                || error_str.contains("permission")
                || error_str.contains("3523")
            {
                writeln!(writer, "Error: Unable to query DBQL log.")?;
                writeln!(writer)?;
                writeln!(
                    writer,
                    "This command requires SELECT access to DBC.QryLogV."
                )?;
                writeln!(writer)?;
                writeln!(writer, "To grant access, a DBA can run:")?;
                writeln!(
                    writer,
                    "  GRANT SELECT ON DBC.QryLogV TO <your_username>;"
                )?;
            } else if error_str.contains("qrylogv")
                && (error_str.contains("not found") || error_str.contains("does not exist"))
            {
                writeln!(writer, "Error: DBQL query log not available.")?;
                writeln!(writer)?;
                writeln!(
                    writer,
                    "DBC.QryLogV requires DBQL (Database Query Log) to be enabled."
                )?;
                writeln!(writer, "Contact your DBA to enable DBQL logging.")?;
            } else {
                writeln!(writer, "Error inspecting queries: {}", e)?;
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Truncate SQL text for table display
fn truncate_sql(sql: &str, max_len: usize) -> String {
    // Normalize whitespace for display
    let normalized: String = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.len() <= max_len {
        normalized
    } else {
        format!("{}...", &normalized[..max_len.saturating_sub(3)])
    }
}

/// Display queries using comfy_table for REPL mode
fn display_repl_table<W: Write>(
    queries: &[QueryInfo],
    session_id: i64,
    writer: &mut W,
) -> Result<()> {
    use comfy_table::{presets, ContentArrangement, Table};

    writeln!(
        writer,
        "Recent Queries for Session {}:",
        session_id
    )?;
    writeln!(writer)?;

    for (i, query) in queries.iter().enumerate() {
        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL);
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec!["Property", "Value"]);

        table.add_row(vec!["Query #", &(i + 1).to_string()]);
        table.add_row(vec!["Start Time", &query.start_time]);
        table.add_row(vec!["Elapsed Time", &query.total_elapsed]);
        table.add_row(vec!["Status", &query.status]);
        table.add_row(vec![
            "SQL",
            &truncate_sql(&query.query_text, TABLE_SQL_MAX_LEN),
        ]);

        writeln!(writer, "{}", table)?;
        if i < queries.len() - 1 {
            writeln!(writer)?;
        }
    }

    Ok(())
}

/// Display queries in table format (batch mode)
fn display_table<W: Write>(
    queries: &[QueryInfo],
    session_id: i64,
    writer: &mut W,
) -> Result<()> {
    display_repl_table(queries, session_id, writer)
}

/// Display queries in CSV format
fn display_csv<W: Write>(queries: &[QueryInfo], writer: &mut W) -> Result<()> {
    writeln!(
        writer,
        "SessionID,StartTime,ElapsedTime,Status,QueryText"
    )?;

    for query in queries {
        writeln!(
            writer,
            "{},{},{},{},{}",
            query.session_id,
            escape_csv(&query.start_time),
            escape_csv(&query.total_elapsed),
            escape_csv(&query.status),
            escape_csv(&query.query_text)
        )?;
    }

    Ok(())
}

/// Display queries in JSON format
fn display_json<W: Write>(queries: &[QueryInfo], writer: &mut W) -> Result<()> {
    let json_rows: Vec<serde_json::Value> = queries
        .iter()
        .map(|query| {
            serde_json::json!({
                "SessionID": query.session_id,
                "StartTime": query.start_time,
                "ElapsedTime": query.total_elapsed,
                "Status": query.status,
                "QueryText": query.query_text,
            })
        })
        .collect();

    let json_output = serde_json::to_string_pretty(&json_rows)?;
    writeln!(writer, "{}", json_output)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // SQL generation tests
    // =========================================================================

    #[test]
    fn test_build_query_sql_contains_session_id() {
        let sql = build_query_sql(1234);
        assert!(sql.contains("SessionID = 1234"));
    }

    #[test]
    fn test_build_query_sql_contains_top_limit() {
        let sql = build_query_sql(1234);
        assert!(sql.contains(&format!("TOP {}", MAX_QUERIES)));
    }

    #[test]
    fn test_build_query_sql_queries_qrylogv() {
        let sql = build_query_sql(1234);
        assert!(sql.contains("DBC.QryLogV"));
    }

    #[test]
    fn test_build_query_sql_orders_by_collect_timestamp() {
        let sql = build_query_sql(1234);
        assert!(sql.contains("ORDER BY CollectTimeStamp DESC"));
    }

    #[test]
    fn test_build_query_sql_filters_recent() {
        let sql = build_query_sql(1234);
        assert!(sql.contains("INTERVAL '1' DAY"));
    }

    // =========================================================================
    // QueryInfo parsing tests
    // =========================================================================

    #[test]
    fn test_query_info_from_row_complete() {
        let row = vec![
            Value::Integer(1234),
            Value::String("SELECT * FROM employees".to_string()),
            Value::String("2026-02-24 10:30:00".to_string()),
            Value::String("00:00:05.123".to_string()),
            Value::String("Complete".to_string()),
        ];

        let info = QueryInfo::from_row(&row);
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.session_id, 1234);
        assert_eq!(info.query_text, "SELECT * FROM employees");
        assert_eq!(info.start_time, "2026-02-24 10:30:00");
        assert_eq!(info.total_elapsed, "00:00:05.123");
        assert_eq!(info.status, "Complete");
    }

    #[test]
    fn test_query_info_from_row_aborted() {
        let row = vec![
            Value::Integer(1234),
            Value::String("DELETE FROM big_table".to_string()),
            Value::String("2026-02-24 10:30:00".to_string()),
            Value::String("00:05:32.000".to_string()),
            Value::String("Aborted".to_string()),
        ];

        let info = QueryInfo::from_row(&row).unwrap();
        assert_eq!(info.status, "Aborted");
    }

    #[test]
    fn test_query_info_from_row_insufficient_columns() {
        let row = vec![
            Value::Integer(1234),
            Value::String("SELECT 1".to_string()),
        ];

        let info = QueryInfo::from_row(&row);
        assert!(info.is_none());
    }

    #[test]
    fn test_query_info_from_row_null_session() {
        let row = vec![
            Value::Null,
            Value::String("SELECT 1".to_string()),
            Value::String("2026-02-24 10:30:00".to_string()),
            Value::String("00:00:01.000".to_string()),
            Value::String("Complete".to_string()),
        ];

        let info = QueryInfo::from_row(&row);
        // session_id is NULL -> extract_integer returns None -> from_row returns None
        assert!(info.is_none());
    }

    #[test]
    fn test_query_info_from_row_null_query_text() {
        let row = vec![
            Value::Integer(1234),
            Value::Null,
            Value::String("2026-02-24 10:30:00".to_string()),
            Value::String("00:00:01.000".to_string()),
            Value::String("Complete".to_string()),
        ];

        let info = QueryInfo::from_row(&row).unwrap();
        assert_eq!(info.query_text, ""); // null_display is ""
    }

    #[test]
    fn test_query_info_from_row_with_whitespace() {
        let row = vec![
            Value::Integer(1234),
            Value::String("  SELECT 1  ".to_string()),
            Value::String("  2026-02-24 10:30:00  ".to_string()),
            Value::String("  00:00:01.000  ".to_string()),
            Value::String("  Complete  ".to_string()),
        ];

        let info = QueryInfo::from_row(&row).unwrap();
        assert_eq!(info.query_text, "SELECT 1");
        assert_eq!(info.start_time, "2026-02-24 10:30:00");
        assert_eq!(info.total_elapsed, "00:00:01.000");
        assert_eq!(info.status, "Complete");
    }

    #[test]
    fn test_query_info_from_row_decimal_session_id() {
        let row = vec![
            Value::Decimal(1234.0),
            Value::String("SELECT 1".to_string()),
            Value::String("2026-02-24 10:30:00".to_string()),
            Value::String("00:00:01.000".to_string()),
            Value::String("Complete".to_string()),
        ];

        let info = QueryInfo::from_row(&row).unwrap();
        assert_eq!(info.session_id, 1234);
    }

    // =========================================================================
    // Truncate SQL tests
    // =========================================================================

    #[test]
    fn test_truncate_sql_short() {
        let sql = "SELECT 1";
        assert_eq!(truncate_sql(sql, 200), "SELECT 1");
    }

    #[test]
    fn test_truncate_sql_long() {
        let sql = "SELECT a, b, c, d, e, f, g, h, i, j FROM very_long_table_name WHERE condition = 'something' AND another_condition = 'something else' AND yet_another = 'value' AND more = 'data' AND extra = 'text'";
        let result = truncate_sql(sql, 100);
        assert!(result.len() <= 100);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_sql_normalizes_whitespace() {
        let sql = "SELECT\n    a,\n    b\nFROM\n    t";
        let result = truncate_sql(sql, 200);
        assert_eq!(result, "SELECT a, b FROM t");
    }

    #[test]
    fn test_truncate_sql_exact_limit() {
        let sql = "SELECT 1"; // 8 chars
        assert_eq!(truncate_sql(sql, 8), "SELECT 1");
    }

    // =========================================================================
    // Display format tests
    // =========================================================================

    #[test]
    fn test_display_csv_output() {
        let queries = vec![
            QueryInfo {
                session_id: 1234,
                query_text: "SELECT * FROM employees".to_string(),
                start_time: "2026-02-24 10:30:00".to_string(),
                total_elapsed: "00:00:05.123".to_string(),
                status: "Complete".to_string(),
            },
            QueryInfo {
                session_id: 1234,
                query_text: "DELETE FROM temp_table".to_string(),
                start_time: "2026-02-24 10:29:00".to_string(),
                total_elapsed: "00:00:01.500".to_string(),
                status: "Aborted".to_string(),
            },
        ];

        let mut output = Vec::new();
        display_csv(&queries, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("SessionID,StartTime,ElapsedTime,Status,QueryText"));
        assert!(output_str.contains("1234,2026-02-24 10:30:00,00:00:05.123,Complete,SELECT * FROM employees"));
        assert!(output_str.contains("1234,2026-02-24 10:29:00,00:00:01.500,Aborted,DELETE FROM temp_table"));
    }

    #[test]
    fn test_display_csv_with_special_chars_in_sql() {
        let queries = vec![QueryInfo {
            session_id: 1234,
            query_text: "SELECT * FROM t WHERE name = 'O\"Brien'".to_string(),
            start_time: "2026-02-24 10:30:00".to_string(),
            total_elapsed: "00:00:01.000".to_string(),
            status: "Complete".to_string(),
        }];

        let mut output = Vec::new();
        display_csv(&queries, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // The SQL text containing a double quote should be properly escaped
        assert!(output_str.contains("\"SELECT * FROM t WHERE name = 'O\"\"Brien'\""));
    }

    #[test]
    fn test_display_json_output() {
        let queries = vec![QueryInfo {
            session_id: 1234,
            query_text: "SELECT * FROM employees".to_string(),
            start_time: "2026-02-24 10:30:00".to_string(),
            total_elapsed: "00:00:05.123".to_string(),
            status: "Complete".to_string(),
        }];

        let mut output = Vec::new();
        display_json(&queries, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        let json: Vec<serde_json::Value> = serde_json::from_str(&output_str).unwrap();
        assert_eq!(json.len(), 1);
        assert_eq!(json[0]["SessionID"], 1234);
        assert_eq!(json[0]["QueryText"], "SELECT * FROM employees");
        assert_eq!(json[0]["StartTime"], "2026-02-24 10:30:00");
        assert_eq!(json[0]["ElapsedTime"], "00:00:05.123");
        assert_eq!(json[0]["Status"], "Complete");
    }

    #[test]
    fn test_display_json_multiple_queries() {
        let queries = vec![
            QueryInfo {
                session_id: 1234,
                query_text: "SELECT 1".to_string(),
                start_time: "2026-02-24 10:30:00".to_string(),
                total_elapsed: "00:00:01.000".to_string(),
                status: "Complete".to_string(),
            },
            QueryInfo {
                session_id: 1234,
                query_text: "SELECT 2".to_string(),
                start_time: "2026-02-24 10:29:00".to_string(),
                total_elapsed: "00:00:02.000".to_string(),
                status: "Error".to_string(),
            },
        ];

        let mut output = Vec::new();
        display_json(&queries, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        let json: Vec<serde_json::Value> = serde_json::from_str(&output_str).unwrap();
        assert_eq!(json.len(), 2);
        assert_eq!(json[0]["QueryText"], "SELECT 1");
        assert_eq!(json[1]["QueryText"], "SELECT 2");
        assert_eq!(json[1]["Status"], "Error");
    }

    #[test]
    fn test_display_table_output() {
        let queries = vec![QueryInfo {
            session_id: 1234,
            query_text: "SELECT * FROM employees".to_string(),
            start_time: "2026-02-24 10:30:00".to_string(),
            total_elapsed: "00:00:05.123".to_string(),
            status: "Complete".to_string(),
        }];

        let mut output = Vec::new();
        display_table(&queries, 1234, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("Recent Queries for Session 1234:"));
        assert!(output_str.contains("Property"));
        assert!(output_str.contains("Value"));
        assert!(output_str.contains("Start Time"));
        assert!(output_str.contains("Status"));
        assert!(output_str.contains("Complete"));
        assert!(output_str.contains("SELECT * FROM employees"));
    }

    #[test]
    fn test_display_table_truncates_long_sql() {
        let long_sql = "SELECT ".to_string() + &"a, ".repeat(200) + "z FROM very_long_table";
        let queries = vec![QueryInfo {
            session_id: 1234,
            query_text: long_sql,
            start_time: "2026-02-24 10:30:00".to_string(),
            total_elapsed: "00:00:05.123".to_string(),
            status: "Complete".to_string(),
        }];

        let mut output = Vec::new();
        display_table(&queries, 1234, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // The SQL in the table output should be truncated
        assert!(output_str.contains("..."));
    }
}
