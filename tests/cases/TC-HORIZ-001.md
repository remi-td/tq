# TC-HORIZ-001: Unit Test - Right Arrow Column Offset Increment

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-001 |
| **Title** | Unit Test - Right Arrow Column Offset Increment |
| **Category** | Unit Test |
| **Priority** | Critical |
| **Feature** | Sprint 29 - Horizontal Paging (AC-1) |
| **Test Type** | Unit |
| **Created** | 2026-01-30 |

## Purpose

Verify that the pager's internal logic correctly increments the column offset when the right arrow key is pressed and columns are hidden to the right.

## Acceptance Criteria Coverage

- **AC-1**: Right arrow (→) key scrolls view one column to the right when columns are hidden

## Scope

This test validates:
- `handle_key()` method processes Right arrow key event
- `col_offset` increments by 1 when columns are hidden to the right
- `col_offset` does not increment past last valid position
- Bounds checking prevents scrolling beyond available columns

## Prerequisites

- Rust test framework available
- Access to `Pager` struct test module in `src/commands/repl/pager.rs`

## Test Procedure

### Test Implementation (in `src/commands/repl/pager.rs`):

```rust
#[test]
fn test_right_arrow_increments_col_offset() {
    // Setup: 20 columns, terminal fits 5 columns, start at col_offset=0
    let mut pager = Pager::new(create_test_table(20), 80);
    assert_eq!(pager.col_offset, 0);

    // Action: Press right arrow
    pager.handle_key(KeyCode::Right);

    // Assert: col_offset incremented
    assert_eq!(pager.col_offset, 1);
}

#[test]
fn test_right_arrow_multiple_presses() {
    let mut pager = Pager::new(create_test_table(20), 80);

    // Press right arrow 5 times
    for _ in 0..5 {
        pager.handle_key(KeyCode::Right);
    }

    assert_eq!(pager.col_offset, 5);
}

#[test]
fn test_right_arrow_at_last_position_no_effect() {
    let mut pager = Pager::new(create_test_table(10), 80);

    // Scroll to last valid position
    while pager.hidden_columns_right() > 0 {
        pager.handle_key(KeyCode::Right);
    }

    let last_offset = pager.col_offset;

    // Press right arrow when already at rightmost position
    pager.handle_key(KeyCode::Right);

    // Assert: col_offset unchanged
    assert_eq!(pager.col_offset, last_offset);
}

#[test]
fn test_right_arrow_bounds_checking() {
    let mut pager = Pager::new(create_test_table(5), 200); // Wide terminal, few columns

    // Attempt to scroll right when all columns fit
    pager.handle_key(KeyCode::Right);

    // Assert: col_offset stays at 0 (no scrolling needed)
    assert_eq!(pager.col_offset, 0);
}
```

## Expected Results

All unit tests pass:
- `col_offset` increments correctly for each right arrow press
- Multiple right arrow presses accumulate correctly
- Scrolling stops at rightmost valid position
- No scrolling occurs when all columns fit in terminal

## Pass/Fail Criteria

**PASS if:**
- All 4 unit tests compile and pass
- `col_offset` increments by 1 per right arrow press
- Bounds checking prevents scrolling past last column
- No panics or overflow errors

**FAIL if:**
- Any unit test fails
- `col_offset` increments incorrectly (e.g., by 2, stays at 0)
- Scrolling past available columns is allowed
- Panic or overflow occurs

## Notes

- This is a UNIT test - no database or PTY required
- Tests internal logic only, not visual output
- Companion to TC-HORIZ-011 (interactive test for same AC)
- Part of the 20-25 unit tests for horizontal paging
