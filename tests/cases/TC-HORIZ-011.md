# TC-HORIZ-011: Interactive Test - Right Arrow Scrolls Right

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-011 |
| **Title** | Interactive Test - Right Arrow Scrolls Right |
| **Category** | Interactive Test |
| **Priority** | Critical |
| **Feature** | Sprint 29 - Horizontal Paging (AC-1) |
| **Test Type** | Interactive (expectrl) |
| **Created** | 2026-01-30 |

## Purpose

Verify that pressing the right arrow key in the pager scrolls the view one column to the right when columns are hidden, with visible confirmation of column shift.

## Acceptance Criteria Coverage

- **AC-1**: Right arrow (→) key scrolls view one column to the right when columns are hidden

## Scope

This test validates:
- Right arrow key is captured in pager mode
- Columns shift visibly to the right
- Leftmost column disappears from view
- New column appears on the right
- Status bar updates to show new column range
- Right indicator updates to show fewer hidden columns

## Prerequisites

- tq binary built and available
- Live Teradata database connection configured (TQ_LOGON)
- Test database with wide table (30+ columns)
- Terminal with PTY support (expectrl)

## Test Procedure

### Test Implementation (in `tests/interactive_tests.rs`):

```rust
#[test]
#[ignore] // Requires live database
fn test_right_arrow_scrolls_columns_right() {
    dotenvy::dotenv().ok();

    // Setup: Create wide table (30 columns)
    setup_wide_test_table(30);

    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    // Query wide table to trigger paging
    p.send_line("SELECT * FROM test_wide_table_30;").expect("Failed to send");
    thread::sleep(Duration::from_millis(500));

    // Enter pager mode (should auto-activate for wide tables)
    let initial_output = read_available_output(&mut p);

    // Verify initial state: columns 1-5 visible (approximately)
    assert!(initial_output.contains("col_1"), "First column should be visible initially");
    assert!(initial_output.contains("Columns 1-"), "Status bar should show starting at column 1");

    // Get initial column headers
    let initial_leftmost = extract_leftmost_column(&initial_output);

    // Press right arrow key
    send_key(&mut p, KeyCode::Right);
    thread::sleep(Duration::from_millis(300));

    let after_scroll_output = read_available_output(&mut p);

    // Verify scroll occurred
    let new_leftmost = extract_leftmost_column(&after_scroll_output);
    assert_ne!(initial_leftmost, new_leftmost,
               "Leftmost column should change after right arrow");

    // Verify status bar updated (Columns 2-X now, not 1-X)
    assert!(after_scroll_output.contains("Columns 2-"),
            "Status bar should show starting at column 2 after one right scroll");

    // Clean exit
    send_key(&mut p, KeyCode::Char('q'));
    p.send_line("/quit").expect("Failed to quit");
}

#[test]
#[ignore] // Requires live database
fn test_right_arrow_multiple_presses() {
    dotenvy::dotenv().ok();

    setup_wide_test_table(30);

    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");

    p.send_line("SELECT * FROM test_wide_table_30;").expect("Failed to send");
    thread::sleep(Duration::from_millis(500));

    // Press right arrow 5 times
    for _ in 0..5 {
        send_key(&mut p, KeyCode::Right);
        thread::sleep(Duration::from_millis(200));
    }

    let output = read_available_output(&mut p);

    // After 5 right scrolls, should be at column 6+
    assert!(output.contains("Columns 6-") || output.contains("Columns 7-"),
            "After 5 right scrolls, should show columns starting at 6 or 7");

    send_key(&mut p, KeyCode::Char('q'));
    p.send_line("/quit").expect("Failed to quit");
}
```

## Expected Results

**Initial state:**
- Leftmost columns visible (1-5 approximately)
- Status bar shows: "Columns 1-5 of 30" (or similar)
- Right indicator shows: "(+25 cols) →" or similar

**After one right arrow:**
- New leftmost column visible (col_2 or col_3)
- Previous leftmost column (col_1 or col_2) no longer visible
- Status bar shows: "Columns 2-6 of 30" (or similar)
- Left indicator appears: "← (+1 cols)"
- Right indicator updates: "(+24 cols) →"

**After multiple right arrows:**
- Column range advances progressively
- Status bar reflects current position accurately

## Pass/Fail Criteria

**PASS if:**
- Right arrow key is captured and processed
- Columns visibly shift to the right
- Status bar updates to new column range
- Indicators update correctly
- Scrolling is smooth and predictable

**FAIL if:**
- Right arrow has no effect
- Columns don't shift visibly
- Status bar doesn't update
- Pager exits or crashes
- Wrong columns displayed

## Notes

- This is an INTERACTIVE test - requires live database and PTY
- Marked with `#[ignore]` - run with `cargo test -- --ignored`
- Companion to TC-HORIZ-001 (unit test for same AC)
- Requires helper functions:
  - `setup_wide_test_table(n)` - creates table with n columns
  - `send_key(p, key)` - sends key event to PTY
  - `extract_leftmost_column(output)` - parses leftmost column name
- Part of the 15-20 interactive tests for horizontal paging
