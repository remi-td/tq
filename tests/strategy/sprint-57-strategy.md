# Sprint 57 Test Strategy: Search Quality & View Search

**Created:** 2026-04-06
**Author:** quality-validator
**Sprint:** Sprint 57
**Features:**
1. Replace hand-rolled JSON in search.rs with serde_json (P0)
2. Promote hard-coded 100000 to named constant MAX_SEARCH_FETCH (P0)
3. Add ORDER BY stability warning to pagination docs (P1)
4. Add `tq search views <keyword>` subcommand and REPL `/search views` (P1)

---

## Feature-by-Feature Test Strategy

---

### Feature 1: Serde JSON Refactoring

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-57-planning.md` §Feature 1 (P0)
- Requirements:
  1. Both `render_table_search_json_with_pagination` and `render_column_search_json_with_pagination` use serde_json instead of manual `write!()` calls (AC-1)
  2. JSON output is byte-identical or semantically equivalent to current output (AC-2)
  3. All existing search JSON tests pass (AC-3)
  4. No new clippy warnings (AC-4)

**Feature Characteristics:**

**User Interaction Type:** Pure Logic — internal rendering refactor, no change in user-observable behavior.

**Explanation:** The refactoring replaces implementation internals while preserving identical JSON output. There is no change to CLI arguments, command names, or output semantics. The observable contract (JSON envelope structure, field names, field types) is frozen.

**Observable Behavior:**
- [x] Structured data output (JSON) — same output produced via different internal path

**External Dependencies:**
- [ ] None (pure logic, writer-injection unit tests, no DB required)

**Validation Challenges:**
- Must prove semantic equivalence of JSON output, not just code structure.
- Edge cases (null numeric values rendered as JSON `null` not `"null"`, empty arrays, special characters) must be exercised explicitly because serde_json handles serialization differently from manual string building.
- Special characters in names (quotes, backslashes, Unicode) were previously handled by `json_escape()`; serde_json handles these natively but the mapping must be verified.

**Critical Behaviors to Validate:**
1. "JSON output is semantically equivalent to current output" — field names, types, envelope keys unchanged (sprint-57-planning.md §F1 AC-2)
2. "null numeric fields render as JSON null, not string 'null'" — `estimated_rows: null`, `size_bytes: null` (existing test `test_render_table_search_json_null_values`)
3. "Special characters in database/table names are correctly escaped" — quotes, backslashes, pipe characters
4. "Pagination sub-object present only when pagination arg is Some" — (existing test `test_table_search_json_no_pagination`)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

- "CLI Batch" checked: render functions are called from batch dispatch → unit tests on render functions are sufficient; no end-to-end execution required for the refactor itself.
- "Database connection" NOT checked for unit tests: render functions accept `&[T]` slices — no DB needed.
- "Structured data output" checked: must verify JSON field names, types, and envelope structure.

**Derived Test Types:**

**Test Type 1: Unit Tests (existing + new edge cases)**
- **Validates:** Semantic equivalence of JSON output after refactoring; edge cases not covered by existing tests.
- **Approach:** Call render functions directly with crafted `TableSearchResult`/`ColumnSearchResult` slices; assert on output string using `serde_json::from_str` (parse and compare) rather than substring checks for new tests.
- **Rationale:** Render functions are pure Write-injected functions testable without DB.
- **Gap if missing:** Regression in JSON shape would go undetected until runtime.
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (DB, #[ignore])**
- **Validates:** End-to-end `tq search tables --format json` continues to produce valid JSON after refactor.
- **Approach:** Run against live DB; parse JSON output with serde_json.
- **Rationale:** Confirms the refactor compiles and executes end-to-end, not just unit-level.
- **Gap if missing:** Module-level compile or dispatch errors not caught by unit tests.
- **Necessity:** RECOMMENDED (marked `#[ignore]`, requires DB)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | Render functions are pure logic testable in isolation | JSON regression undetected | MUST IMPLEMENT |
| Integration tests (DB) | RECOMMENDED | End-to-end execution correctness | Dispatch or compile errors | IMPLEMENT with #[ignore] |
| Benchmark tests | NOT NEEDED | No performance requirements for this refactor | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Test Type(s) | Test Cases |
|----------------|-----------------|--------------|------------|
| F1-AC1 | Both JSON functions use serde_json | Unit (grep/structural) | TC-57-F1-01 |
| F1-AC2 | JSON output semantically equivalent | Unit (parse and compare) | TC-57-F1-02, TC-57-F1-03, TC-57-F1-04 |
| F1-AC3 | All existing search JSON tests pass | Unit (existing test suite) | Existing tests (re-run) |
| F1-AC4 | No new clippy warnings | cargo clippy | TC-57-F1-05 |

