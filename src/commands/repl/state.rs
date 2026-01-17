//! REPL state management
//!
//! Tracks the current state of the interactive session including
//! input buffer, query count, and session timing.

use crate::db::{ConnectionConfig, QueryResult};
use std::time::Instant;

/// State of the REPL session
#[derive(Debug)]
pub struct ReplState {
    /// Accumulated SQL input (for multi-line statements)
    input_buffer: String,

    /// Session start time
    session_start: Instant,

    /// Number of queries executed in this session
    queries_executed: usize,

    /// Total rows returned across all queries
    total_rows: usize,

    /// Connection configuration (for display purposes)
    connection_info: ConnectionConfig,

    /// Last query result (for /export command)
    last_result: Option<QueryResult>,

    /// Whether result paging is enabled (Sprint 6)
    pager_enabled: bool,

    /// Whether colored output is enabled (Sprint 6)
    colors_enabled: bool,
}

impl ReplState {
    /// Create a new REPL state
    pub fn new(connection_info: ConnectionConfig) -> Self {
        Self {
            input_buffer: String::new(),
            session_start: Instant::now(),
            queries_executed: 0,
            total_rows: 0,
            connection_info,
            last_result: None,
            pager_enabled: true,
            colors_enabled: atty::is(atty::Stream::Stdout), // Enable colors for TTY
        }
    }

    /// Check if there is any input in the buffer
    pub fn has_input(&self) -> bool {
        !self.input_buffer.trim().is_empty()
    }

    /// Check if we're in multi-line input mode
    pub fn is_multiline(&self) -> bool {
        self.has_input()
    }

    /// Get the current input buffer
    pub fn input_buffer(&self) -> &str {
        &self.input_buffer
    }

    /// Append a line to the input buffer
    pub fn append_input(&mut self, line: &str) {
        if !self.input_buffer.is_empty() {
            self.input_buffer.push('\n');
        }
        self.input_buffer.push_str(line);
    }

    /// Take the input buffer (clears it and returns the content)
    pub fn take_input(&mut self) -> String {
        std::mem::take(&mut self.input_buffer)
    }

    /// Clear the input buffer without returning it
    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
    }

    /// Record that a query was executed
    pub fn record_query(&mut self, rows: usize) {
        self.queries_executed += 1;
        self.total_rows += rows;
    }

    /// Get session duration
    pub fn session_duration(&self) -> std::time::Duration {
        self.session_start.elapsed()
    }

    /// Get number of queries executed
    pub fn queries_executed(&self) -> usize {
        self.queries_executed
    }

    /// Get total rows returned
    pub fn total_rows(&self) -> usize {
        self.total_rows
    }

    /// Get connection info
    pub fn connection_info(&self) -> &ConnectionConfig {
        &self.connection_info
    }

    /// Get session start time as a formatted string
    pub fn session_start_time(&self) -> String {
        // Calculate when session started based on elapsed time
        let now = chrono::Local::now();
        let duration = chrono::Duration::from_std(self.session_duration()).unwrap_or_default();
        let start = now - duration;
        start.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Set the last query result (Sprint 6: for /export)
    pub fn set_last_result(&mut self, result: QueryResult) {
        self.last_result = Some(result);
    }

    /// Take the last query result (consumes it)
    pub fn take_last_result(&mut self) -> Option<QueryResult> {
        self.last_result.take()
    }

    /// Get reference to last query result
    pub fn last_result(&self) -> Option<&QueryResult> {
        self.last_result.as_ref()
    }

    /// Set pager enabled/disabled (Sprint 6)
    pub fn set_pager(&mut self, enabled: bool) {
        self.pager_enabled = enabled;
    }

    /// Check if pager is enabled (Sprint 6)
    pub fn is_pager_enabled(&self) -> bool {
        self.pager_enabled
    }

    /// Set colors enabled/disabled (Sprint 6)
    pub fn set_colors(&mut self, enabled: bool) {
        self.colors_enabled = enabled;
    }

    /// Check if colors are enabled (Sprint 6)
    pub fn are_colors_enabled(&self) -> bool {
        self.colors_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::LogonMechanism;
    use std::time::Duration;

    fn create_test_config() -> ConnectionConfig {
        ConnectionConfig {
            host: "testhost".to_string(),
            port: 1025,
            database: "testdb".to_string(),
            user: "testuser".to_string(),
            password: None,
            logmech: LogonMechanism::Td2,
            timeout: Duration::from_secs(30),
        }
    }

    #[test]
    fn test_new_state() {
        let config = create_test_config();
        let state = ReplState::new(config);

        assert!(!state.has_input());
        assert!(!state.is_multiline());
        assert_eq!(state.queries_executed(), 0);
        assert_eq!(state.total_rows(), 0);
    }

    #[test]
    fn test_append_input() {
        let config = create_test_config();
        let mut state = ReplState::new(config);

        state.append_input("SELECT");
        assert!(state.has_input());
        assert_eq!(state.input_buffer(), "SELECT");

        state.append_input("  * FROM t;");
        assert_eq!(state.input_buffer(), "SELECT\n  * FROM t;");
    }

    #[test]
    fn test_take_input() {
        let config = create_test_config();
        let mut state = ReplState::new(config);

        state.append_input("SELECT 1;");
        let sql = state.take_input();

        assert_eq!(sql, "SELECT 1;");
        assert!(!state.has_input());
    }

    #[test]
    fn test_clear_input() {
        let config = create_test_config();
        let mut state = ReplState::new(config);

        state.append_input("SELECT");
        state.clear_input();

        assert!(!state.has_input());
    }

    #[test]
    fn test_record_query() {
        let config = create_test_config();
        let mut state = ReplState::new(config);

        state.record_query(10);
        state.record_query(5);

        assert_eq!(state.queries_executed(), 2);
        assert_eq!(state.total_rows(), 15);
    }
}
