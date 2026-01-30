# TC-HORIZ-015: Interactive Test - Pager Exit Returns to REPL

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-015 |
| **Title** | Interactive Test - Pager Exit Returns to REPL |
| **Category** | Interactive Test |
| **Priority** | Critical (Safety) |
| **Feature** | Sprint 29 - Horizontal Paging (AC-5) |
| **Test Type** | Interactive (expectrl) |
| **Created** | 2026-01-30 |

## Purpose

Verify that pressing 'q' or Esc key in the pager exits paging mode and returns to the REPL prompt WITHOUT exiting the entire program.

## Acceptance Criteria Coverage

- **AC-5**: `q` or `Esc` key exits paging mode and returns to REPL prompt

## Scope

This test validates:
- 'q' key exits pager, returns to `tq>` prompt
- Esc key exits pager, returns to `tq>` prompt
- REPL remains active after pager exit
- Connection preserved after pager exit
- User can run new queries after exiting pager
- CRITICAL: Program does NOT exit entirely

## Prerequisites

- Live database, wide test table

## Test Procedure

```rust
#[test]
#[ignore]
fn test_q_key_exits_pager_returns_to_repl() {
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Enter pager
    p.send_line("SELECT * FROM test_wide_table_30;");
    thread::sleep(Duration::from_millis(500));

    // Verify in pager (status bar visible)
    let pager_output = read_available_output(&mut p);
    assert!(pager_output.contains("Columns "), "Should be in pager mode");

    // Press 'q' to exit
    send_key(&mut p, KeyCode::Char('q'));
    thread::sleep(Duration::from_millis(300));

    // Verify back at REPL prompt
    let after_exit = read_available_output(&mut p);
    assert!(after_exit.contains("tq>"), "Should return to tq> prompt");

    // Verify can run new query
    p.send_line("SELECT 1;");
    p.expect("1").expect("Should execute new query");

    p.send_line("/quit").expect("Failed to quit");
}

#[test]
#[ignore]
fn test_esc_key_exits_pager_returns_to_repl() {
    let mut p = spawn_tq_repl();
    p.expect("Connected to");

    p.send_line("SELECT * FROM test_wide_table_30;");
    thread::sleep(Duration::from_millis(500));

    // Press Esc to exit
    send_key(&mut p, KeyCode::Esc);
    thread::sleep(Duration::from_millis(300));

    let after_exit = read_available_output(&mut p);
    assert!(after_exit.contains("tq>"), "Esc should return to prompt");

    // Verify REPL still functional
    p.send_line("/help");
    p.expect("Commands").expect("REPL should still work");

    p.send_line("/quit");
}

#[test]
#[ignore]
fn test_pager_exit_does_not_exit_program() {
    let mut p = spawn_tq_repl();
    p.expect("Connected to");

    // Enter pager multiple times, exit each time
    for _ in 0..3 {
        p.send_line("SELECT * FROM test_wide_table_20;");
        thread::sleep(Duration::from_millis(400));

        send_key(&mut p, KeyCode::Char('q'));
        thread::sleep(Duration::from_millis(300));

        // Verify still at prompt
        let output = read_available_output(&mut p);
        assert!(output.contains("tq>"), "Should still be in REPL");
    }

    p.send_line("/quit");
}
```

## Expected Results

- 'q' key exits pager, shows `tq>` prompt
- Esc key exits pager, shows `tq>` prompt
- REPL remains active (can run new commands)
- Connection preserved
- Program does NOT exit

## Pass/Fail Criteria

PASS: Pager exits to REPL, program stays running, can execute new queries
FAIL: Program exits entirely, hangs, doesn't return to prompt

## Notes

- CRITICAL safety requirement
- Tests both 'q' and Esc keys
- Verifies REPL functionality after exit
- Related to existing pager safety tests