#### 5. Gap Analysis

**Interactive/PTY Tests:** Not needed. There is no PTY interaction — render functions write to a Vec<u8>.

**Performance/Benchmark Tests:** Not needed. Spec has no timing requirement for JSON rendering.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/search.rs` `#[cfg(test)]` module
- **Framework:** Rust built-in `#[test]`
- **Test count:** 5 new tests (plus all 12 existing tests must continue to pass)
- **Key scenarios:**
  1. TC-57-F1-01: Structural grep — `MAX_SEARCH_FETCH` constant exists in source (can be implemented as a doc/grep check in test strategy; verified structurally)
  2. TC-57-F1-02: Table JSON with special characters in name (double-quote, backslash) — parse result with serde_json
  3. TC-57-F1-03: Column JSON with special characters in column_type field
  4. TC-57-F1-04: Table JSON multi-row — parse output as `serde_json::Value`, verify `data` array length matches `row_count`
  5. TC-57-F1-05: `cargo clippy -- -D warnings` produces zero warnings (run in CI/execution phase)
- **Mocking strategy:** None needed; render functions accept slices.

---

### Feature 2: Named Constant MAX_SEARCH_FETCH

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-57-planning.md` §Feature 2 (P0)
- Requirements:
  1. Named constant `MAX_SEARCH_FETCH` defined in `src/commands/search.rs` (AC-1)
  2. Constant used in both table and column search pagination branches (AC-2)
  3. Existing behavior unchanged — same numeric value 100000 (AC-3)

**Feature Characteristics:**

**User Interaction Type:** Pure Logic — code quality refactor, no behavioral change.

**Observable Behavior:** None from user perspective.

**External Dependencies:** None.

**Validation Challenges:**
- The constant must actually replace both occurrences, not just define a dead constant.
- The numeric value must remain 100000 (or be documented if it changes).

**Critical Behaviors to Validate:**
1. Constant `MAX_SEARCH_FETCH` exists with value 100000 (sprint-57-planning.md §F2 AC-1)
2. Literal `100000` no longer appears in the pagination branches (sprint-57-planning.md §F2 AC-2)

#### 2. Test Strategy Derivation

A structural test (grep on source) is the appropriate validation. The constant is used in runtime branches that require a DB to exercise; the unit test for this feature is a source-level assertion.

**Derived Test Types:**

**Test Type 1: Structural source check**
- **Validates:** Constant is defined and magic literal is removed.
- **Approach:** In the test strategy execution, use `grep` to assert `const MAX_SEARCH_FETCH` exists and that raw `100000` does not appear in the pagination branches of `search.rs`.
- **Rationale:** No behavior change means no behavioral test needed; only structural correctness matters.
- **Gap if missing:** Developer might define constant but forget to substitute it.
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Structural source check | REQUIRED | Verifies substitution actually happened | Dead constant, literal remains | MUST IMPLEMENT |
| Unit tests | NOT NEEDED | No behavioral change to test | N/A | SKIP |
| Integration tests | NOT NEEDED | Constant value is identical to hard-coded value | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Test Type(s) | Test Cases |
|----------------|-----------------|--------------|------------|
| F2-AC1 | `MAX_SEARCH_FETCH` constant defined | Structural grep | TC-57-F2-01 |
| F2-AC2 | Constant used in both search functions | Structural grep | TC-57-F2-01 |
| F2-AC3 | Existing behavior unchanged | Existing unit tests pass | Existing tests |

#### 5. Gap Analysis

No significant gaps. Behavioral tests for pagination already exist and cover the code path that uses this constant.

#### 6. Test Implementation Plan

**Test Type: Structural Grep Check**
- **Location:** Test execution phase (grep commands)
- **Commands:**
  ```
  grep -n "MAX_SEARCH_FETCH" src/commands/search.rs
  grep -c "100000" src/commands/search.rs   # should be 0 in pagination branches
  ```
- **Test count:** 1 check (TC-57-F2-01)

---

### Feature 3: ORDER BY Stability Warning in Pagination Docs

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-57-planning.md` §Feature 3 (P1)
- Requirements:
  1. Warning added to pagination section of `docs/specifications/cli-interface.md` (AC-1)
  2. Warning added to `docs/specifications/batch-mode.md` (AC-2)

**Feature Characteristics:**

**User Interaction Type:** Pure documentation — no code change.

**Observable Behavior:** Document content only.

