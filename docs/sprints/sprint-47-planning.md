---
sprint: 47
start_date: 2026-03-23
target_completion: 2026-03-23
status: Planning
---

# Sprint 47 Planning: Tech Debt Elimination & Command Enrichment

## Sprint Overview

**Sprint Goal:** Fix Bug #36, eliminate accumulated code duplication, wire REPL delegation, and enrich batch command output to match specifications.

**Sprint Theme:** Quality consolidation — close the spec/implementation gap that has persisted for 5 sprints while eliminating the growing code duplication.

---

## Reality Check Summary

- Reviewed sprints: 44, 45, 46
- Patterns detected:
  - **Spec/implementation gap (5 sprints)**: Architect implements functional MVP, coordinator doesn't verify output against spec before shipping
  - **Code duplication growing**: json_escape 4x, csv_escape 4x, parse_table_name 3x, truncate_str 3x (with UTF-8 bug)
  - **REPL not delegating**: /describe and /list still use separate implementations instead of calling batch modules
  - **New modules untested**: describe.rs and show_indexes.rs have zero unit tests
- Decision: **Feature Sprint** — No crisis, but tech debt is accumulating. This sprint prioritizes quality consolidation.
- Rationale: Five consecutive sprints with spec gaps and growing duplication. Addressing this now prevents compounding debt.

---

## Objectives

1. **Fix Bug #36**: /inspect shows garbled DDL for views, [NULL] column types for views
2. **Eliminate code duplication**: Extract shared helpers (json_escape, csv_escape, parse_table_name, truncate_str) to a common module
3. **Fix truncate_str UTF-8 safety**: Use char_indices() for proper Unicode boundary handling
4. **Wire REPL delegation**: /describe and /list metacommands delegate to batch modules
5. **Enrich batch command output**: Bring tq describe, tq list, tq show-indexes closer to specification
6. **Add unit tests**: Cover describe.rs and show_indexes.rs with proper test modules

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Bug #36 — /inspect DDL & Column Type Fix

**Description:** /inspect shows garbled view definition (truncated DDL) and [NULL] for all column types on views. The DDL retrieval needs to capture the full RequestText, and column type query needs to handle views correctly.

**Acceptance Criteria:**
- [ ] AC-1: `/inspect dbc.tables` shows complete view definition text (not garbled/truncated)
- [ ] AC-2: `/inspect` on a view shows column types from DBC.ColumnsV (not [NULL])
- [ ] AC-3: `/inspect` on a macro shows full macro definition
- [ ] AC-4: Unit tests cover DDL retrieval for views and macros

**Reference:** `docs/specifications/repl.md#object-inspection`, GitHub Issue #36

**Estimated Complexity:** Medium

---

#### Feature 2: Extract Shared Helpers Module

**Description:** Extract duplicated utility functions from describe.rs, list.rs, show_indexes.rs, inspect.rs into a shared `src/commands/format_helpers.rs` module.

**Acceptance Criteria:**
- [ ] AC-1: `json_escape()` exists once in format_helpers.rs, used by all 4 command modules
- [ ] AC-2: `csv_escape()` exists once in format_helpers.rs, used by all 4 command modules
- [ ] AC-3: `parse_table_name()` exists once in format_helpers.rs (or identifiers.rs), used by describe, show_indexes, inspect
- [ ] AC-4: `truncate_str()` exists once with proper UTF-8 char_indices() handling
- [ ] AC-5: Zero code duplication of these functions across the codebase
- [ ] AC-6: All existing tests pass after extraction

**Reference:** Sprint 46 review recommendation #7, #8

**Estimated Complexity:** Medium

---

#### Feature 3: REPL Delegation to Batch Modules

**Description:** Wire REPL `/describe` and `/list` metacommand handlers to delegate to the batch module implementations instead of maintaining separate code paths.

**Acceptance Criteria:**
- [ ] AC-1: `/describe <table>` in REPL calls `describe::execute_for_repl()`
- [ ] AC-2: `/list databases`, `/list tables`, `/list views` in REPL calls `list::execute_for_repl()`
- [ ] AC-3: REPL output is identical to previous behavior (no regression)
- [ ] AC-4: Existing REPL tests pass without modification

**Reference:** Sprint 46 review recommendation #6

**Estimated Complexity:** Medium

---

### P1 - High Priority (Should Have)

#### Feature 4: Enrich `tq describe` Output

**Description:** Add missing header block, Comments column, and Indexes section to match specification.

**Acceptance Criteria:**
- [ ] AC-1: Output includes object header (database, table name, type)
- [ ] AC-2: Columns table includes CommentString column
- [ ] AC-3: Indexes section shows primary and secondary indexes
- [ ] AC-4: JSON output uses structured `{object, columns[], indexes[]}` wrapper
- [ ] AC-5: Unit tests for describe.rs formatting functions

**Reference:** Sprint 46 review recommendation #1, #5

**Estimated Complexity:** Medium

---

#### Feature 5: Enrich `tq list` Output

**Description:** Add missing columns to list databases and list tables output.

**Acceptance Criteria:**
- [ ] AC-1: `tq list databases` shows Owner, Type columns
- [ ] AC-2: `tq list tables` shows Rows (Est.), Size columns
- [ ] AC-3: JSON output uses structured objects (not flat string arrays)
- [ ] AC-4: Unit tests for list.rs formatting functions

**Reference:** Sprint 46 review recommendation #2, #3, #5

