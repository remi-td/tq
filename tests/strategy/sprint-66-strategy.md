# Sprint 66 Test Strategy: Test Infra Hardening + Process Fixes

**Created:** 2026-04-17
**Author:** quality-validator
**Sprint:** Sprint 66
**Features:**
1. Tiered interactive-test timeouts with PTY buffer dump (`tests/interactive_tests.rs`)
2. Sprint 65 TC097 re-execution with new harness
3. Sequence user-guide writing after implementation (process change)

---

## Feature-by-Feature Test Strategy

### Feature 1: Tiered interactive-test timeouts with PTY buffer dump

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-66-planning.md` — Feature 1 Acceptance Criteria
- Secondary: `docs/sprints/sprint-66-crisis-deliberation.md` — Architect + QA convergent recommendation

**Requirements:**
1. `spawn_tq_repl()` accepts three named timeout tiers: connect/auth (default 45 s), prompt-ready (default 15 s), query-result (default 60 s).
2. Each tier fails with a distinct named error (`PtyError::ConnectTimeout`, `::PromptTimeout`, `::QueryTimeout`).
3. On ANY expect timeout, the last 4096 bytes of the PTY read buffer are written to `tests/results/sprint-66/<test_name>.pty.log` and the failure message references that file.
4. Environment overrides: `TQ_TEST_CONNECT_TIMEOUT`, `TQ_TEST_PROMPT_TIMEOUT`, `TQ_TEST_QUERY_TIMEOUT` (all `u64` seconds).
5. At least one existing passing interactive test is updated to use the new tiered API.
6. A unit test for the buffer-dump path exercises the dump function with synthesised bytes, without a PTY.

**Feature Characteristics:**

**User Interaction Type:**
- [x] Pure Logic (buffer-dump function, timeout-tier construction, env-override parsing — all deterministic)
- [x] Interactive PTY (regression: at least one existing `#[ignore]` interactive test must still pass with the new harness)

**Explanation:** The buffer-dump function writes bytes to a file and can be unit-tested by passing a synthetic `Vec<u8>`. The timeout enum construction is pure data. The env-override parsing is a string→u64 conversion. Only the regression smoke test requires a live PTY + database.

**Observable Behavior:**
- [x] File system side effects — dump function writes `.pty.log` file
- [x] State management — timeout tiers carry env-override values at harness construction time
- [x] Visual output in terminal — regression smoke through existing PTY test

**External Dependencies:**
- [x] File system access — buffer-dump unit test writes a temp file
- [x] Terminal/PTY — regression interactive test only
- [x] Database connection — regression interactive test only

**Validation Challenges:**
1. **PTY buffer capture** — the dump function must be extractable from the expectrl session lifecycle so it can be called with a plain `&[u8]` in unit tests; if it is inlined into the session error handler, a unit test cannot reach it.
2. **Env-override isolation** — unit tests that read env vars must set/clear them within the test to avoid cross-test pollution; use `std::env::set_var` + guard pattern.
3. **Timeout-variant exhaustiveness** — if `PtyError` is a Rust enum, its variants are checked at compile time; the unit test need only construct each variant and confirm the `Display` / `Debug` output contains the tier name.

**Critical Behaviors to Validate:**
1. "buffer-dump function writes last N bytes to the specified path and the file exists with correct content" (AC: unit test for buffer-dump path)
2. "each timeout variant produces a distinct, named error whose message identifies the stage" (AC: named error per tier)
3. "at least one existing passing interactive test runs to completion with the new tiered API" (AC: regression integration)

---

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Pure Logic" checked:
  → Unit tests REQUIRED
  Reason: buffer-dump and timeout-variant construction are deterministic; unit tests are fast and database-free

IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED for regression
  Reason: the new API must not break existing PTY tests

IF "File system side effects" checked:
  → Unit test with temp file assertion REQUIRED
  Reason: confirms dump actually creates the file with expected content
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Buffer-dump function writes correct content and file; timeout-variant enum construction and error messages
- **Approach:** Synthesise a `Vec<u8>` of known bytes, call the dump function with a temp path, assert file exists, size <= 4096, content matches tail of input. Construct each `PtyError` variant, assert `Display` contains tier name.
- **Rationale:** No PTY or database needed. Fast, deterministic, CI-safe.
- **Gap if missing:** File-write bugs and wrong error messages go undetected before interactive tests run.
- **Necessity:** REQUIRED

**Test Type 2: Interactive Tests (expectrl) — Regression**
- **Validates:** At least one existing `#[ignore]` test that previously PASSED continues to pass with the new tiered `spawn_tq_repl()` API.
- **Approach:** Migrate `test_repl_startup_and_quit` (the simplest existing test) to call the new tiered helper. Run with `--ignored`. Confirm it still reaches `Connected to` and `/quit` → `Goodbye!` without a timeout abort.
- **Rationale:** Proves the new API is backward-compatible; the other `#[ignore]` tests are left unchanged to limit blast radius.
- **Gap if missing:** API breakage would ship undetected.
- **Necessity:** REQUIRED

