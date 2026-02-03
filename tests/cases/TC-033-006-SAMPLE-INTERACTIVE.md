# TC-033-006: Interactive Tests - /sample Command in REPL

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-033-006 |
| **Title** | Interactive Tests - /sample Command in REPL |
| **Category** | Interactive Test |
| **Priority** | Critical |
| **Feature** | Sprint 33 - Data Sampling Commands (AC-7, AC-10) |
| **Test Type** | Interactive (#[ignore] - requires live database + PTY) |
| **Created** | 2026-02-03 |

## Purpose

Verify that the `/sample` command works correctly in the interactive REPL environment, including tab completion and help text.

## Acceptance Criteria Coverage

- **AC-7**: Tab completion - Both commands in metacommand completion menu
- **AC-10**: Help text updated - `/help` shows both commands with examples

## Scope

This test validates:
- `/sample` command works in REPL PTY environment
- Tab completion suggests `/sample` command
- Help text documents `/sample` command
- Output is displayed correctly in terminal
- REPL state is maintained after command execution

## Prerequisites

- Live Teradata database access
- TQ_LOGON environment variable or .env file set
- expectrl crate for PTY simulation
- Compiled tq binary

## Test Procedure

### Test Implementation (in `tests/interactive_tests.rs`):

```rust
#[test]
#[ignore] // Requires live database
fn test_sample_command_in_repl() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Execute: /sample dbc.databases
    p.send_line("/sample dbc.databases").expect("Failed to send");

    // Wait for output
    thread::sleep(Duration::from_millis(1000));
    let output = read_available_output(&mut p);

    // Verify: Output contains table data
    assert!(output.contains("DatabaseName") || output.contains("DATABASENAME"),
            "Should show column headers");
    assert!(output.lines().count() > 5, "Should show multiple rows");

    // Clean exit
    p.send_line("/quit").expect("Failed to quit");
}

#[test]
#[ignore] // Requires live database
fn test_sample_command_with_count() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Execute: /sample dbc.databases 3
    p.send_line("/sample dbc.databases 3").expect("Failed to send");

    // Wait for output
    thread::sleep(Duration::from_millis(1000));
    let output = read_available_output(&mut p);

    // Verify: Output contains table data (approximately 3 rows)
    assert!(output.contains("DatabaseName") || output.contains("DATABASENAME"),
            "Should show column headers");

    // Clean exit
    p.send_line("/quit").expect("Failed to quit");
}

#[test]
#[ignore] // Requires live database
fn test_sample_tab_completion() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Type: /sam<TAB>
    p.send("/sam").expect("Failed to send");
    thread::sleep(Duration::from_millis(100));
    p.send("\t").expect("Failed to send TAB"); // Tab key

    // Wait for completion
    thread::sleep(Duration::from_millis(500));
    let output = read_available_output(&mut p);

    // Verify: Tab completion suggests /sample
    assert!(output.contains("sample"),
            "Tab completion should suggest /sample command");

    // Clean exit
    p.send_line("\n/quit").expect("Failed to quit");
}

#[test]
#[ignore] // Requires live database
fn test_sample_help_text() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Execute: /help
    p.send_line("/help").expect("Failed to send");

    // Wait for help output
    thread::sleep(Duration::from_millis(500));
    let output = read_available_output(&mut p);

    // Verify: Help text includes /sample command
    assert!(output.contains("/sample"), "Help should document /sample command");
    assert!(output.contains("random"), "Help should mention random sampling");

    // Clean exit
    p.send_line("/quit").expect("Failed to quit");
}

#[test]
#[ignore] // Requires live database
fn test_sample_invalid_table_error() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Execute: /sample nonexistent_table
    p.send_line("/sample nonexistent_table").expect("Failed to send");

    // Wait for error
    thread::sleep(Duration::from_millis(1000));
    let output = read_available_output(&mut p);

    // Verify: Clear error message
    assert!(
        output.contains("table") || output.contains("object") || output.contains("not found"),
        "Should show clear error for invalid table: {}", output
    );

    // Clean exit
    p.send_line("/quit").expect("Failed to quit");
}

#[test]
#[ignore] // Requires live database
fn test_sample_state_preserved() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Execute /sample
    p.send_line("/sample dbc.databases 5").expect("Failed to send");
    thread::sleep(Duration::from_millis(1000));
    let _output1 = read_available_output(&mut p);

    // Execute another command
    p.send_line("/session").expect("Failed to send");
    thread::sleep(Duration::from_millis(500));
    let output2 = read_available_output(&mut p);

    // Verify: REPL still functional
    assert!(output2.contains("User:") || output2.contains("Connection:"),
            "REPL should remain functional after /sample");

    // Clean exit
    p.send_line("/quit").expect("Failed to quit");
}
```

## Expected Results

All interactive tests pass:
- `/sample` command executes in REPL
- Output is displayed correctly
- Tab completion suggests `/sample`
- Help text documents `/sample`
- Error messages are clear
- REPL state is preserved after command

## Pass/Fail Criteria

**PASS if:**
- All 6 interactive tests pass
- `/sample` command works in REPL
- Tab completion includes `/sample`
- Help text is clear and accurate
- Error handling works correctly
- REPL remains functional after command

**FAIL if:**
- Any interactive test fails
- Command doesn't work in REPL
- Tab completion is broken
- Help text is missing or unclear
- Errors are not handled
- REPL crashes or becomes unresponsive

## Notes

- These are INTERACTIVE tests - require live database + PTY
- Marked with #[ignore] attribute
- Run with: `cargo test --test interactive_tests test_sample -- --ignored`
- Uses expectrl for PTY simulation
- Companion tests: TC-033-002 (unit), TC-033-004 (integration), TC-033-009 (batch)
- Validates AC-7, AC-10 from Sprint 33
- Tests user-facing REPL behavior
