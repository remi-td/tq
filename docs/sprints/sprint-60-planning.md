# Sprint 60 Planning: Watch Mode for Monitoring Commands

## Sprint Overview

**Sprint Goal:** Add auto-refresh watch mode to monitoring commands (Issue #25)

**Sprint Theme:** Real-Time Monitoring

**Date:** 2026-04-14
**Type:** Feature Sprint

## Reality Check Summary
- Reviewed sprints: 57, 58, 59
- Patterns detected: Consistent velocity, clean test pass rates
- Decision: Feature Sprint
- Rationale: Watch mode is the most requested monitoring enhancement. Crossterm already available as dependency.

---

## Objectives

1. Add `--watch` and `--interval` flags to sessions, locks, and resources commands
2. Implement terminal-based auto-refresh with clear-and-redraw
3. Keyboard controls: q, Esc, Ctrl-C to exit watch mode
4. REPL integration for `/sessions --watch`, `/locks --watch`, `/resources --watch`

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Watch Mode Infrastructure

**Description:** Shared watch module using crossterm for terminal clear, key detection, and refresh loop.

**Acceptance Criteria:**
- [ ] `run_watch()` function that takes a render closure and interval
- [ ] Terminal raw mode for key detection during sleep
- [ ] Clean terminal restore on exit (even on error)
- [ ] Status footer with timestamp and interval
- [ ] q, Esc, Ctrl-C all exit cleanly

#### Feature 2: Batch Mode Watch

**Description:** `tq sessions --watch`, `tq locks --watch`, `tq resources --watch`

**Acceptance Criteria:**
- [ ] `--watch` flag on all three commands
- [ ] `--interval N` with default 6, range 2-300
- [ ] `--watch` conflicts with `--output`
- [ ] Works with all format flags

#### Feature 3: REPL Watch Mode

**Description:** `/sessions --watch`, `/locks --watch`, `/resources --watch`

**Acceptance Criteria:**
- [ ] Parse --watch and optional interval from REPL args
- [ ] Auto-refresh in REPL context
- [ ] Return to REPL prompt on exit

---

### Explicitly Out of Scope

- Graphical TUI (Issue #21, #22)
- Alerting thresholds (Issue #23)
- Watch mode for non-monitoring commands

---

## GitHub Issues

### Selected for Sprint
- #25: Dynamic Session Monitoring (core implementation)

### Deferred
- #21, #22: Graphical displays
- #23: Alerting thresholds

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-04-14 | 1.0 | Initial sprint plan | Sprint Coordinator |
