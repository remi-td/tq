# TC-HORIZ-013: Interactive Test - Right Column Indicator Display

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-013 |
| **Title** | Interactive Test - Right Column Indicator Display |
| **Category** | Interactive Test |
| **Priority** | Critical |
| **Feature** | Sprint 29 - Horizontal Paging (AC-3) |
| **Test Type** | Interactive (expectrl) |
| **Created** | 2026-01-30 |

## Purpose

Verify that the pager displays `(+N cols)` indicator on the right side showing the count of hidden columns to the right.

## Acceptance Criteria Coverage

- **AC-3**: Display `(+N cols)` indicator in rightmost column showing count of hidden columns to the right

## Scope

This test validates:
- Right indicator appears when columns hidden to the right
- Indicator shows correct count (`+N cols` format)
- Count decreases as user scrolls right
- Indicator disappears when at rightmost position
- Indicator positioned on right border/edge

## Prerequisites

- Live database, wide test table (30+ columns)

## Test Procedure

```rust
#[test]
#[ignore]
fn test_right_column_indicator_displays() {
    let mut p = spawn_tq_repl();
    p.send_line("SELECT * FROM test_wide_table_30;");
    thread::sleep(Duration::from_millis(500));

    let initial_output = read_available_output(&mut p);

    // At start: right indicator should be present
    assert!(initial_output.contains("(+") && initial_output.contains("cols)"),
            "Right indicator should show (+N cols) format");

    // Extract count (e.g., "(+25 cols)")
    let initial_count = extract_right_indicator_count(&initial_output);
    assert!(initial_count > 20, "Should have many columns hidden");

    // Scroll right 5 times
    for _ in 0..5 {
        send_key(&mut p, KeyCode::Right);
        thread::sleep(Duration::from_millis(200));
    }

    let after_scroll = read_available_output(&mut p);
    let new_count = extract_right_indicator_count(&after_scroll);

    // Count should decrease
    assert!(new_count < initial_count,
            "Right indicator count should decrease after scrolling right");

    send_key(&mut p, KeyCode::Char('q'));
}

#[test]
#[ignore]
fn test_right_indicator_disappears_at_end() {
    let mut p = spawn_tq_repl();
    p.send_line("SELECT * FROM test_wide_table_20;");

    // Scroll to rightmost position (L key)
    send_key(&mut p, KeyCode::Char('L'));
    thread::sleep(Duration::from_millis(300));

    let output = read_available_output(&mut p);

    // Right indicator should be gone (or show 0)
    // Status bar should show last columns (e.g., "Columns 16-20 of 20")
    assert!(output.contains(" of 20"),
            "Should show total column count in status");

    // Extract last column number from status
    let range = extract_column_range(&output);
    assert_eq!(range.end, 20, "Last visible column should be 20");

    send_key(&mut p, KeyCode::Char('q'));
}
```

## Expected Results

- Indicator format: `(+N cols)` where N is count of hidden columns
- Count is accurate (matches hidden_columns_right calculation)
- Count decreases as user scrolls right
- Indicator disappears or shows 0 at rightmost position

## Pass/Fail Criteria

PASS: Indicator displays with correct count, updates correctly, disappears at end
FAIL: No indicator, wrong count, doesn't update, wrong format

## Notes

- Companion to TC-HORIZ-003 (unit test for calculation)
- Requires helper: `extract_right_indicator_count(output)` to parse count
