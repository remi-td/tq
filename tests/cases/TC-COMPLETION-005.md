---
id: TC-COMPLETION-005
title: Tab Completion - Verify NO Keyword Completion
category: Functionality
priority: Critical
sprint: 18
bug: Keyword completion interfering with metadata completion
created: 2026-01-21
updated: 2026-01-21
status: PENDING
---

# Test Case TC-COMPLETION-005: Tab Completion - Verify NO Keyword Completion

## Purpose

Verify that keyword completion has been completely removed from tab completion, and that ONLY metadata (databases, tables, columns) appears in completion suggestions across all contexts.

This is an anti-pattern test: we're explicitly testing for the ABSENCE of a feature that was causing problems.

## Scope

**Testing:**
- NO keyword suggestions in FROM context
- NO keyword suggestions in JOIN context
- NO keyword suggestions in SELECT context
- NO keyword suggestions in WHERE context
- NO keyword suggestions in any other context
- Only metadata (databases, tables, columns) appears

**Not Testing:**
- Whether metadata completion works (covered in other tests)
- Keyword syntax highlighting (not related to completion)

## Prerequisites

- tq binary built in release mode: `cargo build --release`
- Live Teradata database connection
- TQ_LOGON configured in .env or environment variable

## Test Procedure

### Setup

```bash
# Build release binary
cargo build --release

# Start tq REPL
./target/release/tq
```

### Execution Steps

**Step 1: FROM context - verify no keywords**

Type:
```sql
tq> SELECT * FROM
```

Press **Tab**.

**Expected outcome:**
- Suggestions appear (databases and tables)
- NO SQL keywords in list
- Specifically check for absence of: SELECT, FROM, WHERE, JOIN, INSERT, UPDATE, DELETE, etc.
- NO "(SQL keyword)" placeholder text

**Anti-Pattern Check:**
❌ Should NOT see:
- "SELECT"
- "FROM"
- "WHERE"
- "JOIN"
- "INSERT"
- "UPDATE"
- "DELETE"
- "CREATE"
- "DROP"
- "(SQL keyword)"

**Step 2: JOIN context - verify no keywords**

Type:
```sql
tq> SELECT * FROM DBC.DatabasesV JOIN
```

Press **Tab** (after "JOIN ").

**Expected outcome:**
- Suggestions appear (databases and tables)
- NO keywords in list

**Step 3: SELECT context - verify no keywords**

Type:
```sql
tq> SELECT FROM DBC.DatabasesV
```

Move cursor to after "SELECT ", press **Tab**.

**Expected outcome:**
- Suggestions appear (column names)
- NO keywords like "DISTINCT", "TOP", "ALL"

**Step 4: WHERE context - verify no keywords**

Type:
```sql
tq> SELECT * FROM DBC.DatabasesV WHERE
```

Press **Tab** (after "WHERE ").

**Expected outcome:**
- Suggestions appear (column names)
- NO keywords like "AND", "OR", "IN", "EXISTS", "BETWEEN"

**Step 5: Beginning of line - verify no keywords**

Clear line, then press **Tab** (with empty line).

**Expected outcome:**
- Either no suggestions appear
- Or only commands/metacommands (like /help, /quit)
- NO SQL keywords (SELECT, UPDATE, etc.)

**Step 6: Middle of statement - verify no keywords**

Type:
```sql
tq> SELECT * FROM DBC.DatabasesV
```

Move cursor to middle of "DatabasesV", press **Tab**.

**Expected outcome:**
- Either no suggestions (not a valid completion point)
- Or only relevant metadata
- NO keywords

**Step 7: Scan all completion contexts**

For each valid completion context, verify NO keywords appear:

| Context | SQL Pattern | Tab After | Expected Suggestions | Must NOT Contain |
|---------|-------------|-----------|---------------------|------------------|
| FROM | `SELECT * FROM ` | FROM | Databases, Tables | Keywords |
| JOIN | `FROM t1 JOIN ` | JOIN | Databases, Tables | Keywords |
| SELECT | `SELECT  FROM t1` | SELECT | Columns | Keywords |
| WHERE | `WHERE ` | WHERE | Columns | Keywords |
| UPDATE | `UPDATE ` | UPDATE | Databases, Tables | Keywords |

### Verification

**Explicit Anti-Pattern Detection:**

For EVERY completion context tested:

✅ Suggestions MAY contain:
- Database names (DBC, mydb, etc.)
- Table names (TablesV, DatabasesV, etc.)
- Column names (DatabaseName, TableName, etc.)

❌ Suggestions MUST NOT contain:
- SQL keywords (SELECT, FROM, WHERE, JOIN, INSERT, UPDATE, DELETE, etc.)
- Command keywords (COMMIT, ROLLBACK, GRANT, REVOKE, etc.)
- DDL keywords (CREATE, ALTER, DROP, etc.)
- DML keywords (INSERT, UPDATE, DELETE, MERGE, etc.)
- Function keywords (COUNT, SUM, AVG, MAX, MIN, etc.)
- Clause keywords (DISTINCT, TOP, LIMIT, OFFSET, etc.)
- Operator keywords (AND, OR, NOT, IN, EXISTS, BETWEEN, LIKE, etc.)
- Generic placeholder text like "(SQL keyword)"

