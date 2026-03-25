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
    DriverLoad {
        path: String,
        searched_paths: Vec<String>,
        message: String,
    },

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

    /// Transaction control not supported in current session mode
    ///
    /// Sprint 24: REQ-SESSION-004 - Better error messages for session limitations
    /// Teradata has different session types (ANSI, Teradata, BTEQ/DBC/SQL) with
    /// varying transaction control support. This error provides guidance when
    /// transaction operations fail due to session mode limitations.
    #[error("Transaction control not supported in current session mode")]
    SessionModeTransactionError {
        /// The attempted operation (e.g., "COMMIT", "BEGIN TRANSACTION")
        operation: String,
        /// Original error code if available (e.g., 3706)
        error_code: Option<u32>,
        /// Original error message from database
        original_message: String,
    },

    // ========================================================================
    // Internal Errors
    // ========================================================================
    /// SQL parse error (unterminated string, block comment, etc.)
    #[error("SQL parse error at line {line}, column {column}: {message}")]
    SqlParseError {
        message: String,
        line: usize,
        column: usize,
    },

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

            TqError::DriverLoad {
                path,
                searched_paths,
                message,
            } => {
                let searched_list = if searched_paths.is_empty() {
                    format!("  {}", path)
                } else {
                    searched_paths
                        .iter()
                        .map(|p| format!("  - {}", p))
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                format!(
                    "Error: Failed to load Teradata driver\n\n\
                     Path: {}\n\
                     Cause: {}\n\n\
                     Searched directories:\n{}\n\n\
                     Troubleshooting:\n  \
                     - Ensure the teradatasql library is in the same directory as the tq binary\n  \
                     - Override with: --driver-lib-dir /path/to/lib\n  \
                     - Or set: TERADATA_LIB_DIR=/path/to/lib\n  \
                     - Verify library file permissions",
                    path, message, searched_list
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

            TqError::SessionModeTransactionError {
                operation,
                error_code,
                original_message,
            } => {
                let error_code_str = error_code
                    .map(|c| format!(" [Error {}]", c))
                    .unwrap_or_default();

                format!(
                    "Error: Transaction control not supported{}\n\n\
                     {}\n\n\
                     Operation attempted: {}\n\n\
                     This error typically occurs when the session mode does not support\n\
                     explicit transaction control (e.g., DBC/SQL sessions via ODBC/JDBC).\n\n\
                     Troubleshooting:\n  \
                     - Verify the connection session mode supports transactions\n  \
                     - If using --atomic, try without it and manage transactions manually\n  \
                     - For ANSI mode databases, transactions are auto-committed by default\n  \
                     - Contact your DBA to verify session configuration\n\n\
                     Technical details:\n  \
                     Teradata has different session modes:\n  \
                     - ANSI mode: Auto-commit by default, explicit BEGIN required\n  \
                     - Teradata mode: Implicit transactions, COMMIT/ROLLBACK supported\n  \
                     - DBC/SQL (ODBC/JDBC): May restrict transaction control statements",
                    error_code_str, original_message, operation
                )
            }

            // Default: use the Display implementation
            _ => format!("Error: {}", self),
        }
    }

    /// Get a machine-readable error code for structured error output
    pub fn error_code(&self) -> &'static str {
        match self {
            TqError::ConnectionFailed { .. } => "CONNECTION_FAILED",
            TqError::ConnectionTimeout { .. } => "CONNECTION_TIMEOUT",
            TqError::AuthenticationFailed { .. } => "AUTH_FAILED",
            TqError::DriverLoad { .. } => "DRIVER_LOAD_FAILED",
            TqError::SqlSyntaxError { .. } => "SQL_SYNTAX_ERROR",
            TqError::QueryExecution(_) => "QUERY_EXECUTION_FAILED",
            TqError::TableNotFound { .. } => "OBJECT_NOT_FOUND",
            TqError::PermissionDenied(_) => "PERMISSION_DENIED",
            TqError::RowFetch { .. } => "ROW_FETCH_FAILED",
            TqError::ResultParsing { .. } => "RESULT_PARSING_FAILED",
            TqError::ResultSetClose(_) => "RESULT_SET_CLOSE_FAILED",
            TqError::PingFailed(_) => "PING_FAILED",
            TqError::MetadataFetch(_) => "METADATA_FETCH_FAILED",
            TqError::MetadataParsing { .. } => "METADATA_PARSING_FAILED",
            TqError::InvalidConnectionString(_) => "INVALID_ARGUMENT",
            TqError::InvalidConfig(_) => "INVALID_ARGUMENT",
            TqError::MissingPassword => "INVALID_ARGUMENT",
            TqError::ConfigParseError(_) => "INVALID_ARGUMENT",
            TqError::InvalidLogonMechanism(_) => "INVALID_ARGUMENT",
            TqError::InvalidDuration(_) => "INVALID_ARGUMENT",
            TqError::FileReadError { .. } => "IO_ERROR",
            TqError::FileWriteError { .. } => "IO_ERROR",
            TqError::IoError(_) => "IO_ERROR",
            TqError::FormatError(_) => "FORMAT_ERROR",
            TqError::JsonError(_) => "FORMAT_ERROR",
            TqError::CsvError(_) => "FORMAT_ERROR",
            TqError::TransactionError { .. } => "TRANSACTION_FAILED",
            TqError::AtomicConflict => "INVALID_ARGUMENT",
            TqError::SessionModeTransactionError { .. } => "TRANSACTION_FAILED",
            TqError::SqlParseError { .. } => "SQL_PARSE_ERROR",
            TqError::InternalError(_) => "INTERNAL_ERROR",
        }
    }

    /// Get the error category for structured error output
    pub fn error_category(&self) -> &'static str {
        match self {
            TqError::ConnectionFailed { .. }
            | TqError::ConnectionTimeout { .. }
            | TqError::DriverLoad { .. } => "connection",
            TqError::AuthenticationFailed { .. } => "auth",
            TqError::PermissionDenied(_) => "authz",
            TqError::SqlSyntaxError { .. }
            | TqError::QueryExecution(_)
            | TqError::TableNotFound { .. }
            | TqError::PingFailed(_)
            | TqError::SqlParseError { .. } => "query",
            TqError::RowFetch { .. }
            | TqError::ResultParsing { .. }
            | TqError::ResultSetClose(_)
            | TqError::MetadataFetch(_)
            | TqError::MetadataParsing { .. } => "result",
            TqError::InvalidConnectionString(_)
            | TqError::InvalidConfig(_)
            | TqError::MissingPassword
            | TqError::ConfigParseError(_)
            | TqError::InvalidLogonMechanism(_)
            | TqError::InvalidDuration(_)
            | TqError::AtomicConflict => "config",
            TqError::FileReadError { .. }
            | TqError::FileWriteError { .. }
            | TqError::IoError(_) => "io",
            TqError::FormatError(_)
            | TqError::JsonError(_)
            | TqError::CsvError(_) => "format",
            TqError::TransactionError { .. }
            | TqError::SessionModeTransactionError { .. } => "transaction",
            TqError::InternalError(_) => "internal",
        }
    }

    /// Whether this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            TqError::ConnectionFailed { .. }
                | TqError::ConnectionTimeout { .. }
                | TqError::PingFailed(_)
                | TqError::IoError(_)
        )
    }

    /// Get a short hint for troubleshooting (for structured error output)
    pub fn hint(&self) -> Option<&'static str> {
        match self {
            TqError::ConnectionFailed { .. } => {
                Some("Check hostname, port, and network connectivity")
            }
            TqError::ConnectionTimeout { .. } => Some("Increase --timeout or check network"),
            TqError::AuthenticationFailed { .. } => {
                Some("Check username, password, and logon mechanism")
            }
            TqError::DriverLoad { .. } => {
                Some("Install Teradata ODBC driver or set --driver-lib-dir")
            }
            TqError::PermissionDenied(_) => Some("Request appropriate GRANT from DBA"),
            TqError::TableNotFound { .. } => Some("Check object name and database qualification"),
            TqError::MissingPassword => {
                Some("Use --password-file, TQ_PASSWORD env var, or interactive prompt")
            }
            TqError::InvalidConnectionString(_) => {
                Some("Expected format: user:password@host:port/database")
            }
            _ => None,
        }
    }

    /// Format this error as a JSON object string for structured error output
    pub fn to_json(&self) -> String {
        let hint_json = match self.hint() {
            Some(h) => format!(",\"hint\":\"{}\"", h.replace('"', "\\\"")),
            None => String::new(),
        };
        format!(
            "{{\"ok\":false,\"error\":{{\"code\":\"{}\",\"category\":\"{}\",\"retryable\":{},\"message\":\"{}\"{}}}}}",
            self.error_code(),
            self.error_category(),
            self.is_retryable(),
            self.to_string().replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"),
            hint_json
        )
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
    Box::new(std::io::Error::other(s))
}

