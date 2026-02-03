# TC-033-008: Batch Mode Tests - tq sample Command

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-033-008 |
| **Title** | Batch Mode Tests - tq sample Command |
| **Category** | Integration Test |
| **Priority** | Critical |
| **Feature** | Sprint 33 - Data Sampling Commands (AC-11) |
| **Test Type** | Integration (#[ignore] - requires live database) |
| **Created** | 2026-02-03 |

## Purpose

Verify that the `tq sample` command works correctly in batch mode CLI execution, including argument parsing and output format support.

## Acceptance Criteria Coverage

- **AC-11**: Batch mode integration - `tq sample <table>` and `tq peek <table>` commands

## Scope

This test validates:
- Batch mode CLI: `tq sample <table>` works
- Batch mode with count: `tq sample <table> <count>` works
- Output format flags work (--format table/csv/json)
- Exit codes are correct (0 for success, non-zero for error)
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
fn test_batch_sample_basic() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq sample dbc.databases
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "sample", "dbc.databases"])
        .output()
        .expect("Failed to execute command");

    // Verify: Success exit code
    assert!(output.status.success(), "Command should succeed");

    // Verify: Output contains table data
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DatabaseName") || stdout.contains("DATABASENAME"),
            "Should show column headers");
}

#[test]
#[ignore] // Requires live database
fn test_batch_sample_with_count() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq sample dbc.databases 5
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "sample", "dbc.databases", "5"])
        .output()
        .expect("Failed to execute command");

    // Verify: Success exit code
    assert!(output.status.success(), "Command should succeed");

    // Verify: Output contains data
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.len() > 0, "Should produce output");
}

#[test]
#[ignore] // Requires live database
fn test_batch_sample_json_format() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq sample dbc.databases 3 --format json
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "sample", "dbc.databases", "3", "--format", "json"])
        .output()
        .expect("Failed to execute command");

    // Verify: Success exit code
    assert!(output.status.success(), "Command should succeed");

    // Verify: Output is valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "Output should be valid JSON");
}

#[test]
#[ignore] // Requires live database
fn test_batch_sample_csv_format() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq sample dbc.databases 3 --format csv
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "sample", "dbc.databases", "3", "--format", "csv"])
        .output()
        .expect("Failed to execute command");

    // Verify: Success exit code
    assert!(output.status.success(), "Command should succeed");

    // Verify: Output is CSV format
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.lines().count() > 0, "Should have CSV lines");
    assert!(stdout.contains(',') || stdout.contains(';'),
            "CSV should have delimiters");
}

#[test]
#[ignore] // Requires live database
fn test_batch_sample_invalid_table() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq sample nonexistent_table
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "sample", "nonexistent_table"])
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
fn test_batch_sample_qualified_name() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq sample dbc.tables 5
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "sample", "dbc.tables", "5"])
        .output()
        .expect("Failed to execute command");

    // Verify: Success exit code
    assert!(output.status.success(), "Qualified name should work");

    // Verify: Output contains data
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.len() > 0, "Should produce output");
}

#[test]
#[ignore] // Requires live database
fn test_batch_sample_max_count_validation() {
    // Setup
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON required");

    // Execute: tq sample dbc.databases 1001
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "sample", "dbc.databases", "1001"])
        .output()
        .expect("Failed to execute command");

    // Verify: Error exit code (count > 1000)
    assert!(!output.status.success(), "Count > 1000 should fail");

    // Verify: Error message about max limit
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("1000") || stderr.contains("max"),
            "Should mention max limit: {}", stderr);
}
```

## Expected Results

All batch mode tests pass:
- `tq sample <table>` command works
- `tq sample <table> <count>` works with explicit count
- Output formats (JSON, CSV) work with --format flag
- Invalid tables produce error exit codes
- Error messages appear on stderr
- Qualified table names work
- Count validation rejects > 1000

## Pass/Fail Criteria

**PASS if:**
- All 8 batch mode tests pass
- CLI argument parsing works correctly
- All output formats work
- Exit codes are correct (0 success, non-zero error)
- Error messages are clear on stderr
- Qualified names work

**FAIL if:**
- Any batch mode test fails
- CLI arguments are not parsed
- Output formats don't work
- Exit codes are incorrect
- Error messages are missing
- Qualified names are broken

## Notes

- These are BATCH MODE tests - require live database
- Marked with #[ignore] attribute
- Run with: `cargo test --test integration_tests test_batch_sample -- --ignored`
- Uses std::process::Command for subprocess execution
- Companion tests: TC-033-002 (unit), TC-033-004 (integration), TC-033-006 (interactive)
- Validates AC-11 from Sprint 33
- Tests CLI interface, not REPL
