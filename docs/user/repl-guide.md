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
    /edit        Edit last query in external editor
    /help        Show help
    /inspect     Comprehensive object inspection (type, columns, indexes, size, dependencies)
    /list        Schema inspection (databases, tables, views)
    /locks       Display current lock contention and blocking chains
    /logon       Connect/switch database
    /pager       Enable/disable result paging
    /params      Manage YAML parameter files for variable substitution
    /peek        Show first rows and column info (optional row count)
    /ping        Test connection
    /query       Show current SQL query for a session
    /quit        Exit REPL
    /reconnect   Reconnect to database
    /repeat      Re-execute last query
    /sample      Show random sample
    /session     Show session info
    /sessions    List active Teradata sessions with performance metrics
    /set         Set configuration options
    /show        Show schema information (indexes)
    /sysconfig   Display system configuration (version and AMP count)
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
    /locks       Display current lock contention and blocking chains
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

**Tip:** Metacommands accept trailing semicolons — they are silently stripped. If you type `/describe employees;` or `/list tables;` out of SQL habit, the semicolon is ignored and the command works as expected.

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

### Query Editing Commands

REPL provides two powerful commands for working with your query history: `/repeat` to re-execute queries and `/edit` to modify them in your preferred editor.

#### Re-execute Last Query

Re-execute your last query without retyping:

```sql
tq> SELECT COUNT(*) FROM employees WHERE department = 'IT';

┌─────────┐
│ Count   │
├─────────┤
│ 42      │
└─────────┘

tq> /repeat

┌─────────┐
│ Count   │
├─────────┤
│ 42      │
└─────────┘
```

**Short alias:** `\r`

```sql
tq> \r
[Re-executes last query]
```

**When to use:**
- Checking if data changed after an operation
- Running the same query repeatedly while monitoring changes
- Quickly re-executing after reviewing results

**No previous query:**

If you haven't run any SQL yet, you'll see:

```sql
tq> /repeat
No previous query to repeat
```

**Note:** Only SQL queries are repeated. Metacommands (like `/describe` or `/list`) are not stored as repeatable queries.

#### Edit Last Query in External Editor

Open your last SQL query in your preferred text editor, modify it, and automatically execute the edited version:

```sql
tq> SELECT * FROM employees WHERE department = 'IT';

[Shows results: 42 rows]

tq> /edit
[Opens your editor with the query]
[You change 'IT' to 'Sales' and save]

Executing edited query...

[Shows results: 28 rows from Sales department]
```

**Short alias:** `\e`

```sql
tq> \e
[Opens editor with last query]
```

**How it works:**

1. **Editor Selection**: tq uses your preferred editor in this order:
   - `$VISUAL` environment variable (highest priority)
   - `$EDITOR` environment variable
   - `vi` as fallback

2. **Temporary File**: Your query is written to a temporary `.sql` file with proper syntax highlighting

3. **Edit and Save**: Make your changes in the editor, then save and exit

4. **Automatic Execution**: The edited query runs automatically when you exit the editor

**Setting your editor:**

```bash
# Use Visual Studio Code
export VISUAL="code --wait"

# Use nano
export EDITOR="nano"

# Use vim
export EDITOR="vim"
```

**Example workflow - Refining a query:**

```sql
# Start with a basic query
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

# Open in editor to refine
tq> /edit

[Editor opens with your query. You modify it to:]
SELECT employee_id, first_name, last_name, hire_date, salary
FROM employees
WHERE department = 'IT'
  AND hire_date > '2020-01-01'
ORDER BY salary DESC;

[Save and exit editor]

Executing edited query...

┌─────────────┬────────────┬───────────┬────────────┬───────────┐
│ employee_id │ first_name │ last_name │ hire_date  │ salary    │
├─────────────┼────────────┼───────────┼────────────┼───────────┤
│ 103         │ Carol      │ Chen      │ 2021-07-01 │ 95000.00  │
│ 101         │ Alice      │ Anderson  │ 2020-01-15 │ 75000.00  │
└─────────────┴────────────┴───────────┴────────────┴───────────┘

2 rows in set (0.038s)

# The edited query is now your "last query"
tq> /repeat
[Re-executes the refined query]
```

**Canceling an edit:**

If you exit the editor without making changes, or delete all content:

```sql
tq> /edit
[Opens editor]
[Exit without changes]

No changes made

tq> _
```

If you delete all content and save:

```sql
tq> /edit
[Opens editor]
[Delete all text and save]

Edit cancelled (empty query)

tq> _
```

