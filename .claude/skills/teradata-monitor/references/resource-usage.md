# Resource Usage Monitoring

CPU, memory, and I/O monitoring using ResUsage tables and system functions.

## Prerequisites

Resource usage collection is **off by default**. It must be enabled via the `ctl` utility (RSS screen) or `rssctl` command. Verify collection is active before relying on these queries.

## ResUsage Tables Overview

| Table | Scope | Key Metrics |
|-------|-------|-------------|
| `DBC.ResUsageSpma` | Node-level summary | CPU, I/O, memory, processes, bynet |
| `DBC.ResUsageSvpr` | Vproc-level (AMP/PE) | CPU, I/O per vproc |
| `DBC.ResUsageScpu` | CPU detail per node | CPU usage breakdown |
| `DBC.ResUsageShst` | Host-level stats | System-wide aggregates |
| `DBC.ResCpuUsageByAmpView` | AMP CPU breakdown | CPU by AMP and node |

## Node-Level CPU Monitoring

```sql
-- Average and peak CPU usage per node (last hour)
SELECT
    TheDate,
    TheTime,
    NodeID,
    CAST(CPUUServ / NULLIFZERO(NCPUs * Secs) * 100 AS DECIMAL(5,2)) AS User_CPU_Pct,
    CAST(CPUUExec / NULLIFZERO(NCPUs * Secs) * 100 AS DECIMAL(5,2)) AS System_CPU_Pct,
    CAST(CPUIoWait / NULLIFZERO(NCPUs * Secs) * 100 AS DECIMAL(5,2)) AS IOWait_CPU_Pct,
    CAST(CPUIdle / NULLIFZERO(NCPUs * Secs) * 100 AS DECIMAL(5,2)) AS Idle_CPU_Pct
FROM DBC.ResUsageSpma
WHERE TheDate = CURRENT_DATE
    AND TheTime >= CURRENT_TIME - INTERVAL '1' HOUR
ORDER BY NodeID, TheTime;
```

## Node CPU Skew Detection

```sql
-- Identify CPU skew across nodes
SELECT
    TheDate,
    TheTime,
    MAX(CPUUServ) AS MaxNodeCPU,
    AVG(CPUUServ) AS AvgNodeCPU,
    CAST((MAX(CPUUServ) - AVG(CPUUServ)) / NULLIFZERO(MAX(CPUUServ)) * 100
        AS DECIMAL(5,2)) AS NodeCPUSkew_Pct
FROM DBC.ResUsageSpma
WHERE TheDate = CURRENT_DATE
GROUP BY TheDate, TheTime
HAVING NodeCPUSkew_Pct > 20
ORDER BY TheTime DESC;
```

## AMP-Level Resource Usage

```sql
-- AMP CPU usage distribution
SELECT
    NodeID,
    Vproc AS AMP_ID,
    CAST(AmpCPUUServ / NULLIFZERO(Secs) * 100 AS DECIMAL(5,2)) AS AMP_UserCPU_Pct,
    CAST(AmpCPUUExec / NULLIFZERO(Secs) * 100 AS DECIMAL(5,2)) AS AMP_SysCPU_Pct,
    FileAcqs AS Disk_Reads
FROM DBC.ResUsageSvpr
WHERE TheDate = CURRENT_DATE
    AND TheTime >= CURRENT_TIME - INTERVAL '1' HOUR
    AND VprocType = 'AMP'
ORDER BY AMP_UserCPU_Pct DESC;
```

## AMP I/O Skew Detection

```sql
-- I/O skew across AMPs
SELECT
    TheDate,
    TheTime,
    MAX(FileAcqs) AS Max_AMP_IO,
    AVG(FileAcqs) AS Avg_AMP_IO,
    CAST((MAX(FileAcqs) - AVG(FileAcqs)) / NULLIFZERO(MAX(FileAcqs)) * 100
        AS DECIMAL(5,2)) AS AMP_IOSkew_Pct
FROM DBC.ResUsageSvpr
WHERE TheDate = CURRENT_DATE
    AND VprocType = 'AMP'
GROUP BY TheDate, TheTime
HAVING AMP_IOSkew_Pct > 20
ORDER BY TheTime DESC;
```

