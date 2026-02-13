# Sprint 37 Planning: External Editor Integration

**Date:** 2026-02-13
**Type:** Feature Sprint
**Version Target:** v1.18.0

## Reality Check Summary
- Reviewed sprints: 34, 35, 36
- Patterns detected: None (healthy velocity, 3 consecutive 100% test pass sprints)
- Decision: Feature Sprint
- Rationale: Framework is mature and stable. No tech debt, no stuck issues, no framework problems. Continue delivering user-facing features from backlog.

---

## Sprint Goal

Implement the `/edit` command to open the last SQL query in an external editor ($EDITOR), completing the query editing feature set alongside `/repeat` (Sprint 36).

## Sprint Theme

Query Editing - External editor integration for REPL power users

---

## Objectives

1. **Implement `/edit` command** - Open last SQL query in $EDITOR for editing and re-execution
2. **Follow-up items** - Address Sprint 36 low-priority follow-up (optional live-DB test for `/show indexes`)

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: `/edit` Command - External Editor Integration

**Description:** Allow users to open their last SQL query in an external editor ($EDITOR/$VISUAL), edit it, and automatically execute the modified query upon save and exit.

**Acceptance Criteria:**
- [ ] AC-1: `/edit` opens last SQL query in temporary `.sql` file using $EDITOR (or $VISUAL, fallback to `vi`)
- [ ] AC-2: On save and exit, the edited SQL is executed automatically
- [ ] AC-3: On exit without changes (or empty file), no execution occurs
- [ ] AC-4: Alias `\e` works identically to `/edit`
- [ ] AC-5: Tab completion includes `/edit` and `\e` in metacommand menu
- [ ] AC-6: `/help` text includes `/edit` command description
- [ ] AC-7: Error handling: clear message when no previous query exists ("No previous query to edit")
- [ ] AC-8: Error handling: clear message when $EDITOR is not set and fallback `vi` not found
- [ ] AC-9: Temp file uses `.sql` extension for editor syntax highlighting
- [ ] AC-10: Edited query stored as `last_sql` (enabling `/repeat` after `/edit`)
- [ ] AC-11: Works in full REPL mode only (not quick REPL), matching `/repeat` behavior
- [ ] AC-12: Unit tests cover all paths (happy path, no previous query, empty edit, editor error)
- [ ] AC-13: Integration tests validate CLI behavior

**Reference:** `docs/specifications/repl.md` - Query Editing section (line 3180)

**Estimated Complexity:** Medium

---

### P1 - High Priority (Should Have)

#### Feature 2: Optional Live-DB Test for `/show indexes`

**Description:** Add database-dependent test for `/show indexes` that runs when TQ_LOGON is set (Sprint 36 follow-up)

**Acceptance Criteria:**
- [ ] AC-14: `#[ignore]` test validates `/show indexes` with real Teradata connection
- [ ] AC-15: Test validates output format and column headers

**Reference:** Sprint 36 Review - Action Items

**Estimated Complexity:** Low

---

### Explicitly Out of Scope

- Profile editing commands (P1 backlog - larger scope, separate sprint)
- Second TAB accepts selection (blocked by reedline upstream)
- Variable substitution (P2 backlog - different feature area)
- Pager search (P2 backlog - pager still experimental)

---

## Success Criteria

- [ ] All P0 acceptance criteria (AC-1 through AC-13) met
- [ ] P1 criteria (AC-14, AC-15) met if database available
- [ ] 100% test pass rate (unit + integration)
- [ ] Zero regressions on existing 674 tests
- [ ] Documentation updated (specifications, user guide, design doc)
- [ ] Zero technical debt introduced
- [ ] Zero clippy warnings

---

## Dependencies

### External Dependencies
- Standard library `std::process::Command` for editor launching
- `tempfile` crate (already in project dependencies) for temp file creation

### Prerequisite Work
- `last_sql` already tracked in `ReplState` (Sprint 36)
- `/repeat` pattern established (Sprint 36) - `/edit` follows same state access pattern

### Blockers
- None identified

---

## Risks & Mitigation

### Risk 1: Editor not returning proper exit code
- **Probability:** Low
- **Impact:** Medium
- **Mitigation:** Check exit status of editor process; only execute on success (exit code 0)

### Risk 2: Cross-platform temp file handling
- **Probability:** Low
- **Impact:** Low
- **Mitigation:** Use `tempfile` crate which handles cross-platform temp directories

---

## Action Items from Sprint 36

- [x] Fix invalid TOML warning format (completed in Sprint 36)
- [ ] Add optional live-DB test for `/show indexes` (included as P1)
- [x] Consider `/edit` command (included as P0)

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Review `/edit` specification completeness
- Validate UX consistency with `/repeat` command
- Update user documentation

**Deliverables:**
- Specification review/updates for `/edit` in `docs/specifications/repl.md`
- Updated `docs/user/repl-guide.md` with `/edit` usage examples

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement `/edit` command in metacommands module
- Implement editor resolution ($VISUAL → $EDITOR → vi fallback)
- Write unit tests
- Add live-DB test for `/show indexes`

**Deliverables:**
- Working `/edit` implementation
- Unit tests with 100% pass rate
- Updated `docs/design/repl.md` if needed

### quality-validator (Sonnet)
**Responsibilities:**
- Design test cases for `/edit`
- Execute all test suites
- Validate acceptance criteria

**Deliverables:**
- Test cases in `tests/cases/TC-037-*.md`
- Test execution report
- 100% pass rate verification

---

## Files Involved

### Objective 1: `/edit` Command
**Source Files:**
- `src/commands/repl/metacommands.rs` - Add `/edit` handler and alias
- `src/commands/repl/state.rs` - Access `last_sql` (already exists)
- `src/commands/repl/executor.rs` - May need to expose execution function
- `src/commands/repl/prompt.rs` - Tab completion for `/edit`

**Test Files:**
- Unit tests in `src/commands/repl/metacommands.rs` `#[cfg(test)]`
- `tests/integration_edit_command.rs` - Integration tests

**Documentation:**
- `docs/specifications/repl.md` - Verify `/edit` spec completeness
- `docs/design/repl.md` - Add `/edit` design details
- `docs/user/repl-guide.md` - Add `/edit` usage guide

### Objective 2: Live-DB Test for `/show indexes`
**Test Files:**
- `tests/integration_show_indexes.rs` or existing interactive test file

---

## Notes

- `/edit` is the natural companion to `/repeat` (Sprint 36) - together they complete the "query editing" feature set from the specification
- The `last_sql` field in `ReplState` is already available, minimizing new state management
- The `tempfile` crate is already a project dependency
- Single-session execution target to maintain cost efficiency ($15-20 range)

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-13 | 1.0 | Initial sprint plan | Sprint Coordinator |
