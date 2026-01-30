# Sprint 29: Remaining Test Case Specifications

**Purpose:** Detailed specifications for test cases TC-HORIZ-017 through TC-INTEG-013 (54 test cases)

**Status:** Specifications ready for implementation by rust-teradata-architect

---

## Interactive Tests (TC-HORIZ-017 to TC-HORIZ-035)

### TC-HORIZ-017: Combined Horizontal and Vertical Navigation
- **AC:** AC-7
- **Type:** Interactive
- **Objective:** Verify arrow keys work for horizontal scroll while j/k/Space/b work for vertical scroll
- **Steps:**
  1. Query wide tall table
  2. Press → (should scroll columns right)
  3. Press ↓ or j (should scroll rows down)
  4. Press ← (should scroll columns left)
  5. Press ↑ or k (should scroll rows up)
  6. Verify status bar shows both column and row position
- **Expected:** Both navigation modes work independently, status bar shows "Columns X-Y of Z | Rows A-B of C"

### TC-HORIZ-018: Vim h/l Keys Navigation
- **AC:** AC-8
- **Type:** Interactive
- **Objective:** Verify h and l keys scroll horizontally like arrow keys
- **Steps:**
  1. Query wide table
  2. Press 'l' key 3 times (should scroll right)
  3. Press 'h' key 2 times (should scroll left)
  4. Verify columns shift identically to arrow keys
- **Expected:** h/l produce same results as ←/→

### TC-HORIZ-019: H Key Jump to First Column
- **AC:** AC-9
- **Type:** Interactive
- **Objective:** Verify H (uppercase) jumps to leftmost position
- **Steps:**
  1. Query wide table
  2. Scroll right 10 times
  3. Press 'H' (uppercase)
  4. Verify status bar shows "Columns 1-X"
  5. Verify left indicator gone, right indicator shows all hidden
- **Expected:** Instant jump to first column, indicators update

### TC-HORIZ-020: L Key Jump to Last Column
- **AC:** AC-10
- **Type:** Interactive
- **Objective:** Verify L (uppercase) jumps to rightmost position
- **Steps:**
  1. Query wide table
  2. Press 'L' (uppercase)
  3. Verify status bar shows "Columns Y-Z of Z" (last columns)
  4. Verify right indicator gone, left indicator shows all hidden
- **Expected:** Instant jump to last column window

### TC-HORIZ-021: Column Position Preserved - Vertical Scroll
- **AC:** AC-11
- **Type:** Interactive
- **Objective:** Verify horizontal position preserved during vertical scrolling
- **Steps:**
  1. Query wide tall table
  2. Scroll right 5 times (columns 6-10 visible)
  3. Scroll down 10 times (rows change)
  4. Verify status bar still shows "Columns 6-10"
  5. Scroll up 5 times
  6. Verify columns still 6-10
- **Expected:** Column range unchanged during vertical navigation

### TC-HORIZ-022: Help Text Shows Horizontal Navigation
- **AC:** AC-12
- **Type:** Interactive
- **Objective:** Verify help text (? key) documents horizontal navigation keys
- **Steps:**
  1. Query wide table
  2. Press '?' key
  3. Verify help text mentions: ←/→ or h/l (horizontal scroll)
  4. Verify help text mentions: H/L (jump to first/last column)
  5. Press 'q' to exit help
  6. Verify pager still at same position
- **Expected:** Help text includes horizontal navigation documentation

### TC-HORIZ-023: /pager off Disables Paging
- **AC:** AC-13
- **Type:** Interactive
- **Objective:** Verify /pager off shows all columns without paging
- **Steps:**
  1. Send "/pager off" command
  2. Query wide table
  3. Verify all columns shown (truncated if exceed terminal)
  4. Verify no pager activation (direct output)
  5. Verify no status bar or indicators
- **Expected:** All columns output directly, no paging interface

