# Sprint 8 Manual Testing Guide

**Sprint:** 8 - Critical Bug Fixes
**Date:** 2026-01-18
**Tester:** User (manual execution)
**Commit:** e0ec05a (to be updated with fix commit)

---

## Overview

This guide provides step-by-step instructions for manually testing all 4 Sprint 8 bug fixes. Each bug has clear reproduction steps, expected results, and pass/fail criteria.

**Four Bugs Being Fixed:**
1. **Bug 1:** Table Padding - Columns misaligned with excessive padding
2. **Bug 2:** Tab Completion - Doesn't work at all
3. **Bug 3:** Result Paging - Not integrated, arrow keys don't work
4. **Bug 4:** LIMIT Hint - Says "LIMIT" instead of "TOP/SAMPLE"

---

## Prerequisites

### Environment Setup

1. **Build the fixed binary:**
```bash
cd /Users/remi.turpaud/Code/genAI/tq
cargo build --release
```

2. **Verify database connection:**
```bash
./target/release/tq ping
```

Expected: "Connection successful" or similar.

3. **Start REPL mode:**
```bash
./target/release/tq
```

4. **Check current database:**
```sql
tq> SELECT DATABASE;
```

Note the database name for tests.

### Testing Tools

- Terminal with at least 80 columns width (check with `tput cols`)
- Access to DBC system views (DBC.DatabasesV, DBC.TablesV)
- Permission to CREATE/DROP test tables (for cache invalidation tests)
- Notepad or text file for recording results

---

## Bug 1: Table Padding (TC044-TC048)

### Problem

Table output has excessive padding - columns are 60+ characters wide when data is only 3 characters, making tables unreadable.

### Test 1.1: Basic 5-Column Table (TC044)

**Objective:** Verify proper alignment with 5 columns.

**Steps:**
```sql
tq> SELECT TOP 5 DatabaseName, CreatorName, OwnerName, PermSpace, JournalFlag
    FROM DBC.DatabasesV
    ORDER BY DatabaseName;
```

**Expected:**
- Headers align with data columns below them
- Vertical separators (┆) form straight lines
- Column widths are reasonable (not 60+ chars for 3-char data)
- Text is left-aligned, numbers are right-aligned
- Table is readable

**Pass/Fail:**
- [ ] PASS: Headers and data align, reasonable column widths
- [ ] FAIL: Headers misaligned, excessive padding

---

### Test 1.2: Wide Table with 16 Columns (TC045)

**Objective:** Reproduce and verify fix for original bug.

**Steps:**
```sql
tq> SELECT TOP 3 * FROM DBC.DatabasesV;
```

**Expected:**
- Table displays all 16 columns
- Column widths are appropriate (NOT 60+ chars each)
- Total table width is reasonable (not 960+ characters)
- Headers align with data
- May need horizontal scrolling (acceptable), but columns should be reasonable width

**Measurement Check:**
- Look at "DatabaseName" column width
- Data includes "val", "All", "DBC", "mldb" (max 10 chars)
- Column should be ~12-14 characters wide (header length + padding)
- NOT 60+ characters wide

**Pass/Fail:**
- [ ] PASS: Column widths are appropriate (~12-20 chars), headers align
- [ ] FAIL: Excessive padding (60+ chars), total width 960+ chars

---

### Test 1.3: NULL Values (TC046)

**Objective:** Verify NULL handling doesn't break alignment.

**Steps:**
```sql
tq> SELECT TOP 5 DatabaseName, CommentString, AccessCount, LastAccessTimeStamp
    FROM DBC.DatabasesV;
```

**Expected:**
- NULL values display as "[NULL]"
- "[NULL]" aligns with non-NULL values in same column
- Column widths accommodate both NULL and non-NULL data
- No excessive padding due to NULLs

**Pass/Fail:**
- [ ] PASS: NULLs display correctly and align properly
- [ ] FAIL: NULLs cause misalignment or excessive width

---

### Test 1.4: Mixed Data Types (TC048)

**Objective:** Verify type-specific alignment.

**Steps:**
```sql
tq> SELECT DatabaseName, CreatorName, PermSpace, CreateTimeStamp
    FROM DBC.DatabasesV
    WHERE DatabaseName IN ('DBC', 'All', 'val')
    ORDER BY DatabaseName;
```

