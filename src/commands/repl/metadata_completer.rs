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
    pub fn ensure_columns_loaded(&mut self, table_name: &str) -> bool {
        if self.cache.get_columns(table_name).is_none() {
            log::debug!("Tab completion: Triggering lazy load of columns for table: {}", table_name);
            let result = self.cache.load_columns(&self.client, table_name);
            if !result {
                if let Some(error) = self.cache.last_error() {
                    log::error!("Tab completion: Failed to load columns for {}: {}", table_name, error);
                } else {
                    log::error!("Tab completion: Failed to load columns for {} (unknown error)", table_name);
                }
            } else {
                log::debug!("Tab completion: Columns loaded successfully for table: {}", table_name);
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
pub struct MetadataCompleter {
    /// SQL keywords
    keywords: Vec<String>,
    /// Shared completion state (optional - falls back to keywords only)
    state: Option<Arc<Mutex<CompletionState>>>,
}

impl MetadataCompleter {
    /// Create a new metadata completer without database connection
    ///
    /// This will only provide keyword completion.
    pub fn keywords_only() -> Self {
        Self {
            keywords: Self::default_keywords(),
            state: None,
        }
    }

    /// Create a new metadata completer with shared state
    pub fn with_state(state: Arc<Mutex<CompletionState>>) -> Self {
        Self {
            keywords: Self::default_keywords(),
            state: Some(state),
        }
    }

    /// Get default SQL keywords
    fn default_keywords() -> Vec<String> {
        vec![
            // DML statements
            "SELECT",
            "INSERT",
            "UPDATE",
            "DELETE",
            "WITH",
            // DDL statements
            "CREATE",
            "DROP",
            "ALTER",
            "TRUNCATE",
            // Table/Database operations
            "TABLE",
            "DATABASE",
            "SCHEMA",
            "VIEW",
            "INDEX",
            "PROCEDURE",
            "FUNCTION",
            // Clauses
            "FROM",
            "WHERE",
            "GROUP BY",
            "HAVING",
            "ORDER BY",
            "LIMIT",
            "OFFSET",
            "DISTINCT",
            "ALL",
            "TOP",
            // JOINs
            "JOIN",
            "INNER JOIN",
            "LEFT JOIN",
            "RIGHT JOIN",
            "FULL JOIN",
            "CROSS JOIN",
            "ON",
            "USING",
            "AS",
            // Set operations
            "UNION",
            "INTERSECT",
            "EXCEPT",
            // Logical operators
            "AND",
            "OR",
            "NOT",
            "IN",
            "EXISTS",
            "BETWEEN",
            "LIKE",
            "IS NULL",
            "IS NOT NULL",
            // Aggregates and functions
            "COUNT",
            "SUM",
            "AVG",
            "MIN",
            "MAX",
            // Transactions
            "BEGIN",
            "COMMIT",
            "ROLLBACK",
            "TRANSACTION",
            // Conditionals
            "CASE",
            "WHEN",
            "THEN",
            "ELSE",
            "END",
            // Data modification
            "VALUES",
            "SET",
            // Constraints
            "PRIMARY KEY",
            "FOREIGN KEY",
            "UNIQUE",
            "CHECK",
            "CONSTRAINT",
            // Permissions
            "GRANT",
            "REVOKE",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    /// Get keyword completions
    fn complete_keywords(&self, prefix: &str) -> Vec<Suggestion> {
        let prefix_upper = prefix.to_uppercase();

        self.keywords
            .iter()
            .filter(|kw| kw.starts_with(&prefix_upper))
            .map(|kw| Suggestion {
                value: kw.clone(),
                description: Some("SQL keyword".to_string()),
                style: None,
                extra: None,
                span: reedline::Span {
                    start: 0,
                    end: prefix.len(),
                },
                append_whitespace: true,
            })
            .collect()
    }

    /// Get table name completions
    ///
    /// Sprint 8 Bug Fix: Now returns DATABASE NAMES + tables in current database
    /// for Teradata's database.table naming model
    fn complete_tables(&self, prefix: &str) -> Vec<Suggestion> {
        let Some(state) = &self.state else {
            // No database connection - show message instead of silent failure
            return vec![
                self.status_suggestion("No database connection for table completion", prefix.len())
            ];
        };

        let Ok(mut state) = state.lock() else {
            log::warn!("Failed to acquire lock for table completion");
            return vec![
                self.error_suggestion("Unable to load tables (internal lock error)", prefix.len())
            ];
        };

        // Ensure tables are loaded
        if !state.ensure_tables_loaded() {
            // Loading failed - show the error to user
            if let Some(error) = state.cache().last_error() {
                return vec![self.error_suggestion(error, prefix.len())];
            } else {
                return vec![self.error_suggestion("Failed to load table metadata", prefix.len())];
            }
        }

        let mut suggestions = Vec::new();

        // Sprint 8 Fix: Show database names for Teradata's database.table model
        let databases = state.cache().find_databases_by_prefix(prefix);
        for db in databases {
            suggestions.push(Suggestion {
                value: db.clone(),
                description: Some("(database)".to_string()),
                style: None,
                extra: None,
                span: reedline::Span {
                    start: 0,
                    end: prefix.len(),
                },
                append_whitespace: false, // Don't add space after database name (user will type '.')
            });
        }

        // Also show tables in current database (unqualified names)
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
                span: reedline::Span {
                    start: 0,
                    end: prefix.len(),
                },
                append_whitespace: true,
            });
        }

        if suggestions.is_empty() {
            // No databases or tables found - provide helpful message
            if prefix.is_empty() {
                return vec![
                    self.status_suggestion("No databases or tables found", prefix.len())
                ];
            } else {
                return vec![self
                    .status_suggestion(&format!("No databases or tables matching '{}'", prefix), prefix.len())];
            }
        }

        suggestions
    }

    /// Get schema-qualified table completions
    ///
    /// Sprint 8 Bug Fix: Improved error handling for database.table completions
    fn complete_schema_tables(&self, schema: &str, prefix: &str) -> Vec<Suggestion> {
        // Sprint 8 Round 4: Add safety check for empty schema
        if schema.is_empty() {
            log::warn!("complete_schema_tables called with empty schema");
            return vec![self.status_suggestion("Invalid database name", prefix.len())];
        }

        let Some(state) = &self.state else {
            return vec![
                self.status_suggestion("No database connection for table completion", prefix.len())
            ];
        };

        let Ok(mut state) = state.lock() else {
            log::warn!("Failed to acquire lock for schema-qualified table completion");
            return vec![
                self.error_suggestion("Unable to load tables (internal lock error)", prefix.len())
            ];
        };

        // Ensure tables are loaded
        if !state.ensure_tables_loaded() {
            if let Some(error) = state.cache().last_error() {
                return vec![self.error_suggestion(error, prefix.len())];
            } else {
                return vec![self.error_suggestion("Failed to load table metadata", prefix.len())];
            }
        }

        // Find tables in the specified database/schema
        let cache = state.cache();
        let Some(tables) = cache.get_tables() else {
            return vec![self.status_suggestion("No tables loaded", prefix.len())];
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

        if matching_tables.is_empty() {
            // No tables found in schema - provide helpful message
            let msg = if prefix.is_empty() {
                format!("No tables in database '{}'", schema)
            } else {
                format!("No tables in '{}' matching '{}'", schema, prefix)
            };
            return vec![self.status_suggestion(&msg, prefix.len())];
        }

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
                    span: reedline::Span {
                        start: 0,
                        end: prefix.len(),
                    },
                    append_whitespace: true,
                }
            })
            .collect()
    }

    /// Get column name completions
    ///
    /// Sprint 8: Now surfaces errors when column loading fails.
    fn complete_columns(
        &self,
        tables: &[super::sql_context::TableReference],
        prefix: &str,
        _qualifier: Option<&str>,
    ) -> Vec<Suggestion> {
        let Some(state) = &self.state else {
            return vec![self
                .status_suggestion("No database connection for column completion", prefix.len())];
        };

        let Ok(mut state) = state.lock() else {
            log::warn!("Failed to acquire lock for column completion");
            return vec![
                self.error_suggestion("Unable to load columns (internal lock error)", prefix.len())
            ];
        };

        if tables.is_empty() {
            return vec![self.status_suggestion(
                "Cannot determine table context. Specify table in FROM clause first.",
                prefix.len(),
            )];
        }

        let mut suggestions = Vec::new();
        let mut had_error = false;

        for table in tables {
            // Try to load columns for this table
            if !state.ensure_columns_loaded(&table.name) {
                // Loading failed - note the error but continue trying other tables
                had_error = true;
                log::debug!("Failed to load columns for table: {}", table.name);
            }

            // Get matching columns
            let columns = state.cache().find_columns_by_prefix(&table.name, prefix);

            for col in columns {
                suggestions.push(self.column_to_suggestion(col, &table.name, prefix.len()));
            }
        }

        if suggestions.is_empty() {
            if had_error {
                if let Some(error) = state.cache().last_error() {
                    return vec![self.error_suggestion(error, prefix.len())];
                } else {
                    return vec![
                        self.error_suggestion("Failed to load column metadata", prefix.len())
                    ];
                }
            } else if prefix.is_empty() {
                return vec![
                    self.status_suggestion("No columns found for specified table(s)", prefix.len())
                ];
            } else {
                return vec![self.status_suggestion(
                    &format!("No columns matching '{}'", prefix),
                    prefix.len(),
                )];
            }
        }

        suggestions
    }

    /// Convert ColumnInfo to Suggestion
    fn column_to_suggestion(
        &self,
        col: &ColumnInfo,
        table_name: &str,
        prefix_len: usize,
    ) -> Suggestion {
        Suggestion {
            value: col.name.clone(),
            description: Some(format!("{}.{} ({})", table_name, col.name, col.data_type)),
            style: None,
            extra: None,
            span: reedline::Span {
                start: 0,
                end: prefix_len,
            },
            append_whitespace: false, // Don't add space after column name
        }
    }

    /// Calculate the span for completion replacement
    fn calculate_span(&self, line: &str, prefix_len: usize) -> (usize, usize) {
        // Find start of current word
        let start = line.len().saturating_sub(prefix_len);
        (start, line.len())
    }

    /// Create an error suggestion to display to user
    ///
    /// Sprint 8: Surfaces errors as user-visible feedback instead of silent failures.
    fn error_suggestion(&self, message: &str, prefix_len: usize) -> Suggestion {
        Suggestion {
            value: String::new(), // Empty value - can't be selected
            description: Some(format!("[Error: {}]", message)),
            style: None,
            extra: None,
            span: reedline::Span {
                start: 0,
                end: prefix_len,
            },
            append_whitespace: false,
        }
    }

    /// Create a status suggestion (e.g., "Loading..." or "No tables found")
    ///
    /// Sprint 8: Provides user feedback during operations.
    fn status_suggestion(&self, message: &str, prefix_len: usize) -> Suggestion {
        Suggestion {
            value: String::new(), // Empty value - can't be selected
            description: Some(format!("[{}]", message)),
            style: None,
            extra: None,
            span: reedline::Span {
                start: 0,
                end: prefix_len,
            },
            append_whitespace: false,
        }
    }
}

