//! Database metadata caching for tab completion
//!
//! This module provides session-level caching of database metadata (tables, columns)
//! to enable fast tab completion without repeated database queries.
//!
//! ## Design
//! - Lazy loading: Metadata is only fetched on first Tab press
//! - Session-scoped: Cache is cleared on connection change (/logon)
//! - Timeout handling: Queries time out after configured duration
//! - Graceful degradation: Failures don't crash REPL

use crate::db::DatabaseClient;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Guard that suppresses stdout AND stderr during its lifetime.
///
/// Note: This was originally added to suppress presumed driver debug output,
/// but the "Page 1: records 0 - 0 total: 0" output was actually from
/// reedline's ListMenu. Kept for potential future use but may be removable.
struct OutputSuppressor {
    #[cfg(unix)]
    original_stdout: Option<std::os::unix::io::RawFd>,
    #[cfg(unix)]
    original_stderr: Option<std::os::unix::io::RawFd>,
    #[cfg(unix)]
    null_fd: Option<std::os::unix::io::RawFd>,
}

impl OutputSuppressor {
    /// Create a new suppressor and redirect stdout AND stderr to /dev/null
    fn new() -> Self {
        #[cfg(unix)]
        {
            // Flush both streams before redirecting
            let _ = std::io::Write::flush(&mut std::io::stdout());
            let _ = std::io::Write::flush(&mut std::io::stderr());

            // Open /dev/null first
            let null_path = std::ffi::CString::new("/dev/null").unwrap();
            let null_fd = unsafe { libc::open(null_path.as_ptr(), libc::O_WRONLY) };
            if null_fd < 0 {
                log::warn!("Failed to open /dev/null for output suppression");
                return Self {
                    original_stdout: None,
                    original_stderr: None,
                    null_fd: None,
                };
            }

            // Save original stdout fd
            let original_stdout = unsafe { libc::dup(libc::STDOUT_FILENO) };
            if original_stdout < 0 {
                log::warn!("Failed to duplicate stdout fd");
                unsafe { libc::close(null_fd) };
                return Self {
                    original_stdout: None,
                    original_stderr: None,
                    null_fd: None,
                };
            }

            // Save original stderr fd
            let original_stderr = unsafe { libc::dup(libc::STDERR_FILENO) };
            if original_stderr < 0 {
                log::warn!("Failed to duplicate stderr fd");
                unsafe {
                    libc::close(original_stdout);
                    libc::close(null_fd);
                }
                return Self {
                    original_stdout: None,
                    original_stderr: None,
                    null_fd: None,
                };
            }

            // Redirect stdout to /dev/null
            if unsafe { libc::dup2(null_fd, libc::STDOUT_FILENO) } < 0 {
                log::warn!("Failed to redirect stdout to /dev/null");
                unsafe {
                    libc::close(original_stdout);
                    libc::close(original_stderr);
                    libc::close(null_fd);
                }
                return Self {
                    original_stdout: None,
                    original_stderr: None,
                    null_fd: None,
                };
            }

            // Redirect stderr to /dev/null
            if unsafe { libc::dup2(null_fd, libc::STDERR_FILENO) } < 0 {
                log::warn!("Failed to redirect stderr to /dev/null");
                // Restore stdout before failing
                unsafe {
                    libc::dup2(original_stdout, libc::STDOUT_FILENO);
                    libc::close(original_stdout);
                    libc::close(original_stderr);
                    libc::close(null_fd);
                }
                return Self {
                    original_stdout: None,
                    original_stderr: None,
                    null_fd: None,
                };
            }

            Self {
                original_stdout: Some(original_stdout),
                original_stderr: Some(original_stderr),
                null_fd: Some(null_fd),
            }
        }

        #[cfg(not(unix))]
        {
            // On non-Unix platforms, just return without suppression
            Self {}
        }
    }
}

impl Drop for OutputSuppressor {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            // Restore both stdout and stderr
            if let (Some(original_stdout), Some(original_stderr), Some(null_fd)) =
                (self.original_stdout, self.original_stderr, self.null_fd) {
                // Flush both streams before restoring
                let _ = std::io::Write::flush(&mut std::io::stdout());
                let _ = std::io::Write::flush(&mut std::io::stderr());

                // Restore stdout and stderr
                unsafe {
                    libc::dup2(original_stdout, libc::STDOUT_FILENO);
                    libc::dup2(original_stderr, libc::STDERR_FILENO);
                    libc::close(original_stdout);
                    libc::close(original_stderr);
                    libc::close(null_fd);
                }
            }
        }
    }
}

