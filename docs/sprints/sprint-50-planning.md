# Sprint 50 Planning: Query Drill-Down & Explain Plans

**Sprint Duration:** 2026-03-23 (Single-session feature sprint)
**Status:** IN PROGRESS
**Target Version:** v1.31.0
**Issue:** #24 - PMON: Query Drill-Down and Analysis (remaining parts)

---

## Objectives

Complete the query drill-down feature set (partially implemented in Sprint 39):

1. `/explain <sql>` — Show explain plan for a SQL statement (DBC.QryLogStepsV alternative: EXPLAIN prefix)
2. `/skew [session_id]` — Show AMP-level resource distribution for active sessions
3. `tq explain <sql>` — Batch mode explain
4. `tq skew [session_id]` — Batch mode skew analysis

---

## Scope

### P0 — Must Have
- **Explain plan**: Prefix SQL with EXPLAIN to get Teradata execution plan
- **Explain display**: Parse and display step-by-step execution plan
- **Skew analysis**: Query MonitorSession for AMP-level metrics, show hot AMPs
- **Multi-format output**: Table/CSV/JSON for batch mode

### P1 — Should Have
- **Tab completion**: New commands in metacommand completion menu
- **Error handling**: Permission errors, DBQL not enabled, etc.

---

## Technical Approach

### Explain Plan
- Use Teradata's `EXPLAIN <sql>` which returns the execution plan as result rows
- Parse the text-based explain output and display with formatting
- REPL: `/explain SELECT * FROM t` executes `EXPLAIN SELECT * FROM t`
- Batch: `tq explain "SELECT * FROM t"`

### Skew Analysis
- Query MonitorSession for per-AMP metrics
- Calculate and display CPU/IO distribution across AMPs
- Show hottest AMPs and skew factors
- REPL: `/skew` (current session) or `/skew <session_id>`
- Batch: `tq skew <session_id>`
