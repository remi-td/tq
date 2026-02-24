# Sprint 38 Test Strategy: PMON Foundation - System Config & Lock Monitoring

**Created:** 2026-02-24
**Author:** quality-validator
**Sprint:** Sprint 38
**Features:** `/sysconfig` (System Configuration Summary), `/locks` (Session Blocking & Lock Information)

---

## Overview

Sprint 38 delivers the first two PMON (Performance Monitor) features for DBA observability:
1. **Feature 1: System Configuration Summary** (`/sysconfig`, `tq sysconfig`) - 9 acceptance criteria
2. **Feature 2: Session Blocking & Lock Information** (`/locks`, `tq locks`) - 9 acceptance criteria

Both features follow the established `sessions.rs` pattern exactly:
- SQL constant defining query
- Parsed struct with `from_row(row: &[Value]) -> Option<Self>` constructor
- `execute()` function for batch mode (table/csv/json output)
- `execute_for_repl()` function for REPL metacommand handler
- Unit tests inline in `#[cfg(test)] mod tests`

**Total Acceptance Criteria: 18 (9 per feature)**

---

## Feature-by-Feature Test Strategy

### Feature 1: System Configuration Summary (`/sysconfig`, `tq sysconfig`)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-38-planning.md` lines 42-53
- Secondary: `docs/specifications/admin-user-stories.md` Section 1 (US-1.1, US-1.2, US-1.3)
- Context: First PMON command; queries `DBC.DBCInfoV` and `HASHAMP()+1`

**Requirements:**
1. AC-1: `/sysconfig` command queries `DBC.DBCInfoV` for system version and release info
2. AC-2: Command displays total AMP count via `HASHAMP()+1`
3. AC-3: Command displays system version, node count, and AMP/PE topology
4. AC-4: `tq sysconfig` batch mode command with table/csv/json output formats
5. AC-5: Tab completion includes `/sysconfig` in metacommand menu
6. AC-6: Help text documents the command in both compact and extended formats
7. AC-7: Error handling for privilege errors with actionable guidance
8. AC-8: Unit tests for SQL generation, output formatting, and parsing logic
9. AC-9: `/sc` short alias available

**Feature Characteristics:**

**User Interaction Type:**
- ✅ Interactive PTY (REPL `/sysconfig` metacommand)
- ✅ CLI Batch (`tq sysconfig` command)

**Explanation:** This feature has two surfaces: the REPL metacommand (requires interactive PTY simulation) and the batch CLI command (testable via integration tests). Both surfaces share the same underlying data layer.

**Observable Behavior:**
- ✅ Visual output in terminal (table of system config data)
- ✅ Structured data output (JSON, CSV when requested)
- ✅ Database side effects (reads DBC.DBCInfoV, executes HASHAMP()+1)

**External Dependencies:**
- ✅ Database connection (requires live Teradata with DBC privilege access)
- ✅ Terminal/PTY (REPL metacommand requires interactive session)
- ✅ None for unit tests (pure logic: SQL string, struct parsing, formatters)

**Validation Challenges:**
- **Database-specific data**: AMP count, version string are system-dependent - tests must validate structure not exact values
- **Multiple queries**: `/sysconfig` likely requires 2 queries (DBCInfoV + HASHAMP()) - must test combining logic
- **Privilege errors**: Need to test graceful error handling without causing actual permission failures

**Critical Behaviors to Validate:**
1. SQL constants target correct views (`DBC.DBCInfoV`, `HASHAMP()+1`)
2. `SysconfigInfo` struct is correctly populated from query result rows
3. AMP count calculation uses `HASHAMP()+1` pattern
4. Table, CSV, JSON formatters produce correct output structure
5. REPL `execute_for_repl()` displays human-readable summary
6. Batch `execute()` respects `--format` flag
7. Privilege error produces actionable guidance message
8. `/sc` alias routes to same handler as `/sysconfig`
9. Tab completion list includes `/sysconfig` and `/sc`

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" + "REPL metacommand" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: REPL metacommands need PTY simulation for user experience validation

