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
- Multi-line SQL statements are grouped until semicolon terminator (`;`)
- Each complete statement (which may span multiple lines) is stored as single history entry
- Pressing ↑ arrow recalls complete multi-line statement, not individual lines
- Recalled multi-line statements preserve original line breaks and formatting
- Within a recalled multi-line statement, cursor navigation allows line-by-line editing
- History file stores multi-line statements with embedded newlines preserved

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

**Multi-line History Navigation Requirements**:

**REQ-HIST-001: Statement Grouping**
- SQL input continues across lines until semicolon terminator (`;`) is encountered
- The complete multi-line statement is treated as single logical entry
- Statement boundaries defined by `;` character (not by line breaks)

**REQ-HIST-002: History Storage**
- Multi-line statements stored as single entry with embedded newlines preserved
- File format: newlines within statement stored as literal `\n` or actual newlines (implementation choice)
- Backward compatibility: existing history files continue to work

**REQ-HIST-003: History Recall**
- Pressing ↑ arrow recalls previous complete statement
- If statement spans multiple lines, entire statement appears in input buffer
- Original line breaks and indentation preserved
- Cursor positioned at end of recalled statement

**REQ-HIST-004: Navigation Within Recalled Statement**
- After recalling multi-line statement, ↑/↓ arrows move cursor between lines within statement
- Ctrl-P/Ctrl-N also navigate within multi-line statement (Emacs mode)
- j/k keys navigate within statement in Vi normal mode
- Left/right arrows work normally for character navigation

**REQ-HIST-005: Editing Recalled Statement**
- User can edit any line within recalled multi-line statement
- Changes apply to complete statement
- Executing modified statement adds it as new history entry

**REQ-HIST-006: Line-by-Line Input vs Recall Behavior**

During initial input (typing new query):
```sql
tq> SELECT employee_id,     [press Enter - continues to next line]
    first_name,             [press Enter - continues to next line]
    last_name               [press Enter - continues to next line]
  FROM employees            [press Enter - continues to next line]
  WHERE salary > 50000;     [press Enter - executes complete statement]
```

During recall (pressing ↑):
```sql
tq> [press ↑ arrow]
tq> SELECT employee_id,
    first_name,
    last_name
  FROM employees
  WHERE salary > 50000;     [cursor here - complete statement recalled]
```

**REQ-HIST-007: History Traversal**
- First ↑: recalls most recent statement (may be multi-line)
- Second ↑: recalls second-most recent statement (may be multi-line)
- ↓ after ↑: moves forward through history
- History navigation treats each complete statement (single or multi-line) as one entry

**Example Interaction:**

User enters three multi-line queries:

```sql
# Query 1 (multi-line)
tq> SELECT * FROM employees
    WHERE department = 'IT';
[Executes]

# Query 2 (single-line)
tq> SELECT COUNT(*) FROM orders;
[Executes]

# Query 3 (multi-line)
tq> UPDATE employees
    SET status = 'active'
    WHERE hire_date > '2024-01-01';
[Executes]

# Now at empty prompt
tq> [press ↑ once]
# Shows complete Query 3:
tq> UPDATE employees
    SET status = 'active'
    WHERE hire_date > '2024-01-01';

tq> [press ↑ again]
# Shows complete Query 2:
tq> SELECT COUNT(*) FROM orders;

tq> [press ↑ again]
# Shows complete Query 1:
tq> SELECT * FROM employees
    WHERE department = 'IT';
```

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

**CRITICAL REQUIREMENT:** Tab completion MUST NOT produce any pager output or database query result formatting. When pressing TAB, the user should see ONLY completion suggestions, never "Page 1: records 0 - 0 total: 0" or similar pager output.

### Metacommand Completion

#### TC-006: Metacommand Tab Completion

**Requirement:** Metacommands SHALL be discoverable and completable via TAB key, following standard shell completion behavior.

**Specific Requirements:**

1. **TC-006.1** - Typing `/` followed by TAB SHALL display all available metacommands in a completion menu
2. **TC-006.2** - Typing partial metacommand (e.g., `/des`) followed by TAB SHALL complete to matching metacommands
3. **TC-006.3** - If multiple metacommands match the prefix, TAB SHALL display filtered completion menu
4. **TC-006.4** - If single unambiguous match exists, TAB SHALL auto-complete immediately
5. **TC-006.5** - Metacommand completion SHALL work on any line (including multi-line SQL input) when line starts with `/`
6. **TC-006.6** - Completion menu SHALL display metacommand descriptions alongside command names
7. **TC-006.7** - Case-insensitive matching SHALL be supported (e.g., `/DES<TAB>` matches `/describe`)
8. **TC-006.8** - Metacommand completion SHALL use same navigation as other completions (UP/DOWN arrows, ENTER to accept, ESC to cancel)

**Completable Metacommands:**
- `/describe` - Describe table structure
- `/list` - Schema inspection (databases, tables, views)
- `/locks` - Display current lock contention and blocking chains
- `/logon` - Connect/switch database
- `/disconnect` - Disconnect current connection
- `/reconnect` - Reconnect to database
- `/ping` - Test connection
- `/query` - Show current SQL query for a session
- `/sample` - Show random sample
- `/peek` - Show first rows and column info
- `/export` - Export results
- `/session` - Show session info
- `/sessions` - List active Teradata sessions
- `/sysconfig` - Display system configuration (version and AMP count)
- `/timing` - Enable/disable query timing
- `/set` - Set configuration
- `/pager` - Enable/disable result paging
- `/colors` - Enable/disable syntax highlighting
- `/help` - Show help
- `/clear` - Clear screen
- `/history` - Show command history
- `/edit` - Edit last query
- `/repeat` - Re-execute last query
- `/quit` - Exit REPL

**Example Behavior:**

**Show all metacommands:**
```sql
tq> /<TAB>

Available metacommands:
    /clear       Clear screen
    /colors      Enable/disable syntax highlighting
    /describe    Describe table structure
    /disconnect  Disconnect current connection
    /edit        Edit last query in $EDITOR
    /export      Export results
    /help        Show help
    /history     Show command history
    /list        Schema inspection (databases, tables, views)
    /locks       Display current lock contention and blocking chains
    /logon       Connect/switch database
    /pager       Enable/disable result paging
    /peek        Show first rows and column info
    /ping        Test connection
    /query       Show current SQL query for a session
    /quit        Exit REPL
    /reconnect   Reconnect to database
    /repeat      Re-execute last query
    /sample      Show random sample
    /session     Show session info
    /sessions    List active Teradata sessions with performance metrics
    /set         Set configuration
    /sysconfig   Display system topology (version, nodes, AMPs, PEs)
    /timing      Enable/disable query timing
```

**Partial completion - single match:**
```sql
tq> /des<TAB>
tq> /describe _
```

**Partial completion - multiple matches:**
```sql
tq> /l<TAB>

Matching metacommands:
    /list        Schema inspection (databases, tables, views)
    /logon       Connect/switch database
```

**In multi-line mode:**
```sql
tq> SELECT *
    FROM employees
    WHERE dept = 'IT';
tq(multi)> /des<TAB>
tq(multi)> /describe _
```

**Acceptance Test:**
- Type `/<TAB>` and verify all metacommands are shown with descriptions
- Type `/des<TAB>` and verify auto-completion to `/describe`
- Type `/l<TAB>` and verify filtered menu shows `/list` and `/logon`
- Type `/HELP<TAB>` (uppercase) and verify completion to `/help`

---

### Core Requirements

#### TC-001: Complete Database Metadata Coverage

**Requirement:** All databases on the Teradata system SHALL be included in database completion suggestions, including system databases.

**Specific Requirements:**

1. **TC-001.1** - System database `dbc` SHALL appear in database completion suggestions
2. **TC-001.2** - The metadata query SHALL fetch ALL databases without filtering by user access rights during the fetch operation
3. **TC-001.3** - If access to a specific database is denied during query execution (post-fetch), the tool SHALL handle the error gracefully and continue
4. **TC-001.4** - Database metadata SHALL be cached at REPL startup or on first completion request
5. **TC-001.5** - Database metadata query SHALL use Teradata system catalog view that returns complete database list

**Example Behavior:**
```sql
tq> SELECT * FROM d<TAB>

Database suggestions:
    dbc                (database - system)
    demo_user          (database)
    DemoNow_Monitor    (database)
    development        (database)
    production         (database)
```

**Acceptance Test:**
- Type `SELECT * FROM dbc.<TAB>` and verify that `dbc` database is recognized and tables are shown

---

#### TC-002: Universal Table Metadata Fetching

**Requirement:** Table metadata SHALL be fetched for ALL databases on the system, not just a subset. No database with tables should show "NO RECORDS FOUND" when requesting table completion.

**Specific Requirements:**

1. **TC-002.1** - When user types `database.<TAB>`, the tool SHALL attempt to fetch table metadata for that database if not already cached
2. **TC-002.2** - Table fetching SHALL NOT be limited to a pre-determined list of databases
3. **TC-002.3** - If a database has tables but metadata is not cached, pressing TAB SHALL trigger on-demand fetching
4. **TC-002.4** - If table metadata fetch fails due to permissions, the tool SHALL display an informative message instead of "NO RECORDS FOUND"
5. **TC-002.5** - Successfully fetched table metadata SHALL be cached for the session duration
6. **TC-002.6** - The tool SHALL fetch tables from Teradata system catalog using queries that return complete table lists

**Example Behavior:**

**Success case:**
```sql
tq> SELECT * FROM demo_user.<TAB>

Tables in 'demo_user':
    demo_user.customer_data      (table)
    demo_user.sales_records      (table)
    demo_user.inventory          (table)
```

**Permission denied case:**
```sql
tq> SELECT * FROM restricted_db.<TAB>

Error: Access denied to database 'restricted_db'
Cannot fetch table metadata (insufficient privileges)
```

**Acceptance Test:**
- Type `SELECT * FROM demo_user.<TAB>` and verify tables are displayed
- Repeat for multiple different databases and verify consistent behavior

---

#### TC-003: TAB Key Acceptance Behavior

**Requirement:** The TAB key SHALL follow standard bash/zsh completion behavior: first TAB shows completion menu, second TAB accepts the highlighted item.

**Specific Requirements:**

1. **TC-003.1** - First TAB press with multiple matches SHALL display completion menu with first item highlighted
2. **TC-003.2** - Second TAB press (while menu is displayed) SHALL accept the currently highlighted item and insert it into the command line
3. **TC-003.3** - DOWN arrow key SHALL move highlight to next item in completion menu
4. **TC-003.4** - UP arrow key SHALL move highlight to previous item in completion menu
5. **TC-003.5** - ENTER key SHALL accept the currently highlighted item
6. **TC-003.6** - ESC key SHALL dismiss the completion menu without making a selection
7. **TC-003.7** - First TAB press with single unambiguous match SHALL auto-complete immediately (no menu)
8. **TC-003.8** - Typing additional characters SHALL filter the completion menu in real-time

**Example Interaction Flow:**

```sql
# User types and presses TAB
tq> SELECT * FROM dem<TAB>

# Menu appears with first item highlighted
demo_user          (database) ← highlighted
DemoNow_Monitor    (database)

# User presses TAB again
tq> SELECT * FROM demo_user_

# "demo_user" accepted and cursor after the name

# Alternative: User presses DOWN arrow
demo_user          (database)
DemoNow_Monitor    (database) ← highlighted

# User presses ENTER
tq> SELECT * FROM DemoNow_Monitor_
```

**Acceptance Test:**
- Type `SELECT * FROM d<TAB>` (shows menu)
- Press TAB again and verify first item is inserted
- Repeat with arrow navigation and verify highlighted item is accepted

---

#### TC-004: Smart Qualified Name Completion

**Requirement:** When completing a database name followed by a dot, the tool SHALL automatically complete the database name (if unambiguous), append a dot, and immediately display tables in that database.

**Specific Requirements:**

1. **TC-004.1** - When user types partial database name + TAB after FROM/JOIN keyword, if match is unambiguous, the tool SHALL auto-complete the database name
2. **TC-004.2** - After auto-completing database name, the tool SHALL automatically append a dot (`.`) character
3. **TC-004.3** - After appending the dot, the tool SHALL immediately display table completion suggestions for that database (without requiring another TAB press)
4. **TC-004.4** - If database name match is ambiguous, the tool SHALL show database completion menu first (existing behavior)
5. **TC-004.5** - This behavior SHALL work after FROM keyword
6. **TC-004.6** - This behavior SHALL work after JOIN keywords (INNER JOIN, LEFT JOIN, RIGHT JOIN, FULL JOIN, CROSS JOIN)
7. **TC-004.7** - If table metadata for the database is not cached, the tool SHALL fetch it (potentially showing brief loading indicator)

**Example Interaction - Unambiguous:**

```sql
# User types partial database name
tq> SELECT * FROM dem<TAB>

# If only "demo_user" matches, auto-completes to:
tq> SELECT * FROM demo_user.

# Immediately shows tables (no additional TAB needed):
Tables in 'demo_user':
    customer_data      (table)
    sales_records      (table)
    inventory          (table)
```

**Example Interaction - Ambiguous:**

```sql
# User types partial database name
tq> SELECT * FROM dem<TAB>

# If multiple databases match (demo_user, demo_prod), show menu:
demo_user          (database)
demo_prod          (database)

# User presses TAB again to accept highlighted:
tq> SELECT * FROM demo_user.

# Tables shown automatically:
Tables in 'demo_user':
    customer_data      (table)
    sales_records      (table)
```

**Example Interaction - After JOIN:**

```sql
tq> SELECT * FROM orders o JOIN dem<TAB>

# Completes to:
tq> SELECT * FROM orders o JOIN demo_user.

# Shows tables:
Tables in 'demo_user':
    customer_data      (table)
    sales_records      (table)
```

**Acceptance Test:**
- Type `SELECT * FROM dem<TAB>` where "demo_user" is unambiguous
- Verify database name completes, dot is added, and tables appear
- Type `SELECT * FROM d<TAB>` where multiple databases match "d"
- Verify menu appears first, then after selection, dot + tables appear

---

#### TC-005: Tab Completion Regression Testing Support

**Requirement:** Tab completion behavior SHALL be testable through automated regression tests to prevent future defects.

**Specific Requirements:**

1. **TC-005.1** - Metadata fetching logic (database, table, column queries) SHALL be unit-testable in isolation
2. **TC-005.2** - Completion suggestion generation SHALL be testable with mock metadata
3. **TC-005.3** - The completion system SHALL provide APIs or test hooks that allow:
   - Injecting test metadata without database connection
   - Verifying completion suggestions for given input context
   - Testing metadata cache behavior
4. **TC-005.4** - Integration tests SHALL verify completion suggestions at various SQL positions:
   - After FROM keyword
   - After JOIN keywords
   - After WHERE clause (column completion)
   - After qualified names (database.table)
5. **TC-005.5** - Tests SHALL verify no pager output appears during completion operations
6. **TC-005.6** - Tests SHALL verify graceful error handling (permission denied, invalid database, network errors)
7. **TC-005.7** - Where possible, PTY-based tests SHALL verify menu display and navigation behavior
8. **TC-005.8** - Test documentation SHALL indicate which aspects are automatically testable vs. requiring manual validation

**Testable Components:**

**Unit Test Examples:**
- Query parser identifies cursor context (after FROM, after JOIN, etc.)
- Metadata cache stores and retrieves databases/tables correctly
- Completion filter matches partial input correctly (case-insensitive, prefix matching)
- System catalog queries return expected format

**Integration Test Examples:**
- Given metadata cache with known databases, typing `SELECT * FROM d<TAB>` returns filtered list
- Given database "demo_user" with known tables, typing `demo_user.<TAB>` returns table list
- Given no cached metadata for database "newdb", typing `newdb.<TAB>` triggers fetch
- Given permission denied for database "forbidden", typing `forbidden.<TAB>` shows error message

**Manual Validation Required:**
- Visual appearance of completion menu
- Keyboard navigation (arrow keys, TAB acceptance)
- Menu positioning and layout in terminal
- Color and highlighting of selected item

**Acceptance Test:**
- Automated test suite executes without failures
- Manual validation checklist confirms visual behavior matches specification

---

### Loading Indicator for Metadata Fetching

#### TC-007: Loading Indicator During Slow Metadata Queries

**Requirement:** Users SHALL receive visual feedback during slow metadata fetch operations (>500ms) to indicate the system is working and has not frozen.

**Specific Requirements:**

1. **TC-007.1** - If metadata query takes longer than 500ms, a loading indicator SHALL be displayed
2. **TC-007.2** - Loading indicator message format SHALL be: `"Loading tables from <database>..."` (for table fetching)
3. **TC-007.3** - Loading indicator SHALL show animated spinner character to indicate active processing
4. **TC-007.4** - Loading indicator SHALL clear automatically when completion menu appears
5. **TC-007.5** - For cached metadata (instant response <50ms), NO loading indicator SHALL appear
6. **TC-007.6** - If metadata query fails, loading indicator SHALL be replaced with error message
7. **TC-007.7** - Loading indicator SHALL not block other terminal input/output
8. **TC-007.8** - Ctrl-C during loading SHALL cancel the metadata fetch and return to prompt

**Indicator Messages by Context:**

