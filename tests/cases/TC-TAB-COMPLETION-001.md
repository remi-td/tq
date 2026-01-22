# TC-TAB-COMPLETION-001 - Tab Completion After FROM (No Pager Output)

**Test ID:** TC-TAB-COMPLETION-001
**Category:** Functionality (Interactive)
**Priority:** Critical (P0 - BLOCKING)
**Sprint:** 19
**Type:** Manual Interactive Test
**Status:** BLOCKED

---

## Context

**Sprint 18 Failure:** Tab completion tests PASSED but user reports "Page 1: records 0 - 0 total: 0 [FULL]" instead of completions.
**Root Cause:** Sprint 18 PTY automation did not detect pager output during tab completion.
**Sprint 19 Mission:** Verify tab completion shows menu WITHOUT pager debug output.

---

## Objective

Verify that pressing TAB after "FROM " in the REPL:
1. Shows a completion menu with database/table names
2. Does NOT show pager output ("Page 1: records...")
3. Allows user to select and complete the name
4. Works in actual terminal as user experiences it

---

## User Bug Report

From `open-bugs.md` (line 21-24):
```
tq> ? sel * fr
Page 1: records 0 - 0  total: 0  [FULL]
```

User pressed TAB after typing "sel * fr" and saw pager output instead of completion menu.

**User Expectation:**
> "Obviously we should see the completion of the word I'm typing or a choice list!!"

---

## Prerequisites

- [ ] tq binary built: `cargo build --release`
- [ ] Database connection configured in `.env` with accessible databases/tables
- [ ] Terminal with interactive keyboard support
- [ ] Screenshot capture tool available

---

## Test Procedure

### Step 1: Start REPL in Real Terminal

**CRITICAL:** This MUST be done in an ACTUAL terminal, NOT automated test.

```bash
./target/release/tq repl
```

Wait for prompt: `tq>`

### Step 2: Type SQL Fragment

**Action:** Type exactly (DO NOT press Enter):
```
sel * fr
```

Your prompt should show:
```
tq> sel * fr_
```
(cursor after "fr")

### Step 3: Press TAB Key

**Action:** Press the TAB key once.

**WAIT and OBSERVE the output.**

### Step 4: Visual Inspection

**Question 1: What appears after pressing TAB?**

**Option A (CORRECT - Completion Menu):**
```
tq> sel * FROM
[Database list or "FROM" completion shown]
```
OR
```
tq> sel * fr
database1
database2
table1
table2
```

**Option B (BUG - Pager Output):**
```
tq> ? sel * fr
Page 1: records 0 - 0  total: 0  [FULL]
```
OR any output containing "Page", "records", "total".

**Your Observation:**
- [ ] Completion menu appears (CORRECT)
- [ ] Pager output appears ("Page X: records...") (FAIL)
- [ ] Nothing happens (FAIL)
- [ ] Other: _______________

**Question 2: If completion menu appears, what does it contain?**

**Your Observation:**
- [ ] Database names (e.g., "demo_user", "dbc", "information_schema")
- [ ] Table names from current database
- [ ] SQL keywords (e.g., "FROM", "SELECT")
- [ ] Mix of databases and tables
- [ ] Other: _______________

**Question 3: Does the completion work correctly?**

**Your Observation:**
- [ ] Can select a suggestion and it completes correctly
- [ ] Suggestion appears but doesn't insert
- [ ] Cannot interact with suggestions
- [ ] Other: _______________

### Step 5: Capture Screenshot

**Action:** Take screenshot showing the exact output after pressing TAB.

**Screenshot Requirements:**
- Show the prompt line with "sel * fr"
- Show what appeared after TAB
- Show entire visible terminal output
- Visible enough to read any "Page X: records..." if present

**Screenshot File:** Save as `tests/results/sprint-19/tab-completion-from-screenshot.png`

### Step 6: Test Variation - Complete the Word

**Action:** If completion menu appeared, try completing "from":
- Type additional letter(s) to make "fro" or "from"
- Press TAB again
- Observe if "FROM" completes

**Your Observation:**
```
[Describe what happened when trying to complete "FROM"]
```

---

## Expected Results

### Correct Behavior

1. **Completion Menu Appears:**
   - Displays after pressing TAB
   - Shows database names and/or tables
   - NO pager output
   - NO "Page X: records..." text

