# Command-Line Interface Design

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Command Structure](#command-structure)
3. [Global Options](#global-options)
4. [Commands](#commands)
   - [help - Display Help Information](#help---display-help-information)
   - [ping - Test Connectivity](#ping---test-connectivity)
   - [query - Execute SQL](#query---execute-sql)
   - [repl - Interactive Mode](#repl---interactive-mode)
   - [sessions - List Active Sessions](#sessions---list-active-sessions)
   - [sysconfig - System Configuration Summary](#sysconfig---system-configuration-summary)
   - [locks - Lock and Blocking Information](#locks---lock-and-blocking-information)
   - [query-inspect - Inspect Session Query Text](#query-inspect---inspect-session-query-text)
   - [profiles - List Connection Profiles](#profiles---list-connection-profiles)
5. [Input/Output Behavior](#inputoutput-behavior)
6. [Flag Design Guidelines](#flag-design-guidelines)
7. [Help Text Design](#help-text-design)
8. [Version Information](#version-information)

---

## Design Philosophy

The CLI follows these principles:
- **Subcommands for major operations**: `ping`, `query`, `repl`
- **Global options before subcommand**: `-l/--logon`, `--logmech`, `--password-file`
- **Subcommand-specific options after subcommand**: `--format`, `--output`
- **POSIX-compliant**: Short (`-f`) and long (`--format`) flags
- **Environment variable fallbacks**: `TQ_LOGON`, `TQ_LOGMECH`, `TQ_FORMAT`

## Command Structure

```
tq [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

**Available Commands:**
- `help [topic]` - Display help information
- `ping` - Test database connectivity
- `query` - Execute SQL queries
- `repl` - Start interactive mode
- `sessions` - List active Teradata sessions
- `sysconfig` - Display system configuration (version and AMP count)
- `locks` - Display current lock contention and blocking chains
- `query-inspect` - Show SQL text for a specific session
- `profiles` - List connection profiles

## Global Options

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--logon` | `-l` | string | `$TQ_LOGON` | Connection string: `user:password@host:port/database` |
| `--password-file` | - | path | - | Read password from file |
| `--logmech` | - | enum | `TD2` | Authentication: `TD2`, `LDAP`, `KRB5`, `TDNEGO` |
| `--driver-lib-dir` | - | path | bundled | Teradata driver library directory |
| `--timeout` | `-t` | duration | `30s` | Connection timeout |
| `--verbose` | `-v` | flag | false | Verbose output (repeatable: `-vv`, `-vvv`) |
| `--quiet` | `-q` | flag | false | Suppress non-essential output |
| `--color` | - | enum | `auto` | Color output: `auto`, `always`, `never` |
| `--params` | `-p` | path | - | YAML parameter file for variable substitution (repeatable) |
| `--profile` | - | string | - | Select connection profile from config file |
| `--help` | `-h` | flag | - | Show help |
| `--version` | `-V` | flag | - | Show version |

## Commands

### help - Display Help Information

**Purpose**: Display comprehensive help for specific topics

**Usage**:
```bash
tq help [TOPIC]
```

**Topics**:
| Topic | Description |
|-------|-------------|
| (none) | Show general help (equivalent to `tq --help`) |
| `config` | Configuration file format and usage |
| `credentials` | Password and credential management |
| `params` | Variable substitution syntax and YAML parameter files |

**Examples**:
```bash
# General help
tq help

# Configuration help
tq help config

# Credentials help
tq help credentials

# Variable substitution help
tq help params

# Unknown topic handling
tq help unknown
# Error: Unknown help topic 'unknown'
# Available topics: config, credentials, params
```

---

### `tq help params` Content

**REQ-PARAMS-HELP-001: Exact content of `tq help params`**

```
tq - Variable Substitution

Use {{variable}} markers in SQL to substitute values at execution time.
Values come from a YAML parameter file specified with -p/--params.

Usage:
  tq -p <file.yaml> query "SELECT * FROM {{table}}"
  tq -p <file.yaml> query --file script.sql
  tq -p base.yaml -p overrides.yaml query --file report.sql
  cat script.sql | tq -p params.yaml query

Marker Syntax:
  {{key}}              Simple key from YAML file
  {{section.key}}      Nested key using dot notation
  {{$ENV.VAR_NAME}}    Environment variable (no YAML entry needed)

Parameter File Format (YAML):
  # params.yaml
  table: employees
  limit: 100

  target:
    database: PRODUCTION
    schema: HR

  filters:
    region: EMEA
    active: true

Dot Notation for Nested Keys:
  YAML key 'target.database' is accessed with {{target.database}}
  YAML key 'target.schema'   is accessed with {{target.schema}}

Environment Variables:
  {{$ENV.DATABASE_HOST}}  reads the DATABASE_HOST environment variable
  $ENV variables are resolved at execution time from the live environment.
  No YAML entry is needed.

Multiple Parameter Files:
  tq -p base.yaml -p prod.yaml query --file report.sql

  Files are merged left to right. Later files override earlier files
  on conflicting keys. Nested mappings are merged recursively.

  # base.yaml          # prod.yaml (overrides)
  db: STAGING          db: PRODUCTION
  limit: 10            # limit stays 10 (not in prod.yaml)

Quoting:
  Substitution inserts raw text. Add SQL quotes in the template:
    Good:  WHERE name = '{{employee_name}}'
    Bad:   WHERE name = {{employee_name}}   (missing quotes)

Examples:
  # Inline query with params file
  tq -p params.yaml query "SELECT * FROM {{table}} SAMPLE {{limit}}"

  # SQL file with nested params
  tq -p deploy.yaml query --file migrate.sql

  # Environment variable (no params file needed)
  tq query "SELECT * FROM {{$ENV.SCHEMA}}.employees"

  # Base + environment override
  tq -p base.yaml -p prod.yaml query --file report.sql

Error Messages:
  Undefined variable:
    Error: Undefined variable in template
    Variable '{{table}}' is not defined.
    Available variables: limit, target.database, target.schema

  File not found:
    Error: Parameter file not found
    Could not read: params.yaml

  YAML parse error:
    Error: Invalid YAML in parameter file
    Could not parse: params.yaml
    Line 5: mapping values are not allowed in this context

  Env var not set:
    Error: Undefined environment variable in template
    Variable '{{$ENV.SCHEMA}}' references environment variable 'SCHEMA'
    which is not set.

REPL Usage:
  In REPL mode, use /params to manage parameter files interactively:
    /params load <file>    Load a parameter file
    /params unload         Clear all loaded parameters
    /params show           Show currently loaded parameters

See also:
  tq help config          Configuration file format
  tq query --help         Query command options
```

**REQ-PARAMS-HELP-002: Unknown topic error message**

When the user runs `tq help <unknown-topic>`:

```
Error: Unknown help topic '<unknown-topic>'

Available topics: config, credentials, params
```

Exit code: `2`

**Exit Codes**:
- `0`: Help displayed successfully
- `2`: Unknown topic

---

### ping - Test Connectivity

**Purpose**: Verify database connection and measure latency

**Usage**:
```bash
tq [GLOBAL_OPTIONS] ping [OPTIONS]
```

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--count` | `-c` | int | 1 | Number of ping attempts |
| `--interval` | `-i` | duration | `1s` | Interval between pings |

**Examples**:
```bash
# Single ping
tq -l "user:pass@host:1025/db" ping

# Multiple pings
tq ping --count 5 --interval 2s

# Using environment variable
export TQ_LOGON="user:pass@host:1025/db"
tq ping
```

**Output (Success)**:
```
Database connection successful (127ms)
```

**Output (Failure)**:
```
Error: Failed to connect to host:1025
Reason: Connection refused

Troubleshooting:
  - Check that the hostname and port are correct
  - Verify the database is running
  - Check firewall settings
```

**Exit Codes**:
- `0`: Connection successful
- `1`: Connection failed

---

### query - Execute SQL

**Purpose**: Execute a SQL query and display results

**Usage**:
```bash
tq [GLOBAL_OPTIONS] query [OPTIONS] <QUERY>
tq [GLOBAL_OPTIONS] query [OPTIONS] --file <FILE>
tq [GLOBAL_OPTIONS] query [OPTIONS] < script.sql
```

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | path | stdout | Write output to file |
| `--file` | - | path | - | Read SQL from file |
| `--no-header` | - | flag | false | Omit column headers (CSV/table) |
| `--timing` | - | flag | false | Show query execution time |
| `--limit` | `-n` | int | - | Limit number of rows returned |

**Arguments**:
- `<QUERY>`: SQL query string (mutually exclusive with `--file` and stdin)

**Examples**:
```bash
# Basic query
tq query "SELECT * FROM employees LIMIT 10"

# JSON output for scripting
tq query --format json "SELECT user_id, name FROM users"

# CSV export to file
tq query --format csv "SELECT * FROM sales" --output sales.csv
# OR using shell redirection
tq query --format csv "SELECT * FROM sales" > sales.csv

# Read from file
tq query --file script.sql

# Read from stdin
cat query.sql | tq query
echo "SELECT 1" | tq query

# With timing
tq query --timing "SELECT COUNT(*) FROM large_table"
```

**Output (Table Format)**:
```
┌─────────┬──────────┬─────────────┐
│ user_id │ username │ created_at  │
├─────────┼──────────┼─────────────┤
│ 1       │ alice    │ 2024-01-15  │
│ 2       │ bob      │ 2024-01-16  │
└─────────┴──────────┴─────────────┘

2 rows in set (0.234s)
```

**Output (JSON Format)**:
```json
[
  {"user_id": 1, "username": "alice", "created_at": "2024-01-15"},
  {"user_id": 2, "username": "bob", "created_at": "2024-01-16"}
]
```

**Output (CSV Format)**:
```csv
user_id,username,created_at
1,alice,2024-01-15
2,bob,2024-01-16
```

**Exit Codes**:
- `0`: Query executed successfully
- `1`: Query error (syntax error, permission denied, etc.)
- `2`: Usage error (invalid arguments)

---

### repl - Interactive Mode

**Purpose**: Start an interactive Read-Eval-Print Loop session

**Usage**:
```bash
tq [GLOBAL_OPTIONS] repl [OPTIONS]
```

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--no-history` | - | flag | false | Disable command history |
| `--history-file` | - | path | `~/.tq_history` | History file location |
| `--no-syntax-highlight` | - | flag | false | Disable syntax highlighting |
| `--editor-mode` | - | enum | `emacs` | Key bindings: `emacs`, `vi` |

**Examples**:
```bash
# Start REPL with default settings
tq repl

# Start with vim keybindings
tq repl --editor-mode vi

# Start without history
tq repl --no-history
```

**Exit Codes**:
- `0`: Normal exit
- `1`: Connection error or fatal error

---

### sessions - List Active Sessions

**Purpose**: List all active Teradata sessions with performance metrics

**Usage**:
```bash
tq [GLOBAL_OPTIONS] sessions [OPTIONS]
tq [GLOBAL_OPTIONS] --sessions [OPTIONS]
```

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | path | stdout | Write output to file |

**Examples**:
```bash
# Basic session list with table output
tq sessions

# Alternative standalone flag form
tq --sessions

# JSON output for scripting
tq sessions --format json

# CSV export to file
tq sessions --format csv --output sessions.csv
tq sessions -f csv -o sessions.csv

# Pipe to processing tool
tq sessions --format json | jq '.[] | select(.PEstate == "ACTIVE")'

# Using connection profile
tq --profile prod sessions
```

**Output (Table Format)**:
```
Active Sessions on prod-td01.company.com:
┌───────────┬──────────┬────────────────────────┬─────────────┬──────────┬───────────┬───────┬─────────────┬────────────────┬──────────────┐
│ SessionNo │ UserName │ LogonTime              │ PEstate     │ AMPState │ AMPCPUSec │ AMPIO │ ReqSpool    │ Amp CPU Skew % │ Amp IO Skew %│
├───────────┼──────────┼────────────────────────┼─────────────┼──────────┼───────────┼───────┼─────────────┼────────────────┼──────────────┤
│      1076 │ DBC      │ 2026/01/27 15:33:26.00 │ IDLE        │ IDLE     │         0 │     6 │           0 │           [--] │         [--] │
│      1077 │ DBC      │ 2026/01/27 15:33:27.00 │ IDLE        │ IDLE     │     0.376 │  6782 │           0 │           [--] │         [--] │
│      1078 │ DBC      │ 2026/01/27 15:33:28.00 │ DISPATCHING │ ACTIVE   │   366.736 │ 75335 │ 26753187840 │           2.87 │         3.78 │
└───────────┴──────────┴────────────────────────┴─────────────┴──────────┴───────────┴───────┴─────────────┴────────────────┴──────────────┘

3 sessions found (Query time: 0.234s)
```

**Output (CSV Format)**:
```csv
SessionNo,UserName,LogonTime,PEstate,AMPState,AMPCPUSec,AMPIO,ReqSpool,Amp CPU Skew %,Amp IO Skew %
1076,DBC,2026/01/27 15:33:26.00,IDLE,IDLE,0,6,0,,
1077,DBC,2026/01/27 15:33:27.00,IDLE,IDLE,0.376,6782,0,,
1078,DBC,2026/01/27 15:33:28.00,DISPATCHING,ACTIVE,366.736,75335,26753187840,2.87,3.78
```

**Output (JSON Format)**:
```json
[
  {
    "SessionNo": 1076,
    "UserName": "DBC",
    "LogonTime": "2026/01/27 15:33:26.00",
    "PEstate": "IDLE",
    "AMPState": "IDLE",
    "AMPCPUSec": 0.0,
    "AMPIO": 6,
    "ReqSpool": 0,
    "Amp CPU Skew %": null,
    "Amp IO Skew %": null
  },
  {
    "SessionNo": 1077,
    "UserName": "DBC",
    "LogonTime": "2026/01/27 15:33:27.00",
    "PEstate": "IDLE",
    "AMPState": "IDLE",
    "AMPCPUSec": 0.376,
    "AMPIO": 6782,
    "ReqSpool": 0,
    "Amp CPU Skew %": null,
    "Amp IO Skew %": null
  },
  {
    "SessionNo": 1078,
    "UserName": "DBC",
    "LogonTime": "2026/01/27 15:33:28.00",
    "PEstate": "DISPATCHING",
    "AMPState": "ACTIVE",
    "AMPCPUSec": 366.736,
    "AMPIO": 75335,
    "ReqSpool": 26753187840,
    "Amp CPU Skew %": 2.87,
    "Amp IO Skew %": 3.78
  }
]
```

**Behavior Requirements**:

1. **Standalone Operation**: Does NOT require a SQL file argument (unlike `query` command)
2. **Data Source**: Queries Teradata `MonitorSession(-1,'*',0)` table function
3. **Column Display**: 10 columns in this order:
   - SessionNo (session identifier)
   - UserName (logged-in user)
   - LogonTime (session start timestamp: YYYY/MM/DD HH:MM:SS.ss format)
   - PEstate (Parsing Engine state: IDLE/DISPATCHING/ACTIVE)
   - AMPState (AMP state: IDLE/ACTIVE)
   - AMPCPUSec (total AMP CPU seconds consumed)
   - AMPIO (total AMP I/O count)
   - ReqSpool (requested spool space in bytes)
   - Amp CPU Skew % (CPU distribution across AMPs, 0% = perfect balance)
   - Amp IO Skew % (I/O distribution across AMPs, 0% = perfect balance)
4. **NULL Handling**:
   - Table format: Display `[--]` for NULL skew percentages (IDLE sessions)
   - CSV format: Empty string for NULL skew percentages
   - JSON format: `null` for NULL skew percentages
5. **Format Compatibility**: Works with `--format table`, `--format csv`, `--format json`
6. **Output Destination**: Respects `--output` flag for file output, otherwise stdout
7. **Summary Footer**: Includes row count and query execution time (table format only)

**Error Handling**:

**Insufficient Privileges**:
```
Error: Unable to list sessions
Reason: SELECT permission denied on DBC.MonitorSession

This command requires SELECT access to the MonitorSession table function.
Contact your DBA to request access or use the GRANT statement:
  GRANT SELECT ON DBC.MonitorSession TO <your_username>;

Exit code: 1
```

**Connection Failed**:
```
Error: Failed to connect to prod-td01.company.com:1025
Reason: Connection refused

Troubleshooting:
  - Check that the hostname and port are correct
  - Verify the database is running
  - Check firewall settings

Exit code: 1
```

**MonitorSession Not Available**:
```
Error: MonitorSession table function not found
This feature requires Teradata 14.10 or later.
Current database version: 13.10

Alternative: Use DBC.SessionTbl view (limited metrics)

Exit code: 1
```

**Exit Codes**:
- `0`: Sessions listed successfully
- `1`: Query error (privilege denied, connection failed, function not available)
- `2`: Usage error (invalid format, invalid output path)

**Integration with Scripting**:

The `sessions` command is designed for both interactive use and automation:

```bash
# Monitor active queries in cron job
tq --sessions --format json | \
  jq '.[] | select(.PEstate == "ACTIVE" and .AMPCPUSec > 100)' | \
  mail -s "Long-running queries alert" dba@company.com

# Export session snapshot for analysis
tq sessions --format csv --output "/var/log/td-sessions/$(date +%Y%m%d_%H%M%S).csv"

# Check for high skew sessions
tq --sessions -f json | \
  jq '.[] | select(.["Amp CPU Skew %"] > 20) | .SessionNo'

# Count active sessions by user
tq --sessions -f csv | \
  tail -n +2 | \
  cut -d, -f2 | \
  sort | uniq -c
```

**Command Form Equivalence**:

Both forms are functionally identical:
```bash
# Subcommand form (recommended for clarity)
tq sessions

# Flag form (compact)
tq --sessions
```

The flag form (`--sessions`) provides compatibility with single-purpose invocations, while the subcommand form (`sessions`) follows the standard CLI pattern and allows for potential future options.

**Acceptance Test**:
- Execute `tq sessions` and verify table output with all 10 columns
- Execute `tq --sessions` and verify identical behavior to `tq sessions`
- Execute `tq sessions --format csv` and verify CSV output with headers
- Execute `tq sessions --format json` and verify valid JSON array output
- Execute `tq sessions --output sessions.txt` and verify file creation
- Trigger privilege error and verify helpful error message with GRANT example
- Execute on system with no sessions besides current and verify 1 row displayed
- Verify NULL skew percentages appear as `[--]` in table format, empty in CSV, `null` in JSON
- Verify exit code 0 on success, 1 on privilege/connection errors

---

### sysconfig - System Configuration Summary

**Purpose**: Display a compact summary of system configuration including Teradata version and AMP count

**Usage**:
```bash
tq [GLOBAL_OPTIONS] sysconfig [OPTIONS]
```

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | path | stdout | Write output to file |

**Examples**:
```bash
# Display system configuration with table output
tq sysconfig

# JSON output for scripting
tq sysconfig --format json

# CSV export to file
tq sysconfig --format csv --output sysconfig.csv
tq sysconfig -f csv -o sysconfig.csv

# Using connection profile
tq --profile prod sysconfig

# Pipe to processing tool
tq sysconfig --format json | jq '.["AMP Count"]'
```

**Output (Table Format)**:
```
System Configuration:
┌──────────────────┬─────────────────────────────────────┐
│ Property         │ Value                               │
├──────────────────┼─────────────────────────────────────┤
│ Teradata Version │ 17.20.00.17                         │
│ Release          │ 17.20.00.17 (Released: 2024-01-15)  │
│ AMP Count        │ 128                                 │
└──────────────────┴─────────────────────────────────────┘
```

**Output (CSV Format)**:
```csv
Property,Value
Teradata Version,17.20.00.17
Release,"17.20.00.17 (Released: 2024-01-15)"
AMP Count,128
```

**Output (JSON Format)**:
```json
{
  "Teradata Version": "17.20.00.17",
  "Release": "17.20.00.17 (Released: 2024-01-15)",
  "AMP Count": 128
}
```

**Behavior Requirements**:

1. **Standalone Operation**: Does NOT require a SQL file argument (unlike `query` command)
2. **Data Source**: Queries `DBC.DBCInfoV` for version and release; derives AMP count via `HASHAMP()+1`
3. **Properties Displayed** (in this order):
   - Teradata Version (software version string)
   - Release (full release string with build date)
   - AMP Count (total number of Access Module Processors)
4. **Format Compatibility**: Works with `--format table`, `--format csv`, `--format json`
5. **Output Destination**: Respects `--output` flag for file output, otherwise stdout
6. **Unavailable Properties**: If a specific property cannot be retrieved, display `[unavailable]` for that value rather than failing the entire command

**Error Handling**:

**Insufficient Privileges**:
```
Error: Unable to retrieve system configuration
Reason: SELECT permission denied on DBC.DBCInfoV

This command requires SELECT access to DBC system views.
Contact your DBA to request access or use the GRANT statement:
  GRANT SELECT ON DBC.DBCInfoV TO <your_username>;

Exit code: 1
```

**Connection Failed**:
```
Error: Failed to connect to prod-td01.company.com:1025
Reason: Connection refused

Troubleshooting:
  - Check that the hostname and port are correct
  - Verify the database is running
  - Check firewall settings

Exit code: 1
```

**Exit Codes**:
- `0`: System configuration displayed successfully
- `1`: Query error (privilege denied, connection failed, view not available)
- `2`: Usage error (invalid format, invalid output path)

**Integration with Scripting**:

The `sysconfig` command is designed for both interactive use and automation:

```bash
# Capture AMP count for capacity planning
AMP_COUNT=$(tq sysconfig --format json | jq '.["AMP Count"]')
echo "System has ${AMP_COUNT} AMPs"

# Export configuration snapshot
tq sysconfig --format csv --output "/var/log/td-config/$(date +%Y%m%d).csv"

# Verify system version in deployment script
VERSION=$(tq sysconfig --format json | jq -r '.["Teradata Version"]')
if [[ "$VERSION" != "17.20"* ]]; then
  echo "Warning: Unexpected Teradata version: $VERSION"
fi
```

**Acceptance Test**:
- Execute `tq sysconfig` and verify table output with all three properties: Teradata Version, Release, AMP Count
- Execute `tq sysconfig --format csv` and verify CSV output with Property,Value headers and three data rows
- Execute `tq sysconfig --format json` and verify valid JSON object output with three keys
- Execute `tq sysconfig --output sysconfig.txt` and verify file creation
- Trigger privilege error and verify helpful error message with GRANT example
- Verify exit code 0 on success, 1 on privilege/connection errors

---

### locks - Lock and Blocking Information

**Purpose**: Display current lock contention showing locked objects, lock types, locking sessions, and blocking chains to help DBAs diagnose contention issues

**Usage**:
```bash
tq [GLOBAL_OPTIONS] locks [OPTIONS]
```

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | path | stdout | Write output to file |

**Examples**:
```bash
# Basic lock list with table output
tq locks

# JSON output for scripting
tq locks --format json

# CSV export to file
tq locks --format csv --output locks.csv
tq locks -f csv -o locks.csv

# Pipe to processing tool
tq locks --format json | jq '.[] | select(.["Lock Mode"] == "EXCLUSIVE")'

# Using connection profile
tq --profile prod locks
```

**Output (Table Format - Locks Present)**:
```
Lock Information:
┌──────────────────────┬───────────┬────────────┬──────────────┬──────────────┐
│ Locked Object        │ Lock Type │ Lock Mode  │ Locking Sess │ Waiting Sess │
├──────────────────────┼───────────┼────────────┼──────────────┼──────────────┤
│ PRODUCTION.orders    │ Table     │ WRITE      │ 1023         │ 1045, 1067   │
│ PRODUCTION.customers │ Table     │ EXCLUSIVE  │ 1023         │ 1051         │
│ PRODUCTION.employees │ Row Hash  │ READ       │ 1078         │ (none)       │
└──────────────────────┴───────────┴────────────┴──────────────┴──────────────┘

3 lock(s) found - 1 blocking chain(s) detected (Query time: 0.089s)

Blocking Chain:
  Session 1023 blocks sessions: 1045, 1051, 1067
```

**Output (Table Format - No Locks)**:
```
Lock Information:
No locks currently held.

(Query time: 0.023s)
```

**Output (CSV Format)**:
```csv
Locked Object,Lock Type,Lock Mode,Locking Sess,Waiting Sess
PRODUCTION.orders,Table,WRITE,1023,"1045, 1067"
PRODUCTION.customers,Table,EXCLUSIVE,1023,1051
PRODUCTION.employees,Row Hash,READ,1078,
```

**Output (JSON Format)**:
```json
[
  {
    "Locked Object": "PRODUCTION.orders",
    "Lock Type": "Table",
    "Lock Mode": "WRITE",
    "Locking Sess": 1023,
    "Waiting Sess": [1045, 1067]
  },
  {
    "Locked Object": "PRODUCTION.customers",
    "Lock Type": "Table",
    "Lock Mode": "EXCLUSIVE",
    "Locking Sess": 1023,
    "Waiting Sess": [1051]
  },
  {
    "Locked Object": "PRODUCTION.employees",
    "Lock Type": "Row Hash",
    "Lock Mode": "READ",
    "Locking Sess": 1078,
    "Waiting Sess": []
  }
]
```

**Behavior Requirements**:

1. **Standalone Operation**: Does NOT require a SQL file argument (unlike `query` command)
2. **Data Source**: Queries `DBC.LockInfoV` (or platform-equivalent view) for current lock information
3. **Column Display**: 5 columns in this order:
   - Locked Object (fully qualified database.table name)
   - Lock Type (granularity: Table, Row Hash, Database)
   - Lock Mode (severity: READ, WRITE, EXCLUSIVE, ACCESS)
   - Locking Sess (session ID holding the lock)
   - Waiting Sess (comma-separated IDs of waiting sessions, or empty)
4. **Waiting Sessions Handling**:
   - Table format: Display comma-separated list of session IDs, or `(none)` when no waiters
   - CSV format: Comma-separated session IDs (quoted if multiple), or empty string when no waiters
   - JSON format: Array of session ID integers, or empty array `[]` when no waiters
5. **Blocking Chain Section**: In table format, display "Blocking Chain:" summary after the table when any sessions are waiting. Omit this section in CSV and JSON formats.
6. **No Locks**: Display "No locks currently held." message when query returns no rows
7. **Format Compatibility**: Works with `--format table`, `--format csv`, `--format json`
8. **Output Destination**: Respects `--output` flag for file output, otherwise stdout

**Lock Mode Reference**:

| Lock Mode | Blocks | Description |
|-----------|--------|-------------|
| ACCESS | EXCLUSIVE only | Weakest lock. Allows concurrent reads and writes. |
| READ | WRITE, EXCLUSIVE | Shared lock. Allows concurrent reads. |
| WRITE | WRITE, EXCLUSIVE | Exclusive writes. Allows concurrent reads. |
| EXCLUSIVE | All modes | Strongest lock. Blocks all other lock modes. |

**Error Handling**:

**Insufficient Privileges**:
```
Error: Unable to retrieve lock information
Reason: SELECT permission denied on DBC.LockInfoV

This command requires SELECT access to DBC lock views.
Contact your DBA to request access or use the GRANT statement:
  GRANT SELECT ON DBC.LockInfoV TO <your_username>;

Exit code: 1
```

**Lock View Not Available**:
```
Error: Lock information view not available
DBC.LockInfoV is not accessible on this system.

This may indicate a Teradata version compatibility issue or a
configuration restriction. Contact your DBA for assistance.

Exit code: 1
```

**Connection Failed**:
```
Error: Failed to connect to prod-td01.company.com:1025
Reason: Connection refused

Troubleshooting:
  - Check that the hostname and port are correct
  - Verify the database is running
  - Check firewall settings

Exit code: 1
```

**Exit Codes**:
- `0`: Lock information displayed successfully (including the case of no active locks)
- `1`: Query error (privilege denied, connection failed, view not available)
- `2`: Usage error (invalid format, invalid output path)

**Integration with Scripting**:

The `locks` command is designed for both interactive use and automation:

```bash
# Check for exclusive locks in monitoring script
tq locks --format json | \
  jq '.[] | select(.["Lock Mode"] == "EXCLUSIVE")' | \
  mail -s "Exclusive lock alert" dba@company.com

# Export lock snapshot for incident analysis
tq locks --format csv --output "/var/log/td-locks/$(date +%Y%m%d_%H%M%S).csv"

# Count locks by mode
tq locks --format json | \
  jq 'group_by(.["Lock Mode"]) | map({mode: .[0]["Lock Mode"], count: length})'

# Check if any sessions are blocked
BLOCKED=$(tq locks --format json | jq '[.[] | select(.["Waiting Sess"] | length > 0)] | length')
if [[ "$BLOCKED" -gt 0 ]]; then
  echo "WARNING: $BLOCKED blocking lock(s) detected"
fi
```

**Acceptance Test**:
- Execute `tq locks` with no active locks and verify "No locks currently held." output
- Execute `tq locks` with active locks and verify table output with all 5 columns: Locked Object, Lock Type, Lock Mode, Locking Sess, Waiting Sess
- Verify blocking chains section appears in table format when sessions are waiting
- Verify blocking chains section does NOT appear in table format when no waiters
- Execute `tq locks --format csv` and verify CSV output with correct 5-column headers
- Execute `tq locks --format json` and verify valid JSON array output with Waiting Sess as array
- Execute `tq locks --output locks.txt` and verify file creation
- Trigger privilege error and verify helpful error message with GRANT example
- Verify exit code 0 on success (including no-locks case), 1 on privilege/connection errors

---

### query-inspect - Inspect Session Query Text

**Purpose**: Display the SQL text of the most recent query executed by a specific session, enabling drill-down from session activity into the SQL causing resource consumption or blocking

**Usage**:
```bash
tq [GLOBAL_OPTIONS] query-inspect [OPTIONS] <SESSION_ID>
```

**Arguments**:
- `<SESSION_ID>`: Required. An integer session ID as shown in `tq sessions` or `tq locks` output.

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | path | stdout | Write output to file |

**Examples**:
```bash
# Show query text for session 1023
tq query-inspect 1023

# Full query text via JSON (no truncation)
tq query-inspect --format json 1023

# CSV output for scripting
tq query-inspect --format csv 1023

# Using connection profile
tq --profile prod query-inspect 1023

# Extract just the query text
tq query-inspect --format json 1023 | jq -r '.["Query Text"]'
```

**Output (Table Format - Query Found)**:
```
Query for session 1023:
┌────────────┬──────────────────────────────────────────────────────────────────┐
│ Property   │ Value                                                            │
├────────────┼──────────────────────────────────────────────────────────────────┤
│ Session    │ 1023                                                             │
│ User       │ etl_user                                                         │
│ Query Text │ UPDATE PRODUCTION.orders SET status = 'shipped' WHERE order_... │
└────────────┴──────────────────────────────────────────────────────────────────┘

(Query time: 0.123s)
```

**Output (Table Format - Session Not Found)**:
```
No query information found for session 9999.

The session may have already disconnected, or DBQL logging may not be
enabled for this user. Contact your DBA to enable DBQL logging.
```

**Output (CSV Format)**:
```csv
Session,User,Query Text
1023,etl_user,"UPDATE PRODUCTION.orders SET status = 'shipped' WHERE order_date < '2026-01-01'"
```

**Output (JSON Format)**:
```json
{
  "Session": 1023,
  "User": "etl_user",
  "Query Text": "UPDATE PRODUCTION.orders SET status = 'shipped' WHERE order_date < '2026-01-01'"
}
```

**Behavior Requirements**:

1. **Standalone Operation**: Requires a session ID argument
2. **Data Source**: Queries `DBC.QryLogV` for the most recent query log entry for the specified session ID
3. **Properties Displayed** (in this order):
   - Session (the queried session ID)
   - User (database user account for that session)
   - Query Text (SQL text of the most recent logged query)
4. **Query Text Length**:
   - Table format: Truncated at 200 characters with `...` to indicate truncation
   - CSV and JSON formats: Full untruncated query text
5. **Session Not Found**: When no DBQL record exists for the given session ID, display informative message and exit with code 0 (not an error)
6. **Format Compatibility**: Works with `--format table`, `--format csv`, `--format json`
7. **Output Destination**: Respects `--output` flag for file output, otherwise stdout

**Error Handling**:

**Insufficient Privileges**:
```
Error: Unable to retrieve query information
Reason: SELECT permission denied on DBC.QryLogV

This command requires SELECT access to DBC.QryLogV.
Contact your DBA to request access or use the GRANT statement:
  GRANT SELECT ON DBC.QryLogV TO <your_username>;

Exit code: 1
```

**Missing Argument**:
```
Error: Missing required argument <session_id>
Usage: tq query-inspect <session_id>

Example: tq query-inspect 1023

Exit code: 2
```

**Invalid Argument**:
```
Error: Invalid session ID 'abc'
Session ID must be a positive integer.

Example: tq query-inspect 1023

Exit code: 2
```

**Exit Codes**:
- `0`: Query information displayed (including the case of session not found)
- `1`: Query error (privilege denied, connection failed, view not available)
- `2`: Usage error (missing or invalid session ID argument, invalid format)

**Integration with Scripting**:

```bash
# Full PMON workflow: sessions -> query-inspect
tq sessions --format json | \
  jq '.[] | select(.AMPCPUSec > 1000) | .SessionNo' | \
  while read session; do
    echo "=== Session $session ==="
    tq query-inspect --format json "$session" | jq -r '.["Query Text"]'
  done

# Find and inspect all blocking sessions
tq locks --format json | \
  jq '.[].["Locking Sess"]' | sort -u | \
  while read session; do
    echo "Blocking session $session is running:"
    tq query-inspect "$session" --format csv | tail -1 | cut -d, -f3
  done
```

**Acceptance Test**:
- Execute `tq query-inspect <active_session_id>` and verify table output with Session, User, Query Text
- Execute `tq query-inspect <inactive_session_id>` and verify informative not-found message (exit code 0)
- Execute `tq query-inspect --format csv <session_id>` and verify CSV output with full query text
- Execute `tq query-inspect --format json <session_id>` and verify JSON object output with full query text
- Execute without session ID and verify usage error (exit code 2)
- Execute with non-integer session ID and verify invalid argument error (exit code 2)
- Trigger privilege error and verify helpful error message with GRANT example (exit code 1)
- Verify query text is truncated in table format but full in CSV/JSON

---

### profiles - List Connection Profiles

**Purpose**: Display all available connection profiles from the configuration file

**Usage**:
```bash
tq profiles
```

**Options**: None

**Examples**:
```bash
# List all profiles
tq profiles
```

**Output (With Profiles)**:
```
Available profiles (from ~/.tq/config.toml):

  dev
    Host:     dev.company.com:1025
    Database: development
    User:     alice
    Logmech:  TD2

  prod
    Host:     prod.company.com:1025
    Database: production
    User:     alice
    Logmech:  LDAP

  local
    Host:     localhost:1025
    Database: testdb
    User:     dbc
    Logmech:  TD2

Use: tq --profile <name> <command>
```

**Output (No Config File)**:
```
No configuration file found at ~/.tq/config.toml

To create a configuration file with profiles:
  mkdir -p ~/.tq
  cat > ~/.tq/config.toml <<EOF
  [profiles.dev]
  host = "dev.company.com"
  port = 1025
  database = "development"
  user = "alice"
  password_file = "~/.tq/passwords/dev"
  EOF

See 'tq help config' for more information
```

**Output (No Profiles Defined)**:
```
No profiles defined in ~/.tq/config.toml

To add a profile, edit ~/.tq/config.toml:
  [profiles.dev]
  host = "dev.company.com"
  port = 1025
  database = "development"
  user = "alice"
  password_file = "~/.tq/passwords/dev"

See 'tq help config' for more information
```

**Security Considerations**:
- Password fields are NEVER displayed
- Only partial connection information is shown
- Password files paths are not displayed

**Exit Codes**:
- `0`: Profiles listed successfully (or no profiles found)
- `1`: Configuration file parsing error

---

## Input/Output Behavior

### Standard Streams

| Context | stdin | stdout | stderr |
|---------|-------|--------|--------|
| Query from arg | Ignored | Results | Errors, warnings |
| Query from stdin | SQL query | Results | Errors, warnings |
| REPL mode | User input | Results | Errors, warnings |
| Piped output | Varies | Machine format | Human messages |

### Terminal Detection

The tool adjusts behavior based on context:

| Feature | Interactive (TTY) | Piped/Redirected |
|---------|-------------------|------------------|
| Color output | Enabled | Disabled |
| Progress indicators | Shown | Hidden |
| Default format | `table` | `csv` or `json` |
| Pager | Enabled (large results) | Disabled |
| Confirmation prompts | Shown | Auto-yes or error |

### Exit Code Standards

| Code | Meaning | Examples |
|------|---------|----------|
| `0` | Success | Query executed, connection successful |
| `1` | Runtime error | Connection failed, query error, file not found |
| `2` | Usage error | Invalid flag, missing required argument |
| `130` | Interrupted | User pressed Ctrl-C |

---

## Flag Design Guidelines

### Short vs Long Flags

**Short flags** (`-f`):
- Single letter
- For frequently used options
- Can be combined: `-vvv` (very verbose), `-qf json` (quiet JSON)

**Long flags** (`--format`):
- Descriptive kebab-case
- Always available
- Self-documenting

### Boolean Flags

```bash
# Flag present = true
tq query --timing "SELECT 1"

# Explicit negation with --no-prefix
tq repl --no-history

# Avoid --flag=true syntax (non-standard)
```

### Value Flags

```bash
# Space-separated (preferred)
tq query --format json "SELECT 1"

# Equals sign (also supported)
tq query --format=json "SELECT 1"

# Short flag with value
tq query -f json "SELECT 1"
```

---

## Help Text Design

### Top-Level Help

```bash
$ tq --help
```

Output:
```
tq - Teradata Query
A fast, lightweight command-line client for Teradata databases

Usage: tq [OPTIONS] <COMMAND>

Commands:
  ping      Test database connectivity
  query     Execute a SQL query
  repl      Start interactive mode
  sessions  List active Teradata sessions with performance metrics
  sysconfig Display system topology (version, nodes, AMPs, PEs)
  locks     Display current lock contention and blocking chains
  profiles  List available connection profiles
  help      Print help information or help for a topic

Global Options:
  -l, --logon <LOGON>
          Connection string: user:password@host:port/database
          [env: TQ_LOGON]

  --password-file <FILE>
          Read password from file (recommended for security)

  --logmech <LOGMECH>
          Authentication mechanism [default: TD2]
          [possible values: TD2, LDAP, KRB5, TDNEGO]

  --timeout <DURATION>
          Connection timeout [default: 30s]

  -v, --verbose
          Verbose output (repeat for more: -vv, -vvv)

  -q, --quiet
          Suppress non-essential output

  --color <WHEN>
          Color output [default: auto]
          [possible values: auto, always, never]

  -p, --params <FILE>
          YAML parameter file for {{variable}} substitution in SQL
          (repeatable: -p base.yaml -p overrides.yaml)

  -h, --help
          Print help information

  -V, --version
          Print version information

Examples:
  # Quick connection test
  tq -l "user:pass@host:1025/db" ping

  # Execute query with table output
  tq query "SELECT * FROM employees"

  # Export to JSON
  tq query --format json "SELECT * FROM data" > data.json

  # List active sessions (DBA monitoring)
  tq sessions

  # Export session metrics to CSV
  tq sessions --format csv --output sessions.csv

  # Display system topology
  tq sysconfig

  # Check for lock contention
  tq locks

  # Export lock snapshot for incident analysis
  tq locks --format json | jq '.[] | select(.["Lock Mode"] == "EXCLUSIVE")'

  # Secure password handling
  echo "password" > ~/.tq_pass && chmod 0600 ~/.tq_pass
  tq -l "user@host:1025/db" --password-file ~/.tq_pass query "SELECT 1"

Configuration:
  Set TQ_LOGON environment variable to avoid repeating connection string:
    export TQ_LOGON="user:pass@host:1025/db"
    tq ping

  Or create ~/.tq/config.toml with connection profiles:
    [profiles.dev]
    host = "dev.company.com"
    port = 1025
    database = "development"
    user = "alice"
    password_file = "~/.tq/passwords/dev"

  List available profiles:
    tq profiles

  Get detailed configuration help:
    tq help config
    tq help credentials

For more information, visit: https://github.com/yourusername/tq
```

### Subcommand Help

```bash
$ tq query --help
```

Output:
```
tq-query - Execute a SQL query

Usage: tq query [OPTIONS] <QUERY>
       tq query [OPTIONS] --file <FILE>
       tq query [OPTIONS] < script.sql

Arguments:
  <QUERY>  SQL query to execute

Options:
  -f, --format <FORMAT>
          Output format [default: table]
          [possible values: table, json, csv]

  -o, --output <FILE>
          Write output to file instead of stdout

  --file <FILE>
          Read SQL from file

  --no-header
          Omit column headers in output

  --timing
          Show query execution time

  -n, --limit <N>
          Limit number of rows returned

  -h, --help
          Print help information

Examples:
  # Simple query with table output
  tq query "SELECT * FROM employees LIMIT 10"

  # JSON output for scripting
  tq query -f json "SELECT id, name FROM users" | jq '.'

  # CSV export
  tq query -f csv "SELECT * FROM sales" > sales.csv

  # Read from file
  tq query --file script.sql

  # Piped input
  cat queries.sql | tq query
  echo "SELECT 1" | tq query

  # Parameterized query with variable substitution
  tq -p params.yaml query "SELECT * FROM {{table}} SAMPLE {{limit}}"
  tq -p params.yaml query --file report.sql
  tq -p base.yaml -p prod.yaml query --file report.sql

  # Learn about variable substitution
  tq help params
```

---

## Version Information

```bash
$ tq --version
```

Output:
```
tq 1.0.0 (release)
Commit: a3f2b1c
Built: 2024-01-15T10:30:00Z
Target: x86_64-unknown-linux-musl
Teradata Driver: 17.20.00.17
```

---
