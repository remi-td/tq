# REPL Mode User Guide

This guide shows you how to use tq's interactive REPL (Read-Eval-Print Loop) mode for exploring and querying your Teradata database.

## Starting REPL Mode

```bash
# Start REPL with connection from environment
export TQ_LOGON="user:pass@host:1025/database"
tq repl

# Or specify connection directly
tq -l "user:pass@host:1025/database" repl
```

Once connected, you'll see the REPL prompt:

```
tq> _
```

## Interactive Features

### Tab Completion

REPL mode provides intelligent tab completion to help you work faster and discover available commands.

#### Discovering Metacommands

Type `/` and press TAB to see all available metacommands:

```sql
tq> /<TAB>

Available metacommands:
    /clear       Clear screen
    /colors      Enable/disable syntax highlighting
    /describe    Describe table structure
    /disconnect  Disconnect current connection
    /help        Show help
    /list        Schema inspection (databases, tables, views)
    /logon       Connect/switch database
    /pager       Enable/disable result paging
    /peek        Show first rows and column info
    /ping        Test connection
    /quit        Exit REPL
    /reconnect   Reconnect to database
    /sample      Show random sample
    /session     Show session info
    /sessions    List all database sessions
    /timing      Enable/disable query timing
```

#### Completing Metacommands

Start typing a metacommand and press TAB to complete it:

```sql
# Type partial command
tq> /des<TAB>

# Autocompletes to
tq> /describe _
```

If multiple commands match, you'll see a filtered list:

```sql
tq> /l<TAB>

Matching metacommands:
    /list        Schema inspection (databases, tables, views)
    /logon       Connect/switch database
```

Use arrow keys to select, then press ENTER or TAB to accept.

#### SQL Completion

Tab completion also works for SQL keywords, table names, and column names:

```sql
# Complete keywords
tq> SEL<TAB>
tq> SELECT _

# Complete table names after FROM
tq> SELECT * FROM emp<TAB>
employees    emp_archive

# Complete column names
tq> SELECT * FROM employees WHERE <TAB>
employee_id    first_name    last_name    email    hire_date
```

### Schema Exploration Commands

REPL mode includes quick commands to explore your database schema without writing SQL.

#### List All Databases

See all databases you have access to:

```sql
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

**Short alias:** `\l`

```sql
tq> \l
```

#### List Tables

List all tables in your current database:

```sql
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

**Short alias:** `\dt`

```sql
tq> \dt
```

#### Filter Tables by Pattern

Use SQL LIKE patterns to filter tables:

```sql
# Find tables starting with "emp"
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

**Pattern syntax:**
- `%` matches any characters (like `*` in shell)
- `_` matches a single character (like `?` in shell)

**Examples:**
```sql
/list tables test_%       # Tables starting with "test_"
/list tables %_temp       # Tables ending with "_temp"
/list tables sales_2024_% # Tables starting with "sales_2024_"
```

You can also specify a database:

```sql
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

#### List Views

List all views in your current database:

```sql
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

**Short alias:** `\dv`

```sql
tq> \dv
```

### Loading Indicators

When you use tab completion or schema commands, tq may need to fetch metadata from the database. If this takes more than half a second, you'll see a loading indicator:

```sql
tq> SELECT * FROM remote_database.<TAB>
Loading tables from remote_database... ⠋
```

The spinner animates while fetching data. Once loaded, results are cached for the rest of your session, so subsequent completions are instant.

You can press Ctrl-C to cancel a slow metadata fetch if needed:

```sql
tq> SELECT * FROM slow_database.<TAB>
Loading tables from slow_database... ⠹
^C
Metadata fetch cancelled

tq> _
```

## Basic Query Execution

### Running Queries

Type your SQL and press ENTER to execute:

```sql
tq> SELECT * FROM employees WHERE department = 'IT';

┌─────────────┬────────────┬───────────┬─────────────────────┐
│ employee_id │ first_name │ last_name │ email               │
├─────────────┼────────────┼───────────┼─────────────────────┤
│ 101         │ Alice      │ Anderson  │ alice@company.com   │
│ 102         │ Bob        │ Brown     │ bob@company.com     │
│ 103         │ Carol      │ Chen      │ carol@company.com   │
└─────────────┴────────────┴───────────┴─────────────────────┘

