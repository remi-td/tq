# Sprint 55 Test Strategy: Search/Discovery Commands

**Created:** 2026-03-31
**Author:** quality-validator
**Sprint:** Sprint 55
**Features:**
1. `tq search tables <keyword>` — Cross-database table search (batch mode)
2. `tq search columns <keyword>` — Cross-database column search (batch mode)
3. REPL `/search` metacommand with tab completion

---

## Feature-by-Feature Test Strategy

---

### Feature 1: `tq search tables <keyword>`

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-55-planning.md` §Feature 1
- Secondary: `docs/specifications/cli-interface.md` (search command section)
- Requirements:
  1. `tq search tables emp` finds tables containing "emp" across all databases (AC-1)
  2. `--database` flag scopes search to a single database (AC-2)
  3. All four output formats supported: table, JSON, CSV, markdown (AC-3)
  4. JSON output uses standard envelope `{"ok": true, "row_count": N, "data": [...]}` (AC-4)
  5. Agent-safe mode compatible — read-only query (AC-5)
  6. Handles no-results gracefully (AC-6)

**Feature Characteristics:**

**User Interaction Type:** CLI Batch — scripted, piped, non-interactive command execution via `tq search tables <keyword>`.

**Explanation:** This command is invoked from the shell with positional and flag arguments. Its output is consumed programmatically (agents) or human-read directly. No PTY or terminal control sequences involved.

**Observable Behavior:**
- [x] Structured data output (JSON, CSV, markdown)
- [x] Visual output in terminal (formatted table layout)

**External Dependencies:**
- [x] Database connection (requires live database for integration tests)
- [ ] None (writer-injection unit tests use mock data)

**Validation Challenges:**
- SQL query correctness (LIKE matching, scoping by database) requires a live Teradata instance
- JSON envelope structure must match the established contract used by all other commands
- No-results case must output the envelope with `row_count: 0` rather than crashing or printing nothing

**Critical Behaviors to Validate:**
1. "finds tables containing 'emp' across all databases" — keyword matching works with SQL LIKE (sprint-55-planning.md §F1 AC-1)
2. "--database flag scopes search to a single database" — filtering is applied (sprint-55-planning.md §F1 AC-2)
3. "JSON output uses standard envelope" — `{"ok": true, "row_count": N, "data": [...]}` (sprint-55-planning.md §F1 AC-4)
4. "Handles no-results gracefully" — non-zero row_count is 0, no panic, exit 0 (sprint-55-planning.md §F1 AC-6)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "CLI Batch" checked:
  → Integration tests REQUIRED (end-to-end CLI execution needs validation)

IF "Structured data output" checked:
  → Unit tests with writer-injection REQUIRED (validates rendering logic)
  → Integration tests with live DB REQUIRED (validates SQL + rendering together)

IF "Database connection" checked:
  → Integration tests with live DB REQUIRED (mocks cannot catch SQL syntax errors)
```

**Derived Test Types:**

**Test Type 1: Unit Tests with Writer-Injection**
- **Validates:** AC-3, AC-4, AC-6 — rendering functions produce correct output for all formats with both populated and empty result sets
- **Approach:** Build mock `TableSearchResult` structs, call rendering functions (`render_table_format`, `render_json_format`, `render_csv_format`, `render_markdown_format`) with a `Vec<u8>` writer. Parse JSON with serde_json. Assert presence of headers, data values, and envelope structure.
- **Rationale:** These tests exercise the formatting logic entirely in memory — no DB needed. They can run in CI without credentials and provide fast regression coverage.
- **Gap if missing:** Rendering bugs (wrong headers, wrong JSON keys, missing rows) would only be caught with a live DB, slowing the feedback loop significantly.
- **Necessity:** REQUIRED

**Test Type 2: Unit Tests — Struct Construction**
- **Validates:** AC-3 (data model) — `TableSearchResult` fields are correctly populated and accessible
- **Approach:** Construct `TableSearchResult` structs directly in tests, assert field values. Mirrors patterns in `list.rs` (`test_table_entry_structure`).
- **Rationale:** Verifies the data model is correct before testing rendering. Catches typos, wrong types, and missing fields at compile time.
- **Gap if missing:** Struct mismatches (wrong field names, wrong types) would only surface at render time.
- **Necessity:** REQUIRED

