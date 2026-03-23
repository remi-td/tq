# Sprint 48 Test Strategy: Query Layer Consolidation & Spec Alignment

**Created:** 2026-03-23
**Author:** quality-validator
**Sprint:** Sprint 48
**Features:**
1. Extract Shared Query Layer (query_helpers.rs)
2. Fix JSON API Types (nullable boolean, null default, integer rows/size, database key)
3. Fix Bugs (summarize_error UTF-8, TABLE→OBJECT, list type labels, error prefix)
4. Enrich list views & Edge Cases (Owner column, edge case messages)
5. Missing Unit Tests (TC-047-001 DDL tests, writer-injection, column_type_case_sql branches)

---

## Feature-by-Feature Test Strategy

---

### Feature 1: Extract Shared Query Layer

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-48-planning.md` §Objectives §Feature 1
- Requirements:
  1. `query_indexes()` exists once, used by inspect.rs, describe.rs, show_indexes.rs (AC-1)
  2. `query_columns()` exists once, used by inspect.rs, describe.rs (AC-2)
  3. `resolve_database()` exists once, used by inspect.rs, describe.rs (AC-3)
  4. `format_size()` exists once with precision parameter (AC-4)
  5. All existing tests pass after consolidation (AC-5)
  6. Shared IndexGroup and ColumnInfo types defined once (AC-6)

**Feature Characteristics:**

**User Interaction Type:** Pure Logic — refactoring internal module structure. The user-visible behavior is unchanged; this is a structural correctness guarantee.

**Observable Behavior:**
- Structured data output (JSON, CSV, XML) — indirectly: the refactored functions still produce the same output
- No new user-visible behavior

**External Dependencies:**
- None for structural tests (grep/ast checks)
- Database connection for existing integration tests that call these functions indirectly

**Validation Challenges:**
- Can only inspect source structure (grep), not runtime behavior, without a live DB
- Ensuring no duplicate definitions requires static analysis

**Critical Behaviors to Validate:**
1. Each function name appears exactly once as a `fn` definition across all commands
2. All existing `cargo test --lib` tests continue to pass (no regression)

#### 2. Test Strategy Derivation

**Decision Tree Results:**
- "Pure Logic" checked → Unit tests / structural grep REQUIRED
- No new user-visible behavior → Integration tests NOT required for this feature alone
- All existing tests must pass → Regression test run REQUIRED

**Derived Test Types:**

**Test Type 1: Structural Grep Tests**
- **Validates:** AC-1 through AC-4, AC-6 — each function defined exactly once
- **Approach:** Shell grep over `src/commands/*.rs` counting function definition occurrences
- **Rationale:** The only way to verify "exists once" without a live DB
- **Gap if missing:** Duplicate definitions would not be caught until integration failure
- **Necessity:** REQUIRED

**Test Type 2: Regression — cargo test --lib**
- **Validates:** AC-5 — all existing tests still pass after refactoring
- **Approach:** Run `cargo test --lib` and verify zero failures
- **Rationale:** Refactoring must not break any existing behavior
- **Gap if missing:** Silent logic regressions in existing tested code paths
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Structural grep | REQUIRED | Verifies "defined once" constraint | Duplicate definitions undetected | MUST IMPLEMENT |
| Regression (cargo test --lib) | REQUIRED | Verifies no behavioral regression | Silent regressions | MUST IMPLEMENT |
| Integration tests (live DB) | NOT NEEDED for this feature | Behavior unchanged; existing integration tests cover it | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirement | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| F1-AC-1 | query_indexes() exists once | sprint-48-planning.md §F1 | Structural grep | TC-048-001 §A |
| F1-AC-2 | query_columns() exists once | sprint-48-planning.md §F1 | Structural grep | TC-048-001 §A |
| F1-AC-3 | resolve_database() exists once | sprint-48-planning.md §F1 | Structural grep | TC-048-001 §A |
| F1-AC-4 | format_size() exists once | sprint-48-planning.md §F1 | Structural grep | TC-048-001 §A |
| F1-AC-5 | All existing tests pass | sprint-48-planning.md §F1 | Regression run | TC-048-001 §B |
| F1-AC-6 | Shared types defined once | sprint-48-planning.md §F1 | Structural grep | TC-048-001 §A |

#### 5. Gap Analysis

**Integration tests with live DB** — Not needed for this feature. The structural check (grep) and regression run (all lib tests pass) are sufficient to validate the refactoring. Risk: LOW.

#### 6. Test Implementation Plan

**Test Type: Structural Grep Tests**
- **Location:** Test case document TC-048-001 — executed via bash grep commands
- **Framework:** Shell grep + word count
- **Test count estimate:** 6 tests (one per shared symbol)
- **Key scenarios:**
  1. `fn query_indexes` appears in exactly one `*.rs` file definition
  2. `fn query_columns` appears in exactly one `*.rs` file definition
  3. `fn resolve_database` appears in exactly one `*.rs` file definition
  4. `fn format_size` appears in exactly one `*.rs` file definition
  5. `struct IndexGroup` defined once
  6. `struct ColumnInfo` or `struct ColumnRow` defined once
- **Mocking strategy:** None — direct source file inspection

**Test Type: Regression**
- **Location:** `cargo test --lib` run
- **Framework:** Rust built-in test framework
- **Test count estimate:** All ~130+ existing lib tests
- **Key scenarios:**
  1. Full `cargo test --lib` exits 0

#### 7. Coverage Sufficiency Assessment

- Structural grep validates: de-duplication constraint (AC-1..4, AC-6)
- Regression validates: no behavioral regression (AC-5)
- Combined coverage: adequate

**Acceptance criteria:**
- All 6 structural grep tests pass
- `cargo test --lib` exits 0

---

### Feature 2: Fix JSON API Types

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-48-planning.md` §Feature 2
- Requirements:
  1. describe JSON: nullable as boolean (true/false), not string "YES"/"NO" (AC-1)
  2. describe JSON: default as null (not "-") when absent (AC-2)
  3. list tables JSON: estimated_rows as integer, size_bytes as integer (AC-3)
  4. list databases JSON: key "database" not "name" (AC-4)
  5. Unit tests for all JSON type changes (AC-5)

**Feature Characteristics:**

**User Interaction Type:** CLI Batch — machine-readable JSON output from `tq describe`, `tq list tables`, `tq list databases`

**Observable Behavior:**
- Structured data output (JSON)

**External Dependencies:**
- Database connection required for end-to-end JSON output
- Unit tests use writer-injection (Vec<u8> writer) — no DB needed

**Validation Challenges:**
- Rendering functions require a live DB unless writer-injection is used with mock data
- JSON type correctness (boolean vs string) requires parsing the output

**Critical Behaviors to Validate:**
1. `nullable` field is a JSON boolean literal (`true` or `false`), not a string
2. `default` field is JSON `null` when absent, not the string `"-"`
3. `rows_est` and `size` fields are JSON integers, not quoted strings
4. Database object key is `"database"` not `"name"`

#### 2. Test Strategy Derivation

**Decision Tree Results:**
- "CLI Batch" checked → Unit tests with writer-injection REQUIRED
- "Structured data output" checked → JSON parsing validation REQUIRED
- "Database connection" checked → Integration tests RECOMMENDED but writer-injection covers unit path

**Derived Test Types:**

**Test Type 1: Unit Tests with Writer-Injection**
- **Validates:** AC-1, AC-2, AC-3, AC-4 — JSON format correctness using in-memory output
- **Approach:** Build mock ColumnRow/TableEntry/DatabaseEntry structs, call rendering functions with `Vec<u8>` writer, parse JSON output
- **Rationale:** No DB needed; directly validates the output format
- **Gap if missing:** JSON type bugs would go undetected without live DB
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (live DB)**
- **Validates:** End-to-end JSON output with real data
- **Approach:** Run `tq describe dbc.tables --format json`, parse with serde_json, check types
- **Rationale:** Writer-injection tests mock data; live DB tests the full code path
- **Gap if missing:** SQL-level issues (e.g., type coercion) not caught by unit tests
- **Necessity:** RECOMMENDED (DB required, `#[ignore]`)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (writer-injection) | REQUIRED | Validates JSON type logic without DB | JSON type bugs undetected without DB | MUST IMPLEMENT |
| Integration tests | RECOMMENDED | Validates full code path with real data | SQL-level coercion issues | IMPLEMENT (ignored) |
| Benchmark tests | NOT NEEDED | No performance requirement | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| F2-AC-1 | nullable as boolean | sprint-48-planning.md §F2 | Unit (writer-injection) | TC-048-002 §A |
| F2-AC-2 | default as null when absent | sprint-48-planning.md §F2 | Unit (writer-injection) | TC-048-002 §A |
| F2-AC-3 | rows/size as integers | sprint-48-planning.md §F2 | Unit (writer-injection) | TC-048-002 §B |
| F2-AC-4 | key "database" not "name" | sprint-48-planning.md §F2 | Unit (writer-injection) | TC-048-002 §C |
| F2-AC-5 | Unit tests for all changes | sprint-48-planning.md §F2 | Unit | TC-048-002 all |

#### 5. Gap Analysis

Integration tests require live DB. Writer-injection unit tests are the primary validation mechanism for JSON type correctness.

---

### Feature 3: Fix Bugs

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-48-planning.md` §Feature 3
- Requirements:
  1. Fix summarize_error UTF-8 byte-boundary bug (AC-1)
  2. Fix show-indexes `<TABLE>` → `<OBJECT>` in cli.rs (AC-2)
  3. Fix list databases type: "System"/"User" instead of "Database"/"User" (AC-3)
  4. Fix list.rs unknown subcommand missing `Error:` prefix (AC-4)
  5. Rename DescribeArgs.table field to .object (AC-5)

**Feature Characteristics:**

**User Interaction Type:** Pure Logic (AC-1, AC-5) + CLI Batch (AC-2, AC-3, AC-4) — bug fixes in function logic and CLI help text

**Observable Behavior:**
- Visual output in terminal (AC-2 — help text label)
- Structured data output (AC-3 — list databases JSON/table type column)

**External Dependencies:**
- None for unit tests
- Database connection for AC-3 end-to-end type label validation

**Validation Challenges:**
- UTF-8 byte-boundary bugs require multi-byte Unicode test strings
- CLI help text requires spawning the binary
- Type label requires live DB or mocked data

**Critical Behaviors to Validate:**
1. `summarize_error` truncates at a valid UTF-8 character boundary, not mid-codepoint
2. `show-indexes` help text uses `<OBJECT>` not `<TABLE>`
3. `list databases` type column shows "System" for system databases (not "Database")
4. Unknown list subcommand error output starts with "Error:"

#### 2. Test Strategy Derivation

**Test Type 1: Unit Tests — summarize_error UTF-8**
- **Validates:** AC-1 — truncation at valid UTF-8 boundary
- **Approach:** Pass strings containing multi-byte UTF-8 characters (e.g., "é", "中", emoji) longer than 80 chars; verify result is valid UTF-8 and ends with "..."
- **Necessity:** REQUIRED

**Test Type 2: Structural Grep — TABLE→OBJECT in cli.rs**
- **Validates:** AC-2 — `<TABLE>` is gone, `<OBJECT>` is present
- **Approach:** Grep `src/cli.rs` for `value_name = "TABLE"` in show-indexes context; assert 0 matches
- **Necessity:** REQUIRED

**Test Type 3: Unit Tests — list databases type labels**
- **Validates:** AC-3 — "System"/"User" labels
- **Approach:** Writer-injection with mock DatabaseEntry structs; verify JSON output has correct type string
- **Necessity:** REQUIRED

**Test Type 4: Unit Tests — unknown subcommand error prefix**
- **Validates:** AC-4 — "Error:" prefix in error message
- **Approach:** Writer-injection call to list subcommand dispatch with "bogus" subcommand; verify output contains "Error:"
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (UTF-8) | REQUIRED | Only way to test byte-boundary without DB | Silent data corruption at Unicode boundaries | MUST IMPLEMENT |
| Structural grep (TABLE→OBJECT) | REQUIRED | Validates AC-2 without binary spawn | Wrong help text undetected | MUST IMPLEMENT |
| Unit tests (type labels) | REQUIRED | Validates AC-3 without live DB | Wrong type labels in JSON undetected | MUST IMPLEMENT |
| Unit tests (error prefix) | REQUIRED | Validates AC-4 without binary spawn | Missing "Error:" prefix undetected | MUST IMPLEMENT |
| Integration tests | NOT NEEDED (for this feature) | Bugs are in pure logic / formatting | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| F3-AC-1 | summarize_error UTF-8 safe | sprint-48-planning.md §F3 | Unit | TC-048-003 §A |
| F3-AC-2 | TABLE→OBJECT in cli.rs | sprint-48-planning.md §F3 | Structural grep | TC-048-003 §B |
| F3-AC-3 | "System"/"User" type labels | sprint-48-planning.md §F3 | Unit (writer-injection) | TC-048-003 §C |
| F3-AC-4 | Error: prefix on unknown subcommand | sprint-48-planning.md §F3 | Unit (writer-injection) | TC-048-003 §D |
| F3-AC-5 | DescribeArgs.table renamed to .object | sprint-48-planning.md §F3 | Structural grep | TC-048-003 §E |

---

### Feature 4: Enrich list views & Edge Cases

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-48-planning.md` §Feature 4
- Requirements:
  1. `tq list views` shows Owner column (AC-1)
  2. "No indexes defined." message for tables without indexes in describe (AC-2)
  3. "No Primary Index (NoPI)" for NoPI tables in show-indexes (AC-3)
  4. "No secondary indexes." when none exist in show-indexes (AC-4)
  5. Add Rows (Est.) to describe object header for tables (AC-5)

**Feature Characteristics:**

**User Interaction Type:** CLI Batch — `tq list views`, `tq describe`, `tq show-indexes`

**Observable Behavior:**
- Visual output in terminal (formatted table output with new columns and messages)

**External Dependencies:**
- Database connection required for actual command output
- Unit tests can validate message strings via writer-injection

**Critical Behaviors to Validate:**
1. The exact strings "No indexes defined.", "No Primary Index (NoPI)", "No secondary indexes."
2. Owner column presence in `list views` table output

#### 2. Test Strategy Derivation

**Test Type 1: Unit Tests — Edge Case Message Strings**
- **Validates:** AC-2, AC-3, AC-4 — exact message strings for edge cases
- **Approach:** Unit tests that call rendering functions with empty/NoPI index groups via writer-injection; assert output contains exact strings
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (live DB)**
- **Validates:** AC-1, AC-5 — live command output contains Owner column, Rows header
- **Necessity:** RECOMMENDED (`#[ignore]`)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (edge case strings) | REQUIRED | Validates exact message text without DB | Wrong messages in production | MUST IMPLEMENT |
| Integration tests | RECOMMENDED | Validates Owner column and Rows header in real output | Missing columns only caught with live DB | IMPLEMENT (ignored) |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| F4-AC-2 | "No indexes defined." in describe | sprint-48-planning.md §F4 | Unit | TC-048-004 §A |
| F4-AC-3 | "No Primary Index (NoPI)" in show-indexes | sprint-48-planning.md §F4 | Unit | TC-048-004 §B |
| F4-AC-4 | "No secondary indexes." in show-indexes | sprint-48-planning.md §F4 | Unit | TC-048-004 §C |
| F4-AC-1 | Owner column in list views | sprint-48-planning.md §F4 | Integration (ignored) | TC-048-004 §D |
| F4-AC-5 | Rows (Est.) in describe header | sprint-48-planning.md §F4 | Integration (ignored) | TC-048-004 §D |

---

### Feature 5: Missing Unit Tests

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-48-planning.md` §Feature 5
- Secondary: `tests/cases/TC-047-001.md` (DDL tests specified but not implemented)
- Requirements:
  1. 6 DDL unit tests from TC-047-001 implemented in inspect.rs (AC-1)
  2. Writer-injection tests for describe_table rendering (AC-2)
  3. Writer-injection tests for show_indexes_table rendering (AC-3)
  4. Writer-injection tests for list_databases rendering (AC-4)
  5. column_type_case_sql test verifies all 21 WHEN branches (AC-5)

**Feature Characteristics:**

**User Interaction Type:** Pure Logic — unit test implementation that validates internal functions

**Observable Behavior:**
- Test pass/fail output from `cargo test --lib`

**External Dependencies:**
- None — all unit tests, no DB required

**Critical Behaviors to Validate:**
1. The 6 DDL tests from TC-047-001 all pass (test the logic documented there)
2. describe_table renders correct sections (Object, Columns, Indexes) to a Vec<u8> writer
3. show_indexes_table renders Primary/Secondary sections correctly
4. list_databases renders Name/Owner/Type columns correctly
5. column_type_case_sql SQL string contains all 21 WHEN branches

#### 2. Test Strategy Derivation

All tests in this feature are pure unit tests with no external dependencies.

**Test Type 1: Unit Tests — DDL concatenation logic**
- **Validates:** AC-1 — the 6 TC-047-001 DDL tests
- **Approach:** Implement the 6 tests as specified in TC-047-001.md into inspect.rs
- **Necessity:** REQUIRED

**Test Type 2: Unit Tests — Writer-injection rendering**
- **Validates:** AC-2, AC-3, AC-4 — describe_table, show_indexes_table, list_databases
- **Approach:** Create mock structs (ObjectHeader, ColumnRow, IndexGroup, DatabaseEntry) and call rendering functions with Vec<u8> writer; assert output contains expected sections and values
- **Necessity:** REQUIRED

**Test Type 3: Unit Tests — column_type_case_sql completeness**
- **Validates:** AC-5 — all 21 WHEN branches present in the SQL string
- **Approach:** Count WHEN occurrences in column_type_case_sql() return value; verify each type code is present
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (DDL) | REQUIRED | Specified in TC-047-001, still outstanding | Bug #36 fixes not tested | MUST IMPLEMENT |
| Unit tests (writer-injection) | REQUIRED | Validates rendering without DB | Rendering bugs undetected | MUST IMPLEMENT |
| Unit tests (SQL completeness) | REQUIRED | Validates all type codes present | Missing type codes undetected | MUST IMPLEMENT |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| F5-AC-1 | 6 DDL tests from TC-047-001 | sprint-48-planning.md §F5, TC-047-001.md | Unit | TC-048-005 §A |
| F5-AC-2 | describe_table writer-injection | sprint-48-planning.md §F5 | Unit | TC-048-005 §B |
| F5-AC-3 | show_indexes_table writer-injection | sprint-48-planning.md §F5 | Unit | TC-048-005 §C |
| F5-AC-4 | list_databases writer-injection | sprint-48-planning.md §F5 | Unit | TC-048-005 §D |
| F5-AC-5 | column_type_case_sql 21 branches | sprint-48-planning.md §F5 | Unit | TC-048-005 §E |

---

## Strategy Summary

**Total Features Analyzed:** 5

**Test Types Required:**
- Unit tests: REQUIRED for all 5 features
- Structural grep: REQUIRED for Feature 1 (consolidation), Feature 3 (TABLE→OBJECT)
- Regression run (cargo test --lib): REQUIRED for Feature 1
- Integration tests (ignored, live DB): RECOMMENDED for Features 2, 4

**Estimated Test Count:**
- Structural grep: 8 checks (6 for F1, 2 for F3)
- Unit tests: ~50 new tests across 5 test cases
- Integration tests (ignored): ~6 tests
- Regression: All existing ~130 lib tests must still pass
- Total new tests: ~58 tests

**Risk Assessment:**
- HIGH risk gaps: none
- MEDIUM risk gaps: Integration tests for F2 (JSON types end-to-end) and F4 (Owner column, Rows header) require live DB — blocked without credentials
- LOW risk gaps: Performance/benchmark not tested (no requirements)

**Dependencies Required:**
- Live database: Only for integration tests (`#[ignore]`)
- Network access: No
- Specific OS: No
- Other: `cargo test --lib` must compile and run (standard Rust toolchain)

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
**Created Date:** 2026-03-23
**Review Status:** DRAFT
**Submitted for Review:** 2026-03-23

**Reviewer:** tq-project-manager
**Review Status:** PENDING
**Review Date:** —
**Review Comments:** —
