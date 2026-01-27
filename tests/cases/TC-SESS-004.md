# TC-SESS-004: Tab Completion Includes /sessions

**Test Case ID:** TC-SESS-004
**Feature:** Sessions Command - Tab Completion
**Test Type:** Interactive (PTY)
**Priority:** P0
**Created:** 2026-01-27

---

## Objective

Verify that `/sessions` and `/s` alias appear in tab completion suggestions when typing `/s<TAB>` in the REPL.

---

## Prerequisites

- [ ] tq installed and accessible
- [ ] Live Teradata database available
- [ ] TQ_LOGON environment variable set

---

## Test Steps

### Step 1: Start REPL
**Action:** Launch tq in REPL mode
```bash
tq repl
```

**Expected Result:**
- REPL starts successfully
- Prompt shows: `tq>`

### Step 2: Test `/s<TAB>` Completion
**Action:** Type `/s` and press TAB key
```
tq> /s<TAB>
```

**Expected Result:**
- Completion menu appears
- Menu includes at least two items:
  - `/sample` (existing command)
  - `/sessions` (new Sprint 26 command)
- Menu shows descriptions for each command

### Step 3: Verify `/sessions` Description
**Action:** Inspect completion menu text
```
Expected menu format:
    /sample      Show random sample
    /sessions    List active Teradata sessions with performance metrics
```

**Expected Result:**
- `/sessions` has clear description
- Description mentions "sessions" and "performance metrics"

### Step 4: Test `/sess<TAB>` Completion
**Action:** Type `/sess` and press TAB key
```
tq> /sess<TAB>
```

**Expected Result:**
- Auto-completes to `/sessions` (unambiguous match)
- OR shows filtered menu with only `/sessions`
- No menu if unambiguous, cursor after `/sessions`

### Step 5: Test Full `/sessions<TAB>` Completion
**Action:** Type `/sessions` (complete) and press TAB
```
tq> /sessions<TAB>
```

**Expected Result:**
- No additional completion (command is complete)
- TAB does nothing or shows same `/sessions` suggestion

### Step 6: Exit REPL
**Action:** Type `/quit` and press Enter
```
tq> /quit
```

**Expected Result:**
- REPL exits cleanly

---

## Expected Results

### Success Criteria
- [x] `/s<TAB>` shows completion menu
- [x] Menu includes `/sessions` with description
- [x] `/sess<TAB>` completes to `/sessions`
- [x] Description is clear and accurate

### Sample Completion Menu
```
tq> /s<TAB>

Available metacommands:
    /sample      Show random sample of table rows
    /session     Show current session information
    /sessions    List active Teradata sessions with performance metrics
    /set         Configure REPL settings
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** [quality-validator or manual tester]
**Build Version:** [Commit hash]

**Actual Completion Menu:**
```
[Paste actual completion menu here]
```

**Observations:**
- [Note any differences from expected]
- [Note if `/sessions` appears correctly]
- [Note description accuracy]

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Defects Found:**
- [List any completion bugs]
- [List any description issues]

---

## Notes

- This test validates tab completion integration
- Requires PTY environment (expectrl or manual testing)
- May be sensitive to reedline version
- Completion order may vary (alphabetical or by frequency)

---

## Related Requirements

- AC-6: Tab completion suggests `/sessions` command
- REQ-SESS-006.1: Typing `/s<TAB>` SHALL suggest `/sessions` and `/sample`
- REQ-SESS-006.2: Typing `/sess<TAB>` SHALL auto-complete to `/sessions`
- REQ-SESS-006.4: Help text description: "List active Teradata sessions with performance metrics"
