# Sprint 68 Test Strategy: Maintenance — Close TC097 + Sprint 67 Test Gaps

**Created:** 2026-05-29
**Author:** quality-validator
**Sprint:** Sprint 68 (Maintenance)
**Features:**
1. Objective 1: TC097-A..H migration to `spawn_tq_repl_tiered` + `Stage::Query`
2. Objective 2: Sprint 67 REQUIRED test gaps — AC-7 (`col_offset` unit), AC-8 (ANSI `write_value_with_highlights`), AC-1/AC-11 (PTY pager search)
3. Objective 3: Codify the REQUIRED-test rule in `docs/testing/approach.md` and `docs/testing/philosophy.md`
4. Objective 4: `rust-toolchain.toml` — pin CI toolchain to eliminate rustc-drift hazard

---

## Context: Why This Sprint Exists

Sprint 68 is purely maintenance. It closes four categories of carry-forward debt:

- **TC097 (3-sprint stuck issue):** 8 interactive watch tests have timed out in Sprints 65, 66, and 67. The Sprint 66 tiered harness (`spawn_tq_repl_tiered` + `Stage::*`) was built specifically to make these failures observable with PTY dump evidence instead of silent `ExpectTimeout`. The migration was planned but never executed.
- **Sprint 67 REQUIRED gaps:** Three tests were strategy-classified REQUIRED but not authored. Per `docs/testing/honest-assessment.md` REQUIRED-is-not-optional rule (added in Sprint 67), REQUIRED tests that go unwritten are MEDIUM-severity gaps, not accepted fallbacks.
- **REQUIRED-test rule documentation:** The rule exists in `honest-assessment.md` but is absent from the canonical testing methodology docs (`approach.md`, `philosophy.md`), meaning new agents can miss it.
- **`#![deny(warnings)]` CI hazard:** CI runs `dtolnay/rust-toolchain@stable` (latest), which can introduce new lints in any Rust release. Sprint 67 narrowly avoided a CI break. Pinning the toolchain eliminates the ambush pattern.

---

## Feature-by-Feature Test Strategy

---

### Objective 1: TC097-A..H Migration

#### 1. Specification Analysis

**Acceptance Criteria (from sprint-68-planning.md Objective 1):**
1. AC-OBJ1-1: TC097-A..H each use `spawn_tq_repl_tiered` + `Stage::Query` (replacing hardcoded `set_expect_timeout(30s)`).
2. AC-OBJ1-2: TC097-A..H tests pass OR produce a PTY dump explaining the failure (no silent `ExpectTimeout` without evidence).
3. AC-OBJ1-3: Old `set_expect_timeout` overrides at `tests/interactive_tests.rs:3554,3561` removed.

**Feature Characteristics:**

**User Interaction Type:** Interactive PTY — watch tests exercise the REPL `/sessions --watch` command via a live PTY session.

**Observable Behavior:**
- PTY output containing `Connected to`, frame headers (`Refreshing every...`), and the REPL prompt (`tq>`) after exit.
- PTY dump files written to `tests/results/sprint-66/<test_name>.pty.log` on timeout.

**External Dependencies:**
- Live Teradata database via `TQ_LOGON` — REQUIRED for all 8 tests.
- PTY harness (`tests/common/pty_harness.rs`) — already available since Sprint 66.

**Validation Challenges:**
1. **Connect latency:** Live database TLS + auth + first-query warm-up can exceed 45 s on a cold endpoint. This is why TC097 has timed out three sprints in a row. The tiered harness exposes which stage timed out, which is the critical diagnostic improvement.
2. **Watch-mode frame timing:** `/sessions --watch --interval 1` needs at least 2 seconds after sending the command before the first frame renders. `std::thread::sleep(Duration::from_secs(2))` in each test body is the guard; this is retained from the legacy implementation.
3. **`set_expect_timeout` override in TC097-H:** Lines 3554 and 3561 in `interactive_tests.rs` call `p.set_expect_timeout(Some(Duration::from_secs(30)))` on the raw `expectrl::Session`. After migration to `TqPty`, `TqPty::session_mut()` still exposes the underlying session, but the idiomatic form is to rely on the staged timeout and remove the direct override.

**Critical Behaviors to Validate:**
1. Each test that fails MUST produce a PTY dump at `tests/results/sprint-66/<test_name>.pty.log`.
2. Each test that passes MUST do so via `expect_stage(Stage::Connect, "Connected to")` + `expect_stage(Stage::Query, ...)` calls, not via raw `.expect(...)` with a blanket timeout.
3. TC097-H: The two hardcoded `set_expect_timeout` calls must be removed; the tiered harness provides the timeout budget.

---

#### 2. Test Strategy Derivation

```
IF "Interactive PTY" checked:
  → Validation is by running the tests themselves with --ignored.
  → PASS criterion: tests either (a) run and pass or (b) produce a PTY dump
    explaining the failure — not silent ExpectTimeout.
  → The migration itself is validated by code inspection PLUS execution evidence.

IF "Database connection" checked:
  → Tests BLOCKED if TQ_LOGON not set. Must report BLOCKED verdict, not APPROVED.
```