**Test Type 3: Integration Tests (live DB, `#[ignore]`)**
- **Validates:** AC-1, AC-2, AC-5, AC-6 — end-to-end search query execution with real Teradata data
- **Approach:** Call `tq search tables <known_keyword>` via `assert_cmd::Command`, verify exit code 0 and output contains expected results. Test `--database` scoping. Test no-results case with a keyword that matches nothing. Test `--format json` and parse the envelope.
- **Rationale:** Writer-injection tests mock the data layer. Only a live DB can verify the SQL LIKE query runs correctly, the column mapping is right, and the connection path (agent-safe mode) works.
- **Gap if missing:** SQL syntax errors, wrong system table references, or type coercion issues go undetected.
- **Necessity:** REQUIRED (marked `#[ignore]`)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests — struct construction | REQUIRED | Validates data model correctness at compile time | Struct field mismatches only caught at runtime | MUST IMPLEMENT |
| Unit tests — writer-injection rendering | REQUIRED | Validates all four output formats without DB | Rendering bugs need live DB to detect | MUST IMPLEMENT |
| Integration tests (live DB, ignored) | REQUIRED | Validates SQL query, column mapping, scoping flag | SQL bugs, wrong system tables undetected | MUST IMPLEMENT |
| Interactive tests (expectrl) | NOT NEEDED | Feature is batch-only; REPL is Feature 3 | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance SLA specified | N/A | SKIP |

**Summary:**
- REQUIRED test types: 3 — MUST implement all
- NOT NEEDED test types: 2 — explicitly omitted with rationale

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| F1-AC-1 | `tq search tables emp` finds tables containing "emp" across all databases | sprint-55-planning.md §F1 | Integration (ignored) | TC-055-001 §C |
| F1-AC-2 | `--database` flag scopes search to a single database | sprint-55-planning.md §F1 | Integration (ignored) | TC-055-001 §D |
| F1-AC-3 | All four output formats supported | sprint-55-planning.md §F1 | Unit (writer-injection) | TC-055-001 §B |
| F1-AC-4 | JSON output uses standard envelope | sprint-55-planning.md §F1 | Unit (writer-injection) + Integration | TC-055-001 §B, §E |
| F1-AC-5 | Agent-safe mode compatible (read-only) | sprint-55-planning.md §F1 | Integration (ignored) | TC-055-001 §F |
| F1-AC-6 | Handles no-results gracefully | sprint-55-planning.md §F1 | Unit (empty mock) + Integration | TC-055-001 §B, §G |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements
- [x] No unjustified test types

**Coverage Gaps:**
- AC-5 (agent-safe mode) is partially tested by any read-only integration test. Full agent-safe enforcement (e.g., with `--agent-safe` flag if present) is tested by integration test TC-055-001 §F. If the agent-safe flag is not yet surfaced as a CLI argument for the search command, this test will document that gap.

#### 5. Gap Analysis

**Interactive Tests (expectrl)**
- **Reason for omission:** `tq search tables` is a CLI batch command with no PTY interaction
- **What won't be validated:** Terminal color rendering, cursor position (not applicable)
- **Risk assessment:** LOW — not applicable to this feature
- **Mitigation:** N/A
- **Revisit criteria:** N/A

**Benchmark Tests**
- **Reason for omission:** Sprint planning specifies no performance SLA for search commands
- **What won't be validated:** Query latency, memory allocation under large result sets
- **Risk assessment:** LOW — search is exploratory; latency tolerance is high
- **Mitigation:** Monitor if users report slowness; add benchmarks in a dedicated performance sprint
- **Revisit criteria:** If spec adds a performance requirement (e.g., "must complete in <5 s for 10,000 results")

#### 6. Test Implementation Plan

**Test Type: Unit Tests — Struct Construction**
- **Location:** `src/commands/search.rs` — `#[cfg(test)] mod tests`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 2 tests (one for `TableSearchResult`, one for `ColumnSearchResult`)
- **Key scenarios:**
  1. Construct `TableSearchResult` with known field values, assert each field accessible and correct
  2. Construct `ColumnSearchResult` with known field values, assert each field accessible and correct
