# Sprint 63 Test Strategy: Pager Exit Snapshot

**Created:** 2026-04-14
**Author:** quality-validator
**Sprint:** Sprint 63
**Features:**
1. `render_exit_snapshot()` method on `Pager` — prints last visible pager view to stdout as a static table after exiting

---

## Feature-by-Feature Test Strategy

### Feature: Pager Exit Snapshot (`render_exit_snapshot`)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-63-planning.md` Acceptance Criteria
- Requirements:
  1. "Pressing `q` or `Esc` in the pager prints a static table to stdout before returning to the REPL" (AC-1)
  2. "The static table matches the last pager viewport: same rows, same columns, same column offset" (AC-2)
  3. "Hidden columns are reported in a footer message: 'N columns hidden: col1, col2, ...'" (AC-3)
  4. "The 'Use --format csv or --format json to see all columns' hint is shown when columns are hidden" (AC-4)
  5. "Row count and timing are shown: 'N row(s) in set (X.XXXs)'" (AC-5)
  6. "The static output uses no ANSI color codes (plain text, like non-paged output)" (AC-6)
  7. "The static output uses the same box-drawing border characters as the pager" (AC-7)
  8. "No visual artifacts or terminal state issues after pager exit" (AC-8)
  9. "Existing pager navigation and rendering behavior unchanged" (AC-9)
  10. "Unit tests for the snapshot rendering logic" (AC-10)

**Feature Characteristics:**

**User Interaction Type:** Pure Logic (the `render_exit_snapshot` rendering method) + Interactive PTY (the `q`/`Esc` trigger within `Pager::run()`).

**Explanation:** The rendering method itself (`render_exit_snapshot(&self, writer: &mut impl Write)`) accepts a generic `Write` implementor and contains no terminal interaction — it is testable as pure logic by injecting a `Vec<u8>`. The integration of that call into `Pager::run()` (after `LeaveAlternateScreen`) requires a live terminal to fully exercise, but the correctness of the output content is fully covered by the pure-logic unit tests.

**Observable Behavior:**
- [x] Visual output in terminal (box-drawing borders, headers, data rows, footer lines)
- [ ] Structured data output
- [ ] File system side effects
- [ ] Database side effects
- [ ] Network interactions
- [ ] Performance characteristics
- [x] State management (viewport state: `col_offset`, `row_offset`, `page_size`, `term_width`)

**External Dependencies:**
- [ ] Database connection — NOT required for unit tests; the `Pager` is constructed from a `QueryResult` which can be built in-process
- [ ] File system access
- [ ] Network access
- [ ] Terminal/PTY — required only for integration/interactive tests that validate AC-1 (q/Esc trigger) and AC-8 (no terminal artifacts)
- [ ] None (pure logic unit tests use `Vec<u8>` as the writer)

**Validation Challenges:**
- The snapshot method must not emit any ANSI escape sequences. Absence of ANSI codes must be verified explicitly by asserting the output bytes contain no ESC (`\x1b`) characters.
- CJK (double-wide) characters require `unicode_width`-aware padding. The existing `pad_to_display_width()` helper handles this, but the snapshot tests must confirm the method uses it correctly (not `format!` width specifiers).
- The hidden columns footer lists column names in order; test must assert both the count and the exact names rendered.
- The row count / timing footer has a specific format `"N row(s) in set (X.XXXs)"` that must match exactly.
- The `\n` line ending (not `\r\n`) requirement differentiates this method from the live render methods (`render_border`, `render_row`, etc.) which use `\r\n` for raw mode.

