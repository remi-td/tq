# Sprint 36 Test Strategy: Help Text Update + REPL Enhancements

**Created:** 2026-02-13
**Author:** quality-validator
**Sprint:** Sprint 36
**Features:** Config Help Text Polish (7 ACs), `/repeat` Command (7 ACs), `/show indexes` Command (10 ACs)

---

## Overview

Sprint 36 delivers configuration polish and two practical REPL productivity enhancements. All features are highly testable with existing infrastructure.

**Sprint Context:**
- Polish Sprint 35's project config feature with help text improvements
- Add `/repeat` metacommand (re-execute last query)
- Add `/show indexes <table>` metacommand (schema inspection)
- **Total Acceptance Criteria: 24** (7 + 7 + 10)
- **All features testable** with existing test infrastructure
- **No database required** for unit tests (integration tests need database for `/show indexes`)

---

## Feature-by-Feature Test Strategy

### Feature 1: Config Help Text & UX Polish (7 ACs) - P0

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-36-planning.md` lines 44-86
- Secondary: `docs/specifications/configuration.md` (Configuration Hierarchy)
- Context: Sprint 35 review recommendations

**Requirements:**
1. "Update `tq help config` with project config section" (AC-1, line 74)
2. "`tq help config` shows 5-level precedence hierarchy" (AC-2, line 75)
3. "`tq profiles` shows project config file path when present" (AC-3, line 76)
4. "`tq profiles` shows tip about project config when no profiles exist" (AC-4, line 77)
5. "Invalid `.tq.toml` produces stderr warning with file path and error details" (AC-5, line 78)
6. "All existing tests pass (zero regressions)" (AC-6, line 79)
7. "New unit + integration tests for all sub-features" (AC-7, line 80)

**Feature Characteristics:**

**User Interaction Type:** ✅ CLI Batch
**Explanation:** All help text and profile listing features are batch commands (`tq help config`, `tq profiles`). No interactive PTY involvement.

**Observable Behavior:**
- ✅ Structured data output (help text content, profile listing format)
- ✅ File system side effects (reads `.tq.toml` for warning validation)
- ✅ Error messages (stderr warnings for invalid TOML)

**External Dependencies:**
- ✅ File system access (reads `.tq.toml` to test invalid config warnings)
- ❌ No database required (help text is static content)

**Validation Challenges:**
- **Help text content**: Must validate exact text includes project config section and precedence hierarchy
- **Profile output format**: Must verify project config path displayed correctly
- **Empty state messages**: Must validate tip appears when no profiles exist
- **Error handling**: Must verify stderr warning format with file path and error details

**Critical Behaviors to Validate:**
1. `tq help config` output includes project config section with `.tq.toml` description
2. `tq help config` shows 5-level precedence: defaults → user → project → env → CLI
3. `tq profiles` displays project config file path when `.tq.toml` found
4. `tq profiles` omits project config line when no `.tq.toml` exists (no noise)
5. `tq profiles` shows helpful tip when no profiles defined
6. Invalid `.tq.toml` produces stderr warning with format: "Warning: Invalid project config at /path/.tq.toml: <parse error>"
7. Invalid project config is non-blocking (continues operation, graceful degradation)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "CLI Batch" + "Structured output" checked:
  → Integration tests REQUIRED
  Reason: End-to-end CLI execution validates user-visible output

IF "Error messages" + "File system side effects" checked:
  → Unit tests REQUIRED
  Reason: Error handling logic needs isolated validation
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Help text content generation, error message formatting, profile display logic
- **Approach:** Test help generation functions in isolation with mock configurations
- **Rationale:** Pure logic components (text generation, formatting) must be validated independently
- **Gap if missing:** Logic errors in help text not caught until integration
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests**
- **Validates:** Full CLI behavior (`tq help config`, `tq profiles` with various config states)
- **Approach:** Create test directories with `.tq.toml`, execute commands, validate stdout/stderr
- **Rationale:** End-to-end user experience must be validated with real CLI invocations
- **Gap if missing:** CLI integration bugs, output format issues, missing text
- **Necessity:** ✅ REQUIRED

**Test Type 3: Interactive Tests**
- **Validates:** N/A - help text is CLI batch only, not REPL-specific
- **Approach:** N/A
- **Rationale:** Help text accessed via CLI, not REPL metacommands
- **Gap if missing:** None - REPL not involved
- **Necessity:** ❌ NOT NEEDED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates help text content and error formatting | Logic bugs in text generation | MUST IMPLEMENT |
| Integration tests | ✅ REQUIRED | Validates CLI output with real file systems | Help text missing, format wrong | MUST IMPLEMENT |
| Interactive tests | ❌ NOT NEEDED | Help text is CLI batch, not REPL | N/A | SKIP |
| Manual tests | ⚠️ RECOMMENDED | Human validates text clarity and UX | Confusing help text not caught | DOCUMENT CHECKLIST |

**Summary:**
- ✅ REQUIRED test types: 2 (Unit, Integration)
- ⚠️ RECOMMENDED test types: 1 (Manual)
- ❌ NOT NEEDED test types: 1 (Interactive)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-1 | `tq help config` includes project config section | sprint-36-planning.md line 74 | Unit + Integration | Unit tests content, integration tests CLI | TC-036-001 |
| AC-2 | `tq help config` shows 5-level precedence hierarchy | sprint-36-planning.md line 75 | Unit + Integration | Unit tests format, integration tests output | TC-036-001 |
| AC-3 | `tq profiles` shows project config path when present | sprint-36-planning.md line 76 | Integration | Must validate actual CLI behavior | TC-036-002 |
| AC-4 | `tq profiles` shows tip when no profiles exist | sprint-36-planning.md line 77 | Integration | Must test empty state message | TC-036-002 |
| AC-5 | Invalid `.tq.toml` produces stderr warning | sprint-36-planning.md line 78 | Unit + Integration | Unit tests format, integration tests stderr | TC-036-003 |
| AC-6 | All existing tests pass (zero regressions) | sprint-36-planning.md line 79 | Regression Suite | Full test suite execution | Full suite |
| AC-7 | New unit + integration tests for all sub-features | sprint-36-planning.md line 80 | Meta-validation | Verify new tests exist and pass | TC-036-001-003 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements
- ✅ No unjustified test types

**Coverage Gaps:** None identified

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Interactive Tests (REPL-based validation)**
- **Reason for omission:** Help text is accessed via CLI commands (`tq help config`), not REPL metacommands
- **What won't be validated:** REPL-specific help behavior (not applicable)
- **Risk assessment:** NONE - feature is CLI-only by design
- **Mitigation:** N/A - no REPL involvement
- **Revisit criteria:** Never - help text is CLI batch feature

**Performance/Benchmark Tests**
- **Reason for omission:** Help text generation has no performance requirements
- **What won't be validated:** Help text generation speed
- **Risk assessment:** LOW - static text generation is instantaneous
- **Mitigation:** Monitor in practice, add benchmarks if users report slowness
- **Revisit criteria:** If help command takes >10ms or users report delays

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** Inline with source code (`src/cli.rs::tests`, `src/config.rs::tests`)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 6 tests
- **Key scenarios to cover:**
  1. Help text generation includes project config section
  2. Help text includes 5-level precedence hierarchy (defaults → user → project → env → CLI)
  3. Profile display logic shows project config path when present
  4. Profile display omits project config line when absent
  5. Empty state message generated correctly
  6. Invalid TOML warning format: "Warning: Invalid project config at <path>: <error>"
- **Mocking strategy:** Mock config loading, test text generation functions

**Test Type: Integration Tests**
- **Location:** `tests/integration_help_text.rs` (new file)
- **Framework:** Built-in Rust integration test support with std::process::Command
- **Test count estimate:** 5 tests
- **Key scenarios to cover:**
  1. `tq help config` output includes project config section and precedence
  2. `tq profiles` with `.tq.toml` shows project config path
  3. `tq profiles` without `.tq.toml` omits project config line
  4. `tq profiles` with no profiles shows tip message
  5. `tq profiles` with invalid `.tq.toml` produces stderr warning (non-blocking)
- **Setup requirements:** Create temp directories with various `.tq.toml` states (valid, invalid, missing)

**Test Type: Manual Validation Checklist**
- **Checklist:**
  - [ ] `tq help config` text is clear and helpful (not confusing)
  - [ ] Precedence hierarchy explanation is understandable
  - [ ] Project config section uses consistent terminology
  - [ ] Tip message is friendly and actionable
  - [ ] Warning message is clear with actionable guidance
  - [ ] Help text follows existing style/tone
- **Estimated time:** 5-10 minutes

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: Help text content, message formatting, display logic
- Integration tests validate: CLI output correctness, file discovery, error handling
- Manual validation validates: Text clarity and UX quality
- Combined coverage: **Comprehensive**

**Gaps in combined coverage:**
- None identified - unit and integration tests cover all critical behaviors

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Performance testing deferred (LOW risk, no requirements)

---

### Feature 2: `/repeat` Command - Re-execute Last Query (7 ACs) - P1

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-36-planning.md` lines 90-107
- Secondary: `docs/specifications/repl.md` (Query Editing section)
- Context: `ReplState.last_sql` field already exists (Sprint 13)

