# TC-HORIZ-006: Unit Test - Vim h/l Key Handling

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-006 |
| **Title** | Unit Test - Vim h/l Key Handling |
| **Category** | Unit Test |
| **Priority** | High |
| **Feature** | Sprint 29 - Horizontal Paging (AC-8) |
| **Test Type** | Unit |
| **Created** | 2026-01-30 |

## Purpose

Verify that the pager treats Vim-style 'h' and 'l' keys identically to left and right arrow keys for horizontal navigation.

## Acceptance Criteria Coverage

- **AC-8**: Vim-style `h`/`l` keys work for horizontal navigation (alongside arrow keys)

## Scope

This test validates:
- `handle_key()` recognizes 'h' as equivalent to Left arrow
- `handle_key()` recognizes 'l' as equivalent to Right arrow
- 'h' and 'l' produce identical state changes to arrow keys
- Vim keys and arrow keys can be used interchangeably

## Prerequisites

- Rust test framework available
- Access to `Pager` struct test module in `src/commands/repl/pager.rs`

## Test Procedure

### Test Implementation (in `src/commands/repl/pager.rs`):

```rust
#[test]
fn test_vim_l_key_scrolls_right() {
    let mut pager = Pager::new(create_test_table(20), 80);
    assert_eq!(pager.col_offset, 0);

    // Press 'l' key
    pager.handle_key(KeyCode::Char('l'));

    // Should scroll right like Right arrow
    assert_eq!(pager.col_offset, 1);
}

#[test]
fn test_vim_h_key_scrolls_left() {
    let mut pager = Pager::new(create_test_table(20), 80);
    pager.col_offset = 5;

    // Press 'h' key
    pager.handle_key(KeyCode::Char('h'));

    // Should scroll left like Left arrow
    assert_eq!(pager.col_offset, 4);
}

#[test]
fn test_vim_keys_equivalent_to_arrows() {
    let mut pager1 = Pager::new(create_test_table(20), 80);
    let mut pager2 = Pager::new(create_test_table(20), 80);

    // pager1: Use arrow keys
    pager1.handle_key(KeyCode::Right);
    pager1.handle_key(KeyCode::Right);
    pager1.handle_key(KeyCode::Left);

    // pager2: Use vim keys
    pager2.handle_key(KeyCode::Char('l'));
    pager2.handle_key(KeyCode::Char('l'));
    pager2.handle_key(KeyCode::Char('h'));

    // Both should have same col_offset
    assert_eq!(pager1.col_offset, pager2.col_offset);
}

#[test]
fn test_vim_keys_and_arrows_interchangeable() {
    let mut pager = Pager::new(create_test_table(30), 80);

    // Mix vim keys and arrow keys
    pager.handle_key(KeyCode::Right);        // offset = 1
    pager.handle_key(KeyCode::Char('l'));    // offset = 2
    pager.handle_key(KeyCode::Right);        // offset = 3
    pager.handle_key(KeyCode::Char('h'));    // offset = 2
    pager.handle_key(KeyCode::Left);         // offset = 1
    pager.handle_key(KeyCode::Char('l'));    // offset = 2

    assert_eq!(pager.col_offset, 2);
}

#[test]
fn test_vim_h_at_start_no_effect() {
    let mut pager = Pager::new(create_test_table(20), 80);
    assert_eq!(pager.col_offset, 0);

    // Press 'h' at start position
    pager.handle_key(KeyCode::Char('h'));

    // Should not go negative (same as Left arrow behavior)
    assert_eq!(pager.col_offset, 0);
}

#[test]
fn test_vim_l_at_end_no_effect() {
    let mut pager = Pager::new(create_test_table(10), 80);

    // Scroll to rightmost position
    while pager.hidden_columns_right() > 0 {
        pager.handle_key(KeyCode::Right);
    }

    let last_offset = pager.col_offset;

    // Press 'l' at end position
    pager.handle_key(KeyCode::Char('l'));

    // Should not scroll further (same as Right arrow behavior)
    assert_eq!(pager.col_offset, last_offset);
}
```

## Expected Results

All unit tests pass:
- 'l' key scrolls right (increments col_offset)
- 'h' key scrolls left (decrements col_offset)
- Vim keys produce identical results to arrow keys
- Vim keys respect same bounds as arrow keys
- Keys can be mixed freely

## Pass/Fail Criteria

**PASS if:**
- All 6 unit tests compile and pass
- 'h' and Left arrow produce identical state
- 'l' and Right arrow produce identical state
- Bounds checking works same for both key types
- Keys can be used interchangeably

**FAIL if:**
- Any unit test fails
- Vim keys behave differently from arrow keys
- Bounds checking differs between key types
- Case sensitivity issues ('H' vs 'h' for navigation)

## Notes

- This is a UNIT test - no database or PTY required
- Tests key handling logic only, not terminal key capture
- Companion to TC-HORIZ-018 (interactive test for Vim keys)
- Note: 'H' and 'L' (uppercase) are for JUMPS (AC-9, AC-10), tested separately
- This test covers lowercase 'h' and 'l' for scrolling (AC-8)