**When to use:**

- **Complex modifications**: Add JOINs, subqueries, or additional columns
- **Multi-line formatting**: Properly format a long query for readability
- **Iterative refinement**: Start with a simple query, progressively add filters and sorting
- **Learning SQL**: Experiment with query variations using your editor's undo/redo
- **Copy/paste from examples**: Paste query templates from documentation

**Error handling:**

If you haven't run a query yet:

```sql
tq> /edit

Error: No previous query to edit

You haven't executed any SQL queries yet in this session.
Run a query first, then use /edit to modify and re-execute it.
```

If your editor isn't found:

```sql
tq> /edit

Error: Unable to launch editor 'nano'
Reason: Command not found

Suggestions:
  - Install nano: apt install nano (Debian/Ubuntu)
  - Set different editor: export EDITOR=vi
```

**Integration with /repeat:**

After editing a query, you can re-run it with `/repeat`:

```sql
tq> SELECT * FROM orders WHERE status = 'pending';
[Shows 15 rows]

tq> /edit
[Change 'pending' to 'completed']
[Shows 342 rows]

tq> /repeat
[Re-executes the query with status = 'completed']
```

**Tips:**

1. **Use complex editors**: Set `VISUAL="code --wait"` or `VISUAL="emacs"` for full editor features
2. **Multi-line queries**: The editor preserves your formatting, making it easy to work with complex queries
3. **Query templates**: Keep a library of query templates and paste them when editing
4. **Incremental refinement**: Start simple, use `/edit` to add complexity step-by-step
5. **Editor features**: Use your editor's syntax highlighting, autocomplete, and linting for SQL

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

## Parameterized Queries

tq supports SQL templates with `{{variable}}` markers that are resolved from YAML parameter files. In REPL mode you load parameter files with the `/params` metacommand. Once loaded, every query you type is automatically substituted before being sent to Teradata.

### Why Use Parameters?

Instead of rewriting the same query for different tables, databases, or dates, you write a template once and supply values through a YAML file. Parameters also let you switch between environments (staging vs. production) without editing SQL.

### The `/params` Metacommand

The `/params` command manages parameter files within your REPL session:

```
/params load <file>    Load a YAML parameter file
/params unload         Clear all loaded parameters
/params show           Show currently loaded parameters
/params                Display usage help
```

### Loading a Parameter File

Create a YAML file with your variables:

```yaml
# deploy.yaml
target:
  database: PRODUCTION
  schema: HR

run_date: "2026-01-01"
row_count: 100
```

Then load it in the REPL:

```sql
tq> /params load deploy.yaml
Loaded 4 variables from deploy.yaml (4 total)
```

Now every query you type has those variables available:

```sql
tq> SELECT COUNT(*) FROM {{target.database}}.{{target.schema}}.employees
    WHERE hire_date > '{{run_date}}';
-- Executes: SELECT COUNT(*) FROM PRODUCTION.HR.employees WHERE hire_date > '2026-01-01';

┌──────────┐
│ Count(*) │
├──────────┤
│ 312      │
└──────────┘

1 row in set (0.087s)
```

### Viewing Loaded Parameters

Check which variables are available at any time:

```sql
tq> /params show

Active parameters (4 variables from 1 file):

  Variable               Value          Source
  ─────────────────────────────────────────────────────────
  row_count              100            deploy.yaml
  run_date               2026-01-01     deploy.yaml
  target.database        PRODUCTION     deploy.yaml
  target.schema          HR             deploy.yaml

Use {{variable}} in SQL to substitute these values.
Use {{$ENV.VAR_NAME}} for environment variable substitution.
```

When no parameters are loaded:

```sql
tq> /params show
No parameters are currently loaded.

To load parameters: /params load <file.yaml>
To learn more: /help params
```

### Environment Variables

Use `{{$ENV.VAR_NAME}}` to read environment variables directly. No YAML file is needed:

```sql
tq> SELECT * FROM {{$ENV.TARGET_DB}}.HR.employees SAMPLE 10;
```

Environment variables can be combined with YAML params in the same query:

```sql
tq> /params load config.yaml
Loaded 3 variables from config.yaml (3 total)

tq> SELECT * FROM {{$ENV.ENV_NAME}}.{{target.schema}}.employees
    WHERE region = '{{filters.region}}';
```

Environment variable errors are clear:

```sql
tq> SELECT * FROM {{$ENV.MISSING_VAR}}.employees;
Error: Undefined environment variable in template

Variable '{{$ENV.MISSING_VAR}}' references environment variable 'MISSING_VAR'
which is not set in the current environment.

Fix:
  export MISSING_VAR=myvalue
```

### Loading Multiple Files

Load multiple parameter files to combine or override values. Later files override earlier ones on conflicting keys, but non-conflicting keys from earlier files are preserved:

```sql
tq> /params load base.yaml
Loaded 5 variables from base.yaml (5 total)

tq> /params load prod-overrides.yaml
Loaded 2 variables from prod-overrides.yaml (6 total, 1 override)
```

After loading both files, `/params show` reflects the merged result, showing which file each value came from:

```sql
tq> /params show

Active parameters (6 variables from 2 files):

  Variable               Value          Source
  ─────────────────────────────────────────────────────────
  filters.active         true           base.yaml
  filters.region         EMEA           prod-overrides.yaml
  run_date               2026-01-01     base.yaml
  target.database        PRODUCTION     prod-overrides.yaml
  target.schema          HR             base.yaml
  target.table           employees      base.yaml

Use {{variable}} in SQL to substitute these values.
Use {{$ENV.VAR_NAME}} for environment variable substitution.
```

### Clearing Parameters

Clear all loaded parameters with `/params unload`:

```sql
tq> /params unload
Parameters cleared. No variables currently loaded.
```

If no parameters are loaded, you get a notice:

```sql
tq> /params unload
No parameters are currently loaded.
```

### Undefined Variables

If a query references a variable that is not defined, the query is aborted with a clear error. The REPL remains active:

```sql
tq> SELECT * FROM {{undefined_table}};
Error: Undefined variable in template

Variable '{{undefined_table}}' is not defined.

Available variables:
  filters.active    true
  filters.region    EMEA
  target.database   PRODUCTION
  target.schema     HR

Hint: Load a params file with /params load <file> or use /params show to review loaded variables.

tq>
```

### Error Handling

**File not found:**

```sql
tq> /params load missing.yaml
Error: Parameter file not found

Could not read: missing.yaml
Reason: No such file or directory

Check:
  - File path is correct
  - Current directory: /Users/alice/project
```

**YAML parse error:**

```sql
tq> /params load broken.yaml
Error: Invalid YAML in parameter file

Could not parse: broken.yaml
Line 4: mapping values are not allowed in this context

Fix:
  - Verify the file is valid YAML
  - Check for incorrect indentation or missing quotes
```

**Missing file argument:**

```sql
tq> /params load
Error: Missing argument

Usage: /params load <file>

Example: /params load deploy.yaml
```

All `/params` errors are non-fatal: the REPL continues after any error and returns you to the `tq>` prompt.

### Tab Completion

Tab completion works for the `/params` command and its subcommands:

```sql
# Show all /params subcommands
tq> /params <TAB>

Subcommands:
    load    Load a YAML parameter file
    show    Show currently loaded parameters
    unload  Clear all loaded parameters

# Complete partial subcommand
tq> /params l<TAB>
tq> /params load _

# File path completion after 'load'
tq> /params load <TAB>
deploy.yaml  base.yaml  prod-overrides.yaml
```

### Session Persistence

Parameters loaded with `/params load` persist for the duration of your REPL session:

- Parameters survive database reconnects (`/reconnect`)
- Parameters are cleared when you exit (`/quit`, Ctrl-D)
- Parameters are NOT automatically saved between sessions

### Complete Workflow Example

```sql
# 1. Start REPL and load your environment config
tq> /params load config/base.yaml
Loaded 5 variables from base.yaml (5 total)

tq> /params load config/envs/production.yaml
Loaded 2 variables from production.yaml (6 total, 1 override)

# 2. Check what variables are available
tq> /params show

Active parameters (6 variables from 2 files):

  Variable               Value          Source
  ─────────────────────────────────────────────────────────
  filters.active         true           base.yaml
  filters.region         EMEA           production.yaml
  run_date               2026-01-01     base.yaml
  target.database        PRODUCTION     production.yaml
  target.schema          HR             base.yaml
  target.table           employees      base.yaml

# 3. Run parameterized queries
tq> SELECT COUNT(*) FROM {{target.database}}.{{target.schema}}.{{target.table}}
    WHERE region = '{{filters.region}}' AND active = {{filters.active}};

┌──────────┐
│ Count(*) │
├──────────┤
│ 1523     │
└──────────┘

1 row in set (0.091s)

# 4. Quick override: switch to staging without editing any files
tq> /params load config/envs/staging.yaml
Loaded 1 variable from staging.yaml (6 total, 1 override)

tq> SELECT COUNT(*) FROM {{target.database}}.{{target.schema}}.{{target.table}}
    WHERE region = '{{filters.region}}';
-- Now runs against STAGING

# 5. Clear parameters when done
tq> /params unload
Parameters cleared. No variables currently loaded.
```

