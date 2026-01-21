---
id: TC-COMPLETION-002
title: Tab Completion - Table Names After FROM
category: Functionality
priority: Critical
sprint: 18
bug: Tab completion inserts text at wrong position, keyword completion interfering
created: 2026-01-21
updated: 2026-01-21
status: PENDING
---

# Test Case TC-COMPLETION-002: Tab Completion - Table Names After FROM

## Purpose

Verify that pressing Tab after "SELECT * FROM " with context awareness shows appropriate table names, and that completion works correctly for both unqualified and qualified (database.table) patterns.

## Scope

**Testing:**
- Tab completion shows tables in current database
- Tab completion filters to specific database when qualified (e.g., "FROM DBC.")
- NO keyword suggestions appear
- Text inserted at correct cursor position
- Both unqualified and qualified name completion works

**Not Testing:**
- Database name completion (covered in TC-COMPLETION-001)
- Column completion (covered in TC-COMPLETION-003)
- Other SQL contexts (JOIN covered in separate tests)

## Prerequisites

- tq binary built in release mode: `cargo build --release`
- Live Teradata database connection
- TQ_LOGON configured in .env or environment variable
- Test should use DBC database (always available) for predictable results
- User should have permissions to query DBC.TablesV

## Test Procedure

### Setup

```bash
# Build release binary
cargo build --release

# Start tq REPL
./target/release/tq
```

### Execution Steps

**Step 1: Unqualified table completion**

Type the following WITHOUT pressing Enter:
```sql
tq> SELECT * FROM
```

Press **Tab**.

**Expected outcome:**
- Completion shows tables in current database
- Completion shows database names (for qualification)
- NO keyword suggestions

**Step 2: Qualified table completion - with database prefix**

Type:
```sql
tq> SELECT * FROM DBC.
```

Press **Tab** (after the period).

**Expected outcome:**
- Completion shows ONLY tables in DBC database
- List includes system tables (DatabasesV, TablesV, ColumnsV, etc.)
- NO database names appear (already qualified)
- NO keywords appear

**Step 3: Select and complete a table**

With completion menu showing DBC tables, select "TablesV" using arrow keys, then press Enter/Tab.

**Expected outcome:**
- "TablesV" is inserted after "DBC."
- Cursor position after "TablesV"
- Line reads: "SELECT * FROM DBC.TablesV"
- Text NOT inserted at line beginning

**Verification:**
```sql
tq> SELECT * FROM DBC.TablesV
```

NOT:
```sql
TablesVtq> SELECT * FROM DBC.
```

**Step 4: Execute completed query**

Add a LIMIT clause and execute:
```sql
tq> SELECT * FROM DBC.TablesV LIMIT 3;
```

**Expected outcome:**
- Query executes successfully
- Results display (3 rows from DBC.TablesV)
- Proves completion provided valid table name

**Step 5: Test partial match with qualification**

Type:
```sql
tq> SELECT * FROM DBC.Tab
```

Press **Tab**.

**Expected outcome:**
- Completion filters to tables starting with "Tab" (e.g., TablesV, TableTextV)
- Only matching tables shown
- Completion inserts remaining characters at cursor

### Verification

**Content Validation:**

For "FROM DBC." completion:

✅ MUST contain:
- TablesV
- DatabasesV
- ColumnsV
- Other DBC system views

❌ MUST NOT contain:
- SQL keywords
- Database names (already qualified)
- "(SQL keyword)" text
- Non-table objects

**Position Validation:**

After completing "TablesV" following "FROM DBC.", the line should be:
```
SELECT * FROM DBC.TablesV
```

Cursor after "TablesV", not at position 0.

**Queryability Validation:**

Completed table names must be queryable:
```sql
SELECT * FROM [completed_name] LIMIT 1;
```

Should succeed (or fail with "no rows" / permissions, but NOT "table not found").

### Cleanup

```bash
# Exit REPL
/quit
```

## Expected Results

**Unqualified Completion (FROM + Tab):**
```
[Suggestions]
DBC
my_database
table1
table2
...
```

**Qualified Completion (FROM DBC. + Tab):**
```
[Suggestions - DBC tables only]
Accounts
All
Checks
ColumnsV
DatabasesV
FunctionsV
IndicesV
TablesV
...
```

**Text Insertion (Critical Fix):**

Before Tab:
```
tq> SELECT * FROM DBC.█
```

After selecting "TablesV":
```
tq> SELECT * FROM DBC.TablesV█
```

NOT at beginning:
```
TablesVtq> SELECT * FROM DBC.█
```

## Pass/Fail Criteria

**PASS:**
- ✅ Unqualified completion shows tables + databases
- ✅ Qualified completion (database.) shows only tables in that database
- ✅ NO keyword suggestions in any context
- ✅ Text inserted at cursor position (correct span calculation)
- ✅ Completed table names are valid and queryable
- ✅ Partial matching works for qualified names

**FAIL:**
- ❌ Completion shows keywords instead of tables
- ❌ Qualified completion shows wrong tables (from different database)
- ❌ Text inserted at beginning of line (span calculation broken)
- ❌ Keyword suggestions appear
- ❌ Completed names not queryable
- ❌ Qualified completion doesn't filter correctly

## Actual Results

**Status:** [PENDING / PASS / FAIL]

**Test Execution Date:** [Date]
**Tester:** [Name]
**Environment:** [OS, Database]

**Observations:**

**Step 1 - Unqualified completion:**
[Document suggestions shown]

**Step 2 - Qualified completion (DBC.):**
[Document suggestions shown - should be DBC tables only]

**Step 3 - Text insertion:**
[Document where "TablesV" was inserted]

**Step 4 - Query execution:**
[Document if query succeeded]

**Step 5 - Partial match:**
[Document filtered results for "DBC.Tab"]

**Anti-Pattern Check:**
- [ ] No keywords in suggestions
- [ ] Qualified completion filtered to correct database
- [ ] Text inserted at cursor (not line start)
- [ ] All completed names queryable

**Verdict:**
[PASS/FAIL with explanation]

## Notes

**Bug Context:**
- Tab completion previously worked in Sprint 7/8
- Broke in later sprints due to span calculation issues
- Current bug: text inserted at position 0 instead of cursor position

**Critical Behavior:**
The span calculation must correctly identify:
1. Start position: after "FROM " or after "FROM database."
2. End position: current cursor position
3. Replacement text: completion candidate

**Related Files:**
- `src/commands/repl/metadata_completer.rs` - completion logic
- `src/commands/repl/sql_context.rs` - context detection

**Acceptance Criteria Reference:**
From sprint-18-planning.md:
- Tab completion for table names after FROM/JOIN works correctly
- NO keyword completion (dropped completely for now)
- Text inserted at CORRECT cursor position (not beginning of line)
- Span calculation fixed and tested

**Test Type:** Manual + Interactive (automated test with expectrl)

**Dependencies:** Live database with DBC access