| Context | Message Format |
|---------|----------------|
| Fetching tables for database | `Loading tables from <database>...` |
| Fetching columns for table | `Loading columns from <table>...` |
| Fetching databases | `Loading databases...` |

**Spinner Animation:**
- Character sequence: `⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏` (Braille spinner)
- Animation speed: 10 frames per second (100ms per frame)
- Cycles continuously until metadata fetch completes

**Example Behavior:**

**Fast metadata (cached, <50ms) - No indicator:**
```sql
tq> SELECT * FROM production.<TAB>
[Instant, no loading message]
Tables in 'production':
    customers    employees    orders    products
```

**Slow metadata (uncached, >500ms) - With indicator:**
```sql
tq> SELECT * FROM remote_database.<TAB>
Loading tables from remote_database... ⠋
[After 2.3 seconds:]
Tables in 'remote_database':
    table1    table2    table3    table4
```

**User cancellation:**
```sql
tq> SELECT * FROM slow_database.<TAB>
Loading tables from slow_database... ⠹
^C
Metadata fetch cancelled

tq> _
[Returns to prompt without completion]
```

**Error during fetch:**
```sql
tq> SELECT * FROM forbidden_db.<TAB>
Loading tables from forbidden_db... ⠴
Error: Access denied to database 'forbidden_db'
Cannot fetch table metadata (insufficient privileges)
```

**Acceptance Test:**
- Trigger table completion on uncached database with >500ms query time and verify loading indicator appears
- Verify indicator shows spinner animation
- Verify indicator clears when completion menu appears
- Trigger table completion on cached database and verify NO indicator appears (<50ms)
- Press Ctrl-C during loading and verify cancellation returns to prompt

---

### Metadata Caching Strategy

**Database names:**
1. Cached at REPL startup or first completion request
2. Query: Fetch all databases from Teradata system catalog (including system databases)
3. Cache lifetime: Entire REPL session

**Table names:**
1. Cached on-demand when database is first explored
2. Query: Fetch all tables for specific database from Teradata system catalog
3. Cache lifetime: Entire REPL session
4. Behavior: If not cached, fetch on first `database.<TAB>` press
5. Loading indicator: Display if fetch takes >500ms (see TC-007)

**Column names:**
1. Cached on-demand when table is first referenced
2. Query: Fetch all columns for specific table from Teradata system catalog
3. Cache lifetime: Entire REPL session
4. Loading indicator: Display if fetch takes >500ms (see TC-007)

### Completion Menu Behavior Summary

This section summarizes the detailed requirements specified in TC-003 (TAB Key Acceptance Behavior).

**Display:**
- Completion candidates shown in columnar menu format
- Each candidate labeled with type (database, table, column)
- First item highlighted by default

**Interaction:**
- TAB key: First press shows menu, second press accepts highlighted item (see TC-003)
- UP/DOWN arrows: Navigate through candidates
- ENTER: Accept highlighted item
- ESC: Dismiss menu
- Typing: Filter candidates in real-time

**Performance:**
- Cached metadata: < 50ms response time
- Uncached metadata: < 500ms fetch time (with optional loading indicator)

**Output Suppression:**
All metadata queries executed during tab completion MUST suppress stdout/stderr output from the Teradata driver to prevent pager output from appearing in the terminal (see TC-005.5).

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

**Menu-based completion with filtering:**
```sql
tq> SELECT * FROM pro<TAB>
production
products

[Type 'd' to filter further]
tq> SELECT * FROM prod<TAB>
production
products

[Arrow up/down to navigate, Enter to select]
```

**After database name + dot - Show tables in that database**:
```sql
tq> SELECT * FROM production.<TAB>
Tables in 'production':
    customers    employees    orders    products    invoices    [45 more...]

[Filter by typing: 'cus' narrows to 'customers']
tq> SELECT * FROM production.cus<TAB>
customers
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

🧪 **EXPERIMENTAL** - Interactive pager is disabled by default. Enable with `/pager on`.

Result paging uses a three-layer strategy:
1. **Column Windowing** - Limit visible columns to maintain readability
2. **Cell Truncation** - Limit cell content length to prevent layout breaks
3. **Row Paging** - Paginate vertically through long result sets

**CRITICAL REQUIREMENT:** Pager MUST be safe - 'q' key exits pager and returns to REPL, never exits the entire program.

#### Column Windowing (Layer 1)

**Objective:** Display manageable subset of columns, navigate horizontally through remaining columns.

**Navigation Details:** See "Horizontal Column Navigation" section below for complete requirements (REQ-PAGER-HORIZ-001 through REQ-PAGER-HORIZ-014).

**Key Capabilities:**
- Left/right arrow navigation to scroll one column at a time
- Vim-style h/l keys for horizontal movement
- H/L keys to jump to first/last columns
- Column position indicators showing hidden columns
- Column range display in status bar
- Preserved horizontal position during vertical scrolling

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

#### Horizontal Column Navigation

When result sets exceed terminal width, horizontal navigation enables exploration of all columns through left/right scrolling.

**REQ-PAGER-HORIZ-001: Horizontal Scrolling Activation**

The pager SHALL enable horizontal scrolling when the combined width of all columns exceeds the available terminal width:

1. **REQ-PAGER-HORIZ-001.1** - Calculate total required width: sum of all column widths plus borders/padding
2. **REQ-PAGER-HORIZ-001.2** - Compare against current terminal width
3. **REQ-PAGER-HORIZ-001.3** - If total width exceeds terminal width, enable horizontal navigation
4. **REQ-PAGER-HORIZ-001.4** - Initial view displays leftmost columns that fit within terminal width
5. **REQ-PAGER-HORIZ-001.5** - Display indicators showing additional columns exist beyond viewport

**Rationale:** Users need to know when horizontal navigation is available and which columns are currently visible.

**Example Scenario:**
```
Terminal width: 80 characters
Result set: 23 columns, total width 350 characters

Initial view shows columns 1-5 (fit within 80 chars)
Right arrow available to scroll right
Status bar shows: "Columns 1-5 of 23"
```

**REQ-PAGER-HORIZ-002: Right Arrow Navigation**

The right arrow key SHALL scroll the viewport one column to the right when hidden columns exist:

1. **REQ-PAGER-HORIZ-002.1** - Right arrow (→) key scrolls viewport one column rightward
2. **REQ-PAGER-HORIZ-002.2** - Leftmost visible column shifts out of view
3. **REQ-PAGER-HORIZ-002.3** - Next hidden column becomes visible on the right
4. **REQ-PAGER-HORIZ-002.4** - Column width calculations maintain table formatting consistency
5. **REQ-PAGER-HORIZ-002.5** - If already at rightmost position (all columns visible or last column displayed), right arrow has no effect
6. **REQ-PAGER-HORIZ-002.6** - Visual indicator updates to reflect new column range

**Rationale:** Right arrow provides intuitive "move right" navigation consistent with standard UI conventions.

**Example Interaction:**
```
Initial view:
┌──────┬─────────┬──────────┬────────┐
│ id   │ name    │ email    │ dept   │
├──────┼─────────┼──────────┼────────┤
│ 1    │ Alice   │ a@co.com │ IT     │
└──────┴─────────┴──────────┴────────┘
Columns 1-4 of 23 | (+19 cols) →

[Press → key]

After scroll:
┌─────────┬──────────┬────────┬────────┐
│ name    │ email    │ dept   │ salary │
├─────────┼──────────┼────────┼────────┤
│ Alice   │ a@co.com │ IT     │ 75000  │
└─────────┴──────────┴────────┴────────┘
(+1 cols) ← | Columns 2-5 of 23 | (+18 cols) →
```

**Edge Cases:**
- Already at rightmost position: Right arrow does nothing
- Single additional column: Right arrow reveals final column and removes right indicator

**REQ-PAGER-HORIZ-003: Left Arrow Navigation**

The left arrow key SHALL scroll the viewport one column to the left when the view has been scrolled right:

1. **REQ-PAGER-HORIZ-003.1** - Left arrow (←) key scrolls viewport one column leftward
2. **REQ-PAGER-HORIZ-003.2** - Rightmost visible column shifts out of view
3. **REQ-PAGER-HORIZ-003.3** - Previous hidden column becomes visible on the left
4. **REQ-PAGER-HORIZ-003.4** - If already at leftmost position (first column visible), left arrow has no effect
5. **REQ-PAGER-HORIZ-003.5** - Visual indicator updates to reflect new column range

**Rationale:** Left arrow provides intuitive "move left" navigation, enabling users to return to previously viewed columns.

**Example Interaction:**
```
Scrolled view:
┌─────────┬──────────┬────────┬────────┐
│ name    │ email    │ dept   │ salary │
├─────────┼──────────┼────────┼────────┤
│ Alice   │ a@co.com │ IT     │ 75000  │
└─────────┴──────────┴────────┴────────┘
(+1 cols) ← | Columns 2-5 of 23 | (+18 cols) →

[Press ← key]

After scroll left:
┌──────┬─────────┬──────────┬────────┐
│ id   │ name    │ email    │ dept   │
├──────┼─────────┼──────────┼────────┤
│ 1    │ Alice   │ a@co.com │ IT     │
└──────┴─────────┴──────────┴────────┘
Columns 1-4 of 23 | (+19 cols) →
```

**Edge Cases:**
- Already at leftmost position: Left arrow does nothing
- One column scrolled: Left arrow returns to initial view, removes left indicator

**REQ-PAGER-HORIZ-004: Right-Side Column Indicator**

The pager SHALL display a visual indicator in the rightmost column area when columns are hidden to the right:

1. **REQ-PAGER-HORIZ-004.1** - Indicator format: `(+N cols)` where N is count of hidden columns
2. **REQ-PAGER-HORIZ-004.2** - Indicator positioned in status bar on right side
3. **REQ-PAGER-HORIZ-004.3** - Indicator includes right arrow symbol (→) to suggest navigation direction
4. **REQ-PAGER-HORIZ-004.4** - Indicator updates dynamically as user scrolls (N decreases)
5. **REQ-PAGER-HORIZ-004.5** - Indicator disappears when rightmost column is visible
6. **REQ-PAGER-HORIZ-004.6** - Indicator count includes ALL hidden columns to the right, not just immediately adjacent

**Rationale:** Users need clear indication of hidden content and how many columns remain unexplored.

**Example Display:**
```
Status bar when 19 columns hidden on right:
Columns 1-4 of 23 | (+19 cols) →

Status bar when 10 columns hidden on right:
Columns 9-13 of 23 | (+10 cols) →

Status bar when at rightmost position:
(+15 cols) ← | Columns 19-23 of 23
```

**REQ-PAGER-HORIZ-005: Left-Side Column Indicator**

The pager SHALL display a visual indicator when columns are hidden to the left (user has scrolled right):

1. **REQ-PAGER-HORIZ-005.1** - Indicator format: `(+N cols)` where N is count of hidden columns
2. **REQ-PAGER-HORIZ-005.2** - Indicator positioned in status bar on left side
3. **REQ-PAGER-HORIZ-005.3** - Indicator includes left arrow symbol (←) to suggest navigation direction
4. **REQ-PAGER-HORIZ-005.4** - Indicator updates dynamically as user scrolls left/right
5. **REQ-PAGER-HORIZ-005.5** - Indicator disappears when leftmost column (first column) is visible
6. **REQ-PAGER-HORIZ-005.6** - Indicator count includes ALL hidden columns to the left

**Rationale:** Users need awareness of hidden columns on the left to navigate back effectively.

**Example Display:**
```
Status bar when 3 columns hidden on left:
(+3 cols) ← | Columns 4-8 of 23 | (+15 cols) →

Status bar when 10 columns hidden on left:
(+10 cols) ← | Columns 11-15 of 23 | (+8 cols) →

Status bar when scrolled to middle:
(+10 cols) ← | Columns 11-15 of 23 | (+8 cols) →
```

**REQ-PAGER-HORIZ-006: Status Bar Column Range Display**

The status bar SHALL display the current column range visible in the viewport:

1. **REQ-PAGER-HORIZ-006.1** - Format: `Columns X-Y of Z` where X=first visible column number, Y=last visible column number, Z=total column count
2. **REQ-PAGER-HORIZ-006.2** - Column numbers start at 1 (not 0)
3. **REQ-PAGER-HORIZ-006.3** - Range updates immediately as user scrolls horizontally
4. **REQ-PAGER-HORIZ-006.4** - When all columns fit in viewport, display: `Columns 1-Z of Z` (no horizontal navigation available)
5. **REQ-PAGER-HORIZ-006.5** - Display alongside row position indicator: `Columns X-Y of Z | Rows A-B of C (P%)`
6. **REQ-PAGER-HORIZ-006.6** - Column range SHALL be accurate and match actual displayed columns

**Rationale:** Users need precise information about their current position within the column set.

**Example Displays:**
```
Narrow result (all columns fit):
Columns 1-8 of 8 | Rows 1-20 of 1,234 (2%)

Wide result at start:
Columns 1-5 of 23 | Rows 1-20 of 1,234 (2%)

Wide result scrolled right:
Columns 8-12 of 23 | Rows 1-20 of 1,234 (2%)

Wide result at end:
Columns 19-23 of 23 | Rows 1-20 of 1,234 (2%)
```

**REQ-PAGER-HORIZ-007: Vim-Style h/l Keybindings**

The pager SHALL support Vim-style h/l keys for horizontal navigation alongside arrow keys:

1. **REQ-PAGER-HORIZ-007.1** - `h` key scrolls left (equivalent to left arrow ←)
2. **REQ-PAGER-HORIZ-007.2** - `l` key scrolls right (equivalent to right arrow →)
3. **REQ-PAGER-HORIZ-007.3** - `h` and `l` behavior matches arrow key behavior exactly (same scroll amount, same edge handling)
4. **REQ-PAGER-HORIZ-007.4** - Keys work in lowercase only (uppercase reserved for jump commands)
5. **REQ-PAGER-HORIZ-007.5** - Keys SHALL be documented in help text alongside arrow keys

**Rationale:** Vim users expect h/j/k/l navigation; providing h/l for horizontal scrolling maintains consistency with vertical j/k navigation already supported.

**Example Interaction:**
```
[Press 'l' key - same effect as right arrow]
Scrolls one column right

[Press 'h' key - same effect as left arrow]
Scrolls one column left
```

**REQ-PAGER-HORIZ-008: Jump to First Column**

The pager SHALL support jumping to the first column instantly:

1. **REQ-PAGER-HORIZ-008.1** - `H` key (uppercase) jumps to leftmost column position
2. **REQ-PAGER-HORIZ-008.2** - Viewport resets to show columns starting from column 1
3. **REQ-PAGER-HORIZ-008.3** - Left indicator disappears (no hidden columns on left)
4. **REQ-PAGER-HORIZ-008.4** - Status bar updates to show "Columns 1-N of Z"
5. **REQ-PAGER-HORIZ-008.5** - If already at first column, command has no effect (idempotent)
6. **REQ-PAGER-HORIZ-008.6** - Jump preserves current vertical scroll position (row number unchanged)

**Rationale:** Users need quick navigation to beginning of wide result sets without repeated left arrow presses.

**Example Interaction:**
```
Before jump (scrolled right):
(+10 cols) ← | Columns 11-15 of 23 | (+8 cols) →

[Press 'H' key]

After jump:
Columns 1-5 of 23 | (+18 cols) →
```

**REQ-PAGER-HORIZ-009: Jump to Last Column**

The pager SHALL support jumping to the last column instantly:

1. **REQ-PAGER-HORIZ-009.1** - `L` key (uppercase) jumps to rightmost column position
2. **REQ-PAGER-HORIZ-009.2** - Viewport adjusts to show maximum columns that fit, ending with the last column
3. **REQ-PAGER-HORIZ-009.3** - Right indicator disappears (no hidden columns on right)
4. **REQ-PAGER-HORIZ-009.4** - Status bar updates to show "Columns M-Z of Z" where Z is last column
5. **REQ-PAGER-HORIZ-009.5** - If already showing last column, command has no effect (idempotent)
6. **REQ-PAGER-HORIZ-009.6** - Jump preserves current vertical scroll position (row number unchanged)

**Rationale:** Users need quick navigation to end of wide result sets to see trailing columns without repeated right arrow presses.

**Example Interaction:**
```
Before jump (at start):
Columns 1-5 of 23 | (+18 cols) →

[Press 'L' key]

After jump:
(+18 cols) ← | Columns 19-23 of 23
```

**REQ-PAGER-HORIZ-010: Column Position Preservation During Vertical Scrolling**

The pager SHALL maintain horizontal scroll position when user scrolls vertically:

1. **REQ-PAGER-HORIZ-010.1** - When user scrolls down (j, ↓, Space, Page Down), column viewport remains unchanged
2. **REQ-PAGER-HORIZ-010.2** - When user scrolls up (k, ↑, b, Page Up), column viewport remains unchanged
3. **REQ-PAGER-HORIZ-010.3** - When user jumps vertically (g, G, Home, End), column viewport remains unchanged
4. **REQ-PAGER-HORIZ-010.4** - Horizontal position persists across all vertical navigation operations
5. **REQ-PAGER-HORIZ-010.5** - Status bar continues showing current column range during vertical scrolling
6. **REQ-PAGER-HORIZ-010.6** - Only explicit horizontal navigation commands (←, →, h, l, H, L) change column position

**Rationale:** Users exploring specific columns need to scroll vertically through rows without losing their horizontal context.

**Example Interaction:**
```
User scrolls right to columns 8-12:
Columns 8-12 of 23 | Rows 1-20 of 500

