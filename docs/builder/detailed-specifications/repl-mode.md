# REPL Mode Specifications

**Version:** 1.1.0
**Last Updated:** 2026-01-18
**Owner:** cli-ux-designer agent
**Status:** Active Specification - Phase 2 (Sprint 4) in Development

---

## Table of Contents

1. [Overview](#51-overview)
   - [Sprint 4: Interactive Mode Phase 2 - Foundation Features](#511-sprint-4-interactive-mode-phase-2---foundation-features)
2. [Starting REPL Mode](#52-starting-repl-mode)
3. [User Interface](#53-user-interface)
4. [Input Handling](#54-input-handling)
5. [SQL Syntax Highlighting](#55-sql-syntax-highlighting)
6. [Tab Completion](#56-tab-completion)
7. [Result Display](#57-result-display)
8. [Metacommands](#58-metacommands)
9. [Special Features](#59-special-features)

---

## 5.1 Overview

REPL (Read-Eval-Print Loop) mode provides an interactive database session similar to `psql`, `mysql`, or `usql`. This mode is optimized for exploratory data analysis, schema inspection, and ad-hoc querying.

### 5.1.1 Sprint 4: Interactive Mode Phase 2 - Foundation Features

**Sprint Objective**: Build essential metacommands and quality-of-life improvements on top of the MVP foundation.

**Sprint 4 Features** (Current Development):

| Feature ID | Feature | Priority | User Value |
|------------|---------|----------|------------|
| FR-115 | `/describe` metacommand | P0 | Inspect table structure without writing SQL |
| FR-118 | `/ping` metacommand | P0 | Test connection health in long-running sessions |
| FR-104 | Persistent history | P1 | Recall commands across sessions |
| FR-109 | Vi keybindings | P1 | Familiar editing for Vi/Vim users |
| FR-110 | Emacs keybindings | P1 | Familiar editing for Emacs users (default) |

**Design Principles for Sprint 4**:

1. **Consistency with PostgreSQL psql**: Use familiar metacommand syntax (`\d`, `\dt`, `\l`) with slash alternatives (`/describe`, `/list tables`)
2. **Fail gracefully**: Metacommands should never crash REPL, always provide helpful error messages
3. **Progressive enhancement**: Each feature stands alone, no hard dependencies between features
4. **Security first**: History filtering to prevent password leakage, safe file permissions
5. **Performance**: Fast response times, asynchronous operations where possible

**Implementation Sequence**:

1. **Phase 1**: `/describe` metacommand (FR-115)
   - Query Teradata system catalog (DBC.ColumnsV, DBC.IndicesV, DBC.TablesV)
   - Display comprehensive table information
   - Handle errors gracefully (table not found, permission denied)
   - Support both qualified (`db.table`) and unqualified names

2. **Phase 2**: `/ping` metacommand (FR-118)
   - Execute lightweight test query (`SELECT 1`)
   - Measure latency
   - Display connection information
   - Handle connection failures without exiting REPL

3. **Phase 3**: Persistent history (FR-104)
   - Configure reedline's FileBackedHistory
   - Implement password filtering
   - Handle file permissions and errors
   - Support custom history file location

4. **Phase 4**: Editor mode enhancement (FR-109, FR-110)
   - Document existing reedline keybindings
   - Add mode indicator to prompt (Vi mode)
   - Support mode configuration via flag and config file
   - Test all keybindings across platforms

**Success Criteria**:
- ✅ `/describe employees` shows table structure in < 500ms
- ✅ `/ping` completes in < 300ms for healthy connection
- ✅ History persists across sessions, loads in < 100ms
- ✅ All documented keybindings work in both Emacs and Vi modes
- ✅ No regressions in existing MVP features (multi-line, `/help`, `/quit`, `/session`)
- ✅ Comprehensive error handling for all edge cases
- ✅ Zero crashes or panics in normal operation

**Out of Scope for Sprint 4**:
- SQL syntax highlighting (Sprint 5)
- Tab completion (Sprint 5)
- Result paging (Sprint 5)
- Additional metacommands (`/list tables`, `/export`, etc.)
- Query timing display (Sprint 5)

## 5.2 Starting REPL Mode

```bash
# With pre-configured connection
export TQ_LOGON="user:pass@host:1025/db"
tq repl

# With explicit connection
tq -l "user:pass@host:1025/db" repl

# With configuration file
tq repl  # Uses ~/.config/tq/config.toml
```

## 5.3 User Interface

### 5.3.1 Prompt Design

```
tq> SELECT * FROM employees
```

**Prompt Variations**:
- `tq>` - Default prompt (connected)
- `tq(multi)>` - Multi-line continuation
- `tq[disconnected]>` - Not connected to database
- `tq[mydb]>` - Connected to specific database

### 5.3.2 Status Bar (Optional)

```
────────────────────────────────────────────────────────
[user@host:1025/mydb] [TD2] [2.4s] [10 rows]
────────────────────────────────────────────────────────
```

## 5.4 Input Handling

### 5.4.1 Multi-Line SQL

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

### 5.4.2 Command History

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

**Persistent History Specification** (FR-104)

**Purpose**: Save command history across REPL sessions for improved productivity and command recall.

**Default Behavior**:
- History automatically saved to `~/.tq_history` on exit
- History automatically loaded on REPL startup
- Maximum 10,000 entries (configurable)
- Oldest entries pruned when limit reached
- File format: plain text, one entry per line

**History File Location**:

Priority order:
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

Simple newline-delimited text format:
- One command per line
- Multi-line SQL stored with embedded `\n` (literal backslash-n)
- No timestamps or metadata (for simplicity)
- UTF-8 encoding

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

**Security Considerations**:

**Password Filtering**:
Exclude commands containing sensitive patterns:
- Lines matching `password` (case-insensitive)
- Lines with `TQ_LOGON=` environment variable assignments
- Metacommands: `/logon` with passwords

**Example filtered command**:
```
# This WILL be saved to history:
SELECT * FROM users WHERE username = 'alice';

# This will NOT be saved (contains "password"):
ALTER USER alice PASSWORD = 'secret123';

# This will NOT be saved (metacommand):
/logon user:pass@host:1025/db
```

**Behavior Details**:

**Duplicate Handling**:
- Consecutive identical commands: Only store once
- Non-consecutive duplicates: Store each occurrence (allows frequency tracking)

**Multi-line Commands**:
- Multi-line SQL stored as single history entry
- Recalled as complete multi-line block on ↑ arrow

**History Exclusions**:
- Empty lines
- Lines starting with `/` (metacommands)
- Lines matching filter patterns (passwords)
- Lines starting with space (user-requested exclusion)

**History Search** (Ctrl-R):
```
tq> [Press Ctrl-R]
(reverse-i-search)`':

tq> [Type "emp"]
(reverse-i-search)`emp': SELECT * FROM employees WHERE dept = 'IT';

[Press Ctrl-R again to find older matches]
(reverse-i-search)`emp': SELECT COUNT(*) FROM employees;

[Press Enter to execute, or Escape to edit]
```

**History Limits**:
- Default: 10,000 entries
- Configurable via `TQ_HISTORY_SIZE` environment variable
- Minimum: 100 entries
- Maximum: 1,000,000 entries (practical limit)

**Error Handling**:

**History file not writable**:
```
Warning: Cannot write history to ~/.tq_history (Permission denied)
History will be stored in memory only for this session.
```

**History file corrupted**:
```
Warning: History file ~/.tq_history is corrupted (invalid UTF-8)
Starting with empty history. Previous history backed up to ~/.tq_history.bak
```

**Disk full**:
```
Warning: Cannot save history to ~/.tq_history (Disk full)
History for this session will be lost on exit.
```

**Implementation Notes**:
- Use reedline's built-in `FileBackedHistory` implementation
- Load history asynchronously during startup (don't block prompt)
- Flush history on graceful exit (Ctrl-D, `/quit`)
- Attempt to save history on crash (signal handler)
- Use atomic writes (write to temp file, rename) to prevent corruption

### 5.4.3 Line Editing

**Editor Mode Specification** (FR-109, FR-110)

**Purpose**: Provide familiar keybindings for users comfortable with either Emacs or Vi/Vim editing styles.

**Mode Selection**:

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

**Emacs Mode (Default)** (FR-110)

**Navigation**:
- `Ctrl-A` → Beginning of line
- `Ctrl-E` → End of line
- `Ctrl-F` → Forward one character
- `Ctrl-B` → Backward one character
- `Alt-F` → Forward one word
- `Alt-B` → Backward one word
- `Alt-<` → Beginning of buffer (first history entry)
- `Alt->` → End of buffer (latest input)

**Editing**:
- `Ctrl-D` → Delete character under cursor (or exit if line empty)
- `Ctrl-H` / `Backspace` → Delete character before cursor
- `Ctrl-K` → Kill (cut) from cursor to end of line
- `Ctrl-U` → Kill from cursor to beginning of line
- `Ctrl-W` → Kill word before cursor
- `Alt-D` → Kill word after cursor
- `Ctrl-Y` → Yank (paste) last killed text
- `Alt-Y` → Cycle through kill ring (after Ctrl-Y)
- `Ctrl-T` → Transpose (swap) characters
- `Alt-T` → Transpose words
- `Ctrl-L` → Clear screen (redraw)

**History**:
- `Ctrl-P` / `↑` → Previous history entry
- `Ctrl-N` / `↓` → Next history entry
- `Ctrl-R` → Reverse incremental search
- `Ctrl-S` → Forward incremental search
- `Alt-<` → First history entry
- `Alt->` → Last history entry (current input)

**Completion**:
- `Tab` → Trigger completion
- `Tab` again → Cycle through completions

**Control**:
- `Ctrl-C` → Cancel current input / Interrupt query
- `Ctrl-D` → Exit REPL (when line is empty)
- `Ctrl-Z` → Suspend (Unix only)
- `Enter` → Submit line / Continue multi-line

**Vi/Vim Mode** (FR-109)

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
- `f<char>` → Find next occurrence of character
- `t<char>` → Till (before) next occurrence of character
- `;` → Repeat last f/t command
- `,` → Reverse last f/t command

**Normal Mode - Editing**:
- `x` → Delete character under cursor
- `X` → Delete character before cursor
- `dd` → Delete entire line
- `D` → Delete from cursor to end of line
- `C` → Change from cursor to end of line (delete and enter insert)
- `cc` → Change entire line
- `dw` → Delete word forward
- `db` → Delete word backward
- `cw` → Change word forward
- `r<char>` → Replace character under cursor
- `s` → Substitute character (delete and enter insert)
- `~` → Toggle case of character
- `u` → Undo last change
- `Ctrl-R` → Redo last undo
- `p` → Paste after cursor
- `P` → Paste before cursor
- `yy` → Yank (copy) entire line
- `yw` → Yank word forward

**Normal Mode - History Search**:
- `/` → Search history forward (like Ctrl-R in Emacs)
- `?` → Search history backward
- `n` → Repeat last search (same direction)
- `N` → Repeat last search (opposite direction)

**Insert Mode**:
- All printable characters → Insert at cursor
- `Backspace` → Delete character before cursor
- `Delete` → Delete character under cursor
- `Ctrl-W` → Delete word before cursor
- `Ctrl-U` → Delete from cursor to beginning of line
- `ESC` / `Ctrl-[` → Return to Normal mode

**Visual Mode** (Vi extended):
- `v` (in Normal mode) → Enter visual mode
- `h`/`j`/`k`/`l` → Expand selection
- `d` → Delete selection
- `y` → Yank selection
- `c` → Change selection
- `ESC` → Exit visual mode

**Implementation Notes**:
- Use reedline's built-in `EditMode::Vi` and `EditMode::Emacs`
- Default mode: Emacs (more familiar to most users, less modal confusion)
- Persist mode preference in config file (`~/.config/tq/config.toml`)
- Display current mode in prompt for Vi users: `tq[INSERT]>` vs `tq[NORMAL]>`
- Vi mode visual feedback: Change prompt style or cursor shape when available

**Configuration File**:
```toml
# ~/.config/tq/config.toml
[repl]
editor_mode = "vi"  # or "emacs"
```

**Mode Indicator** (Vi mode only):
```
# Insert mode
tq[INS]> SELECT * FROM employees_

# Normal mode
tq[NOR]> SELECT * FROM employees

# Visual mode
tq[VIS]> SELECT * FROM employees
```

**Testing Recommendations**:
- Verify all keybindings work correctly in both modes
- Test on multiple terminal emulators (iTerm2, Terminal.app, Alacritty, Windows Terminal)
- Confirm Alt/Meta key support (may require terminal configuration)
- Test with non-ASCII input (Unicode SQL identifiers)
- Validate history search works in both modes

## 5.5 SQL Syntax Highlighting

**Color Scheme** (customizable):
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

## 5.6 Tab Completion

### 5.6.1 Keyword Completion

```sql
tq> SEL<TAB>
    SELECT

tq> SELECT * FROM emp<TAB>
                  employees
```

### 5.6.2 Context-Aware Completion

**After FROM**:
```sql
tq> SELECT * FROM <TAB>
    employees    departments    projects    users
```

**After WHERE column**:
```sql
tq> SELECT * FROM employees WHERE dept<TAB>
                                  department
```

**Column Name Completion**:
```sql
tq> SELECT emp<TAB>
           employee_id    employee_name    employee_dept
```

### 5.6.3 Metacommand Completion

```sql
tq> \d<TAB>
    \d         \describe   \dt        \databases
```

## 5.7 Result Display

### 5.7.1 Table Formatting

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

### 5.7.2 Large Result Handling

**Wide Tables** (horizontal scrolling):
```
Use arrow keys: ← → to scroll, Q to quit pager
[Columns 1-5 of 20] >>>
```

**Long Results** (vertical paging):
```
Rows 1-50 of 1,234 (4%)
Space: next page | b: previous page | q: quit | /: search
```

**Pager Options**:
- `less`-like navigation
- Search with `/pattern`
- Jump to line with `123G`
- Follow mode for streaming results

### 5.7.3 NULL Handling

Display `NULL` values distinctly:
```
┌─────────┬──────────┐
│ id      │ name     │
├─────────┼──────────┤
│ 1       │ Alice    │
│ 2       │ [NULL]   │  ← grayed out
└─────────┴──────────┘
```

## 5.8 Metacommands

Metacommands provide non-SQL functionality. They start with `/` or `\` and execute immediately.

### 5.8.1 Connection Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/logon <connection>` | `\c` | Connect to database | `/logon user:pass@host:1025/db` |
| `/disconnect` | `\q` | Disconnect current connection | `/disconnect` |
| `/reconnect` | - | Reconnect to current database | `/reconnect` |
| `/ping` | - | Test connection | `/ping` |

**`/ping` Metacommand Specification** (FR-118)

**Purpose**: Test database connection health from within a REPL session without exiting.

**Syntax**:
```
/ping
```

**Behavior**:
- Executes a lightweight query (`SELECT 1`) to test connection responsiveness
- Measures round-trip latency in milliseconds
- Reports success/failure without disrupting the REPL session
- Useful for long-running sessions to verify connection is still alive

**Output Format**:

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
  - Session may have timed out
  - Use /reconnect to establish new connection
```

**Error Handling**:
- Connection timeout → Display timeout duration, suggest `/reconnect`
- Connection lost → Clear message, suggest `/reconnect`
- Query error → Display error but continue session
- Never exit REPL on ping failure

**Implementation Notes**:
- Use existing database client connection (don't create new connection)
- Set short timeout (5-10s) for ping query
- Measure time from query submit to first result
- Should complete in < 500ms for healthy connection
- Display session duration since connection established

### 5.8.2 Schema Inspection Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/describe <table>` | `\d` | Describe table structure | `/describe employees` |
| `/list databases` | `\l` | List all databases | `/list databases` |
| `/list tables` | `\dt` | List tables in current database | `/list tables` |
| `/list tables <pattern>` | `\dt` | List tables matching pattern | `/list tables emp%` |
| `/list views` | `\dv` | List views | `/list views` |
| `/list schemas` | `\dn` | List schemas | `/list schemas` |
| `/show indexes <table>` | `\di` | Show table indexes | `/show indexes employees` |

**`/describe` Metacommand Specification** (FR-115)

**Purpose**: Display comprehensive table structure information within REPL without writing SQL queries.

**Syntax**:
```
/describe <table_name>
/describe <database>.<table_name>
\d <table_name>                    -- Short alias
```

**Arguments**:
- `<table_name>`: Required. Name of table to describe
- `<database>.<table_name>`: Optional qualified name for cross-database lookup
- If database not specified, uses current connection database

**Data Source**:
Query Teradata system catalog views:
- `DBC.ColumnsV` - Column definitions, types, nullability
- `DBC.IndicesV` - Index information (primary keys, secondary indexes)
- `DBC.TablesV` - Table metadata (row count estimate, table kind)

**Output Format**:

```
tq> /describe employees

Table: PRODUCTION.employees
Type: Table
Created: 2024-01-10
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
│ created_at    │ TIMESTAMP    │ NO       │ CURRENT │          │
│ updated_at    │ TIMESTAMP    │ YES      │ NULL    │          │
└───────────────┴──────────────┴──────────┴─────────┴──────────┘

Indexes:
  PRIMARY KEY (employee_id)
  INDEX idx_dept (department_id)
  INDEX idx_manager (manager_id)
  INDEX idx_email (email) UNIQUE

Foreign Keys:
  department_id → departments(id)
  manager_id → employees(employee_id)

Statistics:
  Table Size: 4.2 MB
  Last Collected: 2024-01-15 10:30:00
```

**Type Formatting**:
Display full type specification:
- `INTEGER`, `BIGINT`, `SMALLINT` → As-is
- `DECIMAL(p,s)` → Show precision and scale
- `VARCHAR(n)`, `CHAR(n)` → Show length
- `TIMESTAMP`, `DATE`, `TIME` → As-is

**Nullable Column**:
- `NO` → Column is NOT NULL
- `YES` → Column allows NULL values

**Default Values**:
- `-` → No default value
- `NULL` → Explicit NULL default
- `CURRENT` → Current timestamp/date
- Other values → Show literal value (truncate if very long)

**Comments Column**:
- `PK` → Part of primary key
- `FK` → Foreign key reference
- `UQ` → Part of unique constraint
- Blank → No special constraints

**Error Handling**:

**Table not found**:
```
tq> /describe nonexistent_table
Error: Table 'nonexistent_table' does not exist in database 'production'

Suggestions:
  - Check spelling: /list tables non%
  - List all tables: /list tables
  - Try qualified name: /describe other_db.table_name
```

**Ambiguous table name (exists in multiple databases)**:
```
tq> /describe employees
Error: Table 'employees' exists in multiple databases:
  - production.employees
  - test.employees
  - staging.employees

Use qualified name: /describe production.employees
```

**Permission denied**:
```
tq> /describe secure_table
Error: Permission denied

Details: User 'alice' does not have SELECT privilege on 'secure_table'
Contact your database administrator to request access.
```

**Implementation Notes**:
- Cache table metadata for tab completion
- Support both `/describe` and `\d` aliases
- Handle case-insensitive table names (Teradata default)
- Show statistics only if available (don't fail if missing)
- Format numbers with thousands separators (42,573 not 42573)
- Truncate very wide output to terminal width

### 5.8.3 Data Sampling Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/sample <table> [n]` | Show random sample (default 10 rows) | `/sample employees 20` |
| `/peek <table>` | Show first 5 rows and column info | `/peek employees` |

**Example**:
```sql
tq> /sample employees 5
Random sample of 5 rows from employees:
[Results displayed in table format]
```

### 5.8.4 Export Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/export csv <file>` | Export last result to CSV | `/export csv employees.csv` |
| `/export json <file>` | Export last result to JSON | `/export json data.json` |

### 5.8.5 Session Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/session` | - | Show session info | `/session` |
| `/timing on` | `\t` | Enable query timing | `/timing on` |
| `/timing off` | `\t` | Disable query timing | `/timing off` |
| `/set format <fmt>` | - | Set output format | `/set format json` |
| `/set pager on` | - | Enable result paging | `/set pager on` |

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

### 5.8.6 Utility Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/help` | `\?` | Show help | `/help` |
| `/help <command>` | - | Show command help | `/help describe` |
| `/clear` | `\clear` | Clear screen | `/clear` |
| `/history` | - | Show command history | `/history` |
| `/edit` | `\e` | Edit last query in $EDITOR | `/edit` |
| `/quit` | `\q` | Exit REPL | `/quit` |

## 5.9 Special Features

### 5.9.1 Query Editing

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

### 5.9.2 Transaction Support

```sql
tq> BEGIN TRANSACTION;
tq(tx)> INSERT INTO employees VALUES (101, 'Test', 'User');
tq(tx)> SELECT * FROM employees WHERE id = 101;
tq(tx)> ROLLBACK;
tq> -- Transaction rolled back
```

Prompt changes to `tq(tx)>` when in transaction.

### 5.9.3 Query Cancellation

- **Ctrl-C**: Cancel running query gracefully
- **Double Ctrl-C**: Force quit (last resort)

**Feedback**:
```
Query running... (2.3s) [Press Ctrl-C to cancel]
^C
Query cancelled by user (after 2.3s)
```

### 5.9.4 Autocorrect Suggestions

```sql
tq> SELCT * FROM employees;
Error: Syntax error near "SELCT"
Did you mean: SELECT?

Fix and retry? [Y/n] y
[Executes corrected query]
```

---
