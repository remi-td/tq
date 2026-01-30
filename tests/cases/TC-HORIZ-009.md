# TC-HORIZ-009: Unit Test - Column Position Preserved During Vertical Scroll

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-009 |
| **Title** | Unit Test - Column Position Preserved During Vertical Scroll |
| **Category** | Unit Test |
| **Priority** | Critical |
| **Feature** | Sprint 29 - Horizontal Paging (AC-11) |
| **Test Type** | Unit |
| **Created** | 2026-01-30 |

## Purpose

Verify that the column offset (horizontal position) is preserved when scrolling vertically through rows.

## Acceptance Criteria Coverage

- **AC-11**: Column position preserved when scrolling vertically

## Scope

This test validates:
- `col_offset` remains unchanged when `row_offset` changes
- Vertical navigation keys (j/k, Space/b, g/G) don't affect col_offset
- Horizontal and vertical scrolling are independent
- Integration of horizontal and vertical paging state

## Prerequisites

- Rust test framework available
- Access to `Pager` struct test module in `src/commands/repl/pager.rs`

## Test Procedure

### Test Implementation (in `src/commands/repl/pager.rs`):

```rust
#[test]
fn test_col_offset_preserved_vertical_scroll_down() {
    let mut pager = Pager::new(create_test_table_wide_tall(30, 100), 80);

    // Scroll right to column 10
    pager.col_offset = 10;

    // Scroll down vertically (j key)
    pager.handle_key(KeyCode::Char('j'));

    // col_offset should be unchanged
    assert_eq!(pager.col_offset, 10);
}

#[test]
fn test_col_offset_preserved_vertical_scroll_up() {
    let mut pager = Pager::new(create_test_table_wide_tall(25, 100), 80);

    // Set vertical and horizontal position
    pager.row_offset = 50;
    pager.col_offset = 15;

    // Scroll up vertically (k key)
    pager.handle_key(KeyCode::Char('k'));

    // col_offset should be unchanged
    assert_eq!(pager.col_offset, 15);
}

#[test]
fn test_col_offset_preserved_page_down() {
    let mut pager = Pager::new(create_test_table_wide_tall(30, 100), 80);

    pager.col_offset = 8;

    // Page down (Space key)
    pager.handle_key(KeyCode::Char(' '));

    // col_offset should be unchanged
    assert_eq!(pager.col_offset, 8);
}

#[test]
fn test_col_offset_preserved_page_up() {
    let mut pager = Pager::new(create_test_table_wide_tall(30, 100), 80);

    pager.row_offset = 50;
    pager.col_offset = 12;

    // Page up (b key)
    pager.handle_key(KeyCode::Char('b'));

    // col_offset should be unchanged
    assert_eq!(pager.col_offset, 12);
}

#[test]
fn test_col_offset_preserved_jump_to_top() {
    let mut pager = Pager::new(create_test_table_wide_tall(25, 100), 80);

    pager.row_offset = 75;
    pager.col_offset = 20;

    // Jump to top (g key)
    pager.handle_key(KeyCode::Char('g'));

    // col_offset should be unchanged
    assert_eq!(pager.col_offset, 20);
}

#[test]
fn test_col_offset_preserved_jump_to_bottom() {
    let mut pager = Pager::new(create_test_table_wide_tall(25, 100), 80);

    pager.col_offset = 18;

    // Jump to bottom (G key - uppercase)
    pager.handle_key(KeyCode::Char('G'));

    // col_offset should be unchanged
    assert_eq!(pager.col_offset, 18);
}

#[test]
fn test_col_offset_preserved_complex_sequence() {
    let mut pager = Pager::new(create_test_table_wide_tall(30, 100), 80);

    // Scroll right
    pager.col_offset = 10;

    // Complex vertical navigation sequence
    pager.handle_key(KeyCode::Char('j'));  // down
    pager.handle_key(KeyCode::Char('j'));  // down
    pager.handle_key(KeyCode::Char('k'));  // up
    pager.handle_key(KeyCode::Char(' '));  // page down
    pager.handle_key(KeyCode::Char('b'));  // page up

    // col_offset should still be 10
    assert_eq!(pager.col_offset, 10);
}
```

## Expected Results

All unit tests pass:
- col_offset unchanged by j/k (row up/down)
- col_offset unchanged by Space/b (page down/up)
- col_offset unchanged by g/G (jump to top/bottom)
- Complex vertical navigation sequences preserve col_offset
- Horizontal and vertical state are independent

## Pass/Fail Criteria

**PASS if:**
- All 7 unit tests compile and pass
- col_offset never changes during vertical navigation
- All vertical navigation keys tested (j, k, Space, b, g, G)
- Complex sequences preserve horizontal position

**FAIL if:**
- Any unit test fails
- col_offset changes during vertical scroll
- Horizontal position is lost
- State coupling between axes

## Notes

- This is a UNIT test - no database or PTY required
- Tests state management logic only
- Companion to TC-HORIZ-026 (interactive test for preservation)
- Critical for AC-11 - users expect horizontal position to stay fixed
- Requires test helper: `create_test_table_wide_tall(cols, rows)`
