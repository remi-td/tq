//! Query command implementation
//!
//! Executes SQL queries and formats output.
//! Supports multiple input sources: argument, file, stdin.

use crate::cli::QueryArgs;
use crate::db::DatabaseClient;
use crate::error::{Result, TqError};
use crate::format::{write_output_with_timing, FormatOptions};
use std::fs::File;
use std::io::{self, BufReader, IsTerminal, Read, Write};
use std::path::Path;

/// Execute the query command
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &QueryArgs,
    writer: &mut W,
    use_color: bool,
    verbose: bool,
) -> Result<()> {
    // Get SQL from appropriate source
    let sql = get_sql(args)?;

    if verbose {
        eprintln!("Executing query: {}", truncate_sql(&sql, 100));
    }

    // Execute query
    let result = if let Some(limit) = args.limit {
        client.execute_with_limit(&sql, limit)?
    } else {
        client.execute(&sql)?
    };

    // Configure output formatting
    let format_options = FormatOptions::default()
        .with_header(!args.no_header)
        .with_color(use_color);

    // Write output
    write_output_with_timing(
        &result,
        writer,
        args.format,
        &format_options,
        args.timing,
    )?;

    Ok(())
}

/// Execute query and write to file
pub fn execute_to_file<W: Write>(
    client: &DatabaseClient,
    args: &QueryArgs,
    status_writer: &mut W,
    _use_color: bool,
    verbose: bool,
) -> Result<()> {
    let output_path = args.output.as_ref().ok_or_else(|| {
        TqError::InternalError("execute_to_file called without output path".to_string())
    })?;

    // Get SQL
    let sql = get_sql(args)?;

    if verbose {
        eprintln!("Executing query: {}", truncate_sql(&sql, 100));
    }

    // Execute query
    let result = if let Some(limit) = args.limit {
        client.execute_with_limit(&sql, limit)?
    } else {
        client.execute(&sql)?
    };

    // Create output file
    let file = File::create(output_path).map_err(|e| TqError::FileWriteError {
        path: output_path.clone(),
        source: e,
    })?;
    let mut buffered_writer = io::BufWriter::new(file);

    // Configure formatting (no colors for file output)
    let format_options = FormatOptions::default()
        .with_header(!args.no_header)
        .with_color(false); // Never use colors in file output

    // Write to file
    write_output_with_timing(
        &result,
        &mut buffered_writer,
        args.format,
        &format_options,
        args.timing,
    )?;

    buffered_writer.flush()?;

    // Report success to status writer
    writeln!(
        status_writer,
        "Wrote {} rows to {}",
        result.row_count,
        output_path.display()
    )?;

    Ok(())
}

/// Get SQL from the appropriate source (argument, file, or stdin)
fn get_sql(args: &QueryArgs) -> Result<String> {
    // Priority: argument > file > stdin
    if let Some(ref query) = args.query {
        return Ok(query.clone());
    }

    if let Some(ref file_path) = args.file {
        return read_sql_file(file_path);
    }

    // Read from stdin
    read_sql_stdin()
}

/// Read SQL from a file
fn read_sql_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| TqError::FileReadError {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Read SQL from stdin
fn read_sql_stdin() -> Result<String> {
    let stdin = io::stdin();

    // Check if stdin is a terminal
    if stdin.is_terminal() {
        return Err(TqError::InvalidConfig(
            "No query provided. Use 'tq query \"SELECT ...\"' or pipe SQL via stdin.".to_string(),
        ));
    }

    let mut sql = String::new();
    let mut reader = BufReader::new(stdin.lock());
    reader.read_to_string(&mut sql)?;

    if sql.trim().is_empty() {
        return Err(TqError::InvalidConfig(
            "Empty query. Provide SQL via argument, file, or stdin.".to_string(),
        ));
    }

    Ok(sql)
}

/// Truncate SQL for display (verbose output)
fn truncate_sql(sql: &str, max_len: usize) -> String {
    let normalized: String = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.len() <= max_len {
        normalized
    } else {
        format!("{}...", &normalized[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_sql_short() {
        let sql = "SELECT 1";
        assert_eq!(truncate_sql(sql, 100), "SELECT 1");
    }

    #[test]
    fn test_truncate_sql_long() {
        let sql = "SELECT a, b, c, d, e, f FROM very_long_table_name WHERE condition = 'something'";
        let result = truncate_sql(sql, 50);
        assert!(result.len() <= 50);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_sql_normalizes_whitespace() {
        let sql = "SELECT\n    a,\n    b\nFROM\n    t";
        let result = truncate_sql(sql, 100);
        assert_eq!(result, "SELECT a, b FROM t");
    }
}
