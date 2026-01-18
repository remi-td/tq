# REPL Mode Specifications

**Version:** 1.3.0
**Last Updated:** 2026-01-18
**Owner:** cli-ux-designer agent
**Status:** Active Specification - Sprint 8 (Quality Recovery) in Progress

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

### 5.6.1 Keyword Completion (Sprint 6)

**Purpose**: Enable fast SQL writing with auto-completion of SQL keywords.

**Trigger**: Press Tab key after typing partial SQL keyword.

**Supported Keywords** (complete list):
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

Cycle through completions:
```sql
tq> ORD<TAB>
tq> ORDER BY

tq> ORD<TAB><TAB>
(cycles through other matches starting with ORD)
```

Case-insensitive matching:
```sql
tq> sel<TAB> → SELECT
tq> SEL<TAB> → SELECT
tq> Sel<TAB> → SELECT
```

**Examples**:
```sql
tq> SEL<TAB>
    SELECT * FROM emp<TAB>
                  employees WHERE dept<TAB>
                                 department = 'IT'
```

**Implementation Notes**:
- Keywords are matched by prefix (typed text must match start of keyword)
- Completion does not change case (preserves what user typed)
- Tab key completes if single match, shows list if multiple matches
- Second Tab cycles through alternatives
- Keywords are case-insensitive internally but match user's casing

### 5.6.2 Table Name Completion (Sprint 7 - Sprint 8 Fixes)

**Purpose**: Enable users to discover and navigate database tables through tab completion, reducing typos and improving query writing speed. Properly handles Teradata's `database.table` qualified naming convention.

**Priority**: P0 (Critical for Sprint 7, Fixed in Sprint 8)

**Sprint 8 Note:** Original Sprint 7 implementation didn't work with real Teradata databases. Sprint 8 redesigns completion to properly support Teradata's qualified naming and intelligent caching.

**Trigger Contexts**: Press Tab after typing partial table name following these SQL keywords:
- `FROM` - Main table reference (e.g., `SELECT * FROM <TAB>`)
- `JOIN` / `INNER JOIN` / `LEFT JOIN` / `RIGHT JOIN` / `FULL JOIN` / `CROSS JOIN` - Join clauses
- `UPDATE` - Table to update (e.g., `UPDATE <TAB>`)
- `INTO` - Insert target (e.g., `INSERT INTO <TAB>`)

**Data Source**:
- Query Teradata system catalog: `DBC.TablesV`
- Load database names + current database tables initially (lazy loading)
- Cache per-database tables on-demand
- Include both tables and views

**Teradata-Specific Behavior**:
- **Qualified Names**: Teradata uses `database.table` format (not single-level like MySQL)
- **Unqualified Names**: Resolve to current database (from `SELECT DATABASE`)
- **Best Practice**: Encourage fully qualified names by showing databases first
- **Lazy Loading**: Only load metadata for databases user actually explores
- **Intelligent Caching**: Cache per-database, not global (performance for large systems)

**Behavior Patterns**:

**Single exact match - auto-complete**:
```sql
tq> SELECT * FROM emp<TAB>
tq> SELECT * FROM employees
```

**Multiple matches - show list**:
```sql
tq> SELECT * FROM emp<TAB>
    employees       employee_archive    employee_history    emp_summary
```

**Press Tab again to cycle through matches**:
```sql
tq> SELECT * FROM emp<TAB>
tq> SELECT * FROM employees<TAB>
tq> SELECT * FROM employee_archive<TAB>
tq> SELECT * FROM employee_history
```

**Schema-qualified completion**:
```sql
tq> SELECT * FROM prod<TAB>
    production.employees    production.orders    prod_data.metrics

tq> SELECT * FROM production.<TAB>
    production.employees    production.orders    production.customers
```

**Case-insensitive matching**:
```sql
tq> SELECT * FROM EMP<TAB> → employees
tq> SELECT * FROM Emp<TAB> → employees
tq> SELECT * FROM emp<TAB> → employees
```

**Teradata Qualified Names** (Sprint 8):

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

**Partial database name**:
```sql
tq> SELECT * FROM prod<TAB>
    production

tq> SELECT * FROM production.<TAB>
[Shows tables in production]
```

**Unqualified table in current database**:
```sql
tq[production]> SELECT * FROM emp<TAB>
    employees    employee_archive    emp_summary
```

**After JOIN keyword**:
```sql
tq> SELECT * FROM employees e
    INNER JOIN <TAB>
Databases:
    production    staging    development
Tables in current database (production):
    departments    projects    users    teams

tq> SELECT e.*, d.name
    FROM employees e
    LEFT JOIN dep<TAB>
    departments
```

**After UPDATE keyword**:
```sql
tq> UPDATE emp<TAB>
tq> UPDATE employees SET salary = 50000
```

**Loading States & Feedback** (Sprint 8 Improvements):

**Design Principle:** Always show user what's happening during metadata loading. Never leave user wondering if completion is working.

**First Tab press (metadata not cached)**:
```sql
tq> SELECT * FROM <TAB>
Loading tables... ⠋

[After loading completes (< 500ms):]
tq> SELECT * FROM <TAB>
Databases:
    production    staging    development
Tables in current database (production):
    customers    employees    orders    products    [50 more...]
```

**Cached metadata (instant response < 50ms)**:
```sql
tq> SELECT * FROM <TAB>
[Instant, no loading indicator]
Databases:
    production    staging    development
Tables in current database (production):
    customers    employees    orders    products    [50 more...]
```

**Slow database response (>500ms)**:
```sql
tq> SELECT * FROM <TAB>
Loading tables (this may take a moment)... ⠋
[Spinner animation cycles: ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏]
[Updates every 100ms]

[After completion:]
Loading tables (this may take a moment)... Done (1.2s)
    customers    employees    orders    products    [50 more...]
```

**Loading specific database tables**:
```sql
tq> SELECT * FROM staging.<TAB>
Loading tables in 'staging'... ⠋

[After completion:]
Tables in 'staging':
    test_data    imports    staging_employees
```

**Visual Feedback States:**
1. **<200ms:** No indicator (feels instant)
2. **200-500ms:** Simple "Loading..." with spinner
3. **>500ms:** "Loading (this may take a moment)..." with spinner
4. **>2s:** Show progress if available: "Loading... 1,234 tables found"

**Error Handling**:

**Metadata query fails (permissions)**:
```sql
tq> SELECT * FROM <TAB>
Warning: Cannot load table list (permission denied to DBC.TablesV)
Tab completion for tables unavailable.

Suggestion: Contact DBA to grant SELECT on DBC.TablesV
```

