# Batch Mode Guide

This guide covers using tq for non-interactive batch operations: scripts, automation, CI/CD pipelines, and data processing workflows.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Input Methods](#input-methods)
3. [Output to File](#output-to-file)
4. [Transaction Control](#transaction-control)
5. [Multi-Statement Scripts](#multi-statement-scripts)
6. [Error Handling](#error-handling)
7. [Scripting Patterns](#scripting-patterns)
8. [Performance Tips](#performance-tips)
9. [Common Recipes](#common-recipes)

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

Use shell features for dynamic SQL:

```bash
#!/bin/bash

# Method 1: heredoc with variable expansion
TABLE="employees"
DATE="2024-01-01"

tq query <<EOF
SELECT *
FROM ${TABLE}
WHERE hire_date > '${DATE}'
SAMPLE 10;
EOF

# Method 2: envsubst
export TABLE_NAME=employees
export MIN_SALARY=50000

cat template.sql | envsubst | tq query

# Method 3: sed replacement
sed "s/{{TABLE}}/${TABLE}/g" template.sql | tq query
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

### Sampling

Use Teradata's SAMPLE clause for large tables:

```bash
# Random 1000 rows
tq query "SELECT * FROM large_table SAMPLE 1000"

# Random 10% of rows
tq query "SELECT * FROM large_table SAMPLE 0.10"
```

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
2. **Check exit codes** - Always verify success in scripts
3. **Separate data and errors** - Use proper stream redirection
4. **Minimize connections** - Batch queries in files when possible
5. **Use explicit output** - Prefer `--output` over `>` for safety
6. **Log everything** - Keep audit trails of batch operations
7. **Test with small samples** - Use SAMPLE clause during development
8. **Handle interrupts** - Clean up resources on Ctrl-C
9. **Version control scripts** - Track SQL files in git
10. **Document assumptions** - Add comments to complex scripts

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

- [REPL Guide](repl-guide.md) - Interactive mode documentation
- [Specifications](../specifications/batch-mode.md) - Detailed requirements
- [Examples](https://github.com/example/tq-examples) - More script examples
