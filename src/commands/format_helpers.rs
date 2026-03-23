//! Shared formatting helpers for command modules
//!
//! This module centralizes utility functions that were previously duplicated
//! across describe.rs, list.rs, show_indexes.rs, inspect.rs, and metacommands.rs.
//!
//! Functions:
//! - `json_escape` — Escape a string for JSON output
//! - `csv_escape` — Escape a string for CSV output
//! - `parse_table_name` — Split "db.table" into components
//! - `truncate_str` — UTF-8-safe string truncation with ellipsis
//! - `format_nullable` — Normalize nullable indicators to YES/NO
//! - `column_type_sql` — Generate SQL CASE expression for type code translation

/// Escape a string for safe inclusion in JSON output.
///
/// Handles backslashes, double quotes, newlines, carriage returns, and tabs.
pub fn json_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                // Escape other control characters as \uXXXX
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

/// Escape a string for safe inclusion in CSV output.
///
/// Wraps the string in double quotes if it contains a comma, double quote,
/// or newline character. Internal double quotes are escaped by doubling them.
pub fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Parse a potentially qualified object name into (optional database, object).
///
/// Handles both `database.object` and `object` forms.
/// The first dot is the separator; any further dots remain in the object part.
pub fn parse_table_name(name: &str) -> (Option<&str>, &str) {
    if let Some(dot_pos) = name.find('.') {
        let db = &name[..dot_pos];
        let obj = &name[dot_pos + 1..];
        (Some(db), obj)
    } else {
        (None, name)
    }
}

/// Truncate a string to a maximum display length, appending "..." if truncated.
///
/// Uses `char_indices()` for proper UTF-8 boundary handling. Supports
/// multi-byte characters (CJK, emoji) without panicking on byte boundaries.
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        return s.to_string();
    }
    if max_len <= 3 {
        return ".".repeat(max_len);
    }
    let target = max_len - 3;
    let byte_end = s
        .char_indices()
        .nth(target)
        .map(|(idx, _)| idx)
        .unwrap_or(s.len());
    format!("{}...", &s[..byte_end])
}

/// Format a nullable indicator to a consistent YES/NO string.
///
/// Recognizes Y, YES, TRUE, 1 as YES and N, NO, FALSE, 0 as NO.
/// Returns the original string for unrecognized values.
pub fn format_nullable(s: &str) -> String {
    match s.trim().to_uppercase().as_str() {
        "Y" | "YES" | "TRUE" | "1" => "YES".to_string(),
        "N" | "NO" | "FALSE" | "0" => "NO".to_string(),
        _ => s.to_string(),
    }
}

/// Returns "-" for empty or whitespace-only values, otherwise the original string.
pub fn format_null_display(val: &str) -> &str {
    if val.trim().is_empty() || val == "[NULL]" {
        "-"
    } else {
        val
    }
}