3 rows in set (0.045s)
```

### Smart Column Widths

tq automatically calculates column widths based on actual content, not database schema types. This dramatically improves information density for wide tables.

**How it works:**

Instead of using the schema-defined type width (e.g., VARCHAR(64) = 64 characters), tq measures the actual content in each column and sizes columns accordingly. This means columns with short values don't waste space, allowing more columns to fit on screen.

**Example: Querying system tables**

```sql
tq> SELECT * FROM DBC.Databases;
```

**Before (schema-based widths):**
Only 2 columns visible because VARCHAR(64) fields take 64+ characters each:

```
┌──────────────────────────────────────────────────────────────────┬──────────────────────────────────────────────────────────────────┬──────────────┐
│ DatabaseName                                                     │ CreatorName                                                      │ (+14 cols)   │
├──────────────────────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────┼──────────────┤
│ SystemDB                                                         │ DBC                                                              │ ...          │
│ TempDB                                                           │ DBC                                                              │ ...          │
│ UserDB                                                           │ DBC                                                              │ ...          │
└──────────────────────────────────────────────────────────────────┴──────────────────────────────────────────────────────────────────┴──────────────┘

14 columns hidden: OwnerName, AccountName, ProtectionType, JournalFlag, PermSpace, SpoolSpace, TempSpace, ...
```

**After (content-based widths):**
9 columns visible because columns are sized to actual content (~10-15 characters):

```
┌──────────────┬─────────────┬───────────┬─────────────┬────────────────┬─────────────┬───────────┬───────────┬───────────┬─────────────┐
│ DatabaseName │ CreatorName │ OwnerName │ AccountName │ ProtectionType │ JournalFlag │ PermSpace │ SpoolSpace│ TempSpace │ (+7 cols)   │
├──────────────┼─────────────┼───────────┼─────────────┼────────────────┼─────────────┼───────────┼───────────┼───────────┼─────────────┤
│ SystemDB     │ DBC         │ DBC       │ $SYSTEM     │ None           │ None        │  1048576  │    524288 │    262144 │ ...         │
│ TempDB       │ DBC         │ DBC       │ $SYSTEM     │ None           │ None        │        0  │         0 │  1048576  │ ...         │
│ UserDB       │ DBC         │ UserAdmin │ $USER       │ Read           │ Dual        │  5242880  │  1048576  │    524288 │ ...         │
└──────────────┴─────────────┴───────────┴─────────────┴────────────────┴─────────────┴───────────┴───────────┴───────────┴─────────────┘

7 columns hidden: CreateTimeStamp, LastAlterName, LastAlterTimeStamp, ...
```

**Result:** 4.5x more columns visible (9 vs 2), significantly improving data exploration efficiency.

**Technical details:**
- Column width = maximum of content length and header length across all rows
- Maximum width capped at 100 characters per column (very long values are truncated with "...")
- Works for all data types: strings, numbers, dates, NULL values
- Preserves alignment: numbers right-aligned, text left-aligned

### Multi-line Queries

Queries can span multiple lines. Press ENTER to continue on a new line:

```sql
tq> SELECT
    employee_id,
    first_name,
    last_name
  FROM employees
  WHERE department = 'IT';
```

The query executes when you end it with a semicolon (`;`).

### Query History

Use arrow keys to navigate your command history:

- **↑** (Up Arrow): Previous command
- **↓** (Down Arrow): Next command
- **Ctrl-R**: Search history (reverse search)

Your history persists between sessions in `~/.tq_history`.

#### Multi-line Command History

When you write multi-line SQL statements, tq remembers them as complete commands, not line-by-line. This makes it easy to recall and edit complex queries.

**How it works:**

Each complete SQL statement (ending with `;`) is stored as a single history entry, regardless of how many lines it spans.

**Example interaction:**

```sql
# Enter a multi-line query
tq> SELECT
    employee_id,
    first_name,
    last_name
  FROM employees
  WHERE department = 'IT';

[Query executes, shows results]

# Press ↑ to recall - the ENTIRE query comes back
tq> SELECT
    employee_id,
    first_name,
    last_name
  FROM employees
  WHERE department = 'IT';_
