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
4. Environment variables (`TQ_LOGON`, `TQ_LOGMECH`, `TQ_QUERY_BAND`, etc.)
5. Command-line arguments (`--logon`, `--profile`, `--query-band`)

### Teradata QueryBand (Workload & Telemetry Tagging)

Tag session queries for Teradata workload management (TASM) and DBQL (`DBC.QryLogV`) auditing:

```bash
# Via CLI flag:
tq --query-band "App=DataPipeline;Job=DailySync;" query "SELECT 1"

# Inside connection string:
tq --logon "user:pass@host:1025/db?query_band=App=DataPipeline;Job=DailySync;" query "SELECT 1"

# Via environment variable:
export TQ_QUERY_BAND="App=DataPipeline;Job=DailySync;"
```

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

### Multi-Step Pipeline & Batch Script Execution (`tq query --file`)

For multi-step data pipelines (e.g. creating dimensions, facts, aggregations, and views), **do not invoke 20 individual turn-by-turn CLI queries**. Write a single SQL script (`pipeline.sql`) containing all statements separated by semicolons and execute the entire pipeline in one shot:

```bash
tq query --file pipeline.sql --errorlevel 3807 warning
```

**Why this is recommended for AI agents:**
- **One-Shot Execution**: Eliminates 15–20 interactive CLI turns and tool round-trips, matching the efficiency of monolithic scripts while executing natively within Teradata.
- **Sequential Execution**: `tq` parses the file into individual statements and executes each one sequentially across the connection.
- **Idempotent DDL Support**: Passing `--errorlevel 3807 warning` ensures that initial `DROP TABLE` statements do not halt the script if tables do not yet exist on the first run.
- **Atomic Transactions (Optional)**: Pass `--atomic` to automatically wrap DML statements in a transaction with rollback on failure.

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

### High-Performance Bulk Ingestion (`tq fastload`)

For loading local CSV, TSV, Parquet, or JSON files into Teradata staging tables, use `tq fastload`. It transfers data in parallel directly across AMPs, completing in seconds compared to slow row-by-row `INSERT` statements:

```bash
# Bulk load a CSV file into a table (creates table if not exists):
tq fastload seed_data/stg_customers.csv stg_customers

# Bulk load with explicit delimiter:
tq fastload seed_data/stg_orders.tsv stg_orders --delimiter '\t'
```

### Teradata DDL & Primary Index Guidelines

- **Batch Scripts (`--file`)**: Multi-statement DDL scripts are fully supported via `tq query --file script.sql`. Because `tq` parses statements client-side and dispatches them sequentially to Teradata, statements like `DROP TABLE`, `CREATE TABLE`, `INSERT INTO ... SELECT`, and `CREATE VIEW` run smoothly in sequence without session collisions.
- **Primary Index (PI) Selection**: Always specify a `PRIMARY INDEX (column)` with high cardinality (unique or primary key) to evenly hash rows across AMPs and prevent severe table skew. Validate distribution with `tq space <table_name>`.

### Output Formats & Token Optimization

`tq` supports multiple output formats designed for both human inspection and LLM/agent token efficiency:

```bash
# TOON (Token-Oriented Object Notation) - Default for AI agents (-79% tokens vs JSON)
tq query "SELECT * FROM sales" --format toon

# Compact JSON (Columnar JSON) - Machine-readable JSON without repeated keys (-68% tokens vs JSON)
tq query "SELECT * FROM sales" --format compact

# TSV (Tab-Separated Values) - Clean and lightweight tabular data (-72% tokens vs JSON)
tq query "SELECT * FROM sales" --format tsv

# CSV & JSON
tq query "SELECT * FROM sales" --format csv > report.csv
tq query "SELECT * FROM products" --format json > products.json

# Markdown table
tq query "SELECT * FROM sales" --format markdown
```

---

## AI Agent Mode & Token Optimization (`--agent`)

When AI agents interact with databases, standard JSON or tabular outputs waste thousands of context tokens on repeated keys, whitespace, and column padding. Furthermore, agents face "choice overhead" when deciding which flags to pass.