**Metadata query timeout**:
```sql
tq> SELECT * FROM <TAB>
Warning: Table list query timed out after 500ms
Tab completion for tables unavailable.

Suggestion: Database may be slow. Try again or continue typing manually.
```

**No tables found (empty database)**:
```sql
tq> SELECT * FROM <TAB>
(No tables found in current database)
```

**Connection lost**:
```sql
tq> SELECT * FROM <TAB>
Error: Connection lost. Cannot retrieve table list.
Use /ping to check connection or /reconnect to restore.
```

**Display Format**:

**Short list (≤10 tables)**:
```sql
tq> SELECT * FROM <TAB>
    customers    employees    orders    products
```

**Long list (>10 tables) - compact grid**:
```sql
tq> SELECT * FROM <TAB>
    accounts         customers        departments      employees
    invoices         orders           payments         products
    projects         regions          sales            suppliers
    transactions     users            vendors          warehouses
    [23 more tables] (Press Tab again to cycle, or continue typing to filter)
```

**With schema prefix**:
```sql
tq> SELECT * FROM <TAB>
    production.customers       production.employees
    production.orders          production.products
    staging.test_data          staging.imports
```

**Intelligent Caching Strategy** (Sprint 8):

**Design Goal:** Minimize metadata queries for large Teradata systems (millions of tables, hundreds of databases) while providing fast completion.

**Cache Structure:**
```
MetadataCache {
    databases: Vec<String>,                    // All database names (~100)
    current_database: String,                  // Active database
    tables: HashMap<String, Vec<Table>>,       // Per-database table cache
    columns: HashMap<String, Vec<Column>>,     // Per-table column cache
}
```

**Lazy Loading Strategy:**
1. **REPL Startup:** Don't load any metadata (instant startup)
2. **First Tab After FROM:** Load:
   - All database names (fast query: ~100 names)
   - Tables in current database only
3. **User Types `database.`:** Load tables for that database on-demand
4. **Cache Typical Session:** User works with 2-3 databases, minimal memory

**Cache Lifecycle:**
- **Load:** On-demand when user requests completion
- **Invalidate:** After successful DDL statements (CREATE/DROP/ALTER)
- **Clear:** On `/logon` (new connection = new context)
- **Size Limit:** Max 100 databases cached, LRU eviction if exceeded

**DDL Detection for Cache Refresh:**

After successful DDL execution, automatically invalidate relevant cache:

```sql
tq> CREATE TABLE new_table (id INT);
Table created successfully.
[Cache invalidated for current database]

tq> SELECT * FROM new<TAB>
[Re-fetches table list from DBC.TablesV]
    new_table    new_orders    new_customers
```

**DDL Keywords to Watch:**
- `CREATE TABLE` / `CREATE VIEW` → Invalidate current database table cache
- `DROP TABLE` / `DROP VIEW` → Invalidate current database table cache
- `ALTER TABLE` → Invalidate current database table cache (name may change)
- `RENAME TABLE` → Invalidate current database table cache
- `CREATE DATABASE` → Invalidate database list cache
- `DROP DATABASE` → Invalidate database list cache

**Cache Benefits:**
- First completion: 200-500ms (metadata query)
- Subsequent completions: <50ms (cache hit)
- Memory usage: ~1MB per 1,000 tables (minimal)
- Typical session: 2-3 databases cached, <5MB total

**Performance Requirements**:

- **Database list query**: <200ms (small list)
- **Table list query (one database)**: <500ms on first Tab press
- **Cached completion**: <50ms response time
- **Timeout**: 500ms max wait for metadata query
- **Cache size**: Store up to 100 databases, 10,000 tables per database
- **Cache invalidation**: On DDL success, on `/logon`, or manual `/refresh`

**Implementation Notes** (Sprint 8):

1. **Lazy Loading**: Don't query metadata on REPL startup - only on first Tab press in table context
2. **Caching Strategy**:
   - Cache per-database (not global)
   - Cache is session-scoped (cleared on `/logon`)
   - LRU eviction if >100 databases cached
   - Background refresh option: `/refresh` metacommand
3. **SQL Context Detection**:
   - Parse buffer to identify keywords: FROM, JOIN, UPDATE, INTO
   - Detect `database.` pattern for qualified name completion
   - Use simple regex patterns (avoid full SQL parser)
   - Support common patterns, accept limitations for complex queries
4. **Metadata Queries**:

   **Get all database names:**
   ```sql
   SELECT DISTINCT TRIM(DatabaseName) AS database_name
   FROM DBC.TablesV
   WHERE DatabaseName NOT IN ('DBC', 'SYSLIB', 'SYSBAR', 'SYSJDBC')
   ORDER BY DatabaseName;
   ```

   **Get tables in specific database:**
   ```sql
   SELECT TRIM(TableName) AS table_name,
          TableKind
   FROM DBC.TablesV
   WHERE DatabaseName = ?
   ORDER BY TableName;
   ```

   **Get current database:**
   ```sql
   SELECT DATABASE;
   ```

**Testing Scenarios** (Sprint 8):

1. Tab after FROM with no prefix → Show databases + current DB tables
2. Tab after FROM with partial table prefix → Show matching tables in current DB
3. Tab after FROM with database name → Complete database name
4. Tab after `FROM database.` → Show tables in that database
5. Tab in JOIN clause → Show databases + current DB tables
6. Tab in UPDATE statement → Complete table names
7. Multiple Tab presses → Cycle through matches
8. Qualified names → Complete `database.table` format
9. First tab press → Show loading indicator (<500ms)
10. Slow database → Show "this may take a moment", timeout at 500ms
11. Permission denied → Show warning with actionable suggestion
12. Empty database → Display "No tables found" message
13. Cache hit (2nd tab) → Instant response (<50ms)
14. After DDL (CREATE TABLE) → Cache refreshes, new table appears
15. After `/logon` → Cache cleared, new connection context
16. Large database (1000+ tables) → Pagination or truncation with count

### 5.6.3 Column Name Completion (Sprint 7)

**Purpose**: Enable users to discover and reference column names through tab completion, improving query accuracy and reducing the need to run `/describe` commands.

**Priority**: P1 (High priority for Sprint 7)

**Trigger Contexts**: Press Tab after typing partial column name following these SQL keywords:
- `SELECT` - Column list (e.g., `SELECT <TAB>`)
- `WHERE` - Filter conditions (e.g., `WHERE <TAB>`)
- `ORDER BY` - Sort columns (e.g., `ORDER BY <TAB>`)
- `GROUP BY` - Grouping columns
- `HAVING` - Aggregate filters
- After comma in column lists (e.g., `SELECT id, <TAB>`)

