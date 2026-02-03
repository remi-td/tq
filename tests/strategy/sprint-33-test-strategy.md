# Sprint 33 Test Strategy

**Created:** 2026-02-03
**Author:** quality-validator
**Sprint:** Sprint 33
**Features:** Pager Bug Fix + Data Sampling Commands

---

## Instructions Context

This test strategy follows the rigorous specification-driven approach defined in `tests/strategy/test-strategy-template.md`. Every test type is justified by feature characteristics, not assumptions.

---

## Feature-by-Feature Test Strategy

### Feature 1: Pager Bug Fix (Issue #14)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/specifications/repl.md#horizontal-column-navigation` (Pager specifications)
- Secondary: `docs/design/repl.md#pager-architecture` (Pager implementation details)
- GitHub Issue: #14 - "[BUG] Pager broken and on by default"
- Sprint 31 Review: `docs/sprints/sprint-31-review.md` (Two-pass truncation fix)

**Requirements:**
1. AC-2: "Fix implemented - Correct rendering logic to prevent column overflow"
2. AC-3: "Default disabled - `pager_enabled: false` in src/commands/repl/state.rs"
3. AC-4: "Unit tests pass - All existing pager tests (27 tests) pass at 100%"
4. AC-5: "Integration tests pass - No regressions in interactive tests (48 tests)"
5. AC-6: "Manual test case documented - Create TC-033-PAGER-MANUAL.md"

**Feature Characteristics:**

**User Interaction Type:**
- [x] Interactive PTY (REPL, terminal UI with cursor/colors/rendering)

**Explanation:** The pager is a Type 4 feature - Interactive/Alternate Screen mode with full-screen navigation. It uses crossterm's alternate screen buffer, processes keyboard events (j/k/h/l/q), and renders dynamic output based on terminal dimensions.

**Observable Behavior:**
- [x] Visual output in terminal (colors, formatting, layout, cursor position)
- [x] State management (session state, cache, persistence)
- [x] Terminal/PTY (terminal control sequences, cursor positioning)
- [x] Database side effects (query execution for data)

**External Dependencies:**
- [x] Database connection (requires live database for data)
- [x] Terminal/PTY (alternate screen buffer, raw mode, keyboard events)
- [x] Operating system specific features (crossterm abstracts but terminal behavior varies)

**Validation Challenges:**
1. **Alternate screen buffer invisible to test framework** - The pager operates in crossterm's alternate screen, which PTY tests cannot capture
2. **Terminal width effects not reproducible** - Issue #14 occurs at specific terminal widths (117 chars), hard to simulate
3. **Visual alignment not verifiable programmatically** - Column overflow and line breaks are visual issues
4. **Sprint 31 lesson learned** - 100% test pass rate did NOT prevent this bug

