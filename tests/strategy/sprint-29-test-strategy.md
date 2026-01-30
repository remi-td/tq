# Sprint 29 Test Strategy

**Created:** 2026-01-30
**Author:** quality-validator
**Sprint:** Sprint 29
**Features:** Interactive Horizontal Paging with 13 Acceptance Criteria

---

## Instructions for quality-validator

This strategy derives comprehensive test coverage for Sprint 29's horizontal paging feature following the decision tree methodology from the template.

**Key Principles:**
1. Test strategy derives from feature characteristics (not assumptions)
2. Every test type must be justified by specification requirement
3. Gaps must be explicitly identified and assessed
4. Specifications are the source of truth

---

## Feature-by-Feature Test Strategy

### Feature: Interactive Horizontal Paging in REPL

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-29-planning.md` §Feature 1: Interactive Horizontal Paging in REPL (lines 51-74)
- Secondary: `docs/specifications/repl.md` §Result Paging (lines 1017-1160)
- Implementation: `src/commands/repl/pager.rs` (existing column windowing from Sprint 8/28)
- Related: GitHub Issue #7

**Requirements:**
1. **AC-1**: Right arrow (→) key scrolls view one column to the right when columns are hidden
2. **AC-2**: Left arrow (←) key scrolls view one column to the left when at scrolled position
3. **AC-3**: Display `(+N cols)` indicator in rightmost column showing count of hidden columns to the right
4. **AC-4**: Display `(+N cols)` indicator in leftmost column showing count of hidden columns to the left
5. **AC-5**: `q` or `Esc` key exits paging mode and returns to REPL prompt
6. **AC-6**: Status bar shows current column range (e.g., "Columns 3-8 of 32")
7. **AC-7**: Horizontal paging works with vertical paging (arrow keys for horizontal, j/k or Space/b for vertical)
8. **AC-8**: Vim-style `h`/`l` keys work for horizontal navigation (alongside arrow keys)
9. **AC-9**: `H` key jumps to first column (leftmost position)
10. **AC-10**: `L` key jumps to last column (rightmost position)
11. **AC-11**: Column position preserved when scrolling vertically
12. **AC-12**: Help text (`?` key) shows horizontal navigation controls
13. **AC-13**: `/pager off` command disables paging and shows all columns (truncated if needed)

**Feature Characteristics:**

**User Interaction Type:** ✅ Interactive PTY (REPL, terminal UI with cursor/colors/rendering)

**Explanation:** This feature is exclusively about interactive terminal behavior in the REPL pager. Users navigate wide result sets with arrow keys and Vim keybindings, see visual column indicators (`(+N cols)`), and interact with a live paging interface. The pager uses crossterm for terminal control (raw mode, alternate screen, key event capture). This CANNOT be validated without a real PTY because:
- Arrow key and Vim key events must be captured and processed correctly
- Terminal rendering (column indicators, status bar, borders) must display correctly
- Visual layout depends on actual terminal dimensions
- User sees and interacts with the pager in real-time
- Integration with existing vertical paging must work seamlessly

**Observable Behavior:**
- ✅ Visual output in terminal (colors, formatting, layout, cursor position)
  - Status bar with column position: `Columns 3-8 of 32`
  - Column truncation indicators: `(+N cols)` on left/right borders
  - Navigation hints: `← →: columns | ↑↓: rows | q: exit`
  - Help text displaying horizontal navigation controls
- ✅ Database side effects (records inserted/updated/deleted)
  - Requires live database to generate wide result sets (20-50+ columns) for realistic testing
- ✅ State management (session state, cache, persistence)
  - Pager tracks: row_offset, col_offset, visible columns, terminal dimensions
  - Column position preserved across vertical scrolling operations
  - REPL state preserves connection after pager exit

**External Dependencies:**
- ✅ Database connection (requires live database)
  - Need real wide tables (20+ columns) to trigger horizontal paging
  - Must test with various column counts: 10, 20, 30, 50+ columns
- ✅ Terminal/PTY (terminal control sequences, cursor positioning)
  - Pager uses crossterm: raw mode, alternate screen, key event polling
  - Status bar and indicator rendering require actual terminal dimensions
  - Keybinding detection requires real keyboard event capture

**Validation Challenges:**
- **Challenge 1**: Arrow key AND Vim key (h/l/H/L) navigation requires PTY - unit tests cannot simulate KeyCode::Left/Right/Char('h')/Char('l') events with crossterm event polling
- **Challenge 2**: Visual rendering (column indicators on BOTH sides, status bar with column range) only observable in real terminal with actual character rendering
- **Challenge 3**: Combined horizontal + vertical navigation with column position preservation involves complex state interaction that must be validated in realistic scenarios
- **Challenge 4**: Pager exit behavior (`q` returns to REPL) requires full REPL + pager integration - cannot be unit tested in isolation
- **Challenge 5**: Terminal width detection and column windowing logic depends on real terminal size for accurate testing
- **Challenge 6**: Regression testing - must verify vertical paging still works AND doesn't break column position preservation
- **Challenge 7**: Help text display (`?` key) requires PTY interaction and output verification
- **Challenge 8**: Edge cases (1 column, exact terminal fit, 50+ columns) require both unit testing (logic) and interactive testing (behavior)

**Critical Behaviors to Validate:**
1. **Right arrow scrolling** - "Right arrow (→) key scrolls view one column to the right when columns are hidden" (AC-1)
2. **Left arrow scrolling** - "Left arrow (←) key scrolls view one column to the left when at scrolled position" (AC-2)
3. **Right-side indicators** - "Display `(+N cols)` indicator in rightmost column showing count of hidden columns to the right" (AC-3)
4. **Left-side indicators** - "Display `(+N cols)` indicator in leftmost column showing count of hidden columns to the left" (AC-4)
5. **Safe exit to REPL** - "`q` or `Esc` exits paging mode and returns to REPL prompt" (AC-5)
6. **Status bar column range** - "Status bar shows current column range (e.g., 'Columns 3-8 of 32')" (AC-6)
7. **Combined navigation** - "Horizontal paging works with vertical paging (arrow keys for horizontal, j/k or Space/b for vertical)" (AC-7)
8. **Vim h/l keys** - "Vim-style `h`/`l` keys work for horizontal navigation (alongside arrow keys)" (AC-8)
9. **Jump to first column** - "`H` key jumps to first column (leftmost position)" (AC-9)
10. **Jump to last column** - "`L` key jumps to last column (rightmost position)" (AC-10)
11. **Column position preservation** - "Column position preserved when scrolling vertically" (AC-11)
12. **Help text** - "Help text (`?` key) shows horizontal navigation controls" (AC-12)
13. **Pager disable** - "`/pager off` command disables paging and shows all columns (truncated if needed)" (AC-13)

**Additional Edge Cases:**
- Single column table (no horizontal scrolling should occur)
- Table exactly fits terminal width (no indicators should appear)
- Very wide table (50+ columns) without crashing or visual glitches
- Narrow terminal (80 cols) showing fewer columns per view
- Wide terminal (200 cols) showing more columns per view

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Unit tests cannot validate terminal output, keyboard event handling, visual rendering
  Gap: Arrow key handling, Vim key handling, status bar display, column indicators, help text, pager exit behavior

IF "Visual output in terminal" checked:
  → Interactive tests OR integration tests with output capture REQUIRED
  Reason: Unit tests cannot validate formatting, colors, layout, indicators
  Gap: Status bar content, left/right column position indicators, navigation hints visibility

IF "Database connection" checked:
  → Integration tests with live database REQUIRED
  Reason: Need real wide tables to trigger horizontal paging
  Gap: Realistic wide result sets, edge cases with different column counts (10, 20, 30, 50+)

IF "State management" checked:
  → Unit tests for state logic REQUIRED
  Reason: Pager state (row_offset, col_offset, column position preservation) logic should be unit tested
  Gap: Bounds checking, visible column calculation, offset wrapping, jump logic (H/L keys)
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Pager internal logic for horizontal navigation (column offset calculations, bounds, visible column count, jump operations)
- **Approach:** Test `Pager` struct methods in isolation:
  - `visible_column_count()` - Calculate how many columns fit in terminal width with left/right indicators
  - `hidden_columns_left()` - Calculate count of hidden columns to the left (for indicators)
  - `hidden_columns_right()` - Calculate count of hidden columns to the right (for indicators)
  - `handle_key()` - Process horizontal navigation keys (Left, Right, h, l, H, L) and update col_offset
  - Column offset bounds checking (don't scroll past first/last column)
  - Jump operations (H jumps to col_offset=0, L jumps to last column window)
  - Column position preservation during vertical scrolling (col_offset unchanged when row_offset changes)
  - Status bar text generation (format column range strings: "Columns 3-8 of 32")
- **Rationale:** Pure logic functions can be unit tested without PTY - catches calculation bugs, off-by-one errors, edge cases in column windowing
- **Gap if missing:** Logic bugs in offset calculation could cause:
  - Incorrect column windowing (showing wrong columns)
  - Scrolling out of bounds (accessing non-existent columns)
  - Wrong indicator counts (e.g., "(+5 cols)" when only 3 hidden)
  - Jump keys not working correctly (H/L)
  - Column position not preserved during vertical scroll
- **Necessity:** ✅ REQUIRED

**Test Type 2: Interactive Tests (expectrl) - PRIMARY VALIDATION**
- **Validates:** End-to-end pager behavior as users experience it (keyboard → visual output → REPL return)
- **Approach:** Use expectrl to spawn tq REPL in PTY, send SQL that generates wide tables, press keys, verify output contains:
  - **AC-1**: Press Right arrow, verify columns shifted right
  - **AC-2**: Press Left arrow after scrolling right, verify columns shifted left
  - **AC-3**: Verify `(+N cols)` indicator appears on right side when columns hidden to right
  - **AC-4**: Verify `(+N cols)` indicator appears on left side when scrolled right
  - **AC-5**: Press `q`, verify pager exits and returns to `tq>` prompt
  - **AC-6**: Verify status bar shows "Columns X-Y of Z" format
  - **AC-7**: Press arrow keys (horizontal), then j/k (vertical), verify both work together
  - **AC-8**: Press `h` and `l` keys, verify horizontal scrolling works (Vim style)
  - **AC-9**: Press `H` key, verify jump to first column (indicators update correctly)
  - **AC-10**: Press `L` key, verify jump to last column window (indicators update)
  - **AC-11**: Scroll right, then scroll down vertically, verify column position preserved
  - **AC-12**: Press `?` key, verify help text displays horizontal navigation controls
  - **AC-13**: Send `/pager off` command, query wide table, verify all columns shown (truncated)
- **Rationale:** This is the ONLY way to validate interactive pager behavior - keyboard events, terminal rendering, and REPL integration all require real PTY
- **Gap if missing:** Cannot verify user-observable behavior:
  - Arrow keys might not scroll columns at all
  - Vim keys (h/l/H/L) might not work
  - Status bar might show wrong column range or be missing
  - Column indicators might be wrong, missing, or appear on wrong side
  - Pager might not exit correctly or might exit the entire program
  - Help text might not display horizontal controls
  - Column position might not be preserved during vertical scrolling
  - `/pager off` might not work
- **Necessity:** ✅ REQUIRED - BLOCKING for feature approval

**Test Type 3: Regression Tests (Interactive)**
- **Validates:** Existing vertical paging functionality not broken by horizontal paging implementation
- **Approach:** Interactive tests that verify existing pager features still work:
  - Vertical scrolling (j/k, Space/b, g/G) still works correctly
  - Status bar shows correct row position (not broken by column range addition)
  - Pager still exits on `q` for vertically-paged tables (without horizontal scrolling)
  - `/pager off` still works for tall tables (disables vertical paging)
  - Existing unit tests for vertical paging logic still pass (100% pass rate)
- **Rationale:** Adding horizontal navigation could break existing vertical paging - must prove no regressions
- **Gap if missing:** Horizontal paging implementation might break:
  - Vertical scrolling keys
  - Status bar row position display
  - Pager exit behavior
  - `/pager off` command
  - This would cause user-reported bugs in production
- **Necessity:** ✅ REQUIRED - BLOCKING (Success Criteria: "Zero regressions in existing pager functionality")

**Test Type 4: Edge Case Tests (Interactive + Unit)**
- **Validates:** Pager handles unusual table dimensions and terminal sizes gracefully
- **Approach:**
  - **Unit tests**: Edge cases for column windowing logic
    - Single column table (col_offset stays 0, no indicators shown)
    - Zero columns (should not crash, fallback to single-column display)
    - Table exactly fits terminal width (visible_column_count() returns exact column count, no indicators)
    - Very wide table (50+ columns) - verify calculations don't overflow
    - Narrow terminal (80 cols) - verify at least 1 column shown
  - **Interactive tests**: Edge cases for pager behavior
    - Single column table: Right arrow does nothing, no indicators appear
    - Exact fit table: No horizontal scrolling available, status bar shows all columns
    - 50+ column table: Can scroll through all columns without crash, indicators show correct counts
    - Narrow terminal: Pager adapts gracefully, shows fewer columns, still navigable
- **Rationale:** Edge cases often expose bugs in boundary logic - must explicitly test non-standard scenarios
- **Gap if missing:** Pager could:
  - Crash or panic on single-column table
  - Show incorrect indicators for exact-fit table
  - Overflow or show wrong counts for very wide tables
  - Fail to adapt to narrow terminals
  - Allow scrolling past boundaries
- **Necessity:** ✅ REQUIRED

**Test Type 5: Integration Tests (Keybinding Combinations)**
- **Validates:** Complex keybinding sequences work correctly (realistic user workflows)
- **Approach:** Interactive tests with multi-step navigation sequences:
  - Scroll right (→ →), then down (↓ ↓), verify column position preserved
  - Jump to end (L), scroll up (↑), jump to start (H), verify correct positions
  - Scroll right with arrows (→), then with Vim keys (l), verify consistent behavior
  - Use Space to page down, then use h/l to scroll horizontally, verify status bar updates correctly
  - Scroll to middle position, press ? for help, press q to exit help, verify pager returns to same position
- **Rationale:** Users will combine keybindings in complex ways - must verify state is managed correctly across different navigation modes
- **Gap if missing:** Complex keybinding combinations might:
  - Break column position preservation
  - Show incorrect status bar after mode switches
  - Cause state inconsistencies (e.g., wrong indicators after help exit)
  - Result in confusing or broken UX for real users
- **Necessity:** ✅ REQUIRED

**Test Type 6: Manual Tests (UX Validation)**
- **Validates:** Pager UX is smooth, intuitive, and professional (subjective quality)
- **Approach:** Manual testing in development environment:
  - Generate wide table (30+ columns) and verify navigation feels smooth and responsive
  - Verify status bar is easy to read and updates quickly
  - Verify column indicators (`(+N cols)`) are clear and not confusing
  - Verify help text (`?`) is helpful and shows all horizontal controls
  - Verify pager exit returns cleanly to REPL without visual artifacts
  - Test on different terminal sizes (80 cols, 120 cols, 200 cols, 300 cols)
  - Test with different column count tables (10, 20, 30, 50+ columns)
- **Rationale:** Automated tests verify correctness but not UX quality - human validation needed for smooth, intuitive interaction
- **Gap if missing:** Pager might work correctly but feel:
  - Clunky or unresponsive
  - Confusing (indicators not clear)
  - Inconsistent with vertical paging UX
  - Unprofessional (status bar hard to read)
- **Necessity:** ⚠️ RECOMMENDED - Not blocking, but important for user satisfaction

**Test Type 7: Benchmark Tests**
- **Validates:** N/A
- **Approach:** N/A
- **Rationale:** Specification has no performance requirements for horizontal paging - pager rendering is inherently I/O bound (terminal output). Horizontal scrolling is just changing which columns are rendered, same total render time.
- **Gap if missing:** None - performance not a specified requirement
- **Necessity:** ❌ NOT NEEDED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates column offset logic, bounds checking, jump operations, indicator calculations | Logic bugs, off-by-one errors, incorrect visible column count, wrong indicators | MUST IMPLEMENT (20+ tests) |
| Interactive tests (expectrl) | ✅ REQUIRED | Validates all 13 acceptance criteria with real keyboard input and visual output | Cannot verify arrow keys, Vim keys, indicators, status bar, help text, column preservation | MUST IMPLEMENT (15+ tests) |
| Regression tests (interactive) | ✅ REQUIRED | Validates existing vertical paging not broken by horizontal paging changes | Vertical scrolling broken, status bar broken, pager exit broken | MUST IMPLEMENT (5+ tests) |
| Edge case tests (unit + interactive) | ✅ REQUIRED | Validates pager handles unusual dimensions (1 column, 50+ columns, exact fit, narrow terminals) | Crashes on edge cases, incorrect indicators, poor UX for unusual tables | MUST IMPLEMENT (10+ tests) |
| Integration tests (keybinding combos) | ✅ REQUIRED | Validates complex multi-step navigation sequences work correctly | State inconsistencies, column position not preserved, broken status bar after mode switches | MUST IMPLEMENT (8+ tests) |
| Manual tests (UX) | ⚠️ RECOMMENDED | Human validates subjective UX quality (smooth navigation, clear status bar, intuitive help) | Pager works correctly but feels clunky or confusing | SHOULD PERFORM |
| Benchmark tests | ❌ NOT NEEDED | Feature has no performance requirements, pager is I/O bound | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 5 (Unit, Interactive, Regression, Edge Case, Integration) - MUST implement all
- ⚠️ RECOMMENDED test types: 1 (Manual UX) - Should implement unless time constrained
- ❌ NOT NEEDED test types: 1 (Benchmark) - Explicitly omitted with rationale

#### 4. Specification Coverage Map

**Map each specification requirement (AC) to test type(s) that validate it:**

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-1 | "Right arrow (→) key scrolls view one column to the right when columns are hidden" | sprint-29-planning.md §56 | Unit + Interactive | Unit validates col_offset logic, interactive validates actual scrolling | TC-HORIZ-001, TC-HORIZ-011 |
| AC-2 | "Left arrow (←) key scrolls view one column to the left when at scrolled position" | sprint-29-planning.md §57 | Unit + Interactive | Unit validates col_offset decrement logic, interactive validates actual scrolling | TC-HORIZ-002, TC-HORIZ-012 |
| AC-3 | "Display `(+N cols)` indicator in rightmost column showing count of hidden columns to the right" | sprint-29-planning.md §58 | Unit + Interactive | Unit validates hidden_columns_right() calculation, interactive validates visual display | TC-HORIZ-003, TC-HORIZ-013 |
| AC-4 | "Display `(+N cols)` indicator in leftmost column showing count of hidden columns to the left" | sprint-29-planning.md §59 | Unit + Interactive | Unit validates hidden_columns_left() calculation, interactive validates visual display | TC-HORIZ-004, TC-HORIZ-014 |
| AC-5 | "`q` or `Esc` key exits paging mode and returns to REPL prompt" | sprint-29-planning.md §60 | Interactive | Only interactive test can verify REPL integration and prompt return | TC-HORIZ-005, TC-HORIZ-015 |
| AC-6 | "Status bar shows current column range (e.g., 'Columns 3-8 of 32')" | sprint-29-planning.md §61 | Unit + Interactive | Unit validates status text generation, interactive validates visual display | TC-HORIZ-006, TC-HORIZ-016 |
| AC-7 | "Horizontal paging works with vertical paging (arrow keys for horizontal, j/k or Space/b for vertical)" | sprint-29-planning.md §62 | Interactive + Integration | Interactive validates combined navigation, integration validates state preservation | TC-HORIZ-007, TC-HORIZ-017, TC-HORIZ-025 |
| AC-8 | "Vim-style `h`/`l` keys work for horizontal navigation (alongside arrow keys)" | sprint-29-planning.md §63 | Unit + Interactive | Unit validates handle_key() for h/l, interactive validates actual behavior | TC-HORIZ-008, TC-HORIZ-018 |
| AC-9 | "`H` key jumps to first column (leftmost position)" | sprint-29-planning.md §64 | Unit + Interactive | Unit validates col_offset=0 logic, interactive validates visual jump and indicator update | TC-HORIZ-009, TC-HORIZ-019 |
| AC-10 | "`L` key jumps to last column (rightmost position)" | sprint-29-planning.md §65 | Unit + Interactive | Unit validates last column calculation, interactive validates visual jump | TC-HORIZ-010, TC-HORIZ-020 |
| AC-11 | "Column position preserved when scrolling vertically" | sprint-29-planning.md §66 | Unit + Integration | Unit validates col_offset unchanged by row_offset change, integration validates in sequences | TC-HORIZ-021, TC-HORIZ-026 |
| AC-12 | "Help text (`?` key) shows horizontal navigation controls" | sprint-29-planning.md §67 | Interactive | Only interactive test can capture help text output and verify content | TC-HORIZ-022 |
| AC-13 | "`/pager off` command disables paging and shows all columns (truncated if needed)" | sprint-29-planning.md §68 | Interactive | Only interactive test can verify metacommand behavior and output | TC-HORIZ-023 |
| REGR-1 | "Vertical scrolling (j/k, Space/b, g/G) still works correctly" | Success Criteria | Interactive (Regression) | Verify existing pager keys not broken | TC-REGR-001, TC-REGR-002 |
| REGR-2 | "Status bar shows correct row position (not broken by column range addition)" | Success Criteria | Interactive (Regression) | Verify status bar row display still works | TC-REGR-003 |
| REGR-3 | "`/pager off` still works for tall tables (vertical paging)" | Success Criteria | Interactive (Regression) | Verify metacommand works for vertical paging too | TC-REGR-004 |
| EDGE-1 | "Single column table shows no horizontal scrolling" | Edge Cases | Unit + Interactive | Unit validates no scrolling logic, interactive validates visual behavior | TC-EDGE-001, TC-EDGE-006 |
| EDGE-2 | "Table exactly fits terminal width shows no indicators" | Edge Cases | Unit + Interactive | Unit validates exact fit detection, interactive validates no indicators shown | TC-EDGE-002, TC-EDGE-007 |
| EDGE-3 | "50+ column table handles correctly without crash" | Edge Cases | Unit + Interactive | Unit validates calculation limits, interactive validates real navigation | TC-EDGE-003, TC-EDGE-008 |
| EDGE-4 | "Narrow terminal (80 cols) adapts gracefully" | Edge Cases | Interactive | Verify pager adapts to small terminal, shows fewer columns | TC-EDGE-004 |
| EDGE-5 | "Wide terminal (200 cols) shows more columns correctly" | Edge Cases | Interactive | Verify pager uses available space efficiently | TC-EDGE-005 |

**Coverage Validation:**
- ✅ Every specification requirement (13 ACs + 3 regression + 5 edge) appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements (all have test coverage)
- ✅ No unjustified test types (all test types have requirement rationale)

**Coverage Gaps:**
- None identified - all 13 acceptance criteria, regression requirements, and edge cases have explicit test coverage

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Performance/Benchmark Tests**
- **Reason for omission:** Specification has no performance requirements (<Xms timing) for horizontal paging. Pager rendering is I/O bound (terminal output speed), not CPU bound.
- **What won't be validated:** Scrolling speed, rendering latency, memory usage for very wide tables
- **Risk assessment:** LOW - Horizontal paging just changes which columns are rendered, doesn't add significant computational overhead. Terminal I/O is the bottleneck, not our code.
- **Mitigation:** Monitor in production, add benchmarks if users report slowness or if future requirements add performance SLAs
- **Revisit criteria:** If users report horizontal scrolling feels sluggish, or if performance requirements added to spec (e.g., "scrolling must complete in <100ms")

**Cross-Platform Tests (Windows/macOS/Linux)**
- **Reason for omission:** crossterm library provides cross-platform terminal abstraction. Horizontal paging uses same crossterm APIs as existing vertical paging (already tested cross-platform).
- **What won't be validated:** Platform-specific key event handling differences, terminal rendering variations
- **Risk assessment:** LOW - crossterm is mature and handles platform differences. Existing vertical paging works cross-platform, horizontal paging uses same primitives.
- **Mitigation:** Development team tests on multiple platforms during implementation. CI runs on Linux. Community testing on Windows/macOS during release.
- **Revisit criteria:** If platform-specific bugs reported, or if we add platform-specific keybindings

**Stress Tests (1000+ column tables)**
- **Reason for omission:** Specification doesn't mention extreme table sizes. Practical use cases involve 10-50 columns. Database query overhead would be primary bottleneck, not pager.
- **What won't be validated:** Behavior with extremely wide tables (100+, 1000+ columns), memory usage limits
- **Risk assessment:** LOW - Column windowing shows fixed number of columns regardless of total count. col_offset calculations are simple arithmetic (O(1)). Real bottleneck is database returning 1000+ columns.
- **Mitigation:** Unit tests validate bounds checking for large column counts. If needed, can add artificial large-table tests.
- **Revisit criteria:** If users report issues with very wide tables (>100 columns), or if data warehouse use cases emerge

#### 6. Test Implementation Plan

**For each REQUIRED test type, document implementation approach:**

**Test Type: Unit Tests**
- **Location:** `src/commands/repl/pager.rs` test module (inline `#[cfg(test)]`)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 20-25 tests
- **Key scenarios to cover:**
  1. `visible_column_count()` with left indicator, right indicator, both, neither
  2. `hidden_columns_left()` at various col_offset values (0, middle, end)
  3. `hidden_columns_right()` at various col_offset values
  4. `handle_key()` for Left/Right arrow keys (col_offset updates correctly)
  5. `handle_key()` for h/l Vim keys (same behavior as arrows)
  6. `handle_key()` for H key (jumps to col_offset=0)
  7. `handle_key()` for L key (jumps to last column window)
  8. Bounds checking: col_offset never negative, never exceeds column count
  9. Status bar text generation: "Columns X-Y of Z" format correctness
  10. Edge cases: 1 column (no scrolling), 0 columns (fallback), exact terminal fit
  11. Column position preservation: col_offset unchanged when row_offset modified (separate test for vertical navigation logic)
