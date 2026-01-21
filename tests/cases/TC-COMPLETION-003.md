---
id: TC-COMPLETION-003
title: Tab Completion - Column Names in SELECT and WHERE
category: Functionality
priority: Critical
sprint: 18
bug: Tab completion inserts text at wrong position, keyword completion interfering
created: 2026-01-21
updated: 2026-01-21
status: PENDING
---

# Test Case TC-COMPLETION-003: Tab Completion - Column Names in SELECT and WHERE

## Purpose

Verify that pressing Tab in SELECT clause or WHERE clause shows column names from the table context, and that completion inserts at the correct cursor position without keyword interference.

## Scope

**Testing:**
- Tab completion in SELECT clause shows columns
- Tab completion in WHERE clause shows columns
- Column suggestions based on table context (FROM clause)
- NO keyword suggestions appear
- Text inserted at correct cursor position

**Not Testing:**
- Table/database completion (covered in TC-COMPLETION-001, TC-COMPLETION-002)
- JOIN column completion (covered in TC-COMPLETION-004)
- Complex multi-table contexts

## Prerequisites

- tq binary built in release mode: `cargo build --release`
- Live Teradata database connection
- TQ_LOGON configured in .env or environment variable
- Access to DBC.DatabasesV (known table with predictable columns)

## Test Procedure

### Setup

```bash
# Build release binary
cargo build --release

# Start tq REPL
./target/release/tq
```

### Execution Steps

**Step 1: Column completion in SELECT clause**

Type the following WITHOUT pressing Enter:
```sql
tq> SELECT FROM DBC.DatabasesV
```

Move cursor back to position after "SELECT " (before "FROM"), then press **Tab**.

**Expected outcome:**
- Completion shows column names from DBC.DatabasesV
- List includes: DatabaseName, CreatorName, OwnerName, etc.
- NO SQL keywords appear
- NO "(SQL keyword)" placeholders

**Step 2: Complete a column name**

With completion menu visible, select "DatabaseName" using arrow keys, press Enter/Tab.

**Expected outcome:**
- "DatabaseName" is inserted at cursor position (after "SELECT ")
- Text NOT inserted at line beginning
- Cursor moves after inserted text
- Can continue typing (add comma, more columns)

**Verification:**
```sql
tq> SELECT DatabaseName FROM DBC.DatabasesV
```

NOT:
```sql
DatabaseNametq> SELECT FROM DBC.DatabasesV
```

**Step 3: Column completion in WHERE clause**

Type:
```sql
tq> SELECT * FROM DBC.DatabasesV WHERE
```

Press **Tab** (after "WHERE ").

**Expected outcome:**
- Completion shows column names (same as Step 1)
- NO keywords ("AND", "OR", "IN", etc.)
- Context correctly detected as column context

**Step 4: Select and complete column in WHERE**

Select "DatabaseName" from completion, complete.

**Expected outcome:**
- "DatabaseName" inserted at cursor (after "WHERE ")
- Cursor position correct
- Can continue typing condition (e.g., "= 'DBC'")

**Verification:**
```sql
tq> SELECT * FROM DBC.DatabasesV WHERE DatabaseName
```

**Step 5: Execute completed query**

Complete the WHERE clause and execute:
```sql
tq> SELECT * FROM DBC.DatabasesV WHERE DatabaseName = 'DBC';
```

**Expected outcome:**
- Query executes successfully
- Results display (DBC database row)
- Proves completion provided valid column name

**Step 6: Test partial match in SELECT**

Type:
```sql
tq> SELECT Data FROM DBC.DatabasesV
```

Move cursor to after "Data", press **Tab**.

**Expected outcome:**
- Completion filters to columns starting with "Data" (DatabaseName)
- Only matching columns shown
- Completion inserts remaining characters

### Verification

**Content Validation:**

For "SELECT [Tab] FROM DBC.DatabasesV" completion:

✅ MUST contain:
- DatabaseName
- CreatorName
- OwnerName
- PermSpace
- Other columns from DBC.DatabasesV

❌ MUST NOT contain:
- SQL keywords (SELECT, FROM, WHERE, JOIN, etc.)
- Table names
- Database names
- "(SQL keyword)" text

