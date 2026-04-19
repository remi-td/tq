# Sprint 67 Test Strategy: Search in Pager

**Created:** 2026-04-19
**Author:** quality-validator
**Sprint:** Sprint 67
**Features:**
1. Feature 1 (P0): Forward search in pager — 12 ACs
2. Feature 2 (P1): `handle_tick_result` extraction + unit test — 4 ACs

---

## Feature-by-Feature Test Strategy

### Feature 1: Forward search in pager

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-67-planning.md` — Feature 1 Acceptance Criteria (AC-1..AC-12)
- Secondary: `docs/specifications/repl.md#pager-search` — REQ-PAGER-SEARCH-001..012 (to be authored by cli-ux-designer in Phase 2)
- Reference implementation context: `src/commands/repl/pager.rs` — `TableData`, `Pager`, `render_row`, `handle_key`, `render_status_bar`, `show_help`

**Requirements (from planning doc):**
1. AC-1: Pressing `/` shows prompt in status bar; typed characters appear; backspace deletes; ENTER submits; Esc cancels and restores prior search.
2. AC-2: Submitting a matching pattern scrolls to first match at or after current row; matched substring is highlighted.
3. AC-3: Submitting a non-matching pattern shows `Pattern: <pat>  not found`; no scroll.
4. AC-4: `n` advances to next match; wraps on end-of-results with status `wrapped to first match`.
5. AC-5: `N` retreats to previous match; wraps on start-of-results with status `wrapped to last match`.
6. AC-6: Search is case-insensitive by default; `\c` suffix makes it case-sensitive.
7. AC-7: If the matched cell is outside the horizontal viewport, the pager scrolls horizontally so it is in view.
8. AC-8: Matched substring is rendered with reversed foreground/background inside the cell.
9. AC-9: Status bar shows `Pattern: <pat>  (M matches)` after a successful search.
10. AC-10: Search works when the result set has more rows than `page_size` — matches in rows beyond the initial viewport are found.
11. AC-11: `q`/`Esc` to exit the pager while search is active behaves identically to normal exit — no terminal-state corruption.
12. AC-12: `?` help overlay documents `/pattern`, `n`, `N` alongside existing navigation keys.

**Feature Characteristics:**

**User Interaction Type:**
- [x] Interactive PTY (REPL pager driven by keypresses, renders to an alternate screen)
- [x] Pure Logic (search scanning, match list pre-computation, case-folding — callable from unit tests if extracted as pure functions)

**Explanation:** The pager is a full-screen TUI driven by `crossterm` key events. The search prompt input loop, match scanning, `n`/`N` navigation, and status-bar rendering are all triggered by interactive keypresses against a live PTY. However, the architect is expected to extract the core search logic into a pure function (e.g., `search_result_set(cells, pattern, case_sensitive) -> Vec<(row_idx, col_idx, Range<usize>)>`) to allow unit-level testing without a PTY. This is the same pattern the pager already follows for `render_to_buffer` and `render_exit_snapshot`.

**Observable Behavior:**
- [x] Visual output in terminal — highlight rendering (reversed colors), status-bar format strings
- [x] State management — search state (`pattern`, `matches: Vec<(row_idx, col_idx, char_range)>`, `match_cursor: usize`) added to `Pager`
- [x] Performance characteristics — match pre-computation amortises cost of `n`/`N` (implied by planning doc Risk 2)

**External Dependencies:**
- [x] Terminal/PTY — prompt input loop, highlight rendering, `n`/`N` navigation (AC-1, AC-7, AC-8, AC-11)
- [x] None (pure logic) — search scanning, case-folding, match count, navigation index arithmetic (AC-2 through AC-6, AC-9, AC-10, AC-12 help text)

**Validation Challenges:**
1. **Prompt input loop** (AC-1): The `/` keypress must open a prompt in the status bar. This is a stateful TUI interaction — impossible to validate without a PTY or a very narrow unit test against `handle_key` with simulated key events.
2. **Reversed-color rendering** (AC-8): Terminal color reversal (`crossterm::style::Attribute::Reverse`) is an ANSI escape code. Unit tests writing to a `Vec<u8>` can assert the escape sequence is present; interactive tests confirm the visual result in a real PTY. Neither approach is a substitute for the other.
3. **Exit safety** (AC-11): Raw-mode teardown must happen correctly when exiting the pager with an active search. The existing `RawModeGuard`-style pattern in `pager.rs` (inferred from `watch.rs`) handles this, but the only reliable validation is an interactive test that enters a search, then quits and confirms the terminal is usable afterward.
4. **Column scrolling on match** (AC-7): Horizontal scrolling to a matched column requires the match to be outside the visible window. This is testable in a unit test if `Pager`'s viewport logic (`col_offset`, `visible_column_count`) is accessible, or in an interactive test with a wide result set.

