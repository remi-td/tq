//! Database client for Teradata operations
//!
//! This module provides the main database client that wraps teradatarustapi
//! for executing queries and managing connections.

use crate::db::connection::ConnectionConfig;
use crate::db::types::{ColumnMetadata, QueryResult, Row, TeradataType, Value};
use crate::error::{Result, TqError};
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Global driver state to ensure the driver is only loaded once
static DRIVER_LOADED: OnceLock<()> = OnceLock::new();

/// Determine the platform-specific Teradata driver library filename.
///
/// Returns the filename (not path) of the native library for the current platform.
pub fn determine_library_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "teradatasql.dylib"
    } else if cfg!(target_os = "windows") {
        "teradatasql.dll"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        "teradatasql.arm.so"
    } else {
        "teradatasql.so"
    }
}

/// Resolve the directory containing the Teradata driver library.
///
/// Searches in priority order:
/// 1. Explicit CLI flag (`--driver-lib-dir`) if provided
/// 2. `TERADATA_LIB_DIR` environment variable (runtime, not compile-time)
/// 3. Directory containing the tq executable (`std::env::current_exe()` parent)
/// 4. Current working directory (`"."`)
///
/// For steps 2-3, the function verifies the library file actually exists in the
/// candidate directory before accepting it. Step 1 is trusted unconditionally
/// (the user explicitly specified it). Step 4 is the last-resort fallback.
///
/// Returns a tuple of (chosen_dir, all_searched_paths) where `all_searched_paths`
/// lists every directory that was checked (for error reporting).
pub fn resolve_driver_lib_dir(explicit_dir: Option<&str>) -> (String, Vec<String>) {
    let lib_name = determine_library_name();
    let mut searched = Vec::new();

    // 1. Explicit CLI flag takes highest priority (trusted unconditionally)
    if let Some(dir) = explicit_dir {
        searched.push(dir.to_string());
        return (dir.to_string(), searched);
    }

    // 2. Runtime environment variable
    if let Ok(dir) = std::env::var("TERADATA_LIB_DIR") {
        if !dir.is_empty() {
            searched.push(dir.clone());
            if Path::new(&dir).join(lib_name).exists() {
                return (dir, searched);
            }
        }
    }

    // 3. Executable's directory (primary path for installed binaries)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let dir_str = exe_dir.to_string_lossy().to_string();
            searched.push(dir_str.clone());
            if exe_dir.join(lib_name).exists() {
                return (dir_str, searched);
            }
        }
    }

    // 4. Current working directory (last resort)
    let cwd = ".".to_string();
    searched.push(cwd.clone());
    (cwd, searched)
}

/// Database client for Teradata operations
///
/// Follows one-shot execution model: each operation creates a new connection.
pub struct DatabaseClient {
    config: ConnectionConfig,
    driver_lib_dir: String,
    /// Directories that were searched during driver resolution (for error reporting)
    searched_paths: Vec<String>,
}

impl DatabaseClient {
    /// Create a mock database client for unit testing
    ///
    /// This client has valid config but will not be able to connect to any database.
    /// Used by unit tests that need a DatabaseClient reference without database access.
    #[cfg(test)]
    pub fn mock() -> Self {
        let config = ConnectionConfig {
            host: "mock".to_string(),
            port: 1025,
            user: "mock".to_string(),
            password: None,
            database: "mock".to_string(),
            logmech: crate::cli::LogonMechanism::Td2,
            timeout: std::time::Duration::from_secs(30),
        };
        Self {
            config,
            driver_lib_dir: ".".to_string(),
            searched_paths: vec![".".to_string()],
        }
    }

    /// Create a new database client
    ///
    /// # Arguments
    /// * `config` - Connection configuration
    /// * `driver_lib_dir` - Optional directory containing Teradata driver library
    pub fn new(config: ConnectionConfig, driver_lib_dir: Option<String>) -> Result<Self> {
        let (resolved_dir, searched_paths) =
            resolve_driver_lib_dir(driver_lib_dir.as_deref());

        let client = Self {
            config,
            driver_lib_dir: resolved_dir,
            searched_paths,
        };

        // Load driver at construction time
        client.ensure_driver_loaded()?;

        Ok(client)
    }

    /// Get a reference to the connection configuration
    pub fn config(&self) -> &ConnectionConfig {
        &self.config
    }

    /// Load the Teradata driver (thread-safe, only once per process)
    fn ensure_driver_loaded(&self) -> Result<()> {
        if DRIVER_LOADED.get().is_some() {
            return Ok(());
        }

        log::info!("Loading Teradata driver from: {}", self.driver_lib_dir);
        teradatarustapi::load_driver(&self.driver_lib_dir).map_err(|e| TqError::DriverLoad {
            path: self.driver_lib_dir.clone(),
            searched_paths: self.searched_paths.clone(),
            message: format!("{}. Ensure teradatasql library is present.", e),
        })?;
        log::info!("Teradata driver loaded successfully");

        let _ = DRIVER_LOADED.set(());
        Ok(())
    }

