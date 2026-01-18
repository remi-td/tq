//! SQL context parsing for tab completion
//!
//! This module analyzes partial SQL statements to determine the appropriate
//! completion context (keyword, table name, column name, etc.).
//!
//! ## Design
//! - Simple regex-based parsing (not a full SQL parser)
//! - Handles common cases: FROM, JOIN, UPDATE, INSERT INTO, SELECT, WHERE, ORDER BY
//! - Gracefully handles edge cases by falling back to keyword completion

use std::collections::HashSet;

/// The context for tab completion
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionContext {
    /// Complete SQL keywords (default)
    Keyword,

    /// Complete table names after FROM, JOIN, UPDATE, INSERT INTO
    TableName {
        /// Prefix the user has typed (if any)
        prefix: String,
    },

    /// Complete column names for a specific table
    ColumnName {
        /// Table(s) in context (from FROM clause)
        tables: Vec<TableReference>,
        /// Prefix the user has typed (if any)
        prefix: String,
        /// Optional table alias being qualified (e.g., "e." in "e.<TAB>")
        table_qualifier: Option<String>,
    },

    /// Complete schema-qualified table (after "schema.")
    SchemaQualifiedTable {
        /// Schema name
        schema: String,
        /// Table prefix
        prefix: String,
    },
}

/// A table reference with optional alias
#[derive(Debug, Clone, PartialEq)]
pub struct TableReference {
    /// Full table name (may be schema.table)
    pub name: String,
    /// Optional alias
    pub alias: Option<String>,
}

impl TableReference {
    /// Create a new table reference
    pub fn new(name: impl Into<String>, alias: Option<String>) -> Self {
        Self {
            name: name.into(),
            alias,
        }
    }
}

/// Analyze SQL input and determine the completion context
///
/// # Arguments
/// * `line` - The current input line
/// * `cursor_pos` - The cursor position in the line
///
/// # Returns
/// The appropriate completion context for the cursor position
pub fn analyze_context(line: &str, cursor_pos: usize) -> CompletionContext {
    // Get text up to cursor
    let text = if cursor_pos <= line.len() {
        &line[..cursor_pos]
    } else {
        line
    };

    // Normalize: collapse whitespace
    let normalized = normalize_sql(text);
    let upper = normalized.to_uppercase();

    // Get the last token/word
    let last_word = get_last_word(text);

    // First, check for table alias qualification (e.g., "e.<TAB>")
    // But only if we have table context (not in FROM clause where "e." means schema)
    if let Some(tables) = extract_table_references(&normalized) {
        if let Some(ctx) = check_table_alias_qualified_with_tables(&normalized, text, &tables) {
            return ctx;
        }
    }

    // Check for schema-qualified completion in FROM context (typing after "schema.")
    if let Some(ctx) = check_schema_qualified(&upper, &last_word, text) {
        return ctx;
    }

    // Check what keyword comes immediately before cursor position
    // We need to find the most recent significant keyword

    // Check for table name context (FROM, JOIN, UPDATE, INSERT INTO)
    if is_table_context(&upper, &last_word) {
        return CompletionContext::TableName {
            prefix: last_word.to_string(),
        };
    }

    // Check for column context (WHERE, SELECT with FROM, etc.)
    if let Some(tables) = extract_table_references(&normalized) {
        if is_column_context(&upper, &last_word, &tables) {
            return CompletionContext::ColumnName {
                tables,
                prefix: last_word.to_string(),
                table_qualifier: None,
            };
        }
    }

    // Default to keyword completion
    CompletionContext::Keyword
}