**Critical Behaviors to Validate:**
1. "Same rows, same columns, same column offset" — viewport state (`col_offset`, `row_offset`, `page_size`) must determine exactly which data is rendered (AC-2, sprint-63-planning.md)
2. "N columns hidden: col1, col2, ..." — both the count and the column name list must be correct when `col_offset > 0` or when the visible column count does not reach the last column (AC-3)
3. "Row count and timing: 'N row(s) in set (X.XXXs)'" — exact format string, 3 decimal places (AC-5)
4. "No ANSI color codes" — snapshot output must contain zero `\x1b` bytes (AC-6)
5. "`\n` line endings, not `\r\n`" — each line must end with exactly `\n` not `\r\n` (AC-7 + technical design requirement)
6. "Same box-drawing border characters" — `╭`, `─`, `┬`, `╮`, `├`, `┼`, `┤`, `╰`, `┴`, `╯`, `│` (AC-7)
7. "No hidden-columns footer when all columns visible" — footer absent when `col_offset == 0` and `visible_column_count() == data.columns.len()` (AC-3 negative case)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Pure Logic" (render_exit_snapshot with Vec<u8>):
  → Unit tests REQUIRED
  Reason: No terminal dependency; output is fully deterministic given pager state

IF "Interactive PTY" (q/Esc trigger in Pager::run()):
  → Interactive tests (expectrl) POSSIBLE
  Reason: Only live PTY can confirm output appears after pager exits and screen is restored
  Decision: NOT REQUIRED for this sprint — the q/Esc integration in run() is a
  one-line change (add render_exit_snapshot call); the correctness of the content
  is 100% covered by unit tests. Interactive test would only add coverage for
  terminal state transitions (AC-8), which is already validated by existing pager
  interactive tests for q/Esc behavior.

IF "Visual output in terminal" (box-drawing):
  → Unit tests with string assertions REQUIRED
  Reason: Output is plain text to Vec<u8>; no PTY needed to validate borders/headers/rows
```

**Derived Test Types:**

**Test Type 1: Unit Tests (in-module `#[cfg(test)]`)**
- **Validates:** All rendering logic of `render_exit_snapshot`: correct viewport selection (rows + columns), box-drawing borders, header line, data rows, hidden-columns footer, row-count/timing footer, absence of ANSI codes, `\n` line endings, CJK-safe padding.
- **Approach:** Construct `Pager` instances directly from `QueryResult` using the existing `create_pager_with_width()` / `Pager::new()` pattern. Set `row_offset`, `col_offset`, `page_size`, `term_width` directly on the struct. Call `render_exit_snapshot(&mut buf)` with a `Vec<u8>` buffer. Convert to `String` and assert on content and structure.
- **Rationale:** The method's signature `(&self, writer: &mut impl Write)` was explicitly designed for testability. All 10 acceptance criteria except AC-1 (q/Esc trigger) and AC-8 (terminal state) are fully covered by unit tests alone.
- **Gap if missing:** All rendering bugs (wrong column range, wrong footer format, ANSI escapes, `\r\n` instead of `\n`) would only be found at runtime.
- **Necessity:** REQUIRED

**Test Type 2: Interactive Tests (expectrl)**
- **Validates:** AC-1 (pressing q in a live REPL session causes snapshot to appear on the original screen), AC-8 (no terminal artifacts).
- **Approach:** expectrl spawn tq REPL, run a query that triggers the pager, send `q`, assert snapshot table appears in subsequent output.
- **Rationale:** Only a live PTY can confirm the output reaches the original screen buffer (not the alternate screen).
- **Gap if missing:** `render_exit_snapshot` could be called inside the alternate screen (before `LeaveAlternateScreen`), making output invisible; or the terminal could be left in raw mode. Neither of these bugs is caught by unit tests.
- **Necessity:** RECOMMENDED — the implementation risk for the `run()` integration is low (the change is one call after `LeaveAlternateScreen`), and the existing interactive test infrastructure already tests pager exit. However, the risk of calling snapshot in the wrong terminal state is non-zero.

**Test Type 3: Integration Tests (live DB)**
- **Validates:** End-to-end `tq` → pager launch → q → snapshot visible.
- **Approach:** Would require a live DB to generate a result set large enough to trigger the pager.
- **Rationale:** Subsumed by interactive tests (expectrl with DB) if live DB is available.
- **Gap if missing:** No gap beyond what interactive tests cover.
- **Necessity:** NOT NEEDED as a separate category; covered under interactive tests if DB available.

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (Vec<u8> injection) | REQUIRED | Pure logic, 100% content coverage without PTY | All rendering bugs undetected | MUST IMPLEMENT |
| Interactive tests (expectrl + DB) | RECOMMENDED | Validates terminal state transition after run() | Wrong screen buffer, raw mode not disabled | IMPLEMENT if infra available |
| Benchmark tests | NOT NEEDED | No performance requirements | N/A | SKIP |
| Structural grep checks | RECOMMENDED | Verify `\r\n` removed from snapshot method body | Silent `\r\n` in static output | IMPLEMENT |

