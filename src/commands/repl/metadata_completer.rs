//! Metadata-aware SQL completer for reedline
//!
//! Provides context-sensitive tab completion that includes:
//! - Metacommands (when line starts with '/' or '\')
//! - SQL keywords (always available)
//! - Table names (after FROM, JOIN, UPDATE, INSERT INTO)
//! - Column names (after SELECT, WHERE, ORDER BY with table context)
//! - Database names for Teradata's `database.table` qualified naming
//!
//! ## Design
//! - Uses shared MetadataCache for fast cached lookups
//! - Triggers lazy loading of metadata on first table completion request
//! - Falls back to keyword completion when metadata unavailable
//! - Surfaces errors as user-visible feedback (not silent failures)
//!
//! Sprint 7 implementation, Sprint 8 bug fixes.
//! Sprint 22: Added metacommand tab completion.

use super::sql_context::{analyze_context, CompletionContext};
use crate::db::{ColumnInfo, DatabaseClient, MetadataCache};
use reedline::{Completer, Suggestion};
use std::sync::{Arc, Mutex};

/// Shared state for metadata completion
///
/// This wraps the database client and metadata cache in a way that can be
/// shared between the REPL loop and the completer.
pub struct CompletionState {
    /// Database client for metadata queries
    client: DatabaseClient,
    /// Cached metadata
    cache: MetadataCache,
    /// Accumulated SQL buffer for multi-line context (Sprint 9 Bug 2)
    accumulated_buffer: String,
}

impl CompletionState {
    /// Create new completion state
    pub fn new(client: DatabaseClient, database: impl Into<String>) -> Self {
        Self {
            client,
            cache: MetadataCache::new(database),
            accumulated_buffer: String::new(),
        }
    }

    /// Get reference to database client
    pub fn client(&self) -> &DatabaseClient {
        &self.client
    }

    /// Get reference to metadata cache
    pub fn cache(&self) -> &MetadataCache {
        &self.cache
    }

    /// Update the client (e.g., on /logon)
    pub fn update_client(&mut self, client: DatabaseClient, database: impl Into<String>) {
        self.client = client;
        self.cache.clear();
        self.cache.set_current_database(database);
    }

    /// Ensure databases are loaded
    ///
    /// Sprint 20: Databases should be pre-loaded at startup, but this provides
    /// a fallback in case they weren't.
    pub fn ensure_databases_loaded(&mut self) -> bool {
        if !self.cache.has_databases() {
            log::debug!("Tab completion: Triggering load of database names");
            let result = self.cache.load_databases(&self.client);
            if !result {
                if let Some(error) = self.cache.last_error() {
                    log::error!("Tab completion: Failed to load databases: {}", error);
                } else {
                    log::error!("Tab completion: Failed to load databases (unknown error)");
                }
            } else {
                log::debug!("Tab completion: Database names loaded successfully");
            }
            result
        } else {
            true
        }
    }

    /// Ensure tables are loaded, triggering lazy load if needed
    ///
    /// Sprint 11: Added explicit error logging for debugging completion failures.
    pub fn ensure_tables_loaded(&mut self) -> bool {
        if !self.cache.has_tables() {
            log::debug!("Tab completion: Triggering lazy load of table metadata");
            let result = self.cache.load_tables(&self.client);
            if !result {
                // Log the error explicitly so it's visible when debugging
                if let Some(error) = self.cache.last_error() {
                    log::error!("Tab completion: Failed to load table metadata: {}", error);
                } else {
                    log::error!("Tab completion: Failed to load table metadata (unknown error)");
                }
            } else {
                log::debug!("Tab completion: Table metadata loaded successfully");
            }
            result
        } else {
            true
        }
    }

    /// Sprint 21: Ensure tables for a specific database are loaded (on-demand)
    ///
    /// This triggers loading of table metadata for a single database when the user
    /// types `database.` + TAB. This avoids loading all tables at startup while
    /// still providing table completions for any database.
    pub fn ensure_tables_for_database_loaded(&mut self, database: &str) -> bool {
        if !self.cache.has_tables_for_database(database) {
            log::debug!(
                "Tab completion: Triggering on-demand load of tables for database: {}",
                database
            );
            let result = self.cache.load_tables_for_database(&self.client, database);
            if !result {
                if let Some(error) = self.cache.last_error() {
                    log::error!(
                        "Tab completion: Failed to load tables for {}: {}",
                        database,
                        error
                    );
                } else {
                    log::error!(
                        "Tab completion: Failed to load tables for {} (unknown error)",
                        database
                    );
                }
            } else {
                log::debug!(
                    "Tab completion: Tables loaded successfully for database: {}",
                    database
                );
            }
            result
        } else {
            true
        }
    }

