# TC-TAB-COMPLETION-003 - Tab Completion Without Pager Output (Sprint 20)

**Test ID:** TC-TAB-COMPLETION-003
**Category:** Functionality (Interactive)
**Priority:** Critical (P0 - BLOCKING)
**Sprint:** 20
**Type:** Hybrid (Interactive Automated + Manual Visual)
**Status:** PENDING

---

## Context

**Sprint 18 Failure:** Tab completion tests PASSED but user reports "Page 1: records 0 - 0 total: 0" pager output instead of completion menu.
**Sprint 19 Failure:** Manual-only tests left validation pending, user still reports issues.
**Root Cause:** PTY automation didn't catch pager output appearing during tab completion. Sprint 19 implemented OutputSuppressor but requires validation.
**Sprint 20 Mission:** Verify tab completion shows database/table names WITHOUT pager output through hybrid testing (automated + manual).

---

## Objective

Verify that pressing TAB after "select * from " in the REPL:
1. Shows a completion menu with database/table names
2. Does NOT show pager output ("Page 1: records 0 - 0 total: 0")
3. Completion menu is usable and functional
4. Works correctly in actual terminal as user experiences it
5. OutputSuppressor mechanism prevents teradatarustapi pager output

---

## User Bug Report

From `incoming/open-bugs.md` (lines 24-40):
> "If I press tab after `select * from ` I get:
> ```
> tq> ? select * from
> Page 1: records 0 - 0  total: 0
> ```
> You story about teradatarustapi is writing directly to TTY doesn't make any sense to me since the query functionality works well otherwise and uses the same drivers..."

**User Expectation:**
> "Obviously we should see the completion of the word I'm typing or a choice list!!"

**User's Recommended Solution:**
- Cache database names at startup
- Cache table names incrementally
- Suppress pager output during metadata queries
- Show proper completion menu

---

## Prerequisites

- [ ] tq binary built: `cargo build --release`
- [ ] Database connection configured in `.env` with accessible databases/tables
- [ ] Database has at least 2-3 accessible databases
- [ ] Database has at least 5-10 tables in one database
- [ ] Terminal with interactive keyboard support
- [ ] Screenshot capture tool available

---

## Test Procedure

### Part 1: Automated Component (expectrl)

**Purpose:** Provide automated regression detection and safety net.

**Execution:** Run via `cargo test --test interactive_tests test_tab_completion_no_pager -- --ignored`

**What Automated Test Validates:**
- [ ] REPL starts successfully and connects to database
- [ ] Can send "select * from " without errors
- [ ] Can send TAB key via PTY
- [ ] Pager text "Page" does NOT appear in output (negative assertion)
- [ ] Pager text "records" does NOT appear in output (negative assertion)
- [ ] Pager text "total" does NOT appear in output (negative assertion)
- [ ] Database names DO appear in output (positive assertion)
- [ ] SQL keywords (SELECT, FROM) do NOT appear as completions

**Automated Test Pass Criteria:**
- [ ] Test executes without errors
- [ ] All negative assertions pass (NO pager text detected)
- [ ] All positive assertions pass (database names present)
- [ ] REPL responds correctly to TAB input

**Note:** Automated test validates captured output, NOT visual appearance. Manual validation is REQUIRED to confirm user-facing experience.

---

### Part 2: Manual Component (User Validation)

**CRITICAL:** This MUST be done in an ACTUAL terminal by a human, NOT automated PTY test.

#### Step 1: Start REPL in Real Terminal

```bash
./target/release/tq repl
```

Wait for connection to complete and prompt to appear: `tq>`

#### Step 2: Type SQL Fragment (DO NOT PRESS ENTER)

**Action:** Type exactly (WITHOUT pressing Enter):
```
select * from
```

Your prompt should show:
```
tq> select * from _
```
(cursor after "from ")

**Important:** Type the trailing space after "from". DO NOT press Enter.

#### Step 3: Press TAB Key

**Action:** Press the TAB key once.

**WAIT and OBSERVE the output carefully.**

#### Step 4: Visual Inspection Checklist

**Question 1: What appears immediately after pressing TAB?**

**Option A (CORRECT - Completion Menu):**
```
tq> select * from
database1    database2    database3
table1       table2       table3
```
OR
```
tq> select * from dem
demo_user    demodata
```
Completion menu shows databases/tables, NO pager output.

