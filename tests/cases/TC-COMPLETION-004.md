---
id: TC-COMPLETION-004
title: Tab Completion - Qualified Name Completion (database.table)
category: Functionality
priority: High
sprint: 18
bug: Tab completion inserts text at wrong position, keyword completion interfering
created: 2026-01-21
updated: 2026-01-21
status: PENDING
---

# Test Case TC-COMPLETION-004: Tab Completion - Qualified Name Completion

## Purpose

Verify that tab completion works correctly for qualified database.table patterns, completing both the database name and table name appropriately without keyword interference or position errors.

## Scope

**Testing:**
- Tab completion after database name + period (e.g., "DBC.")
- Completion filters to tables in specified database only
- Qualified completion works in FROM and JOIN contexts
- Text inserted at correct position
- NO keyword suggestions

**Not Testing:**
- Unqualified completion (covered in other tests)
- Column completion (covered in TC-COMPLETION-003)
- Three-part names (database.schema.table - not applicable to Teradata)

## Prerequisites

- tq binary built in release mode: `cargo build --release`
- Live Teradata database connection
- TQ_LOGON configured in .env or environment variable
- Access to multiple databases (at least DBC and one user database)
- Permissions to query DBC.TablesV

## Test Procedure

### Setup

```bash
# Build release binary
cargo build --release

# Start tq REPL
./target/release/tq
```

### Execution Steps

**Step 1: Qualified completion in FROM clause**

Type:
```sql
tq> SELECT * FROM DBC.
```

Press **Tab** (after the period).

**Expected outcome:**
- Completion shows tables in DBC database only
- List includes: DatabasesV, TablesV, ColumnsV, IndicesV, etc.
- NO database names (already specified)
- NO keywords
- Context correctly identified as qualified table name

**Step 2: Complete a table in qualified context**

With completion menu showing DBC tables, select "TablesV", press Enter/Tab.

**Expected outcome:**
- "TablesV" is inserted after "DBC."
- Text NOT at line beginning
- Cursor after "TablesV"
- Line reads: "SELECT * FROM DBC.TablesV"

**Verification:**
```sql
tq> SELECT * FROM DBC.TablesV
```

NOT:
```sql
TablesVtq> SELECT * FROM DBC.
```

**Step 3: Qualified completion in JOIN clause**

Type:
```sql
tq> SELECT * FROM DBC.DatabasesV JOIN DBC.
```

Press **Tab** (after second "DBC.").

**Expected outcome:**
- Completion shows DBC tables again
- Context correctly identified as JOIN table
- NO keywords

**Step 4: Complete JOIN table**

Select "TablesV" from suggestions, complete.

**Expected outcome:**
- "TablesV" inserted at cursor (after "JOIN DBC.")
- Position correct
- Can continue with ON clause

**Verification:**
```sql
tq> SELECT * FROM DBC.DatabasesV JOIN DBC.TablesV
```

**Step 5: Execute completed query**

Add ON clause and execute:
```sql
tq> SELECT * FROM DBC.DatabasesV d JOIN DBC.TablesV t ON d.DatabaseName = t.DatabaseName LIMIT 5;
```

**Expected outcome:**
- Query executes successfully
- Results display (5 rows)
- Proves both qualified completions provided valid table names

**Step 6: Test partial qualified match**

Type:
```sql
tq> SELECT * FROM DBC.Tab
```

Press **Tab** (after "Tab").

**Expected outcome:**
- Completion filters to DBC tables starting with "Tab"
- Shows: TablesV, TableTextV, etc.
- Only matching tables in DBC
- Inserts remaining characters at cursor

**Step 7: Test user database qualified completion**

If user has access to another database (e.g., "mydb"):
```sql
tq> SELECT * FROM mydb.
```

Press **Tab**.

**Expected outcome:**
- Completion shows tables in "mydb" database
- NOT tables from DBC or other databases
- Database filtering works correctly

### Verification

**Content Validation:**

For "FROM DBC." completion:

✅ MUST contain:
- Tables in DBC only (DatabasesV, TablesV, ColumnsV, etc.)

