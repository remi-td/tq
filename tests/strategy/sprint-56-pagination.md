# Sprint 56 Test Strategy: Result Pagination & Sprint 55 Cleanup

**Created:** 2026-03-31
**Author:** quality-validator
**Sprint:** Sprint 56
**Features:**
- Feature 1: Query Result Pagination (`--page-size`, `--page` flags on `query` command)
- Feature 2: Pagination for Search and List commands
- Feature 3: Sprint 55 Tech Debt Cleanup (`esc()` consolidation, search dispatch tests)

---

## Feature-by-Feature Test Strategy

---

### Feature 1: Query Result Pagination

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-56-planning.md` §"Feature 1: Query Result Pagination"
- Secondary: `docs/specifications/cli-interface.md` (pagination flags to be added by cli-ux-designer)
- Requirements extracted from sprint plan:
  1. `--page-size N` slices results to N rows per page
  2. `--page P` selects which page (1-based, default 1)
  3. JSON envelope includes `page`, `page_size`, `has_more`, `total_rows` when pagination is active
  4. `has_more: true` when more pages exist, `false` on last page
  5. Without `--page-size`, output is unchanged (backward compatible)
  6. All output formats supported (table, JSON, CSV, markdown)
  7. Works with `--agent-safe` mode
  8. `--page` without `--page-size` produces an error

**Feature Characteristics:**

**User Interaction Type:**
- [x] CLI Batch (scripted, piped, non-interactive command execution)
- [x] Pure Logic (pagination slicing algorithm is internal)

**Explanation:** Users invoke `tq query --page-size N --page P "SQL"` from a shell. The pagination algorithm is pure logic (slice rows array). CLI Batch because it is a command-line flag combination.

**Observable Behavior:**
- [x] Structured data output (JSON envelope with pagination metadata)
- [x] Visual output in terminal (table/CSV/markdown footer "Page X of Y")

**External Dependencies:**
- [x] Database connection (fetching all rows requires live database for integration tests)
- [x] None for unit tests (pagination slicing is pure logic on in-memory QueryResult)

**Validation Challenges:**
- Integration tests require a live Teradata database (currently offline)
- Verifying `has_more` requires knowledge of total row count relative to page boundary
- Backward compatibility requires verifying that omitting `--page-size` does not change existing output structure

**Critical Behaviors to Validate:**
1. Slicing correctness: page 1 returns rows 1..N, page 2 returns rows N+1..2N, last page returns remaining rows
2. `has_more` accuracy: true when `page * page_size < total_rows`, false on last page
3. `total_rows` reflects the full result set size, not the page size
4. `row_count` in the envelope matches the slice size (not total), for agent parsing compatibility
5. Without `--page-size`, the envelope is unchanged (no `page`, `page_size`, `has_more`, `total_rows` fields)
6. `--page` without `--page-size` returns a meaningful error and non-zero exit code

#### 2. Test Strategy Derivation

**Decision Tree Results:**

- CLI Batch checked → Integration tests REQUIRED for end-to-end flag validation with real DB
- Pure Logic checked → Unit tests REQUIRED for slicing algorithm and envelope construction
- Structured data output checked → Unit tests with output capture can validate JSON structure
- Database connection checked → Integration tests with live DB are REQUIRED but currently BLOCKED (offline)

**Derived Test Types:**

**Test Type 1: Unit Tests (writer-injection pattern)**
- **Validates:** Pagination slicing logic, JSON envelope fields, has_more/total_rows accuracy, backward compatibility (no pagination fields when flag absent), error on --page without --page-size, error on --page-size with --limit
- **Approach:** Construct in-memory `QueryResult` with known rows; call paginated render functions with mock writer; parse JSON output; assert envelope fields and data slice. Use `Vec<u8>` as writer.
- **Rationale:** The slicing algorithm and envelope construction are pure logic with no external deps. Writer-injection pattern already established in codebase (see `src/format/json.rs` tests).
- **Gap if missing:** Logic bugs in slicing (off-by-one on page boundaries, wrong has_more), envelope field naming errors, backward compat regressions
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (live DB, `#[ignore]`)**
- **Validates:** End-to-end: flags parsed by CLI → query executed → rows sliced → output formatted
- **Approach:** Execute `tq query --page-size 5 "SELECT ..."` against live DB; capture stdout; parse JSON; verify page slice and metadata
- **Rationale:** Unit tests mock data; integration tests catch CLI argument parsing bugs, flag wiring to slicing logic, and real DB row counts
- **Gap if missing:** CLI flag wiring defects (--page-size not passed to slicer), SQL result set size mismatches
- **Necessity:** REQUIRED but currently BLOCKED (DB offline). Mark `#[ignore]`.