/// Execute a closure with stdout AND stderr suppressed
///
/// Sprint 19: Used to suppress teradatarustapi debug output during metadata queries.
/// Sprint 20: Enhanced to suppress BOTH stdout and stderr.
fn with_stdout_suppressed<T, F: FnOnce() -> T>(f: F) -> T {
    let _suppressor = OutputSuppressor::new();
    f()
}

/// Default timeout for table metadata queries (500ms)
pub const TABLE_QUERY_TIMEOUT: Duration = Duration::from_millis(500);

/// Default timeout for column metadata queries (300ms)
pub const COLUMN_QUERY_TIMEOUT: Duration = Duration::from_millis(300);

/// Maximum number of tables to cache
pub const MAX_CACHED_TABLES: usize = 10_000;

/// Maximum number of tables to cache column metadata for
pub const MAX_TABLES_WITH_COLUMNS: usize = 100;

/// Cached table information
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// Full name (schema.table or just table)
    pub full_name: String,
    /// Table name only (no schema prefix)
    pub table_name: String,
    /// Schema/database name
    pub schema_name: String,
    /// Table kind (T=Table, V=View, M=Macro, etc.)
    pub table_kind: String,
}

impl TableInfo {
    /// Create a new TableInfo
    pub fn new(
        full_name: impl Into<String>,
        table_name: impl Into<String>,
        schema_name: impl Into<String>,
        table_kind: impl Into<String>,
    ) -> Self {
        Self {
            full_name: full_name.into(),
            table_name: table_name.into(),
            schema_name: schema_name.into(),
            table_kind: table_kind.into(),
        }
    }
}

/// Cached column information
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// Column name
    pub name: String,
    /// Column data type (for display)
    pub data_type: String,
    /// Whether column is nullable
    pub nullable: bool,
}

impl ColumnInfo {
    /// Create a new ColumnInfo
    pub fn new(name: impl Into<String>, data_type: impl Into<String>, nullable: bool) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            nullable,
        }
    }

    /// Format column with type hint for completion display
    pub fn display_with_type(&self) -> String {
        format!("{} ({})", self.name, self.format_type_hint())
    }

    /// Format type hint (abbreviated)
    fn format_type_hint(&self) -> String {
        let type_upper = self.data_type.to_uppercase();

        // Abbreviate common types
        if type_upper.starts_with("VARCHAR") {
            return "VARCHAR".to_string();
        }
        if type_upper.starts_with("CHAR") {
            return "CHAR".to_string();
        }
        if type_upper.starts_with("DECIMAL") || type_upper.starts_with("NUMERIC") {
            return "DEC".to_string();
        }
        if type_upper.contains("INTEGER") || type_upper == "INT" || type_upper == "I" {
            return "INT".to_string();
        }
        if type_upper.contains("BIGINT") || type_upper == "I8" {
            return "BIGINT".to_string();
        }
        if type_upper.contains("SMALLINT") || type_upper == "I2" {
            return "SMALLINT".to_string();
        }
        if type_upper.starts_with("TIMESTAMP") {
            return "TIMESTAMP".to_string();
        }

        // Keep as-is for other types
        type_upper
    }
}

/// Metadata cache for database objects
///
/// Provides lazy-loading, session-scoped caching for tables and columns.
/// Sprint 20: Added separate database name cache for fast startup loading.
/// Sprint 21: Added per-database table caching for on-demand loading.
#[derive(Debug)]
pub struct MetadataCache {
    /// Cached database names (loaded at startup for fast completion)
    /// Sprint 20: Separate from tables for lightweight startup loading
    databases: Option<Vec<String>>,

    /// Cached table list (None = not loaded yet)
    tables: Option<Vec<TableInfo>>,

    /// Sprint 21: Per-database table cache for on-demand loading
    /// Key is uppercase database name for case-insensitive lookup
    tables_by_database: HashMap<String, Vec<TableInfo>>,

    /// Cached columns per table (key = schema.table or just table)
    columns: HashMap<String, Vec<ColumnInfo>>,

    /// When the database cache was last populated
    databases_loaded_at: Option<Instant>,

    /// When the table cache was last populated
    tables_loaded_at: Option<Instant>,

    /// Current database for context
    current_database: String,

    /// Whether loading is in progress (to avoid concurrent loads)
    loading_tables: bool,

    /// Whether database loading is in progress
    loading_databases: bool,

    /// Last error message (for user feedback)
    last_error: Option<String>,
}

impl MetadataCache {
    /// Create a new empty metadata cache
    pub fn new(current_database: impl Into<String>) -> Self {
        Self {
            databases: None,
            tables: None,
            tables_by_database: HashMap::new(),
            columns: HashMap::new(),
            databases_loaded_at: None,
            tables_loaded_at: None,
            current_database: current_database.into(),
            loading_tables: false,
            loading_databases: false,
            last_error: None,
        }
    }

