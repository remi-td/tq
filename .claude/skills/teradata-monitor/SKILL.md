---
name: teradata-monitor
description: Provides SQL queries for monitoring Teradata system usage, activity, and health. Use when building DBA monitoring features, diagnosing performance issues, or implementing system health checks.
---

# Teradata System Monitor

Expert guidance for monitoring Teradata system health, performance, and resource usage from a DBA and platform admin perspective.

## Overview

This skill provides ready-to-use SQL queries for monitoring Teradata systems across these domains:

| Domain | What It Covers |
|--------|---------------|
| **System Info** | Version, configuration, node/AMP topology |
| **Sessions** | Active sessions, connected users, session details |
| **Running Queries** | Currently executing SQL, progress, active AMPs |
| **Query Performance** | DBQL analysis, top consumers by CPU/IO/time |
| **Resource Usage** | CPU, memory, I/O at node and AMP level |
| **Disk Space** | Database/table storage, allocation, skew |
| **Locks** | Lock contention, blocking sessions, deadlocks |
| **Spool Space** | Spool consumption, per-user limits |
| **Workload** | AMP usage by user/account |
| **Security** | Access rights, roles, logon/logoff history |
| **VantageCloud Lake** | Compute cluster status, scaling events |

## When To Use

- Implementing `tq` monitoring subcommands or features
- Building health-check or diagnostic queries
- Investigating slow queries or resource contention
- Designing DBA dashboards or alerting queries

## Quick Reference

### System Configuration

```sql
-- Teradata version and release info
SELECT InfoKey, CAST(InfoData AS VARCHAR(50)) AS InfoData
FROM DBC.DBCInfoV;

-- Node and component topology (AMPs, PEs, GTW, RSG, TVS per node)
SELECT DISTINCT NodeID, NodeType,
    VProcType1 || ': ' || TRIM(VProc1) AS AMPs,
    VProcType2 || ': ' || TRIM(VProc2) AS PEs,
    VProcType3 || ': ' || VProc3 AS GTW,
    VProcType4 || ': ' || VProc4 AS RSG,
    VProcType5 || ': ' || VProc5 AS TVS
FROM DBC.ResUsageSpma
ORDER BY NodeID;

-- Total AMP count
SELECT HASHAMP()+1 AS TotalAMPs;

-- Node count and AMPs per node
SELECT NodeID, COUNT(DISTINCT Vproc) AS AMPs_Per_Node
FROM DBC.ResCpuUsageByAmpView
GROUP BY NodeID;
```

### Active Sessions

```sql
-- All active sessions with client details
SELECT SessionNo, UserName, ClientIpAddress, ClientProgramName,
    ClientSystemUserId, ClientOsName,
    CASE Transaction_Mode
        WHEN 'A' THEN 'ANSI'
        WHEN 'T' THEN 'TDBS'
    END AS TransactionMode,
    CurIsolationLevel
FROM DBC.SessionInfoV
ORDER BY UserName;

-- Session count by user
SELECT UserName, COUNT(*) AS SessionCount
FROM DBC.SessionInfoV
GROUP BY UserName
ORDER BY SessionCount DESC;

-- Detailed session info (logon source, timing)
SELECT UserName, SessionNo, DefaultDatabase,
    LogonDate, LogonTime, LogonSource,
    IFPNo, Partition, LogicalHostId, HostNo
FROM DBC.SessionInfo
ORDER BY LogonDate DESC, LogonTime DESC;
```

### Currently Running Queries

```sql
-- Active sessions with AMP state (requires EXECUTE FUNCTION on SYSLIB)
SELECT HostId, SessionNo, RunVprocNo, LogonTime,
    UserName, UserAccount, ReqStartTime, LogonSource, PEState
FROM TABLE(MonitorSession(-1, '*', 0)) AS dt
WHERE AmpState = 'ACTIVE'
ORDER BY ReqStartTime;

-- Get SQL text for a specific running session
-- (use HostId, SessionNo, RunVprocNo from MonitorSession above)
SELECT *
FROM TABLE(MonitorSQLText({HostId}, {SessionNo}, {RunVprocNo})) AS dt;

-- Monitor step progress for a running query
SELECT *
FROM TABLE(MonitorSQLCurrentStep({HostId}, {SessionNo}, {RunVprocNo})) AS dt;
```

### Disk Space

