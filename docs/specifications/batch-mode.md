# Batch Mode Specifications

## Table of Contents

1. [Overview](#overview)
2. [Execution Modes](#execution-modes)
3. [Multiple Statement Execution](#multiple-statement-execution)
4. [Output Destinations](#output-destinations)
5. [Error Handling](#error-handling)
6. [Scripting Integration](#scripting-integration)
7. [Performance Considerations](#performance-considerations)
8. [Transaction Control](#transaction-control)
9. [Variable Substitution](#variable-substitution)

---

## Overview

Batch mode is designed for non-interactive use: scripts, cron jobs, CI/CD pipelines, and command-line data processing.

**Core Features:**
- stdin input
- File input via `--file` flag
- Multiple statement execution
- Fail-fast error handling
- Output format control (table, JSON, CSV)

## Execution Modes

tq supports three mutually exclusive input sources for SQL queries. The tool automatically detects the source and executes accordingly.

### Input Source Precedence

When determining which input source to use:
1. **Explicit query argument** (highest priority)
2. **File flag** (`--file`)
3. **stdin** (lowest priority)

**Mutual Exclusivity:**
- Only ONE input source can be used per invocation
- Providing multiple sources results in an error
- This prevents ambiguity and accidental data loss

### Inline Query (Argument)

**Use case:** Quick ad-hoc queries, simple one-liners

```bash
tq query "SELECT COUNT(*) FROM users"
tq query "SELECT * FROM employees WHERE salary > 50000 SAMPLE 10"
```

**Characteristics:**
- Most explicit and clear
- Best for short queries
- Shell escaping rules apply (quote the SQL)
- Single statement only (no semicolon splitting)

### File Input

**Use case:** Saved queries, migration scripts, complex multi-statement operations

```bash
tq query --file script.sql
tq query --file /path/to/migrations/v2.sql --format json
```

**File format:**
```sql
-- SQL comments are supported (both styles)
/* Multi-line
   comments work too */

-- Multiple statements separated by semicolons
SELECT * FROM table1 SAMPLE 5;

INSERT INTO table2 SELECT * FROM table1;

UPDATE table2 SET status = 'processed';

-- Final statement
DROP TABLE temp_data;
```

**Characteristics:**
- Supports multi-statement execution (see section on Multiple Statement Execution)
- Statements separated by semicolons
- SQL comments (`--` and `/* */`) are preserved and handled by Teradata
- Path can be absolute or relative to current directory
- File extension doesn't matter (.sql, .txt, or no extension all work)

**Error handling:**
```bash
$ tq query --file nonexistent.sql
Error: File not found

Could not read file: nonexistent.sql
Reason: No such file or directory

Check:
  - File path is correct (relative or absolute)
  - File exists and is readable
  - Current directory: /Users/user/project
```

### stdin Input

**Use case:** Pipeline integration, shell scripts, heredocs

```bash
# Pipe from file
cat query.sql | tq query

# Pipe from command
echo "SELECT 1" | tq query

# Heredoc (great for scripts)
tq query <<EOF
SELECT employee_id, salary
FROM employees
WHERE salary > 50000
ORDER BY salary DESC;
EOF

# Process substitution
tq query < <(echo "SELECT CURRENT_DATE")
```

**Characteristics:**
- Automatically detected (stdin is not a TTY)
- Supports multi-statement execution
- Ideal for shell scripts and pipelines
- Works with pipes, redirects, and heredocs

**Error handling:**
```bash
# Empty stdin
$ echo "" | tq query
Error: Empty query

Provide SQL via argument, file, or stdin.

# No input and TTY (interactive terminal)
$ tq query
Error: No query provided

Use 'tq query "SELECT ..."' or pipe SQL via stdin.

# Multiple sources (conflict)
$ echo "SELECT 1" | tq query "SELECT 2"
Error: Multiple input sources provided

You specified both a query argument and piped stdin.
Only one input source is allowed.
```

## Multiple Statement Execution

Files and stdin input support executing multiple SQL statements in sequence. This is essential for migrations, setup scripts, and complex data operations.

### Statement Parsing

**How it works:**
- SQL input is split on semicolon (`;`) characters
- Empty statements (whitespace-only) are skipped
- Statements execute sequentially in order
- Each statement is trimmed of leading/trailing whitespace

**Example file:**
```sql
-- setup.sql
CREATE TABLE temp_data (id INT, value VARCHAR(100));

INSERT INTO temp_data VALUES (1, 'test');
INSERT INTO temp_data VALUES (2, 'test2');

SELECT * FROM temp_data;

DROP TABLE temp_data;
```

**Execution:**
```bash
$ tq query --file setup.sql
Statement 1: CREATE TABLE - OK
Statement 2: INSERT - OK (1 row affected)
Statement 3: INSERT - OK (1 row affected)
Statement 4: SELECT - 2 rows returned
┌────┬─────────┐
│ id │ value   │
├────┼─────────┤
│ 1  │ test    │
│ 2  │ test2   │
└────┴─────────┘
Statement 5: DROP TABLE - OK

All statements executed successfully
```

### Execution Behavior

**Sequential execution:**
- Statements execute in file order (top to bottom)
- Each statement commits independently (no automatic transaction)
- Results displayed for queries (SELECT), status for DDL/DML
- Fail-fast: stop on first error (see Error Handling section)

**Statement numbering:**
- Displayed in output messages
- Starts at 1 (user-friendly, not 0-indexed)
- Matches line/position in source file

### Known Limitations

**Semicolon in strings:**
Simple semicolon splitting doesn't handle `;` inside quoted strings:
```sql
-- This will split incorrectly:
INSERT INTO messages VALUES ('Hello; World');  -- Splits at `;` inside string
```

**Mitigation:**
- Most real-world SQL doesn't have `;` in strings
- Document this limitation
- Use single-statement execution for complex cases

**Complex SQL:**
- Stored procedure definitions with semicolons may split incorrectly
- Use single-statement execution for complex DDL

### Inline Queries (Single Statement Only)

Inline query arguments do NOT support multiple statements:
```bash
# This executes as ONE statement (semicolon is part of SQL)
tq query "SELECT 1; SELECT 2"
# Result: Teradata error (multiple statements not allowed in single execute)

# Use file or stdin for multiple statements instead
echo "SELECT 1; SELECT 2" | tq query
```

## Output Destinations

### stdout (Default)

By default, query results go to stdout. This enables UNIX-style composition with pipes and redirection.

```bash
# Redirect to file
tq query "SELECT * FROM users" > users.csv

# Pipe to another tool
tq query --format json "SELECT * FROM data" | jq '.[] | select(.active)'

# Combine with other commands
tq query "SELECT email FROM users" | sort | uniq > unique_emails.txt
```

### File Output (--output flag)

Explicit file output with status reporting:

```bash
tq query "SELECT * FROM users" --format csv --output users.csv
# Output: Wrote 1523 rows to users.csv
```

**Compared to shell redirection:**
- `--output`: Shows status message ("Wrote N rows to file")
- Shell redirect (`>`): Silent, UNIX-style
- Both work identically otherwise

**When to use each:**
- Scripts: Use redirect (`>`) - simpler, more standard
- Interactive: Use `--output` - provides confirmation
- Quiet mode: Either works (`--quiet` suppresses status message)

### Error Handling

**Important:** Errors always go to stderr, never stdout.

This ensures data pipelines don't get corrupted with error messages:

```bash
# Errors to file, data to stdout
tq query "SELECT * FROM users" 2> errors.log

# Data to file, errors to file
tq query "SELECT * FROM users" > data.csv 2> errors.log

# Suppress errors (not recommended)
tq query "SELECT * FROM users" 2>/dev/null

# Separate data and errors in script
tq query --file script.sql > output.json 2> script_errors.log
if [ $? -ne 0 ]; then
  echo "Query failed, see script_errors.log"
  exit 1
fi
```

## Error Handling

### Fail-Fast Behavior (Default)

When executing multiple statements, tq stops on the **first error**:

```bash
$ cat script.sql
SELECT 1;           -- Statement 1: succeeds
SELECT 2;           -- Statement 2: succeeds
INVALID SQL;        -- Statement 3: fails HERE
SELECT 3;           -- Statement 4: never executes
SELECT 4;           -- Statement 5: never executes

$ tq query --file script.sql
Statement 1: SELECT - 1 row returned
Statement 2: SELECT - 1 row returned
Error: SQL syntax error in statement 3

Expected something like a 'SELECT' keyword but found 'INVALID'.

Error Code: 3706
Session ID: 1429

Failed statement:
  INVALID SQL;

Statements executed: 1-2
Statements failed: 3 (stopped here)
Statements remaining: 4-5

Exit code: 1
```

### Error Context (Default Mode)

Default error messages include:
- Statement number where error occurred
- Error type (syntax error, permission denied, etc.)
- Teradata error code and session ID
- Failed statement preview (up to 80 characters)
- Count of executed vs remaining statements

Example:
```bash
$ tq query --file migrations/v2.sql
Error: Permission denied in statement 2

User 'alice' does not have DROP privilege on table 'important_data'.

Error Code: 3523
Session ID: 1429

Failed statement:
  DROP TABLE important_data;

Statements executed: 1
Statements remaining: 3-5
```

### Verbose Error Context (-v)

With `--verbose`, errors include additional context:

```bash
$ tq -v query --file script.sql
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

Error Code: 3706
Session ID: 1429

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

### Exit Codes

tq follows standard UNIX exit code conventions:

| Exit Code | Meaning | Example |
|-----------|---------|---------|
| `0` | Success | All statements executed successfully |
| `1` | Runtime error | SQL error, connection failure, permission denied |
| `2` | Usage error | Invalid arguments, missing required flag |
| `130` | Interrupted | User pressed Ctrl-C |

**In shell scripts:**
```bash
#!/bin/bash
set -e  # Exit on any error

# Run migration
tq query --file migrations/v3.sql

# Only runs if migration succeeds
echo "Migration completed successfully"

# Or with explicit checking
if tq query --file migrations/v3.sql; then
  echo "Migration succeeded"
  tq query "INSERT INTO schema_version VALUES (3, CURRENT_TIMESTAMP)"
else
  echo "Migration failed with exit code $?" >&2
  exit 1
fi
```

## Scripting Integration

### Exit Code Checking

```bash
#!/bin/bash

# Simple check
if tq ping; then
  echo "Database is up"
  tq query "SELECT COUNT(*) FROM active_users" --format json | process.py
else
  echo "Database is down" >&2
  exit 1
fi

# set -e: stop on any error
set -e
tq query --file migrations/v2.sql
tq query --file migrations/v3.sql
echo "All migrations completed"

# Explicit error handling
if ! tq query --file critical_update.sql; then
  echo "Critical update failed!" >&2
  tq query "INSERT INTO error_log VALUES ('deploy_failed', CURRENT_TIMESTAMP)"
  exit 1
fi
```

### JSON Processing with jq

```bash
# Filter and transform
tq query --format json "SELECT id, name, email FROM users" | \
  jq '.[] | select(.name | startswith("A"))' | \
  jq -r '.email'

# Extract specific fields
tq query --format json "SELECT * FROM employees SAMPLE 10" | \
  jq -r '.[] | [.employee_id, .name, .salary] | @csv'

# Count results
count=$(tq query --format json "SELECT * FROM active_users" | jq 'length')
echo "Active users: $count"
```

### CSV Processing

```bash
# Extract specific columns (using cut)
tq query --format csv "SELECT * FROM sales" | \
  cut -d',' -f1,3,5 > filtered.csv

# Count rows (subtract 1 for header)
row_count=$(tq query --format csv "SELECT * FROM employees" | wc -l)
data_rows=$((row_count - 1))
echo "Data rows: $data_rows"

# Convert to TSV
tq query --format csv "SELECT * FROM data" | \
  tr ',' '\t' > data.tsv

# Remove header
tq query --format csv --no-header "SELECT * FROM users" > headerless.csv

# Sort by column
tq query --format csv "SELECT name, salary FROM employees" | \
  (head -1; tail -n +2 | sort -t',' -k2 -rn) > sorted_by_salary.csv
```

### Pipeline Composition

```bash
# Multi-stage pipeline
tq query "SELECT email FROM users WHERE active = 1" | \
  sort | \
  uniq | \
  wc -l

# Process and load
tq query --format csv "SELECT * FROM source_table" | \
  python transform.py | \
  tq query --file - --format csv --output transformed.csv

# Conditional execution
tq ping && tq query --file daily_report.sql > report.txt || \
  echo "Database unavailable" | mail -s "Report Failed" admin@example.com

# Data validation
tq query "SELECT COUNT(*) FROM critical_table" | \
  grep -q "^0$" && echo "ERROR: Table is empty!" >&2 && exit 1
```

## Performance Considerations

### File Reading

- Reads entire file into memory before execution
- Simple and reliable
- Works well for files up to ~100MB
- Single `std::fs::read_to_string()` call

**Limitations:**
- Very large files (>100MB) may exhaust memory
- No streaming file reading

### Result Streaming

Results are already streamed to stdout as rows are fetched from Teradata.

```bash
# This works efficiently even for large result sets
tq query --format csv "SELECT * FROM huge_table" > huge.csv
```

No buffering of result set in memory - rows are written incrementally.

### Connection Overhead

**One-shot execution model:**
- Each `tq` invocation: connect → execute → disconnect
- No connection pooling or reuse
- ~100-200ms connection overhead per invocation

**Optimization for multiple queries:**
```bash
# Inefficient: 5 separate connections
tq query "SELECT 1"
tq query "SELECT 2"
tq query "SELECT 3"
tq query "SELECT 4"
tq query "SELECT 5"

# Efficient: 1 connection, 5 statements
cat > queries.sql <<EOF
SELECT 1;
SELECT 2;
SELECT 3;
SELECT 4;
SELECT 5;
EOF
tq query --file queries.sql
```

### Parallel Processing

Multiple `tq` processes can run in parallel:

```bash
# Parallel exports (each with own connection)
tq query "SELECT * FROM data WHERE region = 'North'" > north.csv &
tq query "SELECT * FROM data WHERE region = 'South'" > south.csv &
tq query "SELECT * FROM data WHERE region = 'East'" > east.csv &
tq query "SELECT * FROM data WHERE region = 'West'" > west.csv &
wait
echo "All exports complete"

# Combine results
cat north.csv south.csv east.csv west.csv > all_regions.csv
```

**Considerations:**
- Each parallel `tq` process uses a separate database connection
- Monitor database connection limits
- Teradata handles concurrent queries well

## Transaction Control

### Manual Transaction Control (Current Approach)

Users can manually wrap statements in transactions:
```sql
-- migration.sql
BEGIN TRANSACTION;

CREATE TABLE new_table (...);
INSERT INTO new_table SELECT * FROM old_table;
DROP TABLE old_table;

COMMIT;
```

If any statement fails, the transaction is automatically rolled back by Teradata.

## Variable Substitution

### Current Workaround: Shell Substitution

Use shell features for variable substitution:

```bash
# Method 1: envsubst
export TABLE_NAME=employees
envsubst < template.sql | tq query

# Method 2: heredoc with variable expansion
TABLE="employees"
tq query <<EOF
SELECT * FROM ${TABLE} SAMPLE 10;
EOF

# Method 3: sed
TABLE="employees"
sed "s/{{TABLE}}/${TABLE}/g" template.sql | tq query
```

---