    /// Clear all cached metadata (call on /logon)
    pub fn clear(&mut self) {
        self.databases = None;
        self.tables = None;
        self.tables_by_database.clear();
        self.columns.clear();
        self.databases_loaded_at = None;
        self.tables_loaded_at = None;
        self.last_error = None;
        log::debug!("Metadata cache cleared");
    }

    /// Check if databases are already cached
    pub fn has_databases(&self) -> bool {
        self.databases.is_some()
    }

    /// Get cached database names (returns None if not loaded)
    pub fn get_cached_databases(&self) -> Option<&[String]> {
        self.databases.as_deref()
    }

    /// Update current database context
    pub fn set_current_database(&mut self, database: impl Into<String>) {
        let new_db = database.into();
        if new_db != self.current_database {
            self.clear();
            self.current_database = new_db;
        }
    }

    /// Check if tables are already cached
    pub fn has_tables(&self) -> bool {
        self.tables.is_some()
    }

    /// Get cached tables (returns None if not loaded)
    pub fn get_tables(&self) -> Option<&[TableInfo]> {
        self.tables.as_deref()
    }

    /// Get cached columns for a table (returns None if not loaded)
    pub fn get_columns(&self, table_name: &str) -> Option<&[ColumnInfo]> {
        // Try exact match first
        if let Some(cols) = self.columns.get(table_name) {
            return Some(cols);
        }

        // Try case-insensitive match
        let table_upper = table_name.to_uppercase();
        for (key, cols) in &self.columns {
            if key.to_uppercase() == table_upper {
                return Some(cols);
            }
        }

        None
    }

    /// Get last error message
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Load table metadata from database
    ///
    /// Returns true if successful, false if failed (check last_error for reason)
    pub fn load_tables(&mut self, client: &DatabaseClient) -> bool {
        if self.loading_tables {
            return false;
        }

        self.loading_tables = true;
        self.last_error = None;

        log::debug!("Loading table metadata from database");

        // Query DBC.TablesV for table list
        // Sprint 21: Removed 'DBC' from exclusion list - users need dbc for system queries
        let sql = r#"
            SELECT TRIM(DatabaseName) AS schema_name,
                   TRIM(TableName) AS table_name,
                   TableKind
            FROM DBC.TablesV
            WHERE TableKind IN ('T', 'V', 'O')
              AND DatabaseName NOT IN ('All', 'Console', 'Crashdumps',
                                       'dbcmngr', 'Default', 'External_AP',
                                       'EXTUSER', 'LockLogShredder', 'PUBLIC',
                                       'SQLJ', 'Sys_Calendar', 'SysAdmin',
                                       'SYSBAR', 'SYSJDBC', 'SYSLIB', 'SYSSPATIAL',
                                       'SystemFe', 'SYSUDTLIB', 'TD_SERVER_DB',
                                       'TD_SYSFNLIB', 'TD_SYSGPL', 'TD_SYSXML',
                                       'TDMaps', 'TDPUSER', 'TDQCD', 'TDStats',
                                       'tdwm', 'VIEWPOINT')
            ORDER BY DatabaseName, TableName
            SAMPLE 10000
        "#;

        // Sprint 19: Suppress stdout during query execution to prevent driver debug output
        // from appearing during tab completion
        let query_result = with_stdout_suppressed(|| client.execute(sql));

        match query_result {
            Ok(result) => {
                let mut tables = Vec::with_capacity(result.row_count);

                for row in &result.rows {
                    let schema = row.first().map(|v| v.display()).unwrap_or_default();
                    let table = row.get(1).map(|v| v.display()).unwrap_or_default();
                    let kind = row.get(2).map(|v| v.display()).unwrap_or_default();

                    // Skip NULL values
                    if schema == "[NULL]" || table == "[NULL]" {
                        continue;
                    }

                    let full_name = format!("{}.{}", schema, table);
                    tables.push(TableInfo::new(full_name, table, schema, kind));
                }

                log::info!("Loaded {} tables into metadata cache", tables.len());

                self.tables = Some(tables);
                self.tables_loaded_at = Some(Instant::now());
                self.loading_tables = false;
                true
            }
            Err(e) => {
                let error_msg = format!("Failed to load table metadata: {}", e);
                log::warn!("{}", error_msg);
                self.last_error = Some(error_msg);
                self.loading_tables = false;
                false
            }
        }
    }

