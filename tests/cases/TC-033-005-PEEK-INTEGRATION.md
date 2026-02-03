# TC-033-005: Integration Tests - /peek Command

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-033-005 |
| **Title** | Integration Tests - /peek Command |
| **Category** | Integration Test |
| **Priority** | Critical |
| **Feature** | Sprint 33 - Data Sampling Commands (AC-5, AC-6, AC-8, AC-9) |
| **Test Type** | Integration (#[ignore] - requires live database) |
| **Created** | 2026-02-03 |

## Purpose

Verify that the `/peek` command executes correctly against a live Teradata database, retrieves column metadata, returns first 5 rows, and handles errors gracefully.

## Acceptance Criteria Coverage

- **AC-5**: `/peek` command implemented - Shows first 5 rows + column metadata
- **AC-6**: Column info display - Show data types, nullable, precision for `/peek`
- **AC-8**: Error handling - Clear messages for invalid tables, permissions, syntax
- **AC-9**: Multi-format support - Respect current output format (table/csv/json)

## Scope

This test validates:
- Actual query execution against Teradata
- TOP 5 clause works correctly
- Column metadata retrieval from data dictionary
- Error handling for invalid tables
- Output format support (table, CSV, JSON)

## Prerequisites

- Live Teradata database access
- TQ_LOGON environment variable or .env file set
- Test database with accessible tables (e.g., dbc.databases)

## Test Procedure

### Test Implementation (in `tests/integration_tests.rs`):

```rust
#[test]
#[ignore] // Requires live database
fn test_peek_command_basic() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute: /peek dbc.databases
    let result = execute_peek_command(&client, "dbc.databases");

    assert!(result.is_ok(), "Peek query should succeed");
    let (metadata, data) = result.unwrap();

    // Verify: Column metadata retrieved
    assert!(metadata.columns.len() > 0, "Should have column metadata");
    assert!(metadata.columns.iter().any(|c| c.data_type.is_some()),
            "Should have data type information");

    // Verify: Returns <= 5 rows
    assert!(data.rows.len() <= 5, "Should return at most 5 rows");
    assert!(data.rows.len() > 0, "Should return at least 1 row");
}

#[test]
#[ignore] // Requires live database
fn test_peek_column_metadata_content() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute: /peek dbc.databases
    let result = execute_peek_command(&client, "dbc.databases");
    assert!(result.is_ok());
    let (metadata, _) = result.unwrap();

    // Verify: Column metadata has required fields
    for column in &metadata.columns {
        assert!(column.name.len() > 0, "Column should have name");
        assert!(column.data_type.is_some(), "Column should have data type");
        assert!(column.nullable.is_some(), "Column should have nullable info");
    }

    // Verify: Expected columns for dbc.databases
    let column_names: Vec<String> = metadata.columns.iter()
        .map(|c| c.name.to_lowercase())
        .collect();
    assert!(column_names.contains(&"databasename".to_string()),
            "dbc.databases should have DatabaseName column");
}

#[test]
#[ignore] // Requires live database
fn test_peek_command_invalid_table() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute: /peek nonexistent_table_xyz
    let result = execute_peek_command(&client, "nonexistent_table_xyz");

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
fn test_peek_command_qualified_name() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute: /peek dbc.tables (qualified name)
    let result = execute_peek_command(&client, "dbc.tables");

    assert!(result.is_ok(), "Qualified table name should work");
    let (metadata, data) = result.unwrap();
    assert!(metadata.columns.len() > 0, "Should have metadata");
    assert!(data.rows.len() <= 5, "Should return at most 5 rows");
}

#[test]
#[ignore] // Requires live database
fn test_peek_output_format_table() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute peek and format as table
    let result = execute_peek_command(&client, "dbc.databases");
    assert!(result.is_ok());

    let (metadata, data) = result.unwrap();
    let output = format_peek_as_table(&metadata, &data);

    // Verify: Output contains metadata section and table
    assert!(output.contains("Column"), "Should show column metadata");
    assert!(output.contains("Type"), "Should show data types");
    assert!(output.contains("─"), "Table output should have borders");
}

#[test]
#[ignore] // Requires live database
fn test_peek_output_format_json() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute peek and format as JSON
    let result = execute_peek_command(&client, "dbc.databases");
    assert!(result.is_ok());

    let (metadata, data) = result.unwrap();
    let output = format_peek_as_json(&metadata, &data);

    // Verify: Valid JSON with metadata and data sections
    let parsed: serde_json::Value = serde_json::from_str(&output)
        .expect("Output should be valid JSON");
    assert!(parsed.get("metadata").is_some(), "JSON should have metadata section");
    assert!(parsed.get("data").is_some(), "JSON should have data section");
}

#[test]
#[ignore] // Requires live database
fn test_peek_data_types_display() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute: /peek dbc.databases
    let result = execute_peek_command(&client, "dbc.databases");
    assert!(result.is_ok());

    let (metadata, _) = result.unwrap();

    // Verify: Data types are meaningful
    for column in &metadata.columns {
        let data_type = column.data_type.as_ref().unwrap();
        assert!(
            data_type.contains("CHAR") ||
            data_type.contains("INT") ||
            data_type.contains("DATE") ||
            data_type.contains("DECIMAL") ||
            data_type.contains("VARCHAR"),
            "Data type should be recognizable: {}", data_type
        );
    }
}

#[test]
#[ignore] // Requires live database
fn test_peek_nullable_display() {
    // Setup
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON required");
    let client = create_test_client(&logon);

    // Execute: /peek dbc.databases
    let result = execute_peek_command(&client, "dbc.databases");
    assert!(result.is_ok());

    let (metadata, _) = result.unwrap();

    // Verify: Nullable info is present
    for column in &metadata.columns {
        assert!(column.nullable.is_some(), "Nullable info should be present");
        // Value should be true or false
        let _nullable = column.nullable.unwrap();
    }
}
```

## Expected Results

All integration tests pass:
- Peek query executes successfully against Teradata
- Column metadata is retrieved from data dictionary
- Metadata includes: column name, data type, nullable, precision
- Returns exactly 5 or fewer rows
- Invalid tables produce clear error messages
- Qualified names work correctly
- Output formats (table, JSON) display both metadata and data

## Pass/Fail Criteria

**PASS if:**
- All 9 integration tests pass
- TOP 5 clause executes successfully
- Column metadata is complete and accurate
- Data types are displayed correctly
- Nullable info is included
- Error messages are clear
- All output formats work correctly

**FAIL if:**
- Any integration test fails
- Metadata is missing or incomplete
- Row count is incorrect (> 5)
- Data types are not displayed
- Nullable info is missing
- Error messages are unclear
- Output formats are broken

## Notes

- These are INTEGRATION tests - require live Teradata database
- Marked with #[ignore] attribute
- Run with: `cargo test --test integration_tests test_peek -- --ignored`
- Uses system tables (dbc.databases, dbc.tables) for testing
- Companion tests: TC-033-003 (unit), TC-033-007 (interactive), TC-033-008 (batch)
- Validates AC-5, AC-6, AC-8, AC-9 from Sprint 33
- /peek combines data preview with metadata inspection
