# Sprint 46 Planning: Bug Fixes & /inspect Polish

## Sprint Overview

**Sprint Goal:** Fix two user-reported bugs (#34, #35) and polish /inspect output to match specification.

**Sprint Theme:** Bug Fixes & Spec Compliance
**Date:** 2026-03-23
**Type:** Feature Sprint (bug-heavy)
**Version Target:** v1.27.0

---

## Reality Check Summary
- Reviewed sprints: 43, 44, 45
- Patterns detected: Recurring spec/implementation formatting gaps (Sprints 42-45), two new user-reported bugs
- Decision: Feature Sprint with bugs as P0
- Rationale: No systemic crisis. Bugs are user-facing and need immediate attention. /inspect formatting gaps are carry-over from Sprint 45 and are small fixes.

---

## Objectives

1. **Fix Bug #35**: `/sample dbc.tables;` produces "Error: Table 's' does not exist"
2. **Fix Bug #34**: Add missing CLI batch commands (`tq describe`, `tq list`, `tq show-indexes`)
3. **Polish /inspect output**: Align formatting with REQ-INSPECT specifications from Sprint 45

---

## Scope

### P0 - Critical (Must Have)

#### Bug #35: /sample broken due to identifier quoting (#35)

**Description:** `quote_table_reference()` wraps identifiers in double quotes, which forces case-sensitivity in Teradata. User types `dbc.tables` → SQL becomes `SELECT * FROM "dbc"."tables" SAMPLE 10` → Teradata looks for exact-case "dbc"."tables" → fails because internally stored as DBC.Tables. Secondary bug: `extract_table_name()` finds "TABLE" within "TABLES", extracting just "s" as the table name.

**Root Cause:** Two compounding bugs:
1. `quote_table_reference()` in `metacommands.rs:2765` quotes identifiers preserving lowercase. Teradata quoted identifiers are case-sensitive; unquoted are case-insensitive. User-typed lowercase names fail when quoted.
2. `extract_table_name()` in `client.rs:719` uses `find("TABLE")` without word boundaries, matching substring within "TABLES".

**Fix:**
1. Change `quote_identifier()` to uppercase identifiers before quoting (Teradata standard: identifiers stored uppercase). This preserves SQL injection protection while matching Teradata behavior.
2. Fix `extract_table_name()` to use word-boundary matching for FROM/INTO/UPDATE/TABLE keywords.

**Acceptance Criteria:**
- [ ] AC-1: `/sample dbc.tables;` works correctly in REPL
- [ ] AC-2: `/sample dbc.tables` (no semicolon) works correctly
- [ ] AC-3: `/peek dbc.tables;` works correctly
- [ ] AC-4: `tq sample dbc.tables` works in batch mode
- [ ] AC-5: Case-insensitive table names work: `dbc.TablesV`, `DBC.TABLESV`, `dbc.tablesv`
- [ ] AC-6: `extract_table_name` correctly extracts table from `SELECT * FROM "DBC"."TABLES" SAMPLE 10`
- [ ] AC-7: Unit tests for `quote_identifier()` with uppercase behavior
- [ ] AC-8: Unit tests for `extract_table_name()` word boundary matching

**Reference:** Issue #35
**Estimated Complexity:** Medium

---

#### Bug #34: Helper commands missing from CLI (#34)

**Description:** Several REPL metacommands lack CLI batch-mode equivalents. Users expect `tq describe`, `tq list`, `tq show-indexes` to work from the command line.

**Commands to add:**
1. `tq describe <table>` — Show table structure (columns, types, nullable)
2. `tq list databases|tables|views [pattern]` — List database objects
3. `tq show-indexes <table>` — Show table index information

**Commands NOT added (REPL-specific state required):**
- `/repeat` — requires last query state
- `/edit` — requires editor + last query state
- `/logon` — REPL-only connection switching
- `/params` — REPL-only param management

**Acceptance Criteria:**
- [ ] AC-1: `tq describe <table>` outputs column info in table/CSV/JSON formats
- [ ] AC-2: `tq list databases` outputs database list
- [ ] AC-3: `tq list tables [pattern]` outputs table list with optional glob filter
- [ ] AC-4: `tq list views` outputs view list
- [ ] AC-5: `tq show-indexes <table>` outputs index information
- [ ] AC-6: All new commands support `--format table|csv|json` flag
- [ ] AC-7: All new commands appear in `tq --help` and `tq help`
- [ ] AC-8: Unit tests for argument parsing

**Reference:** Issue #34
**Estimated Complexity:** Medium

---

### P1 - High Priority (Should Have)

#### /inspect formatting compliance (Sprint 45 deferred)

**Description:** Align /inspect output with REQ-INSPECT specifications. All changes are presentation-layer only — no architectural changes needed.

**Acceptance Criteria:**
- [ ] AC-1: Section headers use `── Section Name ──` format (not `=== Section Name ===`)
- [ ] AC-2: Default column shows `-` instead of empty string
- [ ] AC-3: Column count footer: `N columns` displayed after column table
- [ ] AC-4: Skew interpretation hint: `(low)`, `(moderate)`, `(high)` after skew percentage
- [ ] AC-5: `O` TableKind mapped to "Table (NoPI)" instead of "Table"
- [ ] AC-6: Error message uses `Error:` prefix for not-found
- [ ] AC-7: Usage prompt shows examples when `/inspect` called with no argument
- [ ] AC-8: Direct row indexing fixed in `inspect.rs:649-660` (panic risk)

**Reference:** Sprint 45 review, items 1-7 and 12
**Estimated Complexity:** Low

---

### Explicitly Out of Scope

- `/inspect --section` batch flag (deferred to backlog)
- Box-drawing column table in /inspect (backlog)
- Dependencies section for views/macros (backlog)
- JSON output structure matching spec (backlog)
- Graceful degradation unit tests (backlog)

---

## GitHub Issues

### Selected for Sprint
- #35: [BUG] sample command not working (bug, P0)
- #34: [BUG] helper commands should be available as tq CLI command (bug, P0)

### Deferred
- #25: Dynamic Session Monitoring — requires async/TUI architecture
- #24: Query Drill-Down — partially complete, remaining items are P2
- #17-#23: PMON features — ongoing backlog

---

## Action Items from Previous Sprint

- [x] Fix section headers `===` → `──` (Sprint 45 item 1)
- [x] Default column `-` (Sprint 45 item 2)
- [x] Column count footer (Sprint 45 item 3)
- [x] Skew interpretation hint (Sprint 45 item 4)
- [x] `O` → "Table (NoPI)" (Sprint 45 item 5)
- [x] Fix direct row indexing (Sprint 45 item 12)

**Reference:** `docs/sprints/sprint-45-review.md` Section 7

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Update specifications for `tq describe`, `tq list`, `tq show-indexes` CLI commands
- Review /inspect formatting changes against spec
- Validate UX consistency of new CLI commands

### rust-teradata-architect (Opus)
**Responsibilities:**
- Fix `quote_identifier()` uppercase behavior
- Fix `extract_table_name()` word boundaries
- Implement `tq describe`, `tq list`, `tq show-indexes` commands
- Fix /inspect formatting (section headers, defaults, footer, skew, TableKind)
- Fix direct row indexing panic risk
- Write unit tests for all changes

### quality-validator (Sonnet)
**Responsibilities:**
- Design test cases for bugs #34, #35
- Execute full test suite
- Validate acceptance criteria

---

## Files Involved

### Bug #35: Identifier Quoting Fix
- `src/sql/identifiers.rs` — Fix `quote_identifier()` to uppercase
- `src/db/client.rs` — Fix `extract_table_name()` word boundaries
- Unit tests in both files

### Bug #34: CLI Commands
- `src/cli.rs` — Add Describe, List, ShowIndexes commands to Command enum
- `src/main.rs` — Add command dispatch
- `src/commands/describe.rs` — New: batch describe implementation
- `src/commands/list.rs` — New: batch list implementation
- `src/commands/show_indexes.rs` — New: batch show-indexes implementation
- `src/lib.rs` — Re-exports
- `docs/specifications/cli-interface.md` — New command specs
- `docs/design/cli-interface.md` — Design updates

### /inspect Polish
- `src/commands/inspect.rs` — All formatting changes

---

## Risks & Mitigation

### Risk 1: Identifier quoting change affects other commands
- **Probability:** Low
- **Impact:** Medium
- **Mitigation:** All commands using `quote_identifier()` benefit from the fix. Uppercase is Teradata's native storage format.

### Risk 2: Session budget exceeded
- **Probability:** Low
- **Impact:** Medium
- **Mitigation:** /inspect fixes are small (5-15 min each). Bug fixes have clear root causes. If needed, defer P1 items.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-03-23 | 1.0 | Initial sprint plan | Sprint Coordinator |
