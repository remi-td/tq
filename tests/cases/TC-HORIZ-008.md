# TC-HORIZ-008: Unit Test - L Key Jump to Last Column

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-008 |
| **Title** | Unit Test - L Key Jump to Last Column |
| **Category** | Unit Test |
| **Priority** | High |
| **Feature** | Sprint 29 - Horizontal Paging (AC-10) |
| **Test Type** | Unit |
| **Created** | 2026-01-30 |

## Purpose

Verify that pressing the 'L' key (uppercase) jumps to the rightmost valid column position (last visible window).

## Acceptance Criteria Coverage

- **AC-10**: `L` key jumps to last column (rightmost position)

## Scope

This test validates:
- `handle_key()` recognizes 'L' (uppercase) as jump command
- `col_offset` is set to maximum valid position
- Jump works from any position
- Last position shows rightmost columns without overscroll
- Distinct from lowercase 'l' (scroll right by 1)

## Prerequisites

- Rust test framework available
- Access to `Pager` struct test module in `src/commands/repl/pager.rs`

## Test Procedure

### Test Implementation (in `src/commands/repl/pager.rs`):

```rust
#[test]
fn test_uppercase_l_jumps_to_last_column() {
    let mut pager = Pager::new(create_test_table(30), 80);

    // Start at beginning
    assert_eq!(pager.col_offset, 0);

    // Press 'L' (uppercase)
    pager.handle_key(KeyCode::Char('L'));

    // Should jump to last valid position
    // Last position is where hidden_columns_right() == 0
    assert_eq!(pager.hidden_columns_right(), 0);
    assert!(pager.col_offset > 0);
}

#[test]
fn test_uppercase_l_from_various_positions() {
    let test_cases = vec![0, 5, 10, 15];

    for start_offset in test_cases {
        let mut pager = Pager::new(create_test_table(30), 80);
        pager.col_offset = start_offset;

        pager.handle_key(KeyCode::Char('L'));

        // All should end at same last position
        assert_eq!(pager.hidden_columns_right(), 0,
                   "Jump from offset {} should show last columns", start_offset);
    }
}

#[test]
fn test_uppercase_l_at_end_is_idempotent() {
    let mut pager = Pager::new(create_test_table(25), 80);

    // Scroll to end first
    while pager.hidden_columns_right() > 0 {
        pager.handle_key(KeyCode::Right);
    }

    let last_offset = pager.col_offset;

    // Press 'L' when already at end
    pager.handle_key(KeyCode::Char('L'));

    // Should remain at same position
    assert_eq!(pager.col_offset, last_offset);
}

#[test]
fn test_lowercase_l_vs_uppercase_l() {
    let mut pager1 = Pager::new(create_test_table(30), 80);
    let mut pager2 = Pager::new(create_test_table(30), 80);

    // Both start at offset 5
    pager1.col_offset = 5;
    pager2.col_offset = 5;

    // lowercase 'l' scrolls right by 1
    pager1.handle_key(KeyCode::Char('l'));
    assert_eq!(pager1.col_offset, 6);

    // uppercase 'L' jumps to end
    pager2.handle_key(KeyCode::Char('L'));
    assert_eq!(pager2.hidden_columns_right(), 0);
    assert!(pager2.col_offset > 10); // Much further than 6
}

#[test]
fn test_uppercase_l_calculation() {
    let mut pager = Pager::new(create_test_table(25), 80);

    pager.handle_key(KeyCode::Char('L'));

    let visible = pager.visible_column_count();
    let expected_offset = 25 - visible;

    assert_eq!(pager.col_offset, expected_offset,
               "Last offset should be total_columns - visible_columns");
}
```

## Expected Results

All unit tests pass:
- 'L' jumps to last valid position (hidden_columns_right = 0)
- Jump works from any starting position
- 'L' at end is safe (idempotent)
- 'L' (jump) distinct from 'l' (scroll by 1)
- Formula: `col_offset = total_columns - visible_column_count`

## Pass/Fail Criteria

**PASS if:**
- All 5 unit tests compile and pass
- 'L' sets col_offset to last valid position
- No overscroll (hidden_columns_right = 0)
- Distinguishable from lowercase 'l'
- Calculation is mathematically correct

**FAIL if:**
- Any unit test fails
- 'L' scrolls by 1 instead of jumping
- Overscroll occurs (hidden_columns_right < 0 or > 0)
- Case sensitivity not working

## Notes

- This is a UNIT test - no database or PTY required
- Tests jump logic only, not visual output
- Companion to TC-HORIZ-020 (interactive test for L jump)
- Part of Vim-style navigation (h/l scroll, H/L jump)
- Last position calculation: `max(0, total_columns - visible_columns)`
