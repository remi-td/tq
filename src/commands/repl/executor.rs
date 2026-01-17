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
pub fn execute_sql<W: Write>(
    client: &DatabaseClient,
    sql: &str,
    writer: &mut W,
    use_color: bool,
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

    // Execute the query
    let start = Instant::now();
    let result = client.execute(sql_to_execute)?;
    let execution_time = start.elapsed();

    let row_count = result.row_count;

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

    log::debug!(
        "Query completed: {} rows in {:?}",
        row_count,
        execution_time
    );

    Ok(row_count)
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
}
