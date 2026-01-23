//! Error types for the tq CLI
//!
//! This module provides structured error handling with user-friendly messages
//! and actionable troubleshooting guidance.

use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Result type alias using TqError
pub type Result<T> = std::result::Result<T, TqError>;

/// Application error types with structured variants for different error categories
#[derive(Error, Debug)]
pub enum TqError {
    // ========================================================================
    // Connection Errors
    // ========================================================================
    /// Failed to establish connection to database
    #[error("Failed to connect to {host}:{port}")]
    ConnectionFailed {
        host: String,
        port: u16,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Connection timed out
    #[error("Connection timeout after {timeout:?}")]
    ConnectionTimeout { timeout: Duration },

    /// Authentication failed
    #[error("Authentication failed for user '{user}' using {logmech}")]
    AuthenticationFailed {
        user: String,
        logmech: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Failed to load the Teradata driver
    #[error("Failed to load Teradata driver from '{path}': {message}")]
    DriverLoad { path: String, message: String },

    // ========================================================================
    // Query Errors
    // ========================================================================
    /// SQL syntax error
    #[error("SQL syntax error: {message}")]
    SqlSyntaxError {
        message: String,
        query: Option<String>,
    },

    /// Query execution failed
    #[error("Query execution failed: {0}")]
    QueryExecution(String),

    /// Table does not exist
    #[error("Table '{table}' does not exist")]
    TableNotFound { table: String },

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Failed to fetch row
    #[error("Failed to fetch row {row_num}: {message}")]
    RowFetch { row_num: usize, message: String },

    /// Failed to parse result data
    #[error("Failed to parse result data at row {row_num}: {message}")]
    ResultParsing { row_num: usize, message: String },

    /// Failed to close result set
    #[error("Failed to close result set: {0}")]
    ResultSetClose(String),

    /// Ping failed
    #[error("Ping failed: {0}")]
    PingFailed(String),

    /// Failed to fetch column metadata
    #[error("Failed to fetch column metadata: {0}")]
    MetadataFetch(String),

    /// Failed to parse column metadata
    #[error("Failed to parse column metadata: {message}")]
    MetadataParsing { message: String },

    // ========================================================================
    // Configuration Errors
    // ========================================================================
    /// Invalid connection string format
    #[error("Invalid connection string: {0}")]
    InvalidConnectionString(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Missing password
    #[error("Missing password. Use --password-file, TQ_PASSWORD, or interactive prompt")]
    MissingPassword,

    /// Configuration file parse error
    #[error("Failed to parse configuration: {0}")]
    ConfigParseError(String),

    /// Invalid logon mechanism
    #[error("Invalid logon mechanism '{0}'. Supported: TD2, LDAP, KRB5, TDNEGO")]
    InvalidLogonMechanism(String),

    /// Invalid duration format
    #[error("Invalid duration format: {0}")]
    InvalidDuration(String),

    // ========================================================================
    // I/O Errors
    // ========================================================================
    /// Failed to read file
    #[error("Failed to read file '{}': {}", .path.display(), .source)]
    FileReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to write file
    #[error("Failed to write file '{}': {}", .path.display(), .source)]
    FileWriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// General I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    // ========================================================================
    // Formatting Errors
    // ========================================================================
    /// Output formatting error
    #[error("Formatting error: {0}")]
    FormatError(String),

    /// JSON serialization error
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// CSV writing error
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    // ========================================================================
    // Transaction Errors
    // ========================================================================
    /// Transaction operation failed
    #[error("Transaction {operation} failed: {message}")]
    TransactionError {
        operation: String,
        message: String,
    },

    /// Explicit transaction control conflicts with --atomic flag
    #[error("Cannot use --atomic with SQL containing explicit transaction control")]
    AtomicConflict,

    // ========================================================================
    // Internal Errors
    // ========================================================================
    /// Internal error (bug)
    #[error("Internal error: {0}\n\nThis is a bug. Please report it!")]
    InternalError(String),
}

impl TqError {
    /// Get the appropriate exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self {
            // Usage errors (exit code 2)
            TqError::InvalidConnectionString(_)
            | TqError::InvalidConfig(_)
            | TqError::InvalidLogonMechanism(_)
            | TqError::InvalidDuration(_)
            | TqError::ConfigParseError(_)
            | TqError::MissingPassword => 2,

            // Runtime errors (exit code 1)
            _ => 1,
        }
    }

    /// Get a user-friendly error message with context and troubleshooting hints
    pub fn user_message(&self) -> String {
        match self {
            TqError::ConnectionFailed { host, port, source } => {
                format!(
                    "Error: Failed to connect to {}:{}\n\n\
                     Cause: {}\n\n\
                     Possible causes:\n  \
                     - Database is not running\n  \
                     - Hostname or port is incorrect\n  \
                     - Firewall is blocking connection\n  \
                     - Network is unreachable\n\n\
                     Troubleshooting:\n  \
                     1. Verify hostname resolves: ping {}\n  \
                     2. Check port is open: nc -zv {} {}\n  \
                     3. Confirm credentials are correct\n  \
                     4. Check firewall rules",
                    host, port, source, host, host, port
                )
            }

            TqError::ConnectionTimeout { timeout } => {
                format!(
                    "Error: Connection timed out after {:?}\n\n\
                     Troubleshooting:\n  \
                     - Increase timeout: --timeout {}s\n  \
                     - Check network connectivity\n  \
                     - Verify database is responsive",
                    timeout,
                    timeout.as_secs() * 2
                )
            }

            TqError::AuthenticationFailed {
                user,
                logmech,
                source,
            } => {
                let source_msg = source
                    .as_ref()
                    .map(|s| format!("\n\nCause: {}", s))
                    .unwrap_or_default();
                format!(
                    "Error: Authentication failed\n\n\
                     User: {}\n\
                     Logon mechanism: {}{}\n\n\
                     Troubleshooting:\n  \
                     - Verify username and password are correct\n  \
                     - Check if account is locked\n  \
                     - Try different logon mechanism: --logmech LDAP",
                    user, logmech, source_msg
                )
            }

            TqError::DriverLoad { path, message } => {
                format!(
                    "Error: Failed to load Teradata driver\n\n\
                     Path: {}\n\
                     Cause: {}\n\n\
                     Troubleshooting:\n  \
                     - Ensure teradatasql library is installed\n  \
                     - Check library path: --driver-lib-dir /path/to/lib\n  \
                     - Verify library permissions",
                    path, message
                )
            }

            TqError::SqlSyntaxError { message, query } => {
                let query_display = query
                    .as_ref()
                    .map(|q| format!("\n\nQuery:\n  {}", q))
                    .unwrap_or_default();
                format!(
                    "Error: SQL syntax error\n\n\
                     {}{}\n\n\
                     Check your SQL syntax and try again.",
                    message, query_display
                )
            }

            TqError::TableNotFound { table } => {
                format!(
                    "Error: Table '{}' does not exist\n\n\
                     Suggestions:\n  \
                     - Check table name spelling\n  \
                     - Verify database context\n  \
                     - List tables: tq query \"SELECT TableName FROM DBC.TablesV WHERE DatabaseName = DATABASE\"",
                    table
                )
            }

            TqError::InvalidConnectionString(msg) => {
                format!(
                    "Error: Invalid connection string\n\n\
                     {}\n\n\
                     Expected format: user:password@host:port/database\n\n\
                     Examples:\n  \
                     alice:secret@dbhost:1025/mydb\n  \
                     alice@dbhost:1025/mydb  (password from file or prompt)",
                    msg
                )
            }

            TqError::MissingPassword => "Error: No password provided\n\n\
                 Provide password using one of:\n  \
                 - Include in connection string: user:password@host:port/db\n  \
                 - Password file: --password-file ~/.tq_passwords\n  \
                 - Environment variable: TQ_PASSWORD\n  \
                 - Interactive prompt (when stdin is a terminal)"
                .to_string(),

            TqError::FileReadError { path, source } => {
                format!(
                    "Error: Cannot read file '{}'\n\n\
                     Cause: {}\n\n\
                     Verify the file exists and you have read permissions.",
                    path.display(),
                    source
                )
            }

            TqError::TransactionError { operation, message } => {
                format!(
                    "Error: Transaction {} failed\n\n\
                     {}\n\n\
                     Note: When using --atomic, all changes are rolled back on error.\n\
                     Previous statements in this batch may have been undone.",
                    operation, message
                )
            }

            TqError::AtomicConflict => {
                "Error: Cannot use --atomic with explicit transaction control\n\n\
                 Your SQL contains BEGIN TRANSACTION, COMMIT, or ROLLBACK statements.\n\
                 The --atomic flag automatically wraps statements in a transaction.\n\n\
                 Either:\n  \
                 - Remove the --atomic flag and manage transactions manually, OR\n  \
                 - Remove BEGIN/COMMIT/ROLLBACK from your SQL and let --atomic handle it"
                    .to_string()
            }

            // Default: use the Display implementation
            _ => format!("Error: {}", self),
        }
    }

    /// Create a connection failed error from a string message
    pub fn connection_failed(
        host: impl Into<String>,
        port: u16,
        message: impl Into<String>,
    ) -> Self {
        TqError::ConnectionFailed {
            host: host.into(),
            port,
            source: string_to_error(message.into()),
        }
    }
}

/// Helper to convert a string into a boxed error (for use in TqError variants)
pub fn string_to_error(s: String) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::new(std::io::ErrorKind::Other, s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes() {
        assert_eq!(
            TqError::InvalidConnectionString("test".into()).exit_code(),
            2
        );
        assert_eq!(TqError::MissingPassword.exit_code(), 2);
        assert_eq!(TqError::QueryExecution("test".into()).exit_code(), 1);
        assert_eq!(TqError::PingFailed("test".into()).exit_code(), 1);
    }

    #[test]
    fn test_user_message_connection_failed() {
        let err = TqError::connection_failed("myhost", 1025, "Connection refused");
        let msg = err.user_message();
        assert!(msg.contains("myhost:1025"));
        assert!(msg.contains("Troubleshooting"));
    }

    #[test]
    fn test_user_message_invalid_connection_string() {
        let err = TqError::InvalidConnectionString("missing @".into());
        let msg = err.user_message();
        assert!(msg.contains("Expected format"));
        assert!(msg.contains("user:password@host:port/database"));
    }

    #[test]
    fn test_user_message_missing_password() {
        let err = TqError::MissingPassword;
        let msg = err.user_message();
        assert!(msg.contains("--password-file"));
        assert!(msg.contains("TQ_PASSWORD"));
    }
}