`tq` provides a unified **`--agent`** mode (or environment variable **`TQ_AGENT=1`**) that bundles optimal defaults for AI agents:

```bash
# Recommended for AI agents:
tq --agent query "SELECT * FROM dbc.dbcinfo"

# Or set once in the environment:
export TQ_AGENT=1
tq query "SELECT * FROM dbc.dbcinfo"
```

### What `--agent` Enables by Default

1. **Default Format (`toon`)**: Uses Token-Oriented Object Notation by default. Minimizes quotes, omits decorative borders, and achieves **~79% token reduction** compared to standard row-object JSON.
2. **Smart JSON Mapping**: If an agent requests `--json` or `--format json` along with `--agent`, `tq` automatically upgrades the output to `compact` (Columnar JSON: `{"ok":true,"cols":[...],"rows":[[...]]}`), saving **~68% tokens** while remaining 100% valid JSON.
3. **Automatic Token Compression (`--compress-tokens`)**:
   - Caps floating-point numbers to 2 decimal places (e.g. `123.456789` -> `123.46`).
   - Encodes `NULL` as concise `~`.
   - Truncates oversized text cells exceeding 100 characters with an ellipsis indicator.
4. **Token Budget Guardrail (`--token-budget 4000`)**: Enforces a default 4,000 token limit. Results that exceed this limit are dynamically truncated with a clear summary:
   ```text
   # Truncated: Showing 15 of 240 rows to fit 4000 token budget.
   ```
5. **Token Count Feedback (`--show-tokens`)**: Emits estimated token usage (`# tokens: ~N` or `"tokens_est": N`) so agents can track their context consumption.
6. **Agent Safety Restrictions (`--agent-safe`)**:
   - Forbids multi-statement SQL.
   - Enforces a 30-second query timeout (overriding infinite waits).
   - Enforces client-side fetch caps to prevent memory exhaustion.

### Format Comparison for LLMs

| Format | Token Cost vs JSON | LLM Readability | Best Use Case |
|---|---|---|---|
| **`toon`** | **-79%** | Excellent | **Primary default for all LLM/agent queries** |
| **`compact`** | **-68%** | Excellent | Machine parsing when valid JSON is required |
| **`tsv`** | **-72%** | Good | Script piping and simple parsing |
| **`csv`** | **-55%** | Moderate | Exporting tabular data to CSV files |
| **`json`** | Baseline (0%) | Redundant | Legacy pipelines requiring full row objects |
| **`table`** | +15% to +40% | Human-only | Interactive terminal viewing by human users |

### Fine-Tuning Token Settings

When tighter budgets or specific adjustments are required:

```bash
# Constrain output to a strict token budget (e.g., 1,500 tokens)
tq --agent --token-budget 1500 query "SELECT * FROM large_table"

# Request JSON output while maintaining columnar compression
tq --agent --json query "SELECT * FROM sales"

# Enable token compression on human-oriented formats
tq query "SELECT * FROM orders" --format markdown --compress-tokens
```

---

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

### One-Shot Topological Schema Graph (`tq schema`)

Instead of running multiple turn-by-turn discovery queries (`list tables`, `inspect`, `show-indexes`) across different tables, use `tq schema` to extract the complete topological map, foreign key relationships, primary indexes, and candidate join paths in **one single command**:

```bash
# Extract complete schema graph for current database
tq schema

# Target specific database or filter with pattern
tq schema my_database "order*"

# Output in compact format for AI agents (minimal tokens)
tq schema --format compact
```

This provides immediate relational understanding of all tables and join paths in a single shot.

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

## Space Analysis

Analyze disk space, table allocation, and storage skew across databases or specific tables using `tq space` and `tq dbspace`.

### Database & Object Space Breakdown

```bash
# Database-level summary + list of all child tables/views with Perm space and skew
tq space demo_user

# Object-level space analysis (Current Perm, Peak Perm, Perm Skew %)
tq space demo_user.orders

# Database-level summary only (Perm, Spool, Temp space, allocation limits; omits child tables)
tq dbspace demo_user
```

