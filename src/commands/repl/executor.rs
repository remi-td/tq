//! SQL execution within the REPL
//!
//! Handles executing SQL statements and displaying results
//! in the interactive context.

use crate::cli::OutputFormat;
use crate::db::DatabaseClient;
use crate::error::Result;
use crate::format::{write_output_with_timing, FormatOptions};
use std::io::Write;
use std::time::Instant;

/// Execute a SQL statement and write results to the writer
///
/// Returns the number of rows returned on success.
///
/// # Arguments
/// * `client` - Database client to execute the query
/// * `sql` - SQL statement to execute
/// * `writer` - Output writer for results
/// * `use_color` - Whether to use colored output
/// * `default_limit` - Default row limit for SELECT queries (0 = unlimited)
pub fn execute_sql<W: Write>(
    client: &DatabaseClient,
    sql: &str,
    writer: &mut W,
    use_color: bool,
    default_limit: usize,
) -> Result<usize> {
    let trimmed = sql.trim();

    // Skip empty statements
    if trimmed.is_empty() || trimmed == ";" {
        return Ok(0);
    }

    // Strip trailing semicolon for execution
    let sql_to_execute = trimmed.trim_end_matches(';').trim();

    if sql_to_execute.is_empty() {
        return Ok(0);
    }

    log::debug!("Executing SQL: {}", truncate_for_log(sql_to_execute));

    // Check if we should apply the default limit
    let apply_limit = default_limit > 0 && is_select_without_limit(sql_to_execute);

    // Execute the query with or without limit
    let start = Instant::now();
    let result = if apply_limit {
        log::debug!("Applying default REPL limit: {} rows", default_limit);
        client.execute_with_limit(sql_to_execute, default_limit)?
    } else {
        client.execute(sql_to_execute)?
    };
    let execution_time = start.elapsed();

    let row_count = result.row_count;
    let limited = apply_limit && row_count == default_limit;

    // Configure formatting
    let format_options = FormatOptions::default()
        .with_header(true)
        .with_color(use_color);

    // Write the results (always show timing in REPL)
    write_output_with_timing(
        &result,
        writer,
        OutputFormat::Table,
        &format_options,
        true, // Always show timing in REPL
    )?;

    // Show limit message if we applied the default limit
    if limited {
        writeln!(writer)?;
        writeln!(
            writer,
            "Showing first {} rows. Add LIMIT clause for different results.",
            default_limit
        )?;
    }

    log::debug!(
        "Query completed: {} rows in {:?}",
        row_count,
        execution_time
    );

    Ok(row_count)
}

/// Check if SQL is a SELECT statement without an explicit LIMIT or TOP clause
///
/// This is used to determine if the default REPL limit should be applied.
fn is_select_without_limit(sql: &str) -> bool {
    let sql_upper = sql.to_uppercase();
    let sql_trimmed = sql_upper.trim();

    // Must be a SELECT statement (including WITH ... SELECT)
    let is_select = sql_trimmed.starts_with("SELECT")
        || sql_trimmed.starts_with("WITH")
        || sql_trimmed.starts_with("SEL "); // Teradata abbreviation

    if !is_select {
        return false;
    }

    // Check if the query already has a LIMIT, TOP, or SAMPLE clause
    // Note: These checks are intentionally simple - they may have edge cases
    // but cover the common usage patterns
    if sql_upper.contains(" LIMIT ") || sql_upper.contains(" LIMIT\n") {
        return false;
    }

    // Check for TOP N (Teradata syntax for limiting rows)
    if sql_upper.contains(" TOP ") {
        return false;
    }

    // Check for SAMPLE (Teradata random sampling)
    if sql_upper.contains(" SAMPLE ") {
        return false;
    }

    true
}

/// Truncate SQL for logging (avoid huge log entries)
fn truncate_for_log(sql: &str) -> String {
    const MAX_LEN: usize = 200;
    let normalized: String = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.len() <= MAX_LEN {
        normalized
    } else {
        format!("{}...", &normalized[..MAX_LEN - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_for_log_short() {
        let sql = "SELECT 1";
        assert_eq!(truncate_for_log(sql), "SELECT 1");
    }

    #[test]
    fn test_truncate_for_log_normalizes() {
        let sql = "SELECT\n    a,\n    b\nFROM t";
        assert_eq!(truncate_for_log(sql), "SELECT a, b FROM t");
    }

    #[test]
    fn test_is_select_without_limit_basic() {
        assert!(is_select_without_limit("SELECT * FROM table"));
        assert!(is_select_without_limit("  SELECT col FROM t"));
        assert!(is_select_without_limit("SELECT 1"));
    }

    #[test]
    fn test_is_select_without_limit_with_limit() {
        assert!(!is_select_without_limit("SELECT * FROM table LIMIT 10"));
        assert!(!is_select_without_limit("SELECT * FROM table LIMIT\n10"));
    }

    #[test]
    fn test_is_select_without_limit_with_top() {
        assert!(!is_select_without_limit("SELECT TOP 100 * FROM table"));
    }

    #[test]
    fn test_is_select_without_limit_with_sample() {
        assert!(!is_select_without_limit("SELECT * FROM table SAMPLE 100"));
    }

    #[test]
    fn test_is_select_without_limit_with_cte() {
        assert!(is_select_without_limit("WITH cte AS (SELECT 1) SELECT * FROM cte"));
        assert!(!is_select_without_limit(
            "WITH cte AS (SELECT 1) SELECT * FROM cte LIMIT 10"
        ));
    }

    #[test]
    fn test_is_select_without_limit_non_select() {
        assert!(!is_select_without_limit("INSERT INTO table VALUES (1)"));
        assert!(!is_select_without_limit("UPDATE table SET col = 1"));
        assert!(!is_select_without_limit("DELETE FROM table"));
        assert!(!is_select_without_limit("CREATE TABLE t (id INT)"));
    }

    #[test]
    fn test_is_select_without_limit_teradata_abbreviation() {
        assert!(is_select_without_limit("SEL * FROM table"));
        assert!(!is_select_without_limit("SEL TOP 10 * FROM table"));
    }
}