**Critical Behaviors to Validate:**
1. Column content does not overflow past display_width (Issue #14 root cause)
2. Lines fit within terminal width without wrapping
3. Cell truncation matches display_width (Sprint 31 two-pass algorithm)
4. Pager is disabled by default (`pager_enabled: false`)
5. User can enable pager with `/pager on` if desired
6. All existing pager functionality remains working (no regressions)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Unit tests cannot validate terminal output, cursor behavior, visual rendering

IF "Visual output in terminal" checked:
  → Interactive tests OR integration tests with output capture REQUIRED
  Reason: Unit tests cannot validate formatting, colors, layout

IF "Terminal/PTY" checked:
  → Interactive tests REQUIRED
  Reason: Alternate screen buffer only exists in real PTY environment

IF "Type 4 Feature (Alternate Screen)" (per docs/testing/approach.md):
  → MANDATORY Manual Validation
  Reason: Alternate screen buffer invisible to automated tests
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Internal logic, cell truncation algorithm, width calculations, state management
- **Approach:** Test the two-pass truncation logic in `TableData::from_query_result()` to ensure cell values are truncated to display_width BEFORE formatting
- **Rationale:** Validates the fix for Sprint 31's root cause (cell value length > display_width causing overflow)
- **Gap if missing:** Logic bugs in truncation algorithm, width calculation errors
- **Necessity:** ✅ REQUIRED

**Test Type 2: Interactive Tests (expectrl)**
- **Validates:** Pager can be enabled/disabled, pager state transitions, REPL integration
- **Approach:** Use existing interactive_tests.rs framework to validate `/pager on`, `/pager off` commands and state behavior
- **Rationale:** Tests state management and REPL command handling
- **Gap if missing:** Pager toggle broken, state persistence issues, REPL integration bugs
- **Necessity:** ✅ REQUIRED

**Test Type 3: Manual Validation**
- **Validates:** Actual visual rendering at terminal widths where bug occurs (117 chars per Issue #14)
- **Approach:** Document step-by-step manual test procedure with evidence capture (script command)
- **Rationale:** This is a Type 4 feature - alternate screen buffer rendering cannot be validated by automated tests
- **Gap if missing:** The EXACT bug reported in Issue #14 (visual column overflow at specific widths)
- **Necessity:** ✅ REQUIRED (BUT NOT EXECUTABLE - NO HUMAN TESTER AVAILABLE)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates Sprint 31 two-pass truncation fix logic | Cell truncation bugs, width calculation errors | ALREADY EXIST (27 tests) - VERIFY PASS |
| Interactive tests (expectrl) | ✅ REQUIRED | Validates pager enable/disable and REPL integration | State management bugs, toggle command failures | ALREADY EXIST (48 tests) - VERIFY PASS |
| Manual validation | ✅ REQUIRED | Only way to validate visual rendering in alternate screen | The exact visual bug reported in Issue #14 | DOCUMENT TEST CASE - NOT EXECUTABLE |

**Summary:**
- ✅ REQUIRED test types: 3 (unit, interactive, manual)
- ⚠️ RECOMMENDED test types: 0
- ❌ NOT NEEDED test types: Benchmarks (no performance requirement)

**CRITICAL CONSTRAINT:** No human tester available for manual validation. Sprint must ship based on:
1. Automated test pass (necessary but not sufficient)
2. Code review of fix
3. Default-disabled safety measure (`pager_enabled: false`)
4. Documented manual test case for future validation

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| AC-1 | Root cause identified | sprint-33-planning.md AC-1 | Code Review | Sprint 31 already identified truncation bug | N/A - Analysis |
| AC-2 | Fix implemented | sprint-33-planning.md AC-2 | Unit + Code Review | Verify two-pass algorithm works correctly | Existing pager.rs tests |
| AC-3 | Default disabled | sprint-33-planning.md AC-3 | Unit | Test ReplState::default() has pager_enabled=false | New: test_pager_disabled_by_default |
| AC-4 | Unit tests pass | sprint-33-planning.md AC-4 | Unit | Run cargo test --lib, verify pager tests | Existing 27 pager tests |
| AC-5 | Integration tests pass | sprint-33-planning.md AC-5 | Interactive (expectrl) | Run cargo test --test interactive_tests --ignored | Existing 48 tests |
| AC-6 | Manual test case documented | sprint-33-planning.md AC-6 | Manual (documented) | Create TC-033-PAGER-MANUAL.md with validation steps | TC-033-PAGER-MANUAL.md |
| AC-7 | User can enable | sprint-33-planning.md AC-7 | Interactive | Test /pager on command works | Existing test_horizontal_paging_* |
| AC-10 | Zero regressions | sprint-33-planning.md AC-10 | Unit + Interactive | All existing tests still pass | All existing tests |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements (missing test coverage)
- [x] No unjustified test types (test types without requirement rationale)

**Coverage Gaps:**
- AC-6 (Manual validation) will be DOCUMENTED but NOT EXECUTED - no human tester available
- This gap is ACKNOWLEDGED and ACCEPTED per sprint planning constraints

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Performance/Benchmark Tests**
- **Reason for omission:** Pager bug is a correctness issue, not performance issue
- **What won't be validated:** Paging speed, memory usage, rendering performance
- **Risk assessment:** LOW - No performance requirements in specification
- **Mitigation:** Monitor user feedback for performance complaints
- **Revisit criteria:** If users report pager is slow or uses excessive memory

**Cross-Platform Compatibility Tests**
- **Reason for omission:** Crossterm abstracts terminal differences, no platform-specific code
- **What won't be validated:** Windows vs Linux vs macOS terminal behavior
- **Risk assessment:** LOW - Crossterm handles platform differences
- **Mitigation:** User reports from different platforms would surface issues
- **Revisit criteria:** If platform-specific bugs are reported

#### 6. Test Implementation Plan

**Test Type: Unit Tests (Existing)**
- **Location:** `src/commands/repl/pager.rs` test module
- **Framework:** Built-in Rust test framework (#[test])
- **Test count estimate:** 27 existing tests
- **Key scenarios covered:**
  1. Cell truncation logic (two-pass algorithm)
  2. Column width calculations
  3. Display width capping (MAX_COLUMN_WIDTH)
  4. TableData construction from QueryResult
- **New test required:**
  - `test_pager_disabled_by_default` in `src/commands/repl/state.rs` to verify AC-3

**Test Type: Interactive Tests (Existing)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 48 existing tests (pager-related subset)
- **Key scenarios covered:**
  1. `/pager on` enables pager
  2. `/pager off` disables pager
  3. Large result sets trigger pager when enabled
  4. Pager navigation (j/k/h/l keys)
  5. Pager exit (q key)
- **No new tests required:** Existing tests cover REPL integration

**Test Type: Manual Validation (New - Documented Only)**
- **Location:** `tests/cases/TC-033-PAGER-MANUAL.md`
- **Framework:** Human tester with terminal and script command
- **Test count estimate:** 1 comprehensive manual test case
- **Key scenarios to cover:**
  1. Start tq REPL at terminal width 117 chars (Issue #14 problematic width)
  2. Execute `SELECT TOP 10 * FROM dbc.databases;`
  3. Verify columns align, no overflow, no line wrapping
  4. Test at additional widths: 80, 120, 160
  5. Capture evidence with `script` command
- **Implementation notes:** WILL NOT BE EXECUTED (no human tester), but provides procedure for future validation

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: Cell truncation logic, width calculations, state defaults ✅
- Interactive tests validate: Pager enable/disable, REPL integration, state management ✅
- Manual validation would validate: Actual visual rendering at problematic terminal widths ⚠️
- Combined coverage: **ADEQUATE for shipping with caveats**

**Gaps in combined coverage:**
- Gap 1: Visual rendering at terminal width 117 (Issue #14) not validated by automated tests
- Gap 2: Alternate screen buffer content not captured by PTY tests
- Gap 3: Actual user-perceived functionality unknown until manual validation

**Acceptance criteria:**
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [ ] Combined coverage is sufficient to claim "works as specified" - NO, but shipping with pager DISABLED BY DEFAULT
- [x] Known gaps are documented and accepted

**Why gaps are acceptable for Sprint 33:**
- Gap 1-3 are acceptable because:
  1. Pager is DISABLED BY DEFAULT (AC-3) - users protected from broken feature
  2. Sprint planning explicitly acknowledges: "Ship without manual validation"
  3. Sprint 31 philosophy applied: "Don't claim pager works without manual validation"
  4. User can opt-in with `/pager on` if they want to test
  5. Future sprint can re-enable default after manual validation
  6. This is honest shipping: feature attempted, not claimed working, default safe

---

### Feature 2: Data Sampling Commands

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/specifications/repl.md` lines 404-405, 2380-2381 (high-level mention)
- Secondary: `docs/sprints/sprint-33-planning.md` AC-1 through AC-15 (detailed requirements)
- Design: `docs/design/repl.md` (to be updated by rust-teradata-architect)

**Requirements (from sprint planning AC-1 through AC-15):**
1. AC-1: `/sample` command implemented - Accepts table name, optional row count
2. AC-2: Default sample size - 10 rows if count not specified
3. AC-3: Sample size validation - Max 1000 rows
4. AC-4: Random sampling - Use Teradata SAMPLE clause
5. AC-5: `/peek` command implemented - Shows first 5 rows + column metadata
6. AC-6: Column info display - Show data types, nullable, precision
7. AC-7: Tab completion - Both commands in metacommand completion menu
8. AC-8: Error handling - Clear messages for invalid tables, permissions, syntax
9. AC-9: Multi-format support - Respect current output format
10. AC-10: Help text updated - `/help` shows both commands
11. AC-11: Batch mode integration - `tq sample` and `tq peek` commands
12. AC-12: Qualified names - Support database.tablename syntax
13. AC-13: Performance - Fast execution even on large tables
14. AC-14: Documentation complete - User guide, specifications, design docs updated
15. AC-15: 100% test coverage - Unit tests + interactive tests for both commands

**Feature Characteristics:**

**User Interaction Type:**
- [x] Interactive PTY (REPL metacommands)
- [x] CLI Batch (batch mode versions: `tq sample`, `tq peek`)

**Explanation:** Data sampling is primarily a Type 1 feature (logic/API) with Type 2 aspects (I/O with database). The commands generate SQL, execute queries, and format output. The REPL interaction uses existing metacommand framework. Batch mode uses standard CLI execution. Output is structured data (table/CSV/JSON), not visual/interactive rendering.

**Observable Behavior:**
- [x] Structured data output (table, CSV, JSON)
- [x] Database side effects (query execution, no data modification)
- [x] State management (current output format, connection)
- [ ] Visual output in terminal - NO special visual rendering beyond standard table output

**External Dependencies:**
- [x] Database connection (requires live database for query execution)
- [ ] Terminal/PTY - NO special terminal features beyond standard output
- [ ] Network access - NO
- [ ] File system access - NO

**Validation Challenges:**
1. Requires live Teradata database for integration testing
2. SAMPLE clause behavior is Teradata-specific (random sampling)
3. Permission errors depend on database configuration
4. Table existence checks depend on test database schema

**Critical Behaviors to Validate:**
1. `/sample <table>` generates correct SQL with SAMPLE clause (AC-4)
2. Default row count is 10 when not specified (AC-2)
3. Row count validation rejects >1000 (AC-3)
4. `/peek <table>` shows first 5 rows + column metadata (AC-5, AC-6)
5. Tab completion suggests both commands (AC-7)
6. Invalid table name produces clear error message (AC-8)
7. Output respects current format (table/CSV/JSON) (AC-9)
8. `/help` documents both commands (AC-10)
9. Batch mode: `tq sample <table>` works (AC-11)
10. Qualified names: `tq sample mydb.mytable` works (AC-12)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: REPL metacommands need validation in PTY environment

IF "CLI Batch" checked:
  → Integration tests REQUIRED
  Reason: Batch mode CLI execution needs validation

IF "Database connection" checked:
  → Integration tests with live database REQUIRED
  Reason: Cannot mock Teradata SAMPLE clause behavior

IF "Structured data output" checked:
  → Unit tests for formatting REQUIRED
  Reason: Output format logic can be unit tested

IF Type 1 feature (Logic/API):
  → Unit tests sufficient for logic validation
  → Integration/Interactive tests for end-to-end validation
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Command parsing, SQL generation, parameter validation, error handling logic
- **Approach:** Test functions that parse `/sample` and `/peek` commands, generate SQL, validate parameters
- **Rationale:** Validates core logic without requiring database
- **Gap if missing:** SQL syntax errors, parameter validation bugs, parsing errors
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests (Live Database)**
- **Validates:** Actual query execution against Teradata, SAMPLE clause behavior, permission errors, table lookup
- **Approach:** Use live Teradata connection to execute generated SQL and verify results
- **Rationale:** Cannot mock Teradata-specific SAMPLE clause semantics
- **Gap if missing:** SQL syntax bugs that pass parsing but fail execution, Teradata-specific behavior issues
- **Necessity:** ✅ REQUIRED

**Test Type 3: Interactive Tests (expectrl)**
- **Validates:** REPL integration, tab completion, help text, user-facing behavior
- **Approach:** Use expectrl to spawn REPL, test `/sample` and `/peek` commands, verify output
- **Rationale:** Validates end-to-end REPL workflow as user experiences it
- **Gap if missing:** REPL integration bugs, tab completion failures, help text missing
- **Necessity:** ✅ REQUIRED

**Test Type 4: Batch Mode Tests**
- **Validates:** `tq sample` and `tq peek` CLI commands work
- **Approach:** Execute `tq sample <table>` as subprocess, verify output
- **Rationale:** Validates CLI argument parsing and batch execution path
- **Gap if missing:** Batch mode broken while REPL works
- **Necessity:** ✅ REQUIRED (for AC-11)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates command parsing, SQL generation, parameter validation | Logic bugs, SQL syntax errors | MUST IMPLEMENT |
| Integration tests (live DB) | ✅ REQUIRED | Validates query execution against Teradata | Database-specific bugs, SAMPLE clause issues | MUST IMPLEMENT |
| Interactive tests (expectrl) | ✅ REQUIRED | Validates REPL integration, tab completion, help text | User experience bugs, REPL integration failures | MUST IMPLEMENT |
| Batch mode tests | ✅ REQUIRED | Validates CLI commands work (AC-11) | Batch mode broken while REPL works | MUST IMPLEMENT |
| Manual tests | ❌ NOT NEEDED | Output is structured data, not visual/interactive rendering | N/A | SKIP |
| Benchmark tests | ❌ NOT NEEDED | AC-13 says "fast" but no specific timing requirement | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 4 (unit, integration, interactive, batch)
- ⚠️ RECOMMENDED test types: 0
- ❌ NOT NEEDED test types: 2 (manual, benchmarks)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| AC-1 | `/sample` command implemented | planning AC-1 | Unit + Interactive + Integration | Full validation: parsing + REPL + execution | TC-033-SAMPLE-001 |
| AC-2 | Default sample size 10 | planning AC-2 | Unit + Integration | Test default parameter handling | TC-033-SAMPLE-002 |
| AC-3 | Max 1000 rows validation | planning AC-3 | Unit | Parameter validation logic | TC-033-SAMPLE-003 |
| AC-4 | Random sampling (SAMPLE) | planning AC-4 | Unit + Integration | SQL generation + execution | TC-033-SAMPLE-004 |
| AC-5 | `/peek` command implemented | planning AC-5 | Unit + Interactive + Integration | Full validation: parsing + REPL + execution | TC-033-PEEK-001 |
| AC-6 | Column info display | planning AC-6 | Integration | Requires database metadata | TC-033-PEEK-002 |
| AC-7 | Tab completion | planning AC-7 | Interactive | PTY simulation required | TC-033-TAB-001 |
| AC-8 | Error handling | planning AC-8 | Unit + Integration | Unit: error messages; Integration: actual errors | TC-033-ERROR-001 |
| AC-9 | Multi-format support | planning AC-9 | Integration | Output formatting with different formats | TC-033-FORMAT-001 |
| AC-10 | Help text updated | planning AC-10 | Interactive | REPL `/help` command | TC-033-HELP-001 |
| AC-11 | Batch mode integration | planning AC-11 | Batch Mode Tests | CLI subprocess execution | TC-033-BATCH-001 |
| AC-12 | Qualified names | planning AC-12 | Unit + Integration | Parsing + execution | TC-033-QUALIFIED-001 |
| AC-13 | Performance (fast) | planning AC-13 | Integration (manual verification) | No specific timing requirement, just observe | N/A - Manual observation |
| AC-14 | Documentation complete | planning AC-14 | Manual Review | Review docs for updates | N/A - Doc review |
| AC-15 | 100% test coverage | planning AC-15 | All test types | Comprehensive validation | All TC-033-* cases |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements (missing test coverage)
- [x] No unjustified test types (test types without requirement rationale)

**Coverage Gaps:**
- AC-13 (Performance) has no specific timing requirement, will observe during integration tests
- AC-14 (Documentation) is manual review task, not automated test

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Manual Validation Tests**
- **Reason for omission:** Data sampling commands produce structured output (table/CSV/JSON), not visual/interactive features
- **What won't be validated:** Subjective UX quality, visual appearance preferences
- **Risk assessment:** LOW - Output format is deterministic and testable programmatically
- **Mitigation:** Integration tests validate actual output format and content
- **Revisit criteria:** If users report confusing output or UX issues

**Performance/Benchmark Tests**
- **Reason for omission:** AC-13 says "fast execution" but no specific timing requirement (no SLA like "<100ms")
- **What won't be validated:** Exact query execution time, memory usage
- **Risk assessment:** LOW - SAMPLE clause is Teradata-native optimization, expected to be fast
- **Mitigation:** Observe execution time during integration tests, monitor user feedback
- **Revisit criteria:** If users report slow execution or performance complaints

#### 6. Test Implementation Plan

**Test Type: Unit Tests (New)**
- **Location:** `src/commands/repl/sample.rs` test module (to be created)
- **Framework:** Built-in Rust test framework (#[test])
- **Test count estimate:** 15-20 tests
- **Key scenarios to cover:**
  1. Parse `/sample employees` → generates `SELECT * FROM employees SAMPLE 10`
  2. Parse `/sample employees 50` → generates `SELECT * FROM employees SAMPLE 50`
  3. Parse `/sample employees 1001` → returns validation error (max 1000)
  4. Parse `/sample` with no args → returns error "Missing table name"
  5. Parse `/peek employees` → generates metadata query + `SELECT TOP 5 * FROM employees`
  6. Parse `/peek mydb.mytable` → qualified name handling
  7. Parse `/peek` with no args → returns error
  8. Error message formatting (clear, actionable)
- **Mocking strategy:** No database, test pure logic - SQL generation, parameter validation

**Test Type: Integration Tests (New)**
- **Location:** `tests/integration_tests.rs` (add to existing file)
- **Framework:** Built-in Rust integration test support
- **Test count estimate:** 10-15 tests
- **Key scenarios to cover:**
  1. Execute `/sample dbc.databases` → returns 10 rows (default)
  2. Execute `/sample dbc.databases 5` → returns 5 rows
  3. Execute `/sample nonexistent_table` → clear error message
  4. Execute `/peek dbc.databases` → returns 5 rows + column metadata
  5. Execute with different output formats (table, CSV, JSON)
  6. Execute with qualified names (mydb.mytable)
  7. Permission errors (if test database allows)
- **Setup requirements:** Live Teradata connection, TQ_LOGON environment variable
- **Marked with:** `#[ignore]` attribute (requires live database)

**Test Type: Interactive Tests (New)**
- **Location:** `tests/interactive_tests.rs` (add to existing file)
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 8-10 tests
- **Key scenarios to cover:**
  1. Type `/sample dbc.databases` in REPL → displays table output
  2. Type `/peek dbc.databases` in REPL → displays rows + metadata
  3. Type `/sam<TAB>` → tab completion suggests `/sample`
  4. Type `/pee<TAB>` → tab completion suggests `/peek`
  5. Type `/help` → output contains `/sample` and `/peek` documentation
  6. Invalid command `/sample` with no args → error message
  7. Multi-line behavior (if applicable)
- **Implementation notes:** Use existing expectrl helpers, `#[ignore]` attribute (requires database)

**Test Type: Batch Mode Tests (New)**
- **Location:** `tests/integration_tests.rs` or new `tests/batch_mode_tests.rs`
- **Framework:** std::process::Command to spawn tq subprocess
- **Test count estimate:** 4-6 tests
- **Key scenarios to cover:**
  1. `tq sample dbc.databases` → stdout contains table output
  2. `tq sample dbc.databases 5` → stdout contains 5 rows
  3. `tq peek dbc.databases` → stdout contains metadata + rows
  4. `tq sample nonexistent` → stderr contains error, exit code ≠ 0
  5. `tq sample --format json dbc.databases` → JSON output
- **Setup requirements:** Compiled tq binary, TQ_LOGON environment variable

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: Command parsing, SQL generation, parameter validation, error messages ✅
- Integration tests validate: Query execution, Teradata SAMPLE clause, output formats, error handling ✅
- Interactive tests validate: REPL integration, tab completion, help text, user workflow ✅
- Batch mode tests validate: CLI commands work, argument parsing, output ✅
- Combined coverage: **COMPREHENSIVE - Sufficient to claim "works as specified"**

**Gaps in combined coverage:**
- Gap 1: Performance not benchmarked (AC-13 has no specific timing requirement)
  - Acceptable because: No SLA defined, can observe during integration tests
- Gap 2: Documentation updates (AC-14) are manual review, not automated test
  - Acceptable because: Documentation is human artifact, not code

**Acceptance criteria:**
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified"
- [x] Known gaps are documented and accepted

**Why combined coverage is sufficient:**
- This is a Type 1 feature (logic/API) with Type 2 aspects (database I/O)
- Output is structured data (table/CSV/JSON), not visual/interactive rendering
- All observable behaviors are testable programmatically
- REPL integration uses existing framework (tab completion, help text, command parsing)
- No alternate screen buffer, no special visual rendering
- Per `docs/testing/approach.md`, Type 1/2 features do NOT require manual validation

---

## Strategy Summary

**Total Features Analyzed:** 2

**Test Types Required:**
- Unit tests: ✅ Feature 1 (existing, verify pass), Feature 2 (new, implement)
- Interactive tests: ✅ Feature 1 (existing, verify pass), Feature 2 (new, implement)
- Integration tests: ✅ Feature 2 (new, implement)
- Batch mode tests: ✅ Feature 2 (new, implement)
- Manual validation: ⚠️ Feature 1 (documented, not executable)
- Benchmark tests: ❌ Neither feature (no performance requirements)

**Estimated Test Count:**

**Feature 1 (Pager Bug Fix):**
- Unit: 27 existing tests (verify pass) + 1 new test (pager_disabled_by_default)
- Interactive: 48 existing tests (verify pass)
- Manual: 1 test case documented (TC-033-PAGER-MANUAL.md) - NOT EXECUTABLE
- **Subtotal Feature 1:** 76 automated tests, 1 manual test case

**Feature 2 (Data Sampling):**
- Unit: 15-20 new tests
- Integration: 10-15 new tests
- Interactive: 8-10 new tests
- Batch mode: 4-6 new tests
- **Subtotal Feature 2:** 37-51 new tests

**Grand Total:**
- Automated tests: 113-127 tests (76 existing + 37-51 new)
- Manual test cases: 1 (documented only)

**Risk Assessment:**

**HIGH risk gaps:**
- **Feature 1 (Pager):** Manual validation will not be executed - visual rendering bug (Issue #14) cannot be confirmed fixed
  - **Mitigation:** Pager disabled by default (`pager_enabled: false`), users protected from broken feature
  - **Acceptance:** Sprint planning explicitly acknowledges shipping without manual validation

**MEDIUM risk gaps:**
- **Feature 2 (Data Sampling):** Performance not benchmarked (AC-13)
  - **Mitigation:** Observe execution time during integration tests, SAMPLE clause is Teradata-native optimization
  - **Acceptance:** No specific timing SLA defined in requirements

**LOW risk gaps:**
- None identified

**Dependencies Required:**
- Live database: **YES** - Required for integration and interactive tests (both features)
- Network access: **NO**
- Specific OS: **NO** - Cross-platform via crossterm (Feature 1) and standard Rust (Feature 2)
- Other: TQ_LOGON environment variable or .env file for test database connection

---

## Tool Assessment and Requests

### Current Test Infrastructure

**Existing Tools:**
1. **Unit test framework** - Rust built-in `#[test]` - ✅ SUFFICIENT
2. **Integration test framework** - Rust built-in integration tests - ✅ SUFFICIENT
3. **Interactive test framework** - expectrl crate for PTY simulation - ✅ SUFFICIENT
4. **Test helpers** - Existing `spawn_tq_repl()`, `read_available_output()` functions - ✅ SUFFICIENT

**Assessment:** Existing test infrastructure is SUFFICIENT for Sprint 33 testing. No new tools required.

### Tool Gaps Analysis

**Potential Gap 1: Batch Mode Test Helpers**
- **What's needed:** Helper function to spawn `tq` as subprocess with arguments
- **Why needed:** Reduces duplication in batch mode tests (AC-11)
- **Priority:** LOW - Can be implemented inline in tests
- **Recommendation:** Create `spawn_tq_batch()` helper function during test implementation

**Potential Gap 2: Database Fixture Management**
- **What's needed:** Helper to create/drop test tables for data sampling tests
- **Why needed:** Clean test isolation, avoid polluting test database
- **Priority:** LOW - Can use existing system tables (dbc.databases) for most tests
- **Recommendation:** Use system tables where possible, create test tables only if needed

**Potential Gap 3: Output Format Assertion Helpers**
- **What's needed:** Helper functions to validate table/CSV/JSON output format
- **Why needed:** Reduce duplication when testing AC-9 (multi-format support)
- **Priority:** LOW - Can use existing format validation logic
- **Recommendation:** Extract helpers from existing tests if duplication emerges

### Tool Requests

**NO NEW TOOLS REQUIRED** for Sprint 33.

**Rationale:**
1. Existing test infrastructure covers all required test types
2. Both features use standard testing approaches (unit, integration, interactive)
3. Helper functions can be extracted during implementation if duplication emerges
4. Adding new test infrastructure carries risk of over-engineering (Sprint 30 lesson learned)

**Sprint 30 Lesson Applied:**
Sprint 30 built 1,552 lines of test infrastructure (visual_validator.rs, terminal_simulator.rs) that caught zero bugs. For Sprint 33, we prioritize simple, effective tests using existing tools over building new infrastructure.

---

## Strategy Validation Checklist

**Before submitting to tq-project-manager for review:**

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
**Created Date:** 2026-02-03
**Review Status:** READY FOR REVIEW

**Key Decisions:**
1. **Feature 1 (Pager):** Ship with automated tests + default-disabled + documented manual test case (honest assessment, user protection)
2. **Feature 2 (Data Sampling):** Comprehensive automated testing sufficient (Type 1/2 feature, no manual validation required)
3. **Tool Requests:** None - existing infrastructure sufficient
4. **Test Count:** 113-127 automated tests (76 existing verify + 37-51 new)
5. **Risk:** HIGH risk for pager (no manual validation) mitigated by default-disabled setting

**Next Steps:**
1. Implement new tests for Feature 2 (data sampling)
2. Verify all existing tests pass (Features 1 pager tests)
3. Create TC-033-PAGER-MANUAL.md (manual test case documentation)
4. Execute test suite and produce test report
