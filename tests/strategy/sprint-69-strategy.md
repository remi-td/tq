# Sprint 69 Test Strategy: PTY Cursor-Position Fix + Pager Search Status Bar Composition

**Created:** 2026-05-29
**Author:** quality-validator
**Sprint:** Sprint 69
**Features:**
1. Objective 1: PTY harness `[6n` cursor-position fix — TC097-A..H and TC104 must now actually execute
2. Objective 2: Pager search status bar position context — composed `Pattern: <pat>  (M matches)  |  Rows X-Y of Z` format, width-aware

---

## Context: Why This Sprint Exists

Sprint 68 confirmed the exact root cause of TC097's four-sprint failure streak. PTY dump inspection
showed reedline emitting `ESC[6n` (Cursor Position Request) during startup; the harness never
replies, so reedline loops printing "Error reading input: The cursor position could not be read
within a normal duration" until the stage budget expires. Neither `tq>` prompt nor pager activation
ever fires.

- **TC097-A..H** (`test_sessions_watch_*`, 8 tests): have been `#[ignore]`'d and failing since
  Sprint 65. Migration to the tiered harness (Sprint 68 Obj 1) was complete, but all 8 still fail
  with `QueryTimeout` because the REPL never reaches the prompt.
- **TC104** (`test_pager_search_prompt_shows_match_count`): passes via an early-return guard that
  fires on `cursor position` text in the PTY buffer, bypassing all search assertions. The test is
  structurally a lie — it reports PASS without exercising a single search AC.

Sprint 69 Objective 1 fixes the root cause (harness responds `ESC[1;1R` to `ESC[6n`). Once fixed,
TC097-A..H must run without the early-return guard, and TC104 must run without the cursor-guard
early-return, exercising its real assertions.

Sprint 69 Objective 2 delivers the composed status bar format. REQ-PAGER-SEARCH-009 was updated in
Sprint 69 to require `Pattern: <pat>  (M matches)  |  Rows X-Y of Z` (wide terminal) and graceful
truncation (narrow terminal). The existing status bar unit tests must be updated or joined by new
tests covering the composition rule.

---

## Feature-by-Feature Test Strategy

---

### Objective 1: PTY Cursor-Position Fix

#### 1. Specification Analysis

**Acceptance Criteria (from sprint-69-planning.md Objective 1):**
1. AC-PTY-1: The PTY harness correctly handles reedline's `[6n` cursor-position query so the REPL
   prompt appears without the "cursor position could not be read" error loop.
2. AC-PTY-2: TC097-A..H pass on live DB when run with `--ignored`.
3. AC-PTY-3: TC104 (`test_pager_search_prompt_shows_match_count`) executes its search assertions
   without the early-return cursor guard firing, and passes on live DB.
4. AC-PTY-4: No regression in `cargo test --lib` or `cargo test --all-targets`.

**Feature Characteristics:**

**User Interaction Type:** Interactive PTY — the fix is in the test harness
(`tests/common/pty_harness.rs`), not in production code. It makes the harness respond to ANSI
cursor-position requests emitted by the REPL process inside the PTY.

**Observable Behavior:**
- `tq>` prompt appears in PTY output after REPL startup (no cursor-position error loop).
- PTY dump no longer contains "cursor position could not be read" when the harness is running.
- TC097-A..H watchmode commands (`/sessions --watch`) can be sent and responded to.
- TC104's search prompt (`/`, pattern, `n`, `q`) can be exercised without early-return bypass.

**External Dependencies:**
- Live Teradata database via `TQ_LOGON` — required for all TC097-A..H tests and TC104.
- PTY harness (to be modified) in `tests/common/pty_harness.rs`.
- `expectrl` crate underlying the PTY session.

**Validation Challenges:**
1. **The fix may not be sufficient.** If reedline emits `[6n` at multiple points during startup
   (e.g., once on init and once after terminal detection), a single synthetic response may not be
   enough. The PTY dump from any remaining failures will show if further responses are needed.
2. **TC104 cursor guard removal.** The guard at `tests/interactive_tests.rs:3677-3684` returns
   early on "cursor position" text. This guard must be removed (or conditioned on actual fix
   failure) before the test exercises real search assertions. The architect must remove the guard
   as part of the fix.
3. **Watch-mode frame timing.** TC097's watch tests send `/sessions --watch --interval 1` and wait
   for the frame header. Even after the cursor fix, the watch loop must render at least one frame
   within the `Stage::Query` budget (60 s). If the Teradata endpoint is cold, frames may arrive
   late. This is a latency risk, not a cursor-position risk.