❌ MUST NOT contain:
- Tables from other databases
- Database names
- SQL keywords
- "(SQL keyword)" text

**Position Validation:**

Critical test - span calculation must be correct:

Before Tab:
```
SELECT * FROM DBC.█
```

After completing "TablesV":
```
SELECT * FROM DBC.TablesV█
```

Cursor position: end of "TablesV", NOT at position 0.

**Database Filtering:**

Completion MUST filter by database:
- "DBC." → only DBC tables
- "mydb." → only mydb tables
- Different databases have different tables

### Cleanup

```bash
# Exit REPL
/quit
```

## Expected Results

**Qualified Completion Suggestions:**

For "SELECT * FROM DBC." + Tab:
```
[Tables in DBC database]
Accounts
All
Checks
ColumnsV
DatabasesV
FunctionsV
IndicesV
TablesV
TableTextV
...
```

**Text Insertion (Critical Fix):**

The core bug fix: text must insert at cursor, not line beginning.

Correct:
```
SELECT * FROM DBC.TablesV█
                       ↑ cursor here
```

Incorrect (bug):
```
TablesVSELECT * FROM DBC.█
↑ wrongly at position 0
```

**Multi-Context:**

Qualified completion should work in:
- FROM clause: `FROM database.`
- JOIN clause: `JOIN database.`
- UPDATE clause: `UPDATE database.` (future)

## Pass/Fail Criteria

**PASS:**
- ✅ Qualified completion (database.) shows correct tables
- ✅ Completion filters by specified database
- ✅ Works in both FROM and JOIN contexts
- ✅ Text inserted at cursor position (correct span)
- ✅ NO keyword suggestions appear
- ✅ Partial matching works for qualified names
- ✅ Completed names are valid and queryable

**FAIL:**
- ❌ Completion shows wrong tables (from different database)
- ❌ Completion includes database names after qualification
- ❌ Text inserted at beginning of line (span broken)
- ❌ Keywords appear in suggestions
- ❌ Completed names not queryable
- ❌ Database filtering doesn't work

## Actual Results

**Status:** [PENDING / PASS / FAIL]

**Test Execution Date:** [Date]
**Tester:** [Name]
**Environment:** [OS, Database]

**Observations:**

**Step 1 - FROM DBC. completion:**
[Document tables shown - should be DBC tables only]

**Step 2 - Text insertion:**
[Document where "TablesV" was inserted]

**Step 3 - JOIN DBC. completion:**
[Document tables shown]

**Step 4 - JOIN completion insertion:**
[Document position]

**Step 5 - Query execution:**
[Document if query succeeded]

**Step 6 - Partial match (DBC.Tab):**
[Document filtered results]

**Step 7 - User database completion:**
[Document if different database filtering worked]

**Database Filtering Check:**
- [ ] DBC. showed only DBC tables
- [ ] Other database showed only its tables
- [ ] No cross-contamination between databases

**Anti-Pattern Check:**
- [ ] No keywords in suggestions
- [ ] No database names after qualification
- [ ] Text inserted at cursor (not line start)
- [ ] All completed names queryable

**Verdict:**
[PASS/FAIL with explanation]

## Notes

**Bug Context:**
- Qualified name completion is critical for database exploration
- Span calculation bug affects this feature most severely
- Users rely on "database." completion to explore unfamiliar databases

**Implementation Notes:**
The completer must:
1. Parse "database." prefix
2. Query metadata for that specific database's tables
3. Calculate span from period to cursor
4. Insert completion at correct position

**Related Files:**
- `src/commands/repl/metadata_completer.rs` - qualified name logic
- `src/commands/repl/sql_context.rs` - database extraction

**Acceptance Criteria Reference:**
From sprint-18-planning.md:
- Tab completion for table names after FROM/JOIN works correctly
- Text inserted at CORRECT cursor position (not beginning of line)
- Span calculation fixed and tested
- All 3 completion contexts work in isolation

**Test Type:** Manual + Interactive (automated test with expectrl)

**Dependencies:** Live database with multiple databases accessible
