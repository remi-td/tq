# TC-HORIZ-010: Unit Test - Visible Column Count Calculation

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-010 |
| **Title** | Unit Test - Visible Column Count Calculation |
| **Category** | Unit Test |
| **Priority** | Critical |
| **Feature** | Sprint 29 - Horizontal Paging |
| **Test Type** | Unit |
| **Created** | 2026-01-30 |

## Purpose

Verify that the pager correctly calculates how many columns can fit in the current terminal width, accounting for borders, padding, and column indicators.

## Acceptance Criteria Coverage

- Foundation for AC-1 through AC-4 (scrolling and indicators)
- Foundation for AC-6 (status bar column range)

## Scope

This test validates:
- `visible_column_count()` method returns correct count
- Calculation accounts for terminal width
- Calculation accounts for column widths
- Calculation accounts for border characters and padding
- Calculation accounts for left/right indicator space
- Edge cases: very narrow terminal, very wide terminal

## Prerequisites

- Rust test framework available
- Access to `Pager` struct test module in `src/commands/repl/pager.rs`

## Test Procedure

### Test Implementation (in `src/commands/repl/pager.rs`):

```rust
#[test]
fn test_visible_column_count_standard_terminal() {
    // 80-char terminal, columns of ~15 chars each
    let pager = Pager::new(create_test_table(20), 80);

    let visible = pager.visible_column_count();

    // Should fit approximately 4-5 columns
    assert!(visible >= 4 && visible <= 6,
            "80-char terminal should show 4-6 columns, got {}", visible);
}

#[test]
fn test_visible_column_count_wide_terminal() {
    // 200-char terminal
    let pager = Pager::new(create_test_table(20), 200);

    let visible = pager.visible_column_count();

    // Should fit more columns (10-12)
    assert!(visible >= 10 && visible <= 15,
            "200-char terminal should show 10-15 columns, got {}", visible);
}

#[test]
fn test_visible_column_count_narrow_terminal() {
    // 40-char terminal (very narrow)
    let pager = Pager::new(create_test_table(20), 40);

    let visible = pager.visible_column_count();

    // Should fit at least 1 column
    assert!(visible >= 1,
            "Even narrow terminal should show at least 1 column, got {}", visible);
}

#[test]
fn test_visible_column_count_all_fit() {
    // Wide terminal, few columns
    let pager = Pager::new(create_test_table(3), 200);

    let visible = pager.visible_column_count();

    // All 3 columns should fit
    assert_eq!(visible, 3, "All columns should be visible");
}

#[test]
fn test_visible_column_count_single_column() {
    let pager = Pager::new(create_test_table(1), 80);

    let visible = pager.visible_column_count();

    // Single column always visible
    assert_eq!(visible, 1);
}

#[test]
fn test_visible_column_count_at_different_offsets() {
    let mut pager = Pager::new(create_test_table(30), 80);

    let visible_at_start = pager.visible_column_count();

    // Scroll right
    pager.col_offset = 10;

    let visible_after_scroll = pager.visible_column_count();

    // visible_column_count should be same regardless of col_offset
    assert_eq!(visible_at_start, visible_after_scroll,
               "Visible column count should not change with col_offset");
}

#[test]
fn test_visible_column_count_consistency() {
    let pager = Pager::new(create_test_table(25), 80);

    let visible = pager.visible_column_count();
    let hidden_left = pager.hidden_columns_left();
    let hidden_right = pager.hidden_columns_right();

    // Sum should equal total columns
    assert_eq!(hidden_left + visible + hidden_right, 25,
               "hidden_left + visible + hidden_right must equal total columns");
}

#[test]
fn test_visible_column_count_with_indicator_space() {
    let mut pager = Pager::new(create_test_table(30), 80);

    // At start: no left indicator, has right indicator
    let visible_at_start = pager.visible_column_count();

    // Scroll to middle: both left and right indicators
    pager.col_offset = 10;
    let visible_in_middle = pager.visible_column_count();

    // visible count might be slightly less in middle due to both indicators
    // But implementation may reserve space regardless, so could be equal
    assert!(visible_in_middle <= visible_at_start + 1,
            "Visible count in middle should be similar to start");
}
```

## Expected Results

All unit tests pass:
- Correct column count for standard 80-char terminal (4-6 columns)
- More columns visible in wide terminal (10-15 for 200 chars)
- At least 1 column in narrow terminal (40 chars)
- All columns visible when they fit
- Single column case works
- Visible count independent of col_offset
- Consistency: hidden_left + visible + hidden_right = total

## Pass/Fail Criteria

**PASS if:**
- All 8 unit tests compile and pass
- Visible count is reasonable for terminal width
- At least 1 column always visible
- Consistency check passes
- Calculation accounts for indicators

**FAIL if:**
- Any unit test fails
- Visible count is 0
- Visible count exceeds total columns
- Consistency check fails
- Calculation doesn't account for borders/padding

## Notes

- This is a UNIT test - no database or PTY required
- Tests calculation logic only
- Critical foundation for all scrolling and indicator features
- Calculation must account for:
  - Table borders (│ characters)
  - Column padding (spaces)
  - Indicator text space (`(+N cols)`)
  - Terminal width limits
