use crate::connection::ConnectionConfig;
use crate::error::{Result, TqError};
use once_cell::sync::OnceCell;
use std::time::{Duration, Instant};

/// Global driver state to ensure the driver is only loaded once
static DRIVER_LOADED: OnceCell<()> = OnceCell::new();

/// A single row of query results
pub type Row = Vec<String>;

/// Query result set containing all rows
#[derive(Debug, Clone)]
pub struct QueryResults {
    pub rows: Vec<Row>,
}

impl QueryResults {
    /// Create a new QueryResults from a vector of rows
    pub fn new(rows: Vec<Row>) -> Self {
        Self { rows }
    }

    /// Check if the result set is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get the number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get an iterator over the rows
    pub fn iter(&self) -> std::slice::Iter<Row> {
        self.rows.iter()
    }
}

/// Database client for Teradata operations
pub struct DatabaseClient {
    config: ConnectionConfig,
    driver_lib_dir: String,
}

impl DatabaseClient {
    /// Create a new database client with the given configuration
    ///
    /// # Arguments
    /// * `config` - Connection configuration
    /// * `driver_lib_dir` - Optional directory containing the Teradata GoSQL driver library.
    ///   If None, defaults to the directory specified at build time
    ///
    /// # Errors
    /// Returns an error if the Teradata driver cannot be loaded
    ///
    /// # Example
    /// ```no_run
    /// use tq::{ConnectionConfig, DatabaseClient};
    ///
    /// let config = ConnectionConfig::parse(
    ///     "user:pass@host:1025/db",
    ///     "TD2",
    ///     None
    /// )?;
    /// let client = DatabaseClient::new(config, None)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(config: ConnectionConfig, driver_lib_dir: Option<String>) -> Result<Self> {
        // Default to the target directory where build.rs copies the library
        // This is baked in at compile time from the build script
        let default_dir = option_env!("TERADATA_LIB_DIR").unwrap_or(".");
        let driver_lib_dir = driver_lib_dir.unwrap_or_else(|| default_dir.to_string());

        let client = Self {
            config,
            driver_lib_dir,
        };

        // Ensure driver is loaded at construction time
        client.ensure_driver_loaded()?;

