//! REPL state management
//!
//! Tracks the current state of the interactive session including
//! input buffer, query count, session timing, and metadata cache.
//!
//! Sprint 7 additions:
//! - MetadataCache for table/column completion
//! - Connection string storage for /logon metacommand

use crate::db::{ConnectionConfig, MetadataCache, QueryResult};
use crate::params::ParamStore;
use std::fmt;
use std::time::Instant;

/// Pager activation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagerMode {
    /// Pager activates automatically when result is wider than terminal
    Auto,
    /// Pager is always used for qualifying result sets
    On,
    /// Pager is never used
    Off,
}

impl fmt::Display for PagerMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PagerMode::Auto => write!(f, "auto"),
            PagerMode::On => write!(f, "on"),
            PagerMode::Off => write!(f, "off"),
        }
    }
}

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

    /// Last SQL query executed (for /export full dataset - Sprint 12)
    last_sql: Option<String>,

    /// Whether last result was limited by default REPL limit (Sprint 12)
    was_limited: bool,

    /// Pager activation mode (auto/on/off)
    pager_mode: PagerMode,

    /// Whether colored output is enabled (Sprint 6)
    colors_enabled: bool,

    /// Metadata cache for tab completion (Sprint 7)
    metadata_cache: MetadataCache,

    /// Original connection string for reconnection (Sprint 7)
    connection_string: Option<String>,

    /// Default row limit for SELECT queries in REPL (Sprint 36: for /repeat)
    default_limit: usize,

    /// Parameter store for variable substitution (Sprint 40)
    pub params: ParamStore,
}

impl ReplState {
    /// Create a new REPL state
    pub fn new(connection_info: ConnectionConfig) -> Self {
        let database = connection_info.database.clone();
        Self {
            input_buffer: String::new(),
            session_start: Instant::now(),
            queries_executed: 0,
            total_rows: 0,
            connection_info,
            last_result: None,
            last_sql: None,
            was_limited: false,
            pager_mode: PagerMode::Auto,
            colors_enabled: atty::is(atty::Stream::Stdout), // Enable colors for TTY
            metadata_cache: MetadataCache::new(database),
            connection_string: None,
            default_limit: 0,
            params: ParamStore::new(),
        }
    }

    /// Create a new REPL state with connection string stored
    pub fn with_connection_string(
        connection_info: ConnectionConfig,
        connection_string: String,
    ) -> Self {
        let mut state = Self::new(connection_info);
        state.connection_string = Some(connection_string);
        state
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

    /// Set the last SQL query and whether it was limited (Sprint 12)
    pub fn set_last_query(&mut self, sql: String, was_limited: bool) {
        self.last_sql = Some(sql);
        self.was_limited = was_limited;
    }

    /// Get the last SQL query
    pub fn last_sql(&self) -> Option<&str> {
        self.last_sql.as_deref()
    }

    /// Check if the last result was limited by default REPL limit
    pub fn was_last_result_limited(&self) -> bool {
        self.was_limited
    }

    /// Set pager mode (auto/on/off)
    pub fn set_pager_mode(&mut self, mode: PagerMode) {
        self.pager_mode = mode;
    }

    /// Get current pager mode
    pub fn pager_mode(&self) -> PagerMode {
        self.pager_mode
    }

    /// Set colors enabled/disabled (Sprint 6)
    pub fn set_colors(&mut self, enabled: bool) {
        self.colors_enabled = enabled;
    }

    /// Check if colors are enabled (Sprint 6)
    pub fn are_colors_enabled(&self) -> bool {
        self.colors_enabled
    }

    // ========================================================================
    // Sprint 7: Metadata cache methods
    // ========================================================================

    /// Get mutable reference to metadata cache
    pub fn metadata_cache_mut(&mut self) -> &mut MetadataCache {
        &mut self.metadata_cache
    }

    /// Get reference to metadata cache
    pub fn metadata_cache(&self) -> &MetadataCache {
        &self.metadata_cache
    }

    /// Clear metadata cache (call on connection change)
    pub fn clear_metadata_cache(&mut self) {
        self.metadata_cache.clear();
    }

    /// Get stored connection string
    pub fn connection_string(&self) -> Option<&str> {
        self.connection_string.as_deref()
    }

    /// Set the default row limit for SELECT queries (Sprint 36: for /repeat)
    pub fn set_default_limit(&mut self, limit: usize) {
        self.default_limit = limit;
    }

    /// Get the default row limit for SELECT queries (Sprint 36: for /repeat)
    pub fn default_limit(&self) -> usize {
        self.default_limit
    }

    /// Update connection info (for /logon command)
    ///
    /// This clears the metadata cache and resets session-specific state
    /// while preserving settings like pager and colors.
    pub fn update_connection(
        &mut self,
        new_config: ConnectionConfig,
        new_connection_string: Option<String>,
    ) {
        // Clear metadata cache for new connection
        self.metadata_cache.clear();
        self.metadata_cache
            .set_current_database(&new_config.database);

        // Clear last result (not valid for new connection)
        self.last_result = None;

        // Update connection info
        self.connection_info = new_config;
        self.connection_string = new_connection_string;

        // Note: We preserve pager_mode and colors_enabled settings
        // Note: We preserve query count and session start for session statistics
        log::debug!("Connection updated, metadata cache cleared");
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

    #[test]
    fn test_metadata_cache_initialization() {
        let config = create_test_config();
        let state = ReplState::new(config);

        // Metadata cache should be initialized but empty
        assert!(!state.metadata_cache().has_tables());
    }

    #[test]
    fn test_with_connection_string() {
        let config = create_test_config();
        let state = ReplState::with_connection_string(config, "user:pass@host:1025/db".to_string());

        assert_eq!(state.connection_string(), Some("user:pass@host:1025/db"));
    }

    #[test]
    fn test_update_connection() {
        let config = create_test_config();
        let mut state = ReplState::new(config);

        // Set some state
        state.record_query(10);
        state.set_pager_mode(PagerMode::Off);
        state.set_colors(false);

        // Create new connection config
        let new_config = ConnectionConfig {
            host: "newhost".to_string(),
            port: 2025,
            database: "newdb".to_string(),
            user: "newuser".to_string(),
            password: None,
            logmech: LogonMechanism::Td2,
            timeout: Duration::from_secs(30),
        };

        state.update_connection(new_config, Some("newuser@newhost:2025/newdb".to_string()));

        // Connection info should be updated
        assert_eq!(state.connection_info().host, "newhost");
        assert_eq!(state.connection_info().database, "newdb");
        assert_eq!(
            state.connection_string(),
            Some("newuser@newhost:2025/newdb")
        );

        // Settings should be preserved
        assert_eq!(state.pager_mode(), PagerMode::Off);
        assert!(!state.are_colors_enabled());

        // Query count should be preserved (session-level)
        assert_eq!(state.queries_executed(), 1);
    }

    #[test]
    fn test_clear_metadata_cache() {
        let config = create_test_config();
        let mut state = ReplState::new(config);

        // Clear should not panic even if cache is empty
        state.clear_metadata_cache();
        assert!(!state.metadata_cache().has_tables());
    }

    #[test]
    fn test_pager_auto_by_default() {
        let config = create_test_config();
        let state = ReplState::new(config);

        assert_eq!(state.pager_mode(), PagerMode::Auto);
    }
}
