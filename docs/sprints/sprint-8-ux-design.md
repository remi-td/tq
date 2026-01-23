---
sprint: 8
date: 2026-01-18
author: cli-ux-designer agent
status: Design Complete
---

# Sprint 8 UX Design: Critical Bug Fixes

## Overview

This document provides UX design specifications for the four critical bugs being fixed in Sprint 8. Each design addresses user-facing issues with clear, actionable improvements.

---

## Bug 4 (P1): Fixed LIMIT Hint Message

### Problem

**Current Message:**
```
Showing first 100 rows. Add LIMIT clause for different results.
```

**Issue:** Teradata does not support MySQL/PostgreSQL `LIMIT` syntax. This message confuses users by suggesting invalid SQL.

**Teradata Syntax:**
- `SELECT TOP N` - Standard Teradata row limiting
- `SELECT * FROM table SAMPLE N` - Alternative sampling syntax

### Design Solution

**New Message:**
```
Showing first 100 rows. Use TOP N or SAMPLE N for different results.
```

**Examples to Show User:**

When displaying truncated results, optionally show examples:
```
Showing first 100 rows. Use TOP N or SAMPLE N for different results.

Examples:
  SELECT TOP 50 * FROM employees;
  SELECT * FROM employees SAMPLE 200;
```

**Rationale:**
- Mentions both Teradata-specific keywords (TOP and SAMPLE)
- Concise and actionable
- Matches Teradata SQL documentation terminology
- Doesn't overwhelm user with too much detail in basic hint

**Implementation Locations:**
1. REPL result display (after queries returning >100 rows)
2. Batch mode result display
3. Help text for paging/result limits
4. Error messages related to result truncation

### Visual Example

**Before:**
```sql
tq> SELECT * FROM employees;
... 100 rows displayed ...
Showing first 100 rows. Add LIMIT clause for different results.
```

**After:**
```sql
tq> SELECT * FROM employees;
... 100 rows displayed ...
Showing first 100 rows. Use TOP N or SAMPLE N for different results.
```

**With Examples (verbose mode):**
```sql
tq> SELECT * FROM employees;
... 100 rows displayed ...

Showing first 100 rows. Use TOP N or SAMPLE N for different results.

Examples:
  SELECT TOP 50 * FROM employees;       -- Get first 50 rows
  SELECT * FROM employees SAMPLE 200;   -- Sample 200 rows
```

### Help Text Updates

Update `/help` output to mention TOP/SAMPLE instead of LIMIT:

**Before:**
```
Large result sets are automatically limited to 100 rows.
Use LIMIT clause to control result size.
```

**After:**
```
Large result sets are automatically limited to 100 rows.
Use TOP N or SAMPLE N to control result size.

Examples:
  SELECT TOP 50 * FROM table;        -- First 50 rows
  SELECT * FROM table SAMPLE 1000;   -- Sample 1000 rows
```

---

## Bug 2 (P0): Tab Completion Visual Feedback

### Problem

**Current Behavior:**
- User presses Tab
- Nothing happens (no visual feedback)
- No indication that metadata is loading
- No error messages when queries fail
- No indication of what's being completed (databases vs tables vs columns)

**User Impact:** Users don't know if completion is working, loading, or broken.

### Design Solution

#### 1. Loading Indicators

**First Tab Press (Metadata Not Cached):**

Show spinner or loading message while fetching metadata:

```sql
tq> SELECT * FROM <TAB>
Loading tables... ⠋
```

Animate spinner during load:
```
⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
```

After loading completes (< 500ms):
```sql
tq> SELECT * FROM <TAB>
    customers    employees    orders    products    [50 more...]
```

**Slow Database Response:**

If query takes >500ms, show extended loading message:
```sql
tq> SELECT * FROM <TAB>
Loading tables (this may take a moment)... ⠋
```

**Subsequent Tab Presses (Cached):**

Instant response, no loading indicator:
```sql
tq> SELECT * FROM <TAB>
    customers    employees    orders    products    [50 more...]
```

#### 2. Context Indicators

Show what type of completion is being performed:

**Database Names:**
```sql
tq> SELECT * FROM <TAB>
Databases:
    production    staging    development    analytics
```

**Table Names:**
```sql
tq> SELECT * FROM production.<TAB>
Tables in 'production':
    customers    employees    orders    products    invoices
```

**Column Names:**
```sql
tq> SELECT * FROM employees WHERE <TAB>
Columns in 'employees':
    employee_id (INT)    first_name (VARCHAR)    last_name (VARCHAR)
    email (VARCHAR)      hire_date (DATE)        salary (DECIMAL)
```

#### 3. Error Messages

**Metadata Query Fails (Permissions):**
```sql
tq> SELECT * FROM <TAB>
Warning: Cannot load table list (permission denied to DBC.TablesV)
Tab completion for tables unavailable.

Suggestion: Contact DBA to grant SELECT on DBC.TablesV
```

**Metadata Query Timeout:**
```sql
tq> SELECT * FROM <TAB>
Warning: Table list query timed out after 500ms
Tab completion for tables unavailable.

Suggestion: Database may be slow. Try again or continue typing manually.
```