2. **Completion Content:**
   - Lists available databases (e.g., "demo_user", "dbc")
   - May list tables from current database
   - Items are selectable/usable

3. **Completion Works:**
   - User can navigate suggestions
   - Selecting suggestion inserts text at cursor
   - Can complete partial words

### Anti-Patterns (MUST NOT Occur)

- ❌ Pager output: "Page 1: records 0 - 0 total: 0"
- ❌ Pager output: "Page X: records... [FULL]"
- ❌ Any text containing "records", "total", "Page"
- ❌ Empty completion (nothing shows)
- ❌ Keywords only (no databases/tables)

---

## Actual Results

**Test Execution Date:** _______________
**Tester:** _______________
**Terminal:** _______________
**Database:** _______________
**tq Version:** _______________

### Observations

**1. What appeared after TAB?**
```
[Paste or describe the exact output]
```

**2. Was pager output present?**
- [ ] NO pager output (CORRECT)
- [ ] YES pager output detected: "Page X: records..." (FAIL)

**3. What was in the completion menu?**
```
[List what appeared in completion menu, or N/A if no menu]
```

**4. Did completion work correctly?**
- [ ] YES - Could select and complete
- [ ] NO - Menu appeared but doesn't work
- [ ] NO - No menu appeared

**5. Screenshot captured:**
- [ ] Screenshot: `tests/results/sprint-19/tab-completion-from-screenshot.png`

### Test Verdict

- [ ] ✅ PASS - Completion menu works, NO pager output
- [ ] ❌ FAIL - Pager output present OR completion doesn't work
- [ ] ⛔ BLOCKED - Cannot execute test

**Failure Details (if FAIL):**
```
[Describe specific failures]
```

**Blocker Details (if BLOCKED):**
```
[Describe blockers]
```

---

## Debugging Information

If pager output appears, capture this information:

**Pager Output Exact Text:**
```
[Paste exact pager output if it appears]
```

**Log Output (if available):**
```bash
RUST_LOG=debug ./target/release/tq repl
# [Reproduce the issue]
# [Paste relevant log lines]
```

---

## Comparison with Sprint 18

**Sprint 18 Tab Completion Tests:**
- ✅ Verified Tab triggers completion mechanism
- ✅ Verified databases/tables in suggestions
- ✅ Verified span calculation
- ❌ Did NOT detect pager output during completion
- ❌ PTY automation missed the actual bug

**Sprint 19 TC-TAB-COMPLETION-001 Improvements:**
- ✅ Explicitly checks for pager output absence
- ✅ Manual testing in real terminal
- ✅ Screenshot evidence required
- ✅ Tests exact user scenario ("sel * fr")

---

## Root Cause Analysis

**Why Sprint 18 Missed This Bug:**
- PTY automation may not trigger pager rendering
- Tests verified completion DATA, not visual OUTPUT
- No explicit check for pager interference
- Automated tests don't see what user sees

**Sprint 19 Approach:**
- Manual testing sees pager output
- Human validates NO pager text appears
- Screenshot proves pager absent
- Tests in same environment user uses

---

## Notes

**Critical Difference from Sprint 18:**
Sprint 18 test `test_tab_completion_shows_tables_not_keywords` checked for tables in suggestions but didn't check if PAGER OUTPUT was printed instead of/alongside the completion menu.

**What We're Really Testing:**
Not just "does completion return the right data" but "does the USER see a completion menu without pager garbage."

---

## Exit Code

N/A (REPL mode)

---

## Related Tests

- **TC-TAB-COMPLETION-002**: Tab completion after qualified name ("dbc.t")
- **TC-LOGO-002**: Logo ASCII art verification
- **Sprint 18 TC-COMPLETION-002**: Previous test that gave false positive

---

## References

- Bug Report: `docs/builder/incoming/open-bugs.md` (lines 18-32)
- Sprint Planning: `docs/builder/sprints/sprint-19-planning.md`
- Sprint 18 Results: `tests/results/sprint-18/REPORT.md` (false positive)
- Pager Code: `src/commands/repl/pager.rs` (source of "Page X: records..." output)

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-22 | 1.0 | Initial test case for Sprint 19 (RETRY) | quality-validator |