**External Dependencies:** None.

**Validation Challenges:** None significant. Presence check on document content is sufficient.

**Critical Behaviors to Validate:**
1. Warning text is present in `cli-interface.md` pagination section (sprint-57-planning.md §F3 AC-1)
2. Warning text is present in `batch-mode.md` (sprint-57-planning.md §F3 AC-2)

#### 2. Test Strategy Derivation

Document content check (grep) is the appropriate test type.

**Derived Test Types:**

**Test Type 1: Documentation content check**
- **Validates:** Warning about ORDER BY determinism is present in both documents.
- **Approach:** `grep -i "order by" docs/specifications/cli-interface.md` and same for `batch-mode.md`.
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Doc content grep | REQUIRED | Verifies documentation was actually updated | Missing warning, users unaware of instability | MUST IMPLEMENT |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Test Type(s) | Test Cases |
|----------------|-----------------|--------------|------------|
| F3-AC1 | Warning in cli-interface.md pagination section | Doc grep | TC-57-F3-01 |
| F3-AC2 | Warning in batch-mode.md | Doc grep | TC-57-F3-02 |

#### 5. Gap Analysis

No gaps. Documentation acceptance criteria are binary (present/absent).

#### 6. Test Implementation Plan

**Test Type: Documentation Content Check**
- **Location:** Execution phase (grep commands)
- **Commands:**
  ```
  grep -i "order by" docs/specifications/cli-interface.md
  grep -i "order by" docs/specifications/batch-mode.md
  ```
- **Test count:** 2 checks (TC-57-F3-01, TC-57-F3-02)

---

### Feature 4: `tq search views <keyword>` and REPL `/search views`

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-57-planning.md` §Feature 4 (P1)
- Secondary: `docs/specifications/cli-interface.md` (search command section)
- Requirements:
  1. `tq search views <keyword>` works in batch mode with all output formats: table, JSON, CSV, markdown (AC-1)
  2. `/search views <keyword>` works in REPL with `in <db>` scoping (AC-2)
  3. JSON output follows standard agent-mode envelope (`ok`, `row_count`, `data`) (AC-3)
  4. `--limit` and `--database` flags work (AC-4)
  5. Pagination support: `--page`, `--page-size` (AC-5)
  6. Tab completion for `/search views` in REPL (AC-6)
  7. Unit tests for all render functions (AC-7)

**Feature Characteristics:**

**User Interaction Type:** CLI Batch + Interactive PTY (REPL).

**Explanation:** The batch subcommand is a pure CLI Batch interaction. The REPL `/search views` dispatch and tab completion require PTY simulation to validate the interactive path.

**Observable Behavior:**
- [x] Structured data output (JSON, CSV, markdown)
- [x] Visual output in terminal (table format)
- [x] State management (REPL dispatch, `in <db>` scope parsing)

**External Dependencies:**
- [x] Database connection (integration tests, marked `#[ignore]`)
- [x] Terminal/PTY (REPL tab completion — interactive tests)
- [ ] None (unit tests for render functions)

**Validation Challenges:**
- SQL query for views must reference correct Teradata system view (DBC.TablesV with TableKind = 'V') — requires live DB to confirm.
- REPL `in <db>` scoping syntax must be parsed correctly — unit-testable via dispatch string parsing.
- Tab completion adds `views` to the `/search` completion candidates — requires interactive test or direct unit test on the completion list.
- All four render functions must be implemented; unit tests can cover all four without DB.

**Critical Behaviors to Validate:**
1. "All output formats work" — table, JSON, CSV, markdown render functions produce valid output (sprint-57-planning.md §F4 AC-1)
2. "JSON output follows standard envelope" — `{"ok": true, "row_count": N, "data": [...]}` (sprint-57-planning.md §F4 AC-3)
3. "Pagination support" — `render_view_search_json_with_pagination` includes pagination sub-object when provided (sprint-57-planning.md §F4 AC-5)
4. "Empty results handled gracefully" — table format shows `(no views found)`, JSON shows `row_count: 0` (derived from existing tables/columns pattern)
5. "REPL dispatch routes `views` subcommand" — `execute_for_repl("views", ...)` is handled without falling to unknown branch (sprint-57-planning.md §F4 AC-2)
6. "Tab completion includes `views`" — completion candidates list contains "views" (sprint-57-planning.md §F4 AC-6)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

- "CLI Batch" checked: end-to-end integration test recommended but can be deferred to DB-required `#[ignore]` tests.
- "Interactive PTY" checked for REPL tab completion: interactive test with expectrl REQUIRED for tab completion validation.
- "Database connection" checked: integration tests marked `#[ignore]`.
- "Pure Logic" applicable to all render functions: unit tests sufficient for render coverage.