[Press 'j' to scroll down one row]
Columns 8-12 of 23 | Rows 2-21 of 500
[Column range unchanged]

[Press Space to page down]
Columns 8-12 of 23 | Rows 21-40 of 500
[Column range unchanged]

[Press 'G' to jump to last row]
Columns 8-12 of 23 | Rows 481-500 of 500
[Column range unchanged]
```

**Edge Cases:**
- User at rightmost columns, scrolls down: Column position preserved
- User at leftmost columns, pages up: Column position preserved
- User in middle columns, jumps to first row: Column position preserved

**REQ-PAGER-HORIZ-011: Horizontal Navigation in Help Text**

The pager help text SHALL document horizontal navigation keys clearly:

1. **REQ-PAGER-HORIZ-011.1** - Help text activated by `?` key displays horizontal navigation section
2. **REQ-PAGER-HORIZ-011.2** - Section title: "Horizontal Navigation" (separate from vertical navigation)
3. **REQ-PAGER-HORIZ-011.3** - List all horizontal navigation keys with descriptions:
   - `←` or `h`: Scroll left one column
   - `→` or `l`: Scroll right one column
   - `H`: Jump to first column
   - `L`: Jump to last column
4. **REQ-PAGER-HORIZ-011.4** - Help text SHALL explain column indicators: `(+N cols) ←` and `(+N cols) →`
5. **REQ-PAGER-HORIZ-011.5** - Help text SHALL note that column position is preserved during vertical scrolling
6. **REQ-PAGER-HORIZ-011.6** - Help text organized logically: Navigation (vertical, horizontal), Exit, Help

**Rationale:** Discoverability is critical; users must be able to learn horizontal navigation features from within the pager.

**Example Help Text:**
```
tq Pager Help
═════════════

Vertical Navigation:
  j, ↓         Scroll down one row
  k, ↑         Scroll up one row
  Space        Page down
  b            Page up
  g, Home      Jump to first row
  G, End       Jump to last row

Horizontal Navigation:
  ←, h         Scroll left one column
  →, l         Scroll right one column
  H            Jump to first column
  L            Jump to last column

Column indicators: (+N cols) ← means N hidden columns on left
                   (+N cols) → means N hidden columns on right

Note: Column position is preserved when scrolling vertically.

Exit:
  q, Esc       Exit pager, return to REPL prompt

Help:
  ?            Show this help
```

**REQ-PAGER-HORIZ-012: Exit Pager with Horizontal Navigation Active**

The pager SHALL exit cleanly regardless of current horizontal scroll position:

1. **REQ-PAGER-HORIZ-012.1** - `q` key exits pager and returns to REPL prompt
2. **REQ-PAGER-HORIZ-012.2** - `Esc` key exits pager and returns to REPL prompt
3. **REQ-PAGER-HORIZ-012.3** - Exit behavior identical whether user is at leftmost, middle, or rightmost column position
4. **REQ-PAGER-HORIZ-012.4** - Exit SHALL NOT depend on horizontal scroll state
5. **REQ-PAGER-HORIZ-012.5** - Pager state (including column position) SHALL be reset for next query result

**Rationale:** Exit mechanism must be reliable and independent of navigation state.

**Example Interaction:**
```
User scrolled to columns 15-20 of 30:
(+14 cols) ← | Columns 15-20 of 30 | (+10 cols) →

[Press 'q']

tq> _
[Back at REPL prompt, pager state cleared]
```

**REQ-PAGER-HORIZ-013: Horizontal Paging Disabled Mode**

When paging is disabled via `/pager off`, horizontal scrolling SHALL be unavailable:

1. **REQ-PAGER-HORIZ-013.1** - `/pager off` command disables both vertical and horizontal paging
2. **REQ-PAGER-HORIZ-013.2** - Wide result sets SHALL be displayed in full width
3. **REQ-PAGER-HORIZ-013.3** - If result width exceeds terminal width, columns SHALL be truncated or wrapped according to formatter
4. **REQ-PAGER-HORIZ-013.4** - No interactive navigation available (arrow keys not captured)
5. **REQ-PAGER-HORIZ-013.5** - No column indicators displayed
6. **REQ-PAGER-HORIZ-013.6** - No status bar displayed
7. **REQ-PAGER-HORIZ-013.7** - `/pager on` command re-enables both vertical and horizontal paging

**Rationale:** Users need option to bypass interactive paging for scripting, copying output, or personal preference.

**Example Behavior:**
```
tq> /pager off
Result paging disabled

tq> SELECT * FROM wide_table;
┌──────┬─────────┬──────────┬────────┬────────┬─────────┬─[truncated]
│ col1 │ col2    │ col3     │ col4   │ col5   │ col6    │ ...
├──────┼─────────┼──────────┼────────┼────────┼─────────┼─[truncated]
│ val1 │ val2    │ val3     │ val4   │ val5   │ val6    │ ...
└──────┴─────────┴──────────┴────────┴────────┴─────────┴─[truncated]

500 rows in set (0.234s)

tq> /pager on
Result paging enabled

tq> SELECT * FROM wide_table;
[Enters interactive pager with horizontal and vertical navigation]
```

**REQ-PAGER-HORIZ-014: Integration with Vertical Paging Keys**

Horizontal and vertical navigation SHALL operate independently without key conflicts:

1. **REQ-PAGER-HORIZ-014.1** - Arrow keys SHALL be context-aware:
   - `↑` (Up Arrow): Vertical navigation only (previous row)
   - `↓` (Down Arrow): Vertical navigation only (next row)
   - `←` (Left Arrow): Horizontal navigation only (previous column)
   - `→` (Right Arrow): Horizontal navigation only (next column)
2. **REQ-PAGER-HORIZ-014.2** - Vim keys SHALL be consistent:
   - `j`: Vertical navigation (down one row)
   - `k`: Vertical navigation (up one row)
   - `h`: Horizontal navigation (left one column)
   - `l`: Horizontal navigation (right one column)
3. **REQ-PAGER-HORIZ-014.3** - Special keys SHALL maintain semantics:
   - `Space`: Vertical page down only
   - `b`: Vertical page up only
   - `g` / `G`: Vertical jump (first/last row)
   - `H` / `L`: Horizontal jump (first/last column)
4. **REQ-PAGER-HORIZ-014.4** - No key SHALL trigger both horizontal and vertical navigation simultaneously
5. **REQ-PAGER-HORIZ-014.5** - Status bar SHALL reflect current position in both dimensions

**Rationale:** Clear separation of horizontal and vertical navigation prevents confusion and enables intuitive two-dimensional exploration.

**Example Interaction:**
```
Initial position:
Columns 1-5 of 23 | Rows 1-20 of 500

[Press →] - Horizontal scroll right:
Columns 2-6 of 23 | Rows 1-20 of 500

[Press ↓] - Vertical scroll down:
Columns 2-6 of 23 | Rows 2-21 of 500

[Press h] - Horizontal scroll left:
Columns 1-5 of 23 | Rows 2-21 of 500

[Press j] - Vertical scroll down:
Columns 1-5 of 23 | Rows 3-22 of 500

[Press L] - Jump to last column:
Columns 19-23 of 23 | Rows 3-22 of 500

[Press G] - Jump to last row:
Columns 19-23 of 23 | Rows 481-500 of 500
```

**Edge Cases:**
- User rapidly alternates between horizontal and vertical navigation: Both dimensions update correctly
- User at edge positions (first/last row AND first/last column): Appropriate keys disabled, others work
- Terminal resize during navigation: Both horizontal and vertical viewports recalculate

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

**REQ-PAGER-001: Status Bar Layout**

The pager SHALL display a two-line status bar at the bottom of the terminal with the following requirements:

1. **REQ-PAGER-001.1** - Position: Bottom two lines of terminal viewport
2. **REQ-PAGER-001.2** - Visual separation: Top and bottom borders using box-drawing characters
3. **REQ-PAGER-001.3** - Line 1 content: Position indicators (column and row ranges) and column indicators
4. **REQ-PAGER-001.4** - Line 2 content: Navigation hints and exit instructions
5. **REQ-PAGER-001.5** - Status bar SHALL remain visible at all times during paging

**REQ-PAGER-002: Status Bar Content Requirements**

The status bar SHALL display the following information:

1. **REQ-PAGER-002.1** - Column position: `Columns X-Y of Z` format (when horizontal scrolling available, see REQ-PAGER-HORIZ-006)
2. **REQ-PAGER-002.2** - Row position: `Rows X-Y of Z (P%)` format with percentage indicator
3. **REQ-PAGER-002.3** - Column indicators: Left `(+N cols) ←` and right `(+N cols) →` when columns hidden (see REQ-PAGER-HORIZ-004, REQ-PAGER-HORIZ-005)
4. **REQ-PAGER-002.4** - Navigation hints: Key bindings for horizontal and vertical navigation
5. **REQ-PAGER-002.5** - Exit instructions: Clear indication of how to exit pager

**REQ-PAGER-003: Navigation Hints Clarity**

The status bar navigation hints SHALL be clear and discoverable:

1. **REQ-PAGER-003.1** - Horizontal navigation: `←→: columns` or `h/l: columns` when horizontal scrolling available
2. **REQ-PAGER-003.2** - Vertical navigation: `↑↓ Space b: rows` or `j/k Space b: rows` format
3. **REQ-PAGER-003.3** - Jump commands: `H/L: first/last col` and `g/G: first/last row` for quick navigation
4. **REQ-PAGER-003.4** - Exit commands: `q/Esc: exit` prominently displayed
5. **REQ-PAGER-003.5** - Hints SHALL be concise (fit within terminal width)
6. **REQ-PAGER-003.6** - Hints SHALL prioritize most commonly used keys
7. **REQ-PAGER-003.7** - Help hint: `?: help` to discover all navigation keys

**REQ-PAGER-004: Dynamic Status Bar Adaptation**

The status bar SHALL adapt to result set characteristics:

1. **REQ-PAGER-004.1** - Narrow result (all columns fit): Omit horizontal navigation hints and column indicators
2. **REQ-PAGER-004.2** - Wide result sets: Display horizontal navigation hints and column indicators
3. **REQ-PAGER-004.3** - Short result sets (fits in viewport): Indicate all rows visible
4. **REQ-PAGER-004.4** - Terminal width changes: Recalculate column/row layout and reflow status bar dynamically

**Layout Examples:**

**Wide Result (Horizontal Navigation Available):**
```
┌────────────────────────────────────────────────────────────────────────────┐
│ Columns 1-5 of 23 | Rows 1-20 of 1,234 (2%)           | (+18 cols) →      │
│ ←→ h/l: columns | ↑↓ j/k: rows | Space/b: page | H/L g/G: jump | q: exit  │
└────────────────────────────────────────────────────────────────────────────┘
```

**Wide Result (Scrolled Right, Both Indicators):**
```
┌────────────────────────────────────────────────────────────────────────────┐
│ (+3 cols) ← | Columns 4-8 of 23 | Rows 1-20 of 1,234 (2%) | (+15 cols) →  │
│ ←→ h/l: columns | ↑↓ j/k: rows | Space/b: page | H/L g/G: jump | q: exit  │
└────────────────────────────────────────────────────────────────────────────┘
```

**Narrow Result (All Columns Fit, No Horizontal Navigation):**
```
┌────────────────────────────────────────────────────────────────────────────┐
│ Rows 1-20 of 1,234 (2%)                                                    │
│ ↑↓ j/k: rows | Space/b: page | g/G: first/last | ?: help | q: exit        │
└────────────────────────────────────────────────────────────────────────────┘
```

**Compact Layout (Single-Line for Very Narrow Terminals):**
```
┌────────────────────────────────────────────────────────────────────────────┐
│ Cols 1-5/23 | Rows 1-20/1234 (2%) | ←→↑↓: move | Space: page | q: exit    │
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

### Session Monitoring Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/sessions` | `/s` | List active Teradata sessions with performance metrics | `/sessions` |
| `/sysconfig` | `/sc` | Display system configuration: version and AMP count | `/sysconfig` |
| `/locks` | `/lk` | Display current lock contention and blocking chains | `/locks` |
| `/query <session_id>` | `/qi` | Show the SQL text of a session's most recent query | `/query 1023` |

**`/list databases` Metacommand**

**Requirement:** List all databases accessible on the Teradata system.

**Syntax:**
```
/list databases
\l                  -- Short alias
```

**Output Format:**
```
tq> /list databases

Databases:
┌─────────────────────┬──────────────┬─────────────┐
│ Database            │ Owner        │ Type        │
├─────────────────────┼──────────────┼─────────────┤
│ dbc                 │ DBC          │ System      │
│ production          │ dba_user     │ User        │
│ staging             │ dba_user     │ User        │
│ development         │ dev_user     │ User        │
│ analytics           │ analytics    │ User        │
└─────────────────────┴──────────────┴─────────────┘

5 databases found
```

**Behavior Requirements:**

1. **Database Discovery:** Query Teradata system catalog (DBC.DatabasesV or equivalent) to retrieve all databases
2. **Display Columns:**
   - Database name
   - Owner
   - Type (System/User)
3. **Sorting:** Alphabetical by database name, with system databases (dbc, etc.) listed first
4. **Empty Result:** If no databases found (unlikely), display "No databases found"
5. **Error Handling:** If query fails due to permissions, display error message with explanation

**Error Cases:**

**Permission denied:**
```
tq> /list databases

Error: Unable to list databases
Reason: Insufficient privileges to query system catalog
```

**Connection lost:**
```
tq> /list databases

Error: Cannot list databases - connection lost
Use /reconnect to establish new connection
```

**Acceptance Test:**
- Execute `/list databases` and verify all accessible databases are shown
- Verify system database `dbc` appears in results
- Verify results are sorted (system databases first, then alphabetical)

---

**`/list tables [pattern]` Metacommand**

**Requirement:** List tables in the current database, with optional pattern filtering.

**Syntax:**
```
/list tables                    -- List all tables in current database
/list tables <pattern>          -- List tables matching glob pattern
\dt                             -- Short alias (all tables)
\dt <pattern>                   -- Short alias with pattern
```

**Pattern Format:**
- Standard SQL LIKE patterns: `%` (any characters), `_` (single character)
- Case-insensitive matching
- Examples: `emp%`, `%_temp`, `sales_2024_%`

**Output Format:**

**Without pattern (all tables):**
```
tq> /list tables

Tables in 'production':
┌─────────────────────┬──────────┬──────────────┬───────────────┐
│ Table               │ Type     │ Rows (Est.)  │ Size          │
├─────────────────────┼──────────┼──────────────┼───────────────┤
│ customers           │ Table    │ 1,234,567    │ 45.2 MB       │
│ employees           │ Table    │ 42,573       │ 2.1 MB        │
│ orders              │ Table    │ 9,876,543    │ 320.5 MB      │
│ products            │ Table    │ 15,432       │ 890 KB        │
└─────────────────────┴──────────┴──────────────┴───────────────┘

4 tables found in database 'production'
```

**With pattern:**
```
tq> /list tables emp%

Tables in 'production' matching 'emp%':
┌─────────────────────┬──────────┬──────────────┬───────────────┐
│ Table               │ Type     │ Rows (Est.)  │ Size          │
├─────────────────────┼──────────┼──────────────┼───────────────┤
│ employees           │ Table    │ 42,573       │ 2.1 MB        │
│ emp_archive         │ Table    │ 8,123        │ 512 KB        │
└─────────────────────┴──────────┴──────────────┴───────────────┘

2 tables found matching 'emp%'
```

**Qualified pattern (with database name):**
```
tq> /list tables staging.test_%

Tables in 'staging' matching 'test_%':
┌─────────────────────┬──────────┬──────────────┬───────────────┐
│ Table               │ Type     │ Rows (Est.)  │ Size          │
├─────────────────────┼──────────┼──────────────┼───────────────┤
│ test_customers      │ Table    │ 100          │ 8 KB          │
│ test_orders         │ Table    │ 250          │ 12 KB         │
└─────────────────────┴──────────┴──────────────┴───────────────┘

2 tables found in 'staging' matching 'test_%'
```

**Behavior Requirements:**

1. **Current Database Context:** Without qualified name, list tables in current database only
2. **Qualified Pattern:** Support `database.pattern` format to list tables in different database
3. **Display Columns:**
   - Table name
   - Type (Table, always "Table" for this command)
   - Estimated row count
   - Approximate size
4. **Sorting:** Alphabetical by table name
5. **Empty Result:** If no tables found, display "No tables found in database 'X'" or "No tables matching 'pattern'"
6. **Data Source:** Query Teradata system catalog (DBC.TablesV) WHERE TableKind = 'T'

**Error Cases:**

**No current database:**
```
tq> /list tables

Error: No current database selected
Use /logon to connect to a database
```

**Pattern matches no tables:**
```
tq> /list tables xyz%

No tables found in database 'production' matching 'xyz%'
```

**Invalid pattern:**
```
tq> /list tables [invalid

Error: Invalid pattern syntax
Use SQL LIKE patterns: % (any characters), _ (single character)
```

