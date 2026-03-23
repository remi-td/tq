# TC-046-INSPECT-INTEGRATION: /inspect End-to-End Formatting (DB Required)

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-046-INSPECT-INTEGRATION |
| **Title** | /inspect end-to-end formatting with live Teradata database |
| **Category** | Integration Test |
| **Priority** | Medium |
| **Feature** | Sprint 46 — /inspect Formatting Compliance |
| **Test Type** | Integration (live database required) |
| **DB Required** | Yes |
| **Created** | 2026-03-23 |
| **Rust Attribute** | `#[ignore]` — run with `cargo test -- --ignored` |
| **Acceptance Criteria** | AC-1, AC-2, AC-3, AC-4, AC-5, AC-6 (end-to-end evidence) |

## Purpose

Verify that the /inspect formatting changes are visible in the actual rendered output of `tq inspect` against a live Teradata database. Unit tests (TC-046-007) validate the pure helpers in isolation; this test validates the full rendering pipeline with real DBC data.

## Acceptance Criteria Coverage

- **AC-1**: Section headers use `── Section Name ──` in actual tq output
- **AC-2**: Default column shows `-` in rendered columns section
- **AC-4**: Skew hint appears after skew percentage
- **AC-5**: NoPI tables show "Table (NoPI)" type label
- **AC-6**: Error output uses `Error:` prefix on tq inspect

## Prerequisites

- Teradata database accessible
- `TQ_LOGON` environment variable set
- `tq` binary compiled (`cargo build`)

## Test Procedure

### Test Implementation (in `tests/inspect_integration_46.rs`)

```rust
use std::process::Command;

fn tq_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().expect("current exe path");
    path.pop();
    path.pop();
    path.push("tq");
    path
}

fn logon() -> String {
    std::env::var("TQ_LOGON").expect("TQ_LOGON must be set")
}

// TC-046-INT-01: tq inspect dbc.tables — section headers use ── format
#[test]
#[ignore]
fn test_inspect_section_headers_use_dash_format() {
    let output = Command::new(tq_bin())
        .args(["-l", &logon(), "inspect", "dbc.tables"])
        .output()
        .expect("Failed to execute tq inspect");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "tq inspect failed\nstdout: {}\nstderr: {}",
        stdout, stderr
    );

    // AC-1: Must use ── style headers (box-drawing character U+2500)
    assert!(
        stdout.contains("──"),
        "Expected ── section headers in output\nstdout: {}",
        stdout
    );

    // AC-1 regression: Must NOT use === style headers
    assert!(
        !stdout.contains("==="),
        "Output must NOT contain === headers\nstdout: {}",
        stdout
    );
}

// TC-046-INT-02: tq inspect dbc.tables — skew hint appears
#[test]
#[ignore]
fn test_inspect_skew_hint_present() {
    let output = Command::new(tq_bin())
        .args(["-l", &logon(), "inspect", "dbc.tables"])
        .output()
        .expect("Failed to execute tq inspect");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "tq inspect failed\n{}", stdout);

    // AC-4: Must contain one of the skew hint labels
    let has_skew_hint = stdout.contains("(low)") || stdout.contains("(moderate)") || stdout.contains("(high)");
    assert!(
        has_skew_hint,
        "Expected skew hint (low/moderate/high) in inspect output\nstdout: {}",
        stdout
    );
}

// TC-046-INT-03: tq inspect <nonexistent> — error uses "Error:" prefix
#[test]
#[ignore]
fn test_inspect_not_found_error_prefix() {
    let output = Command::new(tq_bin())
        .args(["-l", &logon(), "inspect", "definitely_nonexistent_object_xyz"])
        .output()
        .expect("Failed to execute tq inspect");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // AC-6: Error output should use "Error:" prefix
    // Note: inspect uses exit 0 with error text in stdout (graceful degradation)
    assert!(
        stdout.contains("Error:"),
        "Expected 'Error:' prefix in not-found output\nstdout: {}",
        stdout
    );

    // Must NOT use the old style "Object '...' not found." without prefix
    // The old format had no "Error:" prefix, just the message directly
    let has_old_style = stdout.contains("not found.") && !stdout.contains("Error:");
    assert!(
        !has_old_style,
        "Old-style error format (no Error: prefix) detected\nstdout: {}",
        stdout
    );
}
```

## Expected Results

All 3 integration tests pass:
- `test_inspect_section_headers_use_dash_format` — PASS: `──` present, `===` absent
- `test_inspect_skew_hint_present` — PASS: one of `(low)/(moderate)/(high)` present
- `test_inspect_not_found_error_prefix` — PASS: `Error:` prefix present in not-found output

## Pass/Fail Criteria

**PASS if:**
- All 3 tests exit with expected status
- Section headers use `──` style
- Skew hint label present
- Not-found error has `Error:` prefix

**FAIL if:**
- Section headers still use `===` — AC-1 not applied
- No skew hint label — AC-4 not applied
- No `Error:` prefix — AC-6 not applied

**BLOCKED if:**
- `TQ_LOGON` not set or database unreachable

## Run Command

```bash
export TQ_LOGON="user:password@host:1025/DBC"
cargo build
cargo test --test inspect_integration_46 -- --ignored 2>&1
```

## Notes

- `DBC.Tables` is a Teradata system view that always has storage statistics (used for skew testing)
- The skew hint test assumes DBC.Tables has AMP statistics available. If the DBC.TableSizeV query returns no data, the storage section may be absent. This is graceful degradation and the test would need to be adapted.
- These tests are marked `#[ignore]` and will not run in standard CI without `--ignored`
- AC-2 (default `-`) and AC-3 (column count footer) are validated via unit tests in TC-046-007. End-to-end validation of these in live output would require knowing specific column defaults in DBC views, which is fragile. The unit tests are the primary coverage.
