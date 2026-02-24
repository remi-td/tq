# Sprint 39 Test Strategy: PMON Hardening & Query Inspection

**Created:** 2026-02-24
**Author:** quality-validator
**Sprint:** Sprint 39
**Features:** Monitoring Utilities Extraction (Refactor), Sprint 38 Bug Fixes, `/query` (Query Inspection Command)

---

## Overview

Sprint 39 delivers three distinct deliverables across two objectives:

1. **Feature 1: Monitoring Utilities Extraction** (P0 Refactor) - Extract 4x duplicated helper functions from sessions.rs, sysconfig.rs, locks.rs, sample.rs into a shared `monitoring_utils.rs` module. Pure mechanical refactor with zero user-visible behavior change.

2. **Feature 2: Sprint 38 Bug Fixes** (P0 Remediation) - Fix the CSV "(none)" bug in locks.rs, add error handling unit tests for sysconfig.rs and locks.rs, sync documentation.

3. **Feature 3: Query Inspection Command** (P1) - New `/query <session_id>` REPL metacommand and `tq query <session_id>` batch command that shows SQL text from DBC.QryLogV for the given session.

**Confirmed Test Baseline:** 748 tests passing (verified via `cargo test` before sprint work begins)

**Total Acceptance Criteria: 16 (AC-1 through AC-16)**

---

## Feature-by-Feature Test Strategy

---

### Feature 1: Monitoring Utilities Extraction (Refactor)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-39-planning.md` lines 36-50 (AC-1 through AC-5)
- Secondary: Sprint 38 review recommendation #8 (extract shared monitoring utilities)
- Context: Pure refactoring - no user-visible behavior change. All 748 baseline tests serve as regression safety net.

**Requirements:**
1. AC-1: `extract_integer()`, `extract_trimmed_string()`, `extract_decimal()` exist in `src/commands/monitoring_utils.rs`
2. AC-2: `escape_csv()` exists in the shared module (currently duplicated 4x)
3. AC-3: sessions.rs, sysconfig.rs, locks.rs, sample.rs use shared functions with no local copies
4. AC-4: All 748 existing tests pass after refactor
5. AC-5: Zero clippy warnings

**Feature Characteristics:**

**User Interaction Type:**
- Pure Logic (internal code reorganization, no new user interaction)

**Explanation:** This is a mechanical refactor. The user-visible behavior of all four consuming modules (sessions, sysconfig, locks, sample) must remain identical. No new CLI commands, no new output, no changed behavior. The only "user" is the developer.

**Observable Behavior:**
- File system side effects: New `monitoring_utils.rs` file created; four existing files modified
- No change to structured data output
- No change to database side effects (same SQL queries, same logic)

**External Dependencies:**
- None for refactor validation (pure logic / compilation)
- Existing tests already cover the behavioral correctness of each consuming module

**Validation Challenges:**
- **No behavioral surface to test directly**: The refactor itself has no observable output. Validation is entirely regression-based.
- **Import chain correctness**: Must verify that the 4 consuming modules correctly import and use the shared module without local copies remaining.
- **Clippy compliance**: Zero warnings is an explicit acceptance criterion.