    /// Ping the database to test connectivity
    ///
    /// Returns the round-trip latency if successful.
    pub fn ping(&self) -> Result<Duration> {
        log::debug!(
            "Pinging database at {}:{}",
            self.config.host,
            self.config.port
        );

        let start = Instant::now();

        // Create connection
        let connection_string = self.config.to_json_string();
        let (u_log, conn_handle) = teradatarustapi::create_connection(&connection_string)
            .map_err(|e| self.map_connection_error(&e))?;

        log::debug!("Connection established, executing ping query");

        // Execute ping query
        let query = "SELECT 1 AS ping";
        let result = self.execute_ping_query(u_log, conn_handle, query);

        // Always close connection
        if let Err(e) = teradatarustapi::go_close_connection_wrapper(u_log, conn_handle) {
            log::warn!("Failed to close connection during ping cleanup: {}", e);
        }

        result.map(|_| start.elapsed())
    }

    /// Execute a SQL query and return buffered results
    ///
    /// Use this for queries where you need all data at once.
    /// For large result sets, consider streaming.
    pub fn execute(&self, sql: &str) -> Result<QueryResult> {
        log::debug!(
            "Executing query on {}:{}",
            self.config.host,
            self.config.port
        );
        log::trace!("Query: {}", sql);

        let start = Instant::now();

        // Create connection
        let connection_string = self.config.to_json_string();
        let (u_log, conn_handle) = teradatarustapi::create_connection(&connection_string)
            .map_err(|e| self.map_connection_error(&e))?;

        log::debug!("Connection established, executing query");

        // Execute and fetch results
        let result = self.execute_and_fetch(u_log, conn_handle, sql, start);

        // Always close connection
        if let Err(e) = teradatarustapi::go_close_connection_wrapper(u_log, conn_handle) {
            log::warn!("Failed to close connection during query cleanup: {}", e);
        }

        result
    }

    /// Execute a SQL query with a client-side row limit
    pub fn execute_with_limit(&self, sql: &str, limit: usize) -> Result<QueryResult> {
        log::debug!(
            "Executing query with limit {} on {}:{}",
            limit,
            self.config.host,
            self.config.port
        );
        log::trace!("Query: {}", sql);

        let start = Instant::now();

        // Create connection
        let connection_string = self.config.to_json_string();
        let (u_log, conn_handle) = teradatarustapi::create_connection(&connection_string)
            .map_err(|e| self.map_connection_error(&e))?;

        // Execute and fetch with limit
        let result = self.execute_and_fetch_limited(u_log, conn_handle, sql, limit, start);

        // Always close connection
        if let Err(e) = teradatarustapi::go_close_connection_wrapper(u_log, conn_handle) {
            log::warn!("Failed to close connection during query cleanup: {}", e);
        }

        result
    }

    /// Execute ping query
    fn execute_ping_query(&self, u_log: u64, conn_handle: u64, query: &str) -> Result<()> {
        let bind_values = "null";
        let rows_handle =
            teradatarustapi::rustgo_create_rows_wrapper(u_log, conn_handle, query, bind_values)
                .map_err(|e| TqError::PingFailed(format!("Query execution failed: {}", e)))?;

        // Fetch one row to verify
        let _row = teradatarustapi::rustgo_fetch_row_wrapper(u_log, rows_handle)
            .map_err(|e| TqError::PingFailed(format!("Failed to fetch result: {}", e)))?;

        // Close result set
        teradatarustapi::go_close_rows_wrapper(u_log, rows_handle)
            .map_err(|e| TqError::ResultSetClose(e.to_string()))?;

        Ok(())
    }

    /// Execute query and fetch all results
    fn execute_and_fetch(
        &self,
        u_log: u64,
        conn_handle: u64,
        sql: &str,
        start: Instant,
    ) -> Result<QueryResult> {
        let bind_values = "null";

        // Create result set
        let rows_handle =
            teradatarustapi::rustgo_create_rows_wrapper(u_log, conn_handle, sql, bind_values)
                .map_err(|e| self.map_query_error(&e, sql))?;

        // Get column metadata from the API
        let columns = self.fetch_column_metadata(u_log, rows_handle)?;

        // Fetch all rows with known column metadata
        let rows = self.fetch_all_rows(u_log, rows_handle, &columns)?;

        // Close result set
        teradatarustapi::go_close_rows_wrapper(u_log, rows_handle)
            .map_err(|e| TqError::ResultSetClose(e.to_string()))?;

        log::debug!("Fetched {} rows", rows.len());

        Ok(QueryResult::new(columns, rows, start.elapsed()))
    }

