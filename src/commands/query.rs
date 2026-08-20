//! Query command implementation
//!
//! Executes SQL queries and formats output.
//! Supports multiple input sources: argument, file, stdin.
//! Supports batch mode for multi-statement execution.

use crate::cli::{OutputFormat, QueryArgs};
use crate::db::DatabaseClient;
use crate::error::{Result, Severity, TqError};
use crate::format::{write_output_with_pagination, write_output_with_timing, FormatOptions};
use crate::pagination::PaginationInfo;
use crate::params::ParamStore;
use std::collections::HashMap;
use crate::sql::{
    classify_statement_detailed, has_multiple_statements, parse_statements, ParsedStatement,
    StatementSafety,
};
#[cfg(test)]
use crate::sql::classify_statement;
use std::io::{self, BufReader, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

// ============================================================================
// Input Source Types
// ============================================================================

/// Represents the source of SQL input for the query command
#[derive(Debug, Clone, PartialEq)]
pub enum InputSource {
    /// SQL provided as command-line argument
    Argument(String),
    /// SQL read from a file
    File(PathBuf),
    /// SQL read from stdin (piped input)
    Stdin,
}

impl InputSource {
    /// Get a description of the input source for error messages
    pub fn description(&self) -> String {
        match self {
            InputSource::Argument(_) => "command-line argument".to_string(),
            InputSource::File(path) => format!("file '{}'", path.display()),
            InputSource::Stdin => "stdin".to_string(),
        }
    }
}

// ============================================================================
// Batch Execution Types
// ============================================================================

/// Result of batch execution for reporting
#[derive(Debug)]
pub struct BatchExecutionResult {
    /// Number of statements successfully executed
    pub successful_count: usize,
    /// Total number of statements
    pub total_count: usize,
}

/// Context for a batch execution error
#[derive(Debug)]
pub struct BatchExecutionError {
    /// The statement that failed
    pub statement: ParsedStatement,
    /// The underlying error
    pub error: TqError,
    /// Number of statements executed before failure
    pub successful_count: usize,
    /// Total number of statements in the batch
    pub total_count: usize,
}

impl BatchExecutionError {
    /// Create a new batch execution error
    pub fn new(
        statement: ParsedStatement,
        error: TqError,
        successful_count: usize,
        total_count: usize,
    ) -> Self {
        Self {
            statement,
            error,
            successful_count,
            total_count,
        }
    }

    /// Format the error for display
    pub fn format_error(&self) -> String {
        let mut msg = format!(
            "Error at statement {} (line {}): {}\n",
            self.statement.statement_number, self.statement.start_line, self.error
        );

        // Add statement preview
        msg.push_str(&format!("\nStatement: {}\n", self.statement.preview(80)));

        // Add execution context
        if self.successful_count > 0 {
            msg.push_str(&format!(
                "\nStatements executed: 1-{}\n",
                self.successful_count
            ));
        }

        let remaining_start = self.statement.statement_number + 1;
        if remaining_start <= self.total_count {
            msg.push_str(&format!(
                "Statements remaining: {}-{}\n",
                remaining_start, self.total_count
            ));
        }

        msg
    }
}

// ============================================================================
// Input Source Resolution
// ============================================================================

/// Determine the input source based on QueryArgs.
///
/// The source is selected purely from the *syntax of the invocation*, never
/// from transient file-descriptor readiness. The precedence is:
///
/// 1. **Positional query argument present** -> `Argument`. Stdin is never inspected.
/// 2. **Else `--file <path>` present** -> `File`. Stdin is never inspected.
/// 3. **Else stdin is not a TTY** -> `Stdin`. A normal blocking read to EOF is
///    performed by `read_sql_stdin`, so a delayed producer is handled naturally.
/// 4. **Else** (no arg, no file, stdin is a TTY) -> `No query provided`.
///
/// This matches `psql -c` / `psql -f`: an explicit source wins and stdin is
/// ignored. `clap`'s `conflicts_with` already makes the positional argument and
/// `--file` mutually exclusive, so cases 1 and 2 cannot both apply. There is no
/// readiness probe and no Unix/non-Unix split.
fn determine_input_source(args: &QueryArgs) -> Result<InputSource> {
    if let Some(ref query) = args.query {
        Ok(InputSource::Argument(query.clone()))
    } else if let Some(ref file_path) = args.file {
        Ok(InputSource::File(file_path.clone()))
    } else if !io::stdin().is_terminal() {
        Ok(InputSource::Stdin)
    } else {
        Err(TqError::InvalidConfig(
            "No query provided.\n\n\
             Provide SQL via:\n  \
             - Command argument: tq query \"SELECT 1\"\n  \
             - File: tq query --file script.sql\n  \
             - Stdin: echo \"SELECT 1\" | tq query"
                .to_string(),
        ))
    }
}

/// Read SQL from the determined input source
fn read_input_sql(source: &InputSource) -> Result<String> {
    match source {
        InputSource::Argument(query) => Ok(query.clone()),
        InputSource::File(path) => read_sql_file(path),
        InputSource::Stdin => read_sql_stdin(),
    }
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
    let mut sql = String::new();
    let mut reader = BufReader::new(stdin.lock());
    reader.read_to_string(&mut sql)?;

    if sql.trim().is_empty() {
        return Err(TqError::InvalidConfig(
            "Empty query received from stdin.\n\
             Provide valid SQL via stdin."
                .to_string(),
        ));
    }

    Ok(sql)
}

// ============================================================================
// Main Execute Functions
// ============================================================================

/// Execute the query command
///
/// When `params` is `Some` and non-empty, `{{variable}}` markers in the SQL
/// are substituted before execution.
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &QueryArgs,
    params: Option<&ParamStore>,
    writer: &mut W,
    use_color: bool,
    verbose: bool,
    error_levels: &HashMap<u32, Severity>,
) -> Result<u8> {
    // Determine input source
    let source = determine_input_source(args)?;

    if verbose {
        eprintln!("Reading SQL from {}", source.description());
    }

    // Read SQL from source
    let sql = read_input_sql(&source)?;

    // Apply variable substitution if params provided or SQL contains substitution markers
    let sql = match params {
        Some(p) if !p.is_empty() || ParamStore::has_variables(&sql) => p.substitute(&sql)?,
        None if ParamStore::has_variables(&sql) => {
            let p = ParamStore::new();
            p.substitute(&sql)?
        }
        _ => sql,
    };

    // Check for --dry-run preview
    if args.dry_run {
        writeln!(writer, "{}", sql)?;
        return Ok(0);
    }

    // Agent-safe mode validation
    if args.agent_safe {
        validate_agent_safe(&sql, args)?;
    }

    // Determine execution mode: single statement (fast path) or batch
    // For command-line arguments, always use single statement mode (no splitting)
    // For file/stdin, check for multiple statements
    let use_batch = match source {
        InputSource::Argument(_) => false, // Never split argument SQL
        _ => has_multiple_statements(&sql),
    };

    if use_batch {
        execute_batch(client, &sql, args, writer, use_color, verbose, error_levels)
    } else {
        execute_single(client, &sql, args, writer, use_color, verbose, error_levels)
    }
}

