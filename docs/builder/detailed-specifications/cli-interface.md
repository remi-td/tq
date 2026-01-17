# Command-Line Interface Design

**Version:** 1.1.0
**Last Updated:** 2026-01-18
**Owner:** cli-ux-designer agent
**Status:** Active Specification

---

## Table of Contents

1. [Design Philosophy](#41-design-philosophy)
2. [Command Structure](#42-command-structure)
3. [Global Options](#43-global-options)
4. [Commands](#44-commands)
   - [ping - Test Connectivity](#441-ping---test-connectivity)
   - [query - Execute SQL](#442-query---execute-sql)
   - [repl - Interactive Mode](#443-repl---interactive-mode-future)
5. [Input/Output Behavior](#45-inputoutput-behavior)
6. [Flag Design Guidelines](#46-flag-design-guidelines)
7. [Help Text Design](#47-help-text-design)
8. [Version Information](#48-version-information)

---

## 4.1 Design Philosophy

The CLI follows these principles:
- **Subcommands for major operations**: `ping`, `query`, `repl`
- **Global options before subcommand**: `-l/--logon`, `--logmech`, `--password-file`
- **Subcommand-specific options after subcommand**: `--format`, `--output`
- **POSIX-compliant**: Short (`-f`) and long (`--format`) flags
- **Environment variable fallbacks**: `TQ_LOGON`, `TQ_LOGMECH`, `TQ_FORMAT`

## 4.2 Command Structure

```
tq [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

## 4.3 Global Options

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

## 4.4 Commands

### 4.4.1 `ping` - Test Connectivity

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

### 4.4.2 `query` - Execute SQL

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

### 4.4.3 `repl` - Interactive Mode (Future)

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

## 4.5 Input/Output Behavior

### 4.5.1 Standard Streams

| Context | stdin | stdout | stderr |
|---------|-------|--------|--------|
| Query from arg | Ignored | Results | Errors, warnings |
| Query from stdin | SQL query | Results | Errors, warnings |
| REPL mode | User input | Results | Errors, warnings |
| Piped output | Varies | Machine format | Human messages |

### 4.5.2 Terminal Detection

The tool adjusts behavior based on context:

| Feature | Interactive (TTY) | Piped/Redirected |
|---------|-------------------|------------------|
| Color output | Enabled | Disabled |
| Progress indicators | Shown | Hidden |
| Default format | `table` | `csv` or `json` |
| Pager | Enabled (large results) | Disabled |
| Confirmation prompts | Shown | Auto-yes or error |

### 4.5.3 Exit Code Standards

| Code | Meaning | Examples |
|------|---------|----------|
| `0` | Success | Query executed, connection successful |
| `1` | Runtime error | Connection failed, query error, file not found |
| `2` | Usage error | Invalid flag, missing required argument |
| `130` | Interrupted | User pressed Ctrl-C |

---

## 4.6 Flag Design Guidelines

### 4.6.1 Short vs Long Flags

**Short flags** (`-f`):
- Single letter
- For frequently used options
- Can be combined: `-vvv` (very verbose), `-qf json` (quiet JSON)

**Long flags** (`--format`):
- Descriptive kebab-case
- Always available
- Self-documenting

### 4.6.2 Boolean Flags

```bash
# Flag present = true
tq query --timing "SELECT 1"

# Explicit negation with --no-prefix
tq repl --no-history

# Avoid --flag=true syntax (non-standard)
```

### 4.6.3 Value Flags

```bash
# Space-separated (preferred)
tq query --format json "SELECT 1"

# Equals sign (also supported)
tq query --format=json "SELECT 1"

# Short flag with value
tq query -f json "SELECT 1"
```

---

## 4.7 Help Text Design

### 4.7.1 Top-Level Help

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

### 4.7.2 Subcommand Help

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

## 4.8 Version Information

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
