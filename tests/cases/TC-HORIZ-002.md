# TC-HORIZ-002: Unit Test - Left Arrow Column Offset Decrement

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-002 |
| **Title** | Unit Test - Left Arrow Column Offset Decrement |
| **Category** | Unit Test |
| **Priority** | Critical |
| **Feature** | Sprint 29 - Horizontal Paging (AC-2) |
| **Test Type** | Unit |
| **Created** | 2026-01-30 |

## Purpose

Verify that the pager's internal logic correctly decrements the column offset when the left arrow key is pressed and the view has been scrolled right.

## Acceptance Criteria Coverage

- **AC-2**: Left arrow (←) key scrolls view one column to the left when at scrolled position

## Scope

This test validates:
- `handle_key()` method processes Left arrow key event
- `col_offset` decrements by 1 when scrolled to the right
- `col_offset` does not go below 0 (leftmost position)
- Bounds checking prevents scrolling before first column

## Prerequisites

- Rust test framework available
- Access to `Pager` struct test module in `src/commands/repl/pager.rs`

## Test Procedure

### Test Implementation (in `src/commands/repl/pager.rs`):

```rust
#[test]
fn test_left_arrow_decrements_col_offset() {
    let mut pager = Pager::new(create_test_table(20), 80);

    // Setup: Scroll right first
    pager.col_offset = 5;

    // Action: Press left arrow
    pager.handle_key(KeyCode::Left);

    // Assert: col_offset decremented
    assert_eq!(pager.col_offset, 4);
}

#[test]
fn test_left_arrow_multiple_presses() {
    let mut pager = Pager::new(create_test_table(20), 80);

    // Setup: Start at col_offset=10
    pager.col_offset = 10;

    // Press left arrow 5 times
    for _ in 0..5 {
        pager.handle_key(KeyCode::Left);
    }

    assert_eq!(pager.col_offset, 5);
}

#[test]
fn test_left_arrow_at_first_position_no_effect() {
    let mut pager = Pager::new(create_test_table(20), 80);

    // Setup: Already at leftmost position
    assert_eq!(pager.col_offset, 0);

    // Action: Press left arrow
    pager.handle_key(KeyCode::Left);

    // Assert: col_offset unchanged (stays at 0)
    assert_eq!(pager.col_offset, 0);
}

#[test]
fn test_left_arrow_return_to_start() {
    let mut pager = Pager::new(create_test_table(20), 80);

    // Scroll right 10 times
    for _ in 0..10 {
        pager.handle_key(KeyCode::Right);
    }

    // Scroll left 10 times
    for _ in 0..10 {
        pager.handle_key(KeyCode::Left);
    }

    // Should be back at start
    assert_eq!(pager.col_offset, 0);
}
```

## Expected Results

All unit tests pass:
- `col_offset` decrements correctly for each left arrow press
- Multiple left arrow presses accumulate correctly
- `col_offset` never goes negative
- Scrolling right then left returns to original position

## Pass/Fail Criteria

**PASS if:**
- All 4 unit tests compile and pass
- `col_offset` decrements by 1 per left arrow press
- Bounds checking prevents negative col_offset
- Round-trip (right then left) returns to start

**FAIL if:**
- Any unit test fails
- `col_offset` goes negative
- `col_offset` decrements incorrectly
- Panic occurs

## Notes

- This is a UNIT test - no database or PTY required
- Tests internal logic only, not visual output
- Companion to TC-HORIZ-012 (interactive test for same AC)
- Part of the 20-25 unit tests for horizontal paging