- **Mocking strategy:** None — pure struct construction

**Test Type: Unit Tests — Writer-Injection Rendering**
- **Location:** `src/commands/search.rs` — `#[cfg(test)] mod tests`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 10 tests
  - Table search: table format (populated), table format (empty), JSON format (populated, envelope check), CSV format (header + escaping), markdown format
  - Column search: same 5 scenarios
- **Key scenarios (table search):**
  1. `render_search_tables_table(results, writer)` with 2 rows — assert "Database", "Table", "Type" headers present, row values present, footer "N result(s)"
  2. `render_search_tables_table([], writer)` — assert "No tables found" or row_count 0 message
  3. `render_search_tables_json(results, writer)` — parse JSON, assert `ok: true`, `row_count: 2`, `data` is array with correct fields
  4. `render_search_tables_csv(results, writer)` — assert CSV header line, correct comma separation, quoted fields containing commas
  5. `render_search_tables_markdown(results, writer)` — assert markdown table header `| Database | Table |`
- **Mocking strategy:** Construct `Vec<TableSearchResult>` and `Vec<ColumnSearchResult>` with hardcoded test data; pass `Vec<u8>` as writer

**Test Type: Integration Tests (live DB)**
- **Location:** `tests/integration_tests.rs` (new section) or `tests/integration_search.rs`
- **Framework:** Built-in Rust integration test support with `#[ignore]`
- **Test count estimate:** 7 tests (4 for table search, 3 for column search — see Feature 2)
- **Key scenarios (table search):**
  1. `tq search tables dbc` — exit 0, stdout contains at least one row
  2. `tq search tables dbc --format json` — exit 0, parse JSON, `ok == true`, `row_count >= 1`
  3. `tq search tables dbc --database DBC` — exit 0, results all have `database = "DBC"`
  4. `tq search tables xyzzy_no_match_abc` — exit 0, output indicates 0 results (no panic)
- **Setup requirements:** `TQ_LOGON` set in `.env`, live Teradata database accessible

#### 7. Coverage Sufficiency Assessment

**If all planned tests are implemented and passing, can we claim the feature "works as specified"?**

- Unit tests validate: all four output format renderers produce correct output structure with both populated and empty results
- Integration tests validate: SQL query runs, results flow through the full code path, `--database` scoping works, no-results is graceful, JSON envelope matches spec
- Combined coverage: comprehensive — the data model, rendering, and execution path are all covered

**Gaps in combined coverage:**
- Very large result sets (thousands of rows) are not load-tested — acceptable given no performance SLA
- The exact SQL generated (LIKE pattern construction) is not inspected in unit tests — validated indirectly by integration tests returning correct results

**Acceptance criteria:**
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified"
- [x] Known gaps are documented and accepted

---

