# TC-SESS-001: /sessions Command Execution in REPL

**Test Case ID:** TC-SESS-001
**Feature:** Sessions Command
**Test Type:** Interactive (PTY)
**Priority:** P0
**Created:** 2026-01-27

---

## Objective

Verify that the `/sessions` metacommand executes successfully in REPL mode and displays active Teradata sessions with correct column structure and data.

---

## Prerequisites

- [ ] tq installed and accessible
- [ ] Live Teradata database available
- [ ] TQ_LOGON environment variable set or .env file configured
- [ ] User has SELECT privilege on DBC.MonitorSession

---

## Test Steps

### Step 1: Start REPL
**Action:** Launch tq in REPL mode
```bash
tq repl
```

**Expected Result:**
- REPL starts successfully
- Connection banner appears
- Prompt shows: `tq>`

### Step 2: Execute /sessions Command
**Action:** Type `/sessions` and press Enter
```
tq> /sessions
```

**Expected Result:**
- Query executes successfully
- Table output appears with box-drawing characters
- Header row contains 10 columns (in order):
  1. SessionNo
  2. UserName
  3. LogonTime
  4. PEstate
  5. AMPState
  6. AMPCPUSec
  7. AMPIO
  8. ReqSpool
  9. Amp CPU Skew %
  10. Amp IO Skew %
- At least one session row appears (current session)
- Footer shows: `N sessions found (Query time: X.XXXs)`

### Step 3: Verify Column Data
**Action:** Inspect output data for correctness
```
Expected column types:
- SessionNo: INTEGER (e.g., 1076)
- UserName: STRING (e.g., DBC, alice)
- LogonTime: TIMESTAMP in format YYYY/MM/DD HH:MM:SS.ss
- PEstate: STRING (IDLE, DISPATCHING, ACTIVE)
- AMPState: STRING (IDLE, ACTIVE)
- AMPCPUSec: DECIMAL (e.g., 0, 0.376, 366.736)
- AMPIO: INTEGER (e.g., 6, 6782, 75335)
- ReqSpool: INTEGER (e.g., 0, 26753187840)
- Amp CPU Skew %: DECIMAL (X.XX format) or [--] for IDLE
- Amp IO Skew %: DECIMAL (X.XX format) or [--] for IDLE
```

**Expected Result:**
- All columns display correct data types
- LogonTime format is YYYY/MM/DD HH:MM:SS.ss (not YYYY-MM-DD)
- Skew percentages show [--] for IDLE sessions
- Skew percentages show X.XX format (two decimals) for ACTIVE sessions
- Numbers are right-aligned
- Strings are left-aligned

### Step 4: Verify Table Structure
**Action:** Check visual table formatting
```
Expected structure:
┌───────────┬──────────┬────────────────────────┬...┐
│ SessionNo │ UserName │ LogonTime              │...│
├───────────┼──────────┼────────────────────────┼...┤
│      1076 │ DBC      │ 2026/01/27 15:33:26.00 │...│
└───────────┴──────────┴────────────────────────┴...┘
```

**Expected Result:**
- Top border present (┌─┬─┐)
- Header row with column names
- Header separator (├─┼─┤)
- Data rows with values
- Bottom border (└─┴─┘)
- Columns properly aligned
- No jagged edges or misalignment

### Step 5: Exit REPL
**Action:** Type `/quit` and press Enter
```
tq> /quit
```

**Expected Result:**
- REPL exits cleanly
- No errors displayed

---

## Expected Results

### Success Criteria
- [x] Command executes without errors
- [x] Table displays with 10 columns
- [x] Column headers match specification
- [x] At least one session row displayed
- [x] LogonTime format is YYYY/MM/DD HH:MM:SS.ss
- [x] Skew percentages display correctly (X.XX or [--])
- [x] Table formatting is clean and aligned
- [x] Footer shows session count and query time

### Sample Output
```
tq> /sessions

Active Sessions:
┌───────────┬──────────┬────────────────────────┬─────────────┬──────────┬───────────┬───────┬─────────────┬────────────────┬──────────────┐
│ SessionNo │ UserName │ LogonTime              │ PEstate     │ AMPState │ AMPCPUSec │ AMPIO │ ReqSpool    │ Amp CPU Skew % │ Amp IO Skew %│
├───────────┼──────────┼────────────────────────┼─────────────┼──────────┼───────────┼───────┼─────────────┼────────────────┼──────────────┤
│      1076 │ DBC      │ 2026/01/27 15:33:26.00 │ IDLE        │ IDLE     │         0 │     6 │           0 │           [--] │         [--] │
│      1077 │ DBC      │ 2026/01/27 15:33:27.00 │ IDLE        │ IDLE     │     0.376 │  6782 │           0 │           [--] │         [--] │
│      1078 │ DBC      │ 2026/01/27 15:33:28.00 │ DISPATCHING │ ACTIVE   │   366.736 │ 75335 │ 26753187840 │           2.87 │         3.78 │
└───────────┴──────────┴────────────────────────┴─────────────┴──────────┴───────────┴───────┴─────────────┴────────────────┴──────────────┘

3 sessions found (Query time: 0.234s)
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** [quality-validator or manual tester]
**Build Version:** [Commit hash]

**Actual Output:**
```
[Paste actual output here]
```

**Observations:**
- [Note any differences from expected output]
- [Note any visual issues]
- [Note any performance observations]

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Defects Found:**
- [List any bugs discovered]
- [Link to issue tracker if applicable]

---

## Notes

- This test requires an active Teradata database connection
- Session list will vary based on current database activity
- Skew percentages will vary based on workload
- Query timing may vary based on database load

---

## Related Requirements

- REQ-SESS-001.1: Primary command `/sessions`
- REQ-SESS-002: Data from MonitorSession(-1,'*',0)
- REQ-SESS-003: Output formatted as table with 10 columns
- REQ-SESS-003.3: LogonTime format YYYY/MM/DD HH:MM:SS.ss
- REQ-SESS-004.1: NULL skew as [--]
- REQ-SESS-004.2: Skew format X.XX
- AC-1: `/sessions` command available in REPL
- AC-3: Output displays 10 columns
- AC-5: Logon times formatted correctly