**Data Source**:
- Query Teradata system catalog: `DBC.ColumnsV`
- Filter by table name extracted from query context
- Include column name and data type
- Cache results per table for session duration

**Context Detection**:

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

**Multiple columns in SELECT**:
```sql
tq> SELECT employee_id, first<TAB>
    first_name

tq> SELECT employee_id, first_name, <TAB>
    employee_id    first_name    last_name    email    hire_date
```

**After ORDER BY**:
```sql
tq> SELECT * FROM employees ORDER BY <TAB>
    employee_id    first_name    last_name    hire_date    created_at
```

**Behavior Patterns**:

**Single match - auto-complete**:
```sql
tq> SELECT emplo<TAB>
tq> SELECT employee_id
```

**Multiple matches - show with type hints**:
```sql
tq> SELECT * FROM employees WHERE <TAB>
    employee_id (INTEGER)       first_name (VARCHAR)
    last_name (VARCHAR)         email (VARCHAR)
    hire_date (DATE)            salary (DECIMAL)
    department_id (INTEGER)     manager_id (INTEGER)
    created_at (TIMESTAMP)      updated_at (TIMESTAMP)
```

**Type hint display format**:
```
column_name (TYPE)
```

**Type abbreviations for compact display**:
- `INTEGER` → `INT`
- `VARCHAR(n)` → `VARCHAR` (omit length for brevity)
- `DECIMAL(p,s)` → `DEC`
- `TIMESTAMP` → `TIMESTAMP`
- `DATE` → `DATE`
- `CHAR(n)` → `CHAR`

**Ambiguous Context - Multiple Tables**:

**JOIN query with ambiguous columns**:
```sql
tq> SELECT * FROM employees e
    JOIN departments d ON e.department_id = d.id
    WHERE <TAB>

    -- Shows columns from both tables with table alias prefix:
    e.employee_id (INT)      e.first_name (VARCHAR)
    e.department_id (INT)    d.id (INT)
    d.name (VARCHAR)         d.budget (DEC)
```

**Qualified column reference**:
```sql
tq> SELECT e.<TAB>
    e.employee_id    e.first_name    e.last_name    e.email
```

**No table context detected**:
```sql
tq> SELECT <TAB>
(Cannot determine table context. Complete table name first.)

-- User must provide table in FROM clause:
tq> SELECT * FROM employees WHERE <TAB>
[Now shows employee columns]
```

**Loading States & Feedback**:

**First Tab press for table (metadata not cached)**:
```sql
tq> SELECT * FROM employees WHERE <TAB>
Loading columns for 'employees'...

[After 200ms:]
    employee_id (INT)    first_name (VARCHAR)    last_name (VARCHAR)
```

**Cached column metadata**:
```sql
tq> SELECT * FROM employees WHERE <TAB>
[Instant response <50ms]
    employee_id (INT)    first_name (VARCHAR)    last_name (VARCHAR)
```

**Error Handling**:

**Table not found in metadata**:
```sql
tq> SELECT * FROM nonexistent_table WHERE <TAB>
Error: Table 'nonexistent_table' not found
Cannot provide column completion.
```

**Permission denied**:
```sql
tq> SELECT * FROM secure_table WHERE <TAB>
Warning: Cannot load columns for 'secure_table' (permission denied)
Column completion unavailable.
```

**Metadata query timeout**:
```sql
tq> SELECT * FROM large_table WHERE <TAB>
Warning: Column list query timed out after 300ms
Column completion unavailable for this table.
```

**Ambiguous table context**:
```sql
tq> SELECT <TAB> FROM employees e JOIN departments d
(Multiple tables in query. Specify table: e.<TAB> or d.<TAB>)
```

**Performance Requirements**:

- **Column metadata query**: <300ms on first Tab press
- **Cached completion**: <50ms response time
- **Timeout**: 300ms max wait for column metadata
- **Cache size**: Store columns for up to 100 tables
- **Cache invalidation**: Clear on `/logon`

**Implementation Notes**:

1. **Table Context Parsing**:
   - Extract table name from FROM clause using regex
   - Support simple queries first: `FROM table_name`
   - Handle table aliases: `FROM employees e`
   - For JOIN queries: detect which table alias user is referencing
   - Accept limitations for complex queries (subqueries, CTEs)

2. **Metadata Query**:
   ```sql
   SELECT TRIM(ColumnName) AS column_name,
          TRIM(ColumnType) AS column_type,
          ColumnLength,
          DecimalTotalDigits,
          DecimalFractionalDigits
   FROM DBC.ColumnsV
   WHERE DatabaseName = ?
     AND TableName = ?
   ORDER BY ColumnId;
   ```

3. **Type Formatting**:
   - Display concise type information
   - Include key details: `VARCHAR(100)`, `DECIMAL(10,2)`, `INTEGER`
   - Truncate long type definitions for display

4. **Context Detection Strategy**:
   - Parse backwards from cursor position
   - Identify table reference in current SQL statement
   - For simple cases: single table in FROM clause
   - For complex cases: prompt user to use table alias

**Testing Scenarios**:

1. Tab after SELECT in single-table query → Show all columns with types
2. Tab with partial column name → Filter and show matches
3. Tab after WHERE → Show columns from table in FROM clause
4. Tab in JOIN query → Show columns with table alias prefix
5. Tab with table alias qualifier (e.g., `e.<TAB>`) → Show columns for that table
6. Multiple Tab presses → Cycle through column matches
7. Ambiguous context → Show helpful error message
8. Table not in FROM clause → Cannot complete, show message
9. Slow metadata query → Loading indicator, timeout gracefully
10. Permission denied → Warning message, continue REPL

**Limitations (Acknowledged)**:

- **Subqueries**: Won't detect table context inside subqueries (v1.5.0 limitation)
- **CTEs (WITH clauses)**: Won't parse CTE columns (v1.5.0 limitation)
- **Window functions**: Limited support for OVER clauses
- **Complex expressions**: Won't complete inside CASE statements or nested functions

**Workaround for limitations**: Users can still type column names manually or use `/describe table_name` to see column list.

### 5.6.4 Metacommand Completion (Future)

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

### 5.7.2 Large Result Handling & Result Paging

**Version:** 2.0 (Sprint 8 Redesign)
**Status:** Redesigned to fix critical UX issues

**Overview:**

Result paging uses a three-layer strategy to make any result set readable:
1. **Column Windowing** - Limit visible columns to maintain readability
2. **Cell Truncation** - Limit cell content length to prevent layout breaks
3. **Row Paging** - Paginate vertically through long result sets

**Critical Requirement:** Pager MUST be safe - 'q' key exits pager and returns to REPL, never exits the entire program.