impl Default for MetadataCompleter {
    fn default() -> Self {
        Self::keywords_only()
    }
}

impl Completer for MetadataCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // Sprint 9 Bug 2 Fix: Support multi-line context by prepending accumulated buffer
        let (full_text, adjusted_pos) = if let Some(state) = &self.state {
            let state_lock = state.lock().unwrap();
            let accumulated = state_lock.accumulated_buffer();
            if !accumulated.is_empty() {
                // Prepend accumulated buffer to current line
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
        let context = analyze_context(&full_text, adjusted_pos);

        log::debug!("Completion context: {:?} (full_text len: {}, pos: {})", context, full_text.len(), adjusted_pos);

        let mut suggestions = match context {
            CompletionContext::Keyword => {
                let last_word = get_last_word(line);
                self.complete_keywords(last_word)
            }

            CompletionContext::TableName { prefix } => {
                // Sprint 11 Bug Fix: Do NOT fall back to keywords when in table context.
                // Users expect table/database names here, not SQL keywords.
                // If metadata loading fails, show the error/status message instead.
                self.complete_tables(&prefix)
            }

            CompletionContext::SchemaQualifiedTable { schema, prefix } => {
                self.complete_schema_tables(&schema, &prefix)
            }

            CompletionContext::ColumnName {
                tables,
                prefix,
                table_qualifier: _,
            } => {
                // Sprint 11 Bug Fix: Do NOT fall back to keywords when in column context.
                // Users expect column names here, not SQL keywords.
                // If metadata loading fails, show the error/status message instead.
                self.complete_columns(&tables, &prefix, None)
            }
        };

        // Fix span for all suggestions based on actual line position
        let last_word = get_last_word(line);
        let (start, end) = self.calculate_span(line, last_word.len());

        for sug in &mut suggestions {
            sug.span = reedline::Span { start, end };
        }

        // Sort: actual suggestions first (non-empty values), then by length and alphabetically
        // Status/error messages (empty values) should appear at the end
        suggestions.sort_by(|a, b| {
            // First, prioritize non-empty values
            let a_empty = a.value.is_empty();
            let b_empty = b.value.is_empty();
            if a_empty != b_empty {
                return a_empty.cmp(&b_empty);
            }
            // Then sort by length and alphabetically
            a.value
                .len()
                .cmp(&b.value.len())
                .then_with(|| a.value.cmp(&b.value))
        });

        // Limit to reasonable number
        // Sprint 8 Bug Fix: Increased from 20 to 50 to show more completions
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

    #[test]
    fn test_keywords_only_completer() {
        let mut completer = MetadataCompleter::keywords_only();

        let suggestions = completer.complete("SEL", 3);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.value == "SELECT"));
    }

    #[test]
    fn test_keyword_completion_case_insensitive() {
        let mut completer = MetadataCompleter::keywords_only();

        let suggestions = completer.complete("sel", 3);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.value == "SELECT"));
    }

    #[test]
    fn test_complete_no_match() {
        let mut completer = MetadataCompleter::keywords_only();

        let suggestions = completer.complete("XYZ", 3);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_complete_empty_prefix_table_context() {
        let mut completer = MetadataCompleter::keywords_only();

        let suggestions = completer.complete("SELECT * FROM ", 14);
        // Sprint 11 Bug Fix: Without database connection, we should show a
        // status message about no database connection, NOT fall back to keywords.
        // This is the correct behavior - users expect table names after FROM.
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].value.is_empty()); // Status message has empty value
        assert!(suggestions[0]
            .description
            .as_ref()
            .unwrap()
            .contains("No database connection"));
    }

    #[test]
    fn test_get_last_word() {
        assert_eq!(get_last_word("SELECT * FROM emp"), "emp");
        assert_eq!(get_last_word("SELECT * FROM "), "");
        assert_eq!(get_last_word("SELECT e.name"), "e.name");
        assert_eq!(get_last_word("WHERE id = "), "");
    }

    #[test]
    fn test_default_keywords() {
        let keywords = MetadataCompleter::default_keywords();
        assert!(keywords.contains(&"SELECT".to_string()));
        assert!(keywords.contains(&"FROM".to_string()));
        assert!(keywords.contains(&"WHERE".to_string()));
        assert!(keywords.contains(&"JOIN".to_string()));
    }

    #[test]
    fn test_error_suggestion_format() {
        let completer = MetadataCompleter::keywords_only();
        let suggestion = completer.error_suggestion("Connection failed", 3);

        assert!(suggestion.value.is_empty());
        assert!(suggestion.description.as_ref().unwrap().contains("[Error:"));
        assert!(suggestion
            .description
            .as_ref()
            .unwrap()
            .contains("Connection failed"));
    }

    #[test]
    fn test_status_suggestion_format() {
        let completer = MetadataCompleter::keywords_only();
        let suggestion = completer.status_suggestion("No tables found", 3);

        assert!(suggestion.value.is_empty());
        assert!(suggestion
            .description
            .as_ref()
            .unwrap()
            .contains("[No tables found]"));
    }

    #[test]
    fn test_complete_tables_no_connection() {
        // Without a connection, complete_tables should return a status message
        let completer = MetadataCompleter::keywords_only();
        let suggestions = completer.complete_tables("emp");

        // Should have one status message (empty value with description)
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].value.is_empty());
        assert!(suggestions[0]
            .description
            .as_ref()
            .unwrap()
            .contains("No database connection"));
    }

    // Sprint 11 Bug Fix Tests: Verify no fallback to keywords

    #[test]
    fn test_table_context_no_keyword_fallback() {
        // Sprint 11: When in table context (after FROM), we should NOT fall back to keywords.
        // This test verifies that "SELECT * FROM S<TAB>" does NOT show SQL keywords.
        let mut completer = MetadataCompleter::keywords_only();

        // After FROM with a prefix - this is table context
        let suggestions = completer.complete("SELECT * FROM S", 15);

        // Should show status message about no connection, NOT keywords like "SELECT", "SET"
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].value.is_empty());
        assert!(!suggestions
            .iter()
            .any(|s| s.description.as_ref().unwrap().contains("keyword")));
    }

    #[test]
    fn test_column_context_no_keyword_fallback() {
        // Sprint 11: When in column context (after WHERE), we should NOT fall back to keywords.
        // We can't fully test this without a database connection (need table context first),
        // but we can test that keyword completion is NOT used inappropriately.
        let mut completer = MetadataCompleter::keywords_only();

        // This input has a FROM clause (establishes table context) and WHERE (column context)
        let suggestions = completer.complete("SELECT * FROM employees WHERE n", 31);

        // Should show status message about determining table context, NOT keywords
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].value.is_empty());
        // Should NOT contain "AND", "NOT", "NULL" etc. - just context message
        assert!(!suggestions
            .iter()
            .any(|s| s.description.as_ref().unwrap().contains("keyword")));
    }

    #[test]
    fn test_keyword_context_still_works() {
        // Sprint 11: Keyword completion should still work in keyword context (start of line)
        let mut completer = MetadataCompleter::keywords_only();

        let suggestions = completer.complete("SEL", 3);

        // Should have keyword suggestions
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.value == "SELECT"));
        assert!(suggestions
            .iter()
            .all(|s| s.description.as_ref().unwrap().contains("keyword")));
    }

    #[test]
    fn test_schema_qualified_table_no_fallback() {
        // Sprint 11: Schema-qualified table completion (schema.) should not fall back to keywords
        let mut completer = MetadataCompleter::keywords_only();

        let suggestions = completer.complete("SELECT * FROM prod.", 19);

        // Should show status message about no connection, NOT keywords
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].value.is_empty());
        assert!(!suggestions
            .iter()
            .any(|s| s.description.as_ref().unwrap().contains("keyword")));
    }
}