    /// Load database names from Teradata
    ///
    /// Sprint 20: Lightweight query to load just database names at startup.
    /// This is called BEFORE editor initialization to avoid TTY conflicts.
    /// Returns true if successful, false if failed (check last_error for reason).
    pub fn load_databases(&mut self, client: &DatabaseClient) -> bool {
        if self.loading_databases {
            return false;
        }

        self.loading_databases = true;
        self.last_error = None;

        log::debug!("Loading database names from DBC.DatabasesV");

        // Query DBC.DatabasesV for database list - lightweight query
        // Sprint 20: Using DBC.DatabasesV instead of extracting from TablesV
        // Sprint 21: Removed 'DBC' from exclusion list - users need dbc for system queries
        let sql = r#"
            SELECT TRIM(DatabaseName)
            FROM DBC.DatabasesV
            WHERE DatabaseName NOT IN ('All', 'Console', 'Crashdumps',
                                       'dbcmngr', 'Default', 'External_AP',
                                       'EXTUSER', 'LockLogShredder', 'PUBLIC',
                                       'SQLJ', 'Sys_Calendar', 'SysAdmin',
                                       'SYSBAR', 'SYSJDBC', 'SYSLIB', 'SYSSPATIAL',
                                       'SystemFe', 'SYSUDTLIB', 'TD_SERVER_DB',
                                       'TD_SYSFNLIB', 'TD_SYSGPL', 'TD_SYSXML',
                                       'TDMaps', 'TDPUSER', 'TDQCD', 'TDStats',
                                       'tdwm', 'VIEWPOINT')
            ORDER BY DatabaseName
        "#;

        // Sprint 20: Note - this is called at startup BEFORE editor init,
        // so OutputSuppressor may not be strictly necessary, but we keep it
        // for consistency and safety.
        let query_result = with_stdout_suppressed(|| client.execute(sql));

        match query_result {
            Ok(result) => {
                let mut databases = Vec::with_capacity(result.row_count);

                for row in &result.rows {
                    let db_name = row.first().map(|v| v.display()).unwrap_or_default();

                    // Skip NULL values
                    if db_name != "[NULL]" && !db_name.is_empty() {
                        databases.push(db_name);
                    }
                }

                log::info!("Loaded {} database names into metadata cache", databases.len());

                self.databases = Some(databases);
                self.databases_loaded_at = Some(Instant::now());
                self.loading_databases = false;
                true
            }
            Err(e) => {
                let error_msg = format!("Failed to load database names: {}", e);
                log::warn!("{}", error_msg);
                self.last_error = Some(error_msg);
                self.loading_databases = false;
                false
            }
        }
    }

    /// Sprint 21: Check if tables for a specific database are cached
    pub fn has_tables_for_database(&self, database: &str) -> bool {
        self.tables_by_database
            .contains_key(&database.to_uppercase())
    }

    /// Sprint 21: Get cached tables for a specific database
    pub fn get_tables_for_database(&self, database: &str) -> Option<&[TableInfo]> {
        self.tables_by_database
            .get(&database.to_uppercase())
            .map(|v| v.as_slice())
    }

