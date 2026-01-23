# Error Handling and User Feedback

## Table of Contents

1. [Error Categories](#error-categories)
2. [Error Message Structure](#error-message-structure)
3. [Progress Indicators](#progress-indicators)
4. [Warnings](#warnings)
5. [Verbose Output](#verbose-output)

---

## Error Categories

### User Errors

**Definition**: Caused by incorrect user input
**Exit Code**: `2`
**Strategy**: Show what's wrong and how to fix it

**Examples**:
```bash
$ tq query
Error: Missing required argument: <QUERY>

Usage: tq query <QUERY>

For more information, try '--help'.
```

```bash
$ tq --logon "invalid" query "SELECT 1"
Error: Invalid connection string format

Expected format: user:password@host:port/database
Received: invalid

Examples:
  alice:secret@dbhost:1025/mydb
  alice@dbhost:1025/mydb  (password from file or prompt)
```

### Connection Errors

**Definition**: Cannot connect to database
**Exit Code**: `1`
**Strategy**: Diagnose and suggest solutions

**Example**:
```bash
$ tq ping
Error: Connection refused to myhost:1025

Possible causes:
  - Database is not running
  - Hostname or port is incorrect
  - Firewall is blocking connection
  - Network is unreachable

Troubleshooting steps:
  1. Verify hostname resolves: ping myhost
  2. Check port is open: nc -zv myhost 1025
  3. Confirm credentials: tq --logon "user:pass@myhost:1025/db" ping
  4. Check firewall rules
```

### Authentication Errors

**Definition**: Connection succeeds but authentication fails
**Exit Code**: `1`
**Strategy**: Distinguish auth types and suggest solutions

**Examples**:
```bash
$ tq query "SELECT 1"
Error: Authentication failed

Reason: Invalid username or password
Logon mechanism: TD2
User: alice
Host: myhost:1025

Troubleshooting:
  - Verify username and password are correct
  - Check if account is locked: contact DBA
  - Try different logon mechanism: --logmech LDAP
```

```bash
$ tq --logmech KRB5 query "SELECT 1"
Error: Kerberos authentication failed

Reason: No valid Kerberos ticket found

Troubleshooting:
  1. Check ticket status: klist
  2. Obtain ticket: kinit your-username
  3. Verify ticket: klist
  4. Retry command
```

### Query Errors

**Definition**: SQL syntax or execution error
**Exit Code**: `1`
**Strategy**: Show error with context, suppress internal stack traces

**Format:**
```
Error: [Short Error Type]

[User-Friendly Error Message]

Error Code: [Teradata Error Code]
Session ID: [Session Number]
```

**Key Principles:**
- Show only relevant information (message, error code, session ID)
- Suppress Go stack traces from Teradata driver (all "at gosqldriver/..." lines)
- Professional and actionable format
- Include metadata for debugging with DBAs

**Example 1: Syntax Error**
```bash
$ tq query "SELCT * FROM users"
Error: SQL syntax error

Expected something like a 'SELECT' keyword but found 'SELCT'.

Error Code: 3706
Session ID: 1429
```

**Example 2: Table Not Found**
```bash
$ tq query "SELECT * FROM nonexistent_table"
Error: Table does not exist

Object 'nonexistent_table' does not exist.

Error Code: 3807
Session ID: 1429
```

**Example 3: Incomplete Table Reference**
```bash
$ tq query "SELECT * FROM database."
Error: SQL syntax error

Expected something like an 'UDFCALLNAME' keyword between '.' and the 'AS' keyword.

Error Code: 3707
Session ID: 1429
```

### Permission Errors

**Example**:
```bash
$ tq query "DROP TABLE important_data"
Error: Permission denied

User 'alice' does not have DROP privilege on table 'important_data'.

Error Code: 3523
Session ID: 1429

Action required: Contact your database administrator to request privileges
```

### System Errors

**Definition**: Unexpected errors, bugs
**Exit Code**: `1`
**Strategy**: Apologize, provide debug info, suggest reporting

**Example**:
```bash
$ tq query "SELECT 1"
Error: Internal error occurred

This is a bug in tq. Please report it!

Debug information:
  Version: 1.0.0 (commit a3f2b1c)
  OS: Linux x86_64
  Error: thread 'main' panicked at 'unexpected None value'
  Location: src/db.rs:234

Report this issue:
  https://github.com/yourusername/tq/issues/new

Include:
  - This error message
  - Command that triggered it
  - tq --version output

Workarounds:
  - Try with --verbose for more info
  - Try --driver-lib-dir /custom/path
```

## Error Message Structure

### General Template
```
Error: [Short description]

[Context: what was being attempted]

[Details: technical error message]

[Suggestions: how to fix]

[Links: documentation, issues]
```

### SQL Error Template
```
Error: [Short Error Type]

[User-Friendly Error Message]

Error Code: [Teradata Error Code]
Session ID: [Session Number]
```

**SQL Error Parsing Rules:**
1. Extract session ID from `[Session NNNN]` pattern
2. Extract error code from `[Error NNNN]` pattern
3. Extract message text (after last `]` bracket, before stack traces)
4. **Discard all lines starting with "at"** (internal Go stack traces)
5. Format cleanly with whitespace for readability

**What to Suppress:**
- Stack traces (all "at gosqldriver/..." lines)
- Version information (redundant for SQL errors)
- Internal function names and file paths
- Runtime call stacks from driver internals

**What to Include:**
- Clear error type (syntax error, permission error, etc.)
- Actual SQL error message (descriptive text)
- Teradata error code (for documentation lookup)
- Session ID (for troubleshooting with DBAs)

**Best Practices**:
1. **Be specific**: Not "Error occurred" but "Connection refused to host:1025"
2. **Show context**: Display relevant query/command when helpful
3. **Suggest solutions**: Actionable next steps
4. **Use plain language**: Avoid jargon
5. **Format for scanning**: Use whitespace, bullets
6. **Suppress internal details**: No stack traces for end users

## Progress Indicators

### Spinner (Indeterminate)

```
⠋ Connecting to myhost:1025...
```

**Use Case**: Connection attempts, query execution (unknown duration)

### Progress Bar (Determinate)

```
Exporting data [████████████████░░░░] 80% (8000/10000 rows) ETA: 5s
```

**Use Case**: Large data exports, known row counts

### Multi-Progress (Concurrent)

```
✓ Connecting... Done (123ms)
⠋ Executing query...
  Fetching results...
```

**Use Case**: Multi-stage operations

### Terminal Detection

Automatically disable progress indicators when:
- Output is piped: `tq query "SELECT 1" | jq`
- Not a TTY: `tq query "SELECT 1" > output.json`
- `--quiet` flag is used

## Warnings

Non-fatal issues that don't prevent execution:

```
Warning: Query returned 1,000,000 rows. Consider using LIMIT.

Warning: Connection to host:1025 is not encrypted. Use --ssl for security.

Warning: Password found in TQ_LOGON environment variable. Use --password-file instead.
```

**Suppress Warnings**:
```bash
tq --quiet query "SELECT * FROM huge_table"
```

## Verbose Output

Debug information for troubleshooting:

```bash
$ tq -vv query "SELECT 1"
[DEBUG] Loading config from ~/.config/tq/config.toml
[DEBUG] Parsing connection string: user@host:1025/db
[DEBUG] Using logmech: TD2
[DEBUG] Connecting to host:1025...
[DEBUG] Connection established (127ms)
[DEBUG] Executing query: SELECT 1
[DEBUG] Query execution time: 15ms
[DEBUG] Fetched 1 row(s)
[DEBUG] Formatting output as table
[DEBUG] Disconnecting...
┌───┐
│ 1 │
└───┘
```

**Levels**:
- `-v`: Basic debug info
- `-vv`: Detailed debug info
- `-vvv`: Trace-level (includes driver logs)

---
