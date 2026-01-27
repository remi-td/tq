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
- `/logon` - Connect/switch database
- `/disconnect` - Disconnect current connection
- `/reconnect` - Reconnect to database
- `/ping` - Test connection
- `/sample` - Show random sample
- `/peek` - Show first rows and column info
- `/export` - Export results
- `/session` - Show session info
- `/sessions` - List active Teradata sessions
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
    /logon       Connect/switch database
    /pager       Enable/disable result paging
    /peek        Show first rows and column info
    /ping        Test connection
    /quit        Exit REPL
    /reconnect   Reconnect to database
    /repeat      Re-execute last query
    /sample      Show random sample
    /session     Show session info
    /sessions    List active Teradata sessions with performance metrics
    /set         Set configuration
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

### Session Monitoring Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `/sessions` | `/s` | List active Teradata sessions with performance metrics | `/sessions` |

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
