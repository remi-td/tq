# TC-045-INSPECT-UNIT: /inspect Command — Unit Tests for Pure Logic Helpers

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-045-INSPECT-UNIT |
| **Title** | /inspect Command — Unit Tests for Pure Logic Helpers |
| **Category** | Unit Test |
| **Priority** | Critical |
| **Feature** | Sprint 45 — /inspect Command (Issue #33) |
| **Test Type** | Unit |
| **DB Required** | No |
| **Created** | 2026-03-23 |
| **Covers** | TC-045-008 through TC-045-013 |

## Purpose

Validate all pure-logic helper functions in `src/commands/inspect.rs`:
- Object type mapping (TableKind code → display name)
- Size formatting (bytes → human-readable KB/MB/GB/TB)
- Skew calculation formula and edge cases
- Qualified name parsing
- SQL query construction for each section
- Error handling when object not found
- Graceful degradation when a section fails

## Acceptance Criteria Coverage

- **AC-1**: Object type shown correctly (TableKind mapping)
- **AC-2**: Columns section SQL uses correct DBC.ColumnsV predicates
- **AC-3**: Indexes section SQL uses correct DBC.IndicesV predicates
- **AC-5**: Size/skew calculated and formatted correctly
- **AC-7**: Qualified name `db.obj` parsed correctly
- **AC-10**: Helpful error on non-existent object; graceful degradation on section failure

## Prerequisites

- Rust test framework available
- `src/commands/inspect.rs` module created with helper functions

## Test Procedure

### Test Implementation (in `src/commands/inspect.rs` `#[cfg(test)]` module):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // TC-045-008: TableKind code → display name mapping
    // -------------------------------------------------------------------------

    #[test]
    fn test_table_kind_table() {
        assert_eq!(table_kind_display("T"), "Table");
    }

    #[test]
    fn test_table_kind_view() {
        assert_eq!(table_kind_display("V"), "View");
    }

    #[test]
    fn test_table_kind_macro() {
        assert_eq!(table_kind_display("M"), "Macro");
    }

    #[test]
    fn test_table_kind_join_index() {
        // "J" or "I" depending on Teradata version; implementation may map several codes
        let name = table_kind_display("J");
        assert!(!name.is_empty(), "Should return non-empty string for join index code");
    }

    #[test]
    fn test_table_kind_unknown() {
        let name = table_kind_display("Z");
        // Must not panic; should return a fallback like "Unknown (Z)"
        assert!(name.contains('Z') || name.to_lowercase().contains("unknown"),
            "Unknown code should produce fallback string, got: {}", name);
    }

    // -------------------------------------------------------------------------
    // TC-045-011: Size formatting
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_size_zero_bytes() {
        assert_eq!(format_size_bytes(0), "0 B");
    }

    #[test]
    fn test_format_size_bytes_under_1kb() {
        let s = format_size_bytes(500);
        assert!(s.contains("500") && s.contains('B'),
            "500 bytes should show as '500 B', got: {}", s);
    }

    #[test]
    fn test_format_size_exactly_1kb() {
        let s = format_size_bytes(1024);
        assert!(s.contains("1") && s.to_uppercase().contains("KB"),
            "1024 bytes should show as 1.0 KB, got: {}", s);
    }

    #[test]
    fn test_format_size_megabytes() {
        let s = format_size_bytes(1_048_576);
        assert!(s.to_uppercase().contains("MB"),
            "1 MiB should display with MB, got: {}", s);
    }

    #[test]
    fn test_format_size_gigabytes() {
        let s = format_size_bytes(1_073_741_824);
        assert!(s.to_uppercase().contains("GB"),
            "1 GiB should display with GB, got: {}", s);
    }

    #[test]
    fn test_format_size_terabytes() {
        let s = format_size_bytes(1_099_511_627_776u64);
        assert!(s.to_uppercase().contains("TB"),
            "1 TiB should display with TB, got: {}", s);
    }

    // -------------------------------------------------------------------------
    // TC-045-011b: Skew calculation
    // -------------------------------------------------------------------------

    #[test]
    fn test_skew_balanced_zero_percent() {
        // All AMPs have equal size → skew is 0%
        let amp_sizes: Vec<f64> = vec![100.0, 100.0, 100.0, 100.0];
        let skew = calculate_skew(&amp_sizes);
        assert!(skew.abs() < 0.01, "Balanced AMPs should give ~0% skew, got: {}", skew);
    }

    #[test]
    fn test_skew_one_hot_amp() {
        // One AMP has all the data, rest have nothing → high skew
        let amp_sizes: Vec<f64> = vec![1000.0, 0.0, 0.0, 0.0];
        let skew = calculate_skew(&amp_sizes);
        assert!(skew > 50.0, "One-hot AMP should give >50% skew, got: {}", skew);
    }

    #[test]
    fn test_skew_no_data_no_panic() {
        // All AMPs have 0 bytes → avg is 0 → must not divide by zero
        let amp_sizes: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0];
        let skew = calculate_skew(&amp_sizes);
        // Should return 0.0 or a safe sentinel, not panic
        assert!(skew.is_finite(), "Skew must be finite even with all-zero AMP sizes");
    }

    #[test]
    fn test_skew_empty_amp_list_no_panic() {
        // Edge: no AMPs reported
        let amp_sizes: Vec<f64> = vec![];
        let skew = calculate_skew(&amp_sizes);
        assert!(skew.is_finite(), "Skew must be finite with empty AMP list");
    }

    // -------------------------------------------------------------------------
    // TC-045-012: Qualified name parsing
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_qualified_name_with_dot() {
        let (db, obj) = parse_object_name("mydb.mytable", "sessiondb");
        assert_eq!(db, "mydb");
        assert_eq!(obj, "mytable");
    }

    #[test]
    fn test_parse_unqualified_name_uses_session_db() {
        let (db, obj) = parse_object_name("mytable", "sessiondb");
        assert_eq!(db, "sessiondb");
        assert_eq!(obj, "mytable");
    }

    #[test]
    fn test_parse_name_preserves_case() {
        let (db, obj) = parse_object_name("MyDB.MyTable", "session");
        assert_eq!(db, "MyDB");
        assert_eq!(obj, "MyTable");
    }

    // -------------------------------------------------------------------------
    // TC-045-009 & TC-045-010: SQL construction validation
    // -------------------------------------------------------------------------

    #[test]
    fn test_columns_sql_references_dbc_columnsv() {
        let sql = build_columns_sql("mydb", "mytable");
        assert!(sql.to_uppercase().contains("DBC.COLUMNSV") ||
                sql.to_uppercase().contains("DBC.COLUMNS"),
            "Columns SQL must query DBC.ColumnsV, got: {}", sql);
        assert!(sql.contains("mydb") || sql.to_uppercase().contains("DATABASENAME"),
            "Columns SQL must filter by database, got: {}", sql);
        assert!(sql.contains("mytable") || sql.to_uppercase().contains("TABLENAME"),
            "Columns SQL must filter by table, got: {}", sql);
    }

    #[test]
    fn test_indexes_sql_references_dbc_indicesv() {
        let sql = build_indexes_sql("mydb", "mytable");
        assert!(sql.to_uppercase().contains("DBC.INDICESV") ||
                sql.to_uppercase().contains("DBC.INDICES"),
            "Indexes SQL must query DBC.IndicesV, got: {}", sql);
    }

    #[test]
    fn test_size_sql_references_dbc_tablesizev() {
        let sql = build_size_sql("mydb", "mytable");
        assert!(sql.to_uppercase().contains("DBC.TABLESIZEV") ||
                sql.to_uppercase().contains("DBC.TABLESIZE"),
            "Size SQL must query DBC.TableSizeV, got: {}", sql);
    }

    // -------------------------------------------------------------------------
    // TC-045-013: Error handling
    // -------------------------------------------------------------------------

    #[test]
    fn test_not_found_error_contains_object_name() {
        let err_msg = build_not_found_error_message("mydb", "nonexistent_table");
        assert!(err_msg.contains("nonexistent_table"),
            "Error message must include object name, got: {}", err_msg);
    }
}
```

## Expected Results

All unit tests pass without database access:
- TableKind mapping: T→Table, V→View, M→Macro, unknown→fallback without panic
- Size formatting: 0B, 500B, 1KB, 1MB, 1GB, 1TB all display correct units
- Skew: balanced=0%, one-hot>50%, zero data=0.0 (no panic), empty list=finite (no panic)
- Name parsing: `db.obj` splits correctly, unqualified uses session default, case preserved
- SQL construction: each builder references the correct DBC view
- Error message: object name appears in not-found error

## Pass/Fail Criteria

**PASS if:**
- All unit tests compile and pass with `cargo test --lib`
- No panics (especially for zero-division in skew)
- All DBC view names appear in generated SQL

**FAIL if:**
- Any test fails
- Skew calculation panics on zero average
- Size formatting omits units or panics on large values
- SQL builder omits the DBC view name

## Run Command

```bash
cargo test --lib -- inspect::tests 2>&1
```

## Notes

- This is a UNIT test suite — no database or PTY required
- Function names (`table_kind_display`, `format_size_bytes`, `calculate_skew`, `parse_object_name`, `build_columns_sql`, `build_indexes_sql`, `build_size_sql`, `build_not_found_error_message`) are the expected public or pub(crate) names; adjust if implementation uses different names
- If function signatures differ, update test assertions accordingly while preserving intent