### Feature 2: `tq search columns <keyword>`

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-55-planning.md` §Feature 2
- Requirements:
  1. `tq search columns salary` finds columns containing "salary" across databases (AC-1)
  2. `--database` flag scopes search to a single database (AC-2)
  3. All four output formats supported (AC-3)
  4. JSON output uses standard envelope (AC-4)
  5. Agent-safe mode compatible (AC-5)
  6. Handles no-results gracefully (AC-6)

**Feature Characteristics:**

**User Interaction Type:** CLI Batch — identical pattern to Feature 1, but for columns.

**Explanation:** Same batch execution model as Feature 1. Result set has different fields: database, table, column name, data type, nullable status.

**Observable Behavior:**
- [x] Structured data output (JSON, CSV, markdown)
- [x] Visual output in terminal (formatted table layout)

**External Dependencies:**
- [x] Database connection (live DB for integration tests)
- [ ] None (unit tests use mock data)

**Validation Challenges:**
- Column search returns more fields than table search (database, table, column, type, nullable) — rendering of nullable as boolean in JSON requires explicit validation
- The system catalog table used for column metadata must be correct (must be validated against live DB)

**Critical Behaviors to Validate:**
1. Results include database, table name, column name, data type, nullable status
2. JSON `nullable` field is a JSON boolean (matching established API convention from Sprint 48)
3. `--database` flag correctly restricts column search to one database
4. No-results handled gracefully

#### 2. Test Strategy Derivation

Identical decision tree to Feature 1. Both features share the same architecture: struct + rendering functions + CLI dispatch + integration tests.

**Derived Test Types:**

**Test Type 1: Unit Tests — Struct Construction**
- **Validates:** `ColumnSearchResult` data model has all required fields
- **Necessity:** REQUIRED

**Test Type 2: Unit Tests — Writer-Injection Rendering**
- **Validates:** AC-3, AC-4, AC-6 — all four output formats, JSON boolean for nullable, empty results
- **Approach:** Construct mock `ColumnSearchResult` structs with `nullable: true` and `nullable: false`. Call rendering functions with `Vec<u8>` writer. For JSON, parse with serde_json and assert `data[0]["nullable"]` is a JSON boolean, not a string.
- **Rationale:** The nullable boolean convention is an established API contract (Issue #37, Sprint 48). Verifying it in unit tests catches regressions without a live DB.
- **Necessity:** REQUIRED

**Test Type 3: Integration Tests (live DB, `#[ignore]`)**
- **Validates:** AC-1, AC-2, AC-5, AC-6 — real column metadata query execution
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests — struct construction | REQUIRED | Validates `ColumnSearchResult` data model | Struct mismatches only caught at runtime | MUST IMPLEMENT |
| Unit tests — writer-injection rendering | REQUIRED | Validates all formats + nullable boolean type | Nullable type bug only caught with live DB | MUST IMPLEMENT |
| Integration tests (live DB, ignored) | REQUIRED | Validates column metadata SQL | Wrong system catalog references undetected | MUST IMPLEMENT |
| Interactive tests | NOT NEEDED | Batch command, REPL tested in Feature 3 | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance SLA | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| F2-AC-1 | `tq search columns salary` finds columns containing "salary" | sprint-55-planning.md §F2 | Integration (ignored) | TC-055-002 §C |
| F2-AC-2 | `--database` flag scopes to single database | sprint-55-planning.md §F2 | Integration (ignored) | TC-055-002 §D |
| F2-AC-3 | All four output formats | sprint-55-planning.md §F2 | Unit (writer-injection) | TC-055-002 §B |
| F2-AC-4 | JSON output uses standard envelope; nullable is boolean | sprint-55-planning.md §F2 | Unit (writer-injection) + Integration | TC-055-002 §B, §E |
| F2-AC-5 | Agent-safe mode compatible | sprint-55-planning.md §F2 | Integration (ignored) | TC-055-002 §F |
| F2-AC-6 | Handles no-results gracefully | sprint-55-planning.md §F2 | Unit (empty mock) + Integration | TC-055-002 §B, §G |

**Coverage Gaps:**
- Same as Feature 1: no large-result performance test (acceptable, no SLA)

#### 5. Gap Analysis

**Interactive Tests** — NOT NEEDED for the same reason as Feature 1 (batch command).

**Benchmark Tests** — NOT NEEDED for the same reason as Feature 1.

#### 6. Test Implementation Plan

**Test Type: Unit Tests — Struct Construction + Writer-Injection**
- **Location:** `src/commands/search.rs` — `#[cfg(test)] mod tests`
- **Framework:** `#[test]`
- **Test count estimate:** 6 tests
  1. `ColumnSearchResult` struct construction — assert field types and values
  2. `render_search_columns_table(results, writer)` — populated: assert "Database", "Table", "Column", "Type", "Nullable" headers; 2 rows present; footer
  3. `render_search_columns_table([], writer)` — assert empty/0-result message
  4. `render_search_columns_json(results, writer)` — parse JSON; `ok: true`; `data[0]["nullable"]` is `Value::Bool`, not `Value::String`
  5. `render_search_columns_csv(results, writer)` — header line, correct separation, quoted commas
  6. `render_search_columns_markdown(results, writer)` — markdown table header present
- **Mocking strategy:** `Vec<ColumnSearchResult>` with hardcoded values including both `nullable: true` and `nullable: false`

