# TC-F2-MANUAL: Enhanced Schema Commands

**Feature:** Enhanced Schema Commands (Sprint 22, Feature 2, P0)
**Test Type:** Manual Validation (OUTPUT FORMATTING)
**Priority:** HIGH
**Author:** quality-validator
**Created:** 2026-01-23
**Sprint:** Sprint 22

---

## Objective

Verify that schema inspection commands (`/list databases`, `/list tables`, `/list views`) display correctly formatted, readable output.

**Note:** Automated integration and PTY tests validate **content correctness** (databases/tables are returned). This manual test validates **output formatting quality** (readability, alignment, user-friendliness).

---

## Prerequisites

- `tq` REPL compiled and runnable
- Live Teradata database connection via `TQ_LOGON` environment variable or `.env` file
- Test database should have:
  - Multiple databases (e.g., `dbc`, `demo_user`, `production`)
  - Multiple tables in at least one database
  - At least one view (optional but recommended)

---

## Test Procedure

### Test 1: List Databases

**Steps:**
1. Start `tq` REPL: `tq repl`
2. Wait for connection confirmation
3. Type: `/list databases`
4. Press: `ENTER`
5. **Observe:** Output shows databases in table format

**Expected Result:**
- Databases displayed in readable table/list format
- Columns: Database name, Owner (optional), Type (System/User)
- System database `dbc` is included
- At minimum: 2-3 databases shown (dbc + user databases)
- Output is well-aligned and easy to read

**Pass Criteria:**
- [ ] Command executes without errors
- [ ] Output is formatted as a table or structured list
- [ ] Database names are clearly visible
- [ ] System database `dbc` is included in results
- [ ] Column alignment is correct (no overlap/misalignment)
- [ ] User can easily identify available databases

**Screenshot Required:** YES

---

### Test 2: List Tables (Current Database)

**Steps:**
1. In the same REPL session
2. Type: `/list tables`
3. Press: `ENTER`
4. **Observe:** Output shows tables in current database

**Expected Result:**
- Tables displayed in readable format
- Columns: Table name, Type, Rows (optional), Size (optional)
- At minimum: Shows table names clearly
- If current database has no tables: "No tables found" message

**Pass Criteria:**
- [ ] Command executes without errors
- [ ] Output is formatted and readable
- [ ] Table names are clearly visible
- [ ] If no tables: Clear message displayed
- [ ] Format is consistent with Test 1

**Screenshot Required:** YES

---

### Test 3: List Tables with Pattern (Glob Filtering)

**Steps:**
1. In the same REPL session
2. Identify a table name from Test 2 (e.g., "orders", "customers")
3. Type: `/list tables <prefix>*` (e.g., `/list tables ord*` if "orders" exists)
4. Press: `ENTER`
5. **Observe:** Output shows only tables matching pattern

**Expected Result:**
- Only matching tables displayed
- Pattern filtering works correctly (glob-style: `*` matches any characters)
- If no match: "No tables found matching 'pattern'" message

**Pass Criteria:**
- [ ] Command executes without errors
- [ ] Only matching tables are shown
- [ ] Non-matching tables are filtered out
- [ ] Clear message if no tables match pattern
- [ ] Pattern syntax is intuitive (glob-style)

**Screenshot Required:** YES

---

### Test 4: List Tables with Qualified Pattern

**Steps:**
1. In the same REPL session
2. Type: `/list tables dbc.t*` (list tables in `dbc` database starting with 't')
3. Press: `ENTER`
4. **Observe:** Output shows tables from `dbc` database matching `t*`

**Expected Result:**
- Tables from `dbc` database shown (not current database)
- Only tables starting with 't' are displayed (e.g., `Tables`, `TablesV`)
- Qualified pattern `database.pattern` works correctly

**Pass Criteria:**
- [ ] Command executes without errors
- [ ] Tables from correct database (`dbc`) are shown
- [ ] Pattern filtering works with qualified names
- [ ] Output clearly indicates which database was queried

**Screenshot Required:** YES

---

### Test 5: List Views

**Steps:**
1. In the same REPL session
2. Type: `/list views`
3. Press: `ENTER`
4. **Observe:** Output shows views in current database

**Expected Result:**
- Views displayed in readable format
- Columns: View name, Owner (optional), Definition (truncated, optional)
- If no views: "No views found" message