**Requirements:**
1. "`/repeat` re-executes the last SQL statement" (AC-8, line 95)
2. "When no previous query exists, show clear message: 'No previous query to repeat'" (AC-9, line 96)
3. "`/repeat` works after any SQL statement (SELECT, INSERT, DDL, etc.)" (AC-10, line 97)
4. "Tab completion includes `/repeat` with description 'Re-execute last query'" (AC-11, line 98)
5. "`/help` output includes `/repeat` command" (AC-12, line 99)
6. "Short alias `\r` works (following psql convention)" (AC-13, line 100)
7. "Unit tests validate all behaviors" (AC-14, line 101)

**Feature Characteristics:**

**User Interaction Type:** ✅ Interactive PTY (REPL metacommand)
**Explanation:** `/repeat` is a REPL metacommand that re-executes SQL queries. This is interactive terminal behavior requiring PTY testing.

**Observable Behavior:**
- ✅ Visual output in terminal (query re-execution output, error messages)
- ✅ Database side effects (query re-executed against database)
- ✅ State management (reads `ReplState.last_sql`)

**External Dependencies:**
- ✅ Database connection (query execution requires live database)
- ✅ Terminal/PTY (REPL metacommand requires interactive session)

**Validation Challenges:**
- **REPL state**: Must verify `last_sql` is read correctly
- **Query re-execution**: Must test with various SQL types (SELECT, INSERT, DDL)
- **Empty state handling**: Must verify error message when no previous query
- **Tab completion**: Must validate completion suggestions include `/repeat`
- **Help text**: Must verify `/help` lists `/repeat` command

