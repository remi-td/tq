# Sprint 29: Unit Tests to Implement in Code

**Location:** `src/commands/repl/pager.rs` test module
**Purpose:** Guide for rust-teradata-architect to implement unit tests

---

## Unit Test Implementation Guide

### Module Structure

```rust
// In src/commands/repl/pager.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    // Test helper functions
    fn create_test_table(columns: usize) -> TableData {
        // Create mock table with specified column count
        // Each column: col_1, col_2, ..., col_N
        // 10 rows of test data
    }

    fn create_test_table_wide_tall(columns: usize, rows: usize) -> TableData {
        // Create mock table with specified dimensions
    }

    // Unit tests below...
}
```

---

## Test Group 1: Column Offset Navigation (AC-1, AC-2)

### From TC-HORIZ-001 (Right Arrow)

```rust
#[test]
fn test_right_arrow_increments_col_offset() {
    let mut pager = Pager::new(create_test_table(20), 80);
    assert_eq!(pager.col_offset, 0);

    pager.handle_key(KeyCode::Right);

    assert_eq!(pager.col_offset, 1);
}

#[test]
fn test_right_arrow_multiple_presses() {
    let mut pager = Pager::new(create_test_table(20), 80);

    for _ in 0..5 {
        pager.handle_key(KeyCode::Right);
    }

    assert_eq!(pager.col_offset, 5);
}

#[test]
fn test_right_arrow_at_last_position_no_effect() {
    let mut pager = Pager::new(create_test_table(10), 80);

    // Scroll to last valid position
    while pager.hidden_columns_right() > 0 {
        pager.handle_key(KeyCode::Right);
    }

    let last_offset = pager.col_offset;

    pager.handle_key(KeyCode::Right);

    assert_eq!(pager.col_offset, last_offset);
}

#[test]
fn test_right_arrow_bounds_checking() {
    let mut pager = Pager::new(create_test_table(5), 200); // Wide terminal, few columns

    pager.handle_key(KeyCode::Right);

    assert_eq!(pager.col_offset, 0); // No scrolling needed
}
```

### From TC-HORIZ-002 (Left Arrow)

```rust
#[test]
fn test_left_arrow_decrements_col_offset() {
    let mut pager = Pager::new(create_test_table(20), 80);

    pager.col_offset = 5;

    pager.handle_key(KeyCode::Left);

    assert_eq!(pager.col_offset, 4);
}

#[test]
fn test_left_arrow_multiple_presses() {
    let mut pager = Pager::new(create_test_table(20), 80);

    pager.col_offset = 10;

    for _ in 0..5 {
        pager.handle_key(KeyCode::Left);
    }

    assert_eq!(pager.col_offset, 5);
}

#[test]
fn test_left_arrow_at_first_position_no_effect() {
    let mut pager = Pager::new(create_test_table(20), 80);

    assert_eq!(pager.col_offset, 0);

    pager.handle_key(KeyCode::Left);

    assert_eq!(pager.col_offset, 0);
}

#[test]
fn test_left_arrow_return_to_start() {
    let mut pager = Pager::new(create_test_table(20), 80);

    // Scroll right 10 times
    for _ in 0..10 {
        pager.handle_key(KeyCode::Right);
    }

    // Scroll left 10 times
    for _ in 0..10 {
        pager.handle_key(KeyCode::Left);
    }

    assert_eq!(pager.col_offset, 0);
}
```

---

## Test Group 2: Hidden Column Calculations (AC-3, AC-4)

### From TC-HORIZ-003 (Hidden Right)

```rust
#[test]
fn test_hidden_columns_right_at_start() {
    let pager = Pager::new(create_test_table(20), 80);

    let hidden = pager.hidden_columns_right();
    assert!(hidden > 10, "Should have many columns hidden at start");
}

#[test]
fn test_hidden_columns_right_after_scroll() {
    let mut pager = Pager::new(create_test_table(20), 80);

    let initial_hidden = pager.hidden_columns_right();

    for _ in 0..3 {
        pager.handle_key(KeyCode::Right);
    }

    let new_hidden = pager.hidden_columns_right();
    assert_eq!(new_hidden, initial_hidden - 3);
}

#[test]
fn test_hidden_columns_right_at_end() {
    let mut pager = Pager::new(create_test_table(20), 80);

    while pager.hidden_columns_right() > 0 {
        pager.handle_key(KeyCode::Right);
    }

    assert_eq!(pager.hidden_columns_right(), 0);
}

#[test]
fn test_hidden_columns_right_all_fit() {
    let pager = Pager::new(create_test_table(5), 200);

    assert_eq!(pager.hidden_columns_right(), 0);
}
```

### From TC-HORIZ-004 (Hidden Left)

