---
id: TC-COMPLETION-001
title: Tab Completion - Database Names After FROM
category: Functionality
priority: Critical
sprint: 18
bug: Tab completion inserts text at wrong position, keyword completion interfering
created: 2026-01-21
updated: 2026-01-21
status: PENDING
---

# Test Case TC-COMPLETION-001: Tab Completion - Database Names After FROM

## Purpose

Verify that pressing Tab after "SELECT * FROM " shows database names (and current database tables) as completion suggestions, and that selected completion inserts at the correct cursor position (not beginning of line).

## Scope

**Testing:**
- Tab completion triggers after FROM keyword
- Completion shows database names
- Completion shows tables in current database
- NO keyword suggestions appear
- Selected completion inserts at cursor position (not line start)

**Not Testing:**
- Column completion (covered in TC-COMPLETION-003)
- Qualified name completion (covered in TC-COMPLETION-004)
- Other SQL keywords (JOIN, UPDATE covered in separate tests)

## Prerequisites

- tq binary built in release mode: `cargo build --release`
- Live Teradata database connection with queryable databases
- TQ_LOGON configured in .env or environment variable
- Test database should have at least 2-3 databases visible (e.g., DBC, USER's database)

## Test Procedure

### Setup

```bash
# Build release binary
cargo build --release

# Start tq REPL
./target/release/tq
```

### Execution Steps

**Step 1: Basic FROM completion**

Type the following WITHOUT pressing Enter:
```sql
tq> SELECT * FROM
```

Now press **Tab**.

**Expected outcome:**
- Completion suggestions appear
- List contains database names (e.g., "DBC", user's database)
- List contains tables in current database
- NO SQL keywords appear (no "SELECT", "FROM", "WHERE", etc.)
- No "(SQL keyword)" placeholder text

**Step 2: Select a completion**

With completion menu visible, use arrow keys to select a database name (e.g., "DBC"), then press Enter or Tab to complete.

**Expected outcome:**
- Selected database name is inserted into the command
- Text appears at cursor position (after "FROM ")
- Text does NOT appear at beginning of line
- Cursor moves to end of inserted text
- Can continue typing (e.g., add period for qualified name)

**Verification:**
```sql
tq> SELECT * FROM DBC
```

NOT:
```sql
DBCtq> SELECT * FROM
```

**Step 3: Complete with table from current database**

Clear the line (Ctrl+U), then type:
```sql
tq> SELECT * FROM
```

Press Tab, select a table name from current database, complete.

**Expected outcome:**
- Table name inserted at correct position
- Can execute the query successfully
- Text insertion position is correct

**Step 4: Test with partial match**

Type:
```sql
tq> SELECT * FROM DB
```

Press Tab.

**Expected outcome:**
- Completion filters to names starting with "DB" (e.g., "DBC")
- Only relevant matches shown
- Completion still inserts at cursor position

### Verification

**Content Validation:**

✅ Completion list MUST contain:
- Database names visible to user
- Table names in current database
- Actual queryable object names

❌ Completion list MUST NOT contain:
- SQL keywords ("SELECT", "FROM", "WHERE", "JOIN", etc.)
- Generic placeholders ("(SQL keyword)")
- Non-database objects
- Random text

**Position Validation:**

After completing "DBC" following "SELECT * FROM ", the line should be:
```
SELECT * FROM DBC
```

Cursor should be after "DBC".

**Anti-Pattern Detection:**

The following should NOT happen:
- ✗ Completed text appears at beginning of line
- ✗ Original typed text is deleted
- ✗ Cursor position is wrong
- ✗ Keyword suggestions appear
- ✗ Completion menu shows "(SQL keyword)" repeated

### Cleanup

```bash
# Exit REPL
/quit
```

## Expected Results

**Completion Suggestions (Example):**

After "SELECT * FROM " + Tab:
```
[Completion Menu]
DBC
my_database
my_table_1
my_table_2
...
```

**Text Insertion:**

Before Tab:
```
tq> SELECT * FROM █
```

After selecting "DBC":
```
tq> SELECT * FROM DBC█
```

Where █ represents cursor position.

**Query Execution:**

After completion, should be able to execute:
```sql
tq> SELECT * FROM DBC.DatabasesV LIMIT 5;
```

And get valid results (proving completion provided correct database name).

## Pass/Fail Criteria

**PASS:**
- ✅ Tab after FROM shows databases and current DB tables
- ✅ NO keyword suggestions appear
- ✅ Selected completion inserts at cursor position (after "FROM ")
- ✅ Cursor position correct after completion
- ✅ Completed names are valid and queryable
- ✅ Partial matching works (filters suggestions)

**FAIL:**
- ❌ Tab shows keywords instead of databases
- ❌ Completion inserts at beginning of line (position 0)
- ❌ Cursor position wrong after completion
- ❌ Keyword suggestions appear in list
- ❌ Completion shows generic placeholders
- ❌ Completed names not queryable

## Actual Results

**Status:** [PENDING / PASS / FAIL]

**Test Execution Date:** [Date]
**Tester:** [Name]
**Environment:** [OS, Database]

**Observations:**

**Step 1 - Basic FROM completion:**
[Document what suggestions appeared]

**Step 2 - Text insertion position:**
[Document where text was inserted]

**Step 3 - Table completion:**
[Document results]

**Step 4 - Partial match:**
[Document results]

**Anti-Pattern Check:**
- [ ] No keywords in suggestions
- [ ] No "(SQL keyword)" text
- [ ] Text inserted at cursor (not line start)
- [ ] Cursor position correct

**Verdict:**
[PASS/FAIL with explanation]

## Notes

**Bug Context:**
- User reported: "Press tab and then start typing keyword, text appears. Press enter, it's inserted at beginning of line"
- Root cause: Span calculation is wrong (text inserted at position 0 instead of cursor position)
- Keyword completion is interfering with contextual completion

**User Expectation:**
"This worked like 10 sprints ago" - need to restore working state

**Related Files:**
- `src/commands/repl/metadata_completer.rs` - main completer logic
- `src/commands/repl/sql_context.rs` - context analysis

**Acceptance Criteria Reference:**
From sprint-18-planning.md:
- Tab completion for database names after FROM/JOIN works correctly
- Tab completion for table names after FROM/JOIN works correctly
- NO keyword completion (dropped completely for now)
- Text inserted at CORRECT cursor position (not beginning of line)
- Span calculation fixed and tested

**Test Type:** Manual + Interactive (automated test with expectrl)

**Dependencies:** Live database with queryable databases and tables