**Critical Behaviors to Validate:**
1. `/repeat` reads `ReplState.last_sql` and executes it
2. Empty state (no previous query) produces clear error message
3. Works after SELECT query
4. Works after INSERT/UPDATE/DELETE query
5. Works after DDL (CREATE, DROP, ALTER)
6. Short alias `\r` behaves identically to `/repeat`
7. Tab completion suggests `/repeat` with description
8. `/help` output includes `/repeat` command

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" + "REPL metacommand" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: REPL metacommands require PTY simulation to test user experience

IF "State management" + "Database side effects" checked:
  → Unit tests REQUIRED
  Reason: Command parsing and state logic must be tested in isolation

IF "Database connection" checked:
  → Integration tests REQUIRED
  Reason: Query re-execution must be validated with real database
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Metacommand parsing, `last_sql` state handling, error message generation
- **Approach:** Test `handle_metacommand()` function with mock `ReplState` and `DatabaseClient`
- **Rationale:** Pure logic components (parsing, state reading) must be validated independently
- **Gap if missing:** Logic errors in command handling not caught until integration
- **Necessity:** ✅ REQUIRED

**Test Type 2: Interactive Tests (expectrl)**
- **Validates:** REPL behavior, query re-execution output, tab completion, help text
- **Approach:** Spawn REPL, execute SQL, invoke `/repeat`, validate output matches original
- **Rationale:** End-to-end REPL user experience must be validated in real PTY
- **Gap if missing:** REPL integration bugs, output format issues, completion not working
- **Necessity:** ✅ REQUIRED

**Test Type 3: Integration Tests**
- **Validates:** Query re-execution with real database (various SQL types)
- **Approach:** Not needed - interactive tests with live database cover this
- **Rationale:** Interactive tests already require database and validate execution
- **Gap if missing:** None - interactive tests are comprehensive
- **Necessity:** ❌ NOT NEEDED (covered by interactive tests)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates command parsing and state handling | Logic bugs in metacommand handler | MUST IMPLEMENT |
| Interactive tests | ✅ REQUIRED | Validates REPL experience user sees | REPL bugs, completion missing | MUST IMPLEMENT |
| Integration tests | ❌ NOT NEEDED | Interactive tests already use live database | N/A | SKIP |
| Manual tests | ⚠️ RECOMMENDED | Human validates UX quality | Confusing error messages | DOCUMENT SCENARIOS |

