# TC-TAB-COMPLETION-002 - Tab Completion After Qualified Name (No Pager Output)

**Test ID:** TC-TAB-COMPLETION-002
**Category:** Functionality (Interactive)
**Priority:** Critical (P0 - BLOCKING)
**Sprint:** 19
**Type:** Manual Interactive Test
**Status:** BLOCKED

---

## Context

**Sprint 18 Failure:** Tab completion tests PASSED but user reports "Page 1: records 0 - 0 total: 0" instead of completions.
**Root Cause:** Sprint 18 PTY automation did not detect pager output during tab completion.
**Sprint 19 Mission:** Verify tab completion for qualified names shows table menu WITHOUT pager debug output.

---

## Objective

Verify that pressing TAB after "database." (qualified name) in the REPL:
1. Shows a completion menu with tables from that database
2. Does NOT show pager output ("Page 1: records...")
3. Allows user to select and complete the table name
4. Works in actual terminal as user experiences it

---

## User Bug Report

From `open-bugs.md` (line 28-30):
```
tq> ? sel * from dbc.t
Page 1: records 0 - 0  total: 0
```

User pressed TAB after typing "sel * from dbc.t" and saw pager output instead of table list.

**User Expectation:**
> "Obviously we should see the completion of the word I'm typing or a choice list!!"

User expects to see tables in DBC database starting with "t" (e.g., "tables", "tablesV", etc.).

---

## Prerequisites

- [ ] tq binary built: `cargo build --release`
- [ ] Database connection configured in `.env`
- [ ] Access to DBC database (system catalog)
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

### Step 2: Type SQL Fragment with Qualified Name

**Action:** Type exactly (DO NOT press Enter):
```
sel * from dbc.t
```

Your prompt should show:
```
tq> sel * from dbc.t_
```
(cursor after "dbc.t")

### Step 3: Press TAB Key

**Action:** Press the TAB key once.

**WAIT and OBSERVE the output.**

### Step 4: Visual Inspection

**Question 1: What appears after pressing TAB?**

**Option A (CORRECT - Completion Menu):**
```
tq> sel * from dbc.t
tables
tablesV
tableKinds
[other tables starting with 't' in DBC]
```
OR completion menu showing tables in DBC starting with "t".

**Option B (BUG - Pager Output):**
```
tq> ? sel * from dbc.t
Page 1: records 0 - 0  total: 0
```
OR any output containing "Page", "records", "total".

**Your Observation:**
- [ ] Completion menu with table names (CORRECT)
- [ ] Pager output appears ("Page X: records...") (FAIL)
- [ ] Nothing happens (FAIL)
- [ ] Other: _______________

**Question 2: If completion menu appears, what does it contain?**

**Your Observation:**
- [ ] Tables from DBC database starting with "t"
- [ ] All tables from DBC (no filtering)
- [ ] Tables from wrong database
- [ ] SQL keywords
- [ ] Other: _______________

**Question 3: Are the table names qualified (dbc.tablename) or unqualified?**

**Your Observation:**
- [ ] Qualified: "dbc.tables", "dbc.tablesV"
- [ ] Unqualified: "tables", "tablesV"
- [ ] Mixed
- [ ] Other: _______________

**Question 4: Does the completion work correctly?**

**Your Observation:**
- [ ] Can select a table and it completes correctly
- [ ] Menu appears but doesn't insert selection
- [ ] Cannot interact with suggestions
- [ ] Other: _______________

### Step 5: Capture Screenshot

**Action:** Take screenshot showing the exact output after pressing TAB.

**Screenshot Requirements:**
- Show the prompt line with "sel * from dbc.t"
- Show what appeared after TAB
- Show entire visible terminal output
- Visible enough to read any "Page X: records..." if present

**Screenshot File:** Save as `tests/results/sprint-19/tab-completion-qualified-screenshot.png`

### Step 6: Test Variation - Complete a Table Name

**Action:** If completion menu appeared, try completing "tables":
- Type "a" to make "dbc.ta"
- Press TAB again
- Observe if "tables" or "tablesV" completes

**Your Observation:**
```
[Describe what happened when trying to complete "tables"]
```

### Step 7: Verify Table List Content

**Action:** List actual tables in DBC starting with "t":
```sql
tq> select tablename from dbc.tablesV where databasename = 'DBC' and tablename like 'T%' order by tablename;
```

**Expected Tables (partial list):**
- Tables
- TablesV
- TableKinds (or similar)

**Your Observation - Tables that should appear in completion:**
```
[List tables from DBC that start with 't']
```

**Did these tables appear in the completion menu?**
- [ ] YES - All expected tables present
- [ ] PARTIAL - Some missing: _______________
- [ ] NO - None appeared

---

## Expected Results

### Correct Behavior