```rust
#[test]
fn test_hidden_columns_left_at_start() {
    let pager = Pager::new(create_test_table(20), 80);

    assert_eq!(pager.hidden_columns_left(), 0);
}

#[test]
fn test_hidden_columns_left_after_scroll() {
    let mut pager = Pager::new(create_test_table(20), 80);

    for _ in 0..5 {
        pager.handle_key(KeyCode::Right);
    }

    assert_eq!(pager.hidden_columns_left(), 5);
}

#[test]
fn test_hidden_columns_left_formula() {
    let mut pager = Pager::new(create_test_table(25), 80);

    let test_cases = vec![
        (0, 0),
        (1, 1),
        (10, 10),
        (20, 20),
    ];

    for (offset, expected_hidden) in test_cases {
        pager.col_offset = offset;
        assert_eq!(pager.hidden_columns_left(), expected_hidden);
    }
}

#[test]
fn test_hidden_columns_left_right_consistency() {
    let mut pager = Pager::new(create_test_table(20), 80);
    let visible = pager.visible_column_count();

    for _ in 0..10 {
        let left = pager.hidden_columns_left();
        let right = pager.hidden_columns_right();
        let total = left + visible + right;

        assert_eq!(total, 20);

        pager.handle_key(KeyCode::Right);
    }
}
```

---

## Test Group 3: Status Bar and Display (AC-6)

### From TC-HORIZ-005 (Status Bar Text)

```rust
#[test]
fn test_status_bar_column_range_at_start() {
    let pager = Pager::new(create_test_table(23), 80);

    let status = pager.format_column_range();
    assert!(status.starts_with("Columns 1-"));
    assert!(status.ends_with(" of 23"));
}

#[test]
fn test_status_bar_column_range_after_scroll() {
    let mut pager = Pager::new(create_test_table(23), 80);

    for _ in 0..2 {
        pager.handle_key(KeyCode::Right);
    }

    let status = pager.format_column_range();
    assert!(status.starts_with("Columns 3-"));
}

#[test]
fn test_status_bar_all_columns_fit() {
    let pager = Pager::new(create_test_table(5), 200);

    let status = pager.format_column_range();
    assert_eq!(status, "Columns 1-5 of 5");
}

#[test]
fn test_status_bar_single_column() {
    let pager = Pager::new(create_test_table(1), 80);

    let status = pager.format_column_range();
    assert_eq!(status, "Columns 1-1 of 1");
}
```

---

## Test Group 4: Vim Key Bindings (AC-8, AC-9, AC-10)

### From TC-HORIZ-006 (Vim h/l)

```rust
#[test]
fn test_vim_l_key_scrolls_right() {
    let mut pager = Pager::new(create_test_table(20), 80);
    assert_eq!(pager.col_offset, 0);

    pager.handle_key(KeyCode::Char('l'));

    assert_eq!(pager.col_offset, 1);
}

#[test]
fn test_vim_h_key_scrolls_left() {
    let mut pager = Pager::new(create_test_table(20), 80);
    pager.col_offset = 5;

    pager.handle_key(KeyCode::Char('h'));

    assert_eq!(pager.col_offset, 4);
}

#[test]
fn test_vim_keys_equivalent_to_arrows() {
    let mut pager1 = Pager::new(create_test_table(20), 80);
    let mut pager2 = Pager::new(create_test_table(20), 80);

    pager1.handle_key(KeyCode::Right);
    pager1.handle_key(KeyCode::Right);
    pager1.handle_key(KeyCode::Left);

    pager2.handle_key(KeyCode::Char('l'));
    pager2.handle_key(KeyCode::Char('l'));
    pager2.handle_key(KeyCode::Char('h'));

    assert_eq!(pager1.col_offset, pager2.col_offset);
}
```

### From TC-HORIZ-007 (H jump)

```rust
#[test]
fn test_uppercase_h_jumps_to_first_column() {
    let mut pager = Pager::new(create_test_table(30), 80);

    pager.col_offset = 15;

    pager.handle_key(KeyCode::Char('H'));

    assert_eq!(pager.col_offset, 0);
}

#[test]
fn test_lowercase_h_vs_uppercase_h() {
    let mut pager1 = Pager::new(create_test_table(30), 80);
    let mut pager2 = Pager::new(create_test_table(30), 80);

    pager1.col_offset = 10;
    pager2.col_offset = 10;

    pager1.handle_key(KeyCode::Char('h')); // scroll left by 1
    assert_eq!(pager1.col_offset, 9);

    pager2.handle_key(KeyCode::Char('H')); // jump to 0
    assert_eq!(pager2.col_offset, 0);
}
```

### From TC-HORIZ-008 (L jump)

```rust
#[test]
fn test_uppercase_l_jumps_to_last_column() {
    let mut pager = Pager::new(create_test_table(30), 80);

    assert_eq!(pager.col_offset, 0);

    pager.handle_key(KeyCode::Char('L'));

    assert_eq!(pager.hidden_columns_right(), 0);
    assert!(pager.col_offset > 0);
}

#[test]
fn test_uppercase_l_calculation() {
    let mut pager = Pager::new(create_test_table(25), 80);

    pager.handle_key(KeyCode::Char('L'));

    let visible = pager.visible_column_count();
    let expected_offset = 25 - visible;

    assert_eq!(pager.col_offset, expected_offset);
}
```