**Option B (BUG - Pager Output):**
```
tq> select * from
Page 1: records 0 - 0  total: 0
```
OR any text containing "Page", "records", "total", "[FULL]".

**Option C (Other):**
```
[Nothing appears / Error message / Other behavior]
```

**Your Observation:**
- [ ] Completion menu appears (Option A - CORRECT)
- [ ] Pager output appears (Option B - FAIL - BUG STILL PRESENT)
- [ ] Nothing happens (Option C - FAIL)
- [ ] Other behavior: _______________

**Question 2: If pager output appears, what is the EXACT text?**

**Your Observation (if pager output detected):**
```
[Paste exact pager output text here]
```

**Question 3: If completion menu appears, what does it contain?**

**Expected Content:**
- Database names (e.g., "demo_user", "dbc", "information_schema")
- Table names from current database (if applicable)
- Mix of databases and tables

**Your Observation (if completion menu appeared):**
- [ ] Database names present
- [ ] Table names present
- [ ] Both databases and tables present
- [ ] SQL keywords (SELECT, FROM) present (FAIL if checked)
- [ ] Empty completion (nothing shown)
- [ ] Other: _______________

**List visible completions:**
```
[List 5-10 visible completion suggestions]
```

**Question 4: Can you interact with the completion menu?**

**Your Observation:**
- [ ] Can navigate suggestions with arrow keys or Tab
- [ ] Can select a suggestion and it inserts into prompt
- [ ] Suggestions are visible but not selectable
- [ ] Cannot interact with suggestions
- [ ] N/A - No menu appeared

#### Step 5: Test Qualified Name Completion

**Action:** If basic completion worked, test qualified name completion:

1. Clear the line (Ctrl+U)
2. Type: `select * from dbc.`
3. Press TAB

**Expected Behavior:**
- Completion shows tables in "dbc" database
- NO pager output

**Your Observation:**
- [ ] Shows tables in dbc database (CORRECT)
- [ ] Shows pager output (FAIL)
- [ ] Nothing happens (FAIL)
- [ ] Other: _______________

#### Step 6: Capture Screenshot

**Action:** Take screenshot showing the completion output after pressing TAB.

**Screenshot Requirements:**
- Show the prompt line with "select * from "
- Show what appeared after pressing TAB
- Show entire visible terminal output
- Clear enough to read any "Page X: records..." if present
- Show completion menu if present

**Screenshot File Path:** `tests/results/sprint-20/screenshots/tab-completion-no-pager.png`

**Capture Method:**
- **macOS:** Cmd+Shift+4, select terminal area, save to above path
- **Linux:** `gnome-screenshot -a` or `scrot -s`, save to above path
- **Windows:** Snipping Tool, save to above path

**Screenshot Captured:**
- [ ] Screenshot saved to `tests/results/sprint-20/screenshots/tab-completion-no-pager.png`

---

## Expected Results

### Correct Behavior (MUST ALL BE TRUE)

1. **Completion Menu Appears:**
   - Displays immediately after pressing TAB
   - Shows database names and/or table names
   - NO pager output visible
   - NO "Page X: records..." text
   - NO "[FULL]" indicator
   - NO "total: 0" text

2. **Completion Content:**
   - Lists available databases (e.g., "demo_user", "dbc", "information_schema")
   - May list tables from current database
   - Items are relevant to SQL context (databases after FROM)
   - NO SQL keywords (SELECT, FROM) in completion

3. **Completion Usability:**
   - User can navigate suggestions (arrow keys or Tab)
   - Selecting suggestion inserts text at cursor
   - Can filter by typing more characters
   - Completion menu dismisses gracefully (Esc or continue typing)

4. **Qualified Name Completion:**
   - After "dbc.", completion shows tables in "dbc" database
   - NO pager output during qualified name completion

### Anti-Patterns (MUST NOT OCCUR)

- ❌ Pager output: "Page 1: records 0 - 0 total: 0"
- ❌ Pager output: "Page X: records... [FULL]"
- ❌ Any text containing "records", "total", "Page"
- ❌ Empty completion (nothing shows when databases exist)
- ❌ SQL keywords only (no databases/tables)
- ❌ Crash or error during completion
- ❌ Hang or freeze when pressing TAB

---

## Actual Results