**Critical Behaviors to Validate:**
1. All 748 baseline tests still pass after the refactor (AC-4)
2. The shared module exists at the correct path and exports the required functions (AC-1, AC-2)
3. No local duplicate function definitions remain in consuming modules (AC-3)
4. `cargo clippy` reports zero warnings (AC-5)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" checked? NO → Skip interactive tests
IF "CLI Batch" checked? NO → Skip integration tests for this feature
IF "Database connection" checked? NO → Skip live-DB tests
IF "Pure Logic / Refactor" checked? YES → Unit test for module structure
IF "All existing tests as regression" checked? YES → Full regression suite required
```

**Derived Test Types:**

**Test Type 1: Structural Verification Tests (Unit)**
- **Validates:** AC-1 (shared functions exist), AC-2 (escape_csv in shared module), AC-3 (consuming modules use shared module)
- **Approach:** Write unit tests in `src/commands/monitoring_utils.rs` that verify the exported functions exist and behave correctly. These act as module API contracts.
- **Rationale:** Without tests on the shared module itself, there is no guarantee the extraction was correct (e.g., a function could be extracted but with a subtle logic difference).
- **Gap if missing:** A subtly broken extraction (e.g., off-by-one in `extract_integer`) would not be caught until an end-to-end scenario exercises the specific path.
- **Necessity:** REQUIRED

**Test Type 2: Full Regression Suite**
- **Validates:** AC-4 (all 748 tests pass), AC-3 (consuming modules still work correctly)
- **Approach:** Run `cargo test` - all 748 pre-refactor tests serve as the regression gate. This is the primary validation signal for the refactor.
- **Rationale:** If any of the 748 existing tests fail after the refactor, the refactor introduced a regression. The test suite was designed precisely for this scenario (sprint planning calls it "the regression safety net").
- **Gap if missing:** Behavioral regressions in sessions, sysconfig, locks, or sample commands would go undetected.
- **Necessity:** REQUIRED (mandatory gate)

**Test Type 3: Clippy Lint Check**
- **Validates:** AC-5 (zero clippy warnings)
- **Approach:** `cargo clippy -- -D warnings` to treat warnings as errors
- **Rationale:** AC-5 is an explicit acceptance criterion. Clippy must report clean.
- **Gap if missing:** Lint warnings may indicate dead code, unused imports, or style violations left over from the refactor.
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (shared module) | REQUIRED | Validates exported API of new shared module | Silent extraction bugs | MUST IMPLEMENT |
| Full regression suite | REQUIRED | AC-4: All 748 tests must pass - primary validation gate | Any regression in consuming modules | MUST RUN |
| Clippy lint | REQUIRED | AC-5: Zero warnings is explicit AC | Lingering dead code / unused imports | MUST RUN |
| Interactive tests | NOT NEEDED | No user-visible behavior changes | N/A | SKIP |
| Integration tests (new) | NOT NEEDED | No new CLI commands or behaviors | N/A | SKIP |

**Summary:**
- REQUIRED test types: 3 (Unit on shared module, Regression suite, Clippy)
- NOT NEEDED test types: 2 (Interactive, New integration tests)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-1 | extract_integer/trimmed_string/decimal in monitoring_utils.rs | sprint-39 line 41 | Unit (shared module) | Verify function exports and basic behavior | TC-039-001 |
| AC-2 | escape_csv in shared module | sprint-39 line 42 | Unit (shared module) | Verify escape_csv exported correctly | TC-039-001 |
| AC-3 | Consuming modules use shared functions | sprint-39 line 43 | Regression suite | All existing tests pass = consuming modules still correct | TC-039-001 + full regression |
| AC-4 | All 748 existing tests pass | sprint-39 line 44 | Regression suite | `cargo test` produces 748/748 passing | Full regression run |
| AC-5 | Zero clippy warnings | sprint-39 line 45 | Clippy lint | `cargo clippy -- -D warnings` clean | Clippy check |

**Coverage Validation:**
- Every specification requirement appears in table
- Every requirement maps to at least one test type
- Every test type justified by requirement
- No orphaned requirements

#### 5. Gap Analysis

**Interactive and Integration Tests Omitted**
- **Reason:** This is a pure refactor with no user-visible behavior change. No new commands, no new output.
- **What won't be validated:** The refactor is invisible to all user-facing test types by design.
- **Risk assessment:** LOW - The 748-test regression suite provides comprehensive behavioral coverage. The new unit tests on the shared module cover the extraction itself.
- **Mitigation:** Full regression suite is mandatory gate. Clippy enforces code quality.
- **Revisit criteria:** If any behavioral regressions are found during sprint execution.

#### 6. Test Implementation Plan

**Test Type: Unit Tests for monitoring_utils.rs**
- **Location:** `src/commands/monitoring_utils.rs` - `#[cfg(test)] mod tests`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 8 tests
- **Key scenarios:**
  1. `test_extract_integer_from_integer_value` - Returns correct integer from Value::Integer
  2. `test_extract_integer_from_null_value` - Returns default/None for Value::Null
  3. `test_extract_trimmed_string_trims_whitespace` - Trims surrounding spaces
  4. `test_extract_trimmed_string_from_null` - Returns empty string for Value::Null
  5. `test_extract_decimal_from_decimal_value` - Returns f64 for Value::Decimal
  6. `test_extract_decimal_from_null` - Returns 0.0/None for Value::Null
  7. `test_escape_csv_no_special_chars` - Plain string unchanged
  8. `test_escape_csv_with_comma_and_quotes` - Correctly wraps and escapes
- **Mocking strategy:** Direct `Value::*` enum construction; no database needed.

**Test Type: Regression Suite**
- **Location:** `cargo test` (runs all test suites)
- **Framework:** Cargo test runner
- **Test count:** 748 tests (all pre-existing)
- **Pass criteria:** 748/748 passing, 0 failures
- **Command:** `cargo test`

**Test Type: Clippy Check**
- **Command:** `cargo clippy -- -D warnings`
- **Pass criteria:** Exit code 0 (zero warnings, zero errors)

---