---

## Test Group 5: Column Position Preservation (AC-11)

### From TC-HORIZ-009

```rust
#[test]
fn test_col_offset_preserved_vertical_scroll_down() {
    let mut pager = Pager::new(create_test_table_wide_tall(30, 100), 80);

    pager.col_offset = 10;

    pager.handle_key(KeyCode::Char('j'));

    assert_eq!(pager.col_offset, 10);
}

#[test]
fn test_col_offset_preserved_page_down() {
    let mut pager = Pager::new(create_test_table_wide_tall(30, 100), 80);

    pager.col_offset = 8;

    pager.handle_key(KeyCode::Char(' '));

    assert_eq!(pager.col_offset, 8);
}

#[test]
fn test_col_offset_preserved_complex_sequence() {
    let mut pager = Pager::new(create_test_table_wide_tall(30, 100), 80);

    pager.col_offset = 10;

    // Complex vertical navigation
    pager.handle_key(KeyCode::Char('j'));
    pager.handle_key(KeyCode::Char('j'));
    pager.handle_key(KeyCode::Char('k'));
    pager.handle_key(KeyCode::Char(' '));
    pager.handle_key(KeyCode::Char('b'));

    assert_eq!(pager.col_offset, 10);
}
```

---

## Test Group 6: Visible Column Count (Foundation)

### From TC-HORIZ-010

```rust
#[test]
fn test_visible_column_count_standard_terminal() {
    let pager = Pager::new(create_test_table(20), 80);

    let visible = pager.visible_column_count();

    assert!(visible >= 3 && visible <= 8);
}

#[test]
fn test_visible_column_count_all_fit() {
    let pager = Pager::new(create_test_table(3), 200);

    let visible = pager.visible_column_count();

    assert_eq!(visible, 3);
}

#[test]
fn test_visible_column_count_consistency() {
    let pager = Pager::new(create_test_table(25), 80);

    let visible = pager.visible_column_count();
    let hidden_left = pager.hidden_columns_left();
    let hidden_right = pager.hidden_columns_right();

    assert_eq!(hidden_left + visible + hidden_right, 25);
}
```

---

## Test Group 7: Edge Cases

### Single Column (TC-EDGE-001)

```rust
#[test]
fn test_single_column_no_horizontal_scroll() {
    let mut pager = Pager::new(create_test_table(1), 80);

    pager.handle_key(KeyCode::Right);
    assert_eq!(pager.col_offset, 0);

    assert_eq!(pager.hidden_columns_right(), 0);
    assert_eq!(pager.hidden_columns_left(), 0);
}
```

### Exact Fit (TC-EDGE-002)

```rust
#[test]
fn test_exact_fit_no_scrolling() {
    let pager = Pager::new(create_test_table(5), 200);

    assert_eq!(pager.visible_column_count(), 5);
    assert_eq!(pager.hidden_columns_right(), 0);
    assert_eq!(pager.hidden_columns_left(), 0);
}
```

### Large Column Count (TC-EDGE-003)

```rust
#[test]
fn test_large_column_count() {
    let pager = Pager::new(create_test_table(100), 80);

    assert!(pager.visible_column_count() >= 1);

    let total = pager.hidden_columns_left() +
                pager.visible_column_count() +
                pager.hidden_columns_right();
    assert_eq!(total, 100);
}
```

---

## Summary

**Total Unit Tests to Implement:** 25+

**Implementation Checklist:**

- [ ] Test helper functions: `create_test_table()`, `create_test_table_wide_tall()`
- [ ] Group 1: Right/Left arrow navigation (8 tests)
- [ ] Group 2: Hidden column calculations (8 tests)
- [ ] Group 3: Status bar text (4 tests)
- [ ] Group 4: Vim keybindings (7 tests)
- [ ] Group 5: Column position preservation (3 tests)
- [ ] Group 6: Visible column count (3 tests)
- [ ] Group 7: Edge cases (3 tests)

**Run tests:**
```bash
cargo test --lib pager
```

**Expected Result:** 100% pass rate

---

## Notes for rust-teradata-architect

1. All unit tests go in `src/commands/repl/pager.rs` test module
2. Create helper functions first (`create_test_table`, etc.)
3. Tests should run without database or PTY (pure logic testing)
4. Use `assert_eq!`, `assert!`, `assert_ne!` for validation
5. Each test should be independent (no shared state)
6. Mock `TableData` with known column counts
7. Mock terminal width for `visible_column_count` tests
8. Tests validate internal state (`col_offset`, `hidden_columns_*`)
9. DO NOT test visual output in unit tests (that's for interactive tests)
10. Focus on correctness, bounds checking, edge cases
