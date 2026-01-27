# TC-SESS-010: Manual Validation Checklist

**Test Case ID:** TC-SESS-010
**Feature:** Sessions Command - Manual Validation
**Test Type:** Manual Testing
**Priority:** P1
**Created:** 2026-01-27

---

## Objective

Perform human validation of visual quality, usability, accuracy, and performance of the `/sessions` command that cannot be fully automated.

---

## Prerequisites

- [ ] tq installed and accessible
- [ ] Live Teradata database with realistic workload
- [ ] TQ_LOGON environment variable set
- [ ] User has SELECT privilege on DBC.MonitorSession
- [ ] Access to DBA tools for comparison (optional but recommended)

---

## Manual Test Checklist

### 1. Visual Table Quality

#### Test 1.1: Table Alignment
**Action:** Execute `/sessions` and visually inspect table
```bash
tq repl
tq> /sessions
```

**Checklist:**
- [ ] All columns are properly aligned
- [ ] No jagged edges or wrapping
- [ ] Box-drawing characters render correctly
- [ ] Headers are clearly readable
- [ ] Data rows line up under headers
- [ ] Vertical separators are straight

**Pass Criteria:** Table is visually clean and professional

---

#### Test 1.2: Column Widths
**Action:** Check if column widths are appropriate for content
```
SessionNo column: Wide enough for 10-digit session IDs
UserName column: Wide enough for typical usernames (8-30 chars)
LogonTime column: Fixed width for timestamp format
Skew columns: Wide enough for percentages (5-6 chars)
```

**Checklist:**
- [ ] No column content is truncated
- [ ] Columns are not excessively wide (no wasted space)
- [ ] Consistent spacing between columns
- [ ] Terminal width is respected (no horizontal scrolling)

**Pass Criteria:** Columns sized appropriately for data

---

### 2. Skew Calculation Accuracy

#### Test 2.1: Compare with Known Tool
**Action:** Run `/sessions` and compare skew values to DBA tool (e.g., Teradata Viewpoint, SQL Assistant)
```sql
-- Run same query in another tool
SELECT SessionNo,
       (100 * (1 - (AvgAmpCPUSec / NULLIFZERO(HotAmp1CPU))))(DECIMAL(4,2)) AS CPUSkew
FROM TABLE (MonitorSession(-1,'*',0)) AS t1;
```

**Checklist:**
- [ ] Skew percentages match external tool (within 0.01% tolerance)
- [ ] NULL skew for IDLE sessions is consistent
- [ ] Formula produces expected results for known workloads

**Pass Criteria:** Skew calculations are accurate (match reference tool)

---

### 3. Error Message Clarity

#### Test 3.1: Privilege Error Message
**Action:** If possible, test with user without MonitorSession access
```bash
# Connect as user without privilege
tq repl
tq> /sessions
```

**Checklist:**
- [ ] Error message is clear and understandable
- [ ] Message explains what privilege is needed
- [ ] GRANT statement example is correct
- [ ] Message suggests contacting DBA
- [ ] No technical jargon or stack traces

**Pass Criteria:** Error message is helpful and actionable

---

#### Test 3.2: Connection Error Message
**Action:** Disconnect network and run `/sessions`
```bash
# Disconnect database or network
tq> /sessions
```

**Checklist:**
- [ ] Error message explains connection issue
- [ ] Message suggests `/reconnect`
- [ ] No crash or hang
- [ ] User can recover (REPL continues)

**Pass Criteria:** Error message is clear, REPL remains functional

---

### 4. Query Performance

#### Test 4.1: Execution Time
**Action:** Execute `/sessions` and note query time from footer
```bash
tq> /sessions
...
N sessions found (Query time: X.XXXs)
```

**Checklist:**
- [ ] Query completes in <1 second (typical system)
- [ ] Query completes in <3 seconds (heavily loaded system)
- [ ] No timeout errors
- [ ] Performance is acceptable for interactive use

