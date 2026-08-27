# Technical Design: Real-Time Running Query Monitoring and Step Execution Plan

## Overview

This document specifies the technical design for real-time monitoring of currently running database sessions, active query text, step progress, and full execution plans in Teradata.

Unlike historical DBQL query log analysis (`DBC.QryLogV`), this capability monitors **live, currently executing requests** in memory using Teradata `SYSLIB` monitor table functions.

---

## Architecture & Data Flow

```
                     +---------------------------------------+
                     |         DBC.SessionInfoV              |
                     | (SessionNo, UserName, LogicalHostId,  |
                     |  HostNo, IFPNo/RunVProcNo, PEState)   |
                     +-------------------+-------------------+
                                         |
                       Resolves HostId & RunVProcNo (PE)
                                         |
    +------------------------------------+------------------------------------+
    |                                    |                                    |
    v                                    v                                    v
+------------------------+  +---------------------------+  +--------------------------+
| SYSLIB.MonitorSQLText  |  |SYSLIB.MonitorSQLCurrentStep| |   SYSLIB.MonitorSQLSteps |
| (Real-time SQL Text)   |  | (Active Step & Total #)   |  | (Per-Step Est vs Actual) |
+------------------------+  +---------------------------+  +--------------------------+
```

---

## Teradata DBC & SYSLIB Interface Specifications

### 1. Active Session Discovery
- **Source**: `DBC.SessionInfoV` joined with `TABLE(SYSLIB.MonitorSession(-1, '*', 0))`
- **Key Columns**:
  - `SessionNo`: Unique Teradata session identifier
  - `UserName`: Connected database user
  - `LogicalHostId` / `HostNo`: Logical host identifier (`HostIdIn` for SYSLIB functions)
  - `IFPNo`: Parsing Engine vproc number (`RunVProcNo` for SYSLIB functions)
  - `PEState`: Parsing Engine state (`IDLE`, `ACTIVE`, `DISPATCHING`)
  - `AMPState`: AMP execution state (`IDLE`, `ACTIVE`)

### 2. Active Query Text & Step Progress (`tq active-query <session_id>`)
- **Queries**:
  ```sql
  -- Get active SQL text for currently running request
  SELECT HostId, SessionNo, SeqNum, SQLTxt
  FROM TABLE (SYSLIB.MonitorSQLText({HostId}, {SessionNo}, {RunVprocNo})) AS t1
  ORDER BY SeqNum;

  -- Get current step progress
  SELECT NumOfSteps, CurLvl1StepNo, CurLvl2StepNo, DynamicPlan, PartialSteps, DefaultDBName
  FROM TABLE (SYSLIB.MonitorSQLCurrentStep({HostId}, {SessionNo}, {RunVprocNo})) AS t1;
  ```

### 3. Full Execution Plan with Live Metrics (`tq query-plan <session_id>`)
- **Query**:
  ```sql
  SELECT StepNum, Confidence, EstRowCount, ActRowCount, EstRowCountSkew, ActRowCountSkew, EstElapsedTime, ActElapsedTime, SQLStep
  FROM TABLE (SYSLIB.MonitorSQLSteps({HostId}, {SessionNo}, {RunVprocNo})) AS t1
  ORDER BY StepNum;
  ```
- **Metric Processing**:
  - For completed steps: `ActRowCount >= 0`, `ActElapsedTime >= 0`
  - For pending/in-progress steps: `ActRowCount == -1`, `ActElapsedTime == -1` (displayed as `[in progress]` or `[pending]`)
  - Skew calculation: `EstRowCountSkew` vs `ActRowCountSkew`

### 4. Live Abort Control (`tq abort <session_id> [--query]`)
- **Queries**:
  ```sql
  -- Abort running query only (LogoffSessions = 'N')
  SELECT SessionNo, UserName, AbortStatus
  FROM TABLE (SYSLIB.AbortListSessions({HostId}, '*', {SessionNo}, 'N', 'Y')) AS t1;

  -- Abort full session (LogoffSessions = 'Y')
  SELECT SessionNo, UserName, AbortStatus
  FROM TABLE (SYSLIB.AbortListSessions({HostId}, '*', {SessionNo}, 'Y', 'Y')) AS t1;
  ```

---

## Rust Implementation Strategy

1. **HostId & RunVprocNo Resolution**:
   - Query `DBC.SessionInfoV` for the target `session_id` to retrieve `LogicalHostId` and `IFPNo`.
2. **Commands & Module Layout**:
   - `src/commands/active_query.rs`: Real-time query & step progress inspection (`tq active-query`)
   - `src/commands/query_plan.rs`: Real-time step execution plan display (`tq query-plan`)
   - `src/commands/abort.rs`: Update to use `SYSLIB.AbortListSessions` for target query/session aborting.
   - `src/commands/repl/metacommands.rs`: Add `/active-query`, `/query-plan`, and update `/abort`.

---

## Verification Plan

- Unit tests for metric parsing (`ActRowCount == -1` handling, skew display, multi-sequence SQL concatenation).
- Integration testing against live Teradata instance (`demo-vikzqtnd0db0nglk.env.trial.teradata.com`).