### Feature 2: Sprint 38 Bug Fixes & Doc Alignment

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-39-planning.md` lines 55-67 (AC-6 through AC-9)
- Secondary: Sprint 38 review recommendations #1-4
- Context: Remediation of known gaps from Sprint 38. Specifically: CSV bug in locks.rs, missing error handling unit tests in sysconfig.rs and locks.rs, and documentation drift.

**Requirements:**
1. AC-6: CSV output for locks with no waiters uses empty string, not "(none)"
2. AC-7: `docs/design/repl.md` locks section updated to reflect DBC.LockInfoV implementation
3. AC-8: User guide (`docs/user/repl-guide.md`) updated to match actual implementation
4. AC-9: Error handling unit tests added for sysconfig.rs and locks.rs

**Feature Characteristics:**

**User Interaction Type:**
- CLI Batch (CSV output fix is observable in batch mode CSV output)
- Pure Logic (error handling unit tests - no user interaction)
- Documentation (AC-7, AC-8 are doc changes)

**Explanation:** The CSV bug is a specific output correctness defect in batch mode. Error handling tests are pure unit tests. Documentation fixes are not software behavior and require review validation.

**Observable Behavior:**
- Structured data output: CSV format for locks with no waiters must be empty string, not "(none)"
- No other output changes

**External Dependencies:**
- None for unit tests and CSV fix validation (pure logic)
- Documentation changes validated by review (not automated test)

**Validation Challenges:**
- **CSV output regression**: Must prove the "(none)" string is gone and empty string is correct. Need to construct the specific case (lock with no waiters).
- **Error handling tests**: These are new test additions, not fixes to existing behavior. Must verify they exercise meaningful code paths.
- **Doc alignment**: Documentation changes cannot be automatically tested; requires human review validation.

**Critical Behaviors to Validate:**
1. CSV output for a `LockInfo` with no waiting sessions produces empty string in the waiters column (AC-6)
2. Sysconfig error handling tests cover privilege error path and view-not-found path (AC-9)
3. Locks error handling tests cover privilege error and no-locks empty state (AC-9)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "CLI Batch" (CSV output fix):
  → Unit test for the CSV formatting function directly
  → Integration test with live DB to verify CSV output
  Reason: Unit test catches the logic fix; integration test validates full pipeline

IF "Pure Logic" (error handling tests):
  → Unit tests for error code paths
  Reason: Error paths are directly testable without database

IF "Documentation" (AC-7, AC-8):
  → Manual review validation
  Reason: Cannot automatically test prose correctness
```

**Derived Test Types:**

**Test Type 1: Unit Tests (CSV bug fix + error handling)**
- **Validates:** AC-6 (CSV empty string for no-waiter case), AC-9 (new error handling tests)
- **Approach:** Construct a `LockInfo` row where `is_waiting` is false and waiter count is 0; verify CSV formatter produces empty string in that column. For error handling: call error formatting functions with mock errors and assert message content.
- **Rationale:** Unit tests provide fast, deterministic validation of the specific bug fix. They also constitute the AC-9 requirement itself (the fix is adding unit tests).
- **Gap if missing:** The CSV bug could silently remain; error paths might be untested dead code.
- **Necessity:** REQUIRED

**Test Type 2: Documentation Review (manual)**
- **Validates:** AC-7 (design doc sync), AC-8 (user guide alignment)
- **Approach:** Read `docs/design/repl.md` and `docs/user/repl-guide.md`; confirm locks section references DBC.LockInfoV and guide removes unimplemented column references.
- **Rationale:** Documentation correctness cannot be tested with `cargo test`. Requires human verification.
- **Gap if missing:** Documented behavior diverges from implementation, creating confusion for users and future agents.
- **Necessity:** REQUIRED (manual review step in test execution)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (CSV fix + error handling) | REQUIRED | AC-6: specific output correctness; AC-9: explicit requirement for error tests | Bug may still be present; error paths untested | MUST IMPLEMENT |
| Documentation review | REQUIRED | AC-7, AC-8 cannot be verified by automated tests | Doc drift continues uncorrected | MUST EXECUTE MANUALLY |
| Integration tests (CSV format) | RECOMMENDED | Validates full pipeline for CSV bug fix | Live-DB pipeline not tested | SHOULD RUN (if DB available) |
| Interactive tests | NOT NEEDED | CSV and error handling are not REPL-only behaviors | N/A | SKIP |

**Summary:**
- REQUIRED test types: 2 (Unit, Manual doc review)
- RECOMMENDED: 1 (Integration CSV format with live DB)
- NOT NEEDED: 1 (Interactive)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-6 | CSV locks no-waiter = empty string, not "(none)" | sprint-39 line 57 | Unit | Direct formatter test with no-waiter row | TC-039-002 |
| AC-7 | Design doc sync for locks section | sprint-39 line 58 | Manual review | Documentation cannot be auto-tested | TC-039-002 (review step) |
| AC-8 | User guide updated to match implementation | sprint-39 line 59 | Manual review | Documentation cannot be auto-tested | TC-039-002 (review step) |
| AC-9 | Error handling unit tests for sysconfig + locks | sprint-39 line 60 | Unit | Explicitly requires unit tests | TC-039-002 |

**Coverage Validation:**
- Every specification requirement appears in table
- Every requirement maps to at least one test type
- Every test type justified by requirement
- No orphaned requirements

#### 5. Gap Analysis

**Live-DB Integration Test for CSV Fix**
- **Reason:** The test environment may not have active locks. Running `tq locks --format csv` likely produces empty output, not a row with the no-waiter case.
- **What won't be validated:** Full pipeline for the specific no-waiter CSV scenario end-to-end.
- **Risk assessment:** LOW - Unit test directly validates the formatter function with the exact input that triggers the bug. The live-DB test adds marginal value for this specific fix.
- **Mitigation:** Unit test covers the logic. Integration test for general CSV format already planned as part of the Sprint 38 test suite (TC-038-007) and continues to run.
- **Revisit criteria:** If CSV output bugs surface in production with live lock data.