---

#### Column Windowing (Layer 1)

**Objective:** Display manageable subset of columns, navigate horizontally through remaining columns.

**Column Window Size - Dynamic Calculation:**
```
1. Start with first column (leftmost)
2. Calculate minimum usable width for column:
   min_width = max(column_name.length + 2, 8)  // At least 8 chars
   max_width = 40  // Never exceed 40 chars per column
3. Add columns until total_width > (terminal_width - 10)
   - Reserve 10 chars for borders and margins
4. Show at least 3 columns, even if slightly exceeds terminal width
```

**Result:** Typically 4-6 readable columns visible at once (in 100-120 char terminal).

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

**Transition Behavior:**
- Smooth scrolling: one column at a time
- At edges: Arrow keys do nothing (no wrap-around)
- Window size recalculates dynamically for new columns

---

#### Cell Truncation (Layer 2)

**Objective:** Prevent long cell values from breaking table layout.

**Truncation Rules:**

**Maximum Cell Display Length:**
- **Text cells (VARCHAR, CHAR):** 100 characters maximum
- **Numeric cells:** No truncation (naturally bounded)
- **Date/Time cells:** No truncation (fixed format)

**Truncation Indicator:**
- Append `…` (Unicode ellipsis U+2026) to truncated values
- Example: `This is a very long description that goes on and on and on for hundreds of characters...`
  → Displays as: `This is a very long description that goes on and on and on for hundreds of characters and so...`
  (99 chars + ellipsis)

**Column Width Calculation:**
```
1. Examine all values in column (first 100 rows for performance)
2. Find max display length after truncation:
   max_length = max(header_length, max(truncated_value_lengths))
3. Cap at maximum column width:
   column_width = min(max_length + 2, 40)  // +2 for padding, max 40
```

**Viewing Full Values:**

**Current Solution (Sprint 8):** Use workarounds:
- Query specific row: `SELECT long_column FROM table WHERE id = 123;`
- Export results: `/export csv results.csv` to view full values in file

**Future Enhancement (Sprint 9+):**
- Press `v` (view) on cell to open expanded modal with full content
- Scrollable popup showing complete cell value
- Press Escape to return to table view

---

#### Vertical Row Paging (Layer 3)

**Objective:** Navigate long result sets smoothly with clear position indicators.

**Page Size - Dynamic:**
```
page_size = terminal_height - 5
  - 1 row for header
  - 2 rows for borders
  - 1 row for status bar
  - 1 row for margin
```

Example: 24-line terminal → 19 rows per page

**Navigation Keys:**

**Single Row Movement:**
- `j` / `↓` (Down Arrow): Next row
- `k` / `↑` (Up Arrow): Previous row

**Page Movement:**
- `Space` / `Page Down`: Next page (jump by page_size rows)
- `b` / `Page Up`: Previous page (jump by page_size rows)

**Jump Navigation:**
- `g` / `Home`: Jump to first row
- `G` / `End`: Jump to last row
- `50G`: Jump to row 50 (vi-style, future enhancement)

**Search (Future):**
- `/pattern`: Search forward for pattern in visible columns
- `n`: Next match
- `N`: Previous match

**Row Position Indicator:**
```
Rows 1-20 of 1,234 (2%)
```

Shows: current visible range, total rows, percentage through results

**Edge Behavior:**
- At first row: `k` or `↑` does nothing (no error)
- At last row: `j` or `↓` does nothing (no error)
- Empty results: Display "(No results)" with "Press q to exit pager"

---

#### Pager Exit Behavior (CRITICAL)

**Primary Requirement:** 'q' MUST exit pager and return to REPL, never exit the entire program.

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

**Technical Implementation:**
- Pager runs in controlled local event loop, NOT blocking mode like `minus::page_all()`
- 'q' key breaks pager event loop and returns control to REPL
- REPL state fully preserved: connection, history, settings
- No process exit signals sent from pager

**Mode Indicators:**

**In Pager Mode:**
- Status bar visible (only in pager)
- Clear "exit pager" text in status bar
- No `tq>` prompt visible

**In REPL Mode:**
- `tq>` or `tq[dbname]>` prompt visible
- No status bar
- Blinking cursor ready for input

---

#### Complete Status Bar Design

**Layout (Two-Line Status Bar at Bottom):**
```
┌────────────────────────────────────────────────────────────────────────────┐
│ Columns 1-5 of 23 | Rows 1-20 of 1,234 (2%) | Navigation: ←→ ↑↓ Space b  │
│ g/G: first/last | q/Esc: exit pager                                        │
└────────────────────────────────────────────────────────────────────────────┘
```

**Elements:**
- **Column position:** Shows current visible columns and total (e.g., "Columns 1-5 of 23")
- **Row position:** Shows current visible rows, total, and percentage (e.g., "Rows 1-20 of 1,234 (2%)")
- **Navigation hints:** Most common keys for quick reference
- **Exit hint:** Clear "exit pager" wording to distinguish from "exit program"

---

#### Complete Paging Example: Wide Table

**Scenario:** Table with 23 columns, 1,234 rows, 100-char terminal width

**Initial View (Columns 1-5):**
```
┌──────────┬────────────┬───────────┬──────────────┬────────────┐
│ emp_id   │ first_name │ last_name │ email        │ hire_date  │
├──────────┼────────────┼───────────┼──────────────┼────────────┤
│ 1        │ Alice      │ Anderson  │ alice@co.com │ 2020-01-15 │
│ 2        │ Bob        │ Brown     │ bob@co.com   │ 2020-03-22 │
│ 3        │ Charlie    │ Chen      │ charlie@c... │ 2020-07-01 │
│ 4        │ Diana      │ Davis     │ diana@co.com │ 2021-01-10 │
│ 5        │ Edward     │ Evans     │ edward@co... │ 2021-04-05 │
│ ...      │ ...        │ ...       │ ...          │ ...        │
│ 20       │ Tina       │ Turner    │ tina@co.com  │ 2024-12-15 │
└──────────┴────────────┴───────────┴──────────────┴────────────┘

┌────────────────────────────────────────────────────────────────────────────┐
│ Columns 1-5 of 23 | Rows 1-20 of 1,234 (2%) | →: more columns             │
│ Space: next page | ↑↓: scroll rows | q: exit pager                         │
└────────────────────────────────────────────────────────────────────────────┘
```