### Getting Help

```sql
tq> /help params

/params - Manage YAML parameter files for variable substitution

Usage:
  /params load <file>   Load a YAML parameter file
  /params unload        Clear all loaded parameters
  /params show          Show currently loaded parameters

Variable Syntax:
  Use {{variable}} markers in SQL queries:
    SELECT * FROM {{target.database}}.{{target.schema}}.employees;

  Use dot notation for nested YAML keys:
    {{section.key}}  ->  section: { key: value }

  Use $ENV prefix for environment variables:
    {{$ENV.DATABASE_HOST}}  reads the DATABASE_HOST env var

...

See also:
  tq help params    Full variable substitution reference
```

For the complete syntax reference outside the REPL:

```bash
tq help params
```

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

**Customize row count:**

Specify how many rows to preview:

```sql
tq> /peek products 10

Table: PRODUCTION.products
Type: Table
Approximate Rows: 15,432

Columns:
[... column metadata ...]

First 10 rows:
[... 10 rows of data ...]
```

**What you get:**
- Table metadata (type, row count)
- Column information (names, types, nullable)
- First N rows of actual data (default: 5, customizable)

**When to use:**
- Understanding unfamiliar tables
- Quick combined view of structure and content
- Verifying table has expected columns and data
- Use custom row count for larger or smaller previews

**Qualified names and case-insensitivity:**

Both commands support database.table syntax, and table names are resolved case-insensitively:

```sql
tq> /sample staging.test_data 20
tq> /peek development.customers
tq> /peek development.customers 10
tq> /sample EMPLOYEES         -- same as /sample employees
tq> /peek Production.Orders   -- same as /peek production.orders
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
│ first_name    │ VARCHAR(50)  │ YES      │ -       │          │
│ last_name     │ VARCHAR(50)  │ YES      │ -       │          │
│ email         │ VARCHAR(100) │ YES      │ -       │          │
│ hire_date     │ DATE         │ YES      │ -       │          │
│ salary        │ DECIMAL(10,2)│ YES      │ -       │          │
│ department_id │ INTEGER      │ YES      │ -       │ FK       │
└───────────────┴──────────────┴──────────┴─────────┴──────────┘
```

**Short alias:** `\d`

```sql
tq> \d employees
```

#### Show Table Indexes

View index information for a table to understand query optimization and performance:

```sql
tq> /show indexes employees

Indexes on PRODUCTION.employees:
┌────────────────┬─────────────┬─────────────┬────────────────┐
│ IndexName      │ IndexType   │ ColumnName  │ ColumnPosition │
├────────────────┼─────────────┼─────────────┼────────────────┤
│ PK_employees   │ Primary Key │ employee_id │ 1              │
│ idx_dept       │ Secondary   │ department  │ 1              │
│ idx_dept       │ Secondary   │ hire_date   │ 2              │
│ idx_name       │ Secondary   │ last_name   │ 1              │
│ idx_name       │ Secondary   │ first_name  │ 2              │
└────────────────┴─────────────┴─────────────┴────────────────┘

5 index columns across 3 indexes
```

**Qualified table names:**

You can specify the database explicitly:

```sql
tq> /show indexes staging.employees

Indexes on STAGING.employees:
┌────────────────┬─────────────┬─────────────┬────────────────┐
│ IndexName      │ IndexType   │ ColumnName  │ ColumnPosition │
├────────────────┼─────────────┼─────────────┼────────────────┤
│ PK_employees   │ Primary Key │ employee_id │ 1              │
└────────────────┴─────────────┴─────────────┴────────────────┘

1 index column across 1 index
```

**Short alias:** `\di`

```sql
tq> \di employees
```

**Understanding the output:**

- **IndexName**: Name of the index in the database
- **IndexType**: Type of index (Primary Key, Secondary, Unique)
- **ColumnName**: Column included in the index
- **ColumnPosition**: Position of the column within a multi-column index
  - For composite indexes, position shows the order of columns
  - Position 1 is the leading column, position 2 is second, etc.

**Common index types:**

