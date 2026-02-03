# TC-033-003: Unit Tests - /peek Command

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-033-003 |
| **Title** | Unit Tests - /peek Command |
| **Category** | Unit Test |
| **Priority** | Critical |
| **Feature** | Sprint 33 - Data Sampling Commands (AC-5, AC-6) |
| **Test Type** | Unit |
| **Created** | 2026-02-03 |

## Purpose

Verify that the `/peek` command correctly parses table names and generates SQL to retrieve first 5 rows plus column metadata.

## Acceptance Criteria Coverage

- **AC-5**: `/peek` command implemented - Shows first 5 rows + column metadata
- **AC-6**: Column info display - Show data types, nullable, precision for `/peek`
- **AC-12**: Qualified names - Support database.tablename syntax

## Scope

This test validates:
- Command parsing for `/peek <table>`
- SQL generation for TOP 5 query
- Metadata query generation for column information
- Qualified table names (database.table) are handled
- Error messages for invalid input

## Prerequisites

- Rust test framework available
- Peek command implementation exists

## Test Procedure

### Test Implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peek_command_parsing() {
        // Parse: /peek employees
        let result = parse_peek_command("employees");

        assert!(result.is_ok());
        let table = result.unwrap();
        assert_eq!(table, "employees");
    }

    #[test]
    fn test_peek_command_qualified_name() {
        // Parse: /peek mydb.employees
        let result = parse_peek_command("mydb.employees");

        assert!(result.is_ok());
        let table = result.unwrap();
        assert_eq!(table, "mydb.employees");
    }

    #[test]
    fn test_peek_sql_generation() {
        // Generate SQL for: /peek employees
        let sql = generate_peek_sql("employees");

        assert!(sql.contains("TOP"), "SQL must use TOP clause");
        assert!(sql.contains("5"), "SQL must limit to 5 rows");
        assert!(sql.contains("employees"), "SQL must reference table");

        // Expected format: SELECT TOP 5 * FROM employees
        assert_eq!(sql, "SELECT TOP 5 * FROM employees");
    }

    #[test]
    fn test_peek_sql_qualified_name() {
        // Generate SQL for: /peek mydb.employees
        let sql = generate_peek_sql("mydb.employees");

        assert_eq!(sql, "SELECT TOP 5 * FROM mydb.employees");
    }

    #[test]
    fn test_peek_metadata_query_generation() {
        // Generate metadata query for: /peek employees
        // Should query DBC.ColumnsV or similar
        let metadata_sql = generate_peek_metadata_sql("employees", None);

        assert!(metadata_sql.contains("DBC."), "Should query data dictionary");
        assert!(metadata_sql.contains("ColumnName") || metadata_sql.contains("COLUMNNAME"),
                "Should retrieve column names");
        assert!(metadata_sql.contains("ColumnType") || metadata_sql.contains("COLUMNTYPE"),
                "Should retrieve data types");
        assert!(metadata_sql.contains("Nullable") || metadata_sql.contains("NULLABLE"),
                "Should retrieve nullable info");
    }

    #[test]
    fn test_peek_metadata_query_qualified_name() {
        // Generate metadata query for: /peek mydb.employees
        // Should split database.table for metadata lookup
        let metadata_sql = generate_peek_metadata_sql("mydb.employees", Some("mydb"));

        assert!(metadata_sql.contains("mydb"), "Should use specified database");
        assert!(metadata_sql.contains("employees"), "Should use table name");
    }

    #[test]
    fn test_peek_empty_table_name() {
        // Parse: /peek (no table)
        let result = parse_peek_command("");

        assert!(result.is_err(), "Should reject empty table name");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("table"), "Error should mention table name required");
    }
}
```

## Expected Results

All unit tests pass:
- Table name parsing works for simple and qualified names
- SQL generation uses TOP 5 clause
- Metadata query targets data dictionary (DBC.ColumnsV or similar)
- Qualified names are handled correctly
- Clear error for missing table name

## Pass/Fail Criteria

**PASS if:**
- All 7 unit tests compile and pass
- /peek generates SELECT TOP 5 query
- Metadata query retrieves column info (name, type, nullable)
- Qualified names work correctly
- Error messages are clear

**FAIL if:**
- Any unit test fails
- SQL doesn't use TOP 5
- Metadata query is incorrect
- Qualified names are broken
- Error messages are unclear

## Notes

- This is a UNIT test - no database or PTY required
- Tests core parsing and SQL generation logic
- Companion tests: TC-033-006 (integration), TC-033-007 (interactive), TC-033-008 (batch)
- Validates AC-5, AC-6, AC-12 from Sprint 33
- /peek is complementary to /sample - shows first rows vs random rows