/// Execute a single SQL statement (fast path)
fn execute_single<W: Write>(
    client: &DatabaseClient,
    sql: &str,
    args: &QueryArgs,
    writer: &mut W,
    use_color: bool,
    verbose: bool,
    error_levels: &HashMap<u32, Severity>,
) -> Result<u8> {
    let trimmed_sql = sql.trim();

    if verbose {
        eprintln!("Executing query: {}", truncate_sql(trimmed_sql, 100));
    }

    // Determine effective row limit: explicit --limit takes precedence, then agent-safe max_rows
    let effective_limit = if let Some(limit) = args.limit {
        Some(limit)
    } else if args.agent_safe {
        Some(args.max_rows + 1) // Fetch one extra to detect overflow
    } else {
        None
    };

    // Execute query
    let result_or_err = if let Some(limit) = effective_limit {
        client.execute_with_limit(trimmed_sql, limit)
    } else {
        client.execute(trimmed_sql)
    };

    let mut result = match result_or_err {
        Ok(res) => res,
        Err(err) => {
            if let Some(code) = err.teradata_error_code() {
                if let Some(&severity) = error_levels.get(&code) {
                    if severity <= Severity::Warning {
                        eprintln!("Warning: [Error {}] {}", code, err);
                        return Ok(severity as u8);
                    }
                }
            }
            return Err(err);
        }
    };

    // Agent-safe: check if result exceeds max_rows
    if args.agent_safe && args.limit.is_none() && result.row_count > args.max_rows {
        return Err(TqError::AgentSafeMaxRows {
            limit: args.max_rows,
        });
    }

    // Configure output formatting
    let format_options = FormatOptions::default()
        .with_header(!args.no_header)
        .with_color(use_color);

    let effective_format = if args.json { OutputFormat::Json } else { args.format };

    // Apply pagination if --page-size is set
    if let Some(page_size) = args.page_size {
        let pagination = PaginationInfo::new(args.page, page_size, result.row_count);
        let (start, end) = pagination.row_range();
        if start < result.rows.len() {
            result.rows = result.rows[start..end.min(result.rows.len())].to_vec();
        } else {
            result.rows = Vec::new();
        }
        result.row_count = result.rows.len();

        write_output_with_pagination(
            &result,
            writer,
            effective_format,
            &format_options,
            args.timing,
            Some(&pagination),
        )?;
    } else {
        // Write output
        write_output_with_timing(&result, writer, effective_format, &format_options, args.timing)?;
    }

    Ok(0)
}

