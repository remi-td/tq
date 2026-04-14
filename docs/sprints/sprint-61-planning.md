# Sprint 61 Planning: Extended Session Control & Search Procedures

## Sprint Overview

**Sprint Goal:** Extend session control with bulk operations and add search procedures

**Sprint Theme:** DBA Operations + Discovery

**Date:** 2026-04-14
**Type:** Feature Sprint

## Reality Check Summary
- Reviewed sprints: 58, 59, 60
- Decision: Feature Sprint
- Rationale: Issue #20 partially complete (abort done in Sprint 49). Bulk operations are high-value DBA tools. Search procedures is low-effort since it follows existing patterns.

---

## Objectives

1. `/abort user <username>` — Abort all sessions for a specific user
2. `/abort host <hostname>` — Abort all sessions from a specific host
3. `/logoff idle [--older-than <duration>]` — Log off idle sessions
4. `tq search procedures <keyword>` — Search stored procedures by name

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Abort User Sessions

**Description:** `/abort user <username> [yes]` and `tq abort --user <username> --force`

**Acceptance Criteria:**
- [ ] Find all sessions for a user via MonitorSession
- [ ] Display matching sessions before confirmation
- [ ] Abort each session with MonitorAbortSession
- [ ] REPL confirmation required (append 'yes')
- [ ] Batch requires --force flag
- [ ] Report success/failure per session

#### Feature 2: Abort Host Sessions

**Description:** `/abort host <hostname> [yes]` and `tq abort --host <hostname> --force`

**Acceptance Criteria:**
- [ ] Same pattern as abort user but filtered by host
- [ ] Safety confirmation

#### Feature 3: Logoff Idle Sessions

**Description:** `/logoff idle [--older-than 1h]` and `tq logoff-idle --older-than 1h --force`

**Acceptance Criteria:**
- [ ] Find idle sessions (PEState = IDLE) older than threshold
- [ ] Default threshold: 1 hour
- [ ] Duration parsing reuse from history command
- [ ] Display matching sessions before confirmation
- [ ] Abort each idle session
- [ ] Report results

### P1 - High Priority (Should Have)

#### Feature 4: Search Procedures

**Description:** `tq search procedures <keyword>` and `/search procedures`

**Acceptance Criteria:**
- [ ] Query DBC.TablesV WHERE TableKind = 'P'
- [ ] All 4 output formats
- [ ] Pagination support
- [ ] REPL /search procedures with in <db> scoping
- [ ] Tab completion

---

### Explicitly Out of Scope

- Priority change (MonitorSetResource does not exist in Teradata)
- Release locks (requires aborting holding session — covered by existing /abort)

---

## GitHub Issues

### Selected for Sprint
- #20: Session Control Functions (remaining user stories: US-5.5, US-5.6, US-5.7)

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-04-14 | 1.0 | Initial sprint plan | Sprint Coordinator |
