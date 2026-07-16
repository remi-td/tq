---
sprint: 75
start_date: 2026-07-16
target_completion: 2026-07-16
status: Planning
---

# Sprint 75 Planning: Bug Fixes and Feature Cleanup

## Reality Check Summary
- **Reviewed Sprints:** Sprint 74, 73, 72.
- **Patterns Detected:** None. Healthy delivery velocity, all tests passing.
- **Decision:** Feature Sprint.
- **Rationale:** Focus on resolving the user's reported bugs in priority and closing Issue #50.

---

## Sprint Overview

**Sprint Goal:** Resolve outstanding bugs with `show indexes` failing on no-index tables and `SHOW` statements truncating DDL output in REPL, and close the implemented `fastload` feature issue.

**Sprint Theme:** Robust Metadata Reporting & Output Presentation

---

## Objectives

1. Fix `show indexes` command returning an error for existing tables with no indexes (Issue #53).
2. Fix `SHOW` SQL statement output truncation in REPL mode (Issue #52).
3. Review and close the Fastload implementation feature issue (Issue #50).

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Fix `show indexes` for tables with no indexes (#53)

**Description:** Verify table/view existence using `query_object_header` before returning "No indexes found" error. Show correct `No Primary Index` metadata for tables that do exist but have no indexes.

**Acceptance Criteria:**
- [ ] If a table does not exist, return/print standard table not found error and suggestions.
- [ ] If a table exists but has no indexes, show:
  ```
  No Primary Index (NoPI)
  No secondary indexes.
  ```
  Instead of an error message.
- [ ] Working correctly in Table, JSON, CSV, and Markdown formats.

**Estimated Complexity:** Low

---

#### Feature 2: Fix `SHOW` statement output formatting (#52)

**Description:** Display the entire multi-line DDL output for `SHOW` SQL statements in Table/REPL and Markdown modes, bypassing typical single-cell truncation.

**Acceptance Criteria:**
- [ ] Detect if the query result represents a `SHOW` command (1 column named `Request Text` / `RequestText`).
- [ ] For Table format (REPL/default): Output the DDL text directly, preserving newlines, and no border.
- [ ] For Markdown format: Output the DDL text wrapped in a ` ```sql ` code block.
- [ ] Single-cell column width limit (100) is bypassed for SHOW results.

**Estimated Complexity:** Low

---

### P1 - High Priority (Should Have)

#### Feature 3: Close Issue #50

**Description:** Close GitHub Issue #50 with implementation details of `fastload` and `fastexport` features delivered in Sprint 73 & 74.

---

### Success Criteria

- [ ] All P0 features implemented, tested, and working as specified
- [ ] 100% test pass rate (unit + integration tests)
- [ ] All acceptance criteria met for delivered features
- [ ] Documentation updated to reflect changes
- [ ] GitHub Issue #52 and #53 closed
- [ ] GitHub Issue #50 closed

---

## Files Involved

### Objective 1: show indexes fix
**Source Files:**
- `src/commands/show_indexes.rs`

### Objective 2: SHOW statement output formatting
**Source Files:**
- `src/format/mod.rs`
- `src/format/table.rs`
- `src/format/markdown.rs`

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-07-16 | 1.0 | Initial sprint plan | Sprint Coordinator |
