# Sprint 28 Test Strategy

**Created:** 2026-01-28
**Author:** quality-validator
**Sprint:** Sprint 28
**Features:** Interactive Horizontal Paging (#7), Clean REPL Startup (#11)

---

## Instructions for quality-validator

This strategy derives comprehensive test coverage for Sprint 28's two features following the decision tree methodology from the template.

**Key Principles:**
1. Test strategy derives from feature characteristics (not assumptions)
2. Every test type must be justified by specification requirement
3. Gaps must be explicitly identified and assessed
4. Specifications are the source of truth

---

## Feature-by-Feature Test Strategy

### Feature #7: Interactive Horizontal Paging

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-28-planning.md` §Acceptance Criteria (lines 73-87)
- Secondary: `docs/specifications/repl.md` §Large Result Handling & Result Paging (lines 1017-1160)
- Tertiary: Existing pager code: `src/commands/repl/pager.rs` (Sprint 8 column windowing exists)
- Requirements:
  1. "Pager activates automatically for result sets wider than terminal" (AC line 76)
  2. "Arrow keys (← →) scroll columns left/right by 1 column" (AC line 77)
  3. "Column position indicators show `(+N cols)` on truncated sides" (AC line 78)
  4. "Status bar displays: `Columns X-Y of Z | ← →: scroll | q: exit`" (AC line 79)
  5. "`q` or `Esc` exits pager and returns to REPL prompt" (AC line 80)
  6. "Vertical scrolling still works (↑ ↓, j k, Space, b, g, G)" (AC line 81)
  7. "Combined horizontal + vertical navigation works smoothly" (AC line 82)
  8. "`/pager off` disables pager (shows truncated single-page output)" (AC line 83)
  9. "Works with all output formats (table only, CSV/JSON are single-line)" (AC line 84)
  10. "100% existing tests pass (no regressions)" (AC line 85)

**Feature Characteristics:**

**User Interaction Type:** ✅ Interactive PTY (REPL, terminal UI with cursor/colors/rendering)

**Explanation:** This feature is exclusively about interactive terminal behavior. Users navigate with arrow keys, see visual column indicators, and interact with a live pager. The pager uses crossterm for terminal control (raw mode, alternate screen, cursor positioning). This CANNOT be validated without a real PTY because:
- Arrow key events must be captured and processed
- Terminal rendering (borders, status bar, column indicators) must display correctly
- Visual layout depends on actual terminal dimensions
- User sees and interacts with the pager in real-time

**Observable Behavior:** (Check all that apply)
- ✅ Visual output in terminal (colors, formatting, layout, cursor position)
  - Status bar with column position: `Columns 1-5 of 23`
  - Column truncation indicators: `(+N cols)` on borders
  - Navigation hints: `← →: scroll | q: exit`
- ✅ Database side effects (records inserted/updated/deleted)
  - Requires live database to generate wide result sets for testing
- ✅ State management (session state, cache, persistence)
  - Pager tracks: row_offset, col_offset, visible columns, terminal dimensions
  - REPL state preserves connection after pager exit

**External Dependencies:** (Check all that apply)
- ✅ Database connection (requires live database)
  - Need real wide tables (20+ columns) to test horizontal paging
- ✅ Terminal/PTY (terminal control sequences, cursor positioning)
  - Pager uses crossterm: raw mode, alternate screen, key events
  - Status bar rendering requires actual terminal dimensions

**Validation Challenges:** (What makes this hard to test?)
- **Challenge 1**: Arrow key navigation requires PTY - unit tests cannot simulate KeyCode::Left/Right events with crossterm event polling
- **Challenge 2**: Visual rendering (column indicators, status bar layout) only observable in real terminal with actual character rendering
- **Challenge 3**: Combined horizontal + vertical navigation involves complex state (row_offset + col_offset) that must be validated in realistic scenarios
- **Challenge 4**: Pager exit behavior (`q` returns to REPL) requires full REPL + pager integration - cannot be unit tested
- **Challenge 5**: Terminal width detection and column windowing logic depends on real terminal size
- **Challenge 6**: Regression testing - must verify vertical paging still works after horizontal paging added

**Critical Behaviors to Validate:** (From specification - be specific)
1. **Automatic activation for wide tables** - "Pager activates automatically for result sets wider than terminal" (AC line 76)
2. **Left/right arrow key scrolling** - "Arrow keys (← →) scroll columns left/right by 1 column" (AC line 77)
3. **Column position indicators** - "Column position indicators show `(+N cols)` on truncated sides" (AC line 78)
4. **Status bar with navigation hints** - "Status bar displays: `Columns X-Y of Z | ← →: scroll | q: exit`" (AC line 79)
5. **Safe exit to REPL** - "`q` or `Esc` exits pager and returns to REPL prompt" (AC line 80, REQ-PAGER-001 in repl.md)
6. **Vertical navigation preserved** - "Vertical scrolling still works (↑ ↓, j k, Space, b, g, G)" (AC line 81)
7. **Combined navigation** - "Combined horizontal + vertical navigation works smoothly" (AC line 82)
8. **Pager disable command** - "`/pager off` disables pager (shows truncated single-page output)" (AC line 83)
9. **Edge case: Single column** - Must handle tables with 1 column (no horizontal scrolling)
10. **Edge case: Exact fit** - Must handle tables that exactly fit terminal width (no scrolling needed)
11. **Edge case: Very wide tables** - Must handle 50+ column tables without crashing

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Unit tests cannot validate terminal output, cursor behavior, visual rendering
  Gap: Arrow key handling, status bar display, column indicators, pager exit behavior

IF "Visual output in terminal" checked:
  → Interactive tests OR integration tests with output capture REQUIRED
  Reason: Unit tests cannot validate formatting, colors, layout
  Gap: Status bar content, column position indicators, navigation hints visibility

IF "Database connection" checked:
  → Integration tests with live database REQUIRED
  Reason: Need real wide tables to trigger horizontal paging
  Gap: Realistic wide result sets, edge cases with different column counts

IF "State management" checked:
  → Unit tests for state logic REQUIRED
  Reason: Pager state (row_offset, col_offset) logic should be unit tested
  Gap: Bounds checking, visible column calculation, offset wrapping
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Pager internal logic (column windowing calculations, offset bounds, visible column count)
- **Approach:** Test `Pager` struct methods in isolation:
  - `visible_column_count()` - Calculate how many columns fit in terminal width
  - `handle_key()` - Process navigation keys and update state (returns bool for continue/exit)
  - Column offset bounds checking (don't scroll past first/last column)
  - Status bar text generation (format column position strings)
- **Rationale:** Pure logic functions can be unit tested without PTY - catches calculation bugs, off-by-one errors, edge cases
- **Gap if missing:** Logic bugs in offset calculation could cause incorrect column windowing (showing wrong columns, scrolling out of bounds)
- **Necessity:** ✅ REQUIRED

**Test Type 2: Interactive Tests (expectrl) - PRIMARY VALIDATION**
- **Validates:** End-to-end pager behavior as users experience it (arrow keys → visual output → REPL return)
- **Approach:** Use expectrl to spawn tq REPL in PTY, send SQL that generates wide tables, press arrow keys, verify output contains:
  - Column position indicators: `(+N cols)`
  - Status bar with navigation hints: `← →: scroll | q: exit`
  - Correct columns visible after scrolling right/left
  - Pager exits on `q` and returns to `tq>` prompt
  - Combined horizontal + vertical navigation works
- **Rationale:** This is the ONLY way to validate interactive pager behavior - keyboard events, terminal rendering, and REPL integration all require real PTY
- **Gap if missing:** Cannot verify user-observable behavior:
  - Arrow keys might not work at all
  - Status bar might not display
  - Column indicators might be wrong or missing
  - Pager might not exit correctly
  - REPL session might not be preserved after pager exit
- **Necessity:** ✅ REQUIRED - BLOCKING for feature approval

**Test Type 3: Regression Tests (Interactive)**
- **Validates:** Vertical paging still works after horizontal paging implementation
- **Approach:** Interactive tests that verify existing pager features not broken:
  - Vertical scrolling (j/k, Space/b, g/G) still works
  - Status bar shows correct row position
  - Pager still exits on `q` for vertically-paged tables
  - `/pager off` still works (disables paging)
- **Rationale:** Adding horizontal navigation could break existing vertical paging - must prove no regressions
- **Gap if missing:** Horizontal paging implementation might break existing functionality, causing user-reported bugs in production
- **Necessity:** ✅ REQUIRED

**Test Type 4: Edge Case Tests (Interactive + Unit)**
- **Validates:** Pager handles unusual table dimensions gracefully
- **Approach:**
  - Unit tests: Edge cases for column windowing logic (1 column, 0 columns, exact terminal width fit)
  - Interactive tests: Edge cases for pager behavior (single column table shows no horizontal scrolling, very wide table with 50+ columns)
- **Rationale:** Edge cases often expose bugs in boundary logic - must explicitly test non-standard scenarios
- **Gap if missing:** Pager could crash or behave incorrectly for edge cases (e.g., panic on single-column table, incorrect indicator for exact-fit table)
- **Necessity:** ✅ REQUIRED

**Test Type 5: Manual Tests (UX Validation)**
- **Validates:** Pager UX is smooth, intuitive, and professional (subjective quality)
- **Approach:** Manual testing in development environment:
  - Generate wide table (20+ columns) and verify navigation feels smooth
  - Verify status bar is easy to read and helpful
  - Verify column indicators are clear and not confusing
  - Verify pager exit returns cleanly to REPL without artifacts
  - Test on different terminal sizes (80 cols, 120 cols, 200 cols)
- **Rationale:** Automated tests verify correctness but not UX quality - human validation needed for smooth interaction
- **Gap if missing:** Pager might work correctly but feel clunky or confusing to users
- **Necessity:** ⚠️ RECOMMENDED - Not blocking, but important for quality

**Test Type 6: Benchmark Tests**
- **Validates:** N/A
- **Approach:** N/A
- **Rationale:** Specification has no performance requirements - pager rendering is inherently I/O bound (terminal output)
- **Gap if missing:** None - performance not a specified requirement
- **Necessity:** ❌ NOT NEEDED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates column windowing logic, offset calculations, bounds checking | Logic bugs, off-by-one errors, incorrect visible column count | MUST IMPLEMENT |
| Interactive tests (expectrl) | ✅ REQUIRED | Validates terminal rendering, arrow key navigation, status bar display | Visual bugs, arrow keys not working, status bar missing, pager not exiting correctly | MUST IMPLEMENT |
| Regression tests (interactive) | ✅ REQUIRED | Validates existing vertical paging not broken by horizontal paging | Vertical scrolling broken, pager exit broken, `/pager off` broken | MUST IMPLEMENT |
| Edge case tests (unit + interactive) | ✅ REQUIRED | Validates pager handles unusual dimensions (1 column, 50+ columns, exact fit) | Crashes on edge cases, incorrect indicators, poor UX for unusual tables | MUST IMPLEMENT |
| Manual tests (UX) | ⚠️ RECOMMENDED | Human validates subjective UX quality (smooth navigation, clear status bar) | Pager works correctly but feels clunky or confusing | SHOULD PERFORM |
| Benchmark tests | ❌ NOT NEEDED | Feature has no performance requirements, pager is I/O bound | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 4 (Unit, Interactive, Regression, Edge Case) - MUST implement all
- ⚠️ RECOMMENDED test types: 1 (Manual UX) - Should perform but not blocking
- ❌ NOT NEEDED test types: 1 (Benchmark) - Explicitly omitted with rationale

#### 4. Specification Coverage Map

**Map each specification requirement to test type(s) that validate it:**

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| REQ-7.1 | "Pager activates automatically for result sets wider than terminal" | AC line 76 | Interactive (expectrl) | Only interactive test can verify automatic activation with real terminal width detection | IC-PAGER-001 |
| REQ-7.2 | "Arrow keys (← →) scroll columns left/right by 1 column" | AC line 77 | Interactive (expectrl) + Unit | Unit tests key handler logic, interactive tests actual key events in PTY | IC-PAGER-002, TC-PAGER-001 |
| REQ-7.3 | "Column position indicators show `(+N cols)` on truncated sides" | AC line 78 | Interactive (expectrl) | Visual indicators only observable in terminal output | IC-PAGER-003 |
| REQ-7.4 | "Status bar displays: `Columns X-Y of Z \| ← →: scroll \| q: exit`" | AC line 79 | Interactive (expectrl) + Unit | Unit tests status bar text generation, interactive tests actual display | IC-PAGER-004, TC-PAGER-002 |
| REQ-7.5 | "`q` or `Esc` exits pager and returns to REPL prompt" | AC line 80 | Interactive (expectrl) | Pager exit and REPL integration only testable in full REPL session | IC-PAGER-005 |
| REQ-7.6 | "Vertical scrolling still works (↑ ↓, j k, Space, b, g, G)" | AC line 81 | Interactive (expectrl) - Regression | Regression test to verify existing vertical paging preserved | IC-PAGER-006 |
| REQ-7.7 | "Combined horizontal + vertical navigation works smoothly" | AC line 82 | Interactive (expectrl) | Complex state interaction (row_offset + col_offset) only testable end-to-end | IC-PAGER-007 |
| REQ-7.8 | "`/pager off` disables pager (shows truncated single-page output)" | AC line 83 | Interactive (expectrl) - Regression | Metacommand behavior requires full REPL session | IC-PAGER-008 |
| REQ-7.9 | "Works with all output formats (table only, CSV/JSON are single-line)" | AC line 84 | Interactive (expectrl) | Format selection and pager interaction requires REPL session | IC-PAGER-009 |
| REQ-7.10 | "100% existing tests pass (no regressions)" | AC line 85 | All existing tests | Run full test suite (386 tests) before and after implementation | N/A - existing suite |
| REQ-7.11 | "Edge case: Single column table" | Implied from spec | Unit + Interactive | Unit tests windowing logic, interactive tests UX | TC-PAGER-003, IC-PAGER-010 |
| REQ-7.12 | "Edge case: Exact fit table (columns exactly fit terminal)" | Implied from spec | Unit + Interactive | Unit tests windowing logic, interactive tests no scrolling indicators | TC-PAGER-004, IC-PAGER-011 |
| REQ-7.13 | "Edge case: Very wide table (50+ columns)" | Implied from spec | Unit + Interactive | Unit tests offset bounds, interactive tests navigation doesn't crash | TC-PAGER-005, IC-PAGER-012 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements (missing test coverage)
- ✅ No unjustified test types (test types without requirement rationale)

**Coverage Gaps:**
- None - all requirements have test coverage

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Performance/Benchmark Tests**
- **Reason for omission:** Specification has no performance requirements (no timing SLAs for pager rendering)
- **What won't be validated:** Rendering speed, memory usage for very large result sets, terminal output latency
- **Risk assessment:** LOW
  - Pager is I/O bound (terminal output is slow by nature)
  - Existing pager implementation (Sprint 8) has no reported performance issues
  - Column windowing reduces rendering work (only visible columns rendered)
- **Mitigation:** If users report sluggish paging, add benchmarks in future sprint
- **Revisit criteria:** User reports of slow paging or memory issues with very wide tables (100+ columns)

**Platform-Specific Tests (Windows/Linux/macOS)**
- **Reason for omission:** Sprint resources insufficient for multi-platform validation, development primarily on macOS
- **What won't be validated:** Pager behavior on Windows terminal, Linux console quirks, different terminal emulators
- **Risk assessment:** MEDIUM
  - Crossterm is cross-platform library (should work consistently)
  - Arrow key events might differ across platforms
  - Terminal dimensions detection might fail on some terminals
- **Mitigation:** Document primary test platform (macOS), rely on crossterm's cross-platform guarantees, monitor user reports
- **Revisit criteria:** User reports of platform-specific pager issues or if cross-platform CI becomes available

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/repl/pager.rs` test module (inline `#[cfg(test)]`)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 8 tests
- **Key scenarios to cover:**
  1. TC-PAGER-001: `handle_key()` with Right arrow increments col_offset
  2. TC-PAGER-002: `handle_key()` with Left arrow decrements col_offset (bounds check at 0)
  3. TC-PAGER-003: `visible_column_count()` calculates correct column count for given terminal width
  4. TC-PAGER-004: Status bar text generation includes column position `Columns X-Y of Z`
  5. TC-PAGER-005: Right arrow at last column does not increment offset (bounds check)
  6. TC-PAGER-006: Edge case - single column table (col_offset stays 0, visible_column_count returns 1)
  7. TC-PAGER-007: Edge case - exact fit table (visible_column_count equals total columns)
  8. TC-PAGER-008: Edge case - 50+ column table (offset calculation handles large numbers)
- **Mocking strategy:**
  - Mock terminal dimensions by passing term_width to Pager constructor
  - Mock table data with ColumnData structs
  - No database connection needed (work with parsed TableData)

**Test Type: Interactive Tests (expectrl) - PRIMARY VALIDATION**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 12 tests
- **Key scenarios to cover:**
  1. IC-PAGER-001: Wide table (20+ cols) triggers pager automatically
     - Action: Execute `SELECT * FROM wide_table`
     - Verify: Pager starts, only subset of columns visible, status bar shows `Columns 1-N of 20+`
  2. IC-PAGER-002: Right arrow scrolls columns right
     - Action: In pager, press Right arrow 3 times
     - Verify: Different columns visible, status bar shows `Columns 4-N of 20+`
  3. IC-PAGER-003: Left arrow scrolls columns left
     - Action: Scroll right, then press Left arrow 2 times
     - Verify: Previous columns visible, status bar shows `Columns 2-N of 20+`
  4. IC-PAGER-004: Column position indicators display on truncated sides
     - Action: Execute wide query, verify initial view
     - Verify: Output contains `(+N cols)` indicator on right border
  5. IC-PAGER-005: Status bar shows navigation hints
     - Action: Enter pager
     - Verify: Status bar contains `← →: scroll` and `q: exit`
  6. IC-PAGER-006: `q` exits pager and returns to REPL prompt
     - Action: Enter pager, press `q`
     - Verify: Pager exits, output shows `tq>` prompt, can execute new query
  7. IC-PAGER-007: Vertical scrolling still works (regression test)
     - Action: Execute wide query with 50+ rows, press `j` (down) 5 times
     - Verify: Row offset changes, status bar shows `Rows 6-N of 50+`
  8. IC-PAGER-008: Combined horizontal + vertical navigation
     - Action: Press Right arrow 3 times, then `j` (down) 5 times
     - Verify: Both column offset and row offset changed, status bar shows both correctly
  9. IC-PAGER-009: `/pager off` disables pager (regression test)
     - Action: Execute `/pager off`, then wide query
     - Verify: No pager starts, all output shown immediately (truncated if needed)
  10. IC-PAGER-010: Edge case - single column table shows no horizontal scrolling
     - Action: Execute `SELECT id FROM single_col_table`
     - Verify: Pager starts (if many rows), but no column indicators, no horizontal navigation hints
  11. IC-PAGER-011: Edge case - exact fit table (columns exactly fit terminal)
     - Action: Execute query with columns that exactly fit 80-column terminal
     - Verify: All columns visible, no `(+N cols)` indicator, no horizontal navigation hints
  12. IC-PAGER-012: Edge case - very wide table (50+ columns)
     - Action: Execute query with 50+ columns, scroll to rightmost columns
     - Verify: Pager handles large offset, no crash, status bar shows correct position
- **Implementation notes:**
  - All interactive tests marked `#[ignore]` (require live database)
  - Need test database with wide table fixture (20+ columns)
  - Use `spawn_tq_repl()` helper from existing interactive tests
  - Send keys with `p.send("\x1b[C")` (Right arrow ANSI escape) or `p.send_line("SELECT...")` for queries
  - Parse output with `p.expect("pattern")` for status bar text
  - Tests must be robust to PTY quirks (timing, buffering)

**Test Type: Integration Tests**
- **Location:** N/A
- **Framework:** N/A
- **Test count estimate:** 0 tests
- **Rationale:** Interactive tests (expectrl) provide full integration coverage - no separate integration tests needed

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: Column windowing logic, offset bounds, visible column calculation, status bar text generation
- Interactive tests validate: Arrow key navigation, visual column indicators, status bar display, pager exit to REPL, combined horizontal + vertical navigation, edge cases (single column, exact fit, very wide tables)
- Regression tests validate: Existing vertical paging preserved, `/pager off` still works, no crashes
- Combined coverage: COMPREHENSIVE

**Gaps in combined coverage:**
- Platform-specific behavior (Windows terminal, Linux console) not tested - LOW RISK (crossterm handles platform differences)
- Performance with extremely wide tables (1000+ columns) not tested - LOW RISK (not specified, unlikely use case)
- Manual UX validation (smooth scrolling, clear status bar) recommended but not automated - MEDIUM RISK (subjective quality)

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Gap 1 (Platform-specific): Acceptable because crossterm library provides cross-platform abstraction, development platform is macOS (primary), users can report platform-specific issues if they arise
- Gap 2 (Performance): Acceptable because specification has no performance requirements, column windowing limits rendering work, existing pager (Sprint 8) has no reported performance issues
- Gap 3 (Manual UX): Acceptable because automated tests cover correctness, manual UX validation is recommended (not blocking), subjective quality can be improved in future sprints based on user feedback

**Conclusion: Coverage is SUFFICIENT for feature approval if all automated tests pass.**

---

### Feature #11: Clean REPL Startup

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-28-planning.md` §Acceptance Criteria (lines 88-96)
- GitHub Issue: #11 - "[BUG] Warning and info messages on startup"
- Requirements:
  1. "No cargo warnings visible during `cargo run -- repl`" (AC line 90)
  2. "No 'Finished' or 'Running' messages visible" (AC line 91)
  3. "Only tq logo and connection info displayed" (AC line 92)
  4. "Solution works in both dev (cargo run) and release builds" (AC line 93)
  5. "100% existing tests pass" (AC line 95)

**Feature Characteristics:**

**User Interaction Type:** ✅ CLI Batch (scripted, piped, non-interactive command execution)

**Explanation:** This is NOT a REPL feature - it's a build/dev environment issue. The problem occurs during `cargo run` (development mode) before REPL starts. Cargo emits build output to stderr, which pollutes the terminal. The fix is at the build/development tooling level, not REPL code. However, the observable behavior (clean startup) IS visible in interactive REPL sessions.

**Observable Behavior:** (Check all that apply)
- ✅ Visual output in terminal (colors, formatting, layout, cursor position)
  - REPL startup banner should appear cleanly without cargo warnings
  - No "Finished dev [unoptimized + debuginfo]" messages
  - No "Running `target/debug/tq repl`" messages

**External Dependencies:** (Check all that apply)
- ❌ None (pure logic, no external dependencies)
- Note: This is a build tooling issue, not runtime dependency

**Validation Challenges:** (What makes this hard to test?)
- **Challenge 1**: Automated tests run via `cargo test`, which doesn't trigger the same cargo output as `cargo run` - difficult to reproduce in test environment
- **Challenge 2**: Issue is specific to development mode (`cargo run`), not release builds - test must distinguish between dev and release
- **Challenge 3**: Cargo output goes to stderr, not stdout - test must capture stderr separately
- **Challenge 4**: Different cargo versions and configurations may produce different output - test fragility
- **Challenge 5**: Fix might involve cargo configuration, shell scripts, or build.rs changes - difficult to unit test

**Critical Behaviors to Validate:** (From specification - be specific)
1. **Clean development mode startup** - "No cargo warnings visible during `cargo run -- repl`" (AC line 90)
2. **No cargo metadata** - "No 'Finished' or 'Running' messages visible" (AC line 91)
3. **Professional output** - "Only tq logo and connection info displayed" (AC line 92)
4. **Release build unaffected** - "Solution works in both dev (cargo run) and release builds" (AC line 93)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "CLI Batch" checked:
  → Manual tests REQUIRED (cargo run cannot be automated in test suite)
  Reason: Issue reproduces only with `cargo run`, not `cargo test`
  Gap: Automated verification of clean startup

IF "Visual output in terminal" checked:
  → Manual tests REQUIRED
  Reason: Must visually verify no cargo warnings appear
  Gap: Automated assertion of stderr cleanliness
```

**Derived Test Types:**

**Test Type 1: Manual Verification Tests - PRIMARY VALIDATION**
- **Validates:** Clean REPL startup in development mode (no cargo warnings)
- **Approach:** Manual testing checklist:
  1. Run `cargo run -- repl` in development environment
  2. Visually verify no cargo warnings or build messages appear
  3. Verify only tq logo and connection info displayed
  4. Run `cargo build --release && ./target/release/tq repl`
  5. Verify release build startup is clean
- **Rationale:** This is inherently a manual verification issue - cargo output is not part of tq's runtime behavior, cannot be captured in automated tests
- **Gap if missing:** Cannot verify the bug is actually fixed, users might still see polluted output
- **Necessity:** ✅ REQUIRED - BLOCKING for feature approval

**Test Type 2: Regression Tests (Existing Test Suite)**
- **Validates:** REPL startup code not broken by fix (e.g., if fix involves build.rs changes)
- **Approach:** Run full existing test suite (386 tests), verify 100% pass
- **Rationale:** Fix might involve build.rs or cargo config changes that could break compilation or tests
- **Gap if missing:** Fix could introduce regressions in existing functionality
- **Necessity:** ✅ REQUIRED

**Test Type 3: Documentation Tests**
- **Validates:** Developer documentation explains fix and expected behavior
- **Approach:** Verify `CONTRIBUTING.md` or developer docs document:
  - Expected clean startup behavior
  - How fix works (cargo config, build.rs, or run script)
  - Workarounds if needed (e.g., `cargo run --quiet -- repl`)
- **Rationale:** If fix involves workarounds or configuration, developers need documentation
- **Gap if missing:** Future developers might not understand why fix is needed or how to maintain it
- **Necessity:** ⚠️ RECOMMENDED

**Test Type 4: Unit Tests**
- **Validates:** N/A
- **Approach:** N/A
- **Rationale:** No unit-testable code changes expected - fix is build tooling, not application code
- **Gap if missing:** None
- **Necessity:** ❌ NOT NEEDED

**Test Type 5: Interactive Tests (expectrl)**
- **Validates:** N/A (existing interactive tests already verify REPL starts correctly)
- **Approach:** N/A
- **Rationale:** Existing interactive tests (`test_repl_startup_and_quit`) verify REPL startup behavior - no new tests needed
- **Gap if missing:** None
- **Necessity:** ❌ NOT NEEDED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Manual verification tests | ✅ REQUIRED | Only way to verify cargo output is suppressed during `cargo run` | Cannot verify bug is fixed, users might still see warnings | MUST PERFORM |
| Regression tests (existing suite) | ✅ REQUIRED | Verify fix doesn't break compilation or existing tests | Fix could introduce regressions | MUST RUN |
| Documentation tests | ⚠️ RECOMMENDED | Developer docs should explain fix and expected behavior | Future devs might not understand why fix exists | SHOULD UPDATE |
| Unit tests | ❌ NOT NEEDED | No unit-testable code changes (build tooling fix) | N/A | SKIP |
| Interactive tests | ❌ NOT NEEDED | Existing tests already verify REPL starts correctly | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 2 (Manual Verification, Regression) - MUST perform/run
- ⚠️ RECOMMENDED test types: 1 (Documentation) - Should update but not blocking
- ❌ NOT NEEDED test types: 2 (Unit, Interactive) - Explicitly omitted with rationale

#### 4. Specification Coverage Map

**Map each specification requirement to test type(s) that validate it:**

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| REQ-11.1 | "No cargo warnings visible during `cargo run -- repl`" | AC line 90 | Manual Verification | Cargo output only observable by running cargo run manually | MV-STARTUP-001 |
| REQ-11.2 | "No 'Finished' or 'Running' messages visible" | AC line 91 | Manual Verification | Cargo metadata output only observable manually | MV-STARTUP-002 |
| REQ-11.3 | "Only tq logo and connection info displayed" | AC line 92 | Manual Verification | Visual verification of clean output | MV-STARTUP-003 |
| REQ-11.4 | "Solution works in both dev (cargo run) and release builds" | AC line 93 | Manual Verification | Must test both cargo run (dev) and release binary | MV-STARTUP-004 |
| REQ-11.5 | "100% existing tests pass" | AC line 95 | Regression (existing suite) | Run full test suite before and after fix | N/A - existing suite |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements (missing test coverage)
- ✅ No unjustified test types (test types without requirement rationale)

**Coverage Gaps:**
- None - all requirements have test coverage (manual)

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Unit Tests**
- **Reason for omission:** No application code changes expected - fix is build tooling (cargo config, build.rs, or run script)
- **What won't be validated:** Code-level correctness (not applicable)
- **Risk assessment:** LOW - No code to test
- **Mitigation:** N/A
- **Revisit criteria:** If fix involves application code changes (e.g., stderr redirection in main.rs), add unit tests

**Interactive Tests (expectrl)**
- **Reason for omission:** Existing interactive tests already verify REPL starts correctly - cargo output issue is orthogonal to REPL functionality
- **What won't be validated:** N/A (REPL startup already tested)
- **Risk assessment:** LOW - Existing coverage sufficient
- **Mitigation:** Run existing interactive tests as regression tests
- **Revisit criteria:** None - existing tests adequate

**Automated Cargo Output Tests**
- **Reason for omission:** Cannot reliably automate cargo run output capture in test suite - cargo test runs tests differently than cargo run
- **What won't be validated:** Automatic verification of clean cargo output
- **Risk assessment:** MEDIUM - Must rely on manual verification
- **Mitigation:** Detailed manual test checklist, verify in CI documentation
- **Revisit criteria:** If automated solution found (e.g., custom test harness that runs cargo run and captures output), implement automated test

#### 6. Test Implementation Plan

**Test Type: Manual Verification Tests - PRIMARY VALIDATION**
- **Location:** Test case documentation: `tests/cases/TC-STARTUP-001-manual.md`
- **Framework:** Manual testing checklist
- **Test count estimate:** 4 manual verification steps
- **Key scenarios to cover:**
  1. MV-STARTUP-001: Clean dev mode startup
     - Action: Run `cargo run -- repl` in terminal
     - Verify: No cargo warnings appear, no "warning: unused import" or similar
     - Expected: Only tq banner and "Connected to" message
  2. MV-STARTUP-002: No cargo metadata in dev mode
     - Action: Run `cargo run -- repl` in terminal
     - Verify: No "Finished dev [unoptimized + debuginfo]" message
     - Verify: No "Running `target/debug/tq repl`" message
     - Expected: Clean startup without cargo build metadata
  3. MV-STARTUP-003: Professional output
     - Action: Run `cargo run -- repl` in terminal
     - Verify: Output starts with tq logo/banner
     - Verify: Only connection info displayed after banner
     - Expected: Professional, clean appearance
  4. MV-STARTUP-004: Release build unaffected
     - Action: Run `cargo build --release && ./target/release/tq repl`
     - Verify: Clean startup (no cargo output - release builds already clean)
     - Expected: Release build behavior unchanged
- **Implementation notes:**
  - Document manual test steps in `tests/cases/TC-STARTUP-001-manual.md`
  - Include screenshots of before/after in test case documentation
  - Manual verification required for each sprint before approval
  - Document in CONTRIBUTING.md if workaround is permanent (e.g., use `cargo run --quiet`)

**Test Type: Regression Tests (Existing Suite)**
- **Location:** All existing tests (386 tests across unit, integration, interactive)
- **Framework:** Built-in Rust test framework + expectrl
- **Test count estimate:** 386 tests (existing)
- **Approach:** Run full test suite with `cargo test` to verify no regressions
- **Implementation notes:**
  - Run `cargo test --lib` (unit tests)
  - Run `cargo test --test integration_tests` (integration tests)
  - Run `cargo test --test interactive_tests -- --ignored` (interactive tests with database)
  - Document pass/fail count in test report

**Test Type: Documentation Tests**
- **Location:** `CONTRIBUTING.md` or `docs/builder/developer-guide.md`
- **Framework:** Documentation review
- **Test count estimate:** 1 documentation section
- **Key content to add:**
  1. Expected clean startup behavior (no cargo warnings)
  2. How fix works (cargo config, build.rs, or run script)
  3. If workaround: Document recommended command (e.g., `cargo run --quiet -- repl`)
  4. If permanent fix: Document changes made (e.g., "build.rs suppresses warnings")
- **Implementation notes:**
  - Update during fix implementation
  - Recommend for developer onboarding

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Manual verification validates: Clean cargo run startup, no warnings, no metadata, professional output, release build unaffected
- Regression tests validate: Existing functionality not broken by fix
- Documentation tests validate: Developers understand fix and maintenance
- Combined coverage: ADEQUATE (manual verification is necessary and sufficient)

**Gaps in combined coverage:**
- Automated verification of cargo output cleanliness not possible - MEDIUM RISK (must rely on manual verification)
- Platform-specific cargo behavior (Windows vs macOS vs Linux) not tested - LOW RISK (cargo output is consistent across platforms)
- Different cargo versions might produce different warnings - LOW RISK (fix should suppress all cargo output)

**Acceptance criteria:**
- ✅ All specification requirements have test coverage (manual)
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Gap 1 (Automated verification): Acceptable because cargo run output is not part of tq's runtime behavior, manual verification is standard practice for build tooling issues
- Gap 2 (Platform-specific): Acceptable because cargo output format is consistent across platforms, fix should work universally
- Gap 3 (Cargo versions): Acceptable because fix targets cargo's output mechanism (not specific warnings), should work across versions

**Conclusion: Coverage is SUFFICIENT for feature approval if manual verification passes and existing test suite passes (100%).**

---

## Strategy Summary

**Total Features Analyzed:** 2

**Test Types Required:**
- Unit tests: ✅ Feature #7 (Pager logic) - 8 tests
- Interactive tests: ✅ Feature #7 (Pager behavior) - 12 tests
- Regression tests: ✅ Feature #7 (Vertical paging), Feature #11 (Existing suite) - 12 tests + full suite (386)
- Manual tests: ✅ Feature #7 (UX validation - recommended), Feature #11 (Startup verification - required) - 4 manual verification steps
- Benchmark tests: ❌ None (not needed)

**Estimated Test Count:**
- Unit: 8 tests (Feature #7 pager logic)
- Interactive: 12 tests (Feature #7 pager behavior)
- Regression: Existing suite (386 tests) + no new regression tests needed
- Manual: 4 verification steps (Feature #11 startup)
- Total automated: 20 new tests + 386 existing = 406 tests
- Total manual: 4 verification steps

**Risk Assessment:**
- HIGH risk gaps: None
- MEDIUM risk gaps:
  - Feature #7: Platform-specific pager behavior not tested (mitigated by crossterm cross-platform library)
  - Feature #11: Automated verification not possible (mitigated by detailed manual test checklist)
- LOW risk gaps:
  - Feature #7: Performance with 1000+ column tables not tested (not specified, unlikely use case)
  - Feature #11: Cargo version-specific behavior not tested (fix should work universally)

**Dependencies Required:**
- Live database: Yes (for Feature #7 interactive tests - need wide table fixtures)
- Network access: No
- Specific OS: No (primary test platform: macOS, crossterm handles platform differences)
- Other:
  - Feature #7: Test database with wide table (20+ columns, 50+ rows)
  - Feature #11: Access to development environment with cargo run capability

---

## Strategy Validation Checklist

**Before submitting to tq-project-manager for review:**

- ✅ Every feature has complete specification analysis section
- ✅ Feature characteristics are classified (not assumed)
- ✅ Test strategy is derived from characteristics (not guessed)
- ✅ Every test type has clear rationale
- ✅ Gap analysis is complete and honest
- ✅ Specification coverage map includes all requirements
- ✅ Every requirement maps to at least one test type
- ✅ Test implementation plan is detailed and actionable
- ✅ Coverage sufficiency is assessed
- ✅ No hand-waving or vague justifications

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-01-28
**Review Status:** DRAFT
**Submitted for Review:** [Pending submission to coordinator]

**Reviewer:** tq-project-manager (or sprint-coordinator)
**Review Status:** PENDING
**Review Date:** [To be filled by reviewer]
**Review Comments:** [To be filled by reviewer]

**Approval means:**
- ✅ Test strategy derived from specifications (not assumptions)
- ✅ All required test types identified with clear rationale
- ✅ Coverage gaps explicitly identified and assessed
- ✅ Implementation plan is detailed and achievable
- ✅ Ready to proceed with test implementation

**Approval signature:** [tq-project-manager agent ID and timestamp]