**Test Execution Date:** _______________
**Tester:** _______________
**Terminal:** _______________ (e.g., iTerm2, Terminal.app, gnome-terminal)
**OS:** _______________ (e.g., macOS 13.5, Ubuntu 22.04)
**Database:** _______________ (e.g., Teradata 20.00)
**tq Version:** _______________

### Automated Test Results

**Execution Command:**
```bash
cargo test --test interactive_tests test_tab_completion_no_pager -- --ignored
```

**Test Output:**
```
[Paste test execution output here]
```

**Automated Test Verdict:**
- [ ] ✅ PASS - All automated assertions passed (no pager text, databases present)
- [ ] ❌ FAIL - Automated test failed (see output above)
- [ ] ⛔ BLOCKED - Cannot execute automated test (database unavailable)

### Manual Test Results

**1. What appeared after TAB?**
```
[Describe or paste the exact output]
```

**2. Pager output detected?**
- [ ] NO pager output (CORRECT)
- [ ] YES pager output detected (FAIL)

**If pager output detected, exact text:**
```
[Paste exact pager output if present]
```

**3. Completion menu content:**
- [ ] Database names present (CORRECT)
- [ ] Table names present (CORRECT)
- [ ] SQL keywords present (FAIL)
- [ ] Empty/nothing (FAIL)

**List visible completions:**
```
[List completions that appeared]
```

**4. Completion usability:**
- [ ] Can navigate and select (CORRECT)
- [ ] Visible but not usable (FAIL)
- [ ] N/A - No menu appeared

**5. Qualified name completion (dbc.):**
- [ ] Shows tables in dbc database (CORRECT)
- [ ] Shows pager output (FAIL)
- [ ] Nothing happens (FAIL)
- [ ] Not tested

**6. Screenshot:**
- [ ] Screenshot captured: `tests/results/sprint-20/screenshots/tab-completion-no-pager.png`

### Final Verdict

**Hybrid Test Verdict:**
- [ ] ✅ PASS - Both automated AND manual tests passed, NO pager output
- [ ] ❌ FAIL - At least one test failed (see failures below)
- [ ] ⛔ BLOCKED - Cannot execute tests (see blockers below)

**Failures (if FAIL):**
```
[List specific failures from automated or manual testing]
```

**Blockers (if BLOCKED):**
```
[List blockers preventing test execution]
```

---

## Unit Test Coverage

**Automated Unit Tests (Recommended):**

Unit tests can validate the OutputSuppressor mechanism without requiring database:

**Test Location:** `src/db/metadata.rs` test module

**Test Cases:**
1. **Verify OutputSuppressor redirects stdout:**
   - Create OutputSuppressor instance
   - Write to stdout during suppression
   - Verify output is NOT visible

2. **Verify OutputSuppressor redirects stderr:**
   - Create OutputSuppressor instance
   - Write to stderr during suppression
   - Verify output is NOT visible

3. **Verify OutputSuppressor restores on drop:**
   - Create OutputSuppressor, suppress output
   - Drop OutputSuppressor
   - Verify stdout/stderr restored

4. **Verify OutputSuppressor gracefully handles errors:**
   - Test fd operations that may fail
   - Verify no panics or crashes

**Unit Test Commands:**
```bash
# Run OutputSuppressor unit tests (Unix only)
cargo test --lib output_suppressor

# Or run all metadata unit tests
cargo test --lib metadata
```

**Note:** Unit tests validate mechanism, NOT user-facing behavior. Manual validation is still REQUIRED.

**Additional Interactive Tests (Recommended):**

**Test Location:** `tests/interactive_tests.rs`

**Test Cases:**
1. **test_tab_completion_shows_databases:**
   - Spawn REPL with database
   - Send "select * from "
   - Send TAB
   - Assert database names appear in output
   - Assert "Page" NOT in output

2. **test_tab_completion_qualified_name:**
   - Spawn REPL with database
   - Send "select * from dbc."
   - Send TAB
   - Assert table names appear
   - Assert "Page" NOT in output

3. **test_database_cache_loads:**
   - Verify cache loading mechanism
   - Check that databases are available for completion

**Interactive Test Commands:**
```bash
# Run all tab completion interactive tests
cargo test --test interactive_tests completion -- --ignored --test-threads=1
```

---

## Comparison with Previous Sprints