**Estimated Complexity:** Medium

---

#### Feature 6: Enrich `tq show-indexes` Output

**Description:** Add two-section Primary/Secondary layout with UPI/NUPI/USI/NUSI labels.

**Acceptance Criteria:**
- [ ] AC-1: Output has separate Primary Index and Secondary Indexes sections
- [ ] AC-2: Index types labeled as UPI/NUPI/USI/NUSI
- [ ] AC-3: JSON output uses structured `{primary_index, secondary_indexes[]}` wrapper
- [ ] AC-4: Unit tests for show_indexes.rs formatting functions

**Reference:** Sprint 46 review recommendation #4, #5

**Estimated Complexity:** Medium

---

### P2 - Nice to Have

#### Feature 7: Error Message Consistency

**Description:** Ensure all error messages use `Error:` prefix consistently.

**Acceptance Criteria:**
- [ ] AC-1: All not-found messages prefixed with `Error:`
- [ ] AC-2: `<TABLE>` → `<OBJECT>` in CLI help text

**Reference:** Sprint 46 review recommendation #10, #11

**Estimated Complexity:** Low

---

### Explicitly Out of Scope

- PMON features (Issues #17-25) — deferred to future sprints
- Box-drawing column tables — visual enhancement, not functional
- `--section` batch flag for inspect — lower priority
- Pager improvements — separate concern

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working
- [ ] All P1 features implemented and tested (or explicitly deferred)
- [ ] 100% test pass rate (unit + integration)
- [ ] Zero code duplication of json_escape, csv_escape, parse_table_name, truncate_str
- [ ] Bug #36 closed
- [ ] Documentation updated
- [ ] Zero technical debt introduced

---

## GitHub Issues

### Selected for Sprint
- #36: [BUG] inspect doesn't provide the full DDL or dependencies (bug, P0)

### Deferred
- #17-25: PMON features — Not aligned with this sprint's quality consolidation theme

---

## Dependencies

### Prerequisite Work
- Sprint 46 delivered describe.rs, list.rs, show_indexes.rs — this sprint enriches them
- Sprint 45 delivered inspect.rs — this sprint fixes Bug #36

### Blockers
- None identified

---

## Risks & Mitigation

### Risk 1: REPL delegation may change output subtly
- **Probability:** Medium
- **Impact:** Medium
- **Mitigation:** Run existing REPL tests before and after. Compare output manually.

### Risk 2: Shared helper extraction may break edge cases
- **Probability:** Low
- **Impact:** Medium
- **Mitigation:** Extract function by function, run tests after each extraction.

---

## Action Items from Previous Sprint

- [ ] Extract shared helpers (json_escape, csv_escape, parse_table_name, truncate_str) — Sprint 46 review #7
- [ ] Fix truncate_str UTF-8 byte-boundary bug — Sprint 46 review #8
- [ ] REPL /describe and /list delegation — Sprint 46 review #6
- [ ] Enrich tq describe output — Sprint 46 review #1
- [ ] Enrich tq list output — Sprint 46 review #2, #3
- [ ] Enrich tq show-indexes output — Sprint 46 review #4
- [ ] Structured JSON for all 3 commands — Sprint 46 review #5
- [ ] Unit tests for describe.rs and show_indexes.rs — Sprint 46 review #9
- [ ] Error message prefix consistency — Sprint 46 review #10
- [ ] Help text naming — Sprint 46 review #11

**Reference:** `docs/sprints/sprint-46-review.md`

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Review and update specifications for enriched describe/list/show-indexes output
- Validate /inspect DDL display requirements
- Ensure UX consistency

**Deliverables:**
- Updated `docs/specifications/cli-interface.md` if needed
- UX validation of enriched output formats

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Fix Bug #36 (inspect DDL + column types for views)
- Extract shared helpers to format_helpers.rs
- Fix truncate_str UTF-8 bug
- Wire REPL delegation
- Enrich describe, list, show-indexes output
- Write unit tests for all new/modified code
- Update design docs

**Deliverables:**
- Working implementation of all P0 + P1 features
- Unit tests with 100% pass rate
- Updated design docs

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design test cases for Bug #36 fix
- Design test cases for shared helper extraction
- Execute all test suites
- Validate acceptance criteria

**Deliverables:**
- Test cases in `tests/cases/TC-047-*.md`
- Test execution report
- 100% test pass rate

---

## Files Involved

### Feature 1: Bug #36 Fix
**Source Files:**
- `src/commands/inspect.rs` — Fix DDL retrieval and column type queries

### Feature 2: Shared Helpers Extraction
**Source Files:**
- `src/commands/format_helpers.rs` — NEW: shared utility functions
- `src/commands/mod.rs` — Add format_helpers module
- `src/commands/describe.rs` — Remove duplicates, use format_helpers
- `src/commands/list.rs` — Remove duplicates, use format_helpers
- `src/commands/show_indexes.rs` — Remove duplicates, use format_helpers
- `src/commands/inspect.rs` — Remove duplicates, use format_helpers

### Feature 3: REPL Delegation
**Source Files:**
- `src/commands/repl/metacommands.rs` — Refactor /describe and /list handlers

### Features 4-6: Command Enrichment
**Source Files:**
- `src/commands/describe.rs` — Enrich output
- `src/commands/list.rs` — Enrich output
- `src/commands/show_indexes.rs` — Enrich output

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-03-23 | 1.0 | Initial sprint plan | Sprint Coordinator |