/// Check if we're in a table name context
fn is_table_context(upper: &str, last_word: &str) -> bool {
    let words: Vec<&str> = upper.split_whitespace().collect();

    if words.is_empty() {
        return false;
    }

    // Find the last significant keyword
    for i in (0..words.len()).rev() {
        let word = words[i];

        // If the last word is what we're typing (partial word), check the previous keyword
        if i > 0 && !last_word.is_empty() && word == last_word.to_uppercase() {
            let prev = words[i - 1];
            if ["FROM", "JOIN", "UPDATE", "INTO"].contains(&prev) {
                return true;
            }
            // Check for multi-word JOIN variants
            if i > 1 && words[i - 1] == "JOIN" {
                return true;
            }
            if prev == "OUTER" && i > 1 && words[i - 2] == "JOIN" {
                return true;
            }
            continue;
        }

        // If we've just typed a table keyword and nothing else
        if last_word.is_empty() && ["FROM", "JOIN", "UPDATE", "INTO"].contains(&word) {
            return true;
        }
        // Also check multi-word JOIN at end
        if last_word.is_empty() && word == "JOIN" {
            return true;
        }

        // Stop if we hit a column context keyword
        if [
            "WHERE", "SELECT", "AND", "OR", "ON", "SET", "ORDER", "GROUP", "HAVING",
        ]
        .contains(&word)
        {
            return false;
        }
    }

    false
}

/// Check if we're in a column name context
fn is_column_context(upper: &str, last_word: &str, tables: &[TableReference]) -> bool {
    if tables.is_empty() {
        return false;
    }

    let words: Vec<&str> = upper.split_whitespace().collect();

    if words.is_empty() {
        return false;
    }

    // Find the last significant keyword
    for i in (0..words.len()).rev() {
        let word = words[i];

        // Skip the partial word we're typing
        if i > 0 && !last_word.is_empty() && word == last_word.to_uppercase() {
            continue;
        }

        // Column context keywords
        if ["WHERE", "AND", "OR", "ON", "SET", "HAVING"].contains(&word) {
            return true;
        }

        // SELECT followed by something (and we have FROM)
        if word == "SELECT" && upper.contains(" FROM ") {
            return true;
        }

        // ORDER BY / GROUP BY
        if word == "BY" && i > 0 && (words[i - 1] == "ORDER" || words[i - 1] == "GROUP") {
            return true;
        }

        // Table context keywords mean we're NOT in column context yet
        if ["FROM", "JOIN", "UPDATE", "INTO"].contains(&word) {
            return false;
        }
    }

    false
}