**Derived Test Types:**

**Test Type 1: Unit Tests — render functions**
- **Validates:** All four render functions (table, JSON, CSV, markdown) produce correct output for view search results.
- **Approach:** Implement `ViewSearchResult` struct analogous to `TableSearchResult`. Call render functions with crafted slices. Assert field names, envelope structure, header row content.
- **Rationale:** Render functions are pure Write-injected; no DB required.
- **Gap if missing:** Render bugs only caught at runtime with live DB.
- **Necessity:** REQUIRED

**Test Type 2: Unit Tests — REPL dispatch**
- **Validates:** `execute_for_repl("views", ...)` is correctly routed and does not fall through to the unknown branch.
- **Approach:** Test the string matching logic in `execute_for_repl`. Since this requires a DB client, the dispatch routing can be tested by verifying the match arm exists (structural) and by integration test.
- **Rationale:** Ensures REPL dispatch covers "views" without DB call for routing logic.
- **Gap if missing:** REPL `/search views` silently falls to "unknown subcommand" error.
- **Necessity:** REQUIRED (unit test for dispatch match coverage; integration for full path)

**Test Type 3: Unit Tests — tab completion**
- **Validates:** The completion candidate list for `/search` includes "views".
- **Approach:** Locate completion list for `/search` subcommands in the REPL completion module; assert "views" is present. This is a direct unit test on the completion array/list.
- **Rationale:** Completion is a static list; unit-testable without PTY.
- **Gap if missing:** Tab completion silently omits "views", discovered only during manual testing.
- **Necessity:** REQUIRED

