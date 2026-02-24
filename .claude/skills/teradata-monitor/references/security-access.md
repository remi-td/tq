# Security and Access Monitoring

Queries for auditing access rights, monitoring logon activity, and reviewing security posture.

## Logon/Logoff Activity

```sql
-- Recent logon/logoff events
SELECT
    LogDate,
    LogTime,
    UserName,
    Event,
    LogonSource,
    SessionNo,
    IFPNo
FROM DBC.LogOnOffV
WHERE LogDate >= CURRENT_DATE - 7
ORDER BY LogDate DESC, LogTime DESC;

-- Failed logon attempts (security audit)
SELECT
    LogDate,
    LogTime,
    UserName,
    Event,
    LogonSource
FROM DBC.LogOnOffV
WHERE Event LIKE '%Failed%'
    AND LogDate >= CURRENT_DATE - 7
ORDER BY LogDate DESC, LogTime DESC;

-- Logon frequency by user (last 7 days)
SELECT
    UserName,
    COUNT(CASE WHEN Event = 'Logon' THEN 1 END) AS Logon_Count,
    COUNT(CASE WHEN Event = 'Logoff' THEN 1 END) AS Logoff_Count,
    MIN(LogDate) AS First_Activity,
    MAX(LogDate) AS Last_Activity
FROM DBC.LogOnOffV
WHERE LogDate >= CURRENT_DATE - 7
GROUP BY UserName
ORDER BY Logon_Count DESC;
```

## Access Rights Audit

```sql
-- All rights for a specific user
SELECT
    UserName,
    DatabaseName,
    TableName,
    ColumnName,
    AccessRight,
    GrantAuthority,
    GrantorName,
    CreateTimeStamp
FROM DBC.AllRightsV
WHERE UserName = '{username}'
ORDER BY DatabaseName, TableName;

-- Users with DBA-level access
SELECT DISTINCT
    UserName,
    DatabaseName,
    AccessRight,
    GrantorName
FROM DBC.AllRightsV
WHERE AccessRight IN ('AE', 'GR', 'CT', 'DP', 'PC')  -- admin privileges
ORDER BY UserName, DatabaseName;

-- All users who can access a specific table
SELECT
    UserName,
    AccessRight,
    GrantAuthority,
    GrantorName
FROM DBC.AllRightsV
WHERE DatabaseName = '{database}'
    AND TableName = '{table}'
ORDER BY UserName;
```

## Access Right Codes

| Code | Right |
|------|-------|
| `R` | SELECT (Read) |
| `I` | INSERT |
| `U` | UPDATE |
| `D` | DELETE |
| `CT` | CREATE TABLE |
| `CD` | CREATE DATABASE |
| `CV` | CREATE VIEW |
| `CM` | CREATE MACRO |
| `CP` | CREATE PROCEDURE |
| `CF` | CREATE FUNCTION |
| `DP` | DROP (any object) |
| `AE` | ALTER EXTERNAL PROCEDURE |
| `EF` | EXECUTE FUNCTION |
| `GR` | GRANT/REVOKE |
| `PC` | CREATE PROFILE/ROLE |
| `SH` | SHOW |
| `SS` | SET SESSION |
| `MR` | MONITOR RESOURCE |
| `AS` | ABORT SESSION |

## Role Membership

```sql
-- All roles and their members
SELECT
    RoleName,
    Grantee AS MemberName,
    GrantorName,
    WhenGranted,
    DefaultRole
FROM DBC.RoleMembersV
ORDER BY RoleName, MemberName;

-- Roles assigned to a specific user
SELECT
    RoleName,
    GrantorName,
    WhenGranted,
    DefaultRole
FROM DBC.RoleMembersV
WHERE Grantee = '{username}'
ORDER BY RoleName;

-- Permissions granted to a role
SELECT
    RoleName,
    DatabaseName,
    TableName,
    AccessRight,
    GrantAuthority
FROM DBC.AllRoleRightsV
WHERE RoleName = '{role}'
ORDER BY DatabaseName, TableName;
```

## Database/User Space Quotas

```sql
-- User and database space allocations
SELECT
    DatabaseName,
    DBKind,
    CreatorName,
    OwnerName,
    CAST(PermSpace/(1024*1024*1024) AS DECIMAL(18,2)) AS "PermSpace_GB",
    CAST(SpoolSpace/(1024*1024*1024) AS DECIMAL(18,2)) AS "SpoolSpace_GB",
    CAST(TempSpace/(1024*1024*1024) AS DECIMAL(18,2)) AS "TempSpace_GB",
    CommentString,
    CreateTimeStamp
FROM DBC.DatabasesV
ORDER BY "PermSpace_GB" DESC;
```

## Session Auditing

```sql
-- Currently connected users with their source IP and application
SELECT
    SessionNo,
    UserName,
    ClientIpAddress,
    ClientProgramName,
    ClientSystemUserId,
    ClientOsName,
    LogonDate,
    LogonTime
FROM DBC.SessionInfoV
ORDER BY UserName;

-- Users with multiple concurrent sessions
SELECT
    UserName,
    COUNT(*) AS Session_Count,
    MIN(LogonDate) AS Earliest_Logon
FROM DBC.SessionInfoV
GROUP BY UserName
HAVING Session_Count > 1
ORDER BY Session_Count DESC;
```

## Error Code Lookup

```sql
-- Look up Teradata error message by code
SELECT ErrorCode, ErrorText
FROM DBC.ErrorMsgs
WHERE ErrorCode = {error_code};

-- Search error messages by keyword
SELECT ErrorCode, ErrorText
FROM DBC.ErrorMsgs
WHERE ErrorText LIKE '%{keyword}%'
ORDER BY ErrorCode;
```
