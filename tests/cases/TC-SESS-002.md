# TC-SESS-002: tq sessions Batch Mode Execution

**Test Case ID:** TC-SESS-002
**Feature:** Sessions Command
**Test Type:** Integration (Batch Mode)
**Priority:** P0
**Created:** 2026-01-27

---

## Objective

Verify that `tq sessions` works in batch mode (non-interactive) and produces correct table output to stdout.

---

## Prerequisites

- [ ] tq installed and accessible in PATH
- [ ] Live Teradata database available
- [ ] TQ_LOGON environment variable set or .env file configured
- [ ] User has SELECT privilege on DBC.MonitorSession

---

## Test Steps

### Step 1: Execute tq sessions
**Action:** Run tq sessions in batch mode
```bash
tq sessions
```

**Expected Result:**
- Command executes successfully
- Table output printed to stdout
- Process exits with code 0

### Step 2: Verify Output Structure
**Action:** Capture output and inspect
```bash
tq sessions > output.txt
cat output.txt
```

**Expected Result:**
- Output contains table with 10 columns
- Header row present
- At least one data row (current session)
- Footer with session count
- No REPL prompt or interactive elements

### Step 3: Test with CSV Format
**Action:** Execute with --format csv flag
```bash
tq sessions --format csv
```

**Expected Result:**
- CSV output to stdout
- Header row: `SessionNo,UserName,LogonTime,PEstate,AMPState,AMPCPUSec,AMPIO,ReqSpool,Amp CPU Skew %,Amp IO Skew %`
- Data rows with comma-separated values
- NULL skew as empty field (e.g., `,,` for IDLE sessions)

### Step 4: Test with JSON Format
**Action:** Execute with --format json flag
```bash
tq sessions --format json
```

**Expected Result:**
- Valid JSON array to stdout
- Each element is an object with 10 keys
- NULL skew as `null` (not string "null")
- Numeric fields are numbers (not strings)

### Step 5: Test Output to File
**Action:** Execute with -o flag
```bash
tq sessions -o sessions.txt
cat sessions.txt
```

**Expected Result:**
- sessions.txt file created
- File contains table output
- No output to stdout (only to file)

### Step 6: Verify Exit Code
**Action:** Check exit code on success
```bash
tq sessions
echo $?
```

**Expected Result:**
- Exit code is 0

---

## Expected Results

### Success Criteria
- [x] `tq sessions` executes without REPL prompt
- [x] Table output contains 10 columns
- [x] CSV format produces valid CSV
- [x] JSON format produces valid JSON
- [x] Output to file works correctly
- [x] Exit code 0 on success

### Sample Output (Table Format)
```
┌───────────┬──────────┬────────────────────────┬─────────────┬──────────┬───────────┬───────┬─────────────┬────────────────┬──────────────┐
│ SessionNo │ UserName │ LogonTime              │ PEstate     │ AMPState │ AMPCPUSec │ AMPIO │ ReqSpool    │ Amp CPU Skew % │ Amp IO Skew %│
├───────────┼──────────┼────────────────────────┼─────────────┼──────────┼───────────┼───────┼─────────────┼────────────────┼──────────────┤
│      1076 │ DBC      │ 2026/01/27 15:33:26.00 │ IDLE        │ IDLE     │         0 │     6 │           0 │           [--] │         [--] │
└───────────┴──────────┴────────────────────────┴─────────────┴──────────┴───────────┴───────┴─────────────┴────────────────┴──────────────┘

1 session found (Query time: 0.123s)
```

### Sample Output (CSV Format)
```csv
SessionNo,UserName,LogonTime,PEstate,AMPState,AMPCPUSec,AMPIO,ReqSpool,Amp CPU Skew %,Amp IO Skew %
1076,DBC,2026/01/27 15:33:26.00,IDLE,IDLE,0,6,0,,
1078,DBC,2026/01/27 15:33:28.00,ACTIVE,ACTIVE,366.736,75335,26753187840,2.87,3.78
```

### Sample Output (JSON Format)
```json
[
  {
    "SessionNo": 1076,
    "UserName": "DBC",
    "LogonTime": "2026/01/27 15:33:26.00",
    "PEstate": "IDLE",
    "AMPState": "IDLE",
    "AMPCPUSec": 0.0,
    "AMPIO": 6,
    "ReqSpool": 0,
    "Amp CPU Skew %": null,
    "Amp IO Skew %": null
  }
]
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** [quality-validator or manual tester]
**Build Version:** [Commit hash]

**Actual Output (Table):**
```
[Paste actual table output here]
```

**Actual Output (CSV):**
```
[Paste actual CSV output here]
```

**Actual Output (JSON):**
```
[Paste actual JSON output here]
```

**Exit Code:** [Value]

**Observations:**
- [Note any differences from expected output]
- [Note any formatting issues]

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Defects Found:**
- [List any bugs discovered]

---

## Notes

- This test validates batch mode (no REPL interaction)
- Can be automated in integration tests
- CSV and JSON parsing should validate structure
- Exit code 0 is critical for scripting use cases

---

## Related Requirements

- AC-2: `tq sessions` flag works in batch mode
- AC-3: Output displays 10 columns
- AC-10: Works with all output formats (csv, json, table)
- REQ-SESS-007: Output format compatibility
