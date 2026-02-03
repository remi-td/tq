# TC-033-002: Unit Tests - /sample Command

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-033-002 |
| **Title** | Unit Tests - /sample Command |
| **Category** | Unit Test |
| **Priority** | Critical |
| **Feature** | Sprint 33 - Data Sampling Commands (AC-1 through AC-4) |
| **Test Type** | Unit |
| **Created** | 2026-02-03 |

## Purpose

Verify that the `/sample` command correctly parses arguments, generates proper SQL with Teradata SAMPLE clause, and validates parameters.

## Acceptance Criteria Coverage

- **AC-1**: `/sample` command implemented - Accepts table name, optional row count
- **AC-2**: Default sample size - 10 rows if count not specified
- **AC-3**: Sample size validation - Max 1000 rows (prevent accidental large queries)
- **AC-4**: Random sampling - Use Teradata SAMPLE clause for true random sampling

## Scope

This test validates:
- Command parsing for `/sample <table>` and `/sample <table> <count>`
- Default row count of 10 when not specified
- Validation rejects counts > 1000
- SQL generation includes Teradata SAMPLE clause
- Qualified table names (database.table) are handled
- Error messages for invalid input

## Prerequisites

- Rust test framework available
- Sample command implementation exists

## Test Procedure

### Test Implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_command_default_count() {
        // Parse: /sample employees
        let result = parse_sample_command("employees", None);

        assert!(result.is_ok());
        let (table, count) = result.unwrap();
        assert_eq!(table, "employees");
        assert_eq!(count, 10, "Default sample size should be 10");
    }

    #[test]
    fn test_sample_command_explicit_count() {
        // Parse: /sample employees 50
        let result = parse_sample_command("employees", Some("50"));

        assert!(result.is_ok());
        let (table, count) = result.unwrap();
        assert_eq!(table, "employees");
        assert_eq!(count, 50);
    }

    #[test]
    fn test_sample_command_max_validation() {
        // Parse: /sample employees 1001 (exceeds max)
        let result = parse_sample_command("employees", Some("1001"));

        assert!(result.is_err(), "Should reject count > 1000");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("1000"), "Error should mention max limit");
    }

    #[test]
    fn test_sample_command_qualified_name() {
        // Parse: /sample mydb.employees
        let result = parse_sample_command("mydb.employees", None);

        assert!(result.is_ok());
        let (table, count) = result.unwrap();
        assert_eq!(table, "mydb.employees");
        assert_eq!(count, 10);
    }

    #[test]
    fn test_sample_sql_generation() {
        // Generate SQL for: /sample employees 20
        let sql = generate_sample_sql("employees", 20);

        assert!(sql.contains("SAMPLE"), "SQL must use Teradata SAMPLE clause");
        assert!(sql.contains("employees"), "SQL must reference table");
        assert!(sql.contains("20"), "SQL must include sample count");

        // Expected format: SELECT * FROM employees SAMPLE 20
        assert_eq!(sql, "SELECT * FROM employees SAMPLE 20");
    }

    #[test]
    fn test_sample_sql_qualified_name() {
        // Generate SQL for: /sample mydb.employees 10
        let sql = generate_sample_sql("mydb.employees", 10);

        assert_eq!(sql, "SELECT * FROM mydb.employees SAMPLE 10");
    }

    #[test]
    fn test_sample_invalid_count_format() {
        // Parse: /sample employees abc (non-numeric)
        let result = parse_sample_command("employees", Some("abc"));

        assert!(result.is_err(), "Should reject non-numeric count");
    }

    #[test]
    fn test_sample_zero_count() {
        // Parse: /sample employees 0
        let result = parse_sample_command("employees", Some("0"));

        assert!(result.is_err(), "Should reject zero count");
    }

    #[test]
    fn test_sample_negative_count() {
        // Parse: /sample employees -5
        let result = parse_sample_command("employees", Some("-5"));

        assert!(result.is_err(), "Should reject negative count");
    }
}
```

## Expected Results

All unit tests pass:
- Default sample count is 10
- Explicit counts are parsed correctly
- Validation rejects count > 1000, zero, negative, non-numeric
- SQL uses Teradata SAMPLE clause
- Qualified table names are handled
- Clear error messages for invalid input

## Pass/Fail Criteria

**PASS if:**
- All 9 unit tests compile and pass
- Default count is 10
- Max validation at 1000 rows works
- SQL generation uses SAMPLE clause correctly
- Qualified names are preserved
- Error messages are clear and actionable

**FAIL if:**
- Any unit test fails
- Default count is not 10
- Validation doesn't reject > 1000
- SQL doesn't use SAMPLE clause
- Qualified names are broken
- Error messages are unclear

## Notes

- This is a UNIT test - no database or PTY required
- Tests core parsing and SQL generation logic
- Companion tests: TC-033-003 (integration), TC-033-004 (interactive), TC-033-005 (batch)
- Validates AC-1 through AC-4 from Sprint 33