**Sprint 18 Tab Completion Tests:**
- ✅ Verified Tab triggers completion mechanism
- ✅ Verified databases/tables in suggestions data
- ✅ Verified span calculation
- ❌ Did NOT detect pager output during completion
- ❌ PTY automation missed the actual user-facing bug

**Sprint 19 TC-TAB-COMPLETION-001:**
- ✅ Explicitly checked for pager output absence
- ✅ Manual testing in real terminal
- ✅ Screenshot evidence required
- ❌ Manual-only tests blocked AI agent execution
- ❌ No automated safety net for regression detection

**Sprint 20 TC-TAB-COMPLETION-003 Improvements:**
- ✅ Hybrid testing (automated + manual)
- ✅ Automated negative assertions (pager text NOT present)
- ✅ Automated positive assertions (databases present)
- ✅ Manual validation confirms visual absence of pager
- ✅ Screenshot evidence mandatory
- ✅ Both qualified and unqualified name completion tested
- ✅ Unit tests for OutputSuppressor mechanism

---

## Root Cause and Solution

**Root Cause Analysis:**

teradatarustapi (the Go library) writes pager output directly to stdout/stderr when executing metadata queries. During tab completion, tq executes queries like:
```sql
SELECT DatabaseName FROM DBC.DatabasesV;
SELECT TableName FROM DBC.TablesV WHERE DatabaseName = 'dbc';
```

The pager output "Page 1: records 0 - 0 total: 0" appears because teradatarustapi treats these queries like normal queries and displays pager information.

**Solution (Sprint 19 Implementation):**

OutputSuppressor struct (in `src/db/metadata.rs`) that:
1. Redirects stdout and stderr to /dev/null (Unix) or NUL (Windows)
2. Executes metadata query while output is suppressed
3. Restores stdout/stderr on drop
4. Allows tq to capture query results without pager output appearing in terminal

**What This Test Validates:**

This test confirms that OutputSuppressor works correctly and pager output does NOT appear during tab completion in the user's actual terminal.

---

## Debugging Information

If pager output still appears, capture this information:

**Pager Output Exact Text:**
```
[Paste exact pager output if it appears]
```

**Log Output (if available):**
```bash
RUST_LOG=debug ./target/release/tq repl
# [Type: select * from ]
# [Press TAB]
# [Paste relevant log lines showing metadata query execution]
```

**Environment Information:**
```bash
# OS
uname -a

# Terminal
echo $TERM

# Shell
echo $SHELL

# Rust version
rustc --version

# tq version
./target/release/tq --version
```

---

## Notes

**Why Hybrid Testing is Required:**

**Automated Component:**
- Provides regression detection (catches if pager output reappears)
- Validates technical requirements (no pager text in captured output)
- Can run in CI/CD pipelines
- Fast feedback loop for developers

**Manual Component:**
- Only humans see actual terminal output as user sees it
- Confirms pager text visually absent (not just absent from captured PTY output)
- Validates completion menu usability
- Screenshot provides irrefutable evidence
- Tests in same environment user uses

**Together:** Automated tests prevent regressions, manual tests confirm user experience is correct.

**Critical Success Factor:**
This test is NOT complete until BOTH automated and manual components PASS. Automated-only pass is NOT sufficient (learned from Sprint 18/19).

---

## Exit Code

N/A (REPL mode, exit code not relevant for tab completion)

---

## Related Tests

- **TC-TAB-COMPLETION-001** (Sprint 19): Manual-only test for tab completion
- **TC-TAB-COMPLETION-002** (Sprint 19): Manual-only test for qualified name completion
- **TC-LOGO-003** (Sprint 20): Logo display verification
- **TC026-TC030** (Sprint 7): Original tab completion implementation tests

---

## References

- Bug Report: `incoming/open-bugs.md` (lines 24-40)
- Sprint Planning: `docs/sprints/sprint-20-planning.md` (lines 72-106)
- Test Strategy: `tests/strategy/sprint-20-test-strategy.md` (lines 293-594)
- REPL Specification: `docs/specifications/repl.md#tab-completion`
- Design Document: `docs/design/repl.md` (tab completion architecture)
- OutputSuppressor Implementation: `src/db/metadata.rs` (lines 17-30)

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-23 | 1.0 | Initial test case for Sprint 20 - Hybrid tab completion testing | quality-validator |