4. **Harness thread-safety.** The `[6n` → `[1;1R` responder may run in a background thread or as
   a pre-read intercept hook. Implementation details affect testability — the unit test for the
   responder mechanism itself must exercise whatever API the architect exposes.

**Critical Behaviors to Validate:**
1. After the fix, PTY dump for watch tests no longer contains "cursor position could not be read".
2. TC097-A..H each complete (pass or fail for a non-cursor reason) — no `ConnectTimeout` or
   `PromptTimeout` caused by cursor-detection failure.
3. TC104 reaches `expect_stage(Stage::Query, "Pattern:")` and returns PASS on it — no early-return.

---

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" checked:
  → Validation is by running the tests themselves with --ignored.
  → PASS criterion for TC097-A..H: all 8 tests pass outright (no longer acceptable
    to report "PTY dump present" as the pass bar — the cursor fix must make them pass).
  → PASS criterion for TC104: test reaches Pattern: assertion; no early-return.

IF "Database connection" checked:
  → Tests BLOCKED if TQ_LOGON not set. BLOCKED verdict.

IF "Pure Logic" (harness CPR responder) checked:
  → Unit test for the responder mechanism REQUIRED (no DB, no network).
```

**Derived Test Types:**

**Test Type 1: Unit test for the `[6n` → `[1;1R` responder mechanism (TC107-U01)**
- **Validates:** AC-PTY-1 at the mechanism level. Confirms the new harness code synthesises
  `ESC[1;1R` bytes when `ESC[6n` is detected in the slave-side output stream.
- **Approach:** The responder will be some function or struct method in `pty_harness.rs` (or a
  helper module). Test it directly: feed it a byte slice containing `ESC[6n` (possibly embedded in
  other text) and assert it produces `ESC[1;1R` as the response to write back to the PTY master.
  Test the negative case: byte slice without `[6n` produces no response.
- **Rationale:** A unit test confirms the mechanism works in isolation before relying on a live DB
  run to discover regressions. The mechanism is pure byte-manipulation logic — no PTY or DB needed.
- **Gap if missing:** A bug in the CPR responder (e.g., off-by-one in the escape sequence
  detection, wrong response bytes) would only be detectable via live DB run, making diagnosis
  harder.
- **Necessity:** REQUIRED

**Test Type 2: Structural check — cursor guard removed from TC104 (TC107-structural)**
- **Validates:** AC-PTY-3 at the code level. Confirms the early-return guard at
  `tests/interactive_tests.rs:3677-3684` is absent from the final implementation.
- **Approach:** `grep -n "cursor position" tests/interactive_tests.rs` — confirm no `return;`
  early-exit on cursor-position text in `test_pager_search_prompt_shows_match_count`.
- **Rationale:** If the guard is still present, TC104 will silently pass on the cursor condition
  instead of exercising search assertions. Code review cannot detect this; a targeted grep can.
- **Gap if missing:** TC104 could report PASS via the guard while the fix has no effect, giving
  false confidence.
- **Necessity:** REQUIRED

**Test Type 3: TC097-A..H execution with live DB (TC107-A..H)**
- **Validates:** AC-PTY-2 — the 8 watch tests must PASS (not just produce a PTY dump).
- **Approach:** `cargo test --test interactive_tests watch -- --ignored 2>&1`.
  PASS criterion: all 8 tests report `test ... ok`. Anything else is FAIL.
- **Rationale:** This is the definitive proof that the cursor fix works end-to-end. The Sprint 68
  bar of "PTY dump present = acceptable" is retired for Sprint 69 — the fix's purpose is to make
  the tests pass, not merely produce better diagnostics.
- **Gap if missing:** The fix could be partially effective (e.g., suppresses cursor loop but leaves
  a different reedline startup issue). Only execution reveals this.
- **Necessity:** REQUIRED. BLOCKED if `TQ_LOGON` absent or DB unreachable.

**Minimum PASS bar for TC097-A..H:** All 8 must pass. If fewer than 8 pass, the verdict is
REJECTED (not conditionally approved). Each test maps to a distinct watch-mode AC; a partial pass
leaves interactive watch tests unvalidated. Exception: if a specific test fails due to a reason
clearly unrelated to cursor-position (e.g., a Teradata permission error on a specific system table)
and this is documented with PTY dump evidence, that test may be classified `skipped for reason: DB
permission` provided all others pass.

**Test Type 4: TC104 execution with live DB (TC107-TC104)**
- **Validates:** AC-PTY-3 — `test_pager_search_prompt_shows_match_count` must execute its search
  assertions and pass.
- **Approach:** `cargo test --test interactive_tests test_pager_search_prompt_shows_match_count -- --ignored --nocapture`.
  PASS criterion: test reaches `expect_stage(Stage::Query, Regex("Pattern: DBC"))` and passes.
  The PTY dump must NOT contain early-return-guard text ("PTY cursor detection failed — skipping").
- **Rationale:** TC104 has been "passing" for two sprints via the cursor guard. Sprint 69 must
  prove the search AC is exercised in reality.
- **Gap if missing:** The composed status bar (Objective 2) might be entirely broken, undetected
  because TC104 still bypasses search assertions.
- **Necessity:** REQUIRED. BLOCKED if `TQ_LOGON` absent or DB unreachable.

---

#### 3. Test Type Necessity Matrix (Objective 1)

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit test for CPR responder mechanism | REQUIRED | Mechanism is pure logic; fast isolation | Harness bug only caught by live DB run | MUST IMPLEMENT |
| Structural grep: cursor guard removed from TC104 | REQUIRED | Guard silently bypasses assertions | False PASS for TC104 even if fix does nothing | MUST CHECK |
| TC097-A..H interactive (--ignored) | REQUIRED | Definitive fix proof; all 8 must pass | Fix confirmed by dump only — behavioral pass unproven | MUST EXECUTE (BLOCKED if no DB) |
| TC104 interactive (--ignored) | REQUIRED | Search AC exercised without early-return | Composed status bar could be broken undetected | MUST EXECUTE (BLOCKED if no DB) |

---

### Objective 2: Pager Search Status Bar Position Context

#### 1. Specification Analysis

**Acceptance Criteria (from sprint-69-planning.md Objective 2):**
1. AC-STATUS-1: When search active, status bar shows `Pattern: <pat>  (M matches)  |  Rows X-Y of Z` on wide terminals.
2. AC-STATUS-2: On narrow terminals (<= width threshold), row context drops; only `Pattern: <pat>  (M matches)` shown, truncated to `terminal_width - 2` if needed.
3. AC-STATUS-3: `n`/`N` wrap notices and not-found notices still appear correctly alongside composed status.
4. AC-STATUS-4: REQ-PAGER-SEARCH-009.* updated (spec already updated in sprint-69 planning).
5. AC-STATUS-5: Unit test for composed status bar rendering.

**Specification References (repl.md as updated for Sprint 69):**

- **REQ-PAGER-SEARCH-009.2:** `Rows X-Y of Z` compact form (no `%`); column range omitted.
- **REQ-PAGER-SEARCH-009.3:** Full composed string when `full_composed_width <= terminal_width - 2`.
- **REQ-PAGER-SEARCH-009.4:** Composition algorithm — build full string, measure width, show full
  if fits, else show only search segment (no partial row context).
- **REQ-PAGER-SEARCH-009.5:** Match count `M` = total across all rows (unchanged).
- **REQ-PAGER-SEARCH-009.6:** Composed status persists across `n`/`N` and scroll.
- **REQ-PAGER-SEARCH-009.7:** No column range in search-active state.
- Separator between segments: `  |  ` (two spaces, pipe, two spaces).

**Feature Characteristics:**

**User Interaction Type:** Pure Logic — `render_status_bar_to_buffer` is a private method on
`Pager` that takes a `&mut impl Write` and renders the status bar. The composition rule is
deterministic given `(search_state, matches, row_offset, page_size, total_rows, term_width)`.

**Observable Behavior:**
- String content written to the writer (testable via `Vec<u8>`).
- Width threshold boundary: exact character count of the composed string vs `term_width - 2`.
- The separator `  |  ` is literal bytes in the output.
- Row numbers update after scroll (tested by updating `row_offset` and re-rendering).

**External Dependencies:** None — pure unit tests, `Vec<u8>` writer, in-memory `Pager` fixture.

**Validation Challenges:**
1. **Term width calibration.** The composition rule depends on `term_width`. The test fixture must
   set `term_width` to values that exercise both branches (full compose fits / full compose does not
   fit). The composed string length must be computed precisely for the test pattern.
2. **Existing unit tests.** `status_bar_matches_format_exact` and `status_bar_singular_match_uses_match_not_matches` already test the OLD format (`Pattern: val_  (N matches)` with no row context). These tests must be UPDATED to expect the new composed format, or additional tests added alongside them. Running old tests against new code without updating them will produce failures — the architect and quality-validator must coordinate.
3. **Row context precision.** The `Rows X-Y of Z` segment depends on `row_offset`, `page_size`,
   and `total_rows`. The test fixture must set these fields explicitly to produce deterministic
   expected strings.
4. **Not-found path.** `Pattern: <pat>  not found` has no row context (REQ-PAGER-SEARCH-003 —
   no-match case). The not-found format is unchanged. Tests must confirm `not found` path is NOT
   extended with row context.

**Critical Behaviors to Validate:**
1. Full composed `Pattern: <pat>  (M matches)  |  Rows X-Y of Z` when `full_composed_width <= term_width - 2`. (REQ-PAGER-SEARCH-009.3)
2. Separator is exactly `  |  ` (two spaces, pipe, two spaces). (REQ-PAGER-SEARCH-009.3)
3. Row context is `Rows X-Y of Z` — no `%`, no column range. (REQ-PAGER-SEARCH-009.2)
4. Row context dropped (not partially shown) when full composed string exceeds `term_width - 2`. (REQ-PAGER-SEARCH-009.4)
5. Search segment truncated to `term_width - 2` when search segment alone exceeds terminal width. (REQ-PAGER-SEARCH-009.4)
6. Composed status persists across row scroll (row numbers update; pattern and match count do not). (REQ-PAGER-SEARCH-009.6)
7. Not-found path (`Pattern: <pat>  not found`) is NOT extended with row context. (REQ-PAGER-SEARCH-003)
8. No column range in search-active composed status. (REQ-PAGER-SEARCH-009.7)

---

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Pure Logic" checked:
  → Unit tests REQUIRED. Deterministic function, Vec<u8> writer, no external deps.

IF "Visual output in terminal" NOT the primary concern:
  → No PTY test needed for the composition rule itself.
  → PTY validation for the composed output IS covered by TC104 (Objective 1),
    which will observe `Pattern: ... | Rows` in the PTY if both objectives pass.
```

**Derived Test Types:**

**Test Type 1: Unit tests for `render_status_bar_to_buffer` — composed format (TC108-U)**
- **Validates:** All 8 critical behaviors listed above.
- **Approach:** `Vec<u8>` writer, `Pager` fixture with controlled `term_width`, `row_offset`,
  `page_size`, `total_rows`, `search` state. Call `render_status_bar_to_buffer`. Assert on
  `String::from_utf8(buf)`.
- **Rationale:** The function is pure, writer-injected, and already has a test helper
  (`status_bar_to_string`) in `pager.rs`. New tests follow the exact same pattern.
- **Gap if missing:** Composition logic could be wrong (wrong separator, partial row context, row
  context shows `%`, column range leaks into search-active output) without detection until TC104
  PTY test reveals it at integration level.
- **Necessity:** REQUIRED

**Test Type 2: Update existing status bar unit tests for new format**
- **Validates:** That tests from Sprint 67 (`status_bar_matches_format_exact`,
  `status_bar_singular_match_uses_match_not_matches`) pass against the new implementation.
- **Approach:** The architect will update `render_status_bar_to_buffer`. The old tests assert
  `out.contains("Pattern: val_  (")` — if the new format appends `  |  Rows ...`, these assertions
  still hold IF the new output also contains the old substring. Check this: the new format is a
  strict extension of the old format. If the old assertions remain valid, no change needed. If
  they break (e.g., the old test used a `term_width` too narrow for the new composed string), the
  tests must be updated.
- **Necessity:** REQUIRED (not a new test — maintenance validation that existing tests still pass)
- **Action:** Run `cargo test --lib status_bar` after the implementation; confirm all existing
  status bar tests still pass. If any fail, update the fixture `term_width` or assertions to match
  the new spec.

**Test Type 3: PTY integration observation via TC104 (cross-objective)**
- **Validates:** That the composed status bar appears in the real pager on a live DB session.
- **Approach:** TC104 (now freed from the cursor guard, Objective 1) searches for `"Pattern: DBC"`.
  If Objective 2 is implemented, the actual PTY output will contain `Pattern: DBC  (N matches)  |  Rows ...`.
  The `strip_ansi` + `assert!(clean.contains("match"))` check in TC104 will still pass.
- **Rationale:** TC104 provides end-to-end validation of the composed format in a real PTY session.
  It does not test the width threshold (that requires controlling `term_width` in PTY, which is
  complex). The unit tests cover width logic; TC104 covers the happy path on a real terminal.
- **Necessity:** RECOMMENDED (TC104 is REQUIRED for Objective 1; its status bar observation is a
  bonus for Objective 2 — not separately tracked)

---

#### 3. Detailed Unit Test Cases for Objective 2 (TC108-U01..U07)

**TC108-U01: Full composed format on wide terminal**
- Fixture: `make_pager_with_data(50, 20)`, `term_width = 120`, `submit_search("val_")` (≥2 matches)
- Expected: output contains `Pattern: val_  (` AND `  |  Rows 1-20 of 50`
- Validates: REQ-PAGER-SEARCH-009.3 (full composed string when width fits)

**TC108-U02: Separator is exactly two spaces pipe two spaces**
- Fixture: same as U01
- Expected: output contains the literal bytes `  |  ` between pattern segment and row segment
- Validates: REQ-PAGER-SEARCH-009.3 separator specification

**TC108-U03: Row context uses compact `Rows X-Y of Z` — no percentage**
- Fixture: `make_pager_with_data(100, 20)` with `row_offset = 0`, `term_width = 120`, search active
- Expected: output contains `Rows 1-20 of 100`; output does NOT contain `%` in the search-active status
- Validates: REQ-PAGER-SEARCH-009.2 (compact form, no `%`)

**TC108-U04: Row context dropped when full composed string exceeds `term_width - 2`**
- Fixture: construct a pager with a pattern long enough that the full composed string would exceed
  `term_width - 2`. Example: `term_width = 30`, pattern = `"val_"`, total rows = 50, page_size = 10.
  Full composed = `Pattern: val_  (N matches)  |  Rows 1-10 of 50` = ~47 chars > 28 = `30 - 2`.
  Short composed = `Pattern: val_  (N matches)` = ~26 chars.
- Expected: output contains `Pattern: val_  (` AND does NOT contain `  |  Rows`
- Validates: REQ-PAGER-SEARCH-009.4 (row context dropped entirely, not partially shown)

**TC108-U05: Search segment truncated when even search segment exceeds terminal width**
- Fixture: `term_width = 20`, very long pattern (e.g., `"a_very_long_pattern_string"`). Search
  segment alone = `Pattern: a_very_long_pattern_string  (N matches)` = >20 chars.
- Expected: output length (excluding `\r\n`) <= 18 (= `term_width - 2`)
- Validates: REQ-PAGER-SEARCH-009.4 (search segment truncated to `terminal_width - 2`)

**TC108-U06: Row context updates after scroll (composed status persists)**
- Fixture: wide terminal, submit_search, confirm composed format. Then set `pager.row_offset = 10`
  and re-render.
- Expected: second render contains `Rows 11-` (updated row numbers); pattern and match count unchanged.
- Validates: REQ-PAGER-SEARCH-009.6 (composed status persists, row numbers update live)

**TC108-U07: Not-found path does NOT include row context**
- Fixture: `make_pager_with_data(50, 20)`, wide terminal, `submit_search("xyzzy")` (no match)
- Expected: output contains `Pattern: xyzzy  not found`; output does NOT contain `  |  Rows`
- Validates: REQ-PAGER-SEARCH-003 / REQ-PAGER-SEARCH-009 boundary (not-found is unchanged)

**Note on column range:** The existing `status_bar_default_no_search` test validates that `Columns
1-` appears in default (no-search) state. A regression test confirming `Columns` does NOT appear
in search-active state is desirable. This can be a check within TC108-U01: assert
`!out.contains("Columns")` in search-active state.

---

#### 4. Test Type Necessity Matrix (Objective 2)

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests TC108-U01..U07 | REQUIRED | Deterministic composition logic; no external deps | Width logic, separator, format bugs undetected | MUST IMPLEMENT |
| Update existing status bar unit tests | REQUIRED | Ensure new format doesn't break old assertions | Existing tests fail silently post-refactor | MUST VERIFY (run `cargo test --lib status_bar`) |
| PTY integration via TC104 | RECOMMENDED | Real terminal happy path | Narrow terminal / edge case gaps (covered by units) | BONUS — provided by Objective 1 |

---

## Specification Coverage Map

| Requirement | Description | Test Type | Test Case | BLOCKED if |
|-------------|-------------|-----------|-----------|------------|
| AC-PTY-1 | CPR responder in harness | Unit (mechanism) | TC107-U01 | Never |
| AC-PTY-1 | No cursor-loop in live PTY | Interactive PTY | TC107-A..H | TQ_LOGON absent |
| AC-PTY-2 | TC097-A..H pass | Interactive PTY | TC107-A..H | TQ_LOGON absent |
| AC-PTY-3 | TC104 no early-return guard | Structural grep | TC107-structural | Never |
| AC-PTY-3 | TC104 search assertions execute | Interactive PTY | TC107-TC104 | TQ_LOGON absent |
| AC-PTY-4 | No unit/lib regression | `cargo test --lib` | Run standard lib tests | Never |
| AC-STATUS-1 | Full composed format (wide) | Unit | TC108-U01, TC108-U02 | Never |
| AC-STATUS-1 | Separator `  \|  ` exact | Unit | TC108-U02 | Never |
| REQ-PAGER-SEARCH-009.2 | Row context format `Rows X-Y of Z` no `%` | Unit | TC108-U03 | Never |
| REQ-PAGER-SEARCH-009.4 | Row context dropped when narrow | Unit | TC108-U04 | Never |
| REQ-PAGER-SEARCH-009.4 | Search segment truncated when very narrow | Unit | TC108-U05 | Never |
| REQ-PAGER-SEARCH-009.6 | Composed status persists, row numbers update | Unit | TC108-U06 | Never |
| REQ-PAGER-SEARCH-009.7 | No column range in search-active state | Unit | TC108-U01 (negative check) | Never |
| REQ-PAGER-SEARCH-003 | Not-found: no row context appended | Unit | TC108-U07 | Never |
| AC-STATUS-3 | Wrap notices still appear | Existing test preservation | `status_bar_wrap_notice_*` existing tests | Never |

---

## Harness Strategy

### CPR Responder Implementation Options

The architect has two implementation options for the `[6n` → `[1;1R` response:

**Option A: Inline response in `TqPty::expect_stage`**
- After setting the stage timeout, before calling `session.expect(needle)`, read available bytes
  from the PTY master. If `ESC[6n` is found, write `ESC[1;1R` back to the PTY slave input.
  Repeat in a loop as long as `[6n` is found.
- Pro: no threading complexity.
- Con: timing-sensitive — `[6n` must arrive before the `expect` call blocks.

**Option B: Background responder thread in `TqPty::new`**
- Spawn a background thread that reads the PTY master, detects `ESC[6n`, and writes `ESC[1;1R`.
  The thread runs for the lifetime of `TqPty`.
- Pro: responds immediately whenever `[6n` arrives, regardless of `expect_stage` call timing.
- Con: more complex; requires the PTY handle to be shareable (or the session to expose a write fd).

**Option C: Spawn-time pre-response**
- Before spawning `tq`, configure the PTY environment to set `TERM=dumb` or suppress reedline's
  cursor query (e.g., env var `REEDLINE_NO_CURSOR_POS=1` if such a flag exists in reedline).
- Pro: no byte-interception needed.
- Con: depends on reedline internals; may affect terminal rendering behavior for pager tests.

**Recommendation for test strategy:** The unit test TC107-U01 must test whatever API the architect
exposes. If Option A: test the "detect `[6n` and return `[1;1R`" logic as a pure function. If
Option B: test the responder function in isolation. If Option C: the unit test verifies the env var
is set in the spawn command.

### Live DB Requirements Summary

| Test | Live DB | PTY | Run Command |
|------|---------|-----|-------------|
| TC107-U01 (CPR mechanism) | No | No | `cargo test --lib` or `cargo test -p tq --test common` |
| TC107-structural (grep) | No | No | `grep` bash command |
| TC107-A..H (TC097 watch) | Yes | Yes | `cargo test --test interactive_tests watch -- --ignored` |
| TC107-TC104 (pager search) | Yes | Yes | `cargo test --test interactive_tests test_pager_search_prompt_shows_match_count -- --ignored --nocapture` |
| TC108-U01..U07 (status bar unit) | No | No | `cargo test --lib status_bar` |
| Existing status bar tests update | No | No | `cargo test --lib status_bar` |

---

## Gap Analysis

### Gap 1: TC097 tests may still fail for a non-cursor reason after the fix

- **What:** Even with `[6n` → `[1;1R` correctly implemented, the watch tests might fail if (a) the
  live DB is slow and the watch frame takes >60 s, (b) reedline emits additional cursor queries
  beyond the first one and the responder misses them, or (c) a different startup issue exists.
- **Sprint 69 PASS bar:** All 8 must pass. PTY dump with non-cursor failure evidence is REJECTED,
  not APPROVED. The Sprint 68 "PTY dump = acceptable" bar is retired.
- **Risk:** MEDIUM — PTY dump analysis from Sprint 68 shows the cursor loop is the ONLY failure
  visible; once answered, startup should succeed. But live DB latency is always a risk.
- **Mitigation:** Set `TQ_TEST_CONNECT_TIMEOUT=90` and `TQ_TEST_QUERY_TIMEOUT=120` in `.env` to
  give more budget. Document in evidence.

### Gap 2: TC104 may still fail after cursor guard removal for a search-logic reason

- **What:** The cursor guard was masking TC104's true behavior. After removal, if the pager search
  AC (the `Pattern:` status bar update) has a bug, TC104 will now correctly FAIL rather than
  silently PASS. This is a good failure — it means Objective 2 must also be correct before TC104
  passes.
- **Risk:** LOW for Sprint 69 since Objective 2 implements the composed status bar and TC104
  checks for `Pattern:` which is still present.
- **Mitigation:** TC104 checks `Regex("Pattern: DBC")` — this is a substring of both the old format
  (`Pattern: DBC  (N matches)`) and the new composed format (`Pattern: DBC  (N matches)  |  Rows...`).
  TC104 does not need to be updated for Objective 2.

### Gap 3: Width-threshold PTY test not planned

- **What:** The narrow terminal truncation rule (REQ-PAGER-SEARCH-009.4) is tested by unit tests
  (TC108-U04, TC108-U05) but not by any PTY test. A PTY test for narrow terminal behavior would
  require resizing the PTY window to a narrow width before sending the search pattern.
- **Why acceptable:** PTY window resizing in `expectrl` is possible but complex. The truncation
  logic is pure string formatting in `render_status_bar_to_buffer` — fully covered by unit tests.
  The risk of a regression that unit tests miss but a PTY test would catch is LOW.
- **Risk:** LOW
- **Mitigation:** The unit tests at TC108-U04/U05 set `pager.term_width` directly. No PTY resize needed.

### Gap 4: Existing status bar tests may need updating

- **What:** `status_bar_matches_format_exact` expects `out.contains("Pattern: val_  (")` which
  still holds after the change (new format is a superset). However, `status_bar_default_no_search`
  asserts `Columns 1-` appears — that test is in non-search state and is unaffected. The risk is
  that the existing tests have a `term_width = 120` fixture which should be wide enough to show the
  full composed string (including `  |  Rows`). If the test data in `make_pager_with_data(5, 3)`
  has too few rows (5 rows, page_size 3 means all fit in one page, `Rows 1-3 of 5` — 15 chars),
  the composed string fits easily in 120 cols. No change expected.
- **Risk:** LOW
- **Mitigation:** Run `cargo test --lib status_bar` as the first test step; any failures are
  immediately visible.

---

## Tool Gap Analysis

**No new test tools need to be built.** All infrastructure exists:

1. `tests/common/pty_harness.rs` — the CPR responder is implemented here (by the architect).
   TC107-U01 will test the new responder API in the existing `#[cfg(test)]` module of `pty_harness.rs`.
2. Built-in Rust `#[test]` — for TC108-U01..U07 in `pager.rs` `#[cfg(test)]`.
3. `status_bar_to_string()` helper — already in `pager.rs` tests; TC108-U tests use it directly.
4. `grep` — for TC107-structural.
5. `expectrl` + `TqPty` — for TC107-A..H and TC107-TC104.

**Architect cooperation required (not new tools):**

1. TC107-U01: The CPR responder must expose a testable function. The architecture must provide
   either (a) a free function `respond_to_cpr(bytes: &[u8]) -> Option<&'static [u8]>` that takes
   a byte slice and returns the CPR response bytes if `[6n` is detected, or (b) the background
   thread approach must have an internal function extracted for unit testing. The quality-validator
   cannot write TC107-U01 until the architect defines the responder API.

2. TC107-structural: The architect must remove the cursor guard from
   `test_pager_search_prompt_shows_match_count`. The grep check in TC107-structural validates this.

3. TC108-U: The architect's `render_status_bar_to_buffer` changes must be in place before TC108-U
   tests can be authored. The quality-validator will write TC108-U01..U07 after the architect
   implements the composition logic.

---

## Per-AC Test Assignment Summary

| AC | Test ID | Type | Necessity | PASS Criterion | BLOCKED if |
|----|---------|------|-----------|----------------|------------|
| AC-PTY-1 (CPR mechanism) | TC107-U01 | Unit | REQUIRED | `ESC[1;1R` bytes returned for `ESC[6n` input; no response for non-CPR input | Never |
| AC-PTY-1 (no cursor loop in PTY) | TC107-A..H | Interactive PTY | REQUIRED | PTY dumps do not contain "cursor position could not be read" | TQ_LOGON absent |
| AC-PTY-2 (TC097-A..H pass) | TC107-A..H | Interactive PTY | REQUIRED | All 8 tests report `test ... ok` | TQ_LOGON absent |
| AC-PTY-3 (guard removed) | TC107-structural | Structural grep | REQUIRED | No early-return on "cursor position" text in TC104 function | Never |
| AC-PTY-3 (TC104 real exec) | TC107-TC104 | Interactive PTY | REQUIRED | `Pattern: DBC` seen in PTY; `assert!(clean.contains("match"))` passes | TQ_LOGON absent |
| AC-PTY-4 (no regression) | lib-regression | `cargo test --lib` | REQUIRED | 1134+ unit tests pass; 0 failures | Never |
| AC-STATUS-1 (full composed wide) | TC108-U01 | Unit | REQUIRED | `Pattern: <pat>  (M matches)  |  Rows X-Y of Z` in output | Never |
| AC-STATUS-1 (separator exact) | TC108-U02 | Unit | REQUIRED | Literal `  \|  ` in output | Never |
| REQ-009.2 (row context format) | TC108-U03 | Unit | REQUIRED | `Rows X-Y of Z` in output; no `%` in search-active status | Never |
| REQ-009.4 (narrow: drop row ctx) | TC108-U04 | Unit | REQUIRED | Row context absent when full string > `term_width - 2` | Never |
| REQ-009.4 (very narrow: truncate) | TC108-U05 | Unit | REQUIRED | Output length <= `term_width - 2` | Never |
| REQ-009.6 (persists, rows update) | TC108-U06 | Unit | REQUIRED | Row numbers change after scroll; pattern unchanged | Never |
| REQ-009.7 (no column range) | TC108-U01 (negative) | Unit | REQUIRED | `Columns` absent from search-active status bar output | Never |
| REQ-003 boundary (not-found) | TC108-U07 | Unit | REQUIRED | `not found` path has no `  \|  Rows` | Never |
| Existing tests still pass | status_bar existing | Unit maintenance | REQUIRED | All pre-Sprint-69 status bar unit tests pass | Never |
| AC-STATUS-3 (wrap notices) | Existing wrap tests | Unit (preserve) | REQUIRED | `transient_status` tests unaffected | Never |

**Total REQUIRED tests: 14 test IDs**
**Total RECOMMENDED: 0** (PTY observation of composed format via TC104 is a bonus of Objective 1, not separately tracked)

**New test case files to create:**
- `tests/cases/TC107.md` — PTY cursor-position fix: CPR unit test + structural guard check + TC097-A..H execution + TC104 real execution
- `tests/cases/TC108.md` — Composed status bar unit tests (U01..U07) + existing test maintenance

---

## Strategy Summary

**Total Objectives Analyzed:** 2

**Test Types Required:**
- Unit tests: REQUIRED — Obj 1 (CPR responder mechanism), Obj 2 (status bar composition, U01..U07)
- Structural grep: REQUIRED — Obj 1 (cursor guard removal verification)
- Interactive PTY tests: REQUIRED — Obj 1 (TC097-A..H real pass + TC104 real exec)
- Doc review: NOT NEEDED for this sprint
- Benchmark: NOT NEEDED

**Estimated Test Counts:**
- New unit tests: 1 (TC107-U01 CPR mechanism) + 7 (TC108-U01..U07 status bar) = 8
- Updated/preserved unit tests: existing status bar tests must still pass = 6+ existing
- Structural checks: 1 (cursor guard grep)
- Interactive PTY tests: 8 (TC097-A..H = TC107-A..H) + 1 (TC104 = TC107-TC104) = 9 `#[ignore]` tests

**Risk Assessment:**
- HIGH risk gaps: none
- MEDIUM risk gaps: TC097-A..H may still timeout for non-cursor reasons; PTY harness CPR responder may need multiple response cycles
- LOW risk gaps: existing status bar tests may need minor fixture updates; TC104 search logic may reveal a pre-existing AC gap after guard removal

**Dependencies Required:**
- Live database: YES — for TC107-A..H and TC107-TC104
- Network access: YES — same Teradata endpoint
- Specific OS: NO
- New crates: NONE — expectrl already used
- PTY harness: MODIFIED by architect (CPR responder added)
- Architect cooperation: YES — responder API must be testable; cursor guard must be removed

---

## Strategy Validation Checklist

- [x] Every objective has a complete specification analysis section
- [x] Feature characteristics classified (not assumed)
- [x] Test strategy derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest
- [x] Per-AC assignment table covers all ACs from sprint-69-planning.md
- [x] Every AC maps to at least one test type
- [x] BLOCKED conditions explicitly stated for DB-dependent tests
- [x] Honest evidence convention documented (no APPROVED via code review alone)
- [x] Tool gap analysis explicit (architect cooperation for CPR API + guard removal)
- [x] Sprint 68 "PTY dump = acceptable" bar explicitly retired for TC097-A..H
- [x] TC104 cursor guard removal explicitly required before test can claim PASS

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-05-29
**Review Status:** DRAFT
**Sprint:** 69