**Permission denied:**
```
tq> /list tables restricted_db.%

Error: Unable to list tables in database 'restricted_db'
Reason: Insufficient privileges
```

**Acceptance Test:**
- Execute `/list tables` and verify all tables in current database are shown
- Execute `/list tables emp%` and verify only matching tables shown
- Execute `/list tables staging.test_%` and verify tables from different database
- Execute `/list tables nonexistent%` and verify "No tables found" message

---

**`/list views` Metacommand**

**Requirement:** List views in the current database.

**Syntax:**
```
/list views
\dv                             -- Short alias
```

**Output Format:**
```
tq> /list views

Views in 'production':
┌─────────────────────────┬──────────────┬─────────────────────────┐
│ View                    │ Owner        │ Definition (truncated)  │
├─────────────────────────┼──────────────┼─────────────────────────┤
│ active_employees        │ dba_user     │ SELECT * FROM employe...│
│ sales_summary           │ analytics    │ SELECT dept, SUM(sal... │
│ customer_orders_view    │ dba_user     │ SELECT c.*, o.* FROM ... │
└─────────────────────────┴──────────────┴─────────────────────────┘

3 views found in database 'production'
```

**Behavior Requirements:**

1. **Current Database Context:** List views in current database only
2. **Display Columns:**
   - View name
   - Owner
   - View definition (first 50 characters, truncated with `...`)
3. **Sorting:** Alphabetical by view name
4. **Empty Result:** If no views found, display "No views found in database 'X'"
5. **Data Source:** Query Teradata system catalog (DBC.TablesV) WHERE TableKind = 'V'

**Error Cases:**

**No current database:**
```
tq> /list views

Error: No current database selected
Use /logon to connect to a database
```

**No views exist:**
```
tq> /list views

No views found in database 'development'
```

**Permission denied:**
```
tq> /list views

Error: Unable to list views in database 'production'
Reason: Insufficient privileges to query system catalog
```

**Acceptance Test:**
- Execute `/list views` in database with views and verify all shown
- Execute `/list views` in database without views and verify "No views found" message
- Verify view definitions are truncated to reasonable length

---

**`/sessions` Metacommand**

**Requirement:** List all active sessions on the Teradata system with key performance metrics.

**Syntax:**
```
/sessions
/s                  -- Short alias
```

**Output Format:**
```
tq> /sessions

Active Sessions on <hostname>:
┌───────────┬──────────┬────────────────────────┬─────────────┬──────────┬───────────┬───────┬─────────────┬────────────────┬──────────────┐
│ SessionNo │ UserName │ LogonTime              │ PEstate     │ AMPState │ AMPCPUSec │ AMPIO │ ReqSpool    │ Amp CPU Skew % │ Amp IO Skew %│
├───────────┼──────────┼────────────────────────┼─────────────┼──────────┼───────────┼───────┼─────────────┼────────────────┼──────────────┤
│      1076 │ DBC      │ 2026/01/27 15:33:26.00 │ IDLE        │ IDLE     │         0 │     6 │           0 │           [--] │         [--] │
│      1077 │ DBC      │ 2026/01/27 15:33:27.00 │ IDLE        │ IDLE     │     0.376 │  6782 │           0 │           [--] │         [--] │
│      1078 │ DBC      │ 2026/01/27 15:33:28.00 │ DISPATCHING │ ACTIVE   │   366.736 │ 75335 │ 26753187840 │           2.87 │         3.78 │
│      1079 │ alice    │ 2026/01/27 16:15:42.00 │ ACTIVE      │ ACTIVE   │    15.234 │  3421 │   123456789 │           0.15 │         0.23 │
└───────────┴──────────┴────────────────────────┴─────────────┴──────────┴───────────┴───────┴─────────────┴────────────────┴──────────────┘

4 sessions found (Query time: 0.234s)
```

**Column Descriptions:**

| Column | Type | Description |
|--------|------|-------------|
| SessionNo | INTEGER | Session identifier (unique per connection) |
| UserName | VARCHAR | User account logged into this session |
| LogonTime | TIMESTAMP | When session was established (YYYY/MM/DD HH:MM:SS.ss format) |
| PEstate | VARCHAR | Parsing Engine state: IDLE, DISPATCHING, ACTIVE |
| AMPState | VARCHAR | Access Module Processor state: IDLE, ACTIVE |
| AMPCPUSec | DECIMAL | Total AMP CPU seconds consumed by this session |
| AMPIO | INTEGER | Total AMP I/O count for this session |
| ReqSpool | BIGINT | Requested spool space in bytes |
| Amp CPU Skew % | DECIMAL(4,2) | CPU distribution skew across AMPs (0% = perfect balance, higher = unbalanced) |
| Amp IO Skew % | DECIMAL(4,2) | I/O distribution skew across AMPs (0% = perfect balance, higher = unbalanced) |

**Behavior Requirements:**

**REQ-SESS-001: Command Availability and Aliases**

The `/sessions` command SHALL be available as a metacommand in REPL mode with the following characteristics:

1. **REQ-SESS-001.1** - Primary command: `/sessions`
2. **REQ-SESS-001.2** - Short alias: `/s`
3. **REQ-SESS-001.3** - Both forms SHALL execute identically
4. **REQ-SESS-001.4** - Command SHALL execute immediately (no arguments required)
5. **REQ-SESS-001.5** - Command SHALL be case-insensitive (`/Sessions`, `/SESSIONS`, `/s` all valid)

**REQ-SESS-002: Data Source and Query Execution**

The command SHALL retrieve session information from Teradata system catalog using the MonitorSession table function:

1. **REQ-SESS-002.1** - Data source: `MonitorSession(-1,'*',0)` table function
2. **REQ-SESS-002.2** - Query scope: All sessions system-wide (parameter -1)
3. **REQ-SESS-002.3** - SQL query template:
   ```sql
   SELECT
       SessionNo,
       UserName,
       LogonTime,
       PEState,
       AMPState,
       AMPCPUSec,
       AMPIO,
       ReqSpool,
       (100 * (1 - (AvgAmpCPUSec / NULLIFZERO(HotAmp1CPU))))(DECIMAL(4,2)) AS "Amp CPU Skew %",
       (100 * (1 - (AvgAmpIOCnt / NULLIFZERO(HotAmp1IO))))(DECIMAL(4,2)) AS "Amp IO Skew %"
   FROM TABLE (MonitorSession(-1,'*',0)) AS t1;
   ```
4. **REQ-SESS-002.4** - Column ordering SHALL match the query above (SessionNo first, skew percentages last)
5. **REQ-SESS-002.5** - Query execution SHALL respect current connection timeout settings
6. **REQ-SESS-002.6** - Query execution time SHALL be displayed in summary footer
7. **REQ-SESS-002.7** - ALL sessions returned by the query SHALL be displayed in the output (no filtering by PEState, AMPState, or any other criteria)
8. **REQ-SESS-002.8** - Sessions in ALL state combinations SHALL be included: IDLE/IDLE, IDLE/ACTIVE, DISPATCHING/IDLE, DISPATCHING/ACTIVE, ACTIVE/IDLE, ACTIVE/ACTIVE

**REQ-SESS-003: Output Formatting and Display**

The command SHALL format output as a table with the following requirements:

1. **REQ-SESS-003.1** - Default output format: Table (box-drawing characters)
2. **REQ-SESS-003.2** - Column headers SHALL match column names exactly (including spaces in "Amp CPU Skew %")
3. **REQ-SESS-003.3** - LogonTime format: `YYYY/MM/DD HH:MM:SS.ss` (Teradata default timestamp format)
4. **REQ-SESS-003.4** - Numeric columns SHALL be right-aligned
5. **REQ-SESS-003.5** - Text columns SHALL be left-aligned
6. **REQ-SESS-003.6** - Column widths SHALL auto-adjust to content (minimum width: header width)
7. **REQ-SESS-003.7** - Table SHALL include top border, header row, header separator, data rows, and bottom border
8. **REQ-SESS-003.8** - Summary footer format: `N sessions found (Query time: X.XXXs)`

**REQ-SESS-004: NULL and Special Value Handling**

The command SHALL handle NULL and edge-case values appropriately:

1. **REQ-SESS-004.1** - NULL skew percentages (for IDLE sessions): Display as `[--]` (not `[NULL]` or blank)
2. **REQ-SESS-004.2** - Skew percentage format: `X.XX` (two decimal places, no leading zeros)
   - Examples: `0.15`, `12.34`, `99.99`
3. **REQ-SESS-004.3** - Skew percentage range: 0.00 to 100.00 (validated, warn if out of range)
4. **REQ-SESS-004.4** - Large spool values: Display with thousand separators (e.g., `26,753,187,840`)
5. **REQ-SESS-004.5** - Zero CPU/IO: Display as `0` (not blank)
6. **REQ-SESS-004.6** - Very small CPU values (<1 second): Display with full precision (e.g., `0.376`)

**REQ-SESS-005: Error Handling and Edge Cases**

The command SHALL handle errors and edge cases gracefully:

**Insufficient Privileges:**
```
tq> /sessions

Error: Unable to list sessions
Reason: SELECT permission denied on DBC.MonitorSession

This command requires SELECT access to the MonitorSession table function.
Contact your DBA to request access or use the GRANT statement:
  GRANT SELECT ON DBC.MonitorSession TO <your_username>;
```

**No Active Sessions (besides current):**
```
tq> /sessions

Active Sessions on localhost:
┌───────────┬──────────┬────────────────────────┬─────────┬──────────┬───────────┬───────┬──────────┬────────────────┬──────────────┐
│ SessionNo │ UserName │ LogonTime              │ PEstate │ AMPState │ AMPCPUSec │ AMPIO │ ReqSpool │ Amp CPU Skew % │ Amp IO Skew %│
├───────────┼──────────┼────────────────────────┼─────────┼──────────┼───────────┼───────┼──────────┼────────────────┼──────────────┤
│      1076 │ dbc      │ 2026/01/27 15:33:26.00 │ ACTIVE  │ ACTIVE   │     0.123 │    45 │        0 │           [--] │         [--] │
└───────────┴──────────┴────────────────────────┴─────────┴──────────┴───────────┴───────��──────────┴────────────────┴──────────────┘

1 session found (Query time: 0.012s)
```

**Connection Lost:**
```
tq> /sessions

Error: Cannot list sessions - connection lost
Use /reconnect to establish new connection
```

**Query Timeout:**
```
tq> /sessions

Error: Query timeout after 30s
The MonitorSession query may be slow on heavily loaded systems.
Try increasing timeout: /set timeout 60s
```

**MonitorSession Function Not Available (Old Teradata Version):**
```
tq> /sessions

Error: MonitorSession table function not found
This feature requires Teradata 14.10 or later.
Current database version: 13.10

Alternative: Use DBC.SessionTbl view (limited metrics)
```

**Specific Requirements:**

1. **REQ-SESS-005.1** - Privilege errors SHALL include helpful explanation and GRANT statement example
2. **REQ-SESS-005.2** - Empty result set SHALL still display table with headers
3. **REQ-SESS-005.3** - Connection errors SHALL suggest `/reconnect` metacommand
4. **REQ-SESS-005.4** - Timeout errors SHALL suggest timeout adjustment
5. **REQ-SESS-005.5** - Version compatibility errors SHALL suggest alternative approaches
6. **REQ-SESS-005.6** - All errors SHALL return to REPL prompt (non-fatal)

**REQ-SESS-006: Tab Completion and Help Integration**

The command SHALL be discoverable through standard REPL features:

1. **REQ-SESS-006.1** - Tab completion: Typing `/s<TAB>` SHALL suggest `/sessions` and `/sample`
2. **REQ-SESS-006.2** - Tab completion: Typing `/sess<TAB>` SHALL auto-complete to `/sessions`
3. **REQ-SESS-006.3** - Help text: `/help` SHALL list `/sessions` command with description
4. **REQ-SESS-006.4** - Help text description: "List active Teradata sessions with performance metrics"
5. **REQ-SESS-006.5** - Detailed help: `/help sessions` SHALL display extended help including column descriptions
6. **REQ-SESS-006.6** - Command SHALL appear in metacommand list when typing `/<TAB>`

**REQ-SESS-007: Output Format Compatibility**

The command SHALL work with all output format modes:

1. **REQ-SESS-007.1** - Table format (default): Box-drawing table as shown above
2. **REQ-SESS-007.2** - CSV format: Standard CSV with headers, NULL skew as empty string
   ```csv
   SessionNo,UserName,LogonTime,PEstate,AMPState,AMPCPUSec,AMPIO,ReqSpool,Amp CPU Skew %,Amp IO Skew %
   1076,DBC,2026/01/27 15:33:26.00,IDLE,IDLE,0,6,0,,
   1078,DBC,2026/01/27 15:33:28.00,ACTIVE,ACTIVE,366.736,75335,26753187840,2.87,3.78
   ```
3. **REQ-SESS-007.3** - JSON format: Array of objects, NULL skew as `null`
   ```json
   [
     {
       "SessionNo": 1076,
       "UserName": "DBC",
       "LogonTime": "2026/01/27 15:33:26.00",
       "PEstate": "IDLE",
       "AMPState": "IDLE",
       "AMPCPUSec": 0.0,
       "AMPIO": 6,
       "ReqSpool": 0,
       "Amp CPU Skew %": null,
       "Amp IO Skew %": null
     },
     {
       "SessionNo": 1078,
       "UserName": "DBC",
       "LogonTime": "2026/01/27 15:33:28.00",
       "PEstate": "ACTIVE",
       "AMPState": "ACTIVE",
       "AMPCPUSec": 366.736,
       "AMPIO": 75335,
       "ReqSpool": 26753187840,
       "Amp CPU Skew %": 2.87,
       "Amp IO Skew %": 3.78
     }
   ]
   ```
4. **REQ-SESS-007.4** - Format selection: Command SHALL respect current output format setting (`/set format <fmt>`)
5. **REQ-SESS-007.5** - Format override: NOT supported (use `/set format` before running command)

**REQ-SESS-008: Performance and Resource Considerations**

The command SHALL execute efficiently and provide feedback:

1. **REQ-SESS-008.1** - Target execution time: <1 second for systems with <1000 sessions
2. **REQ-SESS-008.2** - Loading indicator: Display "Loading session information..." if query takes >500ms
3. **REQ-SESS-008.3** - Result caching: NOT cached (each execution is fresh query for real-time monitoring)
4. **REQ-SESS-008.4** - Query cancellation: Ctrl-C SHALL cancel query and return to prompt
5. **REQ-SESS-008.5** - Resource impact: Query SHALL use read-only system views (no locks, no modifications)

**REQ-SESS-009: Skew Percentage Interpretation Guidelines**

The command output SHALL include documentation-referenced guidance for interpreting skew percentages:

1. **REQ-SESS-009.1** - Skew metrics indicate workload distribution imbalance across AMPs
2. **REQ-SESS-009.2** - Interpretation guidelines SHALL be documented in user guide
3. **REQ-SESS-009.3** - Interpretation ranges:
   - **0-5%**: Excellent distribution (well-balanced query)
   - **5-15%**: Good distribution (acceptable for most workloads)
   - **15-25%**: Moderate skew (consider optimization if persistent)
   - **>25%**: High skew (investigate join conditions, PI/NUPI distribution, statistics)
4. **REQ-SESS-009.4** - NULL skew (`[--]`) indicates IDLE session with no active AMP work
5. **REQ-SESS-009.5** - Actionable guidance for DBAs:
   - High CPU skew: Review join conditions, collect statistics, check PI distribution
   - High I/O skew: Investigate data distribution, check for hot spots, review access patterns
   - Persistent skew: Candidate for table redesign or partitioning strategy
6. **REQ-SESS-009.6** - Context matters:
   - Batch ETL jobs: Occasional high skew acceptable during specific operations
   - Interactive queries: Persistent high skew degrades user experience
   - Small result sets: Skew less impactful than on large scans

**Example Interaction:**

**Basic usage:**
```sql
tq> /sessions
[Query executes, displays table with 15 sessions]

tq> /s
[Same output, using short alias]
```

**With active queries:**
```sql
tq> /sessions

Active Sessions on prod-td01.company.com:
┌───────────┬──────────┬────────────────────────┬─────────────┬──────────┬───────────┬────────┬──────────────┬────────────────┬──────────────┐
│ SessionNo │ UserName │ LogonTime              │ PEstate     │ AMPState │ AMPCPUSec │ AMPIO  │ ReqSpool     │ Amp CPU Skew % │ Amp IO Skew %│
├───────────┼──────────┼────────────────────────┼─────────────┼──────────┼───────────┼────────┼──────────────┼────────────────┼──────────────┤
│      1023 │ etl_user │ 2026/01/27 08:15:00.00 │ ACTIVE      │ ACTIVE   │   4523.45 │ 892341 │ 512000000000 │          25.43 │        32.19 │
│      1024 │ analyst1 │ 2026/01/27 09:30:15.00 │ DISPATCHING │ ACTIVE   │    123.67 │  45231 │   8500000000 │           1.23 │         2.45 │
│      1025 │ dbc      │ 2026/01/27 14:22:33.00 │ IDLE        │ IDLE     │      0.05 │     12 │            0 │           [--] │         [--] │
└───────────┴──────────┴────────────────────────┴─────────────┴──────────┴───────────┼────────┼──────────────┼────────────────┼──────────────┤

3 sessions found (Query time: 0.156s)
```

