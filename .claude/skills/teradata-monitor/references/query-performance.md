# Query Performance Monitoring

DBQL (Database Query Log) analysis for identifying top consumers, slow queries, and performance bottlenecks.

## Prerequisites

DBQL logging must be enabled. Verify and enable with:

```sql
-- Check current logging rules
SELECT * FROM DBC.DBQLRulesV;

-- Enable comprehensive logging (run as DBA)
BEGIN QUERY LOGGING WITH SQL, OBJECTS LIMIT SQLTEXT=0 ON ALL;
```

## Top CPU Consumers

```sql
-- Top 50 queries by CPU consumption (last 24 hours)
SELECT TOP 50
    QueryId,
    UserName,
    StartTime,
    FirstRespTime,
    AMPCPUTime,
    MaxAMPCPUTime,
    AMPCPUTime * (HASHAMP()+1) AS TotalCPU_Estimate,
    ParserCPUTime,
    TotalIOCount,
    ReqPhysIO,
    QueryText
FROM DBC.QryLogV
WHERE StartTime >= CURRENT_TIMESTAMP - INTERVAL '24' HOUR
    AND AMPCPUTime > 0
ORDER BY AMPCPUTime DESC;
```

## Top I/O Consumers

```sql
-- Top 50 queries by I/O (last 24 hours)
SELECT TOP 50
    QueryId,
    UserName,
    StartTime,
    TotalIOCount,
    ReqPhysIO,
    ReqPhysIOKB,
    AMPCPUTime,
    SpoolUsage,
    QueryText
FROM DBC.QryLogV
WHERE StartTime >= CURRENT_TIMESTAMP - INTERVAL '24' HOUR
    AND TotalIOCount > 0
ORDER BY TotalIOCount DESC;
```

## Slowest Queries (Elapsed Time)

```sql
-- Top 50 longest-running queries (last 24 hours)
SELECT TOP 50
    QueryId,
    UserName,
    StartTime,
    FirstRespTime,
    CAST(EXTRACT(HOUR FROM ((FirstRespTime - StartTime)
        HOUR(3) TO SECOND(6))) * 3600
        + EXTRACT(MINUTE FROM ((FirstRespTime - StartTime)
        HOUR(3) TO SECOND(6))) * 60
        + EXTRACT(SECOND FROM ((FirstRespTime - StartTime)
        HOUR(3) TO SECOND(6)))
        AS DECIMAL(10,2)) AS Elapsed_Secs,
    AMPCPUTime,
    TotalIOCount,
    DelayTime,
    LockDelay,
    QueryText
FROM DBC.QryLogV
WHERE StartTime >= CURRENT_TIMESTAMP - INTERVAL '24' HOUR
    AND FirstRespTime IS NOT NULL
ORDER BY Elapsed_Secs DESC;
```

## CPU Skew Analysis

High CPU skew indicates poor data distribution or suboptimal query plans.

```sql
-- Queries with high CPU skew (>80%)
SELECT
    QueryId,
    UserName,
    StartTime,
    AMPCPUTime,
    MaxAMPCPUTime,
    NumOfActiveAMPs,
    CAST(100 - (NULLIFZERO(AMPCPUTime / NULLIFZERO(NumOfActiveAMPs))
        / NULLIFZERO(MaxAMPCPUTime) * 100)
        AS DECIMAL(5,2)) AS CPUSkew_Pct,
    QueryText
FROM DBC.QryLogV
WHERE StartTime >= CURRENT_TIMESTAMP - INTERVAL '24' HOUR
    AND AMPCPUTime > 1  -- at least 1 second total CPU
    AND MaxAMPCPUTime > 0
    AND (100 - (NULLIFZERO(AMPCPUTime / NULLIFZERO(NumOfActiveAMPs))
        / NULLIFZERO(MaxAMPCPUTime) * 100)) > 80
ORDER BY CPUSkew_Pct DESC;
```

## Comprehensive Query Diagnostics