/// Detect if SQL contains explicit transaction control statements
fn contains_transaction_control(sql: &str) -> bool {
    let sql_upper = sql.to_uppercase();
    // Check for Teradata transaction control keywords
    // BEGIN TRANSACTION, BT (begin transaction shorthand), COMMIT, ET (end transaction), ROLLBACK
    sql_upper.contains("BEGIN TRANSACTION")
        || sql_upper.contains("BEGIN TRAN")
        || sql_upper.contains("COMMIT")
        || sql_upper.contains("ROLLBACK")
        // Teradata-specific: BT and ET
        || sql_upper
            .split(|c: char| !c.is_alphanumeric())
            .any(|word| word == "BT" || word == "ET")
}

/// Execute multiple SQL statements in batch mode
fn execute_batch<W: Write>(
    client: &DatabaseClient,
    sql: &str,
    args: &QueryArgs,
    writer: &mut W,
    use_color: bool,
    verbose: bool,
    error_levels: &HashMap<u32, Severity>,
) -> Result<u8> {
    // Parse statements
    let statements = parse_statements(sql)
        .map_err(TqError::from)?;
    let total_count = statements.len();

    if verbose {
        eprintln!("Found {} statements to execute", total_count);
    }

    // Check for --atomic flag with single statement
    if args.atomic && total_count == 1 {
        eprintln!(
            "Warning: --atomic has no effect on single statements (statement executes normally)"
        );
    }

    // Check for explicit transaction control if --atomic is used
    if args.atomic && total_count > 1 && contains_transaction_control(sql) {
        return Err(TqError::AtomicConflict);
    }

    // Configure output formatting (no color in batch progress messages to stderr)
    let format_options = FormatOptions::default()
        .with_header(!args.no_header)
        .with_color(use_color);

    // Begin transaction if atomic mode with multiple statements
    let in_transaction = args.atomic && total_count > 1;
    if in_transaction {
        if verbose {
            eprintln!("BEGIN TRANSACTION (--atomic mode)");
        }
        eprintln!("[Begin transaction]");

        if let Err(e) = client.execute("BEGIN TRANSACTION") {
            return Err(TqError::TransactionError {
                operation: "BEGIN".to_string(),
                message: format!("Failed to start transaction: {}", e),
            });
        }
    }

    let mut successful_count = 0;
    let mut batch_result: Result<()> = Ok(());
    let mut max_severity: u8 = 0;

    let effective_format = if args.json { OutputFormat::Json } else { args.format };

    for statement in &statements {
        // Show progress to stderr
        eprint!(
            "Statement {}: {}... ",
            statement.statement_number,
            get_statement_type(&statement.sql)
        );

        // Execute statement
        let result = match args.limit {
            Some(limit) => client.execute_with_limit(&statement.sql, limit),
            None => client.execute(&statement.sql),
        };

        match result {
            Ok(query_result) => {
                successful_count += 1;

                // Format status based on result
                let status = format_statement_status(&query_result, effective_format);
                eprintln!("{}", status);

                // Write results for SELECT queries (those with rows)
                if query_result.row_count > 0 {
                    write_output_with_timing(
                        &query_result,
                        writer,
                        effective_format,
                        &format_options,
                        args.timing,
                    )?;

                    // Add separator between result sets for readability
                    if statement.statement_number < total_count
                        && effective_format == OutputFormat::Table
                    {
                        writeln!(writer)?;
                    }
                }
            }
            Err(error) => {
                let mut demoted = false;
                if let Some(code) = error.teradata_error_code() {
                    if let Some(&severity) = error_levels.get(&code) {
                        if severity <= Severity::Warning {
                            eprintln!("WARNING ([Error {}] {})", code, error);
                            max_severity = std::cmp::max(max_severity, severity as u8);
                            demoted = true;
                        }
                    }
                }

                if !demoted {
                    // Fail-fast: stop on first error
                    eprintln!("FAILED");

                    let batch_error = BatchExecutionError::new(
                        statement.clone(),
                        error,
                        successful_count,
                        total_count,
                    );

                    batch_result = Err(TqError::QueryExecution(batch_error.format_error()));
                    break;
                }
            }
        }
    }

    // Handle transaction completion
    if in_transaction {
        match &batch_result {
            Ok(_) => {
                if verbose {
                    eprintln!("COMMIT (all statements succeeded or warnings only)");
                }
                eprintln!("[Commit transaction]");

                if let Err(e) = client.execute("COMMIT") {
                    return Err(TqError::TransactionError {
                        operation: "COMMIT".to_string(),
                        message: format!(
                            "All statements succeeded but COMMIT failed: {}",
                            e
                        ),
                    });
                }
                eprintln!("Transaction committed");
            }
            Err(_) => {
                if verbose {
                    eprintln!("ROLLBACK (statement failed)");
                }
                eprintln!("[Rollback transaction]");

                // Best effort rollback - don't mask original error
                if let Err(rollback_err) = client.execute("ROLLBACK") {
                    log::warn!("Rollback failed: {}", rollback_err);
                    eprintln!("Warning: Rollback may have failed: {}", rollback_err);
                } else {
                    eprintln!("Transaction rolled back (all changes reverted)");
                }
            }
        }
    }

    // Return the batch result
    batch_result?;

    // Summary message
    if verbose {
        if max_severity > 0 {
            eprintln!("\nBatch completed with warnings (highest severity: {})", max_severity);
        } else {
            eprintln!("\nAll {} statements executed successfully", total_count);
        }
    }

    Ok(max_severity)
}