**Content Type Validation:**

Every completion suggestion must be one of:
1. Database name (queryable in DBC.DatabasesV)
2. Table name (queryable in DBC.TablesV)
3. Column name (queryable in DBC.ColumnsV)
4. Metacommand (starting with /)

NOT:
5. SQL keyword
6. Placeholder text
7. Random strings

### Cleanup

```bash
# Exit REPL
/quit
```

## Expected Results

**Example - FROM Context:**

Before Tab:
```
tq> SELECT * FROM █
```

After Tab - suggestions:
```
DBC
my_database
my_table
user_table
...
```

Should NOT see:
```
SELECT          ← NO KEYWORDS
FROM            ← NO KEYWORDS
WHERE           ← NO KEYWORDS
JOIN            ← NO KEYWORDS
DBC
my_database
...
```

**Example - WHERE Context:**

Before Tab:
```
tq> SELECT * FROM DBC.DatabasesV WHERE █
```

After Tab - suggestions:
```
DatabaseName
CreatorName
OwnerName
PermSpace
...
```

Should NOT see:
```
AND             ← NO KEYWORDS
OR              ← NO KEYWORDS
IN              ← NO KEYWORDS
EXISTS          ← NO KEYWORDS
DatabaseName
...
```

## Pass/Fail Criteria

**PASS:**
- ✅ NO keywords appear in FROM context
- ✅ NO keywords appear in JOIN context
- ✅ NO keywords appear in SELECT context
- ✅ NO keywords appear in WHERE context
- ✅ NO keywords appear in any tested context
- ✅ Only metadata (databases, tables, columns) appears
- ✅ No "(SQL keyword)" placeholder text

**FAIL:**
- ❌ ANY SQL keyword appears in completion suggestions
- ❌ "(SQL keyword)" text appears
- ❌ Generic keyword placeholders appear
- ❌ Keyword completion still present in any context

## Actual Results

**Status:** [PENDING / PASS / FAIL]

**Test Execution Date:** [Date]
**Tester:** [Name]
**Environment:** [OS, Database]

**Observations:**

**Step 1 - FROM context:**
[List ALL suggestions seen, check for keywords]

**Step 2 - JOIN context:**
[List ALL suggestions seen, check for keywords]

**Step 3 - SELECT context:**
[List ALL suggestions seen, check for keywords]

**Step 4 - WHERE context:**
[List ALL suggestions seen, check for keywords]

**Step 5 - Empty line:**
[List suggestions if any]

**Step 6 - Mid-statement:**
[List suggestions if any]

**Keyword Scan Results:**

| Context | Saw Keywords? | Which Keywords? | Pass/Fail |
|---------|---------------|-----------------|-----------|
| FROM    | Yes/No        | [list if any]   | PASS/FAIL |
| JOIN    | Yes/No        | [list if any]   | PASS/FAIL |
| SELECT  | Yes/No        | [list if any]   | PASS/FAIL |
| WHERE   | Yes/No        | [list if any]   | PASS/FAIL |

**Anti-Pattern Detection:**
- [ ] No "SELECT" keyword seen
- [ ] No "FROM" keyword seen
- [ ] No "WHERE" keyword seen
- [ ] No "JOIN" keyword seen
- [ ] No "AND"/"OR" keywords seen
- [ ] No "(SQL keyword)" text seen
- [ ] ALL suggestions are metadata objects

**Verdict:**
[PASS/FAIL with explanation]

If FAIL: List exact keywords that appeared and in which context.

## Notes

**Bug Context:**
- User explicitly requested: "Drop the reserved keywords completion"
- Keywords were interfering with metadata completion
- User said: "FOCUS ON database and tablenames after FROM/JOIN"

**Why This Test Exists:**
This is a regression prevention test. Keyword completion was removed as a feature because:
1. It interfered with metadata completion
2. Context detection couldn't reliably distinguish when to show keywords vs metadata
3. Users don't need keyword suggestions (IDEs handle this)
4. Simpler implementation = fewer bugs

**Acceptance Criteria Reference:**
From sprint-18-planning.md:
- NO keyword completion (dropped completely for now)
- Focus on metadata completion only (databases, tables, columns)

**Related Files:**
- `src/commands/repl/metadata_completer.rs` - should have NO keyword logic
- `src/commands/repl/sql_context.rs` - context analysis (should not have keyword context)

**Critical Success Factor:**
If even ONE keyword appears in completion suggestions, this test FAILS.

This is a zero-tolerance test: keywords were causing bugs, so they must be completely absent.

**Test Type:** Manual + Interactive (automated test with expectrl)

**Dependencies:** Live database for metadata queries
