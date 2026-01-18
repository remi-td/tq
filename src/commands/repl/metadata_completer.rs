//! Metadata-aware SQL completer for reedline
//!
//! Provides context-sensitive tab completion that includes:
//! - SQL keywords (always available)
//! - Table names (after FROM, JOIN, UPDATE, INSERT INTO)
//! - Column names (after SELECT, WHERE, ORDER BY with table context)
//!
//! ## Design
//! - Uses shared MetadataCache for fast cached lookups
//! - Triggers lazy loading of metadata on first table completion request
//! - Falls back to keyword completion when metadata unavailable
//!
//! Sprint 7 implementation.

use super::sql_context::{analyze_context, CompletionContext};
use crate::db::{ColumnInfo, DatabaseClient, MetadataCache, TableInfo};
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
}

impl CompletionState {
    /// Create new completion state
    pub fn new(client: DatabaseClient, database: impl Into<String>) -> Self {
        Self {
            client,
            cache: MetadataCache::new(database),
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
    pub fn ensure_tables_loaded(&mut self) -> bool {
        if !self.cache.has_tables() {
            self.cache.load_tables(&self.client)
        } else {
            true
        }
    }

    /// Ensure columns are loaded for a table
    pub fn ensure_columns_loaded(&mut self, table_name: &str) -> bool {
        if self.cache.get_columns(table_name).is_none() {
            self.cache.load_columns(&self.client, table_name)
        } else {
            true
        }
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
    fn complete_tables(&self, prefix: &str) -> Vec<Suggestion> {
        let Some(state) = &self.state else {
            return vec![];
        };

        let Ok(mut state) = state.lock() else {
            log::warn!("Failed to acquire lock for table completion");
            return vec![];
        };

        // Ensure tables are loaded
        if !state.ensure_tables_loaded() {
            return vec![];
        }

        let tables = state.cache().find_tables_by_prefix(prefix);

        tables
            .into_iter()
            .map(|t| self.table_to_suggestion(t, prefix.len()))
            .collect()
    }

    /// Get schema-qualified table completions
    fn complete_schema_tables(&self, schema: &str, prefix: &str) -> Vec<Suggestion> {
        let Some(state) = &self.state else {
            return vec![];
        };

        let Ok(mut state) = state.lock() else {
            log::warn!("Failed to acquire lock for schema-qualified table completion");
            return vec![];
        };

        // Ensure tables are loaded
        if !state.ensure_tables_loaded() {
            return vec![];
        }

        // Find tables matching schema.prefix
        let full_prefix = format!("{}.{}", schema, prefix);
        let tables = state.cache().find_tables_by_prefix(&full_prefix);

        tables
            .into_iter()
            .map(|t| {
                // Return just the table name (schema already typed)
                Suggestion {
                    value: t.table_name.clone(),
                    description: Some(format!("{}.{}", t.schema_name, t.table_name)),
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
    fn complete_columns(
        &self,
        tables: &[super::sql_context::TableReference],
        prefix: &str,
        _qualifier: Option<&str>,
    ) -> Vec<Suggestion> {
        let Some(state) = &self.state else {
            return vec![];
        };

        let Ok(mut state) = state.lock() else {
            log::warn!("Failed to acquire lock for column completion");
            return vec![];
        };

        let mut suggestions = Vec::new();

        for table in tables {
            // Try to load columns for this table
            state.ensure_columns_loaded(&table.name);

            // Get matching columns
            let columns = state.cache().find_columns_by_prefix(&table.name, prefix);

            for col in columns {
                suggestions.push(self.column_to_suggestion(col, &table.name, prefix.len()));
            }
        }

        suggestions
    }

    /// Convert TableInfo to Suggestion
    fn table_to_suggestion(&self, table: &TableInfo, prefix_len: usize) -> Suggestion {
        let kind = match table.table_kind.as_str() {
            "T" => "table",
            "V" => "view",
            "O" => "object",
            _ => "table",
        };

        Suggestion {
            value: table.table_name.clone(),
            description: Some(format!("{} ({})", table.full_name, kind)),
            style: None,
            extra: None,
            span: reedline::Span {
                start: 0,
                end: prefix_len,
            },
            append_whitespace: true,
        }
    }

    /// Convert ColumnInfo to Suggestion
    fn column_to_suggestion(&self, col: &ColumnInfo, table_name: &str, prefix_len: usize) -> Suggestion {
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
}

impl Default for MetadataCompleter {
    fn default() -> Self {
        Self::keywords_only()
    }
}

impl Completer for MetadataCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // Analyze context to determine what kind of completions to provide
        let context = analyze_context(line, pos);

        log::debug!("Completion context: {:?}", context);

        let mut suggestions = match context {
            CompletionContext::Keyword => {
                let last_word = get_last_word(line);
                self.complete_keywords(last_word)
            }

            CompletionContext::TableName { prefix } => {
                let mut sug = self.complete_tables(&prefix);
                // Also include matching keywords as fallback
                if sug.is_empty() {
                    sug = self.complete_keywords(&prefix);
                }
                sug
            }

            CompletionContext::SchemaQualifiedTable { schema, prefix } => {
                self.complete_schema_tables(&schema, &prefix)
            }

            CompletionContext::ColumnName {
                tables,
                prefix,
                table_qualifier: _,
            } => {
                let mut sug = self.complete_columns(&tables, &prefix, None);
                // Also include matching keywords as fallback
                if sug.is_empty() {
                    sug = self.complete_keywords(&prefix);
                }
                sug
            }
        };

        // Fix span for all suggestions based on actual line position
        let last_word = get_last_word(line);
        let (start, end) = self.calculate_span(line, last_word.len());

        for sug in &mut suggestions {
            sug.span = reedline::Span { start, end };
        }

        // Sort: shorter matches first, then alphabetically
        suggestions.sort_by(|a, b| {
            a.value
                .len()
                .cmp(&b.value.len())
                .then_with(|| a.value.cmp(&b.value))
        });

        // Limit to reasonable number
        suggestions.truncate(20);

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
    fn test_complete_empty_prefix() {
        let mut completer = MetadataCompleter::keywords_only();

        let suggestions = completer.complete("SELECT * FROM ", 14);
        // Without database connection, should return empty for table context
        // but the completer falls back to keywords
        // This is correct behavior - no matching keywords either
        assert!(suggestions.is_empty() || suggestions.iter().any(|s| s.description.as_ref().unwrap().contains("keyword")));
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
}
