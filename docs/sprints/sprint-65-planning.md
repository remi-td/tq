---
sprint: 65
start_date: 2026-04-17
target_completion: 2026-04-17
status: Planning
---

# Sprint 65 Planning: Dynamic Session Monitoring (`/sessions --watch`)

## Sprint Overview

**Sprint Goal:** Ship auto-refreshing session monitoring so DBAs can see near-real-time session activity without manually re-running `/sessions`.

**Sprint Theme:** DBA Live Monitoring
**Type:** Feature Sprint
**Date:** 2026-04-17

---

## Reality Check Summary

- Reviewed sprints: 62, 63, 64
- Patterns detected: None. All three delivered with 100% pass rates. Sprint 64 reviewers caught a P0 spec-compliance issue that was fixed in-sprint per zero-debt policy.
- Decision: Feature Sprint
- Rationale: Healthy velocity. GitHub issue #25 (PMON Dynamic Session Monitoring, priority-medium) is the highest-priority open feature and extends the DBA workflow that Sprints 26 (sessions), 61 (abort/logoff), 62 (security), and 63 (pager UX) have been incrementally building.

---

## Objectives

1. Add auto-refreshing mode to `/sessions` with configurable interval.
2. Make exit cleanly via `q`, `Esc`, or `Ctrl-C` — the exit-snapshot pattern from Sprint 63 applies here too.
3. Keep scope tight: refresh-in-place of the existing `/sessions` output, NOT a full ratatui TUI. A clean incremental change is MORE valuable than a large UX rewrite.

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: `/sessions --watch` with auto-refresh

**Description:** Extend the existing `/sessions` REPL metacommand to accept a `--watch` flag. While in watch mode, the session list redraws at a configurable interval. Exit returns to the REPL prompt without leaving terminal state in raw-mode.

**Acceptance Criteria:**
- [ ] `/sessions --watch` enters watch mode with a default refresh interval of 6 seconds
- [ ] `/sessions --watch --interval 10` uses a 10-second interval
- [ ] `/sessions --watch --interval 2` uses a 2-second interval (minimum 1 second, maximum reasonable ceiling e.g. 3600)
- [ ] Each refresh shows: same columns as non-watch `/sessions`, plus a header line with timestamp and the configured interval
- [ ] `q`, `Esc`, or `Ctrl-C` exits watch mode and returns to REPL prompt
- [ ] On exit, a static snapshot of the last frame is printed (parallel to Sprint 63 pager exit snapshot pattern — copy-paste friendly, no ANSI)
- [ ] Watch mode restores terminal state (leaves alternate screen, disables raw mode) on exit AND on panic
- [ ] If a refresh query fails (e.g. DB hiccup), display the error in the frame header and keep trying on the next tick — do NOT crash out of watch mode
- [ ] No regression in non-watch `/sessions` behaviour
- [ ] Unit tests for interval parsing, at least one interactive test covering enter-and-exit

**Out of scope (explicit):**
- Full ratatui TUI — deferred to a separate sprint if demand is high
- Threshold coloring / alerting — blocked on PMON Alerting (#23)
- Session history / trending — separate feature (#19)
- Batch-mode `tq sessions --watch` — REPL-only for this sprint

**Reference:** GitHub Issue #25, `docs/specifications/admin-user-stories.md` Section 10

**Estimated Complexity:** Medium (leverages existing `/sessions` query + Sprint 63 alternate-screen/raw-mode exit pattern)

---

### Out of Scope

- PMON Alerting (#23) — separate issue, depends on this as foundation
- Graphical session displays (#22) — separate issue
- Full-screen ratatui TUI — too large for one sprint
- Batch-mode `--watch` — REPL-only scope for this sprint
- Sprint 64 P3 follow-ups — deferred

---

## GitHub Issues

### Selected for Sprint
- #25: [FEATURE] PMON: Dynamic Session Monitoring (enhancement, priority-medium)

### Deferred
- #23 (PMON Alerting): Builds on this sprint's foundation, next iteration
- #22 (PMON Graphical Session Displays): Separate sprint
- #21 (PMON Graphical Resource Displays): Depends on perf-summary data collection

---

## Dependencies

- Existing `/sessions` command and underlying MonitorSession query (Sprint 26)
- Terminal state management: alternate screen + raw mode patterns (reused from pager, Sprint 63)

---

## Definition of Done

- [ ] Feature implemented and tested per acceptance criteria
- [ ] 100% unit test pass rate
- [ ] Interactive test (or manual script) validates enter → tick → exit flow
- [ ] No new clippy warnings
- [ ] User docs updated (`docs/user/repl-guide.md`)
- [ ] Version bumped to v1.47.0
- [ ] Git tag + release published
- [ ] GitHub issue #25 closed with fix reference