**Expected:**
- Text columns (DatabaseName, CreatorName) are left-aligned
- Numeric columns (PermSpace) are right-aligned
- Timestamp columns display consistently
- All alignments are correct

**Visual Check:**
```
│ DBC          ┆ DBC         ┆   1000000 ┆ 2025-10-09 17:27:00 │
 ^left-aligned  ^left-aligned ^right-aligned  ^consistent format
```

**Pass/Fail:**
- [ ] PASS: Text left-aligned, numbers right-aligned, clean layout
- [ ] FAIL: Incorrect alignment, mixed rules

---

## Bug 2: Tab Completion (TC049-TC056)

### Problem

Tab completion doesn't work at all - pressing Tab does nothing, no visual feedback, no metadata loading.

### Test 2.1: FROM Tab Completion (TC049)

**Objective:** Verify Tab shows databases and current DB tables.

**Steps:**
```sql
tq> SELECT * FROM <TAB>
```
(Type `SELECT * FROM ` then press Tab key)

**Expected:**
- Completion menu appears
- Shows database names: All, DBC, TD_SYSXML, val, mldb, etc.
- Shows tables in current database
- Clear labeling distinguishes databases from tables
- Response is fast (<500ms after initial metadata load)

**Pass/Fail:**
- [ ] PASS: Tab shows completion menu with databases and tables
- [ ] FAIL: Tab does nothing (no response) ← ORIGINAL BUG

---

### Test 2.2: Database-Specific Table Completion (TC050)

**Objective:** Verify `FROM database.` shows tables in that database.

**Steps:**
```sql
tq> SELECT * FROM DBC.<TAB>
```
(Type `SELECT * FROM DBC.` then press Tab)

**Expected:**
- Shows tables in DBC database
- Includes: DatabasesV, TablesV, ColumnsV, etc.
- Label indicates "Tables in 'DBC':" or similar
- Different from current database's tables

**Pass/Fail:**
- [ ] PASS: Shows DBC tables specifically
- [ ] FAIL: No response or shows wrong tables

---

### Test 2.3: Loading Indicator (TC051)

**Objective:** Verify loading indicator appears on first Tab.

**Steps:**
```sql
tq> /logon
[Reconnect to clear cache]

tq> SELECT * FROM <TAB>
```
(First Tab press after restart)

**Expected:**
- Loading indicator appears: "Loading tables... ⠋" or similar
- Spinner animation visible (if load takes >200ms)
- Indicator disappears when metadata loaded
- Completion list appears

**Pass/Fail:**
- [ ] PASS: Loading indicator visible, then completion appears
- [ ] FAIL: No loading feedback (silent waiting)

---

### Test 2.4: Error Messages (TC052)

**Objective:** Verify clear error messages on failure.

**Note:** This test may be difficult without restricted permissions. If you have access to all metadata, skip this test or note "Cannot test - full permissions".

**Steps:**
(If you can simulate permission denied or connection loss)

```sql
tq> SELECT * FROM <TAB>
```

**Expected (if metadata query fails):**
- Error message displayed: "Warning: Cannot load table list..."
- Explanation of what went wrong
- Suggestion for resolution
- REPL remains usable

**Pass/Fail:**
- [ ] PASS: Clear error message with suggestions
- [ ] FAIL: Silent failure (no error shown)
- [ ] SKIP: Cannot test (full permissions)

---

### Test 2.5: Cache Invalidation After CREATE TABLE (TC053)

**Objective:** Verify cache refreshes after DDL.

**Steps:**
```sql
tq> SELECT * FROM test_spr<TAB>
[Verify "test_sprint8_tab" NOT in list]

tq> CREATE TABLE test_sprint8_tab (id INTEGER, name VARCHAR(50));

tq> SELECT * FROM test_spr<TAB>
```

**Expected:**
- Before CREATE: "test_sprint8_tab" not in completion
- CREATE executes successfully
- After CREATE: "test_sprint8_tab" appears in completion
- Cache refresh is automatic

**Cleanup:**
```sql
tq> DROP TABLE test_sprint8_tab;
```

**Pass/Fail:**
- [ ] PASS: New table appears immediately in completion after CREATE
- [ ] FAIL: Must restart tq to see new table

---

### Test 2.6: Cache Invalidation After DROP TABLE (TC054)

**Objective:** Verify cache refreshes after DROP.