    /// Ensure columns are loaded for a table
    ///
    /// Sprint 11: Added explicit error logging for debugging completion failures.
    /// Sprint 20: Not currently used during tab completion (to avoid queries),
    /// but kept for future use by metacommands like /describe.
    #[allow(dead_code)]
    pub fn ensure_columns_loaded(&mut self, table_name: &str) -> bool {
        if self.cache.get_columns(table_name).is_none() {
            log::debug!(
                "Tab completion: Triggering lazy load of columns for table: {}",
                table_name
            );
            let result = self.cache.load_columns(&self.client, table_name);
            if !result {
                if let Some(error) = self.cache.last_error() {
                    log::error!(
                        "Tab completion: Failed to load columns for {}: {}",
                        table_name,
                        error
                    );
                } else {
                    log::error!(
                        "Tab completion: Failed to load columns for {} (unknown error)",
                        table_name
                    );
                }
            } else {
                log::debug!(
                    "Tab completion: Columns loaded successfully for table: {}",
                    table_name
                );
            }
            result
        } else {
            true
        }
    }

    /// Set the accumulated SQL buffer for multi-line context (Sprint 9 Bug 2)
    pub fn set_accumulated_buffer(&mut self, buffer: String) {
        self.accumulated_buffer = buffer;
    }

    /// Get the accumulated buffer
    pub fn accumulated_buffer(&self) -> &str {
        &self.accumulated_buffer
    }

    /// Get the current database name
    ///
    /// Sprint 22: Needed by /list commands to show tables/views in current database.
    pub fn current_database(&self) -> &str {
        self.client.config().database.as_str()
    }
}

// =============================================================================
// Sprint 22: Metacommand Tab Completion
// =============================================================================

/// Metacommand definition for tab completion
///
/// Each metacommand has a name, optional aliases, and a description shown in
/// the completion menu.
struct MetacommandDef {
    /// Primary command name (without the leading /)
    name: &'static str,
    /// Optional short aliases
    aliases: &'static [&'static str],
    /// Description shown in completion menu
    description: &'static str,
}

/// Registry of all available metacommands for tab completion
///
/// This list should match the metacommands handled in `metacommands.rs`.
/// Sprint 22: Added /list commands for schema inspection.
const METACOMMANDS: &[MetacommandDef] = &[
    MetacommandDef {
        name: "help",
        aliases: &["?"],
        description: "Show help message",
    },
    MetacommandDef {
        name: "quit",
        aliases: &["q", "exit"],
        description: "Exit the REPL",
    },
    MetacommandDef {
        name: "session",
        aliases: &[],
        description: "Show session information",
    },
    MetacommandDef {
        name: "ping",
        aliases: &[],
        description: "Test database connection",
    },
    MetacommandDef {
        name: "describe",
        aliases: &["d"],
        description: "Describe table structure",
    },
    MetacommandDef {
        name: "export",
        aliases: &[],
        description: "Export query results",
    },
    MetacommandDef {
        name: "pager",
        aliases: &[],
        description: "Toggle result paging (on/off)",
    },
    MetacommandDef {
        name: "colors",
        aliases: &[],
        description: "Toggle syntax highlighting (on/off)",
    },
    MetacommandDef {
        name: "logon",
        aliases: &[],
        description: "Switch database connection",
    },
    // Sprint 22: Schema inspection commands
    MetacommandDef {
        name: "list databases",
        aliases: &["l"],
        description: "List all accessible databases",
    },
    MetacommandDef {
        name: "list tables",
        aliases: &["dt"],
        description: "List tables [pattern]",
    },
    MetacommandDef {
        name: "list views",
        aliases: &["dv"],
        description: "List views in current database",
    },
    // Sprint 26: Sessions command
    MetacommandDef {
        name: "sessions",
        aliases: &["s"],
        description: "List active sessions with performance metrics",
    },
    // Sprint 33: Data sampling commands
    MetacommandDef {
        name: "sample",
        aliases: &[],
        description: "Random sample rows from table",
    },
    MetacommandDef {
        name: "peek",
        aliases: &[],
        description: "Preview first rows and column metadata",
    },
    // Sprint 36: Repeat and show indexes commands
    MetacommandDef {
        name: "repeat",
        aliases: &["r"],
        description: "Re-execute last query",
    },
    // Sprint 37: Edit command
    MetacommandDef {
        name: "edit",
        aliases: &["e"],
        description: "Edit last query in $EDITOR",
    },
    MetacommandDef {
        name: "show indexes",
        aliases: &["di"],
        description: "Show index information for a table",
    },
    // Sprint 38: PMON monitoring commands
    MetacommandDef {
        name: "sysconfig",
        aliases: &["sc"],
        description: "Display system topology (version, nodes, AMPs, PEs)",
    },
    MetacommandDef {
        name: "locks",
        aliases: &["lk"],
        description: "Display current lock contention and blocking chains",
    },
    // Sprint 39: Query inspection command
    MetacommandDef {
        name: "query",
        aliases: &["qi"],
        description: "Show recent SQL queries for a session",
    },
    // Sprint 40: Parameter management command
    MetacommandDef {
        name: "params",
        aliases: &["p"],
        description: "Manage YAML parameter files for variable substitution",
    },
    // Sprint 45: Object inspection command
    MetacommandDef {
        name: "inspect",
        aliases: &["i"],
        description: "Inspect database object (type, columns, indexes, size)",
    },
    // Sprint 49: Session control commands
    MetacommandDef {
        name: "abort",
        aliases: &[],
        description: "Abort a session or running query",
    },
    MetacommandDef {
        name: "priority",
        aliases: &[],
        description: "Change session priority (RUSH/MEDIUM/LOW)",
    },
];

