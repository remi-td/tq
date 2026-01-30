# TC-HORIZ-005: Unit Test - Status Bar Column Range Text

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-005 |
| **Title** | Unit Test - Status Bar Column Range Text |
| **Category** | Unit Test |
| **Priority** | High |
| **Feature** | Sprint 29 - Horizontal Paging (AC-6) |
| **Test Type** | Unit |
| **Created** | 2026-01-30 |

## Purpose

Verify that the pager generates correct status bar text showing the current column range in the format "Columns X-Y of Z".

## Acceptance Criteria Coverage

- **AC-6**: Status bar shows current column range (e.g., "Columns 3-8 of 32")

## Scope

This test validates:
- `format_column_range()` method generates correct text
- Format matches specification: "Columns X-Y of Z"
- First and last visible column numbers are correct
- Total column count is accurate

## Prerequisites

- Rust test framework available
- Access to `Pager` struct test module in `src/commands/repl/pager.rs`

## Test Procedure

### Test Implementation (in `src/commands/repl/pager.rs`):

```rust
#[test]
fn test_status_bar_column_range_at_start() {
    let pager = Pager::new(create_test_table(23), 80);

    // At start: showing columns 1-5 of 23 (1-indexed for display)
    let status = pager.format_column_range();
    assert_eq!(status, "Columns 1-5 of 23");
}

#[test]
fn test_status_bar_column_range_after_scroll() {
    let mut pager = Pager::new(create_test_table(23), 80);

    // Scroll right 2 times (col_offset = 2)
    for _ in 0..2 {
        pager.handle_key(KeyCode::Right);
    }

    // Now showing columns 3-7 of 23 (1-indexed)
    let status = pager.format_column_range();
    assert_eq!(status, "Columns 3-7 of 23");
}

#[test]
fn test_status_bar_column_range_at_end() {
    let mut pager = Pager::new(create_test_table(23), 80);

    // Scroll to rightmost position
    while pager.hidden_columns_right() > 0 {
        pager.handle_key(KeyCode::Right);
    }

    // Last window: columns 19-23 of 23
    let status = pager.format_column_range();
    assert_eq!(status, "Columns 19-23 of 23");
}

#[test]
fn test_status_bar_all_columns_fit() {
    let pager = Pager::new(create_test_table(5), 200);

    // All 5 columns fit
    let status = pager.format_column_range();
    assert_eq!(status, "Columns 1-5 of 5");
}

#[test]
fn test_status_bar_single_column() {
    let pager = Pager::new(create_test_table(1), 80);

    // Single column
    let status = pager.format_column_range();
    assert_eq!(status, "Columns 1-1 of 1");
}

#[test]
fn test_status_bar_various_positions() {
    let mut pager = Pager::new(create_test_table(50), 80);

    let test_cases = vec![
        (0, "Columns 1-5 of 50"),
        (5, "Columns 6-10 of 50"),
        (10, "Columns 11-15 of 50"),
        (45, "Columns 46-50 of 50"),
    ];

    for (offset, expected) in test_cases {
        pager.col_offset = offset;
        assert_eq!(pager.format_column_range(), expected);
    }
}
```

## Expected Results

All unit tests pass:
- Format matches "Columns X-Y of Z" exactly
- Column numbers are 1-indexed (user-friendly)
- First visible = `col_offset + 1`
- Last visible = `col_offset + visible_column_count`
- Total = total column count

## Pass/Fail Criteria

**PASS if:**
- All 6 unit tests compile and pass
- Status text format is exactly "Columns X-Y of Z"
- Column numbers are mathematically correct
- Single column case works (1-1 of 1)
- All-columns-fit case works

**FAIL if:**
- Any unit test fails
- Format is incorrect (e.g., "Cols" instead of "Columns")
- Column numbers are wrong (0-indexed, off-by-one)
- Total count is wrong

## Notes

- This is a UNIT test - no database or PTY required
- Tests text generation only, not visual display
- Companion to TC-HORIZ-016 (interactive test for status bar display)
- Critical for AC-6 - users need clear column position feedback
- Remember: Display uses 1-indexed (user-friendly), internal uses 0-indexed
