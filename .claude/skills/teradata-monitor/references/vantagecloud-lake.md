# VantageCloud Lake Monitoring

Queries specific to Teradata VantageCloud Lake for monitoring compute clusters, scaling events, and cloud resource management.

## Compute Cluster Status

```sql
-- Current status of all compute groups and clusters
SELECT *
FROM DBC.ComputeGroupsInfoV
ORDER BY ComputeGroupName;

-- Detailed compute cluster status (state, scaling)
SELECT *
FROM DBC.ComputeGroupStatusDetailsV
ORDER BY ComputeGroupName;
```

## Compute Cluster States

| State | Description |
|-------|-------------|
| `Running` | Cluster is active and processing queries |
| `Suspended` | Cluster is paused, not consuming compute |
| `Suspending` | Cluster is transitioning to suspended |
| `Resuming` | Cluster is transitioning to running |
| `Scaling` | Cluster is adding/removing capacity |

## Compute Cluster Events

```sql
-- Monitor compute cluster lifecycle events
SELECT *
FROM DBC.ComputeClusterEventsV
ORDER BY EventTime DESC;
```

## DBQL with Compute Context

In VantageCloud Lake, DBQL queries can be filtered by compute group:

```sql
-- Query performance by compute cluster (if available)
SELECT
    UserName,
    COUNT(*) AS Query_Count,
    CAST(SUM(AMPCPUTime) AS DECIMAL(18,2)) AS Total_CPU,
    SUM(TotalIOCount) AS Total_IO
FROM DBC.QryLogV
WHERE CAST(StartTime AS DATE) = CURRENT_DATE
GROUP BY UserName
ORDER BY Total_CPU DESC;
```

## Lake-Specific Views

VantageCloud Lake introduces additional system views beyond traditional Teradata:

| View | Purpose |
|------|---------|
| `DBC.ComputeGroupsInfoV` | Compute group configuration |
| `DBC.ComputeGroupStatusDetailsV` | Compute cluster state details |
| `DBC.ComputeClusterEventsV` | Cluster lifecycle events |

## Notes

- Compute cluster views are only available on VantageCloud Lake deployments
- Traditional DBC monitoring views (SessionInfoV, DiskSpaceV, QryLogV, etc.) remain available
- Compute cluster management can also be done programmatically via SQL:
  - `SUSPEND COMPUTE FOR {compute_group}` - Suspend a cluster
  - `RESUME COMPUTE FOR {compute_group}` - Resume a cluster
- Monitor cluster state transitions to track auto-scaling behavior
