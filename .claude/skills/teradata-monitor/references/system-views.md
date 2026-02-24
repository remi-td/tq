# System Views Reference

Complete catalog of DBC views relevant to system monitoring, organized by category.

## System Information

| View | Description | Key Columns |
|------|-------------|-------------|
| `DBC.DBCInfoV` | System version, release, configuration | `InfoKey`, `InfoData` |
| `DBC.DatabasesV` | All databases/users with space quotas | `DatabaseName`, `DBKind`, `PermSpace`, `SpoolSpace`, `TempSpace` |
| `DBC.ErrorMsgs` | Error code to message mapping | `ErrorCode`, `ErrorText` |

## Session and Connection Monitoring

| View | Description | Key Columns |
|------|-------------|-------------|
| `DBC.SessionInfoV` | Active sessions with client details | `SessionNo`, `UserName`, `ClientIpAddress`, `ClientProgramName`, `Transaction_Mode` |
| `DBC.SessionInfoX` | Extended session info (user-owned only) | Same as SessionInfoV with access restrictions |
| `DBC.LogOnOffV` | Logon/logoff audit trail | `LogDate`, `LogTime`, `UserName`, `Event`, `LogonSource` |

## Space and Storage

| View | Description | Key Columns |
|------|-------------|-------------|
| `DBC.DiskSpaceV` | Disk space by database and AMP | `DatabaseName`, `Vproc`, `MaxPerm`, `CurrentPerm`, `MaxSpool`, `CurrentSpool` |
| `DBC.AllSpaceV` | Combined space view | `DatabaseName`, `TableName`, `CurrentPerm`, `MaxPerm` |
| `DBC.TableSizeV` | Table storage by AMP | `DatabaseName`, `TableName`, `Vproc`, `CurrentPerm` |

## Query Logging (DBQL)

| View | Description | Key Columns |
|------|-------------|-------------|
| `DBC.QryLogV` | Main query log (1 row/query) | `QueryId`, `UserName`, `StartTime`, `AMPCPUTime`, `TotalIOCount`, `QueryText` |
| `DBC.QryLogStepsV` | Query step details | `QueryId`, `StepNum`, `StepName`, `CPUTime`, `IOCount` |
| `DBC.QryLogSQLV` | Full SQL text | `QueryId`, `SqlRowNo`, `SqlTextInfo` |
| `DBC.QryLogObjV` | Objects accessed by query | `QueryId`, `ObjectDatabaseName`, `ObjectTableName`, `TypeOfUse` |
| `DBC.QryLogExplainV` | Explain plans | `QueryId`, `ExplainText` |
| `DBC.QryLogUtilityV` | Utility (load/export) logs | `QueryId`, `UtilityName` |
| `DBC.QryLogEventHisV` | Query event history | Event timelines for queries |
| `DBC.DBQLRulesV` | Active DBQL logging rules | Rule definitions |

### DBQL Base Tables

| Table | Description |
|-------|-------------|
| `DBC.DBQLogTbl` | Main query log table |
| `DBC.DBQLObjTbl` | Object usage log |
| `DBC.DBQLSqlTbl` | Full SQL text storage |
| `DBC.DBQLStepTbl` | Step-level details |
| `DBC.DBQLExplainTbl` | Explain plan storage |
| `DBC.DBQLUtilityTbl` | Utility operation log |

### Historical DBQL (PDCR)

If Performance Data Collection and Reporting is configured:

| Table | Description |
|-------|-------------|
| `PDCRINFO.DBQLogTbl_Hst` | Historical query log |
| `PDCRINFO.DBQLObjTbl_Hst` | Historical object log |
| `PDCRINFO.DBQLSqlTbl_Hst` | Historical SQL text |

## Resource Usage

| Table | Description | Key Columns |
|-------|-------------|-------------|
| `DBC.ResUsageSpma` | Node-level summary | `NodeID`, `CPUUServ`, `CPUUExec`, `MemSize`, `MemFreeKB` |
| `DBC.ResUsageSvpr` | Vproc-level detail | `Vproc`, `VprocType`, `AmpCPUUServ`, `FileAcqs` |
| `DBC.ResUsageScpu` | CPU detail per node | `NodeID`, CPU breakdown columns |
| `DBC.ResUsageShst` | Host-level stats | System-wide aggregates |
| `DBC.ResCpuUsageByAmpView` | AMP CPU breakdown | `NodeID`, `Vproc`, CPU columns |
| `DBC.AMPUsage` | Usage by user/account | `UserName`, `AccountName`, `CpuTime`, `DiskIO` |

## Security and Access

| View | Description | Key Columns |
|------|-------------|-------------|
| `DBC.AllRightsV` | All access rights | `UserName`, `DatabaseName`, `TableName`, `AccessRight` |
| `DBC.UserRightsV` | Current user's rights | `DatabaseName`, `TableName`, `AccessRight` |
| `DBC.AllRoleRightsV` | Role-based rights | `RoleName`, `DatabaseName`, `AccessRight` |
| `DBC.UserRoleRightsV` | Current user's role rights | Same as AllRoleRightsV, filtered |
| `DBC.RoleMembersV` | Role membership | `RoleName`, `Grantee`, `DefaultRole` |

## Object Metadata

| View | Description | Key Columns |
|------|-------------|-------------|
| `DBC.TablesV` | Tables, views, macros, procs | `DatabaseName`, `TableName`, `TableKind`, `CreateTimeStamp` |
| `DBC.ColumnsV` | Column definitions | `DatabaseName`, `TableName`, `ColumnName`, `ColumnType` |
| `DBC.IndicesV` | Index definitions | `DatabaseName`, `TableName`, `IndexName`, `IndexType` |
| `DBC.ChildrenV` | Database hierarchy | `Parent`, `Child` |
| `DBC.FunctionsV` | UDF definitions | `DatabaseName`, `FunctionName` |

## TableKind Codes

| Code | Object Type |
|------|------------|
| `T` | Table |
| `V` | View |
| `M` | Macro |
| `P` | Stored Procedure |
| `F` | Standard Function |
| `R` | Table Function |
| `U` | User-Defined Type |
| `O` | Object (other) |
| `I` | Join Index |
| `N` | Hash Index |

## View Naming Conventions

- `DBC.XxxV` - Standard views (accessible to all users with appropriate privileges)
- `DBC.XxxVX` - Restricted views (return only objects owned by the current user)
- `DBC.XxxX` - Extended versions (additional columns or filtering)

## Notes

- Always prefer views (suffix `V`) over base tables for compatibility across versions
- X-suffixed views (`VX`) return only user-owned objects and are useful for non-DBA users
- ResUsage tables require data collection to be enabled
- DBQL views require query logging to be configured
- PDCRINFO tables require Performance Data Collection and Reporting setup
