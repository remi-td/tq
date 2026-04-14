# Sprint 59 Planning: Bug Fix + PMON Resource Monitoring

## Sprint Overview

**Sprint Goal:** Fix failing integration tests and implement PMON resource monitoring command (Issue #17)

**Sprint Theme:** Quality Fix + PMON Feature

**Date:** 2026-04-14
**Type:** Feature Sprint

## Reality Check Summary
- Reviewed sprints: 56, 57, 58
- Patterns detected: 2 integration tests failing since Sprint 53 (JSON envelope change broke test expectations)
- Decision: Feature Sprint with bug fix
- Rationale: Tests are a quick fix (tests expect old array format, not new envelope). Main feature is high-value PMON resource monitoring.

---

## Objectives

1. Fix 2 failing integration tests (test_format_json_output, test_format_json_empty)
2. Implement `/resources` command for CPU, I/O, memory metrics per VPROC and node
3. `tq resources` batch mode with all output formats
4. REPL metacommand with tab completion

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Fix Failing Integration Tests

**Description:** Two integration tests (test_format_json_output, test_format_json_empty) expect old plain-array JSON format but Sprint 53 changed to envelope format `{"ok": true, "row_count": N, "data": [...]}`.

**Acceptance Criteria:**
- [x] Both tests updated to expect envelope format
- [x] All tests pass (967 unit + integration)

**Estimated Complexity:** Low

#### Feature 2: PMON Resource Monitoring Command

**Description:** New `/resources` command (REPL) and `tq resources` (batch) for monitoring CPU, I/O, and memory metrics. Queries DBC.ResUsageSVPR (virtual/per-VPROC, default) and DBC.ResUsageSPMA (physical/per-node).

**Acceptance Criteria:**
- [ ] `tq resources` works in batch mode (default: virtual mode)
- [ ] `tq resources --physical` shows per-node metrics
- [ ] All 4 output formats (table, JSON, CSV, markdown)
- [ ] `/resources` works in REPL with aliases `/res`, `/perf`
- [ ] `--physical` flag works in REPL (`/resources --physical`)
- [ ] Tab completion for all aliases
- [ ] Skew calculation for CPU and I/O across VPROCs/nodes
- [ ] Summary footer with system-wide metrics
- [ ] Privilege error handling (clear guidance message)
- [ ] Pagination support (--page-size, --page)
- [ ] Unit tests for all render functions
- [ ] Zero clippy warnings

**Reference:** Issue #17, Admin User Stories Section 2

**Estimated Complexity:** Medium-High

---

### Explicitly Out of Scope

- Watch mode / auto-refresh (Sprint 60)
- Graphical displays (Issue #21, #22)
- Alerting thresholds (Issue #23)
- Dynamic session monitoring (Issue #25)

---

## Success Criteria

- [ ] All tests pass (unit + integration)
- [ ] Zero clippy warnings
- [ ] Documentation updated (specs, design)
- [ ] All P0 features implemented and tested

---

## GitHub Issues

### Selected for Sprint
- #17: PMON Performance Summary and Resource Usage (partial — core metrics)

### Deferred
- #21: Graphical Resource Displays (requires TUI)
- #22: Graphical Session Displays (requires TUI)
- #23: Alerting and Threshold Configuration
- #25: Dynamic Session Monitoring (Sprint 60)

---

## Agent Assignments

### cli-ux-designer (Sonnet)
- Update specifications with resources command interface
- Design table output layout for virtual and physical modes

### rust-teradata-architect (Opus)
- Implement resources.rs command module
- Wire up CLI, main.rs, lib.rs, REPL integration
- Write unit tests

### quality-validator (Sonnet)
- Execute test suite
- Validate all output formats
- Verify REPL integration

---

## Files Involved

### Bug Fix
- `tests/integration_tests.rs` — Update JSON assertions to envelope format

### Resources Command
- `src/cli.rs` — Add Resources variant and ResourcesArgs
- `src/commands/resources.rs` — NEW: Main implementation
- `src/commands/mod.rs` — Add module and re-export
- `src/main.rs` — Add dispatch
- `src/lib.rs` — Add re-export
- `src/commands/repl/metacommands.rs` — Add /resources handler
- `src/commands/repl/metadata_completer.rs` — Add tab completion

### Documentation
- `docs/specifications/cli-interface.md` — Resources command spec
- `docs/specifications/repl.md` — /resources metacommand
- `docs/design/cli-interface.md` — Resources design

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-04-14 | 1.0 | Initial sprint plan | Sprint Coordinator |