**Summary:**
- ✅ REQUIRED test types: 2 (Unit, Interactive)
- ⚠️ RECOMMENDED test types: 1 (Manual)
- ❌ NOT NEEDED test types: 1 (Integration - redundant)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-8 | `/repeat` re-executes last SQL statement | sprint-36-planning.md line 95 | Unit + Interactive | Unit tests logic, interactive tests behavior | TC-036-004 |
| AC-9 | Clear message when no previous query | sprint-36-planning.md line 96 | Unit + Interactive | Unit tests message, interactive tests display | TC-036-004 |
| AC-10 | Works after any SQL type (SELECT, INSERT, DDL) | sprint-36-planning.md line 97 | Interactive | Must test with real database | TC-036-005 |
| AC-11 | Tab completion includes `/repeat` | sprint-36-planning.md line 98 | Interactive | Must validate PTY completion behavior | TC-036-006 |
| AC-12 | `/help` includes `/repeat` | sprint-36-planning.md line 99 | Interactive | Must validate help output | TC-036-006 |
| AC-13 | Short alias `\r` works | sprint-36-planning.md line 100 | Unit + Interactive | Unit tests alias parsing, interactive tests behavior | TC-036-004 |
| AC-14 | Unit tests validate all behaviors | sprint-36-planning.md line 101 | Meta-validation | Verify unit tests exist and pass | TC-036-004 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements
- ✅ No unjustified test types

**Coverage Gaps:** None identified

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Separate Integration Tests**
- **Reason for omission:** Interactive tests already use live database and validate query execution
- **What won't be validated:** Nothing - interactive tests are comprehensive
- **Risk assessment:** NONE - interactive tests cover all execution scenarios
- **Mitigation:** N/A - no gap exists
- **Revisit criteria:** Never - interactive tests are sufficient

**Performance/Benchmark Tests**
- **Reason for omission:** `/repeat` has no performance requirements beyond query execution itself
- **What won't be validated:** Command parsing speed
- **Risk assessment:** LOW - metacommand parsing is trivial overhead
- **Mitigation:** Monitor in practice
- **Revisit criteria:** If users report `/repeat` is slower than re-typing query

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/repl/metacommands.rs::tests`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 4 tests
- **Key scenarios to cover:**
  1. `/repeat` command parsed correctly
  2. `\r` alias parsed as `/repeat`
  3. `handle_metacommand("/repeat", ...)` with `last_sql = Some(...)` executes query
  4. `handle_metacommand("/repeat", ...)` with `last_sql = None` returns error message
- **Mocking strategy:** Mock `DatabaseClient` for query execution, use real `ReplState` struct

**Test Type: Interactive Tests (expectrl)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 6 tests
- **Key scenarios to cover:**
  1. Execute SELECT query, then `/repeat`, verify output matches
  2. Execute INSERT query, then `/repeat`, verify success
  3. Execute DDL query, then `/repeat`, verify success
  4. `/repeat` with no previous query shows error message
  5. Tab completion includes `/repeat` with description
  6. `/help` output includes `/repeat` command
- **Implementation notes:** Requires live database connection (marked with `#[ignore]`)

**Test Type: Manual Validation Checklist**
- **Checklist:**
  - [ ] `/repeat` feels responsive (no noticeable delay)
  - [ ] Error message is clear and helpful
  - [ ] Tab completion description is accurate
  - [ ] Help text follows existing format
  - [ ] Alias `\r` is intuitive for psql users
- **Estimated time:** 5 minutes

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: Command parsing, alias handling, state reading, error messages
- Interactive tests validate: REPL integration, query re-execution, completion, help text
- Manual validation validates: UX quality and responsiveness
- Combined coverage: **Comprehensive**

**Gaps in combined coverage:**
- None identified - unit and interactive tests cover all critical behaviors

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Performance testing deferred (LOW risk, no requirements)

---