```sql
-- Database space summary (allocated vs used)
SELECT DatabaseName,
    CAST(SUM(MaxPerm)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Allocated_GB",
    CAST(SUM(CurrentPerm)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Used_GB",
    CAST((1 - SUM(CurrentPerm)/NULLIFZERO(SUM(MaxPerm)))*100
        AS DECIMAL(5,2)) AS "Free_Pct"
FROM DBC.DiskSpaceV
GROUP BY DatabaseName
ORDER BY "Used_GB" DESC;

-- Table sizes in a database
SELECT DatabaseName, TableName,
    CAST(SUM(CurrentPerm)/(1024*1024*1024) AS DECIMAL(18,5)) AS "Size_GB"
FROM DBC.TableSizeV
WHERE DatabaseName = '{database}'
GROUP BY DatabaseName, TableName
ORDER BY "Size_GB" DESC;

-- Spool and temp space by database
SELECT DatabaseName,
    CAST(SUM(MaxSpool)/(1024*1024*1024) AS DECIMAL(18,2)) AS "MaxSpool_GB",
    CAST(SUM(CurrentSpool)/(1024*1024*1024) AS DECIMAL(18,2)) AS "CurrSpool_GB",
    CAST(SUM(MaxTemp)/(1024*1024*1024) AS DECIMAL(18,2)) AS "MaxTemp_GB",
    CAST(SUM(CurrentTemp)/(1024*1024*1024) AS DECIMAL(18,2)) AS "CurrTemp_GB"
FROM DBC.DiskSpaceV
GROUP BY DatabaseName
ORDER BY "CurrSpool_GB" DESC;
```

## Detailed References

- **[Query Performance](references/query-performance.md)**: DBQL analysis, top consumers, execution time, CPU/IO skew
- **[Resource Usage](references/resource-usage.md)**: CPU, memory, I/O monitoring via ResUsage tables
- **[Space Management](references/space-management.md)**: Disk space, spool, skew analysis, table-level details
- **[Security & Access](references/security-access.md)**: Access rights, roles, logon history, session auditing
- **[VantageCloud Lake](references/vantagecloud-lake.md)**: Compute cluster monitoring, scaling events
- **[System Views Reference](references/system-views.md)**: Complete DBC view catalog with columns and purposes

## Key DBC Views for Monitoring

| View | Purpose |
|------|---------|
| `DBC.DBCInfoV` | System version and configuration |
| `DBC.SessionInfoV` | Active session details |
| `DBC.DiskSpaceV` | Disk space by database/AMP |
| `DBC.TableSizeV` | Table storage by AMP |
| `DBC.AllSpaceV` | Combined space view |
| `DBC.QryLogV` | Query log (DBQL main view) |
| `DBC.QryLogStepsV` | Query step details |
| `DBC.QryLogSQLV` | Full SQL text for logged queries |
| `DBC.QryLogObjV` | Objects accessed by queries |
| `DBC.ResUsageSpma` | Node-level resource usage |
| `DBC.ResUsageSvpr` | Vproc-level resource usage |
| `DBC.ResUsageScpu` | CPU resource usage |
| `DBC.AMPUsage` | AMP usage by user/account |
| `DBC.LogOnOffV` | Logon/logoff audit trail |
| `DBC.AllRightsV` | Access rights audit |
| `DBC.ErrorMsgs` | Error code lookup |

## Monitor Functions (require EXECUTE FUNCTION privileges)

| Function | Purpose |
|----------|---------|
| `MonitorSession(HostId, User, SessionNo)` | List active sessions |
| `MonitorSQLText(HostId, SessionNo, VprocNo)` | Get running SQL text |
| `MonitorSQLCurrentStep(HostId, SessionNo, VprocNo)` | Query step progress |
| `MonitorPhysicalResource()` | Physical resource metrics |
| `MonitorVirtualResource()` | Virtual resource metrics |

## Guidelines

- Always use views (suffix `V`) over base tables for portability
- ResUsage collection is off by default; verify it's enabled before querying
- DBQL logging must be enabled to get query performance data
- MonitorSession functions require EXECUTE FUNCTION privileges on SYSLIB
- Use `NULLIFZERO()` to avoid division-by-zero in skew calculations
- For large DBQL tables, always filter by date to avoid full scans
- Prefer `PDCRINFO.DBQLogTbl_Hst` for historical DBQL if available