**Test Type 3: Integration Tests**
- **Validates:** Env-override variables (`TQ_TEST_CONNECT_TIMEOUT`, etc.) are read correctly at harness construction.
- **Approach:** In a unit test (no PTY), set the env var, construct the timeout config struct, assert the field matches the env value.
- **Rationale:** Can be done as an extended unit test (no live DB). Verifies the override path is wired end-to-end.
- **Gap if missing:** Env overrides could be silently ignored, making CI tuning impossible.
- **Necessity:** REQUIRED (implemented as unit tests — no DB needed)

---

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (buffer dump) | REQUIRED | Pure function, fast, no PTY | File-write bugs undetected | MUST IMPLEMENT |
| Unit tests (timeout variants) | REQUIRED | Enum construction is pure logic | Wrong tier names in error messages | MUST IMPLEMENT |
| Unit tests (env overrides) | REQUIRED | String→u64 parse, no DB needed | Overrides silently ignored | MUST IMPLEMENT |
| Interactive regression (PTY) | REQUIRED | Backward-compat of new API | API breakage ships undetected | MUST IMPLEMENT (one test) |
| Benchmark tests | NOT NEEDED | No performance requirement defined | N/A | SKIP |

---

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (from planning doc) | Test Type(s) | Test Cases |
|----------------|--------------------------------------|--------------|------------|
| F1-AC-1 | Three named timeout tiers with defaults | Unit (timeout variant construction) | TC-66-U01 |
| F1-AC-2 | Distinct named error per tier | Unit (enum Display/Debug) | TC-66-U02 |
| F1-AC-3 | PTY buffer tail written to `.pty.log` on timeout | Unit (buffer dump function) | TC-66-U03 |
| F1-AC-4 | Env overrides parsed correctly | Unit (env-override parsing) | TC-66-U04 |
| F1-AC-5 | One existing test migrated to new API and passes | Interactive regression | TC-66-I01 |
| F1-AC-6 | Unit test for buffer-dump path (without PTY) | Unit (buffer dump) | TC-66-U03 |

---

#### 5. Gap Analysis

**Interactive tests beyond regression smoke** — the other existing `#[ignore]` tests (tab completion, `test_execute_simple_query`, etc.) are NOT migrated in Sprint 66. They remain on the old `spawn_tq_repl()` signature or can be left unchanged if the new function is a renamed replacement.
- **Risk:** LOW — their pass/fail status predates this sprint; they are not Sprint 66 ACs.
- **Mitigation:** Leave them unchanged unless the architect chooses to rename the old helper.

**Panic-path PTY dump** — if a test panics rather than hitting an `ExpectTimeout`, the dump may not fire.
- **Risk:** LOW — the AC specifies "on any expect timeout"; panic is out of scope.
- **Revisit:** Future sprint if panic-during-PTY-test is observed.

---

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `tests/interactive_tests.rs` (inline `#[cfg(test)]` module) OR `src/testing/pty_harness.rs` unit module, depending on where the architect places the dump function.
- **Framework:** Built-in Rust `#[test]`
- **Test count:** 4 tests (TC-66-U01 through TC-66-U04)
- **Key scenarios:**
  1. TC-66-U01: Construct `TieredTimeouts` with defaults; assert connect=45, prompt=15, query=60.
  2. TC-66-U02: Construct each `PtyError` variant; assert `to_string()` contains "connect", "prompt", "query" respectively.
  3. TC-66-U03: Synthesise 8000 bytes, call `dump_pty_buffer(&bytes, &path)`; assert file exists, byte count <= 4096, content equals last 4096 bytes of input.
  4. TC-66-U04: Set `TQ_TEST_CONNECT_TIMEOUT=90`, construct config, assert connect timeout = 90 s; restore env afterwards.
- **Mocking strategy:** No mocks needed — all functions are pure. Use `tempfile::tempdir()` for TC-66-U03.

**Test Type: Interactive Regression**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl, `#[test] #[ignore]`
- **Test count:** 1 test (TC-66-I01)
- **Key scenario:**
  - TC-66-I01: Call the new tiered `spawn_tq_repl()` (or `spawn_tq_repl_tiered()`); expect `Connected to` within connect timeout; expect REPL prompt within prompt timeout; send `/quit`; expect `Goodbye!` within query timeout (or accept process exit).
- **Implementation notes:** This is a migration of `test_repl_startup_and_quit`. If the architect renames the old helper, the other tests must be updated to call the new one — this is the architect's responsibility; the QA test only validates the new API contract.

---

#### 7. Coverage Sufficiency Assessment

- Unit tests validate: dump function correctness, timeout default values, error variant messages, env-override parsing.
- Interactive regression validates: new API does not break existing passing PTY flow.
- Combined coverage: adequate for the Sprint 66 ACs. The remaining `#[ignore]` tests are not AC-gated for this sprint.

**Known acceptable gap:** No test validates the dump fires during a *real* live timeout (would require a deliberately slow DB query + timeout of <actual latency). This is acceptable because the unit test proves the dump function writes correctly and the harness wires it to the timeout handler — a live-timeout integration test adds no new information at this stage.

---

### Feature 2: Sprint 65 TC097 re-execution with new harness