**Critical Behaviors to Validate:**
1. "Scans in-memory result set from current row forward for first row whose displayed cell text contains the pattern" — pure function testable with a mock `ResultSet`.
2. "Highlights matched substring with reversed colors" — ANSI escape sequence assertion (unit-level), visual check (interactive).
3. "Status bar shows `Pattern: <pat>  (M matches)` or `Pattern: <pat>  not found`" — assertable as a string in a unit test against `render_status_bar` output.
4. "No terminal-state corruption on exit with active search" — requires PTY test to confirm REPL prompt returns cleanly.

---

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" checked:
  → Interactive tests (PTY harness) REQUIRED for AC-1, AC-11.
  Reason: prompt input loop and exit-safety cannot be validated without a live PTY.

IF "Pure Logic" checked:
  → Unit tests REQUIRED for AC-2, AC-3, AC-4, AC-5, AC-6, AC-9, AC-10, AC-12.
  Reason: search scanning, match counting, navigation index arithmetic, and
  help-text content are deterministic and database-free.

IF "Visual output in terminal" checked:
  → Unit tests with ANSI escape assertion for AC-8 (reversed color).
  Reason: `Vec<u8>` writer can capture crossterm escape sequences; no PTY needed
  to assert the bytes are present.

IF "Performance characteristics" checked:
  → No benchmark tests REQUIRED. Pre-computation is an architect design choice;
  functional unit tests exercise the function but do not time it. No SLA defined
  in the planning doc. Risk is low (planning doc Risk 2).