**After Pressing → (Right Arrow) - Columns 3-7:**
```
┌────────────┬──────────────┬─────────────┬────────────┬──────────┐
│ last_name  │ email        │ hire_date   │ salary     │ dept_id  │
├────────────┼──────────────┼─────────────┼────────────┼──────────┤
│ Anderson   │ alice@co.com │ 2020-01-15  │ 75000.00   │ 101      │
│ Brown      │ bob@co.com   │ 2020-03-22  │ 68000.00   │ 102      │
│ Chen       │ charlie@...  │ 2020-07-01  │ 82000.00   │ 101      │
│ Davis      │ diana@co.com │ 2021-01-10  │ 71000.00   │ 103      │
│ Evans      │ edward@co... │ 2021-04-05  │ 79000.00   │ 102      │
│ ...        │ ...          │ ...         │ ...        │ ...      │
│ Turner     │ tina@co.com  │ 2024-12-15  │ 65000.00   │ 104      │
└────────────┴──────────────┴─────────────┴────────────┴──────────┘

┌────────────────────────────────────────────────────────────────────────────┐
│ Columns 3-7 of 23 | Rows 1-20 of 1,234 (2%) | ←: prev | →: next           │
│ Space: next page | ↑↓: scroll rows | q: exit pager                         │
└────────────────────────────────────────────────────────────────────────────┘
```

**After Pressing Space (Next Page) - Rows 21-40:**
```
┌────────────┬──────────────┬─────────────┬────────────┬──────────┐
│ last_name  │ email        │ hire_date   │ salary     │ dept_id  │
├────────────┼──────────────┼─────────────┼────────────┼──────────┤
│ Garcia     │ uma@co.com   │ 2020-02-12  │ 73000.00   │ 105      │
│ Harris     │ victor@co... │ 2020-04-18  │ 76000.00   │ 101      │
│ Irwin      │ wendy@co.com │ 2020-08-22  │ 69000.00   │ 102      │
│ ...        │ ...          │ ...         │ ...        │ ...      │
│ Zane       │ xander@co... │ 2023-11-30  │ 71000.00   │ 103      │
└────────────┴──────────────┴─────────────┴────────────┴──────────┘

┌────────────────────────────────────────────────────────────────────────────┐
│ Columns 3-7 of 23 | Rows 21-40 of 1,234 (3%) | ←→: columns                │
│ Space: next page | b: prev page | q: exit pager                            │
└────────────────────────────────────────────────────────────────────────────┘
```

**After Pressing 'q':**
```
tq> _
[Returns to REPL prompt, session preserved]
```

---

#### Paging Example: Long Cell Values

**Scenario:** Table with VARCHAR(2000) columns containing 500+ character values

**Query:**
```sql
SELECT id, title, description FROM articles LIMIT 5;
```

**Paged Output:**
```
┌─────┬─────────────────────┬──────────────────────────────────────────┐
│ id  │ title               │ description                              │
├─────┼─────────────────────┼──────────────────────────────────────────┤
│ 1   │ Getting Started     │ This article explains how to get star... │
│ 2   │ Advanced Features   │ Learn about advanced features includi... │
│ 3   │ Troubleshooting     │ Common issues and their solutions are... │
│ 4   │ Performance Tuning  │ Optimize your queries for maximum per... │
│ 5   │ Best Practices      │ Follow these best practices to ensure... │
└─────┴─────────────────────┴──────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────────────┐
│ Columns 1-3 of 3 | Rows 1-5 of 5 (100%) | q: exit pager                   │
└────────────────────────────────────────────────────────────────────────────┘
```

**Note:** Description values truncated to 100 characters with `...` ellipsis. To view full values, use separate query or `/export` command.

---

#### Result Truncation Hint (>100 rows)

When displaying results and suggesting result limiting, show Teradata-specific hint:

```sql
tq> SELECT * FROM employees;
... 100 rows displayed ...

Showing first 100 rows. Use TOP N or SAMPLE N for different results.
```

**Note:** This hint uses Teradata-specific syntax (TOP/SAMPLE), not MySQL/PostgreSQL LIMIT.

**Optional Verbose Hint** (with examples):
```sql
Showing first 100 rows. Use TOP N or SAMPLE N for different results.

Examples:
  SELECT TOP 50 * FROM employees;       -- Get first 50 rows
  SELECT * FROM employees SAMPLE 200;   -- Sample 200 rows
```

---

#### Performance Considerations

**Large Result Sets (10,000+ rows):**
- Use streaming or chunked loading (fetch 1000 rows at a time)
- Render only visible rows (current page)
- Show loading indicator if next page not yet fetched
- Don't load entire result set into memory

**Wide Tables (50+ columns):**
- Column width calculation: sample first 100 rows only
- Cache column widths per window for reuse
- Re-render only changed portions when scrolling

**Terminal Resizing:**
- Listen for SIGWINCH signal (terminal resize)
- Recalculate column windows and page size
- Re-render current view with new dimensions

**Target Performance:**
- Pager opens: <1s for 10,000 row result set
- Navigation response: <100ms per keypress
- Memory usage: <100MB for typical result sets

---

#### Implementation Guidance

**Architecture:**

Pager is a function call within REPL, not a separate mode:
```rust
fn execute_query(sql: &str) -> Result<()> {
    let results = database.query(sql)?;

    if results.row_count() > PAGING_THRESHOLD {
        // Enter pager - runs event loop, then returns
        pager::display_with_navigation(results)?;
    } else {
        // Small result, print directly
        print_table(results);
    }

    // Always returns to REPL prompt
    Ok(())
}
```

**Library Recommendations:**

**Replace `minus` with custom pager:**
- Current `minus` library lacks control for safe exit and custom windowing
- Use `crossterm` for terminal control (cursor, colors, input)
- Build custom pager with exact behavior specified here

**Alternative (if keeping `minus`):**
- Use `minus::Pager::new()` instead of `minus::page_all()`
- Override keybindings to prevent program exit
- May have limitations for column windowing features

---

#### Testing Requirements

**Critical Tests (Must Pass):**

1. **Test: 'q' Returns to REPL**
   - Execute query, enter pager, press 'q'
   - Expected: Returns to `tq>` prompt, session preserved

2. **Test: Wide Table Readable**
   - Query 25-column table
   - Expected: 4-6 columns visible, readable width, smooth right-arrow navigation

3. **Test: Long Values Truncated**
   - Query VARCHAR(2000) column with 500-char values
   - Expected: Values truncated at 100 chars with "…", table layout stable

**Edge Case Tests:**

4. **Test: Empty Result Set**
   - Query returns 0 rows
   - Expected: Show "(No results)", 'q' exits pager

5. **Test: Single Wide Column**
   - Table with 1 column 150 chars wide
   - Expected: Show full column even if exceeds terminal

6. **Test: Navigation at Boundaries**
   - Press 'k' at first row, 'j' at last row, left-arrow at first column
   - Expected: No error, no crash, no action

