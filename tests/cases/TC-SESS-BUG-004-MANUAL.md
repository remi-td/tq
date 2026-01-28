# TC-SESS-BUG-004-MANUAL: Bug Fix - Manual Verification with User Scenario

**Test Case ID:** TC-SESS-BUG-004-MANUAL
**Feature:** Bug Fix - Manual Verification of Issue #10 Fix
**Test Type:** Manual (Human Validation)
**Priority:** P0
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Manually verify that the exact bug scenario reported by user in issue #10 is fixed: when 3 sessions exist (with SessionNo 1230, 1231, 1232 and states IDLE, IDLE, DISPATCHING/ACTIVE), all 3 sessions appear in `/sessions` output.

---

## Prerequisites

- [ ] tq installed (Sprint 27 build with bug fix)
- [ ] Live Teradata database available
- [ ] TQ_LOGON configured
- [ ] Ability to create 3 sessions for testing
- [ ] User's original bug report (GitHub issue #10) available for reference

---

## Test Steps

### Step 1: Review User's Bug Report
**Action:** Read GitHub issue #10 to understand exact scenario

**User's Report:**
```
Direct SQL query: SELECT * FROM TABLE(MonitorSession(-1,'*',0))
Returns 3 rows:
- SessionNo 1230: PEstate=IDLE, AMPState=IDLE
- SessionNo 1231: PEstate=IDLE, AMPState=IDLE
- SessionNo 1232: PEstate=DISPATCHING, AMPState=ACTIVE (MISSING from /sessions)

/sessions command:
Shows only 2 sessions (1230, 1231)
Missing SessionNo 1232
```

**Expected Understanding:**
- User expects 3 sessions
- /sessions shows only 2
- Missing session is DISPATCHING/ACTIVE state
- Bug: State-based filtering or row parsing error

### Step 2: Create Test Scenario - 3 Sessions
**Action:** Create 3 database sessions to match user scenario

**Procedure:**
```bash
# Terminal 1: Start first REPL (will be IDLE)
tq repl
tq> -- Leave idle, don't run queries

# Terminal 2: Start second REPL (will be IDLE)
tq repl
tq> -- Leave idle

# Terminal 3: Start third REPL and run long query (will be DISPATCHING/ACTIVE)
tq repl
tq> SELECT t.*, c.* FROM dbc.tables t CROSS JOIN dbc.columns c;
-- Long-running query, leave it running

# Terminal 4: Test the /sessions command
tq repl
tq> /sessions
```

**Expected Result:**
- 3 sessions created successfully
- Sessions in various states (IDLE, DISPATCHING/ACTIVE)
- Test environment matches user scenario

### Step 3: Execute Direct SQL Query
**Action:** In Terminal 4, query MonitorSession directly
```
tq> SELECT SessionNo, UserName, PEstate, AMPState FROM TABLE(MonitorSession(-1,'*',0)) ORDER BY SessionNo;
```

**Expected Result:**
- Query returns at least 3 rows (or more if other sessions exist)
- Shows sessions in IDLE and DISPATCHING/ACTIVE states
- Record exact SessionNo values and states
- **Note the count: EXPECTED_COUNT = [X]**

### Step 4: Execute /sessions Command
**Action:** In same REPL, execute /sessions
```
tq> /sessions
```

**Expected Result:**
- Table displays active sessions
- Footer shows count: "N sessions found"
- **Note the count: ACTUAL_COUNT = [Y]**

### Step 5: Compare Session Counts
**Action:** Manually compare EXPECTED_COUNT vs ACTUAL_COUNT

**Verification:**
```
EXPECTED_COUNT (from direct SQL): [X]
ACTUAL_COUNT (from /sessions footer): [Y]

Match: [YES / NO]
```

**Expected Result:**
- EXPECTED_COUNT == ACTUAL_COUNT
- No sessions missing
- Bug is FIXED

### Step 6: Verify All SessionNo Values Present
**Action:** Compare SessionNo values from direct query vs /sessions output

**Verification:**
```
SessionNo values from direct SQL: [List]
SessionNo values from /sessions: [List]

Missing SessionNo values: [List or "None"]
```

**Expected Result:**
- All SessionNo values from direct query appear in /sessions
- No missing sessions
- Specifically, DISPATCHING/ACTIVE sessions are present

