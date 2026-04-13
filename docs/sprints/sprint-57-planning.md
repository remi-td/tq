# Sprint 57 Planning: Search Quality & View Search

## Sprint Overview

**Sprint Goal:** Clean up technical debt from Sprint 55-56 search implementation and extend search to views

**Sprint Theme:** Code Quality + Search Enhancement

**Date:** 2026-04-06
**Type:** Feature Sprint

## Reality Check Summary
- Reviewed sprints: 54, 55, 56
- Patterns detected: None (healthy velocity, 3 consecutive successful feature sprints)
- Decision: Feature Sprint
- Rationale: All 108 tracked features complete, Issue #37 epic fully delivered. No stuck issues or accumulating debt. P1 follow-ups from S55-S56 are small code quality items. Combining with a small feature for productive sprint.

---

## Objectives

1. Replace hand-rolled JSON in search.rs with serde_json for type safety and consistency
2. Promote hard-coded magic number to named constant
3. Add ORDER BY stability warning to pagination documentation
4. Add `tq search views <keyword>` subcommand and REPL `/search views` metacommand

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Serde JSON in search.rs

**Description:** Replace manual JSON string building in `render_table_search_json_with_pagination` and `render_column_search_json_with_pagination` with serde_json serialization, consistent with format/json.rs approach.

**Acceptance Criteria:**
- [ ] Both JSON render functions use serde_json instead of manual write!() calls
- [ ] JSON output is byte-identical (or semantically equivalent) to current output
- [ ] All existing search JSON tests pass
- [ ] No new clippy warnings

**Reference:** Sprint 56 review follow-up P1

**Estimated Complexity:** Low

#### Feature 2: Named constant for max search fetch

**Description:** Replace hard-coded `100000` sentinel in search.rs with `MAX_SEARCH_FETCH` named constant.

**Acceptance Criteria:**
- [ ] Named constant `MAX_SEARCH_FETCH` defined and used in both table and column search
- [ ] Existing behavior unchanged

**Reference:** Sprint 56 review follow-up P2

**Estimated Complexity:** Low

---

### P1 - High Priority (Should Have)

#### Feature 3: ORDER BY stability warning

**Description:** Add documentation warning that pagination without deterministic ORDER BY produces unstable pages.

**Acceptance Criteria:**
- [ ] Warning added to pagination section of CLI interface specification
- [ ] Warning added to batch mode documentation

**Reference:** Sprint 56 review follow-up P1

**Estimated Complexity:** Low

#### Feature 4: Search Views subcommand

**Description:** Add `tq search views <keyword>` to search for views by name pattern across databases, complementing existing `tq search tables` and `tq search columns`. Also add REPL `/search views` metacommand with `in <db>` scoping.

**Acceptance Criteria:**
- [ ] `tq search views <keyword>` works in batch mode with all output formats (table, JSON, CSV, markdown)
- [ ] `/search views <keyword>` works in REPL with `in <db>` scoping
- [ ] JSON output follows standard agent-mode envelope (`ok`, `row_count`, `data`)
- [ ] `--limit` and `--database` flags work
- [ ] Pagination support (--page, --page-size)
- [ ] Tab completion for `/search views`
- [ ] Unit tests for all render functions

**Reference:** Sprint 55 review follow-up P2, `docs/specifications/cli-interface.md`

**Estimated Complexity:** Medium (follows existing search tables pattern)

---

### Explicitly Out of Scope

- Search keyword highlighting (P3, deferred to backlog)
- REPL dispatch alias tests (low value vs effort)
- Config validation command (separate sprint)
- PMON features (#17, #21, #22, #23, #25)

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working
- [ ] All P1 features implemented and tested
- [ ] 100% test pass rate (unit tests)
- [ ] Zero clippy warnings
- [ ] Documentation updated

---

## GitHub Issues

### Selected for Sprint
- None directly (tech debt follow-ups + backlog feature)

### Closed This Sprint
- #41: Inspect column comments (already implemented, closed during planning)

### Deferred
- #17: PMON Performance Summary (Sprint 58)
- #25: Dynamic Session Monitoring (future)

---

## Action Items from Previous Sprint

- [x] Close Issue #41 (already implemented)
- [ ] Replace hand-rolled JSON in search.rs with serde (Sprint 56 P1)
- [ ] Promote 100000 to named constant (Sprint 56 P2)
- [ ] Add ORDER BY stability warning (Sprint 56 P1)
- [ ] Add search views subcommand (Sprint 55 P2)

**Reference:** `docs/sprints/sprint-56-review.md`, `docs/sprints/sprint-55-review.md`

---

## Agent Assignments

### cli-ux-designer (Sonnet)
- Update `docs/specifications/cli-interface.md` with search views specification
- Add ORDER BY stability warning to pagination docs
- Ensure UX consistency with existing search commands

### rust-teradata-architect (Opus)
- Refactor search.rs JSON rendering to use serde_json
- Add MAX_SEARCH_FETCH constant
- Implement search views command (batch + REPL)
- Write unit tests
- Update design docs

### quality-validator (Sonnet)
- Design test cases for search views
- Execute all test suites
- Validate JSON output consistency after serde refactor

---

## Files Involved

### Objective 1-2: Search.rs refactoring
- `src/commands/search.rs` - Serde refactor + named constant

### Objective 3: Documentation
- `docs/specifications/cli-interface.md` - Pagination ORDER BY warning
- `docs/specifications/batch-mode.md` - Pagination ORDER BY warning

### Objective 4: Search Views
- `src/commands/search.rs` - New search_views function + renderers
- `src/cli.rs` - CLI argument for `search views`
- `src/repl.rs` or equivalent - REPL /search views dispatch
- `docs/specifications/cli-interface.md` - Search views spec

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-04-06 | 1.0 | Initial sprint plan | Sprint Coordinator |