impl From<crate::sql::ParseError> for TqError {
    fn from(e: crate::sql::ParseError) -> Self {
        TqError::SqlParseError {
            message: e.message,
            line: e.line,
            column: e.column,
        }
    }
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

    #[test]
    fn test_user_message_session_mode_transaction_error() {
        let err = TqError::SessionModeTransactionError {
            operation: "COMMIT".to_string(),
            error_code: Some(3706),
            original_message: "COMMIT is not allowed for DBC/SQL session".to_string(),
        };
        let msg = err.user_message();

        // Should contain the operation
        assert!(msg.contains("COMMIT"));

        // Should contain the error code
        assert!(msg.contains("3706"));

        // Should contain the original message
        assert!(msg.contains("DBC/SQL session"));

        // Should contain troubleshooting guidance
        assert!(msg.contains("Troubleshooting"));
        assert!(msg.contains("session mode"));
    }

    #[test]
    fn test_user_message_session_mode_transaction_error_no_code() {
        let err = TqError::SessionModeTransactionError {
            operation: "BEGIN TRANSACTION".to_string(),
            error_code: None,
            original_message: "Transaction control not supported".to_string(),
        };
        let msg = err.user_message();

        // Should contain the operation
        assert!(msg.contains("BEGIN TRANSACTION"));

        // Should not contain error code format since None
        assert!(!msg.contains("[Error "));

        // Should still contain troubleshooting
        assert!(msg.contains("Troubleshooting"));
    }