- **Primary Key**: Unique identifier for table rows
- **Secondary**: Non-unique index for faster lookups
- **Unique**: Enforces uniqueness constraint
- **Join Index**: Specialized Teradata index for join optimization

**When to use:**

- Understanding which columns are indexed before writing queries
- Diagnosing slow query performance
- Planning new indexes for query optimization
- Verifying indexes exist after DDL operations

**Error handling:**

If the table doesn't exist:

```sql
tq> /show indexes nonexistent_table

Error: Table not found

Table 'nonexistent_table' does not exist in database 'production'.
Use /list tables to see available tables.
```

If you don't have permission to view index information:

```sql
tq> /show indexes secure_table

Error: Permission denied

You do not have permission to view index information for 'secure_table'.
Contact your database administrator if you need access.
```

### Inspect a Database Object

The `/inspect` command gives you a single, comprehensive view of any object — its type, columns, index structure, storage metrics, and dependency relationships. It is the "go deeper" companion to `/describe` and `/show indexes`.

**Short alias:** `\i`

**Syntax:**

```sql
/inspect <object>
/inspect <database>.<object>
```

#### Inspect a Table

```sql
tq> /inspect employees

── Object Info ───────────────────────────────────────────

  Type:      Table
  Database:  PRODUCTION
  Name:      employees
  Created:   2023-04-15 09:12:33

── Columns ───────────────────────────────────────────────

┌───────────────┬──────────────┬──────────┬─────────┐
│ Column        │ Type         │ Nullable │ Default │
├───────────────┼──────────────┼──────────┼─────────┤
│ employee_id   │ INTEGER      │ NO       │ -       │
│ first_name    │ VARCHAR(50)  │ YES      │ -       │
│ last_name     │ VARCHAR(50)  │ YES      │ -       │
│ email         │ VARCHAR(100) │ YES      │ -       │
│ hire_date     │ DATE         │ YES      │ -       │
│ salary        │ DECIMAL(10,2)│ YES      │ -       │
│ department_id │ INTEGER      │ YES      │ -       │
└───────────────┴──────────────┴──────────┴─────────┘

7 columns

── Index Structure ───────────────────────────────────────

  Primary Index
    Type:     Unique Primary Index (UPI)
    Columns:  employee_id

  Secondary Indexes
    #1  Non-Unique Secondary Index (NUSI)  (department_id)
    #2  Unique Secondary Index (USI)       (email)

── Storage ───────────────────────────────────────────────

  Current Size:  1.4 GB
  Peak Size:     1.8 GB
  Skew Factor:   8.2%  (low skew)
  AMPs:          32
```

#### Inspect a View

Views show a **Dependencies** section instead of Index Structure and Storage, so you can trace what the view reads from and what objects depend on it.

```sql
tq> /inspect active_employees_view

── Object Info ───────────────────────────────────────────

  Type:      View
  Database:  PRODUCTION
  Name:      active_employees_view
  Created:   2024-01-10 14:22:07

── Columns ───────────────────────────────────────────────

┌───────────────┬──────────────┬──────────┬─────────┐
│ Column        │ Type         │ Nullable │ Default │
├───────────────┼──────────────┼──────────┼─────────┤
│ employee_id   │ INTEGER      │ NO       │ -       │
│ first_name    │ VARCHAR(50)  │ YES      │ -       │
│ last_name     │ VARCHAR(50)  │ YES      │ -       │
│ department_id │ INTEGER      │ YES      │ -       │
└───────────────┴──────────────┴──────────┴─────────┘

4 columns

── Dependencies ──────────────────────────────────────────

  Uses (upstream)
    PRODUCTION.employees          (Table)
    PRODUCTION.departments        (Table)

  Used By (downstream)
    ANALYTICS.employee_report_v   (View)
```

#### Using a Qualified Name

To inspect an object in a different database, prefix the name with the database:

```sql
tq> /inspect staging.orders
tq> \i dbc.tables
```

#### Graceful Degradation

`/inspect` fetches each section independently. If your account lacks access to a specific DBC system view, that section shows an informative note instead of failing the entire command:

```
── Storage ───────────────────────────────────────────────

  (Access denied — requires SELECT on DBC.TableSizeV)
```

All other sections are still displayed normally.

**When to use `/inspect` vs other commands:**

| Command | Best for |
|---------|----------|
| `/describe` | Quick column lookup |
| `/show indexes` | Index details only |
| `/inspect` | Full picture — type, columns, indexes, size, and dependencies in one shot |

**Cross-reference:** For batch and scripting use, see `tq inspect` in the Batch Mode Guide.

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

