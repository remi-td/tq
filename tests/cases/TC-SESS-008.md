# TC-SESS-008: Output Format Compatibility (CSV, JSON, Table)

**Test Case ID:** TC-SESS-008
**Feature:** Sessions Command - Output Formats
**Test Type:** Integration (Format Compatibility)
**Priority:** P0
**Created:** 2026-01-27

---

## Objective

Verify that `/sessions` command works correctly with all output formats (table, CSV, JSON) and handles NULL skew values appropriately in each format.

---

## Prerequisites

- [ ] tq installed and accessible
- [ ] Live Teradata database available
- [ ] TQ_LOGON environment variable set
- [ ] User has SELECT privilege on DBC.MonitorSession

---

## Test Steps

### Test 1: Table Format (Default)

#### Step 1: Execute in REPL with Table Format
**Action:** Run `/sessions` in REPL
```bash
tq repl
tq> /sessions
```

**Expected Result:**
- Table output with box-drawing characters
- NULL skew displayed as `[--]` (not blank, not "null")
- Numeric columns right-aligned
- String columns left-aligned

#### Step 2: Verify NULL Skew Display
**Action:** Find IDLE session row in output
```
Expected row with IDLE state:
│      1076 │ DBC      │ ... │ IDLE │ IDLE │ ... │           [--] │         [--] │
```

**Expected Result:**
- Skew columns show `[--]` for IDLE sessions
- Skew columns show `X.XX` for ACTIVE sessions

---

### Test 2: CSV Format

#### Step 1: Execute in Batch Mode with CSV
**Action:** Run `tq sessions --format csv`
```bash
tq sessions --format csv
```

**Expected Result:**
- CSV header row: `SessionNo,UserName,LogonTime,PEstate,AMPState,AMPCPUSec,AMPIO,ReqSpool,Amp CPU Skew %,Amp IO Skew %`
- Data rows with comma-separated values
- NULL skew as empty field (two consecutive commas)

#### Step 2: Verify CSV NULL Handling
**Action:** Inspect CSV output for IDLE session
```
Expected CSV row with IDLE state:
1076,DBC,2026/01/27 15:33:26.00,IDLE,IDLE,0,6,0,,
                                                  ^^
                                    Empty fields for NULL skew
```

**Expected Result:**
- Empty field between commas for NULL skew
- NOT the string "NULL" or "[--]"

#### Step 3: Parse and Validate CSV
**Action:** Parse CSV output with standard CSV parser
```python
import csv
import io

csv_output = """SessionNo,UserName,...
1076,DBC,...,,,
1078,DBC,...,2.87,3.78
"""

reader = csv.DictReader(io.StringIO(csv_output))
for row in reader:
    assert 'Amp CPU Skew %' in row
    # Empty string for NULL, not None
```

**Expected Result:**
- CSV parses without errors
- NULL skew fields are empty strings
- Valid CSV structure

---

### Test 3: JSON Format

#### Step 1: Execute in Batch Mode with JSON
**Action:** Run `tq sessions --format json`
```bash
tq sessions --format json
```

**Expected Result:**
- Valid JSON array
- Each element is an object with 10 keys
- NULL skew as `null` (JSON null, not string)

#### Step 2: Verify JSON NULL Handling
**Action:** Inspect JSON output for IDLE session
```json
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
```

**Expected Result:**
- `"Amp CPU Skew %": null` (JSON null)
- `"Amp IO Skew %": null` (JSON null)
- NOT string "null" or "[--]"

#### Step 3: Parse and Validate JSON
**Action:** Parse JSON output with standard JSON parser
```python
import json

json_output = """[{"SessionNo": 1076, ..., "Amp CPU Skew %": null}]"""

data = json.loads(json_output)
assert isinstance(data, list)
assert len(data) > 0
assert data[0]["Amp CPU Skew %"] is None  # Python None = JSON null
```

**Expected Result:**
- JSON parses without errors
- NULL skew fields are JSON `null` (None in Python)
- Valid JSON structure
- Numeric fields are numbers (not strings)

---

### Test 4: Format Switching in REPL

#### Step 1: Switch Format in REPL
**Action:** Use `/set format csv` then `/sessions`
```bash
tq repl
tq> /set format csv
Output format set to: csv

tq> /sessions
SessionNo,UserName,LogonTime,...
1076,DBC,2026/01/27 15:33:26.00,...
```

**Expected Result:**
- CSV output in REPL
- Same NULL handling as batch mode

#### Step 2: Switch to JSON
**Action:** Use `/set format json` then `/sessions`
```bash
tq> /set format json
Output format set to: json

tq> /sessions
[
  {"SessionNo": 1076, ...}
]
```

**Expected Result:**
- JSON output in REPL
- Same NULL handling as batch mode

---

## Expected Results

### Success Criteria
- [x] Table format displays NULL skew as `[--]`
- [x] CSV format displays NULL skew as empty field (two commas)
- [x] JSON format displays NULL skew as JSON `null`
- [x] All formats produce valid, parseable output
- [x] Format switching works in REPL
- [x] Numeric fields in JSON are numbers (not strings)

### Format Summary Table

| Format | NULL Skew Representation | Example |
|--------|-------------------------|---------|
| Table  | `[--]` (visual indicator) | `│ [--] │ [--] │` |
| CSV    | Empty field (no value between commas) | `...,,,` |
| JSON   | JSON `null` | `"Amp CPU Skew %": null` |

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** [quality-validator or manual tester]
**Build Version:** [Commit hash]

**Actual CSV Output:**
```
[Paste CSV output here]
```

**Actual JSON Output:**
```
[Paste JSON output here]
```

**CSV Parse Result:** [Success/Failure]
**JSON Parse Result:** [Success/Failure]

**Observations:**
- [Note any format-specific issues]
- [Note NULL handling accuracy]

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Defects Found:**
- [List any format compatibility bugs]
- [List any NULL serialization issues]

---

## Notes

- Each format has specific NULL representation requirements
- CSV parsers expect empty fields for NULL
- JSON parsers expect `null` keyword
- Table format is human-readable with `[--]` indicator
- Format compatibility is critical for scripting use cases

---

## Related Requirements

- AC-10: Works with all output formats (--format csv, json, table)
- REQ-SESS-007.2: CSV format: NULL skew as empty string
- REQ-SESS-007.3: JSON format: NULL skew as `null`
- repl.md lines 1689-1723: Format examples