**Pass Criteria:**
- [ ] Command executes without errors
- [ ] Output is formatted and readable
- [ ] View names are clearly visible
- [ ] If no views: Clear message displayed
- [ ] Format is consistent with previous tests

**Screenshot Required:** YES

---

### Test 6: Output Formatting Quality

**Overall Assessment Across All Tests:**

**Evaluate the following:**
- **Column Alignment:** Are columns properly aligned? No overlap?
- **Readability:** Can user quickly scan and understand output?
- **Consistency:** Do all commands use similar formatting?
- **Borders/Separators:** Are table borders clear (if used)?
- **Whitespace:** Is whitespace used effectively for clarity?

**Pass Criteria:**
- [ ] Column alignment is correct across all outputs
- [ ] Output is readable and easy to scan
- [ ] Formatting is consistent between commands
- [ ] User-friendly presentation (not raw data dump)
- [ ] Borders/separators enhance readability (if used)

---

### Test 7: Error Handling (Optional)

**Steps:**
1. Type: `/list tables restricted_db.*` (use a database with no access)
2. Press: `ENTER`
3. **Observe:** Error message is displayed

**Expected Result:**
- Clear error message: "Access denied" or "Insufficient privileges"
- Error message is user-friendly (not raw SQL error)
- REPL remains functional after error

**Pass Criteria:**
- [ ] Error message is clear and actionable
- [ ] No raw SQL error or stack trace shown
- [ ] REPL continues to work after error

**Screenshot Required:** OPTIONAL

---

## Evidence Collection

**Required Evidence:**
- [ ] Screenshot: `/list databases` output
- [ ] Screenshot: `/list tables` output
- [ ] Screenshot: `/list tables <pattern>*` output (filtered)
- [ ] Screenshot: `/list tables dbc.t*` output (qualified pattern)
- [ ] Screenshot: `/list views` output
- [ ] User confirmation: "Output formatting is clear and readable"

**How to Collect:**
- **Screenshots:** Capture terminal output for each command
- **Written Notes:** Record any formatting issues or suggestions

---

## Acceptance Criteria Summary

✅ **PASS** if ALL of the following are true:
- [ ] `/list databases` shows databases with proper formatting
- [ ] `/list tables` shows tables in current database
- [ ] `/list tables <pattern>` filters correctly by glob pattern
- [ ] `/list tables dbc.t*` filters with qualified database name
- [ ] `/list views` shows views
- [ ] Output formatting is readable and user-friendly
- [ ] Column alignment is correct
- [ ] Error messages (if tested) are clear

❌ **FAIL** if ANY of the following occur:
- Output is unreadable or poorly formatted
- Column alignment is broken (text overlaps)
- Pattern filtering doesn't work
- Error messages are confusing or show raw SQL
- User feedback: "Output is hard to read"

---

## Notes

**Automation vs. Manual:**
- **Automated tests** (integration + PTY): Validate content correctness (right databases/tables returned)
- **Manual test** (this): Validates output formatting quality and user experience

**Priority:** HIGH (but not MANDATORY for APPROVED verdict)
If automated tests pass but manual formatting test fails, consider CONDITIONAL APPROVAL with formatting improvement as follow-up.

---

## Related Tests

- **Integration Tests:** `test_list_databases`, `test_list_tables`, `test_list_views` in `tests/integration_tests.rs`
- **PTY Tests:** `test_list_databases_output`, `test_list_tables_output` in `tests/interactive_tests.rs`
- **Specification:** `docs/specifications/repl.md` lines 1132-1394

---

## Test Result

**Date Executed:** _____________
**Tester:** _____________
**Verdict:** [ ] PASS  [ ] FAIL  [ ] CONDITIONAL PASS
**Notes:**

```
[Record formatting quality, any alignment issues, or suggestions]
```

**Evidence Files:**
- Screenshot 1 (/list databases): _____________
- Screenshot 2 (/list tables): _____________
- Screenshot 3 (/list tables pattern): _____________
- Screenshot 4 (/list views): _____________

**Formatting Assessment:**
- Readability (1-5): _____
- Alignment Quality (1-5): _____
- Consistency (1-5): _____
- Overall User-Friendliness (1-5): _____