### TC-HORIZ-024: Right Arrow at End - No Effect
- **AC:** AC-1 (edge)
- **Type:** Interactive
- **Objective:** Verify right arrow at rightmost position does nothing
- **Steps:**
  1. Query wide table
  2. Press 'L' to jump to last column
  3. Press → 5 times
  4. Verify status bar unchanged (still at last columns)
  5. Verify no visual glitches
- **Expected:** Right arrow ignored at end, stable display

### TC-HORIZ-025: Left Arrow at Start - No Effect
- **AC:** AC-2 (edge)
- **Type:** Interactive
- **Objective:** Verify left arrow at leftmost position does nothing
- **Steps:**
  1. Query wide table (starts at column 1)
  2. Press ← 5 times
  3. Verify status bar still shows "Columns 1-X"
  4. Verify left indicator absent
- **Expected:** Left arrow ignored at start

### TC-HORIZ-026: Complex Keybinding Sequence
- **AC:** Integration
- **Type:** Interactive
- **Objective:** Verify complex mixed navigation sequences work correctly
- **Steps:**
  1. Query wide tall table
  2. Sequence: → → ↓ ← ↑ → Space ← b → → H → → L ← ← g G → → ↓ ↓
  3. Verify no crashes or visual corruption
  4. Verify status bar always shows valid position
  5. Verify indicators always accurate
- **Expected:** Complex sequences handled correctly, state consistent

### TC-HORIZ-027: Arrow Keys and Vim Keys Interchangeable
- **AC:** AC-8
- **Type:** Interactive
- **Objective:** Verify mixing arrow keys and Vim keys produces consistent results
- **Steps:**
  1. Query wide table
  2. Sequence: → l → h ← l h → (alternating)
  3. Track status bar after each press
  4. Verify final position matches expected offset
- **Expected:** Both key types work identically, can be mixed freely

### TC-HORIZ-028: Jump Keys Update Indicators Correctly
- **AC:** AC-9, AC-10
- **Type:** Interactive
- **Objective:** Verify H/L jumps update indicators accurately
- **Steps:**
  1. Query wide table (40 columns)
  2. Scroll to middle (column 20)
  3. Verify both indicators present
  4. Press 'H' (jump to start)
  5. Verify left indicator gone, right indicator shows ~35 cols
  6. Press 'L' (jump to end)
  7. Verify right indicator gone, left indicator shows ~35 cols
- **Expected:** Indicators always accurate after jumps

### TC-HORIZ-029: Wide Table (50+ columns) Navigation
- **AC:** Edge case
- **Type:** Interactive
- **Objective:** Verify 50+ column table navigates without issues
- **Steps:**
  1. Query 50-column table
  2. Scroll right to end (L key or multiple →)
  3. Scroll left to start (H key)
  4. Verify no crashes or integer overflow
  5. Verify indicators show correct counts (e.g., "+45 cols")
- **Expected:** Large column counts handled correctly

### TC-HORIZ-030: Narrow Terminal Adaptation
- **AC:** Edge case
- **Type:** Interactive
- **Objective:** Verify pager adapts to narrow terminal (80 cols)
- **Steps:**
  1. Set terminal to 80 columns (or use tmux pane)
  2. Query wide table (30 columns)
  3. Verify at least 1-2 columns visible
  4. Verify indicators fit in display
  5. Verify scrolling works
- **Expected:** Pager adapts gracefully, shows fewer columns, still usable

### TC-HORIZ-031: Status Bar Integrates Row and Column
- **AC:** AC-6 + vertical integration
- **Type:** Interactive
- **Objective:** Verify status bar shows both row and column position clearly
- **Steps:**
  1. Query wide tall table (30 cols × 100 rows)
  2. Verify status bar format: "Columns 1-5 of 30 | Rows 1-20 of 100"
  3. Scroll right 5 times
  4. Verify column range updates: "Columns 6-10 of 30"
  5. Scroll down (Space)
  6. Verify row range updates, column range unchanged
- **Expected:** Both positions shown clearly, update independently

