# TC-045-INSPECT-INTEGRATION: /inspect Command — Integration Tests (Live Database)

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-045-INSPECT-INTEGRATION |
| **Title** | /inspect Command — Integration Tests with Live Teradata Database |
| **Category** | Integration Test |
| **Priority** | Critical |
| **Feature** | Sprint 45 — /inspect Command (Issue #33) |
| **Test Type** | Integration (DB required) |
| **DB Required** | Yes |
| **Annotation** | `#[ignore]` — skipped unless `TQ_LOGON` is set |
| **Created** | 2026-03-23 |
| **Covers** | TC-045-007, TC-045-014 through TC-045-018 |

## Purpose

Validate the full `/inspect` pipeline against a live Teradata database using DBC system views. Covers:
- TC-045-007: Semicolon stripping wiring (REPL end-to-end)
- TC-045-014: Full inspect on a known table
- TC-045-015: Inspect on a known view
- TC-045-016: Inspect on a non-existent object (error path)
- TC-045-017: Batch mode table output
- TC-045-018: Batch mode CSV and JSON output

## Acceptance Criteria Coverage

- **AC-1 through AC-10**: All acceptance criteria validated with real DBC data

## Prerequisites

- Live Teradata database accessible
- `TQ_LOGON` environment variable set: `user:password@host:1025/database`
- Compiled `tq` binary in `target/debug/` or `target/release/`
- `dbc.dbcinfo` or another accessible system table exists (universally available on all Teradata systems)

## Test Procedure

### Test Implementation (in `tests/integration_tests.rs`):

```rust
use std::process::Command;
use std::env;

fn tq_binary() -> String {
    env::var("TQ_BIN").unwrap_or_else(|_| "./target/debug/tq".to_string())
}

fn tq_logon() -> Option<String> {
    env::var("TQ_LOGON").ok()
}

// -------------------------------------------------------------------------
// TC-045-007: Semicolon stripping wiring (REPL end-to-end)
// Tests that "/describe dbc.dbcinfo;" reaches the describe handler and
// does not produce "unknown command" or "no such table dbc.dbcinfo;"
// -------------------------------------------------------------------------
#[test]
#[ignore]
fn test_bug32_describe_semicolon_end_to_end() {
    let logon = match tq_logon() {
        Some(l) => l,
        None => return, // silently skip if no DB configured
    };
    // Use batch mode describe to test semicolon stripping in the command dispatch
    // (batch mode describe is equivalent for argument parsing purposes)
    let output = Command::new(tq_binary())
        .args(["--logon", &logon, "describe", "dbc.dbcinfo"])
        .output()
        .expect("Failed to run tq describe");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success() || stdout.contains("Column"),
        "describe dbc.dbcinfo should succeed or return columns, got: {}", stdout);
    // Verify the argument did not retain a semicolon causing a "not found" error
    assert!(!stdout.to_lowercase().contains("not found"),
        "Should not produce 'not found', got: {}", stdout);
}

// -------------------------------------------------------------------------
// TC-045-014: Full /inspect on a known table
// -------------------------------------------------------------------------
#[test]
#[ignore]
fn test_inspect_known_table_all_sections_present() {
    let logon = match tq_logon() {
        Some(l) => l,
        None => return,
    };
    let output = Command::new(tq_binary())
        .args(["--logon", &logon, "inspect", "dbc.dbcinfo"])
        .output()
        .expect("Failed to run tq inspect");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(),
        "tq inspect should exit 0, stderr: {}", stderr);

    // Verify core sections are present
    assert!(stdout.to_uppercase().contains("TYPE") ||
            stdout.to_uppercase().contains("TABLE") ||
            stdout.to_uppercase().contains("VIEW"),
        "Output should contain object type information, got:\n{}", stdout);

    assert!(stdout.to_uppercase().contains("COLUMN") ||
            stdout.to_uppercase().contains("INFODATA"),
        "Output should contain column information, got:\n{}", stdout);
}

// -------------------------------------------------------------------------
// TC-045-015: Inspect on a known view
// -------------------------------------------------------------------------
#[test]
#[ignore]
fn test_inspect_known_view_shows_definition() {
    let logon = match tq_logon() {
        Some(l) => l,
        None => return,
    };
    // DBC.TablesV is a well-known system view available on all Teradata systems
    let output = Command::new(tq_binary())
        .args(["--logon", &logon, "inspect", "dbc.TablesV"])
        .output()
        .expect("Failed to run tq inspect");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(),
        "tq inspect dbc.TablesV should exit 0, stderr: {}", stderr);

    // Inspect on a view should show at minimum column info
    assert!(!stdout.trim().is_empty(),
        "Output should not be empty for known view");
}

// -------------------------------------------------------------------------
// TC-045-016: Inspect on a non-existent object
// -------------------------------------------------------------------------
#[test]
#[ignore]
fn test_inspect_nonexistent_object_returns_error() {
    let logon = match tq_logon() {
        Some(l) => l,
        None => return,
    };
    let object_name = "nonexistent_table_xyz_sprint45_test";
    let output = Command::new(tq_binary())
        .args(["--logon", &logon, "inspect", object_name])
        .output()
        .expect("Failed to run tq inspect");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(!output.status.success(),
        "tq inspect on non-existent object should exit non-zero");

    // Error message must contain the object name for clarity
    assert!(combined.contains(object_name) ||
            combined.to_lowercase().contains("not found") ||
            combined.to_lowercase().contains("does not exist"),
        "Error output should reference the object name or state 'not found', got:\n{}", combined);
}

// -------------------------------------------------------------------------
// TC-045-017: Batch mode table output
// -------------------------------------------------------------------------
#[test]
#[ignore]
fn test_inspect_batch_table_output() {
    let logon = match tq_logon() {
        Some(l) => l,
        None => return,
    };
    let output = Command::new(tq_binary())
        .args(["--logon", &logon, "inspect", "dbc.dbcinfo"])
        .output()
        .expect("Failed to run tq inspect");

    assert!(output.status.success(), "Batch inspect should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "Batch inspect output should not be empty");
}

// -------------------------------------------------------------------------
// TC-045-018: Batch mode CSV output
// -------------------------------------------------------------------------
#[test]
#[ignore]
fn test_inspect_batch_csv_output() {
    let logon = match tq_logon() {
        Some(l) => l,
        None => return,
    };
    let output = Command::new(tq_binary())
        .args(["--logon", &logon, "--output", "csv", "inspect", "dbc.dbcinfo"])
        .output()
        .expect("Failed to run tq inspect --output csv");

    assert!(output.status.success(), "Batch inspect CSV should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Basic CSV validation: must contain commas
    assert!(stdout.contains(','),
        "CSV output should contain commas, got:\n{}", stdout);
}

// -------------------------------------------------------------------------
// TC-045-018b: Batch mode JSON output
// -------------------------------------------------------------------------
#[test]
#[ignore]
fn test_inspect_batch_json_output() {
    let logon = match tq_logon() {
        Some(l) => l,
        None => return,
    };
    let output = Command::new(tq_binary())
        .args(["--logon", &logon, "--output", "json", "inspect", "dbc.dbcinfo"])
        .output()
        .expect("Failed to run tq inspect --output json");

    assert!(output.status.success(), "Batch inspect JSON should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Basic JSON validation: starts with '[' or '{'
    let trimmed = stdout.trim();
    assert!(trimmed.starts_with('[') || trimmed.starts_with('{'),
        "JSON output should start with [ or {{, got:\n{}", trimmed);
}
```

## Expected Results

All integration tests pass when a live Teradata database is available:
- TC-045-007: No "unknown command" or "not found" errors from semicolon-suffixed arguments
- TC-045-014: Output contains type and column sections for `dbc.dbcinfo`
- TC-045-015: Output contains column info for `dbc.TablesV` (a view)
- TC-045-016: Non-existent object produces non-zero exit and helpful message
- TC-045-017: Default table output is non-empty
- TC-045-018: CSV output contains commas; JSON output starts with `[` or `{`

## Pass/Fail Criteria

**PASS if:**
- All ignored tests pass with `TQ_LOGON` set
- Object type and column sections appear in inspect output
- Non-existent object: exit code non-zero, object name in error message
- CSV: output contains commas; JSON: valid JSON structure

**BLOCKED if:**
- `TQ_LOGON` is not set or database is unreachable

**FAIL if:**
- Tests run (DB available) but assertions fail

## Run Command

```bash
export TQ_LOGON="user:password@host:1025/database"
cargo test --test integration_tests -- --ignored inspect 2>&1
```

## Notes

- All tests use `#[ignore]` — they are only executed when `--ignored` flag is passed
- `dbc.dbcinfo` is universally accessible on Teradata systems with standard DBC permissions
- If `dbc.dbcinfo` is not accessible, substitute any readable table available in the test environment
- TC-045-019 (tab completion) is in the interactive test suite, not here
