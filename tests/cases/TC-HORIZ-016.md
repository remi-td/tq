# TC-HORIZ-016: Interactive Test - Status Bar Column Range Display

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-HORIZ-016 |
| **Title** | Interactive Test - Status Bar Column Range Display |
| **Category** | Interactive Test |
| **Priority** | High |
| **Feature** | Sprint 29 - Horizontal Paging (AC-6) |
| **Test Type** | Interactive (expectrl) |
| **Created** | 2026-01-30 |

## Purpose

Verify that the status bar displays the current column range in the format "Columns X-Y of Z" and updates correctly during navigation.

## Acceptance Criteria Coverage

- **AC-6**: Status bar shows current column range (e.g., "Columns 3-8 of 32")

## Scope

This test validates:
- Status bar visible in pager mode
- Format matches: "Columns X-Y of Z"
- Range accurate (first visible to last visible)
- Total count accurate
- Updates in real-time during navigation
- Integrates with row position display

## Test Procedure

```rust
#[test]
#[ignore]
fn test_status_bar_shows_column_range() {
    let mut p = spawn_tq_repl();
    p.send_line("SELECT * FROM test_wide_table_32;");
    thread::sleep(Duration::from_millis(500));

    let output = read_available_output(&mut p);

    // Verify status bar format
    assert!(output.contains("Columns "), "Status bar should show 'Columns'");
    assert!(output.contains(" of 32"), "Status bar should show total column count");

    // Extract range
    let range = extract_column_range(&output);
    assert_eq!(range.start, 1, "Should start at column 1");
    assert_eq!(range.total, 32, "Should show 32 total columns");
    assert!(range.end >= 4 && range.end <= 8,
            "Should show approximately 4-8 columns initially");

    send_key(&mut p, KeyCode::Char('q'));
}

#[test]
#[ignore]
fn test_status_bar_updates_during_navigation() {
    let mut p = spawn_tq_repl();
    p.send_line("SELECT * FROM test_wide_table_30;");
    thread::sleep(Duration::from_millis(500));

    let initial = read_available_output(&mut p);
    let initial_range = extract_column_range(&initial);

    // Scroll right 3 times
    for _ in 0..3 {
        send_key(&mut p, KeyCode::Right);
        thread::sleep(Duration::from_millis(200));
    }

    let after_scroll = read_available_output(&mut p);
    let new_range = extract_column_range(&after_scroll);

    // Range should shift right
    assert_eq!(new_range.start, initial_range.start + 3,
               "Column range should shift by 3");
    assert_eq!(new_range.total, 30, "Total should remain 30");

    send_key(&mut p, KeyCode::Char('q'));
}

#[test]
#[ignore]
fn test_status_bar_shows_both_row_and_column() {
    let mut p = spawn_tq_repl();
    p.send_line("SELECT * FROM test_wide_tall_table;"); // Wide and tall
    thread::sleep(Duration::from_millis(500));

    let output = read_available_output(&mut p);

    // Should show both: "Columns 1-5 of 30 | Rows 1-20 of 100"
    assert!(output.contains("Columns "), "Should show column range");
    assert!(output.contains("Rows "), "Should show row range");
    assert!(output.contains(" | ") || output.contains("│"),
            "Should separate column and row info");

    send_key(&mut p, KeyCode::Char('q'));
}
```

## Expected Results

- Status bar format: "Columns X-Y of Z"
- X = first visible column (1-indexed)
- Y = last visible column
- Z = total column count
- Updates in real-time during scroll
- Integrates with row position

## Pass/Fail Criteria

PASS: Status bar visible, format correct, updates accurately
FAIL: No status bar, wrong format, doesn't update, wrong numbers

## Notes

- Companion to TC-HORIZ-005 (unit test for text generation)
- Status bar critical for user orientation
- Should integrate with existing row position display