**Documentation Correctness (AC-7, AC-8)**
- **Reason:** No automated test can verify prose accuracy.
- **Risk assessment:** MEDIUM - Incorrect documentation misleads users and future agents.
- **Mitigation:** Explicit manual review step in test execution plan. Validator reads both documents and confirms specific sections.
- **Revisit criteria:** If user reports confusion from documentation.

#### 6. Test Implementation Plan

**Test Type: Unit Tests for CSV Fix and Error Handling**
- **Location:** `src/commands/locks.rs` and `src/commands/sysconfig.rs` - `#[cfg(test)] mod tests`
- **Framework:** Built-in Rust test framework
- **Test count estimate:** 10 tests (5 locks, 5 sysconfig)
- **Key scenarios for locks.rs:**
  1. `test_csv_no_waiter_produces_empty_string` - CSV row for lock with no waiting sessions has empty waiters column (the bug fix regression test)
  2. `test_csv_with_waiter_shows_session_id` - CSV row for lock with waiter shows session ID (regression guard)
  3. `test_privilege_error_message_format` - Error message for privilege failure contains actionable text
  4. `test_view_not_found_error_message` - Error message for DBC.LockInfoV not existing is clear
  5. `test_error_distinguishes_no_data_from_failure` - Empty result vs error produce different messages
- **Key scenarios for sysconfig.rs:**
  1. `test_privilege_error_message_contains_dbc` - References the specific view that requires privilege
  2. `test_privilege_error_message_is_actionable` - Error message includes suggested action
  3. `test_query_error_is_propagated` - Database error propagates correctly (not silently swallowed)
  4. `test_empty_result_handled_gracefully` - Zero rows from DBCInfoV doesn't panic
  5. `test_error_type_classification` - Different error categories produce different messages
- **Mocking strategy:** `Value::*` enum for row construction; mock error types for error path tests.

**Test Type: Manual Documentation Review**
- **Executed during:** Test execution phase
- **Steps:**
  1. Read `docs/design/repl.md` locks section - verify it references DBC.LockInfoV (not MonitorSession)
  2. Read `docs/user/repl-guide.md` - verify Node Count, PE Count, Blocked Since columns are NOT mentioned for `/locks` if not implemented
  3. Confirm documentation matches current locks.rs implementation
- **Pass criteria:** Both documents accurately describe implemented behavior with no phantom columns or wrong SQL source references.

---

### Feature 3: Query Inspection Command (`/query <session_id>`)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-39-planning.md` lines 72-87 (AC-10 through AC-16)
- Secondary: `docs/specifications/admin-user-stories.md` Section 9 (US-9.1)
- Context: New PMON command. Natural workflow: sessions → locks → inspect SQL. Follows exact same pattern as `/sessions`, `/sysconfig`, `/locks`.

**Requirements:**
1. AC-10: `/query <session_id>` shows SQL text from DBC.QryLogV for the given session
2. AC-11: `tq query <session_id>` works in batch mode with table/CSV/JSON output
3. AC-12: Tab completion includes `/query` and alias `/q`
4. AC-13: Help text describes command usage
5. AC-14: Clear error message when session not found or no query logged
6. AC-15: Handles long SQL text gracefully (truncation with full-text option)
7. AC-16: Unit tests for SQL generation, parsing, and display logic

**Feature Characteristics:**

**User Interaction Type:**
- Interactive PTY (REPL `/query <session_id>` metacommand)
- CLI Batch (`tq query <session_id>` command)

**Explanation:** Identical pattern to `/sysconfig` and `/locks`. Two surfaces share the same data layer. The REPL surface requires PTY simulation for tab completion and command execution validation. The batch CLI surface is integration-testable without PTY.

**Observable Behavior:**
- Visual output in terminal: SQL text display with truncation indicator
- Structured data output: JSON, CSV, table when in batch mode with --format flag
- Database side effects: queries DBC.QryLogV

**External Dependencies:**
- Database connection: requires live Teradata with DBC.QryLogV accessible and DBQL logging enabled
- Terminal/PTY: REPL metacommand requires interactive session for tab completion and display validation
- None for unit tests (pure logic: SQL string construction, struct parsing, formatters, truncation logic)

**Validation Challenges:**

- **DBC.QryLogV availability**: DBQL logging must be enabled on the test system. If not enabled, the view exists but returns no rows. Sprint planning identifies this as Risk 1 (medium probability). Tests must handle both "no rows" and "view accessible" scenarios.
- **Session ID dependency**: Unlike `/sessions` or `/locks`, this command requires a specific session ID as argument. In the interactive test, the tester must either use a known session ID or retrieve one from `/sessions` first. Tests must handle the "session not found" case as the primary testable path.
- **Long SQL truncation**: Need to verify truncation behavior with SQL that exceeds the display width. Unit-testable with a crafted long string.
- **DBQL not enabled**: Clear error message must guide the user. Unit-testable.