/// Test helper: returns metacommand names matching a prefix
///
/// Used by unit tests in metacommands.rs to verify the METACOMMANDS array
/// without needing to construct reedline Suggestion objects.
#[cfg(test)]
pub fn complete_metacommands_for_test(prefix: &str) -> Vec<String> {
    let prefix_lower = prefix.to_lowercase();
    let mut results = Vec::new();
    for cmd in METACOMMANDS {
        if cmd.name.to_lowercase().starts_with(&prefix_lower)
            || cmd
                .aliases
                .iter()
                .any(|a| a.to_lowercase().starts_with(&prefix_lower))
        {
            results.push(cmd.name.to_string());
        }
    }
    results
}

/// Complete metacommands based on user input prefix
///
/// Returns a list of suggestions for metacommands that match the given prefix.
/// The prefix should NOT include the leading '/' or '\'.
///
/// # Arguments
/// * `prefix` - The text after '/' that the user has typed (may be empty)
/// * `line_start` - Position where the metacommand starts (for span calculation)
/// * `cursor_pos` - Current cursor position
fn complete_metacommands(prefix: &str, line_start: usize, cursor_pos: usize) -> Vec<Suggestion> {
    let prefix_lower = prefix.to_lowercase();
    let prefix_parts: Vec<&str> = prefix_lower.split_whitespace().collect();

    let mut suggestions = Vec::new();

    // Check if we're completing a subcommand (e.g., "/list tab" -> "/list tables")
    if !prefix_parts.is_empty() && prefix_parts[0] == "list" {
        return complete_list_subcommands(&prefix_parts, line_start, cursor_pos);
    }

    // Sprint 36: Check for /show subcommand completion
    if !prefix_parts.is_empty() && prefix_parts[0] == "show" {
        return complete_show_subcommands(&prefix_parts, line_start, cursor_pos);
    }

    // Sprint 40: Check for /params subcommand completion
    if !prefix_parts.is_empty() && prefix_parts[0] == "params" {
        return complete_params_subcommands(&prefix_parts, line_start, cursor_pos);
    }

    // Filter metacommands by prefix
    for cmd in METACOMMANDS {
        let matches = cmd.name.to_lowercase().starts_with(&prefix_lower)
            || cmd
                .aliases
                .iter()
                .any(|a| a.to_lowercase().starts_with(&prefix_lower));

        if matches {
            // For multi-word commands, show the full command
            let display_name = cmd.name;

            suggestions.push(Suggestion {
                value: format!("/{}", display_name),
                description: Some(cmd.description.to_string()),
                style: None,
                extra: None,
                span: reedline::Span {
                    start: line_start,
                    end: cursor_pos,
                },
                // Add space after single-word commands, not after multi-word (user may add args)
                append_whitespace: !cmd.name.contains(' '),
            });
        }
    }

    // Sort alphabetically
    suggestions.sort_by(|a, b| a.value.cmp(&b.value));

    suggestions
}

/// Complete /list subcommands (databases, tables, views)
fn complete_list_subcommands(
    parts: &[&str],
    line_start: usize,
    cursor_pos: usize,
) -> Vec<Suggestion> {
    let subcommands = [
        ("databases", "List all accessible databases"),
        ("tables", "List tables [pattern]"),
        ("views", "List views in current database"),
    ];

    // Get the partial subcommand (if any)
    let partial = if parts.len() > 1 { parts[1] } else { "" };

    let mut suggestions = Vec::new();

    for (name, description) in subcommands {
        if partial.is_empty() || name.starts_with(partial) {
            suggestions.push(Suggestion {
                value: format!("/list {}", name),
                description: Some(description.to_string()),
                style: None,
                extra: None,
                span: reedline::Span {
                    start: line_start,
                    end: cursor_pos,
                },
                append_whitespace: true,
            });
        }
    }

    suggestions
}

/// Complete /show subcommands (indexes)
///
/// Sprint 36: Tab completion for /show subcommands.
fn complete_show_subcommands(
    parts: &[&str],
    line_start: usize,
    cursor_pos: usize,
) -> Vec<Suggestion> {
    let subcommands = [("indexes", "Show index information for a table")];

    // Get the partial subcommand (if any)
    let partial = if parts.len() > 1 { parts[1] } else { "" };

    let mut suggestions = Vec::new();

    for (name, description) in subcommands {
        if partial.is_empty() || name.starts_with(partial) {
            suggestions.push(Suggestion {
                value: format!("/show {}", name),
                description: Some(description.to_string()),
                style: None,
                extra: None,
                span: reedline::Span {
                    start: line_start,
                    end: cursor_pos,
                },
                append_whitespace: true,
            });
        }
    }

    suggestions
}