/// Generate a SQL CASE expression that translates Teradata type codes
/// from DBC.ColumnsV into human-readable type names.
///
/// The returned fragment uses these column references from DBC.ColumnsV:
/// - `ColumnType` (the 2-char type code)
/// - `ColumnLength` (for CHAR/VARCHAR/BYTE/VARBYTE)
/// - `DecimalTotalDigits` (for DECIMAL precision)
/// - `DecimalFractionalDigits` (for DECIMAL scale)
///
/// This can be embedded in a SELECT statement as a column expression.
pub fn column_type_case_sql() -> &'static str {
    "CASE TRIM(ColumnType) \
         WHEN 'CV' THEN 'VARCHAR(' || TRIM(CAST(ColumnLength AS VARCHAR(10))) || ')' \
         WHEN 'CF' THEN 'CHAR(' || TRIM(CAST(ColumnLength AS VARCHAR(10))) || ')' \
         WHEN 'I'  THEN 'INTEGER' \
         WHEN 'I1' THEN 'BYTEINT' \
         WHEN 'I2' THEN 'SMALLINT' \
         WHEN 'I8' THEN 'BIGINT' \
         WHEN 'DA' THEN 'DATE' \
         WHEN 'TS' THEN 'TIMESTAMP' \
         WHEN 'D'  THEN 'DECIMAL(' || TRIM(CAST(DecimalTotalDigits AS VARCHAR(10))) || ',' || TRIM(CAST(DecimalFractionalDigits AS VARCHAR(10))) || ')' \
         WHEN 'F'  THEN 'FLOAT' \
         WHEN 'AT' THEN 'TIME' \
         WHEN 'SZ' THEN 'TIMESTAMP WITH TIME ZONE' \
         WHEN 'BF' THEN 'BYTE(' || TRIM(CAST(ColumnLength AS VARCHAR(10))) || ')' \
         WHEN 'BV' THEN 'VARBYTE(' || TRIM(CAST(ColumnLength AS VARCHAR(10))) || ')' \
         WHEN 'CO' THEN 'CLOB' \
         WHEN 'BO' THEN 'BLOB' \
         WHEN 'N'  THEN 'NUMBER' \
         WHEN 'JN' THEN 'JSON' \
         WHEN 'AN' THEN 'ARRAY' \
         WHEN 'UT' THEN 'UDT' \
         WHEN 'PM' THEN 'PERIOD(TIMESTAMP)' \
         WHEN 'PD' THEN 'PERIOD(DATE)' \
         WHEN 'PS' THEN 'PERIOD(TIMESTAMP WITH TIME ZONE)' \
         WHEN 'PT' THEN 'PERIOD(TIME)' \
         ELSE TRIM(ColumnType) \
     END"
}

/// Format a byte count as a human-readable size string.
///
/// The `precision` parameter controls the number of decimal places (e.g., 1 for
/// "1.5 MB", 2 for "1.50 MB"). Values below 1024 bytes are always displayed
/// without decimals.
pub fn format_size(bytes: i64, precision: usize) -> String {
    if bytes < 0 {
        return format!("{} B", bytes);
    }

    const KB: i64 = 1024;
    const MB: i64 = 1024 * KB;
    const GB: i64 = 1024 * MB;
    const TB: i64 = 1024 * GB;

    if bytes >= TB {
        format!("{:.prec$} TB", bytes as f64 / TB as f64, prec = precision)
    } else if bytes >= GB {
        format!("{:.prec$} GB", bytes as f64 / GB as f64, prec = precision)
    } else if bytes >= MB {
        format!("{:.prec$} MB", bytes as f64 / MB as f64, prec = precision)
    } else if bytes >= KB {
        format!("{:.prec$} KB", bytes as f64 / KB as f64, prec = precision)
    } else {
        format!("{} B", bytes)
    }
}

/// Map a TableKind character from DBC.TablesV to a human-readable label.
pub fn map_table_kind(kind: &str) -> String {
    match kind {
        "T" => "Table".to_string(),
        "O" => "Table (NoPI)".to_string(),
        "V" => "View".to_string(),
        "M" => "Macro".to_string(),
        "P" => "Stored Procedure".to_string(),
        "G" => "Trigger".to_string(),
        "A" => "Aggregate".to_string(),
        "E" => "External SP".to_string(),
        "N" => "Hash Index".to_string(),
        "I" => "Join Index".to_string(),
        other => format!("Unknown ({})", other),
    }
}