### Feature 3: `/show indexes <table>` Command (10 ACs) - P1

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-36-planning.md` lines 109-129
- Secondary: `docs/specifications/repl.md` (Schema Inspection Commands)
- Context: Follows established `/describe` and `/list tables` patterns

**Requirements:**
1. "`/show indexes <table>` displays index information from DBC.IndicesV" (AC-15, line 114)
2. "Qualified name support: `/show indexes database.table`" (AC-16, line 115)
3. "Short alias `\di` works" (AC-17, line 116)
4. "Table output shows: IndexName, IndexType, ColumnName, ColumnPosition" (AC-18, line 117)
5. "Error handling for non-existent table with clear message" (AC-19, line 118)
6. "Error handling for permission denied with guidance" (AC-20, line 119)
7. "Tab completion includes `/show indexes` with description" (AC-21, line 120)
8. "`/help` output includes `/show indexes` command" (AC-22, line 121)
9. "Unit tests for SQL generation and argument parsing" (AC-23, line 122)
10. "Integration tests for CLI behavior" (AC-24, line 123)

**Feature Characteristics:**

**User Interaction Type:** ✅ Interactive PTY (REPL metacommand)
**Explanation:** `/show indexes` is a REPL metacommand that queries Teradata system catalog and displays results. This is interactive terminal behavior.

**Observable Behavior:**
- ✅ Visual output in terminal (table display with index information)
- ✅ Database side effects (queries DBC.IndicesV system catalog)
- ✅ Structured data output (table format with columns)

**External Dependencies:**
- ✅ Database connection (queries Teradata system catalog)
- ✅ Terminal/PTY (REPL metacommand requires interactive session)

**Validation Challenges:**
- **SQL generation**: Must validate correct query to DBC.IndicesV
- **Qualified names**: Must parse `database.table` syntax correctly
- **Error handling**: Must test non-existent table and permission denied scenarios
- **Output format**: Must verify table columns: IndexName, IndexType, ColumnName, ColumnPosition
- **Tab completion**: Must validate completion suggestions include `/show indexes`

**Critical Behaviors to Validate:**
1. `/show indexes <table>` generates query: `SELECT IndexName, IndexType, ColumnName, ColumnPosition FROM DBC.IndicesV WHERE DatabaseName = ? AND TableName = ?`
2. Qualified name parsing: `database.table` splits correctly
3. Unqualified name uses current database context
4. Short alias `\di` behaves identically to `/show indexes`
5. Output formatted as table with 4 columns
6. Non-existent table produces clear error message
7. Permission denied produces helpful guidance
8. Tab completion suggests `/show indexes` with description
9. `/help` output includes `/show indexes` command
10. Empty result set (table has no indexes) handled gracefully

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" + "REPL metacommand" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: REPL metacommands require PTY simulation to test user experience

IF "SQL generation" + "Argument parsing" checked:
  → Unit tests REQUIRED
  Reason: SQL query generation must be tested independently

IF "Database connection" + "System catalog query" checked:
  → Integration tests REQUIRED
  Reason: Real database queries validate SQL correctness and error handling
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** SQL generation, argument parsing, qualified name handling
- **Approach:** Test SQL query building functions with various table names
- **Rationale:** SQL generation logic must be validated without database
- **Gap if missing:** SQL syntax errors not caught until database execution
- **Necessity:** ✅ REQUIRED

**Test Type 2: Interactive Tests (expectrl)**
- **Validates:** REPL behavior, table output format, tab completion, help text
- **Approach:** Spawn REPL, execute `/show indexes`, validate table display
- **Rationale:** End-to-end REPL user experience must be validated in real PTY
- **Gap if missing:** REPL integration bugs, output format issues, completion not working
- **Necessity:** ✅ REQUIRED

**Test Type 3: Integration Tests**
- **Validates:** Database query execution, error handling, system catalog access
- **Approach:** Execute command with real database, test various scenarios (success, error, permission)
- **Rationale:** Real database validates SQL correctness and error handling paths
- **Gap if missing:** SQL errors, permission issues not caught
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates SQL generation and parsing | SQL syntax errors | MUST IMPLEMENT |
| Interactive tests | ✅ REQUIRED | Validates REPL experience user sees | REPL bugs, format wrong | MUST IMPLEMENT |
| Integration tests | ✅ REQUIRED | Validates database queries and errors | SQL errors, permission issues | MUST IMPLEMENT |
| Manual tests | ⚠️ RECOMMENDED | Human validates output clarity | Confusing output format | DOCUMENT SCENARIOS |

**Summary:**
- ✅ REQUIRED test types: 3 (Unit, Interactive, Integration)
- ⚠️ RECOMMENDED test types: 1 (Manual)
- ❌ NOT NEEDED test types: 0

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-15 | Displays index info from DBC.IndicesV | sprint-36-planning.md line 114 | Unit + Integration | Unit tests SQL, integration tests execution | TC-036-007 |
| AC-16 | Qualified name support `database.table` | sprint-36-planning.md line 115 | Unit + Integration | Unit tests parsing, integration tests behavior | TC-036-007 |
| AC-17 | Short alias `\di` works | sprint-36-planning.md line 116 | Unit + Interactive | Unit tests alias, interactive tests behavior | TC-036-007 |
| AC-18 | Table shows IndexName, IndexType, ColumnName, ColumnPosition | sprint-36-planning.md line 117 | Integration + Interactive | Must validate actual output | TC-036-008 |
| AC-19 | Error for non-existent table | sprint-36-planning.md line 118 | Integration + Interactive | Must test error path with real DB | TC-036-009 |
| AC-20 | Error for permission denied | sprint-36-planning.md line 119 | Integration + Interactive | Must test permission error path | TC-036-009 |
| AC-21 | Tab completion includes `/show indexes` | sprint-36-planning.md line 120 | Interactive | Must validate PTY completion | TC-036-010 |
| AC-22 | `/help` includes `/show indexes` | sprint-36-planning.md line 121 | Interactive | Must validate help output | TC-036-010 |
| AC-23 | Unit tests for SQL generation and parsing | sprint-36-planning.md line 122 | Meta-validation | Verify unit tests exist and pass | TC-036-007 |
| AC-24 | Integration tests for CLI behavior | sprint-36-planning.md line 123 | Meta-validation | Verify integration tests exist and pass | TC-036-008-009 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements
- ✅ No unjustified test types

**Coverage Gaps:** None identified

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Cross-database Tests (Multiple Teradata Versions)**
- **Reason for omission:** DBC.IndicesV is standard Teradata system catalog view
- **What won't be validated:** Compatibility across Teradata versions
- **Risk assessment:** LOW - system catalog views are stable across versions
- **Mitigation:** Document minimum Teradata version if issues arise
- **Revisit criteria:** If users report view unavailable on older Teradata versions

**Performance/Benchmark Tests**
- **Reason for omission:** System catalog queries are Teradata-optimized, no performance requirements
- **What won't be validated:** Query execution time
- **Risk assessment:** LOW - DBC queries are fast, no user-reported issues expected
- **Mitigation:** Monitor in practice
- **Revisit criteria:** If users report slow index queries

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/repl/metacommands.rs::tests`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 6 tests
- **Key scenarios to cover:**
  1. SQL generation for unqualified table name (uses current database)
  2. SQL generation for qualified name `database.table`
  3. Argument parsing: `/show indexes mytable`
  4. Argument parsing: `/show indexes mydb.mytable`
  5. Alias parsing: `\di` treated as `/show indexes`
  6. Error for missing argument (no table name)