**Test Type 3: Interactive Tests (expectrl PTY)**
- **Validates:** REPL pagination behavior if exposed via metacommand
- **Approach:** Would drive tq REPL with expectrl to execute paginated query and verify output
- **Rationale:** Sprint 56 scope targets batch CLI only; REPL pagination not in scope
- **Gap if missing:** REPL-specific pagination behavior not covered
- **Necessity:** NOT NEEDED for this sprint (REPL pagination out of scope)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | Validates slicing logic, envelope fields, error cases | Logic bugs not caught | MUST IMPLEMENT |
| Integration tests (DB) | REQUIRED but BLOCKED | Validates CLI arg wiring to real query results | Flag wiring defects | IMPLEMENT with `#[ignore]` |
| Interactive tests (PTY) | NOT NEEDED | REPL pagination not in sprint scope | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance SLA defined | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Test Type(s) | Test Cases |
|----------------|-----------------|--------------|------------|
| PAG-REQ-1 | `--page-size N` slices results to N rows | Unit + Integration | TC-056-001 (unit), TC-056-002 (integration) |
| PAG-REQ-2 | `--page P` selects 1-based page, default 1 | Unit + Integration | TC-056-001, TC-056-002 |
| PAG-REQ-3 | JSON envelope includes page/page_size/has_more/total_rows when paginated | Unit | TC-056-001 |
| PAG-REQ-4 | has_more=true when more pages exist, false on last page | Unit | TC-056-001 |
| PAG-REQ-5 | Without --page-size, output unchanged (backward compat) | Unit | TC-056-001 |
| PAG-REQ-6 | All output formats: table, JSON, CSV, markdown | Unit | TC-056-001 |
| PAG-REQ-7 | Works with --agent-safe mode | Unit | TC-056-001 |
| PAG-REQ-8 | --page without --page-size produces error | Unit | TC-056-001 |

**Coverage Gaps:**
- `total_rows` accuracy for very large result sets: only testable with live DB integration test (BLOCKED)
- Non-JSON format footer "Page X of Y": validated in unit tests via output string matching

---

### Feature 2: Pagination for Search and List Commands

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-56-planning.md` §"Feature 2: Pagination for Search and List"
- Requirements:
  1. `tq search tables <kw> --page-size 10` paginates search results
  2. `tq list tables --page-size 20` paginates list results
  3. JSON envelope includes pagination metadata for these commands
  4. `--limit` and `--page-size` are mutually exclusive (error if both provided)

**Feature Characteristics:**

**User Interaction Type:**
- [x] CLI Batch

**Explanation:** Same as Feature 1 but applied to search/list commands which already have `--limit`.

**External Dependencies:**
- [x] Database connection (for integration tests only)
- [x] None for unit tests (render functions use in-memory data structures)

**Critical Behaviors to Validate:**
1. Pagination applied to `TableSearchResult` and `ColumnSearchResult` slices (search)
2. Pagination applied to list command result structs
3. `--limit` and `--page-size` are mutually exclusive
4. JSON envelope includes pagination fields when `--page-size` is active

#### 2. Test Strategy Derivation

Same decision tree as Feature 1: Unit tests REQUIRED, Integration REQUIRED but BLOCKED.

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Pagination applied to search/list render functions, mutual exclusion of `--limit` and `--page-size`, envelope fields
- **Approach:** Build `Vec<TableSearchResult>` or equivalent list structs with known data; call render with pagination params; verify output slice and envelope
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (`#[ignore]`)**
- **Validates:** End-to-end: `tq search tables emp --page-size 5` against real DB
- **Necessity:** REQUIRED but BLOCKED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Decision |
|-----------|------------|----------|
| Unit tests | REQUIRED | MUST IMPLEMENT |
| Integration tests (DB) | REQUIRED but BLOCKED | IMPLEMENT with `#[ignore]` |
| Interactive/PTY | NOT NEEDED | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Test Type(s) | Test Cases |
|----------------|-----------------|--------------|------------|
| SL-PAG-REQ-1 | `tq search tables --page-size N` paginates | Unit + Integration | TC-056-003 |
| SL-PAG-REQ-2 | `tq list tables --page-size N` paginates | Unit + Integration | TC-056-003 |
| SL-PAG-REQ-3 | JSON envelope includes pagination metadata | Unit | TC-056-003 |
| SL-PAG-REQ-4 | `--limit` and `--page-size` mutually exclusive | Unit | TC-056-003 |