```

**Derived Test Types:**

**Test Type 1: Unit Tests (pure-function extraction)**
- **Validates:** AC-2 (forward scan logic), AC-3 (no-match detection), AC-4 (`n` navigation index arithmetic), AC-5 (`N` navigation index arithmetic), AC-6 (case folding), AC-9 (match count), AC-10 (multi-page result navigation).
- **Approach:** Feed a `QueryResult` fixture (see §5 Test Data) directly to the extracted pure search function `search_result_set(...)`. Assert the returned `Vec<(row_idx, col_idx, Range<usize>)>` against known expected matches. Drive navigation index arithmetic with simple index increment/decrement functions.
- **Rationale:** No PTY or database required. Fast and deterministic. This is the primary coverage mechanism for 7 of 12 ACs.
- **Gap if missing:** Core search logic bugs (off-by-one in scan, wrong case comparison) would only surface in interactive tests, making them much harder to diagnose.
- **Necessity:** REQUIRED

**Test Type 2: Unit Tests (status-bar and help-text string assertions)**
- **Validates:** AC-9 (status-bar format strings), AC-12 (help-text content), AC-3 (no-match status string).
- **Approach:** Call `render_status_bar_to_buffer(...)` (writer-injected variant, following the existing `render_to_buffer` / `render_exit_snapshot` pattern) and assert the string contains `Pattern: <pat>  (M matches)`, `Pattern: <pat>  not found`, or the help overlay lines `/pattern`, `n`, `N`. If the architect does not expose a writer-injected variant, assert via the rendered `String`.
- **Rationale:** Status-bar format strings are hard-coded constants. A pure string assertion is far faster and less flaky than an interactive PTY check.
- **Gap if missing:** Format-string typos (e.g., wrong spacing in `  (M matches)`) would silently pass.
- **Necessity:** REQUIRED

**Test Type 3: Unit Tests (ANSI escape / reversed-color assertion)**
- **Validates:** AC-8 (reversed foreground/background on matched substring).
- **Approach:** Call the cell-rendering path with a known pattern match, writing to `Vec<u8>`. Assert the bytes contain the crossterm `Attribute::Reverse` escape sequence (`\x1b[7m`) before the matched substring and the reset sequence (`\x1b[0m` or `\x1b[27m`) after it.
- **Rationale:** The escape bytes are deterministic and do not require a PTY. This is the only unit-level proof that highlighting is wired to render output.
- **Gap if missing:** If highlight is accidentally a no-op, no other unit test detects it.
- **Necessity:** REQUIRED (can be merged with the status-bar unit test file)

**Test Type 4: Interactive Tests (PTY harness — Stage::Prompt and Stage::Connect)**
- **Validates:** AC-1 (prompt opens on `/`, accepts typed characters, Esc cancels), AC-11 (exit while search is active does not corrupt terminal state).
- **Approach:** Use `TqPty` from `tests/common/pty_harness.rs`. Connect to the REPL (`Stage::Connect`), execute a SQL query that returns a result set large enough to page, expect the pager prompt (`Stage::Prompt` / `Stage::Query`), send `/`, expect the search prompt marker in the PTY output, send a pattern + `\n`, expect the status-bar match string, then send `q` and expect the REPL prompt to return cleanly.
- **Rationale:** AC-1 and AC-11 require a live PTY. Unit tests cannot simulate the terminal event loop.
- **Gap if missing:** The prompt interaction and exit-safety ACs have no automated coverage.
- **Necessity:** REQUIRED (but fallback to MANUAL if PTY test is persistently flaky — see Gap Analysis)

**Test Type 5: Manual (visual highlight verification)**
- **Validates:** AC-8 (visual appearance of reversed-color highlight in a real terminal).
- **Approach:** Developer runs `tq` in the REPL, executes a query, enters the pager, types `/pattern`, and visually confirms the matched substring appears with inverted colors.
- **Rationale:** The ANSI unit test (Test Type 3) proves the escape bytes are emitted. Visual confirmation is the only way to ensure the terminal interprets them as the intended visual reversal.
- **Gap if missing:** The bytes could be correct but display incorrectly on a specific terminal emulator.
- **Necessity:** RECOMMENDED (manual, documented in test cases)

---

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit — pure search function | REQUIRED | Core logic is pure; fast, no DB | Logic bugs invisible until interactive tests | MUST IMPLEMENT |
| Unit — status-bar strings | REQUIRED | Format strings are deterministic constants | Typos in status strings silently pass | MUST IMPLEMENT |
| Unit — ANSI escape assertion | REQUIRED | Highlight wiring is a hard boolean | Highlight accidentally no-op goes undetected | MUST IMPLEMENT |
| Interactive PTY — prompt + exit | REQUIRED | AC-1 / AC-11 are not unit-testable | Prompt loop and terminal safety uncovered | MUST IMPLEMENT (fallback: MANUAL if flaky) |
| Manual — visual highlight | RECOMMENDED | Terminal color interpretation varies | Visual regression on specific terminal emulators | DOCUMENT + PERFORM ONCE |
| Benchmark | NOT NEEDED | No performance SLA defined | N/A | SKIP |

---

#### 4. Per-AC Test-Type Assignment

| AC | Description (short) | Test Type | Stage / Function | Justification |
|----|---------------------|-----------|-----------------|---------------|
| AC-1 | `/` opens prompt, input loop, Esc cancels | Interactive (PTY) | Stage::Prompt | Prompt input loop is a TUI event loop; cannot be driven without PTY |
| AC-2 | Forward scan finds first match, scrolls to it | Unit (pure fn) | `search_result_set(...)` | Match scan is pure; scroll result tested via state assertions |
| AC-3 | No match → status `not found`, no scroll | Unit (pure fn + status str) | `search_result_set(...)` + `render_status_bar` | Both the empty-match list and the format string are pure |
| AC-4 | `n` → next match, wraps with status | Unit (navigation fn) | index arithmetic fn | Wrap logic and status string are pure; no TUI needed |
| AC-5 | `N` → previous match, wraps with status | Unit (navigation fn) | index arithmetic fn | Same as AC-4, opposite direction |
| AC-6 | Case-insensitive default; `\c` → case-sensitive | Unit (pure fn) | `search_result_set(...)` with flag | Case folding is a string operation; pure test sufficient |
| AC-7 | Match outside viewport → horizontal scroll | Hybrid | Unit for col_offset logic + optional Interactive smoke | col_offset adjustment is pure state mutation; smoke confirms end-to-end |
| AC-8 | Reversed-color highlight on matched substring | Hybrid | Unit (ANSI bytes) + Manual (visual) | Escape bytes are deterministic; visual appearance needs human eye |
| AC-9 | Status bar `Pattern: <pat>  (M matches)` | Unit (status str) | `render_status_bar` writer-injected | Format string is a constant; assertable on `Vec<u8>` |
| AC-10 | Search finds matches beyond initial viewport | Unit (pure fn) | `search_result_set(...)` on large fixture | Row-count > page_size fixture is a data construction, not a PTY concern |
| AC-11 | Exit with active search does not corrupt terminal | Interactive (PTY) | Stage::Connect → Stage::Prompt → quit | RAII guard behaviour only verifiable when a real PTY session ends |
| AC-12 | `?` help overlay documents `/`, `n`, `N` | Unit (help text str) | `show_help` writer-injected or `render_to_buffer` | Help text is a static string literal; no PTY needed |

---

#### 5. Specification Coverage Map

| Requirement | AC | Test Type(s) | Test Cases (Phase 3 IDs TBD) |
|-------------|----|-------------|-------------------------------|
| REQ-PAGER-SEARCH-001 | AC-1 | Interactive PTY | TC099-I01 |
| REQ-PAGER-SEARCH-002 | AC-2 | Unit (pure fn) | TC099-U01 |
| REQ-PAGER-SEARCH-003 | AC-3 | Unit (pure fn + str) | TC099-U02 |
| REQ-PAGER-SEARCH-004 | AC-4 | Unit (nav fn) | TC099-U03 |
| REQ-PAGER-SEARCH-005 | AC-5 | Unit (nav fn) | TC099-U04 |
| REQ-PAGER-SEARCH-006 | AC-6 | Unit (pure fn) | TC099-U05 |
| REQ-PAGER-SEARCH-007 | AC-7 | Hybrid (Unit + opt. Interactive) | TC099-U06 + TC099-I02 (optional) |
| REQ-PAGER-SEARCH-008 | AC-8 | Hybrid (Unit ANSI + Manual) | TC099-U07 + Manual |
| REQ-PAGER-SEARCH-009 | AC-9 | Unit (str) | TC099-U08 |
| REQ-PAGER-SEARCH-010 | AC-10 | Unit (pure fn) | TC099-U09 |
| REQ-PAGER-SEARCH-011 | AC-11 | Interactive PTY | TC099-I01 (exit subcase) |
| REQ-PAGER-SEARCH-012 | AC-12 | Unit (str) | TC099-U10 |

---

#### 6. Gap Analysis

**AC-1 and AC-11 — Interactive-only, PTY-dependent:**
- The planning doc explicitly acknowledges these may fall back to manual verification if the PTY test is flaky.
- This assessment is CONFIRMED. The prompt input loop (AC-1) and the exit-safety guarantee (AC-11) both require live PTY + database. There is no pure-Rust stub that simulates `crossterm::event::read()` returning `KeyEvent` sequences without a PTY.
- Mitigation: Author one PTY test (`TC099-I01`) that covers both AC-1 (opens prompt, types pattern, submits) and AC-11 (then exits with `q`, checks REPL returns). This is the Sprint 66 pattern for `test_repl_startup_and_quit`. If this test times out, document as `run and failed` and add a manual verification note. Do NOT mark as `skipped for coverage convenience` — the reason must be stated explicitly.

**AC-7 — Hybrid: unit is the primary signal, interactive is optional:**
- Unit test can verify `col_offset` is set to the expected value when a match is in an off-screen column, by examining `Pager` state after calling the navigation handler. This requires the architect to expose `col_offset` (or accept it as a test accessor) and the col-scroll logic to be triggered by the match-jump function.
- The interactive smoke (TC099-I02) is listed as optional. If the unit test adequately covers the col_offset arithmetic, the interactive test is redundant. Decision deferred to Phase 3 based on what the architect exposes.

**AC-8 — ANSI escape unit test is necessary but not sufficient:**
- Unit test (TC099-U07) asserts `\x1b[7m` (or equivalent crossterm `Attribute::Reverse`) is emitted before the matched substring. This is machine-verifiable.
- Visual appearance in a real terminal is not machine-verifiable. Manual verification step must be performed once by a developer during Phase 3. This is explicitly documented as MANUAL.
- Risk: LOW. Reversed-color is a well-known ANSI attribute; if the bytes are correct, the visual will be correct on any VT100-compatible terminal.

**AC-12 — Help text is pure but requires architect cooperation:**
- `show_help` currently waits for a keypress inside a loop (`event::poll` + `event::read`). A writer-injected variant analogous to `render_border_plain` is needed so a unit test can capture the help text without blocking.
- If the architect does not extract a testable `render_help_to_buffer` (or similar), AC-12 will degrade to a `grep` check against the literal string in `pager.rs` source, which is NOT execution. The architect must be made aware of this requirement.
- Risk: MEDIUM (if not addressed, AC-12 has no automated test). Mitigation: flag this requirement explicitly in the Phase 3 test-case doc.

---

#### 7. Test Implementation Plan

**Test Type: Unit Tests (pure search function)**
- **Location:** `src/commands/repl/pager.rs` — `#[cfg(test)]` module
- **Framework:** Built-in Rust `#[test]`
- **Estimated test count:** 9 unit tests (TC099-U01 through TC099-U09)
- **Key scenarios:**
  1. TC099-U01: `search_result_set` on a 5-row fixture containing `"Hello"` — asserts returned matches include `(row_idx, col_idx, range)` for the exact character positions.
  2. TC099-U02: `search_result_set` with a pattern that matches nothing — asserts empty `Vec`; status-bar string is `Pattern: xyz  not found`.
  3. TC099-U03: Navigation function `next_match(matches, current_row)` — three sub-cases: match after current row (returns it), last match (wraps to index 0, status `wrapped to first match`), empty matches (returns `None`).
  4. TC099-U04: `prev_match(matches, current_row)` — three sub-cases: match before current row, first match (wraps to last, status `wrapped to last match`), empty matches.
  5. TC099-U05: Case-insensitive default — `/foo` matches `"Foo"`, `"FOO"`, `"foo"`. Case-sensitive mode (`\c`) — `/Foo\c` matches only `"Foo"`, not `"foo"` or `"FOO"`.
  6. TC099-U06: Column-scroll logic — given a `Pager` with `col_offset=0` and 5 columns, a match in column 4 (outside a 2-column viewport) causes `col_offset` to advance to 4 (or the correct scroll position) after `jump_to_match(...)`.
  7. TC099-U07: Cell render with match range — rendered `Vec<u8>` contains `\x1b[7m` (reverse-video on) before the matched substring bytes and `\x1b[0m` or `\x1b[27m` (reset) after.
  8. TC099-U08: Status-bar string with 3 matches — rendered string contains `Pattern: foo  (3 matches)`.
  9. TC099-U09: Multi-page fixture — fixture with 50 rows, `page_size=10`, match in row 42. `search_result_set` returns the row-42 match; `jump_to_match` sets `row_offset` to 42 (or the page-aligned position that makes row 42 visible).
- **Mocking strategy:** No mocks. Use `QueryResult::new(...)` with `Value::String(...)` and `Value::Null` and `Value::Integer(...)` rows (see §5 Test Data). No DB or PTY.

**Test Type: Unit Test (help-text assertion)**
- **Location:** `src/commands/repl/pager.rs` `#[cfg(test)]`
- **Framework:** Built-in Rust `#[test]`
- **Estimated test count:** 1 (TC099-U10)
- **Key scenario:** Call `render_help_to_buffer()` (writer-injected variant required from architect), assert output contains the strings `/pattern`, `n`, `N`.
- **Fallback if not extracted:** `grep -n "/pattern" src/commands/repl/pager.rs` as a structural check. This is weaker than execution but the only alternative without architect cooperation. Mark explicitly as `skipped for reason: writer-injected variant not provided`.

**Test Type: Interactive Tests (PTY harness)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** `TqPty` from `tests/common/pty_harness.rs` (Sprint 66 tiered harness)
- **Estimated test count:** 1 mandatory (TC099-I01), 1 optional (TC099-I02)
- **Key scenarios:**
  - TC099-I01 (mandatory): Connect (`Stage::Connect`); send SQL query returning at least 30 rows; wait for pager prompt (`Stage::Prompt`); send `/hello\n` (pattern known to exist in fixture data); expect `Pattern: hello` in PTY output (`Stage::Query`); send `q`; expect REPL prompt marker. Covers AC-1 (prompt opened, text typed, submitted) and AC-11 (clean exit).
  - TC099-I02 (optional): As TC099-I01 but with a wide result set where the matched column is off-screen; send `n` after initial search; confirm PTY output contains the match indicator. Covers AC-7 (horizontal scroll on match). Author only if AC-7 unit test is insufficient.
- **Implementation notes:** Use `Stage::Query` for the expect-after-pattern-submit, because rendering the full match results may take slightly longer than a prompt. Set `TQ_TEST_QUERY_TIMEOUT` to at least 30 s for the search render. The test name must be stable (used in PTY log filename by harness).

---

#### 8. Coverage Sufficiency Assessment

**If all planned tests pass:**
- Unit tests validate: match scan correctness, case folding, navigation arithmetic, wrap behavior, status-bar format strings, ANSI highlight bytes, multi-page result traversal, column-scroll state.
- Interactive tests validate: prompt input loop opens and accepts characters, clean exit from pager with active search.
- Combined coverage: adequate. The one genuine gap (visual color appearance) is explicitly accepted as a one-time manual check.

**Known acceptable gap:** AC-11 terminal-corruption safety is validated only by the interactive test succeeding and the REPL prompt returning — it does not run `reset` and verify the tty flags. This is the same level of proof used for all existing pager-exit tests. Risk: LOW.

---

### Feature 2: `handle_tick_result` extraction + unit test

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-67-planning.md` — Feature 2 Acceptance Criteria (AC-1..AC-4)
- Secondary: `docs/sprints/sprint-65-review.md` → Follow-Up Items → second P2 item

**Requirements:**
1. AC-1: `handle_tick_result` is a pure function (no I/O, no global state); returns both display output and retained body.
2. AC-2: Existing `/sessions --watch`, `/locks --watch`, `/resources --watch` behaviour is byte-identical to pre-extraction.
3. AC-3: Unit test `test_handle_tick_result_error_retains_last_body` feeds `Err(...)` and asserts retained body = `last_body` and error line contains the error message.
4. AC-4: Unit test `test_handle_tick_result_success_replaces_body` feeds `Ok(new)` and asserts retained body becomes `new`.

**Feature Characteristics:**

**User Interaction Type:**
- [x] Pure Logic — extraction of a `match render_result { ... }` block into a pure function; no terminal interaction.

**Observable Behavior:**
- [x] State management — what `last_body` holds after each tick.

**External Dependencies:**
- [x] None — pure function unit tests, no DB or PTY.

**Validation Challenges:**
1. **AC-2 (behavioral identity):** Byte-identical behavior with watch commands requires either an interactive test (PTY + DB) or a careful code review confirming the extracted function is called with the same arguments and its output is used identically. Full interactive proof would need live `/sessions --watch` to run for 2+ ticks and compare output, which is expensive. Proposed mitigation: validate AC-2 through code inspection + the existing TC097 interactive tests (which cover `/sessions --watch` end-to-end and will catch regressions if behavior changes).

#### 2. Per-AC Test-Type Assignment

| AC | Description | Test Type | Justification |
|----|-------------|-----------|---------------|
| AC-1 | `handle_tick_result` is a pure function | Unit (compilation check + AC-3/AC-4 tests) | Purity is proved by calling it with no I/O in unit tests |
| AC-2 | Byte-identical watch behavior | Manual (code inspection) + regression guard via TC097 re-run | Interactive test would need live DB for 2+ ticks; TC097 already covers the end-to-end path |
| AC-3 | Error path retains `last_body`, formats error line | Unit | Pure function: feed `Err(...)`, assert outputs |
| AC-4 | Success path replaces `last_body` | Unit | Pure function: feed `Ok(new)`, assert retained body = `new` |

#### 3. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/watch.rs` — `#[cfg(test)]` module
- **Framework:** Built-in Rust `#[test]`
- **Estimated test count:** 2 (the two explicitly named in the ACs)
- **Key scenarios:**
  1. `test_handle_tick_result_error_retains_last_body`: construct `last_body = b"previous output".to_vec()`; call `handle_tick_result(Err(some_error), last_body.clone())`; assert `TickOutcome::retained_body == last_body`; assert `TickOutcome::display_line` contains the error message string.
  2. `test_handle_tick_result_success_replaces_body`: construct `new_body = b"new output".to_vec()`; call `handle_tick_result(Ok(new_body.clone()), b"old".to_vec())`; assert `TickOutcome::retained_body == new_body`.
- **Mocking strategy:** None — `handle_tick_result` takes `Result<Vec<u8>, crate::error::TqError>` and `Vec<u8>`; both can be constructed directly.

#### 4. Gap Analysis

**AC-2 — Behavioral identity not machine-verified end-to-end:**
- A pure extraction that returns the same values and is called with the same arguments is semantically equivalent. The unit tests prove the function computes correctly. Full byte-identical interactive verification would require running `/sessions --watch` for multiple ticks before and after, which is expensive and fragile.
- Risk: LOW — this is a pure refactor with no logic change. The architect writes the extraction; the unit tests verify the extracted function's logic. TC097 interactive tests serve as a regression guard for the watch behavior.
- Mitigation: Explicitly note in Phase 3 report that AC-2 is verified by: (a) unit tests covering the function logic, (b) passing `cargo test --lib` and `cargo test --test interactive_tests -- --ignored` (TC097).

---

## Harness Strategy

### Which ACs need live DB?

| AC | Live DB Required? | Reason |
|----|-------------------|--------|
| Feature 1 AC-1 | YES | PTY test needs a real REPL session to open the pager |
| Feature 1 AC-2 through AC-6, AC-9, AC-10, AC-12 | NO | Pure unit tests against in-memory fixture |
| Feature 1 AC-7 | NO (unit) / YES (optional interactive smoke) | Unit covers col_offset arithmetic; interactive optional |
| Feature 1 AC-8 | NO (ANSI bytes unit) / NO (manual = developer run, not automated) | |
| Feature 1 AC-11 | YES | PTY test for exit safety |
| Feature 2 AC-1, AC-3, AC-4 | NO | Pure unit tests |
| Feature 2 AC-2 | YES (via TC097 regression guard) | Existing interactive tests cover watch behavior |

**Summary:** 9 of 16 ACs are covered by unit tests that require no live DB. 2 ACs (AC-1, AC-11) require a live DB + PTY. 1 AC (AC-2 F2) piggybacks on existing TC097 which requires a live DB. 4 ACs have hybrid or optional interactive coverage.

### PTY Harness Sufficiency

The Sprint 66 tiered PTY harness (`tests/common/pty_harness.rs`) is **sufficient** for Sprint 67's interactive tests. No new harness infrastructure is needed. Specifically:
- `TqPty::expect_stage(Stage::Connect, ...)` covers the connection wait.
- `TqPty::expect_stage(Stage::Prompt, ...)` covers waiting for the REPL prompt.
- `TqPty::expect_stage(Stage::Query, ...)` covers waiting for pager output or search-result status.
- `TqPty::session_mut().send(...)` covers sending individual characters (e.g., `/`, pattern characters, `\n`, `q`).

The existing `Stage` enum already supports the three required phases. No new `Stage` variants are needed.

---

## Honest Evidence Convention

Per Sprint 65 lesson, the Phase 3 execution report (`tests/results/sprint-67/REPORT.md`) MUST categorize every test result using exactly one of these four labels:

| Label | Meaning |
|-------|---------|
| **run and passed** | Test was executed; cargo test reports success |
| **run and failed** | Test was executed; cargo test reports failure or panic |
| **not run** | Test exists in source but was not executed (e.g., wrong `--test` flag) |
| **skipped for reason X** | Test was deliberately not executed; reason must be stated explicitly (e.g., "skipped because no PTY harness was available in CI", "skipped because architect did not expose writer-injected help variant") |

No test may be labeled APPROVED by code review alone. If a test cannot be run, it is either `not run` or `skipped for reason X` — both are BLOCKING for APPROVED verdict.

---

## Test Data (Mock Fixture)

The unit tests require a `QueryResult` fixture that exercises all relevant search scenarios. The following construction is sufficient for all Feature 1 unit tests:

```
Fixture: 5 rows x 3 columns

Columns:
  - "id"     TeradataType::Integer,  nullable=false
  - "name"   TeradataType::Varchar,  nullable=true
  - "status" TeradataType::Varchar,  nullable=true

Rows (Value::display() output shown):
  row 0: [1,    "Hello World",    "active"]
  row 1: [2,    NULL,             "HELLO"]          <- NULL in col 1, uppercase in col 2
  row 2: [3,    "hello again",    "inactive"]       <- lowercase match
  row 3: [4,    "Unrelated text", "Foobar"]         <- no match for "hello"
  row 4: [100,  "Final HELLO",    NULL]             <- match in col 1, NULL in col 2
```

**Why this fixture covers all unit test scenarios:**
- `search_result_set("hello", case_insensitive)` returns matches in rows 0, 2, 4 (col 1), and row 1 col 2 — tests AC-2, AC-4, AC-5, AC-9 (4 matches).
- `search_result_set("nomatch", ...)` returns empty Vec — tests AC-3.
- `search_result_set("Hello", case_sensitive=true)` returns only row 0 col 1 — tests AC-6.
- `search_result_set("HELLO", case_sensitive=true)` returns row 1 col 2 and row 4 col 1 — tests AC-6 case-sensitive distinct from insensitive.
- NULL values in col 1 row 1 and col 2 row 4 are represented as `"[NULL]"` by `Value::display()`; searching `"[NULL]"` should match — confirms search operates on displayed text, not raw values (per spec: "search operates on the displayed cell text").

**For AC-10 (multi-page):** Extend the fixture to 50 rows by repeating the pattern (rows 0-4 repeated 10x). `page_size = 10`. Assert that `search_result_set` finds the match in row 42 and `jump_to_match` sets `row_offset` appropriately.

**For AC-7 (column scroll):** Use a fixture with 8 columns and a terminal width that only shows 3. Match in column 7 (off-screen). Assert `col_offset` advances to expose column 7.

---

## Tool Requests for Coordinator

**None.** All infrastructure exists:
- Built-in Rust `#[test]` for unit tests.
- `expectrl` (already in `[dev-dependencies]`) for PTY tests.
- `tests/common/pty_harness.rs` (Sprint 66 tiered harness) — sufficient as-is.
- `tempfile` (already in `[dev-dependencies]`) — not needed for Feature 1 unit tests but available if needed.

**Architect cooperation required (not a new tool):**
1. The search scan function must be extractable as a `pub(crate)` or `pub` function (even if only `pub` under `#[cfg(test)]`) so unit tests can call it directly.
2. A writer-injected variant of `render_status_bar` or a `render_status_bar_to_string(...)` method is needed for AC-9 string assertion without PTY.
3. A writer-injected variant of `show_help` (e.g., `render_help_text(writer: &mut impl Write)`) is needed for AC-12 without blocking on a keypress.
4. These are the same patterns already present in `pager.rs` (`render_to_buffer`, `render_border_plain`, `render_header_plain`, `render_row_plain`). The architect should follow the same pattern.

If items 2 or 3 are not exposed, the affected tests degrade to structural `grep` checks or are marked `skipped for reason: writer-injected variant not provided`. This would mean AC-9 or AC-12 cannot be marked `run and passed` in the Phase 3 report.

---

## Risk Assessment

| Risk | AC(s) | Probability | Impact | Mitigation |
|------|-------|------------|--------|------------|
| PTY test for search prompt is flaky (timing on character echo) | AC-1 | Medium | Medium | Use `Stage::Prompt` timeout (15 s default) for the prompt wait; `Stage::Query` timeout for post-submit render. If flaky, fall back to MANUAL for AC-1; document as `skipped for reason: PTY timing`. |
| Architect does not expose writer-injected status/help variants | AC-9, AC-12 | Low-Medium | Medium | Flag in Phase 3 prompt; degrade gracefully to `grep` structural check; mark as `skipped for reason: writer-injected variant not provided`. |
| match scan function is not extracted as a callable unit | AC-2..AC-6, AC-9, AC-10 | Low | HIGH | This is the single highest-risk architectural decision. If the architect inlines all search logic into `handle_key`, 7 of 12 ACs have no automated unit coverage. Must be flagged in Phase 3 architect prompt explicitly: "the search scan and navigation functions MUST be extractable `fn` or `pub(crate) fn` items." |
| ANSI highlight bytes are not `\x1b[7m` but a different sequence | AC-8 | Low | Low | Probe the actual crossterm `Attribute::Reverse` output at test time; adjust assertion if needed. Not a strategy risk. |
| Feature 2 is deferred (session budget) | AC-1..AC-4 F2 | Low-Medium | Low | Per planning doc: clean deferral. If deferred, mark all Feature 2 ACs as `skipped for reason: Feature 2 not implemented in session`. No verdict impact on Feature 1. |

---

## Strategy Summary

**Total Features Analyzed:** 2

**Test Types Required:**
- Unit tests: REQUIRED — Feature 1 (search logic, status strings, ANSI bytes, help text), Feature 2 (tick result error/success paths)
- Interactive tests (PTY): REQUIRED — Feature 1 AC-1, AC-11 (fallback: MANUAL)
- Manual: RECOMMENDED — Feature 1 AC-8 (visual color), Feature 2 AC-2 (behavioral identity)
- Benchmark: NOT NEEDED

**Estimated Test Counts:**
- New unit tests: 11 (9 for Feature 1 search logic + 1 for Feature 1 help text + 2 for Feature 2 = 12 total; some may merge by fixture reuse)
- New interactive tests: 1 mandatory (TC099-I01), 1 optional (TC099-I02)
- Manual checks: 2 (AC-8 visual highlight, Feature 2 AC-2 byte-identity)
- Total new automated tests: ~12 unit + 1-2 interactive

**Dependencies Required:**
- Live database: YES — for TC099-I01 and TC097 regression guard (Feature 2 AC-2)
- Network access: YES — same Teradata endpoint as all prior sprints
- Specific OS: NO
- New crates: NONE
- PTY harness: the Sprint 66 `TqPty` harness in `tests/common/pty_harness.rs` is SUFFICIENT

**Risk Assessment:**
- HIGH risk gaps: inline search logic (mitigated by explicit architect prompt requirement)
- MEDIUM risk gaps: writer-injected variants not exposed (mitigation: degrade to grep), PTY timing (mitigation: fall back to manual with documented reason)
- LOW risk gaps: AC-8 ANSI byte sequence, AC-2 F2 behavioral identity

---

## TC099 / TC100 Authoring Commitment

TC099 (`tests/cases/TC099-pager-search.md`) and TC100 (`tests/cases/TC100-handle-tick-result.md`, if Feature 2 ships) will NOT be authored in Phase 2. They will be written in Phase 3, AFTER the architect's Feature 1 code has landed in a working state. This mirrors the Phase 4 Step 1.7 sequential user-guide rule applied to test-case prose (Sprint 66 P2 follow-up, documented in `docs/sprints/sprint-67-planning.md` → Action Items).

---

## Strategy Validation Checklist

- [x] Every feature has a complete specification analysis section
- [x] Feature characteristics are classified (not assumed)
- [x] Test strategy is derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest
- [x] Specification coverage map includes all 16 ACs
- [x] Every AC maps to at least one test type
- [x] Test implementation plan is detailed and actionable
- [x] Coverage sufficiency is assessed
- [x] Honest evidence convention documented verbatim for Phase 3 report
- [x] Test data fixture sketched in full
- [x] Tool requests explicitly stated (none; architect cooperation requirements stated)
- [x] TC099/TC100 explicitly deferred to Phase 3

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-04-19
**Review Status:** DRAFT
**Sprint:** 67