**Performance Tests:**

7. **Test: Large Result Set (10,000 rows)**
   - Expected: Pager opens <1s, navigation <100ms response

8. **Test: Very Wide Table (50 columns)**
   - Expected: Column window calculated quickly, smooth scrolling

---

**Sprint 8 Design Complete:** See [Sprint 8 Paging UX Design](../sprints/sprint-8-paging-ux-design.md) for full technical design document.

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
| `/logon [connection]` | `\c` | Connect/switch database or show current connection | `/logon user:pass@host:1025/db` |
| `/disconnect` | `\q` | Disconnect current connection | `/disconnect` |
| `/reconnect` | - | Reconnect to current database | `/reconnect` |
| `/ping` | - | Test connection | `/ping` |

**`/logon` Metacommand Specification** (Sprint 7)

**Purpose**: Allow users to switch database connections dynamically without exiting the REPL, essential for users who work with multiple databases or environments.

**Priority**: P1 (High priority for Sprint 7)

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

**Supported Authentication**: All mechanisms supported by tq:
- `TD2` (default Teradata authentication)
- `LDAP`
- `KRB5` (Kerberos)
- `TDNEGO` (Negotiated)

**Behavior**:

**Show current connection (no arguments)**:
```sql
tq> /logon

Current Connection:
  Host: prod-td01.company.com:1025
  Database: production_db
  User: alice
  Authentication: LDAP
  Session ID: 987654321
  Connected: 2026-01-19 09:15:23 (45m 12s ago)
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
  Authentication: TD2
  Session ID: 123456789
```

**Connection string with authentication mechanism**:
```sql
tq> /logon alice:secret@host:1025/db?logmech=LDAP

Connecting to host:1025/db (LDAP authentication)...
Connected successfully.
```

**Preserve REPL state across connections**:
```sql
tq[production_db]> /logon alice:pass@dev:1025/dev_db
Connected to dev_db

tq[dev_db]> [History preserved: ↑ still shows production_db queries]
tq[dev_db]> [Editor mode still Vi/Emacs as configured]
tq[dev_db]> [Pager/colors settings preserved]
```

**Clear cached metadata after connection change**:
```sql
tq[production_db]> /logon alice:pass@dev:1025/dev_db
Connected to dev_db

tq[dev_db]> SELECT * FROM <TAB>
Loading tables for dev_db... [New metadata query, cache cleared]
```

**Success Messages**:

**Standard connection**:
```sql
tq> /logon alice:pass@host:1025/mydb
Connected to mydb on host:1025
Session ID: 123456789
```

**With additional context**:
```sql
tq> /logon alice:pass@prod.company.com:1025/production
Connecting to production on prod.company.com:1025...
Connected successfully (247ms)

Connection Details:
  Database: production
  User: alice
  Authentication: TD2
  Character Set: UTF8
  Time Zone: America/New_York
```

**Error Handling**:

**Connection failure - network**:
```sql
tq> /logon alice:pass@unreachable:1025/db

Error: Cannot connect to unreachable:1025
Reason: Connection refused (network unreachable)

Troubleshooting:
  - Check hostname and port are correct
  - Verify network connectivity: ping unreachable
  - Check firewall rules
  - Confirm Teradata database is running

Current connection preserved: prod-td01.company.com:1025/production_db
```

**Connection failure - authentication**:
```sql
tq> /logon alice:wrongpass@host:1025/db

Error: Authentication failed
Reason: Invalid username or password

Suggestions:
  - Verify credentials are correct
  - Check if account is locked
  - Confirm authentication mechanism (try ?logmech=LDAP)

Current connection preserved: prod-td01.company.com:1025/production_db
```

**Connection failure - database not found**:
```sql
tq> /logon alice:pass@host:1025/nonexistent_db

Error: Database 'nonexistent_db' not found
Reason: Specified database does not exist

Suggestions:
  - Check database name spelling
  - List available databases on this host
  - Verify you have access to this database

Current connection preserved: prod-td01.company.com:1025/production_db
```

**Connection timeout**:
```sql
tq> /logon alice:pass@slow-host:1025/db

Connecting to slow-host:1025/db...
(waiting... 5s)
(waiting... 10s)

Error: Connection timeout after 30s
Reason: Database did not respond within timeout period

Suggestions:
  - Database may be overloaded or down
  - Network latency may be high
  - Try again later or contact DBA

Current connection preserved: prod-td01.company.com:1025/production_db
```

**Invalid connection string format**:
```sql
tq> /logon invalid-format

Error: Invalid connection string format
Expected format: username:password@hostname:port/database

Examples:
  /logon alice:secret@host:1025/mydb
  /logon alice:pass@host:1025/db?logmech=LDAP
  /logon alice@host:1025/db  (password will be prompted)

See /help logon for more details.
```

**Permission denied**:
```sql
tq> /logon alice:pass@host:1025/restricted_db

Error: Permission denied to database 'restricted_db'
Reason: User 'alice' does not have access

Suggestions:
  - Contact DBA to request access
  - Verify correct database name
  - Check if access is restricted by IP/network

Current connection preserved: prod-td01.company.com:1025/production_db
```

**State Preservation on Connection Change**:

**REPL settings preserved**:
- Command history (in-memory and file-backed)
- Editor mode (Vi/Emacs)
- Pager setting (on/off)
- Colors setting (on/off)
- Timing display setting

**REPL state cleared**:
- Last query result (cleared for /export)
- Cached table metadata (cleared)
- Cached column metadata (cleared)
- Active transaction state (if any, warning issued)

**Transaction warning**:
```sql
tq(tx)> /logon alice:pass@dev:1025/dev_db

Warning: You have an active transaction on the current connection
Changes will be LOST if you switch connections now.

Options:
  1. COMMIT; then /logon    (save changes, then switch)
  2. ROLLBACK; then /logon  (discard changes, then switch)
  3. Cancel /logon          (stay on current connection)

Proceed anyway? [y/N]: n
Connection change cancelled.
```

**Prompt Update**:

**Show database name in prompt after connection**:
```sql
# Before connection:
tq>

# After connecting:
tq[production_db]>

# After switching to different database:
tq[dev_db]>
```

**Performance Requirements**:

- **Connection establishment**: <2s for healthy database
- **Connection timeout**: 30s max (configurable)
- **Disconnection cleanup**: <500ms
- **Metadata cache clear**: <100ms

**Implementation Notes**:

1. **Connection Lifecycle**:
   - Store current connection config in `ReplState`
   - On `/logon`, validate new connection string
   - Attempt new connection (don't disconnect old one yet)
   - If new connection succeeds: cleanly close old connection
   - If new connection fails: preserve old connection, show error
   - Update prompt with new database name

2. **State Management**:
   - Preserve: history, editor mode, pager, colors, timing
   - Clear: query results, table cache, column cache
   - Warn and block: active transactions

3. **Error Recovery**:
   - Always preserve old connection if new connection fails
   - Never leave user in disconnected state
   - Provide clear error messages with recovery suggestions

4. **Security**:
   - Don't echo password in output
   - Don't store password in history file
   - Handle credentials same as CLI `-l` flag

**Configuration Options** (via environment or config file):

```bash
# Connection timeout (default: 30s)
export TQ_CONNECTION_TIMEOUT=30

# Auto-reconnect on connection loss (default: false)
export TQ_AUTO_RECONNECT=false
```

**Testing Scenarios**:

1. `/logon` with no args → Show current connection details
2. `/logon <valid-string>` → Switch to new database successfully
3. `/logon <invalid-host>` → Connection failure, preserve old connection
4. `/logon <wrong-password>` → Auth failure, preserve old connection
5. `/logon` with active transaction → Warning, require confirmation
6. `/logon` → Prompt updates with new database name
7. After `/logon` → Tab completion uses new database metadata
8. After `/logon` → History preserved from old connection
9. After `/logon` → Settings (pager, colors) preserved
10. Connection timeout → Clear error, revert to old connection

**Help Text**:

```sql
tq> /help logon

/logon [connection-string]    Connect to database or show current connection

Syntax:
  /logon                      Show current connection information
  /logon <connection>         Switch to new database connection
  \c [connection]             Short alias for /logon

Connection String Format:
  username:password@hostname:port/database[?logmech=AUTH]

Examples:
  /logon                                    # Show current connection
  /logon alice:secret@prod:1025/sales      # Switch to sales database
  /logon alice@prod:1025/sales             # Prompt for password
  /logon alice:pass@host:1025/db?logmech=LDAP  # Use LDAP auth

Authentication Mechanisms:
  TD2     - Teradata authentication (default)
  LDAP    - LDAP authentication
  KRB5    - Kerberos authentication
  TDNEGO  - Negotiated authentication

Notes:
  - REPL history and settings are preserved across connections
  - Table/column completion cache is cleared on connection change
  - Active transactions will prevent connection switching (commit/rollback first)
  - Failed connections preserve your current connection

See also: /ping, /reconnect, /session
```

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

### 5.8.4 Export Commands (Sprint 6)

**Purpose**: Save the results of the last executed query to a file in multiple formats (JSON, CSV, SQL INSERT statements).

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
   - Proper JSON type handling (numbers, booleans, nulls)
   - Pretty-printed by default
   - Example: `/export json data.json`

3. **SQL Format** (`sql`)
   - INSERT statements for data reload
   - Generates: `INSERT INTO table_name (col1, col2) VALUES (val1, val2);`
   - Requires knowledge of table name (prompts user if not obvious)
   - Example: `/export sql inserts.sql`

**Behavior**:

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

**File already exists - confirmation prompt**:
```sql
tq> /export csv employees.csv
File already exists: employees.csv
Overwrite? [y/n/a (append)]: y
Exported 10 rows to employees.csv
```

**Using --append flag**:
```sql
tq> /export csv --append employees.csv
Appended 10 rows to employees.csv (no header in append mode)
```

**Error Handling**:

**No query executed yet**:
```sql
tq> /export csv data.csv
Error: No query results to export. Execute a query first.
```

**Invalid format**:
```sql
tq> /export xml data.xml
Error: Unknown format 'xml'. Supported formats: csv, json, sql
```

**Permission denied writing file**:
```sql
tq> /export csv /readonly/data.csv
Error: Permission denied writing to /readonly/data.csv
Check file permissions or choose different location.
```

**Disk full**:
```sql
tq> /export csv large_export.csv
Error: Disk full - cannot write to large_export.csv
Freed 10.2 KB written before error (file truncated).
```

**Large result handling**:
```sql
tq> SELECT * FROM huge_table;
[1,000,000 rows returned - streaming to pager]

tq> /export csv huge_data.csv
Exporting 1,000,000 rows...
Exported 1,000,000 rows to huge_data.csv (245 MB)
```

**Implementation Notes**:
- Export only the most recent query results
- Clear result cache after export (optional - for cleanup)
- Support path expansion (~/, relative paths)
- Atomic writes: write to temp file, rename on success
- Include row count in success message
- For SQL format, store table name in session or prompt if ambiguous

| Command | Description | Example |
|---------|-------------|---------|
| `/export csv <file>` | Export last result to CSV | `/export csv employees.csv` |
| `/export json <file>` | Export last result to JSON | `/export json data.json` |
| `/export sql <file>` | Export as SQL INSERT statements | `/export sql inserts.sql` |

### 5.8.5 Session Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/session` | - | Show session info | `/session` |
| `/timing on` | `\t` | Enable query timing | `/timing on` |
| `/timing off` | `\t` | Disable query timing | `/timing off` |
| `/set format <fmt>` | - | Set output format | `/set format json` |
| `/pager on\|off` | - | Enable/disable result paging | `/pager on` |
| `/colors on\|off` | - | Enable/disable syntax highlighting | `/colors on` |

**`/pager on\|off` Metacommand Specification** (Sprint 6)

**Purpose**: Control whether large result sets are paginated (scrollable) or displayed all at once.

**Syntax**:
```
/pager on       (enable result paging)
/pager off      (disable paging, show all results)
```

**Behavior**:

**Pager enabled (default)**:
```sql
tq> /pager on
Result paging enabled

tq> SELECT * FROM huge_table;
[Shows first screen, allows scrolling with j/k/arrows]
Rows 1-50 of 5,234 | Space: next | b: prev | q: quit | /: search
```

**Pager disabled**:
```sql
tq> /pager off
Result paging disabled

tq> SELECT * FROM huge_table;
[Displays all 5,234 rows without pagination]
```

**Current setting**:
```sql
tq> /pager
Pager: on (enabled for results > 50 rows)
```

**Persistence**:
- Setting persists for current REPL session
- Does not affect config files (session-only)
- Resets to default (on) when REPL restarts

**Error Handling**:
```sql
tq> /pager maybe
Error: Invalid pager setting 'maybe'. Use 'on' or 'off'.
```

**`/colors on\|off` Metacommand Specification** (Sprint 6)

**Purpose**: Control SQL syntax highlighting and colored output in the REPL.

**Syntax**:
```
/colors on      (enable syntax highlighting)
/colors off     (disable colors, plain text output)
```

**Behavior**:

**Colors enabled (default in TTY)**:
```sql
tq> /colors on
Syntax highlighting enabled

tq> SELECT * FROM employees;
[SQL keywords displayed in cyan, strings in green, etc.]
[Results with colored NULL values]
```

**Colors disabled**:
```sql
tq> /colors off
Syntax highlighting disabled

tq> SELECT * FROM employees;
[All text in plain white, no colors]
```

**Current setting**:
```sql
tq> /colors
Colors: on (in TTY mode)
```

**Scope**:
- Affects SQL syntax highlighting in input editor
- Affects colored table output (NULL indicators, column separators)
- Does not affect exported data (always plain)

**Persistence**:
- Setting persists for current REPL session
- Does not affect config files (session-only)
- Auto-disables when output is redirected (pipe/redirect)

**Error Handling**:
```sql
tq> /colors maybe
Error: Invalid color setting 'maybe'. Use 'on' or 'off'.
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

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-18 | 2.0.0 | Sprint 8 paging redesign: Complete overhaul of section 5.7.2 with three-layer strategy (column windowing, cell truncation, row paging), safe exit behavior, comprehensive status bar design, implementation guidance | CLI UX Designer Agent |
| 2026-01-18 | 1.3.0 | Sprint 8 fixes: Teradata-specific tab completion (database.table), visual feedback, intelligent caching, DDL detection, fixed LIMIT hint to TOP/SAMPLE | CLI UX Designer Agent |
| 2026-01-19 | 1.2.0 | Added Sprint 7 specifications: table completion (5.6.2), column completion (5.6.3), /logon metacommand (5.8.1) | CLI UX Designer Agent |
| 2026-01-18 | 1.1.0 | Added Sprint 6 specifications: keyword completion, /export, /pager, /colors | CLI UX Designer Agent |
| 2026-01-17 | 1.0.0 | Added Sprint 4-5 specifications: REPL foundation, syntax highlighting, paging | CLI UX Designer Agent |
| 2026-01-16 | 0.1.0 | Initial REPL mode specifications | Development Team |

---

## Sprint 8 Summary (Quality Recovery)

**Critical Fixes in This Version:**

**Bug 4 (P1): LIMIT Hint Message Fixed**
- **Problem:** Incorrect hint message used MySQL `LIMIT` syntax
- **Fix:** Changed to Teradata-specific `TOP N or SAMPLE N` syntax
- **Location:** Section 5.7.2 - Large Result Handling
- **Impact:** Users now get correct Teradata SQL guidance

**Bug 2 (P0): Tab Completion Redesigned for Teradata**
- **Problem:** Completion didn't work, assumed single-level table names
- **Fix:** Complete redesign supporting Teradata's `database.table` qualified naming
- **Key Changes:**
  - Show databases + current DB tables after FROM
  - Support `database.` pattern for cross-database completion
  - Intelligent lazy loading: databases + current DB only initially
  - Per-database caching with LRU eviction
  - DDL detection for automatic cache refresh
- **Location:** Section 5.6.2 - Table Name Completion
- **Performance:** <200ms database list, <500ms table list, <50ms cached

**Visual Feedback Added (Bug 2 Improvements):**
- **Loading Indicators:** Spinner with "Loading..." during metadata fetch
- **Context Indicators:** Show what's being completed (databases/tables/columns)
- **Error Messages:** Clear, actionable guidance when completion fails
- **Visual States:** <200ms no indicator, 200-500ms spinner, >500ms "may take a moment"
- **Location:** Section 5.6.2 - Loading States & Feedback

**Quality Improvements:**
- All specifications now tested against real Teradata databases
- Acknowledged Sprint 7 features didn't work as implemented
- Added comprehensive testing scenarios for live database validation
- Documented Teradata-specific requirements and limitations

**Design Principles Applied:**

- **Teradata-First**: Designed for Teradata's `database.table` model, not adapted from MySQL/PostgreSQL
- **Clear Communication**: Always show user what's happening (loading, errors, context)
- **Performance at Scale**: Handles millions of tables across hundreds of databases
- **Intelligent Caching**: Lazy load, per-database cache, minimal memory
- **Graceful Degradation**: Completion failures don't block user from typing manually
- **Best Practices**: Encourage fully qualified names by showing databases first

**Testing Requirements:**

All fixes require mandatory live database testing:
- Real Teradata database with multiple databases (3+)
- Database with 100+ tables for completion testing
- Slow network conditions for loading indicator validation
- Permission scenarios (DBC.TablesV access)
- DDL operations for cache refresh validation
- Large result sets (>100 rows) for hint message validation

**Next Steps:**

1. **rust-teradata-architect:** Implement fixes based on these specifications
2. **quality-validator:** Design and execute comprehensive test suite
3. **User:** Acceptance testing to confirm bugs are resolved
4. **Team:** Ensure mandatory live database testing for all future sprints

---

## Sprint 7 Summary

**Features Added in This Version:**

1. **Table Name Completion (5.6.2)** - P0
   - Context-aware completion after FROM, JOIN, UPDATE, INTO keywords
   - Metadata queried from DBC.TablesV with session-scoped caching
   - Loading states, error handling, and performance optimizations
   - <500ms metadata query, <50ms cached response
   - **NOTE:** Sprint 8 revealed these features didn't work against real Teradata databases

2. **Column Name Completion (5.6.3)** - P1
   - Context-aware completion after SELECT, WHERE, ORDER BY keywords
   - Shows column names with type hints for better discoverability
   - Handles simple single-table queries and JOIN queries with table aliases
   - <300ms metadata query, <50ms cached response
   - Acknowledged limitations: subqueries, CTEs, complex expressions
   - **NOTE:** Sprint 8 revealed these features didn't work against real Teradata databases

3. **`/logon` Metacommand (5.8.1)** - P1
   - Dynamic connection switching without exiting REPL
   - Shows current connection with no arguments
   - Preserves REPL state (history, settings) across connections
   - Clears metadata cache on connection change
   - Comprehensive error handling with connection fallback
   - <2s connection time, 30s timeout

**Design Principles Applied:**

- **Consistency**: Follows patterns from Sprint 6 keyword completion
- **Performance**: Lazy loading with aggressive caching
- **Forgiveness**: Graceful degradation on errors, never crash REPL
- **Discoverability**: Loading states and helpful error messages guide users
- **Security**: Password filtering in history, secure credential handling

**Testing Requirements:**

All features require comprehensive testing covering:
- Happy path scenarios (completion works as expected)
- Edge cases (slow database, permissions, empty results)
- Error conditions (connection loss, timeouts, invalid input)
- Performance validation (meet <500ms table, <300ms column requirements)
- State management (cache invalidation, connection switching)

---
