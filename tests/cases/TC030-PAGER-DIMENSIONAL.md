# TC030 - Pager Dimensional Validation Tests

**Sprint:** 30
**Feature:** Pager architectural refactor with dimensional validation
**Created:** 2026-02-03
**Status:** In Progress

## Overview

Sprint 30 refactors the pager to accept `QueryResult` instead of pre-formatted strings, enabling proper column-level width control. These tests validate dimensional correctness that Sprint 29 tests missed.

## Test Infrastructure

**Tools Used:**
- `tests/tools/visual_validator.rs` - Dimensional assertion utilities
- `tests/tools/terminal_simulator.rs` - Terminal width simulation
- PTY testing via `expectrl` for interactive pager tests

**Test Data:**
- DBC.TablesV (30+ columns) - Standard wide table for pager testing
- Custom `QueryResult` fixtures for controlled scenarios

## Test Cases

### TC030-001: Column Width Calculation
**Objective:** Validate that pager calculates column widths correctly to fit within terminal constraints.

**Test Approach:**
```rust
#[test]
fn test_pager_column_widths_fit_terminal_117() {
    // Create QueryResult with 30 columns (simulating DBC.TablesV)
    let result = create_wide_query_result(30);
    let terminal_width = 117; // User-reported terminal from Sprint 29 issue

    // Pager should calculate which columns fit
    let pager = Pager::new(result, terminal_width)?;

    // Render first page
    let rendered = pager.render_page(0);

    // Validate: No line exceeds terminal width
    assert_no_overflow(&rendered, terminal_width);

    // Validate: Column calculations are correct
    assert_column_widths_within_terminal(&rendered, terminal_width);
}
```

**Acceptance Criteria:**
- No output line exceeds 117 characters
- Column width calculations respect terminal constraints
- Truncation indicators appear when needed

---

### TC030-002: Dynamic Width Adjustment
**Objective:** Validate pager adjusts column selection based on terminal width.

**Test Approach:**
```rust
#[test]
fn test_pager_adapts_to_different_terminal_widths() {
    let result = create_wide_query_result(30);

    // Test at various terminal widths
    for width in [80, 117, 120, 160] {
        let pager = Pager::new(result.clone(), width)?;
        let rendered = pager.render_page(0);

        assert_no_overflow(&rendered, width);

        // Wider terminals should show more columns
        let visible_cols = count_visible_columns(&rendered);
        println!("Width {}: {} columns visible", width, visible_cols);
    }
}
```

**Acceptance Criteria:**
- Pager adapts to 80, 117, 120, 160 character terminals
- Wider terminals show more columns
- All output respects terminal width constraints

---

### TC030-003: Truncation Threshold Detection
**Objective:** Validate truncation indicators appear exactly when columns don't fit.

**Test Approach:**
```rust
#[test]
fn test_truncation_markers_appear_when_needed() {
    let result = create_wide_query_result(30); // Many columns
    let narrow_terminal = 80;

    let pager = Pager::new(result, narrow_terminal)?;
    let rendered = pager.render_page(0);

    // Should see truncation marker like "(+25 cols)" in narrow terminal
    assert_truncation_markers_present(&rendered, &[/* expected column indices */]);

    // Verify marker format
    assert!(rendered.contains("(+") && rendered.contains(" cols)"));
}
```

**Acceptance Criteria:**
- Truncation markers appear when columns are hidden
- Marker format is "(+N cols)" where N is hidden column count
- No markers when all columns fit

---

### TC030-004: Single Wide Column
**Objective:** Validate handling of one very wide column (edge case).

**Test Approach:**
```rust
#[test]
fn test_single_wide_column_truncation() {
    // Create QueryResult with 1 column containing 200-char values
    let result = create_single_wide_column_result(200);
    let terminal_width = 80;

    let pager = Pager::new(result, terminal_width)?;
    let rendered = pager.render_page(0);

    // Should truncate within column, not wrap
    assert_no_overflow(&rendered, terminal_width);

    // Should see ellipsis for truncated content
    assert!(rendered.contains("…") || rendered.contains("..."));
}
```

