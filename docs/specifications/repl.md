# REPL Mode Specifications

## Overview

REPL (Read-Eval-Print Loop) mode provides an interactive database session similar to `psql`, `mysql`, or `usql`. This mode is optimized for exploratory data analysis, schema inspection, and ad-hoc querying.

## Starting REPL Mode

```bash
# With pre-configured connection
export TQ_LOGON="user:pass@host:1025/db"
tq repl

# With explicit connection
tq -l "user:pass@host:1025/db" repl

# With configuration file
tq repl  # Uses ~/.config/tq/config.toml
```

## User Interface

### Prompt Design

```
tq> SELECT * FROM employees
```

**Prompt Variations**:
- `tq>` - Default prompt (connected)
- `tq(multi)>` - Multi-line continuation
- `tq[disconnected]>` - Not connected to database
- `tq[mydb]>` - Connected to specific database

### Status Bar (Optional)

```
────────────────────────────────────────────────────────
[user@host:1025/mydb] [TD2] [2.4s] [10 rows]
────────────────────────────────────────────────────────
```

## Input Handling

### Multi-Line SQL

Queries continue across lines until terminated:

```sql
tq> SELECT
    employee_id,
    first_name,
    last_name
  FROM employees
  WHERE department = 'IT';
```

**Termination Rules**:
- Semicolon (`;`) terminates statement
- Slash (`/`) on empty line executes buffered SQL (Oracle-style)
- `\g` metacommand executes buffered SQL (psql-style)

### Command History

**Features**:
- ↑/↓ arrows navigate history
- Ctrl-R for reverse incremental search
- History persisted to `~/.tq_history` (10,000 entries)
- De-duplicates consecutive identical commands
- Excludes metacommands from history

**History Search**:
```
(reverse-i-search)`sel': SELECT * FROM employees
```

**Persistent History**

**Default Behavior**:
- History automatically saved to `~/.tq_history` on exit
- History automatically loaded on REPL startup
- Maximum 10,000 entries (configurable)
- Oldest entries pruned when limit reached
- File format: plain text, one entry per line

**History File Location (Priority order)**:
1. `--history-file <path>` command-line flag
2. `TQ_HISTORY_FILE` environment variable
3. `~/.config/tq/history` (XDG-compliant location)
4. `~/.tq_history` (fallback for backwards compatibility)

**File Format**:
```
SELECT * FROM employees WHERE dept = 'IT';
SELECT COUNT(*) FROM orders;
SELECT TOP 10 * FROM customers ORDER BY created_at DESC;
```

**Configuration Flags**:

```bash
# Disable history persistence (in-memory only)
tq repl --no-history

# Custom history file location
tq repl --history-file ~/my-queries.txt

# Alternative: Environment variable
export TQ_HISTORY_FILE=~/my-queries.txt
tq repl

# Disable history file via environment variable
export TQ_HISTORY_FILE=/dev/null
tq repl
```

**History File Permissions**:
- Created with mode `0600` (owner read/write only)
- Warn if existing file has unsafe permissions (group/world readable)
- Never write sensitive data (passwords) to history

**Security - Password Filtering**:

Exclude commands containing sensitive patterns:
- Lines matching `password` (case-insensitive)
- Lines with `TQ_LOGON=` environment variable assignments
- Metacommands: `/logon` with passwords

**Behavior Details**:

**Duplicate Handling**:
- Consecutive identical commands: Only store once
- Non-consecutive duplicates: Store each occurrence

**Multi-line Commands**:
- Multi-line SQL stored as single history entry
- Recalled as complete multi-line block on ↑ arrow

**History Exclusions**:
- Empty lines
- Lines starting with `/` (metacommands)
- Lines matching filter patterns (passwords)
- Lines starting with space (user-requested exclusion)

**History Limits**:
- Default: 10,000 entries
- Configurable via `TQ_HISTORY_SIZE` environment variable
- Minimum: 100 entries
- Maximum: 1,000,000 entries

### Line Editing

**Editor Mode Selection**:

```bash
# Emacs mode (default)
tq repl
tq repl --editor-mode emacs

# Vi/Vim mode
tq repl --editor-mode vi

