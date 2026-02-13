---
sprint: 36
start_date: 2026-02-13
status: Planning
---

# Sprint 36 Planning: Help Text Update + REPL Enhancements

## Sprint Overview

**Sprint Goal:** Polish Sprint 35's configuration feature with help text and UX improvements, then deliver two practical REPL enhancements: `/repeat` and `/show indexes`.

**Sprint Theme:** Configuration Polish + REPL Productivity

---

## Reality Check Summary

- Reviewed sprints: 33, 34, 35
- Patterns detected: None (healthy velocity, 100% test pass rate across all 3 sprints)
- Decision: Feature Sprint
- Rationale: Framework is MATURE and EFFICIENT. No stuck issues, no accumulating debt, no framework problems. Cost per objective decreasing ($10.47 → $5.09 → $4.95). Clean foundation from Sprint 34-35.

---

## GitHub Issues

### Addressed This Sprint
- **#15**: [BUG] Sprint review process - Sprint 35 retrospective reviewed wrong sprints (7, 8, 9 instead of 32, 33, 34). Process bug in sprint-reviewer skill. Will close with comment explaining this was a one-time Phase 5 coordinator error, now addressed.

### Deferred
- None

---

## Objectives

1. **Complete Sprint 35 Configuration Polish** - Address all 4 recommended follow-up items from Sprint 35 review
2. **Implement `/repeat` Command** - Re-execute last SQL query in REPL mode
3. **Implement `/show indexes <table>` Command** - Display table index information for schema inspection

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Configuration Help Text & UX Polish

**Description:** Complete the 4 recommended follow-up items from Sprint 35's review to polish the project configuration feature.

**Sub-features:**

**1a. Update `tq help config` with project config section**
- Add project config file section (`.tq.toml`)
- Show 5-level precedence hierarchy (defaults → user config → project config → env vars → CLI)
- Mention `.tq.toml.example` for getting started

**1b. Show project config path in `tq profiles` output**
- Display resolved `.tq.toml` path in `tq profiles` header
- Format: "Project config: /path/to/project/.tq.toml"
- When no project config: omit line (no noise)

**1c. Add project config mention to empty state**
- When `tq profiles` shows no profiles, mention project config option
- Message: "Tip: Create .tq.toml in your project root for team-shared profiles"

**1d. Warn on invalid project config to stderr**
- When `.tq.toml` has invalid TOML syntax, show warning to stderr
- Format: "Warning: Invalid project config at /path/.tq.toml: <parse error>"
- Continue operation (non-blocking, graceful degradation)

**Acceptance Criteria:**
- [ ] AC-1: `tq help config` includes project config section with `.tq.toml` description
- [ ] AC-2: `tq help config` shows 5-level precedence hierarchy
- [ ] AC-3: `tq profiles` shows project config file path when present
- [ ] AC-4: `tq profiles` shows tip about project config when no profiles exist
- [ ] AC-5: Invalid `.tq.toml` produces stderr warning with file path and error details
- [ ] AC-6: All existing tests pass (zero regressions)
- [ ] AC-7: New unit + integration tests for all sub-features

**Reference:** Sprint 35 review recommendations, `docs/specifications/configuration.md`

**Estimated Complexity:** Low-Medium

---

### P1 - High Priority (Should Have)

#### Feature 2: `/repeat` Command - Re-execute Last Query

**Description:** Add `/repeat` metacommand to re-execute the most recently executed SQL query. The `last_sql` field already exists in `ReplState`, making this straightforward.

**Acceptance Criteria:**
- [ ] AC-8: `/repeat` re-executes the last SQL statement
- [ ] AC-9: When no previous query exists, show clear message: "No previous query to repeat"
- [ ] AC-10: `/repeat` works after any SQL statement (SELECT, INSERT, DDL, etc.)
- [ ] AC-11: Tab completion includes `/repeat` with description "Re-execute last query"
- [ ] AC-12: `/help` output includes `/repeat` command
- [ ] AC-13: Short alias `\r` works (following psql convention)
- [ ] AC-14: Unit tests validate all behaviors

