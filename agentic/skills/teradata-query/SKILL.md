---
name: teradata-query
description: Install, configure, and use the tq CLI tool (https://github.com/remi-td/tq/) to run Teradata queries, inspect schemas, bulk load data, and execute in-database ELT pipelines.
user-invocable: true
argument-hint: [query or sql-file]
---

# Teradata Query Execution & Data Pipelines with `tq`

`tq` is a high-performance CLI client for Teradata databases. It provides fast one-shot queries, batch SQL execution, parallel bulk data loading (FastLoad), and schema introspection.

---

## 1. Running Queries with AI Agent Mode (`--agent`)

When executing queries from AI agents or LLM workflows, always use `--agent` (or set `export TQ_AGENT=1`):

```bash
# One-shot query with token-optimized output (TOON format by default)
tq --agent query "SELECT * FROM dbc.dbcinfo"

# Execute a SQL script containing multiple statements
tq query --file pipeline.sql --errorlevel 3807 warning
```

*Note*: `tq query` executes SQL against Teradata database tables. To query data from local CSV, Parquet, or JSON files, first load them into staging tables using `tq fastload`.

### What `--agent` Enables Automatically:
- **Default Format (`toon`)**: Uses Token-Oriented Object Notation, achieving **~79% token reduction** compared to standard row JSON by stripping repeated column keys and verbose decorative borders.
- **Smart JSON Mapping**: When `--json` is requested with `--agent`, `tq` outputs compact columnar JSON (`{"ok":true,"cols":[...],"rows":[[...]]}`), saving **~68% tokens**.
- **Automatic Token Compression**: Rounds floats to 2 decimal places, encodes `NULL` as `~`, and truncates cell strings >100 characters.
- **Token Budget Guardrail**: Enforces a default 4,000 token limit with dynamic truncation feedback (`# Truncated: Showing N of Total rows to fit token budget`).
- **Unrestricted DDL & Scripts**: Full statement freedom for multi-statement DDL and DML execution.

---

## 2. High-Performance Bulk Loading (`tq fastload`)

Teradata is an MPP database. Inserting rows one-by-one (`INSERT INTO ... VALUES (...)`) over network connections is slow. Use `tq fastload` to stream data in parallel directly across AMPs:

```bash
# Bulk load a local CSV, Parquet, or JSON file into a Teradata staging table:
tq fastload path/to/data.csv stg_customers

# Specify custom delimiter if needed (default is comma or tab for .tsv):
tq fastload path/to/data.tsv stg_orders --delimiter '\t'
```

**FastLoad Guidelines:**
- **Workflow Order**: When working with local input files, load them into staging tables with `tq fastload` first before querying or exploring them with `tq`.
- **Automatic Staging Creation**: `tq fastload` automatically infers column names from file headers and creates the staging table if it does not already exist. You do not need to manually write DDL for staging tables.
- **Empty Table Requirement**: FastLoad loads into empty tables. If reloading, drop or recreate the staging table first.
- **Immediate Exploration**: Once loaded into staging tables, explore the columns, row counts, and sample data using `tq peek`, `tq inspect`, or `tq schema`.

---

## 3. Teradata Schema & Data Exploration

`tq` provides built-in subcommands for exploring database objects without writing custom catalog queries:

```bash
# Preview table structure, column data types, and first few rows:
tq peek stg_customers

# Inspect full metadata (column data types, primary index, storage metrics):
tq inspect stg_customers

# One-shot topological schema map and candidate join paths:
tq schema

# List tables in the current database (optional pattern filter):
tq list tables "stg_%"

# Retrieve a random sample of N rows:
tq sample stg_customers 10
```

---

## 4. DDL & Primary Index Best Practices

### A. Idempotent DDL with `--errorlevel 3807 warning`
In deployment and pipeline scripts, dropping a table before recreating it (`DROP TABLE my_table; CREATE TABLE my_table ...;`) fails on the first run if the table does not exist, returning Teradata error **3807** ("Object does not exist").
Pass `--errorlevel 3807 warning` so `tq` logs a warning and continues execution:

```bash
tq query --file deploy_schema.sql --errorlevel 3807 warning
```

### B. Primary Index (PI) Selection
Always specify an explicit `PRIMARY INDEX (column)` with high cardinality (unique or primary key, e.g. `customer_id`, `order_id`) to evenly hash rows across AMPs and prevent data skew:

```sql
CREATE TABLE dim_customers (
    customer_id INTEGER NOT NULL,
    company_name VARCHAR(100),
    signup_date DATE
) PRIMARY INDEX (customer_id);
```

### C. DDL Transaction Boundaries
Teradata strictly prohibits DDL statements (`CREATE`, `DROP`, `ALTER`) inside active transaction blocks alongside DML.
- Never place DDL inside `BT; ... ET;` blocks.
- Do not pass `--atomic` on scripts containing DDL.
- Multi-statement SQL files execute statements sequentially in auto-commit mode by default.

---

## 5. In-Database ELT & Analytics Modeling

Execute set-based data transformations directly inside Teradata using `CREATE TABLE ... AS (...) WITH DATA` or `INSERT INTO ... SELECT ...`. This pushes computation into Teradata's MPP engine rather than pulling datasets into client memory:

```sql
-- Dimensional Transformation
CREATE TABLE dim_customers AS (
    SELECT 
        CAST(customer_id AS INTEGER) AS customer_id,
        CAST(company_name AS VARCHAR(100)) AS company_name,
        CAST(signup_date AS DATE) AS signup_date
    FROM stg_customers
) WITH DATA PRIMARY INDEX (customer_id);

-- Fact Table with Aggregations and Metrics
CREATE TABLE fct_orders AS (
    SELECT 
        CAST(o.order_id AS INTEGER) AS order_id,
        CAST(o.customer_id AS INTEGER) AS customer_id,
        CAST(o.order_date AS DATE) AS order_date,
        CAST(o.amount AS DECIMAL(18,2)) AS amount,
        c.company_name
    FROM stg_orders o
    JOIN dim_customers c ON o.customer_id = c.customer_id
) WITH DATA PRIMARY INDEX (order_id);
```

**ELT Best Practices:**
- **Explicit Type Casting**: Staging tables created by FastLoad store raw strings (`VARCHAR`). Always explicitly cast columns to their target types when performing arithmetic, joins, or aggregations:
  - Numeric: `CAST(units AS DECIMAL(18,2))` or `CAST(id AS INTEGER)`
  - Dates: `CAST(date_col AS DATE)`
  - Timestamps: `CAST(SUBSTRING(ts_col FROM 1 FOR 19) AS TIMESTAMP(0))`

---

## 6. Verification & Auditing

Verify counts, totals, and reconciliation metrics directly via queries:

```bash
# Verify row counts and integrity
tq --agent query "SELECT COUNT(*) FROM dim_customers"
tq --agent query "SELECT COUNT(*), SUM(amount) FROM fct_orders"

# Run a complete audit SQL script
tq --agent query --file audit.sql
```

---

## 7. Troubleshooting & Connection Diagnostics (Reactive Only)

Environment variables and database credentials are pre-configured. Do not proactively execute `tq ping` before queries. If a query or fastload fails with connection errors:
- Test connectivity: `tq ping`
- Verify credentials: `tq` resolves connection parameters from `DATABASE_URI` or `TQ_LOGON` automatically.
