//! Database client for Teradata operations
//!
//! This module provides the main database client that wraps teradatarustapi
//! for executing queries and managing connections.

use crate::db::connection::ConnectionConfig;
use crate::db::types::{ColumnMetadata, QueryResult, Row, TeradataType, Value};
use crate::error::{Result, TqError};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Global driver state to ensure the driver is only loaded once
static DRIVER_LOADED: OnceLock<()> = OnceLock::new();

/// Database client for Teradata operations
///
/// Follows one-shot execution model: each operation creates a new connection.
pub struct DatabaseClient {
    config: ConnectionConfig,
    driver_lib_dir: String,
}

impl DatabaseClient {
    /// Create a new database client
    ///
    /// # Arguments
    /// * `config` - Connection configuration
    /// * `driver_lib_dir` - Optional directory containing Teradata driver library
    pub fn new(config: ConnectionConfig, driver_lib_dir: Option<String>) -> Result<Self> {
        // Default to the target directory from build time
        let default_dir = option_env!("TERADATA_LIB_DIR").unwrap_or(".");
        let driver_lib_dir = driver_lib_dir.unwrap_or_else(|| default_dir.to_string());

        let client = Self {
            config,
            driver_lib_dir,
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
        log::debug!("Pinging database at {}:{}", self.config.host, self.config.port);

        let start = Instant::now();

        // Create connection
        let connection_string = self.config.to_json_string();
        let (u_log, conn_handle) =
            teradatarustapi::create_connection(&connection_string).map_err(|e| {
                self.map_connection_error(&e)
            })?;

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
        log::debug!("Executing query on {}:{}", self.config.host, self.config.port);
        log::trace!("Query: {}", sql);

        let start = Instant::now();

        // Create connection
        let connection_string = self.config.to_json_string();
        let (u_log, conn_handle) =
            teradatarustapi::create_connection(&connection_string).map_err(|e| {
                self.map_connection_error(&e)
            })?;

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
        let (u_log, conn_handle) =
            teradatarustapi::create_connection(&connection_string).map_err(|e| {
                self.map_connection_error(&e)
            })?;

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

        // Fetch all rows
        let (columns, rows) = self.fetch_all_rows(u_log, rows_handle)?;

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

        // Fetch rows up to limit
        let (columns, rows) = self.fetch_rows_limited(u_log, rows_handle, limit)?;

        // Close result set
        teradatarustapi::go_close_rows_wrapper(u_log, rows_handle)
            .map_err(|e| TqError::ResultSetClose(e.to_string()))?;

        log::debug!("Fetched {} rows (limit: {})", rows.len(), limit);

        Ok(QueryResult::new(columns, rows, start.elapsed()))
    }

    /// Fetch all rows from result set
    fn fetch_all_rows(
        &self,
        u_log: u64,
        rows_handle: u64,
    ) -> Result<(Vec<ColumnMetadata>, Vec<Row>)> {
        let mut rows = Vec::new();
        let mut row_num = 0;
        let mut columns: Option<Vec<ColumnMetadata>> = None;

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

            // Extract column metadata from first row
            if columns.is_none() {
                columns = Some(self.infer_columns(&values));
            }

            // Convert to typed values
            let row = self.convert_row(&values, columns.as_ref().unwrap())?;
            rows.push(row);
            row_num += 1;
        }

        Ok((columns.unwrap_or_default(), rows))
    }

    /// Fetch rows up to limit
    fn fetch_rows_limited(
        &self,
        u_log: u64,
        rows_handle: u64,
        limit: usize,
    ) -> Result<(Vec<ColumnMetadata>, Vec<Row>)> {
        let mut rows = Vec::new();
        let mut row_num = 0;
        let mut columns: Option<Vec<ColumnMetadata>> = None;

        while rows.len() < limit {
            match teradatarustapi::rustgo_fetch_row_wrapper(u_log, rows_handle) {
                Ok(Some(row_json)) => {
                    let values: Vec<serde_json::Value> =
                        serde_json::from_str(&row_json).map_err(|e| TqError::ResultParsing {
                            row_num,
                            message: e.to_string(),
                        })?;

                    if columns.is_none() {
                        columns = Some(self.infer_columns(&values));
                    }

                    let row = self.convert_row(&values, columns.as_ref().unwrap())?;
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

        Ok((columns.unwrap_or_default(), rows))
    }

    /// Infer column metadata from JSON values
    ///
    /// Since teradatarustapi returns JSON, we infer types from values
    fn infer_columns(&self, values: &[serde_json::Value]) -> Vec<ColumnMetadata> {
        values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let data_type = match v {
                    serde_json::Value::Null => TeradataType::Unknown,
                    serde_json::Value::Bool(_) => TeradataType::Boolean,
                    serde_json::Value::Number(n) => {
                        if n.is_i64() {
                            TeradataType::Integer
                        } else {
                            TeradataType::Decimal
                        }
                    }
                    serde_json::Value::String(_) => TeradataType::Varchar,
                    _ => TeradataType::Unknown,
                };
                ColumnMetadata::new(format!("col{}", i + 1), data_type, true)
            })
            .collect()
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
        let message = if error.contains("Connection refused") {
            format!(
                "Connection refused. Ensure the Teradata database is running. {}",
                error
            )
        } else if error.contains("timeout") || error.contains("Timeout") {
            format!(
                "Connection timeout. Check network connectivity. {}",
                error
            )
        } else if error.contains("Invalid credentials")
            || error.contains("Logon failed")
            || error.contains("Authentication")
        {
            return TqError::AuthenticationFailed {
                user: self.config.user.clone(),
                logmech: self.config.logmech.to_string(),
                source: Some(crate::error::string_to_error(error.to_string())),
            };
        } else {
            error.to_string()
        };

        TqError::connection_failed(&self.config.host, self.config.port, message)
    }

    /// Map query error to TqError
    fn map_query_error(&self, error: &str, sql: &str) -> TqError {
        let error_lower = error.to_lowercase();

        if error_lower.contains("syntax") || error_lower.contains("parse") {
            TqError::SqlSyntaxError {
                message: error.to_string(),
                query: Some(sql.to_string()),
            }
        } else if error_lower.contains("does not exist") || error_lower.contains("not found") {
            // Try to extract table name
            TqError::TableNotFound {
                table: extract_table_name(sql).unwrap_or_else(|| "unknown".to_string()),
            }
        } else if error_lower.contains("permission") || error_lower.contains("privilege") {
            TqError::PermissionDenied(error.to_string())
        } else {
            TqError::QueryExecution(error.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