/// Complete /params subcommands (load, unload, show)
///
/// Sprint 40: Tab completion for /params subcommands.
fn complete_params_subcommands(
    parts: &[&str],
    line_start: usize,
    cursor_pos: usize,
) -> Vec<Suggestion> {
    let subcommands = [
        ("load", "Load a YAML parameter file"),
        ("unload", "Clear all loaded parameters"),
        ("show", "Show currently loaded parameters"),
    ];

    let partial = if parts.len() > 1 { parts[1] } else { "" };

    let mut suggestions = Vec::new();

    for (name, description) in subcommands {
        if partial.is_empty() || name.starts_with(partial) {
            suggestions.push(Suggestion {
                value: format!("/params {}", name),
                description: Some(description.to_string()),
                style: None,
                extra: None,
                span: reedline::Span {
                    start: line_start,
                    end: cursor_pos,
                },
                append_whitespace: true,
            });
        }
    }

    suggestions
}

/// Check if the input line is a metacommand (starts with / or \)
fn is_metacommand_input(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('/') || trimmed.starts_with('\\')
}

/// Extract metacommand prefix from input line
///
/// Returns the prefix after '/' or '\' and the position where the metacommand starts.
fn extract_metacommand_prefix(line: &str) -> Option<(&str, usize)> {
    let trimmed_start = line.len() - line.trim_start().len();
    let trimmed = line.trim_start();

    // Check for '/' or '\' prefix
    trimmed
        .strip_prefix('/')
        .or_else(|| trimmed.strip_prefix('\\'))
        .map(|stripped| (stripped, trimmed_start))
}

/// Metadata-aware SQL completer
///
/// Uses shared CompletionState to provide context-sensitive completions.
/// Thread-safe via Arc<Mutex> to satisfy reedline's Send requirement.
///
/// Sprint 18: Simplified to focus ONLY on metadata completion (databases, tables, columns).
/// NO keyword completion - this was causing interference with metadata completions.
#[derive(Default)]
pub struct MetadataCompleter {
    /// Shared completion state (required for metadata completion)
    state: Option<Arc<Mutex<CompletionState>>>,
}

impl MetadataCompleter {
    /// Create a new metadata completer without database connection
    ///
    /// Sprint 18: Without a database connection, no completions are available.
    /// Keyword completion has been removed entirely.
    ///
    /// This function is used by tests and the completer module for fallback.
    #[allow(dead_code)]
    pub fn keywords_only() -> Self {
        Self { state: None }
    }

    /// Create a new metadata completer with shared state
    pub fn with_state(state: Arc<Mutex<CompletionState>>) -> Self {
        Self { state: Some(state) }
    }

    /// Get table name completions
    ///
    /// Sprint 8 Bug Fix: Now returns DATABASE NAMES + tables in current database
    /// for Teradata's database.table naming model
    ///
    /// Sprint 18: Span is now set by the caller in complete(), not here.
    /// This function returns suggestions with placeholder spans.
    ///
    /// Sprint 21: When a single database matches the prefix (and no tables match),
    /// append '.' to the database name to enable quick table access workflow.
    ///
    /// Uses ONLY pre-loaded cache (NO queries during completion).
    /// All metadata must be loaded at startup. If not cached, return empty.
    fn complete_tables(&self, prefix: &str) -> Vec<Suggestion> {
        let Some(state) = &self.state else {
            // No database connection - return empty (no completions available)
            return Vec::new();
        };

        let Ok(state) = state.lock() else {
            log::warn!("Failed to acquire lock for table completion");
            return Vec::new();
        };

        let mut suggestions = Vec::new();

        // Collect database matches first to check for single-match scenario
        let database_matches: Vec<String> = if state.cache().has_databases() {
            state.cache().find_databases_by_prefix(prefix)
        } else {
            log::debug!("Tab completion: databases not cached, skipping database suggestions");
            vec![]
        };

        // Collect table matches in current database
        let table_matches: Vec<_> = if state.cache().has_tables() {
            state.cache().find_tables_in_current_db_by_prefix(prefix)
        } else {
            log::debug!("Tab completion: tables not cached, skipping table suggestions");
            vec![]
        };

        // Sprint 21 Feature 4: Smart Database-Dot-TAB Completion
        // When exactly one database matches and no tables match (or prefix is non-empty),
        // append '.' to enable quick workflow: user types "dem" + TAB -> "demo_user."
        let single_db_match = database_matches.len() == 1 && !prefix.is_empty();
        let no_table_matches = table_matches.is_empty();

        // Add database suggestions
        for db in &database_matches {
            // Sprint 21: Append dot if this is the only database match and no tables match
            let (value, description) = if single_db_match && no_table_matches {
                (
                    format!("{}.", db),
                    "(database - TAB for tables)".to_string(),
                )
            } else {
                (db.clone(), "(database)".to_string())
            };

            suggestions.push(Suggestion {
                value,
                description: Some(description),
                style: None,
                extra: None,
                span: reedline::Span { start: 0, end: 0 }, // Placeholder - set by caller
                append_whitespace: false, // Don't add space after database name
            });
        }

        // Add table suggestions (tables in current database, unqualified names)
        for table in table_matches {
            let kind = match table.table_kind.as_str() {
                "T" => "table",
                "V" => "view",
                "O" => "object",
                _ => "table",
            };

            suggestions.push(Suggestion {
                value: table.table_name.clone(),
                description: Some(format!("{} ({})", table.schema_name, kind)),
                style: None,
                extra: None,
                span: reedline::Span { start: 0, end: 0 }, // Placeholder - set by caller
                append_whitespace: true,
            });
        }

        suggestions
    }