/// Execute query and write to file atomically
///
/// Uses a temp-file-then-rename pattern for atomic writes:
/// 1. Write to temporary file in same directory
/// 2. Rename to final path only on success
/// 3. Prevents partial files on error or interruption
///
/// When `params` is `Some` and non-empty, `{{variable}}` markers in the SQL
/// are substituted before execution.
pub fn execute_to_file<W: Write>(
    client: &DatabaseClient,
    args: &QueryArgs,
    params: Option<&ParamStore>,
    status_writer: &mut W,
    _use_color: bool,
    verbose: bool,
    error_levels: &HashMap<u32, Severity>,
) -> Result<u8> {
    let output_path = args.output.as_ref().ok_or_else(|| {
        TqError::InternalError("execute_to_file called without output path".to_string())
    })?;

    // Determine input source
    let source = determine_input_source(args)?;

    if verbose {
        eprintln!("Reading SQL from {}", source.description());
    }

    // Read SQL from source
    let sql = read_input_sql(&source)?;

    // Apply variable substitution if params provided or SQL contains substitution markers
    let sql = match params {
        Some(p) if !p.is_empty() || ParamStore::has_variables(&sql) => p.substitute(&sql)?,
        None if ParamStore::has_variables(&sql) => {
            let p = ParamStore::new();
            p.substitute(&sql)?
        }
        _ => sql,
    };

    // Check for --dry-run preview
    if args.dry_run {
        writeln!(status_writer, "{}", sql)?;
        return Ok(0);
    }

    // Agent-safe mode validation
    if args.agent_safe {
        validate_agent_safe(&sql, args)?;
    }

    // Determine execution mode
    let use_batch = match source {
        InputSource::Argument(_) => false,
        _ => has_multiple_statements(&sql),
    };

    // Create temp file in same directory as output (ensures same filesystem for atomic rename)
    let parent_dir = output_path.parent().unwrap_or(Path::new("."));
    let temp_file = NamedTempFile::new_in(parent_dir).map_err(|e| TqError::FileWriteError {
        path: output_path.clone(),
        source: e,
    })?;

    let mut buffered_writer = io::BufWriter::new(&temp_file);

    // Configure formatting (no colors for file output)
    let format_options = FormatOptions::default()
        .with_header(!args.no_header)
        .with_color(false);

    let mut max_severity = 0;

    let row_count = if use_batch {
        // Execute batch and write to file
        let statements = parse_statements(&sql)
            .map_err(TqError::from)?;
        let total_count = statements.len();
        let mut total_rows = 0;

        for statement in &statements {
            if verbose {
                eprint!(
                    "Statement {}: {}... ",
                    statement.statement_number,
                    get_statement_type(&statement.sql)
                );
            }

            let result = match args.limit {
                Some(limit) => client.execute_with_limit(&statement.sql, limit),
                None => client.execute(&statement.sql),
            };

            match result {
                Ok(query_result) => {
                    if verbose {
                        eprintln!("OK ({} rows)", query_result.row_count);
                    }

                    total_rows += query_result.row_count;

                    if query_result.row_count > 0 {
                        write_output_with_timing(
                            &query_result,
                            &mut buffered_writer,
                            args.format,
                            &format_options,
                            args.timing,
                        )?;
                    }
                }
                Err(error) => {
                    let mut demoted = false;
                    if let Some(code) = error.teradata_error_code() {
                        if let Some(&severity) = error_levels.get(&code) {
                            if severity <= Severity::Warning {
                                if verbose {
                                    eprintln!("WARNING ([Error {}] {})", code, error);
                                } else {
                                    eprintln!("Statement {}: WARNING ([Error {}] {})", statement.statement_number, code, error);
                                }
                                max_severity = std::cmp::max(max_severity, severity as u8);
                                demoted = true;
                            }
                        }
                    }

                    if !demoted {
                        if verbose {
                            eprintln!("FAILED");
                        }
                        // Temp file will be automatically cleaned up when dropped
                        return Err(error);
                    }
                }
            }
        }

        buffered_writer.flush()?;

        writeln!(
            status_writer,
            "Wrote {} rows from {} statements to {}",
            total_rows,
            total_count,
            output_path.display()
        )?;

        total_rows
    } else {
        // Single statement execution
        let trimmed_sql = sql.trim();

        if verbose {
            eprintln!("Executing query: {}", truncate_sql(trimmed_sql, 100));
        }

        let result_or_err = if let Some(limit) = args.limit {
            client.execute_with_limit(trimmed_sql, limit)
        } else {
            client.execute(trimmed_sql)
        };

        let result = match result_or_err {
            Ok(res) => res,
            Err(err) => {
                if let Some(code) = err.teradata_error_code() {
                    if let Some(&severity) = error_levels.get(&code) {
                        if severity <= Severity::Warning {
                            eprintln!("Warning: [Error {}] {}", code, err);
                            max_severity = severity as u8;
                            crate::db::QueryResult::new(Vec::new(), Vec::new(), std::time::Duration::from_secs(0))
                        } else {
                            return Err(err);
                        }
                    } else {
                        return Err(err);
                    }
                } else {
                    return Err(err);
                }
            }
        };

        let rows = result.row_count;

        write_output_with_timing(
            &result,
            &mut buffered_writer,
            args.format,
            &format_options,
            args.timing,
        )?;

        buffered_writer.flush()?;

        writeln!(
            status_writer,
            "Wrote {} rows to {}",
            rows,
            output_path.display()
        )?;

        rows
    };

    // Drop the buffered writer to release the file handle before persist
    drop(buffered_writer);

    // Atomic rename to final destination
    // persist() moves the temp file to the target path atomically
    temp_file
        .persist(output_path)
        .map_err(|e| TqError::FileWriteError {
            path: output_path.clone(),
            source: e.error,
        })?;

    if verbose {
        eprintln!("File written atomically ({} rows)", row_count);
    }

    Ok(max_severity)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Truncate SQL for display (verbose output)
fn truncate_sql(sql: &str, max_len: usize) -> String {
    let normalized: String = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.len() <= max_len {
        normalized
    } else {
        format!("{}...", &normalized[..max_len.saturating_sub(3)])
    }
}

/// Get the statement type (first keyword) for progress messages
fn get_statement_type(sql: &str) -> &str {
    // Skip leading comments to find the real keyword
    let sql_trimmed = sql.trim();

    // Check for line comment
    if sql_trimmed.starts_with("--") {
        // Find the first line that's not a comment
        for line in sql_trimmed.lines() {
            let line_trimmed = line.trim();
            if !line_trimmed.is_empty() && !line_trimmed.starts_with("--") {
                return line_trimmed
                    .split_whitespace()
                    .next()
                    .unwrap_or("SQL")
                    .trim_matches(|c: char| !c.is_alphanumeric());
            }
        }
    }

    // Check for block comment
    if sql_trimmed.starts_with("/*") {
        if let Some(end_pos) = sql_trimmed.find("*/") {
            let after_comment = sql_trimmed[end_pos + 2..].trim();
            return after_comment
                .split_whitespace()
                .next()
                .unwrap_or("SQL")
                .trim_matches(|c: char| !c.is_alphanumeric());
        }
    }

    // No leading comment, get first word
    sql_trimmed
        .split_whitespace()
        .next()
        .unwrap_or("SQL")
        .trim_matches(|c: char| !c.is_alphanumeric())
}

/// Format the status message for a completed statement
fn format_statement_status(result: &crate::db::QueryResult, format: OutputFormat) -> String {
    let row_count = result.row_count;

    if row_count == 0 {
        "OK".to_string()
    } else {
        match format {
            OutputFormat::Table => format!("{} rows returned", row_count),
            OutputFormat::Json => format!("{} rows (JSON)", row_count),
            OutputFormat::Csv => format!("{} rows (CSV)", row_count),
            OutputFormat::Markdown | OutputFormat::Md => {
                format!("{} rows (Markdown)", row_count)
            }
        }
    }
}

// ============================================================================
// Agent-Safe Mode
// ============================================================================

/// Validate SQL against agent-safe mode restrictions.
///
/// Uses the structural classifier in `crate::sql::classifier`, which sees
/// through leading comments, `WITH` CTE prologues, and `LOCKING` request
/// modifiers to the effective top-level operation. The reported
/// `statement_type` is the *effective resolved* operation (e.g. `UPDATE` for
/// `LOCKING ... UPDATE`). Statements that cannot be classified fail closed with
/// a distinct `AgentSafeUnclassified` error rather than being mislabelled DDL.
fn validate_agent_safe(sql: &str, args: &QueryArgs) -> Result<()> {
    // Reject multi-statement input
    if has_multiple_statements(sql) {
        return Err(TqError::AgentSafeBlocked {
            statement_type: "MULTI_STATEMENT".to_string(),
            message: "Agent-safe mode requires single-statement input".to_string(),
        });
    }

    let classification = classify_statement_detailed(sql);
    let effective_op = classification
        .effective_op
        .clone()
        .unwrap_or_else(|| "UNKNOWN".to_string());

    match classification.safety {
        StatementSafety::ReadOnly => Ok(()),
        StatementSafety::Maintenance => {
            if args.allow_maintenance {
                Ok(())
            } else {
                Err(TqError::AgentSafeBlocked {
                    statement_type: effective_op,
                    message: "Maintenance statements (e.g. COLLECT STATISTICS) are blocked in agent-safe mode. Use --allow-maintenance to permit them.".to_string(),
                })
            }
        }
        StatementSafety::Dml => {
            if args.allow_dml {
                Ok(())
            } else {
                Err(TqError::AgentSafeBlocked {
                    statement_type: effective_op,
                    message: "DML statements are blocked in agent-safe mode. Use --allow-dml to permit write operations.".to_string(),
                })
            }
        }
        StatementSafety::Ddl => Err(TqError::AgentSafeBlocked {
            statement_type: effective_op,
            message: "DDL statements are always blocked in agent-safe mode.".to_string(),
        }),
        StatementSafety::Unknown { token, reason } => {
            Err(TqError::AgentSafeUnclassified { token, reason })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_transaction_control_begin() {
        assert!(contains_transaction_control("BEGIN TRANSACTION"));
        assert!(contains_transaction_control("begin transaction"));
        assert!(contains_transaction_control("BEGIN TRAN"));
        assert!(contains_transaction_control("begin tran"));
    }

    #[test]
    fn test_contains_transaction_control_commit_rollback() {
        assert!(contains_transaction_control("COMMIT"));
        assert!(contains_transaction_control("commit"));
        assert!(contains_transaction_control("ROLLBACK"));
        assert!(contains_transaction_control("rollback"));
    }

    #[test]
    fn test_contains_transaction_control_teradata_shortcuts() {
        // Teradata-specific BT and ET (begin/end transaction)
        assert!(contains_transaction_control("BT;"));
        assert!(contains_transaction_control("ET;"));
        assert!(contains_transaction_control("SELECT 1; BT; INSERT..."));
        assert!(contains_transaction_control("SELECT 1; ET; INSERT..."));
    }

    #[test]
    fn test_contains_transaction_control_no_false_positives() {
        // Should not match these
        assert!(!contains_transaction_control("SELECT * FROM my_table"));
        assert!(!contains_transaction_control("INSERT INTO t VALUES (1)"));
        assert!(!contains_transaction_control("UPDATE t SET x = 1"));
        // Words containing BT or ET should not match
        assert!(!contains_transaction_control("SELECT BETTER FROM t"));
        assert!(!contains_transaction_control("SELECT BTEQ FROM t"));
    }

    #[test]
    fn test_contains_transaction_control_in_multi_statement() {
        let sql = "INSERT INTO t VALUES (1);\nBEGIN TRANSACTION;\nUPDATE t SET x = 2;\nCOMMIT;";
        assert!(contains_transaction_control(sql));
    }

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

    #[test]
    fn test_input_source_description() {
        assert_eq!(
            InputSource::Argument("SELECT 1".to_string()).description(),
            "command-line argument"
        );
        assert_eq!(
            InputSource::File(PathBuf::from("test.sql")).description(),
            "file 'test.sql'"
        );
        assert_eq!(InputSource::Stdin.description(), "stdin");
    }

    #[test]
    fn test_get_statement_type() {
        assert_eq!(get_statement_type("SELECT * FROM t"), "SELECT");
        assert_eq!(get_statement_type("INSERT INTO t VALUES (1)"), "INSERT");
        assert_eq!(get_statement_type("UPDATE t SET x = 1"), "UPDATE");
        assert_eq!(get_statement_type("DELETE FROM t"), "DELETE");
        assert_eq!(get_statement_type("CREATE TABLE t (x INT)"), "CREATE");
        // Comments are skipped to find the real statement type
        assert_eq!(get_statement_type("-- Comment\nSELECT 1"), "SELECT");
        assert_eq!(
            get_statement_type("/* Block comment */\nUPDATE t SET x = 1"),
            "UPDATE"
        );
    }

    #[test]
    fn test_batch_execution_error_format() {
        let statement = ParsedStatement::new("SELECT * FROM nonexistent".to_string(), 2, 5);
        let error = TqError::TableNotFound {
            table: "nonexistent".to_string(),
        };
        let batch_error = BatchExecutionError::new(statement, error, 1, 5);

        let formatted = batch_error.format_error();

        assert!(formatted.contains("statement 2"));
        assert!(formatted.contains("line 5"));
        assert!(formatted.contains("SELECT * FROM nonexistent"));
        assert!(formatted.contains("Statements executed: 1-1"));
        assert!(formatted.contains("Statements remaining: 3-5"));
    }

    #[test]
    fn test_batch_execution_error_no_remaining() {
        let statement = ParsedStatement::new("SELECT 1".to_string(), 3, 1);
        let error = TqError::QueryExecution("test error".to_string());
        let batch_error = BatchExecutionError::new(statement, error, 2, 3);

        let formatted = batch_error.format_error();

        // Last statement failed, no remaining
        assert!(formatted.contains("Statements executed: 1-2"));
        assert!(!formatted.contains("Statements remaining"));
    }

    #[test]
    fn test_batch_execution_error_first_statement() {
        let statement = ParsedStatement::new("SELECT 1".to_string(), 1, 1);
        let error = TqError::QueryExecution("test error".to_string());
        let batch_error = BatchExecutionError::new(statement, error, 0, 3);

        let formatted = batch_error.format_error();

        // First statement failed, no executed
        assert!(!formatted.contains("Statements executed"));
        assert!(formatted.contains("Statements remaining: 2-3"));
    }

    // Sprint 54: Agent-safe mode tests

    #[test]
    fn test_classify_select_is_readonly() {
        assert_eq!(classify_statement("SELECT * FROM t"), StatementSafety::ReadOnly);
        assert_eq!(classify_statement("  select 1"), StatementSafety::ReadOnly);
        assert_eq!(classify_statement("SEL * FROM t"), StatementSafety::ReadOnly);
        assert_eq!(classify_statement("SHOW VIEW db.v"), StatementSafety::ReadOnly);
        assert_eq!(classify_statement("EXPLAIN SELECT 1"), StatementSafety::ReadOnly);
        assert_eq!(classify_statement("HELP TABLE t"), StatementSafety::ReadOnly);
        // Sprint 71: COLLECT STATISTICS now classifies as Maintenance, not ReadOnly.
        // See tc111_u16_collect_statistics_is_maintenance below.
        assert_eq!(classify_statement("LOCKING t FOR ACCESS SELECT *"), StatementSafety::ReadOnly);
    }

    #[test]
    fn test_classify_dml() {
        assert_eq!(classify_statement("INSERT INTO t VALUES (1)"), StatementSafety::Dml);
        assert_eq!(classify_statement("UPDATE t SET x=1"), StatementSafety::Dml);
        assert_eq!(classify_statement("DELETE FROM t"), StatementSafety::Dml);
        assert_eq!(classify_statement("MERGE INTO t USING s"), StatementSafety::Dml);
        assert_eq!(classify_statement("INS INTO t VALUES (1)"), StatementSafety::Dml);
    }

    #[test]
    fn test_classify_ddl() {
        assert_eq!(classify_statement("CREATE TABLE t (id INT)"), StatementSafety::Ddl);
        assert_eq!(classify_statement("DROP TABLE t"), StatementSafety::Ddl);
        assert_eq!(classify_statement("ALTER TABLE t ADD x INT"), StatementSafety::Ddl);
        assert_eq!(classify_statement("RENAME TABLE t TO t2"), StatementSafety::Ddl);
        assert_eq!(classify_statement("GRANT SELECT ON t TO u"), StatementSafety::Ddl);
        assert_eq!(classify_statement("REVOKE SELECT ON t FROM u"), StatementSafety::Ddl);
    }

    #[test]
    fn test_classify_with_comments() {
        assert_eq!(
            classify_statement("-- comment\nSELECT 1"),
            StatementSafety::ReadOnly
        );
        assert_eq!(
            classify_statement("/* block */ INSERT INTO t VALUES (1)"),
            StatementSafety::Dml
        );
    }

    #[test]
    fn test_validate_agent_safe_allows_select() {
        let args = make_agent_safe_args(false);
        assert!(validate_agent_safe("SELECT 1", &args).is_ok());
    }

    #[test]
    fn test_validate_agent_safe_blocks_insert() {
        let args = make_agent_safe_args(false);
        let err = validate_agent_safe("INSERT INTO t VALUES (1)", &args).unwrap_err();
        assert!(matches!(err, TqError::AgentSafeBlocked { .. }));
    }

    #[test]
    fn test_validate_agent_safe_blocks_ddl() {
        let args = make_agent_safe_args(false);
        let err = validate_agent_safe("DROP TABLE t", &args).unwrap_err();
        assert!(matches!(err, TqError::AgentSafeBlocked { .. }));
    }

    #[test]
    fn test_validate_agent_safe_allows_dml_with_flag() {
        let args = make_agent_safe_args(true);
        assert!(validate_agent_safe("INSERT INTO t VALUES (1)", &args).is_ok());
    }

    #[test]
    fn test_validate_agent_safe_blocks_ddl_even_with_allow_dml() {
        let args = make_agent_safe_args(true);
        let err = validate_agent_safe("CREATE TABLE t (id INT)", &args).unwrap_err();
        assert!(matches!(err, TqError::AgentSafeBlocked { .. }));
    }

    #[test]
    fn test_validate_agent_safe_blocks_multi_statement() {
        let args = make_agent_safe_args(false);
        let err = validate_agent_safe("SELECT 1; SELECT 2", &args).unwrap_err();
        assert!(matches!(err, TqError::AgentSafeBlocked { .. }));
    }

    /// Helper to create QueryArgs for agent-safe testing
    fn make_agent_safe_args(allow_dml: bool) -> QueryArgs {
        QueryArgs {
            query: Some("test".to_string()),
            file: None,
            format: OutputFormat::Json,
            output: None,
            no_header: false,
            timing: false,
            limit: None,
            atomic: false,
            agent_safe: true,
            max_rows: 10000,
            allow_dml,
            allow_maintenance: false, // Sprint 71: new field
            page_size: None,
            page: 1,
            json: false,
            dry_run: false,
        }
    }

    // Sprint 71: test_multiple_input_sources_error_message_content deleted —
    //   the "Multiple input sources" error is intentionally removed in Sprint 71.
    //   See TC110-I10 in tests/integration_tests.rs for the regression guard.
    // Sprint 71: test_stdin_has_data_does_not_panic deleted —
    //   stdin_has_data() is deleted in Sprint 71 (no longer needed).

    #[test]
    fn test_no_query_provided_error_message_content() {
        // Validates the error message shown when tq query is invoked with
        // no positional arg, no --file, and no readable stdin (e.g. running
        // `tq query < /dev/null` with no `query` argument). Ensures the
        // guidance stays actionable for CI users who hit this path.
        let msg = "No query provided.\n\n\
                   Provide SQL via:\n  \
                   - Command argument: tq query \"SELECT 1\"\n  \
                   - File: tq query --file script.sql\n  \
                   - Stdin: echo \"SELECT 1\" | tq query";

        assert!(msg.contains("No query provided"));
        assert!(msg.contains("Command argument"));
        assert!(msg.contains("--file"));
        assert!(msg.contains("echo"));
    }

    #[test]
    fn test_dry_run_query_substitution() {
        let args = QueryArgs {
            query: Some("SELECT * FROM {{table}} WHERE id = ${ID}".to_string()),
            file: None,
            format: OutputFormat::Json,
            output: None,
            no_header: false,
            timing: false,
            limit: None,
            atomic: false,
            agent_safe: false,
            max_rows: 10000,
            allow_dml: false,
            allow_maintenance: false,
            page_size: None,
            page: 1,
            json: false,
            dry_run: true,
        };

        let mut store = ParamStore::new();
        store.insert_define("table=employees").unwrap();
        unsafe {
            std::env::set_var("ID", "99");
        }

        let source = determine_input_source(&args).unwrap();
        let sql = read_input_sql(&source).unwrap();
        let substituted = store.substitute(&sql).unwrap();

        unsafe {
            std::env::remove_var("ID");
        }

        assert_eq!(substituted, "SELECT * FROM employees WHERE id = 99");
        assert!(args.dry_run);
    }
}