```sql
-- Detailed query analysis with all key metrics
SELECT
    QueryId,
    SessionId,
    UserName,
    StartTime,
    FirstRespTime,
    -- Timing
    DelayTime,
    LockDelay,
    -- CPU
    AMPCPUTime,
    MaxAMPCPUTime,
    ParserCPUTime,
    AMPCPUTime + ParserCPUTime AS TotalCPU,
    -- CPU Skew
    CAST(100 - (NULLIFZERO(AMPCPUTime / NULLIFZERO(NumOfActiveAMPs))
        / NULLIFZERO(MaxAMPCPUTime) * 100)
        AS DECIMAL(5,2)) AS CPUSkew_Pct,
    -- I/O
    TotalIOCount,
    ReqPhysIO,
    ReqPhysIOKB,
    -- Spool
    SpoolUsage,
    CAST(SpoolUsage / (1024*1024*1024.0) AS DECIMAL(18,2)) AS Spool_GB,
    -- Processing
    NumOfActiveAMPs,
    NumResultRows,
    -- Errors
    ErrorCode,
    -- Classification
    StatementType,
    QueryBand,
    AppID,
    -- SQL
    QueryText
FROM DBC.QryLogV
WHERE StartTime >= CURRENT_TIMESTAMP - INTERVAL '24' HOUR
ORDER BY AMPCPUTime DESC;
```

## Query Volume and Throughput

```sql
-- Hourly query volume and resource consumption
SELECT
    EXTRACT(HOUR FROM StartTime) AS Hour_Of_Day,
    COUNT(*) AS Query_Count,
    CAST(SUM(AMPCPUTime) AS DECIMAL(18,2)) AS Total_AMP_CPU,
    CAST(AVG(AMPCPUTime) AS DECIMAL(18,4)) AS Avg_AMP_CPU,
    SUM(TotalIOCount) AS Total_IO,
    SUM(CASE WHEN ErrorCode <> 0 THEN 1 ELSE 0 END) AS Error_Count
FROM DBC.QryLogV
WHERE CAST(StartTime AS DATE) = CURRENT_DATE
GROUP BY 1
ORDER BY 1;
```

## User Activity Summary

```sql
-- Resource consumption by user (today)
SELECT
    UserName,
    COUNT(*) AS Query_Count,
    CAST(SUM(AMPCPUTime) AS DECIMAL(18,2)) AS Total_CPU,
    SUM(TotalIOCount) AS Total_IO,
    CAST(MAX(SpoolUsage)/(1024*1024*1024.0) AS DECIMAL(18,2)) AS Max_Spool_GB,
    SUM(CASE WHEN ErrorCode <> 0 THEN 1 ELSE 0 END) AS Errors
FROM DBC.QryLogV
WHERE CAST(StartTime AS DATE) = CURRENT_DATE
GROUP BY UserName
ORDER BY Total_CPU DESC;
```

## Query with Full SQL Text

The `QueryText` column in `DBC.QryLogV` is truncated. For full SQL:

```sql
-- Get full SQL text for a specific query
SELECT q.QueryId, q.UserName, q.StartTime, q.AMPCPUTime,
    s.SqlTextInfo
FROM DBC.QryLogV q
JOIN DBC.QryLogSQLV s ON q.QueryId = s.QueryId
    AND q.ProcId = s.ProcId
WHERE q.QueryId = {query_id}
ORDER BY s.SqlRowNo;
```

## Objects Accessed by Query

```sql
-- Tables/views accessed by a specific query
SELECT o.QueryId, o.ObjectDatabaseName, o.ObjectTableName,
    o.ObjectType, o.FreqOfUse, o.TypeOfUse
FROM DBC.QryLogObjV o
WHERE o.QueryId = {query_id}
ORDER BY o.ObjectDatabaseName, o.ObjectTableName;
```

## DBQL Key Columns Reference

| Column | Description |
|--------|-------------|
| `QueryId` | Unique query identifier |
| `SessionId` | Session that ran the query |
| `UserName` | User who submitted the query |
| `StartTime` | Query submission timestamp |
| `FirstRespTime` | First response to client |
| `AMPCPUTime` | Total AMP CPU seconds |
| `MaxAMPCPUTime` | Max CPU on any single AMP |
| `ParserCPUTime` | Parser/optimizer CPU |
| `TotalIOCount` | Total logical I/O count |
| `ReqPhysIO` | Physical I/O count |
| `ReqPhysIOKB` | Physical I/O in KB |
| `SpoolUsage` | Peak spool bytes used |
| `NumOfActiveAMPs` | AMPs involved in query |
| `NumResultRows` | Rows returned |
| `ErrorCode` | Error code (0 = success) |
| `StatementType` | SQL statement type |
| `QueryBand` | Application-set query band |
| `DelayTime` | Total delay before execution |
| `LockDelay` | Time waiting for locks |
| `QueryText` | Truncated SQL text |
