//! SQL execution within the REPL
//!
//! Handles executing SQL statements and displaying results
//! in the interactive context.
//!
//! Features:
//! - Enhanced timing display with breakdown
//! - Result paging for large result sets (Sprint 8 integration)
//! - Automatic row limiting for SELECT queries

// Sprint 29: Pager re-enabled with horizontal scrolling support
use super::pager::{display_with_pager, PagerConfig};
use super::state::ReplState;
use crate::cli::OutputFormat;
use crate::db::DatabaseClient;
use crate::error::Result;
use crate::format::{write_output_for_pager, write_output_with_timing, FormatOptions};
use std::io::Write;
use std::time::{Duration, Instant};

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
    // If we applied a limit, we potentially don't have the full dataset
    // (even if row_count < default_limit, we used execute_with_limit which may not fetch all rows)
    let limited = apply_limit;

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
            "Result limited to {} rows. Use TOP N or SAMPLE N in query for different limit, or /export to save all rows to file.",
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

/// Execute SQL with state management (stores results and uses state colors)
///
/// This version stores the query result in REPL state for later export,
/// and uses the color setting from state.
///
/// Sprint 8: Now integrates with the pager for large result sets.
///
/// Returns the number of rows returned on success.
pub fn execute_sql_with_state<W: Write>(
    client: &DatabaseClient,
    state: &mut ReplState,
    sql: &str,
    writer: &mut W,
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
    // If we applied a limit, we potentially don't have the full dataset
    // (even if row_count < default_limit, we used execute_with_limit which may not fetch all rows)
    let limited = apply_limit;

    // Store result in state for /export command (Sprint 6)
    let result_clone = result.clone();
    state.set_last_result(result);

    // Store SQL and limited flag for full dataset export (Sprint 12)
    state.set_last_query(sql_to_execute.to_string(), limited);

    // Use color setting from state (Sprint 6)
    let use_color = state.are_colors_enabled();

    // Configure formatting
    let format_options = FormatOptions::default()
        .with_header(true)
        .with_color(use_color);

    // Sprint 29: Pager re-enabled with horizontal scrolling support
    // Check if pager is enabled in state (controlled by /pager on|off metacommand)
    let pager_enabled = state.is_pager_enabled();

    if pager_enabled {
        // Format output for pager with ALL columns (no truncation)
        // Sprint 29 fix: Pager needs full table to implement horizontal scrolling
        let mut output_buffer = Vec::new();
        write_output_for_pager(
            &result_clone,
            &mut output_buffer,
            &format_options,
        )?;
        let output_str = String::from_utf8_lossy(&output_buffer).to_string();

        // Try to use pager - if it's not needed (small result), it returns false
        let pager_config = PagerConfig::default();
        match display_with_pager(&output_str, row_count, &pager_config) {
            Ok(true) => {
                // Pager was used, output already displayed
            }
            Ok(false) => {
                // Pager not needed, write directly to output
                writer.write_all(&output_buffer)?;
            }
            Err(e) => {
                // Pager failed, fall back to direct output
                log::warn!("Pager failed, falling back to direct output: {}", e);
                writer.write_all(&output_buffer)?;
            }
        }
    } else {
        // Pager disabled - format and write output directly
        write_output_with_timing(
            &result_clone,
            writer,
            OutputFormat::Table,
            &format_options,
            true, // Always show timing in REPL
        )?;
    }

    // Show limit message if we applied the default limit
    if limited {
        writeln!(writer)?;
        writeln!(
            writer,
            "Result limited to {} rows. Use TOP N or SAMPLE N in query for different limit, or /export to save all rows to file.",
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

/// Enhanced timing information for query execution
#[derive(Debug, Clone)]
pub struct QueryTiming {
    /// Total execution time (from submit to all results received)
    pub total: Duration,
    /// Time to receive first row (latency)
    pub first_row: Option<Duration>,
    /// Time to receive all rows (transfer time)
    pub transfer: Option<Duration>,
}

impl QueryTiming {
    /// Create a new timing record with total time only
    pub fn new(total: Duration) -> Self {
        Self {
            total,
            first_row: None,
            transfer: None,
        }
    }

    /// Create a timing record with full breakdown
    pub fn with_breakdown(total: Duration, first_row: Duration, transfer: Duration) -> Self {
        Self {
            total,
            first_row: Some(first_row),
            transfer: Some(transfer),
        }
    }

    /// Format timing as a simple string (e.g., "0.123s")
    pub fn format_simple(&self) -> String {
        format!("{:.3}s", self.total.as_secs_f64())
    }

    /// Format timing with full breakdown
    pub fn format_enhanced(&self, row_count: usize) -> String {
        let mut parts = Vec::new();

        // Total time
        parts.push(format!("Total: {:.3}s", self.total.as_secs_f64()));

        // First row latency
        if let Some(first_row) = self.first_row {
            parts.push(format!("First row: {:.3}s", first_row.as_secs_f64()));
        }

        // Transfer time
        if let Some(transfer) = self.transfer {
            parts.push(format!("Transfer: {:.3}s", transfer.as_secs_f64()));
        }

        // Rows per second (if meaningful)
        if row_count > 0 && self.total.as_secs_f64() > 0.001 {
            let rows_per_sec = row_count as f64 / self.total.as_secs_f64();
            if rows_per_sec > 1.0 {
                parts.push(format!("{:.0} rows/s", rows_per_sec));
            }
        }

        parts.join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_timing_format_simple() {
        let timing = QueryTiming::new(Duration::from_millis(123));
        assert_eq!(timing.format_simple(), "0.123s");
    }

    #[test]
    fn test_query_timing_format_enhanced() {
        let timing = QueryTiming::with_breakdown(
            Duration::from_millis(500),
            Duration::from_millis(50),
            Duration::from_millis(450),
        );
        let result = timing.format_enhanced(100);
        assert!(result.contains("Total: 0.500s"));
        assert!(result.contains("First row: 0.050s"));
        assert!(result.contains("Transfer: 0.450s"));
        assert!(result.contains("rows/s"));
    }

    #[test]
    fn test_query_timing_rows_per_second() {
        let timing = QueryTiming::new(Duration::from_secs(1));
        let result = timing.format_enhanced(1000);
        assert!(result.contains("1000 rows/s"));
    }

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
        assert!(is_select_without_limit(
            "WITH cte AS (SELECT 1) SELECT * FROM cte"
        ));
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