### Display System Configuration

Check the Teradata version and AMP count at a glance:

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
```

**Short alias:** `/sc`

```sql
tq> /sc
```

**What you see:**

- **Teradata Version** - The installed software version retrieved from `DBC.DBCInfoV`
- **Release** - Full release string including build date
- **AMP Count** - Total number of Access Module Processors (computed via `HASHAMP()+1`)

**Use cases:**

- Confirm the Teradata version before running version-specific SQL features
- Verify AMP count during capacity planning
- Document configuration snapshots as part of change management
- Quickly orient yourself when connecting to an unfamiliar system

**Output formats:**

Use `/set format` to switch output format before running the command:

```sql
# CSV output - useful for logging and spreadsheet analysis
tq> /set format csv
tq> /sysconfig
Property,Value
Teradata Version,17.20.00.17
Release,"17.20.00.17 (Released: 2024-01-15)"
AMP Count,128

# JSON output - useful for scripting
tq> /set format json
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

tq> /sys<TAB>
tq> /sysconfig_
```

**Batch mode equivalent:**

```bash
# Table output (default)
tq sysconfig

# JSON output for scripting
tq sysconfig --format json

# Capture AMP count in a shell script
AMP_COUNT=$(tq sysconfig --format json | jq '.["AMP Count"]')
echo "System has ${AMP_COUNT} AMPs"

# Export a configuration snapshot to CSV
tq sysconfig --format csv --output sysconfig_$(date +%Y%m%d).csv
```

**Note:** This command requires SELECT privilege on `DBC.DBCInfoV`. If you see a permission error, contact your DBA and request access with:
```sql
GRANT SELECT ON DBC.DBCInfoV TO <your_username>;
```

---

---

### Display Lock Information

Inspect current lock contention on the system. This shows which objects are locked, what type of lock is held, which session holds it, and which sessions are waiting:

```sql
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

When no locks are active, you'll see a clean confirmation instead of an empty table:

```sql
tq> /locks

Lock Information:
No locks currently held.

(Query time: 0.023s)
```

**Short alias:** `/lk`

```sql
tq> /lk
```

**What you see:**

- **Locked Object** - Fully qualified name of the locked object (`database.table`)
- **Lock Type** - Granularity of the lock: `Table`, `Row Hash`, or `Database`
- **Lock Mode** - Severity of the lock (see lock mode table below)
- **Locking Sess** - Session ID that currently holds the lock
- **Waiting Sess** - Comma-separated list of session IDs waiting for this lock, or `(none)` if none are waiting

**Lock mode reference:**

| Lock Mode | Blocks | Description |
|-----------|--------|-------------|
| ACCESS | EXCLUSIVE only | Weakest lock. Allows concurrent reads and writes. |
| READ | WRITE, EXCLUSIVE | Shared lock. Allows concurrent reads. |
| WRITE | WRITE, EXCLUSIVE | Exclusive writes. Allows concurrent reads. |
| EXCLUSIVE | All modes | Strongest lock. Blocks all other lock modes. |

**Blocking chain summary:**

When one or more sessions are waiting for a lock, tq automatically identifies the blocking chain and summarizes it below the table:

```
Blocking Chain:
  Session 1023 blocks sessions: 1045, 1051, 1067
  Session 1089 blocks sessions: 1092
```

This makes it immediately clear which session to investigate or ask to commit/rollback.

**Use cases:**

- Diagnose why a query is hanging - check if it is waiting for a lock
- Identify which session is causing blocking and contact its owner
- Confirm the system is clear before running a maintenance operation
- Investigate lock contention during peak ETL load windows

**Output formats:**

```sql
# JSON output - useful for automated monitoring scripts
tq> /set format json
tq> /locks
[
  {
    "Locked Object": "PRODUCTION.orders",
    "Lock Type": "Table",
    "Lock Mode": "WRITE",
    "Locking Sess": 1023,
    "Waiting Sess": [1045, 1067]
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

Note: The "Blocking Chain" summary section appears only in table format. In CSV and JSON output, the raw data is returned so you can derive chains programmatically.

**Tab completion:**

```sql
tq> /lk<TAB>
tq> /locks_

tq> /lo<TAB>

Matching metacommands:
    /locks   Display current lock contention and blocking chains
    /logon   Connect/switch database
```

**Batch mode equivalent:**

```bash
# Table output (default)
tq locks

# JSON output for scripting
tq locks --format json

