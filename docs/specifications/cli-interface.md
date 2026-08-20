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
   - [resources - PMON Resource Usage](#resources---pmon-resource-usage)
   - [space - Perm/Spool/Temp Space Analysis](#space---permspooltemp-space-analysis)
   - [dbspace - Database-Level Space Analysis](#dbspace---database-level-space-analysis)
   - [query-inspect - Inspect Session Query Text](#query-inspect---inspect-session-query-text)
   - [inspect - Inspect a Database Object](#inspect---inspect-a-database-object)
   - [describe - Describe Table Structure](#describe---describe-table-structure)
   - [list - List Database Objects](#list---list-database-objects)
   - [search - Search Across Databases](#search---search-across-databases)
   - [show-indexes - Show Table Index Structure](#show-indexes---show-table-index-structure)
   - [fastload - Bulk Load Data (FastLoad)](#fastload---bulk-load-data-fastload)
   - [fastexport - Bulk Export Data (FastExport)](#fastexport---bulk-export-data-fastexport)
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
- `resources` - Display CPU, I/O, and memory metrics from PMON resource usage tables
- `query-inspect` (alias `qi`) - Show SQL text for a specific session
- `inspect` - Comprehensive inspection of a database object (type, columns, indexes, size, dependencies)
- `describe` - Show column structure and indexes for a table or view
- `list` - List database objects: `databases`, `tables [pattern]`, `views`
- `search` - Search for objects across all accessible databases: `tables <keyword>`, `columns <keyword>`, `views <keyword>`, `procedures <keyword>`
- `show-indexes` (alias `di`) - Show index structure for a table
- `params` - Inspect and validate YAML parameter files
- `errorlevel` - Inspect error severity classifications and overrides
- `fastload` - Bulk load CSV, Parquet, or JSON files into a table in parallel
- `fastexport` - Bulk export a table or query directly to a CSV file in parallel
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
| `--color` | - | enum\* | `auto` | Color output: `auto`, `always`, `never` |
| `--params` | `-p` | path | - | YAML parameter file for variable substitution (repeatable) |
| `--profile` | - | string | - | Select connection profile from config file |
| `--agent-safe` | - | flag | false | Enforce agent-safe restrictions globally (env: `$TQ_AGENT_SAFE`) |
| `--json` | - | flag | false | Direct boolean shortcut for `--format json` across all commands |
| `--help` | `-h` | flag | - | Show help |
| `--version` | `-V` | flag | - | Show version |

\* `--color auto` (the default) additionally disables color when the `NO_COLOR` environment variable is set to any non-empty value, regardless of TTY status — see [NO_COLOR](https://no-color.org/). `--color never` and `--color always` are absolute and ignore both `NO_COLOR` and TTY detection. See [Output Formats: Color and Severity Formatting](output-formats.md#color-and-severity-formatting) for how color mode interacts with severity-based coloring.

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
- `<QUERY>`: SQL query string. Mutually exclusive with `--file`. Also mutually exclusive with stdin when stdin is a non-TTY with bytes available (see REQ-CLI-STDIN-002). An empty or TTY stdin is silently ignored when a positional query is provided.

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
| `--watch` | - | flag | off | Auto-refresh display at a fixed interval |
| `--interval` | - | integer | `6`\* | Refresh interval in seconds (min: 2, max: 300). Only valid with `--watch` |

\* The built-in default of 6 seconds is overridden by `[monitoring.thresholds].refresh_interval` in configuration when set (see [Configuration](configuration.md#monitoring-configuration)). The `--interval` flag, when given, always wins over both the config value and the built-in default.

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

# Watch mode: refresh every 6 seconds (default)
tq sessions --watch

# Watch mode: refresh every 10 seconds
tq sessions --watch --interval 10
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
8. **Watch Mode**: When `--watch` is specified, the display auto-refreshes at the given interval (see Watch Mode section below)

**Watch Mode**:

When `--watch` is specified, the command enters a continuous refresh loop:

1. **Format enforcement**: Watch mode forces table output format. `--format csv` and `--format json` are incompatible with `--watch` and SHALL produce a usage error (exit code 2)
2. **Output incompatibility**: `--output` (file output) is incompatible with `--watch` and SHALL produce a usage error (exit code 2)
3. **Screen clearing**: Each refresh clears the terminal and redraws the full table from the top of the screen
4. **Refresh interval**: Controlled by `--interval N` (default: `[monitoring.thresholds].refresh_interval` from configuration, or 6 seconds if unset; minimum: 2, maximum: 300). The `--interval` flag always overrides the configured default. Values outside the allowed range SHALL produce a usage error (exit code 2)
5. **Footer line**: After the summary footer, display: `Last updated: HH:MM:SS | Refreshing every Ns | Press Ctrl-C to stop`
6. **Clean exit**: Ctrl-C exits watch mode cleanly, restoring terminal state, with exit code 0

**Watch mode output example**:
```
Active Sessions on prod-td01.company.com:
┌───────────┬──────────┬────────────────────────┬─────────────┬──────────┬───────────┬───────┬─────────────┬────────────────┬──────────────┐
│ SessionNo │ UserName │ LogonTime              │ PEstate     │ AMPState │ AMPCPUSec │ AMPIO │ ReqSpool    │ Amp CPU Skew % │ Amp IO Skew %│
├───────────┼──────────┼────────────────────────┼─────────────┼──────────┼───────────┼───────┼─────────────┼────────────────┼──────────────┤
│      1076 │ DBC      │ 2026/01/27 15:33:26.00 │ IDLE        │ IDLE     │         0 │     6 │           0 │           [--] │         [--] │
│      1078 │ DBC      │ 2026/01/27 15:33:28.00 │ DISPATCHING │ ACTIVE   │   366.736 │ 75335 │ 26753187840 │           2.87 │         3.78 │
└───────────┴──────────┴────────────────────────┴─────────────┴──────────┴───────────┴───────┴─────────────┴────────────────┴──────────────┘

2 sessions found (Query time: 0.198s)
Last updated: 10:42:15 | Refreshing every 6s | Press Ctrl-C to stop
```

**Incompatible flag error**:
```
Error: --watch is incompatible with --format csv
Watch mode requires table output format.

Exit code: 2
```

**Invalid interval error**:
```
Error: --interval value must be between 2 and 300 seconds (got: 1)

Exit code: 2
```

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
- `0`: Sessions listed successfully (including clean Ctrl-C exit from watch mode)
- `1`: Query error (privilege denied, connection failed, function not available)
- `2`: Usage error (invalid format, invalid output path, incompatible flags with `--watch`, invalid `--interval` value)

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
- Execute `tq sessions --watch` and verify: table redraws on each interval, footer shows "Last updated: HH:MM:SS | Refreshing every 6s | Press Ctrl-C to stop", Ctrl-C exits cleanly with exit code 0
- Execute `tq sessions --watch --interval 10` and verify footer shows "Refreshing every 10s"
- Execute `tq sessions --watch --format csv` and verify usage error (exit code 2)
- Execute `tq sessions --watch --output sessions.txt` and verify usage error (exit code 2)
- Execute `tq sessions --watch --interval 1` and verify usage error for out-of-range interval (exit code 2)

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
| `--watch` | - | flag | off | Auto-refresh display at a fixed interval |
| `--interval` | - | integer | `6`\* | Refresh interval in seconds (min: 2, max: 300). Only valid with `--watch` |

\* The built-in default of 6 seconds is overridden by `[monitoring.thresholds].refresh_interval` in configuration when set (see [Configuration](configuration.md#monitoring-configuration)). The `--interval` flag, when given, always wins over both the config value and the built-in default.

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

# Watch mode: refresh every 6 seconds (default)
tq locks --watch

# Watch mode: refresh every 15 seconds
tq locks --watch --interval 15
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
9. **Watch Mode**: When `--watch` is specified, the display auto-refreshes at the given interval (see Watch Mode section below)

**Watch Mode**:

When `--watch` is specified, the command enters a continuous refresh loop:

1. **Format enforcement**: Watch mode forces table output format. `--format csv` and `--format json` are incompatible with `--watch` and SHALL produce a usage error (exit code 2)
2. **Output incompatibility**: `--output` (file output) is incompatible with `--watch` and SHALL produce a usage error (exit code 2)
3. **Screen clearing**: Each refresh clears the terminal and redraws the full output from the top of the screen, including the blocking chain section when present
4. **Refresh interval**: Controlled by `--interval N` (default: `[monitoring.thresholds].refresh_interval` from configuration, or 6 seconds if unset; minimum: 2, maximum: 300). The `--interval` flag always overrides the configured default. Values outside the allowed range SHALL produce a usage error (exit code 2)
5. **Footer line**: After the summary footer (or "No locks currently held." line), display: `Last updated: HH:MM:SS | Refreshing every Ns | Press Ctrl-C to stop`
6. **Clean exit**: Ctrl-C exits watch mode cleanly, restoring terminal state, with exit code 0

**Watch mode output example (locks present)**:
```
Lock Information:
┌──────────────────────┬───────────┬────────────┬──────────────┬──────────────┐
│ Locked Object        │ Lock Type │ Lock Mode  │ Locking Sess │ Waiting Sess │
├──────────────────────┼───────────┼────────────┼──────────────┼──────────────┤
│ PRODUCTION.orders    │ Table     │ WRITE      │ 1023         │ 1045, 1067   │
│ PRODUCTION.employees │ Row Hash  │ READ       │ 1078         │ (none)       │
└──────────────────────┴───────────┴────────────┴──────────────┴──────────────┘

2 lock(s) found - 1 blocking chain(s) detected (Query time: 0.091s)

Blocking Chain:
  Session 1023 blocks sessions: 1045, 1067

Last updated: 10:42:15 | Refreshing every 6s | Press Ctrl-C to stop
```

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
- `0`: Lock information displayed successfully, including the case of no active locks and clean Ctrl-C exit from watch mode
- `1`: Query error (privilege denied, connection failed, view not available)
- `2`: Usage error (invalid format, invalid output path, incompatible flags with `--watch`, invalid `--interval` value)

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
- Execute `tq locks --watch` and verify: table redraws on each interval, footer shows "Last updated: HH:MM:SS | Refreshing every 6s | Press Ctrl-C to stop", Ctrl-C exits cleanly with exit code 0
- Execute `tq locks --watch --interval 15` and verify footer shows "Refreshing every 15s"
- Execute `tq locks --watch` when no locks exist and verify "No locks currently held." is shown with the watch footer
- Execute `tq locks --watch --format csv` and verify usage error (exit code 2)
- Execute `tq locks --watch --output locks.txt` and verify usage error (exit code 2)
- Execute `tq locks --watch --interval 1` and verify usage error for out-of-range interval (exit code 2)

---

### resources - PMON Resource Usage

**Purpose**: Display CPU, I/O, and memory metrics from Teradata's ResUsage tables for the most recent collection period, enabling DBAs to assess system-wide resource consumption and detect imbalances across VPROCs or physical nodes

**Usage**:
```bash
tq [GLOBAL_OPTIONS] resources [OPTIONS]
```

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--virtual` | - | flag | true | Show per-VPROC metrics from ResUsageSVPR (default mode) |
| `--physical` | - | flag | false | Show per-node metrics from ResUsageSPMA |
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | path | stdout | Write output to file |
| `--page-size` | - | integer | 50 | Number of rows per page in table format |
| `--page` | - | integer | 1 | Page number to display (1-based) |
| `--watch` | - | flag | off | Auto-refresh display at a fixed interval |
| `--interval` | - | integer | `6`\* | Refresh interval in seconds (min: 2, max: 300). Only valid with `--watch` |

\* The built-in default of 6 seconds is overridden by `[monitoring.thresholds].refresh_interval` in configuration when set (see [Configuration](configuration.md#monitoring-configuration)). The `--interval` flag, when given, always wins over both the config value and the built-in default.

Note: `--virtual` and `--physical` are mutually exclusive. If both are specified, the command exits with a usage error (exit code 2).
Note: `--watch` is incompatible with `--format csv`, `--format json`, and `--output`. `--page` and `--page-size` are also incompatible with `--watch` since the full dataset is shown on each refresh.

**Examples**:
```bash
# Show per-VPROC resource metrics (default)
tq resources

# Show per-node (physical) resource metrics
tq resources --physical

# Explicitly request virtual mode (same as default)
tq resources --virtual

# JSON output for scripting
tq resources --format json

# Physical mode with CSV export
tq resources --physical --format csv --output resources.csv
tq resources --physical -f csv -o resources.csv

# Pipe to processing tool
tq resources --format json | jq '.rows[] | select(.["AvgCPU%"] > 80)'

# Using connection profile
tq --profile prod resources

# Page through large result sets
tq resources --page-size 20 --page 2

# Watch mode: refresh every 6 seconds (default)
tq resources --watch

# Watch mode in physical mode: refresh every 30 seconds
tq resources --physical --watch --interval 30
```

**Output (Table Format - Virtual Mode, default)**:
```
Resource Usage (Virtual Mode) — Collection period ending: 2026-04-14 10:15:00

┌───────┬──────────┬──────────┬──────────┬──────────┬────────────┬───────────┐
│ VPROC │ AvgCPU%  │ PeakCPU% │ AvgIO/s  │ PeakIO/s │ MemUsed MB │ MemAvl MB │
├───────┼──────────┼──────────┼──────────┼──────────┼────────────┼───────────┤
│     0 │    12.34 │    45.67 │   1023.4 │   4512.0 │      48234 │     15766 │
│     1 │    11.89 │    38.22 │    987.1 │   3890.5 │      47901 │     16099 │
│     2 │    35.71 │    82.45 │   3102.8 │   9231.6 │      52100 │     11900 │
│     3 │    12.01 │    40.10 │    998.2 │   3754.0 │      48050 │     15950 │
└───────┴──────────┴──────────┴──────────┴──────────┴────────────┴───────────┘

4 VPROCs | Avg CPU: 18.0% | Peak CPU: 82.5% | CPU Skew: 31.2% ⚠ | IO Skew: 28.9% ⚠
(Query time: 0.182s)
```

**Output (Table Format - Physical Mode)**:
```
Resource Usage (Physical Mode) — Collection period ending: 2026-04-14 10:15:00

┌──────┬──────────┬──────────┬──────────────┬──────────────┬────────────┬───────────┐
│ Node │ AvgCPU%  │ PeakCPU% │  AvgIOCnt    │  PeakIOCnt   │ MemUsed MB │ MemAvl MB │
├──────┼──────────┼──────────┼──────────────┼──────────────┼────────────┼───────────┤
│    0 │    14.23 │    48.90 │      2034567 │      8912345 │     192012 │     63988 │
│    1 │    13.55 │    41.30 │      1987234 │      7823109 │     190341 │     65659 │
│    2 │    37.88 │    85.10 │      6203450 │     18423000 │     208400 │     47600 │
│    3 │    13.78 │    43.20 │      1998432 │      7504800 │     191200 │     64800 │
└──────┴──────────┴──────────┴──────────────┴──────────────┴────────────┴───────────┘

4 nodes | Avg CPU: 19.9% | Peak CPU: 85.1% | CPU Skew: 30.8% ⚠ | IO Skew: 29.4% ⚠
(Query time: 0.204s)
```

**Summary Footer Format**:

The footer always appears after the table and contains:
- Count of VPROCs or nodes
- System-wide average CPU%
- System-wide peak CPU%
- CPU skew indicator with warning symbol if skew exceeds threshold
- IO skew indicator with warning symbol if skew exceeds threshold
- Query execution time

Skew warning thresholds:
- `⚠` (warning): skew >= 20%
- No symbol: skew < 20%

**Output (JSON Format)**:
```json
{
  "mode": "virtual",
  "collection_end": "2026-04-14T10:15:00",
  "rows": [
    {
      "VPROC": 0,
      "AvgCPU%": 12.34,
      "PeakCPU%": 45.67,
      "AvgIO/s": 1023.4,
      "PeakIO/s": 4512.0,
      "MemUsedMB": 48234,
      "MemAvailMB": 15766
    },
    {
      "VPROC": 1,
      "AvgCPU%": 11.89,
      "PeakCPU%": 38.22,
      "AvgIO/s": 987.1,
      "PeakIO/s": 3890.5,
      "MemUsedMB": 47901,
      "MemAvailMB": 16099
    }
  ],
  "summary": {
    "count": 4,
    "avg_cpu_pct": 18.0,
    "peak_cpu_pct": 82.45,
    "cpu_skew_pct": 31.2,
    "io_skew_pct": 28.9
  }
}
```

For `--physical` mode, the `mode` field is `"physical"`, rows use `"Node"` instead of `"VPROC"`, and `"AvgIO/s"`/`"PeakIO/s"` become `"AvgIOCnt"`/`"PeakIOCnt"` (raw I/O counts from the physical table).

**Output (CSV Format)**:
```csv
VPROC,AvgCPU%,PeakCPU%,AvgIO/s,PeakIO/s,MemUsedMB,MemAvailMB
0,12.34,45.67,1023.4,4512.0,48234,15766
1,11.89,38.22,987.1,3890.5,47901,16099
2,35.71,82.45,3102.8,9231.6,52100,11900
3,12.01,40.10,998.2,3754.0,48050,15950
```

Note: The summary footer is omitted from CSV output. The collection period timestamp is also omitted.

**Column Descriptions (Virtual Mode)**:

| Column | Type | Source | Description |
|--------|------|--------|-------------|
| VPROC | INTEGER | ResUsageSVPR.vproc | Virtual processor ID |
| AvgCPU% | DECIMAL(5,2) | Derived from ResUsageSVPR | Average CPU utilization percentage during the collection period |
| PeakCPU% | DECIMAL(5,2) | Derived from ResUsageSVPR | Peak CPU utilization percentage during the collection period |
| AvgIO/s | DECIMAL(10,1) | Derived from ResUsageSVPR | Average I/O operations per second during the collection period |
| PeakIO/s | DECIMAL(10,1) | Derived from ResUsageSVPR | Peak I/O operations per second during the collection period |
| MemUsed MB | INTEGER | ResUsageSVPR | Memory used in megabytes at the end of the collection period |
| MemAvl MB | INTEGER | ResUsageSVPR | Memory available in megabytes at the end of the collection period |

**Column Descriptions (Physical Mode)**:

| Column | Type | Source | Description |
|--------|------|--------|-------------|
| Node | INTEGER | ResUsageSPMA.nodenumber | Physical node number |
| AvgCPU% | DECIMAL(5,2) | Derived from ResUsageSPMA | Average CPU utilization percentage during the collection period |
| PeakCPU% | DECIMAL(5,2) | Derived from ResUsageSPMA | Peak CPU utilization percentage during the collection period |
| AvgIOCnt | BIGINT | ResUsageSPMA | Average total I/O count during the collection period |
| PeakIOCnt | BIGINT | ResUsageSPMA | Peak total I/O count during the collection period |
| MemUsed MB | INTEGER | ResUsageSPMA | Memory used in megabytes at the end of the collection period |
| MemAvl MB | INTEGER | ResUsageSPMA | Memory available in megabytes at the end of the collection period |

**Behavior Requirements**:

1. **Standalone Operation**: Does NOT require a SQL file argument
2. **Default Mode**: Virtual mode (`--virtual`) is the default when neither `--virtual` nor `--physical` is specified
3. **Data Source (Virtual)**: Queries `ResUsageSVPR` for the most recent collection period
4. **Data Source (Physical)**: Queries `ResUsageSPMA` for the most recent collection period
5. **Most Recent Period**: Both modes retrieve only the single most recent completed collection period (identified by the maximum `TheDate`/`TheTime` combination)
6. **Row Ordering**: Rows are sorted ascending by VPROC ID (virtual mode) or Node number (physical mode)
7. **Skew Calculation**: CPU skew and IO skew are computed as `(max - avg) / max * 100` across all VPROCs or nodes
8. **Skew Warning**: The `⚠` symbol appears in the summary footer when skew >= 20%
9. **Pagination**: `--page-size` and `--page` apply only to table format output. JSON and CSV always return all rows
10. **Format Compatibility**: Works with `--format table`, `--format csv`, `--format json`
11. **Output Destination**: Respects `--output` flag for file output, otherwise stdout
12. **Collection Timestamp**: Displayed as a header line in table format; present in JSON `collection_end` field; omitted from CSV
13. **Watch Mode**: When `--watch` is specified, the display auto-refreshes at the given interval (see Watch Mode section below)

**Watch Mode**:

When `--watch` is specified, the command enters a continuous refresh loop:

1. **Format enforcement**: Watch mode forces table output format. `--format csv` and `--format json` are incompatible with `--watch` and SHALL produce a usage error (exit code 2)
2. **Output incompatibility**: `--output` (file output) is incompatible with `--watch` and SHALL produce a usage error (exit code 2)
3. **Pagination incompatibility**: `--page` and `--page-size` are incompatible with `--watch`. The full dataset is always displayed on each refresh. Specifying them alongside `--watch` SHALL produce a usage error (exit code 2)
4. **Screen clearing**: Each refresh clears the terminal and redraws the full table from the top of the screen, including the collection period header and the summary footer
5. **Refresh interval**: Controlled by `--interval N` (default: `[monitoring.thresholds].refresh_interval` from configuration, or 6 seconds if unset; minimum: 2, maximum: 300). The `--interval` flag always overrides the configured default. Values outside the allowed range SHALL produce a usage error (exit code 2)
6. **Footer line**: After the summary footer, display: `Last updated: HH:MM:SS | Refreshing every Ns | Press Ctrl-C to stop`
7. **Clean exit**: Ctrl-C exits watch mode cleanly, restoring terminal state, with exit code 0
8. **Mode compatibility**: `--watch` can be combined with either `--virtual` (default) or `--physical`

**Watch mode output example (virtual mode)**:
```
Resource Usage (Virtual Mode) — Collection period ending: 2026-04-14 10:15:00

┌───────┬──────────┬──────────┬──────────┬──────────┬────────────┬───────────┐
│ VPROC │ AvgCPU%  │ PeakCPU% │ AvgIO/s  │ PeakIO/s │ MemUsed MB │ MemAvl MB │
├───────┼──────────┼──────────┼──────────┼──────────┼────────────┼───────────┤
│     0 │    12.34 │    45.67 │   1023.4 │   4512.0 │      48234 │     15766 │
│     2 │    35.71 │    82.45 │   3102.8 │   9231.6 │      52100 │     11900 │
└───────┴──────────┴──────────┴──────────┴──────────┴────────────┴───────────┘

2 VPROCs | Avg CPU: 24.0% | Peak CPU: 82.5% | CPU Skew: 57.8% ⚠ | IO Skew: 51.2% ⚠
(Query time: 0.175s)
Last updated: 10:42:15 | Refreshing every 6s | Press Ctrl-C to stop
```

**Error Handling**:

**Insufficient Privileges**:
```
Error: Unable to retrieve resource usage data
Reason: SELECT permission denied on ResUsageSVPR

This command requires SELECT access to ResUsage tables.
Contact your DBA to request access or use the GRANT statement:
  GRANT SELECT ON ResUsageSVPR TO <your_username>;
  GRANT SELECT ON ResUsageSPMA TO <your_username>;

Exit code: 1
```

**No Data Available**:
```
Resource Usage (Virtual Mode) — No data available

No resource usage data found in ResUsageSVPR.
The ResUsage logging may not be enabled on this system.
Contact your DBA to enable ResUsage logging (PMON feature).

(Query time: 0.041s)
```

**Mutually Exclusive Flags**:
```
Error: --virtual and --physical are mutually exclusive
Usage: tq resources [--virtual | --physical]

Exit code: 2
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
- `0`: Resource usage displayed successfully, including the case of no data and clean Ctrl-C exit from watch mode
- `1`: Query error (privilege denied, connection failed, table not available)
- `2`: Usage error (mutually exclusive flags, invalid format, invalid output path, incompatible flags with `--watch`, invalid `--interval` value)

**Integration with Scripting**:

The `resources` command is designed for both interactive use and automation:

```bash
# Alert when any VPROC CPU exceeds 80%
tq resources --format json | \
  jq '.rows[] | select(.["AvgCPU%"] > 80) | .VPROC' | \
  while read vp; do echo "High CPU on VPROC $vp"; done

# Export snapshot for trending
tq resources --physical --format csv \
  --output "/var/log/td-resources/$(date +%Y%m%d_%H%M%S)_physical.csv"

# Check system-wide CPU skew
SKEW=$(tq resources --format json | jq '.summary.cpu_skew_pct')
if (( $(echo "$SKEW > 25" | bc -l) )); then
  echo "WARNING: High CPU skew detected ($SKEW%)"
fi

# Extract memory pressure across all VPROCs
tq resources --format json | \
  jq '.rows[] | {vproc: .VPROC, used_pct: (.MemUsedMB / (.MemUsedMB + .MemAvailMB) * 100)}'
```

**Acceptance Test**:
- Execute `tq resources` with no flags and verify virtual mode output with all 7 columns: VPROC, AvgCPU%, PeakCPU%, AvgIO/s, PeakIO/s, MemUsed MB, MemAvl MB
- Execute `tq resources --physical` and verify physical mode output with all 7 columns: Node, AvgCPU%, PeakCPU%, AvgIOCnt, PeakIOCnt, MemUsed MB, MemAvl MB
- Verify collection period timestamp appears as a header line in table format
- Verify summary footer includes VPROC/node count, avg CPU%, peak CPU%, CPU skew%, IO skew%
- Verify `⚠` appears in footer when skew >= 20%, absent when skew < 20%
- Execute `tq resources --format csv` and verify CSV output with correct column headers (no footer)
- Execute `tq resources --format json` and verify valid JSON with `mode`, `collection_end`, `rows`, and `summary` keys
- Execute `tq resources --output resources.txt` and verify file creation
- Execute `tq resources --virtual --physical` and verify usage error (exit code 2)
- Execute when ResUsageSVPR has no rows and verify "No data available" message (exit code 0)
- Trigger privilege error and verify helpful error message with GRANT examples (exit code 1)
- Verify rows are sorted ascending by VPROC ID in virtual mode and by Node in physical mode
- Execute `tq resources --page-size 10 --page 2` and verify second page of rows is shown
- Execute `tq resources --watch` and verify: table redraws on each interval, footer shows "Last updated: HH:MM:SS | Refreshing every 6s | Press Ctrl-C to stop", Ctrl-C exits cleanly with exit code 0
- Execute `tq resources --physical --watch --interval 30` and verify physical mode redraws every 30 seconds
- Execute `tq resources --watch --format csv` and verify usage error (exit code 2)
- Execute `tq resources --watch --output resources.txt` and verify usage error (exit code 2)
- Execute `tq resources --watch --page-size 10` and verify usage error (exit code 2)
- Execute `tq resources --watch --interval 1` and verify usage error for out-of-range interval (exit code 2)

---

### space - Perm/Spool/Temp Space Analysis

**Purpose**: Give a database administrator a one-command view of permanent, spool, and temporary space usage and allocation for a database and the objects it contains, without hand-writing `DBC.DiskSpaceV` / `DBC.TableSizeV` queries.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] space [OPTIONS] <TARGET>
```

**Arguments**:
- `<TARGET>`: Required. Either a database name (`<database>`) or a fully qualified object name (`<database>.<object>`). The presence of a `.` determines which of the two invocation shapes below applies.

**Two invocation shapes**:

| Invocation | Behavior |
|------------|----------|
| `tq space <database>` | Returns one row for the database itself (aggregate perm/spool/temp metrics) followed by one row per object directly contained in that database. |
| `tq space <database>.<object>` | Returns exactly one row: perm-space metrics for that single object. |

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv`, `markdown` (alias: `md`) |
| `--output` | `-o` | path | stdout | Write output to file |

**Examples**:
```bash
# Database rollup + one row per object
tq space demo_user

# Single object detail
tq space demo_user.orders

# JSON for scripting/alerting
tq space demo_user --format json

# CSV export for spreadsheet analysis
tq space demo_user --format csv --output demo_user_space.csv

# Using a connection profile
tq --profile prod space production.orders

# Markdown for a report or GitHub issue
tq space demo_user --format markdown
```

**Metrics returned**:

| Row kind | Metrics |
|----------|---------|
| Object (single-object form, and each object row in the database form) | `CurrentPerm`, `PeakPerm`, `PermSkew%` |
| Database (database form's header row) | All object metrics, plus `MaxPerm`, `PermUsed%` (percentage of `MaxPerm` currently used), and Spool/Temp (`Current`, `Max`, `Peak`, `Skew%` each) |

**Skew formula** (applies identically to `CurrentPerm`, `CurrentSpool`, `CurrentTemp`, computed across the per-AMP rows for the object or database being measured):

```
Skew% = 100 - (AVG(Current<X>) / NULLIFZERO(MAX(Current<X>)) * 100)
```

This is the exact formula from issue #54 and SHALL be used verbatim (not `(max - avg) / max * 100`, which is the formula used by `tq resources`/`tq skew` for CPU/IO skew — the two commands intentionally use different but related formulas per their respective source issues).

**Output (Table Format — `tq space <database>`)**:
```
Space Analysis — Database: demo_user

┌──────────┬────────────┬─────────────┬────────────┬──────────┬───────────┬───────────┬───────────┬────────────┬───────────┬─────────┬────────────┬───────────┬───────────┬───────────┐
│ Kind     │ Object     │ CurrentPerm │ PeakPerm   │ PermSkew%│ MaxPerm   │ PermUsed% │ SpoolCurr │ SpoolMax   │ SpoolPeak │SpoolSkew%│ TempCurr   │ TempMax   │ TempPeak  │ TempSkew% │
├──────────┼────────────┼─────────────┼────────────┼──────────┼───────────┼───────────┼───────────┼────────────┼───────────┼─────────┼────────────┼───────────┼───────────┼───────────┤
│ DATABASE │ demo_user  │     6.8 GB  │     7.1 GB │      4.2 │   10.0 GB │      68.0 │  120.0 MB │     2.0 GB │  340.0 MB │     12.5 │       0 B  │   1.0 GB  │      0 B  │    [--]   │
│ TABLE    │ customers  │     2.1 GB  │     2.2 GB │      2.0 │        -  │        -  │        -  │        -   │        -  │       -  │        -   │       -   │       -   │       -   │
│ TABLE    │ logs       │     1.5 GB  │     1.5 GB │      0.0 │        -  │        -  │        -  │        -   │        -  │       -  │        -   │       -   │       -   │       -   │
│ TABLE    │ orders     │     3.2 GB  │     3.4 GB │      5.1 │        -  │        -  │        -  │        -   │        -  │       -  │        -   │       -   │       -   │       -   │
└──────────┴────────────┴─────────────┴────────────┴──────────┴───────────┴───────────┴───────────┴────────────┴───────────┴─────────┴────────────┴───────────┴───────────┴───────────┘

4 rows (1 database, 3 objects) | Total object CurrentPerm: 6.8 GB
(Query time: 0.087s)
```

In a narrower terminal, this wide table is subject to the standard [column truncation rules](output-formats.md#column-truncation-strategy) (leftmost columns prioritized, `(+n cols)` indicator) — use `--format csv`, `--format json`, or pipe to `less` to see every column at once.

**Output (Table Format — `tq space <database>.<object>`)**:
```
Space Analysis — Object: demo_user.orders

┌──────────┬────────┬─────────────┬───────────┬───────────┐
│ Database │ Object │ CurrentPerm │ PeakPerm  │ PermSkew% │
├──────────┼────────┼─────────────┼───────────┼───────────┤
│ demo_user│ orders │      3.2 GB │    3.4 GB │       5.1 │
└──────────┴────────┴─────────────┴───────────┴───────────┘

(Query time: 0.041s)
```

**Output (JSON Format — `tq space <database>`)**:

Object rows omit the database-only keys entirely (they are not applicable), rather than setting them to `null` — matching the section-omission convention used by `tq inspect` (REQ-INSPECT-BATCH-004/006). Byte fields are raw integers; there are no human-readable string mirrors in JSON or CSV for this command (unlike `tq inspect`) — this keeps threshold/scripting comparisons (`jq '.[] | select(.current_perm_bytes > N)'`) unambiguous.

```json
[
  {
    "kind": "DATABASE",
    "object": "demo_user",
    "current_perm_bytes": 7301444780,
    "peak_perm_bytes": 7645041818,
    "perm_skew_pct": 4.2,
    "max_perm_bytes": 10737418240,
    "perm_used_pct": 68.0,
    "spool_current_bytes": 125829120,
    "spool_max_bytes": 2147483648,
    "spool_peak_bytes": 356515840,
    "spool_skew_pct": 12.5,
    "temp_current_bytes": 0,
    "temp_max_bytes": 1073741824,
    "temp_peak_bytes": 0,
    "temp_skew_pct": null
  },
  {
    "kind": "TABLE",
    "object": "customers",
    "current_perm_bytes": 2254857830,
    "peak_perm_bytes": 2361393152,
    "perm_skew_pct": 2.0
  },
  {
    "kind": "TABLE",
    "object": "orders",
    "current_perm_bytes": 3435973836,
    "peak_perm_bytes": 3650722201,
    "perm_skew_pct": 5.1
  }
]
```

**Output (JSON Format — `tq space <database>.<object>`)**:
```json
{
  "database": "demo_user",
  "object": "orders",
  "current_perm_bytes": 3435973836,
  "peak_perm_bytes": 3650722201,
  "perm_skew_pct": 5.1
}
```

**Output (CSV Format — `tq space <database>`)**:
```csv
Kind,Object,CurrentPerm,PeakPerm,PermSkew%,MaxPerm,PermUsed%,SpoolCurrent,SpoolMax,SpoolPeak,SpoolSkew%,TempCurrent,TempMax,TempPeak,TempSkew%
DATABASE,demo_user,7301444780,7645041818,4.2,10737418240,68.0,125829120,2147483648,356515840,12.5,0,1073741824,0,
TABLE,customers,2254857830,2361393152,2.0,,,,,,,,,,
TABLE,orders,3435973836,3650722201,5.1,,,,,,,,,,
```

Note: CSV, like JSON, uses raw byte integers (never humanized strings). Inapplicable and NULL fields are both rendered as empty fields, per the standard [CSV NULL representation](output-formats.md#null-representation-1).

**Byte-size humanization (table and markdown formats only)**:

1. **REQ-SPACE-HUMAN-001**: In `table` and `markdown` formats, all byte-valued fields (`CurrentPerm`, `PeakPerm`, `MaxPerm`, `SpoolCurrent`, `SpoolMax`, `SpoolPeak`, `TempCurrent`, `TempMax`, `TempPeak`) SHALL be rendered as human-readable strings using base-1024 units (`B`, `KB`, `MB`, `GB`, `TB`) with one decimal place, matching the convention established by `tq list tables` (see REQ-LIST-010), e.g. `3.2 GB`, `890.0 KB`, `0 B`.
2. **REQ-SPACE-HUMAN-002**: In `json` and `csv` formats, the same fields SHALL be rendered as raw byte integers (snake_case keys suffixed `_bytes` in JSON; unsuffixed integer columns in CSV). No humanized mirror fields are produced in these two formats.
3. **REQ-SPACE-HUMAN-003**: Percentage fields (`PermSkew%`, `PermUsed%`, `SpoolSkew%`, `TempSkew%`) are never humanized; they are always rendered as a plain decimal number (one decimal place in table/markdown) in every format.

**NULL and "not applicable" handling**:

1. **REQ-SPACE-NULL-001**: A percentage field computed via `NULLIFZERO` that evaluates to NULL (zero denominator, e.g. an empty object where `MAX(CurrentPerm) = 0`) SHALL be displayed as `[--]` in table/markdown, an empty field in CSV, and `null` in JSON — matching the existing NULL-skew convention used by `tq sessions` (see the NULL skew handling for `tq sessions`, above).
2. **REQ-SPACE-NULL-002**: A field that does not apply to a given row (e.g. `MaxPerm` on a `TABLE` row within `tq space <database>`) SHALL be displayed as `-` in table/markdown and an empty field in CSV. In JSON, inapplicable keys are omitted from that row's object entirely (not present as `null`) — this lets a consumer distinguish "not applicable" (key absent) from "computed but NULL" (`key: null`).
3. **REQ-SPACE-NULL-003**: `MaxPerm = 0` in Teradata conventionally means "no perm space limit" (unlimited), not "zero capacity". When `MaxPerm = 0`, `PermUsed%` SHALL be displayed as `[unlimited]` in table/markdown (not `[--]`), remains `null` in JSON, and the JSON object additionally includes a sibling boolean key `"perm_unlimited": true` (this key is entirely absent — not `false` — when `MaxPerm` is greater than zero). In CSV, the field remains an empty field (CSV has no room for a second boolean signal); use `--format json` to script around the unlimited case.

**Behavior Requirements**:

1. **REQ-SPACE-001**: The target argument SHALL be parsed by presence of a single `.` separator: no dot → database form; exactly one dot → single-object form. A target with more than one dot SHALL produce a usage error (exit code 2): `Error: Invalid object reference '<target>' — expected <database> or <database>.<object>`.
2. **REQ-SPACE-002**: Database-level metrics are sourced from `DBC.DiskSpaceV`, aggregated per database across its per-AMP rows. Object-level metrics are sourced from `DBC.TableSizeV`, aggregated per object across its per-AMP rows. (Exact column availability — including `PeakPerm` on `DBC.DiskSpaceV`, which issue #54's reference SQL assumes exists — SHALL be verified against a live Teradata system before implementation; do not assume undocumented columns exist.)
3. **REQ-SPACE-003**: In the database form, the database's own row SHALL always be the first row returned, with `Kind = DATABASE`. Object rows follow, `Kind = TABLE`, sorted alphabetically by object name (matching the sort convention of `tq list tables`, REQ-LIST-009).
4. **REQ-SPACE-004**: Object rows in the database form include only tables (and other objects that carry a physical storage footprint in `DBC.TableSizeV`, per live-system verification) — views carry no storage and SHALL NOT appear as rows.
5. **REQ-SPACE-005**: If the database named in the database form contains zero objects, the command SHALL still return the single `DATABASE` row (an empty database is not an error).
6. **REQ-SPACE-006**: `--format` and `--output` follow the same semantics as all other tq commands (see [Flag Design Guidelines](#flag-design-guidelines)).
7. **REQ-SPACE-007**: The table format summary footer (database form only) reports the row count broken out as `N rows (1 database, M objects)`, the total `CurrentPerm` across object rows, and query execution time. The summary footer is omitted from CSV and JSON output, consistent with `tq resources`.
8. **REQ-SPACE-008**: Severity coloring (Normal/Warning/Critical) is applied to `PermUsed%`, `PermSkew%`, `SpoolSkew%`, and `TempSkew%` cells in table format per [Output Formats: Color and Severity Formatting](output-formats.md#color-and-severity-formatting) and the `space_warning`/`space_critical`/`skew_warning`/`skew_critical` thresholds in [Configuration: Monitoring Configuration](configuration.md#monitoring-configuration).

**Error Handling**:

**Database Not Found**:
```
Error: Database 'demo_usre' not found.

Did you mean: demo_user

Exit code: 1
```

**Object Not Found**:
```
Error: Object 'demo_user.orderss' not found.

Did you mean: orders

Exit code: 1
```

**Invalid Target Format**:
```
Error: Invalid object reference 'a.b.c' — expected <database> or <database>.<object>
Usage: tq space <database>[.<object>]

Examples:
  tq space demo_user
  tq space demo_user.orders

Exit code: 2
```

**Missing Argument**:
```
Error: Missing required argument <target>
Usage: tq space <database>[.<object>]

Exit code: 2
```

**Insufficient Privileges**:
```
Error: Unable to retrieve space data for 'demo_user'
Reason: SELECT permission denied on DBC.DiskSpaceV

Contact your DBA to request access:
  GRANT SELECT ON DBC.DiskSpaceV TO <your_username>;
  GRANT SELECT ON DBC.TableSizeV TO <your_username>;

Exit code: 1
```

**Exit Codes**:
- `0`: Space data displayed successfully, including an empty database (database row only, zero object rows)
- `1`: Database/object not found, permission error, or connection failure
- `2`: Usage error (missing/malformed target argument, invalid flag value)

**Integration with Scripting**:
```bash
# Alert when a database is above 90% perm allocation
PCT=$(tq space demo_user --format json | jq '.[] | select(.kind=="DATABASE") | .perm_used_pct')
if (( $(echo "$PCT > 90" | bc -l) )); then
  echo "WARNING: demo_user is at $PCT% of MaxPerm"
fi

# Find the largest object in a database
tq space demo_user --format json | \
  jq -r '.[] | select(.kind=="TABLE") | [.object, .current_perm_bytes] | @tsv' | \
  sort -k2 -n -r | head -1

# Export a full space snapshot for trending
tq space demo_user --format csv --output "/var/log/td-space/$(date +%Y%m%d)_demo_user.csv"
```

**Acceptance Tests**:
- Execute `tq space <db>` and verify the first row has `Kind=DATABASE` and every subsequent row has `Kind=TABLE`, sorted alphabetically by object name
- Execute `tq space <db>.<object>` and verify exactly one row is returned with only `Database`, `Object`, `CurrentPerm`, `PeakPerm`, `PermSkew%` columns
- Execute `tq space <db>` on an empty database and verify one `DATABASE` row with zero object rows and exit code 0
- Execute `tq space <db> --format json` and verify object rows omit `max_perm_bytes`, `perm_used_pct`, and all `spool_*`/`temp_*` keys entirely
- Execute `tq space <db> --format csv` and verify inapplicable object-row fields render as empty CSV fields, and all byte fields are raw integers (no unit suffix)
- Verify `PermSkew%`/`SpoolSkew%`/`TempSkew%`/`PermUsed%` render as `[--]` when the underlying `NULLIFZERO` denominator is zero, and confirm this is `null` in JSON and empty in CSV
- Verify `PermUsed%` renders as `[unlimited]` (not `[--]`) and JSON carries `"perm_unlimited": true` when `MaxPerm = 0`
- Execute `tq space a.b.c` and verify usage error (exit code 2)
- Execute `tq space nonexistent_db` and verify not-found error with spelling suggestion (exit code 1)
- Execute `tq space demo_user.nonexistent_object` and verify not-found error with spelling suggestion (exit code 1)
- Trigger a `DBC.DiskSpaceV`/`DBC.TableSizeV` permission error and verify the GRANT-example error message (exit code 1)
- Execute `tq space demo_user --format markdown` and verify a valid GitHub-Flavored Markdown table

---

### dbspace - Database-Level Space Analysis

**Purpose**: A focused variant of `tq space` that returns database-level perm/spool/temp metrics only, and clearly rejects any input that isn't a plain database name — for administrators who specifically want the database rollup and want a loud error if they accidentally pass an object reference.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] dbspace [OPTIONS] <DATABASE>
```

**Arguments**:
- `<DATABASE>`: Required. A single database (or user) name. Must NOT contain a `.` — qualified `database.object` references are rejected (use `tq space <database>.<object>` instead).

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv`, `markdown` (alias: `md`) |
| `--output` | `-o` | path | stdout | Write output to file |

**Examples**:
```bash
# Database-level space rollup
tq dbspace demo_user

# JSON for scripting/alerting
tq dbspace demo_user --format json

# Using a connection profile
tq --profile prod dbspace production
```

**Output (Table Format)**:
```
Space Analysis — Database: demo_user

┌───────────┬─────────────┬───────────┬───────────┬───────────┬───────────┬─────────────┬───────────┬───────────┬───────────┬────────────┬───────────┬───────────┬───────────┐
│ Database  │ CurrentPerm │ PeakPerm  │ PermSkew% │ MaxPerm   │ PermUsed% │ SpoolCurrent│ SpoolMax  │ SpoolPeak │SpoolSkew% │ TempCurrent│ TempMax   │ TempPeak  │ TempSkew% │
├───────────┼─────────────┼───────────┼───────────┼───────────┼───────────┼─────────────┼───────────┼───────────┼───────────┼────────────┼───────────┼───────────┼───────────┤
│ demo_user │      6.8 GB │    7.1 GB │       4.2 │   10.0 GB │      68.0 │    120.0 MB │    2.0 GB │  340.0 MB │      12.5 │       0 B  │   1.0 GB  │      0 B  │   [--]    │
└───────────┴─────────────┴───────────┴───────────┴───────────┴───────────┴─────────────┴───────────┴───────────┴───────────┴────────────┴───────────┴───────────┴───────────┘

(Query time: 0.052s)
```

**Output (JSON Format)**:
```json
{
  "database": "demo_user",
  "current_perm_bytes": 7301444780,
  "peak_perm_bytes": 7645041818,
  "perm_skew_pct": 4.2,
  "max_perm_bytes": 10737418240,
  "perm_used_pct": 68.0,
  "spool_current_bytes": 125829120,
  "spool_max_bytes": 2147483648,
  "spool_peak_bytes": 356515840,
  "spool_skew_pct": 12.5,
  "temp_current_bytes": 0,
  "temp_max_bytes": 1073741824,
  "temp_peak_bytes": 0,
  "temp_skew_pct": null
}
```

Note: unlike `tq space <database>`, this is a single JSON object, not an array — `tq dbspace` always returns exactly one entity.

**Output (CSV Format)**:
```csv
Database,CurrentPerm,PeakPerm,PermSkew%,MaxPerm,PermUsed%,SpoolCurrent,SpoolMax,SpoolPeak,SpoolSkew%,TempCurrent,TempMax,TempPeak,TempSkew%
demo_user,7301444780,7645041818,4.2,10737418240,68.0,125829120,2147483648,356515840,12.5,0,1073741824,0,
```

**Behavior Requirements**:

1. **REQ-DBSPACE-001**: The `<DATABASE>` argument SHALL NOT contain a `.`. If it does, the command SHALL produce a usage error (exit code 2) directing the user to `tq space <database>.<object>` — it SHALL NOT silently fall back to object-level behavior.
2. **REQ-DBSPACE-002**: If `<DATABASE>` does not resolve to an entry in `DBC.DatabasesV` (a database or user), the command SHALL produce a not-found error (exit code 1) with a spelling suggestion where available, identical in style to `tq describe`'s not-found error.
3. **REQ-DBSPACE-003**: If `<DATABASE>` resolves to an object that is not a database/user (e.g., it is only a table name reachable under the session's default database), the command SHALL produce a clear "not a database" error (exit code 1) rather than guessing intent, and SHALL suggest the correct command: `tq space <database>.<object>`.
4. **REQ-DBSPACE-004**: Column set, humanization, NULL/"not applicable"/"unlimited" handling, and the skew formula are identical to the database row produced by `tq space <database>` (see REQ-SPACE-HUMAN-001 through REQ-SPACE-NULL-003) — `tq dbspace <db>` is equivalent in content to the first row of `tq space <db>`, minus the `Kind` column (always a database, so the column is redundant) and without any object rows.
5. **REQ-DBSPACE-005**: `--format` and `--output` follow the same semantics as all other tq commands.
6. **REQ-DBSPACE-006**: Severity coloring is applied identically to `tq space`'s database row (see REQ-SPACE-008).

**Error Handling**:

**Qualified Name Rejected**:
```
Error: tq dbspace requires a plain database name, not a qualified object reference.

  Given:    demo_user.orders
  Expected: tq dbspace demo_user
  Or:       tq space demo_user.orders   (for object-level detail)

Exit code: 2
```

**Not a Database**:
```
Error: 'orders' is not a database.
Reason: 'orders' is a table under database 'demo_user', not a database or user.

Try:
  tq space demo_user.orders   (object-level detail)
  tq dbspace demo_user        (database-level rollup)

Exit code: 1
```

**Database Not Found**:
```
Error: Database 'demo_usre' not found.

Did you mean: demo_user

Exit code: 1
```

**Missing Argument**:
```
Error: Missing required argument <database>
Usage: tq dbspace <database>

Exit code: 2
```

**Insufficient Privileges**:
```
Error: Unable to retrieve space data for 'demo_user'
Reason: SELECT permission denied on DBC.DiskSpaceV

Contact your DBA to request access:
  GRANT SELECT ON DBC.DiskSpaceV TO <your_username>;

Exit code: 1
```

**Exit Codes**:
- `0`: Database space data displayed successfully
- `1`: Database not found, target is not a database, permission error, or connection failure
- `2`: Usage error (qualified name given, missing argument, invalid flag value)

**Integration with Scripting**:
```bash
# Nightly capacity check across a list of databases
for db in demo_user analytics staging; do
  PCT=$(tq dbspace "$db" --format json | jq '.perm_used_pct')
  echo "$db: ${PCT}% of MaxPerm used"
done

# Fail a CI job if a database is unexpectedly near its perm limit
tq dbspace production --format json | jq -e '.perm_used_pct < 85' > /dev/null \
  || { echo "production database is above 85% perm allocation"; exit 1; }
```

**Acceptance Tests**:
- Execute `tq dbspace <db>` and verify a single row with all 14 metric columns (no `Kind` column)
- Execute `tq dbspace <db>.<object>` and verify a usage error (exit code 2) that names `tq space <db>.<object>` as the alternative
- Execute `tq dbspace <table_name_used_as_db>` (a name that exists only as a table, not a database) and verify the "not a database" error (exit code 1)
- Execute `tq dbspace nonexistent_db` and verify not-found error with spelling suggestion (exit code 1)
- Execute `tq dbspace` with no argument and verify usage error (exit code 2)
- Execute `tq dbspace demo_user --format json` and verify a single JSON object (not an array) with all 14 metric keys present
- Verify the numeric content of `tq dbspace <db>` and the `DATABASE` row of `tq space <db>` are identical for the same database
- Trigger a `DBC.DiskSpaceV` permission error and verify the GRANT-example error message (exit code 1)

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
│ first_name    │ VARCHAR(50)  │ YES      │ -       │
│ last_name     │ VARCHAR(50)  │ YES      │ -       │
│ email         │ VARCHAR(100) │ YES      │ -       │
│ hire_date     │ DATE         │ YES      │ -       │
│ salary        │ DECIMAL(10,2)│ YES      │ -       │
│ department_id │ INTEGER      │ YES      │ -       │
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
4. **REQ-INSPECT-BATCH-004**: In JSON format, section keys use snake_case (`object_info`, `columns`, `index_structure`, `storage`, `definition`, `dependencies`)
5. **REQ-INSPECT-BATCH-005**: In JSON format, size values SHALL be expressed both as raw bytes (integer) and human-readable string, to support both machine processing and human review
6. **REQ-INSPECT-BATCH-006**: Section applicability rules are identical to REPL mode (see `docs/specifications/repl.md` REQ-INSPECT-014)
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

### describe - Describe Table Structure

**Purpose**: Display the column structure and index information for a table or view. This is the batch-mode equivalent of the REPL `/describe` command, optimized for scripting, documentation, and quick schema lookups without entering the interactive session.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] describe [OPTIONS] <OBJECT>
```

**Arguments**:
- `<OBJECT>`: Required. An object name (`employees`) or a fully qualified name (`database.employees`). Object names are case-insensitive (Teradata convention).

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | path | stdout | Write output to file |

**Examples**:
```bash
# Describe a table in the current database
tq describe employees

# Describe using a qualified name
tq describe production.orders

# JSON output for scripting
tq describe --format json employees

# CSV output for documentation generation
tq describe --format csv production.orders > orders-schema.csv

# Using a connection profile
tq --profile prod describe employees

# Write report to file
tq describe --output employees-schema.txt employees
```

**Output — Table Format (no column comments)**:
```
── Object ──
  Type:      Table
  Database:  PRODUCTION
  Name:      employees

── Columns (7) ──
  Column                   Type                 Nullable   Default
  ----------------------------------------------------------------------
  employee_id              INTEGER              NO         -
  first_name               VARCHAR(50)          YES        -
  last_name                VARCHAR(50)          YES        -
  email                    VARCHAR(100)         YES        -
  hire_date                DATE                 YES        -
  salary                   DECIMAL(10,2)        YES        -
  department_id            INTEGER              YES        -
  7 column(s)

── Indexes ──
  Primary Index (UPI): employee_id
  Secondary Index (NUSI): department_id
  Secondary Index (USI): email

```

**Output — Table Format (with column comments)**:

When any column has a comment, a `Comment` column is added to the right. The `Comment` column is omitted entirely when no columns have comments.

```
── Object ──
  Type:      Table
  Database:  PRODUCTION
  Name:      employees

── Columns (7) ──
  Column                   Type                 Nullable   Default         Comment
  ------------------------------------------------------------------------------------------
  employee_id              INTEGER              NO         -               Primary key
  first_name               VARCHAR(50)          YES        -
  last_name                VARCHAR(50)          YES        -
  email                    VARCHAR(100)         YES        -               Unique contact email
  hire_date                DATE                 YES        -
  salary                   DECIMAL(10,2)        YES        -
  department_id            INTEGER              YES        -
  7 column(s)

── Indexes ──
  Primary Index (UPI): employee_id
  Secondary Index (NUSI): department_id
  Secondary Index (USI): email

```

**Output — CSV Format**:
```csv
# Object: PRODUCTION.employees (Table)
Column,Type,Nullable,Default,Comment
employee_id,INTEGER,NO,-,
first_name,VARCHAR(50),YES,-,
last_name,VARCHAR(50),YES,-,
email,VARCHAR(100),YES,-,
hire_date,DATE,YES,-,
salary,DECIMAL(10,2),YES,-,
department_id,INTEGER,YES,-,
```

**Output — JSON Format**:
```json
{
  "object": {
    "database": "PRODUCTION",
    "name": "employees",
    "type": "Table"
  },
  "columns": [
    { "name": "employee_id",   "type": "INTEGER",        "nullable": false, "default": null },
    { "name": "first_name",    "type": "VARCHAR(50)",    "nullable": true,  "default": null },
    { "name": "last_name",     "type": "VARCHAR(50)",    "nullable": true,  "default": null },
    { "name": "email",         "type": "VARCHAR(100)",   "nullable": true,  "default": null },
    { "name": "hire_date",     "type": "DATE",           "nullable": true,  "default": null },
    { "name": "salary",        "type": "DECIMAL(10,2)",  "nullable": true,  "default": null },
    { "name": "department_id", "type": "INTEGER",        "nullable": true,  "default": null }
  ],
  "indexes": [
    { "name": "(unnamed)", "type": "UPI",  "columns": ["employee_id"] },
    { "name": "(unnamed)", "type": "NUSI", "columns": ["department_id"] },
    { "name": "(unnamed)", "type": "USI",  "columns": ["email"] }
  ]
}
```

When a column has a comment, a `"comment"` key is included on that column entry. When absent, the key is omitted entirely (not `null`). Example:
```json
{ "name": "employee_id", "type": "INTEGER", "nullable": false, "default": null, "comment": "Primary key" }
```

When an index has a name, the `"name"` field holds that name. Unnamed indexes use `"(unnamed)"`.

**Behavior Requirements**:

1. **REQ-DESCRIBE-001**: The command SHALL require exactly one object argument; invoking without an argument SHALL exit with code 2 and print the usage error.
2. **REQ-DESCRIBE-002**: Object names SHALL be treated case-insensitively (Teradata convention). `tq describe DBC.TABLES`, `tq describe dbc.tables`, and `tq describe Dbc.Tables` SHALL all resolve to the same object.
3. **REQ-DESCRIBE-003**: Data sources:
   - Object type: `DBC.TablesV`
   - Column definitions (in ordinal order): `DBC.ColumnsV`
   - Index structure: `DBC.IndicesV`
4. **REQ-DESCRIBE-004**: The `Default` column SHALL display `-` (table/CSV format) when no column default is defined, and the actual default value (as stored in the catalog) when one is present.
5. **REQ-DESCRIBE-005**: Column comments from the Teradata catalog SHALL be shown in a `Comment` column only when at least one column has a non-empty comment. When no columns have comments, the `Comment` column SHALL be omitted entirely from the output.
6. **REQ-DESCRIBE-006**: Columns SHALL be displayed in ordinal position order (`ColumnId` ascending), matching the table definition order.
7. **REQ-DESCRIBE-007**: The `── Indexes ──` section SHALL be shown only for tables (not views). For tables, if no indexes exist, display `No indexes defined.`. For views, the indexes section is omitted silently.
8. **REQ-DESCRIBE-008**: Table format output uses `──` section headers (`── Object ──`, `── Columns (N) ──`, `── Indexes ──`) with two-space indented content below each header.
9. **REQ-DESCRIBE-009**: In JSON output, `nullable` SHALL be a boolean (`true`/`false`, never a string), `default` SHALL be `null` when no default is set (never the string `"-"`), and `comment` SHALL be included as a string key only when a non-empty comment exists (not present when absent).
10. **REQ-DESCRIBE-010**: In CSV output, a comment header (`# Object: database.name (Type)`) precedes the column rows, and column headers use `Column,Type,Nullable,Default,Comment`.
11. **REQ-DESCRIBE-011**: The `--format` and `--output` flags follow the same semantics as all other tq commands (see [Flag Design Guidelines](#flag-design-guidelines)).
12. **REQ-DESCRIBE-012**: For object type reporting, the same `TableKind` mapping used by `tq inspect` (see `docs/specifications/repl.md` REQ-INSPECT-002.2) SHALL be applied.
13. **REQ-DESCRIBE-013**: In table format, the index display is inline, one line per index: `  <Full Label> (<Short>): col1, col2`. Named indexes include the name in quotes: `  <Full Label> (<Short>) "<name>": col1, col2`.

**Relationship to `tq inspect`**: `tq describe` is a focused schema command — it shows columns and index structure only. Use `tq inspect` when storage metrics, object dependencies, or a full consolidated view are needed.

**Error Handling**:

**Object Not Found**:
```
Error: Object 'PRODUCTION.employeees' not found.

Did you mean: employees

Exit code: 1
```

**Missing Argument**:
```
Error: Missing required argument <object>
Usage: tq describe <object>

Examples:
  tq describe employees
  tq describe production.orders

Exit code: 2
```

**Insufficient Privileges**:
```
Error: Cannot describe 'employees'.
Reason: SELECT permission denied on DBC.TablesV.

Contact your DBA to request access:
  GRANT SELECT ON DBC.TablesV TO <your_username>;

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
- `0`: Object described successfully
- `1`: Object not found, permission error, or connection failure
- `2`: Usage error (missing argument, invalid flag value)

**Integration with Scripting**:
```bash
# List all column names for a table in JSON
tq describe --format json employees | jq '.columns[].name'

# Extract nullable columns to CSV
tq describe --format json employees | \
  jq -r '.columns[] | select(.nullable) | [.name, .type] | @csv'

# Generate schema documentation
tq --profile prod describe --format csv production.orders > orders-schema.csv

# Check if a column exists before running a query
tq describe --format json employees | \
  jq -e '.columns[] | select(.name == "salary")' > /dev/null && \
  echo "salary column exists"
```

**Acceptance Tests**:
- Execute `tq describe <table>` and verify `──` section headers for Object, Columns, and Indexes sections; columns shown as aligned plain-text rows (no box-drawing borders)
- Execute `tq describe <table>` where no columns have comments and verify the `Comment` column is absent
- Execute `tq describe <table>` where at least one column has a comment and verify the `Comment` column appears in the output
- Execute `tq describe <view>` and verify columns are shown, the Indexes section is absent
- Execute `tq describe <database>.<object>` (qualified name) and verify correct database resolution
- Execute `tq describe --format json <table>` and verify valid JSON with `object` (containing `database`, `name`, `type`), `columns`, and `indexes` keys; `nullable` is a boolean (`true`/`false`), `default` is `null` when absent (not the string `"-"`), and `comment` key appears only for columns that have comments
- Execute `tq describe --format csv <table>` and verify comment header row followed by `Column,Type,Nullable,Default,Comment` header and data rows
- Execute `tq describe <nonexistent>` and verify not-found error (exit code 1)
- Execute `tq describe` with no argument and verify usage error (exit code 2)
- Execute `tq describe --output schema.txt <table>` and verify file is created with correct content
- Verify column `Default` shows `-` in table/CSV format when no default is defined; `null` in JSON format
- Verify object names are treated case-insensitively: `tq describe DBC.TABLES` and `tq describe dbc.tables` return the same result

---

### list - List Database Objects

**Purpose**: List database objects (databases, tables, or views) accessible to the connected user. This is the batch-mode equivalent of the REPL `/list` family of commands, designed for scripting, auditing, and pipeline use.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] list <SUBCOMMAND> [OPTIONS] [ARGS]
```

**Subcommands**:
| Subcommand | Arguments | Description |
|------------|-----------|-------------|
| `databases` | (none) | List all accessible databases |
| `tables` | `[pattern]` | List tables in the current or specified database, with optional name filter |
| `views` | (none) | List views in the current or specified database |

**Options** (shared across all subcommands):
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | path | stdout | Write output to file |
| `--database` | `-d` | string | (from logon string) | Target database for `tables` and `views` subcommands |

**Examples**:
```bash
# List all accessible databases
tq list databases

# List tables in the current (logon) database
tq list tables

# List tables matching a glob pattern (* = any characters, ? = single character)
tq list tables 'emp*'

# List tables in a specific database
tq list tables --database staging

# List tables in a specific database with a pattern
tq list tables --database staging 'test_*'

# List views
tq list views

# JSON output for scripting
tq list databases --format json

# CSV export to file
tq list tables --format csv --output tables.csv

# Using a connection profile
tq --profile prod list databases

# Pipe to jq for filtering
tq list databases --format json | jq '.[] | select(.type == "User") | .name'
```

---

#### list databases

**Purpose**: List all databases accessible to the current user.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] list databases [OPTIONS]
```

**Output — Table Format**:
```
Databases (5):
Name                           Owner                Type
------------------------------------------------------------
DBC                            DBC                  System
analytics                      analytics            User
development                    dev_user             User
production                     dba_user             User
staging                        dba_user             User

5 database(s)
```

**Output — CSV Format**:
```csv
DatabaseName,Owner,Type
DBC,DBC,System
analytics,analytics,User
development,dev_user,User
production,dba_user,User
staging,dba_user,User
```

**Output — JSON Format**:
```json
[
  { "name": "DBC",         "owner": "DBC",        "type": "System" },
  { "name": "analytics",   "owner": "analytics",  "type": "User"   },
  { "name": "development", "owner": "dev_user",   "type": "User"   },
  { "name": "production",  "owner": "dba_user",   "type": "User"   },
  { "name": "staging",     "owner": "dba_user",   "type": "User"   }
]
```

**Output — No Databases Found**:
```
No databases found.
```

---

#### list tables

**Purpose**: List tables in the current or specified database, with optional name-pattern filtering.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] list tables [OPTIONS] [PATTERN]
```

**Arguments**:
- `[PATTERN]`: Optional. A glob pattern to filter table names. Uses `*` (any sequence of characters) and `?` (any single character) as wildcards. Case-insensitive. If omitted, all tables are listed. Glob syntax is more natural for CLI use than SQL LIKE patterns.

**Pattern examples**:
- `emp*` — all tables whose name starts with `emp`
- `*_archive` — all tables ending with `_archive`
- `sales_2024_*` — all tables starting with `sales_2024_`
- `emp?oyees` — tables matching with any single character in the `?` position

**Output — Table Format (all tables)**:
```
Tables in (current):
Name                                Type     Rows (Est.)     Size
-----------------------------------------------------------------
customers                           TABLE      1234567     45.2 MB
employees                           TABLE        42573      2.1 MB
orders                              TABLE      9876543    320.5 MB
products                            TABLE        15432      890 KB

4 table(s)
```

**Output — Table Format (with pattern)**:
```
Tables in (current) matching 'emp*':
Name                                Type     Rows (Est.)     Size
-----------------------------------------------------------------
employees                           TABLE        42573      2.1 MB
emp_archive                         TABLE         8123    512.0 KB

2 table(s)
```

**Output — No Tables Found**:
```
0 table(s)
```

**Output — CSV Format**:
```csv
TableName,Type,RowsEst,Size
customers,TABLE,1234567,45.2 MB
employees,TABLE,42573,2.1 MB
orders,TABLE,9876543,320.5 MB
products,TABLE,15432,890.0 KB
```

**Output — JSON Format**:

The JSON output uses string representations for rows and size as returned from the database. Integer conversion of raw row counts and byte values is handled by the caller. The `rows_est` and `size` fields are strings matching the table display format.

```json
[
  { "name": "customers",  "type": "TABLE", "rows_est": "1234567", "size": "45.2 MB"   },
  { "name": "employees",  "type": "TABLE", "rows_est": "42573",   "size": "2.1 MB"    },
  { "name": "orders",     "type": "TABLE", "rows_est": "9876543", "size": "320.5 MB"  },
  { "name": "products",   "type": "TABLE", "rows_est": "15432",   "size": "890.0 KB"  }
]
```

---

#### list views

**Purpose**: List views in the current or specified database.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] list views [OPTIONS]
```

**Output — Table Format**:
```
Views in (current):
Name                           Owner
----------------------------------------
active_employees               dba_user
customer_orders_view           dba_user
sales_summary                  analytics

3 view(s)
```

**Output — No Views Found**:
```
0 view(s)
```

**Output — CSV Format**:
```csv
ViewName,Owner
active_employees,dba_user
customer_orders_view,dba_user
sales_summary,analytics
```

**Output — JSON Format**:
```json
[
  { "name": "active_employees",     "owner": "dba_user"  },
  { "name": "customer_orders_view", "owner": "dba_user"  },
  { "name": "sales_summary",        "owner": "analytics" }
]
```

---

#### list — Behavior Requirements

1. **REQ-LIST-001**: The `list` command SHALL require a subcommand (`databases`, `tables`, `views`); invoking `tq list` with no subcommand SHALL print subcommand help and exit with code 2. Invoking with an unknown subcommand SHALL print `Error: Unknown list subcommand: <name>` followed by the available subcommands, and exit with code 2.
2. **REQ-LIST-002**: `list databases` data source: `DBC.DatabasesV`. Columns displayed: Name, Owner, Type.
3. **REQ-LIST-003**: `list databases` type classification: databases owned by `DBC` SHALL be shown as type `System`; all others SHALL be shown as type `User`.
4. **REQ-LIST-004**: `list databases` sorting: alphabetical by database name.
5. **REQ-LIST-005**: `list databases` JSON key for database name SHALL be `"name"`.
6. **REQ-LIST-006**: `list tables` data source: `DBC.TablesV WHERE TableKind IN ('T', 'O')` joined with `DBC.TableSizeV` for row count and size estimates. Columns displayed: Name, Type, Rows (Est.), Size.
7. **REQ-LIST-007**: `list tables` with `--database <db>` SHALL list tables in the specified database, regardless of the database in the logon string. Without `--database`, the command targets the current (session default) database.
8. **REQ-LIST-008**: `list tables [PATTERN]` filter: the pattern uses glob syntax with `*` (any sequence of characters) and `?` (any single character). Matching is case-insensitive. Glob syntax is preferred over SQL LIKE because it follows standard shell conventions familiar to CLI users.
9. **REQ-LIST-009**: `list tables` sorting: alphabetical by table name.
10. **REQ-LIST-010**: `list tables` size values: in all formats (table, CSV, JSON), size SHALL be displayed as a human-readable string (`890.0 KB`, `2.1 MB`, `320.5 MB`). The `rows_est` and `size` values in JSON and CSV are strings matching the display format. Raw byte values are not exposed — use `tq inspect` when precise byte counts are needed.
11. **REQ-LIST-011**: `list views` data source: `DBC.TablesV WHERE TableKind = 'V'` joined to get owner from `CreatorName`. Columns displayed: Name, Owner.
12. **REQ-LIST-012**: `list views` with `--database <db>` SHALL list views in the specified database. Without `--database`, the command targets the current (session default) database.
13. **REQ-LIST-013**: `list views` sorting: alphabetical by view name.
14. **REQ-LIST-014**: When a subcommand returns no results, the count line (`0 table(s)`, `0 view(s)`, `0 database(s)`) SHALL be displayed and the command SHALL exit with code 0 (not an error).
15. **REQ-LIST-015**: The `--format` and `--output` flags follow the same semantics as all other tq commands (see [Flag Design Guidelines](#flag-design-guidelines)).
16. **REQ-LIST-016**: In JSON format, all list subcommands return a JSON array (`[]`). An empty result set returns an empty array `[]`, not `null` or an error.

**Error Handling**:

**Unknown Subcommand**:
```
Error: Unknown subcommand 'schema'
Usage: tq list <databases|tables|views> [OPTIONS] [ARGS]

Available subcommands:
  databases   List all accessible databases
  tables      List tables in the current or specified database
  views       List views in the current or specified database

Exit code: 2
```

**Missing Subcommand**:
```
Usage: tq list <databases|tables|views> [OPTIONS] [ARGS]

Available subcommands:
  databases   List all accessible databases
  tables      List tables in the current or specified database
  views       List views in the current or specified database

Exit code: 2
```

**No Current Database (tables and views subcommands)**:
```
Error: No database specified.
The --database flag or the /database component of --logon is required for 'tq list tables'.

Examples:
  tq -l user:pass@host/mydb list tables
  tq list tables --database mydb

Exit code: 2
```

**Insufficient Privileges**:
```
Error: Unable to list tables in database 'production'.
Reason: SELECT permission denied on DBC.TablesV.

Contact your DBA to request access:
  GRANT SELECT ON DBC.TablesV TO <your_username>;

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
- `0`: Objects listed successfully (including the case of zero results)
- `1`: Query error (privilege denied, connection failed, view not available)
- `2`: Usage error (missing or unknown subcommand, missing database, invalid flag value)

**Integration with Scripting**:
```bash
# Find all databases with "staging" in the name
tq list databases --format json | jq -r '.[] | select(.name | test("staging";"i")) | .name'

# Count tables per database (using --database flag)
for db in $(tq list databases --format json | jq -r '.[].name'); do
  count=$(tq list tables --database "$db" --format json | jq 'length')
  echo "$db: $count tables"
done

# Find all views in staging
tq list views --database staging --format json | jq '.[].name'

# Export table inventory to CSV
tq --profile prod list tables --format csv --output tables-$(date +%Y%m%d).csv
```

**Acceptance Tests**:
- Execute `tq list databases` and verify plain-text table output with Name, Owner, Type columns
- Execute `tq list databases --format json` and verify JSON array with `"name"`, `"owner"`, `"type"` keys (not `"database"`); empty result is `[]`
- Execute `tq list databases --format csv` and verify CSV with `DatabaseName,Owner,Type` header row
- Execute `tq list databases` and verify DBC-owned databases show type `System`, others show type `User`
- Execute `tq list tables` with a valid database in logon string and verify table output with Name, Type, Rows (Est.), Size columns
- Execute `tq list tables 'emp*'` (glob) and verify only tables whose names start with `emp` are shown
- Execute `tq list tables 'emp?loyees'` (glob `?`) and verify single-character wildcard matching
- Execute `tq list tables --database staging` and verify tables from the `staging` database are listed
- Execute `tq list tables 'xyz_nonexistent*'` and verify `0 table(s)` output with exit code 0
- Execute `tq list views` and verify output with Name and Owner columns; alphabetical order
- Execute `tq list views --format json` and verify JSON array with `"name"` and `"owner"` keys
- Execute `tq list views --format csv` and verify CSV with `ViewName,Owner` header
- Execute `tq list` with no subcommand and verify usage help is shown with exit code 2
- Execute `tq list unknown_sub` and verify `Error: Unknown list subcommand: unknown_sub` message with exit code 2
- Execute `tq list databases --output db-list.txt` and verify file is created

---

### search - Search Across Databases

**Purpose**: Search for tables or columns by keyword across all accessible databases. Unlike `tq list tables`, which lists objects within a single database, `search` queries the entire system catalog and returns matching objects regardless of which database they belong to. This is the primary discovery tool when users or agents do not know which database contains the objects they are looking for.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] search <SUBCOMMAND> [OPTIONS] <KEYWORD>
```

**Subcommands**:
| Subcommand | Arguments | Description |
|------------|-----------|-------------|
| `tables` | `<keyword>` | Search for tables whose name contains the keyword, across all accessible databases |
| `columns` | `<keyword>` | Search for columns whose name contains the keyword, across all accessible tables and databases |
| `views` | `<keyword>` | Search for views whose name contains the keyword, across all accessible databases |
| `procedures` | `<keyword>` | Search for stored procedures whose name contains the keyword, across all accessible databases |

**Options** (shared across all subcommands):
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv`, `md` |
| `--output` | `-o` | path | stdout | Write output to file |
| `--database` | `-d` | string | (all databases) | Scope search to a single database |
| `--limit` | `-n` | integer | 100 | Maximum number of results; use `0` for unlimited. Mutually exclusive with `--page-size`. |
| `--page-size` | - | integer | (disabled) | Number of rows per page. Enables pagination. Mutually exclusive with `--limit`. |
| `--page` | - | integer | `1` | Page number to retrieve (1-based). Requires `--page-size`. |

**Key Distinction from `list`**:
- `tq list tables` — lists objects in **one** database (current or `--database`)
- `tq search tables <keyword>` — searches for objects **across all** databases, filtered by a name keyword
- `tq list views` — lists views in **one** database
- `tq search views <keyword>` — searches for views **across all** databases, filtered by a name keyword
- `tq search procedures <keyword>` — searches for stored procedures **across all** databases, filtered by a name keyword

**Keyword Matching**:
The keyword argument is matched against object names using SQL `LIKE` with automatic leading and trailing wildcards. A keyword of `emp` is equivalent to the SQL pattern `%emp%`. The match is case-insensitive.

- `emp` — matches `employees`, `temp_emp`, `emp_archive`, `dept_emp_map`
- `salary` — matches `salary`, `base_salary`, `salary_history`
- `2024` — matches `sales_2024_q1`, `archive_2024`

Users who need exact-prefix or exact-suffix matching should use `tq list tables` with a glob pattern instead.

**Examples**:
```bash
# Search for tables containing "emp" anywhere in the name, across all databases
tq search tables emp

# Search tables in a specific database only
tq search tables emp --database production

# Search for columns named like "salary" across all databases
tq search columns salary

# Search for columns in a specific database
tq search columns salary --database hr

# Search for views containing "summary" across all databases
tq search views summary

# Search for views in a specific database
tq search views summary --database reporting

# Search for stored procedures containing "update" across all databases
tq search procedures update

# Search for procedures in a specific database
tq search procedures update --database hr

# JSON output (uses standard envelope) for agent/scripting use
tq search tables emp --format json

# CSV output for spreadsheet analysis
tq search columns id --format csv --output columns.csv

# Markdown output for documentation
tq search tables order --format md

# Paginate large result sets
tq search views report --page-size 25 --page 1

# Pipe to jq to filter by database
tq search tables emp --format json | jq '.data[] | select(.database == "production")'

# Using a connection profile
tq --profile prod search tables customer
```

---

#### search tables

**Purpose**: Find tables whose names match a keyword, across all accessible databases (or one database when `--database` is specified).

**Usage**:
```bash
tq [GLOBAL_OPTIONS] search tables [OPTIONS] <KEYWORD>
```

**Arguments**:
- `<KEYWORD>`: Required. A plain string. Automatically wrapped as `%keyword%` for SQL LIKE matching. Case-insensitive.

**Output Columns**:
| Column | Description |
|--------|-------------|
| Database | Database that contains the table |
| Table | Table name |
| Type | Object kind: `TABLE`, `TABLE (NoPI)` |
| Rows (Est.) | Estimated row count (human-readable) |
| Size | Estimated storage size (human-readable) |

**Sorting**: Results are sorted first by Database name (ascending), then by Table name (ascending).

**Output — Table Format**:
```
Search results for tables matching 'emp' (3 found):
Database             Table                      Type         Rows (Est.)     Size
---------------------------------------------------------------------------------
analytics            emp_summary                TABLE             12,400    1.2 MB
hr                   employees                  TABLE             42,573    2.1 MB
staging              emp_archive                TABLE              8,123   512 KB

3 table(s) found
```

**Output — Table Format (with --database)**:
```
Search results for tables matching 'emp' in database 'hr' (1 found):
Database             Table                      Type         Rows (Est.)     Size
---------------------------------------------------------------------------------
hr                   employees                  TABLE             42,573    2.1 MB

1 table(s) found
```

**Output — Table Format (no results)**:
```
No tables found matching 'emp'.
```

**Output — JSON Format**:

JSON output uses the standard envelope: `{"ok": true, "row_count": N, "data": [...]}`.

```json
{
  "ok": true,
  "row_count": 3,
  "data": [
    { "database": "analytics", "table": "emp_summary",  "type": "TABLE", "rows_est": "12400",  "size": "1.2 MB"  },
    { "database": "hr",        "table": "employees",    "type": "TABLE", "rows_est": "42573",  "size": "2.1 MB"  },
    { "database": "staging",   "table": "emp_archive",  "type": "TABLE", "rows_est": "8123",   "size": "512 KB"  }
  ]
}
```

When no results are found:
```json
{
  "ok": true,
  "row_count": 0,
  "data": []
}
```

**Output — CSV Format**:
```csv
Database,Table,Type,RowsEst,Size
analytics,emp_summary,TABLE,12400,1.2 MB
hr,employees,TABLE,42573,2.1 MB
staging,emp_archive,TABLE,8123,512 KB
```

**Output — Markdown Format**:
```markdown
| Database  | Table        | Type  | Rows (Est.) | Size   |
|-----------|--------------|-------|-------------|--------|
| analytics | emp_summary  | TABLE | 12,400      | 1.2 MB |
| hr        | employees    | TABLE | 42,573      | 2.1 MB |
| staging   | emp_archive  | TABLE | 8,123       | 512 KB |
```

---

#### search columns

**Purpose**: Find columns whose names match a keyword, across all accessible tables and databases (or one database when `--database` is specified).

**Usage**:
```bash
tq [GLOBAL_OPTIONS] search columns [OPTIONS] <KEYWORD>
```

**Arguments**:
- `<KEYWORD>`: Required. A plain string. Automatically wrapped as `%keyword%` for SQL LIKE matching. Case-insensitive.

**Output Columns**:
| Column | Description |
|--------|-------------|
| Database | Database that contains the parent table |
| Table | Table that contains the column |
| Column | Column name |
| Type | Teradata data type (e.g., `INTEGER`, `VARCHAR(100)`, `DECIMAL(10,2)`) |
| Nullable | Whether the column accepts NULL: `YES` or `NO` |

**Sorting**: Results are sorted first by Database name (ascending), then by Table name (ascending), then by Column name (ascending).

**Output — Table Format**:
```
Search results for columns matching 'salary' (4 found):
Database     Table               Column               Type              Nullable
--------------------------------------------------------------------------------
hr           employees           salary               DECIMAL(10,2)     YES
hr           salary_bands        base_salary          DECIMAL(12,2)     NO
hr           salary_bands        max_salary           DECIMAL(12,2)     NO
payroll      payroll_history     gross_salary         DECIMAL(12,2)     YES

4 column(s) found
```

**Output — Table Format (with --database)**:
```
Search results for columns matching 'salary' in database 'hr' (3 found):
Database     Table               Column               Type              Nullable
--------------------------------------------------------------------------------
hr           employees           salary               DECIMAL(10,2)     YES
hr           salary_bands        base_salary          DECIMAL(12,2)     NO
hr           salary_bands        max_salary           DECIMAL(12,2)     NO

3 column(s) found
```

**Output — Table Format (no results)**:
```
No columns found matching 'salary'.
```

**Output — JSON Format**:

JSON output uses the standard envelope: `{"ok": true, "row_count": N, "data": [...]}`.

```json
{
  "ok": true,
  "row_count": 4,
  "data": [
    { "database": "hr",      "table": "employees",       "column": "salary",       "type": "DECIMAL(10,2)", "nullable": "YES" },
    { "database": "hr",      "table": "salary_bands",    "column": "base_salary",  "type": "DECIMAL(12,2)", "nullable": "NO"  },
    { "database": "hr",      "table": "salary_bands",    "column": "max_salary",   "type": "DECIMAL(12,2)", "nullable": "NO"  },
    { "database": "payroll", "table": "payroll_history", "column": "gross_salary", "type": "DECIMAL(12,2)", "nullable": "YES" }
  ]
}
```

When no results are found:
```json
{
  "ok": true,
  "row_count": 0,
  "data": []
}
```

**Output — CSV Format**:
```csv
Database,Table,Column,Type,Nullable
hr,employees,salary,DECIMAL(10_2),YES
hr,salary_bands,base_salary,DECIMAL(12_2),NO
hr,salary_bands,max_salary,DECIMAL(12_2),NO
payroll,payroll_history,gross_salary,DECIMAL(12_2),YES
```

Note: Parentheses in type strings (e.g., `DECIMAL(10,2)`) are preserved in CSV and JSON output. The underscore shown above is for illustration only; the actual output preserves the exact Teradata type string.

**Output — Markdown Format**:
```markdown
| Database | Table           | Column       | Type          | Nullable |
|----------|-----------------|--------------|---------------|----------|
| hr       | employees       | salary       | DECIMAL(10,2) | YES      |
| hr       | salary_bands    | base_salary  | DECIMAL(12,2) | NO       |
| hr       | salary_bands    | max_salary   | DECIMAL(12,2) | NO       |
| payroll  | payroll_history | gross_salary | DECIMAL(12,2) | YES      |
```

---

#### search views

**Purpose**: Find views whose names match a keyword, across all accessible databases (or one database when `--database` is specified). Views are virtual tables defined by a stored SELECT statement. This subcommand complements `search tables` for environments that separate reporting or transformation logic into views.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] search views [OPTIONS] <KEYWORD>
```

**Arguments**:
- `<KEYWORD>`: Required. A plain string. Automatically wrapped as `%keyword%` for SQL LIKE matching. Case-insensitive.

**Output Columns**:
| Column | Description |
|--------|-------------|
| Database | Database that contains the view |
| View | View name |
| Owner | Creator or owning user of the view |

**Sorting**: Results are sorted first by Database name (ascending), then by View name (ascending).

**Output — Table Format**:
```
Search results for views matching 'summary' (3 found):
Database             View                       Owner
------------------------------------------------------
analytics            daily_summary              dba_user
reporting            sales_summary              rpt_owner
reporting            weekly_summary             rpt_owner

3 view(s) found
```

**Output — Table Format (with --database)**:
```
Search results for views matching 'summary' in database 'reporting' (2 found):
Database             View                       Owner
------------------------------------------------------
reporting            sales_summary              rpt_owner
reporting            weekly_summary             rpt_owner

2 view(s) found
```

**Output — Table Format (no results)**:
```
No views found matching 'summary'.
```

**Output — Table Format (with pagination)**:
```
Search results for views matching 'report' (page 1 of 3):
Database             View                       Owner
------------------------------------------------------
analytics            report_daily               dba_user
analytics            report_monthly             dba_user
...

Page 1 of 3 (72 total rows)
```

**Output — JSON Format**:

JSON output uses the standard envelope: `{"ok": true, "row_count": N, "data": [...]}`.

```json
{
  "ok": true,
  "row_count": 3,
  "data": [
    { "database": "analytics", "view": "daily_summary",  "owner": "dba_user"   },
    { "database": "reporting", "view": "sales_summary",  "owner": "rpt_owner"  },
    { "database": "reporting", "view": "weekly_summary", "owner": "rpt_owner"  }
  ]
}
```

When no results are found:
```json
{
  "ok": true,
  "row_count": 0,
  "data": []
}
```

When pagination is active, a `pagination` object is appended to the envelope:
```json
{
  "ok": true,
  "row_count": 25,
  "data": [ ... ],
  "pagination": {
    "page": 1,
    "page_size": 25,
    "total_rows": 72,
    "total_pages": 3,
    "has_more": true
  }
}
```

**Output — CSV Format**:
```csv
Database,View,Owner
analytics,daily_summary,dba_user
reporting,sales_summary,rpt_owner
reporting,weekly_summary,rpt_owner
```

**Output — Markdown Format**:
```markdown
| Database  | View           | Owner      |
|-----------|----------------|------------|
| analytics | daily_summary  | dba_user   |
| reporting | sales_summary  | rpt_owner  |
| reporting | weekly_summary | rpt_owner  |
```

---

#### search procedures

**Purpose**: Find stored procedures whose names match a keyword, across all accessible databases (or one database when `--database` is specified). Stored procedures (`TableKind = 'P'` in `DBC.TablesV`) encapsulate reusable SQL logic. This subcommand complements `search tables` and `search views` for discovering procedural assets.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] search procedures [OPTIONS] <KEYWORD>
```

**Arguments**:
- `<KEYWORD>`: Required. A plain string. Automatically wrapped as `%keyword%` for SQL LIKE matching. Case-insensitive.

**Output Columns**:
| Column | Description |
|--------|-------------|
| Database | Database that contains the procedure |
| Name | Procedure name |
| Owner | Creator or owning user of the procedure |

**Sorting**: Results are sorted first by Database name (ascending), then by Procedure name (ascending).

**Output -- Table Format**:
```
Procedures matching 'update' (3):
Database             Name                           Owner
-------------------------------------------------------------------
hr                   update_salary                  alice
hr                   update_department              alice
payroll              update_benefits                bob

3 procedure(s)
```

**Output -- Table Format (no results)**:
```
Procedures matching 'xyz' (0):
Database             Name                           Owner
-------------------------------------------------------------------
(no procedures found)

0 procedure(s)
```

**Output -- JSON Format**:

JSON output uses the standard envelope: `{"ok": true, "row_count": N, "data": [...]}`.

```json
{
  "ok": true,
  "row_count": 3,
  "data": [
    { "database": "hr",      "procedure_name": "update_salary",     "owner": "alice" },
    { "database": "hr",      "procedure_name": "update_department", "owner": "alice" },
    { "database": "payroll", "procedure_name": "update_benefits",   "owner": "bob"   }
  ]
}
```

When no results are found:
```json
{
  "ok": true,
  "row_count": 0,
  "data": []
}
```

When pagination is active, a `pagination` object is appended to the envelope:
```json
{
  "ok": true,
  "row_count": 25,
  "data": [ ... ],
  "pagination": {
    "page": 1,
    "page_size": 25,
    "total_rows": 72,
    "total_pages": 3,
    "has_more": true
  }
}
```

**Output -- CSV Format**:
```csv
Database,ProcedureName,Owner
hr,update_salary,alice
hr,update_department,alice
payroll,update_benefits,bob
```

**Output -- Markdown Format**:
```markdown
| Database | Name              | Owner |
| :---     | :---              | :---  |
| hr       | update_salary     | alice |
| hr       | update_department | alice |
| payroll  | update_benefits   | bob   |
```

---

#### search -- Pagination

The `--page-size` and `--page` flags enable client-side pagination across all search subcommands (`tables`, `columns`, `views`, `procedures`). When `--page-size` is specified, the tool fetches the full result set and slices it into pages in memory.

**How pagination works**:
- `--page-size <N>` activates pagination and sets the number of rows per page.
- `--page <P>` selects which page to display (1-based, defaults to `1`).
- `--page-size` and `--limit` are mutually exclusive. Specifying both is a usage error.
- `--page` without `--page-size` is a usage error.

**Page footer** (all non-JSON formats):
```
Page 2 of 5 (47 total rows)
```

**Pagination in JSON format**: The standard envelope gains a `pagination` key with `page`, `page_size`, `total_rows`, `total_pages`, and `has_more` fields.

**Out-of-range page**: Requesting a page beyond the last page returns an empty `data` array. The pagination footer still shows the correct totals.

> **Warning — ORDER BY stability**: Pagination relies on a deterministic sort order. The search commands apply a fixed sort order (`DatabaseName ASC, TableName/ViewName/ColumnName ASC`) so pages are stable across invocations against an unchanged catalog. If the underlying catalog data changes between requests, rows may shift between pages.

---

#### search — Behavior Requirements

1. **REQ-SEARCH-001**: The `search` command SHALL require both a subcommand (`tables`, `columns`, `views`) and a keyword argument. Invoking `tq search` with no subcommand SHALL print subcommand help and exit with code 2. Invoking with an unknown subcommand SHALL print `Error: Unknown search subcommand: <name>` followed by available subcommands and exit with code 2.
2. **REQ-SEARCH-002**: The keyword argument is REQUIRED. Invoking a subcommand without a keyword SHALL print `Error: Missing required argument: <keyword>` with usage guidance and exit with code 2.
3. **REQ-SEARCH-003**: `search tables` data source: `DBC.TablesV WHERE TableKind IN ('T', 'O')` joined with `DBC.TableSizeV` for size and row estimates. The `TableName` column is filtered using `LIKE '%<keyword>%'` (case-insensitive). Both `DatabaseName` and `TableName` are returned.
4. **REQ-SEARCH-004**: `search columns` data source: `DBC.ColumnsV` joined with `DBC.TablesV WHERE TableKind IN ('T', 'O', 'V')`. The `ColumnName` column is filtered using `LIKE '%<keyword>%'` (case-insensitive). `DatabaseName`, `TableName`, `ColumnName`, `ColumnType`, and `Nullable` are returned.
5. **REQ-SEARCH-005**: When `--database <db>` is provided, the search is scoped to the specified database by adding `AND DatabaseName = '<db>'` to the query predicate. Without `--database`, all accessible databases are searched.
6. **REQ-SEARCH-006**: Results SHALL be sorted by `DatabaseName ASC, TableName ASC` for `search tables` and `search views`, and by `DatabaseName ASC, TableName ASC, ColumnName ASC` for `search columns`.
7. **REQ-SEARCH-007**: When a subcommand returns no results, an appropriate message SHALL be displayed (`No tables found matching '<keyword>'.`, `No columns found matching '<keyword>'.`, or `No views found matching '<keyword>'.`) and the command SHALL exit with code 0 (not an error).
8. **REQ-SEARCH-008**: In JSON format, all search subcommands use the standard envelope: `{"ok": true, "row_count": N, "data": [...]}`. An empty result returns `{"ok": true, "row_count": 0, "data": []}`. When pagination is active, a `pagination` object is included.
9. **REQ-SEARCH-009**: Size and row-count values follow the same human-readable string formatting as `tq list tables` (e.g., `2.1 MB`, `42,573`). In JSON and CSV, `rows_est` and `size` are strings matching the display format.
10. **REQ-SEARCH-010**: The `search` command is safe for use with `--agent-safe` mode. All subcommands execute read-only queries against system catalog views and perform no data modification.
11. **REQ-SEARCH-011**: The `--format` and `--output` flags follow the same semantics as all other tq commands (see [Flag Design Guidelines](#flag-design-guidelines)). The `md` (Markdown) format is supported in addition to `table`, `json`, and `csv`.
12. **REQ-SEARCH-012**: Object types returned by `search tables` SHALL use the same display labels as `tq list tables`: `TABLE` for `TableKind = 'T'`, `TABLE (NoPI)` for `TableKind = 'O'`.
13. **REQ-SEARCH-013**: `search views` data source: `DBC.TablesV WHERE TableKind = 'V'`. The `TableName` column is filtered using `LIKE '%<keyword>%'` (case-insensitive). `DatabaseName`, `TableName`, and `CreatorName` (displayed as Owner) are returned.
14. **REQ-SEARCH-014**: Pagination is enabled by `--page-size <N>`. When active, the full result set is fetched and sliced client-side. `--page-size` and `--limit` are mutually exclusive; specifying both SHALL produce a usage error (exit code 2). `--page` requires `--page-size`; specifying `--page` alone SHALL produce a usage error (exit code 2).
15. **REQ-SEARCH-015**: When `--page-size` is active, a footer line `Page P of T (N total rows)` SHALL be appended after results in all non-JSON formats. Requesting a page beyond the last page returns an empty result with the footer showing the correct totals and `has_more: false` in JSON.

**Error Handling**:

**Unknown Subcommand**:
```
Error: Unknown subcommand 'macros'
Usage: tq search <tables|columns|views> [OPTIONS] <KEYWORD>

Available subcommands:
  tables    Search for tables by name keyword, across all databases
  columns   Search for columns by name keyword, across all databases
  views     Search for views by name keyword, across all databases

Exit code: 2
```

**Missing Keyword**:
```
Error: Missing required argument: <keyword>
Usage: tq search tables [OPTIONS] <KEYWORD>

Example:
  tq search tables emp
  tq search tables emp --database production

Exit code: 2
```

**Missing Subcommand**:
```
Usage: tq search <tables|columns|views> [OPTIONS] <KEYWORD>

Available subcommands:
  tables    Search for tables by name keyword, across all databases
  columns   Search for columns by name keyword, across all databases
  views     Search for views by name keyword, across all databases

Exit code: 2
```

**Insufficient Privileges**:
```
Error: Unable to search tables.
Reason: SELECT permission denied on DBC.TablesV.

Contact your DBA to request access:
  GRANT SELECT ON DBC.TablesV TO <your_username>;
  GRANT SELECT ON DBC.TableSizeV TO <your_username>;

Exit code: 1
```

**Exit Codes**:
- `0`: Search completed successfully (including zero results)
- `1`: Query error (privilege denied, connection failed, system view unavailable)
- `2`: Usage error (missing subcommand, missing keyword, unknown subcommand, invalid flag value)

**Integration with Scripting**:
```bash
# Find all tables containing "customer" across all databases, output as JSON
tq search tables customer --format json

# Scope to production and pipe to jq
tq search tables order --database production --format json | jq '.data[].table'

# Find all columns named like "id" and export to CSV for auditing
tq search columns _id --format csv --output id-columns.csv

# Count how many databases expose a "salary" column
tq search columns salary --format json | jq '[.data[].database] | unique | length'

# Find all views containing "summary" and list by database
tq search views summary --format json | jq '.data[] | "\(.database).\(.view)"'

# Paginate view search results for large catalogs
tq search views report --page-size 25 --page 1 --format json

# Use with a profile for production scanning
tq --profile prod search tables temp --format json | jq '.data | length'

# Find all tables that might be staging/temp tables across all databases
tq search tables temp_ --format json | jq '.data[] | "\(.database).\(.table)"'
```

**Acceptance Tests**:
- Execute `tq search tables emp` and verify table output with Database, Table, Type, Rows (Est.), Size columns; results from multiple databases shown
- Execute `tq search tables emp --database hr` and verify only tables in the `hr` database appear
- Execute `tq search tables emp --format json` and verify JSON envelope `{"ok": true, "row_count": N, "data": [...]}` with keys `database`, `table`, `type`, `rows_est`, `size`
- Execute `tq search tables emp --format csv` and verify CSV with `Database,Table,Type,RowsEst,Size` header
- Execute `tq search tables emp --format md` and verify Markdown table output
- Execute `tq search tables xyz_nonexistent_xyz` and verify `No tables found matching 'xyz_nonexistent_xyz'.` with exit code 0
- Execute `tq search tables emp --format json` with no matching results and verify `{"ok": true, "row_count": 0, "data": []}`
- Execute `tq search columns salary` and verify table output with Database, Table, Column, Type, Nullable columns
- Execute `tq search columns salary --database hr` and verify only columns in the `hr` database appear
- Execute `tq search columns salary --format json` and verify JSON envelope with keys `database`, `table`, `column`, `type`, `nullable`
- Execute `tq search columns salary --format csv` and verify CSV with `Database,Table,Column,Type,Nullable` header
- Execute `tq search columns xyz_nonexistent_xyz` and verify `No columns found matching 'xyz_nonexistent_xyz'.` with exit code 0
- Execute `tq search views summary` and verify table output with Database, View, Owner columns; results from multiple databases shown
- Execute `tq search views summary --database reporting` and verify only views in the `reporting` database appear
- Execute `tq search views summary --format json` and verify JSON envelope `{"ok": true, "row_count": N, "data": [...]}` with keys `database`, `view`, `owner`
- Execute `tq search views summary --format csv` and verify CSV with `Database,View,Owner` header
- Execute `tq search views summary --format md` and verify Markdown table output
- Execute `tq search views xyz_nonexistent_xyz` and verify `No views found matching 'xyz_nonexistent_xyz'.` with exit code 0
- Execute `tq search views summary --page-size 5 --page 1` and verify footer `Page 1 of N (T total rows)` appended
- Execute `tq search views summary --page-size 5 --format json` and verify `pagination` key in JSON envelope
- Execute `tq search views summary --page-size 5 --limit 100` and verify usage error (exit code 2) — flags are mutually exclusive
- Execute `tq search` with no subcommand and verify usage help with exit code 2
- Execute `tq search tables` with no keyword and verify `Error: Missing required argument: <keyword>` with exit code 2
- Execute `tq search unknown_sub emp` and verify `Error: Unknown subcommand 'unknown_sub'` with exit code 2
- Verify results are sorted: database ascending, then table/view ascending
- Execute `tq search tables emp --output results.txt` and verify file is created

---

### show-indexes - Show Table Index Structure

**Purpose**: Display the complete index structure for a table, including primary index type and columns, and all secondary indexes. This is the batch-mode equivalent of the REPL `/show indexes` command.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] show-indexes [OPTIONS] <OBJECT>
```

**Arguments**:
- `<OBJECT>`: Required. A table name (`employees`) or a fully qualified name (`database.employees`). Object names are case-insensitive (Teradata convention).

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | `-f` | enum | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | path | stdout | Write output to file |

**Examples**:
```bash
# Show indexes for a table in the current database
tq show-indexes employees

# Show indexes using a qualified name
tq show-indexes production.orders

# JSON output for scripting
tq show-indexes --format json employees

# CSV output for documentation
tq show-indexes --format csv production.orders > orders-indexes.csv

# Using a connection profile
tq --profile prod show-indexes employees

# Write to file
tq show-indexes --output indexes.txt employees
```

**Output — Table Format (table with primary and secondary indexes)**:
```
Indexes on PRODUCTION.employees:

── Primary Index ──
  Primary Index (UPI): employee_id

── Secondary Indexes ──
  Secondary Index (NUSI): department_id
  Secondary Index (USI): email

3 index(es), 3 index column(s)
```

**Output — Table Format (NoPI table)**:
```
Indexes on PRODUCTION.fact_sales:

── Primary Index ──
  No Primary Index (NoPI)

── Secondary Indexes ──
  Secondary Index (NUSI): region_id, sale_date

2 index(es), 3 index column(s)
```

**Output — Table Format (primary index only, no secondary indexes)**:
```
Indexes on PRODUCTION.config:

── Primary Index ──
  Primary Index (NUPI): config_key

No secondary indexes.

1 index(es), 1 index column(s)
```

**Output — Table Format (named index)**:

When an index has a name (other than `(unnamed)`), the name is shown in quotes:

```
── Secondary Indexes ──
  Secondary Index (NUSI) "idx_dept": department_id
```

**Output — CSV Format**:
```csv
IndexName,IndexType,ShortType,IsPrimary,Columns
(unnamed),Primary Index,UPI,Yes,employee_id
(unnamed),Secondary Index,NUSI,No,department_id
(unnamed),Secondary Index,USI,No,email
```

**Output — JSON Format**:
```json
{
  "object": "PRODUCTION.employees",
  "primary_index": {
    "type": "UPI",
    "columns": ["employee_id"]
  },
  "secondary_indexes": [
    { "name": "(unnamed)", "type": "NUSI", "columns": ["department_id"] },
    { "name": "(unnamed)", "type": "USI",  "columns": ["email"] }
  ]
}
```

**Output — JSON Format (NoPI table)**:
```json
{
  "object": "PRODUCTION.fact_sales",
  "primary_index": null,
  "secondary_indexes": [
    { "name": "(unnamed)", "type": "NUSI", "columns": ["region_id", "sale_date"] }
  ]
}
```

**Behavior Requirements**:

1. **REQ-SHOW-IDX-001**: The command SHALL require exactly one object argument; invoking without an argument SHALL exit with code 2 and print the usage error.
2. **REQ-SHOW-IDX-002**: Object names SHALL be treated case-insensitively (Teradata convention).
3. **REQ-SHOW-IDX-003**: Data source: `DBC.IndicesV`. The command fetches all index records for the specified object.
4. **REQ-SHOW-IDX-004**: Index type labels in table format display as `<type_label> (<short_label>)`:
   | Index category | type_label | short_label | Full display |
   |----------------|------------|-------------|--------------|
   | Unique Primary Index | `Primary Index` | `UPI` | `Primary Index (UPI)` |
   | Non-Unique Primary Index | `Primary Index` | `NUPI` | `Primary Index (NUPI)` |
   | Unique Secondary Index | `Secondary Index` | `USI` | `Secondary Index (USI)` |
   | Non-Unique Secondary Index | `Secondary Index` | `NUSI` | `Secondary Index (NUSI)` |
   | Partitioned Primary Index | `Partitioned Primary Index` | `PPI` | `Partitioned Primary Index (PPI)` |
   | No Primary Index | N/A | N/A | `No Primary Index (NoPI)` |
5. **REQ-SHOW-IDX-005**: The `short_label` (e.g., `UPI`, `NUSI`) SHALL be used as the `"type"` field in JSON output and as `ShortType` in CSV output.
6. **REQ-SHOW-IDX-006**: When no secondary indexes exist, the table format SHALL display `No secondary indexes.` on its own line (without a `── Secondary Indexes ──` header); the `secondary_indexes` key SHALL be an empty array `[]` in JSON format; no secondary index rows SHALL appear in CSV format.
7. **REQ-SHOW-IDX-007**: For NoPI tables (where no primary index rows exist in `DBC.IndicesV`), the `── Primary Index ──` section SHALL display `  No Primary Index (NoPI)`; in JSON format `"primary_index"` SHALL be `null`.
8. **REQ-SHOW-IDX-008**: When multiple columns form a composite index, all columns SHALL be listed in index column order, separated by `, ` in table and CSV formats, and as a JSON array in JSON format.
9. **REQ-SHOW-IDX-009**: Secondary indexes SHALL be displayed in index number order (ascending `IndexNumber` from `DBC.IndicesV`).
10. **REQ-SHOW-IDX-010**: The `--format` and `--output` flags follow the same semantics as all other tq commands (see [Flag Design Guidelines](#flag-design-guidelines)).
11. **REQ-SHOW-IDX-011**: This command applies only to tables. Invoking `tq show-indexes` against a view SHALL display an informative message (views have no indexes) and exit with code 0.
12. **REQ-SHOW-IDX-012**: Table format output uses `──` section headers (`── Primary Index ──`, `── Secondary Indexes ──`) with two-space indented content below each header. Each index is shown on a single inline line: `  <Full Label> (<Short>): col1, col2`. Named indexes include the name in quotes before the colon: `  <Full Label> (<Short>) "<name>": col1, col2`.
13. **REQ-SHOW-IDX-013**: A summary line at the end of table format output shows the total number of indexes and index columns: `N index(es), M index column(s)`.

**View Target**:
```
No indexes: 'PRODUCTION.active_employees' is a View.
Views do not have indexes. Use 'tq describe' to see the view's column structure.
```

**Error Handling**:

**Object Not Found**:
```
Error: Object 'PRODUCTION.employeees' not found.

Did you mean: employees

Exit code: 1
```

**Missing Argument**:
```
Error: Missing required argument <object>
Usage: tq show-indexes <object>

Examples:
  tq show-indexes employees
  tq show-indexes production.orders

Exit code: 2
```

**Insufficient Privileges**:
```
Error: Cannot retrieve index information for 'employees'.
Reason: SELECT permission denied on DBC.IndicesV.

Contact your DBA to request access:
  GRANT SELECT ON DBC.IndicesV TO <your_username>;

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
- `0`: Index information displayed successfully (including the case of a view target — informative message)
- `1`: Object not found, permission error, or connection failure
- `2`: Usage error (missing argument, invalid flag value)

**Integration with Scripting**:
```bash
# Check if a table has a UPI (ideal for distribution)
tq show-indexes --format json employees | \
  jq -r '.primary_index.type'

# Find all USI columns on a table
tq show-indexes --format json employees | \
  jq -r '.secondary_indexes[] | select(.type == "USI") | .columns[]'

# Export index definitions for documentation
tq --profile prod show-indexes --format csv --output employees-indexes.csv employees

# Identify NUSI indexes (candidates for review in large tables)
tq show-indexes --format json production.orders | \
  jq '[.secondary_indexes[] | select(.type == "NUSI")]'
```

**Acceptance Tests**:
- Execute `tq show-indexes <table>` with a UPI and secondary indexes and verify `──` section headers and inline index lines (no box-drawing, no multi-line type/columns)
- Execute `tq show-indexes <nopi_table>` and verify `── Primary Index ──` section shows `  No Primary Index (NoPI)`
- Execute `tq show-indexes <table_without_secondary_indexes>` and verify `No secondary indexes.` line appears (without a secondary section header)
- Execute `tq show-indexes <view>` and verify informative message is shown with exit code 0
- Execute `tq show-indexes <database>.<table>` (qualified name) and verify correct database resolution
- Execute `tq show-indexes --format json <table>` and verify valid JSON with `"object"` string, `"primary_index"` (object or null), `"secondary_indexes"` array; each secondary index has `"name"`, `"type"`, `"columns"` keys
- Execute `tq show-indexes --format json <nopi_table>` and verify `"primary_index"` is `null`
- Execute `tq show-indexes --format csv <table>` and verify CSV with `IndexName,IndexType,ShortType,IsPrimary,Columns` header
- Execute `tq show-indexes` with no argument and verify usage error (exit code 2)
- Execute `tq show-indexes <nonexistent>` and verify not-found error (exit code 1)
- Verify composite index columns are listed in correct order in all output formats
- Execute `tq show-indexes --output indexes.txt <table>` and verify file is created with correct content

---

### fastload - Bulk Load Data (FastLoad)

**Purpose**: Bulk load large datasets into an empty permanent Teradata table in parallel using the FastLoad protocol.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] fastload [OPTIONS] <SOURCE_FILE> <TARGET_TABLE>
```

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--source-format` | - | enum | auto-detect | Source format: `csv`, `parquet`, `json` |
| `--delimiter` | - | string | auto-detect | Field separator for CSV/TSV source files (comma, or tab for `.tsv`) |
| `--no-create` | - | flag | false | Disable automatic table creation if it does not exist |
| `--sessions` | - | int | database choice | Number of parallel data transfer connections |
| `--error-table-db` | - | string | target db | Database where the FastLoad error tables should reside |
| `--error-table-1-suffix` | - | string | `_ERR_1` | Suffix for FastLoad Error Table 1 |
| `--error-table-2-suffix` | - | string | `_ERR_2` | Suffix for FastLoad Error Table 2 |

**Behavior Requirements**:
1. **Parallel Execution**: Opens multiple parallel data transfer connections matching the sessions parameter (capped at the number of AMPs).
2. **Lazy-Load Creation**: If the target table does not exist and `--no-create` is not set, checks the schema of the source file (CSV headers, Parquet metadata, or first JSON object properties) and automatically issues a `CREATE TABLE` statement with permissive `VARCHAR(1000)` columns (or type-mapped columns for Parquet) before loading.
3. **Transaction Control**: Disables auto-commit before starting FastLoad. Performs all batch insertions under a transaction. If successful, issues `COMMIT`; if any data error or driver failure occurs, issues `ROLLBACK` and cleans up error tables.
4. **Format Auto-Detection**: Infers the source format from the file extension (`.csv`, `.parquet`, `.json`, `.ndjson`). Override using `--source-format`.
5. **Streaming/Conversion**:
   - CSV: Streams directly to the driver using native `{fn teradata_read_csv}`.
   - Parquet/JSON: Streamingly converts records to a temporary CSV file on disk, then loads the CSV natively. Memory usage must remain constant regardless of file size.
6. **Error Reporting**: Inspects and prints FastLoad warnings and errors (from Error Table 1 and 2) using `{fn teradata_get_warnings}` and `{fn teradata_get_errors}`.
7. **Delimiter Handling**: The `--delimiter` option is only valid for CSV/TSV format. Specifying `--delimiter` with Parquet or JSON formats will result in a validation error. Temporary CSV files generated during Parquet/JSON streaming conversion always use the default comma delimiter.

---

### fastexport - Bulk Export Data (FastExport)

**Purpose**: Bulk export large datasets from a Teradata table or view to a CSV file in parallel using the FastExport protocol.

**Usage**:
```bash
tq [GLOBAL_OPTIONS] fastexport [OPTIONS] <SOURCE_TABLE> <TARGET_FILE>
```

**Options**:
| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--sessions` | - | int | database choice | Number of parallel data transfer connections |

**Behavior Requirements**:
1. **Parallel Extraction**: Opens multiple parallel connections to download the table in chunks.
2. **Native CSV Writing**: Uses driver-native `{fn teradata_write_csv}` and `{fn teradata_require_fastexport}` to stream the query results directly to the target CSV file, bypassing local memory caching.
3. **Progress Reporting**: Prints the number of rows exported (from `activity_count` in metadata) and the elapsed time on completion.

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
| Query from arg (stdin TTY or empty) | Ignored | Results | Errors, warnings |
| Query from arg + piped stdin with bytes | Conflict error | — | Conflict error |
| Query from stdin | SQL query | Results | Errors, warnings |
| REPL mode | User input | Results | Errors, warnings |
| Piped output | Varies | Machine format | Human messages |

### stdin Detection for `tq query`

The following requirements govern how `tq query` decides whether stdin constitutes a second input source, in order to correctly distinguish an intentional SQL pipe from an accidental or empty redirection.

**REQ-CLI-STDIN-001: Definition of "stdin as input source"**

stdin is treated as a SQL input source if and only if ALL of the following conditions are true at startup:

1. stdin is not a TTY (`isatty(0)` returns false).
2. stdin has at least one byte available to read (a non-blocking peek or equivalent OS-level check confirms data is present).

A redirected-but-empty stdin (e.g., `< /dev/null`, `< empty_file`, or a pipe whose write end was closed without writing) satisfies condition (1) but NOT condition (2) and MUST NOT be treated as a SQL input source.

**REQ-CLI-STDIN-002: Conflict detection rule**

A "multiple input sources" conflict exists when and only when a positional `<QUERY>` argument is present AND stdin satisfies both conditions in REQ-CLI-STDIN-001. Any combination where only one real input source is present MUST NOT produce a conflict error.

Scenario truth table:

| Positional `<QUERY>` arg | stdin is TTY | stdin has bytes | Result |
|--------------------------|--------------|-----------------|--------|
| Yes | Yes | N/A | Run arg query; stdin ignored (interactive terminal) |
| Yes | No | No | Run arg query; stdin ignored (empty redirect) |
| Yes | No | Yes | Error: multiple input sources |
| No | Yes | N/A | Error: no query provided |
| No | No | No | Error: no query provided (empty stdin) |
| No | No | Yes | Run stdin query |

**REQ-CLI-STDIN-003: Error message for legitimate conflict**

When a genuine conflict is detected (positional arg AND stdin with bytes available), the error message is:

```
Error: Multiple input sources provided

You specified both a query argument and piped stdin.
Only one input source is allowed.

Either provide SQL as argument OR pipe via stdin, not both.
```

Exit code: `2`

**REQ-CLI-STDIN-004: No error for empty redirected stdin**

The invocations below MUST all succeed without a conflict error when a positional query argument is present:

```bash
# Empty device
tq query "SELECT 1" < /dev/null

# Empty here-string
tq query "SELECT 1" <<< ""

# Empty file
tq query "SELECT 1" < /tmp/empty_file

# Agent or CI harness that closes stdin before invoking tq
```

These cases are indistinguishable from each other at the OS level: stdin is not a TTY and has zero bytes available. The tool MUST NOT produce a conflict error for any of them.

**REQ-CLI-STDIN-005: Pipe with bytes still triggers conflict**

When a positional query argument is present AND a pipe with actual content is attached to stdin, the tool MUST reject with the conflict error defined in REQ-CLI-STDIN-003. The presence of bytes distinguishes an intentional pipe from an accidental redirection.

```bash
# This MUST produce the conflict error:
echo "SELECT 2" | tq query "SELECT 1"
```

**REQ-CLI-STDIN-006: Interactive terminal not affected**

When stdin is a TTY (interactive terminal), the stdin availability check does not apply. A TTY stdin is never treated as a SQL input source regardless of whether the positional argument is present.

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

## Global Error Message Standards

All error messages across every `tq` command and metacommand SHALL conform to the following rules. These rules apply to both batch mode (CLI commands) and REPL mode metacommands.

### REQ-ERR-001: Error Prefix

Every user-visible error message SHALL begin with `Error:` (capital E, followed by a colon and a space). No command SHALL emit an error without this prefix. This applies to:
- Object not found errors
- Permission denied errors
- Usage errors (missing arguments, invalid flags)
- Unknown subcommand errors
- Connection failures
- Any other failure condition that produces a non-zero exit code or an inline error display in REPL mode

**Correct:**
```
Error: Object 'employees' not found.
Error: Missing required argument <object>
Error: Cannot describe 'employees'.
Error: Unknown list subcommand: schema
```

**Incorrect:**
```
employees not found.
Missing argument
Cannot describe employees.
Unknown list subcommand: schema
```

### REQ-ERR-002: Object Placeholder in Usage Text

All command help text and usage-error messages SHALL use `<OBJECT>` (not `<TABLE>`) when referring to a generic database object argument, because these commands operate on tables, views, macros, and other object types equally.

**Correct:**
```
Usage: tq describe <object>
Usage: tq show-indexes <object>
Usage: tq inspect <object>
```

**Incorrect:**
```
Usage: tq describe <table>
Usage: tq show-indexes <table>
```

**Rationale:** Using `<TABLE>` implies the command only works on tables, which misleads users trying to describe or inspect views, macros, or other Teradata object types.

### REQ-ERR-003: Error Message Structure

Non-trivial errors (not simple usage errors) SHALL follow a three-part structure:
1. **What went wrong** — `Error: <concise description>`
2. **Why** — `Reason: <technical detail>` (when the cause is known and actionable)
3. **How to fix** — A specific, actionable suggestion or contact instruction

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
