# TC-033-007: Interactive Tests - /peek Command in REPL

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-033-007 |
| **Title** | Interactive Tests - /peek Command in REPL |
| **Category** | Interactive Test |
| **Priority** | Critical |
| **Feature** | Sprint 33 - Data Sampling Commands (AC-7, AC-10) |
| **Test Type** | Interactive (#[ignore] - requires live database + PTY) |
| **Created** | 2026-02-03 |

## Purpose

Verify that the `/peek` command works correctly in the interactive REPL environment, including tab completion and help text.

## Acceptance Criteria Coverage

- **AC-7**: Tab completion - Both commands in metacommand completion menu
- **AC-10**: Help text updated - `/help` shows both commands with examples

## Scope

This test validates:
- `/peek` command works in REPL PTY environment
- Tab completion suggests `/peek` command
- Help text documents `/peek` command
- Metadata and data are displayed correctly
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
fn test_peek_command_in_repl() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Execute: /peek dbc.databases
    p.send_line("/peek dbc.databases").expect("Failed to send");

    // Wait for output
    thread::sleep(Duration::from_millis(1500));
    let output = read_available_output(&mut p);

    // Verify: Output contains metadata section
    assert!(output.contains("Column") || output.contains("Type"),
            "Should show column metadata");

    // Verify: Output contains data section
    assert!(output.contains("DatabaseName") || output.contains("DATABASENAME"),
            "Should show table data");

    // Clean exit
    p.send_line("/quit").expect("Failed to quit");
}

#[test]
#[ignore] // Requires live database
fn test_peek_metadata_display() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Execute: /peek dbc.databases
    p.send_line("/peek dbc.databases").expect("Failed to send");

    // Wait for output
    thread::sleep(Duration::from_millis(1500));
    let output = read_available_output(&mut p);

    // Verify: Metadata includes data types
    assert!(
        output.contains("VARCHAR") || output.contains("CHAR") || output.contains("INT"),
        "Should show data types in metadata"
    );

    // Verify: Metadata includes nullable info
    assert!(
        output.contains("Nullable") || output.contains("NOT NULL") || output.contains("YES") || output.contains("NO"),
        "Should show nullable information"
    );

    // Clean exit
    p.send_line("/quit").expect("Failed to quit");
}

#[test]
#[ignore] // Requires live database
fn test_peek_tab_completion() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Type: /pee<TAB>
    p.send("/pee").expect("Failed to send");
    thread::sleep(Duration::from_millis(100));
    p.send("\t").expect("Failed to send TAB");

    // Wait for completion
    thread::sleep(Duration::from_millis(500));
    let output = read_available_output(&mut p);

    // Verify: Tab completion suggests /peek
    assert!(output.contains("peek"),
            "Tab completion should suggest /peek command");

    // Clean exit
    p.send_line("\n/quit").expect("Failed to quit");
}

#[test]
#[ignore] // Requires live database
fn test_peek_help_text() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Execute: /help
    p.send_line("/help").expect("Failed to send");

    // Wait for help output
    thread::sleep(Duration::from_millis(500));
    let output = read_available_output(&mut p);

    // Verify: Help text includes /peek command
    assert!(output.contains("/peek"), "Help should document /peek command");
    assert!(
        output.contains("first") || output.contains("preview") || output.contains("column"),
        "Help should describe peek functionality"
    );

    // Clean exit
    p.send_line("/quit").expect("Failed to quit");
}

#[test]
#[ignore] // Requires live database
fn test_peek_invalid_table_error() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Execute: /peek nonexistent_table
    p.send_line("/peek nonexistent_table").expect("Failed to send");

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
fn test_peek_qualified_name() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Execute: /peek dbc.tables
    p.send_line("/peek dbc.tables").expect("Failed to send");

    // Wait for output
    thread::sleep(Duration::from_millis(1500));
    let output = read_available_output(&mut p);

    // Verify: Qualified name works
    assert!(output.contains("Column") || output.contains("Type"),
            "Should show metadata for qualified table name");

    // Clean exit
    p.send_line("/quit").expect("Failed to quit");
}

#[test]
#[ignore] // Requires live database
fn test_peek_state_preserved() {
    // Spawn REPL
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Execute /peek
    p.send_line("/peek dbc.databases").expect("Failed to send");
    thread::sleep(Duration::from_millis(1500));
    let _output1 = read_available_output(&mut p);

    // Execute another command
    p.send_line("/session").expect("Failed to send");
    thread::sleep(Duration::from_millis(500));
    let output2 = read_available_output(&mut p);

    // Verify: REPL still functional
    assert!(output2.contains("User:") || output2.contains("Connection:"),
            "REPL should remain functional after /peek");

    // Clean exit
    p.send_line("/quit").expect("Failed to quit");
}
```

## Expected Results

All interactive tests pass:
- `/peek` command executes in REPL
- Metadata and data are displayed correctly
- Tab completion suggests `/peek`
- Help text documents `/peek`
- Error messages are clear
- REPL state is preserved after command

## Pass/Fail Criteria

**PASS if:**
- All 7 interactive tests pass
- `/peek` command works in REPL
- Metadata is displayed (columns, types, nullable)
- Data is displayed (first 5 rows)
- Tab completion includes `/peek`
- Help text is clear and accurate
- Error handling works correctly
- REPL remains functional after command

**FAIL if:**
- Any interactive test fails
- Command doesn't work in REPL
- Metadata is missing or incomplete
- Data is not shown
- Tab completion is broken
- Help text is missing or unclear
- Errors are not handled
- REPL crashes or becomes unresponsive

## Notes

- These are INTERACTIVE tests - require live database + PTY
- Marked with #[ignore] attribute
- Run with: `cargo test --test interactive_tests test_peek -- --ignored`
- Uses expectrl for PTY simulation
- Companion tests: TC-033-003 (unit), TC-033-005 (integration), TC-033-010 (batch)
- Validates AC-7, AC-10 from Sprint 33
- Tests user-facing REPL behavior with metadata display
