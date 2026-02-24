# Sprint 39 Planning: PMON Hardening & Query Inspection

## Sprint Overview

**Sprint Goal:** Harden the PMON monitoring foundation by eliminating code duplication, fixing Sprint 38 gaps, and adding query inspection capability.

**Sprint Theme:** PMON Hardening + Query Drill-Down (Issue #24, partial)

**Date:** 2026-02-24
**Type:** Feature Sprint
**Target:** Single-session execution

---

## Reality Check Summary

- Reviewed sprints: 36 (9.0), 37 (9.3), 38 (7.8)
- Patterns detected: Utility code duplication across 4 monitoring modules (sessions.rs, sysconfig.rs, locks.rs, sample.rs), spec/implementation gaps in Sprint 38
- Decision: Feature Sprint with Sprint 38 remediation bundled
- Rationale: Healthy velocity, no crisis. Sprint 38 gaps are quality issues, not systemic problems. Remediation is low effort and fits alongside new feature work.

---

## Objectives

1. **Eliminate monitoring code duplication** - Extract shared utilities into a reusable module
2. **Fix Sprint 38 quality gaps** - CSV bug, design doc drift, user guide alignment
3. **Add query inspection capability** - `/query <session_id>` to view SQL text (Issue #24, partial)

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Monitoring Utilities Extraction

**Description:** Extract duplicated functions from sessions.rs, sysconfig.rs, locks.rs, and sample.rs into a shared `monitoring_utils.rs` module.

**Acceptance Criteria:**
- [ ] AC-1: `extract_integer()`, `extract_trimmed_string()`, `extract_decimal()` exist in `src/commands/monitoring_utils.rs`
- [ ] AC-2: `escape_csv()` exists in shared module (currently duplicated 4x)
- [ ] AC-3: sessions.rs, sysconfig.rs, locks.rs, sample.rs use shared functions (no local copies)
- [ ] AC-4: All existing tests pass after refactor (748/748)
- [ ] AC-5: Zero clippy warnings

**Reference:** Sprint 38 review recommendation #8

**Estimated Complexity:** Medium

---

#### Feature 2: Sprint 38 Bug Fixes & Doc Alignment

**Description:** Address the P0 remediation items from Sprint 38 review.

**Acceptance Criteria:**
- [ ] AC-6: CSV output for locks with no waiters uses empty string, not "(none)"
- [ ] AC-7: `docs/design/repl.md` locks section updated to reflect DBC.LockInfoV implementation (not MonitorSession)
- [ ] AC-8: User guide (`docs/user/repl-guide.md`) updated to match actual implementation (remove references to Node Count, PE Count, Blocked Since if not implemented)
- [ ] AC-9: Error handling unit tests added for sysconfig.rs and locks.rs

**Reference:** Sprint 38 review, recommendations #1-4

**Estimated Complexity:** Low

---

### P1 - Should Have

#### Feature 3: Query Inspection Command (#24, partial)

**Description:** New `/query <session_id>` command to show the SQL text of a session's most recent query. This is the natural next step in the PMON workflow: see sessions → see locks → inspect the SQL.

**Acceptance Criteria:**
- [ ] AC-10: `/query <session_id>` shows SQL text from DBC.QryLogV for the given session
- [ ] AC-11: `tq query <session_id>` works in batch mode with table/CSV/JSON output
- [ ] AC-12: Tab completion includes `/query` and alias `/q`
- [ ] AC-13: Help text describes command usage
- [ ] AC-14: Clear error message when session not found or no query logged
- [ ] AC-15: Handles long SQL text gracefully (truncation with full-text option)
- [ ] AC-16: Unit tests for SQL generation, parsing, and display logic

**Reference:** GitHub Issue #24, `docs/specifications/admin-user-stories.md` Section 9 (US-9.1)

**Estimated Complexity:** Medium

---

### Explicitly Out of Scope

- `/explain <session_id>` and `/skew <session_id>` (rest of Issue #24) - deferred to future sprint
- Node Count, PE Count for `/sysconfig` - insufficient SQL source identified in Sprint 38
- "Blocked Since" column for `/locks` - deferred
- Dynamic session monitoring (#25) - requires async/TUI architecture
- Graphical displays (#21, #22) - requires TUI framework
- Session Control (#20) - complex safety requirements, separate sprint

---

## GitHub Issues

### Selected for Sprint
- #24: PMON: Query Drill-Down and Analysis (partial - `/query` only)

### Deferred
- #17: Performance Summary - complex, requires ResUsage views
- #19: Session History - deferred to future sprint
- #20: Session Control - complex safety requirements
- #21, #22: Graphical Displays - requires TUI framework
- #23: Alerting - requires graphical displays
- #25: Dynamic Monitoring - requires async architecture

---

## Action Items from Previous Sprint

- [x] Sync design doc with DBC.LockInfoV implementation (Sprint 38 review #1)
- [x] Fix CSV "(none)" → empty string (Sprint 38 review #2)
- [x] Add error handling unit tests (Sprint 38 review #3)
- [x] Update user guide to match implementation (Sprint 38 review #4)
- [x] Extract shared monitoring utilities module (Sprint 38 review #8)

**Reference:** `docs/sprints/sprint-38-review.md`

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Update specifications for `/query` command
- Review and fix user guide alignment for Sprint 38 features
- Ensure UX consistency with existing monitoring commands

**Deliverables:**
- Updated `docs/specifications/repl.md` with REQ-QUERY sections
- Updated `docs/specifications/cli-interface.md` with `tq query` section
- Fixed `docs/user/repl-guide.md` for Sprint 38 alignment
- UX validation report

### rust-teradata-architect (Opus)
**Responsibilities:**
- Extract monitoring utilities into shared module
- Fix CSV "(none)" bug in locks.rs
- Implement `/query <session_id>` command
- Sync design documentation
- Write unit tests for all changes

**Deliverables:**
- `src/commands/monitoring_utils.rs` - shared utilities module
- Refactored sessions.rs, sysconfig.rs, locks.rs, sample.rs
- `src/commands/query.rs` - query inspection command
- Updated `docs/design/repl.md`
- Unit tests with 100% pass rate

### quality-validator (Sonnet)
**Responsibilities:**
- Design test cases for monitoring refactor and new command
- Execute full test suite
- Validate all acceptance criteria

**Deliverables:**
- Test strategy and test cases
- Test execution report with 100% pass rate
- Acceptance criteria validation

---

## Files Involved

### Objective 1: Monitoring Utilities Extraction
**Source Files:**
- `src/commands/monitoring_utils.rs` - NEW shared module
- `src/commands/mod.rs` - Register new module
- `src/commands/sessions.rs` - Remove duplicated functions, use shared module
- `src/commands/sysconfig.rs` - Remove duplicated functions, use shared module
- `src/commands/locks.rs` - Remove duplicated functions, use shared module
- `src/commands/sample.rs` - Remove duplicated escape_csv, use shared module

### Objective 2: Sprint 38 Fixes
**Source Files:**
- `src/commands/locks.rs` - Fix CSV "(none)" output
- `src/commands/sysconfig.rs` - Error handling tests
- `src/commands/locks.rs` - Error handling tests

**Documentation:**
- `docs/design/repl.md` - Sync locks section
- `docs/user/repl-guide.md` - Fix implementation alignment

### Objective 3: Query Inspection
**Source Files:**
- `src/commands/query.rs` - NEW query inspection command
- `src/commands/mod.rs` - Register query module
- `src/cli.rs` - QueryArgs, Command variant
- `src/main.rs` - Wire query command
- `src/commands/repl/metacommands.rs` - /query handler
- `src/commands/repl/metadata_completer.rs` - Tab completion

**Documentation:**
- `docs/specifications/repl.md` - REQ-QUERY sections
- `docs/specifications/cli-interface.md` - tq query section
- `docs/design/repl.md` - Query inspection design
- `docs/user/repl-guide.md` - User guide section

---

## Risks & Mitigation

### Risk 1: DBC.QryLogV Availability
- **Probability:** Medium
- **Impact:** Medium
- **Mitigation:** QryLogV requires DBQL logging to be enabled. If not available, show clear error message with guidance. This is the standard Teradata view for query text and is commonly enabled.

### Risk 2: Refactoring Regressions
- **Probability:** Low
- **Impact:** High
- **Mitigation:** Monitoring utilities extraction is a mechanical refactor. All 748 existing tests serve as regression safety net. Run full test suite after each module refactored.

### Risk 3: Session Budget
- **Probability:** Low
- **Impact:** Medium
- **Mitigation:** P0 items (refactor + fixes) are estimated ~2 hours. P1 (query command) follows established monitoring pattern. If session budget is tight, defer P1 to next sprint.

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working
- [ ] P1 feature implemented or explicitly deferred with rationale
- [ ] 100% test pass rate (unit + integration)
- [ ] All acceptance criteria met for delivered features
- [ ] Documentation synchronized with implementation (spec-implementation cross-check)
- [ ] Zero technical debt introduced
- [ ] Zero clippy warnings

---

## Notes

- This sprint addresses Sprint 38's main lesson: spec-implementation alignment verification
- The monitoring utilities extraction reduces tech debt from 4x duplication to 1 shared module
- `/query` command follows the exact same architectural pattern as `/sessions`, `/sysconfig`, `/locks`
- Spec-implementation cross-check step added to Phase 4 (new mitigation from Sprint 38)

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-24 | 1.0 | Initial sprint plan | Sprint Coordinator |