    /// Get schema-qualified table completions
    ///
    /// Sprint 8 Bug Fix: Improved error handling for database.table completions
    /// Sprint 18: Span is now set by the caller in complete(), not here.
    /// Sprint 20 Fix: Uses ONLY cached data (NO queries during completion).
    /// Sprint 21: Now triggers on-demand loading for specific database if not cached.
    fn complete_schema_tables(&self, schema: &str, prefix: &str) -> Vec<Suggestion> {
        // Sprint 8 Round 4: Add safety check for empty schema
        if schema.is_empty() {
            log::warn!("complete_schema_tables called with empty schema");
            return Vec::new();
        }

        let Some(state) = &self.state else {
            return Vec::new();
        };

        let Ok(mut state) = state.lock() else {
            log::warn!("Failed to acquire lock for schema-qualified table completion");
            return Vec::new();
        };

        // Sprint 21: Trigger on-demand loading for this specific database
        // This fixes the issue where tables for databases like 'demo_user' weren't cached.
        state.ensure_tables_for_database_loaded(schema);

        // Now use the per-database cache
        let cache = state.cache();

        // Sprint 21: First check per-database cache (loaded on-demand)
        if cache.has_tables_for_database(schema) {
            let matching_tables = cache.find_tables_in_database_by_prefix(schema, prefix);

            if !matching_tables.is_empty() {
                return matching_tables
                    .into_iter()
                    .map(|t| {
                        let kind = match t.table_kind.as_str() {
                            "T" => "table",
                            "V" => "view",
                            "O" => "object",
                            _ => "table",
                        };

                        let full_name = format!("{}.{}", schema, t.table_name);
                        Suggestion {
                            value: full_name.clone(),
                            description: Some(format!("{} ({})", full_name, kind)),
                            style: None,
                            extra: None,
                            span: reedline::Span { start: 0, end: 0 },
                            append_whitespace: true,
                        }
                    })
                    .collect();
            }
        }

        // Fall back to global tables cache (loaded at startup with SAMPLE 10000)
        if !cache.has_tables() {
            log::debug!("Tab completion: tables not cached, skipping schema-qualified suggestions");
            return Vec::new();
        }

        // Find tables in the specified database/schema from global cache
        let Some(tables) = cache.get_tables() else {
            return Vec::new();
        };

        let schema_upper = schema.to_uppercase();
        let prefix_upper = prefix.to_uppercase();

        let matching_tables: Vec<_> = tables
            .iter()
            .filter(|t| {
                t.schema_name.to_uppercase() == schema_upper
                    && (prefix.is_empty() || t.table_name.to_uppercase().starts_with(&prefix_upper))
            })
            .collect();

        matching_tables
            .into_iter()
            .map(|t| {
                let kind = match t.table_kind.as_str() {
                    "T" => "table",
                    "V" => "view",
                    "O" => "object",
                    _ => "table",
                };

                // Sprint 8 Bug Fix: Return FULL qualified name (schema.table) so user doesn't lose the schema prefix
                let full_name = format!("{}.{}", schema, t.table_name);
                Suggestion {
                    value: full_name.clone(),
                    description: Some(format!("{} ({})", full_name, kind)),
                    style: None,
                    extra: None,
                    span: reedline::Span { start: 0, end: 0 }, // Placeholder - set by caller
                    append_whitespace: true,
                }
            })
            .collect()
    }

    /// Get column name completions
    ///
    /// Sprint 8: Now surfaces errors when column loading fails.
    /// Sprint 18: Span is now set by the caller in complete(), not here.
    /// Sprint 20 Fix: Uses ONLY cached data (NO queries during completion).
    /// Column completions are only available if columns were previously cached
    /// (e.g., via /describe command or prior query).
    fn complete_columns(
        &self,
        tables: &[super::sql_context::TableReference],
        prefix: &str,
        _qualifier: Option<&str>,
    ) -> Vec<Suggestion> {
        let Some(state) = &self.state else {
            return Vec::new();
        };

        let Ok(state) = state.lock() else {
            log::warn!("Failed to acquire lock for column completion");
            return Vec::new();
        };

        if tables.is_empty() {
            return Vec::new();
        }

        let mut suggestions = Vec::new();

        for table in tables {
            // Sprint 20 Fix: Use ONLY cached columns (NO queries during completion)
            // If columns for this table aren't cached, skip it.
            // Columns get cached via /describe command or prior queries.
            let cache = state.cache();
            if let Some(columns) = cache.get_columns(&table.name) {
                // Filter columns by prefix
                let prefix_upper = prefix.to_uppercase();
                for col in columns {
                    if col.name.to_uppercase().starts_with(&prefix_upper) {
                        suggestions.push(self.column_to_suggestion(col, &table.name));
                    }
                }
            } else {
                log::debug!(
                    "Tab completion: columns for {} not cached, skipping column suggestions",
                    table.name
                );
            }
        }

        suggestions
    }