**Acceptance Criteria:**
- Single wide column truncates gracefully
- No line wrapping occurs
- Ellipsis indicates truncation

---

### TC030-005: All Columns Require Truncation
**Objective:** Validate proportional truncation when all columns are too wide.

**Test Approach:**
```rust
#[test]
fn test_all_columns_truncated_proportionally() {
    // Create QueryResult where every column is 50 chars wide
    let result = create_all_wide_columns_result(10, 50);
    let terminal_width = 80;

    let pager = Pager::new(result, terminal_width)?;
    let rendered = pager.render_page(0);

    // All columns should be truncated
    assert_no_overflow(&rendered, terminal_width);

    // Verify some columns are visible (not all hidden)
    let visible_cols = count_visible_columns(&rendered);
    assert!(visible_cols >= 2, "Should show at least 2 columns");
}
```

**Acceptance Criteria:**
- Proportional truncation applied
- At least 2 columns visible
- No overflow despite all columns being wide

---

### TC030-006: Zero-Width Terminal (Edge Case)
**Objective:** Validate graceful degradation for invalid terminal width.

**Test Approach:**
```rust
#[test]
fn test_zero_width_terminal_degrades_gracefully() {
    let result = create_test_query_result();

    // Edge case: terminal width = 0 (should not panic)
    let result = std::panic::catch_unwind(|| {
        let pager = Pager::new(result, 0);
        // If no panic, pager should either:
        // 1. Use a minimum width (e.g., 40)
        // 2. Return an error gracefully
    });

    // Should not panic
    assert!(result.is_ok(), "Pager should handle zero-width gracefully");
}
```

**Acceptance Criteria:**
- No panic on zero or very small terminal width
- Pager uses sensible minimum width or returns error

---

### TC030-007: Pre-Formatted Input Rejected (Compile-Time Check)
**Objective:** Validate that old API (pre-formatted string) is gone.

**Test Approach:**
This is validated by the Rust compiler - the old API signature should not compile:

```rust
// This should NOT compile in Sprint 30:
// let pager = Pager::new("formatted string".to_string(), 100, &config);

// This SHOULD compile in Sprint 30:
let result = create_test_query_result();
let pager = Pager::new(result, 117)?;
```

**Acceptance Criteria:**
- Old API `Pager::new(String, usize, &PagerConfig)` does not exist
- New API `Pager::new(QueryResult, usize)` is the only constructor
- Attempting to use old API results in compile error

---

## Test Helpers

### create_test_query_result()
Creates a minimal QueryResult with known structure (3 columns, 5 rows).

### create_wide_query_result(col_count: usize)
Creates QueryResult with `col_count` columns for width testing.

### create_single_wide_column_result(value_width: usize)
Creates QueryResult with 1 column containing very long values.

### create_all_wide_columns_result(cols: usize, width: usize)
Creates QueryResult where every column has values of specified width.

### count_visible_columns(rendered: &str) -> usize
Parses rendered pager output to count visible columns.

## Integration with Sprint 29 Tests

The 23 existing Sprint 29 PTY tests remain mostly unchanged. They continue to test:
- Navigation (arrow keys, vim keys, h/l/H/L)
- Exit behavior (q, Esc)
- Status bar indicators
- Help text
- Combined horizontal/vertical navigation

These tests validate behavioral correctness through the REPL interface. They will pass if the architectural refactor is correct.

## Validation Strategy

1. **Unit Tests** (TC030-001 through TC030-006): Test dimensional logic directly
2. **Compile-Time Test** (TC030-007): Validate API change via compiler
3. **PTY Tests** (Sprint 29 tests): Validate end-to-end behavior through REPL

## Success Criteria

- All 7 new dimensional tests pass
- All 23 Sprint 29 PTY tests pass (proving no regressions)
- Zero manual verification (all automated with Track 3 utilities)
- 100% pass rate: `cargo test --test interactive_tests -- --ignored`
