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
    /ping        Test connection
    /quit        Exit REPL
    /reconnect   Reconnect to database
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

## Keyboard Shortcuts Reference

| Key | Action |
|-----|--------|
| TAB | Show completions |
| ↑/↓ | Navigate history (recalls complete multi-line statements) |
| Ctrl-R | Search history |
| Ctrl-C | Cancel current operation |
| Ctrl-D | Exit REPL (on empty line) |
| Ctrl-L | Clear screen |
| Ctrl-A | Move to start of line |
| Ctrl-E | Move to end of line |

## Next Steps

- Learn about [advanced REPL features](../specifications/repl.md) (multi-line editing, result paging, etc.)
- Explore [batch mode](../specifications/batch-mode.md) for running scripts
- Read about [output formats](../specifications/output-formats.md) (JSON, CSV)
