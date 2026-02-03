# TC-033-004: Integration Tests - /sample Command

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-033-004 |
| **Title** | Integration Tests - /sample Command |
| **Category** | Integration Test |
| **Priority** | Critical |
| **Feature** | Sprint 33 - Data Sampling Commands (AC-4, AC-8, AC-9, AC-13) |
| **Test Type** | Integration (#[ignore] - requires live database) |
| **Created** | 2026-02-03 |

## Purpose

Verify that the `/sample` command executes correctly against a live Teradata database, returns proper results, handles errors gracefully, and respects output formats.

## Acceptance Criteria Coverage

- **AC-4**: Random sampling - Use Teradata SAMPLE clause for true random sampling
- **AC-8**: Error handling - Clear messages for invalid tables, permissions, syntax
- **AC-9**: Multi-format support - Respect current output format (table/csv/json)
- **AC-13**: Performance - Fast execution even on large tables (SAMPLE is efficient)

## Scope

This test validates:
- Actual query execution against Teradata
- SAMPLE clause works with Teradata database
- Error handling for invalid tables
- Output format support (table, CSV, JSON)
- Performance is acceptable

## Prerequisites

- Live Teradata database access
- TQ_LOGON environment variable or .env file set
- Test database with accessible tables (e.g., dbc.databases)

## Test Procedure

### Test Implementation (in `tests/integration_tests.rs`):

```rust
#[test]
#[ignore] // Requires live database
fn test_sample_command_default_count() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute: /sample dbc.databases (default 10 rows)
    let result = execute_sample_command(&client, "dbc.databases", 10);

    assert!(result.is_ok(), "Sample query should succeed");
    let query_result = result.unwrap();

    // Verify: Should return <= 10 rows
    assert!(query_result.rows.len() <= 10, "Should return at most 10 rows");
    assert!(query_result.rows.len() > 0, "Should return at least 1 row");

    // Verify: Columns present (DatabaseName, etc.)
    assert!(query_result.columns.len() > 0, "Should have columns");
}

#[test]
#[ignore] // Requires live database
fn test_sample_command_explicit_count() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute: /sample dbc.databases 5
    let result = execute_sample_command(&client, "dbc.databases", 5);

    assert!(result.is_ok(), "Sample query should succeed");
    let query_result = result.unwrap();

    // Verify: Should return <= 5 rows
    assert!(query_result.rows.len() <= 5, "Should return at most 5 rows");
}

#[test]
#[ignore] // Requires live database
fn test_sample_command_invalid_table() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute: /sample nonexistent_table_xyz
    let result = execute_sample_command(&client, "nonexistent_table_xyz", 10);

    // Verify: Should fail with clear error
    assert!(result.is_err(), "Invalid table should produce error");
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("table") || error_msg.contains("object") || error_msg.contains("does not exist"),
        "Error should indicate table not found: {}", error_msg
    );
}

#[test]
#[ignore] // Requires live database
fn test_sample_command_qualified_name() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute: /sample dbc.tables (qualified name)
    let result = execute_sample_command(&client, "dbc.tables", 10);

    assert!(result.is_ok(), "Qualified table name should work");
    let query_result = result.unwrap();
    assert!(query_result.rows.len() <= 10, "Should return at most 10 rows");
}

#[test]
#[ignore] // Requires live database
fn test_sample_output_format_table() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute sample and format as table
    let result = execute_sample_command(&client, "dbc.databases", 5);
    assert!(result.is_ok());

    let query_result = result.unwrap();
    let output = format_as_table(&query_result);

    // Verify: Output contains table borders and data
    assert!(output.contains("─"), "Table output should have borders");
    assert!(output.contains("│"), "Table output should have column separators");
}

#[test]
#[ignore] // Requires live database
fn test_sample_output_format_json() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute sample and format as JSON
    let result = execute_sample_command(&client, "dbc.databases", 5);
    assert!(result.is_ok());

    let query_result = result.unwrap();
    let output = format_as_json(&query_result);

    // Verify: Valid JSON array
    let parsed: serde_json::Value = serde_json::from_str(&output)
        .expect("Output should be valid JSON");
    assert!(parsed.is_array(), "JSON output should be array");
    assert!(parsed.as_array().unwrap().len() <= 5, "Array should have <= 5 items");
}

#[test]
#[ignore] // Requires live database
fn test_sample_output_format_csv() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute sample and format as CSV
    let result = execute_sample_command(&client, "dbc.databases", 5);
    assert!(result.is_ok());

    let query_result = result.unwrap();
    let output = format_as_csv(&query_result);

    // Verify: CSV format (headers + rows)
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines.len() > 0, "CSV should have headers");
    assert!(lines.len() <= 6, "CSV should have header + <= 5 data rows");
}

#[test]
#[ignore] // Requires live database
fn test_sample_performance() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute: /sample on large table
    let start = std::time::Instant::now();
    let result = execute_sample_command(&client, "dbc.tables", 100);
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Sample query should succeed");

    // Verify: Completes in reasonable time (< 5 seconds for sample)
    assert!(
        elapsed.as_secs() < 5,
        "Sample should be fast (< 5s), took {:?}", elapsed
    );
}
```

## Expected Results

All integration tests pass:
- Sample query executes successfully against Teradata
- Returns correct number of rows (<= requested count)
- Invalid tables produce clear error messages
- Qualified names work correctly
- All output formats (table, CSV, JSON) work
- Performance is fast (< 5 seconds even on large tables)

## Pass/Fail Criteria

**PASS if:**
- All 9 integration tests pass
- SAMPLE clause executes successfully
- Row count validation works
- Error messages are clear and actionable
- All output formats work correctly
- Performance meets expectations

**FAIL if:**
- Any integration test fails
- SAMPLE clause doesn't work
- Row count is incorrect
- Error messages are unclear
- Output formats are broken
- Performance is slow (> 5 seconds)

## Notes

- These are INTEGRATION tests - require live Teradata database
- Marked with #[ignore] attribute
- Run with: `cargo test --test integration_tests test_sample -- --ignored`
- Uses system tables (dbc.databases, dbc.tables) for testing
- Companion tests: TC-033-002 (unit), TC-033-005 (interactive), TC-033-006 (batch)
- Validates AC-4, AC-8, AC-9, AC-13 from Sprint 33