- **Mocking strategy:** Test SQL string generation functions independently

**Test Type: Interactive Tests (expectrl)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 5 tests
- **Key scenarios to cover:**
  1. `/show indexes <table>` displays table output with 4 columns
  2. Alias `\di` works identically
  3. Tab completion includes `/show indexes` with description
  4. `/help` output includes `/show indexes` command
  5. Error message for non-existent table is clear
- **Implementation notes:** Requires live database connection (marked with `#[ignore]`)

**Test Type: Integration Tests**
- **Location:** `tests/integration_show_indexes.rs` (new file)
- **Framework:** Built-in Rust integration test support
- **Test count estimate:** 4 tests
- **Key scenarios to cover:**
  1. Query DBC.IndicesV succeeds and returns expected columns
  2. Qualified name `database.table` works correctly
  3. Non-existent table produces clear error message
  4. Permission denied error handled gracefully
- **Setup requirements:** Live database with test tables (one with indexes, one without)

**Test Type: Manual Validation Checklist**
- **Checklist:**
  - [ ] Table output is readable and well-formatted
  - [ ] Column headers are clear and descriptive
  - [ ] Index types are human-readable (not internal codes)
  - [ ] Error messages are actionable (suggest next steps)
  - [ ] Empty result (no indexes) displays helpful message
  - [ ] Large index lists (>20 indexes) display correctly
- **Estimated time:** 10 minutes

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: SQL generation, argument parsing, qualified name handling
- Interactive tests validate: REPL integration, table display, completion, help text
- Integration tests validate: Database queries, error handling, permission issues
- Manual validation validates: Output clarity and UX quality
- Combined coverage: **Comprehensive**

**Gaps in combined coverage:**
- None identified - all three test types provide comprehensive coverage

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Cross-version compatibility deferred (LOW risk, standard system catalog)
- Performance testing deferred (LOW risk, no requirements)

---

## Strategy Summary

**Total Features Analyzed:** 3