/// Map an index type character and unique flag to structured labels.
///
/// Returns `(type_label, short_label)` where short_label is like "UPI", "NUSI", etc.
pub fn classify_index(index_type: &str, is_unique: bool) -> (&'static str, &'static str) {
    let uniqueness = if is_unique { "U" } else { "NU" };
    match index_type.trim() {
        "P" | "Primary" => {
            if is_unique {
                ("Primary Index", "UPI")
            } else {
                ("Primary Index", "NUPI")
            }
        }
        "S" | "Secondary" => {
            if is_unique {
                ("Secondary Index", "USI")
            } else {
                ("Secondary Index", "NUSI")
            }
        }
        "Q" | "PPI" => ("Partitioned Primary Index", "PPI"),
        "J" | "Join" => ("Join Index", "JI"),
        "K" | "Primary Key" => ("Primary Key", "PK"),
        "U" | "Unique" => ("Unique Index", "UI"),
        "V" | "Value-Ordered" => ("Value-Ordered Index", "VOSI"),
        "H" | "Hash" => ("Hash Index", "HI"),
        _ => {
            if is_unique {
                ("Index", "UI")
            } else {
                let _ = uniqueness; // suppress unused variable
                ("Index", "I")
            }
        }
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // json_escape
    // =========================================================================

    #[test]
    fn test_json_escape_plain_text() {
        assert_eq!(json_escape("hello world"), "hello world");
    }

    #[test]
    fn test_json_escape_quotes() {
        assert_eq!(json_escape(r#"say "hi""#), r#"say \"hi\""#);
    }

    #[test]
    fn test_json_escape_backslashes() {
        assert_eq!(json_escape(r"path\to\file"), r"path\\to\\file");
    }

    #[test]
    fn test_json_escape_newlines() {
        assert_eq!(json_escape("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_json_escape_tabs() {
        assert_eq!(json_escape("col1\tcol2"), "col1\\tcol2");
    }

    #[test]
    fn test_json_escape_carriage_return() {
        assert_eq!(json_escape("a\rb"), "a\\rb");
    }

    #[test]
    fn test_json_escape_control_chars() {
        // Bell character (0x07)
        let s = String::from("\x07");
        assert_eq!(json_escape(&s), "\\u0007");
    }

    #[test]
    fn test_json_escape_combined() {
        assert_eq!(
            json_escape("line1\n\"quoted\"\t\\end"),
            "line1\\n\\\"quoted\\\"\\t\\\\end"
        );
    }

    #[test]
    fn test_json_escape_empty() {
        assert_eq!(json_escape(""), "");
    }

    // =========================================================================
    // csv_escape
    // =========================================================================

    #[test]
    fn test_csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn test_csv_escape_comma() {
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
    }

    #[test]
    fn test_csv_escape_quotes() {
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_csv_escape_newline() {
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn test_csv_escape_empty() {
        assert_eq!(csv_escape(""), "");
    }

    #[test]
    fn test_csv_escape_all_special() {
        assert_eq!(csv_escape("a,b\"c\nd"), "\"a,b\"\"c\nd\"");
    }

    // =========================================================================
    // parse_table_name
    // =========================================================================

    #[test]
    fn test_parse_unqualified() {
        let (db, obj) = parse_table_name("employees");
        assert!(db.is_none());
        assert_eq!(obj, "employees");
    }

    #[test]
    fn test_parse_qualified() {
        let (db, obj) = parse_table_name("mydb.employees");
        assert_eq!(db, Some("mydb"));
        assert_eq!(obj, "employees");
    }

    #[test]
    fn test_parse_multiple_dots() {
        let (db, obj) = parse_table_name("a.b.c");
        assert_eq!(db, Some("a"));
        assert_eq!(obj, "b.c");
    }

    #[test]
    fn test_parse_empty() {
        let (db, obj) = parse_table_name("");
        assert!(db.is_none());
        assert_eq!(obj, "");
    }

    // =========================================================================
    // truncate_str (UTF-8 safe)
    // =========================================================================

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate_str("exactly10c", 10), "exactly10c");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate_str("this is a long string", 10), "this is...");
    }

    #[test]
    fn test_truncate_max_three() {
        assert_eq!(truncate_str("abcdef", 3), "...");
    }

    #[test]
    fn test_truncate_max_zero() {
        assert_eq!(truncate_str("abc", 0), "");
    }

    #[test]
    fn test_truncate_multibyte_utf8() {
        // German umlauts (2 bytes each)
        let s = "Uberfuhrung";
        assert_eq!(truncate_str(s, 8), "Uberf...");
    }

    #[test]
    fn test_truncate_cjk_characters() {
        // Each CJK character is 3 bytes but 1 char
        let s = "\u{4e2d}\u{6587}\u{5b57}\u{7b26}\u{6d4b}\u{8bd5}"; // 6 CJK chars
        let result = truncate_str(s, 5);
        // Should be 2 CJK chars + "..."
        assert_eq!(result.chars().count(), 5);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_emoji() {
        let s = "\u{1f600}\u{1f601}\u{1f602}\u{1f603}\u{1f604}\u{1f605}"; // 6 emoji (4 bytes each)
        let result = truncate_str(s, 5);
        // Should be 2 emoji + "..."
        assert_eq!(result.chars().count(), 5);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_emoji_exact_fit() {
        let s = "\u{1f600}\u{1f601}\u{1f602}\u{1f603}\u{1f604}"; // 5 emoji
        let result = truncate_str(s, 5);
        // Exactly fits, no truncation
        assert_eq!(result, s);
    }

    #[test]
    fn test_truncate_shorter_than_max() {
        assert_eq!(truncate_str("ab", 10), "ab");
    }

    // =========================================================================
    // format_nullable
    // =========================================================================

    #[test]
    fn test_format_nullable_yes_variants() {
        assert_eq!(format_nullable("Y"), "YES");
        assert_eq!(format_nullable("YES"), "YES");
        assert_eq!(format_nullable("TRUE"), "YES");
        assert_eq!(format_nullable("1"), "YES");
        assert_eq!(format_nullable(" y "), "YES");
    }

    #[test]
    fn test_format_nullable_no_variants() {
        assert_eq!(format_nullable("N"), "NO");
        assert_eq!(format_nullable("NO"), "NO");
        assert_eq!(format_nullable("FALSE"), "NO");
        assert_eq!(format_nullable("0"), "NO");
    }

    #[test]
    fn test_format_nullable_unknown() {
        assert_eq!(format_nullable("maybe"), "maybe");
    }

    // =========================================================================
    // format_null_display
    // =========================================================================

    #[test]
    fn test_format_null_display_empty() {
        assert_eq!(format_null_display(""), "-");
    }

    #[test]
    fn test_format_null_display_whitespace() {
        assert_eq!(format_null_display("   "), "-");
    }

    #[test]
    fn test_format_null_display_null_marker() {
        assert_eq!(format_null_display("[NULL]"), "-");
    }

    #[test]
    fn test_format_null_display_value() {
        assert_eq!(format_null_display("hello"), "hello");
    }

    // =========================================================================
    // column_type_case_sql
    // =========================================================================

    #[test]
    fn test_column_type_case_sql_contains_types() {
        let sql = column_type_case_sql();
        assert!(sql.contains("VARCHAR"));
        assert!(sql.contains("INTEGER"));
        assert!(sql.contains("DECIMAL"));
        assert!(sql.contains("TIMESTAMP WITH TIME ZONE"));
        assert!(sql.contains("JSON"));
        assert!(sql.contains("PERIOD(DATE)"));
    }

    // =========================================================================
    // map_table_kind
    // =========================================================================

    #[test]
    fn test_map_table_kind_all() {
        assert_eq!(map_table_kind("T"), "Table");
        assert_eq!(map_table_kind("O"), "Table (NoPI)");
        assert_eq!(map_table_kind("V"), "View");
        assert_eq!(map_table_kind("M"), "Macro");
        assert_eq!(map_table_kind("P"), "Stored Procedure");
    }

    #[test]
    fn test_map_table_kind_unknown() {
        assert_eq!(map_table_kind("X"), "Unknown (X)");
    }

    // =========================================================================
    // classify_index
    // =========================================================================

    #[test]
    fn test_classify_index_upi() {
        let (label, short) = classify_index("P", true);
        assert_eq!(label, "Primary Index");
        assert_eq!(short, "UPI");
    }

    #[test]
    fn test_classify_index_nupi() {
        let (label, short) = classify_index("P", false);
        assert_eq!(label, "Primary Index");
        assert_eq!(short, "NUPI");
    }

    #[test]
    fn test_classify_index_usi() {
        let (label, short) = classify_index("S", true);
        assert_eq!(label, "Secondary Index");
        assert_eq!(short, "USI");
    }

    #[test]
    fn test_classify_index_nusi() {
        let (label, short) = classify_index("S", false);
        assert_eq!(label, "Secondary Index");
        assert_eq!(short, "NUSI");
    }

    #[test]
    fn test_classify_index_from_label() {
        // Also works with human-readable labels from CASE expressions
        let (label, short) = classify_index("Primary", true);
        assert_eq!(label, "Primary Index");
        assert_eq!(short, "UPI");
    }

    #[test]
    fn test_classify_index_ppi() {
        let (_, short) = classify_index("Q", false);
        assert_eq!(short, "PPI");
    }

    #[test]
    fn test_classify_index_unknown() {
        let (label, short) = classify_index("X", false);
        assert_eq!(label, "Index");
        assert_eq!(short, "I");
    }

    // =========================================================================
    // format_size
    // =========================================================================

    #[test]
    fn test_format_size_precision_1() {
        assert_eq!(format_size(0, 1), "0 B");
        assert_eq!(format_size(512, 1), "512 B");
        assert_eq!(format_size(1024, 1), "1.0 KB");
        assert_eq!(format_size(1536, 1), "1.5 KB");
        assert_eq!(format_size(1048576, 1), "1.0 MB");
        assert_eq!(format_size(1073741824, 1), "1.0 GB");
        assert_eq!(format_size(1099511627776, 1), "1.0 TB");
        assert_eq!(format_size(-100, 1), "-100 B");
    }

    #[test]
    fn test_format_size_precision_2() {
        assert_eq!(format_size(0, 2), "0 B");
        assert_eq!(format_size(1024, 2), "1.00 KB");
        assert_eq!(format_size(1536, 2), "1.50 KB");
        assert_eq!(format_size(1048576, 2), "1.00 MB");
        assert_eq!(format_size(1572864, 2), "1.50 MB");
        assert_eq!(format_size(1073741824, 2), "1.00 GB");
        assert_eq!(format_size(1319413964, 2), "1.23 GB");
        assert_eq!(format_size(1099511627776, 2), "1.00 TB");
        assert_eq!(format_size(-100, 2), "-100 B");
    }

    // =========================================================================
    // column_type_case_sql completeness (all 21 WHEN branches)
    // =========================================================================

    #[test]
    fn test_column_type_case_sql_all_21_when_branches() {
        let sql = column_type_case_sql();
        // All 21 type codes must be present
        let expected_codes = vec![
            "CV", "CF", "'I'", "I1", "I2", "I8", "DA", "TS", "'D'", "'F'",
            "AT", "SZ", "BF", "BV", "CO", "BO", "'N'", "JN", "AN", "UT",
            "PM", "PD", "PS", "PT",
        ];
        for code in &expected_codes {
            assert!(
                sql.contains(code),
                "column_type_case_sql missing type code: {}",
                code
            );
        }
        // Verify human-readable type names
        let expected_types = vec![
            "VARCHAR", "CHAR", "INTEGER", "BYTEINT", "SMALLINT", "BIGINT",
            "DATE", "TIMESTAMP", "DECIMAL", "FLOAT", "TIME",
            "TIMESTAMP WITH TIME ZONE", "BYTE", "VARBYTE", "CLOB", "BLOB",
            "NUMBER", "JSON", "ARRAY", "UDT", "PERIOD(TIMESTAMP)",
            "PERIOD(DATE)", "PERIOD(TIMESTAMP WITH TIME ZONE)", "PERIOD(TIME)",
        ];
        for type_name in &expected_types {
            assert!(
                sql.contains(type_name),
                "column_type_case_sql missing type name: {}",
                type_name
            );
        }
    }
}
