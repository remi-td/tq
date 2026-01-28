# TC-SESS-BUG-001: Bug Fix - All Sessions Displayed (Row Count Match)

**Test Case ID:** TC-SESS-BUG-001
**Feature:** Bug Fix - /sessions Command Incorrect Session Count (#10)
**Test Type:** Integration (PTY + Database Query Comparison)
**Priority:** P0
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Verify that the `/sessions` command displays ALL sessions returned by MonitorSession(-1,'*',0) with no rows lost, specifically addressing the bug where 3 sessions exist but only 2 are displayed.

---

## Prerequisites

- [ ] tq installed and accessible
- [ ] Live Teradata database available
- [ ] TQ_LOGON environment variable set or .env file configured
- [ ] User has SELECT privilege on DBC.MonitorSession
- [ ] At least 3 active sessions on database (can be created by opening multiple REPL sessions)

---

## Test Steps

### Step 1: Query MonitorSession Directly
**Action:** Execute direct SQL query to count actual sessions
```bash
tq query "SELECT COUNT(*) as session_count FROM TABLE(MonitorSession(-1,'*',0))"
```

**Expected Result:**
- Query executes successfully
- Returns a count (e.g., 3, 4, 5 depending on active sessions)
- Record this count as EXPECTED_COUNT

### Step 2: Start REPL
**Action:** Launch tq in REPL mode
```bash
tq repl
```

**Expected Result:**
- REPL starts successfully
- Prompt shows: `tq>`

### Step 3: Execute /sessions Command
**Action:** Type `/sessions` and press Enter
```
tq> /sessions
```

**Expected Result:**
- Query executes successfully
- Table output appears
- Footer shows: `N sessions found (Query time: X.XXXs)`
- Record footer count as ACTUAL_COUNT

### Step 4: Compare Counts
**Action:** Verify ACTUAL_COUNT from /sessions matches EXPECTED_COUNT from direct query

**Expected Result:**
- ACTUAL_COUNT == EXPECTED_COUNT
- No sessions are missing from /sessions output
- All sessions from MonitorSession(-1,'*',0) are displayed

### Step 5: Verify Specific Session States
**Action:** Check that sessions in various states are ALL displayed
```
Look for sessions with different PEState/AMPState combinations:
- IDLE / IDLE
- IDLE / ACTIVE
- DISPATCHING / ACTIVE
- ACTIVE / IDLE
- ACTIVE / ACTIVE
```

**Expected Result:**
- All session states from database query appear in /sessions output
- No filtering based on PEState or AMPState
- Sessions in DISPATCHING/ACTIVE state are NOT filtered out (user's original bug)

### Step 6: Verify User's Exact Bug Scenario
**Action:** If 3 sessions exist with states IDLE, IDLE, and DISPATCHING/ACTIVE (user's scenario)
```
Direct query shows:
SessionNo 1230 - IDLE/IDLE
SessionNo 1231 - IDLE/IDLE
SessionNo 1232 - DISPATCHING/ACTIVE

/sessions should show:
All 3 sessions with all 3 SessionNo values visible
```

**Expected Result:**
- All 3 sessions appear in /sessions output
- SessionNo 1232 (or equivalent DISPATCHING/ACTIVE session) is NOT missing
- Bug is FIXED

### Step 7: Exit REPL
**Action:** Type `/quit` and press Enter
```
tq> /quit
```

**Expected Result:**
- REPL exits cleanly

---

## Expected Results

### Success Criteria
- [x] Direct query count matches /sessions footer count
- [x] All sessions from MonitorSession(-1,'*',0) appear in output
- [x] Sessions with DISPATCHING/ACTIVE state are displayed
- [x] No filtering based on session state
- [x] Row count in footer is accurate
- [x] Bug from issue #10 is FIXED (3 sessions show as 3, not 2)

### Pass Conditions
1. **Row count match**: EXPECTED_COUNT == ACTUAL_COUNT
2. **No missing sessions**: All SessionNo values from direct query appear in /sessions
3. **State coverage**: All session states (IDLE, DISPATCHING, ACTIVE) are displayed
4. **User scenario fixed**: If 3 sessions with varied states exist, all 3 appear

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** quality-validator
**Build Version:** [Commit hash]

**Direct Query Count (EXPECTED_COUNT):** [Fill in]

**Example:**
```
┌───────────────┐
│ session_count │
├───────────────┤
│             3 │
└───────────────┘
```

**/sessions Footer Count (ACTUAL_COUNT):** [Fill in]

**Example:**
```
3 sessions found (Query time: 0.234s)
```

**Full /sessions Output:**
```
[Paste actual /sessions output here]
```

**Observations:**
- [Note if counts match]
- [Note any missing sessions]
- [Note any state-specific filtering]

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Defects Found:**
- [If FAIL: List which sessions are missing]
- [If FAIL: Note which states are filtered out]

---

## Notes

- This test directly validates the user's bug report from issue #10
- Test requires at least 3 active sessions for meaningful validation
- Session states may vary - test focuses on count match, not specific states
- If only 1-2 sessions exist, create additional sessions by:
  - Opening multiple `tq repl` instances
  - Running long queries in separate sessions
  - Having database administrator create test sessions

---

## Related Requirements

- GitHub Issue #10: BUG - Incorrect number of sessions
- AC-BUG-FIX-001: "All 3 sessions from user example are displayed correctly" (sprint-27-planning.md:85)
- AC-BUG-FIX-002: "Regression test added to prevent recurrence" (sprint-27-planning.md:86)
- REQ-SESS-002.1: "Data source: MonitorSession(-1,'*',0)" (repl.md:1560)
- BUG-ROOT-001: "No filtering applied based on session state" (test strategy hypothesis)
- BUG-ROOT-003: "Session count in footer matches database count" (test strategy hypothesis)