### TC-HORIZ-032: Help Accessible During Horizontal Scroll
- **AC:** AC-12
- **Type:** Interactive
- **Objective:** Verify help can be accessed mid-scroll and returns to same position
- **Steps:**
  1. Query wide table
  2. Scroll right 10 times (columns 11-15 visible)
  3. Press '?'
  4. Read help text
  5. Press 'q' to exit help
  6. Verify back at columns 11-15 (position preserved)
- **Expected:** Help accessible anytime, returns to previous position

### TC-HORIZ-033: Multiple Pager Sessions Preserve State
- **AC:** Integration
- **Type:** Interactive
- **Objective:** Verify entering/exiting pager multiple times works correctly
- **Steps:**
  1. Query wide table, scroll right 5 times, exit (q)
  2. Run different query (narrow table), exit (q)
  3. Query wide table again
  4. Verify starts at column 1 (fresh state for new query)
- **Expected:** Each query gets fresh pager state, no carryover

### TC-HORIZ-034: Rapid Key Presses (Stress Test)
- **AC:** Robustness
- **Type:** Interactive
- **Objective:** Verify pager handles rapid key input without corruption
- **Steps:**
  1. Query wide table
  2. Hold down → key (or send many rapid → events)
  3. Verify scrolls smoothly to end
  4. Hold down ← key
  5. Verify returns smoothly to start
  6. Mix rapid → ↓ ← ↑ inputs
  7. Verify no crashes, corruption, or incorrect state
- **Expected:** Handles rapid input gracefully, state remains consistent

### TC-HORIZ-035: Single Column Table - No Horizontal Scroll
- **AC:** Edge case
- **Type:** Interactive
- **Objective:** Verify single-column table shows no horizontal scrolling
- **Steps:**
  1. Query single-column table
  2. Verify pager displays (if table is tall)
  3. Press → (should have no effect)
  4. Press 'l' (should have no effect)
  5. Verify no indicators appear
  6. Verify status bar shows "Columns 1-1 of 1" or just row info
- **Expected:** No horizontal scrolling possible, no indicators, stable

---

## Regression Tests (TC-REGR-001 to TC-REGR-010)

### TC-REGR-001: Vertical Scrolling Still Works (j/k)
- **Type:** Interactive (Regression)
- **Objective:** Verify j/k keys still scroll rows up/down
- **Steps:**
  1. Query tall table (narrow, 5 columns × 100 rows)
  2. Press 'j' 10 times
  3. Verify status bar shows "Rows 11-30" (or similar)
  4. Press 'k' 5 times
  5. Verify status bar shows "Rows 6-25"
- **Expected:** Vertical scrolling unchanged by horizontal paging implementation

### TC-REGR-002: Page Up/Down Still Works (Space/b)
- **Type:** Interactive (Regression)
- **Objective:** Verify Space and b still page through rows
- **Steps:**
  1. Query tall table
  2. Press Space (page down)
  3. Verify jumped ~20 rows
  4. Press 'b' (page up)
  5. Verify returned to previous position
- **Expected:** Page navigation unchanged

### TC-REGR-003: Jump to Top/Bottom Still Works (g/G)
- **Type:** Interactive (Regression)
- **Objective:** Verify g and G still jump to first/last row
- **Steps:**
  1. Query tall table
  2. Scroll to middle (Space several times)
  3. Press 'g' (lowercase)
  4. Verify status bar shows "Rows 1-20"
  5. Press 'G' (uppercase)
  6. Verify status bar shows last rows "Rows 81-100"
- **Expected:** Row jumps unchanged

### TC-REGR-004: Status Bar Shows Correct Row Position
- **Type:** Interactive (Regression)
- **Objective:** Verify row position display not broken by column range addition
- **Steps:**
  1. Query tall narrow table (no horizontal scrolling)
  2. Scroll down 30 rows
  3. Verify status bar shows accurate row range "Rows 31-50"
  4. Verify no column information shown (or "Columns 1-5 of 5")
- **Expected:** Row position display accurate and clear

### TC-REGR-005: /pager off Works for Tall Tables
- **Type:** Interactive (Regression)
- **Objective:** Verify /pager off disables vertical paging too
- **Steps:**
  1. Send "/pager off"
  2. Query tall table (100 rows)
  3. Verify all rows output directly (no pager)
  4. Verify can scroll terminal buffer to see all rows