**Test Types Required:**
- Unit tests: ✅ [All 3 features] (required)
- Integration tests: ✅ [Feature 1, Feature 3] (required)
- Interactive tests: ✅ [Feature 2, Feature 3] (required)
- Manual validation: ⚠️ [All 3 features] (recommended)

**Estimated Test Count:**
- Unit: 6 (Feature 1) + 4 (Feature 2) + 6 (Feature 3) = **16 tests**
- Integration: 5 (Feature 1) + 0 (Feature 2) + 4 (Feature 3) = **9 tests**
- Interactive: 0 (Feature 1) + 6 (Feature 2) + 5 (Feature 3) = **11 tests**
- **Total new automated tests: 36**
- **Baseline: ~650 tests → Target: ~686 tests**

**Test Cases to Document:**
- TC-036-001: Config help text content (unit + integration)
- TC-036-002: Profile command output with project config (integration)
- TC-036-003: Invalid project config warning (unit + integration)
- TC-036-004: `/repeat` command basic behavior (unit + interactive)
- TC-036-005: `/repeat` with various SQL types (interactive)
- TC-036-006: `/repeat` completion and help text (interactive)
- TC-036-007: `/show indexes` SQL generation and parsing (unit)
- TC-036-008: `/show indexes` output format (integration + interactive)
- TC-036-009: `/show indexes` error handling (integration + interactive)
- TC-036-010: `/show indexes` completion and help text (interactive)

**Risk Assessment:**
- **HIGH risk gaps:** None
- **MEDIUM risk gaps:** None
- **LOW risk gaps:**
  - Performance testing deferred (all 3 features - no requirements)
  - Cross-version compatibility deferred (Feature 3 - standard system catalog)

**Dependencies Required:**
- Live database: ✅ YES (Feature 2 interactive tests, Feature 3 all tests)
- Network access: ❌ NO
- Specific OS: ❌ NO
- File system: ✅ YES (Feature 1 - tempfile for config testing)

**Sprint 36 Specific Notes:**
- **Feature 1 (Config Polish):** No database required for unit/integration tests
- **Feature 2 (`/repeat`):** Database required for interactive tests only
- **Feature 3 (`/show indexes`):** Database required for integration and interactive tests
- **Test complexity:** LOW-MEDIUM (all features follow established patterns)
- **Test infrastructure:** MATURE (no new tools needed)

---

## Tool Requirements Assessment

### Current Testing Tools

**Available:**
- ✅ Unit test framework (built-in Rust)
- ✅ Integration test framework (std::process::Command)
- ✅ Interactive test framework (expectrl + PTY)
- ✅ Tempfile crate for test fixtures
- ✅ Environment variable management (dotenvy)

**Needed for Sprint 36:**
- ✅ tempfile - Already available (for Feature 1 config testing)
- ✅ std::fs - Built-in (for file operations)
- ✅ std::process::Command - Built-in (for CLI testing)
- ✅ expectrl - Already available (for REPL testing)

**New Tools Required:** NONE

All necessary testing infrastructure already exists. Sprint 36 tests will use:
1. **tempfile::TempDir** - Create test directory structures (Feature 1)
2. **std::process::Command** - Execute CLI commands (Feature 1)
3. **expectrl** - Spawn REPL and simulate user input (Feature 2, Feature 3)
4. **Mock DatabaseClient** - Unit test database interactions (Feature 2, Feature 3)

### Tool Assessment Summary

**Can current tools test all Sprint 36 features?** ✅ YES

**New tools needed for any feature?** ❌ NO
- Feature 1: Existing integration test patterns
- Feature 2: Existing unit + interactive test patterns
- Feature 3: Existing unit + integration + interactive test patterns

**Recommendation:** Proceed with existing tools. No new tool development required.

---

## Test Execution Strategy

### Phase 1: Unit Tests (1-2 hours)
**Priority:** High (validates core logic)

**Sequence:**
1. Feature 1 unit tests (help text generation, error formatting) - 30 min
2. Feature 2 unit tests (metacommand parsing, state handling) - 30 min
3. Feature 3 unit tests (SQL generation, argument parsing) - 30 min
4. Run: `cargo test --lib` - verify all pass

**Expected results:** 16 new unit tests passing

### Phase 2: Integration Tests (1-2 hours)
**Priority:** High (validates CLI behavior)

**Sequence:**
1. Feature 1 integration tests (help text CLI, profiles CLI) - 45 min
2. Feature 3 integration tests (database queries, error handling) - 45 min
3. Run: `cargo test --test integration_*` - verify all pass

