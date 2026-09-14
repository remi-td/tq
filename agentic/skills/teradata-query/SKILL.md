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


## Running Queries

### One-Shot Query

```bash
tq query "SELECT * FROM dbc.dbcinfo"
```

### Execute a SQL File

```bash
tq query --file path/to/script.sql
```

### Batch Script Execution for Predictable Pipelines (`tq query --file`)

While exploratory tasks (inspecting unknown schemas, sampling dirty data, checking column distributions) are naturally **interactive** using `tq schema`, `tq describe`, and `tq sample`, once the pipeline logic and data model are known:
- **Do not invoke 20 individual turn-by-turn CLI queries** for **predictable multi-step data pipelines** (e.g. creating dimensions, facts, aggregations, and views).
- Write a dedicated, descriptively named SQL script (e.g. `<feature_or_product_name>_pipeline.sql` or `order_fulfillment_pipeline.sql`) containing all statements separated by semicolons.
- Execute the entire pipeline in one shot:

```bash
tq query --file order_fulfillment_pipeline.sql --errorlevel 3807 warning
```

**Why this is recommended for AI agents:**
- **One-Shot Execution**: Eliminates 15–20 interactive CLI turns and tool round-trips, matching the efficiency of monolithic scripts while executing natively within Teradata.
- **Sequential Execution**: `tq` parses the file into individual statements and executes each one sequentially across the connection.
- **Idempotent DDL Support**: Passing `--errorlevel 3807 warning` ensures that initial `DROP TABLE` statements do not halt the script if tables do not yet exist on the first run.
- **Transaction Management (`--atomic` vs DDL)**:
  - `--atomic` wraps the entire batch in `BEGIN TRANSACTION ... COMMIT`. This **ONLY works if there is NO DDL** (`CREATE`, `DROP`, `ALTER`) in the script.
  - In Teradata, DDL within an active transaction block causes error `3932: Only an ET or null statement is legal after a DDL Statement`. Therefore, **do NOT use `--atomic` on scripts containing DDL**.
  - If you need atomic rollback for a specific set of DML statements inside a script that also contains DDL, explicitly surround only that DML block with `BT;` (Begin Transaction) and `ET;` (End Transaction) statements, keeping all DDL statements outside of `BT ... ET`.

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

- **DDL Transaction Boundaries**: Teradata strictly prohibits DDL statements (`CREATE`, `DROP`, `ALTER`) inside active transaction blocks alongside DML. Never place DDL inside `BT; ... ET;` or run a script containing DDL with `--atomic`. Multi-statement scripts mixing DDL and DML execute cleanly without `--atomic` because `tq` dispatches each statement sequentially in auto-commit mode.
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
6. **Full Statement & Script Freedom**: `tq` permits all DDL, DML, and multi-statement batch scripts without client-side blocking. Database access and security boundaries are enforced natively at the Teradata user/role level.

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

### Autonomous Data Product & Pipeline Pattern

When building multi-table pipelines or data products, execute phases in batched SQL scripts or an orchestration shell script rather than conversational turn-by-turn queries:

```bash
# 1. Staging DDL (ignore 3807 object-not-found on first run)
tq query --file ddl_staging.sql --errorlevel 3807 warning

# 2. High-performance FastLoad
tq fastload seed_data/accounts.csv stg_accounts
tq fastload seed_data/events.csv stg_events

# 3. In-database ELT transformations & audits in one shot
tq query --file transform_and_audit.sql
```

This pattern completes entire multi-stage data products in 1 to 2 turns, matching Python's conversational turn speed while delivering Teradata's native MPP performance.

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

## Troubleshooting & Installation Setup

> **Note**: In automated agent harnesses or benchmark environments, `tq` and database credentials (`TQ_LOGON` / `DATABASE_URI`) are already pre-configured. Use this section **reactively** only if a command fails with a missing executable or connection error.

### 1. Connection Diagnostics

If a query fails with connection errors:

```bash
tq ping
```

### 2. If `tq` is Not Installed

Install the pre-built binary using the official installer:

```bash
curl -sSL https://raw.githubusercontent.com/remi-td/tq/master/install.sh | sh -s -- --accept-license
```

Verify installation:
```bash
tq --version
```

### 3. Connection Configuration

tq resolves credentials in this order (later overrides earlier):
1. Command-line arguments (`--logon`, `--profile`, `--query-band`)
2. Environment variables (`TQ_LOGON`, `TQ_LOGMECH`, `TQ_QUERY_BAND`, etc.)
3. Project config (`.tq.toml`)
4. User config (`~/.tq/config.toml`)

**Option A: Environment variable (simplest)**
```bash
export TQ_LOGON="user:password@host:1025/database"
```

**Option B: Connection profile**
Profiles are stored in `~/.tq/config.toml` (user) or `.tq.toml` (project):
```toml
[profiles.dev]
host = "dev-td.company.com"
port = 1025
database = "dev_db"
user = "my_user"
logmech = "TD2"
password_file = "~/.tq/passwords/dev"
```
Test with: `tq --profile dev ping`

**QueryBand Tagging (TASM / DBQL Telemetry):**
```bash
export TQ_QUERY_BAND="App=DataPipeline;Job=DailySync;"
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

- **Always use `--agent` (or `export TQ_AGENT=1`)** when executing queries from AI agents or LLM workflows. This minimizes token consumption by ~79% (`toon` format, columnar JSON, 4000 token budget) without blocking DDL or multi-statement scripts.
- **Use `--agent --json`** (which outputs `compact` columnar JSON) when JSON format is strictly required by programmatic parsers, avoiding heavy repeating key overhead.
- **Never hardcode credentials** in SQL files, scripts, or command-line arguments visible in shell history.
- **Use password files** (`--password-file` or profile `password_file`) rather than embedding passwords in `TQ_LOGON` or command-line args.
- **Use `--file`** for executing SQL files rather than pasting long statements inline.
- **Confirm environment** before executing against non-dev targets -- always confirm with the user before running against staging or production.
- **Use `--atomic`** for multi-statement DML migrations that should be all-or-nothing (do not use on scripts containing DDL statements).