```

**Benefits:**

1. **Easy editing** - Recall the full query to make changes
2. **Natural workflow** - Works like you'd expect from a modern SQL client
3. **Preserved formatting** - Your line breaks and indentation are maintained
4. **Navigation within query** - Use ↑/↓ arrows to move between lines in the recalled query

**History navigation example:**

```sql
# Type three queries (two multi-line, one single-line)

tq> SELECT * FROM employees
    WHERE department = 'IT';
[Executes]

tq> SELECT COUNT(*) FROM orders;
[Executes]

tq> UPDATE employees
    SET status = 'active'
    WHERE hire_date > '2024-01-01';
[Executes]

# Now at empty prompt - press ↑ once
tq> UPDATE employees
    SET status = 'active'
    WHERE hire_date > '2024-01-01';

# Press ↑ again
tq> SELECT COUNT(*) FROM orders;

# Press ↑ again
tq> SELECT * FROM employees
    WHERE department = 'IT';
```

Each press of ↑ recalls one complete statement, whether it was typed on one line or many.

## Other Useful Commands

### Data Sampling Commands

Quick commands to sample data from tables without writing full SQL queries.

#### Sample Random Rows

Get a random sample from any table:

```sql
tq> /sample employees

Random sample from PRODUCTION.employees (10 rows):
┌─────────────┬────────────┬───────────┬─────────────────────┐
│ employee_id │ first_name │ last_name │ email               │
├─────────────┼────────────┼───────────┼─────────────────────┤
│ 157         │ Diana      │ Davis     │ diana@company.com   │
│ 023         │ Frank      │ Foster    │ frank@company.com   │
│ 091         │ Helen      │ Harris    │ helen@company.com   │
│ 234         │ Ivan       │ Ivanov    │ ivan@company.com    │
│ 012         │ Julia      │ Jackson   │ julia@company.com   │
│ 187         │ Kevin      │ King      │ kevin@company.com   │
│ 145         │ Laura      │ Lee       │ laura@company.com   │
│ 098         │ Mike       │ Miller    │ mike@company.com    │
│ 203         │ Nancy      │ Nelson    │ nancy@company.com   │
│ 176         │ Oscar      │ Olson     │ oscar@company.com   │
└─────────────┴────────────┴───────────┴─────────────────────┘

10 rows sampled from employees (Query time: 0.045s)
```

**Specify sample size:**

```sql
tq> /sample customers 50

Random sample from PRODUCTION.customers (50 rows):
[Shows 50 random rows...]

50 rows sampled from customers (Query time: 0.092s)
```

**How it works:**
- Default: 10 rows if count not specified
- Maximum: 1000 rows (prevents accidental huge queries)
- Uses Teradata SAMPLE clause for true random sampling
- Fast even on huge tables

**Common uses:**
- Quick data inspection during exploration
- Checking data quality without full table scans
- Validating ETL results
- Finding example values for testing

#### Peek at Table Structure and Data

Get a quick preview of table structure with sample data:

```sql
tq> /peek products

Table: PRODUCTION.products
Type: Table
Approximate Rows: 15,432

Columns:
┌─────────────┬──────────────┬──────────┬──────────┐
│ Column      │ Type         │ Nullable │ Comments │
├─────────────┼──────────────┼──────────┼──────────┤
│ product_id  │ INTEGER      │ NO       │ PK       │
│ name        │ VARCHAR(100) │ NO       │          │
│ category    │ VARCHAR(50)  │ YES      │          │
│ price       │ DECIMAL(10,2)│ YES      │          │
│ in_stock    │ INTEGER      │ YES      │          │
└─────────────┴──────────────┴──────────┴──────────┘

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

**What you get:**
- Table metadata (type, row count)
- Column information (names, types, nullable)
- First 5 rows of actual data

**When to use:**
- Understanding unfamiliar tables
- Quick combined view of structure and content
- Verifying table has expected columns and data

**Qualified names:**

Both commands support database.table syntax:

```sql
tq> /sample staging.test_data 20
tq> /peek development.customers
```

**Error handling:**

Clear messages for common issues:

```sql
tq> /sample nonexistent_table
Error: Table not found

Table 'nonexistent_table' does not exist in database 'production'.
Use /list tables to see available tables.

tq> /sample employees 5000
Error: Invalid sample size

Sample size must be between 1 and 1000.
Requested: 5000
Maximum: 1000

Example: /sample employees 1000
```