**Reference:** `docs/specifications/repl.md` (Query Editing section)

**Estimated Complexity:** Low

---

#### Feature 3: `/show indexes <table>` Command

**Description:** Add schema inspection command to display index information for a table. Queries Teradata system catalog (DBC.IndicesV) to show index names, types, and columns.

**Acceptance Criteria:**
- [ ] AC-15: `/show indexes <table>` displays index information from DBC.IndicesV
- [ ] AC-16: Qualified name support: `/show indexes database.table`
- [ ] AC-17: Short alias `\di` works
- [ ] AC-18: Table output shows: IndexName, IndexType, ColumnName, ColumnPosition
- [ ] AC-19: Error handling for non-existent table with clear message
- [ ] AC-20: Error handling for permission denied with guidance
- [ ] AC-21: Tab completion includes `/show indexes` with description
- [ ] AC-22: `/help` output includes `/show indexes` command
- [ ] AC-23: Unit tests for SQL generation and argument parsing
- [ ] AC-24: Integration tests for CLI behavior

**Reference:** `docs/specifications/repl.md` (Schema Inspection Commands)

**Estimated Complexity:** Medium

---

### Explicitly Out of Scope

- `/edit` command (external editor integration) - requires more design work
- Profile editing commands (`tq profile add/edit/delete`) - deferred to future sprint
- Batch mode `tq indexes` command - can be added later if needed
- Pager improvements - still experimental, separate concern

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working
- [ ] All P1 features implemented and tested
- [ ] 100% test pass rate (unit + integration)
- [ ] All 24 acceptance criteria met
- [ ] Documentation updated (specifications, design, user guides)
- [ ] Zero technical debt introduced
- [ ] Zero regressions
- [ ] GitHub Issue #15 closed

---

## Dependencies

### External Dependencies
- Teradata system catalog `DBC.IndicesV` for `/show indexes` query
- No new crate dependencies expected

### Prerequisite Work
- Sprint 35 complete (project config foundation) ✅

### Blockers
- None identified

---

## Risks & Mitigation

### Risk 1: DBC.IndicesV Schema Uncertainty
- **Probability:** Low
- **Impact:** Medium (affects `/show indexes` feature)
- **Mitigation:** Research Teradata catalog views during design phase. Fall back to DBC.Indices if IndicesV unavailable.

### Risk 2: Tab Completion Conflicts with `/show` Prefix
- **Probability:** Low
- **Impact:** Low
- **Mitigation:** `/show indexes` is a two-word command like `/list tables`. Existing pattern already handles this.

---

## Action Items from Previous Sprint

- [ ] Update `tq help config` with project config (Sprint 35 recommendation #1)
- [ ] Show project config path in `tq profiles` (Sprint 35 recommendation #2)
- [ ] Add project config mention to empty state (Sprint 35 recommendation #3)
- [ ] Warn on invalid project config (Sprint 35 recommendation #4)

**Reference:** `docs/sprints/sprint-35-review.md` Section 14

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Update specifications for `/repeat` and `/show indexes` if needed
- Review help text changes for UX quality
- Validate config polish changes meet UX standards

**Deliverables:**
- Updated `docs/specifications/repl.md` if needed
- UX review of all changes

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement all 3 features (config polish, `/repeat`, `/show indexes`)
- Write unit tests for all new code
- Update `docs/design/repl.md` and `docs/design/configuration.md`

**Deliverables:**
- Working implementation of all features
- Unit tests with 100% pass rate
- Updated design documentation

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design test cases for all features
- Execute all test suites
- Validate acceptance criteria

**Deliverables:**
- Test cases in `tests/cases/TC-036-*.md`
- Test execution report
- 100% test pass rate validation

---

## Notes

- `/repeat` is very low effort since `last_sql` already tracked in ReplState
- Config help text update is the most impactful user-facing improvement (Sprint 35 gap)
- `/show indexes` follows established `/describe` pattern, reducing implementation risk
- GitHub Issue #15 is a process issue, not a code issue - will be closed with explanation

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-13 | 1.0 | Initial sprint plan | Sprint Coordinator |