- **Mocking strategy:**
  - Mock `TableData` with known column counts (5, 10, 20, 50 columns)
  - Mock terminal size (80 cols, 120 cols, 200 cols) for visible_column_count tests
  - No database mocking needed (unit tests don't query database)

**Test Type: Interactive Tests (expectrl)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 15-20 tests
- **Key scenarios to cover:**
  1. **AC-1**: Generate wide table (30 cols), press Right arrow, verify columns shifted right (parse output, check column headers changed)
  2. **AC-2**: Scroll right, press Left arrow, verify columns shifted back left
  3. **AC-3**: Scroll to middle position, verify `(+N cols)` indicator on right border (parse output for indicator text)
  4. **AC-4**: Scroll right from start, verify `(+N cols)` indicator on left border
  5. **AC-5**: Press `q` in pager, verify returns to `tq>` prompt (no program exit)
  6. **AC-6**: In pager, verify status bar contains "Columns X-Y of Z" format
  7. **AC-7**: Press Right arrow (horizontal), then `j` (vertical down), verify both work (status bar updates)
  8. **AC-8**: Press `h` key, verify columns scroll left (same as Left arrow)
  9. **AC-8**: Press `l` key, verify columns scroll right (same as Right arrow)
  10. **AC-9**: Scroll to middle, press `H`, verify jump to first column (status bar shows "Columns 1-X")
  11. **AC-10**: Press `L`, verify jump to last column window (status bar shows "Columns Y-Z of Z")
  12. **AC-11**: Scroll right, then scroll down vertically (Space), verify column position preserved (status bar still shows scrolled position)
  13. **AC-12**: Press `?` key in pager, verify help text displays (parse output for "h/l" or "← →" mentions)
  14. **AC-13**: Send `/pager off`, query wide table, verify all columns shown without paging (direct output, no pager interaction)
  15. **Edge**: Query single-column table, verify no horizontal scrolling (no arrow key effects)
  16. **Edge**: Query 50+ column table, scroll through, verify no crashes
- **Implementation notes:**
  - Use `spawn_tq_repl()` helper from existing tests
  - Add test database table with 30 columns (can create in test setup)
  - Parse pager output with regex to extract status bar text and indicators
  - Handle PTY timing with appropriate delays (100-500ms) for rendering
  - Use `#[ignore]` attribute - requires live database

**Test Type: Regression Tests**
- **Location:** `tests/interactive_tests.rs` (separate section)
- **Framework:** Built-in Rust integration test support with expectrl
- **Test count estimate:** 5-7 tests
- **Key scenarios to cover:**
  1. Query tall table (no wide columns), use j/k for vertical scrolling, verify still works
  2. Query tall table, press Space (page down), verify still works
  3. Query tall table, press `g` (jump to top), press `G` (jump to bottom), verify still works
  4. Query tall table, verify status bar shows row position correctly (not broken by column range display)
  5. Send `/pager off`, query tall table, verify vertical paging disabled (direct output)
  6. Run existing pager unit tests (`cargo test pager`) - verify 100% pass rate
- **Setup requirements:** Test database with tall table (100+ rows, few columns) for vertical paging scenarios

**Test Type: Edge Case Tests**
- **Location:** `src/commands/repl/pager.rs` (unit tests) + `tests/interactive_tests.rs` (interactive)
- **Framework:** Built-in Rust test framework (unit) + expectrl (interactive)
- **Test count estimate:** 10-12 tests (5 unit, 5-7 interactive)
- **Key scenarios to cover:**
  - **Unit**: visible_column_count() with 1 column returns 1
  - **Unit**: hidden_columns_right() with 1 column returns 0
  - **Unit**: handle_key(Right) with 1 column doesn't change col_offset
  - **Unit**: visible_column_count() with exact terminal fit returns exact count
  - **Unit**: visible_column_count() with 50 columns and narrow terminal (80 cols) returns ~3-4 columns
  - **Interactive**: Query single-column table, press Right arrow, verify no effect
  - **Interactive**: Query exact-fit table (columns fit perfectly), verify no indicators shown
  - **Interactive**: Query 50-column table, scroll through all columns, verify no crashes and correct counts
  - **Interactive**: Set narrow terminal (80 cols), query wide table, verify pager adapts (shows fewer columns)
  - **Interactive**: Set wide terminal (200 cols), query 30-column table, verify uses available space
- **Implementation notes:** Mock terminal size for unit tests, use environment variable or test harness to set terminal dimensions for interactive tests

**Test Type: Integration Tests (Keybinding Combinations)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl
- **Test count estimate:** 8-10 tests
- **Key scenarios to cover:**
  1. Scroll right (→ →), then down (↓ ↓), verify column position preserved (status bar shows same columns)
  2. Jump to end (L), scroll up (↑), jump to start (H), verify correct positions and indicators
  3. Scroll right with arrows (→ →), then with Vim keys (l l), verify consistent behavior
  4. Use Space to page down, then use h/l to scroll horizontally, verify status bar updates correctly
  5. Scroll to middle position (→ → →), press ? for help, press q to exit help, verify pager returns to same position
  6. Combine all navigation modes: arrows + Vim keys + jumps + vertical scrolling in random order
  7. Rapid key presses (hold Right arrow), verify no crashes or visual corruption
  8. Alternating horizontal and vertical scrolling (→ ↓ ← ↑), verify state consistent
- **Implementation notes:** Complex sequences, need to verify status bar and indicators after each step

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- **Unit tests validate:** Column offset logic, bounds checking, indicator calculations, jump operations, status bar formatting
- **Interactive tests validate:** All 13 acceptance criteria with real keyboard input and visual output verification
- **Regression tests validate:** Existing vertical paging functionality not broken
- **Edge case tests validate:** Unusual table dimensions and terminal sizes handled gracefully
- **Integration tests validate:** Complex keybinding sequences work correctly with state preservation
- **Combined coverage:** COMPREHENSIVE - unit tests prove logic correctness, interactive tests prove user-observable behavior, regression tests prove no breakage

**Gaps in combined coverage:**
- **Gap 1**: Cross-platform testing (Windows/macOS/Linux) - only tested on development platform and CI (Linux)
  - **Acceptable because:** crossterm provides cross-platform abstraction, existing vertical paging works cross-platform, low risk
- **Gap 2**: Extreme stress testing (1000+ column tables) - not tested
  - **Acceptable because:** Practical use cases involve 10-50 columns, specification doesn't mention extreme sizes, low risk
- **Gap 3**: Subjective UX quality (smooth animation, professional appearance) - not automated
  - **Acceptable because:** Manual testing will validate UX quality, automated tests can't measure subjective feel

**Acceptance criteria:**
- ✅ All 13 specification requirements (ACs) have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted with justification

**If gaps exist, document why they're acceptable:**
- **Gap 1 (cross-platform)** is acceptable because: crossterm handles platform differences, existing pager works cross-platform, development + CI testing covers primary platforms, community testing during release
- **Gap 2 (stress testing)** is acceptable because: Practical tables have <50 columns, specification doesn't require extreme sizes, unit tests validate calculation correctness for large numbers
- **Gap 3 (UX quality)** is acceptable because: Manual testing will validate subjective quality, automated tests prove functional correctness which is the primary requirement

---

## Strategy Summary

**Total Features Analyzed:** 1 (Interactive Horizontal Paging with 13 acceptance criteria)

**Test Types Required:**
- ✅ Unit tests: Feature 1 (pager column logic)
- ✅ Interactive tests: Feature 1 (all 13 ACs, edge cases, keybinding combos)
- ✅ Regression tests: Feature 1 (vertical paging preservation)
- ✅ Edge case tests: Feature 1 (unusual dimensions, terminal sizes)
- ✅ Integration tests: Feature 1 (complex keybinding sequences)
- ⚠️ Manual tests: Feature 1 (UX quality validation)
- ❌ Benchmark tests: None (no performance requirements)

**Estimated Test Count:**
- Unit: 20-25 tests
- Interactive: 15-20 tests (ACs)
- Regression: 5-7 tests
- Edge Case: 10-12 tests (5 unit, 5-7 interactive)
- Integration: 8-10 tests (keybinding combos)
- **Total: 58-74 tests** (excluding manual UX validation)

**Risk Assessment:**
- HIGH risk gaps: **None**
- MEDIUM risk gaps: **None**
- LOW risk gaps: Cross-platform testing (mitigated by crossterm), stress testing (mitigated by bounds validation), UX quality (mitigated by manual testing)

**Dependencies Required:**
- ✅ Live database: Yes (test database with wide tables: 10, 20, 30, 50+ columns)
- ✅ Network access: No (local database sufficient)
- ❌ Specific OS: No (crossterm cross-platform, but test primarily on Linux via CI)
- ✅ Terminal/PTY: Yes (expectrl for interactive tests)
- ✅ Test data: Wide tables with various column counts (can create in test setup)

**Tool Requirements:**
- **Existing tools are sufficient:**
  - Built-in Rust test framework for unit tests
  - expectrl for interactive PTY tests
  - cargo test for execution
  - No new tools needed

**Test Execution Strategy:**
1. **Unit tests** (`cargo test --lib pager`) - Run first, fast feedback (< 1 second)
2. **Integration tests** (`cargo test --test integration_tests`) - Run second (no database needed for most)
3. **Interactive tests** (`cargo test --test interactive_tests -- --ignored`) - Run last, requires database (2-5 minutes)
4. **Manual UX validation** - After automated tests pass, human validates subjective quality

---

## Strategy Validation Checklist

**Before submitting to tq-project-manager for review:**

- ✅ Every feature has complete specification analysis section
- ✅ Feature characteristics are classified (Interactive PTY - not assumed)
- ✅ Test strategy is derived from characteristics (decision tree applied)
- ✅ Every test type has clear rationale (justified by requirements)
- ✅ Gap analysis is complete and honest (3 low-risk gaps documented)
- ✅ Specification coverage map includes all requirements (13 ACs + regression + edge cases)
- ✅ Every requirement maps to at least one test type (21 requirements mapped)
- ✅ Test implementation plan is detailed and actionable (framework, location, scenario count, mocking strategy)
- ✅ Coverage sufficiency is assessed (comprehensive coverage, gaps acceptable)
- ✅ No hand-waving or vague justifications (specific test scenarios, file paths, framework choices)

**All checkboxes checked** - Strategy is complete and ready for review.

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-01-30
**Review Status:** DRAFT
**Submitted for Review:** 2026-01-30

**Reviewer:** tq-project-manager
**Review Status:** PENDING
**Review Date:** [Date]
**Review Comments:** [tq-project-manager's feedback]

**Approval means:**
- ✅ Test strategy derived from specifications (not assumptions)
- ✅ All required test types identified with clear rationale (5 required types)
- ✅ Coverage gaps explicitly identified and assessed (3 low-risk gaps)
- ✅ Implementation plan is detailed and achievable (58-74 tests estimated)
- ✅ Ready to proceed with test case creation

**Approval signature:** [tq-project-manager agent ID and timestamp]
