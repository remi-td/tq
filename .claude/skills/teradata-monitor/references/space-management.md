# Space Management Monitoring

Disk space, spool space, and data distribution monitoring.

## Database Space Overview

```sql
-- All databases with space allocation and usage
SELECT
    DatabaseName,
    CAST(SUM(MaxPerm)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Perm_Alloc_GB",
    CAST(SUM(CurrentPerm)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Perm_Used_GB",
    CAST((1 - SUM(CurrentPerm)/NULLIFZERO(SUM(MaxPerm)))*100
        AS DECIMAL(5,2)) AS "Perm_Free_Pct",
    CAST(SUM(MaxSpool)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Spool_Limit_GB",
    CAST(SUM(CurrentSpool)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Spool_Used_GB",
    CAST(SUM(MaxTemp)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Temp_Limit_GB",
    CAST(SUM(CurrentTemp)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Temp_Used_GB"
FROM DBC.DiskSpaceV
GROUP BY DatabaseName
ORDER BY "Perm_Used_GB" DESC;
```

## Databases Running Low on Space

```sql
-- Databases with less than 10% free permanent space
SELECT
    DatabaseName,
    CAST(SUM(MaxPerm)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Alloc_GB",
    CAST(SUM(CurrentPerm)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Used_GB",
    CAST((SUM(CurrentPerm)/NULLIFZERO(SUM(MaxPerm)))*100
        AS DECIMAL(5,2)) AS "Used_Pct"
FROM DBC.DiskSpaceV
GROUP BY DatabaseName
HAVING "Used_Pct" > 90
ORDER BY "Used_Pct" DESC;
```

## Table-Level Space Analysis

```sql
-- Largest tables in a database
SELECT
    DatabaseName,
    TableName,
    CAST(SUM(CurrentPerm)/(1024*1024*1024) AS DECIMAL(18,5)) AS "Size_GB",
    COUNT(*) AS AMP_Count,
    MAX(CurrentPerm) AS Max_AMP_Bytes,
    MIN(CurrentPerm) AS Min_AMP_Bytes,
    CAST((MAX(CurrentPerm) - AVG(CurrentPerm))
        / NULLIFZERO(MAX(CurrentPerm)) * 100
        AS DECIMAL(5,2)) AS "DataSkew_Pct"
FROM DBC.TableSizeV
WHERE DatabaseName = '{database}'
GROUP BY DatabaseName, TableName
ORDER BY "Size_GB" DESC;
```

## System-Wide Largest Tables

```sql
-- Top 50 largest tables across all databases
SELECT TOP 50
    DatabaseName,
    TableName,
    CAST(SUM(CurrentPerm)/(1024*1024*1024) AS DECIMAL(18,3)) AS "Size_GB"
FROM DBC.TableSizeV
GROUP BY DatabaseName, TableName
ORDER BY "Size_GB" DESC;
```

## Data Skew Analysis

```sql
-- Tables with high data skew (>30%)
SELECT
    DatabaseName,
    TableName,
    CAST(SUM(CurrentPerm)/(1024*1024*1024) AS DECIMAL(18,3)) AS "Size_GB",
    MAX(CurrentPerm) AS Max_AMP_Bytes,
    AVG(CurrentPerm) AS Avg_AMP_Bytes,
    CAST((MAX(CurrentPerm) - AVG(CurrentPerm))
        / NULLIFZERO(MAX(CurrentPerm)) * 100
        AS DECIMAL(5,2)) AS "Skew_Pct"
FROM DBC.TableSizeV
GROUP BY DatabaseName, TableName
HAVING "Skew_Pct" > 30 AND "Size_GB" > 0.1
ORDER BY "Skew_Pct" DESC;
```

## Row Distribution Skew (requires table scan)

```sql
-- Row-level skew for a specific table using PI hash
SELECT
    SUM(cnt) AS Total_Rows,
    COUNT(*) AS Total_AMPs,
    MAX(cnt) AS Max_Rows_Per_AMP,
    MIN(cnt) AS Min_Rows_Per_AMP,
    CAST(AVG(cnt) AS DECIMAL(18,0)) AS Avg_Rows_Per_AMP,
    CAST((MAX(cnt) - AVG(cnt)) / NULLIFZERO(MAX(cnt)) * 100
        AS DECIMAL(5,2)) AS "RowSkew_Pct"
FROM (
    SELECT HASHAMP(HASHBUCKET(HASHROW({primary_index_columns}))),
        COUNT(*) AS cnt
    FROM {database}.{table}
    GROUP BY 1
) dt(amp, cnt);
```

## Spool Space Usage

```sql
-- Current spool consumers (from DiskSpaceV)
SELECT
    DatabaseName,
    CAST(SUM(MaxSpool)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Limit_GB",
    CAST(SUM(CurrentSpool)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Used_GB",
    CAST(SUM(CurrentSpool)/NULLIFZERO(SUM(MaxSpool))*100
        AS DECIMAL(5,2)) AS "Used_Pct"
FROM DBC.DiskSpaceV
WHERE CurrentSpool > 0
GROUP BY DatabaseName
ORDER BY "Used_GB" DESC;

-- Peak spool by AMP (identify hot spots)
SELECT
    DatabaseName,
    Vproc AS AMP_ID,
    CAST(PeakSpool/(1024*1024*1024) AS DECIMAL(18,3)) AS "PeakSpool_GB",
    CAST(CurrentSpool/(1024*1024*1024) AS DECIMAL(18,3)) AS "CurrSpool_GB",
    CAST(MaxSpool/(1024*1024*1024) AS DECIMAL(18,3)) AS "MaxSpool_GB"
FROM DBC.DiskSpaceV
WHERE PeakSpool > 0
ORDER BY PeakSpool DESC;
```

## Temp Space Usage

```sql
-- Current temp space consumers
SELECT
    DatabaseName,
    CAST(SUM(MaxTemp)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Limit_GB",
    CAST(SUM(CurrentTemp)/(1024*1024*1024) AS DECIMAL(18,2)) AS "Used_GB"
FROM DBC.DiskSpaceV
WHERE CurrentTemp > 0
GROUP BY DatabaseName
ORDER BY "Used_GB" DESC;
```

## DiskSpaceV Key Columns

| Column | Description |
|--------|-------------|
| `DatabaseName` | Database/user name |
| `TableName` | Object name (or database aggregate) |
| `Vproc` | AMP number |
| `MaxPerm` | Permanent space limit (bytes) |
| `CurrentPerm` | Permanent space used (bytes) |
| `MaxSpool` | Spool space limit (bytes) |
| `CurrentSpool` | Spool space currently used |
| `PeakSpool` | Peak spool usage since logon |
| `MaxTemp` | Temp space limit (bytes) |
| `CurrentTemp` | Temp space currently used |
| `PeakTemp` | Peak temp usage |