# Filter for only EXCLUSIVE locks
tq locks --format json | jq '.[] | select(.["Lock Mode"] == "EXCLUSIVE")'

# Check if any sessions are blocked and alert
BLOCKED=$(tq locks --format json | jq '[.[] | select(.["Waiting Sess"] | length > 0)] | length')
if [[ "$BLOCKED" -gt 0 ]]; then
  echo "WARNING: $BLOCKED blocking lock(s) detected"
fi

# Export a lock snapshot for incident analysis
tq locks --format csv --output locks_$(date +%Y%m%d_%H%M%S).csv
```

**Note:** This command requires SELECT privilege on `DBC.LockInfoV`. If you see a permission error, contact your DBA and request access with:
```sql
GRANT SELECT ON DBC.LockInfoV TO <your_username>;
```

---

### Inspect a Session's Current Query

When you see a session in `/sessions` that is consuming significant resources, use `/query` to see the recent SQL queries it has been running:

```sql
tq> /query 1023

Recent Queries for Session 1023:

┌──────────────┬─────────────────────────────────────────────────────────────┐
│ Property     │ Value                                                       │
├──────────────┼─────────────────────────────────────────────────────────────┤
│ Query #      │ 1                                                           │
│ Start Time   │ 2026-02-24 10:30:00                                         │
│ Elapsed Time │ 00:00:05.123                                                │
│ Status       │ Complete                                                    │
│ SQL          │ UPDATE PRODUCTION.orders SET status = 'shipped' WHERE ...  │
└──────────────┴─────────────────────────────────────────────────────────────┘

1 recent query(ies) for session 1023 (Query time: 0.123s)
```

**Short alias:** `/qi`

```sql
tq> /qi 1023
```

**What you see:**

Up to 5 most recent queries for the session, each showing:

- **Query #** - Sequential number (1 = most recent)
- **Start Time** - When the query started
- **Elapsed Time** - How long the query ran
- **Status** - Query status (Complete, Active, Aborted, Error)
- **SQL** - The SQL text (truncated at 200 characters in table view)

**Viewing the full SQL text:**

Long queries are truncated to 200 characters in the table view. To see the complete SQL, switch to CSV or JSON output:

```sql
# JSON output - full untruncated query text
tq> /set format json
tq> /query 1023
[
  {
    "SessionID": 1023,
    "StartTime": "2026-02-24 10:30:00",
    "ElapsedTime": "00:00:05.123",
    "Status": "Complete",
    "QueryText": "UPDATE PRODUCTION.orders SET status = 'shipped' WHERE order_date < '2026-01-01' AND status = 'pending' AND warehouse_id IN (SELECT id FROM warehouses WHERE region = 'WEST')"
  }
]

# CSV output - also returns full untruncated query text
tq> /set format csv
tq> /query 1023
SessionID,StartTime,ElapsedTime,Status,QueryText
1023,2026-02-24 10:30:00,00:00:05.123,Complete,"UPDATE PRODUCTION.orders SET status = 'shipped' WHERE order_date < '2026-01-01' AND status = 'pending'"
```

**Typical workflow - drill down from sessions:**

```sql
# Step 1: Find resource-heavy sessions
tq> /sessions

Session Information:
┌───────────┬──────────┬──────────┬─────────────┬────────────┬──────────────┬─────────────┐
│ SessionNo │ UserName │ PEState  │ AMPCPUTime  │ SpoolUsage │ AmpCPUSkew%  │ SpoolSkew%  │
├───────────┼──────────┼──────────┼─────────────┼────────────┼──────────────┼─────────────┤
│ 1023      │ etl_user │ Active   │ 847.3       │ 12.4 GB    │ 23%          │ 31%         │
│ 1045      │ bi_user  │ Blocked  │ 0.0         │ 0 B        │ 0%           │ 0%          │
└───────────┴──────────┴──────────┴─────────────┴────────────┴──────────────┴─────────────┘

# Step 2: Inspect the SQL for the heavy session
tq> /query 1023

Recent Queries for Session 1023:

┌──────────────┬─────────────────────────────────────────────────────────────┐
│ Property     │ Value                                                       │
├──────────────┼─────────────────────────────────────────────────────────────┤
│ Query #      │ 1                                                           │
│ Start Time   │ 2026-02-24 10:30:00                                         │
│ Elapsed Time │ 00:00:05.123                                                │
│ Status       │ Active                                                      │
│ SQL          │ UPDATE PRODUCTION.orders SET status = 'shipped' WHERE ...  │
└──────────────┴─────────────────────────────────────────────────────────────┘

