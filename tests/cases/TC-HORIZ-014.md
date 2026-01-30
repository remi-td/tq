# TC-HORIZ-014: Interactive Test - Left Column Indicator Display

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-014 |
| **Title** | Interactive Test - Left Column Indicator Display |
| **Category** | Interactive Test |
| **Priority** | Critical |
| **Feature** | Sprint 29 - Horizontal Paging (AC-4) |
| **Test Type** | Interactive (expectrl) |
| **Created** | 2026-01-30 |

## Purpose

Verify that the pager displays `(+N cols)` indicator on the left side showing the count of hidden columns to the left when scrolled right.

## Acceptance Criteria Coverage

- **AC-4**: Display `(+N cols)` indicator in leftmost column showing count of hidden columns to the left

## Scope

This test validates:
- Left indicator appears after scrolling right
- Indicator shows correct count
- Count increases as user scrolls right
- Indicator not present at start position
- Format matches: `(+N cols) ←` or similar

## Prerequisites

- Live database, wide test table

## Test Procedure

```rust
#[test]
#[ignore]
fn test_left_column_indicator_displays() {
    let mut p = spawn_tq_repl();
    p.send_line("SELECT * FROM test_wide_table_30;");
    thread::sleep(Duration::from_millis(500));

    let initial_output = read_available_output(&mut p);

    // At start: no left indicator
    let has_left_indicator_at_start =
        initial_output.contains("← ") || initial_output.contains("(+1 cols)");
    assert!(!has_left_indicator_at_start,
            "Left indicator should NOT appear at start position");

    // Scroll right 5 times
    for _ in 0..5 {
        send_key(&mut p, KeyCode::Right);
        thread::sleep(Duration::from_millis(200));
    }

    let after_scroll = read_available_output(&mut p);

    // Now left indicator should appear
    assert!(after_scroll.contains("(+") && after_scroll.contains("cols"),
            "Left indicator should appear after scrolling right");

    let left_count = extract_left_indicator_count(&after_scroll);
    assert_eq!(left_count, 5, "Left indicator should show 5 hidden columns");

    send_key(&mut p, KeyCode::Char('q'));
}

#[test]
#[ignore]
fn test_both_indicators_in_middle_position() {
    let mut p = spawn_tq_repl();
    p.send_line("SELECT * FROM test_wide_table_40;");

    // Scroll to middle (20 columns)
    for _ in 0..20 {
        send_key(&mut p, KeyCode::Right);
        thread::sleep(Duration::from_millis(100));
    }

    let output = read_available_output(&mut p);

    // Both indicators should be present
    assert!(output.contains("←"), "Left indicator arrow should be present");
    assert!(output.contains("→"), "Right indicator arrow should be present");

    // Extract both counts
    let left_count = extract_left_indicator_count(&output);
    let right_count = extract_right_indicator_count(&output);

    assert!(left_count > 0, "Left indicator should show hidden columns");
    assert!(right_count > 0, "Right indicator should show hidden columns");

    send_key(&mut p, KeyCode::Char('q'));
}
```

## Expected Results

- No left indicator at start
- Left indicator appears after scrolling right
- Count equals number of scrolls
- Both indicators present in middle position

## Pass/Fail Criteria

PASS: Indicator appears correctly, count accurate, format correct
FAIL: Indicator at start, wrong count, doesn't appear after scroll

## Notes

- Companion to TC-HORIZ-004 (unit test)
- Validates both indicators can coexist