    #[test]
    fn test_sql_parse_error_struct_variant() {
        let err = TqError::SqlParseError {
            message: "unterminated string".to_string(),
            line: 3,
            column: 7,
        };
        let display = format!("{}", err);
        assert!(display.contains("line 3"));
        assert!(display.contains("column 7"));
        assert!(display.contains("unterminated string"));
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn test_sql_parse_error_from_parse_error() {
        let parse_err = crate::sql::ParseError {
            message: "unterminated block comment".to_string(),
            line: 5,
            column: 12,
        };
        let tq_err: TqError = parse_err.into();
        match tq_err {
            TqError::SqlParseError {
                message,
                line,
                column,
            } => {
                assert_eq!(message, "unterminated block comment");
                assert_eq!(line, 5);
                assert_eq!(column, 12);
            }
            other => panic!("Expected SqlParseError, got: {:?}", other),
        }
    }

    #[test]
    fn test_session_mode_error_exit_code() {
        let err = TqError::SessionModeTransactionError {
            operation: "COMMIT".to_string(),
            error_code: Some(3706),
            original_message: "test".to_string(),
        };
        // Runtime errors should return exit code 1
        assert_eq!(err.exit_code(), 1);
    }

    // Sprint 53: Structured error classification tests

    #[test]
    fn test_error_codes() {
        assert_eq!(
            TqError::connection_failed("host", 1025, "refused").error_code(),
            "CONNECTION_FAILED"
        );
        assert_eq!(
            TqError::AuthenticationFailed {
                user: "u".into(),
                logmech: "TD2".into(),
                source: None
            }
            .error_code(),
            "AUTH_FAILED"
        );
        assert_eq!(
            TqError::PermissionDenied("test".into()).error_code(),
            "PERMISSION_DENIED"
        );
        assert_eq!(
            TqError::TableNotFound {
                table: "t".into()
            }
            .error_code(),
            "OBJECT_NOT_FOUND"
        );
        assert_eq!(
            TqError::QueryExecution("test".into()).error_code(),
            "QUERY_EXECUTION_FAILED"
        );
        assert_eq!(
            TqError::InvalidConnectionString("test".into()).error_code(),
            "INVALID_ARGUMENT"
        );
        assert_eq!(
            TqError::InternalError("test".into()).error_code(),
            "INTERNAL_ERROR"
        );
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(
            TqError::connection_failed("h", 1, "e").error_category(),
            "connection"
        );
        assert_eq!(
            TqError::AuthenticationFailed {
                user: "u".into(),
                logmech: "TD2".into(),
                source: None
            }
            .error_category(),
            "auth"
        );
        assert_eq!(
            TqError::PermissionDenied("t".into()).error_category(),
            "authz"
        );
        assert_eq!(
            TqError::QueryExecution("t".into()).error_category(),
            "query"
        );
        assert_eq!(
            TqError::InvalidConfig("t".into()).error_category(),
            "config"
        );
        assert_eq!(
            TqError::FormatError("t".into()).error_category(),
            "format"
        );
    }

    #[test]
    fn test_retryable() {
        assert!(TqError::connection_failed("h", 1, "e").is_retryable());
        assert!(TqError::ConnectionTimeout {
            timeout: Duration::from_secs(30)
        }
        .is_retryable());
        assert!(TqError::PingFailed("e".into()).is_retryable());
        assert!(!TqError::PermissionDenied("t".into()).is_retryable());
        assert!(!TqError::QueryExecution("t".into()).is_retryable());
        assert!(!TqError::InvalidConfig("t".into()).is_retryable());
    }

    #[test]
    fn test_hint() {
        assert!(TqError::connection_failed("h", 1, "e").hint().is_some());
        assert!(TqError::PermissionDenied("t".into()).hint().is_some());
        assert!(TqError::MissingPassword.hint().is_some());
        assert!(TqError::QueryExecution("t".into()).hint().is_none());
    }

    #[test]
    fn test_to_json() {
        let err = TqError::PermissionDenied("SELECT on DBC.Tables".into());
        let json = err.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["ok"], false);
        assert_eq!(parsed["error"]["code"], "PERMISSION_DENIED");
        assert_eq!(parsed["error"]["category"], "authz");
        assert_eq!(parsed["error"]["retryable"], false);
        assert!(parsed["error"]["message"]
            .as_str()
            .unwrap()
            .contains("SELECT on DBC.Tables"));
        assert!(parsed["error"]["hint"].as_str().is_some());
    }

    #[test]
    fn test_to_json_no_hint() {
        let err = TqError::QueryExecution("some error".into());
        let json = err.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["ok"], false);
        assert_eq!(parsed["error"]["code"], "QUERY_EXECUTION_FAILED");
        // No hint field when hint() returns None
        assert!(parsed["error"]["hint"].is_null());
    }

    #[test]
    fn test_to_json_with_special_chars() {
        let err = TqError::QueryExecution("error with \"quotes\" and \\backslash".into());
        let json = err.to_json();
        // Should be valid JSON even with special chars
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["ok"], false);
        assert!(parsed["error"]["message"]
            .as_str()
            .unwrap()
            .contains("quotes"));
    }
}