## Memory Monitoring

```sql
-- Node memory usage (from ResUsageSpma)
SELECT
    TheDate,
    TheTime,
    NodeID,
    MemSize AS Total_Memory_KB,
    MemFreeKB AS Free_Memory_KB,
    CAST((1 - MemFreeKB / NULLIFZERO(MemSize * 1.0)) * 100
        AS DECIMAL(5,2)) AS Memory_Used_Pct,
    MemCtxtPageReads AS Context_Page_Reads,
    MemCtxtPageWrites AS Context_Page_Writes
FROM DBC.ResUsageSpma
WHERE TheDate = CURRENT_DATE
    AND TheTime >= CURRENT_TIME - INTERVAL '1' HOUR
ORDER BY NodeID, TheTime;
```

## AMP Worker Task (AWT) Usage

```sql
-- AWT availability (from ResUsageSvpr)
SELECT
    TheDate,
    TheTime,
    NodeID,
    Vproc AS AMP_ID,
    InUseMax AS AWT_InUse_Max,
    InUseAvg AS AWT_InUse_Avg
FROM DBC.ResUsageSvpr
WHERE TheDate = CURRENT_DATE
    AND TheTime >= CURRENT_TIME - INTERVAL '1' HOUR
    AND VprocType = 'AMP'
    AND InUseMax > 0
ORDER BY InUseMax DESC;
```

## User/Account Resource Consumption

```sql
-- Resource usage by user and account (DBC.AMPUsage)
SELECT
    UserName,
    AccountName,
    CAST(SUM(ElapsedTime) AS DECIMAL(18,2)) AS Total_Elapsed,
    CAST(SUM(CpuTime) AS DECIMAL(18,2)) AS Total_CPU_Secs,
    SUM(DiskIO) AS Total_DiskIO,
    SUM(LogicalIO) AS Total_LogicalIO
FROM DBC.AMPUsage
GROUP BY UserName, AccountName
ORDER BY Total_CPU_Secs DESC;
```

## I/O Throttle Monitoring

```sql
-- Check for I/O throttling events
SELECT
    TheDate,
    TheTime,
    NodeID,
    IOThrottleCount,
    IOThrottleTime,
    IOThrottleTimeMax
FROM DBC.ResUsageSpma
WHERE TheDate = CURRENT_DATE
    AND IOThrottleCount > 0
ORDER BY IOThrottleCount DESC;
```

## ResUsageSpma Key Columns

| Column | Description |
|--------|-------------|
| `NodeID` | Physical node identifier |
| `TheDate` / `TheTime` | Sampling timestamp |
| `NCPUs` | Number of CPUs on node |
| `Secs` | Sampling interval (seconds) |
| `CPUUServ` | User-mode CPU time |
| `CPUUExec` | System-mode CPU time |
| `CPUIoWait` | CPU time waiting for I/O |
| `CPUIdle` | Idle CPU time |
| `MemSize` | Total memory (KB) |
| `MemFreeKB` | Free memory (KB) |
| `MemCtxtPageReads` | Context page-in operations |
| `IOThrottleCount` | I/O throttle occurrences |
| `IOThrottleTime` | Total throttle time (ms) |
| `VProcQty` | Number of vprocs on node |

## ResUsageSvpr Key Columns

| Column | Description |
|--------|-------------|
| `Vproc` | Vproc number |
| `VprocType` | AMP, PE, GTW, etc. |
| `AmpCPUUServ` | User CPU for this vproc |
| `AmpCPUUExec` | System CPU for this vproc |
| `FileAcqs` | Disk read operations |
| `InUseMax` | Max AWT in use |
| `InUseAvg` | Avg AWT in use |
