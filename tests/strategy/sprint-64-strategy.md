# Sprint 64 Test Strategy: File Mode Parser & Stdin Detection Bug Fixes

**Created:** 2026-04-17
**Author:** quality-validator
**Sprint:** Sprint 64
**Features:**
1. Track BEGIN/END depth in file-mode statement splitter (Issue #42)
2. Correct stdin detection when stdin is redirected but empty (Issue #43)

---

## Feature-by-Feature Test Strategy

### Feature 1: BEGIN/END Depth Tracking in Statement Splitter (#42)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-64-planning.md` Feature 1 Acceptance Criteria
- Secondary: GitHub Issue #42 repro script and description
- Requirements:
  1. "`tq query --file repro_sp.sql` submits the entire `REPLACE PROCEDURE ... BEGIN ... END;` as a single statement"
  2. "Nested BEGIN/END blocks (e.g., `BEGIN ... IF THEN ... END IF; ... END`) are handled correctly"
  3. "String literals containing `BEGIN` or `END` do NOT affect block depth"
  4. "Comments (`--` line and `/* */` block) containing BEGIN/END do not affect block depth"
  5. "Plain multi-statement scripts WITHOUT procedure bodies continue to split correctly"
  6. "`CREATE | REPLACE PROCEDURE | TRIGGER | MACRO` headers are detected case-insensitively"
  7. "No regression in existing `--file` splitter tests"

**Feature Characteristics:**

**User Interaction Type:** Pure Logic (state-machine lexer in `src/sql/parser.rs`) + CLI Batch (end-to-end `tq query --file` with live DB for integration validation).

**Explanation:** The core fix is a deterministic state-machine extension. The lexer is a pure function: `parse_statements(sql: &str) -> Result<Vec<ParsedStatement>, ParseError>`. Its correctness is fully testable without any external dependency. The end-to-end CLI path (`--file` flag reads a file and calls the splitter) requires a live DB only to confirm the whole procedure deploys successfully.

**Observable Behavior:**
- [x] Structured data output (the `Vec<ParsedStatement>` returned by the splitter — verifiable in unit tests)
- [x] Database side effects (procedure compiled in Teradata — only in integration test with live DB)

**External Dependencies:**
- [ ] Database connection — NOT required for unit tests; `parse_statements()` is a pure function
- [x] Database connection — REQUIRED for the one integration test (deploy the repro script against live Teradata)
- [x] File system access — integration test reads a `.sql` fixture file

**Validation Challenges:**
- The parser must distinguish keyword `BEGIN` appearing as a statement opener versus `BEGIN` as a fragment inside a string literal or comment. This requires the state-machine composition to be correct (InProcedureBody does not override InSingleQuotedString or InLineComment).
- Header detection must be case-insensitive and span possible whitespace between keywords (e.g., `REPLACE   PROCEDURE`).
- `END IF`, `END LOOP`, `END CASE`, `END FOR` must decrement depth by 1 (they terminate a nested block, not the procedure). Only a bare `END;` at depth 1 terminates the procedure.
- Unit tests must assert the full procedure body text is preserved verbatim (semicolons inside the body are NOT stripped).

**Critical Behaviors to Validate:**
1. "Single REPLACE PROCEDURE body is emitted as exactly one statement" — the repro script from #42 must produce `statements.len() == 1`
2. "Nested END IF / END LOOP do not close the outer body" — block depth counter tracks sub-block nesting
3. "BEGIN inside a string literal does not open a block" — string-state preempts keyword detection
4. "BEGIN inside a line or block comment does not open a block" — comment-state preempts keyword detection
5. "Plain `SELECT 1; SELECT 2;` still yields two statements" — no regression in non-SPL paths

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Pure Logic" (parse_statements is a pure function):
  -> Unit tests REQUIRED
  Reason: All 8 acceptance criteria except live-DB compilation are
  fully testable via assert_eq! on Vec<ParsedStatement>

IF "CLI Batch" (tq query --file <file>):
  -> Integration test with live DB REQUIRED
  Reason: Unit tests do not confirm the procedure actually compiles in
  Teradata; only a real connection validates end-to-end deployment

IF "Database connection" checked for integration test:
  -> Integration test marked #[ignore], requires TQ_LOGON
  Reason: Must not block CI when DB is unavailable
```

**Derived Test Types:**

**Test Type 1: Unit Tests (in-module `#[cfg(test)]` in `src/sql/parser.rs`)**
- **Validates:** All BEGIN/END depth-tracking logic. Statement count, statement content, nested blocks, string/comment immunity, case-insensitivity, multi-procedure scripts, mixed SPL + regular statements.
- **Approach:** Call `parse_statements(sql)` directly with crafted SQL strings. Assert `Vec<ParsedStatement>` length and content.
- **Rationale:** Pure function — no external dependencies needed. Covers 8 of 8 acceptance criteria for parser logic correctness.
- **Gap if missing:** Parser regressions would only be caught at runtime against the live DB. All edge-case logic (nested blocks, string literals, comments) would be unvalidated.
- **Necessity:** REQUIRED

**Test Type 2: Integration Test with Live Database (`tests/integration_tests.rs`, `#[ignore]`)**
- **Validates:** End-to-end: `tq query --file repro_sp.sql` compiles the procedure successfully on Teradata. Confirms the fix works through the full CLI stack, not just the parser in isolation.
- **Approach:** Write the repro script from Issue #42 to a temp file, run `tq` binary as a `Command::new`, assert exit code 0 and absence of error output.
- **Rationale:** Unit tests cannot verify that the SQL text produced by the parser is accepted by Teradata. The exact repro from #42 must pass end-to-end.
- **Gap if missing:** Parser could produce correct-looking SQL that Teradata still rejects for reasons unrelated to splitting (e.g., whitespace artifact, statement count error).
- **Necessity:** REQUIRED (blocked when DB unavailable — mark `#[ignore]`)

**Test Type 3: Regression Guard (existing unit tests must still pass)**
- **Validates:** All 20+ existing `parse_statements` tests in `src/sql/parser.rs` continue to pass — plain multi-statement scripts, quoted semicolons, comment stripping, error cases.
- **Approach:** Run `cargo test --lib` — existing tests act as the regression suite.
- **Rationale:** The BEGIN/END extension must not break normal multi-statement splitting.
- **Gap if missing:** Silent regression in non-SPL `--file` usage.
- **Necessity:** REQUIRED (already exists — no new work, just must pass)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (parser logic) | REQUIRED | Pure function, all logic testable without DB | All edge-case logic unvalidated | MUST IMPLEMENT |
| Integration test (live DB, `#[ignore]`) | REQUIRED | End-to-end validation of the exact repro from #42 | Parser fix might produce text Teradata rejects | MUST IMPLEMENT |
| Regression guard (existing tests) | REQUIRED | Non-SPL splitting must not regress | Silent breakage of existing `--file` users | ALREADY EXISTS |
| Interactive tests (expectrl) | NOT NEEDED | `--file` is a non-interactive CLI batch path | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirement for the parser | N/A | SKIP |

**Summary:**
- REQUIRED test types: 3 — MUST implement all
- NOT NEEDED test types: 2 — explicitly omitted

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Test Type(s) | Test Cases |
|----------------|-----------------|--------------|------------|
| #42-AC-1 | "repro_sp.sql submits entire REPLACE PROCEDURE as single statement" | Unit + Integration | TC094 Part A, TC094 Part I |
| #42-AC-2 | "Nested BEGIN/END blocks handled correctly" | Unit | TC094 Part B |
| #42-AC-3 | "String literals containing BEGIN/END do not affect block depth" | Unit | TC094 Part C |
| #42-AC-4 | "Comments containing BEGIN/END do not affect block depth" | Unit | TC094 Part D |
| #42-AC-5 | "Plain multi-statement scripts continue to split correctly" | Unit + Regression | TC094 Part E, existing tests |
| #42-AC-6 | "CREATE/REPLACE PROCEDURE/TRIGGER/MACRO detected case-insensitively" | Unit | TC094 Parts F, G |
| #42-AC-7 | "Unit tests cover single procedure, nested, comments, strings, multi-proc, mixed" | Unit | TC094 Parts A–H |
| #42-AC-8 | "No regression in existing splitter tests" | Regression guard | Existing test suite |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements
- [x] No unjustified test types

#### 5. Gap Analysis

**Interactive/PTY Tests**
- **Reason for omission:** `tq query --file` is a non-interactive batch path. No terminal rendering or cursor behavior involved.
- **What won't be validated:** Nothing — the batch path has no interactive component.
- **Risk:** NONE

**Benchmark Tests**
- **Reason for omission:** No performance requirement for the parser in the specification.
- **Risk:** LOW

**Integration Test when DB unavailable**
- **Reason:** Must be `#[ignore]` — live Teradata instance required.
- **What won't be validated:** End-to-end deployment path when DB is offline.
- **Risk:** LOW — unit tests validate parser logic completely; integration test confirms DB acceptance only.
- **Mitigation:** Developer runs `cargo test -- --ignored` with live DB before merging.
- **Revisit:** Required if CI gains a Teradata instance.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/sql/parser.rs` — `#[cfg(test)]` module (append after existing tests, under `// --- Bug #42: BEGIN/END depth tracking ---` comment)
- **Framework:** Built-in Rust `#[test]`
- **Test count:** 9 unit tests (TC094 Parts A–H + regression sentinel)
- **Key scenarios:**
  1. TC094-A: Single procedure — exact repro from #42 → 1 statement, body contains `DECLARE v INTEGER`
  2. TC094-B: Nested BEGIN/END — IF/THEN/END IF inside body → 1 statement, inner semicolons preserved
  3. TC094-C: BEGIN inside string literal — `'BEGIN'` in body does not open new depth
  4. TC094-D: BEGIN/END inside comments — `-- BEGIN` and `/* END */` do not affect depth
  5. TC094-E: Multi-procedure script — two REPLACE PROCEDUREs → 2 statements
  6. TC094-F: Mixed SPL + regular — PROCEDURE followed by SELECT → 2 statements, SELECT splits normally
  7. TC094-G: Case-insensitive headers — `replace procedure`, `CREATE TRIGGER`, `create macro` all detected
  8. TC094-H: CREATE vs REPLACE — both `CREATE PROCEDURE` and `REPLACE PROCEDURE` trigger body tracking
  9. TC094-I: Regression — existing `SELECT 1; SELECT 2;` still yields 2 statements (guard)
- **Mocking strategy:** None. `parse_statements()` is pure.

**Test Type: Integration Test (live DB)**
- **Location:** `tests/integration_tests.rs` — new `#[ignore]` test block
- **Framework:** `std::process::Command` to invoke the `tq` binary; `tempfile` crate for temp SQL file
- **Test count:** 1 test
- **Key scenario:** Write repro_sp.sql from #42 to a temp file, run `tq query --file <path>`, assert exit code 0. Optionally assert stdout contains "1 statement(s)" or no error text.
- **Setup requirements:** `TQ_LOGON` env var or `.env` file, `demo_user` schema writable (or substitute a schema that is writable — the test should parameterize the schema)

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Unit tests validate: all 8 parser-logic acceptance criteria — statement count, body preservation, nested depth, string/comment immunity, case-insensitivity, multi-procedure, mixed scripts
- Integration test validates: end-to-end deployment path — Teradata accepts the procedure text produced by the fixed parser
- Regression guard validates: no existing `--file` usage broken

**Combined coverage:** Comprehensive. Every acceptance criterion has at least one test. The only gap is the integration test being conditional on DB availability, which is explicitly accepted and documented.

**Acceptance criteria:**
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified"
- [x] Known gaps documented and accepted

---

### Feature 2: Correct Stdin Detection When Stdin Redirected but Empty (#43)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-64-planning.md` Feature 2 Acceptance Criteria
- Secondary: GitHub Issue #43 description and suggested fix
- Requirements:
  1. "`tq query \"SELECT 1\" < /dev/null` runs the query successfully"
  2. "`tq query \"SELECT 1\" <<< \"\"` runs the query successfully"
  3. "`echo \"SELECT 2\" | tq query` still reads from stdin as before (regression guard)"
  4. "`echo \"SELECT 2\" | tq query \"SELECT 1\"` still rejects with 'multiple input sources' error (regression guard)"
  5. "`tq query \"SELECT 1\"` in an interactive terminal (TTY stdin) still works"
  6. "Error message quality unchanged for the legitimate conflict case"
  7. "Unit tests or integration test covering all four scenarios above"

**Feature Characteristics:**

**User Interaction Type:** CLI Batch — the fix lives in `determine_input_source()` in `src/commands/query.rs`. The function's logic depends on OS-level I/O state (`isatty`, bytes available) that cannot be replicated via a pure unit test. All meaningful tests must spawn the `tq` binary as a subprocess with controlled file descriptors.

**Explanation:** Unlike the parser which is a pure function, `determine_input_source()` calls `io::stdin().is_terminal()`, which inspects the process's actual stdin file descriptor. Mocking this without refactoring to inject a trait is non-trivial and risks testing the mock rather than the real behavior. Process-level integration tests with fd redirection give direct proof of the fix.

**Observable Behavior:**
- [x] Structured data output (query result when fix works; error message when conflict detected)
- [x] Database side effects (query sent to Teradata when no conflict — required for AC-1 through AC-3)

**External Dependencies:**
- [x] Database connection — REQUIRED for AC-1, AC-2, AC-3, AC-4 (the process must run a query to confirm no error)
- [x] File system access — `/dev/null` used as redirect source in AC-1
- [x] Operating system specific — `/dev/null` and `<<<` are POSIX/Unix shell features; tests must run on macOS/Linux (confirmed platform: macOS 24.6.0)

**Validation Challenges:**
- Tests must spawn `tq` as a subprocess and redirect stdin using `std::process::Command::stdin(Stdio::null())` or similar. The Rust test harness does not support fd manipulation at the `#[test]` level.
- The `<<< ""` heredoc (AC-2) behaves differently from `/dev/null`: the shell opens a pipe and immediately closes it after writing 0 bytes. The fix must handle both cases. In a Rust subprocess test, `Stdio::piped()` with zero bytes written to the pipe writer before dropping it replicates this behavior.
- AC-5 (TTY stdin works) cannot be tested headlessly — the CI environment does not have a TTY attached to stdin. This AC is validated by manual smoke test on developer workstation or by confirming the logic: the fix only changes behavior when `isatty(0) == false`, so TTY case is unchanged.
- The fix requires the DB to be reachable for the positive cases (AC-1 through AC-3). Without a DB, these tests are BLOCKED.

**Critical Behaviors to Validate:**
1. "Empty pipe from /dev/null: positional arg succeeds" — `Command::stdin(Stdio::null())` produces no error (AC-1)
2. "Empty heredoc pipe: positional arg succeeds" — `Command::stdin(Stdio::piped())` with pipe writer dropped immediately (AC-2)
3. "Non-empty pipe, no arg: stdin used as source" — `echo "SELECT 1" | tq query` succeeds (AC-3 regression)
4. "Non-empty pipe + positional arg: error emitted" — `echo "SELECT 1" | tq query "SELECT 2"` exits non-zero with 'Multiple input sources' message (AC-4 regression)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "CLI Batch" (determine_input_source depends on OS stdin fd):
  -> Integration tests spawning the binary REQUIRED
  Reason: isatty() and bytes-available peek must be tested against a real
  file descriptor; unit tests cannot mock OS-level fd state without
  invasive refactoring

IF "Database connection" checked:
  -> Live DB required for positive tests (AC-1, AC-2, AC-3)
  -> AC-4 (error path) does NOT need a DB — tq exits before connecting
  -> Mark live-DB tests as #[ignore]

IF "Operating system specific" (/dev/null, piped stdin):
  -> Tests verified on macOS (the only declared platform)
  -> No Windows concern (tq is macOS/Linux targeted)
```

**Derived Test Types:**

**Test Type 1: Process Integration Tests (live DB, `#[ignore]` for positive-path tests)**
- **Validates:** AC-1 (`/dev/null` redirect), AC-2 (empty heredoc pipe), AC-3 (stdin-only regression), AC-4 (real conflict regression).
- **Approach:** Use `std::process::Command` to spawn `cargo run -- query "SELECT 1"` (or the compiled binary) with different stdin configurations:
  - `Stdio::null()` for `/dev/null`
  - `Stdio::piped()` with pipe writer dropped for empty heredoc
  - `Stdio::piped()` with `"SELECT 2"` written and flushed for real pipe input
  Check exit code and stderr/stdout.
- **Rationale:** Only process-level fd control gives accurate simulation of shell redirections.
- **Gap if missing:** The stdin detection fix cannot be verified without actual OS fd manipulation. Code review of the fix logic is not a substitute.
- **Necessity:** REQUIRED

**Test Type 2: Unit Test (error message content, no DB needed)**
- **Validates:** AC-6 — error message quality for the legitimate conflict case. Verifies the error string from `determine_input_source` matches specification.
- **Approach:** The error message string is a constant in `query.rs`. A unit test can verify its content directly (no process spawn needed).
- **Rationale:** Low-cost validation that the error message wording matches specification. Can run without DB.
- **Gap if missing:** Silent regression if error message is accidentally changed.
- **Necessity:** RECOMMENDED (already partially covered by existing `test_input_source_description`)

**Test Type 3: AC-4 Error Path Test (no DB needed)**
- **Validates:** AC-4 — `tq query "SELECT 1"` with non-empty piped stdin exits with the "multiple input sources" error. This is an error path that does not reach the DB.
- **Approach:** Process integration test: spawn binary with `Stdio::piped()`, write `"SELECT 2\n"` to the stdin pipe, provide positional arg. Assert exit code non-zero and stderr contains "Multiple input sources".
- **Rationale:** Error path exits before DB connection — no `#[ignore]` needed.
- **Necessity:** REQUIRED (regression guard, no DB dependency)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Process integration (live DB, `#[ignore]`) — AC-1, AC-2, AC-3 | REQUIRED | Only OS-level fd control validates the fix | Fix not proven to work against real stdin states | MUST IMPLEMENT |
| Process integration (no DB) — AC-4 error path | REQUIRED | Regression guard; no DB needed | Silent regression if conflict detection breaks | MUST IMPLEMENT |
| Unit test for error message text | RECOMMENDED | Validates AC-6 message wording | Silent regression on error message content | SHOULD IMPLEMENT |
| Interactive tests (expectrl) | NOT NEEDED | No REPL/PTY interaction in `tq query` batch path | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirement | N/A | SKIP |

**Summary:**
- REQUIRED test types: 2 — MUST implement
- RECOMMENDED test types: 1 — should implement
- NOT NEEDED test types: 2 — explicitly omitted

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Test Type(s) | Test Cases |
|----------------|-----------------|--------------|------------|
| #43-AC-1 | "`tq query \"SQL\" < /dev/null` runs successfully" | Process integration (live DB, `#[ignore]`) | TC095 Part A |
| #43-AC-2 | "`tq query \"SQL\" <<< \"\"` runs successfully" | Process integration (live DB, `#[ignore]`) | TC095 Part B |
| #43-AC-3 | "`echo SQL | tq query` still reads stdin (regression)" | Process integration (live DB, `#[ignore]`) | TC095 Part C |
| #43-AC-4 | "`echo SQL | tq query \"SQL\"` rejects with multiple-sources error (regression)" | Process integration (no DB) | TC095 Part D |
| #43-AC-5 | "TTY stdin still works (interactive terminal)" | Manual smoke test only | — |
| #43-AC-6 | "Error message quality unchanged for legitimate conflict" | Unit test (no DB) | TC095 Part E |
| #43-AC-7 | "Unit tests or integration tests covering all four scenarios" | Process integration | TC095 Parts A–D |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] AC-5 explicitly documented as manual-only with justification
- [x] No unjustified test types

**Coverage Gaps:**
- AC-5 (TTY stdin) is not machine-testable without a real PTY. This is acceptable: the fix only changes behavior on the `isatty() == false` branch, so TTY behavior is structurally unchanged. Manual smoke test suffices.

#### 5. Gap Analysis

**AC-5: TTY Stdin (Interactive Terminal)**
- **Reason for omission:** Cannot assign a real TTY to a subprocess stdin in headless test environments. Spawning with `Stdio::inherit()` would inherit the test harness's stdin, not a TTY.
- **What won't be validated:** `isatty(0) == true` branch behavior.
- **Risk assessment:** LOW — The fix only modifies the `!isatty(0)` branch. TTY path is structurally unmodified by the fix.
- **Mitigation:** Manual smoke test: run `tq query "SELECT 1"` in a terminal after the fix is applied.
- **Revisit:** If regression reported in interactive terminal mode.

**Interactive Tests (expectrl)**
- **Reason for omission:** `tq query` is a batch command, not a REPL. No PTY/terminal-control behavior to validate.
- **Risk:** NONE

#### 6. Test Implementation Plan

**Test Type: Process Integration Tests (with live DB)**
- **Location:** `tests/integration_tests.rs` — new `#[ignore]` test block
- **Framework:** `std::process::Command`, `std::process::Stdio`
- **Test count:** 3 live-DB tests (TC095 Parts A, B, C) + 1 no-DB test (TC095 Part D) + 1 unit test (TC095 Part E)
- **Key scenarios:**
  1. TC095-A: Spawn binary with `Stdio::null()` as stdin + positional arg `"SELECT 1 AS x"` → exit code 0
  2. TC095-B: Spawn binary with `Stdio::piped()` as stdin, drop the pipe writer immediately, positional arg `"SELECT 1 AS x"` → exit code 0
  3. TC095-C: Spawn binary with `Stdio::piped()` as stdin, write `"SELECT 1 AS x\n"` to pipe, no positional arg → exit code 0, stdout contains row data
  4. TC095-D: Spawn binary with `Stdio::piped()` as stdin, write `"SELECT 2\n"` to pipe, also pass positional arg `"SELECT 1"` → exit code non-zero, stderr contains "Multiple input sources"
  5. TC095-E: Unit test — assert the error constant in `query.rs` contains the expected substring "Multiple input sources provided"
- **Setup requirements:** `TQ_LOGON` env var or `.env` file for Parts A, B, C. Part D and E have no DB dependency.
- **Binary invocation:** Use `env!("CARGO_BIN_EXE_tq")` macro to get the compiled binary path, or `cargo run --bin tq -- query ...`.

**Tool Note:** The `std::process::Command` + `Stdio` API is part of the Rust standard library — no new crates needed. The test infrastructure already uses this pattern in `tests/integration_tests.rs` for live-DB tests. No new testing tools are required.

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Process integration tests validate: all four runtime stdin scenarios (AC-1, AC-2, AC-3, AC-4)
- Unit test validates: error message content (AC-6)
- Manual smoke test covers: TTY path (AC-5)
- Combined coverage: **comprehensive** — all machine-testable acceptance criteria have automated tests; the one untestable criterion (AC-5) is explicitly accepted with low risk

**Gaps in combined coverage:**
- AC-5 (TTY stdin): Not machine-validated. Acceptable given the structural argument above.

**Acceptance criteria:**
- [x] All specification requirements have test coverage (machine or documented manual)
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified"
- [x] Known gaps documented and accepted

---

## Tool Needs Assessment

### Existing Infrastructure — Sufficient

- `std::process::Command` with `Stdio::null()`, `Stdio::piped()`: already in Rust std — handles `/dev/null` and empty-pipe simulation without any new tools
- `env!("CARGO_BIN_EXE_tq")` macro: built into Cargo — resolves path to compiled binary
- `tempfile` crate: already in dev-dependencies (used in existing tests) — creates temp `.sql` fixture files for the `--file` integration test
- `dotenvy`: already present — loads `TQ_LOGON` from `.env`

### No New Tools Required

No new crates, binaries, or testing utilities are needed for Sprint 64. All required test infrastructure is already present.

**FLAG TO COORDINATOR:** No tool requests. Both bugs are testable with current infrastructure.

---

## Strategy Summary

**Total Features Analyzed:** 2

**Test Types Required:**

| Feature | Unit Tests | Process Integration (live DB) | Process Integration (no DB) | Manual |
|---------|-----------|------------------------------|-----------------------------|--------|
| #42 BEGIN/END parser | REQUIRED (9 tests) | REQUIRED (1 test, `#[ignore]`) | — (regression via existing tests) | — |
| #43 Stdin detection | RECOMMENDED (1 test) | REQUIRED (3 tests, `#[ignore]`) | REQUIRED (1 test) | AC-5 only |

**Estimated Test Count:**

| Category | Count |
|----------|-------|
| Unit tests — parser BEGIN/END (new, `src/sql/parser.rs`) | 9 |
| Unit test — error message content (`tests/integration_tests.rs`) | 1 |
| Process integration, live DB, `#[ignore]` — #42 integration | 1 |
| Process integration, live DB, `#[ignore]` — #43 AC-1, AC-2, AC-3 | 3 |
| Process integration, no DB — #43 AC-4 | 1 |
| **Total new tests** | **15** |

**Risk Assessment:**
- HIGH risk gaps: None
- MEDIUM risk gaps: None
- LOW risk gaps:
  - #42 integration test: DB-dependent, `#[ignore]` — mitigated by complete unit test coverage of parser logic
  - #43 AC-5 (TTY path): manually validated — mitigated by structural argument that fix is branch-isolated

**Dependencies Required:**
- Live database: Yes (for `#[ignore]` integration tests in both features)
- Network access: No
- Specific OS: macOS / Linux (`/dev/null`, POSIX fd semantics)
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
**Created Date:** 2026-04-17
**Review Status:** DRAFT
**Submitted for Review:** 2026-04-17