### Space Analysis Output Formats

```bash
# Export space metrics to CSV (raw byte counts for automated parsing)
tq space demo_user --format csv

# Structured JSON format with nested metric objects
tq space demo_user --format json

# GitHub-flavored Markdown table for documentation
tq space demo_user --format markdown
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
| `/errorlevel [CODE...] [SEVERITY]` | View or set error code severity overrides |
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

## Error Handling & Error Level Control

- If a query fails, `tq` prints the Teradata error code and message to stderr.
- For batch file execution, `tq` stops on the first error by default -- it does not continue executing subsequent statements in the batch unless the error severity is downgraded.
- Common Teradata errors:
  - **3807** -- Object does not exist (e.g. table, view, or database not found)
  - **3706** -- Syntax error (check SQL syntax)
  - **2801** -- Authentication failed (check credentials or profile config)
  - **6706** -- Untranslatable character (check for non-ASCII characters in SQL)

### Controlling Severity Levels (`--errorlevel`)

By default, database errors halt execution. You can downgrade specific error codes to `warning` (or upgrade/map them) using `--errorlevel CODE [CODE...] SEVERITY` so that scripts continue running when encountering non-fatal expected errors.

Supported severities: `warning` (4), `error` (8), `severe` (12), `fatal` (16).

#### Common Pattern: Deployment Scripts & Idempotent DDL (`DROP TABLE` before `CREATE TABLE`)

In DDL deployment scripts, dropping a table before recreating it (`DROP TABLE my_table; CREATE TABLE my_table ...;`) is a common pattern. On a first deployment, the table does not exist, so Teradata returns error **3807** ("Table 'my_table' does not exist"). Without `--errorlevel`, this error halts the script before the `CREATE TABLE` statement is reached.

By passing `--errorlevel 3807 warning`, `tq` logs the missing table as a warning and continues to create the table:

```bash
# Downgrade error 3807 (Object does not exist) to warning so deployment continues
tq query --file deploy_schema.sql --errorlevel 3807 warning
```

*Example `deploy_schema.sql`:*
```sql
-- If table orders does not exist (first run), Teradata returns error 3807.
-- With '--errorlevel 3807 warning', tq logs a warning and proceeds to CREATE TABLE.
DROP TABLE demo_user.orders;

CREATE TABLE demo_user.orders (
    order_id INTEGER NOT NULL,
    customer_id INTEGER,
    order_date DATE
) PRIMARY INDEX (order_id);
```

#### Multiple Error Code Overrides

```bash
# Map both 3807 (object not found) and 3802 (database not found) to warning
tq query --file migration.sql --errorlevel 3807 3802 warning

# Combine multiple mappings
tq query --file script.sql --errorlevel 3807 warning --errorlevel 3523 error
```

#### REPL Metacommand (`/errorlevel`)

In interactive REPL sessions, manage error level mappings dynamically:

```text
/errorlevel 3807 warning     # Set 3807 to warning severity
/errorlevel                  # Display active errorlevel overrides
/errorlevel clear            # Reset all custom severity mappings
```

---

## Key Rules

- **Always use `--agent` (or `export TQ_AGENT=1`)** when executing queries from AI agents or LLM workflows. This minimizes token consumption by ~79%, enforces safe execution, prevents context window overflow with automatic 4000 token budgeting, and applies query timeouts.
- **Use `--agent --json`** (which outputs `compact` columnar JSON) when JSON format is strictly required by programmatic parsers, avoiding heavy repeating key overhead.
- **Never hardcode credentials** in SQL files, scripts, or command-line arguments visible in shell history.
- **Use password files** (`--password-file` or profile `password_file`) rather than embedding passwords in `TQ_LOGON` or command-line args.
- **Use `--file`** for executing SQL files rather than pasting long statements inline.
- **Confirm environment** before executing against non-dev targets -- always confirm with the user before running against staging or production.
- **Use `--atomic`** for multi-statement migrations that should be all-or-nothing.