        Ok(client)
    }

    /// Load the Teradata driver (only once per process)
    ///
    /// # Thread Safety
    /// This function is thread-safe. The driver is loaded exactly once
    /// per process using `OnceCell`, with subsequent calls being no-ops.
    fn ensure_driver_loaded(&self) -> Result<()> {
        DRIVER_LOADED.get_or_try_init(|| {
            log::info!("Loading Teradata driver from: {}", self.driver_lib_dir);
            teradatarustapi::load_driver(&self.driver_lib_dir).map_err(|e| {
                TqError::Database(format!(
                    "Failed to load driver from '{}': {}. Ensure teradatasql library is present.",
                    self.driver_lib_dir, e
                ))
            })?;
            log::info!("Teradata driver loaded successfully");
            Ok(())
        })?;
        Ok(())
    }

    /// Ping the database to test connectivity
    ///
    /// This method establishes a connection, executes a simple query,
    /// and closes the connection to verify that the database is reachable.
    ///
    /// # Returns
    /// - `Ok(Duration)` with the round-trip latency if the ping succeeds
    /// - `Err(TqError)` if the connection or query fails
    ///
    /// # Example
    /// ```no_run
    /// # use tq::{ConnectionConfig, DatabaseClient};
    /// # let config = ConnectionConfig::parse("user:pass@host:1025/db", "TD2", None)?;
    /// let client = DatabaseClient::new(config, None)?;
    /// let latency = client.ping()?;
    /// println!("Ping: {:.2}ms", latency.as_secs_f64() * 1000.0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn ping(&self) -> Result<Duration> {
        log::debug!(
            "Pinging database at {}:{}",
            self.config.host,
            self.config.port
        );

        // Start timing
        let start = Instant::now();

        // Create connection using JSON parameters
        let connection_string = self.config.to_json_string();
        let (u_log, conn_handle) =
            teradatarustapi::create_connection(&connection_string).map_err(|e| {
                TqError::Connection {
                    host: self.config.host.clone(),
                    message: Self::parse_connection_error(&e),
                }
            })?;

        log::debug!("Connection established, executing ping query");

        // Execute a simple ping query
        let query = "SELECT 1 AS ping";
        let bind_values = "null"; // No bind parameters

        // Execute the query and handle cleanup on error
        let result = self.execute_ping_query(u_log, conn_handle, query, bind_values);

        // Always attempt to close the connection, even if query failed
        if let Err(e) = teradatarustapi::go_close_connection_wrapper(u_log, conn_handle) {
            log::warn!("Failed to close connection during ping cleanup: {}", e);
        }

        // Calculate elapsed time
        let elapsed = start.elapsed();

        // Return elapsed time if ping succeeded
        result.map(|_| elapsed)
    }

    /// Execute the ping query and handle result fetching
    fn execute_ping_query(
        &self,
        u_log: u64,
        conn_handle: u64,
        query: &str,
        bind_values: &str,
    ) -> Result<()> {
        let rows_handle =
            teradatarustapi::rustgo_create_rows_wrapper(u_log, conn_handle, query, bind_values)
                .map_err(|e| TqError::Database(format!("Ping query failed: {}", e)))?;

        // Fetch one row to verify the query executed successfully
        let _row_result = teradatarustapi::rustgo_fetch_row_wrapper(u_log, rows_handle)
            .map_err(|e| TqError::Database(format!("Failed to fetch ping result: {}", e)))?;

        // Close the result set
        teradatarustapi::go_close_rows_wrapper(u_log, rows_handle)
            .map_err(|e| TqError::Database(format!("Failed to close result set: {}", e)))?;

        Ok(())
    }

    /// Execute a SQL query and return the results
    ///
    /// This method establishes a connection, executes the query,
    /// fetches all results, and closes the connection.
    ///
    /// # Arguments
    /// * `query` - The SQL query to execute
    ///
    /// # Returns
    /// - `Ok(QueryResults)` with the query results
    /// - `Err(TqError)` if the connection or query fails
    ///
    /// # Example
    /// ```no_run
    /// # use tq::{ConnectionConfig, DatabaseClient};
    /// # let config = ConnectionConfig::parse("user:pass@host:1025/db", "TD2", None)?;
    /// let client = DatabaseClient::new(config, None)?;
    /// let results = client.execute_query("SELECT * FROM my_table")?;
    /// println!("Retrieved {} rows", results.row_count());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn execute_query(&self, query: &str) -> Result<QueryResults> {
        log::debug!(
            "Executing query on {}:{}",
            self.config.host,
            self.config.port
        );
        log::trace!("Query: {}", query);

        // Create connection using JSON parameters
        let connection_string = self.config.to_json_string();
        let (u_log, conn_handle) =
            teradatarustapi::create_connection(&connection_string).map_err(|e| {
                TqError::Connection {
                    host: self.config.host.clone(),
                    message: Self::parse_connection_error(&e),
                }
            })?;

        log::debug!("Connection established, executing query");

        // Execute the query and fetch results
        let result = self.execute_and_fetch_results(u_log, conn_handle, query);

        // Always attempt to close the connection, even if query failed
        if let Err(e) = teradatarustapi::go_close_connection_wrapper(u_log, conn_handle) {
            log::warn!("Failed to close connection during query cleanup: {}", e);
        }

        result
    }

    /// Execute query and fetch all results
    fn execute_and_fetch_results(
        &self,
        u_log: u64,
        conn_handle: u64,
        query: &str,
    ) -> Result<QueryResults> {
        let bind_values = "null"; // No bind parameters for now

        // Create result set
        let rows_handle =
            teradatarustapi::rustgo_create_rows_wrapper(u_log, conn_handle, query, bind_values)
                .map_err(|e| TqError::Database(format!("Query execution failed: {}", e)))?;

        // Fetch all rows
        let mut rows = Vec::new();
        while let Some(row_json) = teradatarustapi::rustgo_fetch_row_wrapper(u_log, rows_handle)
            .map_err(|e| TqError::Database(format!("Failed to fetch row: {}", e)))?
        {
            // Parse JSON array of column values
            let values: Vec<serde_json::Value> = serde_json::from_str(&row_json)
                .map_err(|e| TqError::Database(format!("Failed to parse row data: {}", e)))?;

            // Convert values to strings
            let row: Vec<String> = values
                .iter()
                .map(|v| match v {
                    serde_json::Value::Null => "NULL".to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                })
                .collect();

            rows.push(row);
        }

        log::debug!("Fetched {} rows", rows.len());

        // Close the result set
        teradatarustapi::go_close_rows_wrapper(u_log, rows_handle)
            .map_err(|e| TqError::Database(format!("Failed to close result set: {}", e)))?;

        Ok(QueryResults::new(rows))
    }

    /// Parse connection errors to provide more helpful messages
    fn parse_connection_error(error: &str) -> String {
        if error.contains("Connection refused") {
            format!("Connection refused. Ensure the Teradata database is running and accessible. Error: {}", error)
        } else if error.contains("timeout") {
            format!(
                "Connection timeout. Check network connectivity and firewall settings. Error: {}",
                error
            )
        } else if error.contains("Invalid credentials") || error.contains("Logon failed") {
            format!(
                "Authentication failed. Verify username and password. Error: {}",
                error
            )
        } else {
            format!("Connection failed: {}", error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_results() {
        let rows = vec![
            vec!["col1".to_string(), "col2".to_string()],
            vec!["val1".to_string(), "val2".to_string()],
        ];
        let results = QueryResults::new(rows.clone());

        assert_eq!(results.row_count(), 2);
        assert!(!results.is_empty());
        assert_eq!(results.rows, rows);
    }

    #[test]
    fn test_query_results_empty() {
        let results = QueryResults::new(vec![]);
        assert_eq!(results.row_count(), 0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_connection_error() {
        let error = DatabaseClient::parse_connection_error("Connection refused");
        assert!(error.contains("Ensure the Teradata database is running"));

        let error = DatabaseClient::parse_connection_error("timeout occurred");
        assert!(error.contains("network connectivity"));

        let error = DatabaseClient::parse_connection_error("Invalid credentials");
        assert!(error.contains("Verify username"));
    }
}
