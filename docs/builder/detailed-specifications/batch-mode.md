# Batch Mode Specifications

**Version:** 1.1.0
**Last Updated:** 2026-01-18
**Owner:** cli-ux-designer agent
**Status:** Active Specification

---

## Table of Contents

1. [Overview](#61-overview)
2. [Execution Modes](#62-execution-modes)
3. [Output Destinations](#63-output-destinations)
4. [Scripting Integration](#64-scripting-integration)
5. [Performance Considerations](#65-performance-considerations)
6. [Transaction Control](#66-transaction-control-future)
7. [Variable Substitution](#67-variable-substitution-future)

---

## 6.1 Overview

Batch mode is designed for non-interactive use: scripts, cron jobs, CI/CD pipelines, and command-line data processing.

## 6.2 Execution Modes

### 6.2.1 Inline Query

```bash
tq query "SELECT COUNT(*) FROM users"
```

### 6.2.2 File Input

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

### 6.2.3 stdin Input

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

## 6.3 Output Destinations

### 6.3.1 stdout (Default)

```bash
tq query "SELECT * FROM users" > users.csv
```

### 6.3.2 File Output

```bash
tq query "SELECT * FROM users" --output users.csv
```

### 6.3.3 Error Handling

Errors always go to stderr:
```bash
tq query "INVALID SQL" 2> errors.log
tq query "SELECT * FROM users" > data.csv 2> errors.log
```

## 6.4 Scripting Integration

### 6.4.1 Exit Code Checking

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

### 6.4.2 JSON Processing with jq

```bash
tq query --format json "SELECT id, name, email FROM users" | \
  jq '.[] | select(.name | startswith("A"))' | \
  jq -r '.email'
```

### 6.4.3 CSV Processing

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

## 6.5 Performance Considerations

### 6.5.1 Streaming Results

For large datasets, stream results instead of buffering:

```bash
# Stream 10M rows without exhausting memory
tq query --format csv "SELECT * FROM huge_table" > huge.csv
```

**Implementation**: Write rows incrementally to stdout as they're fetched.

### 6.5.2 Parallel Processing

```bash
# Split large export into chunks
tq query "SELECT * FROM data WHERE date = '2024-01-01'" &
tq query "SELECT * FROM data WHERE date = '2024-01-02'" &
tq query "SELECT * FROM data WHERE date = '2024-01-03'" &
wait
```

### 6.5.3 Connection Pooling

Batch mode uses one-shot connections (connect → query → disconnect). No connection pooling needed.

## 6.6 Transaction Control (Future)

```bash
# Atomic script execution
tq query --file migration.sql --atomic

# Equivalent to:
BEGIN TRANSACTION;
[Execute all statements in file]
COMMIT; -- or ROLLBACK on error
```

## 6.7 Variable Substitution (Future)

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
