# TC-HORIZ-004: Unit Test - Hidden Columns Left Calculation

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-004 |
| **Title** | Unit Test - Hidden Columns Left Calculation |
| **Category** | Unit Test |
| **Priority** | Critical |
| **Feature** | Sprint 29 - Horizontal Paging (AC-4) |
| **Test Type** | Unit |
| **Created** | 2026-01-30 |

## Purpose

Verify that the pager correctly calculates the count of hidden columns to the left of the current viewport.

## Acceptance Criteria Coverage

- **AC-4**: Display `(+N cols)` indicator in leftmost column showing count of hidden columns to the left

## Scope

This test validates:
- `hidden_columns_left()` method returns correct count
- Calculation based on current `col_offset`
- Edge cases: 0 at start, maximum when scrolled far right

## Prerequisites

- Rust test framework available
- Access to `Pager` struct test module in `src/commands/repl/pager.rs`

## Test Procedure

### Test Implementation (in `src/commands/repl/pager.rs`):

```rust
#[test]
fn test_hidden_columns_left_at_start() {
    let pager = Pager::new(create_test_table(20), 80);

    // At start: col_offset=0, no columns hidden to left
    assert_eq!(pager.hidden_columns_left(), 0);
}

#[test]
fn test_hidden_columns_left_after_scroll() {
    let mut pager = Pager::new(create_test_table(20), 80);

    // Scroll right 5 times (col_offset = 5)
    for _ in 0..5 {
        pager.handle_key(KeyCode::Right);
    }

    // Now columns 0-4 are hidden to the left
    assert_eq!(pager.hidden_columns_left(), 5);
}

#[test]
fn test_hidden_columns_left_maximum() {
    let mut pager = Pager::new(create_test_table(30), 80);

    // Scroll right 20 times
    for _ in 0..20 {
        pager.handle_key(KeyCode::Right);
    }

    // 20 columns hidden to left
    assert_eq!(pager.hidden_columns_left(), 20);
}

#[test]
fn test_hidden_columns_left_formula() {
    let mut pager = Pager::new(create_test_table(25), 80);

    // Test at various positions
    let test_cases = vec![
        (0, 0),    // Start
        (1, 1),    // One scroll right
        (10, 10),  // Middle
        (20, 20),  // Far right
    ];

    for (offset, expected_hidden) in test_cases {
        pager.col_offset = offset;
        assert_eq!(pager.hidden_columns_left(), expected_hidden,
                   "At offset {}, expected {} hidden columns left", offset, expected_hidden);
    }
}

#[test]
fn test_hidden_columns_left_right_consistency() {
    let mut pager = Pager::new(create_test_table(20), 80);
    let visible = pager.visible_column_count();

    // At any position: hidden_left + visible + hidden_right = total
    for _ in 0..10 {
        let left = pager.hidden_columns_left();
        let right = pager.hidden_columns_right();
        let total = left + visible + right;

        assert_eq!(total, 20,
                   "Sum of hidden_left ({}) + visible ({}) + hidden_right ({}) must equal total (20)",
                   left, visible, right);

        pager.handle_key(KeyCode::Right);
    }
}
```

## Expected Results

All unit tests pass:
- Hidden left count is 0 at start position
- Count equals `col_offset` at any position
- Count increases as user scrolls right
- Formula: `hidden_columns_left() = col_offset`
- Consistency: `hidden_left + visible + hidden_right = total_columns`

## Pass/Fail Criteria

**PASS if:**
- All 5 unit tests compile and pass
- Hidden left count equals col_offset
- Consistency check passes (sum equals total)
- No negative values returned

**FAIL if:**
- Any unit test fails
- Hidden left count incorrect
- Consistency check fails
- Calculation error

## Notes

- This is a UNIT test - no database or PTY required
- Tests calculation logic only, not visual indicator display
- Companion to TC-HORIZ-014 (interactive test for visual indicator)
- Simpler than hidden_right: just returns col_offset