    /// Execute query and fetch with limit
    fn execute_and_fetch_limited(
        &self,
        u_log: u64,
        conn_handle: u64,
        sql: &str,
        limit: usize,
        start: Instant,
    ) -> Result<QueryResult> {
        let bind_values = "null";

        // Create result set
        let rows_handle =
            teradatarustapi::rustgo_create_rows_wrapper(u_log, conn_handle, sql, bind_values)
                .map_err(|e| self.map_query_error(&e, sql))?;

        // Get column metadata from the API
        let columns = self.fetch_column_metadata(u_log, rows_handle)?;

        // Fetch rows up to limit with known column metadata
        let rows = self.fetch_rows_limited(u_log, rows_handle, limit, &columns)?;

        // Close result set
        teradatarustapi::go_close_rows_wrapper(u_log, rows_handle)
            .map_err(|e| TqError::ResultSetClose(e.to_string()))?;

        log::debug!("Fetched {} rows (limit: {})", rows.len(), limit);

        Ok(QueryResult::new(columns, rows, start.elapsed()))
    }

    /// Fetch column metadata from the result set
    ///
    /// Uses rustgo_result_metadata_wrapper to get actual column names and types
    fn fetch_column_metadata(&self, u_log: u64, rows_handle: u64) -> Result<Vec<ColumnMetadata>> {
        let (_, _, _, column_metadata_json) =
            teradatarustapi::rustgo_result_metadata_wrapper(u_log, rows_handle)
                .map_err(|e| TqError::MetadataFetch(e.to_string()))?;

        log::debug!("Column metadata JSON: {}", column_metadata_json);

        self.parse_column_metadata(&column_metadata_json)
    }

    /// Parse column metadata JSON from teradatarustapi
    ///
    /// The Teradata API returns column-oriented data (map of arrays):
    /// ```json
    /// {
    ///   "ColumnName": ["test_col", "another_col"],
    ///   "TypeName": ["BYTEINT", "VARCHAR"],
    ///   "Nullable": [false, true],
    ///   "Precision": [3, 100],
    ///   "Scale": [0, 0],
    ///   "MaxByteCount": [1, 100]
    /// }
    /// ```
    fn parse_column_metadata(&self, metadata_json: &str) -> Result<Vec<ColumnMetadata>> {
        // Handle empty metadata (e.g., for DDL statements)
        if metadata_json.is_empty() || metadata_json == "null" || metadata_json == "{}" {
            return Ok(Vec::new());
        }

        // Teradata API returns column-oriented data (map of arrays)
        #[derive(serde::Deserialize)]
        struct MetadataMap {
            #[serde(rename = "ColumnName")]
            column_names: Vec<String>,
            #[serde(rename = "TypeName")]
            type_names: Vec<String>,
            #[serde(rename = "Nullable", default)]
            nullable: Vec<bool>,
        }

        let metadata_map: MetadataMap =
            serde_json::from_str(metadata_json).map_err(|e| TqError::MetadataParsing {
                message: format!("Failed to parse column metadata: {}", e),
            })?;

        // Verify array lengths match
        let num_columns = metadata_map.column_names.len();
        if metadata_map.type_names.len() != num_columns {
            return Err(TqError::MetadataParsing {
                message: format!(
                    "Metadata array length mismatch: {} column names but {} type names",
                    num_columns,
                    metadata_map.type_names.len()
                ),
            });
        }

        // Transpose from column-oriented to row-oriented format
        let columns: Vec<ColumnMetadata> = metadata_map
            .column_names
            .into_iter()
            .zip(metadata_map.type_names)
            .enumerate()
            .map(|(i, (name, type_name))| {
                let nullable = metadata_map.nullable.get(i).copied().unwrap_or(true);
                let data_type = map_type_name_to_teradata_type(&type_name);
                ColumnMetadata::new(name, data_type, nullable)
            })
            .collect();

        Ok(columns)
    }

    /// Fetch all rows from result set using pre-fetched column metadata
    fn fetch_all_rows(
        &self,
        u_log: u64,
        rows_handle: u64,
        columns: &[ColumnMetadata],
    ) -> Result<Vec<Row>> {
        let mut rows = Vec::new();
        let mut row_num = 0;

        while let Some(row_json) = teradatarustapi::rustgo_fetch_row_wrapper(u_log, rows_handle)
            .map_err(|e| TqError::RowFetch {
                row_num,
                message: e.to_string(),
            })?
        {
            // Parse JSON array
            let values: Vec<serde_json::Value> =
                serde_json::from_str(&row_json).map_err(|e| TqError::ResultParsing {
                    row_num,
                    message: e.to_string(),
                })?;

            // Convert to typed values using actual column metadata
            let row = self.convert_row(&values, columns)?;
            rows.push(row);
            row_num += 1;
        }

        Ok(rows)
    }