/// Normalize SQL by collapsing whitespace
fn normalize_sql(sql: &str) -> String {
    sql.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Get the last word/token before cursor
fn get_last_word(text: &str) -> &str {
    // Find the last word that could be a partial identifier
    let trimmed = text.trim_end();

    // If line ends with whitespace, there's no partial word
    if text.ends_with(|c: char| c.is_whitespace()) {
        return "";
    }

    // Split on whitespace and non-identifier chars
    let bytes = trimmed.as_bytes();
    let mut start = trimmed.len();

    for i in (0..bytes.len()).rev() {
        let c = bytes[i] as char;
        if c.is_whitespace() || c == ',' || c == '(' || c == ')' || c == '=' {
            break;
        }
        start = i;
    }

    &trimmed[start..]
}

/// Check if cursor is positioned for schema-qualified completion in FROM context
fn check_schema_qualified(upper: &str, last_word: &str, text: &str) -> Option<CompletionContext> {
    // Only relevant if we're in a table context
    if !is_table_context(upper, "") {
        return None;
    }

    // Pattern: "schema." at end
    if text.trim_end().ends_with('.') && !last_word.is_empty() {
        // last_word before the dot
        let trimmed = text.trim_end();
        let without_dot = &trimmed[..trimmed.len() - 1];
        let schema = get_last_word(without_dot);

        if !schema.is_empty() {
            return Some(CompletionContext::SchemaQualifiedTable {
                schema: schema.to_string(),
                prefix: String::new(),
            });
        }
    }

    // Pattern: "schema.partial" (last_word contains a dot)
    if last_word.contains('.') {
        let parts: Vec<&str> = last_word.splitn(2, '.').collect();
        if parts.len() == 2 {
            return Some(CompletionContext::SchemaQualifiedTable {
                schema: parts[0].to_string(),
                prefix: parts[1].to_string(),
            });
        }
    }

    None
}

/// Check if cursor is after table alias qualification (e.g., "e.") with known tables
fn check_table_alias_qualified_with_tables(
    _normalized: &str,
    original: &str,
    tables: &[TableReference],
) -> Option<CompletionContext> {
    let last_word = get_last_word(original);

    // Check for "alias." pattern
    if last_word.ends_with('.') {
        let alias = &last_word[..last_word.len() - 1];
        let alias_upper = alias.to_uppercase();

        // Check if this is a known alias
        for table in tables {
            if let Some(ref tbl_alias) = table.alias {
                if tbl_alias.to_uppercase() == alias_upper {
                    return Some(CompletionContext::ColumnName {
                        tables: vec![table.clone()],
                        prefix: String::new(),
                        table_qualifier: Some(alias.to_string()),
                    });
                }
            }
        }

        // Also check table names (unaliased)
        for table in tables {
            let table_name_upper = table
                .name
                .split('.')
                .last()
                .unwrap_or(&table.name)
                .to_uppercase();
            if table_name_upper == alias_upper || table.name.to_uppercase() == alias_upper {
                return Some(CompletionContext::ColumnName {
                    tables: vec![table.clone()],
                    prefix: String::new(),
                    table_qualifier: Some(alias.to_string()),
                });
            }
        }
    }

    // Check for "alias.partial" pattern
    if last_word.contains('.') && !last_word.ends_with('.') {
        let parts: Vec<&str> = last_word.splitn(2, '.').collect();
        if parts.len() == 2 {
            let alias = parts[0];
            let prefix = parts[1];
            let alias_upper = alias.to_uppercase();

            // Check if this is a known alias or table name
            for table in tables {
                let is_match = table
                    .alias
                    .as_ref()
                    .map(|a| a.to_uppercase() == alias_upper)
                    .unwrap_or(false)
                    || table
                        .name
                        .split('.')
                        .last()
                        .unwrap_or(&table.name)
                        .to_uppercase()
                        == alias_upper;

                if is_match {
                    return Some(CompletionContext::ColumnName {
                        tables: vec![table.clone()],
                        prefix: prefix.to_string(),
                        table_qualifier: Some(alias.to_string()),
                    });
                }
            }
        }
    }

    None
}

/// Extract table references from SQL
///
/// Parses FROM clause to find table names and aliases.
fn extract_table_references(sql: &str) -> Option<Vec<TableReference>> {
    let upper = sql.to_uppercase();

    // Find FROM clause
    let from_pos = upper.find(" FROM ")?;
    let after_from = &sql[from_pos + 6..]; // 6 = " FROM ".len()

    // Find end of table list (WHERE, GROUP BY, ORDER BY, etc.)
    let end_keywords = [
        " WHERE ", " GROUP ", " ORDER ", " HAVING ", " LIMIT ", " UNION ",
    ];
    let table_section_end = end_keywords
        .iter()
        .filter_map(|kw| upper[from_pos..].find(kw))
        .min()
        .unwrap_or(after_from.len());

    let table_section = &after_from[..table_section_end.min(after_from.len())];

    // Parse table references (handle JOIN syntax)
    let mut tables = Vec::new();
    let mut current = table_section;

    loop {
        // Skip leading whitespace and commas
        current = current.trim_start_matches(|c: char| c.is_whitespace() || c == ',');

        if current.is_empty() {
            break;
        }

        let current_upper = current.to_uppercase();

        // Skip ON clause (e.g., "ON e.dept_id = d.id")
        if current_upper.starts_with("ON ") {
            // Find next JOIN or end
            if let Some(next_join_pos) = find_next_join_position(&current_upper) {
                current = &current[next_join_pos..];
                continue;
            } else {
                // No more JOINs, we're done
                break;
            }
        }

        // Check for JOIN keyword
        let join_offset = [
            "INNER JOIN ",
            "LEFT OUTER JOIN ",
            "RIGHT OUTER JOIN ",
            "FULL OUTER JOIN ",
            "LEFT JOIN ",
            "RIGHT JOIN ",
            "FULL JOIN ",
            "CROSS JOIN ",
            "JOIN ",
        ]
        .iter()
        .filter_map(|j| {
            if current_upper.starts_with(j) {
                Some(j.len())
            } else {
                None
            }
        })
        .next();

        if let Some(offset) = join_offset {
            current = current[offset..].trim_start();
            continue;
        }

        // Check if we hit a stop keyword (should not happen but safety)
        let first_word = current_upper.split_whitespace().next().unwrap_or("");
        if ["WHERE", "GROUP", "ORDER", "HAVING", "LIMIT", "UNION"].contains(&first_word) {
            break;
        }

        // Parse table name and optional alias
        if let Some((table_ref, rest)) = parse_table_reference(current) {
            tables.push(table_ref);
            current = rest;
        } else {
            break;
        }
    }

    if tables.is_empty() {
        None
    } else {
        Some(tables)
    }
}

/// Find the position of the next JOIN keyword in the string
fn find_next_join_position(upper: &str) -> Option<usize> {
    let join_keywords = [
        " INNER JOIN ",
        " LEFT OUTER JOIN ",
        " RIGHT OUTER JOIN ",
        " FULL OUTER JOIN ",
        " LEFT JOIN ",
        " RIGHT JOIN ",
        " FULL JOIN ",
        " CROSS JOIN ",
        " JOIN ",
    ];

    join_keywords.iter().filter_map(|kw| upper.find(kw)).min()
}

/// Parse a single table reference (name + optional alias)
fn parse_table_reference(s: &str) -> Option<(TableReference, &str)> {
    let s = s.trim_start();

    if s.is_empty() {
        return None;
    }

    // Find end of table name (could be schema.table)
    let mut name_end = 0;
    let mut chars = s.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' || c == '.' {
            name_end += c.len_utf8();
            chars.next();
        } else {
            break;
        }
    }

    if name_end == 0 {
        return None;
    }

    let table_name = s[..name_end].to_string();
    let rest = &s[name_end..];

    // Check for alias (optional "AS" keyword)
    let rest_trimmed = rest.trim_start();
    let rest_upper = rest_trimmed.to_uppercase();

    // Check for join keywords or clause endings (no alias)
    let stop_words: HashSet<&str> = [
        "JOIN", "INNER", "LEFT", "RIGHT", "FULL", "CROSS", "ON", "WHERE", "GROUP", "ORDER",
        "HAVING", "LIMIT", "UNION", ",",
    ]
    .iter()
    .copied()
    .collect();

    let first_word = rest_upper.split_whitespace().next().unwrap_or("");
    if stop_words.contains(first_word) || rest_trimmed.starts_with(',') {
        return Some((TableReference::new(table_name, None), rest));
    }

    // Try to parse alias
    let alias_start = if rest_upper.starts_with("AS ") {
        rest_trimmed[3..].trim_start() // Skip "AS "
    } else {
        rest_trimmed
    };

    // Parse alias name
    let mut alias_end = 0;
    for c in alias_start.chars() {
        if c.is_alphanumeric() || c == '_' {
            alias_end += c.len_utf8();
        } else {
            break;
        }
    }

    if alias_end > 0 {
        let alias = alias_start[..alias_end].to_string();
        let remaining = &alias_start[alias_end..];

        // Check alias isn't a keyword
        let alias_upper = alias.to_uppercase();
        if !stop_words.contains(alias_upper.as_str()) && alias_upper != "AS" {
            return Some((TableReference::new(table_name, Some(alias)), remaining));
        }
    }

    Some((TableReference::new(table_name, None), rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_context_keyword_default() {
        let ctx = analyze_context("SEL", 3);
        assert_eq!(ctx, CompletionContext::Keyword);
    }

    #[test]
    fn test_analyze_context_table_after_from() {
        let ctx = analyze_context("SELECT * FROM ", 14);
        assert!(matches!(ctx, CompletionContext::TableName { prefix } if prefix.is_empty()));

        let ctx = analyze_context("SELECT * FROM emp", 17);
        assert!(matches!(ctx, CompletionContext::TableName { prefix } if prefix == "emp"));
    }

    #[test]
    fn test_analyze_context_table_after_join() {
        let ctx = analyze_context("SELECT * FROM t1 JOIN ", 22);
        assert!(matches!(ctx, CompletionContext::TableName { .. }));

        let ctx = analyze_context("SELECT * FROM t1 LEFT JOIN ", 27);
        assert!(matches!(ctx, CompletionContext::TableName { .. }));
    }

    #[test]
    fn test_analyze_context_table_after_update() {
        let ctx = analyze_context("UPDATE ", 7);
        assert!(matches!(ctx, CompletionContext::TableName { .. }));
    }

    #[test]
    fn test_analyze_context_column_after_where() {
        let ctx = analyze_context("SELECT * FROM employees WHERE ", 30);
        if let CompletionContext::ColumnName { tables, .. } = ctx {
            assert_eq!(tables.len(), 1);
            assert_eq!(tables[0].name, "employees");
        } else {
            panic!("Expected ColumnName context");
        }
    }

    #[test]
    fn test_analyze_context_column_with_alias_qualifier() {
        let ctx = analyze_context("SELECT * FROM employees e WHERE e.", 34);
        if let CompletionContext::ColumnName {
            tables,
            table_qualifier,
            ..
        } = ctx
        {
            assert_eq!(tables.len(), 1);
            assert_eq!(table_qualifier, Some("e".to_string()));
        } else {
            panic!("Expected ColumnName context, got {:?}", ctx);
        }
    }

    #[test]
    fn test_analyze_context_schema_qualified() {
        let ctx = analyze_context("SELECT * FROM prod.", 19);
        if let CompletionContext::SchemaQualifiedTable { schema, prefix } = ctx {
            assert_eq!(schema, "prod");
            assert!(prefix.is_empty());
        } else {
            panic!("Expected SchemaQualifiedTable context, got {:?}", ctx);
        }
    }

    #[test]
    fn test_extract_table_references_simple() {
        let tables = extract_table_references("SELECT * FROM employees").unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "employees");
        assert_eq!(tables[0].alias, None);
    }

    #[test]
    fn test_extract_table_references_with_alias() {
        let tables = extract_table_references("SELECT * FROM employees e").unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "employees");
        assert_eq!(tables[0].alias, Some("e".to_string()));
    }

    #[test]
    fn test_extract_table_references_with_as_alias() {
        let tables = extract_table_references("SELECT * FROM employees AS e").unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "employees");
        assert_eq!(tables[0].alias, Some("e".to_string()));
    }

    #[test]
    fn test_extract_table_references_join() {
        let tables = extract_table_references(
            "SELECT * FROM employees e JOIN departments d ON e.dept_id = d.id",
        )
        .unwrap();
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].name, "employees");
        assert_eq!(tables[0].alias, Some("e".to_string()));
        assert_eq!(tables[1].name, "departments");
        assert_eq!(tables[1].alias, Some("d".to_string()));
    }

    #[test]
    fn test_extract_table_references_schema_qualified() {
        let tables = extract_table_references("SELECT * FROM prod.employees").unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "prod.employees");
    }

    #[test]
    fn test_get_last_word() {
        assert_eq!(get_last_word("SELECT * FROM emp"), "emp");
        assert_eq!(get_last_word("SELECT * FROM "), "");
        assert_eq!(get_last_word("WHERE name = "), "");
        assert_eq!(get_last_word("WHERE name"), "name");
    }
}
