---
name: teradata-query
description: Install, configure, and use the tq CLI tool (https://github.com/remi-td/tq/) to run Teradata queries, explore schemas, monitor sessions, and manage database objects from the command line.
user-invocable: true
argument-hint: [query or sql-file]
---

# Teradata Query Execution with tq

You are running Teradata queries using the **tq** CLI tool.

## What is tq?

tq is a lightweight, Rust-powered CLI client for Teradata databases. It provides one-shot queries, batch SQL file execution, schema exploration, session monitoring, and an interactive REPL -- with no Java dependencies.

Repository: https://github.com/remi-td/tq/

## Readiness Checklist

Before running any query, verify **both** prerequisites in order:

### 1. tq Installation

```bash
tq --version
```

**If missing**, follow the **tq Installation** section below.

### 2. Connection Configuration

tq needs a connection to the Teradata database. Check in this order:

**Option A: Environment variable (simplest)**

```bash
echo $TQ_LOGON
```

If set, tq is ready. Verify with `tq ping`.

**Option B: Connection profile**

```bash
tq profiles
```

If profiles exist, use `tq --profile <name> ping` to test.

**Option C: Project config file**

Check if `.tq.toml` exists in the project root with connection profiles.

**If nothing is configured**, guide the user through setup (see **Connection Setup** below).

**If connection is ready**, skip to **Running Queries**.

---

## tq Installation

Install the pre-built binary using the official installer:

```bash
curl -sSL https://raw.githubusercontent.com/remi-td/tq/master/install.sh | sh -s -- --accept-license
```

The `--accept-license` flag is required for non-interactive installs (the Teradata driver is bundled and requires license acceptance).

This downloads the correct binary for your platform (macOS/Linux, Intel/ARM), verifies the checksum, and installs to `~/.local/bin/tq`.

To install to a custom location:

```bash
TQ_INSTALL_DIR=/path/to/bin curl -sSL https://raw.githubusercontent.com/remi-td/tq/master/install.sh | sh -s -- --accept-license
```

**Verify:**

```bash
tq --version
```

---

## Connection Setup

tq supports multiple connection methods. Choose the one that fits the project.

### Method 1: Environment Variable

The simplest approach -- set `TQ_LOGON` for the session:

```bash
export TQ_LOGON="user:password@host:1025/database"
```

For security, omit the password and use a password file instead:

```bash
export TQ_LOGON="user@host:1025/database"
```

### Method 2: Connection Profiles (recommended)

Profiles are stored in `~/.tq/config.toml` (user-level) or `.tq.toml` (project-level).

**Create a profile interactively:**

```bash
tq profile add dev
```

**Or create the config file manually.** Ask the user for:
- **Host** -- Teradata server hostname (e.g., `dev-td.company.com`)
- **Port** -- usually `1025`
- **Database** -- default database
- **Username**
- **Auth mechanism** -- TD2 (default), LDAP, KRB5, or TDNEGO

Then write `~/.tq/config.toml`:

```toml
[profiles.dev]
host = "dev-td.company.com"
port = 1025
database = "dev_db"
user = "my_user"
logmech = "TD2"
password_file = "~/.tq/passwords/dev"
```

**Set up the password file (secure):**

```bash
mkdir -p ~/.tq/passwords
echo "the_password" > ~/.tq/passwords/dev
chmod 0600 ~/.tq/passwords/dev
```

**Test the profile:**

```bash
tq --profile dev ping
```

### Method 3: Project Config (.tq.toml)

For team-shared profiles, create `.tq.toml` in the project root:

```toml
[profiles.dev]
host = "dev-td.company.com"
database = "dev_db"
user = "shared_dev_user"
password_file = "~/.tq/passwords/dev"

[profiles.prod]
host = "prod-td.company.com"
database = "prod_db"
logmech = "LDAP"
password_file = "~/.tq/passwords/prod"
```

**Important:** Never store passwords in `.tq.toml`. Always use `password_file` pointing to a chmod 0600 file.

### Configuration Precedence

tq resolves configuration in this order (later overrides earlier):
1. Built-in defaults
2. User config (`~/.tq/config.toml`)
3. Project config (`.tq.toml`)
4. Environment variables (`TQ_LOGON`, `TQ_LOGMECH`, etc.)
5. Command-line arguments (`--logon`, `--profile`)

---

## Running Queries

### One-Shot Query

```bash
tq query "SELECT * FROM dbc.dbcinfo"
```

### Execute a SQL File

```bash
tq query --file path/to/script.sql
```

### Batch Statements (multi-statement file or stdin)

```bash
# From a file with multiple statements separated by semicolons
tq query --file multi_statement.sql

# From stdin
tq query <<'EOF'
SELECT CURRENT_DATE;
SELECT DATABASE;
EOF
```

### Atomic Transactions

Wrap multi-statement execution in a transaction (rollback on failure):

```bash
tq query --file migration.sql --atomic
```

### Export Results

```bash
# CSV
tq query "SELECT * FROM sales" --format csv > report.csv
tq query "SELECT * FROM sales" --format csv --output report.csv

# JSON
tq query "SELECT * FROM products" --format json > products.json
```

### Limit Rows

```bash
tq query "SELECT * FROM large_table" --limit 100
```

### Show Timing

```bash
tq query "SELECT * FROM orders" --timing
```

### Variable Substitution (Parameterized SQL)

Use YAML parameter files to inject variables into SQL:

```bash
tq -p params.yaml query "SELECT * FROM {{target.database}}.{{table_name}}"
tq -p base.yaml -p overrides.yaml query --file report.sql
```

Parameter file format:

```yaml
target:
  database: PRODUCTION
table_name: employees
limit: 100
```

Environment variables can also be referenced: `{{$ENV.DATABASE_HOST}}`

### Using a Specific Profile

```bash
tq --profile prod query "SELECT COUNT(*) FROM orders"
```

---

## Schema Exploration

### List Objects

```bash
tq list databases              # List all databases
tq list tables                 # List tables in current database
tq list tables emp%            # Filter with pattern
tq list views                  # List views
```

### Inspect an Object

```bash
tq inspect employees                  # Full metadata: columns, indexes, size
tq inspect mydb.employees             # Qualified name
```

### Show Indexes

```bash
tq show-indexes employees
```

### Peek at Data

```bash
tq peek products              # Preview structure + first rows
```

### Random Sample

```bash
tq sample customers 20        # 20 random rows
```

---

## Monitoring and Administration

### Active Sessions

```bash
tq sessions                   # List active sessions with CPU/IO metrics
```

### System Configuration

```bash
tq sysconfig                  # Version, nodes, AMPs, PEs
```

### Lock Contention

```bash
tq locks                      # Current locks and blocking chains
```

### Query Inspection

```bash
tq query-inspect <session_id> # Recent queries for a session
```

### AMP Skew Analysis

```bash
tq skew                       # Top sessions by skew
tq skew <session_id>          # Skew detail for a session
```

### Session History

```bash
tq history                    # Logon/logoff history and trends
```

### Execution Plans

```bash
tq explain "SELECT * FROM employees WHERE dept = 'ENG'"
```

### Abort a Session

```bash
tq abort <session_id> --force  # Terminate a session (--force required in batch)
```

---

## Interactive REPL

For exploratory work:

```bash
tq repl
tq --profile dev repl
```

### REPL Metacommands

| Command | Purpose |
|---------|---------|
| `/list databases` | List all databases |
| `/list tables pattern%` | List tables matching pattern |
| `/describe table_name` | Show table structure |
| `/sample table_name 20` | Random sample (20 rows) |
| `/peek table_name` | Preview structure + data |
| `/sessions` | Monitor active sessions |
| `/params load file.yaml` | Load parameter file for variable substitution |
| `/params show` | Show loaded parameters |
| `/params unload` | Clear loaded parameters |

### REPL Options

```bash
tq repl --default-limit 50       # Limit SELECT results (default: 100)
tq repl --editor-mode vi         # Vi keybindings (default: emacs)
tq repl --no-pager               # Disable result paging
tq repl --enhanced-timing        # Detailed timing breakdown
```

---

## Connection Check

```bash
tq ping
```

---

## Error Handling

- If a query fails, tq prints the Teradata error code and message to stderr.
- For batch file execution, stop on first error -- do not continue executing subsequent files.
- Common Teradata errors:
  - **3807** -- Object does not exist (check database/table name)
  - **3706** -- Syntax error (check SQL syntax)
  - **2801** -- Authentication failed (check credentials or profile config)
  - **6706** -- Untranslatable character (check for non-ASCII characters in SQL)

---

## Key Rules

- **Never hardcode credentials** in SQL files, scripts, or command-line arguments visible in shell history.
- **Use password files** (`--password-file` or profile `password_file`) rather than embedding passwords in `TQ_LOGON` or command-line args.
- **Use `--file`** for executing SQL files rather than pasting long statements inline.
- **Use `--format json`** when query results will be processed programmatically by other tools or scripts.
- **Confirm environment** before executing against non-dev targets -- always confirm with the user before running against staging or production.
- **Use `--atomic`** for multi-statement migrations that should be all-or-nothing.
