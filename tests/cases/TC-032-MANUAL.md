# TC-032-MANUAL: Manual Validation - Content-Based Column Width

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-032-MANUAL |
| **Title** | Manual Validation - Content-Based Column Width in REPL |
| **Category** | Manual Validation |
| **Priority** | **BLOCKING - MANDATORY** |
| **Feature** | Sprint 32 - Content-Based Column Width (AC-4, AC-6) |
| **Test Type** | Manual (Type 4: Visual/Interactive) |
| **Created** | 2026-02-03 |

## Purpose

**MANDATORY manual validation** of visual column density improvement in actual REPL environment at multiple terminal widths. This validation is **BLOCKING** for sprint closure per Sprint 31 testing philosophy.

## Acceptance Criteria Coverage

- **AC-4**: `SELECT * FROM DBC.Databases` displays 8+ columns in 117-char terminal (PRIMARY TEST)
- **AC-6**: Manual validation: REPL query shows improved density (MANDATORY)
- **AC-8**: Works correctly with NULL values (visual alignment)
- **AC-9**: Works correctly with numeric alignment (visual alignment)
- **AC-2**: Maximum width cap graceful (visual truncation)

## Type 4 Feature Classification

Per `docs/testing/approach.md` §Feature Types and Their Testing Limitations:

> #### Type 4: Interactive/Alternate Screen (Minimal Automated Coverage)
>
> **Limitations:**
> - Terminal width-dependent behavior
> - User-observable visual improvements
> - Alignment and spacing quality
>
> **Mitigation:** Limited PTY tests for state changes + **MANDATORY manual validation**

**Feature #13 is Type 4 because:**
- User pain point is VISUAL: "only 2 columns visible" → "8+ columns visible"
- Terminal width-dependent rendering (80, 117, 120, 160 chars)
- Alignment quality cannot be validated by automated tests
- Density improvement is subjective visual assessment

## Quality Validator Role

Per `docs/testing/philosophy.md` line 298:

> **quality-validator verdict is ADVISORY for visual features.** The sprint coordinator must manually verify before approval.

**This test must be performed by the sprint coordinator (or designated manual tester).** The quality-validator can design this test and execute automated tests, but **cannot approve the sprint** based on automated tests alone.

## Prerequisites

- [ ] tq installed with Sprint 32 changes
- [ ] Live Teradata database available
- [ ] TQ_LOGON environment variable set or .env file configured
- [ ] Terminal emulator with resize capability
- [ ] `script` command available for evidence capture

## Test Procedure

### Setup: Capture Before State (Optional but Recommended)

If previous version available, capture baseline for comparison:

```bash
# Terminal width: 117 characters
script /tmp/sprint32-before-117.txt
tq repl
tq> SELECT * FROM DBC.Databases;
# Note: How many columns visible? (Expected: 2-3 with schema-based)
tq> /quit
exit
```

### Test 1: Primary Test - 117-Character Terminal (AC-4)

**This is the PRIMARY test - MUST PASS for sprint approval**

**Step 1:** Resize terminal to exactly 117 characters width

```bash
# Verify terminal width
tput cols
# Should output: 117
```

**Step 2:** Start evidence capture

```bash
script /tmp/sprint32-test-117.txt
```

**Step 3:** Launch tq REPL

```bash
tq repl
```

**Step 4:** Execute test query

```sql
tq> SELECT * FROM DBC.Databases;
```

**Step 5:** Count visible columns

Count the number of column headers displayed BEFORE the "(+N cols)" indicator.

**Expected Result:**
- **8 or more columns visible** (AC-4 requirement)
- Columns should include: DatabaseName, CreatorName, OwnerName, AccountName, ProtectionType, JournalFlag, PermSpace, SpoolSpace, and possibly TempSpace
- Truncation indicator shows remaining hidden columns (e.g., "(+7 cols)")