#### Strategy

**This is a re-execution task, not a new test design task.**

- All 8 Sprint 65 interactive tests (TC097-A through TC097-H as defined in `tests/cases/TC097.md`) are to be run with the new tiered harness after Feature 1 is implemented and TC-66-I01 passes.
- Run command: `cargo test --test interactive_tests -- --ignored 2>&1 | tee tests/results/sprint-66/tc097-rerun.log`
- The quality-validator records outcome per test in `tests/results/sprint-66/tc097-failure-analysis.md` using this schema:

```
| Test ID | Test Name | Outcome | Failure Stage (if failed) | PTY log file |
```

- If a test PASSES: mark AC covered, no further action.
- If a test FAILS: extract the stage from the `PtyError` variant (`ConnectTimeout` / `PromptTimeout` / `QueryTimeout`) and the `.pty.log` tail, and propose a concrete next action (e.g., "connect takes 38 s cold — increase `TQ_TEST_CONNECT_TIMEOUT` to 90 s", or "prompt never appears — investigate auth path").
- A follow-up GitHub issue is created for any test that still fails after raising the timeout to the env-override maximum.

**Test count:** 0 new test cases written. 8 existing tests re-executed.

---

### Feature 3: Sequence user-guide writing after implementation (process change)

#### Strategy

**This feature is not machine-verifiable via `cargo test`.**

Validation is performed by the quality-validator reading the updated process documents and confirming each acceptance criterion by inspection:

| AC | Document to check | Check |
|----|------------------|-------|
| Phase 2 updated: cli-ux-designer produces specs only, no user-guide prose | `.claude/skills/sprint-coordinator/process/phase2-design.md` | Read file, confirm absence of "user guide" authoring instruction |
| Phase 3/4 updated: user-guide prose is explicit Phase 4 step, run AFTER code is merged | `.claude/skills/sprint-coordinator/process/phase3-build-test.md` and `phase4-ship.md` | Read files, confirm sequencing instruction present |
| Phase 4 checklist includes grep-verify instruction for example output strings | `.claude/skills/sprint-coordinator/process/phase4-ship.md` | Read file, confirm "UX verified all user-guide example output strings exist verbatim in source literals (grep check)" line present |
| No existing specs or user-guide content altered | `git diff HEAD~1 -- docs/specifications/ docs/user/` | Confirm zero diff |

The quality-validator documents findings in the sprint report. APPROVED only if all four checks pass. This is explicitly accepted as non-machine-verifiable — process changes are governance, not code.

**Test count:** 0 new test cases. 4 manual AC checks.

---

## Strategy Summary

**Total Features Analyzed:** 3

**Test Types Required:**
- Unit tests: REQUIRED — Feature 1 (buffer dump, timeout variants, env overrides)
- Interactive tests (PTY regression): REQUIRED — Feature 1 (TC-66-I01)
- Re-execution of existing tests: REQUIRED — Feature 2 (TC097-A..H)
- Manual document inspection: REQUIRED — Feature 3 (process change verification)
- Integration tests (DB): NOT NEEDED — env-override parsing covered by unit tests
- Benchmark tests: NOT NEEDED

**Test Counts:**
- New unit tests: 4 (TC-66-U01 through TC-66-U04)
- New interactive regression tests: 1 (TC-66-I01)
- Existing tests re-executed: 8 (TC097-A..H)
- Manual AC checks: 4
- Total new test code: 5 tests

**Risk Assessment:**
- HIGH risk gaps: none
- MEDIUM risk gaps: "live-timeout dump integration" not tested end-to-end (mitigated by unit test proving dump function + harness wiring)
- LOW risk gaps: other `#[ignore]` tests not migrated; panic-path dump not validated

**Dependencies Required:**
- Live database: YES — TC-66-I01 and TC097 re-runs require `TQ_LOGON` set in `.env`
- Network access: YES — same (Teradata endpoint)
- Specific OS: NO
- `tempfile` crate: YES — TC-66-U03 (already in dev-dependencies or add to `[dev-dependencies]`)

---

## Tool Requests for Coordinator

**None.** All planned tests use existing infrastructure:
- Built-in Rust `#[test]` for unit tests
- `expectrl` (already in dev-dependencies) for interactive regression
- `tempfile` crate for the dump-path unit test — confirm it is already in `[dev-dependencies]`; if not, the architect must add it before TC-66-U03 can compile.

The only blocking dependency is the live Teradata endpoint configured in `.env`. If unavailable, TC-66-I01 and the TC097 re-runs must be reported as BLOCKED.

---

## Strategy Validation Checklist

- [x] Every feature has complete specification analysis section
- [x] Feature characteristics are classified (not assumed)
- [x] Test strategy is derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest
- [x] Specification coverage map includes all requirements (Feature 1)
- [x] Every requirement maps to at least one test type
- [x] Test implementation plan is detailed and actionable
- [x] Coverage sufficiency is assessed
- [x] Feature 2 explicitly defers to re-execution, not new test code
- [x] Feature 3 explicitly accepts manual-only verification

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-04-17
**Review Status:** DRAFT
**Submitted for Review:** 2026-04-17
