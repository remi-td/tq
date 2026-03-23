---
sprint: 48
start_date: 2026-03-23
target_completion: 2026-03-23
status: Planning
---

# Sprint 48 Planning: Query Layer Consolidation & Spec Alignment

## Sprint Overview

**Sprint Goal:** Eliminate query-level duplication, fix JSON API types, close remaining spec gaps, and add missing unit tests.

**Sprint Theme:** Complete the quality consolidation started in Sprint 47. Close the spec/implementation gap definitively.

---

## Reality Check Summary

- Reviewed sprint: 47
- Patterns detected:
  - **Query duplication dominant**: query_indexes 3x, query_columns 2x, resolve_database 2x, format_size 2x
  - **Spec gap persists (6th sprint)**: JSON nullable as string, list views sparse, edge case messages missing
  - **UTF-8 bug in summarize_error**: Same class as the truncate_str bug fixed in Sprint 47
  - **Test gap**: TC-047-001 DDL tests specified but not implemented
- Decision: **Feature Sprint** — No crisis. Complete the consolidation work.
- Rationale: Sprint 47 reduced formatter duplication; Sprint 48 finishes the job with query layer.

---

## Objectives

1. **Extract shared query layer**: Consolidate query_indexes, query_columns, resolve_database into shared module
2. **Fix JSON API types**: nullable as boolean, default as null, structured integers for rows/size
3. **Fix remaining spec gaps**: list views enrichment, edge case messages, list databases type labels
4. **Fix bugs**: summarize_error UTF-8, show-indexes TABLE→OBJECT
5. **Implement missing tests**: TC-047-001 DDL tests, writer-injection rendering tests

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Extract Shared Query Layer

**Description:** Consolidate duplicated query functions into a shared module (extend format_helpers.rs or create query_helpers.rs).

**Acceptance Criteria:**
- [ ] AC-1: `query_indexes()` exists once, used by inspect.rs, describe.rs, show_indexes.rs
- [ ] AC-2: `query_columns()` exists once, used by inspect.rs, describe.rs
- [ ] AC-3: `resolve_database()` exists once, used by inspect.rs, describe.rs
- [ ] AC-4: `format_size()` exists once with precision parameter, used by inspect.rs, list.rs
- [ ] AC-5: All existing tests pass after consolidation
- [ ] AC-6: Shared IndexGroup and ColumnInfo types defined once

**Estimated Complexity:** Medium

---

#### Feature 2: Fix JSON API Types

**Description:** Fix JSON output to use proper types per specification.

**Acceptance Criteria:**
- [ ] AC-1: describe JSON: nullable as boolean (true/false), not string "YES"/"NO"
- [ ] AC-2: describe JSON: default as null (not "-") when absent
- [ ] AC-3: list tables JSON: estimated_rows as integer, size_bytes as integer
- [ ] AC-4: list databases JSON: key "database" not "name"
- [ ] AC-5: Unit tests for all JSON type changes

**Estimated Complexity:** Low-Medium

---

#### Feature 3: Fix Bugs & Missing Fixes

**Description:** Address specific bugs and incomplete fixes from Sprint 47.

**Acceptance Criteria:**
- [ ] AC-1: Fix summarize_error UTF-8 byte-boundary bug (inspect.rs:778)
- [ ] AC-2: Fix show-indexes `<TABLE>` → `<OBJECT>` in cli.rs
- [ ] AC-3: Fix list databases type: "System"/"User" instead of "Database"/"User"
- [ ] AC-4: Fix list.rs unknown subcommand missing Error: prefix
- [ ] AC-5: Rename DescribeArgs.table field to .object

**Estimated Complexity:** Low

---

### P1 - High Priority (Should Have)

#### Feature 4: Enrich list views & Edge Cases

**Description:** Add Owner column to list views. Add missing edge case messages.

**Acceptance Criteria:**
- [ ] AC-1: `tq list views` shows Owner column (from DBC.TablesV CreatorName)
- [ ] AC-2: "No indexes defined." message for tables without indexes in describe
- [ ] AC-3: "No Primary Index (NoPI)" for NoPI tables in show-indexes
- [ ] AC-4: "No secondary indexes." when none exist in show-indexes
- [ ] AC-5: Add Rows (Est.) to describe object header for tables

**Estimated Complexity:** Medium

---

#### Feature 5: Missing Unit Tests

**Description:** Implement tests that were specified but not delivered in Sprint 47.

**Acceptance Criteria:**
- [ ] AC-1: 6 DDL unit tests from TC-047-001 implemented in inspect.rs
- [ ] AC-2: Writer-injection tests for describe_table rendering
- [ ] AC-3: Writer-injection tests for show_indexes_table rendering
- [ ] AC-4: Writer-injection tests for list_databases rendering
- [ ] AC-5: column_type_case_sql test verifies all 21 WHEN branches

**Estimated Complexity:** Medium

---

#### Feature 6: Spec Canonicalization

**Description:** Update specifications to match implementation where implementation decisions are sound.

**Acceptance Criteria:**
- [ ] AC-1: Spec updated to use `──` section headers (not box-drawing)
- [ ] AC-2: Spec updated for inline index format in describe
- [ ] AC-3: Spec updated for conditional Comment column
- [ ] AC-4: Glob vs LIKE decision documented (keep glob, update spec)

**Estimated Complexity:** Low

---

### Explicitly Out of Scope

- PMON features (Issues #17-25) — Not enough session budget after consolidation work
- Box-drawing table borders — Deferred permanently (── headers are better)
- serde_json migration — Nice-to-have, not urgent

---

## GitHub Issues

### Selected for Sprint
- No new GitHub issues — this is a consolidation sprint addressing Sprint 47 review items

### Deferred
- #17-25: PMON features

---

## Agent Assignments

### rust-teradata-architect (Opus)
- Extract shared query layer (query_helpers.rs)
- Fix JSON API types
- Fix bugs (summarize_error, TABLE→OBJECT, list type labels)
- Add Rows (Est.) to describe, edge case messages
- Implement missing unit tests

### cli-ux-designer (Sonnet)
- Canonicalize specs to match implementation
- Update user docs for JSON type changes
- Validate list views enrichment

### quality-validator (Sonnet)
- Execute all tests
- Validate shared query extraction
- Verify JSON output types

---

## Files Involved

### Feature 1: Query Layer Extraction
- `src/commands/query_helpers.rs` — NEW: shared query functions
- `src/commands/format_helpers.rs` — Add format_size with precision param
- `src/commands/inspect.rs` — Remove query duplicates
- `src/commands/describe.rs` — Remove query duplicates
- `src/commands/show_indexes.rs` — Remove query duplicates

### Feature 2-3: JSON Fixes & Bugs
- `src/commands/describe.rs` — JSON nullable/default types
- `src/commands/list.rs` — JSON integer types, type labels
- `src/cli.rs` — TABLE→OBJECT
- `src/commands/inspect.rs` — summarize_error fix

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-03-23 | 1.0 | Initial sprint plan | Sprint Coordinator |