**Position Validation:**

After completing "DatabaseName" following "SELECT ":
```
SELECT DatabaseName FROM DBC.DatabasesV
```

Cursor after "DatabaseName", not at position 0.

**Context Awareness:**

Column suggestions must match the table in FROM clause:
- If FROM DBC.DatabasesV → show DatabasesV columns
- If FROM DBC.TablesV → show TablesV columns
- If no FROM clause → no column suggestions (or error)

### Cleanup

```bash
# Exit REPL
/quit
```

## Expected Results

**SELECT Clause Completion:**

Before Tab:
```
tq> SELECT █FROM DBC.DatabasesV
```

Suggestions:
```
[Columns from DBC.DatabasesV]
DatabaseName
CreatorName
OwnerName
PermSpace
JournalFlag
...
```

After completing "DatabaseName":
```
tq> SELECT DatabaseName█FROM DBC.DatabasesV
```

**WHERE Clause Completion:**

Before Tab:
```
tq> SELECT * FROM DBC.DatabasesV WHERE █
```

Suggestions:
```
[Same columns]
DatabaseName
CreatorName
OwnerName
...
```

After completion:
```
tq> SELECT * FROM DBC.DatabasesV WHERE DatabaseName█
```

**Text Insertion (Critical):**

Cursor position MUST be at insertion point, NOT at line beginning.

## Pass/Fail Criteria

**PASS:**
- ✅ Tab in SELECT clause shows columns from FROM table
- ✅ Tab in WHERE clause shows columns from FROM table
- ✅ NO keyword suggestions in either context
- ✅ Text inserted at cursor position (correct span)
- ✅ Completed column names are valid (query executes)
- ✅ Partial matching filters columns correctly
- ✅ Context detection works (recognizes SELECT/WHERE contexts)

**FAIL:**
- ❌ Completion shows keywords instead of columns
- ❌ Completion shows columns from wrong table
- ❌ Text inserted at beginning of line (span broken)
- ❌ Keyword suggestions appear
- ❌ Completed column names invalid (query fails)
- ❌ No suggestions in SELECT/WHERE (context detection broken)

## Actual Results

**Status:** [PENDING / PASS / FAIL]

**Test Execution Date:** [Date]
**Tester:** [Name]
**Environment:** [OS, Database]

**Observations:**

**Step 1 - SELECT completion:**
[Document column suggestions shown]

**Step 2 - Text insertion in SELECT:**
[Document where "DatabaseName" was inserted]

**Step 3 - WHERE completion:**
[Document column suggestions shown]

**Step 4 - Text insertion in WHERE:**
[Document where text was inserted]

**Step 5 - Query execution:**
[Document if query succeeded]

**Step 6 - Partial match:**
[Document filtered results for "Data"]

**Anti-Pattern Check:**
- [ ] No keywords in suggestions
- [ ] Columns match FROM table
- [ ] Text inserted at cursor (not line start)
- [ ] All completed columns valid

**Context Detection:**
- [ ] SELECT context recognized
- [ ] WHERE context recognized
- [ ] Correct columns for table context

**Verdict:**
[PASS/FAIL with explanation]

## Notes

**Bug Context:**
- Column completion context detection may be affected by same span calculation bug
- Need to verify context analysis still works correctly
- Critical: cursor position for completion insertion

**Context Detection Requirements:**
1. Parse SQL to find FROM clause
2. Extract table name (qualified or unqualified)
3. Query metadata for that table's columns
4. Detect if cursor is in SELECT or WHERE clause
5. Provide column suggestions

**Related Files:**
- `src/commands/repl/metadata_completer.rs` - completion logic
- `src/commands/repl/sql_context.rs` - context analysis (critical for this test)

**Acceptance Criteria Reference:**
From sprint-18-planning.md:
- Tab completion for column names in SELECT/WHERE works correctly
- NO keyword completion (dropped completely for now)
- Text inserted at CORRECT cursor position (not beginning of line)
- All 3 completion contexts work in isolation

**Test Type:** Manual + Interactive (automated test with expectrl)

**Dependencies:** Live database with DBC.DatabasesV access