**Derived Test Types:**

**Test Type 1: Code-level migration validation (structural, pre-execution)**
- **Validates:** AC-OBJ1-1 (API change) and AC-OBJ1-3 (removed overrides)
- **Approach:** `grep` + code reading to confirm all 8 test functions use `TqPty` and `expect_stage(Stage::*, ...)`, and that `set_expect_timeout` calls at lines 3554/3561 are absent.
- **Rationale:** The migration is a mechanical API substitution. Code review can confirm it happened before running the live test suite.
- **Gap if missing:** Tests could be migrated incorrectly (e.g., using `Stage::Connect` where `Stage::Query` is needed).
- **Necessity:** REQUIRED (fast, zero-dependency verification step; runs before live DB tests)

**Test Type 2: Execution with live DB (Interactive PTY, #[ignore])**
- **Validates:** AC-OBJ1-2 — tests pass or produce a PTY dump with evidence.
- **Approach:** `cargo test --test interactive_tests watch -- --ignored`. If database is available, tests pass or produce dump files. If database is unavailable, verdict is BLOCKED.
- **Rationale:** The only proof that a PTY test works is running it. Code review cannot prove runtime behavior.
- **Gap if missing:** Tests might be syntactically migrated but still time out at the same stage as before (e.g., because the watch render takes longer than `Stage::Query`'s 60 s budget). Without running, we cannot know.
- **Necessity:** REQUIRED (with BLOCKED fallback if no DB — PTY dump is acceptable as evidence of diagnostic improvement even if the test fails)

**PASS criterion for AC-OBJ1-2:**
- Acceptable outcomes: (a) test runs and passes, or (b) test times out and a PTY dump file exists at the expected path with non-trivial content (> 0 bytes). A PTY dump proves the harness worked — the test has observable failure evidence. Silent `ExpectTimeout` without a dump is NOT acceptable.
- Unacceptable outcome: `ExpectTimeout` with no dump file, or `ExpectTimeout` in a stage that suggests the migration was incorrect (e.g., `PromptTimeout` when the prior code passed the connect stage).

---

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Code-level migration check (grep/read) | REQUIRED | Mechanical API substitution; verifiable without DB | Wrong stage used; timeout overrides not removed | MUST VERIFY |
| Interactive PTY execution (--ignored) | REQUIRED | Live proof of PTY dump behavior | Migration might be correct but tests still fail silently | MUST EXECUTE (BLOCKED if no DB) |

---

### Objective 2: Sprint 67 REQUIRED Test Gaps

Three sub-objectives, each a separate test. They share the context that they were strategy-classified REQUIRED in Sprint 67 but not authored.

---

#### 2a. AC-7 Unit Test: `scroll_to_match_snaps_to_rightmost_column`

**Acceptance Criterion (AC-OBJ2-1):** Unit test added to `pager.rs` tests exercising the `col_offset` branch in `scroll_to_match_index` for horizontal scroll when a match is in an off-screen column.

**Specification of behavior (from sprint-68-planning.md):**
- "If the matched cell is outside the horizontal viewport, the pager scrolls horizontally so it is in view." — Sprint 67 AC-7.
- The `scroll_to_match_index` function at `pager.rs:1158` sets `self.col_offset = m.col + 1 - visible.max(1)` when `m.col >= self.col_offset + visible`.

**What the test must prove:**
- Fixture: multiple columns (e.g., 8) where the viewport shows fewer (e.g., 3).
- Pattern: matches a cell in column 6 (0-indexed), which is outside the initial viewport (`col_offset = 0`, visible = 3 → visible range = [0..3)).
- After `submit_search(pattern)`, `pager.col_offset` must equal `6 + 1 - 3 = 4`.
- The column 6 (`col_offset=4`, visible=3 → visible range = [4..7)) is now in view.

**Test fixture design:**

```
Columns: 8 columns, all Varchar, each named "c0".."c7"
Rows: 5 rows
  - All cells are "x" except row 2, column 6, which is "TARGET"
term_width: set to a value that makes visible_column_count() = 3
  (term_width must be calibrated to column widths — a safe approach is to
   use narrow columns: all 3 chars wide + borders = ~4 chars each, so
   term_width ≈ 14 gives visible ≈ 3. OR use a pager.term_width value
   that the test can assert visible_column_count() == 3 before searching.)
page_size: ≥ 5 (all rows visible)
col_offset: 0 (initial)
```

**Assertions:**
1. Before `submit_search`: `pager.col_offset == 0`.
2. `pager.submit_search("TARGET")` succeeds (at least 1 match).
3. After `submit_search`: `pager.col_offset > 0` (scrolled right).
4. The match at `col=6` is now within `[col_offset, col_offset + visible_column_count())`.
5. Exact value: `pager.col_offset == 6 + 1 - 3 = 4` (if visible == 3).

**Test type:** Unit — no DB or PTY. Exercises `scroll_to_match_index` directly via `submit_search`.

**Test location:** `src/commands/repl/pager.rs` `#[cfg(test)]` module.

**Necessity:** REQUIRED

**Risk:** The `visible_column_count()` method is private (`fn`, not `pub`). The test is in the same module (`#[cfg(test)]`), so it can call it. The critical requirement is that the test must construct a pager where `visible_column_count()` returns 3 and the match is in column 6. This may require adjusting `term_width` and column widths carefully. If the column width calculation makes this hard to pin, an alternative is to assert `pager.col_offset >= 4` (match is visible) rather than `== 4` exactly.

---

#### 2b. AC-8 Unit Test: `write_value_with_highlights_emits_reverse_video`

**Acceptance Criterion (AC-OBJ2-2):** Unit test in `pager.rs` asserting that `write_value_with_highlights` emits `\x1b[7m` (crossterm `Attribute::Reverse`) before the matched substring and `\x1b[27m` (`Attribute::NoReverse`) after it.

**What the test must prove:**

The function signature is:
```rust
fn write_value_with_highlights(
    &self,
    stdout: &mut impl Write,
    value: &str,
    matches: &[(usize, usize)],
) -> io::Result<()>
```

This is a private `fn`, so the test must be in the `#[cfg(test)]` module of `pager.rs`.

**Test fixture:**
- `value = "fooBAR baz"` (10 bytes)
- `matches = &[(3, 6)]` — the substring `"BAR"` at bytes 3..6.
- `writer = Vec::<u8>::new()`.

**Assertions:**
1. Call `pager.write_value_with_highlights(&mut writer, "fooBAR baz", &[(3, 6)])`.
2. Convert `writer` to a string (with `String::from_utf8_lossy`).
3. Assert the output contains `b"\x1b[7m"` (the crossterm `SetAttribute(Attribute::Reverse)` escape) before `"BAR"`.
4. Assert the output contains `b"\x1b[27m"` (the crossterm `SetAttribute(Attribute::NoReverse)` escape) after `"BAR"`.
5. Assert `"foo"` appears before the `\x1b[7m` sequence (non-matching prefix is written unmodified).
6. Assert `" baz"` appears after the `\x1b[27m"` sequence (non-matching suffix is written unmodified).

**Exact byte sequences to assert:**

Crossterm `execute!(stdout, SetAttribute(Attribute::Reverse))` emits CSI code `\x1b[7m`.
Crossterm `execute!(stdout, SetAttribute(Attribute::NoReverse))` emits CSI code `\x1b[27m`.

The assertion approach: `assert!(output_bytes.windows(4).any(|w| w == b"\x1b[7m"))`.

**Note on constructing `pager` for the test:** `write_value_with_highlights` takes `&self`, so a `Pager` instance must exist. Since the function only reads `self` to access the `write_value_with_highlights` method (the function body itself does not access `self` fields — it takes `value` and `matches` as parameters), a minimal `Pager` constructed via `Pager::new(&minimal_result, &PagerConfig::default())` is sufficient. The test can reuse the existing `make_pager_with_data(1, 1)` helper from the test module.

**Test type:** Unit — no DB or PTY. Exercises ANSI byte emission via `Vec<u8>` writer.

**Test location:** `src/commands/repl/pager.rs` `#[cfg(test)]` module.

**Necessity:** REQUIRED

---

#### 2c. PTY Pager Search Test: `test_pager_search_prompt_and_exit`

**Acceptance Criterion (AC-OBJ2-3):** At least one PTY test using the tiered harness that:
- Opens the pager search prompt (sends `/`).
- Enters a pattern known to exist in the result set.
- Observes `Pattern:` in the PTY output (status bar updated).
- Exits the pager with `q` and confirms the REPL prompt returns.

This test was AC-1 and AC-11 from Sprint 67, strategy-classified REQUIRED, fallback-accepted, and now promoted to REQUIRED with no fallback in Sprint 68.

**What the test must prove:**

- AC-1: The `/` keypress opens the search prompt (visible in PTY output as `/` or `Pattern:` or similar status-bar text).
- AC-11: Exiting the pager after a search does not corrupt terminal state (REPL prompt `tq>` reappears cleanly).

**Execution plan (step-by-step):**

```
1. spawn_tq_repl_tiered("test_pager_search_prompt_and_exit")
2. p.expect_stage(Stage::Connect, "Connected to")   -- auth stage
3. sleep 1s (warmup)
4. p.session_mut().send_line("SELECT 'hello' AS val UNION ALL SELECT 'world' UNION ALL ...")
   -- Use a query that returns ≥ page_size rows to force pager mode.
   -- Alternatively, use a known large table from the Teradata system catalog
   -- (e.g., SELECT TOP 100 DatabaseName FROM DBC.DatabasesV)
   -- that is reliably populated on any Teradata instance.
5. p.expect_stage(Stage::Query, "val")              -- pager is showing
   -- The pager is in alternate screen; "val" is the column name in the header.
6. sleep 500ms (let pager settle)
7. p.session_mut().send("/")
8. sleep 200ms
9. p.session_mut().send("hello\n")   -- submit the search pattern
10. p.expect_stage(Stage::Query, "Pattern:")        -- status bar updated
11. p.session_mut().send("q")         -- exit pager
12. p.expect_stage(Stage::Prompt, "tq>")            -- REPL prompt returned
```

**PASS criterion:**
- Step 10 succeeds: PTY output contains `Pattern:` after sending `hello\n`. This proves the search was processed and the status bar updated.
- Step 12 succeeds: `tq>` appears after `q`. This proves AC-11 (clean exit, no terminal corruption).

**Alternative needle for step 10 if `Pattern:` appears in ANSI-wrapped form:**
- Use `expectrl::Regex("Pattern:|hello")` — either the literal status bar string or the match indicator.

**Key concern — pager mode detection:**
The pager runs in an alternate screen buffer. The PTY can observe bytes from the alternate screen only if the terminal does not suppress them. `expectrl` captures raw PTY bytes including alternate-screen content, so `expect_stage(Stage::Query, "val")` at step 5 should work if the column header bytes flow through the PTY. This is the same approach used by `test_repl_startup_and_quit` which successfully observes pager output in Sprint 67.

The test must use `--no-syntax-highlight` (already the default in `spawn_tq_repl_tiered`) to avoid ANSI-wrapping the column header.

**Live DB requirement:** The query must return more rows than `PagerConfig::default().effective_page_size()`. A safe universal query: `SELECT ColumnName FROM DBC.ColumnsV WHERE DatabaseName = 'DBC' AND TableName = 'SessionInfoV'` (DBC.SessionInfoV is a system view always present on Teradata). If this table has fewer columns than `page_size`, use `DBC.ColumnsV WHERE DatabaseName = 'DBC'` (hundreds of rows guaranteed).

**Test type:** Interactive PTY + live DB. Location: `tests/interactive_tests.rs`. `#[ignore]` attribute required.

**Necessity:** REQUIRED. This test was fallback-accepted in Sprint 67. Sprint 68 removes the fallback.

**BLOCKED condition:** If `TQ_LOGON` is not set or the database is unreachable, verdict for this test is BLOCKED. It is NOT acceptable to mark it as `skipped for reason: DB unavailable` and claim APPROVED.

---

#### 3. Necessity Matrix for Objective 2

| Test | Type | Necessity | PASS criterion | BLOCKED if |
|------|------|-----------|----------------|------------|
| `scroll_to_match_snaps_to_rightmost_column` (AC-7) | Unit | REQUIRED | `col_offset` snaps to place column 6 in viewport | Never blocked (no external deps) |
| `write_value_with_highlights_emits_reverse_video` (AC-8) | Unit | REQUIRED | `\x1b[7m` and `\x1b[27m` bytes present in `Vec<u8>` writer | Never blocked |
| `test_pager_search_prompt_and_exit` (AC-1/AC-11) | Interactive PTY | REQUIRED | `Pattern:` in PTY output + `tq>` after `q` | `TQ_LOGON` absent or DB unreachable |

---

### Objective 3: Testing Documentation Updates

#### Acceptance Criteria
- **AC-OBJ3-1:** `docs/testing/approach.md` contains the explicit statement: "A test classified REQUIRED in the test strategy that is not authored must be reported as a MEDIUM-severity gap in the quality report — it is not resolved by code inspection or manual verification."
- **AC-OBJ3-2:** `docs/testing/philosophy.md` updated to align.
- **AC-OBJ3-3:** `docs/testing/honest-assessment.md` entry added or updated if applicable (Sprint 68 note: the rule is already present in `honest-assessment.md` from Sprint 67, but the canonical testing methodology docs do not reference it).

#### Test Strategy

**User Interaction Type:** Documentation — pure text files. No executable code.

**Validation Approach:** This is a documentation-only objective. The test type is review-only.

**Test Type: Manual Review (Doc Correctness)**
- **Validates:** That the stated text appears in the specified files.
- **Approach:** `grep` the exact required phrase in `docs/testing/approach.md` and `docs/testing/philosophy.md`. Additionally, confirm the two files cross-reference each other or reference `honest-assessment.md`.
- **PASS criterion:** `grep "REQUIRED in the test strategy" docs/testing/approach.md` returns a non-empty match. `grep "MEDIUM-severity" docs/testing/approach.md` returns a non-empty match. Same for `philosophy.md` or confirmation that `philosophy.md` references `approach.md`.
- **Rationale:** Doc updates are verified by reading and grep, not by compiling or running tests. This is appropriate for documentation.
- **Gap if missing:** Future agents follow `approach.md` and miss the REQUIRED-is-binding rule, repeating the Sprint 65/66/67 pattern.
- **Necessity:** REQUIRED

**Note on honest-assessment.md:** The Sprint 67 rule is already present at `docs/testing/honest-assessment.md:169-184`. The Sprint 68 work is to propagate it into `approach.md` and `philosophy.md`. No new content needs to be invented.

---

### Objective 4: `rust-toolchain.toml`

#### Acceptance Criteria
- **AC-OBJ4-1:** `rust-toolchain.toml` added to project root with `channel = "stable"` (or a specific version string) that pins CI to a known-good toolchain.
- **AC-OBJ4-2:** CI passes with the pinned toolchain.
- **AC-OBJ4-3:** Decision on `#![deny(warnings)]` documented: keep (now safe with pin) or remove in favor of CI `-D warnings` flag alone.

#### Test Strategy

**User Interaction Type:** CI configuration change — no user-facing behavior.

**Feature Characteristics:**
- `rust-toolchain.toml` is a Cargo/Rustup convention file. Its presence causes `rustup` to read it and override the active toolchain for any `cargo` invocation in the project directory.
- This has no runtime behavior on the binary itself — it affects only the build toolchain.

**Validation Challenges:**
1. **CI-level verification:** The only way to prove the pin works is to observe CI pass with the new file present. Local verification can confirm the file is syntactically valid and that `cargo build` succeeds locally with the pinned toolchain.
2. **Channel vs. pinned version:** Using `channel = "stable"` means CI always uses the latest stable (same as before, but now observable via `rustup show` output in CI logs). Using `channel = "1.95.0"` means CI is pinned to a specific version indefinitely. The planning doc says "use `stable` channel not a hard version string" — this is the correct approach.
3. **`#![deny(warnings)]` interaction:** If `channel = "stable"` is chosen, new lint additions in future stable releases can still break CI. The mitigation is that the pin makes the toolchain upgrade explicit (a developer must change `rust-toolchain.toml`) rather than implicit (CI picks up the new stable silently). The Sprint 68 requirement is to document which approach was chosen, not to make a specific choice.

**Derived Test Types:**

**Test Type 1: File existence and syntax check**
- **Validates:** AC-OBJ4-1.
- **Approach:** `cat rust-toolchain.toml` — confirm file exists and contains `[toolchain]` section with `channel` key.
- **Necessity:** REQUIRED (trivial to execute, zero risk of false positive)

**Test Type 2: Local `cargo build` with pinned toolchain**
- **Validates:** AC-OBJ4-2 (local proxy for CI pass).
- **Approach:** `cargo build 2>&1 | head -5` — confirm build succeeds without toolchain errors. `scripts/ci-check.sh` (clippy + `cargo test --lib`) as the full local CI mirror.
- **Necessity:** REQUIRED
- **PASS criterion:** `scripts/ci-check.sh` exits 0 after `rust-toolchain.toml` is added.

**Test Type 3: CI pass (remote)**
- **Validates:** AC-OBJ4-2 (actual CI).
- **Approach:** Observe GitHub Actions CI run on the sprint branch after pushing.
- **Necessity:** REQUIRED — per `docs/sprints/sprint-68-planning.md`: "CI passes with pinned toolchain."
- **Note:** This test cannot be executed locally; it requires a push. The sprint commit will trigger CI. The sprint is not APPROVED until CI green is confirmed on the `v1.50.0` tag.

**Test Type 4: `#![deny(warnings)]` decision documentation review**
- **Validates:** AC-OBJ4-3.
- **Approach:** `grep "deny.warnings\|D warnings" src/lib.rs docs/` — confirm either (a) `#![deny(warnings)]` is still present in `src/lib.rs` with a comment citing the toolchain pin, or (b) it is removed and a CI `-D warnings` flag is documented in `docs/design/` or in `.github/workflows/`.
- **Necessity:** REQUIRED (documentation verification only)

---

#### Necessity Matrix for Objective 4

| Test | Type | Necessity | PASS criterion |
|------|------|-----------|----------------|
| `rust-toolchain.toml` file exists with valid `[toolchain]` section | Structural (file read) | REQUIRED | File present, `channel` key present |
| `scripts/ci-check.sh` passes locally | Unit/Clippy | REQUIRED | Exit code 0, 0 clippy warnings |
| CI green on sprint commit/tag | Remote CI | REQUIRED | GitHub Actions workflow passes |
| `#![deny(warnings)]` decision documented | Doc review | REQUIRED | Either kept with comment or removed with documented reason |

---

## Harness Strategy

### Which objectives need a live DB?

| Objective | Live DB Required? | Reason |
|-----------|-------------------|--------|
| Obj 1: TC097-A..H migration | YES | All 8 tests are interactive watch tests |
| Obj 2a: AC-7 col_offset unit | NO | Pure unit test, in-memory fixture |
| Obj 2b: AC-8 ANSI unit | NO | Pure unit test, Vec<u8> writer |
| Obj 2c: PTY pager search (AC-1/AC-11) | YES | REPL PTY test needs live DB |
| Obj 3: Doc updates | NO | Documentation review only |
| Obj 4: rust-toolchain.toml | NO (local) / YES (CI) | CI needs the push, not DB |

**Summary:** 9 of the test execution steps require a live DB. All others are executable in isolation.

### PTY Harness Sufficiency

The Sprint 66 tiered PTY harness (`tests/common/pty_harness.rs`) is **sufficient as-is** for both Objective 1 (TC097 migration) and Objective 2c (pager search PTY test). No new harness capabilities are needed.

Specifically:
- `TqPty::expect_stage(Stage::Connect, "Connected to")` — covers the auth wait.
- `TqPty::expect_stage(Stage::Query, ...)` — covers waiting for pager output or status bar.
- `TqPty::expect_stage(Stage::Prompt, "tq>")` — covers waiting for REPL prompt after pager exit.
- `TqPty::session_mut().send(...)` and `.send_line(...)` — covers keypress simulation (including `/`, pattern text, `\n`, `q`).

**PTY dump logs:** The harness writes `tests/results/sprint-66/<test_name>.pty.log` on timeout. No change to the dump path is needed for Sprint 68. (A future sprint could update the dump path to `sprint-68`, but it is not required for this sprint's PASS criterion.)

---

## Honest Evidence Convention

Per `docs/testing/honest-assessment.md` and Sprint 67 precedent, the Phase 4 execution report (`tests/results/sprint-68/REPORT.md`) MUST use exactly one of these four labels for each test:

| Label | Meaning |
|-------|---------|
| **run and passed** | Test was executed; cargo test / CI reports success |
| **run and failed** | Test was executed; cargo test reports failure or panic |
| **not run** | Test exists in source but was not executed |
| **skipped for reason X** | Test was deliberately not executed; reason must be stated explicitly |

**CRITICAL: No test may be labeled APPROVED by code review alone.** If a PTY test cannot run (no DB), it is `skipped for reason: DB unavailable (BLOCKED)`. The verdict for the sprint is then BLOCKED, not APPROVED.

**For the TC097 migration specifically:** If a test times out after migration to the tiered harness, it is `run and failed — ExpectTimeout at Stage::Connect/Query (PTY dump written to tests/results/sprint-66/<name>.pty.log)`. This is better evidence than `skipped for reason: TC097 pre-existing timeout` — the dump proves the harness ran and the failure is observable.

---

## Per-AC Test Assignment Summary

| AC | Test ID | Type | Necessity | PASS Criterion | BLOCKED if |
|----|---------|------|-----------|----------------|------------|
| OBJ1-1 (API migration) | TC101-code-check | Structural/grep | REQUIRED | All 8 fns use `TqPty` + `expect_stage` | Never |
| OBJ1-2 (pass or dump) | TC101-A..H | Interactive PTY | REQUIRED | Pass or PTY dump exists | TQ_LOGON absent |
| OBJ1-3 (remove overrides) | TC101-code-check | Structural/grep | REQUIRED | Lines 3554/3561 patterns absent | Never |
| OBJ2-1 (col_offset unit) | TC102-U01 | Unit | REQUIRED | `col_offset` snaps to expose off-screen match | Never |
| OBJ2-2 (ANSI bytes unit) | TC103-U01 | Unit | REQUIRED | `\x1b[7m` + `\x1b[27m` in Vec<u8> output | Never |
| OBJ2-3 (PTY pager search) | TC104-I01 | Interactive PTY | REQUIRED | `Pattern:` in PTY + `tq>` after `q` | TQ_LOGON absent |
| OBJ3-1 (approach.md text) | TC106-approach | Doc review (grep) | REQUIRED | Phrase present in approach.md | Never |
| OBJ3-2 (philosophy.md text) | TC106-philosophy | Doc review (grep) | REQUIRED | Alignment present in philosophy.md | Never |
| OBJ3-3 (honest-assessment.md) | TC106-honest | Doc review | RECOMMENDED | New entry if applicable | Never |
| OBJ4-1 (file exists) | TC105-file | Structural | REQUIRED | File present with valid TOML | Never |
| OBJ4-2 (ci-check.sh) | TC105-local | Unit/Clippy | REQUIRED | ci-check.sh exits 0 | Never |
| OBJ4-2 (CI pass) | TC105-ci | Remote CI | REQUIRED | GitHub Actions green | Never |
| OBJ4-3 (deny-warnings doc) | TC105-doc | Doc review | REQUIRED | Decision documented | Never |

**Total REQUIRED tests: 12 (10 automated executable, 2 remote CI/doc review)**
**Total RECOMMENDED: 1 (honest-assessment.md optional entry)**

**Test case files created:**
- `tests/cases/TC101.md` — TC097 migration validation (Obj 1: structural + interactive A..H)
- `tests/cases/TC102.md` — `scroll_to_match_snaps_to_rightmost_column` unit test (Obj 2a, AC-7)
- `tests/cases/TC103.md` — `write_value_with_highlights_emits_reverse_video` unit test (Obj 2b, AC-8)
- `tests/cases/TC104.md` — `test_pager_search_prompt_and_exit` PTY test (Obj 2c, AC-1/AC-11)
- `tests/cases/TC105.md` — `rust-toolchain.toml` file + CI validation (Obj 4)
- `tests/cases/TC106.md` — REQUIRED-test rule in testing docs (Obj 3)

**Note on numbering:** TC100 was already used by Sprint 67 (`handle_tick_result` test).
Sprint 68 test cases start at TC101.

---

## Gap Analysis

### Gap 1: TC097 tests may still time out after migration

- **What:** Even after migrating to `spawn_tq_repl_tiered`, the watch tests might fail at `Stage::Connect` or `Stage::Query` due to live DB cold-start latency or watch-mode frame timing.
- **Why acceptable for Sprint 68:** The planning doc explicitly states "acceptable if dump explains root cause." The Sprint 68 PASS criterion for AC-OBJ1-2 includes the case where tests produce a PTY dump. The improvement over the current state (silent `ExpectTimeout` with no dump) is the key deliverable.
- **Risk:** MEDIUM if all 8 tests time out with dumps (migration verified but behavioral improvement unproven). LOW if at least some tests pass.
- **Mitigation:** Adjust `TQ_TEST_CONNECT_TIMEOUT` / `TQ_TEST_QUERY_TIMEOUT` upward (e.g., 90 s connect, 120 s query) to give watch-mode tests more budget. Document in test-evidence.

### Gap 2: `test_pager_search_prompt_and_exit` query fixture dependency

- **What:** The PTY pager search test requires a query that returns enough rows to trigger pager mode. The safe choice is `DBC.ColumnsV WHERE DatabaseName = 'DBC'` (reliable on any Teradata). However, if the test database restricts access to DBC views, the test fails at the query step.
- **Risk:** LOW — DBC.ColumnsV is readable by all sessions on standard Teradata installs (including demo environments). If not accessible, the test produces a PTY dump at `Stage::Query` rather than silently timing out.
- **Mitigation:** Document the query choice in TC104. If DBC.ColumnsV is unavailable, fall back to a multi-row UNION query (e.g., `SELECT 'hello' AS v UNION ALL SELECT 'world' UNION ALL ...` × 30 repetitions to exceed page_size). Note in evidence.

### Gap 3: `rust-toolchain.toml` does not prevent new lint introduction

- **What:** `channel = "stable"` means the toolchain still upgrades with each stable Rust release. The CI hazard is reduced (the upgrade becomes explicit) but not eliminated.
- **Why acceptable:** The planning doc explicitly chose `stable` over a pinned version string. A hard version pin is a valid alternative but introduces maintenance burden (the pin must be bumped periodically). The `stable` channel with `rust-toolchain.toml` is a pragmatic middle ground.
- **Risk:** LOW — the current issue is that CI upgrades without any developer awareness. With `rust-toolchain.toml`, the upgrade is at least observable in CI logs via `rustup show`.

### Gap 4: Doc updates (Objective 3) are not execution-tested

- **What:** Documentation changes cannot be run through `cargo test`. The only verification is reading the files and confirming the text is present.
- **Why acceptable:** This is inherent to documentation. The REQUIRED-test rule is a procedural constraint, not a code path. Its presence in the right files is verified by grep.
- **Risk:** LOW — the exact phrase to add is specified in the acceptance criteria. A grep can confirm it was added.

---

## Tool Requests for Coordinator

**No new test tools or harness capabilities are needed.** All infrastructure exists:
- Built-in Rust `#[test]` for Objective 2a and 2b unit tests.
- `tests/common/pty_harness.rs` tiered harness (Sprint 66) — sufficient as-is for Objectives 1 and 2c.
- `scripts/ci-check.sh` — already the local CI mirror.
- `grep` for documentation verification (Objectives 3, 4).

**Architect cooperation required (not new tools):**

1. **Objective 2a (AC-7 unit test):** `write_value_with_highlights` is a private `fn` on `Pager`. The test will be in the `#[cfg(test)]` module of `pager.rs`, so it can access private members. However, `scroll_to_match_index` is also private, and `submit_search` calls it. The test calls `submit_search` (which is also private but accessible from `#[cfg(test)]`). No `pub(crate)` changes needed.
   - But `visible_column_count()` must return a predictable value for the test assertion. The method is private and computes from `term_width` and column widths. The architect must either (a) document how to set `term_width` to get `visible_column_count() == N` for a known column set, or (b) accept an assertion of `col_offset > 0` (match scrolled right) rather than an exact `col_offset == 4`. Both are acceptable for Sprint 68.

2. **Objective 2b (AC-8 unit test):** `write_value_with_highlights` requires a `&Pager` receiver. The test needs to construct a minimal `Pager`. The existing `make_pager_with_data(rows, page_size)` helper in the test module is sufficient if it exists; if not, `Pager::new(&minimal_result, &PagerConfig::default())` works.

3. **Objective 2c (PTY pager search test):** The test needs the REPL to enter pager mode. The `spawn_tq_repl_tiered` helper uses `--no-syntax-highlight --no-pager` flags — NOTE: `--no-pager` must be REMOVED for this test, otherwise the pager will not activate. The test must use a custom command string that omits `--no-pager`. See implementation plan below.

---

## Implementation Notes

### TC097 Migration Pattern

For each of TC097-A..H, the migration pattern is:

**Before:**
```rust
fn test_sessions_watch_<name>() {
    let mut p = spawn_tq_repl();
    p.expect("Connected to").expect("Failed to connect");
    std::thread::sleep(Duration::from_secs(1));
    p.send_line("/sessions --watch --interval 1")...
    std::thread::sleep(Duration::from_secs(2));
    // ... test body
    p.expect(expectrl::Regex("tq>|tq >")).expect("...");
}
```

**After:**
```rust
fn test_sessions_watch_<name>() {
    let mut p = spawn_tq_repl_tiered("test_sessions_watch_<name>");
    p.expect_stage(Stage::Connect, "Connected to")
        .expect("connect stage: failed to see 'Connected to'");
    std::thread::sleep(Duration::from_secs(1));
    p.session_mut().send_line("/sessions --watch --interval 1")
        .expect("send /sessions --watch");
    std::thread::sleep(Duration::from_secs(2));
    // ... test body using p.session_mut() for sends and p.expect_stage(...) for expects
    p.expect_stage(Stage::Prompt, expectrl::Regex("tq>|tq >"))
        .expect("prompt stage: REPL prompt must reappear");
}
```

**TC097-H specifically:** The two `p.set_expect_timeout(Some(Duration::from_secs(30)))` calls at lines 3554 and 3561 must be deleted. After migration, the `TqPty` wrapper manages timeouts via `expect_stage` stage budgets. Direct `set_expect_timeout` on the inner session bypasses the harness.

### `test_pager_search_prompt_and_exit` — No-Pager Flag Issue

`spawn_tq_repl_tiered` constructs the command as `tq repl --no-syntax-highlight --no-pager`. The `--no-pager` flag disables the pager. The pager search PTY test MUST NOT use this flag.

Solution: author a separate spawn helper or inline the command construction:

```rust
fn spawn_tq_repl_tiered_with_pager(test_name: &str) -> TqPty {
    let bin_path = assert_cmd::cargo::cargo_bin!("tq");
    let cmd = format!("{} repl --no-syntax-highlight", bin_path.display());
    let session = spawn(cmd).expect("Failed to spawn tq");
    TqPty::new(session, test_name, Timeouts::from_env())
}
```

This is a one-line difference from `spawn_tq_repl_tiered`. Whether to add a new helper or pass a flag is the architect's implementation choice; the test-authoring requirement is to not use `--no-pager` in the pager search test.

---

## Strategy Summary

**Total Objectives Analyzed:** 4

**Test Types Required:**
- Unit tests: REQUIRED — Obj 2a (`col_offset` fixture), Obj 2b (ANSI bytes)
- Interactive PTY tests: REQUIRED — Obj 1 (TC097-A..H migration), Obj 2c (pager search)
- Doc review (grep): REQUIRED — Obj 3 (doc text verification), Obj 4 (toolchain file + deny-warnings)
- Remote CI: REQUIRED — Obj 4 (CI pass proof)
- Benchmark: NOT NEEDED

**Estimated Test Counts:**
- New unit tests: 2 (TC102-U01 col_offset, TC103-U01 ANSI bytes)
- New interactive PTY tests: 1 (TC104-I01 pager search) + 8 migrated (TC097-A..H)
- Doc review verifications: 4 (approach.md, philosophy.md, rust-toolchain.toml exists, deny-warnings doc)
- Total new automated tests: 3 (2 unit + 1 PTY)
- Total migrated tests: 8 (TC097-A..H — no new logic, API substitution only)

**Risk Assessment:**
- HIGH risk gaps: none
- MEDIUM risk gaps: TC097 still times out after migration (acceptable with PTY dump evidence)
- LOW risk gaps: DBC.ColumnsV access in pager search test, col_offset exact-value assertion

**Dependencies Required:**
- Live database: YES — for TC097 (TC101-A..H) and TC104-I01
- Network access: YES — same Teradata endpoint
- Specific OS: NO
- New crates: NONE
- PTY harness: Sprint 66 `TqPty` harness SUFFICIENT AS-IS

---

## Strategy Validation Checklist

- [x] Every objective has a complete specification analysis section
- [x] Feature characteristics classified (not assumed)
- [x] Test strategy derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest
- [x] Per-AC assignment table covers all 13 ACs
- [x] Every AC maps to at least one test type
- [x] BLOCKED conditions explicitly stated for DB-dependent tests
- [x] Honest evidence convention documented
- [x] Tool requests stated explicitly (none new; architect cooperation requirements noted)
- [x] TC101/TC102/TC103/TC104/TC105/TC106 authoring notes provided (see tests/cases/)
- [x] `--no-pager` flag issue for pager search test explicitly called out

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-05-29
**Review Status:** DRAFT
**Sprint:** 68
