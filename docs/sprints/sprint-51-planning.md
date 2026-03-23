# Sprint 51 Planning: Session History & Trends

**Sprint Duration:** 2026-03-23 (Single-session feature sprint)
**Status:** IN PROGRESS
**Target Version:** v1.32.0
**Issue:** #19 - PMON: Session History

---

## Objectives

Implement session history analysis for capacity planning and trend analysis:

1. `/history [--last <duration>]` — Show session logon/logoff activity
2. `tq history [--last <duration>]` — Batch mode session history
3. Time range filtering with human-readable durations

---

## Scope

### P0 — Must Have
- **Session history**: Query DBC.LogOnOffV for recent session activity
- **Time filtering**: `--last 1h`, `--last 24h`, `--last 7d` (default: 1h)
- **Summary stats**: Total logons, logoffs, peak concurrent sessions
- **Multi-format output**: Table/CSV/JSON

### P1 — Should Have
- **Tab completion**: `/history` in metacommand completion menu
- **User filtering**: `--user <username>` to filter by specific user
- **Error handling**: Privilege errors, unavailable views

---

## Technical Approach

### SQL Source
- Query DBC.LogOnOffV for logon/logoff events
- Time-bounded with `WHERE LogDate >= CURRENT_DATE - INTERVAL '1' DAY`
- Duration parsing: `1h` = 1 hour, `24h` = 24 hours, `7d` = 7 days

### Module Structure
- `src/commands/history.rs` — Session history implementation
- CLI args in `src/cli.rs`
- REPL handler in `src/commands/repl/metacommands.rs`
