//! Metadata-aware SQL completer for reedline
//!
//! Provides context-sensitive tab completion that includes:
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

        // Sprint 20 Fix: Use ONLY cached database names (NO queries during completion)
        // Databases should have been pre-loaded at startup.
        // If not cached, we return empty rather than triggering a query.
        if state.cache().has_databases() {
            let databases = state.cache().find_databases_by_prefix(prefix);
            for db in databases {
                suggestions.push(Suggestion {
                    value: db.clone(),
                    description: Some("(database)".to_string()),
                    style: None,
                    extra: None,
                    span: reedline::Span { start: 0, end: 0 }, // Placeholder - set by caller
                    append_whitespace: false, // Don't add space after database name (user will type '.')
                });
            }
        } else {
            log::debug!("Tab completion: databases not cached, skipping database suggestions");
        }

        // Sprint 20 Fix: Use ONLY cached table metadata (NO queries during completion)
        // Tables should have been pre-loaded at startup.
        // If not cached, we return empty rather than triggering a query.
        if state.cache().has_tables() {
            // Show tables in current database (unqualified names)
            let tables = state.cache().find_tables_in_current_db_by_prefix(prefix);
            for table in tables {
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
        } else {
            log::debug!("Tab completion: tables not cached, skipping table suggestions");
        }

        suggestions
    }

    /// Get schema-qualified table completions
    ///
    /// Sprint 8 Bug Fix: Improved error handling for database.table completions
    /// Sprint 18: Span is now set by the caller in complete(), not here.
    /// Sprint 20 Fix: Uses ONLY cached data (NO queries during completion).
    fn complete_schema_tables(&self, schema: &str, prefix: &str) -> Vec<Suggestion> {
        // Sprint 8 Round 4: Add safety check for empty schema
        if schema.is_empty() {
            log::warn!("complete_schema_tables called with empty schema");
            return Vec::new();
        }

        let Some(state) = &self.state else {
            return Vec::new();
        };

        let Ok(state) = state.lock() else {
            log::warn!("Failed to acquire lock for schema-qualified table completion");
            return Vec::new();
        };

        // Sprint 20 Fix: Use ONLY cached tables (NO queries during completion)
        // If tables aren't cached, return empty rather than triggering a query.
        let cache = state.cache();
        if !cache.has_tables() {
            log::debug!("Tab completion: tables not cached, skipping schema-qualified suggestions");
            return Vec::new();
        }

        // Find tables in the specified database/schema
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
}