**Test Type: Integration Tests (live DB)**
- **Location:** Same file as Feature 1 integration tests
- **Framework:** `#[ignore]`
- **Test count estimate:** 3 tests
  1. `tq search columns name` — exit 0, at least one result row
  2. `tq search columns name --format json` — parse JSON, `ok == true`, `nullable` is boolean
  3. `tq search columns xyzzy_no_match_abc` — exit 0, 0 results, no panic

#### 7. Coverage Sufficiency Assessment

- Unit tests validate: `ColumnSearchResult` data model, all four renderers, nullable boolean type in JSON, empty results handling
- Integration tests validate: SQL column metadata query runs against real Teradata, column mapping is correct, `--database` scoping works
- Combined coverage: comprehensive for all six acceptance criteria

---

### Feature 3: REPL `/search` Metacommand

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-55-planning.md` §Feature 3
- Requirements:
  1. `/search tables <keyword>` works in REPL (AC-1)
  2. `/search columns <keyword>` works in REPL (AC-2)
  3. Tab completion for `/search` and its subcommands (AC-3)
  4. `/search` without arguments shows help text (AC-4)

**Feature Characteristics:**

**User Interaction Type:** Interactive PTY — the REPL is a readline-based interactive session in a pseudo-terminal. User types metacommands, sees output, and can use tab completion.

**Explanation:** Even though the underlying search logic is shared with batch mode, the REPL metacommand path goes through the REPL dispatch loop (`/` prefix handling) and uses a different execution path than the CLI batch command. Tab completion requires PTY interaction that cannot be simulated with writer-injection.

**Observable Behavior:**
- [x] Visual output in terminal (REPL output rendered in PTY)
- [x] State management (REPL session state: current database context)

**External Dependencies:**
- [x] Database connection (REPL requires live DB to connect at startup)
- [x] Terminal/PTY (tab completion requires PTY character-by-character input)

**Validation Challenges:**
- Tab completion can only be verified in a real PTY — unit tests cannot simulate the keypress → completion → display cycle
- REPL output is interleaved with the prompt; extracting specific output requires expectrl pattern matching
- `/search` without arguments must produce help text — the exact text must be verified

**Critical Behaviors to Validate:**
1. `/search tables emp` executes and produces a formatted result in the REPL session (AC-1)
2. `/search columns salary` executes and produces a formatted result (AC-2)
3. Typing `/search` + Tab shows `/search tables` and `/search columns` as completions (AC-3)
4. Typing `/search` alone (no subcommand or keyword) shows help text (AC-4)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Unit tests cannot validate REPL dispatch, PTY rendering, tab completion

IF "Database connection" checked:
  → Integration tests with live database REQUIRED
  Reason: REPL connects at startup; metacommand executes a real query
```

**Derived Test Types:**

**Test Type 1: Interactive Tests (expectrl)**
- **Validates:** AC-1, AC-2, AC-3, AC-4 — REPL metacommand dispatch, tab completion, help text
- **Approach:** Spawn `tq repl --no-syntax-highlight --no-pager` via `expectrl::spawn`. Wait for "Connected to". Send `/search tables dbc\r`, wait for output containing a result row or "0 result(s)". Send `/search\r`, wait for help text. For tab completion: send `/search ` then `\t`, wait for `tables` and `columns` in the buffer.
- **Rationale:** The REPL metacommand dispatch code path is entirely separate from batch dispatch. Only PTY-based tests exercise this path.
- **Gap if missing:** REPL dispatch bugs (e.g., `/search` not recognized, wrong result rendering in REPL context) go undetected.
- **Necessity:** REQUIRED