---

### Feature 3: Sprint 55 Tech Debt Cleanup

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-56-planning.md` §"Feature 3: Sprint 55 Tech Debt Cleanup"
- Requirements:
  1. `esc()` markdown escape function consolidated into `format_helpers.rs`
  2. REPL `/search` dispatch tests added for alias routing (`"t"`, `"table"`, `"col"`, `"column"`)
  3. Unused `_use_color` parameter addressed in `search.rs`

**Feature Characteristics:**

**User Interaction Type:**
- [x] Pure Logic (refactoring internal helpers)

**Explanation:** `esc()` is currently a nested function duplicated in `search.rs` and `list.rs`. Consolidating it is a pure refactoring. `/search` dispatch routing tests are pure unit tests of match arms.

**External Dependencies:**
- [x] None (pure logic, no external dependencies)

**Critical Behaviors to Validate:**
1. After consolidation, `markdown_escape_pipe()` in `format_helpers.rs` behaves identically to the inline `esc()` (replaces `|` with `\|`, leaves other chars unchanged)
2. REPL dispatch: `execute_for_repl` routes `"t"`, `"table"`, `"tables"` to table search; `"col"`, `"column"`, `"columns"`, `"c"` to column search; unknown strings produce error message
3. `_use_color` parameter: either removed or properly used (structural check)

#### 2. Test Strategy Derivation

- Pure Logic → Unit tests REQUIRED
- No external dependencies → No integration tests needed
- No PTY output → No interactive tests needed

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** `markdown_escape_pipe()` behavior, dispatch routing aliases
- **Approach:** Direct function call tests with `assert_eq!`. For dispatch, call `execute_for_repl` with mock writer and verify output contains expected section headers or error messages.
- **Necessity:** REQUIRED

**Test Type 2: Structural Grep Checks**
- **Validates:** `esc()` no longer defined inline in search.rs or list.rs; `_use_color` removed or used
- **Approach:** `grep -n "fn esc" src/commands/search.rs` must return empty; `grep -n "fn esc" src/commands/list.rs` must return empty
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Decision |
|-----------|------------|----------|
| Unit tests | REQUIRED | MUST IMPLEMENT |
| Structural grep | REQUIRED | MUST IMPLEMENT |
| Integration tests | NOT NEEDED | SKIP |
| Interactive/PTY | NOT NEEDED | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Test Type(s) | Test Cases |
|----------------|-----------------|--------------|------------|
| DEBT-REQ-1 | `esc()` consolidated into `format_helpers.rs` | Unit + Structural | TC-056-004 |
| DEBT-REQ-2 | REPL `/search` dispatch tests for all aliases | Unit | TC-056-004 |
| DEBT-REQ-3 | `_use_color` addressed | Structural | TC-056-004 |

---

## Strategy Summary

**Total Features Analyzed:** 3

**Test Types Required:**
- Unit tests: REQUIRED — Feature 1, Feature 2, Feature 3
- Integration tests (DB, `#[ignore]`): REQUIRED but BLOCKED — Feature 1, Feature 2
- Structural grep: REQUIRED — Feature 3
- Interactive/PTY: NOT NEEDED
- Benchmark tests: NOT NEEDED

**Estimated Test Count:**
- Unit: ~30 tests (TC-056-001: ~14, TC-056-003: ~8, TC-056-004: ~8)
- Integration (`#[ignore]`): ~6 tests (TC-056-002: ~4, TC-056-003 integration: ~2)
- Structural grep: ~3 checks (TC-056-004)
- Total: ~39 tests + 3 structural checks

**Risk Assessment:**
- HIGH risk gaps: Integration tests BLOCKED by offline DB — cannot verify CLI flag wiring end-to-end
- MEDIUM risk gaps: Non-JSON format footer not testable with live data (unit tests use fixed datasets)
- LOW risk gaps: REPL pagination not tested (out of scope)

**Dependencies Required:**
- Live database: Yes (for integration tests, currently BLOCKED)
- Network access: No
- Specific OS: No

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