    /// Fetch rows up to limit using pre-fetched column metadata
    fn fetch_rows_limited(
        &self,
        u_log: u64,
        rows_handle: u64,
        limit: usize,
        columns: &[ColumnMetadata],
    ) -> Result<Vec<Row>> {
        let mut rows = Vec::new();
        let mut row_num = 0;

        while rows.len() < limit {
            match teradatarustapi::rustgo_fetch_row_wrapper(u_log, rows_handle) {
                Ok(Some(row_json)) => {
                    let values: Vec<serde_json::Value> =
                        serde_json::from_str(&row_json).map_err(|e| TqError::ResultParsing {
                            row_num,
                            message: e.to_string(),
                        })?;

                    let row = self.convert_row(&values, columns)?;
                    rows.push(row);
                    row_num += 1;
                }
                Ok(None) => break,
                Err(e) => {
                    return Err(TqError::RowFetch {
                        row_num,
                        message: e.to_string(),
                    })
                }
            }
        }

        Ok(rows)
    }

    /// Convert JSON values to typed Row
    fn convert_row(&self, values: &[serde_json::Value], columns: &[ColumnMetadata]) -> Result<Row> {
        values
            .iter()
            .zip(columns.iter())
            .map(|(v, col)| self.convert_value(v, col))
            .collect()
    }

    /// Convert single JSON value to Value
    fn convert_value(&self, v: &serde_json::Value, _col: &ColumnMetadata) -> Result<Value> {
        Ok(match v {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Decimal(f)
                } else {
                    Value::String(n.to_string())
                }
            }
            serde_json::Value::String(s) => {
                // Try to detect date/timestamp formats
                if Self::looks_like_date(s) {
                    Value::Date(s.clone())
                } else if Self::looks_like_timestamp(s) {
                    Value::Timestamp(s.clone())
                } else {
                    Value::String(s.clone())
                }
            }
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                Value::String(v.to_string())
            }
        })
    }

    /// Check if string looks like a date (YYYY-MM-DD)
    fn looks_like_date(s: &str) -> bool {
        if s.len() != 10 {
            return false;
        }
        let chars: Vec<char> = s.chars().collect();
        chars[4] == '-' && chars[7] == '-' && chars[..4].iter().all(|c| c.is_ascii_digit())
    }

    /// Check if string looks like a timestamp
    fn looks_like_timestamp(s: &str) -> bool {
        s.len() > 10 && (s.contains('T') || (s.contains('-') && s.contains(':')))
    }

    /// Map connection error to TqError
    fn map_connection_error(&self, error: &str) -> TqError {
        let clean_error = strip_go_stack_trace(error);
        let message = if error.contains("Connection refused") {
            format!(
                "Connection refused. Ensure the Teradata database is running. {}",
                clean_error
            )
        } else if error.contains("timeout") || error.contains("Timeout") {
            format!(
                "Connection timeout. Check network connectivity. {}",
                clean_error
            )
        } else if error.contains("Invalid credentials")
            || error.contains("Logon failed")
            || error.contains("Authentication")
        {
            return TqError::AuthenticationFailed {
                user: self.config.user.clone(),
                logmech: self.config.logmech.to_string(),
                source: Some(crate::error::string_to_error(clean_error)),
            };
        } else {
            clean_error
        };

        TqError::connection_failed(&self.config.host, self.config.port, message)
    }

    /// Map query error to TqError
    fn map_query_error(&self, error: &str, sql: &str) -> TqError {
        let error_lower = error.to_lowercase();
        let clean_error = strip_go_stack_trace(error);

        // Sprint 24: Detect transaction control errors due to session mode limitations
        // Error 3706 with specific messages indicates session mode restrictions
        if is_transaction_session_error(&error_lower, sql) {
            let operation = extract_transaction_operation(sql);
            let error_code = extract_error_code(error);

            return TqError::SessionModeTransactionError {
                operation,
                error_code,
                original_message: clean_error,
            };
        }

        if error_lower.contains("syntax") || error_lower.contains("parse") {
            TqError::SqlSyntaxError {
                message: clean_error,
                query: Some(sql.to_string()),
            }
        } else if error_lower.contains("does not exist") || error_lower.contains("not found") {
            // Try to extract table name
            TqError::TableNotFound {
                table: extract_table_name(sql).unwrap_or_else(|| "unknown".to_string()),
            }
        } else if error_lower.contains("permission") || error_lower.contains("privilege") {
            TqError::PermissionDenied(clean_error)
        } else {
            TqError::QueryExecution(clean_error)
        }
    }
}