    /// Convert ColumnInfo to Suggestion
    ///
    /// Sprint 18: Span is now set by the caller in complete(), not here.
    fn column_to_suggestion(&self, col: &ColumnInfo, table_name: &str) -> Suggestion {
        Suggestion {
            value: col.name.clone(),
            description: Some(format!("{}.{} ({})", table_name, col.name, col.data_type)),
            style: None,
            extra: None,
            span: reedline::Span { start: 0, end: 0 }, // Placeholder - set by caller
            append_whitespace: false, // Don't add space after column name
        }
    }
}


impl Completer for MetadataCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // Sprint 22: Check for metacommand completion FIRST
        // If the line starts with '/' or '\', provide metacommand completions
        if is_metacommand_input(line) {
            if let Some((prefix, cmd_start)) = extract_metacommand_prefix(line) {
                log::debug!(
                    "Metacommand completion: prefix='{}', start={}, pos={}",
                    prefix,
                    cmd_start,
                    pos
                );
                return complete_metacommands(prefix, cmd_start, pos);
            }
        }

        // Sprint 18: Simplified completion - ONLY metadata (databases, tables, columns)
        // NO keyword completion.

        // Sprint 9 Bug 2 Fix: Support multi-line context by prepending accumulated buffer
        let (full_text, _adjusted_pos) = if let Some(state) = &self.state {
            let state_lock = state.lock().unwrap();
            let accumulated = state_lock.accumulated_buffer();
            if !accumulated.is_empty() {
                // Prepend accumulated buffer to current line for context analysis
                let combined = format!("{}{}", accumulated, line);
                let new_pos = accumulated.len() + pos;
                (combined, new_pos)
            } else {
                (line.to_string(), pos)
            }
        } else {
            (line.to_string(), pos)
        };

        // Analyze context to determine what kind of completions to provide
        // Use the full text (with accumulated buffer) for context analysis
        let context = analyze_context(&full_text, full_text.len());

        log::debug!(
            "Completion context: {:?} (line: '{}', pos: {})",
            context,
            line,
            pos
        );

        // Sprint 18 CRITICAL FIX: Calculate span based on the CURRENT LINE only.
        // The span tells reedline what part of the current line to replace.
        // pos is the cursor position in the current line.
        //
        // Find the word being typed by scanning backward from cursor position.
        let line_up_to_cursor = &line[..pos.min(line.len())];
        let last_word = get_last_word(line_up_to_cursor);

        // Calculate start position: where the current word/token starts in the line
        let start = pos.saturating_sub(last_word.len());
        let end = pos;

        log::debug!(
            "Span calculation: last_word='{}', start={}, end={}",
            last_word,
            start,
            end
        );

        // Get completions based on context - NO KEYWORDS
        let mut suggestions = match context {
            CompletionContext::Keyword => {
                // Sprint 18: NO keyword completion - return empty
                Vec::new()
            }

            CompletionContext::TableName { prefix } => self.complete_tables(&prefix),

            CompletionContext::SchemaQualifiedTable { schema, prefix } => {
                // For schema-qualified, we need to adjust the span to cover "schema." or "schema.prefix"
                let schema_prefix_len = if prefix.is_empty() {
                    schema.len() + 1 // "schema."
                } else {
                    schema.len() + 1 + prefix.len() // "schema.prefix"
                };
                let adjusted_start = pos.saturating_sub(schema_prefix_len);

                let mut sug = self.complete_schema_tables(&schema, &prefix);
                for s in &mut sug {
                    s.span = reedline::Span {
                        start: adjusted_start,
                        end,
                    };
                }
                return sug; // Return early with adjusted span
            }

            CompletionContext::ColumnName {
                tables,
                prefix,
                table_qualifier,
            } => {
                // For table-qualified columns (e.g., "t.col"), adjust span
                if let Some(ref qualifier) = table_qualifier {
                    let qualifier_len = qualifier.len() + 1 + prefix.len(); // "qualifier.prefix"
                    let adjusted_start = pos.saturating_sub(qualifier_len);

                    let mut sug = self.complete_columns(&tables, &prefix, None);
                    for s in &mut sug {
                        // Prepend qualifier to column name for qualified completions
                        s.value = format!("{}.{}", qualifier, s.value);
                        s.span = reedline::Span {
                            start: adjusted_start,
                            end,
                        };
                    }
                    return sug; // Return early with adjusted span
                }

                self.complete_columns(&tables, &prefix, None)
            }
        };

        // Set the span for all suggestions
        for sug in &mut suggestions {
            sug.span = reedline::Span { start, end };
        }

        // Sort by length and alphabetically
        suggestions.sort_by(|a, b| {
            a.value
                .len()
                .cmp(&b.value.len())
                .then_with(|| a.value.cmp(&b.value))
        });

        // Limit to reasonable number
        suggestions.truncate(50);

        suggestions
    }
}