### Describe Tables

See the structure of a table:

```sql
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
└───────────────┴──────────────┴──────────┴─────────┴──────────┘
```

**Short alias:** `\d`

```sql
tq> \d employees
```

### Check Connection

Test your database connection:

```sql
tq> /ping

Connection OK (127ms)
Host: myhost.company.com:1025
Database: production
User: alice
Session active for: 15m 23s
```

### Show Session Info

View current connection details:

```sql
tq> /session

Session Information:
  Host: prod-td01.company.com:1025
  Database: production_db
  User: alice
  Session ID: 123456789
  Connected: 2024-01-15 10:30:45
  Duration: 15m 23s
  Logon Mechanism: LDAP
  Character Set: UTF8
  Queries Executed: 42
```

### List All Database Sessions

Monitor all active sessions on the Teradata system:

```sql
tq> /sessions

Sessions:
┌───────────┬──────────┬────────────────────────┬─────────────┬──────────┬───────────┬───────┬─────────────┬────────────────┬──────────────┐
│ SessionNo │ UserName │ LogonTime              │ PEstate     │ AMPState │ AMPCPUSec │ AMPIO │ ReqSpool    │ Amp CPU Skew % │ Amp IO Skew %│
├───────────┼──────────┼────────────────────────┼─────────────┼──────────┼───────────┼───────┼─────────────┼────────────────┼──────────────┤
│      1230 │ DBC      │ 2026/01/27 19:31:24.00 │ IDLE        │ IDLE     │     0.000 │     6 │           0 │           [--] │         [--] │
│      1231 │ DBC      │ 2026/01/27 19:31:24.00 │ IDLE        │ IDLE     │     0.020 │   224 │           0 │           [--] │         [--] │
│      1232 │ DBC      │ 2026/01/27 19:31:25.00 │ DISPATCHING │ ACTIVE   │     9.564 │  5084 │  1168793600 │           4.05 │          .86 │
└───────────┴──────────┴────────────────────────┴─────────────┴──────────┴───────────┴───────┴─────────────┴────────────────┴──────────────┘

3 active session(s) (Query time: 0.749s)
```

**Short alias:** `/s`

```sql
tq> /s
```

**What you see:**

The `/sessions` command displays ALL active sessions on the Teradata system, regardless of their state. You'll see sessions in various states:

- **IDLE/IDLE** - Session connected but not running queries
- **DISPATCHING/ACTIVE** - Session actively executing a query
- **ACTIVE/ACTIVE** - Session processing query results
- **IDLE/ACTIVE** - Session with active AMP operations but idle PE

**Column meanings:**

- **SessionNo** - Unique session identifier
- **UserName** - Database user running the session
- **LogonTime** - When the session connected
- **PEstate** - Parser Engine state (IDLE, DISPATCHING, ACTIVE)
- **AMPState** - AMP state (IDLE, ACTIVE)
- **AMPCPUSec** - Total CPU seconds consumed by AMPs
- **AMPIO** - Total I/O operations by AMPs
- **ReqSpool** - Spool space used by this session (bytes)
- **Amp CPU Skew %** - CPU usage imbalance across AMPs (shows `[--]` for IDLE sessions)
- **Amp IO Skew %** - I/O imbalance across AMPs (shows `[--]` for IDLE sessions)

**Use cases:**

- Monitor active queries and their resource consumption
- Identify sessions using excessive spool space
- Check for workload imbalances (high skew percentages)
- Find long-running sessions
- Verify your own session is connected

**Note:** This command requires SELECT permission on `DBC.MonitorSession`. If you see a permission error, contact your DBA.

### Clear Screen

Clear the terminal:

```sql
tq> /clear
```

### Get Help

Show help information:

```sql
tq> /help

# Or get help for a specific command
tq> /help describe
```

### Exit REPL

Exit the REPL session:

```sql
tq> /quit
```

**Alternative:** Press Ctrl-D on an empty line

## Tips

