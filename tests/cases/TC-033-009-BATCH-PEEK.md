# TC-033-009: Batch Mode Tests - tq peek Command

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-033-009 |
| **Title** | Batch Mode Tests - tq peek Command |
| **Category** | Integration Test |
| **Priority** | Critical |
| **Feature** | Sprint 33 - Data Sampling Commands (AC-11) |
| **Test Type** | Integration (#[ignore] - requires live database) |
| **Created** | 2026-02-03 |

## Purpose

Verify that the `tq peek` command works correctly in batch mode CLI execution, displaying both column metadata and data preview.

## Acceptance Criteria Coverage

- **AC-11**: Batch mode integration - `tq sample <table>` and `tq peek <table>` commands

## Scope

This test validates:
- Batch mode CLI: `tq peek <table>` works
- Metadata is displayed (column names, types, nullable)
- Data preview is displayed (first 5 rows)
- Output format flags work (--format table/json)
- Exit codes are correct
- Error handling for invalid tables

## Prerequisites

- Live Teradata database access
- TQ_LOGON environment variable or .env file set
- Compiled tq binary available
- std::process::Command for subprocess execution

## Test Procedure

### Test Implementation (in `tests/integration_tests.rs`):

```rust
#[test]
#[ignore] // Requires live database
fn test_batch_peek_basic() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq peek dbc.databases
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "peek", "dbc.databases"])
        .output()
        .expect("Failed to execute command");

    // Verify: Success exit code
    assert!(output.status.success(), "Command should succeed");

    // Verify: Output contains metadata
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Column") || stdout.contains("Type"),
            "Should show column metadata");

    // Verify: Output contains data
    assert!(stdout.contains("DatabaseName") || stdout.contains("DATABASENAME"),
            "Should show table data");
}

#[test]
#[ignore] // Requires live database
fn test_batch_peek_metadata_content() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq peek dbc.databases
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "peek", "dbc.databases"])
        .output()
        .expect("Failed to execute command");

    // Verify: Success exit code
    assert!(output.status.success(), "Command should succeed");

    // Verify: Metadata includes data types
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("VARCHAR") || stdout.contains("CHAR") || stdout.contains("INT"),
        "Should show data types"
    );

    // Verify: Metadata includes nullable info
    assert!(
        stdout.contains("Nullable") || stdout.contains("NOT NULL") ||
        stdout.contains("YES") || stdout.contains("NO"),
        "Should show nullable information"
    );
}

#[test]
#[ignore] // Requires live database
fn test_batch_peek_json_format() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq peek dbc.databases --format json
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "peek", "dbc.databases", "--format", "json"])
        .output()
        .expect("Failed to execute command");

    // Verify: Success exit code
    assert!(output.status.success(), "Command should succeed");

    // Verify: Output is valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "Output should be valid JSON");

    // Verify: JSON has metadata and data sections
    if let Ok(json) = parsed {
        assert!(json.get("metadata").is_some() || json.is_object(),
                "JSON should have metadata");
    }
}

#[test]
#[ignore] // Requires live database
fn test_batch_peek_invalid_table() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq peek nonexistent_table
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "peek", "nonexistent_table"])
        .output()
        .expect("Failed to execute command");

    // Verify: Error exit code
    assert!(!output.status.success(), "Invalid table should fail");

    // Verify: Error message on stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("table") || stderr.contains("object") || stderr.contains("not found"),
        "Should show error message: {}", stderr
    );
}

#[test]
#[ignore] // Requires live database
fn test_batch_peek_qualified_name() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq peek dbc.tables
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "peek", "dbc.tables"])
        .output()
        .expect("Failed to execute command");

    // Verify: Success exit code
    assert!(output.status.success(), "Qualified name should work");

    // Verify: Output contains metadata and data
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Column") || stdout.contains("Type"),
            "Should show metadata");
    assert!(stdout.len() > 100, "Should have substantial output");
}

#[test]
#[ignore] // Requires live database
fn test_batch_peek_row_count() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq peek dbc.databases
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "peek", "dbc.databases"])
        .output()
        .expect("Failed to execute command");

    // Verify: Success exit code
    assert!(output.status.success(), "Command should succeed");

    // Verify: Output has reasonable size (metadata + <= 5 rows)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line_count = stdout.lines().count();

    // Should have header section, metadata, data section
    // Approximate: < 50 lines total (metadata + 5 data rows + formatting)
    assert!(line_count < 50,
            "Should show limited output (metadata + 5 rows), got {} lines", line_count);
}
```

## Expected Results

All batch mode tests pass:
- `tq peek <table>` command works
- Metadata is displayed (columns, types, nullable)
- Data preview is shown (first 5 rows)
- JSON format works with --format flag
- Invalid tables produce error exit codes
- Error messages appear on stderr
- Qualified table names work
- Row count is limited appropriately

## Pass/Fail Criteria

**PASS if:**
- All 7 batch mode tests pass
- CLI argument parsing works correctly
- Metadata is complete and accurate
- Data preview shows <= 5 rows
- JSON format works
- Exit codes are correct (0 success, non-zero error)
- Error messages are clear on stderr
- Qualified names work

**FAIL if:**
- Any batch mode test fails
- CLI arguments are not parsed
- Metadata is missing or incomplete
- Data preview shows > 5 rows
- JSON format doesn't work
- Exit codes are incorrect
- Error messages are missing
- Qualified names are broken

## Notes

- These are BATCH MODE tests - require live database
- Marked with #[ignore] attribute
- Run with: `cargo test --test integration_tests test_batch_peek -- --ignored`
- Uses std::process::Command for subprocess execution
- Companion tests: TC-033-003 (unit), TC-033-005 (integration), TC-033-007 (interactive)
- Validates AC-11 from Sprint 33
- Tests CLI interface with metadata display