/// Check if the error is a transaction control error due to session mode limitations
///
/// Sprint 24: REQ-SESSION-004 - Detect session mode transaction errors
fn is_transaction_session_error(error_lower: &str, sql: &str) -> bool {
    let sql_lower = sql.to_lowercase();

    // Check if SQL is a transaction control statement
    let is_transaction_statement = sql_lower.contains("commit")
        || sql_lower.contains("rollback")
        || sql_lower.contains("begin transaction")
        || sql_lower.contains("begin tran")
        // Teradata-specific shortcuts
        || sql_lower.trim() == "bt"
        || sql_lower.trim() == "et";

    if !is_transaction_statement {
        return false;
    }

    // Check for common session mode restriction error patterns
    // Error 3706 is "Syntax error" but often indicates session mode issues for transaction control
    // Error 3932 is "Transaction is not active" which can happen in wrong session modes
    let session_mode_patterns = [
        "not allowed",
        "not supported",
        "not valid",
        "not permitted",
        "invalid request",
        "cannot be issued",
        "3706", // Syntax error - often used for invalid commands in session context
        "3932", // Transaction not active
    ];

    session_mode_patterns
        .iter()
        .any(|pattern| error_lower.contains(pattern))
}

/// Extract the transaction operation from SQL for error reporting
fn extract_transaction_operation(sql: &str) -> String {
    let sql_upper = sql.to_uppercase().trim().to_string();

    if sql_upper.starts_with("COMMIT") || sql_upper == "ET" {
        "COMMIT".to_string()
    } else if sql_upper.starts_with("ROLLBACK") {
        "ROLLBACK".to_string()
    } else if sql_upper.starts_with("BEGIN TRANSACTION")
        || sql_upper.starts_with("BEGIN TRAN")
        || sql_upper == "BT"
    {
        "BEGIN TRANSACTION".to_string()
    } else {
        "Transaction control".to_string()
    }
}

/// Extract Teradata error code from error message (e.g., "[Error 3706]" -> Some(3706))
fn extract_error_code(error: &str) -> Option<u32> {
    // Look for patterns like "[Error 3706]" or "Error 3706"
    let patterns = ["[Error ", "Error "];

    for pattern in patterns {
        if let Some(start) = error.find(pattern) {
            let after_pattern = &error[start + pattern.len()..];
            // Find the end of the number (first non-digit or ']')
            let end = after_pattern
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(after_pattern.len());
            if end > 0 {
                if let Ok(code) = after_pattern[..end].parse::<u32>() {
                    return Some(code);
                }
            }
        }
    }
    None
}

/// Map Teradata type name string to TeradataType enum
fn map_type_name_to_teradata_type(type_name: &str) -> TeradataType {
    // Remove any length/precision info in parentheses, e.g., "VARCHAR(100)" -> "VARCHAR"
    let base_type = type_name
        .split('(')
        .next()
        .unwrap_or(type_name)
        .trim()
        .to_uppercase();

    match base_type.as_str() {
        "INTEGER" | "INT" | "I" => TeradataType::Integer,
        "BIGINT" | "I8" => TeradataType::BigInt,
        "SMALLINT" | "I2" => TeradataType::SmallInt,
        "BYTEINT" | "I1" => TeradataType::SmallInt, // Map BYTEINT to SmallInt
        "DECIMAL" | "NUMERIC" | "NUMBER" | "D" => TeradataType::Decimal,
        "FLOAT" | "DOUBLE" | "DOUBLE PRECISION" | "REAL" | "F" => TeradataType::Float,
        "CHAR" | "CHARACTER" | "CF" | "CV" => TeradataType::Char,
        "VARCHAR" | "CHARACTER VARYING" | "LONG VARCHAR" => TeradataType::Varchar,
        "DATE" | "DA" => TeradataType::Date,
        "TIME" | "AT" => TeradataType::Time,
        "TIMESTAMP" | "TS" | "TIMESTAMP WITH TIME ZONE" | "TIMESTAMP WITH ZONE" | "TZ" | "SZ" => {
            TeradataType::Timestamp
        }
        "BOOLEAN" | "BOOL" => TeradataType::Boolean,
        "BLOB" | "BINARY LARGE OBJECT" | "BF" | "BV" => TeradataType::Blob,
        "CLOB" | "CHARACTER LARGE OBJECT" => TeradataType::Clob,
        "JSON" | "JN" => TeradataType::Varchar, // JSON mapped to Varchar for display
        "XML" => TeradataType::Varchar,         // XML mapped to Varchar for display
        "INTERVAL" => TeradataType::Varchar,    // Intervals displayed as strings
        _ => {
            log::debug!(
                "Unknown Teradata type: {}, defaulting to Varchar",
                type_name
            );
            TeradataType::Unknown
        }
    }
}