**Steps:**
```sql
tq> CREATE TABLE test_drop_tab (id INTEGER);

tq> SELECT * FROM test_drop<TAB>
[Verify "test_drop_tab" IS in list]

tq> DROP TABLE test_drop_tab;

tq> SELECT * FROM test_drop<TAB>
```

**Expected:**
- After CREATE: "test_drop_tab" in completion
- After DROP: "test_drop_tab" NOT in completion
- Cache refresh is automatic

**Pass/Fail:**
- [ ] PASS: Dropped table disappears immediately from completion
- [ ] FAIL: Dropped table still appears in completion

---

## Bug 3: Result Paging (TC057-TC063)

### Problem

Result paging doesn't work at all - arrow keys don't work, results just dump to terminal with no interactive paging despite "paging enabled" banner.

### Test 3.1: Vertical Paging with j/k Keys (TC057)

**Objective:** Verify pager activates and j/k keys work.

**Steps:**
```sql
tq> SELECT TOP 100 * FROM DBC.TablesV;
```

**Expected:**
- Pager activates (results don't scroll past terminal)
- Status line appears at bottom: "Lines 1-25 of 100 (0%) | j/k=scroll, q=quit"
- Interactive mode (waiting for user input)
- Press 'j' to scroll down → moves down one line
- Press 'k' to scroll up → moves up one line
- Status line updates with each scroll
- Press 'q' to exit → returns to REPL prompt

**Pass/Fail:**
- [ ] PASS: Pager activates, j/k keys work, can navigate and exit
- [ ] FAIL: No paging (all results dump), j/k don't work ← ORIGINAL BUG

---

### Test 3.2: PageUp/PageDown Keys (TC058)

**Objective:** Verify page-at-a-time navigation.

**Steps:**
```sql
tq> SELECT TOP 200 * FROM DBC.TablesV;
```
(In pager)

**Expected:**
- Press PageDown → scrolls down one full page (20-25 lines)
- Press PageUp → scrolls up one full page
- Status line updates: "Lines 1-25" → "Lines 26-50" → etc.
- Fast navigation through results

**Pass/Fail:**
- [ ] PASS: PageUp/PageDown work for fast scrolling
- [ ] FAIL: Keys don't work or pager not active

---

### Test 3.3: Horizontal Paging with h/l Keys (TC059)

**Objective:** Verify horizontal scrolling for wide tables.

**Steps:**
```sql
tq> SELECT TOP 10 * FROM DBC.DatabasesV;
```
(16 columns - likely wider than terminal)

**Expected:**
- Pager activates
- Shows first N columns that fit in terminal
- Status indicates: "Cols 1-8 of 16" or similar
- Press 'l' → scrolls right (shows more columns)
- Press 'h' → scrolls left (shows earlier columns)
- Can view all columns by scrolling

**Pass/Fail:**
- [ ] PASS: Horizontal paging works, can see all columns with h/l
- [ ] FAIL: No horizontal paging, wide table truncated

---

### Test 3.4: Pager Exit (TC062)

**Objective:** Verify clean exit from pager.

**Steps:**
```sql
tq> SELECT TOP 100 * FROM DBC.TablesV;
```
(In pager)

**Expected:**
- Press 'q' → pager exits immediately
- Returns to REPL prompt: `tq>`
- Terminal restored (not corrupted)
- Can execute next command normally

Also test:
- Press 'Esc' → same result as 'q'

**Pass/Fail:**
- [ ] PASS: Both 'q' and 'Esc' exit cleanly
- [ ] FAIL: Can't exit pager or terminal corrupted

---

### Test 3.5: /pager on and /pager off (TC063)

**Objective:** Verify metacommands control paging.

**Steps:**
```sql
tq> /pager off

tq> SELECT TOP 100 * FROM DBC.TablesV;
```

**Expected with /pager off:**
- All results dump immediately (no paging)
- Results scroll past terminal
- Returns to prompt immediately

```sql
tq> /pager on

tq> SELECT TOP 100 * FROM DBC.TablesV;
```

**Expected with /pager on:**
- Pager activates (interactive mode)
- Can navigate with j/k
- Exit with q

**Pass/Fail:**
- [ ] PASS: /pager off disables paging, /pager on enables it
- [ ] FAIL: /pager off has no effect (paging still occurs)

---

## Bug 4: LIMIT Hint (TC064-TC065)

### Problem

Hint message says "Add LIMIT clause" but Teradata doesn't support LIMIT (uses TOP or SAMPLE instead). Confusing for users.

### Test 4.1: Hint Message Shows TOP/SAMPLE (TC064)

**Objective:** Verify hint uses correct Teradata syntax.

**Steps:**
```sql
tq> SELECT * FROM DBC.TablesV;
```
(Query that returns >100 rows)

**Expected:**
- Query returns first 100 rows
- Hint message appears after results:

**CORRECT:**
```
Showing first 100 rows. Use TOP N or SAMPLE N for different results.
```

**INCORRECT (Bug):**
```
Showing first 100 rows. Add LIMIT clause for different results.
```

**Pass/Fail:**
- [ ] PASS: Hint mentions "TOP N or SAMPLE N", NOT "LIMIT"
- [ ] FAIL: Hint still says "LIMIT" ← ORIGINAL BUG

---

### Test 4.2: Help Text Uses Teradata Syntax (TC065)

**Objective:** Verify help documentation is correct.

**Steps:**
```sql
tq> /help
```

**Expected:**
- Search output for any mentions of "LIMIT"
- If result limiting is mentioned, should use "TOP N" or "SAMPLE N"
- Examples (if provided) use valid Teradata SQL
- NO mentions of "LIMIT clause"

**Pass/Fail:**
- [ ] PASS: Help uses TOP/SAMPLE, no LIMIT mentioned
- [ ] FAIL: Help still mentions LIMIT
- [ ] N/A: Help doesn't mention row limiting

---

## Test Results Summary

After completing all tests, fill out this summary:

### Bug 1: Table Padding

| Test | Status | Notes |
|------|--------|-------|
| TC044: Basic 5-Column | [ ] PASS [ ] FAIL |  |
| TC045: Wide 16-Column | [ ] PASS [ ] FAIL |  |
| TC046: NULL Values | [ ] PASS [ ] FAIL |  |
| TC048: Mixed Types | [ ] PASS [ ] FAIL |  |

**Bug 1 Fixed?** [ ] YES [ ] NO

---

### Bug 2: Tab Completion

| Test | Status | Notes |
|------|--------|-------|
| TC049: FROM Tab | [ ] PASS [ ] FAIL |  |
| TC050: Database.Tab | [ ] PASS [ ] FAIL |  |
| TC051: Loading Indicator | [ ] PASS [ ] FAIL |  |
| TC052: Error Messages | [ ] PASS [ ] FAIL [ ] SKIP |  |
| TC053: CREATE TABLE Cache | [ ] PASS [ ] FAIL |  |
| TC054: DROP TABLE Cache | [ ] PASS [ ] FAIL |  |

**Bug 2 Fixed?** [ ] YES [ ] NO

---

### Bug 3: Result Paging

| Test | Status | Notes |
|------|--------|-------|
| TC057: j/k Keys | [ ] PASS [ ] FAIL |  |
| TC058: PageUp/PageDown | [ ] PASS [ ] FAIL |  |
| TC059: h/l Keys (Horizontal) | [ ] PASS [ ] FAIL |  |
| TC062: Exit Pager | [ ] PASS [ ] FAIL |  |
| TC063: /pager on/off | [ ] PASS [ ] FAIL |  |

**Bug 3 Fixed?** [ ] YES [ ] NO

---

### Bug 4: LIMIT Hint

| Test | Status | Notes |
|------|--------|-------|
| TC064: Hint Message | [ ] PASS [ ] FAIL |  |
| TC065: Help Text | [ ] PASS [ ] FAIL [ ] N/A |  |

**Bug 4 Fixed?** [ ] YES [ ] NO

---

## Overall Sprint 8 Assessment

**Total Tests:** 22
**Passed:** ___
**Failed:** ___
**Skipped:** ___

**All Bugs Fixed?** [ ] YES [ ] NO

**Sprint 8 Ready for Completion?** [ ] YES [ ] NO

---

## Notes and Observations

(Record any issues, unexpected behavior, or additional findings)

---

## Screenshots/Evidence

(Attach screenshots or copy/paste terminal output showing key tests, especially:)
- Bug 1: Wide table output (TC045)
- Bug 2: Tab completion menu (TC049)
- Bug 3: Pager with status line (TC057)
- Bug 4: Hint message (TC064)

---

**Testing Completed:** _______________ (date/time)
**Tester:** _______________________
**Commit Tested:** ________________ (git rev-parse HEAD)
