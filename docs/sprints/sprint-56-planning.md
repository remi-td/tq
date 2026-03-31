# Sprint 56 Planning: Result Pagination & Sprint 55 Cleanup

**Date:** 2026-03-31
**Type:** Feature Sprint
**Status:** Planning

---

## Reality Check Summary
- Reviewed sprints: 53, 54, 55
- Patterns detected: Sprint 55 integration tests blocked by offline DB (environment issue, not code)
- Decision: Feature Sprint
- Rationale: Issue #37 (agent mode) at 5/6 parts. Completing pagination finishes the epic. Also address Sprint 55 minor tech debt.

---

## Sprint Goal

Complete Issue #37 by adding result pagination to batch commands, enabling agents to iterate through large result sets without context window overflow. Also address Sprint 55 tech debt items.

## Sprint Theme

Agent-mode pagination - completing Issue #37 part 6, plus code quality improvements.

---

## Objectives

1. Add `--page` and `--page-size` flags to the `query` command for offset-based pagination
2. Extend JSON envelope with pagination metadata (`page`, `page_size`, `has_more`, `total_rows`)
3. Apply pagination support to `search` and `list` commands
4. Address Sprint 55 tech debt: consolidate `esc()`, add dispatch tests

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Query Result Pagination

**Description:** Add client-side offset-based pagination to `tq query` and other data-returning commands. When `--page-size` is specified, results are sliced to the requested page. The JSON envelope includes pagination metadata so agents can iterate stateless.

**Design Approach:**
- Client-side pagination: fetch results, slice to page boundaries
- `--page-size N` sets page size (e.g., 50 rows per page)
- `--page P` selects which page (1-based, default 1)
- When pagination is active, JSON envelope includes: `page`, `page_size`, `has_more`, `total_rows`
- Non-JSON formats show a footer line: "Page X of Y (Z total rows)"
- Without `--page-size`, behavior is unchanged (all results returned)

**Acceptance Criteria:**
- [ ] `tq query --page-size 10 "SELECT ..."` returns first 10 rows
- [ ] `tq query --page-size 10 --page 2 "SELECT ..."` returns rows 11-20
- [ ] JSON envelope includes `page`, `page_size`, `has_more`, `total_rows` when paginated
- [ ] `has_more: true` when more pages exist, `false` on last page
- [ ] Without `--page-size`, output is unchanged (backward compatible)
- [ ] All output formats supported (table, JSON, CSV, markdown)
- [ ] Works with `--agent-safe` mode
- [ ] `--page` without `--page-size` produces an error

**Estimated Complexity:** Medium

#### Feature 2: Pagination for Search and List

**Description:** Extend pagination support to `tq search` and `tq list` commands. These already have `--limit` - add `--page` and `--page-size` alongside.

**Acceptance Criteria:**
- [ ] `tq search tables emp --page-size 10` paginates search results
- [ ] `tq list tables --page-size 20` paginates list results
- [ ] JSON envelope includes pagination metadata for these commands
- [ ] `--limit` and `--page-size` are mutually exclusive (error if both provided)

**Estimated Complexity:** Low (extends Feature 1 pattern)

### P1 - High Priority (Should Have)

#### Feature 3: Sprint 55 Tech Debt Cleanup

**Description:** Address minor technical debt identified in Sprint 55 review.

**Acceptance Criteria:**
- [ ] `esc()` markdown escape function consolidated into `format_helpers.rs`
- [ ] REPL `/search` dispatch tests added for alias routing (`"t"`, `"table"`, `"col"`, `"column"`)
- [ ] Unused `_use_color` parameter addressed in search.rs

**Estimated Complexity:** Low

### Explicitly Out of Scope

- Server-side pagination (ROW_NUMBER wrapping) - too fragile for arbitrary SQL
- Cursor/token-based pagination (stateful) - client-side offset is simpler and stateless
- Streaming large results - deferred to future sprint

---

## GitHub Issues

### Selected for Sprint
- #37: Agent mode (part 6: result pagination) - completing the epic

### Deferred
- #25: PMON Dynamic Session Monitoring
- #17: PMON Performance Summary

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working
- [ ] 100% unit test pass rate
- [ ] All acceptance criteria met
- [ ] Zero clippy warnings
- [ ] Zero new tech debt
- [ ] Backward compatibility maintained (no pagination flags = same behavior)

---

## Agent Assignments

### cli-ux-designer (Sonnet)
- Update `docs/specifications/cli-interface.md` with pagination flags
- Update `docs/specifications/output-formats.md` with paginated envelope

### rust-teradata-architect (Opus)
- Implement pagination in `src/format/json.rs` (envelope extension)
- Add `--page` and `--page-size` flags to `QueryArgs`, `SearchArgs`, `ListArgs`
- Implement pagination logic in query, search, list commands
- Consolidate `esc()` into `format_helpers.rs`
- Write unit tests

### quality-validator (Sonnet)
- Design and execute test cases for pagination
- Verify backward compatibility (no regression)

---

## Files Involved

### Feature 1: Query Pagination
- `src/cli.rs` - Add --page, --page-size to QueryArgs
- `src/format/json.rs` - Extend envelope with pagination metadata
- `src/commands/query.rs` - Apply pagination slicing
- `src/main.rs` - Pass pagination params through

### Feature 2: Search/List Pagination
- `src/cli.rs` - Add --page, --page-size to SearchArgs, ListArgs
- `src/commands/search.rs` - Apply pagination
- `src/commands/list.rs` - Apply pagination

### Feature 3: Tech Debt
- `src/commands/format_helpers.rs` - Add markdown_escape_pipe()
- `src/commands/search.rs` - Use shared esc(), address _use_color
- `src/commands/list.rs` - Use shared esc()

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-03-31 | 1.0 | Initial sprint plan | Sprint Coordinator |