**Test Type 2: Unit Tests — REPL Dispatch Registration**
- **Validates:** AC-3 (partial), AC-4 — the REPL metacommand registry includes `/search` with its subcommands; the help text string is correct
- **Approach:** Unit test that inspects the metacommand list/registry (if it exposes a testable API) or verifies that the help text constant contains the correct subcommand descriptions. Mirrors existing patterns in `src/commands/repl/mod.rs`.
- **Rationale:** If the metacommand registry is exposed as a data structure, unit tests can verify registration without a PTY. This is a fast, non-interactive supplement.
- **Gap if missing:** Registration bugs require PTY tests to detect. Risk is LOW because PTY tests cover this.
- **Necessity:** RECOMMENDED — implement if the registry is testable as a data structure; skip if it requires PTY

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Interactive tests (expectrl) | REQUIRED | Only test type that exercises REPL metacommand path and PTY tab completion | REPL bugs undetectable; completion bugs undetectable | MUST IMPLEMENT |
| Unit tests — dispatch registration | RECOMMENDED | Fast validation of metacommand list if registry is testable | Covered by interactive tests; low risk if omitted | IMPLEMENT IF FEASIBLE |
| Unit tests — writer-injection | NOT NEEDED for REPL path | Rendering is shared with batch mode and already tested in F1/F2 | N/A — no new rendering code | SKIP |
| Integration tests (non-interactive) | NOT NEEDED | REPL is interactive-only; batch path tested in F1/F2 | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance SLA | N/A | SKIP |

**Summary:**
- REQUIRED: 1 (interactive tests)
- RECOMMENDED: 1 (unit dispatch registration, if feasible)
- NOT NEEDED: 3

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| F3-AC-1 | `/search tables <keyword>` works in REPL | sprint-55-planning.md §F3 | Interactive (expectrl) | TC-055-003 §A |
| F3-AC-2 | `/search columns <keyword>` works in REPL | sprint-55-planning.md §F3 | Interactive (expectrl) | TC-055-003 §B |
| F3-AC-3 | Tab completion for `/search` and subcommands | sprint-55-planning.md §F3 | Interactive (expectrl) | TC-055-003 §C |
| F3-AC-4 | `/search` without args shows help | sprint-55-planning.md §F3 | Interactive (expectrl) + Unit | TC-055-003 §D |

**Coverage Gaps:**
- Tab completion in the PTY may be partially obscured by reedline cursor position detection issues (documented known limitation in `tests/README.md`). The interactive test for AC-3 may need to be lenient about exact completion list rendering, verifying at minimum that `/search tables` appears in the buffer. This is an inherent limitation of the test environment.

#### 5. Gap Analysis

**Unit Tests for Rendering (Writer-Injection)**
- **Reason for omission:** The REPL path calls the same rendering functions as the batch path (Feature 1 and 2). Those functions are already fully covered by F1/F2 writer-injection tests.
- **What won't be validated:** N/A — nothing is omitted
- **Risk assessment:** LOW

**Performance / Benchmark Tests**
- **Reason for omission:** No performance SLA on REPL metacommands
- **Risk assessment:** LOW

**Tab Completion Completeness**
- **Reason for gap:** PTY cursor issues may prevent reliable verification of the exact completion list display. The test will verify that output after pressing Tab contains the expected subcommand text.
- **Risk assessment:** MEDIUM — tab completion bugs could be missed if the PTY output is ambiguous
- **Mitigation:** In addition to the interactive test, verify the completer source in `src/commands/repl/metadata_completer.rs` includes `/search` entries via a structural grep check (unit-level verification that the string "/search" appears in the completer)

#### 6. Test Implementation Plan

**Test Type: Interactive Tests (expectrl)**
- **Location:** `tests/interactive_tests.rs` — new section "Sprint 55: Search Metacommands"
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 4 tests
- **Key scenarios:**
  1. `/search tables dbc` — wait for output, assert result line present (database "DBC" in output) or "result(s)" in footer
  2. `/search columns name` — wait for output, assert at least one column name result or "result(s)"
  3. `/search` alone — assert help text appears containing both "tables" and "columns" subcommand descriptions
  4. Tab completion: send `/search ` + `\t` — assert buffer contains "tables" and "columns" as completion candidates (lenient: check at least one)
- **Implementation notes:**
  - Use `std::thread::sleep(Duration::from_secs(2))` after sending the search command to allow query execution
  - All tests marked `#[ignore]` — require live DB
  - Follow existing pattern: `spawn_tq_repl()`, `p.expect("Connected to")`, send command, sleep, use `p.expect(pattern)` or buffer inspection
  - For no-results verification, use a keyword unlikely to match anything: `xyzzy_no_match_abc_55`
  - Clean up with `/quit` at end of each test

