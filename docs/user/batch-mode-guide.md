# Batch Mode Guide

This guide covers using tq for non-interactive batch operations: scripts, automation, CI/CD pipelines, and data processing workflows.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Input Methods](#input-methods)
3. [Output to File](#output-to-file)
4. [Transaction Control](#transaction-control)
5. [Multi-Statement Scripts](#multi-statement-scripts)
6. [Variable Substitution](#variable-substitution)
7. [Error Handling](#error-handling)
8. [Scripting Patterns](#scripting-patterns)
9. [Performance Tips](#performance-tips)
10. [Common Recipes](#common-recipes)
11. [Teradata Session Types and Transaction Support](#teradata-session-types-and-transaction-support)
12. [Best Practices](#best-practices)
13. [Troubleshooting](#troubleshooting)

---

## Quick Start

Three ways to run SQL in batch mode:

```bash
# 1. Inline query (quick one-liners)
tq query "SELECT COUNT(*) FROM users"

# 2. From file (saved scripts)
tq query --file migration.sql

# 3. From stdin (pipes and heredocs)
echo "SELECT CURRENT_DATE" | tq query
```

All three methods:
- Execute immediately and exit
- Return exit code 0 on success, non-zero on failure
- Output results to stdout by default

---

## Input Methods

### Inline Queries

Best for simple, one-off queries in shell scripts.

```bash
# Basic query
tq query "SELECT COUNT(*) FROM orders WHERE date = CURRENT_DATE"

# With formatting
tq query "SELECT * FROM users SAMPLE 10" --format json

# Store result in variable
user_count=$(tq query "SELECT COUNT(*) FROM users" --format csv --no-header)
echo "Total users: $user_count"
```

**Limitations:**
- Single statement only (no semicolon splitting)
- Shell quoting rules apply
- Not ideal for complex SQL

**Tips:**
- Use double quotes for SQL, single quotes for strings inside SQL
- Escape special characters with backslash
- Keep queries under ~200 characters for readability

### File Input

Best for saved queries, migration scripts, and complex operations.

```bash
# Basic usage
tq query --file setup.sql

# With output formatting
tq query --file report.sql --format csv > report.csv

# Combine with other flags
tq query --file migration.sql --atomic --verbose
```

**File format:**
```sql
-- comments.sql
-- Single-line comments supported

/* Multi-line comments
   also work */

-- Multiple statements separated by semicolons
CREATE TABLE test (id INT);
INSERT INTO test VALUES (1), (2), (3);
SELECT * FROM test;
DROP TABLE test;
```

**Advantages:**
- Multi-statement execution
- Easy to version control
- Supports SQL comments
- Reusable and maintainable

### stdin Input

Best for pipelines, dynamic SQL generation, and shell integration.

```bash
# Pipe from file
cat query.sql | tq query

# Pipe from command
echo "SELECT 1" | tq query

# Heredoc (great for embedded SQL in scripts)
tq query <<EOF
SELECT employee_id, salary
FROM employees
WHERE department = 'Engineering'
ORDER BY salary DESC
SAMPLE 10;
EOF

# Process substitution
tq query < <(generate_query.py)
```

**Use cases:**
- Dynamic SQL generation
- Template expansion
- Multi-source concatenation
- Integration with other tools

---

## Output to File

### Basic File Output

Use `--output` (or `-o`) to write results directly to a file:

```bash
# Write query results to file
tq query "SELECT * FROM users" --output users.csv --format csv

# Shorter form
tq query "SELECT * FROM products" -o products.json --format json
```

**Status message:**
```bash
$ tq query "SELECT * FROM users" --output users.csv --format csv
Wrote 1523 rows to users.csv
```

### Why Use --output Instead of Shell Redirection?

Both methods work, but `--output` provides:

1. **Status confirmation** (shows row count)
2. **Better error handling** (atomic writes)
3. **Interactive confirmation** (prompts before overwrite)

**Comparison:**

```bash
# Shell redirection (UNIX style, silent)
tq query "SELECT * FROM users" > users.csv

# Explicit output (verbose, safe)
tq query "SELECT * FROM users" --output users.csv
```

Use shell redirection (`>`) for:
- UNIX pipelines
- Scripts where silence is golden
- When you don't need confirmation

Use `--output` for:
- Interactive use
- When you want confirmation
- Safety-critical operations

### File Overwrite Protection

When the output file already exists:

**Interactive mode (terminal):**
```bash
$ tq query "SELECT * FROM users" --output users.csv
File exists: users.csv

Overwrite? [y/N]: _
```

- Type `y` or `yes` to overwrite
- Press Enter or type `n` to abort
- Press Ctrl-C to cancel

**Non-interactive mode (scripts, CI):**
```bash
$ tq query "SELECT * FROM users" --output users.csv
Error: File exists: users.csv
$ echo $?
2
```

In non-interactive environments, the operation aborts if the output file exists. Delete the existing file first or use shell redirection (`>`) which always overwrites.

### Atomic Writes

The `--output` flag uses atomic file operations:

1. Query results written to `<output>.tmp.<random>`
2. On success: temporary file renamed to final path
3. On error: temporary file deleted

**Benefits:**
- No partial files on query errors
- No partial files on interruption (Ctrl-C)
- Safe for concurrent operations

**Example:**
```bash
# If this query fails halfway through...
$ tq query "SELECT * FROM huge_table" --output data.csv
Error: Query timeout

# ...you won't have a half-written data.csv
# The temporary file is cleaned up automatically
```

### Quiet Mode

Suppress status messages with `--quiet`:

```bash
# Silent operation (useful for scripts)
tq query "SELECT * FROM users" --output users.csv --format csv --quiet

# Only errors printed
$ tq query "SELECT * FROM users" --output /readonly/users.csv --quiet
Error: Cannot write to file
Could not write to: /readonly/users.csv
Reason: Permission denied
```

### Multi-Statement Scripts with Output

When running multi-statement scripts, only the LAST SELECT query result goes to the file:

```sql
-- report.sql
CREATE TABLE temp_summary AS (
  SELECT department, COUNT(*) as count
  FROM employees
  GROUP BY department
) WITH DATA;

-- This result goes to the output file
SELECT * FROM temp_summary ORDER BY count DESC;

DROP TABLE temp_summary;
```

```bash
$ tq query --file report.sql --output summary.csv --format csv
Statement 1: CREATE TABLE - OK
Statement 2: SELECT - 5 rows returned
Wrote 5 rows to summary.csv
Statement 3: DROP TABLE - OK
```

---

## Transaction Control

### Why Use Transactions?

Transactions ensure all-or-nothing execution: either ALL statements succeed, or ALL changes are rolled back.

**Use cases:**
- Database migrations
- Multi-step data updates
- Critical operations that must be atomic
- Complex transformations

**Without transactions:**
```sql
-- migration.sql
UPDATE users SET status = 'active';      -- Succeeds
UPDATE orders SET processed = 1;         -- Succeeds
UPDATE audit_log SET reviewed = true;    -- FAILS
-- Result: First two updates committed, last one failed. Inconsistent state!
```

**With transactions:**
```sql
-- Same script, but with --atomic flag
UPDATE users SET status = 'active';      -- Succeeds
UPDATE orders SET processed = 1;         -- Succeeds
UPDATE audit_log SET reviewed = true;    -- FAILS
-- Result: ALL changes rolled back. Database unchanged.
```

### Using --atomic Flag

The `--atomic` flag automatically wraps your statements in a transaction:

```bash
# Execute script with transaction protection
tq query --file migration.sql --atomic
```

**What it does:**
1. Begins transaction automatically
2. Executes all your statements
3. On success: commits transaction
4. On error: rolls back transaction

**Success output:**
```bash
$ tq query --file update.sql --atomic
Statement 1: UPDATE users - OK (150 rows affected)
Statement 2: UPDATE orders - OK (340 rows affected)
Statement 3: INSERT INTO audit_log - OK (1 row affected)

All statements executed successfully
Transaction committed
```

**Failure output:**
```bash
$ tq query --file update.sql --atomic
Statement 1: UPDATE users - OK (150 rows affected)
Statement 2: UPDATE orders - OK (340 rows affected)
Statement 3: INSERT INTO invalid_table - FAILED

Error: Table does not exist in statement 3

Table or view 'invalid_table' does not exist.

Transaction rolled back (all changes reverted)
Statements executed: 1-2
Statement failed: 3

Exit code: 1
```

### Manual vs Automatic Transactions

**Manual (explicit BEGIN/COMMIT):**
```sql
-- manual_transaction.sql
BEGIN TRANSACTION;

UPDATE users SET status = 'active';
UPDATE orders SET processed = 1;

COMMIT;
```

```bash
# Run without --atomic
tq query --file manual_transaction.sql
```

**Automatic (--atomic flag):**
```sql
-- auto_transaction.sql
-- No BEGIN/COMMIT needed

UPDATE users SET status = 'active';
UPDATE orders SET processed = 1;
```

```bash
# tq adds transaction wrapper automatically
tq query --file auto_transaction.sql --atomic
```

**Recommendation:** Use `--atomic` for simplicity and better error handling.

### Important Limitations

#### Single Statement Queries

The `--atomic` flag has no effect on single statements:

```bash
$ tq query "UPDATE users SET status = 'active'" --atomic
Warning: --atomic has no effect on single statements
Statement 1: UPDATE - OK (150 rows affected)
```

Single statements are already atomic in database systems.

#### Explicit Transaction Conflicts

You cannot use `--atomic` with scripts that already have transaction control:

```bash
$ cat script.sql
BEGIN TRANSACTION;
UPDATE users SET status = 'active';
COMMIT;

$ tq query --file script.sql --atomic
Error: Cannot use --atomic with explicit transaction control

Your script contains BEGIN, COMMIT, or ROLLBACK statements.
Remove these statements to use --atomic, or run without the flag.

Exit code: 2
```

#### DDL Limitations

Some database operations cannot be in transactions:

```bash
$ cat ddl.sql
CREATE TABLE test (id INT);
DROP TABLE old_table;

$ tq query --file ddl.sql --atomic
Statement 1: CREATE TABLE - FAILED

Error: Statement not allowed in transaction

Teradata does not allow this DDL statement inside a transaction.
Remove --atomic flag for DDL-heavy scripts.

Transaction rolled back
Exit code: 1
```

**Solution:** Don't use `--atomic` for DDL operations (CREATE, DROP, ALTER).

### Interrupt Handling

Pressing Ctrl-C during atomic execution automatically rolls back:

```bash
$ tq query --file long_update.sql --atomic
Statement 1: UPDATE - OK (1000 rows affected)
Statement 2: UPDATE - in progress...
^C
Interrupted. Rolling back transaction...
Transaction rolled back (all changes reverted)

Exit code: 130
```

Your database is always left in a consistent state.

---

## Multi-Statement Scripts

### Statement Separation

Separate statements with semicolons:

```sql
-- multiple.sql
SELECT COUNT(*) FROM users;

SELECT COUNT(*) FROM orders;

SELECT COUNT(*) FROM products;
```

**Execution:**
```bash
$ tq query --file multiple.sql
Statement 1: SELECT - 1 row returned
┌───────┐
│ count │
├───────┤
│ 1523  │
└───────┘

Statement 2: SELECT - 1 row returned
┌───────┐
│ count │
├───────┤
│ 8401  │
└───────┘

Statement 3: SELECT - 1 row returned
┌───────┐
│ count │
├───────┤
│ 342   │
└───────┘

All statements executed successfully
```

### Mixed Statement Types

Combine queries, updates, and DDL:

```sql
-- mixed.sql
CREATE TABLE temp_data (id INT, value VARCHAR(100));

INSERT INTO temp_data VALUES (1, 'test');
INSERT INTO temp_data VALUES (2, 'test2');

SELECT * FROM temp_data;

UPDATE temp_data SET value = 'updated' WHERE id = 1;

SELECT * FROM temp_data;

DROP TABLE temp_data;
```

Only SELECT results are displayed. Other statements show status.

### Comments

Both SQL comment styles work:

```sql
-- single_line_comment.sql

-- This is a single-line comment
SELECT 1;

/* This is a
   multi-line
   comment */
SELECT 2;

/* Comments can be /* nested in some cases */ */
SELECT 3;
```

### Known Limitation: Semicolons in Strings

The simple semicolon splitting doesn't handle `;` inside strings:

```sql
-- This will split incorrectly:
INSERT INTO messages VALUES ('Hello; World');
-- tq sees two statements: "INSERT INTO..." and "World')"
```

**Workarounds:**
1. Use single-statement execution for complex strings
2. Most real-world SQL doesn't have `;` in strings
3. Escape or avoid semicolons in string literals

---

## Variable Substitution

tq provides native variable substitution for SQL templates. Write your SQL with `{{variable}}` markers and supply values from a YAML file using the `-p`/`--params` flag. This keeps SQL logic separate from execution parameters and enables safe reuse of scripts across environments.

### Quick Example

**1. Create a parameter file:**

```yaml
# params.yaml
table: employees
limit: 25
```

**2. Write a SQL template:**

```sql
SELECT * FROM {{table}} SAMPLE {{limit}};
```

**3. Execute with parameters:**

```bash
tq -p params.yaml query --file report.sql
# Executes: SELECT * FROM employees SAMPLE 25
```

### The `--params` / `-p` Flag

The `-p` flag is a global option, meaning it comes before the subcommand:

```bash
tq -p params.yaml query "SELECT * FROM {{table}}"
tq -p params.yaml query --file script.sql
cat script.sql | tq -p params.yaml query
```

The `--params` form works the same way:

```bash
tq --params params.yaml query --file report.sql
```

### Parameter File Format (YAML)

Parameter files are standard YAML files with key-value pairs. Both flat and nested structures are supported:

```yaml
# deploy.yaml

# Simple scalars
table: employees
limit: 100
run_date: "2026-01-01"

# Nested structure (accessed via dot notation)
target:
  database: PRODUCTION
  schema: HR

filters:
  region: EMEA
  active: true
```

**Supported value types:**

| YAML Type | Example | Substituted As |
|-----------|---------|----------------|
| String | `name: employees` | `employees` |
| Integer | `count: 100` | `100` |
| Float | `threshold: 99.5` | `99.5` |
| Boolean | `active: true` | `true` |
| Null | `filter: ~` | `NULL` (SQL keyword) |

### Dot Notation for Nested Keys

Access nested YAML keys using dot notation in your markers:

```yaml
# deploy.yaml
target:
  database: PRODUCTION
  schema: HR
```

```sql
-- migrate.sql
SELECT * FROM {{target.database}}.{{target.schema}}.employees;
-- Executes: SELECT * FROM PRODUCTION.HR.employees;
```

This is especially useful for grouping related parameters together:

```yaml
# config.yaml
source:
  database: STAGING
  schema: HR

dest:
  database: PRODUCTION
  schema: HR
```

```sql
-- copy.sql
INSERT INTO {{dest.database}}.{{dest.schema}}.employees
SELECT * FROM {{source.database}}.{{source.schema}}.employees
WHERE hire_date > '{{run_date}}';
```

### Environment Variable Substitution

Use the `{{$ENV.VAR_NAME}}` syntax to read values directly from environment variables. No YAML file is needed for this:

```bash
# Read database host from environment
export TARGET_DB=PRODUCTION

tq query "SELECT * FROM {{$ENV.TARGET_DB}}.HR.employees SAMPLE 10"
# Executes: SELECT * FROM PRODUCTION.HR.employees SAMPLE 10
```

Mix environment variables and YAML params in the same query:

```yaml
# params.yaml
schema: HR
limit: 50
```

```bash
export ENV_NAME=prod

tq -p params.yaml query \
  "SELECT * FROM {{$ENV.ENV_NAME}}.{{schema}}.employees SAMPLE {{limit}}"
# Executes: SELECT * FROM prod.HR.employees SAMPLE 50
```

**When the environment variable is not set:**

```
Error: Undefined environment variable in template

Variable '{{$ENV.TARGET_DB}}' references environment variable 'TARGET_DB'
which is not set in the current environment.

Fix:
  export TARGET_DB=myvalue
  tq query -p params.yaml "..."
```

### Multiple Parameter Files

Supply multiple `-p` flags to merge parameter files. Later files override earlier files on conflicting keys, but non-conflicting keys are preserved:

```bash
tq -p base.yaml -p prod-overrides.yaml query --file report.sql
```

**How merging works:**

```yaml
# base.yaml
database: STAGING
schema: HR
filters:
  region: GLOBAL
  active: true
```

```yaml
# prod-overrides.yaml
database: PRODUCTION       # overrides base.yaml
filters:
  region: EMEA             # overrides base.yaml filters.region
  # filters.active is not here, so it stays 'true' from base.yaml
```

**Effective parameters after merge:**

```yaml
database: PRODUCTION       # from prod-overrides.yaml
schema: HR                 # from base.yaml (not overridden)
filters:
  region: EMEA             # from prod-overrides.yaml
  active: true             # from base.yaml (not overridden)
```

**Pattern:** Keep stable defaults in `base.yaml`, environment-specific overrides in separate files:

```bash
# Development
tq -p base.yaml -p envs/dev.yaml query --file report.sql

# Production
tq -p base.yaml -p envs/prod.yaml query --file report.sql
```

### Quoting in SQL Templates

Variable substitution inserts values as raw text. You are responsible for SQL quoting in the template:

```yaml
# params.yaml
department: Sales
```

```sql
-- Correct: quotes are in the template
SELECT * FROM employees WHERE department = '{{department}}';
-- Executes: SELECT * FROM employees WHERE department = 'Sales';

-- Incorrect: missing quotes will fail
SELECT * FROM employees WHERE department = {{department}};
-- Executes: SELECT * FROM employees WHERE department = Sales; (syntax error)
```

### Error Handling

**Undefined variable:**

```
Error: Undefined variable in template

Variable '{{target_schema}}' is not defined.

Available variables:
  database          PRODUCTION
  filters.region    EMEA
  filters.active    true
  row_count         100

Fix:
  Add 'target_schema: <value>' to your parameter file, or
  use '-p another-file.yaml' with the missing key defined.

Hint: Run 'tq help params' for syntax reference.
```

**Parameter file not found:**

```
Error: Parameter file not found

Could not read: myparams.yaml
Reason: No such file or directory

Check:
  - File path is correct (relative paths are resolved from current directory)
  - File exists and is readable
  - Current directory: /Users/alice/project
```

**YAML parse error:**

```
Error: Invalid YAML in parameter file

Could not parse: params.yaml
Line 7: mapping values are not allowed in this context

Fix:
  - Verify the file is valid YAML
  - Check for missing quotes around special characters
  - Check for incorrect indentation
```

All variable substitution errors produce exit code 2 (usage error). No SQL is sent to Teradata when a variable cannot be resolved.

### Interaction with --atomic

Variable substitution runs before transaction wrapping. All markers in all statements are resolved first. If any marker is undefined, no statements execute and no transaction starts:

```bash
tq -p params.yaml query --file migration.sql --atomic
```

### Variable Substitution in Scripts

**Deploy across environments:**

```bash
#!/bin/bash
# deploy.sh

ENV=${1:-staging}

tq -p config/base.yaml \
   -p config/envs/${ENV}.yaml \
   query --file migrations/latest.sql --atomic

echo "Deployed to ${ENV}"
```

**Parameterized reporting:**

```bash
#!/bin/bash
# generate_report.sh

DATE=$(date +%Y-%m-%d)
QUARTER=$(date +Q%q-%Y)

# Create a temporary params file for today's run
cat > /tmp/run_params.yaml <<EOF
run_date: "${DATE}"
quarter: "${QUARTER}"
output_table: reports_${DATE//-/}
EOF

tq -p config/report_base.yaml \
   -p /tmp/run_params.yaml \
   query --file reports/quarterly_summary.sql \
   --format csv \
   --output "reports/summary_${DATE}.csv"

rm /tmp/run_params.yaml
```

**Full reference:** Run `tq help params` for complete variable substitution syntax.

---

## Error Handling

### Exit Codes

tq follows standard UNIX conventions:

| Exit Code | Meaning | Use Case |
|-----------|---------|----------|
| 0 | Success | All statements executed successfully |
| 1 | Runtime error | SQL error, connection failure, permission denied |
| 2 | Usage error | Invalid arguments, file conflicts |
| 130 | Interrupted | User pressed Ctrl-C |

**In scripts:**
```bash
#!/bin/bash

# Stop on any error
set -e

tq query --file migration.sql
echo "Migration completed"

# Or check explicitly
if tq query --file migration.sql; then
  echo "Success"
  exit 0
else
  echo "Failed with exit code $?" >&2
  exit 1
fi
```

### Fail-Fast Behavior

By default, tq stops on the first error:

```sql
-- script.sql
SELECT 1;           -- Succeeds
SELECT 2;           -- Succeeds
INVALID SQL;        -- FAILS HERE
SELECT 3;           -- Never executes
SELECT 4;           -- Never executes
```

**Why fail-fast?**
- Prevents cascading errors
- Safe default for migrations
- Clear error identification
- Prevents partial state

### Error Context

Default error messages show:
- Statement number where error occurred
- Error description and Teradata error code
- Failed statement preview
- Execution summary

```bash
$ tq query --file script.sql
Statement 1: SELECT - 1 row returned
Statement 2: INSERT - FAILED

Error: Permission denied in statement 2

User 'alice' does not have INSERT privilege on table 'protected_data'.

Error Code: 3523
Session ID: 1429

Failed statement:
  INSERT INTO protected_data VALUES (1, 'test');

Statements executed: 1
Statements remaining: 3-5

Exit code: 1
```

### Verbose Error Context

Add `--verbose` for detailed execution logs:

```bash
$ tq --verbose query --file script.sql
[INFO] Reading SQL from script.sql
[INFO] Found 5 statements

Statement 1: SELECT COUNT(*) FROM users
  Status: OK - 1 row returned (0.12s)

Statement 2: INSERT INTO audit_log VALUES (...)
  Status: OK - 1 row affected (0.08s)

Statement 3: SELCT * FROM users
  Status: FAILED

Error: SQL syntax error in statement 3 of script.sql

Expected something like a 'SELECT' keyword but found 'SELCT'.

File: script.sql
Statement number: 3 of 5
Failed statement:
  SELCT * FROM users WHERE active = 1;

Statements executed successfully:
  1. SELECT COUNT(*) FROM users - OK (0.12s)
  2. INSERT INTO audit_log VALUES (...) - OK (0.08s)

Statements not executed:
  3. SELCT * FROM users WHERE active = 1; (FAILED)
  4. UPDATE users SET last_seen = CURRENT_TIMESTAMP;
  5. COMMIT;

Exit code: 1
```

### Separating Data and Errors

Errors always go to stderr, data to stdout:

```bash
# Capture data and errors separately
tq query "SELECT * FROM users" > data.csv 2> errors.log

# Check for errors
if [ -s errors.log ]; then
  echo "Query failed:"
  cat errors.log
  exit 1
fi

# Suppress errors (not recommended)
tq query "SELECT * FROM users" 2>/dev/null

# Both to same file
tq query --file script.sql > output.log 2>&1
```

---

## Scripting Patterns

### Exit Code Checking

```bash
#!/bin/bash

# Method 1: set -e (stop on any error)
set -e
tq query --file step1.sql
tq query --file step2.sql
echo "All steps completed"

# Method 2: explicit check
if ! tq query --file migration.sql; then
  echo "Migration failed" >&2
  exit 1
fi

# Method 3: capture and analyze
tq query --file report.sql > report.txt 2> errors.txt
if [ $? -ne 0 ]; then
  echo "Report generation failed"
  cat errors.txt >&2
  exit 1
fi
```

### Conditional Execution

```bash
#!/bin/bash

# Check database connectivity first
if tq ping; then
  echo "Database available"
  tq query --file daily_report.sql > report.txt
else
  echo "Database unavailable" >&2
  exit 1
fi

# Chained operations
tq ping && tq query --file step1.sql && tq query --file step2.sql || {
  echo "Pipeline failed" >&2
  exit 1
}
```

### Variable Substitution

For reusable SQL templates, use tq's native `-p`/`--params` flag with a YAML parameter file. This is safer and more expressive than shell-based substitution:

```bash
#!/bin/bash

# Recommended: tq native variable substitution
tq -p params.yaml query --file report.sql
```

See the [Variable Substitution](#variable-substitution) section above for full details.

For simple one-off cases, shell features also work:

```bash
#!/bin/bash

# heredoc with shell variable expansion
TABLE="employees"
DATE="2024-01-01"

tq query <<EOF
SELECT *
FROM ${TABLE}
WHERE hire_date > '${DATE}'
SAMPLE 10;
EOF

# envsubst (requires envsubst tool)
export TABLE_NAME=employees
cat template.sql | envsubst | tq query
```

### Loop Processing

```bash
#!/bin/bash

# Process multiple dates
for date in 2024-01-01 2024-01-02 2024-01-03; do
  echo "Processing $date..."
  tq query "SELECT * FROM orders WHERE date = '${date}'" \
    --format csv \
    --output "orders_${date}.csv"
done

# Process multiple tables
for table in users orders products; do
  echo "Exporting $table..."
  tq query "SELECT * FROM ${table}" \
    --format json \
    --output "${table}.json"
done
```

### Error Recovery

```bash
#!/bin/bash

# Retry logic
MAX_RETRIES=3
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
  if tq query --file critical_update.sql; then
    echo "Update succeeded"
    exit 0
  else
    RETRY_COUNT=$((RETRY_COUNT + 1))
    echo "Attempt $RETRY_COUNT failed, retrying..." >&2
    sleep 5
  fi
done

echo "Update failed after $MAX_RETRIES attempts" >&2
exit 1
```

### Logging

```bash
#!/bin/bash

LOG_FILE="/var/log/tq_batch.log"

# Log function
log() {
  echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

# Run with logging
log "Starting daily export"

if tq query --file daily_export.sql --output export.csv; then
  log "Export completed successfully"
else
  log "Export failed with exit code $?"
  exit 1
fi

log "Export finished"
```

---

## Performance Tips

### Minimize Connections

Each `tq` invocation creates a new database connection (~100-200ms overhead).

**Inefficient (5 connections):**
```bash
tq query "SELECT 1"
tq query "SELECT 2"
tq query "SELECT 3"
tq query "SELECT 4"
tq query "SELECT 5"
```

**Efficient (1 connection):**
```sql
-- queries.sql
SELECT 1;
SELECT 2;
SELECT 3;
SELECT 4;
SELECT 5;
```
```bash
tq query --file queries.sql
```

### Parallel Processing

Run independent queries in parallel:

```bash
#!/bin/bash

# Export regions in parallel
tq query "SELECT * FROM data WHERE region = 'North'" -o north.csv &
tq query "SELECT * FROM data WHERE region = 'South'" -o south.csv &
tq query "SELECT * FROM data WHERE region = 'East'" -o east.csv &
tq query "SELECT * FROM data WHERE region = 'West'" -o west.csv &

# Wait for all to complete
wait

echo "All exports complete"
```

**Considerations:**
- Each process uses a separate connection
- Monitor database connection limits
- Teradata handles concurrent queries well

### Large Result Sets

Results are streamed, so large queries are memory-efficient:

```bash
# This works efficiently even for huge tables
tq query "SELECT * FROM huge_table" --format csv --output huge.csv
```

Rows written incrementally, not buffered in memory.

**Table format optimization:**

When using table format output (the default for terminal display), tq calculates column widths based on actual content rather than database schema types. This means:

- Columns are sized to fit their actual data, not the VARCHAR(N) or CHAR(N) schema width
- More columns fit on screen for wide tables
- Maximum column width is capped at 100 characters
- Works in both interactive (TTY) and batch (piped) contexts

Example:
```bash
# Schema: DatabaseName VARCHAR(64), but actual content ~15 chars
# tq shows compact columns, not 64-character-wide columns
tq query "SELECT * FROM DBC.Databases"
```

In batch mode (when piping to files or other commands), all columns are shown without terminal width limits, but columns are still sized efficiently based on content.

### Data Sampling

#### Quick Sampling with `tq sample`

tq provides a dedicated `sample` command for fast random data exploration:

```bash
# Sample 10 rows (default)
tq sample employees

# Sample custom row count
tq sample customers 50

# Sample from qualified table name
tq sample staging.test_data 20

# Export sample to CSV
tq sample huge_table 100 --format csv --output sample.csv

# Sample as JSON
tq sample products 25 --format json
```

**How `tq sample` works:**
- Uses Teradata's SAMPLE clause for efficient random sampling
- Default: 10 rows if count not specified
- Maximum: 1000 rows per sample
- Fast even on huge tables (no full table scan)
- Table names are resolved case-insensitively: `tq sample Employees` and `tq sample EMPLOYEES` are equivalent

**Common use cases:**
- Quick data inspection during development
- Validating ETL results
- Finding example values for testing
- Checking data quality

#### Table Structure and Data with `tq peek`

Get table metadata and sample data in one command:

```bash
# Peek at table (shows 5 rows + metadata by default)
tq peek products

# Peek with custom row count
tq peek customers 10

# Peek at qualified table name
tq peek development.orders 15

# Export peek results to JSON
tq peek employees 20 --format json --output peek.json
```

**What you get:**
- Table metadata (type, row count estimate)
- Column information (name, type, nullable, precision)
- First N rows of actual data (default: 5)

**Example output:**

```bash
$ tq peek products

Table: PRODUCTION.products
Type: Table
Approximate Rows: 15,432

Column Information:
┌─────────────┬──────────────┬──────────┬───────────┐
│ Column      │ Type         │ Nullable │ Precision │
├─────────────┼──────────────┼──────────┼───────────┤
│ product_id  │ INTEGER      │ NO       │ -         │
│ name        │ VARCHAR(100) │ NO       │ 100       │
│ category    │ VARCHAR(50)  │ YES      │ 50        │
│ price       │ DECIMAL(10,2)│ YES      │ 10,2      │
│ in_stock    │ INTEGER      │ YES      │ -         │
└─────────────┴──────────────┴──────────┴───────────┘

First 5 rows:
┌────────────┬─────────────────┬───────────┬─────────┬──────────┐
│ product_id │ name            │ category  │ price   │ in_stock │
├────────────┼─────────────────┼───────────┼─────────┼──────────┤
│ 1001       │ Laptop Pro      │ Computer  │ 1299.99 │ 45       │
│ 1002       │ Wireless Mouse  │ Computer  │ 29.99   │ 230      │
│ 1003       │ USB-C Cable     │ Computer  │ 12.99   │ 890      │
│ 1004       │ Desk Chair      │ Furniture │ 249.99  │ 12       │
│ 1005       │ Monitor 27"     │ Computer  │ 399.99  │ 67       │
└────────────┴─────────────────┴───────────┴─────────┴──────────┘
```

**When to use:**
- Understanding unfamiliar tables
- Combining structure and data inspection
- Quick validation of table contents
- Scripting data exploration workflows

**Note:** Table names are resolved case-insensitively: `tq peek Products` and `tq peek PRODUCTS` are equivalent.

#### Advanced: Using Teradata SAMPLE Clause in SQL

For more complex sampling scenarios, you can use Teradata's SAMPLE clause directly:

```bash
# Random 10% of rows
tq query "SELECT * FROM large_table SAMPLE 0.10"

# Sample with specific columns and filters
tq query "SELECT id, name, status FROM customers WHERE region='US' SAMPLE 50" --format csv
```

**SAMPLE clause features:**
- Can specify percentage (e.g., 0.10 for 10%)
- Works with WHERE, ORDER BY, and other clauses
- Combine with aggregations for statistical sampling

#### Object Inspection with `tq inspect`

Use `tq inspect` to get a comprehensive view of any database object — its type, column definitions, index structure, storage metrics, and view definitions. This is the batch equivalent of the REPL `/inspect` command.

```bash
# Full inspection of a table
tq inspect employees

# Inspect a table in another database
tq inspect production.orders

# JSON output for programmatic use
tq inspect --format json employees

# CSV output (column list is the tabular representation)
tq inspect --format csv employees > employees-schema.csv

# Write a full inspection report to a file
tq inspect --output employees-report.txt employees

# Using a connection profile
tq --profile prod inspect employees
```

**Example output — table:**

```
── Object Info ──
  Type:      Table
  Database:  PRODUCTION
  Name:      employees
  Created:   2023-04-15 09:12:33

── Columns (7) ──
  Column                   Type                 Nullable   Default
  ──────────────────────── ────────────────────  ──────── ───────────────
  employee_id              INTEGER              NO         -
  first_name               VARCHAR(50)          YES        -
  last_name                VARCHAR(50)          YES        -
  email                    VARCHAR(100)         YES        -
  hire_date                DATE                 YES        -
  salary                   DECIMAL(10,2)        YES        -
  department_id            INTEGER              YES        -
7 columns

── Indexes ──
  Primary Index (UPI): employee_id
  Secondary Index (NUSI) "idx_dept": department_id
  Secondary Index (USI) "idx_email": email

── Storage ──
  Current Size:  1.40 GB
  Peak Size:     1.80 GB
  Skew Factor:   8.2% (low)
  AMP Count:     32
```

**Example output — view:**

```
── Object Info ──
  Type:      View
  Database:  PRODUCTION
  Name:      active_employees_view
  Created:   2024-01-10 14:22:07

── Columns (4) ──
  Column                   Type                 Nullable   Default
  ──────────────────────── ────────────────────  ──────── ───────────────
  employee_id              INTEGER              NO         -
  first_name               VARCHAR(50)          YES        -
  last_name                VARCHAR(50)          YES        -
  department_id            INTEGER              YES        -
4 columns

── Definition ──
  REPLACE VIEW "PRODUCTION"."active_employees_view" AS
  SELECT employee_id, first_name, last_name, department_id
  FROM employees
  WHERE status = 'A'
```

**Options:**

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--format` | `-f` | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | stdout | Write output to file |

**Scripting examples:**

```bash
# Extract column list as CSV for documentation
tq inspect --format csv employees > employees-columns.csv

# Check table size across environments
for env in dev staging prod; do
  echo "=== $env ==="
  tq --profile $env inspect orders
done

# Audit all tables for skew (pipe JSON through jq)
tq inspect --format json orders | jq '.storage.skew_factor'
```

**When to use `tq inspect` vs `tq query`:**
- Use `tq inspect` when you want structured metadata about an object (schema, indexes, size)
- Use `tq query` when you want to execute ad-hoc SQL and return rows of data

**Cross-reference:** For interactive use, see `/inspect` in the REPL Guide.

#### Schema Commands: `tq describe`, `tq list`, `tq show-indexes`

Three focused schema commands give you fast, scriptable access to object structure without writing SQL. These are the batch equivalents of the `/describe`, `/list`, and `/show indexes` REPL metacommands.

---

##### `tq describe` — Table and View Structure

Show column definitions and index structure for any table or view:

```bash
# Describe a table in the current database
tq describe employees

# Describe using a qualified name
tq describe production.orders

# JSON output for scripting
tq describe --format json employees

# CSV output for documentation generation
tq describe --format csv production.orders > orders-schema.csv

# Write report to file
tq describe --output employees-schema.txt employees

# Using a connection profile
tq --profile prod describe employees
```

**Object names are case-insensitive:** `tq describe DBC.TABLES` and `tq describe dbc.tables` return the same result.

**Example output (table without column comments):**

```
── Object ──
  Type:      Table
  Database:  PRODUCTION
  Name:      employees
  Rows (Est.): 42573

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
  Secondary Index (NUSI) "idx_dept": department_id
  Secondary Index (USI) "idx_email": email
```

**Example output (table with column comments):**

```
── Object ──
  Type:      Table
  Database:  PRODUCTION
  Name:      orders
  Rows (Est.): 9876543

── Columns (4) ──
  Column                   Type                 Nullable   Default         Comment
  ------------------------------------------------------------------------------------------
  order_id                 INTEGER              NO         -               Primary key
  customer_id              INTEGER              NO         -               FK to customers
  order_date               DATE                 YES        -
  total_amount             DECIMAL(12,2)        YES        0
  4 column(s)

── Indexes ──
  Primary Index (UPI): order_id
```

**Example output (table with no indexes defined):**

```
── Object ──
  Type:      Table
  Database:  PRODUCTION
  Name:      staging_load

── Columns (3) ──
  Column                   Type                 Nullable   Default
  ----------------------------------------------------------------------
  batch_id                 INTEGER              NO         -
  load_ts                  TIMESTAMP            YES        -
  payload                  VARCHAR(8000)        YES        -
  3 column(s)

No indexes defined.
```

The `Rows (Est.)` line appears in the Object header only for tables (not views). It is omitted when row statistics have not been collected. The `Comment` column is included in the header only when at least one column has a comment string defined. The `Default` column shows `-` when no default is defined. The `Indexes` section is omitted for views (views have no indexes). Tables with no indexes defined show `No indexes defined.` instead of an indexes section.

**Options:**

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--format` | `-f` | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | stdout | Write output to file |

**Scripting examples:**

The JSON output has the structure `{"object":{...}, "columns":[...], "indexes":[...]}`. Each column entry has fields: `name`, `type`, `nullable` (boolean), `default` (`null` when no default is set, a string otherwise), and optionally `comment`.

```json
{
  "object": {"database": "PRODUCTION", "name": "employees", "type": "Table"},
  "columns": [
    {"name": "employee_id", "type": "INTEGER",      "nullable": false, "default": null},
    {"name": "first_name",  "type": "VARCHAR(50)",  "nullable": true,  "default": null},
    {"name": "salary",      "type": "DECIMAL(10,2)","nullable": true,  "default": null}
  ],
  "indexes": [
    {"name": null,        "type": "UPI",  "columns": ["employee_id"]},
    {"name": "idx_dept",  "type": "NUSI", "columns": ["department_id"]}
  ]
}
```

```bash
# List all column names
tq describe --format json employees | jq -r '.columns[].name'

# Find nullable columns (nullable is a boolean)
tq describe --format json employees | \
  jq -r '.columns[] | select(.nullable) | [.name, .type] | @csv'

# Check if a column exists before querying
tq describe --format json employees | \
  jq -e '.columns[] | select(.name == "salary")' > /dev/null && \
  echo "salary column exists"

# Get the primary index type
tq describe --format json employees | \
  jq -r '.indexes[] | select(.type == "UPI") | .columns[]'
```

**Cross-reference:** For interactive use, see `/describe` in the REPL Guide.

---

##### `tq list` — List Databases, Tables, and Views

List database objects accessible to the connected user:

```bash
# List all accessible databases
tq list databases

# List tables in the current (logon) database
tq list tables

# List tables matching a glob pattern (* and ? wildcards, case-insensitive)
tq list tables emp*

# List tables in a specific database
tq list tables --database staging

# List tables in a specific database with a pattern
tq list tables --database staging test_*

# List views
tq list views

# JSON output for scripting
tq list databases --format json

# CSV export to file
tq list tables --format csv --output tables.csv

# Using a connection profile
tq --profile prod list databases
```

**Shared options (all subcommands):**

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--format` | `-f` | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | stdout | Write output to file |
| `--database` | `-d` | (from logon string) | Target database for `tables` and `views` |

**`tq list databases` — example output:**

```
Databases (4):
Name                           Owner                Type
------------------------------------------------------------
analytics                      analytics            User
development                    dev_user             User
production                     dba_user             User
DBC                            DBC                  System

4 database(s)
```

Results are sorted alphabetically by name. The `Type` column shows `System` for DBC-owned databases and `User` for all others. Common internal system databases (Console, Crashdumps, SYSLIB, etc.) are excluded from the listing to reduce noise.

**`tq list tables` — example output:**

```
Tables in (current):
Name                           Type       Rows (Est.)       Size Owner
------------------------------------------------------------------------------
customers                      TABLE        1234567      45.2 MB alice
employees                      TABLE          42573       2.1 MB alice
orders                         TABLE        9876543     320.5 MB bob
fact_sales                     NoPI               -           -

4 table(s)
```

The `Type` column shows `TABLE` for standard tables and `NoPI` for tables with no primary index. The `Rows (Est.)` and `Size` columns show `-` when statistics have not been collected. Rows and Size values are right-aligned. The `Owner` column shows the creator name. Pattern filtering uses glob syntax: `*` matches any sequence, `?` matches a single character (case-insensitive).

**`tq list views` — example output:**

```
Views in (current):
Name                                Owner
--------------------------------------------------
active_employees                    alice
customer_orders_view                alice
sales_summary                       bob

3 view(s)
```

The view list shows names and their owner (creator). To see a view's columns or definition, use `tq describe <view>` or `tq inspect <view>`.

**Scripting examples:**

The JSON output for databases has the structure `[{"database":"...","owner":"...","type":"..."}]` (note: the key is `"database"`, not `"name"`). For tables: `[{"name":"...","type":"...","estimated_rows":<integer|null>,"size_bytes":<integer|null>,"owner":"..."}]`. For views: `[{"name":"...","owner":"..."}]`.

```bash
# Find all staging-related databases
tq list databases --format json | \
  jq -r '.[] | select(.database | test("staging";"i")) | .database'

# Count tables per database
for db in $(tq list databases --format json | jq -r '.[].database'); do
  count=$(tq list tables --database "$db" --format json | jq 'length')
  echo "$db: $count tables"
done

# Export full table inventory to CSV
tq --profile prod list tables --format csv --output tables-$(date +%Y%m%d).csv
```

**Cross-reference:** For interactive use, see `/list` in the REPL Guide.

---

##### `tq show-indexes` — Table Index Structure

Display the complete index structure for a table — primary index type and columns, plus all secondary indexes:

```bash
# Show indexes for a table in the current database
tq show-indexes employees

# Show indexes using a qualified name
tq show-indexes production.orders

# JSON output for scripting
tq show-indexes --format json employees

# CSV output for documentation
tq show-indexes --format csv production.orders > orders-indexes.csv

# Write to file
tq show-indexes --output indexes.txt employees

# Using a connection profile
tq --profile prod show-indexes employees
```

**Object names are case-insensitive.** This command applies only to tables — invoking it against a view shows an informative message and exits with code 0.

**Example output (table with primary and secondary indexes):**

```
Indexes on production.employees:

── Primary Index ──
  Primary Index (UPI): employee_id

── Secondary Indexes ──
  Secondary Index (NUSI) "idx_dept": department_id
  Secondary Index (USI) "idx_email": email

3 index(es), 3 index column(s)
```

**Example output (composite primary index + named secondary index):**

```
Indexes on production.orders:

── Primary Index ──
  Primary Index (NUPI) "pk_orders": order_id

── Secondary Indexes ──
  Secondary Index (NUSI) "idx_customer": customer_id, order_date

2 index(es), 3 index column(s)
```

**Example output (primary index only, no secondary indexes):**

```
Indexes on production.config:

── Primary Index ──
  Primary Index (NUPI): config_key

No secondary indexes.

1 index(es), 1 index column(s)
```

**Example output (NoPI table — no primary index, with secondary indexes):**

```
Indexes on production.fact_sales:

No Primary Index (NoPI)

── Secondary Indexes ──
  Secondary Index (NUSI) "idx_region": region_code

1 index(es), 1 index column(s)
```

When an index has no name in the catalog, the `"name"` segment is omitted and the format is `  <type> (<short>): <columns>`. Named indexes appear as `  <type> (<short>) "<name>": <columns>`. A table with no primary index shows `No Primary Index (NoPI)`. A table with no secondary indexes shows `No secondary indexes.`

**Options:**

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--format` | `-f` | `table` | Output format: `table`, `json`, `csv` |
| `--output` | `-o` | stdout | Write output to file |

**Scripting examples:**

```bash
# Check primary index type (UPI vs NUPI)
tq show-indexes --format json employees | \
  jq -r '.primary_index.type'

# Find all USI columns on a table
tq show-indexes --format json employees | \
  jq -r '.secondary_indexes[] | select(.type == "USI") | .columns[]'

# Audit tables for non-unique primary indexes (potential skew risk)
for tbl in customers orders products; do
  pi_type=$(tq show-indexes --format json "$tbl" | jq -r '.primary_index.type')
  echo "$tbl: $pi_type"
done
```

**When to use `tq show-indexes` vs `tq inspect`:**
- Use `tq show-indexes` for focused index auditing and scripting (clean, single-purpose output)
- Use `tq inspect` when you also want columns, storage metrics, and dependencies in one view

**Cross-reference:** For interactive use, see `/show indexes` in the REPL Guide.

---

##### Schema Command Comparison

| Command | Shows | Use When |
|---------|-------|----------|
| `tq describe <table>` | Columns + indexes | You need column definitions |
| `tq list tables` | Table inventory + sizes | You need a database inventory |
| `tq show-indexes <table>` | Index structure only | You are auditing indexes |
| `tq inspect <object>` | Everything (type, columns, indexes, storage, deps) | You want the full picture |

---

## Common Recipes

### Daily Report Generation

```bash
#!/bin/bash
# daily_report.sh

DATE=$(date +%Y-%m-%d)
REPORT_FILE="report_${DATE}.csv"

tq query --file reports/daily_summary.sql \
  --format csv \
  --output "$REPORT_FILE"

if [ $? -eq 0 ]; then
  echo "Report generated: $REPORT_FILE"
  # Email or upload report
  mail -s "Daily Report" admin@example.com < "$REPORT_FILE"
else
  echo "Report generation failed" >&2
  exit 1
fi
```

### Database Migration

```bash
#!/bin/bash
# migrate.sh

set -e  # Stop on any error

echo "Starting migration..."

# Run migration with transaction protection
tq query --file migrations/v1_to_v2.sql --atomic --verbose

# Update version
tq query "INSERT INTO schema_version VALUES (2, CURRENT_TIMESTAMP)"

echo "Migration completed successfully"
```

### Data Export Pipeline

```bash
#!/bin/bash
# export_pipeline.sh

# Export data
tq query "SELECT * FROM source_table" \
  --format csv \
  --output raw_data.csv

# Transform with Python
python transform.py raw_data.csv > transformed.csv

# Load back
tq query --file load_transformed.sql

echo "ETL pipeline complete"
```

### JSON Processing with jq

```bash
#!/bin/bash

# Extract specific fields
tq query --format json "SELECT * FROM users SAMPLE 10" | \
  jq -r '.[] | [.id, .name, .email] | @csv' > users.csv

# Filter and count
active_count=$(
  tq query --format json "SELECT * FROM users" | \
  jq '[.[] | select(.active == true)] | length'
)
echo "Active users: $active_count"

# Complex transformation
tq query --format json "SELECT * FROM employees" | \
  jq 'group_by(.department) | map({department: .[0].department, count: length})'
```

### Backup and Restore

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="backups/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

# Backup all tables
for table in users orders products; do
  echo "Backing up $table..."
  tq query "SELECT * FROM ${table}" \
    --format csv \
    --output "${BACKUP_DIR}/${table}.csv"
done

echo "Backup completed: $BACKUP_DIR"
```

### Automated Testing

```bash
#!/bin/bash
# test_data.sh

# Setup test data
tq query --file tests/setup.sql --atomic

# Run tests
test_count=$(tq query "SELECT COUNT(*) FROM test_results WHERE status = 'pass'" --format csv --no-header)
total_count=$(tq query "SELECT COUNT(*) FROM test_results" --format csv --no-header)

echo "Tests passed: $test_count / $total_count"

# Cleanup
tq query --file tests/cleanup.sql

# Exit with error if any test failed
if [ "$test_count" -ne "$total_count" ]; then
  exit 1
fi
```

### Monitoring and Alerting

```bash
#!/bin/bash
# monitor.sh

THRESHOLD=1000

# Check table size
row_count=$(tq query "SELECT COUNT(*) FROM critical_table" --format csv --no-header)

if [ "$row_count" -lt "$THRESHOLD" ]; then
  echo "ALERT: critical_table has only $row_count rows (threshold: $THRESHOLD)" >&2
  # Send alert
  echo "Table critically low" | mail -s "Database Alert" admin@example.com
  exit 1
fi

echo "Table size OK: $row_count rows"
```

---

## Teradata Session Types and Transaction Support

Teradata supports different session types with varying transaction capabilities. Understanding these modes helps avoid errors when using transactional features.

### Session Modes

**ANSI Mode (Recommended for Transactions)**
- Explicit transaction control supported
- Uses standard SQL syntax: BEGIN TRANSACTION, COMMIT, ROLLBACK
- Statements do not auto-commit by default
- Best for migration scripts and multi-step operations
- Set by DBA or connection parameter

**Teradata Mode (Default)**
- Implicit commit for most statements
- Uses Teradata-specific syntax: BT (Begin Transaction), ET (End Transaction)
- Some DDL operations cannot be in transactions
- Common in legacy systems

**BTEQ Mode**
- Legacy batch mode with limited transaction control
- Not recommended for new applications

### How Session Mode Affects --atomic Flag

The `--atomic` flag behavior depends on your session mode:

**ANSI Mode:**
```bash
$ tq query --file migration.sql --atomic
[Uses BEGIN TRANSACTION/COMMIT/ROLLBACK]
Statement 1: UPDATE - OK
Statement 2: INSERT - OK
Transaction committed
```

**Teradata Mode:**
```bash
$ tq query --file migration.sql --atomic
[Uses BT/ET commands]
Statement 1: UPDATE - OK
Statement 2: INSERT - OK
Transaction committed
```

The tool detects your session mode automatically and uses appropriate syntax.

### DDL Transaction Limitations

Some Teradata DDL statements cannot execute within transactions:

```bash
$ cat ddl_script.sql
CREATE TABLE test (id INT);
DROP TABLE old_table;
ALTER TABLE users ADD COLUMN status VARCHAR(20);

$ tq query --file ddl_script.sql --atomic
Error: This statement type cannot be executed within a transaction

Statement 1: CREATE TABLE - FAILED
Reason: CREATE TABLE not allowed in transaction (Teradata mode)

Transaction rolled back

Suggestions:
  - Remove --atomic flag for DDL operations
  - Execute DDL statements separately
  - Use ANSI mode for better transaction support (contact DBA)
```

**Solution:** Don't use `--atomic` for scripts with DDL statements.

```bash
# Execute DDL without transactions
$ tq query --file ddl_script.sql
Statement 1: CREATE TABLE - OK
Statement 2: DROP TABLE - OK
Statement 3: ALTER TABLE - OK
```

### Checking Your Session Mode

To see your current session mode:

```bash
$ tq query "SELECT SessionMode FROM DBC.SessionInfoV WHERE SessionNo = SESSION"
```

**Output:**
```
┌─────────────┐
│ SessionMode │
├─────────────┤
│ ANSI        │
└─────────────┘
```

### Best Practices

1. **Use ANSI mode for transactional workloads** - More reliable transaction support
2. **Avoid --atomic with DDL scripts** - DDL statements often cannot be in transactions
3. **Test in development first** - Session mode behavior can vary by environment
4. **Separate DDL and DML** - Run schema changes separately from data changes
5. **Contact your DBA** - They can change default session mode if needed

### Error Messages

If you encounter transaction errors, the tool provides guidance:

```bash
Error: Transaction control not supported

This session type does not support explicit transactions.
Session mode: BTEQ
Operation attempted: BEGIN TRANSACTION

Suggestions:
  - Reconnect in ANSI or Teradata mode for transaction support
  - Execute statements without --atomic flag
  - Contact DBA to change default session mode
```

---

## Best Practices

1. **Use transactions for critical operations** - Wrap multi-step changes with `--atomic`
2. **Parameterize SQL templates** - Use `-p params.yaml` instead of shell string substitution for maintainable scripts
3. **Separate environments with parameter files** - Use `base.yaml` + `prod.yaml` overlays for multi-environment deployments
4. **Check exit codes** - Always verify success in scripts
5. **Separate data and errors** - Use proper stream redirection
6. **Minimize connections** - Batch queries in files when possible
7. **Use explicit output** - Prefer `--output` over `>` for safety
8. **Log everything** - Keep audit trails of batch operations
9. **Test with small samples** - Use SAMPLE clause during development
10. **Handle interrupts** - Clean up resources on Ctrl-C
11. **Version control scripts and params** - Track both SQL files and YAML parameter files in git
12. **Document assumptions** - Add comments to complex scripts

---

## Troubleshooting

### Query fails but exit code is 0
- Check that you're capturing the correct exit code: `$?` immediately after command
- Verify no other commands run between tq and exit code check

### File output is empty
- Check that query returns results
- Verify file permissions and disk space
- Look for errors in stderr

### Transaction rollback but no error
- Some Teradata modes don't support transactions for DDL
- Remove `--atomic` for DDL-heavy scripts

### Semicolon splitting issues
- Avoid `;` inside string literals
- Use single-statement execution for complex cases
- Consider manual transaction control

### Performance is slow
- Minimize number of tq invocations
- Use parallel execution for independent queries
- Check Teradata query performance (not a tq issue)

---

## See Also

- [REPL Guide](repl-guide.md) - Interactive mode with `/params` command for live parameter management
- `tq help params` - Full variable substitution syntax reference
- [Specifications](../specifications/batch-mode.md) - Detailed requirements