1. **Completion Menu Appears:**
   - Displays after pressing TAB
   - Shows tables from DBC database
   - Filters to tables starting with "t"
   - NO pager output
   - NO "Page X: records..." text

2. **Completion Content:**
   - Lists DBC tables: "tables", "tablesV", etc.
   - Items match prefix "t"
   - May show qualified names (dbc.tables) or unqualified (tables)

3. **Completion Works:**
   - User can navigate suggestions
   - Selecting suggestion inserts table name
   - Can complete partial table names

### Anti-Patterns (MUST NOT Occur)

- ❌ Pager output: "Page 1: records 0 - 0 total: 0"
- ❌ Pager output: "Page X: records..."
- ❌ Any text containing "records", "total", "Page"
- ❌ Empty completion (nothing shows)
- ❌ Wrong database tables (not from DBC)
- ❌ Unfiltered list (tables not starting with "t")

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
[List all items that appeared, or N/A if no menu]
```

**4. Were tables from DBC database?**
- [ ] YES - Tables are from DBC
- [ ] NO - Tables from wrong database
- [ ] N/A - No tables shown

**5. Were tables filtered to "t*"?**
- [ ] YES - Only tables starting with "t"
- [ ] NO - Unfiltered list
- [ ] N/A - No tables shown

**6. Did completion work correctly?**
- [ ] YES - Could select and complete
- [ ] NO - Menu appeared but doesn't insert
- [ ] NO - No menu appeared

**7. Screenshot captured:**
- [ ] Screenshot: `tests/results/sprint-19/tab-completion-qualified-screenshot.png`

### Test Verdict

- [ ] ✅ PASS - Completion menu works, NO pager output, correct tables
- [ ] ❌ FAIL - Pager output present OR wrong tables OR doesn't work
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

If pager output appears or wrong tables shown:

**Pager Output Exact Text (if present):**
```
[Paste exact pager output]
```

**Completion Items Received (if wrong):**
```
[List what completion showed]
```

**Expected Items:**
```
[List what SHOULD have been shown]
```

**Log Output:**
```bash
RUST_LOG=debug ./target/release/tq repl
# [Reproduce the issue]
# [Paste relevant log lines mentioning "completion", "metadata", or "pager"]
```

---

## Comparison with Sprint 18

**Sprint 18 Tab Completion Tests:**
- ✅ Verified schema-qualified completion returns data
- ✅ Verified tables from correct database
- ✅ Unit tests for qualified name parsing
- ❌ Did NOT detect pager output during completion
- ❌ PTY automation missed the visual bug

**Sprint 19 TC-TAB-COMPLETION-002 Improvements:**
- ✅ Explicitly checks for pager output absence
- ✅ Manual testing in real terminal
- ✅ Screenshot evidence required
- ✅ Tests exact user scenario ("dbc.t[TAB]")
- ✅ Verifies table list content matches database

---

## Root Cause Hypotheses

**Why Pager Might Be Triggered:**

1. **Hypothesis A:** Metadata query runs but results displayed via pager instead of completer
2. **Hypothesis B:** Completer triggers pager accidentally when formatting suggestions
3. **Hypothesis C:** Error in completion triggers pager with empty result set
4. **Hypothesis D:** Completion context analysis bug causes pager invocation

**How This Test Will Reveal:**
Manual testing will show exact output sequence, revealing whether pager is called and why.

---

## Notes

**Key Insight:**
Sprint 18 verified that the COMPLETER returned the right DATA (tables from DBC). But it didn't verify that the USER saw a COMPLETION MENU instead of PAGER OUTPUT.

**What Makes This Test Different:**
1. Tests in real terminal (sees pager rendering)
2. Screenshot proves no pager output
3. Human validates visual appearance
4. Tests exact string user reported: "dbc.t"

**Related Code:**
- `src/commands/repl/metadata_completer.rs`: Completion logic
- `src/commands/repl/pager.rs`: Source of "Page X: records..." (line 480-486)
- Pager should NEVER be called during tab completion

---

## Exit Code

N/A (REPL mode)

---

## Related Tests

- **TC-TAB-COMPLETION-001**: Tab completion after FROM
- **TC-LOGO-002**: Logo ASCII art verification
- **Sprint 18 TC-COMPLETION-004**: Qualified name completion (gave false positive)

---

## References

- Bug Report: `docs/builder/incoming/open-bugs.md` (lines 28-30)
- Sprint Planning: `docs/builder/sprints/sprint-19-planning.md`
- Sprint 18 Results: `tests/results/sprint-18/REPORT.md` (false positive)
- Metadata Completer: `src/commands/repl/metadata_completer.rs`
- Pager Module: `src/commands/repl/pager.rs`

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-22 | 1.0 | Initial test case for Sprint 19 (RETRY) | quality-validator |