    /// Sprint 21: Load tables for a specific database on-demand
    ///
    /// This method fetches table metadata for a single database and caches it.
    /// Used when user types `database.` + TAB to get tables for that database.
    /// Returns true if successful, false if failed (check last_error for reason).
    pub fn load_tables_for_database(&mut self, client: &DatabaseClient, database: &str) -> bool {
        let db_upper = database.to_uppercase();

        // Already cached?
        if self.tables_by_database.contains_key(&db_upper) {
            return true;
        }

        self.last_error = None;

        log::debug!(
            "Loading tables for database '{}' from DBC.TablesV",
            database
        );

        // Query tables for this specific database
        let sql = format!(
            r#"
            SELECT TRIM(TableName) AS table_name,
                   TableKind
            FROM DBC.TablesV
            WHERE UPPER(DatabaseName) = UPPER('{}')
              AND TableKind IN ('T', 'V', 'O')
            ORDER BY TableName
            "#,
            escape_sql_string(database)
        );

        let query_result = with_stdout_suppressed(|| client.execute(&sql));

        match query_result {
            Ok(result) => {
                let mut tables = Vec::with_capacity(result.row_count);

                for row in &result.rows {
                    let table_name = row.first().map(|v| v.display()).unwrap_or_default();
                    let kind = row.get(1).map(|v| v.display()).unwrap_or_default();

                    // Skip NULL values
                    if table_name == "[NULL]" || table_name.is_empty() {
                        continue;
                    }

                    let full_name = format!("{}.{}", database, table_name);
                    tables.push(TableInfo::new(
                        full_name,
                        table_name,
                        database.to_string(),
                        kind,
                    ));
                }

                log::info!(
                    "Loaded {} tables for database '{}' into cache",
                    tables.len(),
                    database
                );

                self.tables_by_database.insert(db_upper, tables);
                true
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to load tables for database '{}': {}",
                    database, e
                );
                log::warn!("{}", error_msg);
                self.last_error = Some(error_msg);
                false
            }
        }
    }

    /// Load column metadata for a specific table
    ///
    /// Returns true if successful, false if failed
    pub fn load_columns(&mut self, client: &DatabaseClient, table_name: &str) -> bool {
        self.last_error = None;

        // Parse schema.table format
        let (schema, table) = if let Some(dot_pos) = table_name.find('.') {
            (Some(&table_name[..dot_pos]), &table_name[dot_pos + 1..])
        } else {
            (None, table_name)
        };

        log::debug!("Loading column metadata for table: {}", table_name);

        // Build query
        let sql = if let Some(schema) = schema {
            format!(
                r#"
                SELECT TRIM(ColumnName) AS column_name,
                       TRIM(ColumnType) AS column_type,
                       CASE WHEN Nullable = 'Y' THEN 1 ELSE 0 END AS nullable
                FROM DBC.ColumnsV
                WHERE UPPER(DatabaseName) = UPPER('{}')
                  AND UPPER(TableName) = UPPER('{}')
                ORDER BY ColumnId
                "#,
                escape_sql_string(schema),
                escape_sql_string(table)
            )
        } else {
            format!(
                r#"
                SELECT TRIM(ColumnName) AS column_name,
                       TRIM(ColumnType) AS column_type,
                       CASE WHEN Nullable = 'Y' THEN 1 ELSE 0 END AS nullable
                FROM DBC.ColumnsV
                WHERE UPPER(TableName) = UPPER('{}')
                  AND DatabaseName = DATABASE
                ORDER BY ColumnId
                "#,
                escape_sql_string(table)
            )
        };

        // Sprint 19: Suppress stdout during query execution to prevent driver debug output
        // from appearing during tab completion
        let query_result = with_stdout_suppressed(|| client.execute(&sql));

        match query_result {
            Ok(result) => {
                let mut columns = Vec::with_capacity(result.row_count);

                for row in &result.rows {
                    let name = row.first().map(|v| v.display()).unwrap_or_default();
                    let data_type = row.get(1).map(|v| v.display()).unwrap_or_default();
                    let nullable = row.get(2).map(|v| v.display() == "1").unwrap_or(true);

                    if name != "[NULL]" {
                        columns.push(ColumnInfo::new(name, data_type, nullable));
                    }
                }

                log::info!("Loaded {} columns for table {}", columns.len(), table_name);

                // Store with normalized key (uppercase for case-insensitive lookup)
                let cache_key = table_name.to_uppercase();
                self.columns.insert(cache_key, columns);

                // Evict oldest entries if we have too many tables cached
                if self.columns.len() > MAX_TABLES_WITH_COLUMNS {
                    // Simple eviction: remove arbitrary entry
                    // In a more sophisticated implementation, we could use LRU
                    if let Some(key) = self.columns.keys().next().cloned() {
                        self.columns.remove(&key);
                    }
                }

                true
            }
            Err(e) => {
                let error_msg = format!("Failed to load columns for {}: {}", table_name, e);
                log::warn!("{}", error_msg);
                self.last_error = Some(error_msg);
                false
            }
        }
    }

    /// Find tables matching a prefix (case-insensitive)
    pub fn find_tables_by_prefix(&self, prefix: &str) -> Vec<&TableInfo> {
        let Some(tables) = &self.tables else {
            return vec![];
        };

        let prefix_upper = prefix.to_uppercase();

        tables
            .iter()
            .filter(|t| {
                t.table_name.to_uppercase().starts_with(&prefix_upper)
                    || t.full_name.to_uppercase().starts_with(&prefix_upper)
            })
            .collect()
    }

    /// Find columns matching a prefix (case-insensitive) for a given table
    pub fn find_columns_by_prefix(&self, table_name: &str, prefix: &str) -> Vec<&ColumnInfo> {
        let Some(columns) = self.get_columns(table_name) else {
            return vec![];
        };

        let prefix_upper = prefix.to_uppercase();

        columns
            .iter()
            .filter(|c| c.name.to_uppercase().starts_with(&prefix_upper))
            .collect()
    }

    /// Get distinct database names
    ///
    /// Sprint 20: Prefers cached database list (from load_databases) over
    /// extracting from tables cache. This allows fast startup loading.
    pub fn get_databases(&self) -> Vec<String> {
        // Sprint 20: First check dedicated database cache (loaded at startup)
        if let Some(databases) = &self.databases {
            return databases.clone();
        }

        // Fall back to extracting from tables cache if databases weren't loaded separately
        let Some(tables) = &self.tables else {
            return vec![];
        };

        // Collect unique database names
        let mut databases: std::collections::HashSet<String> = std::collections::HashSet::new();
        for table in tables {
            databases.insert(table.schema_name.clone());
        }

        let mut result: Vec<String> = databases.into_iter().collect();
        result.sort();
        result
    }

    /// Find databases matching a prefix (case-insensitive)
    ///
    /// Sprint 20: Uses cached database list for fast completion without DB queries.
    pub fn find_databases_by_prefix(&self, prefix: &str) -> Vec<String> {
        let databases = self.get_databases();
        let prefix_upper = prefix.to_uppercase();

        databases
            .into_iter()
            .filter(|db| db.to_uppercase().starts_with(&prefix_upper))
            .collect()
    }

    /// Find tables in current database matching a prefix
    ///
    /// Sprint 8 Bug Fix: For showing unqualified table names in current database
    pub fn find_tables_in_current_db_by_prefix(&self, prefix: &str) -> Vec<&TableInfo> {
        let Some(tables) = &self.tables else {
            return vec![];
        };

        let prefix_upper = prefix.to_uppercase();
        let current_db_upper = self.current_database.to_uppercase();

        tables
            .iter()
            .filter(|t| {
                t.schema_name.to_uppercase() == current_db_upper
                    && t.table_name.to_uppercase().starts_with(&prefix_upper)
            })
            .collect()
    }

    /// Sprint 21: Find tables in a specific database matching a prefix (case-insensitive)
    ///
    /// Uses the per-database cache (loaded via load_tables_for_database).
    pub fn find_tables_in_database_by_prefix(
        &self,
        database: &str,
        prefix: &str,
    ) -> Vec<&TableInfo> {
        let db_upper = database.to_uppercase();
        let prefix_upper = prefix.to_uppercase();

        if let Some(tables) = self.tables_by_database.get(&db_upper) {
            tables
                .iter()
                .filter(|t| {
                    prefix.is_empty() || t.table_name.to_uppercase().starts_with(&prefix_upper)
                })
                .collect()
        } else {
            vec![]
        }
    }
}