**Actual Result:**
- Visible columns: [COUNT]
- Column names visible: [LIST]
- Truncation indicator: [YES/NO, text shown]

**Step 6:** Exit and save evidence

```sql
tq> /quit
```

```bash
exit  # Exit script capture
```

**Pass/Fail:**
- **PASS**: 8 or more columns visible
- **FAIL**: Fewer than 8 columns visible

---

### Test 2: Narrow Terminal - 80 Characters

**Step 1:** Resize terminal to 80 characters width

```bash
# Verify
tput cols  # Should output: 80
```

**Step 2:** Start evidence capture

```bash
script /tmp/sprint32-test-80.txt
```

**Step 3:** Execute test query

```bash
tq repl
tq> SELECT * FROM DBC.Databases;
tq> /quit
exit
```

**Expected Result:**
- More columns visible than schema-based approach (even if fewer than 117-char terminal)
- Table renders correctly (no visual corruption)
- Truncation indicator present if columns hidden

**Actual Result:**
- Visible columns: [COUNT]
- Visual quality: [GOOD/ISSUES]

---

### Test 3: Standard Wide Terminal - 120 Characters

**Step 1:** Resize terminal to 120 characters width

```bash
tput cols  # Should output: 120
```

**Step 2:** Execute test query (with evidence)

```bash
script /tmp/sprint32-test-120.txt
tq repl
tq> SELECT * FROM DBC.Databases;
tq> /quit
exit
```

**Expected Result:**
- Efficient use of available space
- More columns visible than 117-char test
- No excessive whitespace between columns

**Actual Result:**
- Visible columns: [COUNT]
- Space utilization: [EFFICIENT/WASTEFUL]

---

### Test 4: Very Wide Terminal - 160 Characters

**Step 1:** Resize terminal to 160 characters width

```bash
tput cols  # Should output: 160
```

**Step 2:** Execute test query (with evidence)

```bash
script /tmp/sprint32-test-160.txt
tq repl
tq> SELECT * FROM DBC.Databases;
tq> /quit
exit
```

**Expected Result:**
- Many columns visible (10+)
- Table looks professional and balanced
- No excessive padding

**Actual Result:**
- Visible columns: [COUNT]
- Visual quality: [PROFESSIONAL/ISSUES]

---

### Test 5: Visual Alignment Check (AC-8, AC-9)

**Step 1:** Execute query with mixed data types and NULLs

```sql
tq> SELECT SessionNo, UserName, AMPCPUSec, ReqSpool FROM DBC.MonitorSession(-1,'*',0);
```

**Expected Result:**
- NULL values display as `[NULL]` and are aligned correctly
- Numeric columns (SessionNo, AMPCPUSec, ReqSpool) are right-aligned
- Text columns (UserName) are left-aligned
- Column headers aligned with content
- No jagged edges or misalignment

**Actual Result:**
- NULL alignment: [CORRECT/ISSUES]
- Numeric alignment: [RIGHT-ALIGNED/ISSUES]
- Text alignment: [LEFT-ALIGNED/ISSUES]
- Overall visual quality: [GOOD/ISSUES]

---

### Test 6: Truncation Check - Very Long Content (AC-2)

**Step 1:** Create test table with long VARCHAR content (if possible)

```sql
-- If able to create test table:
tq> CREATE TABLE test_long_content (id INTEGER, description VARCHAR(500));
tq> INSERT INTO test_long_content VALUES (1, 'This is a very long description that exceeds 100 characters and should be truncated gracefully with an ellipsis to prevent the column from consuming the entire screen width and making other columns invisible...');
tq> SELECT * FROM test_long_content;
```

**Expected Result:**
- Long content truncated at 100 characters (or configured MAX_COLUMN_WIDTH)
- Truncation shows ellipsis: "...text ends here..."
- Truncation is graceful (no visual corruption)
- Other columns still visible

**Actual Result:**
- Truncation at: [CHAR_COUNT]
- Ellipsis shown: [YES/NO]
- Visual quality: [GRACEFUL/CORRUPT]

