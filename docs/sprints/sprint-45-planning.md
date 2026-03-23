# Sprint 45 Planning: Helper Bug Fix & Object Inspection

## Sprint Overview

**Sprint Goal:** Fix broken helper commands (Bug #32), implement comprehensive `/inspect` command (Issue #33), and resolve Sprint 44 deferred items.

**Sprint Theme:** Schema Inspection & Bug Fix

**Date:** 2026-03-23
**Type:** Feature Sprint
**Status:** Planning

---

## Reality Check Summary

- Reviewed sprints: 42, 43, 44
- Patterns detected: Recurring spec/implementation alignment gaps (minor, improving)
- Decision: Feature Sprint
- Rationale: Healthy velocity — 100% test pass rates, single sessions, reducing tech debt. No crisis patterns. Bug #32 is user-facing and must be fixed. Issue #33 is high-priority and builds on existing schema inspection foundation.

---

## Objectives

1. **Fix Bug #32**: Strip trailing semicolons from metacommand arguments so `/describe a;`, `/list tables;`, `/sample dbc.tables;` work correctly
2. **Implement /inspect command (Issue #33)**: Comprehensive object inspection showing type, columns, indexes, size/skew (tables), and dependencies (views/macros)
3. **Resolve Sprint 44 deferred items**: Doc drift, message polish, debug logging

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Fix Helper Commands (Bug #32)

**Description:** Metacommands don't strip trailing semicolons from arguments. Users habitually type semicolons after commands (SQL habit), causing all helper commands to fail. Root cause: `handle_metacommand_with_state()` and `handle_metacommand()` in `metacommands.rs` don't call `trim_end_matches(';')` on input before parsing, unlike the SQL executor which does.

**Acceptance Criteria:**
- [ ] AC-1: `/describe tablename;` resolves to table `tablename` (semicolon stripped)
- [ ] AC-2: `/list tables;` shows tables (not "Unknown list subcommand: tables;")
- [ ] AC-3: `/sample dbc.tables;` samples from `dbc.tables`
- [ ] AC-4: `/show indexes tablename;` shows indexes
- [ ] AC-5: All other metacommands with trailing semicolons work correctly
- [ ] AC-6: Unit tests cover semicolon stripping for at least 4 commands

**Root Cause:** Lines ~255 and ~46 in `metacommands.rs` — input is trimmed but not semicolon-stripped before `split_whitespace()`.

**Fix:** Add `trim_end_matches(';')` to input processing in both `handle_metacommand()` and `handle_metacommand_with_state()`, matching the pattern already used in `executor.rs`.

**Estimated Complexity:** Low

---

#### Feature 2: /inspect Command (Issue #33)

**Description:** Comprehensive object inspection command that consolidates and extends existing schema commands. Shows object type, columns/types, index structure, table size/skew, and object dependencies.

**Acceptance Criteria:**
- [ ] AC-1: `/inspect <table>` shows object type (Table, View, Macro, etc.)
- [ ] AC-2: `/inspect <table>` shows columns with types, nullable, default values
- [ ] AC-3: `/inspect <table>` shows primary index structure (PI columns, PPI, NoPI)
- [ ] AC-4: `/inspect <table>` shows secondary indexes if any
- [ ] AC-5: `/inspect <table>` shows table size (CurrentPerm) and skew factor
- [ ] AC-6: `/inspect <view>` shows column info and upstream dependencies
- [ ] AC-7: `/inspect` supports qualified names (`database.object`)
- [ ] AC-8: `tq inspect <object>` batch mode with table/CSV/JSON output
- [ ] AC-9: Tab completion for `/inspect` in REPL
- [ ] AC-10: Helpful error messages for non-existent objects or permission errors

**DBC Views Required:**
- `DBC.TablesV` — Object type (TableKind: T=Table, V=View, M=Macro, O=Table, etc.)
- `DBC.ColumnsV` — Column metadata (already used by /describe)
- `DBC.IndicesV` — Index information (already used by /show indexes)
- `DBC.TableSizeV` — Table size (CurrentPerm, PeakPerm) and per-AMP distribution for skew
- `DBC.TVM` + `DBC.TextTbl` + `DBC.Dbase` — Dependency analysis (from Issue #33 query)

**Reference:** Issue #33 on GitHub

**Estimated Complexity:** High

---

### P1 - High Priority (Should Have)

#### Feature 3: Sprint 44 Deferred Items

**Description:** Quick fixes from Sprint 44 review recommendations.

**Acceptance Criteria:**
- [ ] AC-1: Update `docs/design/connection-management.md` to match actual `resolve_driver_lib_dir` signature
- [ ] AC-2: `--force` description changed to "Skip confirmation prompt"
- [ ] AC-3: Abort message includes profile name: "Aborted. Profile 'NAME' was not deleted."
- [ ] AC-4: Add `log::debug!` at each fallback step in `resolve_driver_lib_dir`

**Estimated Complexity:** Low

---

### Explicitly Out of Scope

- Install script `/dev/tty` fix (REQ-INSTALL-010.2) — shell script testing infrastructure not yet available
- Install script `--accept-license` always show license (REQ-INSTALL-010.3.1) — deferred with install script items
- PMON features (Issues #17-25) — separate sprint focus area
- Dependency analysis using recursive CTE (Issue #33's advanced query) — deferred to Sprint 46 if the simpler dependency approach works; the recursive CTE requires careful testing against different Teradata versions

---

## GitHub Issues

### Selected for Sprint
- #32: [BUG] Helper command not working (bug)
- #33: [FEATURE] Need an inspect command (priority-high, enhancement)

### Deferred
- #17-25: PMON features — separate sprint focus area

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working
- [ ] P1 deferred items resolved
- [ ] 100% test pass rate (unit + integration)
- [ ] All acceptance criteria met
- [ ] Documentation updated
- [ ] Zero new technical debt

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Update `docs/specifications/repl.md` with `/inspect` command specification
- Update `docs/specifications/cli-interface.md` with `tq inspect` batch command
- Define output format for each object type

### rust-teradata-architect (Opus)
**Responsibilities:**
- Fix Bug #32: semicolon stripping in metacommand parsing
- Implement `/inspect` command (REPL + batch mode)
- Implement Sprint 44 deferred items (P1)
- Update `docs/design/` as needed
- Write unit tests for all new code

### quality-validator (Sonnet)
**Responsibilities:**
- Design test cases for Bug #32 fix and /inspect command
- Execute all test suites
- Validate acceptance criteria

---

## Files Involved

### Objective 1: Bug #32 Fix
**Source Files:**
- `src/commands/repl/metacommands.rs` — Strip trailing semicolons in both handler functions

**Test Files:**
- Unit tests in `metacommands.rs` `#[cfg(test)]` module

### Objective 2: /inspect Command
**Source Files:**
- `src/commands/inspect.rs` — New module for inspect logic
- `src/commands/mod.rs` — Export inspect module
- `src/commands/repl/metacommands.rs` — REPL dispatch for /inspect
- `src/commands/repl/metadata_completer.rs` — Tab completion
- `src/cli.rs` — CLI args for `tq inspect`
- `src/main.rs` — Command dispatch
- `src/lib.rs` — Re-exports if needed

**Documentation:**
- `docs/specifications/repl.md` — /inspect specification
- `docs/specifications/cli-interface.md` — tq inspect batch spec
- `docs/design/cli-interface.md` — Technical design
- `docs/user/repl-guide.md` — User documentation

### Objective 3: Sprint 44 Deferred
**Source Files:**
- `src/db/client.rs` — Debug logging in resolve_driver_lib_dir
- `src/commands/profile.rs` — Abort message, --force description
- `src/cli.rs` — --force help text
- `docs/design/connection-management.md` — Doc drift fix

---

## Risks & Mitigation

### Risk 1: /inspect DBC view permissions
- **Probability:** Medium
- **Impact:** Low (graceful degradation)
- **Mitigation:** Each section (type, columns, indexes, size, deps) fetched independently. If one DBC view is inaccessible, show what we can and note the missing section.

### Risk 2: TableSizeV availability
- **Probability:** Low
- **Impact:** Low
- **Mitigation:** Size/skew section is informational. If DBC.TableSizeV query fails, display "Size information unavailable" and continue.

---

## Action Items from Sprint 44

- [ ] Update `docs/design/connection-management.md` to match actual `resolve_driver_lib_dir` signature (Sprint 44 #1)
- [ ] `--force` description: "Skip confirmation prompt" (Sprint 44 #6)
- [ ] Abort message with profile name (Sprint 44 #5)
- [ ] `log::debug!` at each fallback step (Sprint 44 #7)

**Reference:** `docs/sprints/sprint-44-review.md` Section 7

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-03-23 | 1.0 | Initial sprint plan | Sprint Coordinator |