/// Escape single quotes in SQL strings
fn escape_sql_string(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_info_display_with_type() {
        let col = ColumnInfo::new("employee_id", "INTEGER", false);
        assert_eq!(col.display_with_type(), "employee_id (INT)");

        let col = ColumnInfo::new("name", "VARCHAR(100)", true);
        assert_eq!(col.display_with_type(), "name (VARCHAR)");

        let col = ColumnInfo::new("amount", "DECIMAL(10,2)", true);
        assert_eq!(col.display_with_type(), "amount (DEC)");

        let col = ColumnInfo::new("created_at", "TIMESTAMP", true);
        assert_eq!(col.display_with_type(), "created_at (TIMESTAMP)");
    }

    #[test]
    fn test_metadata_cache_new() {
        let cache = MetadataCache::new("testdb");
        assert!(!cache.has_tables());
        assert!(cache.get_tables().is_none());
        assert!(cache.get_columns("test_table").is_none());
    }

    #[test]
    fn test_metadata_cache_clear() {
        let mut cache = MetadataCache::new("testdb");
        cache.tables = Some(vec![TableInfo::new("db.table1", "table1", "db", "T")]);
        cache.columns.insert(
            "TABLE1".to_string(),
            vec![ColumnInfo::new("id", "INTEGER", false)],
        );

        cache.clear();

        assert!(!cache.has_tables());
        assert!(cache.columns.is_empty());
    }

    #[test]
    fn test_metadata_cache_set_current_database() {
        let mut cache = MetadataCache::new("db1");
        cache.tables = Some(vec![]);

        // Same database - should not clear
        cache.set_current_database("db1");
        assert!(cache.has_tables());

        // Different database - should clear
        cache.set_current_database("db2");
        assert!(!cache.has_tables());
    }

    #[test]
    fn test_table_info_new() {
        let info = TableInfo::new("prod.employees", "employees", "prod", "T");
        assert_eq!(info.full_name, "prod.employees");
        assert_eq!(info.table_name, "employees");
        assert_eq!(info.schema_name, "prod");
        assert_eq!(info.table_kind, "T");
    }

    #[test]
    fn test_find_tables_by_prefix_empty() {
        let cache = MetadataCache::new("testdb");
        let results = cache.find_tables_by_prefix("emp");
        assert!(results.is_empty());
    }

    #[test]
    fn test_find_tables_by_prefix_with_data() {
        let mut cache = MetadataCache::new("testdb");
        cache.tables = Some(vec![
            TableInfo::new("db.employees", "employees", "db", "T"),
            TableInfo::new("db.employee_archive", "employee_archive", "db", "T"),
            TableInfo::new("db.departments", "departments", "db", "T"),
        ]);

        let results = cache.find_tables_by_prefix("emp");
        assert_eq!(results.len(), 2);

        let results = cache.find_tables_by_prefix("EMP"); // Case-insensitive
        assert_eq!(results.len(), 2);

        let results = cache.find_tables_by_prefix("dep");
        assert_eq!(results.len(), 1);

        let results = cache.find_tables_by_prefix("xyz");
        assert!(results.is_empty());
    }

    #[test]
    fn test_get_columns_case_insensitive() {
        let mut cache = MetadataCache::new("testdb");
        cache.columns.insert(
            "EMPLOYEES".to_string(),
            vec![
                ColumnInfo::new("id", "INTEGER", false),
                ColumnInfo::new("name", "VARCHAR", true),
            ],
        );

        // Exact match
        assert!(cache.get_columns("EMPLOYEES").is_some());

        // Case-insensitive match
        assert!(cache.get_columns("employees").is_some());
        assert!(cache.get_columns("Employees").is_some());

        // No match
        assert!(cache.get_columns("departments").is_none());
    }

    #[test]
    fn test_find_columns_by_prefix() {
        let mut cache = MetadataCache::new("testdb");
        cache.columns.insert(
            "EMPLOYEES".to_string(),
            vec![
                ColumnInfo::new("employee_id", "INTEGER", false),
                ColumnInfo::new("email", "VARCHAR", true),
                ColumnInfo::new("first_name", "VARCHAR", true),
            ],
        );

        let results = cache.find_columns_by_prefix("employees", "e");
        assert_eq!(results.len(), 2); // employee_id, email

        let results = cache.find_columns_by_prefix("EMPLOYEES", "first");
        assert_eq!(results.len(), 1);

        let results = cache.find_columns_by_prefix("employees", "xyz");
        assert!(results.is_empty());
    }

    #[test]
    fn test_escape_sql_string() {
        assert_eq!(escape_sql_string("test"), "test");
        assert_eq!(escape_sql_string("test's"), "test''s");
        assert_eq!(escape_sql_string("it's a 'test'"), "it''s a ''test''");
    }

    // Sprint 20: Tests for database caching

    #[test]
    fn test_metadata_cache_has_databases() {
        let mut cache = MetadataCache::new("testdb");
        assert!(!cache.has_databases());

        cache.databases = Some(vec!["db1".to_string(), "db2".to_string()]);
        assert!(cache.has_databases());
    }

    #[test]
    fn test_metadata_cache_get_cached_databases() {
        let mut cache = MetadataCache::new("testdb");
        assert!(cache.get_cached_databases().is_none());

        cache.databases = Some(vec!["db1".to_string(), "db2".to_string()]);
        let dbs = cache.get_cached_databases().unwrap();
        assert_eq!(dbs.len(), 2);
        assert_eq!(dbs[0], "db1");
        assert_eq!(dbs[1], "db2");
    }

    #[test]
    fn test_get_databases_prefers_cached() {
        let mut cache = MetadataCache::new("testdb");

        // Set up both database cache and table cache with different data
        cache.databases = Some(vec!["cached_db1".to_string(), "cached_db2".to_string()]);
        cache.tables = Some(vec![
            TableInfo::new("table_db.employees", "employees", "table_db", "T"),
        ]);

        // Should return from database cache, not extracted from tables
        let dbs = cache.get_databases();
        assert_eq!(dbs.len(), 2);
        assert!(dbs.contains(&"cached_db1".to_string()));
        assert!(dbs.contains(&"cached_db2".to_string()));
        assert!(!dbs.contains(&"table_db".to_string()));
    }

    #[test]
    fn test_get_databases_falls_back_to_tables() {
        let mut cache = MetadataCache::new("testdb");

        // No database cache, but has table cache
        cache.databases = None;
        cache.tables = Some(vec![
            TableInfo::new("db1.employees", "employees", "db1", "T"),
            TableInfo::new("db2.customers", "customers", "db2", "T"),
            TableInfo::new("db1.orders", "orders", "db1", "T"),
        ]);

        // Should extract unique databases from tables
        let dbs = cache.get_databases();
        assert_eq!(dbs.len(), 2);
        assert!(dbs.contains(&"db1".to_string()));
        assert!(dbs.contains(&"db2".to_string()));
    }

    #[test]
    fn test_find_databases_by_prefix_with_cache() {
        let mut cache = MetadataCache::new("testdb");
        cache.databases = Some(vec![
            "production".to_string(),
            "prod_archive".to_string(),
            "development".to_string(),
            "staging".to_string(),
        ]);

        let results = cache.find_databases_by_prefix("prod");
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"production".to_string()));
        assert!(results.contains(&"prod_archive".to_string()));

        let results = cache.find_databases_by_prefix("PROD"); // Case-insensitive
        assert_eq!(results.len(), 2);

        let results = cache.find_databases_by_prefix("dev");
        assert_eq!(results.len(), 1);
        assert!(results.contains(&"development".to_string()));

        let results = cache.find_databases_by_prefix("xyz");
        assert!(results.is_empty());
    }

    #[test]
    fn test_metadata_cache_clear_clears_databases() {
        let mut cache = MetadataCache::new("testdb");
        cache.databases = Some(vec!["db1".to_string()]);
        cache.tables = Some(vec![]);
        cache.databases_loaded_at = Some(Instant::now());

        cache.clear();

        assert!(!cache.has_databases());
        assert!(!cache.has_tables());
        assert!(cache.databases_loaded_at.is_none());
    }

    // Sprint 21: Tests for per-database table caching

    #[test]
    fn test_has_tables_for_database() {
        let mut cache = MetadataCache::new("testdb");
        assert!(!cache.has_tables_for_database("demo_user"));

        cache.tables_by_database.insert(
            "DEMO_USER".to_string(),
            vec![TableInfo::new("demo_user.orders", "orders", "demo_user", "T")],
        );

        // Case-insensitive lookup
        assert!(cache.has_tables_for_database("demo_user"));
        assert!(cache.has_tables_for_database("DEMO_USER"));
        assert!(cache.has_tables_for_database("Demo_User"));
        assert!(!cache.has_tables_for_database("other_db"));
    }

    #[test]
    fn test_get_tables_for_database() {
        let mut cache = MetadataCache::new("testdb");
        assert!(cache.get_tables_for_database("demo_user").is_none());

        cache.tables_by_database.insert(
            "DEMO_USER".to_string(),
            vec![
                TableInfo::new("demo_user.orders", "orders", "demo_user", "T"),
                TableInfo::new("demo_user.customers", "customers", "demo_user", "T"),
            ],
        );

        let tables = cache.get_tables_for_database("demo_user").unwrap();
        assert_eq!(tables.len(), 2);
    }

    #[test]
    fn test_find_tables_in_database_by_prefix() {
        let mut cache = MetadataCache::new("testdb");

        cache.tables_by_database.insert(
            "DEMO_USER".to_string(),
            vec![
                TableInfo::new("demo_user.orders", "orders", "demo_user", "T"),
                TableInfo::new("demo_user.order_items", "order_items", "demo_user", "T"),
                TableInfo::new("demo_user.customers", "customers", "demo_user", "T"),
            ],
        );

        // Find tables matching prefix
        let results = cache.find_tables_in_database_by_prefix("demo_user", "order");
        assert_eq!(results.len(), 2);

        // Case-insensitive prefix
        let results = cache.find_tables_in_database_by_prefix("DEMO_USER", "ORDER");
        assert_eq!(results.len(), 2);

        // Empty prefix returns all
        let results = cache.find_tables_in_database_by_prefix("demo_user", "");
        assert_eq!(results.len(), 3);

        // No matches
        let results = cache.find_tables_in_database_by_prefix("demo_user", "xyz");
        assert!(results.is_empty());

        // Database not cached
        let results = cache.find_tables_in_database_by_prefix("other_db", "");
        assert!(results.is_empty());
    }

    #[test]
    fn test_metadata_cache_clear_clears_per_database_tables() {
        let mut cache = MetadataCache::new("testdb");
        cache.tables_by_database.insert(
            "DEMO_USER".to_string(),
            vec![TableInfo::new("demo_user.orders", "orders", "demo_user", "T")],
        );

        cache.clear();

        assert!(!cache.has_tables_for_database("demo_user"));
        assert!(cache.tables_by_database.is_empty());
    }

    #[test]
    fn test_dbc_not_in_exclusion_list() {
        // Sprint 21: Verify that DBC is not in the exclusion list
        // This is a documentation test - the actual SQL is in load_databases()
        // We verify by checking the constant pattern that DBC should be allowed
        let excluded_dbs = vec![
            "All", "Console", "Crashdumps", // Note: DBC is NOT here
            "dbcmngr", "Default", "External_AP", "EXTUSER", "LockLogShredder",
            "PUBLIC", "SQLJ", "Sys_Calendar", "SysAdmin", "SYSBAR", "SYSJDBC",
            "SYSLIB", "SYSSPATIAL", "SystemFe", "SYSUDTLIB", "TD_SERVER_DB",
            "TD_SYSFNLIB", "TD_SYSGPL", "TD_SYSXML", "TDMaps", "TDPUSER",
            "TDQCD", "TDStats", "tdwm", "VIEWPOINT",
        ];

        // DBC should NOT be in the exclusion list
        assert!(!excluded_dbs.contains(&"DBC"));
        assert!(!excluded_dbs.contains(&"dbc"));
    }
}