**Critical Behaviors to Validate:**
1. SQL constant targets `DBC.QryLogV` view (AC-10)
2. `QueryInspectInfo` struct correctly extracts session ID, SQL text, query start time (AC-10, AC-16)
3. Truncation logic: SQL longer than threshold is truncated with indicator (AC-15)
4. "Session not found" error message is clear and actionable (AC-14)
5. "DBQL not enabled" error message provides guidance (AC-14)
6. Batch mode `tq query <session_id>` accepts session ID argument and respects `--format` flag (AC-11)
7. Tab completion includes `/query` and `/q` (AC-12)
8. Help text accurately describes usage including the session_id argument (AC-13)
9. `/q` alias routes to same handler as `/query` (AC-12)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" + "REPL metacommand" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Tab completion (AC-12) and help text (AC-13) in REPL context only validatable with PTY

IF "CLI Batch" checked:
  → Integration tests REQUIRED
  Reason: AC-11 requires batch mode; validates session_id argument parsing, --format flag, exit codes

IF "Database connection" checked:
  → Live DB tests REQUIRED (marked #[ignore])
  Reason: Real DBC.QryLogV query validates SQL correctness end-to-end

IF "Complex argument (session_id)" checked:
  → Unit tests for argument parsing and SQL generation required
  Reason: Session ID must be validated and safely inserted into SQL query

IF "Long SQL truncation" checked (AC-15):
  → Unit tests for truncation logic required
  Reason: Truncation is pure logic - deterministic with known-length strings
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** SQL constant structure (DBC.QryLogV reference), `QueryInspectInfo::from_row()` parsing, truncation logic (AC-15), error message text for session-not-found and DBQL-not-enabled cases (AC-14), alias routing for `/q` (AC-12 partial), help text content (AC-13 partial), SQL generation with session ID parameter (AC-16)
- **Approach:** Construct mock `Value::*` rows to test struct parsing. Use crafted long strings for truncation. Use mock error values for error message tests.
- **Rationale:** AC-16 explicitly requires unit tests. Truncation logic (AC-15) is pure logic best validated in isolation. Error messages (AC-14) need to be tested without live-DB state.
- **Gap if missing:** Truncation edge cases (exact boundary), wrong error messages, SQL construction bugs all go undetected until live DB tests.
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (CLI Batch Mode)**
- **Validates:** `tq query <session_id>` CLI subcommand existence (AC-11), session ID argument parsing, `--format` flag behavior (table/csv/json), exit codes, error output for missing session ID argument
- **Approach:** `Command::new(tq).arg("query").arg("<session_id>")` - no-DB tests validate CLI wiring and argument requirements. Live-DB tests validate full pipeline with `#[ignore]`.
- **Rationale:** AC-11 requires batch mode. Without integration tests, CLI argument parsing bugs (e.g., wrong argument position, missing required arg) would not be detected.
- **Gap if missing:** Batch mode dispatch broken, session_id arg not accepted, format flag ignored - invisible to unit tests.
- **Necessity:** REQUIRED

**Test Type 3: Interactive Tests (REPL, expectrl)**
- **Validates:** `/query <session_id>` metacommand behavior (AC-10), tab completion includes `/query` and `/q` (AC-12), help text in REPL `/help` output (AC-13), alias `/q` user experience, error display for session-not-found in REPL context (AC-14)
- **Approach:** Spawn REPL via expectrl. Type `/q` + Tab to verify completion includes `/query`. Type `/help` and scan output for query entry. Type `/query 99999999` (non-existent session) to verify error message. All tests marked `#[ignore]` (require live database).
- **Rationale:** AC-12 (tab completion) and AC-13 (help text) are REPL-only behaviors that require PTY simulation. The session-not-found path (AC-14) can be exercised in interactive tests without needing a real session to inspect.
- **Gap if missing:** Tab completion missing, help text entry wrong, alias broken, REPL error display broken - none visible to unit or integration tests.
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | AC-16 explicit; truncation logic (AC-15); error messages (AC-14); SQL structure | Truncation bugs, wrong error messages, SQL generation errors | MUST IMPLEMENT |
| Integration tests (CLI) | REQUIRED | AC-11 requires batch mode; validates session_id arg parsing and CLI wiring | CLI dispatch broken, arg parsing wrong | MUST IMPLEMENT |
| Interactive tests (REPL) | REQUIRED | AC-12 tab completion; AC-13 help text; REPL-only behaviors | Tab completion missing, help text wrong, alias broken | MUST IMPLEMENT |
| Benchmark tests | NOT NEEDED | No performance requirements for query inspection | N/A | SKIP |

**Summary:**
- REQUIRED test types: 3 (Unit, Integration CLI, Interactive REPL)
- NOT NEEDED: 1 (Benchmark)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-10 | `/query <session_id>` shows SQL text from DBC.QryLogV | sprint-39 line 76 | Unit + Interactive | Unit validates SQL constant and struct parsing; interactive validates live query display | TC-039-003, TC-039-006 |
| AC-11 | `tq query <session_id>` batch mode with table/csv/json | sprint-39 line 77 | Integration | CLI batch testing validates format flags and session_id arg | TC-039-004 |
| AC-12 | Tab completion includes `/query` and `/q` | sprint-39 line 78 | Interactive | PTY-only validation of completion menu | TC-039-005 |
| AC-13 | Help text describes command usage | sprint-39 line 79 | Unit + Interactive | Unit validates text content; interactive validates REPL `/help` output | TC-039-005 |
| AC-14 | Clear error when session not found or no query logged | sprint-39 line 80 | Unit + Interactive | Unit validates error messages; interactive validates REPL error display | TC-039-003, TC-039-006 |
| AC-15 | Long SQL text truncated with full-text option | sprint-39 line 81 | Unit | Truncation is pure logic - unit-testable with crafted strings | TC-039-003 |
| AC-16 | Unit tests for SQL generation, parsing, display | sprint-39 line 82 | Unit (meta) | Explicitly requires unit tests | TC-039-003 |

**Coverage Validation:**
- Every specification requirement appears in table
- Every requirement maps to at least one test type
- Every test type justified by requirement
- No orphaned requirements

#### 5. Gap Analysis

**DBC.QryLogV Availability and DBQL Logging**
- **Reason:** DBC.QryLogV requires DBQL logging enabled on the Teradata system. The view may exist but return no rows for sessions without logged queries. The interactive tests cannot guarantee a real SQL-generating session exists.
- **What won't be validated:** Full end-to-end successful SQL display with real query text in interactive tests.
- **Risk assessment:** MEDIUM - Sprint planning identifies this as Risk 1. The "no rows" path (session not found / DBQL disabled) is actually the primary testable path and is more important to validate.
- **Mitigation:** Interactive tests use a deliberately non-existent session ID to validate the error path, which is the most important user-facing behavior when DBQL is disabled. Unit tests validate the full display pipeline with mock data. A note is added to mark the successful-display interactive test as conditionally executable.
- **Revisit criteria:** If users report issues with successful SQL display when DBQL is enabled.

**Long SQL Full-Text Option**
- **Reason:** AC-15 mentions "truncation with full-text option" - the spec references a `--full` or `--no-truncate` option. This requires unit and integration tests but the exact flag name depends on implementation.
- **What won't be validated:** The exact flag name is implementation-determined. Tests must be written to match the implemented flag.
- **Risk assessment:** LOW - Core truncation logic is unit-testable regardless of flag name. The flag itself is an integration test.
- **Mitigation:** Unit tests validate the truncation function independently. Integration tests validate the CLI flag once implemented.
- **Revisit criteria:** If the flag is not implemented in Sprint 39 scope.

**Session ID Injection Safety**
- **Reason:** Session IDs are integers. The risk of SQL injection is low (integer type enforcement), but must still validate that non-integer inputs produce appropriate errors rather than malformed SQL.
- **What won't be validated:** Behavior with unusual session ID edge cases (0, negative, MAX_INT).
- **Risk assessment:** LOW - Integer parsing in Rust is type-safe by default. Clap will reject non-integer inputs.
- **Mitigation:** Unit test validates parsing of edge case session IDs. Integration test validates CLI behavior with non-integer argument.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/query_inspect.rs` (or whatever module name is chosen) - `#[cfg(test)] mod tests`
- **Note:** The existing `src/commands/query.rs` implements the SQL execution command (batch query). The new module for PMON query inspection should use a different name (e.g., `query_inspect.rs`) to avoid collision. The exact module name is implementation-determined.
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 14 tests
- **Key scenarios:**
  1. `test_query_inspect_sql_contains_qrylogv` - SQL constant references DBC.QryLogV
  2. `test_query_inspect_sql_uses_session_parameter` - SQL has a session ID placeholder/parameter
  3. `test_query_inspect_info_from_row_valid` - Parse complete row into QueryInspectInfo struct
  4. `test_query_inspect_info_from_row_insufficient_columns` - Returns None for short rows
  5. `test_query_inspect_info_from_row_null_sql_text` - Handles NULL SQL text gracefully
  6. `test_truncation_short_sql_unchanged` - SQL under threshold is displayed without truncation
  7. `test_truncation_long_sql_is_truncated` - SQL over threshold is truncated with indicator
  8. `test_truncation_boundary_exact_threshold` - SQL exactly at threshold is not truncated
  9. `test_truncation_indicator_appended` - Truncated output has "..." or similar indicator
  10. `test_session_not_found_error_message` - Error message when zero rows returned is clear
  11. `test_dbql_not_enabled_error_message` - Error message guides user to enable DBQL
  12. `test_format_table_output_nonempty` - Table formatter produces headers + SQL text row
  13. `test_format_csv_output` - CSV formatter produces correct output
  14. `test_format_json_output` - JSON formatter produces valid parseable JSON
- **Mocking strategy:** `Value::*` enum for row construction; long strings for truncation tests; mock error types for error message tests.

**Test Type: Integration Tests (CLI Batch)**
- **Location:** `tests/integration_query_inspect.rs` (new file)
- **Framework:** `std::process::Command`
- **Test count estimate:** 5 tests (3 without DB, 2 with DB marked `#[ignore]`)
- **Key scenarios:**
  1. `test_query_inspect_requires_session_id_arg` - Without session_id, exits non-zero with usage error
  2. `test_query_inspect_subcommand_exists` - `tq help query` succeeds (validates CLI wiring; note: must not conflict with existing `tq query` SQL execution command - check subcommand naming)
  3. `test_query_inspect_session_not_found_exits_nonzero` - With live DB, non-existent session_id produces error and non-zero exit (DB required, `#[ignore]`)
  4. `test_query_inspect_csv_format` - With live DB, `--format csv` produces CSV headers (DB required, `#[ignore]`)
  5. `test_query_inspect_json_format` - With live DB, `--format json` produces valid JSON array (DB required, `#[ignore]`)
- **Setup requirements:** Live database for `#[ignore]` tests; no DB for wiring tests.
- **Important note:** The subcommand name for the new PMON query inspection must not collide with the existing `tq query` SQL execution subcommand. This is a naming risk that the architect must resolve. Tests must be written against the actual subcommand name chosen (e.g., `tq pmon-query` or `tq inspect-query`).

**Test Type: Interactive Tests (REPL)**
- **Location:** `tests/interactive_tests.rs` (append to existing file)
- **Framework:** expectrl crate (existing infrastructure)
- **Test count estimate:** 5 tests (all marked `#[ignore]`)
- **Key scenarios:**
  1. `test_query_inspect_tab_completion_shows_command` - Tab after `/q` shows `/query` in completion list
  2. `test_query_inspect_alias_q_shows_in_completion` - `/q` appears in completion list
  3. `test_query_inspect_help_text_contains_description` - `/help` output includes query inspection entry with session_id description
  4. `test_query_inspect_session_not_found_shows_error` - `/query 99999999` (non-existent session) displays clear error message
  5. `test_query_inspect_repl_command_executes` - `/query <valid_session_id>` displays SQL text or "no query logged" (DB required, session-dependent; mark as `#[ignore]`)
- **Implementation notes:** All tests require live database. Test 5 (successful display) is the hardest to execute reliably in CI because it requires both DBQL enabled and an active session. All tests marked `#[ignore]`. The "session not found" test (test 4) is the most important and most reliably executable.

---

## Strategy Summary

**Total Features Analyzed:** 3

**Test Types Required:**

| Feature | Unit | Integration | Interactive | Clippy | Manual |
|---------|------|-------------|-------------|--------|--------|
| Feature 1: Monitoring Utils Extraction | REQUIRED (8 new tests) | NOT NEEDED | NOT NEEDED | REQUIRED | NOT NEEDED |
| Feature 2: Sprint 38 Bug Fixes | REQUIRED (10 new tests) | RECOMMENDED | NOT NEEDED | NOT NEEDED | REQUIRED (doc review) |
| Feature 3: Query Inspection | REQUIRED (14 new tests) | REQUIRED (5 tests) | REQUIRED (5 tests) | NOT NEEDED | NOT NEEDED |

**Estimated Test Count:**

| Type | Feature 1 (utils) | Feature 2 (fixes) | Feature 3 (query) | Total New |
|------|--------------------|---------------------|---------------------|-----------|
| Unit tests | 8 | 10 | 14 | **32** |
| Integration tests (CLI) | 0 | 0 | 5 (3 no-DB + 2 live-DB) | **5** |
| Interactive tests (REPL) | 0 | 0 | 5 (all `#[ignore]`) | **5** |
| **Total** | **8** | **10** | **24** | **42** |

**Baseline:** 748 tests (confirmed via `cargo test` pre-sprint)
**Target:** ~790 tests (748 + 42 new)

**Test Case Documents to Produce:**

| Test Case ID | Title | Feature | Type |
|--------------|-------|---------|------|
| TC-039-001 | Monitoring Utils Shared Module Unit Tests | Feature 1 | Unit + Regression |
| TC-039-002 | Sprint 38 Bug Fixes: CSV Output and Error Handling | Feature 2 | Unit + Manual review |
| TC-039-003 | QueryInspectInfo SQL, Parsing, Truncation, Error Unit Tests | Feature 3 | Unit |
| TC-039-004 | Query Inspect Batch Mode CLI Integration Tests | Feature 3 | Integration |
| TC-039-005 | Query Inspect REPL Tab Completion and Help | Feature 3 | Interactive |
| TC-039-006 | Query Inspect REPL Command Execution and Alias | Feature 3 | Interactive |

**Risk Assessment:**

- **HIGH risk gaps:** None
- **MEDIUM risk gaps:**
  - DBC.QryLogV availability (DBQL must be enabled on test system) - interactive and live-DB integration tests for successful query display may not be executable; mitigated by unit tests with mock data and error-path interactive tests.
  - Subcommand naming collision: new `tq query` PMON command must not conflict with existing `tq query` SQL execution command; architect must resolve naming before tests can be finalized.
- **LOW risk gaps:**
  - Privilege error paths for sysconfig and locks (error path not exercisable live; mitigated by unit tests)
  - Non-deterministic lock state in live tests (no guaranteed active locks; mitigated by unit tests)
  - Documentation correctness for AC-7 and AC-8 (manual review only; no automated test)

**Dependencies Required:**

- Live database: YES (for Feature 3 interactive tests and live-DB integration tests)
- Network access: NO
- Specific OS: NO
- External tools: NO (no new infrastructure needed; existing expectrl and Value::* sufficient)
- DBQL enabled on test database: CONDITIONALLY REQUIRED (for Feature 3 "successful display" path)

---

## Subcommand Naming Risk Note

**IMPORTANT:** The sprint planning document lists `src/commands/query.rs` in the Objective 3 file list as a NEW file for the query inspection command. However, `src/commands/query.rs` ALREADY EXISTS - it implements the `tq query <sql>` SQL execution subcommand (the primary tq function).

This creates a naming conflict risk:
- **Existing:** `tq query <sql_statement>` - executes SQL
- **New (AC-11):** `tq query <session_id>` - inspects query text

These have fundamentally different argument semantics. The architect must either:
1. Use a different subcommand name (e.g., `tq pmon query <session_id>` or `tq qlog <session_id>`)
2. Implement the PMON query inspection in a separate module with a distinct CLI name

The quality-validator's integration tests will be written to match whatever subcommand name the architect chooses. This is flagged here as a coordination dependency that must be resolved before TC-039-004 is written.

---

## Test Execution Strategy

### Phase 1: Regression Baseline Verification (before any sprint work)

```bash
# Confirm 748 baseline tests
cargo test 2>&1 | grep "test result:"
# Expected: all suites green, total = 748 passed
```

### Phase 2: Feature 1 Validation (after monitoring_utils.rs extraction)

```bash
# New unit tests on shared module
cargo test --lib commands::monitoring_utils::tests

# Full regression (critical gate - must be 748+8 = 756 passing)
cargo test --lib

# Clippy check
cargo clippy -- -D warnings
```

### Phase 3: Feature 2 Validation (after CSV fix and error tests added)

```bash
# Locks unit tests (includes new CSV fix regression test)
cargo test --lib commands::locks::tests

# Sysconfig unit tests (includes new error handling tests)
cargo test --lib commands::sysconfig::tests

# Full regression (critical gate)
cargo test --lib
```

### Phase 4: Feature 3 Validation (after query inspect command implemented)

```bash
# New unit tests for query inspect
cargo test --lib commands::query_inspect::tests  # (or whatever module name)

# Integration tests - no database required (CLI wiring)
cargo test --test integration_query_inspect

# Full unit test regression
cargo test --lib

# Interactive tests (requires live database)
cargo test --test interactive_tests query_inspect -- --ignored --test-threads=1
```

### Phase 5: Full Regression (final gate)

```bash
# All unit tests
cargo test --lib

# All integration tests (no DB required subset)
cargo test --test integration_tests
cargo test --test integration_query_inspect

# All interactive tests (requires database)
cargo test --test interactive_tests -- --ignored --test-threads=1

# Clippy final check
cargo clippy -- -D warnings

# Expected: ~790 tests passing (748 baseline + 42 new)
```

---

## Coverage Sufficiency Assessment

### Overall Coverage Analysis

**Feature 1 (Monitoring Utils Extraction):**
- Unit tests validate: shared module exports, function behavior with mock values
- Regression suite validates: all consuming modules still work correctly end-to-end
- Clippy validates: no dead code, clean imports
- Combined coverage: **Comprehensive for a pure refactor**

**Feature 2 (Sprint 38 Bug Fixes):**
- Unit tests validate: CSV formatter outputs empty string for no-waiter case (bug fix), error message content for privilege and view-not-found cases
- Manual review validates: design doc and user guide accuracy
- Combined coverage: **Comprehensive for targeted fixes; documentation gap accepted as manual-review-only**

**Feature 3 (Query Inspection):**
- Unit tests validate: SQL constant, struct parsing, truncation logic, error messages, formatters
- Integration tests validate: CLI argument parsing, session_id requirement, format flags, exit codes
- Interactive tests validate: tab completion, help text, alias, REPL error display
- Known gap: Live successful query display depends on DBQL-enabled test system; mitigated by unit tests with mock data
- Combined coverage: **Comprehensive for nominal paths; medium-risk gap on DBC.QryLogV availability mitigated**

**Question: If all planned tests pass, can we claim features "work as specified"?**

**Answer: YES for Feature 1 (refactor) and Feature 2 (bug fixes). YES for Feature 3 nominal paths; the successful-SQL-display path requires DBQL-enabled system and is best-effort.**

---

## Strategy Validation Checklist

- Every feature has complete specification analysis section
- Feature characteristics are classified (not assumed)
- Test strategy is derived from characteristics (not guessed)
- Every test type has clear rationale
- Gap analysis is complete and honest
- Specification coverage map includes all requirements
- Every requirement maps to at least one test type
- Test implementation plan is detailed and actionable
- Coverage sufficiency is assessed
- No hand-waving or vague justifications
- Subcommand naming collision flagged as coordination dependency
- Tool requirements assessed (no new tools needed)

**Strategy Status:** READY FOR REVIEW

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-02-24
**Review Status:** DRAFT
**Sprint:** 39 - PMON Hardening & Query Inspection
**Submitted for Review:** 2026-02-24

**Reviewer:** tq-project-manager (pending)
**Review Status:** PENDING
**Review Date:** (pending)
**Review Comments:** (pending)
