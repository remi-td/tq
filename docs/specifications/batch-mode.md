# Batch Mode Specifications

## Table of Contents

1. [Overview](#overview)
2. [Execution Modes](#execution-modes)
3. [Multiple Statement Execution](#multiple-statement-execution)
4. [SQL File Parser Requirements](#sql-file-parser-requirements)
5. [Output Destinations](#output-destinations)
6. [Error Handling](#error-handling)
7. [Scripting Integration](#scripting-integration)
8. [Performance Considerations](#performance-considerations)
9. [Pagination](#pagination)
10. [Transaction Control](#transaction-control)
11. [Variable Substitution](#variable-substitution)

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
- SQL comments (`--` and `/* */`) are recognised and stripped before execution (see REQ-PARSE-008 through REQ-PARSE-013)
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
- SQL input is split on semicolon (`;`) characters that are outside quoted strings and comments
- Empty statements (whitespace-only) are skipped
- Statements execute sequentially in order
- Each statement is trimmed of leading/trailing whitespace
- Multi-line statements are supported; newlines are treated as whitespace within a statement
- Comments (`--` and `/* */`) are recognised and do not interfere with statement boundaries

See [SQL File Parser Requirements](#sql-file-parser-requirements) for the precise parsing rules.

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

### Inline Queries (Single Statement Only)

Inline query arguments do NOT support multiple statements:
```bash
# This executes as ONE statement (semicolon is part of SQL)
tq query "SELECT 1; SELECT 2"
# Result: Teradata error (multiple statements not allowed in single execute)

# Use file or stdin for multiple statements instead
echo "SELECT 1; SELECT 2" | tq query
```

## SQL File Parser Requirements

This section defines the precise rules the parser must follow when splitting a SQL file or stdin input into individual statements. These rules ensure that structural SQL syntax (quoted strings, comments, multi-line statements) is handled correctly and that error reporting remains accurate.

The parser operates as a linear, single-pass state machine over the input character stream. It tracks the current lexical context to determine whether a semicolon terminates a statement or is part of a literal or comment.

---

### Statement Boundary Rules

**REQ-PARSE-001: Semicolon as Terminator**

A semicolon (`;`) terminates the current statement if and only if it appears outside of a single-quoted string literal, a line comment, or a block comment. Semicolons in any other context are part of the statement text and must not split it.

Compliant examples:

```sql
-- Valid terminators (outside any context)
SELECT 1;
INSERT INTO t VALUES (42);

-- Semicolons NOT terminators (inside quoted string)
INSERT INTO messages VALUES ('Hello; World');

-- Semicolons NOT terminators (inside line comment)
SELECT 1 -- use ';' to end; ok
;

-- Semicolons NOT terminators (inside block comment)
SELECT /* a;b;c */ 1;
```

**REQ-PARSE-002: Trailing Content After Final Terminator**

If the input ends with a non-empty, non-whitespace sequence that is not followed by a semicolon, that trailing content is treated as an implicit final statement. This allows files that omit the trailing semicolon on the last statement to execute correctly.

```sql
-- Both of these must produce exactly one statement:
SELECT 1;
SELECT 1
```

**REQ-PARSE-003: Empty Statement Suppression**

Statements that are empty or contain only whitespace and comments after trimming are silently discarded. They are not counted in statement numbering, not sent to the database, and do not appear in progress output.

```sql
-- These produce zero executable statements:
;
   ;
-- just a comment
;
/* block comment only */;
```

**REQ-PARSE-004: Whitespace and Newlines Within Statements**

Newlines, carriage returns, tabs, and spaces within a statement are preserved as-is and passed to the database. The parser never collapses or strips internal whitespace. A statement boundary is formed by a terminating semicolon or end-of-input, not by blank lines.

```sql
-- This is ONE statement:
SELECT
    employee_id,
    first_name,
    last_name
FROM employees
WHERE department_id = 10
ORDER BY last_name;
```

---

### Quoted String Rules

**REQ-PARSE-005: Single-Quoted String Recognition**

The parser enters "quoted string" context on encountering a single-quote character (`'`) that is outside any existing string or comment context. It exits that context on the next closing single quote that is not part of an escaped quote sequence.

While in quoted string context:
- Semicolons are not statement terminators.
- Comment-opening sequences (`--` and `/*`) are not recognised as comments.
- The string content is passed through verbatim (no unescaping by the parser).

**REQ-PARSE-006: Escaped Single Quotes Inside Strings**

A doubled single-quote (`''`) inside a quoted string is the standard SQL escape for a literal single-quote character. The parser must treat `''` as a single escaped character and remain in quoted string context; it does not end the string.

```sql
-- ONE statement; the string contains an apostrophe
INSERT INTO greetings VALUES ('it''s fine');

-- ONE statement; semicolon inside string does not split
INSERT INTO notes VALUES ('step 1; step 2; step 3');

-- ONE statement; multiple escapes
SELECT 'can''t stop; won''t stop' AS phrase;
```

**REQ-PARSE-007: Unterminated Quoted String Error**

If end-of-input is reached while still inside a quoted string context, the parser must report an error identifying the line number where the unterminated string began and the statement number in which it appeared.

```
Error: Unterminated string literal

  File:      script.sql
  Statement: 3
  Line:      12

The single-quoted string opened on line 12 was never closed.
Check for a missing closing quote.
```

---

### Line Comment Rules

**REQ-PARSE-008: Line Comment Recognition**

The two-character sequence `--` that appears outside a quoted string or block comment begins a line comment. A line comment extends to the end of the current line (the next `\n` character or end-of-input, whichever comes first).

While in line comment context:
- Semicolons are not statement terminators.
- Block comment opening sequences (`/*`) are not recognised.
- Single-quote characters do not start a quoted string.

**REQ-PARSE-009: Line Comment Does Not Affect Statement Content**

A line comment is stripped from the statement text before it is sent to the database. The newline that ends the comment is preserved as whitespace.

```sql
-- This file contains two statements:
SELECT 1; -- first query; ignore this semicolon
SELECT 2; -- second query
```

---

### Block Comment Rules

**REQ-PARSE-010: Block Comment Recognition**

The two-character sequence `/*` that appears outside a quoted string or line comment begins a block comment. The block comment ends at the first subsequent occurrence of `*/`.

While in block comment context:
- Semicolons are not statement terminators.
- Line comment sequences (`--`) are not recognised.
- Single-quote characters do not start a quoted string.

**REQ-PARSE-011: Block Comments May Span Multiple Lines**

A block comment may extend across any number of lines. The parser must remain in block comment context across newlines until the closing `*/` is encountered.

```sql
/*
 * This is a multi-line block comment.
 * It describes the following migration.
 * Semicolons here; and here; are not terminators.
 */
INSERT INTO schema_version VALUES (42);

SELECT /* inline block comment; still one statement */ 1;
```

**REQ-PARSE-012: Nested Block Comments Not Supported**

Teradata SQL does not support nested block comments. A `/*` sequence encountered inside an already-open block comment does not open a new nesting level. The first `*/` sequence always closes the outermost block comment.

**REQ-PARSE-013: Unterminated Block Comment Error**

If end-of-input is reached while still inside a block comment context, the parser must report an error identifying the line number where the unterminated comment began.

```
Error: Unterminated block comment

  File:      script.sql
  Statement: 2
  Line:      7

The block comment opened on line 7 was never closed.
Check for a missing '*/' sequence.
```

---

### Error Reporting and Line Numbers

**REQ-PARSE-014: Line Number Tracking**

The parser must maintain an accurate line number counter throughout the entire input. Line numbers increment on each `\n` character, regardless of the current lexical context (inside strings, comments, or plain SQL). Line numbers start at 1.

The line number counter must not reset between statements; it reflects the position within the source file or stdin stream at all times.

**REQ-PARSE-015: Statement Start Line Recording**

The parser records the current line number as the "start line" of a statement at the moment it encounters the first non-whitespace, non-comment character that belongs to that statement. This is the line number used in error messages and verbose progress output.

Rationale: "begins accumulating characters" is ambiguous when leading whitespace or comment tokens precede the first SQL keyword. The start line must reflect where meaningful SQL content begins, not where blank lines or comment lines were skipped. For example:

```sql
-- comment on line 1
-- comment on line 2

SELECT 1;   -- start line is 4 (first non-whitespace SQL character)
```

The recorded start line for the `SELECT 1` statement is line 4, not line 1.

**REQ-PARSE-016: Error Message Line Numbers**

When a database error occurs while executing a statement, the error message must include the start line of the failing statement as it appears in the source file. This allows users to navigate directly to the problematic SQL in their editor.

```
Error: SQL syntax error in statement 3

  File:      migrations/v2.sql
  Statement: 3 of 5
  Line:      14

Expected something like a 'SELECT' keyword but found 'SELCT'.

Error Code: 3706
Session ID: 1429

Failed statement (line 14):
  SELCT * FROM employees;
```

**REQ-PARSE-017: Parse Error Line Numbers**

Parse errors (unterminated strings, unterminated block comments) must also reference the exact line number in the source file where the offending construct began, not the line number at end-of-input.

---

### Parser Correctness Examples

The following table summarises parser behaviour across representative inputs.

| Input | Statements produced | Notes |
|---|---|---|
| `SELECT 1;` | 1 | Standard case |
| `SELECT 1` | 1 | Implicit terminator at EOF |
| `SELECT 1; SELECT 2;` | 2 | Two statements |
| `INSERT INTO t VALUES ('a;b');` | 1 | Semicolon inside string |
| `INSERT INTO t VALUES ('it''s');` | 1 | Escaped quote, then terminator |
| `SELECT 1; -- comment; ignored` | 1 | Line comment after terminator |
| `SELECT /* a;b */ 1;` | 1 | Block comment inline |
| `SELECT\n1\nFROM\nt;` | 1 | Multi-line statement |
| `;;;` | 0 | All empty, suppressed |
| `-- comment only` | 0 | Comment only, no statement |

---

### Comment Space-Injection

**REQ-PARSE-018: Space Injection When Stripping Comments**

When a comment token is removed from the SQL text and the characters immediately before and immediately after the comment are both non-whitespace, the parser MUST inject a single space character in place of the comment. This prevents two adjacent SQL tokens from being merged into an unrecognised token after comment removal.

**Rationale:** Consider the input `SELECT--comment\nfoo`. After the line comment is stripped, the remaining characters are `SELECT` and `foo`. Without space injection the concatenated result would be `SELECTfoo`, which is not valid SQL. Injecting one space produces `SELECT foo`, which is two valid tokens.

**Space-injection examples:**

| Input (comments shown explicitly) | Output after comment removal | Notes |
|---|---|---|
| `SELECT--comment\nfoo` | `SELECT foo` | Space injected between keyword and identifier |
| `SELECT /*c*/ 1` | `SELECT  1` | Comment replaced by single space (adjacent whitespace is already present before the comment, no additional injection needed; the `/*c*/` itself is replaced by one space) |
| `col1--note\n+col2` | `col1 +col2` | Space injected before operator |
| `SELECT\n--full line comment\nfoo` | `SELECT\nfoo` | Newline already separates tokens; no injection needed |

**Detailed rule:**

The parser injects a space if and only if all three of the following conditions hold at the point where the comment is removed:

1. The character immediately preceding the comment (in the original source) is a non-whitespace character.
2. The character immediately following the comment (in the original source) is a non-whitespace character.
3. The two characters are not already separated by any whitespace in the output being built.

When a line comment is followed by a newline (`\n`), the newline is preserved in the output; no additional space is injected because the newline itself serves as a token separator.

**Scope:** Space injection applies to both line comments (`--`) and block comments (`/* */`).

---

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

The `--output` flag provides explicit file output with better error handling and status reporting than shell redirection.

#### Requirements

**REQ-OUT-001: Flag Syntax**
- Flag name: `--output <PATH>` or `-o <PATH>`
- Single argument: file path (absolute or relative)
- Available in `query` command only
- Not available in REPL mode

**REQ-OUT-002: Path Handling**
- Relative paths resolved from current working directory
- Absolute paths used as-is
- `~` expansion supported (home directory)
- Parent directories must exist (no auto-creation)
- Path validation performed before query execution

**REQ-OUT-003: Format Support**
- All output formats supported: `table`, `csv`, `json`
- Format specified via `--format` flag (independent)
- Default format is `table` (same as stdout)
- Format must be specified before query execution

**REQ-OUT-004: File Overwrite Behavior**

When output file already exists:

**Interactive mode (TTY):**
```bash
$ tq query "SELECT * FROM users" --output users.csv
File exists: users.csv

Overwrite? [y/N]: _
```
- Prompt user for confirmation
- `y` or `yes` (case-insensitive): overwrite
- `n`, `no`, or Enter: abort with exit code 2
- Ctrl-C: abort with exit code 130

**Non-interactive mode (no TTY, scripts, CI):**
- Abort with error message and exit code 2
- Error message: "File exists: <path>"
- No prompt displayed

**REQ-OUT-005: Atomic Write**
- Write to temporary file first: `<output>.tmp.<random>`
- Rename to final path only on success
- Prevents partial file creation on query errors
- Temporary file cleanup on failure or interruption
- Random suffix prevents temp file collisions

**REQ-OUT-006: Status Messages**

On success:
```bash
Wrote 1523 rows to users.csv
```

Message format:
- "Wrote N rows to <path>" for SELECT queries
- "Query completed, output written to <path>" for non-SELECT
- Message goes to stderr (not stdout)
- Suppressed by `--quiet` flag

**REQ-OUT-007: Error Handling**

File write errors:
```bash
Error: Cannot write to file

Could not write to: /protected/file.csv
Reason: Permission denied

Check:
  - File path is writable
  - Parent directory exists
  - Sufficient disk space available
```

Error categories:
- Permission denied (exit code 1)
- Disk full (exit code 1)
- Invalid path (exit code 2)
- File exists (exit code 2)
- Parent directory does not exist (exit code 2)

**REQ-OUT-008: Multi-Statement Handling**

When executing multiple statements:
- Only the LAST SELECT query result is written to file
- Non-SELECT statements (INSERT, UPDATE, etc.) execute normally
- Status messages for all statements go to stderr
- If no SELECT query exists, error with exit code 2

Example:
```sql
-- setup.sql
CREATE TABLE temp (id INT);
INSERT INTO temp VALUES (1), (2);
SELECT * FROM temp;  -- This result goes to file
DROP TABLE temp;
```

```bash
$ tq query --file setup.sql --output result.csv
Statement 1: CREATE TABLE - OK
Statement 2: INSERT - OK (2 rows affected)
Statement 3: SELECT - 2 rows returned
Wrote 2 rows to result.csv
Statement 4: DROP TABLE - OK
```

**REQ-OUT-009: Interaction with --quiet**
- `--quiet` suppresses status messages
- File is still written
- Errors still displayed
- Useful for silent scripting

### Teradata Session Type Compatibility

Teradata supports different session types with varying capabilities. The tool must respect these limitations when executing queries.

**Session Types:**

1. **ANSI Mode Session**
   - Explicit transaction control required
   - Supports BEGIN TRANSACTION, COMMIT, ROLLBACK
   - Best for transactional workloads

2. **Teradata Mode Session**
   - Auto-commit by default for most statements
   - Supports BT (Begin Transaction), ET (End Transaction)
   - Some DDL statements cannot be in transactions

3. **BTEQ Mode Session**
   - Legacy batch mode with specific transaction semantics
   - Limited transaction control capabilities

**Transaction Control Requirements:**

**REQ-SESSION-001: Session Mode Detection**
- Tool should query session mode at connection time
- Detection query: `SELECT SessionMode FROM DBC.SessionInfoV WHERE SessionNo = SESSION`
- Cache result for session duration
- Use appropriate transaction syntax based on mode

**REQ-SESSION-002: Transaction Syntax Adaptation**
- ANSI mode: Use BEGIN TRANSACTION/COMMIT/ROLLBACK
- Teradata mode: Use BT/ET/ABORT
- Fallback: Attempt ANSI first, then Teradata syntax on error

**REQ-SESSION-003: DDL Transaction Limitations**

Some Teradata operations cannot execute within transactions:
- CREATE TABLE (in some modes)
- DROP TABLE (in some modes)
- ALTER TABLE statements
- Some utility operations

When these statements fail with "not allowed in transaction":
- Error message: "This statement type cannot be executed within a transaction in current session mode"
- Guidance: "Remove --atomic flag or execute DDL separately"
- Error category: Usage error (exit code 2)

**REQ-SESSION-004: Error Messages**

Transaction control errors should explain session limitations:

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

## Pagination

The `search` subcommands (`search tables`, `search columns`, `search views`) support client-side result pagination via `--page-size` and `--page` flags. This section documents the pagination behavior and its implications for scripting.

### How Pagination Works

When `--page-size <N>` is provided, `tq` fetches the full result set and slices it into pages of `N` rows in memory. The `--page <P>` flag (1-based, default: `1`) selects which page to return.

```bash
# Return the first 25 views matching "report"
tq search views report --page-size 25 --page 1

# Return the second page of 25
tq search views report --page-size 25 --page 2
```

`--page-size` and `--limit` are mutually exclusive. Providing both is a usage error.

### Page Footer

In all non-JSON output formats, a footer line is appended after the results:

```
Page 2 of 5 (47 total rows)
```

In JSON format, the standard `{"ok": true, "row_count": N, "data": [...]}` envelope gains a `pagination` key:

```json
{
  "ok": true,
  "row_count": 25,
  "data": [ ... ],
  "pagination": {
    "page": 2,
    "page_size": 25,
    "total_rows": 72,
    "total_pages": 3,
    "has_more": true
  }
}
```

### Scripting Paginated Results

```bash
# Fetch all pages of a view search and combine into one JSON file
for page in 1 2 3; do
  tq search views report --page-size 25 --page "$page" --format json
done | jq -s '[.[].data[]]'
```

> **Warning — ORDER BY stability**: Pagination in `search` subcommands relies on a deterministic sort order (`DatabaseName ASC`, `TableName/ViewName/ColumnName ASC`). Pages are stable across invocations as long as the underlying catalog data does not change. If objects are created, dropped, or renamed between paginated requests, rows may shift between pages and some results may appear on multiple pages or be skipped entirely. For fully consistent multi-page iteration, use `--format json` without pagination and process the complete result set at once.

## Transaction Control

### Manual Transaction Control

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

### Automatic Transaction Control (--atomic flag)

The `--atomic` flag automatically wraps all statements in a transaction, with automatic rollback on failure.

#### Requirements

**REQ-TXN-001: Flag Syntax**
- Flag name: `--atomic`
- No arguments (boolean flag)
- Available in `query` command only
- Not available in REPL mode
- Only applies to multi-statement execution (file or stdin)

**REQ-TXN-002: Single Statement Behavior**

When `--atomic` is used with a single statement (inline query):
- Warning issued: "Warning: --atomic has no effect on single statements"
- Statement executes normally (no transaction wrapper)
- Exit code 0 (success)
- Warning goes to stderr

**REQ-TXN-003: Transaction Wrapping**

With `--atomic`, the tool automatically:
1. Executes `BEGIN TRANSACTION` before first statement
2. Executes all user statements in sequence
3. On success: executes `COMMIT`
4. On error: executes `ROLLBACK`

User statements never include explicit BEGIN/COMMIT/ROLLBACK.

**REQ-TXN-004: Explicit Transaction Conflicts**

If user statements contain explicit transaction control:
```sql
BEGIN TRANSACTION;
SELECT 1;
COMMIT;
```

With `--atomic` flag:
- Error: "Cannot use --atomic with explicit transaction control"
- Detection: scan for BEGIN, COMMIT, ROLLBACK, BT, ET keywords
- Case-insensitive detection
- Exit code 2 (usage error)
- No statements executed

**REQ-TXN-005: Execution Flow**

Normal flow (success):
```bash
$ tq query --file script.sql --atomic
[Begin transaction]
Statement 1: INSERT - OK (1 row affected)
Statement 2: UPDATE - OK (5 rows affected)
Statement 3: SELECT - 5 rows returned
[Commit transaction]

All statements executed successfully
Transaction committed
```

Error flow (failure):
```bash
$ tq query --file script.sql --atomic
[Begin transaction]
Statement 1: INSERT - OK (1 row affected)
Statement 2: UPDATE - OK (5 rows affected)
Statement 3: INVALID SQL - FAILED
[Rollback transaction]

Error: SQL syntax error in statement 3

Expected something like a 'SELECT' keyword but found 'INVALID'.

Transaction rolled back (all changes reverted)
Statements executed: 1-2
Statement failed: 3

Exit code: 1
```

**REQ-TXN-006: Status Messages**

Transaction messages (stderr):
- Start: "[Begin transaction]" (verbose mode only)
- Success: "Transaction committed"
- Failure: "Transaction rolled back (all changes reverted)"
- Messages suppressed by `--quiet` flag

**REQ-TXN-007: Error Handling**

Transaction errors:
- BEGIN TRANSACTION fails: abort with exit code 1, no user statements executed
- User statement fails: automatic ROLLBACK, exit code 1
- COMMIT fails: automatic ROLLBACK attempt, exit code 1
- ROLLBACK fails: report error but continue (Teradata handles this)

**REQ-TXN-008: Interrupt Handling**

On Ctrl-C during atomic execution:
- Attempt ROLLBACK before exit
- Display: "Interrupted. Rolling back transaction..."
- Exit code 130 (interrupted)
- If ROLLBACK fails, warn but still exit

**REQ-TXN-009: Teradata Transaction Modes**

Teradata has two transaction modes:
1. **ANSI mode**: Explicit transactions required
2. **Teradata mode**: Auto-commit by default

The `--atomic` flag works in both modes:
- ANSI mode: Uses BEGIN TRANSACTION/COMMIT/ROLLBACK (standard)
- Teradata mode: Uses BT (Begin Transaction)/ET (End Transaction) commands
- Mode detection: automatic (query database mode)
- Fallback: try ANSI first, then Teradata syntax if ANSI fails

**REQ-TXN-010: Verbose Output**

With `--verbose` flag:
```bash
$ tq --verbose query --file script.sql --atomic
[INFO] Transaction mode: ANSI
[INFO] Executing: BEGIN TRANSACTION
[Begin transaction]

[INFO] Statement 1: INSERT INTO users VALUES (...)
Statement 1: INSERT - OK (1 row affected)

[INFO] Statement 2: UPDATE users SET active = 1
Statement 2: UPDATE - OK (1 row affected)

[INFO] Executing: COMMIT
[Commit transaction]

All statements executed successfully
Transaction committed
```

**REQ-TXN-011: Interaction with --output**

`--atomic` and `--output` work together:
```bash
tq query --file script.sql --atomic --output results.csv
```

Behavior:
- Transaction controls entire execution
- File written atomically (separate from transaction)
- If query succeeds but file write fails:
  - Transaction still commits
  - Error reported for file write
  - Exit code 1

Rationale: File I/O is not part of database transaction.

**REQ-TXN-012: Non-Transactional Statements**

Some Teradata statements cannot be in transactions:
- DDL statements (CREATE, DROP, ALTER in some modes)
- Some utility commands

If `--atomic` is used with non-transactional statements:
- Teradata returns error: "Statement not allowed in transaction"
- Automatic ROLLBACK attempted
- Error message displayed
- Exit code 1

User guidance: Don't use `--atomic` with DDL-heavy scripts.

## Variable Substitution

Variable substitution allows SQL templates to contain placeholder markers that are resolved at execution time from a YAML parameter file. This enables parameterized, reusable SQL scripts without manual string manipulation.

### Marker Syntax

**REQ-PARAMS-001: Variable Marker Format**

Variables in SQL are written using double curly braces:

```
{{variable_name}}
```

- Markers are case-sensitive: `{{Table}}` and `{{table}}` are distinct.
- Markers may appear anywhere in SQL text: in identifiers, string literals, numeric values, or comments.
- A marker with no whitespace inside the braces is required: `{{ var }}` is invalid, `{{var}}` is valid.
- Markers that span multiple lines are not supported.

**Examples:**

```sql
-- Simple scalar substitution
SELECT * FROM {{target_table}} SAMPLE {{row_count}};

-- Nested path substitution (dot notation into YAML hierarchy)
SELECT * FROM {{env.database}}.{{env.schema}} WHERE region = '{{filters.region}}';

-- Environment variable substitution
SELECT * FROM {{$ENV.TARGET_DB}}.employees WHERE hire_date > '{{start_date}}';
```

---

### Parameter File Format

**REQ-PARAMS-002: YAML File Format**

Parameter files must be valid YAML. The top-level document is a mapping of keys to scalar or nested mapping values. YAML sequences (lists) are not valid as variable values.

```yaml
# params.yaml
target_table: employees
row_count: 100

env:
  database: PRODUCTION
  schema: HR

filters:
  region: EMEA
  min_salary: 50000

start_date: "2024-01-01"
```

**Supported YAML value types:**

| YAML Type | Example | Substituted As |
|-----------|---------|----------------|
| String | `name: employees` | `employees` |
| Integer | `count: 100` | `100` |
| Float | `threshold: 99.5` | `99.5` |
| Boolean | `active: true` | `true` |
| Null | `filter: ~` | `NULL` (SQL keyword) |

**Unsupported YAML value types** (produce an error when referenced):

| YAML Type | Example | Error |
|-----------|---------|-------|
| Sequence | `tables: [a, b]` | Variable value is a list, not a scalar |
| Nested mapping (when used as leaf) | `db.schema` referenced as `{{db}}` | Variable value is a map, not a scalar |

---

### Dot Notation for Nested Keys

**REQ-PARAMS-003: Nested Key Resolution**

Dot notation in marker names traverses the YAML hierarchy:

```
{{section.subsection.key}}
```

Resolution algorithm:
1. Split marker text by `.`
2. Traverse the YAML mapping hierarchy level by level
3. Return the scalar value at the final key

**Example:**

```yaml
# params.yaml
target:
  database:
    name: PRODUCTION
    schema: HR
```

```sql
SELECT * FROM {{target.database.name}}.{{target.database.schema}};
-- Substituted: SELECT * FROM PRODUCTION.HR;
```

**Edge cases:**
- A key containing a literal `.` cannot be expressed in dot notation. Name such keys without dots.
- An intermediate key that resolves to a scalar (not a mapping) produces an error: `Variable 'target.database' is a scalar, cannot traverse further`.

---

### Environment Variable Access

**REQ-PARAMS-004: `$ENV` Special Dictionary**

Within a marker, the prefix `$ENV.` reads from the process environment. No YAML file entry is needed:

```
{{$ENV.VARIABLE_NAME}}
```

- `VARIABLE_NAME` is the exact environment variable name (case-sensitive on Linux/macOS).
- `$ENV` is a reserved prefix and cannot be used as a YAML key name.
- `$ENV` access is evaluated at substitution time from the live environment.

**Examples:**

```sql
-- Read database host from environment
SELECT * FROM {{$ENV.DB_HOST}}.HR.employees;

-- Mix YAML params and ENV vars in the same query
INSERT INTO {{target_schema}}.{{$ENV.TABLE_NAME}} SELECT * FROM staging;
```

**Error when environment variable is not set:**

```
Error: Undefined environment variable in template

Variable '{{$ENV.TARGET_DB}}' references environment variable 'TARGET_DB'
which is not set in the current environment.

Fix:
  export TARGET_DB=myvalue
  tq query -p params.yaml "..."
```

---

### CLI Flag

**REQ-PARAMS-005: `--params` / `-p` Flag**

The `--params` flag is a global option available on all commands. It specifies the path to a YAML parameter file.

```bash
tq -p params.yaml query "SELECT * FROM {{table}}"
tq --params params.yaml query --file script.sql
tq -p base.yaml -p overrides.yaml query --file script.sql
```

- Flag name: `--params` (long), `-p` (short)
- Argument: file path (absolute or relative to current working directory)
- Repeatable: multiple `-p` flags are accepted; later files override earlier files on key conflicts
- Scope: global (applies to any command that accepts SQL input)
- The flag is silently ignored when the executed command does not process SQL templates

---

### Multiple Parameter Files: Merge Semantics

**REQ-PARAMS-006: Merge Rules for Multiple `-p` Flags**

When two or more `-p` flags are provided, parameter files are merged in the order specified. Conflicts are resolved by last-writer-wins at the key level.

```bash
tq -p base.yaml -p env-overrides.yaml query --file report.sql
```

**Merge behavior:**

```yaml
# base.yaml
database: STAGING
schema: HR
filters:
  region: GLOBAL
  active: true

# env-overrides.yaml
database: PRODUCTION    # overrides base.yaml
filters:
  region: EMEA          # overrides base.yaml filters.region
  # filters.active is NOT in env-overrides.yaml, so it remains 'true' from base.yaml
```

After merge, effective parameters:
```yaml
database: PRODUCTION    # from env-overrides.yaml
schema: HR              # from base.yaml (not overridden)
filters:
  region: EMEA          # from env-overrides.yaml
  active: true          # from base.yaml (not overridden)
```

Merge is deep (recursive): nested mappings are merged key-by-key. A top-level key in a later file does NOT replace the entire nested mapping from an earlier file unless the key in the later file is a scalar that overwrites the mapping.

---

### Substitution Execution

**REQ-PARAMS-007: Substitution Timing and Scope**

Variable substitution is performed after SQL is read from its source (argument, file, or stdin) and before the SQL is sent to Teradata. The original SQL source is never modified.

- Substitution applies to all statements in a multi-statement file.
- Substitution applies to the inline query argument when `-p` is provided.
- All markers must be resolved before execution; a single undefined marker aborts the entire execution.
- Substituted values are inserted as raw text (no quoting or escaping is added). The user is responsible for correct SQL quoting in the template.

**Example with quoting:**

```yaml
# params.yaml
department: Sales
```

```sql
-- Template must include the quotes explicitly:
SELECT * FROM employees WHERE department = '{{department}}';
-- After substitution: SELECT * FROM employees WHERE department = 'Sales';

-- NOT:
SELECT * FROM employees WHERE department = {{department}};
-- After substitution: SELECT * FROM employees WHERE department = Sales; (invalid SQL)
```

---

### Error Handling

**REQ-PARAMS-008: Undefined Variable Error**

When a marker references a key not found in the merged parameter set (and it is not a `$ENV.*` marker):

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

Exit code: `2` (usage error)

---

**REQ-PARAMS-009: YAML Parse Error**

When a parameter file cannot be parsed as valid YAML:

```
Error: Invalid YAML in parameter file

Could not parse: params.yaml
Line 7: mapping values are not allowed in this context

Fix:
  - Verify the file is valid YAML
  - Check for missing quotes around special characters
  - Check for incorrect indentation

Hint: Run 'tq help params' for parameter file format reference.
```

Exit code: `2` (usage error)

---

**REQ-PARAMS-010: Parameter File Not Found**

When the file path given to `-p` does not exist or is not readable:

```
Error: Parameter file not found

Could not read: myparams.yaml
Reason: No such file or directory

Check:
  - File path is correct (relative paths are resolved from current directory)
  - File exists and is readable
  - Current directory: /Users/alice/project
```

Exit code: `2` (usage error)

---

**REQ-PARAMS-011: Non-Scalar Variable Value Error**

When a marker resolves to a YAML sequence or nested mapping (not a scalar):

```
Error: Variable value is not a scalar

Variable '{{filters}}' resolved to a mapping, not a scalar value.

The variable 'filters' contains nested keys:
  filters.region    EMEA
  filters.active    true

Fix:
  Use dot notation to access a specific key: {{filters.region}}
```

Exit code: `2` (usage error)

---

**REQ-PARAMS-012: Circular Reference Detection**

Variable markers within values are not recursively expanded. A value `"{{other_var}}"` in a YAML file is treated as the literal string `{{other_var}}` and substituted as-is into the SQL. Circular references are therefore impossible and do not need detection.

---

**REQ-PARAMS-013: Empty YAML File**

An empty YAML file (zero bytes or whitespace-only) is accepted as a valid parameter file with zero variables. If the SQL template contains any markers, REQ-PARAMS-008 applies.

---

**REQ-PARAMS-014: Unused Variables**

Variables defined in the parameter file but not referenced by any marker in the SQL template are silently ignored. No warning is emitted.

---

### Interaction with Other Features

**REQ-PARAMS-015: Interaction with `--file`**

Variable substitution applies to SQL loaded from a file:

```bash
tq -p params.yaml query --file report.sql
```

All markers in `report.sql` are resolved against the merged parameter set before execution.

---

**REQ-PARAMS-016: Interaction with stdin**

Variable substitution applies to SQL read from stdin:

```bash
cat report.sql | tq -p params.yaml query
```

---

**REQ-PARAMS-017: Interaction with `--atomic`**

Variable substitution is performed before transaction wrapping. All markers in all statements are resolved first; if any marker is undefined, no statements execute and no transaction is started.

---

**REQ-PARAMS-018: Interaction with `--output`**

Variable substitution has no effect on output handling. The `--output` flag operates on query results, not on the SQL template.

---

### Help Topic

**REQ-PARAMS-019: `tq help params` Topic**

`tq help params` displays a dedicated help page for variable substitution. See the CLI Interface specification for the exact content.

---

### Examples

**Simple parameterized query:**

```bash
# params.yaml
# table: employees
# limit: 25

tq -p params.yaml query "SELECT * FROM {{table}} SAMPLE {{limit}}"
# Executes: SELECT * FROM employees SAMPLE 25
```

**Parameterized SQL file with nested keys:**

```yaml
# deploy.yaml
target:
  db: PRODUCTION
  schema: HR
run_date: "2026-01-01"
```

```sql
-- migrate.sql
INSERT INTO {{target.db}}.{{target.schema}}.audit
  SELECT CURRENT_TIMESTAMP AS ts, '{{run_date}}' AS run_date, COUNT(*) AS cnt
  FROM {{target.db}}.{{target.schema}}.employees;
```

```bash
tq -p deploy.yaml query --file migrate.sql
```

**Base + override pattern:**

```bash
tq -p base.yaml -p prod-overrides.yaml query --file report.sql
```

**Using environment variables:**

```bash
export SCHEMA=PRODUCTION
tq query "SELECT * FROM {{$ENV.SCHEMA}}.employees SAMPLE 10"
```

---