1. **Use tab completion liberally** - It helps you discover commands and avoid typos
2. **Use `/list` commands** - They're faster than writing SQL for schema exploration
3. **Use patterns** - Filter tables with `/list tables pattern` instead of viewing all tables
4. **Watch for loading indicators** - They tell you when tq is fetching data
5. **Use short aliases** - Commands like `\l`, `\dt`, `\d` save typing
6. **Write readable multi-line queries** - They're stored as single history entries, so formatting makes them easier to recall and edit
7. **Sample before selecting** - Use `/sample` to inspect data before writing complex queries
8. **Peek for quick exploration** - `/peek` shows structure and data together, perfect for unfamiliar tables
9. **Start with small samples** - When exploring large tables, use `/sample table 10` to avoid overwhelming output
10. **Combine sampling with patterns** - First use `/list tables pattern%` to find tables, then `/sample` to inspect them
11. **The pager is experimental** - By default, wide results show with truncation. You can try `/pager on` to test interactive navigation

## Keyboard Shortcuts Reference

### General Editing

| Key | Action |
|-----|--------|
| TAB | Show completions |
| ↑/↓ | Navigate command history (recalls complete multi-line statements) |
| Ctrl-R | Search history |
| Ctrl-C | Cancel current operation |
| Ctrl-D | Exit REPL (on empty line) |
| Ctrl-L | Clear screen |
| Ctrl-A | Move to start of line |
| Ctrl-E | Move to end of line |

### Result Pager Navigation

When viewing query results, these additional keys are available:

**Vertical (Row) Navigation:**

| Key | Action |
|-----|--------|
| j, ↓ | Scroll down one row |
| k, ↑ | Scroll up one row |
| Space, Page Down | Scroll down one page |
| b, Page Up | Scroll up one page |
| g, Home | Jump to first row |
| G, End | Jump to last row |

**Horizontal (Column) Navigation:**

| Key | Action |
|-----|--------|
| l, → | Scroll right one column |
| h, ← | Scroll left one column |
| L (uppercase) | Jump to last column |
| H (uppercase) | Jump to first column |

**Pager Control:**

| Key | Action |
|-----|--------|
| ? | Show navigation help |
| q, Esc | Exit pager, return to REPL |

## Navigating Wide Result Sets

When query results contain many columns that exceed your terminal width, tq automatically enables horizontal paging. This lets you explore all columns by scrolling left and right through the data.

### How Horizontal Paging Works

If your result set is wider than the terminal, tq displays as many columns as fit and shows indicators for hidden columns:

```sql
tq> SELECT * FROM dbc.tables WHERE databasename='dbc';

╭──────────┬───────────┬─────────┬───────┬────────────┬────────────┬─────────────┬────────────╮
│ Database │ TableName │ Version │ Kind  │ Protection │ Journal    │ CreatorName │ (+16 cols) →│
├──────────┼───────────┼─────────┼───────┼────────────┼────────────┼─────────────┼────────────┤
│ dbc      │ tables    │       1 │ T     │ F          │ N          │ DBC         │ ...        │
│ dbc      │ columns   │       1 │ T     │ F          │ N          │ DBC         │ ...        │
│ dbc      │ indexes   │       1 │ T     │ F          │ N          │ DBC         │ ...        │
╰──────────┴───────────┴─────────┴───────┴────────────┴────────────┴─────────────┴────────────╯

Columns 1-7 of 23 | Rows 1-20 of 156 (13%)

Press → or l to see more columns
Press ? for navigation help
Press q or Esc to exit pager
```

The `(+16 cols) →` indicator tells you there are 16 more columns hidden to the right. The status bar shows you're viewing columns 1-7 out of 23 total.

### Horizontal Navigation Keys

**Arrow Keys:**
- **→** (Right Arrow): Scroll one column to the right
- **←** (Left Arrow): Scroll one column to the left

**Vim-Style Keys:**
- **l**: Scroll right (same as →)
- **h**: Scroll left (same as ←)
- **L** (uppercase): Jump to the last column instantly
- **H** (uppercase): Jump to the first column instantly

### Navigation Example

Start with the leftmost columns:

```sql
╭──────────┬───────────┬─────────┬───────┬────────────╮
│ Database │ TableName │ Version │ Kind  │ Protection │
├──────────┼───────────┼─────────┼───────┼────────────┤
│ dbc      │ tables    │       1 │ T     │ F          │
╰──────────┴───────────┴─────────┴───────┴────────────╯

Columns 1-5 of 23 | (+18 cols) →
```

Press **→** or **l** to scroll right:

