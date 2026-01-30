# TC-HORIZ-007: Unit Test - H Key Jump to First Column

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-007 |
| **Title** | Unit Test - H Key Jump to First Column |
| **Category** | Unit Test |
| **Priority** | High |
| **Feature** | Sprint 29 - Horizontal Paging (AC-9) |
| **Test Type** | Unit |
| **Created** | 2026-01-30 |

## Purpose

Verify that pressing the 'H' key (uppercase) jumps to the leftmost column position (col_offset = 0).

## Acceptance Criteria Coverage

- **AC-9**: `H` key jumps to first column (leftmost position)

## Scope

This test validates:
- `handle_key()` recognizes 'H' (uppercase) as jump command
- `col_offset` is set to 0
- Jump works from any position
- Distinct from lowercase 'h' (scroll left by 1)

## Prerequisites

- Rust test framework available
- Access to `Pager` struct test module in `src/commands/repl/pager.rs`

## Test Procedure

### Test Implementation (in `src/commands/repl/pager.rs`):

```rust
#[test]
fn test_uppercase_h_jumps_to_first_column() {
    let mut pager = Pager::new(create_test_table(30), 80);

    // Scroll right to middle position
    pager.col_offset = 15;

    // Press 'H' (uppercase)
    pager.handle_key(KeyCode::Char('H'));

    // Should jump to start
    assert_eq!(pager.col_offset, 0);
}

#[test]
fn test_uppercase_h_from_various_positions() {
    let test_cases = vec![5, 10, 20, 25];

    for start_offset in test_cases {
        let mut pager = Pager::new(create_test_table(30), 80);
        pager.col_offset = start_offset;

        pager.handle_key(KeyCode::Char('H'));

        assert_eq!(pager.col_offset, 0,
                   "Jump from offset {} should go to 0", start_offset);
    }
}

#[test]
fn test_uppercase_h_at_start_is_idempotent() {
    let mut pager = Pager::new(create_test_table(20), 80);
    assert_eq!(pager.col_offset, 0);

    // Press 'H' when already at start
    pager.handle_key(KeyCode::Char('H'));

    // Should remain at 0
    assert_eq!(pager.col_offset, 0);
}

#[test]
fn test_lowercase_h_vs_uppercase_h() {
    let mut pager1 = Pager::new(create_test_table(30), 80);
    let mut pager2 = Pager::new(create_test_table(30), 80);

    // Both start at offset 10
    pager1.col_offset = 10;
    pager2.col_offset = 10;

    // lowercase 'h' scrolls left by 1
    pager1.handle_key(KeyCode::Char('h'));
    assert_eq!(pager1.col_offset, 9);

    // uppercase 'H' jumps to 0
    pager2.handle_key(KeyCode::Char('H'));
    assert_eq!(pager2.col_offset, 0);
}
```

## Expected Results

All unit tests pass:
- 'H' sets col_offset to 0 from any position
- Jump works from middle, end positions
- 'H' at start is safe (idempotent)
- 'H' (jump) distinct from 'h' (scroll by 1)

## Pass/Fail Criteria

**PASS if:**
- All 4 unit tests compile and pass
- 'H' always sets col_offset to 0
- Works from any starting position
- Distinguishable from lowercase 'h'

**FAIL if:**
- Any unit test fails
- 'H' scrolls by 1 instead of jumping
- col_offset not set to 0
- Case sensitivity not working

## Notes

- This is a UNIT test - no database or PTY required
- Tests jump logic only, not visual output
- Companion to TC-HORIZ-019 (interactive test for H jump)
- Part of Vim-style navigation (h/l scroll, H/L jump)