- **Expected:** Both horizontal and vertical paging disabled

### TC-REGR-006: Pager Exit (q) Still Safe
- **Type:** Interactive (Regression)
- **Objective:** Verify 'q' key safety - never exits program
- **Steps:**
  1. Query any table
  2. Press 'q' in pager
  3. Verify returns to tq> prompt
  4. Repeat 10 times (enter pager, press q)
  5. Verify program never exits entirely
- **Expected:** 'q' always returns to REPL, never exits tq

### TC-REGR-007: Existing Unit Tests Still Pass
- **Type:** Unit test validation
- **Objective:** Verify existing pager unit tests not broken
- **Steps:**
  1. Run `cargo test --lib pager`
  2. Verify 100% pass rate
  3. Check for new compiler warnings
- **Expected:** All existing tests pass, no warnings

### TC-REGR-008: Cell Truncation Still Works
- **Type:** Interactive (Regression)
- **Objective:** Verify long cell content still truncated with ellipsis
- **Steps:**
  1. Query table with very long VARCHAR column (200+ chars)
  2. Verify cell content truncated at ~100 chars
  3. Verify ellipsis (…) appended
  4. Scroll horizontally
  5. Verify truncation still applied
- **Expected:** Cell truncation layer still functional

### TC-REGR-009: Table Formatting Consistent
- **Type:** Interactive (Regression)
- **Objective:** Verify table borders and formatting not corrupted
- **Steps:**
  1. Query various tables (narrow, wide, mixed types)
  2. Verify borders (│ ├ ┤ ─ ┬ ┴) render correctly
  3. Scroll horizontally
  4. Verify borders still aligned
  5. Verify column headers aligned with data
- **Expected:** Table formatting remains clean and aligned

### TC-REGR-010: REPL Commands Work After Paging
- **Type:** Interactive (Regression)
- **Objective:** Verify REPL metacommands work after exiting pager
- **Steps:**
  1. Query table, enter pager, exit (q)
  2. Send "/help" - verify works
  3. Send "/describe tablename" - verify works
  4. Query another table - verify works
  5. Send "/quit" - verify exits cleanly
- **Expected:** REPL fully functional after paging

---

## Edge Case Tests (TC-EDGE-001 to TC-EDGE-012)

### TC-EDGE-001: Single Column Table - Unit
- **Type:** Unit
- **Objective:** Verify col_offset stays 0 for single-column table
- **Implementation:**
```rust
#[test]
fn test_single_column_no_horizontal_scroll() {
    let mut pager = Pager::new(create_test_table(1), 80);
    pager.handle_key(KeyCode::Right);
    assert_eq!(pager.col_offset, 0);
    assert_eq!(pager.hidden_columns_right(), 0);
}
```

### TC-EDGE-002: Exact Terminal Fit - Unit
- **Type:** Unit
- **Objective:** Verify visible_column_count equals total when exact fit
- **Implementation:**
```rust
#[test]
fn test_exact_fit_no_scrolling() {
    let pager = Pager::new(create_test_table(5), 200);
    assert_eq!(pager.visible_column_count(), 5);
    assert_eq!(pager.hidden_columns_right(), 0);
}
```

### TC-EDGE-003: 50+ Columns - Unit
- **Type:** Unit
- **Objective:** Verify calculations handle large column counts
- **Implementation:**
```rust
#[test]
fn test_large_column_count() {
    let pager = Pager::new(create_test_table(100), 80);
    assert!(pager.visible_column_count() >= 1);
    assert_eq!(pager.hidden_columns_left() + pager.visible_column_count() +
               pager.hidden_columns_right(), 100);
}
```