**Pass Criteria:** Query execution time is reasonable (<1s typical)

---

#### Test 4.2: Large Result Sets
**Action:** Run on system with many sessions (100+ if available)
```bash
tq> /sessions
```

**Checklist:**
- [ ] Large result sets display correctly
- [ ] No performance degradation
- [ ] Table formatting remains clean
- [ ] No truncation or errors

**Pass Criteria:** Handles large result sets gracefully

---

### 5. Usability and Discoverability

#### Test 5.1: Command Discoverability
**Action:** Imagine you're a new user trying to find session information
```bash
tq repl
tq> /help
tq> /s<TAB>
```

**Checklist:**
- [ ] `/sessions` appears in `/help` output
- [ ] Command description is clear
- [ ] Tab completion suggests `/sessions`
- [ ] Alias `/s` is documented
- [ ] User can easily discover the feature

**Pass Criteria:** Feature is discoverable without external documentation

---

#### Test 5.2: Output Readability
**Action:** Show output to non-technical stakeholder
```bash
tq> /sessions
```

**Checklist:**
- [ ] Column names are self-explanatory
- [ ] Data is readable without manual
- [ ] Skew percentages are understandable
- [ ] Timestamp format is clear
- [ ] Output makes sense to DBA

**Pass Criteria:** Output is readable to target users (DBAs)

---

### 6. Format Compatibility (Visual Check)

#### Test 6.1: CSV Visual Inspection
**Action:** Execute with CSV format and inspect
```bash
tq sessions --format csv
```

**Checklist:**
- [ ] CSV structure is valid
- [ ] No unescaped commas in data
- [ ] Header row is correct
- [ ] NULL skew is empty field (not "null" text)
- [ ] Can import into spreadsheet

**Pass Criteria:** CSV is valid and usable

---

#### Test 6.2: JSON Visual Inspection
**Action:** Execute with JSON format and inspect
```bash
tq sessions --format json | jq .
```

**Checklist:**
- [ ] JSON is valid (jq parses it)
- [ ] Structure is logical
- [ ] NULL skew is JSON `null`
- [ ] Numeric fields are numbers (not strings)
- [ ] Pretty-printed format is readable

**Pass Criteria:** JSON is valid and well-structured

---

### 7. Integration with Existing Features

#### Test 7.1: Works with Other Metacommands
**Action:** Use `/sessions` alongside other commands
```bash
tq> /list databases
tq> /sessions
tq> /help
```

**Checklist:**
- [ ] No interference with other metacommands
- [ ] State is maintained correctly
- [ ] Output format persists (if set)

**Pass Criteria:** Integrates cleanly with existing REPL

---

#### Test 7.2: Works After Other Operations
**Action:** Run SQL query, then `/sessions`
```bash
tq> SELECT COUNT(*) FROM DBC.TablesV;
tq> /sessions
```

**Checklist:**
- [ ] Command works after SQL execution
- [ ] No state corruption
- [ ] Output is correct

**Pass Criteria:** Command works in all REPL contexts

---

## Summary

### Manual Validation Results

**Total Checks:** 18

**Passed:** [count]
**Failed:** [count]
**Blocked:** [count]

### Critical Issues Found
- [List any blocking issues]

### Minor Issues Found
- [List any usability concerns]

### Recommendations
- [Suggestions for improvement]

---

## Pass/Fail Status

**Overall Status:** [PASS | FAIL | BLOCKED]

**Tester Name:** [Name]
**Test Date:** [Date]
**Build Version:** [Commit hash]

---

## Notes

- Manual validation is subjective but critical for UX
- Some checks require specific database conditions
- DBA feedback is valuable for accuracy validation
- Visual quality varies by terminal emulator

---

## Related Requirements

- All AC-1 through AC-10 (holistic validation)
- REQ-SESS-008.1: Target execution time <1 second
- User experience and usability (implied)