**Tab completion:**
```sql
tq> /s<TAB>
Matching metacommands:
    /sample      Show random sample
    /sessions    List active Teradata sessions with performance metrics

tq> /sess<TAB>
tq> /sessions_
[Auto-completed]
```

**With different output format:**
```sql
tq> /set format csv
Output format set to: csv

tq> /sessions
SessionNo,UserName,LogonTime,PEstate,AMPState,AMPCPUSec,AMPIO,ReqSpool,Amp CPU Skew %,Amp IO Skew %
1076,DBC,2026/01/27 15:33:26.00,IDLE,IDLE,0,6,0,,
1078,DBC,2026/01/27 15:33:28.00,ACTIVE,ACTIVE,366.736,75335,26753187840,2.87,3.78
```

**Acceptance Test:**
- Execute `/sessions` and verify all active sessions are displayed with correct columns
- Execute `/s` (alias) and verify identical behavior
- Verify skew percentages show `[--]` for IDLE sessions
- Verify skew percentages show decimal values (X.XX format) for ACTIVE sessions
- Trigger privilege error by revoking access and verify helpful error message
- Execute on system with single session and verify table still displays
- Execute with `/set format csv` and verify CSV output
- Execute with `/set format json` and verify JSON output
- Type `/s<TAB>` and verify tab completion suggestions include `/sessions`
- Execute `/help` and verify `/sessions` appears in command list

---

**`/sysconfig` Metacommand**

**Requirement:** Display a compact system configuration summary showing Teradata version and AMP count so DBAs can quickly verify software version and system scale.

**Syntax:**
```
/sysconfig
/sc                 -- Short alias
```

**Output Format:**
```
tq> /sysconfig

System Configuration:
┌──────────────────┬─────────────────────────────────────┐
│ Property         │ Value                               │
├──────────────────┼─────────────────────────────────────┤
│ Teradata Version │ 17.20.00.17                         │
│ Release          │ 17.20.00.17 (Released: 2024-01-15)  │
│ AMP Count        │ 128                                 │
└──────────────────┴─────────────────────────────────────┘
```

**Property Descriptions:**

| Property | Description |
|----------|-------------|
| Teradata Version | Installed software version from DBC.DBCInfoV |
| Release | Full release string including build date |
| AMP Count | Total number of Access Module Processors (via HASHAMP()+1) |

**Behavior Requirements:**

**REQ-SYSCONFIG-001: Command Availability and Aliases**

The `/sysconfig` command SHALL be available as a metacommand in REPL mode with the following characteristics:

1. **REQ-SYSCONFIG-001.1** - Primary command: `/sysconfig`
2. **REQ-SYSCONFIG-001.2** - Short alias: `/sc`
3. **REQ-SYSCONFIG-001.3** - Both forms SHALL execute identically
4. **REQ-SYSCONFIG-001.4** - Command SHALL execute immediately (no arguments required)
5. **REQ-SYSCONFIG-001.5** - Command SHALL be case-insensitive (`/Sysconfig`, `/SYSCONFIG`, `/sc` all valid)

**REQ-SYSCONFIG-002: Data Source and Query Execution**

The command SHALL retrieve system configuration from Teradata system views:

1. **REQ-SYSCONFIG-002.1** - Teradata version and release: Query `DBC.DBCInfoV` (InfoKey IN ('RELEASE','VERSION'))
2. **REQ-SYSCONFIG-002.2** - AMP count: Compute via `HASHAMP() + 1`
3. **REQ-SYSCONFIG-002.3** - Query execution time SHALL be displayed in summary footer
4. **REQ-SYSCONFIG-002.4** - Queries SHALL be read-only (no side effects)

**REQ-SYSCONFIG-003: Output Formatting and Display**

The command SHALL format output as a two-column key-value table:

1. **REQ-SYSCONFIG-003.1** - Default output format: Two-column key-value table (box-drawing characters)
2. **REQ-SYSCONFIG-003.2** - Column 1 header: `Property`; Column 2 header: `Value`
3. **REQ-SYSCONFIG-003.3** - Properties displayed in order: Teradata Version, Release, AMP Count
4. **REQ-SYSCONFIG-003.4** - Header line: `System Configuration:` above the table
5. **REQ-SYSCONFIG-003.5** - Column widths SHALL auto-adjust to content

**REQ-SYSCONFIG-004: Output Format Compatibility**

The command SHALL work with all output format modes:

1. **REQ-SYSCONFIG-004.1** - Table format (default): Two-column key-value table as shown above
2. **REQ-SYSCONFIG-004.2** - CSV format: Two columns (`Property,Value`) with one row per property
   ```csv
   Property,Value
   Teradata Version,17.20.00.17
   Release,"17.20.00.17 (Released: 2024-01-15)"
   AMP Count,128
   ```
3. **REQ-SYSCONFIG-004.3** - JSON format: Single object with property names as keys
   ```json
   {
     "Teradata Version": "17.20.00.17",
     "Release": "17.20.00.17 (Released: 2024-01-15)",
     "AMP Count": 128
   }
   ```
4. **REQ-SYSCONFIG-004.4** - Format selection: Command SHALL respect current output format setting (`/set format <fmt>`)

**REQ-SYSCONFIG-005: Error Handling**

The command SHALL handle errors and edge cases gracefully:

**Insufficient Privileges:**
```
tq> /sysconfig

Error: Unable to retrieve system configuration
Reason: SELECT permission denied on DBC.DBCInfoV

This command requires SELECT access to DBC system views.
Contact your DBA to request access or use the GRANT statement:
  GRANT SELECT ON DBC.DBCInfoV TO <your_username>;
```

**Connection Lost:**
```
tq> /sysconfig

Error: Cannot retrieve system configuration - connection lost
Use /reconnect to establish new connection
```

**Specific Requirements:**

1. **REQ-SYSCONFIG-005.1** - Privilege errors SHALL include helpful explanation and GRANT statement example
2. **REQ-SYSCONFIG-005.2** - Connection errors SHALL suggest `/reconnect` metacommand
3. **REQ-SYSCONFIG-005.3** - All errors SHALL return to REPL prompt (non-fatal)
4. **REQ-SYSCONFIG-005.4** - If a specific property cannot be retrieved, display `[unavailable]` for that row rather than failing the entire command

**REQ-SYSCONFIG-006: Tab Completion and Help Integration**

The command SHALL be discoverable through standard REPL features:

1. **REQ-SYSCONFIG-006.1** - Tab completion: Typing `/sc<TAB>` SHALL auto-complete to `/sysconfig`
2. **REQ-SYSCONFIG-006.2** - Tab completion: Typing `/sys<TAB>` SHALL auto-complete to `/sysconfig`
3. **REQ-SYSCONFIG-006.3** - Help text: `/help` SHALL list `/sysconfig` command with description
4. **REQ-SYSCONFIG-006.4** - Help text description: "Display system configuration (version and AMP count)"
5. **REQ-SYSCONFIG-006.5** - Detailed help: `/help sysconfig` SHALL display extended help including property descriptions
6. **REQ-SYSCONFIG-006.6** - Command SHALL appear in metacommand list when typing `/<TAB>`

**Example Help Output:**
```sql
tq> /help sysconfig

/sysconfig - Display system configuration summary

SYNTAX:
  /sysconfig
  /sc                        Short alias

DESCRIPTION:
  Display a compact summary of the Teradata system configuration including
  software version and AMP count. Useful for verifying system version and
  scale at a glance.

  Data is retrieved from DBC.DBCInfoV and HASHAMP()+1.
  Requires SELECT privilege on DBC system views.

EXAMPLES:
  /sysconfig                 Show system configuration
  /sc                        Same using short alias

RELATED COMMANDS:
  /sessions                  List active sessions
  /locks                     Display lock contention
  /query                     Show current SQL for a session

For more information, see documentation at: docs/user/metacommands.md
```

**REQ-SYSCONFIG-007: Performance Requirements**

1. **REQ-SYSCONFIG-007.1** - Target execution time: <500ms
2. **REQ-SYSCONFIG-007.2** - Loading indicator: Display "Loading system configuration..." if query takes >500ms
3. **REQ-SYSCONFIG-007.3** - Result caching: NOT cached (always queries fresh for current state)
4. **REQ-SYSCONFIG-007.4** - Query cancellation: Ctrl-C SHALL cancel query and return to prompt

**Example Interaction:**

**Basic usage:**
```sql
tq> /sysconfig

System Configuration:
┌──────────────────┬─────────────────────────────────────┐
│ Property         │ Value                               │
├──────────────────┼─────────────────────────────────────┤
│ Teradata Version │ 17.20.00.17                         │
│ Release          │ 17.20.00.17 (Released: 2024-01-15)  │
│ AMP Count        │ 128                                 │
└──────────────────┴─────────────────────────────────────┘

tq> /sc
[Same output using short alias]
```

**With JSON output:**
```sql
tq> /set format json
Output format set to: json

tq> /sysconfig
{
  "Teradata Version": "17.20.00.17",
  "Release": "17.20.00.17 (Released: 2024-01-15)",
  "AMP Count": 128
}
```

**Tab completion:**
```sql
tq> /sc<TAB>
tq> /sysconfig_
[Auto-completed]

tq> /sys<TAB>
tq> /sysconfig_
[Auto-completed]
```

**Acceptance Test:**
- Execute `/sysconfig` and verify all three properties are displayed: Teradata Version, Release, AMP Count
- Execute `/sc` (alias) and verify identical behavior
- Execute with `/set format csv` and verify CSV output with Property,Value headers and three data rows
- Execute with `/set format json` and verify JSON object output with three keys
- Trigger privilege error and verify helpful error message with GRANT example
- Type `/sc<TAB>` and verify tab completion resolves to `/sysconfig`
- Execute `/help sysconfig` and verify extended help is displayed

---

**`/locks` Metacommand**

**Requirement:** Display current lock contention information showing locked objects, lock types, locking sessions, and waiting sessions so DBAs can diagnose and resolve blocking issues.

**Syntax:**
```
/locks
/lk                 -- Short alias
```

**Output Format:**

**When locks exist:**
```
tq> /locks

Lock Information:
┌──────────────────────┬───────────┬────────────┬──────────────┬──────────────┐
│ Locked Object        │ Lock Type │ Lock Mode  │ Locking Sess │ Waiting Sess │
├──────────────────────┼───────────┼────────────┼──────────────┼──────────────┤
│ PRODUCTION.orders    │ Table     │ WRITE      │ 1023         │ 1045, 1067   │
│ PRODUCTION.customers │ Table     │ EXCLUSIVE  │ 1023         │ 1051         │
│ PRODUCTION.employees │ Row Hash  │ READ       │ 1078         │ (none)       │
└──────────────────────┴───────────┴────────────┴──────────────┴──────────────┘

3 lock(s) found - 1 blocking chain(s) detected (Query time: 0.089s)

Blocking Chain:
  Session 1023 blocks sessions: 1045, 1051, 1067
```

**When no locks exist:**
```
tq> /locks

Lock Information:
No locks currently held.

(Query time: 0.023s)
```

**Column Descriptions:**

| Column | Type | Description |
|--------|------|-------------|
| Locked Object | VARCHAR | Fully qualified name of the locked database object (database.table) |
| Lock Type | VARCHAR | Granularity of the lock: Table, Row Hash, Database |
| Lock Mode | VARCHAR | Lock severity: READ, WRITE, EXCLUSIVE, ACCESS |
| Locking Sess | INTEGER | Session ID that holds the lock |
| Waiting Sess | VARCHAR | Comma-separated list of session IDs waiting for this lock, or `(none)` when no waiters |

**Lock Mode Definitions:**

| Lock Mode | Description |
|-----------|-------------|
| ACCESS | Weakest lock - prevents only EXCLUSIVE locks. Allows concurrent reads and writes. |
| READ | Shared lock - allows concurrent reads, blocks WRITE and EXCLUSIVE. |
| WRITE | Exclusive on writes - blocks other WRITE and EXCLUSIVE, allows READ. |
| EXCLUSIVE | Strongest lock - blocks all other lock modes including ACCESS. |

**Behavior Requirements:**

**REQ-LOCKS-001: Command Availability and Aliases**

The `/locks` command SHALL be available as a metacommand in REPL mode with the following characteristics:

1. **REQ-LOCKS-001.1** - Primary command: `/locks`
2. **REQ-LOCKS-001.2** - Short alias: `/lk`
3. **REQ-LOCKS-001.3** - Both forms SHALL execute identically
4. **REQ-LOCKS-001.4** - Command SHALL execute immediately (no arguments required)
5. **REQ-LOCKS-001.5** - Command SHALL be case-insensitive (`/Locks`, `/LOCKS`, `/lk` all valid)

**REQ-LOCKS-002: Data Source and Query Execution**

The command SHALL retrieve lock information from Teradata system views:

1. **REQ-LOCKS-002.1** - Primary data source: `DBC.LockInfoV` (or platform-equivalent view)
2. **REQ-LOCKS-002.2** - Query scope: All current locks system-wide
3. **REQ-LOCKS-002.3** - Query results SHALL include all active locks regardless of lock mode
4. **REQ-LOCKS-002.4** - Query execution time SHALL be displayed in summary footer
5. **REQ-LOCKS-002.5** - Queries SHALL be read-only (no side effects)
6. **REQ-LOCKS-002.6** - If `DBC.LockInfoV` is unavailable, the command SHALL report the unavailability clearly rather than silently returning empty results

**REQ-LOCKS-003: Output Formatting and Display**

The command SHALL format lock information as a table:

1. **REQ-LOCKS-003.1** - Default output format: Table (box-drawing characters)
2. **REQ-LOCKS-003.2** - Column headers SHALL match column names exactly as specified above
3. **REQ-LOCKS-003.3** - Waiting sessions: Comma-separated session IDs or `(none)` when no waiters
4. **REQ-LOCKS-003.4** - Header line: `Lock Information:` above the table
5. **REQ-LOCKS-003.5** - Summary footer format: `N lock(s) found - M blocking chain(s) detected (Query time: X.XXXs)`
6. **REQ-LOCKS-003.6** - When no locks exist, display `No locks currently held.` instead of an empty table
7. **REQ-LOCKS-003.7** - Column widths SHALL auto-adjust to content (minimum width: header width)

**REQ-LOCKS-004: Blocking Chain Identification**

The command SHALL identify and display blocking chains:

1. **REQ-LOCKS-004.1** - A blocking chain exists when one or more sessions are waiting for a lock held by another session
2. **REQ-LOCKS-004.2** - After the main lock table, display a "Blocking Chain:" section if any chains exist
3. **REQ-LOCKS-004.3** - Each chain entry format: `Session <N> blocks sessions: <id1>, <id2>, ...`
4. **REQ-LOCKS-004.4** - Multiple independent blocking chains SHALL each be listed on a separate line
5. **REQ-LOCKS-004.5** - If no blocking chains exist, the "Blocking Chain:" section SHALL NOT be shown
6. **REQ-LOCKS-004.6** - Chain detection SHALL be based on the Waiting Sess column data

**Blocking chain example with multiple chains:**
```
Blocking Chain:
  Session 1023 blocks sessions: 1045, 1051, 1067
  Session 1089 blocks sessions: 1092
```

**REQ-LOCKS-005: Output Format Compatibility**

The command SHALL work with all output format modes:

1. **REQ-LOCKS-005.1** - Table format (default): Box-drawing table as shown above
2. **REQ-LOCKS-005.2** - CSV format: Standard CSV with headers, waiting sessions as quoted comma-separated string or empty string when no waiters
   ```csv
   Locked Object,Lock Type,Lock Mode,Locking Sess,Waiting Sess
   PRODUCTION.orders,Table,WRITE,1023,"1045, 1067"
   PRODUCTION.customers,Table,EXCLUSIVE,1023,1051
   PRODUCTION.employees,Row Hash,READ,1078,
   ```
3. **REQ-LOCKS-005.3** - JSON format: Array of lock objects, waiting sessions as JSON array of integers
   ```json
   [
     {
       "Locked Object": "PRODUCTION.orders",
       "Lock Type": "Table",
       "Lock Mode": "WRITE",
       "Locking Sess": 1023,
       "Waiting Sess": [1045, 1067]
     },
     {
       "Locked Object": "PRODUCTION.customers",
       "Lock Type": "Table",
       "Lock Mode": "EXCLUSIVE",
       "Locking Sess": 1023,
       "Waiting Sess": [1051]
     },
     {
       "Locked Object": "PRODUCTION.employees",
       "Lock Type": "Row Hash",
       "Lock Mode": "READ",
       "Locking Sess": 1078,
       "Waiting Sess": []
     }
   ]
   ```
4. **REQ-LOCKS-005.4** - Format selection: Command SHALL respect current output format setting (`/set format <fmt>`)
5. **REQ-LOCKS-005.5** - In CSV and JSON formats, the Blocking Chain section SHALL NOT be included (chains can be derived from the data)

**REQ-LOCKS-006: Error Handling**

The command SHALL handle errors and edge cases gracefully:

**Insufficient Privileges:**
```
tq> /locks

Error: Unable to retrieve lock information
Reason: SELECT permission denied on DBC.LockInfoV

This command requires SELECT access to DBC lock views.
Contact your DBA to request access or use the GRANT statement:
  GRANT SELECT ON DBC.LockInfoV TO <your_username>;
```

**Lock View Not Available:**
```
tq> /locks

Error: Lock information view not available
DBC.LockInfoV is not accessible on this system.

This may indicate a Teradata version compatibility issue or a
configuration restriction. Contact your DBA for assistance.
```

**Connection Lost:**
```
tq> /locks

Error: Cannot retrieve lock information - connection lost
Use /reconnect to establish new connection
```

**Specific Requirements:**

1. **REQ-LOCKS-006.1** - Privilege errors SHALL include helpful explanation and GRANT statement example
2. **REQ-LOCKS-006.2** - View availability errors SHALL explain the issue without suggesting it is a privilege problem
3. **REQ-LOCKS-006.3** - Connection errors SHALL suggest `/reconnect` metacommand
4. **REQ-LOCKS-006.4** - All errors SHALL return to REPL prompt (non-fatal)
5. **REQ-LOCKS-006.5** - Empty result (no locks) SHALL display the "No locks currently held." message, not an error

**REQ-LOCKS-007: Tab Completion and Help Integration**

The command SHALL be discoverable through standard REPL features:

1. **REQ-LOCKS-007.1** - Tab completion: Typing `/lk<TAB>` SHALL auto-complete to `/locks`
2. **REQ-LOCKS-007.2** - Tab completion: Typing `/lo<TAB>` SHALL suggest `/locks` and `/logon`
3. **REQ-LOCKS-007.3** - Help text: `/help` SHALL list `/locks` command with description
4. **REQ-LOCKS-007.4** - Help text description: "Display current lock contention and blocking chains"
5. **REQ-LOCKS-007.5** - Detailed help: `/help locks` SHALL display extended help including lock mode definitions
6. **REQ-LOCKS-007.6** - Command SHALL appear in metacommand list when typing `/<TAB>`

**Example Help Output:**
```sql
tq> /help locks

/locks - Display current lock contention and blocking chains

SYNTAX:
  /locks
  /lk                        Short alias

DESCRIPTION:
  Display all current locks held on the Teradata system, identifying
  which sessions hold locks, which sessions are waiting, and the
  nature of the contention. Blocking chains are automatically
  identified and summarized.

  Lock Modes:
    ACCESS    - Weakest lock; blocks only EXCLUSIVE
    READ      - Shared lock; blocks WRITE and EXCLUSIVE
    WRITE     - Blocks other WRITE and EXCLUSIVE
    EXCLUSIVE - Strongest lock; blocks all other modes

  Data is retrieved from DBC.LockInfoV.
  Requires SELECT privilege on DBC system views.

EXAMPLES:
  /locks                     Show all current locks
  /lk                        Same using short alias

RELATED COMMANDS:
  /sessions                  List active sessions
  /sysconfig                 Display system configuration
  /query                     Show current SQL for a session

For more information, see documentation at: docs/user/metacommands.md
```

**REQ-LOCKS-008: Performance Requirements**

1. **REQ-LOCKS-008.1** - Target execution time: <1 second for systems with typical lock activity
2. **REQ-LOCKS-008.2** - Loading indicator: Display "Loading lock information..." if query takes >500ms
3. **REQ-LOCKS-008.3** - Result caching: NOT cached (each execution is a fresh query for real-time monitoring)
4. **REQ-LOCKS-008.4** - Query cancellation: Ctrl-C SHALL cancel query and return to prompt
5. **REQ-LOCKS-008.5** - Resource impact: Query SHALL use read-only system views (no locks, no modifications)

**Example Interaction:**

**No locks:**
```sql
tq> /locks

Lock Information:
No locks currently held.

(Query time: 0.023s)
```

**With active locks and blocking:**
```sql
tq> /locks

Lock Information:
┌──────────────────────┬───────────┬────────────┬──────────────┬──────────────┐
│ Locked Object        │ Lock Type │ Lock Mode  │ Locking Sess │ Waiting Sess │
├──────────────────────┼───────────┼────────────┼──────────────┼──────────────┤
│ PRODUCTION.orders    │ Table     │ WRITE      │ 1023         │ 1045, 1067   │
│ PRODUCTION.customers │ Table     │ EXCLUSIVE  │ 1023         │ 1051         │
└──────────────────────┴───────────┴────────────┴──────────────┴──────────────┘

2 lock(s) found - 1 blocking chain(s) detected (Query time: 0.089s)

Blocking Chain:
  Session 1023 blocks sessions: 1045, 1051, 1067
```

**With JSON output:**
```sql
tq> /set format json
Output format set to: json

tq> /locks
[
  {
    "Locked Object": "PRODUCTION.orders",
    "Lock Type": "Table",
    "Lock Mode": "WRITE",
    "Locking Sess": 1023,
    "Waiting Sess": [1045, 1067]
  }
]
```

**Tab completion:**
```sql
tq> /lk<TAB>
tq> /locks_
[Auto-completed]

tq> /lo<TAB>

Matching metacommands:
    /locks   Display current lock contention and blocking chains
    /logon   Connect/switch database
```

**Acceptance Test:**
- Execute `/locks` with no active locks and verify "No locks currently held." message
- Execute `/locks` with active locks and verify all five columns are displayed: Locked Object, Lock Type, Lock Mode, Locking Sess, Waiting Sess
- Execute `/lk` (alias) and verify identical behavior
- Verify blocking chains section appears when sessions are waiting
- Verify blocking chains section does NOT appear when no waiters exist
- Execute with `/set format csv` and verify CSV output
- Execute with `/set format json` and verify JSON array output with Waiting Sess as array
- Trigger privilege error and verify helpful error message with GRANT example
- Type `/lk<TAB>` and verify tab completion resolves to `/locks`
- Execute `/help locks` and verify extended help with lock mode definitions is displayed

---

**`/query` Metacommand**

**Requirement:** Display the SQL text of the most recent query executed by a given session, enabling DBAs to drill down from session activity data into the specific SQL causing resource consumption or blocking.

**Syntax:**
```
/query <session_id>
/qi <session_id>        -- Short alias
```

**Arguments:**
- `<session_id>`: Required. An integer session ID as shown in `/sessions` or `/locks` output.

**Output Format:**

**When query text is found:**
```
tq> /query 1023

Query for session 1023:
┌────────────┬──────────────────────────────────────────────────────────────────┐
│ Property   │ Value                                                            │
├────────────┼──────────────────────────────────────────────────────────────────┤
│ Session    │ 1023                                                             │
│ User       │ etl_user                                                         │
│ Query Text │ UPDATE PRODUCTION.orders SET status = 'shipped' WHERE order_... │
└────────────┴──────────────────────────────────────────────────────────────────┘

(Query time: 0.123s)
```

**When no query information is found:**
```
tq> /query 9999

No query information found for session 9999.

The session may have already disconnected, or DBQL logging may not be
enabled for this user. Contact your DBA to enable DBQL logging.
```

**Property Descriptions:**

| Property | Description |
|----------|-------------|
| Session | The session ID that was queried |
| User | The database user account running that session |
| Query Text | The most recent SQL text logged for the session (truncated in table view) |

**Behavior Requirements:**

**REQ-QUERY-001: Query Text Display**

The command SHALL display SQL text for a given session ID:

1. **REQ-QUERY-001.1** - Retrieve query text from `DBC.QryLogV` for the specified session ID
2. **REQ-QUERY-001.2** - Return the most recent (latest `CollectTimeStamp`) query log entry for the session
3. **REQ-QUERY-001.3** - Display output as a two-column key-value table with properties: Session, User, Query Text
4. **REQ-QUERY-001.4** - Header line: `Query for session <N>:` above the table
5. **REQ-QUERY-001.5** - Footer: `(Query time: X.XXXs)` below the table
6. **REQ-QUERY-001.6** - SQL query template:
   ```sql
   SELECT TOP 1
       LogonName,
       QueryText
   FROM DBC.QryLogV
   WHERE LogDate >= DATE - 1
     AND SessionNo = <session_id>
   ORDER BY CollectTimeStamp DESC
   ```

**REQ-QUERY-002: Batch Mode Support**

The command SHALL be available as a batch mode subcommand:

1. **REQ-QUERY-002.1** - Batch mode syntax: `tq query-inspect <session_id> [OPTIONS]`
2. **REQ-QUERY-002.2** - Batch options SHALL include `--format` (table/csv/json) and `--output`
3. **REQ-QUERY-002.3** - Batch mode behavior SHALL be identical to REPL mode for equivalent inputs
4. **REQ-QUERY-002.4** - Exit code 0 on success (including session-not-found), exit code 1 on errors

**REQ-QUERY-003: Output Format Compatibility**

The command SHALL work with all output format modes:

1. **REQ-QUERY-003.1** - Table format (default): Two-column key-value table as shown above, with Query Text truncated to 200 characters with `...`
2. **REQ-QUERY-003.2** - CSV format: Three columns (`Session,User,Query Text`) with full untruncated query text
   ```csv
   Session,User,Query Text
   1023,etl_user,"UPDATE PRODUCTION.orders SET status = 'shipped' WHERE order_date < '2026-01-01'"
   ```
3. **REQ-QUERY-003.3** - JSON format: Single object with full untruncated query text
   ```json
   {
     "Session": 1023,
     "User": "etl_user",
     "Query Text": "UPDATE PRODUCTION.orders SET status = 'shipped' WHERE order_date < '2026-01-01'"
   }
   ```
4. **REQ-QUERY-003.4** - Format selection: Command SHALL respect current output format setting (`/set format <fmt>`)
5. **REQ-QUERY-003.5** - Full query text is always available in CSV and JSON formats regardless of length

**REQ-QUERY-004: Tab Completion and Aliases**

The command SHALL be discoverable and accessible via short forms:

1. **REQ-QUERY-004.1** - Primary command: `/query`
2. **REQ-QUERY-004.2** - Short alias: `/qi`
3. **REQ-QUERY-004.3** - Both forms SHALL execute identically
4. **REQ-QUERY-004.4** - Tab completion: Typing `/qi<TAB>` SHALL auto-complete to `/qi ` (with space, awaiting session ID argument)
5. **REQ-QUERY-004.5** - Tab completion: Typing `/query<TAB>` SHALL auto-complete to `/query ` (with space, awaiting session ID argument)
6. **REQ-QUERY-004.6** - Command SHALL appear in metacommand list when typing `/<TAB>`
7. **REQ-QUERY-004.7** - Help text: `/help` SHALL list `/query` command with description "Show current SQL query for a session"

**REQ-QUERY-005: Error Handling**

The command SHALL handle all error conditions gracefully:

**Missing argument:**
```
tq> /query

Usage: /query <session_id>

Provide the session ID of the session to inspect.
Example: /query 1023

Use /sessions to list active session IDs.
```

**Invalid argument (non-integer):**
```
tq> /query abc

Error: Invalid session ID 'abc'
Session ID must be an integer.
Example: /query 1023
```

**Session not found (no DBQL record):**
```
tq> /query 9999

No query information found for session 9999.

The session may have already disconnected, or DBQL logging may not be
enabled for this user. Contact your DBA to enable DBQL logging.
```

**DBQL not enabled (view not accessible):**
```
tq> /query 1023

No query information found for session 1023.

DBQL (Database Query Log) logging may not be enabled on this system.
Query text is only available when DBQL logging is active.
Contact your DBA to enable DBQL logging.
```

**Insufficient privileges:**
```
tq> /query 1023

Error: Unable to retrieve query information.

This command requires SELECT access to DBC.QryLogV.
Contact your DBA to request access:
  GRANT SELECT ON DBC.QryLogV TO <your_username>;
```

**Connection lost:**
```
tq> /query 1023

Error: Cannot retrieve query information - connection lost.
Use /reconnect to establish a new connection.
```

**Specific Requirements:**

1. **REQ-QUERY-005.1** - Missing argument SHALL display usage hint with example
2. **REQ-QUERY-005.2** - Non-integer argument SHALL display type error with example
3. **REQ-QUERY-005.3** - Session not found (no row in DBC.QryLogV) SHALL display informative message, not an error
4. **REQ-QUERY-005.4** - Privilege errors SHALL include GRANT statement example
5. **REQ-QUERY-005.5** - All errors SHALL return to REPL prompt (non-fatal)
6. **REQ-QUERY-005.6** - Session-not-found condition SHALL exit with code 0 in batch mode (not an error state)

**REQ-QUERY-006: Long SQL Text Handling**

The command SHALL handle SQL text of arbitrary length:

1. **REQ-QUERY-006.1** - Table format: Truncate query text at 200 characters, appending `...` to indicate truncation
2. **REQ-QUERY-006.2** - CSV and JSON formats: Always output full, untruncated query text
3. **REQ-QUERY-006.3** - When query text is truncated in table view, no additional message is required (use of CSV/JSON format is the documented path to full text)
4. **REQ-QUERY-006.4** - Empty query text (empty string from DBQL): Treat as "no query information found" and display the not-found message

**Example Interaction:**

**Drill-down workflow: sessions -> query:**
```sql
tq> /sessions
[Shows session 1023 with high AMPCPUSec and ReqSpool]

tq> /query 1023

Query for session 1023:
┌────────────┬────────────────────────────────────────────────────────────────────┐
│ Property   │ Value                                                              │
├────────────┼────────────────────────────────────────────────────────────────────┤
│ Session    │ 1023                                                               │
│ User       │ etl_user                                                           │
│ Query Text │ UPDATE PRODUCTION.orders SET status = 'shipped' WHERE order_da... │
└────────────┴────────────────────────────────────────────────────────────────────┘

(Query time: 0.123s)
```

**Full text via JSON:**
```sql
tq> /set format json
Output format set to: json

tq> /query 1023
{
  "Session": 1023,
  "User": "etl_user",
  "Query Text": "UPDATE PRODUCTION.orders SET status = 'shipped' WHERE order_date < '2026-01-01' AND status = 'pending'"
}
```

**Drill-down workflow: locks -> query:**
```sql
tq> /locks
[Shows session 1023 blocking sessions 1045 and 1067]

tq> /qi 1023
[Shows the SQL that session 1023 is running that holds the lock]
```

**Tab completion:**
```sql
tq> /qi<TAB>
tq> /qi _
[Awaiting session ID]
```

**Acceptance Test:**
- Execute `/query <valid_session_id>` and verify output shows Session, User, and Query Text properties
- Execute `/qi <session_id>` (alias) and verify identical behavior
- Execute `/query <inactive_session_id>` and verify informative not-found message (not an error)
- Execute without argument and verify usage message with example
- Execute `/query abc` and verify invalid-argument error
- Execute with `/set format csv` and verify CSV output with full (untruncated) query text
- Execute with `/set format json` and verify JSON object output with full query text
- Trigger privilege error by revoking access and verify helpful error with GRANT example
- Type `/q<TAB>` and verify completion suggests `/query` and `/quit`
- Execute `/help query` and verify extended help is displayed

---

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

Data sampling commands provide fast exploratory data analysis without writing full SQL queries. These commands target data analysts and DBAs who need quick table inspection during interactive sessions.

| Command | Description | Example |
|---------|-------------|---------|
| `/sample <table> [n]` | Show random sample (default 10 rows) | `/sample employees 20` |
| `/peek <table> [n]` | Show first N rows and column info (default 5) | `/peek employees 10` |

---

**REQ-SAMPLE-001: Command Availability and Syntax**

The `/sample` and `/peek` commands SHALL be available as metacommands in REPL mode with the following characteristics:

1. **REQ-SAMPLE-001.1** - `/sample` primary syntax: `/sample <table> [n]`
2. **REQ-SAMPLE-001.2** - `/peek` primary syntax: `/peek <table> [n]`
3. **REQ-SAMPLE-001.3** - Both commands SHALL execute immediately (no semicolon required)
4. **REQ-SAMPLE-001.4** - Commands SHALL be case-insensitive (`/Sample`, `/PEEK` are valid)
5. **REQ-SAMPLE-001.5** - Table name parameter is REQUIRED (error if omitted)
6. **REQ-SAMPLE-001.6** - Row count parameter for both commands is OPTIONAL (`/sample` defaults to 10, `/peek` defaults to 5)

**Rationale:** Simple, discoverable syntax consistent with existing metacommands like `/describe` and `/list tables`.

**Example Usage:**
```sql
tq> /sample employees
[Shows 10 random rows from employees table]

tq> /sample employees 50
[Shows 50 random rows from employees table]

tq> /peek products
[Shows first 5 rows + column metadata from products table]

tq> /peek products 10
[Shows first 10 rows + column metadata from products table]
```

---

**REQ-SAMPLE-002: `/sample` Command Behavior**

The `/sample` command SHALL retrieve a random sample of rows from the specified table:

1. **REQ-SAMPLE-002.1** - Sample SHALL be truly random using Teradata SAMPLE clause
2. **REQ-SAMPLE-002.2** - Default sample size: 10 rows
3. **REQ-SAMPLE-002.3** - User-specified sample size: 1 to 1000 rows (inclusive)
4. **REQ-SAMPLE-002.4** - SQL generation: `SELECT * FROM <table> SAMPLE <n>`
5. **REQ-SAMPLE-002.5** - Sample SHALL include all columns from the table
6. **REQ-SAMPLE-002.6** - Column order SHALL match table definition order
7. **REQ-SAMPLE-002.7** - If table has fewer rows than requested, return all available rows
8. **REQ-SAMPLE-002.8** - Each execution MAY return different rows (non-deterministic sampling)