### TC-EDGE-004 to TC-EDGE-012: Similar Edge Cases
- **TC-EDGE-004:** Narrow terminal unit test (40 cols)
- **TC-EDGE-005:** Wide terminal unit test (300 cols)
- **TC-EDGE-006:** Single column interactive (verify no visual scroll)
- **TC-EDGE-007:** Exact fit interactive (verify no indicators)
- **TC-EDGE-008:** 50+ columns interactive (full navigation test)
- **TC-EDGE-009:** Narrow terminal interactive (graceful adaptation)
- **TC-EDGE-010:** Wide terminal interactive (efficient space use)
- **TC-EDGE-011:** Empty result set (no crash)
- **TC-EDGE-012:** Very wide columns (200+ char cells)

---

## Integration Tests (TC-INTEG-001 to TC-INTEG-013)

### TC-INTEG-001: Right Scroll + Down Scroll
- **Type:** Interactive (Integration)
- **Objective:** Verify combined horizontal + vertical navigation
- **Sequence:** → → ↓ ↓
- **Expected:** Column position preserved during vertical scroll

### TC-INTEG-002: Jump End + Up + Jump Start
- **Type:** Interactive (Integration)
- **Objective:** Verify jump keys work with vertical navigation
- **Sequence:** L ↑ ↑ H
- **Expected:** Column jumps work, vertical position changes

### TC-INTEG-003: Arrows + Vim Keys Mixed
- **Type:** Interactive (Integration)
- **Objective:** Verify arrow and Vim keys interchangeable
- **Sequence:** → l → h ← l
- **Expected:** Both key types produce same results

### TC-INTEG-004: Page Down + Horizontal Scroll
- **Type:** Interactive (Integration)
- **Objective:** Verify page navigation + horizontal scroll
- **Sequence:** Space h l Space h
- **Expected:** Both navigation modes work independently

### TC-INTEG-005: Help During Horizontal Scroll
- **Type:** Interactive (Integration)
- **Objective:** Verify help accessible mid-scroll, returns to position
- **Sequence:** → → → ? (read help) q
- **Expected:** Returns to scrolled position after help

### TC-INTEG-006 to TC-INTEG-013: Complex Integration Scenarios
- **TC-INTEG-006:** All navigation modes combined
- **TC-INTEG-007:** Rapid alternating scroll
- **TC-INTEG-008:** Jump + scroll + jump sequences
- **TC-INTEG-009:** Vertical page + horizontal scroll combinations
- **TC-INTEG-010:** Multi-scroll then jump (verify jump accuracy)
- **TC-INTEG-011:** Horizontal scroll at various row positions
- **TC-INTEG-012:** Exit and re-enter pager (state reset)
- **TC-INTEG-013:** Column position across different queries (isolation)

---

## Implementation Notes

**For rust-teradata-architect:**

1. All interactive tests go in `tests/interactive_tests.rs`
2. Use existing test infrastructure: `spawn_tq_repl()`, `#[ignore]`
3. All tests marked `#[ignore]` - require live database
4. Create helper functions as documented in INDEX-SPRINT-29.md
5. Unit tests go in `src/commands/repl/pager.rs` test module
6. Regression tests verify existing functionality not broken

**Test Data Requirements:**
- Create test tables with `setup_wide_test_table(n)` function
- Support column counts: 1, 5, 20, 30, 32, 40, 50
- Support combined wide+tall: 30×100, 40×100

**Helper Functions Priority:**
1. `send_key(p, key)` - Send KeyCode to PTY
2. `extract_column_range(output)` - Parse status bar
3. `extract_right_indicator_count(output)` - Parse indicator
4. `extract_left_indicator_count(output)` - Parse indicator
5. `setup_wide_test_table(n)` - Create test table in database

---

## Summary

**Total Test Cases:** 70
- **Created (detailed files):** 16 (TC-HORIZ-001 to TC-HORIZ-016)
- **Specified (this document):** 54 (TC-HORIZ-017 to TC-INTEG-013)

**Acceptance Criteria Coverage:** 100% (all 13 ACs have 2-13 tests each)

**Next Steps for rust-teradata-architect:**
1. Implement unit tests in `src/commands/repl/pager.rs`
2. Implement interactive tests in `tests/interactive_tests.rs`
3. Create helper functions
4. Run test suite and iterate until 100% pass
