# Sprint 38 Planning: PMON Foundation - System Config & Lock Monitoring

**Date:** 2026-02-24
**Type:** Feature Sprint
**Status:** Planning

---

## Reality Check Summary

- **Reviewed sprints:** 35, 36, 37
- **Patterns detected:** None (healthy velocity, zero tech debt, 100% test pass rates)
- **Decision:** Feature Sprint
- **Rationale:** Three consecutive excellent sprints (9.2, 9.0, 9.3). Zero technical debt. Framework is mature. New PMON feature requests (10 GitHub issues) provide high-value DBA functionality to build upon the existing `/sessions` foundation.

---

## Sprint Overview

**Sprint Goal:** Establish the PMON (Performance Monitor) foundation by adding system configuration display and lock/blocking analysis commands.

**Sprint Theme:** DBA Monitoring Foundation - First two PMON features that give DBAs essential system visibility.

---

## Objectives

1. **System Configuration Summary** (Issue #16) - Display Teradata system topology (nodes, AMPs, PEs, version) via `/sysconfig` REPL command and `tq sysconfig` batch command
2. **Session Blocking & Lock Information** (Issue #18) - Display lock contention and blocking chains via `/locks` REPL command and `tq locks` batch command

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: System Configuration Summary (#16)

**Description:** New `/sysconfig` REPL metacommand and `tq sysconfig` batch command that queries DBC system views to display system topology, version, and resource counts in a compact summary.

**Acceptance Criteria:**
- [ ] AC-1: `/sysconfig` command queries DBC.DBCInfoV for system version and release info
- [ ] AC-2: Command displays total AMP count via HASHAMP()+1
- [ ] AC-3: Command displays system version, node count, and AMP/PE topology
- [ ] AC-4: `tq sysconfig` batch mode command with table/csv/json output formats
- [ ] AC-5: Tab completion includes `/sysconfig` in metacommand menu
- [ ] AC-6: Help text documents the command in both compact and extended formats
- [ ] AC-7: Error handling for privilege errors with actionable guidance
- [ ] AC-8: Unit tests for SQL generation, output formatting, and parsing logic
- [ ] AC-9: `/sc` short alias available

**Reference:** `docs/specifications/admin-user-stories.md` Section 1, Issue #16

**Estimated Complexity:** Medium

---

#### Feature 2: Session Blocking & Lock Information (#18)

**Description:** New `/locks` REPL metacommand and `tq locks` batch command that displays current lock contention, blocking sessions, and lock details to help DBAs diagnose contention issues.

**Acceptance Criteria:**
- [ ] AC-1: `/locks` command queries DBC.LockInfoV (or equivalent) for current lock information
- [ ] AC-2: Display shows locked object, lock type (READ/WRITE/EXCLUSIVE), locking session, and waiting sessions
- [ ] AC-3: Blocking chain identification - which sessions block which
- [ ] AC-4: `tq locks` batch mode command with table/csv/json output formats
- [ ] AC-5: Tab completion includes `/locks` in metacommand menu
- [ ] AC-6: Help text documents the command in both compact and extended formats
- [ ] AC-7: Error handling for privilege errors with actionable guidance
- [ ] AC-8: Unit tests for SQL generation, output formatting, lock type mapping, and parsing
- [ ] AC-9: `/lk` short alias available

**Reference:** `docs/specifications/admin-user-stories.md` Section 3, Issue #18

**Estimated Complexity:** Medium-High

---

### Explicitly Out of Scope

- Real-time auto-refresh / dynamic monitoring (Issue #25) - requires async architecture
- Graphical displays (Issues #21, #22) - requires TUI framework
- Session control functions (Issue #20) - requires careful safety design
- Alerting and thresholds (Issue #23) - future sprint
- Performance resource monitoring (Issue #17) - depends on ResUsage being enabled
- Session history (Issue #19) - future sprint
- Query drill-down (Issue #24) - future sprint

---

## GitHub Issues

### Selected for Sprint
- #16: [FEATURE] PMON: System Configuration Summary (priority-high, enhancement)
- #18: [FEATURE] PMON: Session Blocking and Lock Information (priority-high, enhancement)

### Deferred
- #17: Performance Summary - Depends on ResUsage collection being enabled
- #19: Session History - Lower priority than real-time monitoring
- #20: Session Control Functions - Requires safety review for destructive operations
- #21-#22: Graphical Displays - Requires TUI framework selection
- #23: Alerting - Depends on monitoring foundation
- #24: Query Drill-Down - Depends on session monitoring
- #25: Dynamic Monitoring - Requires async/refresh architecture

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working
- [ ] 100% test pass rate (unit + integration)
- [ ] All acceptance criteria met
- [ ] Documentation updated (specifications, design, user guide)
- [ ] Zero technical debt introduced
- [ ] Code quality meets project standards
- [ ] Single-session execution

---

## Dependencies

### External Dependencies
- DBC.DBCInfoV system view (standard Teradata)
- DBC.LockInfoV or MonitorSession lock data
- SELECT privileges on DBC system views

### Prerequisite Work
- Existing `/sessions` command (Sprint 26) establishes the pattern
- `comfy_table` crate already available for table formatting
- `teradata-monitor` skill provides reference SQL queries

---

## Risks & Mitigation

### Risk 1: Lock View Availability
- **Probability:** Low
- **Impact:** Medium
- **Mitigation:** Use multiple query strategies (DBC.LockInfoV primary, fallback queries). Design code to handle missing views gracefully.

### Risk 2: Session Budget Overrun
- **Probability:** Low
- **Impact:** Medium
- **Mitigation:** Both features follow the established sessions.rs pattern. Limit scope to query + display (no real-time refresh).

---

## Action Items from Previous Sprint

- [ ] Track PTY test infrastructure limitation (Sprint 37, Medium)
- [ ] Consider richer error messages for /edit (Sprint 37, Low) - DEFERRED
- [ ] Add $VISUAL to help text (Sprint 37, Low) - DEFERRED

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Design specifications for `/sysconfig` and `/locks` commands
- Update `docs/specifications/repl.md` with new command requirements
- Ensure UX consistency with existing monitoring commands

**Deliverables:**
- Updated REPL specification with sysconfig and locks sections
- CLI interface specification updates for batch mode commands

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement `src/commands/sysconfig.rs` module
- Implement `src/commands/locks.rs` module
- Add CLI argument structures to `src/cli.rs`
- Wire commands in `src/main.rs`
- Add REPL metacommand handlers in `src/commands/repl/metacommands.rs`
- Update tab completion in `src/commands/repl/metadata_completer.rs`
- Write unit tests for all new code
- Update `docs/design/repl.md` with monitoring section

**Deliverables:**
- Working implementation of both features
- Unit tests with 100% pass rate
- Design documentation updates

### quality-validator (Sonnet)
**Responsibilities:**
- Design test cases for both features
- Execute all test suites
- Validate acceptance criteria

**Deliverables:**
- Test cases in `tests/cases/TC-038-*.md`
- Test execution report with proof of execution
- 100% test pass rate validation

---

## Files Involved

### Feature 1: System Configuration Summary
**Source Files:**
- `src/commands/sysconfig.rs` (NEW) - Sysconfig command implementation
- `src/commands/mod.rs` - Register new module
- `src/cli.rs` - Add SysconfigArgs and Command variant
- `src/main.rs` - Wire sysconfig command
- `src/commands/repl/metacommands.rs` - Add /sysconfig handler
- `src/commands/repl/metadata_completer.rs` - Tab completion

**Documentation:**
- `docs/specifications/repl.md` - Specification for /sysconfig
- `docs/design/repl.md` - Design for monitoring commands

### Feature 2: Session Blocking & Lock Information
**Source Files:**
- `src/commands/locks.rs` (NEW) - Locks command implementation
- `src/commands/mod.rs` - Register new module
- `src/cli.rs` - Add LocksArgs and Command variant
- `src/main.rs` - Wire locks command
- `src/commands/repl/metacommands.rs` - Add /locks handler
- `src/commands/repl/metadata_completer.rs` - Tab completion

**Documentation:**
- `docs/specifications/repl.md` - Specification for /locks
- `docs/design/repl.md` - Design for lock monitoring

---

## Notes

- Follow the `sessions.rs` pattern exactly: SQL constant, parsed struct, execute/execute_for_repl functions, table/csv/json formatters
- Use the `teradata-monitor` skill as SQL reference
- Both commands are read-only DBA monitoring - no destructive operations
- This sprint establishes the PMON foundation for future monitoring features

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-24 | 1.0 | Initial sprint plan | Sprint Coordinator |
