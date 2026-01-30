# TC-HORIZ-003: Unit Test - Hidden Columns Right Calculation

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-003 |
| **Title** | Unit Test - Hidden Columns Right Calculation |
| **Category** | Unit Test |
| **Priority** | Critical |
| **Feature** | Sprint 29 - Horizontal Paging (AC-3) |
| **Test Type** | Unit |
| **Created** | 2026-01-30 |

## Purpose

Verify that the pager correctly calculates the count of hidden columns to the right of the current viewport.

## Acceptance Criteria Coverage

- **AC-3**: Display `(+N cols)` indicator in rightmost column showing count of hidden columns to the right

## Scope

This test validates:
- `hidden_columns_right()` method returns correct count
- Calculation accounts for current `col_offset`
- Calculation accounts for visible column count
- Edge cases: 0 hidden columns, all columns hidden

## Prerequisites

- Rust test framework available
- Access to `Pager` struct test module in `src/commands/repl/pager.rs`

## Test Procedure

### Test Implementation (in `src/commands/repl/pager.rs`):

```rust
#[test]
fn test_hidden_columns_right_at_start() {
    // 20 columns, terminal fits 5
    let pager = Pager::new(create_test_table(20), 80);

    // At start: showing columns 0-4, hiding 5-19 (15 hidden)
    let hidden = pager.hidden_columns_right();
    assert_eq!(hidden, 15);
}

#[test]
fn test_hidden_columns_right_after_scroll() {
    let mut pager = Pager::new(create_test_table(20), 80);

    // Scroll right 3 times (col_offset = 3)
    for _ in 0..3 {
        pager.handle_key(KeyCode::Right);
    }

    // Now showing columns 3-7, hiding 8-19 (12 hidden)
    let hidden = pager.hidden_columns_right();
    assert_eq!(hidden, 12);
}

#[test]
fn test_hidden_columns_right_at_end() {
    let mut pager = Pager::new(create_test_table(20), 80);

    // Scroll to last position
    while pager.hidden_columns_right() > 0 {
        pager.handle_key(KeyCode::Right);
    }

    // At end: no columns hidden to the right
    assert_eq!(pager.hidden_columns_right(), 0);
}

#[test]
fn test_hidden_columns_right_all_fit() {
    // 5 columns, wide terminal (all fit)
    let pager = Pager::new(create_test_table(5), 200);

    // All columns fit, none hidden
    assert_eq!(pager.hidden_columns_right(), 0);
}

#[test]
fn test_hidden_columns_right_formula() {
    let mut pager = Pager::new(create_test_table(30), 80);

    // Test at various positions
    let test_cases = vec![
        (0, 25),   // Start position
        (5, 20),   // Middle
        (10, 15),  // Further right
        (25, 0),   // Near end
    ];

    for (offset, expected_hidden) in test_cases {
        pager.col_offset = offset;
        assert_eq!(pager.hidden_columns_right(), expected_hidden,
                   "At offset {}, expected {} hidden columns", offset, expected_hidden);
    }
}
```

## Expected Results

All unit tests pass:
- Correct hidden column count at start position
- Count decreases as user scrolls right
- Count reaches 0 at rightmost position
- Count is 0 when all columns fit in terminal
- Formula: `max(0, total_columns - (col_offset + visible_columns))`

## Pass/Fail Criteria

**PASS if:**
- All 5 unit tests compile and pass
- Hidden column counts are mathematically correct
- Edge cases (0, exact fit) handled correctly
- Calculation never returns negative numbers

**FAIL if:**
- Any unit test fails
- Hidden column count is incorrect
- Negative values returned
- Off-by-one errors in calculation

## Notes

- This is a UNIT test - no database or PTY required
- Tests calculation logic only, not visual indicator display
- Companion to TC-HORIZ-013 (interactive test for visual indicator)
- Critical for AC-3 - indicator text depends on this calculation