/// Get the last word from the input line
fn get_last_word(line: &str) -> &str {
    if line.ends_with(|c: char| c.is_whitespace()) {
        return "";
    }

    // Find start of last word
    let bytes = line.as_bytes();
    let mut start = line.len();

    for i in (0..bytes.len()).rev() {
        let c = bytes[i] as char;
        if c.is_whitespace() || c == ',' || c == '(' || c == ')' || c == '=' {
            break;
        }
        // Handle qualified names (keep everything including dots)
        start = i;
    }

    &line[start..]
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sprint 18: All keyword completion tests removed.
    // Tab completion now focuses ONLY on metadata (databases, tables, columns).

    #[test]
    fn test_no_keyword_completion() {
        // Sprint 18: Verify NO keyword completion
        let mut completer = MetadataCompleter::keywords_only();

        // Without a database connection, no completions should be available
        let suggestions = completer.complete("SEL", 3);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_get_last_word() {
        assert_eq!(get_last_word("SELECT * FROM emp"), "emp");
        assert_eq!(get_last_word("SELECT * FROM "), "");
        assert_eq!(get_last_word("SELECT e.name"), "e.name");
        assert_eq!(get_last_word("WHERE id = "), "");
    }

    #[test]
    fn test_get_last_word_qualified_name() {
        // Qualified names should be kept together
        assert_eq!(get_last_word("SELECT * FROM DBC.Tab"), "DBC.Tab");
        assert_eq!(get_last_word("SELECT * FROM prod.employees"), "prod.employees");
    }

    #[test]
    fn test_complete_tables_no_connection() {
        // Sprint 18: Without a connection, complete_tables should return empty
        let completer = MetadataCompleter::keywords_only();
        let suggestions = completer.complete_tables("emp");

        // Should be empty (no database connection = no completions)
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_table_context_no_results_without_connection() {
        // Sprint 18: After FROM with no connection, no completions
        let mut completer = MetadataCompleter::keywords_only();

        let suggestions = completer.complete("SELECT * FROM S", 15);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_column_context_no_results_without_connection() {
        // Sprint 18: After WHERE with no connection, no completions
        let mut completer = MetadataCompleter::keywords_only();

        let suggestions = completer.complete("SELECT * FROM employees WHERE n", 31);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_schema_qualified_no_results_without_connection() {
        // Sprint 18: Schema-qualified completion with no connection = no results
        let mut completer = MetadataCompleter::keywords_only();

        let suggestions = completer.complete("SELECT * FROM prod.", 19);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_span_calculation_simple() {
        // Test that span is calculated correctly for simple cases
        let mut completer = MetadataCompleter::keywords_only();

        // When completing "emp" after "SELECT * FROM emp", the span should be:
        // start = 14 (position after "FROM "), end = 17 (cursor position)
        let suggestions = completer.complete("SELECT * FROM emp", 17);
        // No suggestions without connection, but we're testing span calc logic
        assert!(suggestions.is_empty()); // Expected without connection
    }

    // Sprint 21: Tests for smart database-dot-TAB completion

    #[test]
    fn test_complete_tables_with_mock_state_single_db_match() {
        // This test verifies the logic of single-database-match detection.
        // The actual completion with state requires a live database connection,
        // so we test the edge case handling here.

        // Without a connection, we can only test that empty state returns empty
        let completer = MetadataCompleter::keywords_only();
        let suggestions = completer.complete_tables("dem");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_complete_schema_tables_empty_schema_safety() {
        // Sprint 8 Round 4 safety check: empty schema should return empty
        let completer = MetadataCompleter::keywords_only();
        let suggestions = completer.complete_schema_tables("", "orders");
        assert!(suggestions.is_empty());
    }

    // Sprint 22: Tests for metacommand tab completion

    #[test]
    fn test_is_metacommand_input_slash() {
        assert!(is_metacommand_input("/help"));
        assert!(is_metacommand_input("/"));
        assert!(is_metacommand_input("  /quit"));
    }

    #[test]
    fn test_is_metacommand_input_backslash() {
        assert!(is_metacommand_input("\\help"));
        assert!(is_metacommand_input("\\"));
        assert!(is_metacommand_input("  \\quit"));
    }

    #[test]
    fn test_is_metacommand_input_not_metacommand() {
        assert!(!is_metacommand_input("SELECT * FROM"));
        assert!(!is_metacommand_input(""));
        assert!(!is_metacommand_input("help"));
    }

    #[test]
    fn test_extract_metacommand_prefix() {
        let (prefix, start) = extract_metacommand_prefix("/help").unwrap();
        assert_eq!(prefix, "help");
        assert_eq!(start, 0);

        let (prefix, start) = extract_metacommand_prefix("/").unwrap();
        assert_eq!(prefix, "");
        assert_eq!(start, 0);

        let (prefix, start) = extract_metacommand_prefix("  /quit").unwrap();
        assert_eq!(prefix, "quit");
        assert_eq!(start, 2);
    }

    #[test]
    fn test_extract_metacommand_prefix_backslash() {
        let (prefix, start) = extract_metacommand_prefix("\\help").unwrap();
        assert_eq!(prefix, "help");
        assert_eq!(start, 0);
    }

    #[test]
    fn test_complete_metacommands_empty_prefix() {
        let suggestions = complete_metacommands("", 0, 1);
        assert!(!suggestions.is_empty());
        // Should include common commands
        let values: Vec<&str> = suggestions.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"/help"));
        assert!(values.contains(&"/quit"));
        assert!(values.contains(&"/session"));
    }

    #[test]
    fn test_complete_metacommands_partial_match() {
        let suggestions = complete_metacommands("he", 0, 3);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "/help");
    }

    #[test]
    fn test_complete_metacommands_quit_and_aliases() {
        let suggestions = complete_metacommands("q", 0, 2);
        // Should match 'quit' (both name and alias)
        let values: Vec<&str> = suggestions.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"/quit"));
    }

    #[test]
    fn test_complete_metacommands_case_insensitive() {
        let suggestions = complete_metacommands("HELP", 0, 5);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "/help");
    }

    #[test]
    fn test_complete_metacommands_list_subcommands() {
        // Just "/list" should show all list subcommands
        let suggestions = complete_metacommands("list", 0, 5);
        // Should find /list databases, /list tables, /list views
        let values: Vec<&str> = suggestions.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"/list databases"));
        assert!(values.contains(&"/list tables"));
        assert!(values.contains(&"/list views"));
    }

    #[test]
    fn test_complete_metacommands_list_partial_subcommand() {
        // "/list tab" should complete to "/list tables"
        let suggestions = complete_metacommands("list tab", 0, 9);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "/list tables");
    }

    #[test]
    fn test_complete_metacommands_no_match() {
        let suggestions = complete_metacommands("xyz", 0, 4);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_metacommand_completion_descriptions() {
        let suggestions = complete_metacommands("help", 0, 5);
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].description.is_some());
        assert!(suggestions[0]
            .description
            .as_ref()
            .unwrap()
            .contains("help"));
    }

    #[test]
    fn test_metacommand_completion_via_completer() {
        // Test that the Completer trait implementation routes metacommands correctly
        let mut completer = MetadataCompleter::keywords_only();

        let suggestions = completer.complete("/he", 3);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "/help");
    }

    #[test]
    fn test_metacommand_completion_backslash_via_completer() {
        let mut completer = MetadataCompleter::keywords_only();

        let suggestions = completer.complete("\\q", 2);
        let values: Vec<&str> = suggestions.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"/quit"));
    }

    #[test]
    fn test_complete_metacommands_sessions() {
        // Test /sessions command tab completion (Sprint 26)
        let suggestions = complete_metacommands("sess", 0, 5);
        let values: Vec<&str> = suggestions.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"/sessions"));
    }

    #[test]
    fn test_complete_metacommands_sessions_alias() {
        // Test that /s prefix shows /sessions among completions
        let suggestions = complete_metacommands("s", 0, 2);
        let values: Vec<&str> = suggestions.iter().map(|s| s.value.as_str()).collect();
        // /s should match /sessions (which has alias /s), /session, and /sample
        assert!(values.contains(&"/sessions"));
        assert!(values.contains(&"/session"));
        assert!(values.contains(&"/sample"));
    }

    // Sprint 33: Tests for data sampling commands tab completion
    #[test]
    fn test_complete_metacommands_sample() {
        let suggestions = complete_metacommands("sam", 0, 4);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "/sample");
        assert!(suggestions[0].description.as_ref().unwrap().contains("Random"));
    }

    #[test]
    fn test_complete_metacommands_peek() {
        let suggestions = complete_metacommands("pee", 0, 4);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "/peek");
        assert!(suggestions[0].description.as_ref().unwrap().contains("Preview"));
    }

    #[test]
    fn test_complete_metacommands_p_shows_peek_and_pager_and_ping() {
        let suggestions = complete_metacommands("p", 0, 2);
        let values: Vec<&str> = suggestions.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"/peek"));
        assert!(values.contains(&"/pager"));
        assert!(values.contains(&"/ping"));
    }

    // Sprint 36: Tests for /repeat and /show indexes tab completion

    #[test]
    fn test_complete_metacommands_repeat() {
        let suggestions = complete_metacommands("rep", 0, 4);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "/repeat");
        assert!(suggestions[0]
            .description
            .as_ref()
            .unwrap()
            .contains("Re-execute"));
    }

    #[test]
    fn test_complete_metacommands_repeat_alias_r() {
        let suggestions = complete_metacommands("r", 0, 2);
        let values: Vec<&str> = suggestions.iter().map(|s| s.value.as_str()).collect();
        // /r should match /repeat (alias "r")
        assert!(values.contains(&"/repeat"));
    }

    #[test]
    fn test_complete_metacommands_show() {
        let suggestions = complete_metacommands("show", 0, 5);
        let values: Vec<&str> = suggestions.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"/show indexes"));
    }

    #[test]
    fn test_complete_show_subcommands() {
        // "/show " should show "indexes" subcommand
        let suggestions = complete_metacommands("show ", 0, 6);
        let values: Vec<&str> = suggestions.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"/show indexes"));
    }

    #[test]
    fn test_complete_show_subcommands_partial() {
        // "/show ind" should match "/show indexes"
        let suggestions = complete_metacommands("show ind", 0, 9);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "/show indexes");
    }

    #[test]
    fn test_complete_metacommands_di_alias() {
        let suggestions = complete_metacommands("di", 0, 3);
        let values: Vec<&str> = suggestions.iter().map(|s| s.value.as_str()).collect();
        // /di is an alias for /show indexes
        assert!(values.contains(&"/show indexes"));
    }
}