```sql
╭───────────┬─────────┬───────┬────────────┬────────────╮
│ TableName │ Version │ Kind  │ Protection │ Journal    │
├───────────┼─────────┼───────┼────────────┼────────────┤
│ tables    │       1 │ T     │ F          │ N          │
╰───────────┴─────────┴───────┴────────────┴────────────╯

(+1 cols) ← | Columns 2-6 of 23 | (+17 cols) →
```

Notice the `(+1 cols) ←` indicator now appears on the left, showing one column is hidden to the left. You can press **←** or **h** to scroll back.

Press **L** (uppercase) to jump to the last columns:

```sql
╭───────────────┬──────────────┬────────────────┬────────────┬─────────────╮
│ RequestText   │ RequestSize  │ CommentString  │ CreateTime │ LastAlterTime│
├───────────────┼──────────────┼────────────────┼────────────┼─────────────┤
│ CREATE TABLE..│ 512          │ System table   │ 2024-01-01 │ 2024-01-01  │
╰───────────────┴──────────────┴────────────────┴────────────┴─────────────╯

(+18 cols) ← | Columns 19-23 of 23
```

Press **H** (uppercase) to jump back to the beginning:

```sql
╭──────────┬───────────┬─────────┬───────┬────────────╮
│ Database │ TableName │ Version │ Kind  │ Protection │
├──────────┼───────────┼─────────┼───────┼────────────┤
│ dbc      │ tables    │       1 │ T     │ F          │
╰──────────┴───────────┴─────────┴───────┴────────────╯

Columns 1-5 of 23 | (+18 cols) →
```

### Column Position is Preserved

When you scroll vertically through rows, your horizontal position stays the same. This lets you focus on specific columns while exploring different rows.

**Example:**

Scroll right to see salary information:

```sql
╭────────────┬────────────┬────────────┬──────────╮
│ Department │ Salary     │ Bonus      │ StartDate│
├────────────┼────────────┼────────────┼──────────┤
│ IT         │ 75000      │ 5000       │ 2020-03-15│
│ Sales      │ 68000      │ 12000      │ 2019-07-01│
╰────────────┴────────────┴────────────┴──────────╯

(+3 cols) ← | Columns 4-7 of 15 | Rows 1-2 of 500
```

Press **j** or **↓** to scroll down rows:

```sql
╭────────────┬────────────┬────────────┬──────────╮
│ Department │ Salary     │ Bonus      │ StartDate│
├────────────┼────────────┼────────────┼──────────┤
│ Sales      │ 68000      │ 12000      │ 2019-07-01│
│ Engineering│ 82000      │ 7500       │ 2021-01-10│
╰────────────┴────────────┴────────────┴──────────╯

(+3 cols) ← | Columns 4-7 of 15 | Rows 2-3 of 500
```

Notice the column range (4-7) stays the same while the row range changed. You can scroll through all 500 rows while staying on these salary columns.

### Understanding Column Indicators

**Right Indicator: `(+N cols) →`**
- Shows how many columns are hidden to the right
- Appears in the status bar
- Disappears when you reach the rightmost column

**Left Indicator: `(+N cols) ←`**
- Shows how many columns are hidden to the left
- Appears after you scroll right
- Disappears when you scroll back to the first column

**Status Bar: `Columns X-Y of Z`**
- **X**: First visible column number
- **Y**: Last visible column number
- **Z**: Total number of columns

### About the Interactive Pager

**Status: Experimental (Disabled by Default)**

The interactive pager is currently experimental and disabled by default. When query results are too wide for your terminal, they will be displayed with truncated columns rather than entering pager mode.

**Why disabled?** The pager is undergoing refinement to ensure perfect column alignment and rendering across all terminal widths. Rather than risk a suboptimal experience, it's off by default while improvements continue.

**Want to try it?** You can enable the pager if you'd like to test it:

```sql
tq> /pager on
Pager enabled (experimental)

tq> SELECT * FROM wide_table;
[Enters interactive pager mode if result is wide]
```

Once enabled, you can navigate with j/k/h/l keys and q to exit (see navigation keys section above).

**Disable it again:**

```sql
tq> /pager off
Pager disabled
```

When the pager is disabled (default), wide results display all columns with truncation as needed to fit your terminal width.

### Quick Reference: Navigation Keys

When viewing paged results, these keys control navigation:

**Vertical Navigation (rows):**
- **j** or **↓**: Scroll down one row
- **k** or **↑**: Scroll up one row
- **Space** or **Page Down**: Scroll down one page
- **b** or **Page Up**: Scroll up one page
- **g** or **Home**: Jump to first row
- **G** or **End**: Jump to last row

**Horizontal Navigation (columns):**
- **l** or **→**: Scroll right one column
- **h** or **←**: Scroll left one column
- **L** (uppercase): Jump to last column
- **H** (uppercase): Jump to first column

**Pager Control:**
- **?**: Show help (lists all navigation keys)
- **q** or **Esc**: Exit pager and return to REPL

### Navigation Tips

1. **Use jump keys for efficiency** - Press **L** to quickly see the last columns, or **H** to return to the start
2. **Column position stays locked** - When scrolling vertically, you stay on the same columns. This is perfect for comparing values across rows
3. **Check the status bar** - It always shows your current position (both rows and columns)
4. **Press ? for help** - If you forget the keys, press **?** to see a complete navigation reference
5. **Disable paging when needed** - Use `/pager off` if you prefer to see the full width output (truncated if necessary)

### Example: Exploring a Wide Analytics Table

Let's say you're analyzing a sales report with 40 columns:

```sql
tq> SELECT * FROM sales_report WHERE year = 2024;

╭──────────┬────────────┬───────────┬────────────┬─────────────╮
│ Region   │ SalesPerson│ ProductID │ Units      │ Revenue     │
├──────────┼────────────┼───────────┼────────────┼─────────────┤
│ East     │ Alice      │ P-1001    │ 150        │ 45000.00    │
│ West     │ Bob        │ P-1002    │ 230        │ 68000.00    │
╰──────────┴────────────┴───────────┴────────────┴─────────────╯

Columns 1-5 of 40 | Rows 1-2 of 5,234 (0%)
Press → to see more columns | Press ? for help
```

**Step 1:** Press **L** to jump to the last columns and see year-end totals:

```sql
╭──────────────┬─────────────┬──────────────┬─────────────╮
│ Q4_Revenue   │ YTD_Revenue │ Target_Met   │ Commission  │
├──────────────┼─────────────┼──────────────┼─────────────┤
│ 12000.00     │ 45000.00    │ Yes          │ 2250.00     │
│ 18500.00     │ 68000.00    │ Yes          │ 3400.00     │
╰──────────────┴─────────────┴──────────────┴─────────────╯

(+36 cols) ← | Columns 37-40 of 40 | Rows 1-2 of 5,234 (0%)
```

**Step 2:** Press **j** repeatedly to scan through rows, still viewing the final columns:

```sql
(+36 cols) ← | Columns 37-40 of 40 | Rows 15-16 of 5,234 (0%)
```

**Step 3:** Press **h** a few times to scroll left and see Q3 data:

```sql
╭──────────────┬─────────────┬──────────────┬─────────────╮
│ Q3_Revenue   │ Q4_Revenue  │ YTD_Revenue  │ Target_Met  │
├──────────────┼─────────────┼──────────────┼─────────────┤
│ 10500.00     │ 11000.00    │ 42500.00     │ Yes         │
╰──────────────┴─────────────┴──────────────┴─────────────╯

(+32 cols) ← | Columns 33-36 of 40 | Rows 15-16 of 5,234 (0%)
```

**Step 4:** Press **H** to jump back to the start when you're done:

```sql
╭──────────┬────────────┬───────────┬────────────┬─────────────╮
│ Region   │ SalesPerson│ ProductID │ Units      │ Revenue     │
├──────────┼────────────┼───────────┼────────────┼─────────────┤
│ East     │ Carol      │ P-1015    │ 195        │ 58500.00    │
╰──────────┴────────────┴───────────┴────────────┴─────────────╯

Columns 1-5 of 40 | Rows 15-16 of 5,234 (0%)
```

**Step 5:** Press **q** to exit the pager and return to the REPL prompt:

```sql
tq> _
```

## Next Steps

- Learn about [advanced REPL features](../specifications/repl.md) (multi-line editing, result paging, etc.)
- Explore [batch mode](../specifications/batch-mode.md) for running scripts
- Read about [output formats](../specifications/output-formats.md) (JSON, CSV)