1 recent query(ies) for session 1023 (Query time: 0.123s)

# Step 3: If blocking is involved, correlate with /locks
tq> /locks

Blocking Chain:
  Session 1023 blocks sessions: 1045
```

**Error scenarios:**

When no session ID is provided:

```sql
tq> /query

Usage: /query <session_id>
       /qi <session_id>

Show recent SQL queries for a given session.

Examples:
  /query 1234
  /qi 1234

Use /sessions to list active session IDs.
```

When a non-integer session ID is given:

```sql
tq> /query abc

Error: 'abc' is not a valid session ID. Expected a number.
```

When the session is not found (disconnected or no DBQL record):

```sql
tq> /query 9999

No queries found for session 9999.
The session may be idle, or DBQL logging may not be enabled.
```

When DBQL logging is not available on the system:

```sql
tq> /query 1023

Error: DBQL query log not available.

DBC.QryLogV requires DBQL (Database Query Log) to be enabled.
Contact your DBA to enable DBQL logging.
```

**Use cases:**

- Identify the SQL being run by a resource-heavy session found via `/sessions`
- Correlate a blocking session (from `/locks`) with the query causing the block
- Audit what a specific session has been running

**Tab completion:**

```sql
tq> /q<TAB>

Matching metacommands:
    /query   Show the current SQL query for a session
    /quit    Exit REPL

tq> /query <TAB>
[Accepts session ID argument]
```

**Batch mode equivalent:**

```bash
# Show query for session 1023
tq query-inspect 1023

# JSON output for scripting
tq query-inspect 1023 --format json

# Extract query text in a monitoring script
tq query-inspect 1023 --format json | jq -r '.[0].QueryText'

# CSV export for reporting
tq query-inspect 1023 --format csv
```

**Note:** This command requires SELECT privilege on `DBC.QryLogV`. Query text is only available when DBQL (Database Query Log) logging is enabled on the system. If you see a permission error, contact your DBA and request access with:
```sql
GRANT SELECT ON DBC.QryLogV TO <your_username>;
```

---

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
5. **Use short aliases** - Commands like `/sc`, `/lk`, `/s`, `\l`, `\dt`, `\d`, `\i` save typing
6. **Write readable multi-line queries** - They're stored as single history entries, so formatting makes them easier to recall and edit
7. **Use `/edit` for complex changes** - When you need to refine a query with JOINs, filters, or formatting, use `/edit` to open it in your editor
8. **Sample before selecting** - Use `/sample` to inspect data before writing complex queries
9. **Peek for quick exploration** - `/peek` shows structure and data together, perfect for unfamiliar tables. Use `/peek table 10` to see more rows
10. **Start with small samples** - When exploring large tables, use `/sample table 10` to avoid overwhelming output
11. **Combine sampling with patterns** - First use `/list tables pattern%` to find tables, then `/sample` to inspect them
12. **Iterative query development** - Start simple, run it, then use `/edit` to add complexity step by step
13. **Use `/inspect` to understand unfamiliar objects** - When you encounter a table or view you have never seen before, `/inspect` gives you type, columns, indexes, storage size, and dependencies in a single command
14. **The pager is experimental** - By default, wide results show with truncation. You can try `/pager on` to test interactive navigation
15. **Orient yourself on new systems** - Run `/sysconfig` as soon as you connect to an unfamiliar Teradata instance to check the version and AMP count
16. **Use `/locks` before maintenance** - Always check for active locks before running DDL or bulk operations to avoid unexpected blocking
17. **Diagnose hangs with `/locks` + `/sessions`** - If a query is hanging, use `/locks` to see if it is blocked, then cross-reference with `/sessions` to identify the blocking session's workload
18. **Drill into heavy sessions with `/query`** - When `/sessions` shows a session with high CPU or spool, use `/query <session_id>` to see what SQL it is executing
19. **Use `/params` for repeatable analysis** - Load a YAML parameter file once, then run the same SQL template against different databases or dates without retyping queries
20. **Switch environments quickly with `/params load`** - Load a different override file to point queries at staging or production without editing SQL
21. **Use `/params show` to audit active parameters** - Check which values are loaded before running a critical query

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

**Status: 🧪 Experimental (Disabled by Default)**

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
- Explore [batch mode guide](batch-mode-guide.md) for running scripts with `--params` and other flags
- Read about [output formats](../specifications/output-formats.md) (JSON, CSV)
- Run `tq help params` for the complete variable substitution syntax reference