/// Try to extract table name from SQL for error messages
fn extract_table_name(sql: &str) -> Option<String> {
    let sql_upper = sql.to_uppercase();
    let keywords = ["FROM", "INTO", "UPDATE", "TABLE"];

    for keyword in keywords {
        if let Some(pos) = sql_upper.find(keyword) {
            let after = &sql[pos + keyword.len()..].trim_start();
            let end = after
                .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                .unwrap_or(after.len());
            if end > 0 {
                return Some(after[..end].to_string());
            }
        }
    }
    None
}

/// Strip Go stack traces from Teradata driver error messages
///
/// The teradatasql driver (written in Go) includes full stack traces in error messages.
/// This function removes those traces to provide clean, user-friendly error messages.
///
/// Example input:
/// ```text
/// [Error 3707] Syntax error...
///  at gosqldriver/teradatasql.formatError ErrorUtil.go:101
///  at gosqldriver/teradatasql.(*teradataConnection).formatDatabaseError ErrorUtil.go:210
/// ```
///
/// Example output:
/// ```text
/// [Error 3707] Syntax error...
/// ```
fn strip_go_stack_trace(error: &str) -> String {
    // Find the first occurrence of Go stack trace marker
    if let Some(pos) = error.find("\n at ") {
        // Truncate at first stack frame
        error[..pos].trim().to_string()
    } else if let Some(pos) = error.find(" at gosqldriver") {
        // Handle case where stack trace doesn't have leading newline
        error[..pos].trim().to_string()
    } else {
        // No stack trace found, return as-is
        error.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_library_name_returns_platform_specific() {
        let name = determine_library_name();
        // Should return a non-empty filename
        assert!(!name.is_empty());
        // Should start with "teradatasql"
        assert!(name.starts_with("teradatasql"));
        // On macOS specifically
        if cfg!(target_os = "macos") {
            assert_eq!(name, "teradatasql.dylib");
        }
    }

    #[test]
    fn test_resolve_driver_lib_dir_explicit_override() {
        let (dir, searched) = resolve_driver_lib_dir(Some("/custom/path"));
        assert_eq!(dir, "/custom/path");
        assert_eq!(searched, vec!["/custom/path".to_string()]);
    }

    #[test]
    fn test_resolve_driver_lib_dir_fallback_to_cwd() {
        // With no explicit dir, no env var, and assuming library is not next to
        // the test binary, the function should eventually fall back to "."
        let saved = std::env::var("TERADATA_LIB_DIR").ok();
        std::env::remove_var("TERADATA_LIB_DIR");

        let (dir, searched) = resolve_driver_lib_dir(None);
        // Should have searched at least exe dir and cwd
        assert!(!searched.is_empty());
        // Last entry should be "." (cwd fallback)
        assert_eq!(searched.last().unwrap(), ".");
        // The resolved dir could be exe dir (if lib exists there) or "."
        assert!(!dir.is_empty());

        // Restore
        if let Some(val) = saved {
            std::env::set_var("TERADATA_LIB_DIR", val);
        }
    }

    #[test]
    fn test_looks_like_date() {
        assert!(DatabaseClient::looks_like_date("2024-01-15"));
        assert!(!DatabaseClient::looks_like_date("2024-1-15"));
        assert!(!DatabaseClient::looks_like_date("not a date"));
        assert!(!DatabaseClient::looks_like_date("2024-01-15T10:30:00"));
    }

    #[test]
    fn test_looks_like_timestamp() {
        assert!(DatabaseClient::looks_like_timestamp("2024-01-15T10:30:00"));
        assert!(DatabaseClient::looks_like_timestamp("2024-01-15 10:30:00"));
        assert!(!DatabaseClient::looks_like_timestamp("2024-01-15"));
        assert!(!DatabaseClient::looks_like_timestamp("hello"));
    }

    #[test]
    fn test_extract_table_name() {
        assert_eq!(
            extract_table_name("SELECT * FROM employees WHERE id = 1"),
            Some("employees".to_string())
        );
        assert_eq!(
            extract_table_name("INSERT INTO users VALUES (1)"),
            Some("users".to_string())
        );
        assert_eq!(
            extract_table_name("UPDATE orders SET status = 'done'"),
            Some("orders".to_string())
        );
    }

    #[test]
    fn test_map_type_name_to_teradata_type_integer_types() {
        assert!(matches!(
            map_type_name_to_teradata_type("INTEGER"),
            TeradataType::Integer
        ));
        assert!(matches!(
            map_type_name_to_teradata_type("INT"),
            TeradataType::Integer
        ));
        assert!(matches!(
            map_type_name_to_teradata_type("BIGINT"),
            TeradataType::BigInt
        ));
        assert!(matches!(
            map_type_name_to_teradata_type("SMALLINT"),
            TeradataType::SmallInt
        ));
    }

    #[test]
    fn test_map_type_name_to_teradata_type_string_types() {
        assert!(matches!(
            map_type_name_to_teradata_type("VARCHAR"),
            TeradataType::Varchar
        ));
        assert!(matches!(
            map_type_name_to_teradata_type("VARCHAR(100)"),
            TeradataType::Varchar
        ));
        assert!(matches!(
            map_type_name_to_teradata_type("CHAR"),
            TeradataType::Char
        ));
        assert!(matches!(
            map_type_name_to_teradata_type("CHAR(10)"),
            TeradataType::Char
        ));
    }

    #[test]
    fn test_map_type_name_to_teradata_type_date_time_types() {
        assert!(matches!(
            map_type_name_to_teradata_type("DATE"),
            TeradataType::Date
        ));
        assert!(matches!(
            map_type_name_to_teradata_type("TIME"),
            TeradataType::Time
        ));
        assert!(matches!(
            map_type_name_to_teradata_type("TIMESTAMP"),
            TeradataType::Timestamp
        ));
    }

    #[test]
    fn test_map_type_name_to_teradata_type_case_insensitive() {
        assert!(matches!(
            map_type_name_to_teradata_type("integer"),
            TeradataType::Integer
        ));
        assert!(matches!(
            map_type_name_to_teradata_type("Integer"),
            TeradataType::Integer
        ));
        assert!(matches!(
            map_type_name_to_teradata_type("VARCHAR"),
            TeradataType::Varchar
        ));
    }

    #[test]
    fn test_map_type_name_to_teradata_type_unknown() {
        assert!(matches!(
            map_type_name_to_teradata_type("CUSTOM_TYPE"),
            TeradataType::Unknown
        ));
    }

    // Helper to create a mock DatabaseClient for testing parse_column_metadata
    fn create_test_client() -> DatabaseClient {
        let config = ConnectionConfig {
            host: "test".to_string(),
            port: 1025,
            user: "test".to_string(),
            password: None,
            database: "test".to_string(),
            logmech: crate::cli::LogonMechanism::Td2,
            timeout: std::time::Duration::from_secs(30),
        };
        // Skip driver loading for unit tests
        DatabaseClient {
            config,
            driver_lib_dir: ".".to_string(),
            searched_paths: vec![".".to_string()],
        }
    }

    #[test]
    fn test_parse_column_metadata_map_of_arrays_format() {
        let client = create_test_client();

        // This is the actual format returned by the Teradata API
        let metadata_json = r#"{
            "ColumnName": ["test_col", "text_col"],
            "TypeName": ["BYTEINT", "VARCHAR"],
            "Nullable": [false, true]
        }"#;

        let columns = client.parse_column_metadata(metadata_json).unwrap();

        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0].name, "test_col");
        assert!(matches!(columns[0].data_type, TeradataType::SmallInt)); // BYTEINT maps to SmallInt
        assert!(!columns[0].nullable);

        assert_eq!(columns[1].name, "text_col");
        assert!(matches!(columns[1].data_type, TeradataType::Varchar));
        assert!(columns[1].nullable);
    }

    #[test]
    fn test_parse_column_metadata_empty_cases() {
        let client = create_test_client();

        // Empty string
        let columns = client.parse_column_metadata("").unwrap();
        assert!(columns.is_empty());

        // Null
        let columns = client.parse_column_metadata("null").unwrap();
        assert!(columns.is_empty());

        // Empty object
        let columns = client.parse_column_metadata("{}").unwrap();
        assert!(columns.is_empty());
    }

    #[test]
    fn test_parse_column_metadata_missing_nullable() {
        let client = create_test_client();

        // Nullable field is optional and should default to true
        let metadata_json = r#"{
            "ColumnName": ["col1"],
            "TypeName": ["INTEGER"]
        }"#;

        let columns = client.parse_column_metadata(metadata_json).unwrap();

        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0].name, "col1");
        assert!(columns[0].nullable); // Default to true when not specified
    }

    #[test]
    fn test_parse_column_metadata_multiple_columns() {
        let client = create_test_client();

        let metadata_json = r#"{
            "ColumnName": ["id", "name", "created_at", "amount"],
            "TypeName": ["INTEGER", "VARCHAR", "TIMESTAMP", "DECIMAL"],
            "Nullable": [false, true, true, true]
        }"#;

        let columns = client.parse_column_metadata(metadata_json).unwrap();

        assert_eq!(columns.len(), 4);

        assert_eq!(columns[0].name, "id");
        assert!(matches!(columns[0].data_type, TeradataType::Integer));
        assert!(!columns[0].nullable);

        assert_eq!(columns[1].name, "name");
        assert!(matches!(columns[1].data_type, TeradataType::Varchar));
        assert!(columns[1].nullable);

        assert_eq!(columns[2].name, "created_at");
        assert!(matches!(columns[2].data_type, TeradataType::Timestamp));
        assert!(columns[2].nullable);

        assert_eq!(columns[3].name, "amount");
        assert!(matches!(columns[3].data_type, TeradataType::Decimal));
        assert!(columns[3].nullable);
    }

    #[test]
    fn test_parse_column_metadata_mismatched_array_lengths() {
        let client = create_test_client();

        let metadata_json = r#"{
            "ColumnName": ["col1", "col2"],
            "TypeName": ["INTEGER"]
        }"#;

        let result = client.parse_column_metadata(metadata_json);
        assert!(result.is_err());

        if let Err(TqError::MetadataParsing { message }) = result {
            assert!(message.contains("mismatch"));
        } else {
            panic!("Expected MetadataParsing error");
        }
    }

    #[test]
    fn test_strip_go_stack_trace() {
        // Test with full Go stack trace
        let error_with_trace = "[Version 20.0.49] [Session 1429] [Teradata Database] [Error 3707] Syntax error\n at gosqldriver/teradatasql.formatError ErrorUtil.go:101\n at gosqldriver/teradatasql.(*teradataConnection).formatDatabaseError ErrorUtil.go:210";
        let cleaned = strip_go_stack_trace(error_with_trace);
        assert_eq!(
            cleaned,
            "[Version 20.0.49] [Session 1429] [Teradata Database] [Error 3707] Syntax error"
        );
        assert!(!cleaned.contains(" at "));
        assert!(!cleaned.contains("gosqldriver"));

        // Test with stack trace without leading newline
        let error_no_newline = "SQL error at gosqldriver/teradatasql.query Query.go:50";
        let cleaned = strip_go_stack_trace(error_no_newline);
        assert_eq!(cleaned, "SQL error");

        // Test error without stack trace
        let error_clean = "Simple error message";
        assert_eq!(strip_go_stack_trace(error_clean), "Simple error message");

        // Test empty error
        assert_eq!(strip_go_stack_trace(""), "");
    }

    // Sprint 24: Tests for transaction session error detection

    #[test]
    fn test_is_transaction_session_error_commit() {
        // COMMIT with "not allowed" should be detected
        assert!(is_transaction_session_error(
            "[error 3706] commit is not allowed for dbc/sql session",
            "COMMIT"
        ));

        // COMMIT with "not supported" should be detected
        assert!(is_transaction_session_error(
            "commit statement not supported",
            "COMMIT"
        ));

        // Regular query error should not be detected as session error
        assert!(!is_transaction_session_error(
            "syntax error in select",
            "SELECT * FROM t"
        ));
    }

    #[test]
    fn test_is_transaction_session_error_rollback() {
        assert!(is_transaction_session_error(
            "rollback not permitted in this session mode",
            "ROLLBACK"
        ));
    }

    #[test]
    fn test_is_transaction_session_error_begin_transaction() {
        assert!(is_transaction_session_error(
            "begin transaction cannot be issued",
            "BEGIN TRANSACTION"
        ));

        // Teradata shorthand BT
        assert!(is_transaction_session_error("error 3706: invalid request", "BT"));
    }

    #[test]
    fn test_is_transaction_session_error_non_transaction_sql() {
        // Error with "not allowed" but not a transaction statement
        assert!(!is_transaction_session_error(
            "select is not allowed",
            "SELECT * FROM t"
        ));

        // Transaction-like keywords in data should not trigger
        assert!(!is_transaction_session_error(
            "error",
            "INSERT INTO t (col) VALUES ('COMMIT')"
        ));
    }

    #[test]
    fn test_extract_transaction_operation() {
        assert_eq!(extract_transaction_operation("COMMIT"), "COMMIT");
        assert_eq!(extract_transaction_operation("commit;"), "COMMIT");
        assert_eq!(extract_transaction_operation("ROLLBACK"), "ROLLBACK");
        assert_eq!(
            extract_transaction_operation("BEGIN TRANSACTION"),
            "BEGIN TRANSACTION"
        );
        assert_eq!(
            extract_transaction_operation("BEGIN TRAN"),
            "BEGIN TRANSACTION"
        );
        assert_eq!(extract_transaction_operation("BT"), "BEGIN TRANSACTION");
        assert_eq!(extract_transaction_operation("ET"), "COMMIT");
        assert_eq!(
            extract_transaction_operation("SELECT 1"),
            "Transaction control"
        );
    }

    #[test]
    fn test_extract_error_code() {
        // Standard format
        assert_eq!(
            extract_error_code("[Error 3706] Syntax error"),
            Some(3706)
        );

        // Without brackets
        assert_eq!(
            extract_error_code("Error 3932 Transaction not active"),
            Some(3932)
        );

        // No error code
        assert_eq!(extract_error_code("Some other error message"), None);

        // Multiple occurrences (should return first)
        assert_eq!(
            extract_error_code("[Error 1234] followed by Error 5678"),
            Some(1234)
        );
    }
}