IF "CLI Batch" checked:
  → Integration tests REQUIRED
  Reason: Batch mode validates --format flag, exit codes, stdout output

IF "Database connection" checked:
  → Live DB tests REQUIRED (marked #[ignore])
  Reason: Real system data validates query correctness end-to-end

IF "Pure logic" (SQL string, struct parsing, formatters):
  → Unit tests REQUIRED
  Reason: Core logic must be validated independently without database
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** SQL constants, `SysconfigInfo::from_row()` parsing, formatter functions (table/csv/json), error message text, alias parsing
- **Approach:** Test pure functions directly using `Value::*` mock data. No database needed.
- **Rationale:** AC-8 explicitly requires unit tests. Core parsing and formatting logic must be validated independently for fast feedback.
- **Gap if missing:** Logic errors in struct construction, wrong column extraction, format bugs not caught until live DB test
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests (CLI Batch Mode)**
- **Validates:** `tq sysconfig` CLI command execution, `--format` flag behavior (table/csv/json), exit codes
- **Approach:** `Command::new(tq).arg("sysconfig")` - tests without database use error output, tests with database use live connection
- **Rationale:** AC-4 requires batch mode. Integration tests validate the wiring of CLI args → command dispatch → output without PTY overhead.
- **Gap if missing:** CLI wiring bugs (wrong subcommand name, broken format flag, wrong exit code) undetected
- **Necessity:** ✅ REQUIRED

**Test Type 3: Interactive Tests (REPL, expectrl)**
- **Validates:** `/sysconfig` metacommand behavior in REPL, tab completion includes `/sysconfig`/`/sc`, help text content, alias `\sc` if applicable, error display in REPL context
- **Approach:** Spawn REPL via expectrl, type `/sysconfig`, verify output structure. All marked `#[ignore]` (require live database).
- **Rationale:** AC-5, AC-6 require tab completion and help text validation. REPL context is only testable in PTY mode.
- **Gap if missing:** Tab completion missing, help text wrong, REPL output formatting broken - all invisible to unit/integration tests
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | AC-8 explicit requirement; validates SQL, parsing, formatting | Logic bugs in core data pipeline | MUST IMPLEMENT |
| Integration tests (CLI) | ✅ REQUIRED | AC-4 requires batch mode; validates CLI wiring | CLI dispatch bugs, format flag broken | MUST IMPLEMENT |
| Interactive tests (REPL) | ✅ REQUIRED | AC-5, AC-6 require tab completion + help; REPL behavior not validatable otherwise | Tab completion missing, help text wrong | MUST IMPLEMENT |
| Benchmark tests | ❌ NOT NEEDED | No performance requirements specified | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 3 (Unit, Integration CLI, Interactive REPL)
- ❌ NOT NEEDED test types: 1 (Benchmark)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-1 | Queries DBC.DBCInfoV for version/release info | sprint-38 line 43 | Unit + Interactive | Unit validates SQL constant; interactive validates live query | TC-038-001, TC-038-004 |
| AC-2 | Displays AMP count via HASHAMP()+1 | sprint-38 line 44 | Unit + Interactive | Unit validates AMP calculation logic; interactive validates display | TC-038-001, TC-038-004 |
| AC-3 | Displays version, node count, AMP/PE topology | sprint-38 line 45 | Unit + Interactive | Unit validates struct fields + formatters; interactive validates display | TC-038-001, TC-038-002, TC-038-004 |
| AC-4 | `tq sysconfig` with table/csv/json output | sprint-38 line 46 | Integration | CLI batch testing validates format flags | TC-038-002 |
| AC-5 | Tab completion includes `/sysconfig` | sprint-38 line 47 | Interactive | PTY-only validation of completion menu | TC-038-003 |
| AC-6 | Help text in compact and extended formats | sprint-38 line 48 | Unit + Interactive | Unit validates text content; interactive validates REPL `/help` output | TC-038-003 |
| AC-7 | Error handling for privilege errors | sprint-38 line 49 | Unit + Interactive | Unit validates error message generation; interactive validates display | TC-038-005, TC-038-004 |
| AC-8 | Unit tests for SQL generation, formatting, parsing | sprint-38 line 50 | Unit (meta) | Explicitly requires unit tests | TC-038-001 |
| AC-9 | `/sc` short alias available | sprint-38 line 51 | Unit + Interactive | Unit validates alias routing; interactive validates user experience | TC-038-001, TC-038-004 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type justified by requirement
- ✅ No orphaned requirements

#### 5. Gap Analysis

**Privilege Error Test (AC-7)**
- **Reason:** Testing actual privilege failure requires a user without SELECT on DBC.DBCInfoV - hard to guarantee in test environment
- **What won't be validated:** Real database privilege error path end-to-end
- **Risk assessment:** LOW - Error message logic can be fully unit-tested using mock errors; pattern established in sessions.rs
- **Mitigation:** Unit tests validate error message text; interactive test documents the expected behavior
- **Revisit criteria:** If users report incorrect error messages in production

**Benchmark/Performance Tests**
- **Reason:** No performance requirements specified for sysconfig command
- **Risk assessment:** LOW
- **Revisit criteria:** If system config query shows performance issues in practice

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/sysconfig.rs` - `#[cfg(test)] mod tests`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 12 tests
- **Key scenarios:**
  1. `test_sysconfig_sql_contains_dbcinfov` - SQL constant references DBC.DBCInfoV
  2. `test_sysconfig_sql_contains_hashamp` - SQL constant includes HASHAMP()+1
  3. `test_sysconfiginfo_from_row_valid` - Parse complete row into SysconfigInfo struct
  4. `test_sysconfiginfo_from_row_insufficient_columns` - Returns None for short rows
  5. `test_sysconfiginfo_from_row_null_fields` - Handles NULL values gracefully
  6. `test_format_table_output_nonempty` - Table formatter produces headers + rows
  7. `test_format_table_output_empty` - Empty result produces "(no data)" message
  8. `test_format_csv_output` - CSV formatter produces correct comma-separated lines
  9. `test_format_json_output` - JSON formatter produces valid parseable JSON
  10. `test_privilege_error_message_contains_guidance` - Error message text is actionable
  11. `test_sysconfig_repl_output_shows_amp_count` - REPL summary includes AMP count
  12. `test_sysconfig_repl_output_shows_version` - REPL summary includes version string
- **Mocking strategy:** `DatabaseClient::mock()` for signature-level tests; `Value::*` enum for row construction in parsing tests

**Test Type: Integration Tests (CLI Batch)**
- **Location:** `tests/integration_sysconfig.rs` (new file)
- **Framework:** `std::process::Command` + `assert_cmd` crate
- **Test count estimate:** 5 tests (3 without DB, 2 with DB marked `#[ignore]`)
- **Key scenarios:**
  1. `test_sysconfig_requires_logon_flag` - Without `--logon`, exits non-zero with usage error
  2. `test_sysconfig_subcommand_exists` - `tq help sysconfig` succeeds (validates CLI wiring)
  3. `test_sysconfig_table_format_is_default` - With live DB, default output is table format (DB required, `#[ignore]`)
  4. `test_sysconfig_csv_format` - With live DB, `--format csv` produces CSV headers (DB required, `#[ignore]`)
  5. `test_sysconfig_json_format` - With live DB, `--format json` produces valid JSON array (DB required, `#[ignore]`)
- **Setup requirements:** Live database for `#[ignore]` tests; no DB needed for wiring tests

**Test Type: Interactive Tests (REPL)**
- **Location:** `tests/interactive_tests.rs` (append to existing file)
- **Framework:** expectrl crate (existing infrastructure)
- **Test count estimate:** 5 tests (all marked `#[ignore]`)
- **Key scenarios:**
  1. `test_sysconfig_repl_command_executes` - `/sysconfig` in REPL produces output with AMP count
  2. `test_sysconfig_alias_sc_works` - `/sc` produces same output as `/sysconfig`
  3. `test_sysconfig_tab_completion_shows_command` - Tab after `/sys` completes to `/sysconfig`
  4. `test_sysconfig_help_text_contains_description` - `/help` output includes sysconfig entry
  5. `test_sysconfig_privilege_error_shows_guidance` - Privilege error produces clear guidance text
- **Implementation notes:** All tests require live database (marked `#[ignore]`); run with `cargo test -- --ignored --test-threads=1`

---

### Feature 2: Session Blocking & Lock Information (`/locks`, `tq locks`)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-38-planning.md` lines 59-75
- Secondary: `docs/specifications/admin-user-stories.md` Section 3 (US-3.2, US-3.3, US-3.6)
- Context: Second PMON command; queries `DBC.LockInfoV` or equivalent for lock contention data

**Requirements:**
1. AC-1: `/locks` command queries `DBC.LockInfoV` (or equivalent) for current lock information
2. AC-2: Display shows locked object, lock type (READ/WRITE/EXCLUSIVE), locking session, waiting sessions
3. AC-3: Blocking chain identification - which sessions block which
4. AC-4: `tq locks` batch mode command with table/csv/json output formats
5. AC-5: Tab completion includes `/locks` in metacommand menu
6. AC-6: Help text documents the command in both compact and extended formats
7. AC-7: Error handling for privilege errors with actionable guidance
8. AC-8: Unit tests for SQL generation, output formatting, lock type mapping, and parsing
9. AC-9: `/lk` short alias available

**Feature Characteristics:**

**User Interaction Type:**
- ✅ Interactive PTY (REPL `/locks` metacommand)
- ✅ CLI Batch (`tq locks` command)

**Explanation:** Identical pattern to Feature 1. Two surfaces share the same data layer. REPL requires PTY; batch mode is integration-testable.

**Observable Behavior:**
- ✅ Visual output in terminal (table of lock data, possibly empty if no locks)
- ✅ Structured data output (JSON, CSV when requested)
- ✅ Database side effects (queries DBC.LockInfoV or equivalent)

**External Dependencies:**
- ✅ Database connection (requires live Teradata with lock view access)
- ✅ Terminal/PTY (REPL metacommand)
- ✅ None for unit tests (pure logic: SQL string, struct parsing, lock type mapping, formatters)

**Validation Challenges:**
- **Non-deterministic lock state**: Tests cannot guarantee locks exist when tests run. Tests must handle empty result set gracefully.
- **Lock type enum**: READ/WRITE/EXCLUSIVE/SHARE lock types need mapping from string → display string. Unit-testable.
- **Blocking chain logic**: Chain identification (session A blocks session B) is complex logic - needs thorough unit testing.
- **View availability**: DBC.LockInfoV may not exist on all Teradata versions. Code must handle "view not found" gracefully.
- **Privilege errors**: Same challenge as Feature 1.

**Critical Behaviors to Validate:**
1. SQL constant targets correct view (`DBC.LockInfoV` or equivalent)
2. `LockInfo` struct correctly extracts locked object, lock type, session IDs, wait counts
3. Lock type string mapping: raw values → display strings (READ, WRITE, EXCLUSIVE, SHARE, etc.)
4. Blocking chain logic: correctly identifies which sessions block which
5. Empty result set (no current locks) displays "(no active locks)" message
6. Table, CSV, JSON formatters produce correct structure
7. REPL output distinguishes "no locks" from "error querying locks"
8. Privilege error produces actionable guidance
9. `/lk` alias routes to same handler as `/locks`

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" + "REPL metacommand" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Same as Feature 1

IF "CLI Batch" checked:
  → Integration tests REQUIRED
  Reason: Same as Feature 1

IF "Complex lock type mapping + blocking chain logic" checked:
  → Unit tests REQUIRED with special emphasis on edge cases
  Reason: Lock type mapping and blocking chain logic are complex; unit tests must cover all lock type values and chain scenarios

IF "Non-deterministic lock state" checked:
  → Tests must handle empty result (no locks present)
  Reason: CI/test environment likely has no active locks - cannot rely on locks existing
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** SQL constants, `LockInfo::from_row()` parsing, lock type mapping (`"RD"` → `"READ"`, `"WR"` → `"WRITE"`, `"EX"` → `"EXCLUSIVE"`, `"SR"` → `"SHARE"`), blocking chain logic, formatter functions, error message text
- **Approach:** Construct mock `Value::*` rows for each lock type; test blocking chain with multiple rows representing blocking scenarios
- **Rationale:** AC-8 explicitly requires unit tests. Lock type mapping and blocking chain are the most complex logic - require thorough isolation testing.
- **Gap if missing:** Lock type mapping bugs (wrong display string), chain logic errors (wrong blocker identified), format bugs not caught
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests (CLI Batch Mode)**
- **Validates:** `tq locks` CLI command, `--format` flag, exit codes, "no locks" display behavior
- **Approach:** `Command::new(tq).arg("locks")` - empty result (no locks in test env) tests no-data handling; live DB tests validate full pipeline
- **Rationale:** AC-4 requires batch mode. Integration tests catch CLI wiring bugs.
- **Gap if missing:** CLI dispatch broken, format flag ignored - undetectable by unit tests
- **Necessity:** ✅ REQUIRED

**Test Type 3: Interactive Tests (REPL, expectrl)**
- **Validates:** `/locks` metacommand behavior, tab completion, help text, `/lk` alias, graceful "no locks" display in REPL
- **Approach:** Same as Feature 1 - expectrl PTY tests, all `#[ignore]`
- **Rationale:** AC-5, AC-6 require tab completion and help text validation in REPL context.
- **Gap if missing:** Tab completion missing, help text wrong, alias broken - invisible to other test types
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | AC-8 explicit; complex lock type mapping + chain logic need isolation | Lock type bugs, chain logic errors | MUST IMPLEMENT |
| Integration tests (CLI) | ✅ REQUIRED | AC-4 requires batch mode; validates CLI wiring | CLI dispatch bugs | MUST IMPLEMENT |
| Interactive tests (REPL) | ✅ REQUIRED | AC-5, AC-6 require tab completion + help; REPL behavior PTY-only | Tab completion missing, alias broken | MUST IMPLEMENT |
| Benchmark tests | ❌ NOT NEEDED | No performance requirements | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 3 (Unit, Integration CLI, Interactive REPL)
- ❌ NOT NEEDED test types: 1 (Benchmark)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-1 | Queries DBC.LockInfoV for lock information | sprint-38 line 61 | Unit + Interactive | Unit validates SQL constant; interactive validates live query | TC-038-006, TC-038-009 |
| AC-2 | Displays locked object, lock type, locking/waiting sessions | sprint-38 line 62 | Unit + Interactive | Unit validates struct + formatter fields; interactive validates display | TC-038-006, TC-038-007, TC-038-009 |
| AC-3 | Blocking chain identification | sprint-38 line 63 | Unit | Complex logic - fully unit-testable with mock row data | TC-038-006 |
| AC-4 | `tq locks` with table/csv/json output | sprint-38 line 64 | Integration | CLI batch testing validates format flags | TC-038-007 |
| AC-5 | Tab completion includes `/locks` | sprint-38 line 65 | Interactive | PTY-only validation | TC-038-008 |
| AC-6 | Help text in compact and extended formats | sprint-38 line 66 | Unit + Interactive | Unit validates text; interactive validates REPL `/help` | TC-038-008 |
| AC-7 | Error handling for privilege errors | sprint-38 line 67 | Unit + Interactive | Unit validates error message; interactive validates display | TC-038-010, TC-038-009 |
| AC-8 | Unit tests for SQL, formatting, lock type mapping, parsing | sprint-38 line 68 | Unit (meta) | Explicitly requires unit tests | TC-038-006 |
| AC-9 | `/lk` short alias | sprint-38 line 69 | Unit + Interactive | Unit validates routing; interactive validates UX | TC-038-006, TC-038-009 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type justified by requirement
- ✅ No orphaned requirements

#### 5. Gap Analysis

**Non-Deterministic Lock State**
- **Reason:** Test environment has no guaranteed active locks. Cannot test "active locks present" case without creating artificial lock contention.
- **What won't be validated:** Full end-to-end with real lock data in interactive tests
- **Risk assessment:** LOW - Unit tests fully validate the parsing and display logic with mock data; "no locks" path is more common and fully tested
- **Mitigation:** Unit tests construct mock lock rows explicitly; integration tests validate "no locks" display
- **Revisit criteria:** If users report display issues with actual lock data

**Lock View Availability (DBC.LockInfoV)**
- **Reason:** DBC.LockInfoV may not exist on all Teradata versions. Code must handle "view not found" gracefully, but test environment may not support simulating missing views.
- **What won't be validated:** "View not found" error path end-to-end
- **Risk assessment:** MEDIUM - Sprint planning identifies this as Risk 1 (mitigated by multiple query strategies)
- **Mitigation:** Unit tests validate the error message for "view not found" case; integration test documents the behavior
- **Revisit criteria:** If users on older Teradata versions report errors

**Privilege Error Test**
- **Reason:** Same challenge as Feature 1
- **Risk assessment:** LOW - fully covered by unit tests
- **Mitigation:** Unit tests validate error message content

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/locks.rs` - `#[cfg(test)] mod tests`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 15 tests
- **Key scenarios:**
  1. `test_locks_sql_contains_lockinfov` - SQL references DBC.LockInfoV (or equivalent)
  2. `test_lockinfo_from_row_valid_read_lock` - Parse READ lock row into LockInfo
  3. `test_lockinfo_from_row_valid_write_lock` - Parse WRITE lock row
  4. `test_lockinfo_from_row_valid_exclusive_lock` - Parse EXCLUSIVE lock row
  5. `test_lockinfo_from_row_valid_share_lock` - Parse SHARE lock row
  6. `test_lockinfo_from_row_insufficient_columns` - Returns None for short rows
  7. `test_lockinfo_from_row_null_fields` - Handles NULL values gracefully
  8. `test_lock_type_mapping_read` - "RD" or "Read" maps to "READ"
  9. `test_lock_type_mapping_write` - "WR" or "Write" maps to "WRITE"
  10. `test_lock_type_mapping_exclusive` - "EX" or "Exclusive" maps to "EXCLUSIVE"
  11. `test_lock_type_mapping_unknown` - Unknown lock type preserved as-is
  12. `test_format_table_empty_locks` - Empty lock list produces "(no active locks)" message
  13. `test_format_table_nonempty_locks` - Table formatter produces correct headers
  14. `test_format_csv_locks` - CSV formatter produces comma-separated lines
  15. `test_format_json_locks` - JSON formatter produces valid array
- **Mocking strategy:** `DatabaseClient::mock()` for signature tests; explicit `Value::*` row construction for parsing tests

**Test Type: Integration Tests (CLI Batch)**
- **Location:** `tests/integration_locks.rs` (new file)
- **Framework:** `std::process::Command` + `assert_cmd`
- **Test count estimate:** 5 tests (3 without DB, 2 with DB marked `#[ignore]`)
- **Key scenarios:**
  1. `test_locks_requires_logon_flag` - Without `--logon`, exits non-zero
  2. `test_locks_subcommand_exists` - `tq help locks` succeeds
  3. `test_locks_no_locks_output` - With live DB, shows "(no active locks)" or lock table (DB required, `#[ignore]`)
  4. `test_locks_csv_format` - With live DB, `--format csv` produces CSV (DB required, `#[ignore]`)
  5. `test_locks_json_format` - With live DB, `--format json` produces valid JSON (DB required, `#[ignore]`)
- **Setup requirements:** Live database for `#[ignore]` tests; no DB for wiring tests

**Test Type: Interactive Tests (REPL)**
- **Location:** `tests/interactive_tests.rs` (append to existing)
- **Framework:** expectrl (existing)
- **Test count estimate:** 5 tests (all `#[ignore]`)
- **Key scenarios:**
  1. `test_locks_repl_command_executes` - `/locks` produces output (locks or "no active locks")
  2. `test_locks_alias_lk_works` - `/lk` produces same output as `/locks`
  3. `test_locks_tab_completion_shows_command` - Tab after `/lo` shows `/locks`
  4. `test_locks_help_text_contains_description` - `/help` includes locks entry
  5. `test_locks_privilege_error_shows_guidance` - Privilege error shows actionable text

---

## Strategy Summary

**Total Features Analyzed:** 2

**Test Types Required:**
- Unit tests: ✅ Feature 1 (`/sysconfig`), Feature 2 (`/locks`) - both required
- Integration tests (CLI batch): ✅ Feature 1, Feature 2 - both required
- Interactive tests (REPL): ✅ Feature 1, Feature 2 - both required
- Benchmark tests: ❌ None - explicitly not needed

**Estimated Test Count:**

| Type | Feature 1 (sysconfig) | Feature 2 (locks) | Total New |
|------|-----------------------|-------------------|-----------|
| Unit tests | 12 | 15 | 27 |
| Integration tests (CLI) | 5 (3 no-DB + 2 live-DB) | 5 (3 no-DB + 2 live-DB) | 10 |
| Interactive tests (REPL) | 5 (all `#[ignore]`) | 5 (all `#[ignore]`) | 10 |
| **Total** | **22** | **25** | **47** |

**Baseline:** Sprint 37 delivered ~721 tests
**Target:** ~768 tests (721 + 47 new)

**Test Case Documents to Produce:**

| Test Case ID | Title | Feature | Type |
|--------------|-------|---------|------|
| TC-038-001 | SysconfigInfo SQL and Struct Unit Tests | Feature 1 | Unit |
| TC-038-002 | Sysconfig Batch Mode CLI Tests | Feature 1 | Integration |
| TC-038-003 | Sysconfig REPL Completion and Help | Feature 1 | Interactive |
| TC-038-004 | Sysconfig REPL Command Execution | Feature 1 | Interactive |
| TC-038-005 | Sysconfig Error Handling | Feature 1 | Unit + Interactive |
| TC-038-006 | LockInfo SQL, Parsing, Lock Type Mapping Unit Tests | Feature 2 | Unit |
| TC-038-007 | Locks Batch Mode CLI Tests | Feature 2 | Integration |
| TC-038-008 | Locks REPL Completion and Help | Feature 2 | Interactive |
| TC-038-009 | Locks REPL Command Execution | Feature 2 | Interactive |
| TC-038-010 | Locks Error Handling | Feature 2 | Unit + Interactive |

**Risk Assessment:**
- **HIGH risk gaps:** None
- **MEDIUM risk gaps:**
  - Lock view availability (DBC.LockInfoV) on older Teradata versions (mitigated by fallback design + unit test coverage)
- **LOW risk gaps:**
  - Non-deterministic lock state in live tests (mitigated by unit tests with mock data)
  - Privilege error path end-to-end (mitigated by unit test validation of error messages)
  - Performance testing deferred (no requirements)

**Dependencies Required:**
- Live database: ✅ YES (for interactive tests and live-DB integration tests)
- Network access: ❌ NO
- Specific OS: ❌ NO
- File system: ❌ NO
- External tools: ❌ NO (no new infrastructure needed)

---

## Tool Requirements Assessment

### Current Testing Tools

**Available:**
- ✅ Unit test framework (built-in Rust)
- ✅ Integration test framework (`std::process::Command`, `assert_cmd`)
- ✅ Interactive test framework (`expectrl` + PTY)
- ✅ `DatabaseClient::mock()` for unit testing
- ✅ `Value::*` enum for mock data rows
- ✅ `sessions.rs` as established pattern to follow exactly

**Needed for Sprint 38:**
- ✅ All existing tools sufficient
- ❌ No new tools required

### Tool Assessment Summary

**Can current tools test all Sprint 38 features?** YES - No new infrastructure needed.

Both features follow the established `sessions.rs` pattern exactly, and all existing test infrastructure (unit tests with `Value::*` mocks, integration tests with `Command::new`, interactive tests with `expectrl`) is sufficient.

**Recommendation:** No new tools to build before implementation begins. Existing patterns cover all test types needed.

---

## Test Execution Strategy

### Phase 1: Unit Tests (after implementation)

**Priority:** Critical (validates core logic, no database needed)

**Sequence:**
```bash
# Feature 1 unit tests
cargo test --lib commands::sysconfig::tests

# Feature 2 unit tests
cargo test --lib commands::locks::tests

# All unit tests (regression check)
cargo test --lib
```

**Expected results:** 27 new unit tests passing

### Phase 2: Integration Tests - No Database (CLI wiring)

**Priority:** High (validates CLI argument dispatch)

**Sequence:**
```bash
# Sysconfig CLI wiring
cargo test --test integration_sysconfig

# Locks CLI wiring
cargo test --test integration_locks
```

**Expected results:** 6 new tests passing (3 per feature, no-DB subset)

### Phase 3: Integration Tests - Live Database

**Priority:** High (validates full pipeline)

**Prerequisites:** Live database configured in `.env`

**Sequence:**
```bash
cargo test --test integration_sysconfig -- --ignored
cargo test --test integration_locks -- --ignored
```

**Expected results:** 4 new live-DB integration tests passing (2 per feature)

### Phase 4: Interactive Tests - REPL (Live Database)

**Priority:** High (validates user experience)

**Prerequisites:** Live database connection

**Sequence:**
```bash
cargo test --test interactive_tests sysconfig -- --ignored --test-threads=1
cargo test --test interactive_tests locks -- --ignored --test-threads=1
```

**Expected results:** 10 new interactive tests passing (5 per feature)

### Phase 5: Full Regression

**Priority:** Critical (ensure zero regressions)

**Sequence:**
```bash
# All unit tests
cargo test --lib

# All integration tests (no DB required)
cargo test --test integration_tests
cargo test --test integration_sysconfig
cargo test --test integration_locks

# All interactive tests (requires database)
cargo test --test interactive_tests -- --ignored --test-threads=1

# Expected: ~768 tests passing (721 baseline + 47 new)
```

---

## Coverage Sufficiency Assessment

### Overall Coverage Analysis

**Feature 1 (`/sysconfig`):**
- Unit tests validate: SQL constants, struct parsing from mock rows, all formatters, error messages, alias routing
- Integration tests validate: CLI argument wiring, format flags, exit codes, live query pipeline
- Interactive tests validate: REPL user experience, tab completion, help text, alias UX
- Combined coverage: **Comprehensive**
- Known gap: Privilege error path only unit-tested (not live-DB end-to-end) - LOW risk

**Feature 2 (`/locks`):**
- Unit tests validate: SQL constants, struct parsing (all lock types), lock type mapping, blocking chain logic, formatters, error messages
- Integration tests validate: CLI wiring, format flags, "no locks" display, live query pipeline
- Interactive tests validate: REPL user experience, tab completion, help text, alias UX
- Combined coverage: **Comprehensive with MEDIUM-risk gap (lock view availability on older Teradata)**
- Known gap: Non-deterministic lock state in live tests (no guaranteed locks exist) - LOW risk, mitigated by unit tests

**Question: If all planned tests pass, can we claim features "work as specified"?**

**Answer: YES for nominal cases.** Both features have complete automated test coverage across all three required test types. The known gaps (privilege errors end-to-end, lock view availability on older Teradata, non-deterministic lock state) are either LOW risk or MEDIUM risk with documented mitigations.

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage sufficient to claim "works as specified"
- ✅ Known gaps documented with risk assessment

---

## Strategy Validation Checklist

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
- ✅ Tool requirements assessed (no new tools needed)

**Strategy Status:** READY FOR REVIEW

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-02-24
**Review Status:** DRAFT
**Sprint:** 38 - PMON Foundation (System Config & Lock Monitoring)
**Submitted for Review:** 2026-02-24

**Reviewer:** tq-project-manager (pending)
**Review Status:** PENDING
**Review Date:** (pending)
**Review Comments:** (pending)