**Test Type 4: Integration Tests (DB, #[ignore])**
- **Validates:** Live SQL query against DBC.TablesV with TableKind='V' returns correct results; all formats produce parseable output.
- **Approach:** Run `tq search views <keyword>` against live DB; parse JSON output.
- **Rationale:** SQL syntax errors not caught by unit tests.
- **Gap if missing:** Invalid SQL or wrong system table not caught before release.
- **Necessity:** RECOMMENDED (marked `#[ignore]`)

**Test Type 5: Interactive Tests (expectrl) — REPL tab completion**
- **Validates:** TAB key after `/search ` shows "views" in completion candidates in the live REPL.
- **Approach:** expectrl spawn tq REPL; type `/search `, send TAB, assert "views" appears in output.
- **Rationale:** Only PTY test can confirm completion actually renders in terminal.
- **Gap if missing:** Completion registered in list but not wired to REPL input handler.
- **Necessity:** RECOMMENDED (existing interactive test infrastructure; only adds one scenario)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests — render functions | REQUIRED | Pure logic, no DB, 4 render functions | Render bugs only found at runtime | MUST IMPLEMENT |
| Unit tests — REPL dispatch | REQUIRED | Verify "views" routing without DB | Silent "unknown subcommand" failure | MUST IMPLEMENT |
| Unit tests — completion list | REQUIRED | Static list, unit-testable | Tab completion silently missing | MUST IMPLEMENT |
| Integration tests (DB) | RECOMMENDED | SQL syntax validation | Invalid SQL undetected | IMPLEMENT with #[ignore] |
| Interactive tests (expectrl) | RECOMMENDED | PTY completion rendering | Completion wired but not rendered | IMPLEMENT if infra available |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Test Type(s) | Test Cases |
|----------------|-----------------|--------------|------------|
| F4-AC1-table | Table format renders view results | Unit | TC-57-F4-01 |
| F4-AC1-json | JSON format follows envelope | Unit | TC-57-F4-02 |
| F4-AC1-csv | CSV format has correct header + rows | Unit | TC-57-F4-03 |
| F4-AC1-markdown | Markdown format has pipe table | Unit | TC-57-F4-04 |
| F4-AC2 | REPL `/search views` dispatches correctly | Unit (dispatch) + Integration | TC-57-F4-05, TC-57-F4-10 |
| F4-AC3 | JSON envelope: ok, row_count, data | Unit | TC-57-F4-02 |
| F4-AC4 | --limit and --database flags work | Integration (#[ignore]) | TC-57-F4-10 |
| F4-AC5 | Pagination: --page, --page-size | Unit (render with PaginationInfo) | TC-57-F4-06 |
| F4-AC6 | Tab completion includes "views" | Unit (completion list) | TC-57-F4-07 |
| F4-AC7 | Unit tests for all render functions | Unit | TC-57-F4-01 through TC-57-F4-04 |
| F4-empty | Empty results handled gracefully | Unit | TC-57-F4-08 |
| F4-special | Special chars in view names | Unit | TC-57-F4-09 |

#### 5. Gap Analysis

**Interactive PTY Tests for REPL tab completion:**
- **Reason for possible omission:** expectrl infrastructure may require setup time; this sprint is medium-complexity.
- **What won't be validated:** Terminal rendering of completion candidates.
- **Risk assessment:** LOW — unit test on completion list catches the registration; PTY test catches wiring.
- **Mitigation:** If interactive tests are omitted, add a manual test note in test case TC-57-F4-07.
- **Revisit criteria:** Add interactive test if tab completion issues are reported after release.

**SQL correctness (DBC.TablesV TableKind='V'):**
- **Reason for possible omission:** Requires live DB.
- **Risk assessment:** MEDIUM — wrong TableKind filter silently returns empty results.
- **Mitigation:** Integration test marked `#[ignore]` covers this; DB must be available for full validation.
- **Revisit criteria:** Unblock if DB credentials available.

#### 6. Test Implementation Plan

**Test Type: Unit Tests — render functions**
- **Location:** `src/commands/search.rs` `#[cfg(test)]` module
- **Framework:** Rust built-in `#[test]`
- **Test count:** 9 new unit tests (TC-57-F4-01 through TC-57-F4-09)
- **Key scenarios:**
  1. TC-57-F4-01: `render_view_search_table` with 2 rows — check header, rows, count line
  2. TC-57-F4-02: `render_view_search_json_with_pagination` with 1 row, no pagination — envelope structure
  3. TC-57-F4-03: `render_view_search_csv` — header row + data row
  4. TC-57-F4-04: `render_view_search_markdown` — pipe table header + data row
  5. TC-57-F4-05: REPL dispatch — `execute_for_repl("views", ...)` routes to view search (structural)
  6. TC-57-F4-06: `render_view_search_json_with_pagination` with `PaginationInfo` — pagination sub-object present
  7. TC-57-F4-07: Completion candidates list includes "views" (unit test on completion array)
  8. TC-57-F4-08: `render_view_search_table` with empty slice — shows "(no views found)"
  9. TC-57-F4-09: `render_view_search_json_with_pagination` with special chars in view name — output parses as valid JSON

**Test Type: Integration Tests (DB, #[ignore])**
- **Location:** `tests/integration_tests.rs` or inline `#[ignore]` in `search.rs`
- **Framework:** Rust built-in `#[test]` with `#[ignore]`
- **Test count:** 1 integration test (TC-57-F4-10)
- **Key scenarios:**
  1. TC-57-F4-10: `tq search views <keyword> --format json` against live DB — output parses as valid JSON, `ok: true`
- **Setup requirements:** `TQ_LOGON` environment variable set in `.env`

---

## Strategy Summary

**Total Features Analyzed:** 4

**Test Types Required:**

- Unit tests: REQUIRED for Features 1, 2 (structural), 4
- Documentation content checks: REQUIRED for Feature 3
- Integration tests (DB, #[ignore]): RECOMMENDED for Features 1, 4
- Interactive tests (expectrl): RECOMMENDED for Feature 4 (tab completion)
- Benchmark tests: NOT NEEDED for any feature

**Estimated Test Count:**

| Category | Count |
|----------|-------|
| Existing unit tests (must continue passing) | 12 |
| New unit tests — Feature 1 serde edge cases | 5 |
| New unit tests — Feature 2 structural | 1 (grep-based in execution) |
| New unit tests — Feature 4 render + dispatch + completion | 9 |
| Documentation content checks — Feature 3 | 2 |
| Integration tests (#[ignore]) | 1 |
| **Total new tests** | **18** |

**All tests that can run without a DB are unit tests requiring no `#[ignore]`.**
**Integration tests are marked `#[ignore]` and require `TQ_LOGON` to be set.**

**Risk Assessment:**

- HIGH risk gaps: None
- MEDIUM risk gaps: SQL correctness for views query (mitigated by integration test when DB available)
- LOW risk gaps: Interactive PTY tab completion rendering (mitigated by completion list unit test)

**Dependencies Required:**

- Live database: Yes (for integration tests only; all unit tests run offline)
- Network access: No
- Specific OS: No
- Other: `TQ_LOGON` env var in `.env` for integration tests

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
**Created Date:** 2026-04-06
**Review Status:** DRAFT
**Submitted for Review:** 2026-04-06
