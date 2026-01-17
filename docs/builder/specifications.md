# tq (Teradata Query) - Comprehensive Specifications

**Version:** 1.0.0
**Status:** Draft
**Last Updated:** 2026-01-16

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [User Personas and Use Cases](#2-user-personas-and-use-cases)
3. [Functional Requirements](#3-functional-requirements)
4. [Command-Line Interface Design](#4-command-line-interface-design)
5. [REPL Mode Specifications](#5-repl-mode-specifications)
6. [Batch Mode Specifications](#6-batch-mode-specifications)
7. [Configuration and Credential Management](#7-configuration-and-credential-management)
8. [Output Format Specifications](#8-output-format-specifications)
9. [Error Handling and User Feedback](#9-error-handling-and-user-feedback)
10. [Security Requirements](#10-security-requirements)
11. [Performance Considerations](#11-performance-considerations)
12. [Future Enhancements](#12-future-enhancements)

---

## 1. Project Overview

### 1.1 Vision

`tq` is a best-in-class, lightweight command-line client for Teradata databases, designed to be fast, intuitive, and composable. It follows UNIX philosophy while providing a rich interactive experience comparable to `psql` for PostgreSQL.

### 1.2 Goals

- **Simplicity**: Zero-configuration for basic use cases
- **Composability**: Works seamlessly in scripts and pipelines
- **Performance**: Fast startup, efficient query execution, minimal memory footprint
- **Security**: Secure credential handling, no password leaks
- **Cross-platform**: Works identically on Linux, macOS, and Windows
- **Self-contained**: Single static binary with no runtime dependencies

### 1.3 Non-Goals

- **GUI Interface**: Strictly command-line
- **Data Transformation**: Use external tools like `jq`, `csvkit`
- **Schema Migration**: Use dedicated tools like Liquibase
- **Connection Pooling**: One-shot execution model for batch mode

### 1.4 Design Principles

1. **Convention over Configuration**: Sensible defaults for 80% of use cases
2. **Progressive Disclosure**: Simple things easy, complex things possible
3. **Fail Fast**: Clear error messages with actionable suggestions
4. **Respect UNIX Conventions**: `-h/--help`, `-V/--version`, stdin/stdout, exit codes
5. **Terminal Context Awareness**: Human output for TTY, machine output for pipes

---

## 2. User Personas and Use Cases

### 2.1 User Personas

#### Persona 1: Database Administrator (DBA)
**Profile**: Sarah, Senior DBA
**Needs**: Quick health checks, connection testing, schema inspection
**Pain Points**: GUI tools are slow to start, need lightweight diagnostics
**Usage Pattern**: 50+ quick commands per day, values speed and reliability

#### Persona 2: Data Analyst
**Profile**: Mike, Business Intelligence Analyst
**Needs**: Ad-hoc queries, data exploration, CSV exports
**Pain Points**: Current tools don't integrate with shell workflows
**Usage Pattern**: Interactive sessions, frequent exports to Excel/CSV

#### Persona 3: DevOps Engineer
**Profile**: Alex, Platform Engineer
**Needs**: Automated health checks, scripted data extraction, monitoring
**Pain Points**: Hard to integrate database checks in CI/CD
**Usage Pattern**: Scripted usage, JSON output, cron jobs

#### Persona 4: Data Engineer
**Profile**: Jamie, ETL Developer
**Needs**: Large result sets, streaming data, performance optimization
**Pain Points**: Memory issues with large datasets, slow tools
**Usage Pattern**: Batch processing, pipeline integration, performance-critical

### 2.2 Primary Use Cases

#### UC-1: Quick Connection Test
**Actor**: DBA
**Goal**: Verify database connectivity
**Flow**:
```bash
export TQ_LOGON="user:pass@host:1025/db"
tq ping
# Output: Database connection successful (127ms)
```

#### UC-2: One-Shot Query
**Actor**: Data Analyst
**Goal**: Execute single query and view results
**Flow**:
```bash
tq query "SELECT * FROM employees WHERE dept = 'IT'" --format table
```

#### UC-3: Export to CSV
**Actor**: Data Analyst
**Goal**: Export query results for analysis
**Flow**:
```bash
tq query "SELECT * FROM sales_2024" --format csv > sales.csv
```

#### UC-4: Scripted Health Check
**Actor**: DevOps Engineer
**Goal**: Automated database monitoring
**Flow**:
```bash
#!/bin/bash
if tq ping --timeout 5s; then
  echo "Database healthy"
else
  alert_ops "Database down"
fi
```

#### UC-5: Interactive Exploration
**Actor**: Data Analyst
**Goal**: Explore database schema and query data
**Flow**:
```bash
tq repl
> \l                    -- list databases
> \dt public.*          -- list tables
> \d employees          -- describe table
> SELECT * FROM employees LIMIT 10;
> \export csv employees.csv
```

#### UC-6: Pipeline Integration
**Actor**: Data Engineer
**Goal**: Extract data for processing
**Flow**:
```bash
tq query "SELECT user_id, activity FROM events" --format json | \
  jq '.[] | select(.activity == "login")' | \
  transform_script.py | \
  load_to_warehouse.sh
```

---

## 3. Functional Requirements

### 3.1 Core Features (MVP - Current)

| ID | Feature | Priority | Status |
|----|---------|----------|--------|
| FR-001 | Execute single SQL query | P0 | ✅ Implemented |
| FR-002 | Ping database connectivity | P0 | ✅ Implemented |
| FR-003 | Multiple output formats (table/JSON/CSV) | P0 | ✅ Implemented |
| FR-004 | TD2 authentication | P0 | ✅ Implemented |
| FR-005 | LDAP authentication | P0 | ✅ Implemented |
| FR-006 | Kerberos authentication | P0 | ✅ Implemented |
| FR-007 | Connection string parsing | P0 | ✅ Implemented |
| FR-008 | TQ_LOGON environment variable | P0 | ✅ Implemented |
| FR-009 | Password file support | P0 | ✅ Implemented |
| FR-010 | Secure credential handling | P0 | ✅ Implemented |

### 3.2 REPL Mode Features

**Phase 1 - MVP (In Progress)**
See detailed specification: `docs/builder/detailed-specifications/interactive-mode-mvp.md`

| ID | Feature | Priority | Status |
|----|---------|----------|--------|
| FR-101 | Interactive prompt | P0 | 🚧 In Progress |
| FR-102 | Multi-line SQL input | P0 | 🚧 In Progress |
| FR-103 | Command history (in-memory) | P0 | 🚧 In Progress |
| FR-116 | `/session` metacommand | P0 | 🚧 In Progress |
| FR-120 | `/quit` metacommand | P0 | 🚧 In Progress |
| FR-121 | `/help` metacommand | P0 | 🚧 In Progress |

**Phase 2 - Enhanced REPL (Planned)**

| ID | Feature | Priority | Status |
|----|---------|----------|--------|
| FR-104 | History persistence | P1 | 📋 To Do |
| FR-105 | SQL syntax highlighting | P1 | 📋 To Do |
| FR-109 | Vim keybindings | P1 | 📋 To Do |
| FR-110 | Emacs keybindings | P1 | 📋 To Do |
| FR-111 | Result paging (left-right) | P1 | 📋 To Do |
| FR-112 | Result paging (up-down) | P1 | 📋 To Do |
| FR-114 | Query timing display | P1 | 📋 To Do |
| FR-115 | `/describe` metacommand | P0 | 📋 To Do |
| FR-118 | `/ping` metacommand | P0 | 📋 To Do |

**Phase 3 - Advanced REPL (Future)**

| ID | Feature | Priority | Status |
|----|---------|----------|--------|
| FR-106 | Tab completion - keywords | P1 | 📋 To Do |
| FR-107 | Tab completion - table names | P1 | 📋 To Do |
| FR-108 | Tab completion - column names | P2 | 📋 To Do |
| FR-113 | Export last result | P1 | 📋 To Do |
| FR-117 | `/logon` metacommand | P1 | 📋 To Do |
| FR-119 | `/export` metacommand | P1 | 📋 To Do |

### 3.3 Batch Mode Features

| ID | Feature | Priority | Status |
|----|---------|----------|--------|
| FR-201 | Execute from file | P0 | 🔲 Planned |
| FR-202 | Read SQL from stdin | P0 | 🔲 Planned |
| FR-203 | Output to stdout | P0 | ✅ Implemented |
| FR-204 | Output to file | P1 | 🔲 Planned |
| FR-205 | Streaming large results | P1 | 🔲 Planned |
| FR-206 | Multiple statement execution | P1 | 🔲 Planned |
| FR-207 | Transaction control | P2 | 🔲 Planned |
| FR-208 | Variable substitution | P2 | 🔲 Planned |

### 3.4 Configuration Features

| ID | Feature | Priority | Status |
|----|---------|----------|--------|
| FR-301 | User config file | P1 | 🔲 Planned |
| FR-302 | Project config file | P1 | 🔲 Planned |
| FR-303 | Connection profiles | P1 | 🔲 Planned |
| FR-304 | Default format preference | P2 | 🔲 Planned |
| FR-305 | Keyring integration | P2 | 🔲 Planned |

---

## 4. Command-Line Interface Design

### 4.1 Design Philosophy

The CLI follows these principles:
- **Subcommands for major operations**: `ping`, `query`, `repl`
- **Global options before subcommand**: `-l/--logon`, `--logmech`, `--password-file`
- **Subcommand-specific options after subcommand**: `--format`, `--output`
- **POSIX-compliant**: Short (`-f`) and long (`--format`) flags
- **Environment variable fallbacks**: `TQ_LOGON`, `TQ_LOGMECH`, `TQ_FORMAT`

### 4.2 Command Structure

```
tq [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

### 4.3 Global Options

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
| `--help` | `-h` | flag | - | Show help |
| `--version` | `-V` | flag | - | Show version |

### 4.4 Commands

#### 4.4.1 `ping` - Test Connectivity

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

#### 4.4.2 `query` - Execute SQL

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

#### 4.4.3 `repl` - Interactive Mode (Future)

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

### 4.5 Input/Output Behavior

#### 4.5.1 Standard Streams

| Context | stdin | stdout | stderr |
|---------|-------|--------|--------|
| Query from arg | Ignored | Results | Errors, warnings |
| Query from stdin | SQL query | Results | Errors, warnings |
| REPL mode | User input | Results | Errors, warnings |
| Piped output | Varies | Machine format | Human messages |

#### 4.5.2 Terminal Detection

The tool adjusts behavior based on context:

| Feature | Interactive (TTY) | Piped/Redirected |
|---------|-------------------|------------------|
| Color output | Enabled | Disabled |
| Progress indicators | Shown | Hidden |
| Default format | `table` | `csv` or `json` |
| Pager | Enabled (large results) | Disabled |
| Confirmation prompts | Shown | Auto-yes or error |

#### 4.5.3 Exit Code Standards

| Code | Meaning | Examples |
|------|---------|----------|
| `0` | Success | Query executed, connection successful |
| `1` | Runtime error | Connection failed, query error, file not found |
| `2` | Usage error | Invalid flag, missing required argument |
| `130` | Interrupted | User pressed Ctrl-C |

---

### 4.6 Flag Design Guidelines

#### 4.6.1 Short vs Long Flags

**Short flags** (`-f`):
- Single letter
- For frequently used options
- Can be combined: `-vvv` (very verbose), `-qf json` (quiet JSON)

**Long flags** (`--format`):
- Descriptive kebab-case
- Always available
- Self-documenting

#### 4.6.2 Boolean Flags

```bash
# Flag present = true
tq query --timing "SELECT 1"

# Explicit negation with --no-prefix
tq repl --no-history

# Avoid --flag=true syntax (non-standard)
```

#### 4.6.3 Value Flags

```bash
# Space-separated (preferred)
tq query --format json "SELECT 1"

# Equals sign (also supported)
tq query --format=json "SELECT 1"

# Short flag with value
tq query -f json "SELECT 1"
```

---

### 4.7 Help Text Design

#### 4.7.1 Top-Level Help

```bash
$ tq --help
```

Output:
```
tq - Teradata Query
A fast, lightweight command-line client for Teradata databases

Usage: tq [OPTIONS] <COMMAND>

Commands:
  ping   Test database connectivity
  query  Execute a SQL query
  repl   Start interactive mode [future]
  help   Print this message or help for a subcommand

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

  # Secure password handling
  echo "password" > ~/.tq_pass && chmod 0600 ~/.tq_pass
  tq -l "user@host:1025/db" --password-file ~/.tq_pass query "SELECT 1"

Configuration:
  Set TQ_LOGON environment variable to avoid repeating connection string:
    export TQ_LOGON="user:pass@host:1025/db"
    tq ping

  Or create ~/.config/tq/config.toml:
    host = "myhost"
    port = 1025
    user = "myuser"
    database = "mydb"

For more information, visit: https://github.com/yourusername/tq
```

#### 4.7.2 Subcommand Help

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
```

---

### 4.8 Version Information

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

## 5. REPL Mode Specifications

### 5.1 Overview

REPL (Read-Eval-Print Loop) mode provides an interactive database session similar to `psql`, `mysql`, or `usql`. This mode is optimized for exploratory data analysis, schema inspection, and ad-hoc querying.

### 5.2 Starting REPL Mode

```bash
# With pre-configured connection
export TQ_LOGON="user:pass@host:1025/db"
tq repl

# With explicit connection
tq -l "user:pass@host:1025/db" repl

# With configuration file
tq repl  # Uses ~/.config/tq/config.toml
```

### 5.3 User Interface

#### 5.3.1 Prompt Design

```
tq> SELECT * FROM employees
```

**Prompt Variations**:
- `tq>` - Default prompt (connected)
- `tq(multi)>` - Multi-line continuation
- `tq[disconnected]>` - Not connected to database
- `tq[mydb]>` - Connected to specific database

#### 5.3.2 Status Bar (Optional)

```
────────────────────────────────────────────────────────
[user@host:1025/mydb] [TD2] [2.4s] [10 rows]
────────────────────────────────────────────────────────
```

### 5.4 Input Handling

#### 5.4.1 Multi-Line SQL

Queries continue across lines until terminated:

```sql
tq> SELECT
    employee_id,
    first_name,
    last_name
  FROM employees
  WHERE department = 'IT';
```

**Termination Rules**:
- Semicolon (`;`) terminates statement
- Slash (`/`) on empty line executes buffered SQL (Oracle-style)
- `\g` metacommand executes buffered SQL (psql-style)

#### 5.4.2 Command History

**Features**:
- ↑/↓ arrows navigate history
- Ctrl-R for reverse incremental search
- History persisted to `~/.tq_history` (10,000 entries)
- De-duplicates consecutive identical commands
- Excludes metacommands from history

**History Search**:
```
(reverse-i-search)`sel': SELECT * FROM employees
```

#### 5.4.3 Line Editing

**Emacs Mode (Default)**:
- Ctrl-A: Beginning of line
- Ctrl-E: End of line
- Ctrl-K: Kill to end of line
- Ctrl-U: Kill entire line
- Alt-B: Backward word
- Alt-F: Forward word

**Vi Mode** (enabled with `--editor-mode vi`):
- ESC: Enter command mode
- i: Insert mode
- A: Append
- dd: Delete line
- 0: Beginning of line
- $: End of line

### 5.5 SQL Syntax Highlighting

**Color Scheme** (customizable):
- **Keywords** (SELECT, FROM, WHERE): Cyan bold
- **Strings** ('text'): Green
- **Numbers** (123, 45.67): Yellow
- **Comments** (-- comment, /* */): Gray italic
- **Functions** (COUNT, SUM): Magenta
- **Operators** (=, !=, AND, OR): White

**Example**:
```sql
tq> SELECT COUNT(*) FROM employees WHERE dept = 'IT';
     ^^^^^^ ^^^^^^^      ^^^^^^^^^       ^^^^   ^^
     cyan   magenta      keyword         cyan  green
```

### 5.6 Tab Completion

#### 5.6.1 Keyword Completion

```sql
tq> SEL<TAB>
    SELECT

tq> SELECT * FROM emp<TAB>
                  employees
```

#### 5.6.2 Context-Aware Completion

**After FROM**:
```sql
tq> SELECT * FROM <TAB>
    employees    departments    projects    users
```

**After WHERE column**:
```sql
tq> SELECT * FROM employees WHERE dept<TAB>
                                  department
```

**Column Name Completion**:
```sql
tq> SELECT emp<TAB>
           employee_id    employee_name    employee_dept
```

#### 5.6.3 Metacommand Completion

```sql
tq> \d<TAB>
    \d         \describe   \dt        \databases
```

### 5.7 Result Display

#### 5.7.1 Table Formatting

**Default (Fits Terminal)**:
```
┌─────────┬──────────┬─────────────┬─────────┐
│ id      │ name     │ email       │ active  │
├─────────┼──────────┼─────────────┼─────────┤
│ 1       │ Alice    │ a@test.com  │ true    │
│ 2       │ Bob      │ b@test.com  │ false   │
│ 3       │ Charlie  │ c@test.com  │ true    │
└─────────┴──────────┴─────────────┴─────────┘

3 rows in set (0.123s)
```

**Expanded Display** (toggle with `\x`):
```
-[ RECORD 1 ]------------------
id     | 1
name   | Alice
email  | a@test.com
active | true

-[ RECORD 2 ]------------------
id     | 2
name   | Bob
email  | b@test.com
active | false
```

#### 5.7.2 Large Result Handling

**Wide Tables** (horizontal scrolling):
```
Use arrow keys: ← → to scroll, Q to quit pager
[Columns 1-5 of 20] >>>
```

**Long Results** (vertical paging):
```
Rows 1-50 of 1,234 (4%)
Space: next page | b: previous page | q: quit | /: search
```

**Pager Options**:
- `less`-like navigation
- Search with `/pattern`
- Jump to line with `123G`
- Follow mode for streaming results

#### 5.7.3 NULL Handling

Display `NULL` values distinctly:
```
┌─────────┬──────────┐
│ id      │ name     │
├─────────┼──────────┤
│ 1       │ Alice    │
│ 2       │ [NULL]   │  ← grayed out
└─────────┴──────────┘
```

### 5.8 Metacommands

Metacommands provide non-SQL functionality. They start with `/` or `\` and execute immediately.

#### 5.8.1 Connection Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/logon <connection>` | `\c` | Connect to database | `/logon user:pass@host:1025/db` |
| `/disconnect` | `\q` | Disconnect current connection | `/disconnect` |
| `/reconnect` | - | Reconnect to current database | `/reconnect` |
| `/ping` | - | Test connection | `/ping` |

#### 5.8.2 Schema Inspection Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/describe <table>` | `\d` | Describe table structure | `/describe employees` |
| `/list databases` | `\l` | List all databases | `/list databases` |
| `/list tables` | `\dt` | List tables in current database | `/list tables` |
| `/list tables <pattern>` | `\dt` | List tables matching pattern | `/list tables emp%` |
| `/list views` | `\dv` | List views | `/list views` |
| `/list schemas` | `\dn` | List schemas | `/list schemas` |
| `/show indexes <table>` | `\di` | Show table indexes | `/show indexes employees` |

**Example Output**:
```sql
tq> /describe employees
Table: employees
┌───────────────┬──────────┬──────────┬─────────┬─────────┐
│ Column        │ Type     │ Nullable │ Default │ Index   │
├───────────────┼──────────┼──────────┼─────────┼─────────┤
│ employee_id   │ INTEGER  │ NO       │ -       │ PRIMARY │
│ first_name    │ VARCHAR  │ YES      │ NULL    │ -       │
│ last_name     │ VARCHAR  │ YES      │ NULL    │ -       │
│ hire_date     │ DATE     │ YES      │ NULL    │ -       │
│ salary        │ DECIMAL  │ YES      │ NULL    │ -       │
│ department_id │ INTEGER  │ YES      │ NULL    │ FOREIGN │
└───────────────┴──────────┴──────────┴─────────┴─────────┘

Indexes:
  PRIMARY KEY (employee_id)
  FOREIGN KEY (department_id) REFERENCES departments(id)
```

#### 5.8.3 Data Sampling Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/sample <table> [n]` | Show random sample (default 10 rows) | `/sample employees 20` |
| `/peek <table>` | Show first 5 rows and column info | `/peek employees` |

**Example**:
```sql
tq> /sample employees 5
Random sample of 5 rows from employees:
[Results displayed in table format]
```

#### 5.8.4 Export Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/export csv <file>` | Export last result to CSV | `/export csv employees.csv` |
| `/export json <file>` | Export last result to JSON | `/export json data.json` |

#### 5.8.5 Session Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/session` | - | Show session info | `/session` |
| `/timing on` | `\t` | Enable query timing | `/timing on` |
| `/timing off` | `\t` | Disable query timing | `/timing off` |
| `/set format <fmt>` | - | Set output format | `/set format json` |
| `/set pager on` | - | Enable result paging | `/set pager on` |

**Session Info Output**:
```sql
tq> /session
Session Information:
  Host: myhost.company.com:1025
  Database: production_db
  User: alice
  Session ID: 123456789
  Connected: 2024-01-15 10:30:45
  Duration: 15m 23s
  Logon Mechanism: LDAP
  Character Set: UTF8
  Queries Executed: 42
```

#### 5.8.6 Utility Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/help` | `\?` | Show help | `/help` |
| `/help <command>` | - | Show command help | `/help describe` |
| `/clear` | `\clear` | Clear screen | `/clear` |
| `/history` | - | Show command history | `/history` |
| `/edit` | `\e` | Edit last query in $EDITOR | `/edit` |
| `/quit` | `\q` | Exit REPL | `/quit` |

### 5.9 Special Features

#### 5.9.1 Query Editing

**External Editor**:
```sql
tq> /edit
[Opens $EDITOR with last query]
[On save and exit, executes query]
```

**Re-execute Last Query**:
```sql
tq> /repeat
[Executes most recent SQL query]
```

#### 5.9.2 Transaction Support

```sql
tq> BEGIN TRANSACTION;
tq(tx)> INSERT INTO employees VALUES (101, 'Test', 'User');
tq(tx)> SELECT * FROM employees WHERE id = 101;
tq(tx)> ROLLBACK;
tq> -- Transaction rolled back
```

Prompt changes to `tq(tx)>` when in transaction.

#### 5.9.3 Query Cancellation

- **Ctrl-C**: Cancel running query gracefully
- **Double Ctrl-C**: Force quit (last resort)

**Feedback**:
```
Query running... (2.3s) [Press Ctrl-C to cancel]
^C
Query cancelled by user (after 2.3s)
```

#### 5.9.4 Autocorrect Suggestions

```sql
tq> SELCT * FROM employees;
Error: Syntax error near "SELCT"
Did you mean: SELECT?

Fix and retry? [Y/n] y
[Executes corrected query]
```

---

## 6. Batch Mode Specifications

### 6.1 Overview

Batch mode is designed for non-interactive use: scripts, cron jobs, CI/CD pipelines, and command-line data processing.

### 6.2 Execution Modes

#### 6.2.1 Inline Query

```bash
tq query "SELECT COUNT(*) FROM users"
```

#### 6.2.2 File Input

```bash
tq query --file script.sql
```

File format:
```sql
-- comments are supported
SELECT * FROM table1;

-- multiple statements separated by semicolons
INSERT INTO table2 SELECT * FROM table1;
UPDATE table2 SET status = 'processed';
```

#### 6.2.3 stdin Input

```bash
# Pipe from file
cat query.sql | tq query

# Pipe from command
echo "SELECT 1" | tq query

# Heredoc
tq query <<EOF
SELECT employee_id, salary
FROM employees
WHERE salary > 50000
EOF
```

### 6.3 Output Destinations

#### 6.3.1 stdout (Default)

```bash
tq query "SELECT * FROM users" > users.csv
```

#### 6.3.2 File Output

```bash
tq query "SELECT * FROM users" --output users.csv
```

#### 6.3.3 Error Handling

Errors always go to stderr:
```bash
tq query "INVALID SQL" 2> errors.log
tq query "SELECT * FROM users" > data.csv 2> errors.log
```

### 6.4 Scripting Integration

#### 6.4.1 Exit Code Checking

```bash
#!/bin/bash
if tq ping; then
  echo "Database is up"
  tq query "SELECT COUNT(*) FROM active_users" --format json | process.py
else
  echo "Database is down" >&2
  exit 1
fi
```

#### 6.4.2 JSON Processing with jq

```bash
tq query --format json "SELECT id, name, email FROM users" | \
  jq '.[] | select(.name | startswith("A"))' | \
  jq -r '.email'
```

#### 6.4.3 CSV Processing

```bash
# Extract specific columns
tq query --format csv "SELECT * FROM sales" | \
  cut -d',' -f1,3,5 > filtered.csv

# Count rows
tq query --format csv "SELECT * FROM employees" | wc -l

# Convert to TSV
tq query --format csv "SELECT * FROM data" | \
  tr ',' '\t' > data.tsv
```

### 6.5 Performance Considerations

#### 6.5.1 Streaming Results

For large datasets, stream results instead of buffering:

```bash
# Stream 10M rows without exhausting memory
tq query --format csv "SELECT * FROM huge_table" > huge.csv
```

**Implementation**: Write rows incrementally to stdout as they're fetched.

#### 6.5.2 Parallel Processing

```bash
# Split large export into chunks
tq query "SELECT * FROM data WHERE date = '2024-01-01'" &
tq query "SELECT * FROM data WHERE date = '2024-01-02'" &
tq query "SELECT * FROM data WHERE date = '2024-01-03'" &
wait
```

#### 6.5.3 Connection Pooling

Batch mode uses one-shot connections (connect → query → disconnect). No connection pooling needed.

### 6.6 Transaction Control (Future)

```bash
# Atomic script execution
tq query --file migration.sql --atomic

# Equivalent to:
BEGIN TRANSACTION;
[Execute all statements in file]
COMMIT; -- or ROLLBACK on error
```

### 6.7 Variable Substitution (Future)

```bash
# Using environment variables
export TABLE_NAME=employees
tq query "SELECT * FROM ${TABLE_NAME}"

# Using --var flag
tq query --var table=employees --var limit=100 --file template.sql
```

`template.sql`:
```sql
SELECT * FROM {{table}} LIMIT {{limit}};
```

---

## 7. Configuration and Credential Management

### 7.1 Configuration Hierarchy

Configuration is loaded in this order (later overrides earlier):

1. **Built-in defaults**
2. **System config** (`/etc/tq/config.toml`)
3. **User config** (`~/.config/tq/config.toml`)
4. **Project config** (`./.tq.toml`)
5. **Environment variables** (`TQ_*`)
6. **Command-line arguments**

### 7.2 Configuration File Format

#### 7.2.1 User Config (`~/.config/tq/config.toml`)

```toml
# Default connection
[connection]
host = "myteradata.company.com"
port = 1025
user = "myusername"
database = "mydatabase"
logmech = "LDAP"
timeout = "30s"

# Output preferences
[output]
format = "table"
color = "auto"
pager = true
timing = false

# REPL preferences
[repl]
history_file = "~/.tq_history"
history_size = 10000
editor_mode = "emacs"
syntax_highlight = true
autocomplete = true

# Named connection profiles
[profiles.prod]
host = "prod.company.com"
port = 1025
database = "production"
logmech = "KRB5"

[profiles.dev]
host = "dev.company.com"
port = 1025
database = "development"
logmech = "TD2"

[profiles.local]
host = "localhost"
port = 1025
database = "testdb"
logmech = "TD2"
```

#### 7.2.2 Project Config (`.tq.toml`)

For team-shared settings (committed to version control):

```toml
[connection]
host = "shared-dev.company.com"
port = 1025
database = "team_database"
# Note: Never commit passwords!

[output]
format = "json"  # Project prefers JSON output
```

### 7.3 Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `TQ_LOGON` | Complete connection string | `user:pass@host:1025/db` |
| `TQ_HOST` | Database hostname | `myteradata.company.com` |
| `TQ_PORT` | Database port | `1025` |
| `TQ_USER` | Database username | `myuser` |
| `TQ_PASSWORD` | Database password (discouraged) | `mypassword` |
| `TQ_DATABASE` | Database name | `mydatabase` |
| `TQ_LOGMECH` | Authentication mechanism | `LDAP` |
| `TQ_FORMAT` | Default output format | `json` |
| `TQ_TIMEOUT` | Connection timeout | `30s` |
| `TQ_PROFILE` | Configuration profile to use | `prod` |

**Usage**:
```bash
# Set for entire session
export TQ_LOGON="user:pass@host:1025/db"
tq ping
tq query "SELECT 1"

# Set for single command
TQ_FORMAT=json tq query "SELECT * FROM users"
```

### 7.4 Connection Profiles

#### 7.4.1 Using Profiles

```bash
# Select profile via environment
export TQ_PROFILE=prod
tq query "SELECT COUNT(*) FROM users"

# Select profile via flag
tq --profile prod query "SELECT COUNT(*) FROM users"

# List available profiles
tq profile list
```

#### 7.4.2 Managing Profiles

```bash
# Create new profile
tq profile create staging --host staging.db.com --port 1025

# Update profile
tq profile update prod --timeout 60s

# Delete profile
tq profile delete old-dev

# Show profile details
tq profile show prod
```

### 7.5 Credential Management

#### 7.5.1 Security Principles

1. **Never use passwords in CLI arguments** - visible in `ps`, shell history
2. **Never log passwords** - sanitize all debug output
3. **Use file permissions** - `chmod 0600` for credential files
4. **Prefer keyring integration** - OS-native secure storage
5. **Support password prompts** - interactive secure input

#### 7.5.2 Password Sources (Priority Order)

1. **Keyring** (most secure)
2. **Password file** (`--password-file`)
3. **Configuration file** (protected)
4. **Environment variable** (`TQ_PASSWORD`) - discouraged
5. **Interactive prompt** - for missing password

#### 7.5.3 Password File

**Format** (similar to `.pgpass`):
```
# hostname:port:database:username:password
myhost:1025:mydb:alice:secret123
prodhost:1025:*:bob:prodpass
*:1025:*:admin:adminpass
```

**Usage**:
```bash
# Create password file
cat > ~/.tq_passwords <<EOF
myhost:1025:mydb:alice:secret123
EOF
chmod 0600 ~/.tq_passwords

# Use default location (~/.tq_passwords)
tq -l "alice@myhost:1025/mydb" query "SELECT 1"

# Use custom location
tq -l "alice@myhost:1025/mydb" --password-file ~/my-passwords query "SELECT 1"
```

#### 7.5.4 Keyring Integration (Future)

```bash
# Store password in OS keyring
tq password set prod
Enter password: ****

# Use stored password
tq --profile prod query "SELECT 1"
# Automatically retrieves password from keyring

# List stored passwords
tq password list

# Delete stored password
tq password delete prod
```

#### 7.5.5 Interactive Password Prompt

```bash
# Connection without password prompts for it
tq -l "user@host:1025/db" query "SELECT 1"
Password: ****  # secure input, not echoed
```

### 7.6 SSL/TLS Configuration (Future)

```toml
[connection]
host = "secure.company.com"
port = 1025
ssl = true
ssl_mode = "require"  # options: disable, allow, prefer, require, verify-ca, verify-full
ssl_ca_file = "/path/to/ca-cert.pem"
ssl_cert_file = "/path/to/client-cert.pem"
ssl_key_file = "/path/to/client-key.pem"
```

---

## 8. Output Format Specifications

### 8.1 Format Selection

#### 8.1.1 Selection Priority

1. **Command-line flag**: `--format table`
2. **Environment variable**: `TQ_FORMAT=json`
3. **Configuration file**: `format = "csv"`
4. **Context-based default**:
   - TTY (interactive): `table`
   - Piped: `csv` or `json` (configurable)

#### 8.1.2 Format Types

| Format | Use Case | File Extension | MIME Type |
|--------|----------|----------------|-----------|
| `table` | Human-readable, interactive | - | `text/plain` |
| `json` | Scripting, APIs, parsing | `.json` | `application/json` |
| `csv` | Data export, Excel, analysis | `.csv` | `text/csv` |

### 8.2 Table Format

#### 8.2.1 ASCII Table (Default)

```
┌──────────┬──────────┬─────────┐
│ id       │ name     │ active  │
├──────────┼──────────┼─────────┤
│ 1        │ Alice    │ true    │
│ 2        │ Bob      │ false   │
└──────────┴──────────┴─────────┘

2 rows in set (0.123s)
```

**Features**:
- Box-drawing characters
- Auto-sizing columns
- Truncation with ellipsis for wide content
- Row count and timing footer

#### 8.2.2 Simple Table (--table-style simple)

```
 id  | name  | active
-----+-------+--------
 1   | Alice | true
 2   | Bob   | false

(2 rows)
```

**Use Case**: Better for copying/pasting, terminal compatibility

#### 8.2.3 Compact Table (--table-style compact)

```
id name  active
 1 Alice true
 2 Bob   false
```

**Use Case**: Dense output, logs

#### 8.2.4 Markdown Table (--table-style markdown)

```
| id | name  | active |
|----|-------|--------|
| 1  | Alice | true   |
| 2  | Bob   | false  |
```

**Use Case**: Documentation, GitHub issues

#### 8.2.5 Column Alignment

- **Numbers**: Right-aligned
- **Text**: Left-aligned
- **Booleans**: Centered
- **Dates**: Left-aligned

#### 8.2.6 Wide Content Handling

```
┌──────────┬──────────┬──────────────────────┐
│ id       │ name     │ description          │
├──────────┼──────────┼──────────────────────┤
│ 1        │ Alice    │ Senior Software E... │  ← truncated
│ 2        │ Bob      │ Product Manager      │
└──────────┴──────────┴──────────────────────┘

Use --no-truncate to see full content
```

#### 8.2.7 NULL Representation

```
┌──────────┬──────────┬─────────┐
│ id       │ name     │ email   │
├──────────┼──────────┼─────────┤
│ 1        │ Alice    │ a@ex.co │
│ 2        │ Bob      │ [NULL]  │  ← grayed, italic
└──────────┴──────────┴─────────┘
```

### 8.3 JSON Format

#### 8.3.1 Array of Objects (Default)

```json
[
  {
    "id": 1,
    "name": "Alice",
    "email": "alice@example.com",
    "active": true,
    "created_at": "2024-01-15T10:30:00Z"
  },
  {
    "id": 2,
    "name": "Bob",
    "email": "bob@example.com",
    "active": false,
    "created_at": "2024-01-16T11:45:00Z"
  }
]
```

**Features**:
- Each row is a JSON object
- Column names as keys
- Type preservation (numbers, booleans, null)
- ISO 8601 for dates/timestamps

#### 8.3.2 Streaming JSONL (--json-format lines)

```jsonl
{"id":1,"name":"Alice","email":"alice@example.com","active":true}
{"id":2,"name":"Bob","email":"bob@example.com","active":false}
```

**Use Case**: Large datasets, streaming processing

#### 8.3.3 Metadata Wrapper (--json-format wrapped)

```json
{
  "query": "SELECT id, name FROM users",
  "execution_time_ms": 123,
  "row_count": 2,
  "columns": [
    {"name": "id", "type": "INTEGER"},
    {"name": "name", "type": "VARCHAR"}
  ],
  "rows": [
    {"id": 1, "name": "Alice"},
    {"id": 2, "name": "Bob"}
  ]
}
```

**Use Case**: APIs, complete metadata

#### 8.3.4 Type Mapping

| Teradata Type | JSON Type | Example |
|---------------|-----------|---------|
| INTEGER, BIGINT | number | `42` |
| DECIMAL, FLOAT | number | `3.14` |
| VARCHAR, CHAR | string | `"text"` |
| DATE | string | `"2024-01-15"` |
| TIMESTAMP | string | `"2024-01-15T10:30:00Z"` |
| BOOLEAN | boolean | `true`, `false` |
| NULL | null | `null` |
| BLOB, CLOB | string (base64) | `"YWJjMTIz"` |

### 8.4 CSV Format

#### 8.4.1 Standard CSV (RFC 4180)

```csv
id,name,email,active,created_at
1,Alice,alice@example.com,true,2024-01-15T10:30:00Z
2,Bob,bob@example.com,false,2024-01-16T11:45:00Z
```

**Features**:
- Header row (optional with `--no-header`)
- Double-quote escaping for special characters
- Comma separator (configurable with `--delimiter`)
- LF line endings (`\n`)

#### 8.4.2 Excel-Compatible CSV

```bash
tq query --format csv --excel "SELECT * FROM users" > users.csv
```

**Differences**:
- BOM (Byte Order Mark) for UTF-8
- CRLF line endings (`\r\n`)
- Date format: `YYYY-MM-DD`

#### 8.4.3 Custom Delimiter (TSV)

```bash
tq query --format csv --delimiter '\t' "SELECT * FROM data" > data.tsv
```

Output:
```tsv
id      name    email
1       Alice   alice@example.com
2       Bob     bob@example.com
```

#### 8.4.4 Escaping Rules

```csv
id,name,description
1,Alice,"Senior Engineer, Team Lead"
2,Bob,"Quote: ""Hello World"""
3,Charlie,"Line 1
Line 2"
```

**Rules**:
- Fields with commas → quoted
- Fields with quotes → quoted, quotes doubled
- Fields with newlines → quoted

#### 8.4.5 NULL Representation

```csv
id,name,email
1,Alice,alice@example.com
2,Bob,
```

**Options**:
- Empty field (default)
- `--null-string "NULL"` → explicit marker

### 8.5 Format Comparison

| Feature | Table | JSON | CSV |
|---------|-------|------|-----|
| Human-readable | ✅ Excellent | ⚠️ Okay | ⚠️ Okay |
| Machine-parseable | ❌ Poor | ✅ Excellent | ✅ Good |
| Type preservation | ❌ No | ✅ Yes | ❌ No (all strings) |
| Streaming friendly | ✅ Yes | ⚠️ JSONL only | ✅ Yes |
| Excel compatible | ❌ No | ❌ No | ✅ Yes |
| File size | N/A | Medium | Small |
| Processing speed | Fast | Medium | Fast |

---

## 9. Error Handling and User Feedback

### 9.1 Error Categories

#### 9.1.1 User Errors

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

#### 9.1.2 Connection Errors

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

#### 9.1.3 Authentication Errors

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

#### 9.1.4 Query Errors

**Definition**: SQL syntax or execution error
**Exit Code**: `1`
**Strategy**: Show error with context, suggest fixes

**Example**:
```bash
$ tq query "SELCT * FROM users"
Error: SQL syntax error

Query:
  SELCT * FROM users
  ^^^^^ Syntax error here

Details: [3706] Syntax error: expected SELECT but found SELCT

Suggestion: Did you mean "SELECT"?
```

```bash
$ tq query "SELECT * FROM nonexistent_table"
Error: Table does not exist

Query:
  SELECT * FROM nonexistent_table
                ^^^^^^^^^^^^^^^^^

Details: [3807] Object 'nonexistent_table' does not exist

Suggestions:
  - Check spelling: tq query "\dt %table%"
  - List tables: tq query "\dt"
  - Check schema: tq query "\dn"
```

#### 9.1.5 Permission Errors

**Example**:
```bash
$ tq query "DROP TABLE important_data"
Error: Permission denied

Query:
  DROP TABLE important_data

Details: [3523] User 'alice' does not have DROP privilege on table 'important_data'

Action required:
  Contact your database administrator to request privileges
```

#### 9.1.6 System Errors

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

### 9.2 Error Message Structure

**Template**:
```
Error: [Short description]

[Context: what was being attempted]

[Details: technical error message]

[Suggestions: how to fix]

[Links: documentation, issues]
```

**Best Practices**:
1. **Be specific**: Not "Error occurred" but "Connection refused to host:1025"
2. **Show context**: Display relevant query/command
3. **Suggest solutions**: Actionable next steps
4. **Use plain language**: Avoid jargon
5. **Format for scanning**: Use whitespace, bullets

### 9.3 Progress Indicators

#### 9.3.1 Spinner (Indeterminate)

```
⠋ Connecting to myhost:1025...
```

**Use Case**: Connection attempts, query execution (unknown duration)

#### 9.3.2 Progress Bar (Determinate)

```
Exporting data [████████████████░░░░] 80% (8000/10000 rows) ETA: 5s
```

**Use Case**: Large data exports, known row counts

#### 9.3.3 Multi-Progress (Concurrent)

```
✓ Connecting... Done (123ms)
⠋ Executing query...
  Fetching results...
```

**Use Case**: Multi-stage operations

#### 9.3.4 Terminal Detection

Automatically disable progress indicators when:
- Output is piped: `tq query "SELECT 1" | jq`
- Not a TTY: `tq query "SELECT 1" > output.json`
- `--quiet` flag is used

### 9.4 Warnings

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

### 9.5 Verbose Output

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

### 9.6 Logging (Future)

```bash
# Log to file
tq --log-file /tmp/tq.log query "SELECT 1"

# Log to stderr (default)
tq --log-level debug query "SELECT 1" 2> debug.log
```

**Log Levels**: `error`, `warn`, `info`, `debug`, `trace`

---

## 10. Security Requirements

### 10.1 Credential Security

#### 10.1.1 Never Log Credentials

**Implementation**:
- Use `secrecy::Secret<String>` for passwords
- Redact in logs: `user@host:****/db`
- Sanitize error messages
- Clear memory on drop

**Example**:
```rust
// Good
log::debug!("Connecting to {}", sanitize_connection_string(&conn));

// Bad
log::debug!("Connecting with: {}", raw_connection_string);
```

#### 10.1.2 Prevent Credential Leaks

**Avoid**:
- ❌ Passwords in CLI arguments: `tq --password secret123`
- ❌ Passwords in environment (minimize): `TQ_PASSWORD=secret`
- ❌ World-readable config files

**Prefer**:
- ✅ Password files with `0600` permissions
- ✅ OS keyring integration
- ✅ Interactive prompts
- ✅ External credential providers

#### 10.1.3 File Permissions

```bash
# Check config file permissions
$ ls -la ~/.config/tq/config.toml
-rw-------  1 alice  staff  256 Jan 15 10:30 config.toml  # ✅ Good (0600)

# Warn on unsafe permissions
$ chmod 0644 ~/.config/tq/config.toml
$ tq query "SELECT 1"
Warning: Config file ~/.config/tq/config.toml has unsafe permissions (644)
Expected: 0600 (owner read/write only)
Fix: chmod 0600 ~/.config/tq/config.toml

Continuing anyway (use --strict to abort)...
```

### 10.2 SQL Injection Prevention

#### 10.2.1 Parameterized Queries (Future)

```bash
# Safe: Use placeholders
tq query "SELECT * FROM users WHERE id = ?" --param 123

# Unsafe: String concatenation
# DON'T ENABLE THIS:
USER_INPUT="1 OR 1=1"
tq query "SELECT * FROM users WHERE id = $USER_INPUT"
```

#### 10.2.2 Input Validation

**Current Scope**: `tq` passes SQL directly to Teradata
**Mitigation**: Document security best practices
**Future**: Add `--safe` mode that validates common injection patterns

### 10.3 Connection Security

#### 10.3.1 TLS/SSL Encryption (Future)

```bash
# Enforce encrypted connections
tq --ssl-mode require query "SELECT 1"

# Verify server certificate
tq --ssl-mode verify-full --ssl-ca-file ca-cert.pem query "SELECT 1"
```

#### 10.3.2 Connection Timeout

Prevent hanging on unresponsive hosts:

```bash
# Default: 30s timeout
tq query "SELECT 1"

# Custom timeout
tq --timeout 5s ping
```

### 10.4 Data Privacy

#### 10.4.1 Redact Sensitive Data (Future)

```bash
# Mask sensitive columns in logs/errors
tq query "SELECT ssn, name FROM users" --redact ssn
```

#### 10.4.2 Audit Logging

For compliance environments:

```bash
# Log all queries
export TQ_AUDIT_LOG=/var/log/tq-audit.log
tq query "SELECT * FROM sensitive_table"

# Audit log format (JSON):
{"timestamp":"2024-01-15T10:30:00Z","user":"alice","host":"myhost","database":"prod","query":"SELECT * FROM sensitive_table","rows":42}
```

### 10.5 Supply Chain Security

#### 10.5.1 Dependency Auditing

```bash
# CI/CD pipeline
cargo audit --deny warnings
cargo outdated --exit-code 1
```

#### 10.5.2 Binary Verification

```bash
# Provide checksums for releases
$ sha256sum tq-1.0.0-linux-x86_64.tar.gz
a3f2b1c... tq-1.0.0-linux-x86_64.tar.gz

# Sign releases with GPG
$ gpg --verify tq-1.0.0-linux-x86_64.tar.gz.sig
```

### 10.6 Security Hardening

#### 10.6.1 Principle of Least Privilege

**Documentation**:
- Recommend read-only accounts for analysts
- Separate accounts for admin vs. query users
- Use service accounts for automation

#### 10.6.2 Security Defaults

- ✅ Secure by default: warn on insecure configs
- ✅ Fail closed: abort on auth errors
- ✅ No auto-retry: prevent account lockouts
- ✅ Clear secrets: zero memory after use

---

## 11. Performance Considerations

### 11.1 Startup Performance

**Target**: < 100ms cold start

**Strategies**:
1. **Minimal dependencies**: Avoid heavy crates
2. **Lazy initialization**: Don't load config if not needed
3. **Static binary**: No dynamic library loading
4. **Optimized build**: LTO, single codegen unit

**Measurement**:
```bash
time tq --version  # Should be < 50ms
time tq --help     # Should be < 100ms
```

### 11.2 Query Execution Performance

#### 11.2.1 Connection Pooling (REPL Only)

**Batch Mode**: One-shot connections (no pooling)
**REPL Mode**: Maintain single persistent connection

**Configuration**:
```toml
[repl.connection]
idle_timeout = "5m"  # Disconnect after inactivity
ping_interval = "30s"  # Keep-alive ping
```

#### 11.2.2 Result Streaming

**Implementation**: Use iterators/streams to avoid buffering entire result set

```rust
// Good: Stream rows as they arrive
for row in query.execute()? {
    output.write_row(row)?;
}

// Bad: Buffer all rows in memory
let rows = query.execute()?.collect::<Vec<_>>();
output.write_rows(&rows)?;
```

**Benefits**:
- Constant memory usage
- Faster time-to-first-byte
- Handles result sets larger than RAM

#### 11.2.3 Parallel Processing

Not applicable: Teradata connection is inherently sequential

### 11.3 Memory Usage

**Targets**:
- Idle: < 10 MB
- Small query (< 1000 rows): < 20 MB
- Large query (streaming): < 50 MB (constant)

**Strategies**:
1. **Streaming results**: Don't buffer
2. **Efficient data structures**: Avoid clones
3. **Drop early**: Release connections ASAP

**Profiling**:
```bash
# Check memory usage
/usr/bin/time -v tq query "SELECT * FROM huge_table" > /dev/null
```

### 11.4 Large Result Sets

#### 11.4.1 Streaming Output

```bash
# Efficiently export 10M rows
tq query --format csv "SELECT * FROM massive_table" > data.csv
```

**Implementation**: Write rows incrementally, no intermediate buffer

#### 11.4.2 Client-Side Limits

```bash
# Prevent accidental large queries
tq query --limit 1000 "SELECT * FROM table"

# Override limit
tq query --limit -1 "SELECT * FROM table"  # unlimited
```

#### 11.4.3 Server-Side Limits

Use Teradata's `TOP` clause:
```bash
tq query "SELECT TOP 1000 * FROM table"
```

### 11.5 Network Performance

#### 11.5.1 Compression (Future)

```bash
# Enable result compression
tq --compress query "SELECT * FROM large_table"
```

#### 11.5.2 Batching (Future)

For multiple queries:
```bash
tq query --file queries.sql --batch-size 10
```

### 11.6 Build Optimization

**Cargo Profile** (`Cargo.toml`):
```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = "fat"            # Full LTO
codegen-units = 1      # Single codegen unit
strip = "symbols"      # Strip debug symbols
panic = "abort"        # No unwinding
```

**Target Size**:
- Linux (musl): < 5 MB
- macOS: < 4 MB
- Windows: < 5 MB

### 11.7 Performance Monitoring

```bash
# Query timing
tq query --timing "SELECT COUNT(*) FROM large_table"
# Output: (Executed in 2.345s)

# Verbose timing breakdown
tq -v query "SELECT 1"
# [DEBUG] Connection: 127ms
# [DEBUG] Query: 15ms
# [DEBUG] Fetch: 3ms
# [DEBUG] Total: 145ms
```

---

## 12. Future Enhancements

### 12.1 Phase 2: Enhanced REPL (Q1 2024)

| ID | Feature | Priority | Effort |
|----|---------|----------|--------|
| FE-201 | REPL mode implementation | P0 | High |
| FE-202 | Multi-line SQL editing | P0 | Medium |
| FE-203 | Command history | P0 | Medium |
| FE-204 | SQL syntax highlighting | P1 | High |
| FE-205 | Tab completion | P1 | High |
| FE-206 | Result paging | P1 | Medium |
| FE-207 | Vim/Emacs keybindings | P1 | Medium |
| FE-208 | Export last result | P1 | Low |

**Dependencies**:
- `reedline` or `rustyline` for REPL
- `syntect` for syntax highlighting
- `tree-sitter` for intelligent completion

### 12.2 Phase 3: Advanced Features (Q2 2024)

| ID | Feature | Priority | Effort |
|----|---------|----------|--------|
| FE-301 | Configuration profiles | P1 | Medium |
| FE-302 | Connection pooling (REPL) | P1 | Medium |
| FE-303 | Transaction management | P1 | Low |
| FE-304 | Variable substitution | P2 | Medium |
| FE-305 | Query templates | P2 | Medium |
| FE-306 | Batch processing | P2 | High |
| FE-307 | SSL/TLS support | P1 | High |

### 12.3 Phase 4: Enterprise Features (Q3 2024)

| ID | Feature | Priority | Effort |
|----|---------|----------|--------|
| FE-401 | Keyring integration | P2 | Medium |
| FE-402 | SSO/SAML authentication | P2 | High |
| FE-403 | Audit logging | P2 | Medium |
| FE-404 | Query result caching | P3 | Medium |
| FE-405 | Query plan visualization | P3 | High |
| FE-406 | Schema diff tool | P3 | High |

### 12.4 Phase 5: Ecosystem Integration (Q4 2024)

| ID | Feature | Priority | Effort |
|----|---------|----------|--------|
| FE-501 | Homebrew formula | P1 | Low |
| FE-502 | APT/YUM repositories | P2 | Medium |
| FE-503 | Docker image | P2 | Low |
| FE-504 | Shell completion (bash/zsh/fish) | P1 | Medium |
| FE-505 | Man pages | P1 | Low |
| FE-506 | Self-update command | P2 | Medium |

### 12.5 Potential Future Features

#### 12.5.1 Data Transformation

```bash
# Apply transformations during export
tq query "SELECT * FROM users" | tq transform --uppercase name --hash email
```

#### 12.5.2 Query Builder

```bash
# Interactive query builder
tq builder
> select id, name
> from employees
> where department = "IT"
> limit 10
> execute
```

#### 12.5.3 Schema Migration

```bash
# Apply migrations
tq migrate up --file migrations/001_add_users.sql

# Rollback
tq migrate down

# Status
tq migrate status
```

#### 12.5.4 Visualization

```bash
# ASCII charts
tq query "SELECT department, COUNT(*) FROM employees GROUP BY department" | tq chart bar

# Export to image
tq query "SELECT * FROM sales" | tq chart line --output sales.png
```

#### 12.5.5 Monitoring Dashboard

```bash
# Launch web dashboard
tq monitor --port 8080
# Opens http://localhost:8080 with query history, performance metrics
```

---

## Appendix A: CLI Design Checklist

Based on [Command Line Interface Guidelines](https://clig.dev):

### A.1 Basics

- [x] Use a clear, descriptive name (`tq`)
- [x] Provide `--help` and `--version`
- [x] Single binary, no installation script needed
- [x] Exit with code 0 on success, non-zero on error
- [x] Write errors to stderr, output to stdout
- [x] Support `--` to stop parsing flags
- [x] Support `-` to read from stdin

### A.2 Flags

- [x] Prefer flags over positional args for options
- [x] Use consistent flag naming (kebab-case)
- [x] Provide short (`-f`) and long (`--format`) forms
- [x] Boolean flags default to false (use `--no-*` for negation)
- [x] Support both `--flag value` and `--flag=value`
- [x] Group related flags logically

### A.3 Arguments

- [x] Use clear, descriptive argument names in help
- [x] Support reading from stdin when no file argument provided
- [x] Validate arguments early
- [x] Provide defaults for optional arguments

### A.4 Output

- [x] Human-friendly output for TTY
- [x] Machine-parseable output for pipes
- [x] Respect `NO_COLOR` environment variable
- [x] Support `--color always|auto|never`
- [x] Use progress indicators for long operations
- [x] Stream output when possible

### A.5 Errors

- [x] Write error messages to stderr
- [x] Provide actionable error messages
- [x] Show context (what was attempted)
- [x] Suggest solutions
- [x] Use appropriate exit codes

### A.6 Configuration

- [x] Support environment variables
- [x] Support configuration files
- [x] Clear precedence order
- [x] Validate configuration early
- [x] Provide `--dry-run` for safety (planned)

### A.7 Performance

- [x] Fast startup (< 100ms)
- [x] Stream large outputs
- [x] Don't buffer entire datasets
- [x] Provide progress feedback for long operations

### A.8 Future-Proofing

- [x] Use subcommands for related functionality
- [x] Reserve space for future features
- [x] Maintain backward compatibility
- [x] Version configuration file formats

---

## Appendix B: References

### B.1 Standards and Guidelines

- [Command Line Interface Guidelines](https://clig.dev)
- [POSIX Utility Conventions](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html)
- [GNU Coding Standards](https://www.gnu.org/prep/standards/html_node/Command_002dLine-Interfaces.html)
- [RFC 4180 - CSV Format](https://datatracker.ietf.org/doc/html/rfc4180)

### B.2 Inspiration

- **psql** (PostgreSQL): REPL design, metacommands
- **usql** (Universal SQL): Multi-database support
- **ripgrep**: CLI design, performance
- **bat**: Output formatting, syntax highlighting
- **jq**: JSON processing integration

### B.3 Teradata Documentation

- [Teradata SQL Reference](https://docs.teradata.com/)
- [Teradata Rust API](https://github.com/Teradata/teradatarustapi)
- [Teradata JDBC Driver](https://downloads.teradata.com/download/connectivity/jdbc-driver)

---

## Appendix C: Glossary

| Term | Definition |
|------|------------|
| **Batch Mode** | Non-interactive execution of queries via command-line arguments |
| **DSN** | Data Source Name - connection string format |
| **JSONL** | JSON Lines - newline-delimited JSON format |
| **LTO** | Link-Time Optimization - compiler optimization technique |
| **Metacommand** | Special command in REPL mode (e.g., `/describe`, `\dt`) |
| **REPL** | Read-Eval-Print Loop - interactive shell |
| **TTY** | Teletypewriter - terminal context detection |
| **Logmech** | Teradata authentication mechanism (TD2, LDAP, KRB5) |

---

## Appendix D: Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2024-01-10 | Initial MVP (ping, query, basic formats) |
| 1.0.0 | 2024-01-15 | Comprehensive specifications drafted |

---

**Document Status**: Draft - Ready for Review
**Next Review Date**: 2024-02-01
**Owner**: Development Team
**Approvers**: Product Owner, Lead Engineer
