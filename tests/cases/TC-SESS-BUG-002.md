# TC-SESS-BUG-002: Bug Fix - Session State Coverage (All States Displayed)

**Test Case ID:** TC-SESS-BUG-002
**Feature:** Bug Fix - /sessions Command Session State Filtering
**Test Type:** Integration (PTY + State Verification)
**Priority:** P0
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Verify that `/sessions` command displays sessions in ALL state combinations (IDLE/IDLE, IDLE/ACTIVE, DISPATCHING/IDLE, DISPATCHING/ACTIVE, ACTIVE/IDLE, ACTIVE/ACTIVE) without filtering based on PEState or AMPState.

---

## Prerequisites

- [ ] tq installed and accessible
- [ ] Live Teradata database available
- [ ] TQ_LOGON environment variable set or .env file configured
- [ ] User has SELECT privilege on DBC.MonitorSession
- [ ] Database has sessions in various states (or ability to create them)

---

## Test Steps

### Step 1: Query All Session States Directly
**Action:** Execute direct SQL to see all session states in database
```bash
tq query "SELECT SessionNo, PEstate, AMPState FROM TABLE(MonitorSession(-1,'*',0)) ORDER BY SessionNo"
```

**Expected Result:**
- Query returns all active sessions
- Shows variety of PEState values (IDLE, DISPATCHING, ACTIVE)
- Shows variety of AMPState values (IDLE, ACTIVE)
- Record all SessionNo and their states

### Step 2: Start REPL and Execute /sessions
**Action:** Launch REPL and run /sessions
```bash
tq repl
```
```
tq> /sessions
```

**Expected Result:**
- Table displays with all sessions
- All SessionNo values from Step 1 appear

### Step 3: Verify IDLE/IDLE Sessions Displayed
**Action:** Check output for sessions with PEstate=IDLE and AMPState=IDLE

**Expected Result:**
- At least one IDLE/IDLE session visible (current session should be IDLE)
- PEstate column shows "IDLE"
- AMPState column shows "IDLE"
- Skew percentages show [--]

### Step 4: Verify IDLE/ACTIVE Sessions Displayed
**Action:** Check output for sessions with PEstate=IDLE and AMPState=ACTIVE

**Expected Result:**
- If such sessions exist in database, they appear in output
- PEstate column shows "IDLE"
- AMPState column shows "ACTIVE"
- Skew percentages may show numeric values or [--]

### Step 5: Verify DISPATCHING/ACTIVE Sessions Displayed (User's Bug Case)
**Action:** Check output for sessions with PEstate=DISPATCHING and AMPState=ACTIVE

**Expected Result:**
- **CRITICAL:** If such sessions exist in database, they MUST appear in output
- This was the missing session state in user's bug report
- PEstate column shows "DISPATCHING"
- AMPState column shows "ACTIVE"
- Skew percentages show numeric values (e.g., 2.87, 3.78)

### Step 6: Verify ACTIVE Sessions Displayed
**Action:** Check output for sessions with PEstate=ACTIVE

**Expected Result:**
- If such sessions exist in database, they appear in output
- PEstate column shows "ACTIVE"
- AMPState column shows corresponding state
- Sessions are NOT filtered out

### Step 7: Create DISPATCHING/ACTIVE Session for Testing
**Action:** If no DISPATCHING/ACTIVE sessions exist naturally, create one:
```
# In separate terminal, start tq and run a long query:
tq query "SELECT * FROM dbc.tables CROSS JOIN dbc.columns"

# While query runs, check /sessions in original REPL:
tq> /sessions
```

**Expected Result:**
- The long-running query appears as DISPATCHING or ACTIVE session
- Session is visible in /sessions output
- Session count includes the running query

### Step 8: Compare State Coverage
**Action:** Verify all states from Step 1 direct query appear in /sessions output

**Expected Result:**
- 100% state coverage - no states filtered out
- All SessionNo values present
- All PEState values represented
- All AMPState values represented

---

## Expected Results

### Success Criteria
- [x] All session states from database appear in /sessions
- [x] DISPATCHING/ACTIVE sessions are NOT filtered out
- [x] IDLE sessions are displayed
- [x] ACTIVE sessions are displayed
- [x] No state-based filtering occurs
- [x] Session count matches database count (see TC-SESS-BUG-001)

### State Coverage Matrix
| PEState | AMPState | Expected in Output | Observed |
|---------|----------|--------------------|----------|
| IDLE | IDLE | ✅ YES | [Fill in] |
| IDLE | ACTIVE | ✅ YES | [Fill in] |
| DISPATCHING | IDLE | ✅ YES | [Fill in] |
| DISPATCHING | ACTIVE | ✅ YES (BUG FIX) | [Fill in] |
| ACTIVE | IDLE | ✅ YES | [Fill in] |
| ACTIVE | ACTIVE | ✅ YES | [Fill in] |

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** quality-validator
**Build Version:** [Commit hash]

**Direct Query Output (Step 1):**
```
[Paste SELECT SessionNo, PEstate, AMPState output here]
```

**/sessions Output (Step 2):**
```
[Paste /sessions output here]
```

**State Coverage Analysis:**
```
States found in database:
- IDLE/IDLE: [Count]
- IDLE/ACTIVE: [Count]
- DISPATCHING/IDLE: [Count]
- DISPATCHING/ACTIVE: [Count] (CRITICAL - user's bug)
- ACTIVE/IDLE: [Count]
- ACTIVE/ACTIVE: [Count]

States found in /sessions:
- IDLE/IDLE: [Count]
- IDLE/ACTIVE: [Count]
- DISPATCHING/IDLE: [Count]
- DISPATCHING/ACTIVE: [Count] (MUST MATCH DATABASE)
- ACTIVE/IDLE: [Count]
- ACTIVE/ACTIVE: [Count]
```

**Missing States:**
- [List any states present in database but missing from /sessions]
- [If DISPATCHING/ACTIVE is missing, this is the original bug - FAIL]

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Pass Condition:**
- PASS: All states in database appear in /sessions output (100% coverage)
- FAIL: Any state is filtered out (especially DISPATCHING/ACTIVE)
- BLOCKED: Cannot create sessions in required states

**Defects Found:**
- [If FAIL: List which states are filtered out]
- [If FAIL: Specifically note if DISPATCHING/ACTIVE is missing]

---

## Notes

- This test validates the hypothesis that bug is caused by state-based filtering
- DISPATCHING/ACTIVE is the critical state from user's bug report
- If no sessions in certain states exist, attempt to create them via long-running queries
- Some state combinations may be rare in test environment
- Minimum requirement: Verify DISPATCHING/ACTIVE sessions are NOT filtered (user's bug)

---

## Related Requirements

- GitHub Issue #10: BUG - Session 1232 (DISPATCHING/ACTIVE) was missing
- AC-BUG-FIX-001: "All 3 sessions from user example are displayed correctly"
- BUG-ROOT-001: "No filtering applied based on session state" (test strategy)
- REQ-SESS-002.1: "Data source: MonitorSession(-1,'*',0)" - No filtering specification