**Rationale:** Teradata SAMPLE clause provides efficient random sampling without full table scans. Non-deterministic behavior enables exploration of different data patterns.

**Example Interaction:**
```sql
tq> /sample customers 20

Random sample from PRODUCTION.customers (20 rows):
┌─────────┬───────────────┬─────────────────────────┬───────────┐
│ cust_id │ name          │ email                   │ region    │
├─────────┼───────────────┼─────────────────────────┼───────────┤
│ 10234   │ Alice Johnson │ alice.j@example.com     │ Northeast │
│ 45671   │ Bob Smith     │ bob.smith@example.com   │ West      │
│ 78234   │ Carol Lee     │ carol.lee@example.com   │ Southeast │
│ ...     │ ...           │ ...                     │ ...       │
└─────────┴───────────────┴─────────────────────────┴───────────┘

20 rows sampled from customers (Query time: 0.045s)
```

---

**REQ-SAMPLE-003: Sample Size Validation**

The `/sample` command SHALL validate the row count parameter:

1. **REQ-SAMPLE-003.1** - Minimum sample size: 1 row
2. **REQ-SAMPLE-003.2** - Maximum sample size: 1000 rows
3. **REQ-SAMPLE-003.3** - Sample size MUST be a positive integer
4. **REQ-SAMPLE-003.4** - Non-numeric values SHALL trigger error
5. **REQ-SAMPLE-003.5** - Zero or negative values SHALL trigger error
6. **REQ-SAMPLE-003.6** - Values exceeding 1000 SHALL trigger error with suggestion

**Rationale:** Prevent accidental large queries that could impact performance. 1000-row limit balances exploration needs with system responsiveness.

**Error Cases:**

**Invalid sample size (non-numeric):**
```sql
tq> /sample employees abc

Error: Invalid sample size 'abc'
Sample size must be a positive integer between 1 and 1000
Example: /sample employees 50
```

**Sample size exceeds maximum:**
```sql
tq> /sample employees 5000

Error: Sample size 5000 exceeds maximum (1000)
For larger samples, use SQL: SELECT * FROM employees SAMPLE 5000;
```

**Zero or negative sample size:**
```sql
tq> /sample employees 0

Error: Sample size must be at least 1
Example: /sample employees 10
```

---

**REQ-SAMPLE-004: `/peek` Command Behavior**

The `/peek` command SHALL provide a quick preview of table contents with column metadata:

1. **REQ-SAMPLE-004.1** - Retrieve first N rows from table (default: 5, configurable via optional N parameter)
2. **REQ-SAMPLE-004.2** - SQL generation: `SELECT TOP N * FROM <table>` where N defaults to 5
3. **REQ-SAMPLE-004.3** - Display column metadata BEFORE data rows
4. **REQ-SAMPLE-004.4** - Column metadata SHALL include: name, data type, nullable, precision/scale (if applicable)
5. **REQ-SAMPLE-004.5** - Display data rows in table format
6. **REQ-SAMPLE-004.6** - If table has fewer than N rows, display all available rows
7. **REQ-SAMPLE-004.7** - If table is empty, display column metadata only with "Table is empty" message
8. **REQ-SAMPLE-004.8** - Optional N parameter: `/peek <table> [N]` allows custom row count
9. **REQ-SAMPLE-004.9** - Parameter validation: N must be positive integer

**Rationale:** Default 5-row preview provides quick table understanding without overwhelming output. Optional N parameter allows flexibility for different table sizes. Column metadata helps users understand data structure before sampling or querying.

**Example Interaction:**
```sql
tq> /peek employees

Table: PRODUCTION.employees
Approximate Rows: 42,573

Column Information:
┌───────────────┬──────────────┬──────────┬───────────┐
│ Column        │ Type         │ Nullable │ Precision │
├───────────────┼──────────────┼──────────┼───────────┤
│ employee_id   │ INTEGER      │ NO       │ -         │
│ first_name    │ VARCHAR(50)  │ YES      │ 50        │
│ last_name     │ VARCHAR(50)  │ YES      │ 50        │
│ hire_date     │ DATE         │ YES      │ -         │
│ salary        │ DECIMAL(10,2)│ YES      │ 10,2      │
└───────────────┴──────────────┴──────────┴───────────┘

First 5 rows:
┌─────────────┬────────────┬───────────┬────────────┬───────────┐
│ employee_id │ first_name │ last_name │ hire_date  │ salary    │
├─────────────┼────────────┼───────────┼────────────┼───────────┤
│ 1           │ Alice      │ Anderson  │ 2020-01-15 │ 75000.00  │
│ 2           │ Bob        │ Brown     │ 2019-03-22 │ 82000.00  │
│ 3           │ Carol      │ Chen      │ 2021-07-01 │ 68000.00  │
│ 4           │ David      │ Davis     │ 2018-11-30 │ 95000.00  │
│ 5           │ Emma       │ Evans     │ 2022-02-14 │ 71000.00  │
└─────────────┴────────────┴───────────┴────────────┴───────────┘

(Query time: 0.023s)
```

**Custom row count:**
```sql
tq> /peek employees 10

Table: PRODUCTION.employees
Approximate Rows: 42,573

Column Information:
┌───────────────┬──────────────┬──────────┬───────────┐
│ Column        │ Type         │ Nullable │ Precision │
├───────────────┼──────────────┼──────────┼───────────┤
│ employee_id   │ INTEGER      │ NO       │ -         │
│ first_name    │ VARCHAR(50)  │ YES      │ 50        │
│ last_name     │ VARCHAR(50)  │ YES      │ 50        │
│ hire_date     │ DATE         │ YES      │ -         │
│ salary        │ DECIMAL(10,2)│ YES      │ 10,2      │
└───────────────┴──────────────┴──────────┴───────────┘

First 10 rows:
┌─────────────┬────────────┬───────────┬────────────┬───────────┐
│ employee_id │ first_name │ last_name │ hire_date  │ salary    │
├─────────────┼────────────┼───────────┼────────────┼───────────┤
│ 1           │ Alice      │ Anderson  │ 2020-01-15 │ 75000.00  │
│ 2           │ Bob        │ Brown     │ 2019-03-22 │ 82000.00  │
│ 3           │ Carol      │ Chen      │ 2021-07-01 │ 68000.00  │
│ 4           │ David      │ Davis     │ 2018-11-30 │ 95000.00  │
│ 5           │ Emma       │ Evans     │ 2022-02-14 │ 71000.00  │
│ 6           │ Frank      │ Foster    │ 2020-08-10 │ 79000.00  │
│ 7           │ Grace      │ Garcia    │ 2019-12-05 │ 88000.00  │
│ 8           │ Henry      │ Harris    │ 2021-03-18 │ 73000.00  │
│ 9           │ Iris       │ Irving    │ 2022-06-21 │ 69000.00  │
│ 10          │ Jack       │ Johnson   │ 2020-04-09 │ 91000.00  │
└─────────────┴────────────┴───────────┴────────────┴───────────┘

(Query time: 0.028s)
```

**Empty table:**
```sql
tq> /peek empty_table

Table: PRODUCTION.empty_table
Approximate Rows: 0

Column Information:
┌───────────┬──────────┬──────────┬───────────┐
│ Column    │ Type     │ Nullable │ Precision │
├───────────┼──────────┼──────────┼───────────┤
│ id        │ INTEGER  │ NO       │ -         │
│ name      │ VARCHAR  │ YES      │ 100       │
└───────────┴──────────┴──────────┴───────────┘

Table is empty (0 rows)
```

---

**REQ-SAMPLE-005: Qualified Table Names Support**

Both `/sample` and `/peek` commands SHALL support qualified table names:

1. **REQ-SAMPLE-005.1** - Unqualified syntax: `/sample <table>` (uses current database)
2. **REQ-SAMPLE-005.2** - Qualified syntax: `/sample <database>.<table>` (explicit database)
3. **REQ-SAMPLE-005.3** - Qualified names SHALL work even when current database differs
4. **REQ-SAMPLE-005.4** - If no current database AND unqualified name used, trigger error
5. **REQ-SAMPLE-005.5** - Database and table names SHALL follow Teradata identifier rules
6. **REQ-SAMPLE-005.6** - Case-insensitive matching for database and table names

**Rationale:** Consistency with `/describe` and `/list tables` commands. Enables cross-database exploration.

**Example Usage:**
```sql
tq> /sample production.orders 15
[Samples from production.orders regardless of current database]

tq> /peek staging.test_data
[Peeks at staging.test_data table]

tq> /sample orders
[Uses current database context]
```

**Error case (no current database):**
```sql
tq> /sample orders

Error: No current database selected
Either specify database: /sample production.orders
Or connect to a database: /logon user:pass@host/database
```

---

**REQ-SAMPLE-006: Error Handling - Table Not Found**

Both commands SHALL handle non-existent tables gracefully:

1. **REQ-SAMPLE-006.1** - If table does not exist, display clear error message
2. **REQ-SAMPLE-006.2** - Error SHALL suggest using `/list tables` to discover tables
3. **REQ-SAMPLE-006.3** - Error SHALL include the full table name attempted (with database if qualified)
4. **REQ-SAMPLE-006.4** - If table name is close to existing table (typo), suggest correction
5. **REQ-SAMPLE-006.5** - Error SHALL return to REPL prompt (non-fatal)

**Rationale:** Help users discover correct table names rather than just reporting failure.

**Example Error:**
```sql
tq> /sample employes 10

Error: Table 'employes' does not exist in database 'production'

Did you mean 'employees'?

Use /list tables to see all available tables
```

**Example (no suggestion):**
```sql
tq> /peek nonexistent_table

Error: Table 'nonexistent_table' not found in database 'production'

Use /list tables to see all available tables
Use /list databases to see all databases
```

---

**REQ-SAMPLE-007: Error Handling - Permission Denied**

Both commands SHALL handle permission errors clearly:

1. **REQ-SAMPLE-007.1** - If user lacks SELECT privilege, display permission error
2. **REQ-SAMPLE-007.2** - Error SHALL explain required privilege (SELECT)
3. **REQ-SAMPLE-007.3** - Error SHALL suggest contacting DBA or provide GRANT syntax example
4. **REQ-SAMPLE-007.4** - Error SHALL include table name that caused permission failure

**Rationale:** Security errors are common in enterprise databases. Clear guidance helps users resolve access issues.

**Example Error:**
```sql
tq> /sample restricted_table 10

Error: Permission denied on table 'restricted_table'

You do not have SELECT privilege on PRODUCTION.restricted_table

Contact your DBA to request access, or use:
  GRANT SELECT ON production.restricted_table TO <your_username>;
```

---

**REQ-SAMPLE-008: Output Format Compatibility**

Both commands SHALL respect current output format settings:

1. **REQ-SAMPLE-008.1** - Table format (default): Box-drawing table with borders
2. **REQ-SAMPLE-008.2** - CSV format: Standard CSV output (header row + data rows)
3. **REQ-SAMPLE-008.3** - JSON format: Array of objects (one object per row)
4. **REQ-SAMPLE-008.4** - Format SHALL be controlled by `/set format <fmt>` metacommand
5. **REQ-SAMPLE-008.5** - Column metadata (for `/peek`) SHALL adapt to output format
6. **REQ-SAMPLE-008.6** - Summary footer SHALL adapt to output format (omitted in CSV/JSON)

**Rationale:** Consistency with query result formatting. Enables scripting and data export workflows.

**CSV Example:**
```sql
tq> /set format csv
Output format set to: csv

tq> /sample employees 3
employee_id,first_name,last_name,hire_date,salary
1,Alice,Anderson,2020-01-15,75000.00
2,Bob,Brown,2019-03-22,82000.00
3,Carol,Chen,2021-07-01,68000.00
```

**JSON Example:**
```sql
tq> /set format json
Output format set to: json

tq> /sample employees 2
[
  {
    "employee_id": 1,
    "first_name": "Alice",
    "last_name": "Anderson",
    "hire_date": "2020-01-15",
    "salary": 75000.00
  },
  {
    "employee_id": 2,
    "first_name": "Bob",
    "last_name": "Brown",
    "hire_date": "2019-03-22",
    "salary": 82000.00
  }
]
```

---

**REQ-SAMPLE-009: Tab Completion Integration**

Both commands SHALL be integrated into tab completion system:

1. **REQ-SAMPLE-009.1** - Typing `/s<TAB>` SHALL suggest `/sample` and `/sessions`
2. **REQ-SAMPLE-009.2** - Typing `/sa<TAB>` SHALL auto-complete to `/sample`
3. **REQ-SAMPLE-009.3** - Typing `/p<TAB>` SHALL suggest `/peek` and `/pager`
4. **REQ-SAMPLE-009.4** - Typing `/pe<TAB>` SHALL auto-complete to `/peek`
5. **REQ-SAMPLE-009.5** - After `/sample ` (with space), SHALL suggest table names from current database
6. **REQ-SAMPLE-009.6** - After `/peek ` (with space), SHALL suggest table names from current database
7. **REQ-SAMPLE-009.7** - Table name completion SHALL support qualified names (`database.<TAB>`)
8. **REQ-SAMPLE-009.8** - Commands SHALL appear in metacommand list when typing `/<TAB>`

**Rationale:** Tab completion is critical for discoverability and efficient command entry.

**Example Interaction:**
```sql
tq> /sa<TAB>
tq> /sample _

tq> /sample <TAB>
Available tables in 'production':
  customers    employees    orders    products

tq> /sample emp<TAB>
tq> /sample employees _

tq> /peek staging.<TAB>
Available tables in 'staging':
  test_customers    test_orders    test_products
```

---

**REQ-SAMPLE-010: Help Text Integration**

Both commands SHALL be documented in help system:

1. **REQ-SAMPLE-010.1** - `/help` command SHALL list both `/sample` and `/peek`
2. **REQ-SAMPLE-010.2** - `/help sample` SHALL display detailed help for `/sample` command
3. **REQ-SAMPLE-010.3** - `/help peek` SHALL display detailed help for `/peek` command
4. **REQ-SAMPLE-010.4** - Help text SHALL include: description, syntax, examples, related commands
5. **REQ-SAMPLE-010.5** - Help SHALL cross-reference related commands (`/describe`, `/list tables`)

**Example Help Output:**
```sql
tq> /help sample

/sample - Show random sample of table data

SYNTAX:
  /sample <table> [n]

DESCRIPTION:
  Retrieve a random sample of rows from the specified table using
  Teradata's SAMPLE clause for efficient data exploration.

  Default sample size: 10 rows
  Maximum sample size: 1000 rows

EXAMPLES:
  /sample employees          Show 10 random rows from employees
  /sample orders 50          Show 50 random rows from orders
  /sample staging.test 5     Sample from different database

RELATED COMMANDS:
  /peek <table>              Show first 5 rows with column info
  /describe <table>          Show table structure
  /list tables               List all tables in database

For more information, see documentation at: docs/user/metacommands.md
```

```sql
tq> /help peek

/peek - Quick preview of table with column metadata

SYNTAX:
  /peek <table>

DESCRIPTION:
  Show the first 5 rows of a table along with column metadata
  (names, types, nullable status). Useful for quick table inspection
  before writing queries or sampling data.

EXAMPLES:
  /peek employees            Preview employees table
  /peek staging.test_data    Preview table in different database

RELATED COMMANDS:
  /sample <table> [n]        Show random sample of rows
  /describe <table>          Show detailed table structure
  /list tables               List all tables in database

For more information, see documentation at: docs/user/metacommands.md
```

---

**REQ-SAMPLE-011: Batch Mode Integration**

Both commands SHALL be available in batch mode (one-shot execution):

1. **REQ-SAMPLE-011.1** - Batch syntax: `tq sample <table> [n]` (without leading `/`)
2. **REQ-SAMPLE-011.2** - Batch syntax: `tq peek <table>` (without leading `/`)
3. **REQ-SAMPLE-011.3** - Connection SHALL be established, command executed, connection closed
4. **REQ-SAMPLE-011.4** - Output format SHALL default to table (unless `--format` flag specified)
5. **REQ-SAMPLE-011.5** - Exit code 0 on success, non-zero on error
6. **REQ-SAMPLE-011.6** - Batch mode SHALL support `--format csv|json|table` flag
7. **REQ-SAMPLE-011.7** - Connection credentials via `TQ_LOGON` env var or `--logon` flag

**Rationale:** Enable scripting and automation workflows. Users should be able to sample data in shell scripts.

**Batch Mode Examples:**
```bash
# Sample 20 rows using environment credentials
$ export TQ_LOGON="user:pass@host:1025/production"
$ tq sample employees 20

# Peek at table with explicit connection
$ tq peek customers --logon "user:pass@host/production"

# Sample to CSV for processing
$ tq sample orders 100 --format csv > orders_sample.csv

# Sample and pipe to jq
$ tq sample products 50 --format json | jq '.[] | select(.price > 100)'
```

---

**REQ-SAMPLE-012: Performance Requirements**

Both commands SHALL execute efficiently:

1. **REQ-SAMPLE-012.1** - `/sample` target execution time: <1 second for sample sizes up to 1000 rows
2. **REQ-SAMPLE-012.2** - `/peek` target execution time: <500ms (fetching only 5 rows)
3. **REQ-SAMPLE-012.3** - Commands SHALL NOT perform full table scans
4. **REQ-SAMPLE-012.4** - `/sample` SHALL use Teradata SAMPLE clause (efficient row sampling)
5. **REQ-SAMPLE-012.5** - `/peek` SHALL use TOP clause (efficient row limiting)
6. **REQ-SAMPLE-012.6** - Loading indicator: Display "Sampling data..." if query exceeds 500ms
7. **REQ-SAMPLE-012.7** - Query cancellation: Ctrl-C SHALL cancel query and return to prompt
8. **REQ-SAMPLE-012.8** - Commands SHALL work efficiently on tables with billions of rows

**Rationale:** Fast execution is critical for interactive exploration. Teradata SAMPLE clause provides efficient random sampling without scanning entire tables.

**Example (with loading indicator):**
```sql
tq> /sample huge_table 1000
Sampling data from huge_table...
[Query completes after 800ms]

Random sample from PRODUCTION.huge_table (1000 rows):
[Table output...]

1000 rows sampled (Query time: 0.812s)
```

---

**REQ-SAMPLE-013: Connection State Handling**

Both commands SHALL handle connection state appropriately:

1. **REQ-SAMPLE-013.1** - If no active connection, display connection error
2. **REQ-SAMPLE-013.2** - Error SHALL suggest using `/logon` to establish connection
3. **REQ-SAMPLE-013.3** - If connection lost during execution, display reconnection error
4. **REQ-SAMPLE-013.4** - Connection errors SHALL return to REPL prompt (non-fatal)

**Example Errors:**

**No active connection:**
```sql
tq> /sample employees

Error: No active database connection

Connect to a database first:
  /logon user:pass@host:1025/database

Or use environment variable:
  export TQ_LOGON="user:pass@host:1025/database"
```

**Connection lost during query:**
```sql
tq> /sample huge_table 1000
Sampling data from huge_table...

Error: Connection lost during query execution

Use /reconnect to establish a new connection
```

---

**REQ-SAMPLE-014: Result Display Headers**

Both commands SHALL provide clear result headers:

1. **REQ-SAMPLE-014.1** - `/sample` header format: `Random sample from <DATABASE>.<TABLE> (<N> rows):`
2. **REQ-SAMPLE-014.2** - `/peek` header format: `Table: <DATABASE>.<TABLE>`
3. **REQ-SAMPLE-014.3** - `/peek` SHALL display approximate row count from system catalog
4. **REQ-SAMPLE-014.4** - Footer SHALL show query execution time
5. **REQ-SAMPLE-014.5** - Footer SHALL indicate number of rows returned
6. **REQ-SAMPLE-014.6** - Headers SHALL be omitted in CSV and JSON output formats

**Rationale:** Clear headers help users understand what data they're viewing, especially when running multiple commands.

**Sample Header Example:**
```sql
tq> /sample employees 25

Random sample from PRODUCTION.employees (25 rows):
┌─────────────┬────────────┬───────────┐
│ employee_id │ first_name │ last_name │
[...]
└─────────────┴────────────┴───────────┘

25 rows sampled from employees (Query time: 0.067s)
```

**Peek Header Example:**
```sql
tq> /peek customers

Table: PRODUCTION.customers
Approximate Rows: 1,234,567

Column Information:
[...]

First 5 rows:
[...]

(Query time: 0.034s)
```

---

**REQ-SAMPLE-015: Views and Table Types Support**

Both commands SHALL support different table types:

1. **REQ-SAMPLE-015.1** - Commands SHALL work on regular tables
2. **REQ-SAMPLE-015.2** - Commands SHALL work on views
3. **REQ-SAMPLE-015.3** - Commands SHALL work on volatile tables
4. **REQ-SAMPLE-015.4** - Commands SHALL work on global temporary tables
5. **REQ-SAMPLE-015.5** - Error handling SHALL differentiate between table types if query fails
6. **REQ-SAMPLE-015.6** - For views with complex queries, performance may vary (document in help)

**Rationale:** Teradata supports multiple table types. Commands should work consistently across all types.

**Example (sampling a view):**
```sql
tq> /sample active_employees_view 15

Random sample from PRODUCTION.active_employees_view (15 rows):
┌─────────────┬────────────┬──────────┬────────────┐
│ employee_id │ name       │ dept     │ hire_date  │
[...]
└─────────────┴────────────┴──────────┴────────────┘

15 rows sampled from active_employees_view (Query time: 0.123s)
Note: Sampling a view may take longer than sampling a table
```

**Example (peeking at volatile table):**
```sql
tq> /peek session_temp_results

Table: PRODUCTION.session_temp_results (Volatile)
Approximate Rows: 42

Column Information:
[...]

First 5 rows:
[...]

(Query time: 0.012s)
```

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

**Status:** EXPERIMENTAL - The interactive pager is currently experimental and disabled by default due to ongoing rendering issues with wide result sets. Users may enable it with `/pager on` if they wish to test the feature.

**Syntax**:
```
/pager on       (enable result paging)
/pager off      (disable paging, show all results - DEFAULT)
```

**Pager enabled (opt-in)**:
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
| `/repeat` | `\r` | Re-execute last query | `/repeat` |
| `/quit` | `\q` | Exit REPL | `/quit` |

---

**`/edit` Metacommand**

**Requirement:** Open the last executed SQL query in an external text editor for modification, then execute the edited query upon save and exit.

**Syntax:**
```
/edit
\e                  -- Short alias
```

**Behavior Requirements:**

**REQ-EDIT-001: Command Availability and Aliases**

The `/edit` command SHALL be available as a metacommand in full REPL mode with the following characteristics:

1. **REQ-EDIT-001.1** - Primary command: `/edit`
2. **REQ-EDIT-001.2** - Short alias: `\e`
3. **REQ-EDIT-001.3** - Both forms SHALL execute identically
4. **REQ-EDIT-001.4** - Command SHALL execute immediately (no arguments required)
5. **REQ-EDIT-001.5** - Command SHALL be case-insensitive (`/Edit`, `/EDIT`, `\e`, `\E` all valid)
6. **REQ-EDIT-001.6** - Command SHALL be available in full REPL mode only (not in quick REPL mode)

**Rationale:** Follows established metacommand patterns. Quick REPL mode exclusion matches `/repeat` behavior since both commands operate on stored query state.

---

**REQ-EDIT-002: Editor Resolution**

The command SHALL resolve the external editor using the following priority order:

1. **REQ-EDIT-002.1** - First priority: `$VISUAL` environment variable
2. **REQ-EDIT-002.2** - Second priority: `$EDITOR` environment variable
3. **REQ-EDIT-002.3** - Third priority: `vi` as fallback
4. **REQ-EDIT-002.4** - If no editor found (fallback `vi` not available), display clear error message
5. **REQ-EDIT-002.5** - Environment variable values SHALL be trimmed of whitespace
6. **REQ-EDIT-002.6** - Empty environment variables SHALL be ignored (treated as unset)
7. **REQ-EDIT-002.7** - Editor command MAY include arguments (e.g., `$EDITOR="code --wait"`)

**Rationale:** Standard UNIX convention prioritizes `$VISUAL` over `$EDITOR`. Fallback to `vi` ensures availability on most UNIX-like systems.

**Example Editor Resolution:**
```bash
# User has VISUAL set
export VISUAL="emacs"
tq> /edit
[Opens emacs]

# User has only EDITOR set
export EDITOR="nano"
tq> /edit
[Opens nano]

# No environment variables set
tq> /edit
[Opens vi]
```

---

**REQ-EDIT-003: Temporary File Handling**

The command SHALL create a temporary file containing the last query:

1. **REQ-EDIT-003.1** - Temp file SHALL use `.sql` extension for syntax highlighting
2. **REQ-EDIT-003.2** - Temp file SHALL be created in system temp directory
3. **REQ-EDIT-003.3** - Temp file name format: `tq_edit_<random>.sql` where `<random>` is cryptographically secure random string
4. **REQ-EDIT-003.4** - Temp file SHALL be populated with last SQL query before editor opens
5. **REQ-EDIT-003.5** - Temp file SHALL preserve original query formatting (whitespace, line breaks, indentation)
6. **REQ-EDIT-003.6** - Temp file SHALL be automatically deleted after editor closes
7. **REQ-EDIT-003.7** - Temp file deletion SHALL occur even if editor exits with error
8. **REQ-EDIT-003.8** - File permissions SHALL be user-only (0600) for security

**Rationale:** `.sql` extension enables syntax highlighting in most editors. Secure random naming prevents conflicts and information disclosure. Automatic cleanup prevents temp file accumulation.

---

**REQ-EDIT-004: Editor Execution and Process Management**

The command SHALL launch the editor as a child process:

1. **REQ-EDIT-004.1** - Editor process SHALL inherit current terminal
2. **REQ-EDIT-004.2** - REPL SHALL block and wait for editor to exit
3. **REQ-EDIT-004.3** - Editor exit status SHALL be captured
4. **REQ-EDIT-004.4** - Non-zero exit status SHALL trigger error (query not executed)
5. **REQ-EDIT-004.5** - Zero exit status SHALL proceed to query execution
6. **REQ-EDIT-004.6** - Editor process SHALL receive proper signal handling (SIGINT, SIGTERM)
7. **REQ-EDIT-004.7** - If editor cannot be spawned, display clear error with editor path

**Rationale:** Blocking ensures user completes editing before REPL resumes. Exit status validation prevents execution of potentially corrupted edits.

**Error Case - Editor Launch Failure:**
```sql
tq> /edit

Error: Unable to launch editor 'nano'
Reason: Command not found

Suggestions:
  - Install nano: apt install nano (Debian/Ubuntu) or brew install nano (macOS)
  - Set different editor: export EDITOR=vi
  - Check PATH includes editor location
```

---

**REQ-EDIT-005: Query Execution After Edit**

The command SHALL execute the edited query based on file contents and editor exit status:

1. **REQ-EDIT-005.1** - On successful editor exit (code 0), read edited file contents
2. **REQ-EDIT-005.2** - If file is empty, display message and do not execute (return to REPL prompt)
3. **REQ-EDIT-005.3** - If file contains only whitespace, treat as empty (no execution)
4. **REQ-EDIT-005.4** - If file unchanged from original query, display message and do not execute
5. **REQ-EDIT-005.5** - If file changed and non-empty, execute edited SQL query
6. **REQ-EDIT-005.6** - Edited query SHALL be stored as `last_sql` (enabling `/repeat` afterward)
7. **REQ-EDIT-005.7** - Query execution SHALL use normal REPL execution path (same error handling, output formatting)
8. **REQ-EDIT-005.8** - Query execution errors SHALL be displayed normally (do not suppress)

**Rationale:** Empty file or no changes indicates user intent to cancel. Storing edited query as `last_sql` maintains consistency with normal query execution.

**Example Interaction - Successful Edit:**
```sql
tq> SELECT * FROM employees WHERE dept = 'IT';
[Shows results: 42 rows]

tq> /edit
[Opens editor with query]
[User changes query to: SELECT * FROM employees WHERE dept = 'Sales';]
[User saves and exits]

Executing edited query...

[Shows results: 28 rows from Sales department]

tq> /repeat
[Re-executes the edited query about Sales department]
```

**Example Interaction - Empty File (Cancel):**
```sql
tq> SELECT * FROM employees;

tq> /edit
[Opens editor with query]
[User deletes all content, saves and exits]

Edit cancelled (empty query)

tq> _
```

**Example Interaction - No Changes:**
```sql
tq> SELECT COUNT(*) FROM orders;

tq> /edit
[Opens editor with query]
[User exits without making changes]

No changes made

tq> _
```

---

**REQ-EDIT-006: Error Handling - No Previous Query**

The command SHALL handle the case where no previous query exists:

1. **REQ-EDIT-006.1** - If no previous SQL query executed in session, display error message
2. **REQ-EDIT-006.2** - Error message SHALL be clear and actionable
3. **REQ-EDIT-006.3** - Error SHALL return to REPL prompt (non-fatal)
4. **REQ-EDIT-006.4** - Metacommands (e.g., `/describe`, `/list`) SHALL NOT be considered as "last query"
5. **REQ-EDIT-006.5** - Only user-entered SQL queries SHALL be editable

**Rationale:** Users may invoke `/edit` immediately after starting REPL. Clear guidance prevents confusion.

**Error Case:**
```sql
tq> /edit

Error: No previous query to edit

You haven't executed any SQL queries yet in this session.
Run a query first, then use /edit to modify and re-execute it.

Example:
  tq> SELECT * FROM employees;
  tq> /edit
```

---

**REQ-EDIT-007: Error Handling - Editor Exit with Error**

The command SHALL handle abnormal editor termination:

1. **REQ-EDIT-007.1** - If editor exits with non-zero status, display error message
2. **REQ-EDIT-007.2** - Error message SHALL include exit code
3. **REQ-EDIT-007.3** - Query SHALL NOT be executed after editor error
4. **REQ-EDIT-007.4** - Original query SHALL remain as `last_sql` (unchanged)
5. **REQ-EDIT-007.5** - Temp file SHALL still be cleaned up

**Rationale:** Non-zero exit status may indicate editor crash or user cancellation (Ctrl-C in vim). Preventing execution avoids potential data corruption.

**Error Case:**
```sql
tq> /edit
[Editor opens, user presses Ctrl-C in vim without saving]

Error: Editor exited with error (exit code: 1)
Query not executed

Your original query is unchanged. Use /repeat to execute it, or /edit to try again.
```

---

**REQ-EDIT-008: Integration with REPL State**

The command SHALL integrate properly with REPL query history:

1. **REQ-EDIT-008.1** - Edited query (if executed) SHALL be added to command history
2. **REQ-EDIT-008.2** - Edited query SHALL be retrievable with Up arrow key
3. **REQ-EDIT-008.3** - Edited query SHALL replace `last_sql` state (used by `/repeat`)
4. **REQ-EDIT-008.4** - Multi-line edited queries SHALL be stored as single history entry
5. **REQ-EDIT-008.5** - Original query SHALL remain in history (not deleted)

**Rationale:** Natural workflow allows users to further refine queries using history navigation or `/repeat` command.

---

**REQ-EDIT-009: Tab Completion**

The command SHALL be discoverable through tab completion:

1. **REQ-EDIT-009.1** - `/edit` SHALL appear in metacommand completion menu
2. **REQ-EDIT-009.2** - `\e` SHALL appear in metacommand completion menu
3. **REQ-EDIT-009.3** - Typing `/e<TAB>` SHALL show `/edit` as completion option
4. **REQ-EDIT-009.4** - Completion description: "Edit last query in $EDITOR"

---

**REQ-EDIT-010: Help Text**

The command SHALL be documented in REPL help:

1. **REQ-EDIT-010.1** - `/help` SHALL list `/edit` command
2. **REQ-EDIT-010.2** - `/help edit` SHALL show detailed command help
3. **REQ-EDIT-010.3** - Help SHALL explain editor resolution order ($VISUAL → $EDITOR → vi)
4. **REQ-EDIT-010.4** - Help SHALL mention `\e` short alias

**Example Help Output:**
```sql
tq> /help edit

/edit - Edit last query in external editor

Opens your last SQL query in an external text editor. After you save and exit,
the edited query is automatically executed.

Editor Resolution:
  1. $VISUAL environment variable
  2. $EDITOR environment variable
  3. vi (fallback)

Usage:
  /edit           Open last query in editor
  \e              Short alias

Workflow:
  1. Run a query: SELECT * FROM employees;
  2. Edit it: /edit
  3. Modify query in your editor, save and exit
  4. Modified query executes automatically

Notes:
  - Exiting editor without changes cancels execution
  - Edited query becomes new "last query" for /repeat
  - Only works with SQL queries (not metacommands)

See also: /repeat
```

---

**Example Complete Workflow:**

```sql
# Initial query
tq> SELECT employee_id, first_name, last_name
    FROM employees
    WHERE department = 'IT';

┌─────────────┬────────────┬───────────┐
│ employee_id │ first_name │ last_name │
├─────────────┼────────────┼───────────┤
│ 101         │ Alice      │ Anderson  │
│ 102         │ Bob        │ Brown     │
│ 103         │ Carol      │ Chen      │
└─────────────┴────────────┴───────────┘

3 rows in set (0.045s)

# Open in editor
tq> /edit
[Editor opens with query]

# User changes 'IT' to 'Sales' and adds ORDER BY
# Query now reads:
#   SELECT employee_id, first_name, last_name
#   FROM employees
#   WHERE department = 'Sales'
#   ORDER BY last_name;

[User saves and exits editor]

Executing edited query...

┌─────────────┬────────────┬───────────┐
│ employee_id │ first_name │ last_name │
├─────────────┼────────────┼───────────┤
│ 205         │ David      │ Adams     │
│ 211         │ Emma       │ Clark     │
│ 198         │ Frank      │ Foster    │
└─────────────┴────────────┴───────────┘

3 rows in set (0.038s)

# Can re-execute edited query
tq> /repeat
[Re-executes the Sales query with ORDER BY]
```

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