**Expected results:** 9 new integration tests passing

### Phase 3: Interactive Tests (1-2 hours)
**Priority:** High (validates REPL user experience)

**Prerequisites:** Live database connection required

**Sequence:**
1. Feature 2 interactive tests (`/repeat` behavior, completion) - 45 min
2. Feature 3 interactive tests (`/show indexes` display, completion) - 45 min
3. Run: `cargo test --test interactive_tests -- --ignored --test-threads=1`

**Expected results:** 11 new interactive tests passing

### Phase 4: Full Regression (15-30 minutes)
**Priority:** Critical (ensure zero regressions)

**Sequence:**
```bash
# Run all unit tests
cargo test --lib

# Run all integration tests
cargo test --test integration_*

# Run all interactive tests
cargo test --test interactive_tests -- --ignored --test-threads=1

# Expected: ~686 tests passing (650 baseline + 36 new)
```

### Phase 5: Manual Validation (30 minutes)
**Priority:** Medium (validates UX quality)

**Sequence:**
1. Feature 1: Review help text clarity and formatting - 10 min
2. Feature 2: Test `/repeat` responsiveness and error messages - 10 min
3. Feature 3: Verify index display readability and error guidance - 10 min

### Phase 6: Test Report Generation
**Priority:** Critical (documents results)

Create `tests/results/sprint-36/REPORT.md` with:
- Test execution proof (cargo output)
- Pass rate summary (X/Y tests passed)
- Coverage assessment (all ACs validated)
- Verdict: APPROVED / REJECTED / BLOCKED

---

## Coverage Sufficiency Assessment

### Overall Coverage Analysis

**Feature 1 (Config Help Text):**
- Unit tests validate: Text generation, error formatting, display logic
- Integration tests validate: CLI output, file discovery, stderr warnings
- Coverage: **Comprehensive** (11 tests cover all critical paths)

**Feature 2 (`/repeat` Command):**
- Unit tests validate: Command parsing, state handling, error messages
- Interactive tests validate: REPL integration, query re-execution, completion
- Coverage: **Comprehensive** (10 tests cover all critical paths)

**Feature 3 (`/show indexes` Command):**
- Unit tests validate: SQL generation, argument parsing, qualified names
- Integration tests validate: Database queries, error handling, permissions
- Interactive tests validate: REPL display, completion, help text
- Coverage: **Comprehensive** (15 tests cover all critical paths)

**Combined Sprint Coverage:**
- All 24 acceptance criteria have automated tests
- All critical behaviors validated across 3 test types
- No high-risk gaps identified
- **Overall: Comprehensive coverage for Sprint 36 deliverables**

---

## Success Criteria

Sprint 36 test strategy is successful if:

1. **Test Coverage Complete:**
   - ✅ All 3 features have test strategy defined
   - ✅ All 24 acceptance criteria mapped to tests
   - ✅ Test types derived from feature characteristics

2. **Test Implementation Achievable:**
   - ✅ 36 automated tests (clear scope, well-defined)
   - ✅ No new tools required (existing infrastructure sufficient)
   - ✅ Test setup straightforward (tempfile + expectrl)

3. **Quality Assurance Robust:**
   - ✅ Feature 1 has unit + integration coverage
   - ✅ Feature 2 has unit + interactive coverage
   - ✅ Feature 3 has unit + integration + interactive coverage

4. **Gaps Identified and Accepted:**
   - ✅ Performance tests deferred (LOW risk, no requirements)
   - ✅ Cross-version tests deferred (LOW risk, standard catalog)
   - ✅ All gaps have risk assessment

5. **Execution Plan Clear:**
   - ✅ Test phases defined with dependencies
   - ✅ Priority order established (unit → integration → interactive)
   - ✅ Success criteria for each phase

---

## Strategy Validation Checklist

**Before submitting for review:**

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

**Strategy Status:** READY FOR REVIEW

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-02-13
**Review Status:** DRAFT
**Sprint:** 36 - Help Text Update + REPL Enhancements
**Submitted for Review:** 2026-02-13

**Reviewer:** tq-project-manager (pending)
**Review Status:** PENDING
**Review Date:** (pending)
**Review Comments:** (pending)

**Approval means:**
- ✅ Test strategy derived from specifications (not assumptions)
- ✅ All required test types identified with clear rationale
- ✅ Coverage gaps explicitly identified and assessed
- ✅ Implementation plan is detailed and achievable
- ✅ Ready to proceed with test implementation