**Test Type: Unit Tests — Dispatch Registration (if feasible)**
- **Location:** `src/commands/repl/mod.rs` or `src/commands/repl/metadata_completer.rs` — `#[cfg(test)] mod tests`
- **Framework:** `#[test]`
- **Test count estimate:** 1-2 tests
- **Key scenarios:**
  1. The metacommand help text or registry includes the string "search" with "tables" and "columns" subcommand descriptions
  2. The completer hint strings include "/search tables" and "/search columns"
- **Mocking strategy:** None — pure unit-level string/data inspection

#### 7. Coverage Sufficiency Assessment

- Interactive tests validate: REPL metacommand dispatch, actual search execution in REPL context, help text display, tab completion behavior
- Unit tests (if feasible): validate registration of the metacommand at the data structure level
- Combined coverage: comprehensive for all four REPL acceptance criteria

**Gaps in combined coverage:**
- PTY cursor ambiguity may reduce confidence in tab completion verification (MEDIUM risk, mitigated by structural grep of completer source)

**Acceptance criteria:**
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified"
- [x] Known gaps are documented and accepted (PTY cursor limitation, tab completion ambiguity)

---

## Strategy Summary

**Total Features Analyzed:** 3

**Test Types Required:**
- Unit tests (struct construction + writer-injection): REQUIRED for Features 1 and 2
- Interactive tests (expectrl): REQUIRED for Feature 3
- Integration tests (live DB, `#[ignore]`): REQUIRED for Features 1 and 2

**Test Types Recommended:**
- Unit tests (REPL dispatch registration): RECOMMENDED for Feature 3 (implement if registry is testable)

**Estimated Test Count:**
- Unit tests (inline in `src/commands/search.rs`): ~16 tests
  - Feature 1: 6 tests (1 struct + 5 rendering)
  - Feature 2: 6 tests (1 struct + 5 rendering)
  - Feature 3 dispatch registration: 1-2 tests (if feasible)
  - Structural grep check for completer: 1 test
- Interactive tests (in `tests/interactive_tests.rs`): 4 tests
- Integration tests (in `tests/integration_tests.rs` or `tests/integration_search.rs`): 7 tests
  - Feature 1: 4 tests
  - Feature 2: 3 tests
- **Total new tests: ~27 tests**

**Risk Assessment:**
- HIGH risk gaps: none
- MEDIUM risk gaps: Tab completion verification in PTY may be ambiguous due to reedline cursor detection issues (mitigated by completer structural check)
- LOW risk gaps: No performance SLA tested; no large-result set tested; agent-safe flag coverage depends on CLI implementation

**Dependencies Required:**
- Live database: Yes — for integration tests (Features 1, 2) and interactive tests (Feature 3)
- Network access: No
- Specific OS: No
- Other: `TQ_LOGON` set in `.env`; `expectrl` crate in dev-dependencies (already present)

---

## Tool Requirements

**No new testing tools are required for this sprint.**

The existing infrastructure is fully sufficient:
- **Writer-injection pattern** (`Vec<u8>` as `Write` implementor) — established in `list.rs`, `explain.rs`, and all recent sprint tests. Directly applicable to search rendering functions.
- **`#[test]` / `#[ignore]`** — standard Rust test framework. Used for all integration tests.
- **`expectrl`** — already present in dev-dependencies and used in `tests/interactive_tests.rs`. Directly applicable to REPL metacommand tests.
- **`assert_cmd`** — already used for CLI invocation in integration tests. Directly applicable to batch `tq search` tests.
- **`serde_json`** — already used in integration tests for JSON parsing. Needed for JSON envelope validation.

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
**Created Date:** 2026-03-31
**Review Status:** DRAFT
**Submitted for Review:** 2026-03-31

**Reviewer:** tq-project-manager
**Review Status:** PENDING
**Review Date:** —
**Review Comments:** —

**Approval means:**
- Test strategy derived from specifications (not assumptions)
- All required test types identified with clear rationale
- Coverage gaps explicitly identified and assessed
- Implementation plan is detailed and achievable
- Ready to proceed with test implementation
