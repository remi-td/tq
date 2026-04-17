---
sprint: 64
start_date: 2026-04-17
target_completion: 2026-04-17
status: Planning
---

# Sprint 64 Planning: Bug Fixes — File Mode Parser & Stdin Detection

## Sprint Overview

**Sprint Goal:** Eliminate two user-reported bugs that break real-world usage of `tq query` in stored procedure deployment and automation/CI contexts.

**Sprint Theme:** Bug Fix Sprint
**Type:** Feature Sprint
**Date:** 2026-04-17

---

## Reality Check Summary

- Reviewed sprints: 61, 62, 63
- Patterns detected: None (healthy velocity — 4/4, 6/6, 1/1 features, 100% pass rates)
- Decision: Feature Sprint, bug-fix focused
- Rationale: Two high-value open bugs (#42 high severity, #43 low but hits agent/CI workflows). Both are well-scoped with clear repro steps and suggested fixes. Clearing these keeps the `tq query` surface trustworthy for deployment automation — which is a core use case.

---

## Objectives

1. Fix stored-procedure BEGIN/END body splitting so `tq query --file` can deploy SPL, macros, and triggers in one shot.
2. Fix stdin detection so `tq query "SQL" < /dev/null` (and equivalent CI/agent redirection patterns) runs without spurious "multiple input sources" error.

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Track BEGIN/END depth in file-mode statement splitter (#42)

**Description:** Extend the existing state-machine lexer that parses `--file` input to track BEGIN/END block nesting. While inside a procedure/trigger/macro body, internal `;` characters are body-internal and must not be treated as top-level statement terminators.

**Acceptance Criteria:**
- [ ] `tq query --file repro_sp.sql` from issue #42 submits the entire `REPLACE PROCEDURE ... BEGIN ... END;` as a single statement
- [ ] Nested BEGIN/END blocks (e.g., `BEGIN ... IF THEN ... END IF; ... END`) are handled correctly
- [ ] String literals containing `BEGIN` or `END` (e.g., `'BEGIN'`) do NOT affect block depth (existing string-state handling must compose correctly)
- [ ] Comments (`--` line and `/* */` block) containing BEGIN/END do not affect block depth
- [ ] Plain multi-statement scripts WITHOUT procedure bodies continue to split correctly
- [ ] `CREATE | REPLACE PROCEDURE | TRIGGER | MACRO` headers are detected case-insensitively
- [ ] Unit tests cover: single procedure, nested blocks, comments inside body, strings containing keywords, multi-procedure script, mixed SPL + regular statements
- [ ] No regression in existing `--file` splitter tests

**Reference:** GitHub Issue #42, `src/sql/` (existing lexer from Sprint 42)
**Estimated Complexity:** Medium

---

#### Feature 2: Correct stdin detection when stdin is redirected but empty (#43)

**Description:** Stop treating an empty redirected stdin (e.g., `< /dev/null`) as a second input source. Current detection flags any non-TTY stdin as "stdin provided" even when there are no bytes available. Fix: when a positional query argument is present, ignore stdin unless it actually has data available, OR only treat stdin as an input source when `!isatty(0) && has_bytes_available()`.

**Acceptance Criteria:**
- [ ] `tq query "SELECT 1" < /dev/null` runs the query successfully
- [ ] `tq query "SELECT 1" <<< ""` runs the query successfully
- [ ] `echo "SELECT 2" | tq query` still reads from stdin as before (regression guard)
- [ ] `echo "SELECT 2" | tq query "SELECT 1"` still rejects with "multiple input sources" error (regression guard — real conflict)
- [ ] `tq query "SELECT 1"` in an interactive terminal (TTY stdin) still works
- [ ] Error message quality unchanged for the legitimate conflict case
- [ ] Unit tests or integration test covering all four scenarios above

**Reference:** GitHub Issue #43, `src/main.rs` or `src/commands/query.rs` (stdin handling)
**Estimated Complexity:** Low

---

### Out of Scope

- PMON features (#21, #22, #23, #25) — deferred to Sprint 65+
- MySQL-style `DELIMITER //` escape hatch (alternative in #42) — primary fix uses BEGIN/END tracking, delimiter is a future enhancement if needed
- Sprint 62 follow-up items (TLS, abort.rs cleanup, etc.) — deferred
- Sprint 63 follow-up items (Ctrl-C in pager, context-aware format hints) — deferred

---

## GitHub Issues

### Selected for Sprint
- #42: Stored-procedure BEGIN/END bodies split at internal semicolons in --file mode (bug, high severity)
- #43: `tq query "SQL"` rejects command when stdin is redirected from empty source (bug, low severity)

### Deferred
- #25 (PMON Dynamic Session Monitoring): Larger feature, targeted for Sprint 65
- #23, #22, #21 (PMON visualization/alerting): Lower priority, after core PMON lands
- #42 alternative: DELIMITER directive — not needed if BEGIN/END tracking works

---

## Dependencies

- None. Both bugs touch isolated subsystems (SQL statement splitter, CLI arg/stdin detection).

---

## Definition of Done

- [ ] Both bugs fixed per acceptance criteria
- [ ] 100% unit test pass rate
- [ ] No new clippy warnings
- [ ] Design docs updated if parser architecture changes
- [ ] Version bumped to v1.46.0
- [ ] Git tag + release published
- [ ] GitHub issues #42 and #43 closed with fix references
