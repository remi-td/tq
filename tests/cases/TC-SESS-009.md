# TC-SESS-009: Aliases (/s) Work Correctly

**Test Case ID:** TC-SESS-009
**Feature:** Sessions Command - Alias Support
**Test Type:** Interactive (PTY)
**Priority:** P0
**Created:** 2026-01-27

---

## Objective

Verify that the `/s` alias executes identically to the full `/sessions` command in both REPL and batch modes.

---

## Prerequisites

- [ ] tq installed and accessible
- [ ] Live Teradata database available
- [ ] TQ_LOGON environment variable set
- [ ] User has SELECT privilege on DBC.MonitorSession

---

## Test Steps

### Test 1: REPL Mode Alias

#### Step 1: Start REPL
**Action:** Launch tq in REPL mode
```bash
tq repl
```

**Expected Result:**
- REPL starts successfully
- Prompt shows: `tq>`

#### Step 2: Execute /s Alias
**Action:** Type `/s` and press Enter
```
tq> /s
```

**Expected Result:**
- Same output as `/sessions`
- Table displays with 10 columns
- Sessions listed correctly
- Footer shows session count

#### Step 3: Compare with /sessions
**Action:** Execute both commands and compare
```
tq> /sessions
[Output 1]

tq> /s
[Output 2]
```

**Expected Result:**
- Outputs are identical (except timestamps may differ)
- Same column structure
- Same formatting
- Same data

#### Step 4: Test Case Insensitivity
**Action:** Try uppercase alias
```
tq> /S
```

**Expected Result:**
- Works identically to `/s` (case-insensitive)

---

### Test 2: Help Text Documents Alias

#### Step 1: Check Help Output
**Action:** Execute `/help` and search for `/sessions`
```
tq> /help
```

**Expected Result:**
- Help text shows: `/sessions, /s` (both forms documented)
- OR separate lines for each form
- User can discover the alias from help

---

### Test 3: Tab Completion for Alias

#### Step 1: Type /s and Press TAB
**Action:** Tab completion test
```
tq> /s<TAB>
```

**Expected Result:**
- Completion menu includes `/s` or `/sessions`
- Typing `/s` followed by TAB shows both `/sample` and `/sessions`

---

### Test 4: Exit REPL
**Action:** Type `/quit` and press Enter
```
tq> /quit
```

**Expected Result:**
- REPL exits cleanly

---

## Expected Results

### Success Criteria
- [x] `/s` executes identically to `/sessions`
- [x] Output format is the same
- [x] Data content is the same
- [x] Case-insensitive (`/S` works)
- [x] Help text documents the alias
- [x] Tab completion works for alias

### Output Comparison
```
tq> /sessions
[Standard sessions table output]

tq> /s
[Identical sessions table output]
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** [quality-validator or manual tester]
**Build Version:** [Commit hash]

**Output from /sessions:**
```
[Paste output here]
```

**Output from /s:**
```
[Paste output here]
```

**Difference Check:** [Identical/Different]

**Observations:**
- [Note any differences between /sessions and /s]
- [Note if alias is case-sensitive]

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Defects Found:**
- [List any alias issues]
- [List any behavioral differences]

---

## Notes

- Alias should be functionally equivalent to full command
- Common pattern in SQL CLIs (e.g., `\d` in psql)
- Improves user experience for frequently-used commands
- Both forms should be documented

---

## Related Requirements

- AC-1: `/sessions` command available in REPL with `/s` alias
- REQ-SESS-001.2: Short alias: `/s`
- REQ-SESS-001.3: Both forms SHALL execute identically
- REQ-SESS-001.5: Case-insensitive