**Summary:**
- REQUIRED test types: 1 (unit tests) — MUST implement
- RECOMMENDED test types: 2 (interactive, structural grep) — should implement unless blocked
- NOT NEEDED test types: 1 (benchmark) — explicitly omitted

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (from sprint-63-planning.md) | Test Type(s) | Test Cases |
|----------------|-----------------------------------------------|--------------|------------|
| AC-1 | "Pressing q or Esc prints a static table to stdout before returning to the REPL" | Interactive (recommended) | TC-63-001 Part G |
| AC-2 | "Static table matches last pager viewport: same rows, same columns, same column offset" | Unit | TC-63-001 Parts A, B, C, D |
| AC-3 | "Hidden columns reported in footer: 'N columns hidden: col1, col2, ...'" | Unit | TC-63-001 Parts B, F |
| AC-4 | "'Use --format csv or --format json...' hint when columns hidden" | Unit | TC-63-001 Part F |
| AC-5 | "Row count and timing: 'N row(s) in set (X.XXXs)'" | Unit | TC-63-001 Part E |
| AC-6 | "No ANSI color codes (plain text)" | Unit | TC-63-001 Part H |
| AC-7 | "Same box-drawing border characters as pager" | Unit | TC-63-001 Parts A, B |
| AC-8 | "No visual artifacts or terminal state issues" | Interactive (recommended) | TC-63-001 Part G |
| AC-9 | "Existing pager navigation and rendering unchanged" | Unit (regression: existing tests pass) | Existing test suite |
| AC-10 | "Unit tests for snapshot rendering logic" | Unit | TC-63-001 (entire file) |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements
- [x] No unjustified test types

**Coverage Gaps:**
- AC-1 and AC-8 (terminal state) are not covered by unit tests — interactive tests are RECOMMENDED to cover them; if interactive tests are not executed, the risk is LOW because the integration code path is a single function call after `LeaveAlternateScreen` which is already tested by existing pager exit tests.

#### 5. Gap Analysis

**Interactive/PTY Tests**
- **Reason for possible omission:** Interactive tests require an active PTY environment and a DB connection. The DB may not be available in CI.
- **What won't be validated:** That `render_exit_snapshot` is called after (not before) `LeaveAlternateScreen`; that the terminal is not left in raw mode; that the snapshot is visible on the primary screen.
- **Risk assessment:** LOW — The integration wiring is a one-line change; unit tests validate all content correctness; existing pager q/Esc tests confirm clean terminal exit.
- **Mitigation:** Manual smoke test on developer workstation after implementation.
- **Revisit criteria:** If users report the snapshot appearing on the wrong screen or terminal being left in raw mode.

**Performance/Benchmark Tests**
- **Reason for omission:** No performance requirement exists for snapshot rendering in the specification.
- **Risk:** LOW
- **Mitigation:** None needed.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/repl/pager.rs` — `#[cfg(test)]` module at bottom of file
- **Framework:** Built-in Rust `#[test]` (same framework used by all existing pager tests)
- **Test count estimate:** 9 unit tests
- **Key scenarios:**
  1. TC-63-001-A: Basic snapshot — small result (3 cols, 5 rows), all columns visible, row_offset=0 — verify top border, header, 5 data rows, bottom border, row count footer, no hidden-columns footer
  2. TC-63-001-B: Horizontal scroll — result with many columns, `col_offset=2` — verify only columns from offset visible, hidden-columns footer lists columns 0 and 1 by name with correct count
  3. TC-63-001-C: Vertical scroll — result with many rows, `row_offset=3`, `page_size=4` — verify exactly 4 rows rendered starting from row 3, not row 0
  4. TC-63-001-D: Both offsets — col_offset=1, row_offset=2, page_size=3 — verify correct 3-row, reduced-column viewport
  5. TC-63-001-E: Row count and timing footer format — exact string match "N row(s) in set (X.XXXs)" including 3 decimal places
  6. TC-63-001-F: Hidden columns footer format — exact string match "N columns hidden: col1, col2, ..." and "Use --format csv or --format json to see all columns" hint
  7. TC-63-001-G: No hidden-columns footer when all visible — verify footer absent when col_offset=0 and all columns fit
  8. TC-63-001-H: No ANSI codes — assert output bytes contain no `\x1b` (0x1B) byte
  9. TC-63-001-I: Line endings are `\n` not `\r\n` — assert no `\r\n` sequence in output
  10. TC-63-001-J: CJK/Unicode — column with CJK header and data — verify output lines have consistent visual width using `UnicodeWidthStr::width`