# Via environment variable
export TQ_EDITOR_MODE=vi
tq repl
```

**Emacs Mode (Default)**

**Navigation**:
- `Ctrl-A` → Beginning of line
- `Ctrl-E` → End of line
- `Ctrl-F` → Forward one character
- `Ctrl-B` → Backward one character
- `Alt-F` → Forward one word
- `Alt-B` → Backward one word

**Editing**:
- `Ctrl-D` → Delete character under cursor (or exit if line empty)
- `Ctrl-H` / `Backspace` → Delete character before cursor
- `Ctrl-K` → Kill (cut) from cursor to end of line
- `Ctrl-U` → Kill from cursor to beginning of line
- `Ctrl-W` → Kill word before cursor
- `Alt-D` → Kill word after cursor
- `Ctrl-Y` → Yank (paste) last killed text
- `Ctrl-T` → Transpose (swap) characters
- `Ctrl-L` → Clear screen (redraw)

**History**:
- `Ctrl-P` / `↑` → Previous history entry
- `Ctrl-N` / `↓` → Next history entry
- `Ctrl-R` → Reverse incremental search
- `Ctrl-S` → Forward incremental search

**Completion**:
- `Tab` → Trigger completion
- `Tab` again → Cycle through completions

**Control**:
- `Ctrl-C` → Cancel current input / Interrupt query
- `Ctrl-D` → Exit REPL (when line is empty)
- `Ctrl-Z` → Suspend (Unix only)
- `Enter` → Submit line / Continue multi-line

**Vi/Vim Mode**

Vi mode uses modal editing: **Insert mode** for typing, **Normal mode** for navigation.

**Mode Switching**:
- `i` → Insert mode (insert at cursor)
- `I` → Insert at beginning of line
- `a` → Append (insert after cursor)
- `A` → Append at end of line
- `ESC` / `Ctrl-[` → Return to Normal mode

**Normal Mode - Navigation**:
- `h` → Left one character
- `l` → Right one character
- `j` → Next history entry
- `k` → Previous history entry
- `w` → Forward one word (start)
- `b` → Backward one word (start)
- `e` → Forward to end of word
- `0` → Beginning of line
- `^` → First non-whitespace character
- `$` → End of line

**Normal Mode - Editing**:
- `x` → Delete character under cursor
- `X` → Delete character before cursor
- `dd` → Delete entire line
- `D` → Delete from cursor to end of line
- `C` → Change from cursor to end of line
- `cc` → Change entire line
- `dw` → Delete word forward
- `db` → Delete word backward
- `cw` → Change word forward
- `r<char>` → Replace character under cursor
- `u` → Undo last change
- `Ctrl-R` → Redo last undo
- `p` → Paste after cursor
- `P` → Paste before cursor
- `yy` → Yank (copy) entire line

**Insert Mode**:
- All printable characters → Insert at cursor
- `Backspace` → Delete character before cursor
- `Delete` → Delete character under cursor
- `Ctrl-W` → Delete word before cursor
- `Ctrl-U` → Delete from cursor to beginning of line
- `ESC` / `Ctrl-[` → Return to Normal mode

**Mode Indicator** (Vi mode only):
```
# Insert mode
tq[INS]> SELECT * FROM employees_

# Normal mode
tq[NOR]> SELECT * FROM employees
```

## SQL Syntax Highlighting

**Color Scheme**:
- **Keywords** (SELECT, FROM, WHERE): Cyan bold
- **Strings** ('text'): Green
- **Numbers** (123, 45.67): Yellow
- **Comments** (-- comment, /* */): Gray italic
- **Functions** (COUNT, SUM): Magenta
- **Operators** (=, !=, AND, OR): White

**Example**:
```sql
tq> SELECT COUNT(*) FROM employees WHERE dept = 'IT';
     ^^^^^^ ^^^^^^^      ^^^^^^^^^       ^^^^   ^^
     cyan   magenta      keyword         cyan  green
```

## Tab Completion

### Keyword Completion

**Trigger**: Press Tab key after typing partial SQL keyword.

**Supported Keywords**:
- SELECT, INSERT, UPDATE, DELETE, CREATE, DROP, ALTER, TRUNCATE
- FROM, WHERE, JOIN, INNER JOIN, LEFT JOIN, RIGHT JOIN, FULL JOIN, CROSS JOIN
- GROUP BY, HAVING, ORDER BY, LIMIT, OFFSET
- AND, OR, NOT, IN, EXISTS, BETWEEN, LIKE, IS NULL
- DISTINCT, ALL, TOP, UNION, INTERSECT, EXCEPT
- AS, ON, USING, CASE, WHEN, THEN, ELSE, END
- VALUES, SET, WITH
- PRIMARY KEY, FOREIGN KEY, UNIQUE, INDEX
- DATABASE, TABLE, VIEW, SCHEMA, PROCEDURE, FUNCTION
- BEGIN, COMMIT, ROLLBACK, TRANSACTION
- GRANT, REVOKE, CONSTRAINT

**Behavior**:

Single match - auto-complete:
```sql
tq> SEL<TAB>
tq> SELECT
```

Multiple matches - show options:
```sql
tq> UPD<TAB>
    UPDATE    UPDATEXML (if supported)
```

Case-insensitive matching:
```sql
tq> sel<TAB> → SELECT
tq> SEL<TAB> → SELECT
```

### Table Name Completion

**Trigger Contexts**: Press Tab after typing partial table name following:
- `FROM` - Main table reference
- `JOIN` / `INNER JOIN` / `LEFT JOIN` / `RIGHT JOIN` / `FULL JOIN` / `CROSS JOIN`
- `UPDATE` - Table to update
- `INTO` - Insert target

**Data Source**:
- Query Teradata system catalog: `DBC.TablesV`
- Load database names + current database tables initially
- Cache per-database tables on-demand
- Include both tables and views

**Teradata-Specific Behavior**:
- **Qualified Names**: Teradata uses `database.table` format
- **Unqualified Names**: Resolve to current database
- **Lazy Loading**: Only load metadata for databases user actually explores
- **Intelligent Caching**: Cache per-database, not global

**After FROM with no prefix - Show databases + current DB tables**:
```sql
tq> SELECT * FROM <TAB>

Databases:
    production    staging    development    analytics

Tables in current database (production):
    customers    employees    orders    products    [50 more...]
```

**After database name + dot - Show tables in that database**:
```sql
tq> SELECT * FROM production.<TAB>
Tables in 'production':
    customers    employees    orders    products    invoices    [45 more...]
```

**Loading States**:

**First Tab press (metadata not cached)**:
```sql
tq> SELECT * FROM <TAB>
Loading tables... ⠋

[After loading completes (< 500ms):]
    customers    employees    orders    products    [50 more...]
```

**Cached metadata (instant response < 50ms)**:
```sql
tq> SELECT * FROM <TAB>
[Instant, no loading indicator]
    customers    employees    orders    products    [50 more...]
```

### Column Name Completion

**Trigger Contexts**: Press Tab after typing partial column name following:
- `SELECT` - Column list
- `WHERE` - Filter conditions
- `ORDER BY` - Sort columns
- `GROUP BY` - Grouping columns
- `HAVING` - Aggregate filters
- After comma in column lists

**Data Source**:
- Query Teradata system catalog: `DBC.ColumnsV`
- Filter by table name extracted from query context
- Include column name and data type
- Cache results per table for session duration

**Simple query - single table**:
```sql
tq> SELECT * FROM employees WHERE <TAB>
    employee_id    first_name    last_name    email    hire_date
    salary         department_id    manager_id    created_at    updated_at
```

**Column name prefix matching**:
```sql
tq> SELECT * FROM employees WHERE emp<TAB>
    employee_id
```

**Multiple matches - show with type hints**:
```sql
tq> SELECT * FROM employees WHERE <TAB>
    employee_id (INTEGER)       first_name (VARCHAR)
    last_name (VARCHAR)         email (VARCHAR)
    hire_date (DATE)            salary (DECIMAL)
    department_id (INTEGER)     manager_id (INTEGER)
```

## Result Display

### Table Formatting

**Default (Fits Terminal)**:
```
┌─────────┬──────────┬─────────────┬─────────┐
│ id      │ name     │ email       │ active  │
├─────────┼──────────┼─────────────┼─────────┤
│ 1       │ Alice    │ a@test.com  │ true    │
│ 2       │ Bob      │ b@test.com  │ false   │
│ 3       │ Charlie  │ c@test.com  │ true    │
└─────────┴──────────┴─────────────┴─────────┘

3 rows in set (0.123s)
```

**Expanded Display** (toggle with `\x`):
```
-[ RECORD 1 ]------------------
id     | 1
name   | Alice
email  | a@test.com
active | true

-[ RECORD 2 ]------------------
id     | 2
name   | Bob
email  | b@test.com
active | false
```

### Large Result Handling & Result Paging

Result paging uses a three-layer strategy:
1. **Column Windowing** - Limit visible columns to maintain readability
2. **Cell Truncation** - Limit cell content length to prevent layout breaks
3. **Row Paging** - Paginate vertically through long result sets

**CRITICAL REQUIREMENT:** Pager MUST be safe - 'q' key exits pager and returns to REPL, never exits the entire program.

#### Column Windowing (Layer 1)

**Objective:** Display manageable subset of columns, navigate horizontally through remaining columns.

**Column Navigation:**
- `←` (Left Arrow): Shift window left by 1 column
- `→` (Right Arrow): Shift window right by 1 column
- `Ctrl-←`: Jump to first column group
- `Ctrl-→`: Jump to last column group
- `Home`: Jump to first column
- `End`: Jump to last column

**Column Position Indicator:**
```
Columns 1-5 of 23 | Rows 1-20 of 1,234 (2%)
```

#### Cell Truncation (Layer 2)

**Maximum Cell Display Length:**
- **Text cells (VARCHAR, CHAR):** 100 characters maximum
- **Numeric cells:** No truncation
- **Date/Time cells:** No truncation

**Truncation Indicator:**
- Append `…` (Unicode ellipsis U+2026) to truncated values
- Example: `This is a very long description...` (99 chars + ellipsis)

#### Vertical Row Paging (Layer 3)

**Navigation Keys:**

**Single Row Movement:**
- `j` / `↓` (Down Arrow): Next row
- `k` / `↑` (Up Arrow): Previous row

**Page Movement:**
- `Space` / `Page Down`: Next page
- `b` / `Page Up`: Previous page

**Jump Navigation:**
- `g` / `Home`: Jump to first row
- `G` / `End`: Jump to last row

**Row Position Indicator:**
```
Rows 1-20 of 1,234 (2%)
```

#### Pager Exit Behavior (CRITICAL)

**Exit Keys:**
- `q` (lowercase): Exit pager, return to `tq>` prompt
- `Escape`: Also exits pager
- `Ctrl-C`: Cancel pager, return to prompt

**Exit Program (From REPL Only, NOT from Pager):**
- `Ctrl-D`: Exit tq program (when at empty prompt)
- `/quit`: Exit tq program (metacommand)

**Exit Flow Example:**
```
tq> SELECT * FROM employees;
[Query executes, enters pager mode with results displayed]

┌─────────────┬──────────────┬──────────────┐
│ employee_id │ first_name   │ last_name    │
├─────────────┼──────────────┼──────────────┤
│ 1           │ Alice        │ Anderson     │
│ 2           │ Bob          │ Brown        │
└─────────────┴──────────────┴──────────────┘

Rows 1-20 of 500 | q: exit pager

[User presses 'q']

tq> _
[Back at REPL prompt, session fully preserved]
```

#### Complete Status Bar Design

**Layout (Two-Line Status Bar at Bottom):**
```
┌────────────────────────────────────────────────────────────────────────────┐
│ Columns 1-5 of 23 | Rows 1-20 of 1,234 (2%) | Navigation: ←→ ↑↓ Space b  │
│ g/G: first/last | q/Esc: exit pager                                        │
└────────────────────────────────────────────────────────────────────────────┘
```

#### Result Truncation Hint (>100 rows)

When displaying results and suggesting result limiting:

```sql
tq> SELECT * FROM employees;
... 100 rows displayed ...

Showing first 100 rows. Use TOP N or SAMPLE N for different results.
```

### NULL Handling

Display `NULL` values distinctly:
```
┌─────────┬──────────┐
│ id      │ name     │
├─────────┼──────────┤
│ 1       │ Alice    │
│ 2       │ [NULL]   │  ← grayed out
└─────────┴──────────┘
```

## Metacommands

Metacommands provide non-SQL functionality. They start with `/` or `\` and execute immediately.

### Connection Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/logon [connection]` | `\c` | Connect/switch database or show current connection | `/logon user:pass@host:1025/db` |
| `/disconnect` | `\q` | Disconnect current connection | `/disconnect` |
| `/reconnect` | - | Reconnect to current database | `/reconnect` |
| `/ping` | - | Test connection | `/ping` |

**`/logon` Metacommand**

**Syntax**:
```
/logon [connection-string]      # Connect to new database
/logon                           # Show current connection info
\c [connection-string]           # Short alias
```

**Connection String Format**: Same format as CLI `-l` flag:
```
username:password@hostname:port/database
```

**Show current connection (no arguments)**:
```sql
tq> /logon

Current Connection:
  Host: prod-td01.company.com:1025
  Database: production_db
  User: alice
  Authentication: LDAP
  Session ID: 987654321
  Status: Active
```

**Connect to new database**:
```sql
tq> /logon alice:secret@dev-td01.company.com:1025/dev_db

Disconnecting from prod-td01.company.com:1025/production_db...
Connecting to dev-td01.company.com:1025/dev_db...
Connected successfully.

New Connection:
  Host: dev-td01.company.com:1025
  Database: dev_db
  User: alice
  Session ID: 123456789
```

**`/ping` Metacommand**

**Syntax**:
```
/ping
```

**Behavior**:
- Executes a lightweight query (`SELECT 1`) to test connection responsiveness
- Measures round-trip latency in milliseconds
- Reports success/failure without disrupting the REPL session

**Success case**:
```
tq> /ping
Connection OK (127ms)
Host: myhost.company.com:1025
Database: production
User: alice
Session active for: 15m 23s
```

**Failure case**:
```
tq> /ping
Connection FAILED (timeout after 30s)

Error: Connection lost to myhost.company.com:1025
Reason: Read timeout

Suggestions:
  - Network issue - check connectivity
  - Database may be overloaded
  - Use /reconnect to establish new connection
```

### Schema Inspection Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/describe <table>` | `\d` | Describe table structure | `/describe employees` |
| `/list databases` | `\l` | List all databases | `/list databases` |
| `/list tables` | `\dt` | List tables in current database | `/list tables` |
| `/list tables <pattern>` | `\dt` | List tables matching pattern | `/list tables emp%` |
| `/list views` | `\dv` | List views | `/list views` |
| `/list schemas` | `\dn` | List schemas | `/list schemas` |
| `/show indexes <table>` | `\di` | Show table indexes | `/show indexes employees` |

**`/describe` Metacommand**

**Syntax**:
```
/describe <table_name>
/describe <database>.<table_name>
\d <table_name>                    -- Short alias
```

**Output Format**:

```
tq> /describe employees

Table: PRODUCTION.employees
Type: Table
Approximate Rows: 42,573

Columns:
┌───────────────┬──────────────┬──────────┬─────────┬──────────┐
│ Column        │ Type         │ Nullable │ Default │ Comments │
├───────────────┼──────────────┼──────────┼─────────┼──────────┤
│ employee_id   │ INTEGER      │ NO       │ -       │ PK       │
│ first_name    │ VARCHAR(50)  │ YES      │ NULL    │          │
│ last_name     │ VARCHAR(50)  │ YES      │ NULL    │          │
│ email         │ VARCHAR(100) │ YES      │ NULL    │          │
│ hire_date     │ DATE         │ YES      │ NULL    │          │
│ salary        │ DECIMAL(10,2)│ YES      │ NULL    │          │
│ department_id │ INTEGER      │ YES      │ NULL    │ FK       │
│ manager_id    │ INTEGER      │ YES      │ NULL    │ FK       │
└───────────────┴──────────────┴──────────┴─────────┴──────────┘

Indexes:
  PRIMARY KEY (employee_id)
  INDEX idx_dept (department_id)
  INDEX idx_email (email) UNIQUE

Foreign Keys:
  department_id → departments(id)
  manager_id → employees(employee_id)
```

### Data Sampling Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/sample <table> [n]` | Show random sample (default 10 rows) | `/sample employees 20` |
| `/peek <table>` | Show first 5 rows and column info | `/peek employees` |

### Export Commands

**Syntax**:
```
/export <format> [file]
/export <format> --append        (append to file instead of overwrite)
/export <format>                 (output to stdout if no file specified)
```

**Supported Formats**:

1. **CSV Format** (`csv`)
   - Standard CSV format with comma delimiters
   - Double quotes for fields containing special characters
   - Header row with column names
   - Example: `/export csv employees.csv`

2. **JSON Format** (`json`)
   - Array of objects, one object per row
   - Column names as object keys
   - Proper JSON type handling
   - Pretty-printed by default
   - Example: `/export json data.json`

3. **SQL Format** (`sql`)
   - INSERT statements for data reload
   - Generates: `INSERT INTO table_name (col1, col2) VALUES (val1, val2);`
   - Example: `/export sql inserts.sql`

**With file specified**:
```sql
tq> SELECT * FROM employees WHERE dept = 'IT';
[10 rows returned]

tq> /export csv employees_it.csv
Exported 10 rows to employees_it.csv

tq> /export json employees_it.json
Exported 10 rows to employees_it.json (1.2 KB)
```

**Without file specified (stdout)**:
```sql
tq> /export csv
id,name,email
1,Alice,alice@example.com
2,Bob,bob@example.com
```

| Command | Description | Example |
|---------|-------------|---------|
| `/export csv <file>` | Export last result to CSV | `/export csv employees.csv` |
| `/export json <file>` | Export last result to JSON | `/export json data.json` |
| `/export sql <file>` | Export as SQL INSERT statements | `/export sql inserts.sql` |

### Session Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/session` | - | Show session info | `/session` |
| `/timing on` | `\t` | Enable query timing | `/timing on` |
| `/timing off` | `\t` | Disable query timing | `/timing off` |
| `/set format <fmt>` | - | Set output format | `/set format json` |
| `/pager on\|off` | - | Enable/disable result paging | `/pager on` |
| `/colors on\|off` | - | Enable/disable syntax highlighting | `/colors on` |

**`/pager on|off` Metacommand**

**Syntax**:
```
/pager on       (enable result paging)
/pager off      (disable paging, show all results)
```

**Pager enabled (default)**:
```sql
tq> /pager on
Result paging enabled

tq> SELECT * FROM huge_table;
[Shows first screen, allows scrolling with j/k/arrows]
Rows 1-50 of 5,234 | Space: next | b: prev | q: quit
```

**Pager disabled**:
```sql
tq> /pager off
Result paging disabled

tq> SELECT * FROM huge_table;
[Displays all 5,234 rows without pagination]
```

**`/colors on|off` Metacommand**

**Syntax**:
```
/colors on      (enable syntax highlighting)
/colors off     (disable colors, plain text output)
```

**Colors enabled (default in TTY)**:
```sql
tq> /colors on
Syntax highlighting enabled

tq> SELECT * FROM employees;
[SQL keywords displayed in cyan, strings in green, etc.]
```

**Colors disabled**:
```sql
tq> /colors off
Syntax highlighting disabled

tq> SELECT * FROM employees;
[All text in plain white, no colors]
```

**Session Info Output**:
```sql
tq> /session
Session Information:
  Host: myhost.company.com:1025
  Database: production_db
  User: alice
  Session ID: 123456789
  Connected: 2024-01-15 10:30:45
  Duration: 15m 23s
  Logon Mechanism: LDAP
  Character Set: UTF8
  Queries Executed: 42
```

### Utility Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/help` | `\?` | Show help | `/help` |
| `/help <command>` | - | Show command help | `/help describe` |
| `/clear` | `\clear` | Clear screen | `/clear` |
| `/history` | - | Show command history | `/history` |
| `/edit` | `\e` | Edit last query in $EDITOR | `/edit` |
| `/quit` | `\q` | Exit REPL | `/quit` |

## Special Features

### Query Editing

**External Editor**:
```sql
tq> /edit
[Opens $EDITOR with last query]
[On save and exit, executes query]
```

**Re-execute Last Query**:
```sql
tq> /repeat
[Executes most recent SQL query]
```

### Transaction Support

```sql
tq> BEGIN TRANSACTION;
tq(tx)> INSERT INTO employees VALUES (101, 'Test', 'User');
tq(tx)> SELECT * FROM employees WHERE id = 101;
tq(tx)> ROLLBACK;
tq> -- Transaction rolled back
```

Prompt changes to `tq(tx)>` when in transaction.

### Query Cancellation

- **Ctrl-C**: Cancel running query gracefully
- **Double Ctrl-C**: Force quit (last resort)

**Feedback**:
```
Query running... (2.3s) [Press Ctrl-C to cancel]
^C
Query cancelled by user (after 2.3s)
```

### Autocorrect Suggestions

```sql
tq> SELCT * FROM employees;
Error: Syntax error near "SELCT"
Did you mean: SELECT?

Fix and retry? [Y/n] y
[Executes corrected query]
```