### Step 7: Verify DISPATCHING/ACTIVE Session Visible
**Action:** Look for the long-running query session (DISPATCHING/ACTIVE) in /sessions output

**Verification:**
```
Session from Terminal 3 (long query):
- SessionNo: [X]
- PEstate: DISPATCHING
- AMPState: ACTIVE
- Visible in /sessions: [YES / NO]
```

**Expected Result:**
- Long-running query session appears in /sessions
- PEstate shows "DISPATCHING"
- AMPState shows "ACTIVE"
- Skew percentages show numeric values (not [--])
- **This is the session that was MISSING in user's bug report**

### Step 8: Visual Inspection - Output Quality
**Action:** Visually inspect /sessions output for correctness

**Check:**
- [ ] Table formatting is clean (no jagged edges)
- [ ] All 10 columns present and aligned
- [ ] LogonTime format is YYYY/MM/DD HH:MM:SS.ss
- [ ] Skew percentages formatted correctly (X.XX or [--])
- [ ] Numbers right-aligned, strings left-aligned
- [ ] Footer count matches visible row count

**Expected Result:**
- Output is professional and readable
- No visual defects
- All data correct

### Step 9: Cleanup
**Action:** Exit all REPL sessions
```
# In each terminal:
tq> /quit
```

**Expected Result:**
- All sessions exit cleanly
- Long-running query terminated

---

## Expected Results

### Success Criteria
- [x] Direct SQL count matches /sessions footer count
- [x] All SessionNo values from direct SQL appear in /sessions
- [x] DISPATCHING/ACTIVE session is visible (user's bug fixed)
- [x] No state-based filtering occurs
- [x] Output quality is high (formatting, alignment, data accuracy)
- [x] Bug from issue #10 is definitively FIXED

### User Scenario Recreation
```
User's scenario: 3 sessions (1230, 1231, 1232)
User's bug: Only 2 sessions shown (1232 missing)
User's missing session: DISPATCHING/ACTIVE

Test scenario: [X] sessions created
Test result: [Y] sessions shown in /sessions
Missing sessions: [None / List]

BUG FIXED: [YES / NO]
```

---

## Actual Results

**Test Execution Date:** [To be filled by manual tester]
**Tester:** [Name]
**Build Version:** [Commit hash]
**Database:** [Host, version]

**Screenshot: Direct SQL Query**
```
[Paste screenshot or text output of direct MonitorSession query]
```

**Screenshot: /sessions Output**
```
[Paste screenshot or text output of /sessions command]
```

**Session Count Comparison:**
```
Direct SQL count: [X]
/sessions count: [Y]
Match: [YES / NO]
```

**SessionNo Values Comparison:**
```
Direct SQL SessionNo values: [List]
/sessions SessionNo values: [List]
Missing values: [None / List]
```

**DISPATCHING/ACTIVE Session:**
```
Visible in /sessions: [YES / NO]
SessionNo: [X]
PEstate: [Value]
AMPState: [Value]
```

**Visual Quality Assessment:**
```
Table formatting: [GOOD / ISSUES]
Column alignment: [CORRECT / INCORRECT]
Data accuracy: [CORRECT / INCORRECT]
Overall quality: [PASS / FAIL]
```

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Pass Criteria (ALL must be true):**
1. Session counts match (direct SQL == /sessions)
2. All SessionNo values present
3. DISPATCHING/ACTIVE sessions visible
4. No visual defects
5. User's exact bug scenario is fixed

**Defects Found:**
- [If FAIL: Describe exactly what is still broken]
- [If FAIL: Include evidence (screenshots, output)]

---

## Notes

- This is a MANUAL test because it requires human judgment and real-world validation
- Automated tests (TC-SESS-BUG-001, TC-SESS-BUG-002) complement this but don't replace human verification
- This test directly validates user's exact bug report
- Screenshot evidence is valuable for documentation
- If this test FAILS, automated tests may have given false positives (see Sprint 18/19/20 lessons)

---

## Related Requirements

- GitHub Issue #10: [BUG] Incorrect number of sessions
- User Report: 3 sessions exist, only 2 displayed, SessionNo 1232 missing
- AC-BUG-FIX-001: "All 3 sessions from user example are displayed correctly"
- AC-BUG-FIX-007: "Manual verification with user's example query" (sprint-27-planning.md:89)
