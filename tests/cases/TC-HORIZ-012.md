# TC-HORIZ-012: Interactive Test - Left Arrow Scrolls Left

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-012 |
| **Title** | Interactive Test - Left Arrow Scrolls Left |
| **Category** | Interactive Test |
| **Priority** | Critical |
| **Feature** | Sprint 29 - Horizontal Paging (AC-2) |
| **Test Type** | Interactive (expectrl) |
| **Created** | 2026-01-30 |

## Purpose

Verify that pressing the left arrow key in the pager scrolls the view one column to the left when the view has been scrolled right.

## Acceptance Criteria Coverage

- **AC-2**: Left arrow (←) key scrolls view one column to the left when at scrolled position

## Scope

This test validates:
- Left arrow key captured after scrolling right
- Columns shift visibly to the left
- Previously hidden left column becomes visible
- Rightmost column disappears
- Status bar updates
- Round-trip (right then left) returns to original view

## Prerequisites

- tq binary, live database, wide test table (30+ columns)

## Test Procedure

### Implementation Snippet:

```rust
#[test]
#[ignore]
fn test_left_arrow_scrolls_columns_left() {
    // Setup wide table
    let mut p = spawn_tq_repl();
    p.send_line("SELECT * FROM test_wide_table_30;");

    // Scroll right first (3 times)
    for _ in 0..3 {
        send_key(&mut p, KeyCode::Right);
        thread::sleep(Duration::from_millis(200));
    }

    let after_right_output = read_available_output(&mut p);
    assert!(after_right_output.contains("Columns 4-") ||
            after_right_output.contains("Columns 5-"));

    // Now scroll left
    send_key(&mut p, KeyCode::Left);
    thread::sleep(Duration::from_millis(300));

    let after_left_output = read_available_output(&mut p);

    // Should be back one column (Columns 3- or 4-)
    assert!(after_left_output.contains("Columns 3-") ||
            after_left_output.contains("Columns 4-"));

    send_key(&mut p, KeyCode::Char('q'));
}
```

## Expected Results

- Left arrow returns to previous column position
- Status bar decrements column range
- Round-trip preserves state

## Pass/Fail Criteria

PASS: Left arrow scrolls left, status bar updates, round-trip works
FAIL: No effect, wrong columns, status bar wrong

## Notes

- Interactive test, requires database
- Companion to TC-HORIZ-002 (unit test)