**No Tables Found (Empty Database):**
```sql
tq> SELECT * FROM <TAB>
(No tables found in current database)

Suggestion: Check you're connected to the correct database with /session
```

**Connection Lost:**
```sql
tq> SELECT * FROM <TAB>
Error: Connection lost. Cannot retrieve table list.

Suggestion: Use /ping to check connection or /logon to reconnect.
```

**Cannot Determine Context:**
```sql
tq> SELECT <TAB>
Cannot determine table context. Specify table in FROM clause first.

Example: SELECT * FROM employees WHERE <TAB>
```

#### 4. Progress Indicators for Large Metadata Sets

When fetching large numbers of objects, show progress:

```sql
tq> SELECT * FROM <TAB>
Loading tables... 1,234 found ⠋
```

#### 5. Cache Status Indicator (Optional)

For advanced users, show cache status in verbose mode:

```sql
tq> /verbose on
tq> SELECT * FROM <TAB>
[Cache hit: 156 tables in 12ms]
    customers    employees    orders    products    [152 more...]
```

### Visual Design Principles

1. **Non-Blocking:** Loading indicators don't prevent user from continuing to type
2. **Timeout:** Maximum 500ms wait before showing "this may take a moment"
3. **Cancellable:** User can press Esc to cancel loading and continue typing
4. **Informative:** Always tell user what's happening and why
5. **Graceful Degradation:** If completion fails, user can still type manually

### Implementation Notes