---

### Test 7: Before/After Comparison (If Available)

**Step 1:** Compare captured outputs

```bash
# View before capture (schema-based)
less /tmp/sprint32-before-117.txt

# View after capture (content-based)
less /tmp/sprint32-test-117.txt
```

**Comparison Metrics:**
- Columns visible (before): [COUNT]
- Columns visible (after): [COUNT]
- Improvement: [PERCENTAGE or X more columns]

---

## Evidence Documentation

**Mandatory Evidence Files:**

1. `/tmp/sprint32-test-117.txt` - PRIMARY TEST (117-char terminal)
2. `/tmp/sprint32-test-80.txt` - Narrow terminal
3. `/tmp/sprint32-test-120.txt` - Standard wide terminal
4. `/tmp/sprint32-test-160.txt` - Very wide terminal

**Store evidence in:** `tests/results/sprint-32/manual-validation/`

**Evidence must include:**
- Terminal width (from `tput cols`)
- Actual table output showing column headers and data rows
- Visible column count
- Truncation indicator text (if any)

---

## Pass/Fail Criteria

**OVERALL PASS if:**
- ✅ **Test 1 (117-char) shows 8+ columns** (AC-4 - BLOCKING)
- ✅ All terminal widths show improved column density vs. schema-based
- ✅ Table rendering is clean and professional at all widths
- ✅ NULL values aligned correctly
- ✅ Numeric columns right-aligned correctly
- ✅ Long content truncated gracefully
- ✅ No visual corruption or layout issues

**OVERALL FAIL if:**
- ❌ Test 1 (117-char) shows fewer than 8 columns (AC-4 FAIL)
- ❌ Any visual corruption (jagged edges, misalignment)
- ❌ NULL or numeric alignment broken
- ❌ Truncation not graceful or causes corruption

**BLOCKED if:**
- Database unavailable
- Cannot create test data
- Terminal cannot be resized

---

## Sign-off

**Manual Tester:** [Sprint Coordinator or Designated Tester]

**Test Execution Date:** [YYYY-MM-DD]

**Build/Commit Hash:** [Git commit]

**Test Result:** [PASS / FAIL / BLOCKED]

**Evidence Location:** `tests/results/sprint-32/manual-validation/`

**Notes:**
- [Any observations, issues, or recommendations]

**Verdict:**
- [ ] **APPROVED** - All tests passed, visual improvement confirmed
- [ ] **REJECTED** - Tests failed, visual improvement not achieved
- [ ] **BLOCKED** - Tests could not be executed

---

## Sprint 31 Lessons Applied

**Type 4 Feature Classification:**
- Feature #13 correctly classified as Type 4 (visual/interactive)
- Manual validation MANDATORY, not optional
- Quality validator verdict is ADVISORY
- Sprint coordinator must perform validation before approval

**Evidence Requirements:**
- Script command output captures terminal session
- Multiple terminal widths tested (80, 117, 120, 160)
- Before/after comparison if available
- Evidence stored in `tests/results/sprint-32/manual-validation/`

**Honest Assessment:**
- This test is BLOCKING for sprint closure
- Automated test pass rates (100%) are ADVISORY inputs, not conclusions
- Manual validation result is the FINAL verdict for AC-4 and AC-6
- If visual improvement not observed, sprint CANNOT be approved

---

## Related Test Cases

- **TC-032-001 to TC-032-005**: Unit tests (logic correctness)
- **TC-032-011 to TC-032-012**: Integration tests (pipeline correctness)
- **BENCH-032-001**: Performance benchmarks (AC-7)

**This manual validation complements automated tests by validating what automated tests CANNOT:**
- Visual density improvement (user-observable benefit)
- Alignment quality (visual appearance)
- Terminal width-dependent rendering (actual terminal behavior)
- Subjective assessment of "looks good" and "is usable"
