# Command-Line Interface Design

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Command Structure](#command-structure)
3. [Global Options](#global-options)
4. [Driver Library Resolution](#driver-library-resolution)
5. [Commands](#commands)
   - [help - Display Help Information](#help---display-help-information)
   - [ping - Test Connectivity](#ping---test-connectivity)
   - [query - Execute SQL](#query---execute-sql)
   - [repl - Interactive Mode](#repl---interactive-mode)
   - [sessions - List Active Sessions](#sessions---list-active-sessions)
   - [sysconfig - System Configuration Summary](#sysconfig---system-configuration-summary)
   - [locks - Lock and Blocking Information](#locks---lock-and-blocking-information)
   - [query-inspect - Inspect Session Query Text](#query-inspect---inspect-session-query-text)
   - [inspect - Inspect a Database Object](#inspect---inspect-a-database-object)
   - [profiles - List Connection Profiles](#profiles---list-connection-profiles)
   - [profile - Manage Connection Profiles](#profile---manage-connection-profiles)
6. [Input/Output Behavior](#inputoutput-behavior)
7. [Flag Design Guidelines](#flag-design-guidelines)
8. [Help Text Design](#help-text-design)
9. [Version Information](#version-information)
10. [Installation Experience](#installation-experience)

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
- `inspect` - Comprehensive inspection of a database object (type, columns, indexes, size, dependencies)
- `profiles` - List connection profiles
- `profile` - Manage connection profiles (add, edit, delete, list)

## Global Options

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--logon` | `-l` | string | `$TQ_LOGON` | Connection string: `user:password@host:port/database` |
| `--password-file` | - | path | - | Read password from file |
| `--logmech` | - | enum | `TD2` | Authentication: `TD2`, `LDAP`, `KRB5`, `TDNEGO` |
| `--driver-lib-dir` | - | path | see below | Override Teradata driver library search path |
| `--timeout` | `-t` | duration | `30s` | Connection timeout |
| `--verbose` | `-v` | flag | false | Verbose output (repeatable: `-vv`, `-vvv`) |
| `--quiet` | `-q` | flag | false | Suppress non-essential output |
| `--color` | - | enum | `auto` | Color output: `auto`, `always`, `never` |
| `--params` | `-p` | path | - | YAML parameter file for variable substitution (repeatable) |
| `--profile` | - | string | - | Select connection profile from config file |
| `--help` | `-h` | flag | - | Show help |
| `--version` | `-V` | flag | - | Show version |

## Driver Library Resolution

The `tq` binary requires the Teradata SQL driver shared library (`teradatasql` or equivalent) to be present at runtime. Because the library cannot be statically linked, its location is resolved at startup using a well-defined search order.

### REQ-DRIVER-001: Runtime Search Order

The binary SHALL locate the driver library by checking the following locations in order, stopping at the first location where the library file is found:

1. **Executable-adjacent directory** - The directory that contains the `tq` binary itself (resolved via the OS-provided executable path at runtime, not a compile-time path).
2. **`--driver-lib-dir` flag** - The path supplied by the user on the command line.
3. **`TERADATA_LIB_DIR` environment variable** - The directory specified by this environment variable.
4. **Current working directory** - The directory from which `tq` was invoked.

**Rationale for this order:**
- Placing the executable-adjacent directory first ensures that the library distributed alongside the binary in a release archive is always found without any user configuration.
- The CLI flag comes before the environment variable so that one-off overrides always win over persistent settings.
- The current working directory serves as a developer convenience for local builds and ad-hoc testing.

### REQ-DRIVER-002: Executable-Adjacent Resolution

The executable-adjacent path SHALL be resolved at runtime using the operating system's mechanism for locating the current executable (e.g., `std::env::current_exe()` on Linux/macOS). It MUST NOT use a path that was baked in at compile time. If the OS cannot resolve the executable path, this step is silently skipped and the search continues with the next location.

### REQ-DRIVER-003: Error When Library Not Found

When none of the four locations yields a valid library file, `tq` SHALL exit with code `1` and print a diagnostic message that:
- States the library was not found.
- Lists every path that was searched, in the order they were checked.
- Tells the user how to fix the problem.

**Required error format:**

```
Error: Teradata driver library not found.

Searched the following locations (in order):
  1. Executable directory:   /home/alice/.local/bin            [not found]
  2. --driver-lib-dir flag:  (not provided)
  3. TERADATA_LIB_DIR env:   (not set)
  4. Current directory:      /home/alice/projects              [not found]

Fix: Place the driver library alongside the tq binary, or set TERADATA_LIB_DIR:
  export TERADATA_LIB_DIR=/path/to/driver
  tq ...

Download the Teradata SQL driver from:
  https://pypi.org/project/teradatasql/
```

Requirements:
1. **REQ-DRIVER-003.1** - Every search location SHALL be listed, even if it was not explicitly provided (show `(not provided)` or `(not set)` for absent flag/variable).
2. **REQ-DRIVER-003.2** - Each location SHALL include the resolved path (expanded tilde, resolved symlinks) or the placeholder for unset values.
3. **REQ-DRIVER-003.3** - Each entry SHALL include a `[not found]` or `[not set]` / `(not provided)` annotation.
4. **REQ-DRIVER-003.4** - The fix section SHALL always include both the environment variable approach and the reference to where the driver can be downloaded.

### REQ-DRIVER-004: `--driver-lib-dir` Flag Behaviour

When `--driver-lib-dir <path>` is provided:
- The path is used as search location 2 in the resolution order above.
- The path is treated as a directory, not a file. `tq` looks for the driver library file inside that directory.
- If the directory does not exist or the library is not found inside it, the search continues to the next location (no early exit).
- In verbose mode (`-v`), the resolved path and outcome for each search step SHALL be printed to stderr.

**Example verbose output:**
```
[debug] Driver search:
  [1] exe dir /home/alice/.local/bin: libteradatasql.so -> found
```

### REQ-DRIVER-005: Environment Variable (`TERADATA_LIB_DIR`)

The `TERADATA_LIB_DIR` environment variable, when set, specifies a directory to search as location 3. Its value is treated identically to `--driver-lib-dir` but at lower precedence. If the variable is set but the directory does not contain the library, the search continues to location 4.

### REQ-DRIVER-006: Release Archive Packaging

The release tar.gz archive SHALL include the driver library file in the same directory as the `tq` binary so that users who install via the archive or the install script get a working setup with no additional configuration.

```
tq-1.0.0-x86_64-unknown-linux-gnu.tar.gz
├── tq                          (binary)
└── libteradatasql.so           (driver library, same directory)
```

---

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

### inspect - Inspect a Database Object

**Purpose**: Display a comprehensive inspection of a single database object — its type, column definitions, index structure, storage metrics, and dependency relationships. This is the batch equivalent of the REPL `/inspect` command, designed for scripting and programmatic use.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] inspect [OPTIONS] <OBJECT>
```

**Arguments**:
- `<OBJECT>`: Required. An object name (`tablename`) or a fully qualified name (`database.tablename`).

**Options**:

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | path | stdout | Write output to file |
| `--section` | - | enum | (all) | Show only one section: `info`, `columns`, `indexes`, `storage`, `dependencies` |

**Examples**:
```bash
# Full inspection of a table
tq inspect employees

# Qualified name (cross-database)
tq inspect dbc.tables

# JSON output for scripting
tq inspect --format json employees

# Only show storage metrics
tq inspect --section storage employees

# CSV output piped to analysis tool
tq inspect --format csv --section columns orders | csvkit ...

# Using a connection profile
tq --profile prod inspect employees

# Write report to file
tq inspect --output employees-report.txt employees
```

**Output — Table Format (Full Table Inspection)**:
```
── Object Info ───────────────────────────────────────────

  Type:      Table
  Database:  PRODUCTION
  Name:      employees
  Created:   2023-04-15 09:12:33

── Columns ───────────────────────────────────────────────

┌───────────────┬──────────────┬──────────┬─────────┐
│ Column        │ Type         │ Nullable │ Default │
├───────────────┼──────────────┼──────────┼─────────┤
│ employee_id   │ INTEGER      │ NO       │ -       │
│ first_name    │ VARCHAR(50)  │ YES      │ NULL    │
│ last_name     │ VARCHAR(50)  │ YES      │ NULL    │
│ email         │ VARCHAR(100) │ YES      │ NULL    │
│ hire_date     │ DATE         │ YES      │ NULL    │
│ salary        │ DECIMAL(10,2)│ YES      │ NULL    │
│ department_id │ INTEGER      │ YES      │ NULL    │
└───────────────┴──────────────┴──────────┴─────────┘

7 columns

── Index Structure ───────────────────────────────────────

  Primary Index
    Type:     Unique Primary Index (UPI)
    Columns:  employee_id

  Secondary Indexes
    #1  Non-Unique Secondary Index (NUSI)  (department_id)
    #2  Unique Secondary Index (USI)       (email)

── Storage ───────────────────────────────────────────────

  Current Size:  1.4 GB
  Peak Size:     1.8 GB
  Skew Factor:   8.2%  (low skew)
  AMPs:          32
```

**Output — JSON Format**:

In JSON format, all sections are represented as a single object. Sections that are not applicable to the object type are omitted.

```json
{
  "object_info": {
    "type": "Table",
    "database": "PRODUCTION",
    "name": "employees",
    "created": "2023-04-15T09:12:33"
  },
  "columns": [
    { "column": "employee_id",   "type": "INTEGER",       "nullable": false, "default": null },
    { "column": "first_name",    "type": "VARCHAR(50)",   "nullable": true,  "default": null },
    { "column": "last_name",     "type": "VARCHAR(50)",   "nullable": true,  "default": null },
    { "column": "email",         "type": "VARCHAR(100)",  "nullable": true,  "default": null },
    { "column": "hire_date",     "type": "DATE",          "nullable": true,  "default": null },
    { "column": "salary",        "type": "DECIMAL(10,2)", "nullable": true,  "default": null },
    { "column": "department_id", "type": "INTEGER",       "nullable": true,  "default": null }
  ],
  "index_structure": {
    "primary_index": {
      "type": "Unique Primary Index (UPI)",
      "columns": ["employee_id"]
    },
    "secondary_indexes": [
      { "index_no": 1, "type": "Non-Unique Secondary Index (NUSI)", "columns": ["department_id"] },
      { "index_no": 2, "type": "Unique Secondary Index (USI)",      "columns": ["email"] }
    ]
  },
  "storage": {
    "current_size_bytes": 1503238553,
    "current_size_human": "1.4 GB",
    "peak_size_bytes": 1932735283,
    "peak_size_human": "1.8 GB",
    "skew_factor_pct": 8.2,
    "amps": 32
  }
}
```

**Output — CSV Format**:

In CSV format, each section is output as a separate block with a section header comment row, followed by column headers and data rows. This enables downstream filtering by section.

```csv
#section,object_info
type,database,name,created
Table,PRODUCTION,employees,2023-04-15T09:12:33
#section,columns
column,type,nullable,default
employee_id,INTEGER,NO,
first_name,VARCHAR(50),YES,
last_name,VARCHAR(50),YES,
#section,index_structure
...
#section,storage
current_size_bytes,current_size_human,peak_size_bytes,peak_size_human,skew_factor_pct,amps
1503238553,1.4 GB,1932735283,1.8 GB,8.2,32
```

**Behavior Requirements**:

1. **REQ-INSPECT-BATCH-001**: The command SHALL require exactly one object argument; missing argument exits with code 2
2. **REQ-INSPECT-BATCH-002**: When `--section` is specified, only that section is rendered (other sections are fetched but not displayed)
3. **REQ-INSPECT-BATCH-003**: The `--section` flag is valid with all `--format` values
4. **REQ-INSPECT-BATCH-004**: In JSON format, section keys use snake_case (`object_info`, `columns`, `index_structure`, `storage`, `dependencies`)
5. **REQ-INSPECT-BATCH-005**: In JSON format, size values SHALL be expressed both as raw bytes (integer) and human-readable string, to support both machine processing and human review
6. **REQ-INSPECT-BATCH-006**: Section applicability rules are identical to REPL mode (see `docs/specifications/repl.md` REQ-INSPECT-013)
7. **REQ-INSPECT-BATCH-007**: Graceful degradation on DBC permission errors applies identically to REPL mode (see REQ-INSPECT-003.2 and REQ-INSPECT-007.4)
8. **REQ-INSPECT-BATCH-008**: In table format, section separator lines (`── Section Name ─────...`) SHALL be included in `--output` file output but SHALL be omitted when stdout is piped to another process (TTY detection)
9. **REQ-INSPECT-BATCH-009**: In CSV and JSON formats, section separator lines SHALL never be included

**Error Handling**:

**Object Not Found**:
```
Error: Object 'PRODUCTION.employeees' not found.

Suggestions:
  - Check spelling (did you mean 'employees'?)
  - Try a qualified name: tq inspect <database>.<object>

Exit code: 1
```

**Missing Argument**:
```
Error: Missing required argument <object>
Usage: tq inspect <object>

Example: tq inspect employees
         tq inspect production.orders

Exit code: 2
```

**Permission Denied (object lookup)**:
```
Error: Cannot determine object type for 'employees'.
Reason: SELECT permission denied on DBC.TablesV.

Contact your DBA to request access:
  GRANT SELECT ON DBC.TablesV TO <your_username>;

Exit code: 1
```

**Invalid --section value**:
```
Error: Unknown section 'sizes'
Valid sections: info, columns, indexes, storage, dependencies

Exit code: 2
```

**Exit Codes**:
- `0`: Object inspected successfully (even if some sections were unavailable due to permissions — graceful degradation)
- `1`: Object not found, permission error on primary object lookup, or connection failure
- `2`: Usage error (missing argument, invalid flag value)

**Integration with Scripting**:

```bash
# Extract skew factor for all large tables (JSON + jq)
tq sessions --format json | jq -r '.[].TableName' | \
  while read tbl; do
    tq inspect --format json --section storage "$tbl" | \
      jq -r '"$tbl: " + (.storage.skew_factor_pct | tostring) + "%"'
  done

# Export column definitions to CSV for documentation
tq inspect --format csv --section columns production.orders > orders-schema.csv

# Check if a table has high skew before running heavy queries
skew=$(tq inspect --format json --section storage employees | jq '.storage.skew_factor_pct')
if (( $(echo "$skew > 30" | bc -l) )); then
  echo "Warning: High skew on employees table ($skew%)"
fi
```

**Acceptance Tests**:
- Execute `tq inspect <table>` and verify all four sections (Object Info, Columns, Index Structure, Storage) are displayed
- Execute `tq inspect <view>` and verify three sections (Object Info, Columns, Dependencies) are displayed; Storage and Index Structure are absent
- Execute `tq inspect <database>.<object>` (qualified name) and verify correct database resolution
- Execute `tq inspect --format json <table>` and verify valid JSON with `object_info`, `columns`, `index_structure`, `storage` keys; no `dependencies` key for table
- Execute `tq inspect --format csv <table>` and verify CSV output with `#section` separator rows
- Execute `tq inspect --section storage <table>` and verify only Storage data is rendered
- Execute `tq inspect --section storage --format json <table>` and verify JSON contains only `storage` key
- Execute `tq inspect <nonexistent>` and verify not-found error with suggestions (exit code 1)
- Execute `tq inspect` with no argument and verify usage error (exit code 2)
- Execute `tq inspect --section invalid <table>` and verify invalid section error (exit code 2)
- Trigger `DBC.TableSizeV` permission error and verify graceful degradation: other sections render, storage section shows inline note, exit code 0
- Execute `tq inspect --output report.txt <table>` and verify file is created with correct content

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

### profile - Manage Connection Profiles

**Purpose**: Create, update, and delete connection profiles in `~/.tq/config.toml` without manually editing the file.

**Usage**:
```bash
tq profile <SUBCOMMAND> [OPTIONS]
```

**Subcommands**:
| Subcommand | Description |
|------------|-------------|
| `add <name>` | Create a new profile |
| `edit <name>` | Update fields on an existing profile |
| `delete <name>` | Remove a profile |
| `list` | List profiles (alias for `tq profiles`) |

**Relationship to `tq profiles`**: `tq profile list` and `tq profiles` are equivalent commands. `tq profiles` is kept for backwards compatibility and discoverable shorthand.

---

#### profile add

**Purpose**: Create a new named profile in `~/.tq/config.toml`.

**Usage**:
```bash
tq profile add <name> --host <host> [OPTIONS]
```

**Arguments**:
- `<name>`: Profile name (required). Must be a valid TOML key: letters, digits, hyphens, and underscores only.

**Options**:
| Option | Short | Type | Required | Description |
|--------|-------|------|----------|-------------|
| `--host` | - | string | Yes | Database hostname |
| `--port` | - | integer | No | Database port (default: 1025) |
| `--database` | `-d` | string | No | Default database name |
| `--user` | `-u` | string | No | Username |
| `--logmech` | - | enum | No | Auth mechanism: `TD2`, `LDAP`, `KRB5`, `TDNEGO` (default: `TD2`) |
| `--password-file` | - | path | No | Path to file containing the password |

**Examples**:
```bash
# Minimal profile (host only)
tq profile add local --host localhost

# Full profile
tq profile add dev \
  --host dev.company.com \
  --port 1025 \
  --database development \
  --user alice \
  --logmech LDAP \
  --password-file ~/.tq/passwords/dev

# Profile with non-default port
tq profile add dev --host dev.company.com --port 2025
```

**Output (Success)**:
```
Profile 'dev' added to ~/.tq/config.toml
```

**Output (Dry-run confirmation - verbose mode)**:
```
Adding profile 'dev' to ~/.tq/config.toml:
  Host:          dev.company.com:1025
  Database:      development
  User:          alice
  Logmech:       LDAP
  Password file: ~/.tq/passwords/dev

Profile 'dev' added.
```

**Error - Profile Already Exists**:
```
Error: Profile 'dev' already exists in ~/.tq/config.toml

Use 'tq profile edit dev' to update an existing profile.
Use 'tq profile delete dev' to remove it first.
```

Exit code: `1`

**Error - Invalid Profile Name**:
```
Error: Invalid profile name 'my profile'
Profile names may only contain letters, digits, hyphens, and underscores.

Examples of valid names: dev, staging, prod-us, my_db
```

Exit code: `2`

**Error - Missing Required Flag**:
```
Error: Missing required flag --host
Usage: tq profile add <name> --host <host> [OPTIONS]

Example: tq profile add dev --host dev.company.com
```

Exit code: `2`

**Exit Codes**:
- `0`: Profile created successfully
- `1`: Profile already exists, or config file cannot be written
- `2`: Usage error (missing required flag, invalid name, invalid option value)

---

#### profile edit

**Purpose**: Update one or more fields of an existing profile in `~/.tq/config.toml`. Only the flags provided are changed; unspecified fields are left unchanged.

**Usage**:
```bash
tq profile edit <name> [OPTIONS]
```

**Arguments**:
- `<name>`: Name of the profile to edit (required). Must already exist.

**Options**: Same set as `profile add` (all optional):
| Option | Short | Type | Description |
|--------|-------|------|-------------|
| `--host` | - | string | Update database hostname |
| `--port` | - | integer | Update database port |
| `--database` | `-d` | string | Update default database name |
| `--user` | `-u` | string | Update username |
| `--logmech` | - | enum | Update auth mechanism: `TD2`, `LDAP`, `KRB5`, `TDNEGO` |
| `--password-file` | - | path | Update path to password file |

At least one option flag must be provided; calling `tq profile edit <name>` with no flags is an error.

**Examples**:
```bash
# Update just the host
tq profile edit dev --host new-dev.company.com

# Update database and user
tq profile edit dev --database dev2 --user bob

# Switch to LDAP authentication
tq profile edit prod --logmech LDAP
```

**Output (Success)**:
```
Profile 'dev' updated in ~/.tq/config.toml
```

**Output (Verbose)**:
```
Updating profile 'dev' in ~/.tq/config.toml:
  host: dev.company.com -> new-dev.company.com

Profile 'dev' updated.
```

**Error - Profile Not Found**:
```
Error: Profile 'staging' not found in ~/.tq/config.toml

Available profiles: dev, prod, local
Use 'tq profile add staging' to create a new profile.
```

Exit code: `1`

**Error - No Fields Provided**:
```
Error: No fields specified to update.
Provide at least one option flag.

Usage: tq profile edit <name> [--host <host>] [--port <port>] ...
Example: tq profile edit dev --host new-dev.company.com
```

Exit code: `2`

**Exit Codes**:
- `0`: Profile updated successfully
- `1`: Profile not found, or config file cannot be written
- `2`: Usage error (no flags provided, invalid option value)

---

#### profile delete

**Purpose**: Remove a named profile from `~/.tq/config.toml`.

**Usage**:
```bash
tq profile delete <name> [--force]
```

**Arguments**:
- `<name>`: Name of the profile to delete (required). Must already exist.

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--force` | `-f` | flag | false | Skip confirmation prompt |

**Interactive confirmation (TTY, without `--force`)**:
```
Delete profile 'prod' from ~/.tq/config.toml? [y/N] _
```

- Pressing `y` or `Y` proceeds with deletion.
- Any other input (including Enter) aborts.
- Default is `N` (abort), shown in uppercase.

**Non-interactive behaviour (stdin not a TTY)**:

When stdin is not a TTY and `--force` is not provided, the command exits with an error rather than attempting to read from a non-interactive pipe:

```
Error: Interactive confirmation required but stdin is not a terminal.
Use --force to bypass confirmation:
  tq profile delete prod --force
```

Exit code: `2`

**Examples**:
```bash
# With interactive confirmation
tq profile delete prod

# Skip confirmation (scripting, CI/CD)
tq profile delete old-profile --force
```

**Output (Success)**:
```
Profile 'prod' deleted from ~/.tq/config.toml
```

**Output (Abort)**:
```
Aborted. Profile 'prod' was not deleted.
```

Exit code: `0` (abort is not an error)

**Error - Profile Not Found**:
```
Error: Profile 'staging' not found in ~/.tq/config.toml

Available profiles: dev, prod, local
```

Exit code: `1`

**Exit Codes**:
- `0`: Profile deleted successfully, or deletion was aborted interactively
- `1`: Profile not found, or config file cannot be written
- `2`: Usage error (non-interactive without `--force`)

---

#### profile list

**Purpose**: List all available connection profiles. Alias for `tq profiles`.

**Usage**:
```bash
tq profile list
```

**Output**: Identical to `tq profiles`. See [profiles - List Connection Profiles](#profiles---list-connection-profiles) for full output specification.

**Exit Codes**: Same as `tq profiles`.

---

#### profile - Shared Behaviour

**REQ-PROFILE-CLI-001: Config File Creation**

If `~/.tq/config.toml` does not exist when `tq profile add` is run, the tool SHALL create the file (and the `~/.tq/` directory if needed) before writing the profile.

**REQ-PROFILE-CLI-002: Config File Preservation**

All commands that modify the config file MUST preserve all existing content, including comments, whitespace formatting, and sections unrelated to the profile being modified. The tool MUST NOT reformat or reorder the file.

**REQ-PROFILE-CLI-003: Atomic Writes**

Config file writes MUST be atomic: write to a temporary file in the same directory, then rename to replace the original. This prevents partial writes from corrupting the config file.

**REQ-PROFILE-CLI-004: No Password Display**

The `--password-file` path is accepted as input but the path itself is considered sensitive metadata. In verbose output, show the path. Never display the contents of the password file.

**REQ-PROFILE-CLI-005: Logmech Validation**

The `--logmech` flag MUST only accept: `TD2`, `LDAP`, `KRB5`, `TDNEGO` (case-insensitive input, stored in uppercase). Any other value produces:

```
Error: Invalid logmech value 'OAUTH'
Accepted values: TD2, LDAP, KRB5, TDNEGO
```

Exit code: `2`

**REQ-PROFILE-CLI-006: Port Validation**

The `--port` flag MUST accept only integers in the range 1-65535. Any other value produces:

```
Error: Invalid port value '99999'
Port must be an integer between 1 and 65535.
```

Exit code: `2`

**REQ-PROFILE-CLI-007: No Connection Established**

Profile management commands (`add`, `edit`, `delete`, `list`) do NOT connect to the database. They operate solely on the local config file. Connection flags (`--logon`, `--logmech`, `--password-file`, `--profile`) are ignored for these subcommands.

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

## Installation Experience

The install script (`install.sh`) is the primary installation method for Linux
and macOS users. It is the very first CLI interaction a user has with tq, so it
must communicate clearly, succeed silently on the happy path, and provide
actionable guidance when things go wrong.

### Design Principles

- **Minimal noise on success**: Print only essential progress, not verbose internals.
- **Actionable errors**: Every error message tells the user what to do next.
- **Transparent about actions**: The user sees what platform was detected, what
  was downloaded, where it was installed. No surprises.
- **POSIX compatibility**: The script runs under `sh`, `bash`, and `dash`.
  No bashisms.

---

### REQ-INSTALL-001: Happy Path Output

On a successful install, the script SHALL print a concise, structured progress
summary:

```
Detected: macOS aarch64 (Apple Silicon)
Downloading tq 1.22.0... done
Verifying checksum... OK
Installing to /Users/alice/.local/bin/tq... done

tq 1.22.0 installed successfully.

Run:  tq --version
Docs: https://github.com/remi-td/tq
```

Requirements:
1. **REQ-INSTALL-001.1** - The detected OS and architecture SHALL be shown on the first line.
2. **REQ-INSTALL-001.2** - Each step (download, checksum, install) SHALL be on its own line with a trailing `done` or `OK` on completion.
3. **REQ-INSTALL-001.3** - A blank line SHALL precede the success summary.
4. **REQ-INSTALL-001.4** - The post-install call to action (`tq --version`) SHALL always be shown.
5. **REQ-INSTALL-001.5** - The docs URL SHALL be shown so users know where to get help.

---

### REQ-INSTALL-002: PATH Notice

When `~/.local/bin` is not present in the user's `$PATH`, the script SHALL
print an advisory notice after the success summary:

```
Note: ~/.local/bin is not in your PATH.
  Add this line to ~/.bashrc or ~/.zshrc:
    export PATH="$HOME/.local/bin:$PATH"
  Then reload your shell: source ~/.bashrc
```

Requirements:
1. **REQ-INSTALL-002.1** - The notice SHALL appear only when `~/.local/bin` is not in `$PATH`.
2. **REQ-INSTALL-002.2** - The notice SHALL show the exact export line to add, verbatim.
3. **REQ-INSTALL-002.3** - The notice SHALL show both `.bashrc` and `.zshrc` as options.
4. **REQ-INSTALL-002.4** - The notice SHALL show the `source` reload command.
5. **REQ-INSTALL-002.5** - When `TQ_INSTALL_DIR` is set to a non-default location, the
   PATH notice SHALL use that directory instead of `~/.local/bin`.

---

### REQ-INSTALL-003: Custom Install Directory

The install directory defaults to `~/.local/bin` and can be overridden with the
`TQ_INSTALL_DIR` environment variable:

```bash
TQ_INSTALL_DIR=/usr/local/bin curl -sSL .../install.sh | sh
```

Requirements:
1. **REQ-INSTALL-003.1** - When `TQ_INSTALL_DIR` is set, the script SHALL use it as the
   install destination.
2. **REQ-INSTALL-003.2** - The script SHALL display the resolved install path (not the
   variable name) in the progress output.
3. **REQ-INSTALL-003.3** - If `TQ_INSTALL_DIR` does not exist, the script SHALL attempt
   to create it with `mkdir -p` and report success or failure.

---

### REQ-INSTALL-004: Unsupported Platform Error

When the script detects an unsupported OS or architecture, it SHALL exit with a
clear, actionable error:

```
Error: Unsupported platform: Windows x86_64

Prebuilt binaries are not available for this platform via this script.
For manual installation options, visit:
  https://github.com/remi-td/tq/releases

To build from source (requires Rust toolchain):
  https://rustup.rs
  git clone https://github.com/remi-td/tq.git
  cd tq && cargo install --path .
```

Requirements:
1. **REQ-INSTALL-004.1** - The error line SHALL name the detected platform exactly as
   detected (e.g., `Windows x86_64`, `Linux armv7`).
2. **REQ-INSTALL-004.2** - The releases URL SHALL always be included so the user can
   attempt manual download.
3. **REQ-INSTALL-004.3** - Build-from-source instructions SHALL always be included as
   a fallback.
4. **REQ-INSTALL-004.4** - The script SHALL exit with a non-zero exit code.

---

### REQ-INSTALL-005: Download Failure Error

When the binary download fails (network error, 404, rate limit), the script
SHALL show:

```
Error: Download failed (HTTP 404)

Could not download: tq-1.22.0-x86_64-unknown-linux-gnu.tar.gz
From: https://github.com/remi-td/tq/releases/download/v1.22.0/...

Check your network connection, or download manually:
  https://github.com/remi-td/tq/releases
```

Requirements:
1. **REQ-INSTALL-005.1** - The HTTP status code SHALL be included when available.
2. **REQ-INSTALL-005.2** - The exact URL attempted SHALL be shown.
3. **REQ-INSTALL-005.3** - The manual releases URL SHALL be included.
4. **REQ-INSTALL-005.4** - The script SHALL exit with a non-zero exit code.

---

### REQ-INSTALL-006: Checksum Verification Failure

When SHA256 verification fails, the script SHALL abort the install and show:

```
Error: Checksum verification failed

Expected: <expected-sha256>
Got:      <actual-sha256>

The downloaded file may be corrupted or tampered with.
The partial download has been removed.

Try again, or download manually and verify:
  https://github.com/remi-td/tq/releases
```

Requirements:
1. **REQ-INSTALL-006.1** - Both the expected and actual checksums SHALL be shown so
   the user can compare.
2. **REQ-INSTALL-006.2** - The partial/corrupt download SHALL be deleted before exiting.
3. **REQ-INSTALL-006.3** - The script SHALL exit with a non-zero exit code.
4. **REQ-INSTALL-006.4** - The script SHALL never install a binary that failed checksum
   verification.

---

### REQ-INSTALL-007: Missing Dependencies

When required tools (`curl`, `sha256sum` or `shasum`, `tar`) are not available,
the script SHALL report which tool is missing and how to get it:

```
Error: Required tool not found: sha256sum

Install it and try again:
  Ubuntu/Debian: apt install coreutils
  Fedora/RHEL:   dnf install coreutils
  macOS:         shasum is available by default (macOS 10.5+)
```

Requirements:
1. **REQ-INSTALL-007.1** - The missing tool name SHALL be named exactly.
2. **REQ-INSTALL-007.2** - Platform-appropriate install instructions SHALL be shown
   where known.
3. **REQ-INSTALL-007.3** - The script SHALL check for all required tools at startup,
   before beginning any download.
4. **REQ-INSTALL-007.4** - The script SHALL exit with a non-zero exit code.

---

### REQ-INSTALL-008: Permission Error

When the install directory is not writable, the script SHALL show:

```
Error: Cannot write to /usr/local/bin

Permission denied. Try one of:
  Run with sudo:
    curl -sSL .../install.sh | sudo TQ_INSTALL_DIR=/usr/local/bin sh
  Or install to your home directory (default):
    curl -sSL .../install.sh | sh
```

Requirements:
1. **REQ-INSTALL-008.1** - The target directory SHALL be named in the error.
2. **REQ-INSTALL-008.2** - The `sudo` invocation form SHALL be shown correctly
   (the env var must come after `sudo`).
3. **REQ-INSTALL-008.3** - The default home-directory install SHALL always be offered
   as an alternative.

---

### REQ-INSTALL-009: Supported Platforms

The install script SHALL detect and support the following platform combinations:

| OS | Architecture | Detection | Binary Target |
|----|-------------|-----------|---------------|
| Linux | x86_64 | `uname -m` → `x86_64` | `x86_64-unknown-linux-gnu` |
| Linux | aarch64 | `uname -m` → `aarch64` or `arm64` | `aarch64-unknown-linux-gnu` |
| macOS | x86_64 | `uname -s` → `Darwin`, `uname -m` → `x86_64` | `x86_64-apple-darwin` |
| macOS | aarch64 | `uname -s` → `Darwin`, `uname -m` → `arm64` | `aarch64-apple-darwin` |

Requirements:
1. **REQ-INSTALL-009.1** - Platform detection SHALL use `uname -s` (OS) and `uname -m`
   (architecture).
2. **REQ-INSTALL-009.2** - Both `aarch64` and `arm64` from `uname -m` SHALL be
   treated as the same architecture (arm64 is the macOS alias for aarch64).
3. **REQ-INSTALL-009.3** - Windows SHALL be detected and produce the unsupported
   platform error (REQ-INSTALL-004) with a pointer to the `.zip` release artifact.
4. **REQ-INSTALL-009.4** - Any other platform (musl, armv7, FreeBSD, etc.) SHALL
   produce the unsupported platform error (REQ-INSTALL-004).

---

### REQ-INSTALL-010: Teradata License Acceptance

The `tq` binary bundles the Teradata SQL driver library, which is subject to a separate Teradata license agreement. The install script MUST present this license and obtain explicit acceptance before downloading or installing any files.

**Rationale:** The Teradata driver is not open-source software. Redistribution and use require the end-user to accept Teradata's license terms. Acceptance must be recorded before installation to avoid silent redistribution of proprietary software.

#### REQ-INSTALL-010.1: License Display

Before initiating any download, the script SHALL display a concise license notice:

```
Teradata SQL Driver License Notice
====================================
The tq binary includes the Teradata SQL Driver for Python/Rust
(teradatasql), which is proprietary software subject to Teradata's
license agreement.

By installing tq you agree to the Teradata license terms available at:
  https://github.com/Teradata/python-driver/blob/master/LICENSE

The full license text is also included in the downloaded archive as:
  LICENSE.teradata
```

Requirements:
- The notice SHALL appear before any download begins.
- The URL to the full Teradata license SHALL be included in the notice.
- The location of the bundled license file within the archive SHALL be stated.

#### REQ-INSTALL-010.2: Interactive Acceptance Prompt (TTY)

When stdin is a terminal (TTY), the script SHALL display an acceptance prompt immediately after the license notice and wait for user input:

```
Do you accept the Teradata license terms? [y/N] _
```

- Pressing `y` or `Y` followed by Enter proceeds with installation.
- Any other input (including Enter alone, `n`, `N`, or any other character) aborts with the message below and exit code `1`:

```
Installation aborted. You must accept the license terms to install tq.

To accept the license and install, run:
  curl -sSL https://github.com/remi-td/tq/install.sh | sh

Or to accept non-interactively (scripts, CI/CD):
  curl -sSL https://github.com/remi-td/tq/install.sh | sh -s -- --accept-license
```

- The default response is `N` (reject), shown in uppercase in the prompt.
- Input is read from `/dev/tty` directly, not from stdin, to handle piped installations correctly.

#### REQ-INSTALL-010.3: Non-Interactive Acceptance Flag

The `--accept-license` flag allows non-interactive installations to proceed without a TTY prompt:

```bash
# Scripted installation
curl -sSL https://github.com/remi-td/tq/install.sh | sh -s -- --accept-license

# With custom install directory
curl -sSL https://github.com/remi-td/tq/install.sh | sh -s -- --accept-license TQ_INSTALL_DIR=/usr/local/bin
```

When `--accept-license` is provided:
1. The license notice SHALL still be displayed (REQ-INSTALL-010.1).
2. The interactive prompt SHALL be skipped.
3. A confirmation line SHALL be printed to indicate acceptance was provided via flag:

```
License accepted via --accept-license flag.
```

#### REQ-INSTALL-010.4: Piped Installation Without `--accept-license`

When the script detects that stdin is not a TTY (i.e., the script is piped, e.g., `curl | sh`) and `--accept-license` is not provided, the script SHALL exit with code `1` and display:

```
Error: License acceptance required.

This script is running non-interactively (stdin is not a terminal).
To accept the Teradata license terms non-interactively, use:

  curl -sSL https://github.com/remi-td/tq/install.sh | sh -s -- --accept-license

Review the license at:
  https://github.com/Teradata/python-driver/blob/master/LICENSE
```

Requirements:
1. **REQ-INSTALL-010.4.1** - The error SHALL be printed before any download begins.
2. **REQ-INSTALL-010.4.2** - The exact flag syntax for non-interactive acceptance SHALL be shown in the error message.
3. **REQ-INSTALL-010.4.3** - The Teradata license URL SHALL be included so the user can review before re-running.
4. **REQ-INSTALL-010.4.4** - The script SHALL exit with code `1`.

#### REQ-INSTALL-010.5: License File in Archive

The release tar.gz archive SHALL include the Teradata license as a file named `LICENSE.teradata` alongside the binary and driver library:

```
tq-1.0.0-x86_64-unknown-linux-gnu.tar.gz
├── tq                          (binary)
├── libteradatasql.so           (driver library)
└── LICENSE.teradata            (Teradata driver license text)
```

Requirements:
1. **REQ-INSTALL-010.5.1** - `LICENSE.teradata` SHALL contain the full Teradata driver license text, stored in the repository (not fetched remotely).
2. **REQ-INSTALL-010.5.2** - The license file SHALL be included in every release archive for every supported platform.

---