- Use ANSI escape codes for spinner animation
- Spinner updates every 100ms
- Loading message appears after 200ms (avoid flash for fast queries)
- Error messages go to stderr in red color (if colors enabled)
- Success indicators in subtle gray (don't distract from completions)

---

## Bug 2 (P0): Teradata-Specific Tab Completion Behavior

### Problem

**Current Approach:**
- Assumes single-level table names (MySQL/PostgreSQL style)
- Doesn't handle Teradata's `database.table` qualified naming
- No intelligent caching strategy for large Teradata systems

**Teradata Reality:**
- Tables are fully qualified: `database.table`
- Unqualified names resolve to current database
- Teradata systems can have millions of tables across hundreds of databases
- Best practice: Always use fully qualified names

### Design Solution

#### Completion Strategy

**Context: After FROM Keyword**

Show two types of completions:
1. **Database names** (for fully qualified references)
2. **Table names in current database** (for quick local access)

```sql
tq> SELECT * FROM <TAB>

Databases:
    production    staging    development    analytics    [10 more...]

Tables in current database (production):
    customers    employees    orders    products    [50 more...]
```

**User Types Database Name:**
```sql
tq> SELECT * FROM prod<TAB>
    production
```

**User Types Database Dot:**
```sql
tq> SELECT * FROM production.<TAB>
Tables in 'production':
    customers    employees    orders    products    invoices    [45 more...]
```

**User Types Partial Table Name in Current Database:**
```sql
tq> SELECT * FROM emp<TAB>
    employees    employee_archive    emp_summary
```

#### Intelligent Caching Strategy

**Lazy Loading Approach:**

1. **On REPL Startup:** Don't load any metadata
2. **First Tab After FROM:** Load:
   - All database names (typically <100, very fast)
   - Tables in current database only
3. **User Explores New Database:** Cache tables for that database on-demand
4. **Typical Session:** User works with 2-3 databases, caching is minimal

**Cache Structure:**
```
MetadataCache {
    databases: Vec<String>,           // All database names (small)
    current_database: String,         // Active database
    tables: HashMap<String, Vec<Table>>,  // Per-database table cache
    columns: HashMap<String, Vec<Column>>, // Per-table column cache
}
```

**Cache Lifecycle:**

1. **Load:** On-demand when user requests completion
2. **Invalidate:** On successful DDL (CREATE/DROP/ALTER)
3. **Clear:** On `/logon` (new connection, new database context)
4. **Size Limit:** Max 100 databases cached, LRU eviction

**DDL Detection for Cache Refresh:**

After successful DDL statements, invalidate relevant cache:

```sql
tq> CREATE TABLE new_table (id INT);
Table created successfully.
[Cache invalidated for current database]

tq> SELECT * FROM <TAB>
[Re-fetches table list, includes new_table]
```

DDL keywords to watch:
- CREATE TABLE
- DROP TABLE
- ALTER TABLE
- RENAME TABLE
- CREATE DATABASE
- DROP DATABASE

#### Completion Behavior Examples

**Scenario 1: Simple Query in Current Database**
```sql
tq> SELECT * FROM emp<TAB>
    employees    employee_archive    emp_summary

tq> SELECT * FROM employees WHERE first_n<TAB>
    first_name
```

**Scenario 2: Cross-Database Query**
```sql
tq> SELECT * FROM prod<TAB>
    production

tq> SELECT * FROM production.<TAB>
Tables in 'production':
    customers    employees    orders    products

tq> SELECT * FROM production.emp<TAB>
    employees
```

**Scenario 3: JOIN with Multiple Databases**
```sql
tq> SELECT * FROM production.employees e
    JOIN staging.<TAB>
Tables in 'staging':
    test_data    imports    staging_employees
```

**Scenario 4: After DDL**
```sql
tq> CREATE TABLE test_table (id INT);
Table created successfully.

tq> SELECT * FROM test<TAB>
[Cache refreshed]
    test_table    test_data    test_archive
```

#### Performance Requirements

| Operation | Target | Rationale |
|-----------|--------|-----------|
| Load database names | <200ms | Small list, infrequent |
| Load tables in one database | <500ms | Moderate list, cached |
| Cached completion | <50ms | Instant user feedback |
| DDL cache refresh | <500ms | Background, non-blocking |

#### Metadata Queries

**List All Databases:**
```sql
SELECT DISTINCT TRIM(DatabaseName) AS database_name
FROM DBC.TablesV
ORDER BY DatabaseName;
```

**List Tables in Database:**
```sql
SELECT TRIM(TableName) AS table_name,
       TableKind
FROM DBC.TablesV
WHERE DatabaseName = ?
ORDER BY TableName;
```

**Get Current Database:**
```sql
SELECT DATABASE;
```

#### UX Refinements

**Visual Grouping:**

When showing both databases and tables, use clear visual separation:

```sql
tq> SELECT * FROM <TAB>

── Databases ──────────────────────
  production    staging    development

── Tables (current: production) ───
  customers     employees    orders
```

**Prioritization:**

Show most relevant completions first:
1. Exact prefix matches
2. Tables in current database
3. Database names
4. Tables in other databases (if requested)

**Typing Feedback:**

As user types, narrow completions:
```sql
tq> SELECT * FROM e<TAB>
Tables in 'production' starting with 'e':
    employees    employee_archive    events
```

### Error Handling

**Database Not Found:**
```sql
tq> SELECT * FROM nonexistent_db.<TAB>
Error: Database 'nonexistent_db' not found

Available databases: production, staging, development
```

**Permission Denied on Database:**
```sql
tq> SELECT * FROM restricted_db.<TAB>
Warning: Cannot access database 'restricted_db' (permission denied)

Suggestion: Contact DBA for access or use different database
```

### Implementation Notes

1. Parse SQL context to determine completion type
2. Detect `FROM keyword + partial text` for database/table completion
3. Detect `FROM database.` pattern for table-in-database completion
4. Use regex for simple cases, accept limitations for complex queries
5. Cache metadata in REPL session state
6. Refresh cache asynchronously after DDL
7. Clear cache on connection change

---

## Summary of UX Improvements

### Bug 4: LIMIT Hint
- **Change:** "LIMIT clause" → "TOP N or SAMPLE N"
- **Impact:** Users get correct Teradata syntax guidance
- **Locations:** Result hints, help text, documentation

### Bug 2: Tab Completion Feedback
- **Loading Indicators:** Spinner, status messages during metadata fetch
- **Context Indicators:** Show what's being completed (databases/tables/columns)
- **Error Messages:** Clear guidance when completion fails
- **Visual Design:** Non-blocking, informative, graceful degradation

### Bug 2: Teradata-Specific Completion
- **Qualified Names:** Support `database.table` pattern
- **Intelligent Caching:** Lazy load, per-database cache, LRU eviction
- **DDL Detection:** Auto-refresh cache after CREATE/DROP/ALTER
- **Performance:** <500ms load, <50ms cached, minimal memory

---

## Design Principles Applied

1. **Clear Communication:** Always tell user what's happening
2. **Forgiveness:** Graceful degradation when things fail
3. **Performance:** Fast feedback, intelligent caching
4. **Context Awareness:** Understand Teradata's naming model
5. **Best Practices:** Encourage fully qualified names
6. **Actionable Errors:** Every error suggests a solution

---

## Testing Recommendations

### Manual Testing Scenarios

1. **Hint Message:** Query >100 rows, verify TOP/SAMPLE mentioned
2. **Loading Indicator:** First tab press shows spinner
3. **Cached Response:** Second tab press is instant
4. **Database Completion:** `FROM <TAB>` shows databases + tables
5. **Qualified Completion:** `FROM db.<TAB>` shows tables in db
6. **Error Handling:** Disconnect database, verify error messages
7. **DDL Refresh:** CREATE TABLE, verify new table appears in completion
8. **Performance:** Measure load times, verify <500ms target

### Edge Cases to Test

1. Empty database (no tables)
2. Database with 10,000+ tables
3. Permission denied on DBC.TablesV
4. Slow network (simulate with delay)
5. Connection lost mid-completion
6. Multiple rapid tab presses
7. Tab press while query running

---

## Documentation Updates Required

1. Update help text to mention TOP/SAMPLE
2. Update REPL mode specification with Teradata tab completion
3. Update user guide with tab completion examples
4. Add troubleshooting section for completion issues

---

## Next Steps

1. **rust-teradata-architect:** Implement fixes based on this UX design
2. **quality-validator:** Design test cases covering all scenarios
3. **Main agent:** Validate fixes against real Teradata database
4. **User:** Acceptance testing to confirm bugs are resolved

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-18 | 1.0 | Initial UX design for Sprint 8 bug fixes | CLI UX Designer Agent |