- **Mocking strategy:** No mocks needed. `Pager` is constructed from a `QueryResult` (built in-process). The `Vec<u8>` buffer is the writer. `term_width` and `page_size` are set directly on the struct fields (as done in existing `create_pager_with_width()` helper).

**Test Type: Structural Grep Check**
- **Location:** Test execution phase
- **Commands:**
  ```bash
  # Verify render_exit_snapshot method exists
  grep -n "render_exit_snapshot" src/commands/repl/pager.rs

  # Verify no \r\n in snapshot method (plain text, not raw mode)
  # The method body should use \n, not \r\n
  grep -n "\\\\r\\\\n" src/commands/repl/pager.rs
  ```
- **Test count:** 2 checks

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: viewport selection (AC-2), border characters (AC-7), all footer formats (AC-3, AC-4, AC-5), ANSI-free output (AC-6), `\n` line endings (technical requirement), CJK rendering
- Structural grep validates: method exists, no `\r\n` in snapshot body
- Interactive tests (if run) validate: AC-1 trigger, AC-8 terminal state
- Combined coverage: **adequate** — all content acceptance criteria are fully covered by unit tests; only terminal-state integration (AC-1, AC-8) requires interactive tests which are recommended

**Gaps in combined coverage:**
- If interactive tests are not executed: AC-1 and AC-8 are not machine-validated. This is acceptable given the low implementation risk of the integration point.

**Acceptance criteria:**
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified" (with noted gap for AC-1/AC-8 if interactive tests omitted)
- [x] Known gaps are documented and accepted

---

## Strategy Summary

**Total Features Analyzed:** 1

**Test Types Required:**
- Unit tests: REQUIRED — `render_exit_snapshot` with `Vec<u8>` writer
- Structural grep checks: RECOMMENDED — verify method signature, no `\r\n`
- Interactive tests (expectrl): RECOMMENDED — AC-1 / AC-8 terminal state validation
- Benchmark tests: NOT NEEDED

**Estimated Test Count:**

| Category | Count |
|----------|-------|
| Unit tests (new, in pager.rs `#[cfg(test)]`) | 10 |
| Structural grep checks | 2 |
| Interactive tests (recommended, DB required) | 1 scenario |
| **Total new tests** | **10 unit + 2 grep + 1 interactive** |

**Risk Assessment:**
- HIGH risk gaps: None
- MEDIUM risk gaps: None
- LOW risk gaps: AC-1/AC-8 terminal state if interactive tests are omitted (mitigated by existing pager exit tests and manual smoke test)

**Dependencies Required:**
- Live database: No (unit tests); Yes (interactive tests only, optional)
- Network access: No
- Specific OS: No
- Other: None

---

## Strategy Validation Checklist

- [x] Every feature has complete specification analysis section
- [x] Feature characteristics are classified (not assumed)
- [x] Test strategy is derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest
- [x] Specification coverage map includes all requirements
- [x] Every requirement maps to at least one test type
- [x] Test implementation plan is detailed and actionable
- [x] Coverage sufficiency is assessed
- [x] No hand-waving or vague justifications

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-04-14
**Review Status:** DRAFT
**Submitted for Review:** 2026-04-14
