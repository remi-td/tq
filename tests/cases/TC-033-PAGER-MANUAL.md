---
id: TC-033-PAGER-MANUAL
title: Pager Visual Rendering Validation (Manual - Issue #14)
category: Manual Validation
priority: Critical
sprint: 33
issue: 14
created: 2026-02-03
status: DOCUMENTED (NOT EXECUTED - NO HUMAN TESTER)
---

# Test Case TC-033-PAGER-MANUAL: Pager Visual Rendering Validation

## Purpose

Manually validate that the pager bug reported in Issue #14 (column misalignment, line wrapping) is resolved. This test case documents the validation procedure for future manual testing when a human tester becomes available.

**CRITICAL:** This test case is **DOCUMENTED ONLY** and will **NOT BE EXECUTED** in Sprint 33 due to lack of human tester. Sprint 33 ships with pager disabled by default (`pager_enabled: false`) to protect users from potential rendering bugs.

## Scope

**Testing:**
- Visual column alignment in pager alternate screen mode
- Line wrapping behavior at various terminal widths
- Cell truncation correctness (Sprint 31 two-pass algorithm)
- Pager rendering at Issue #14's problematic width (117 characters)

**Not Testing:**
- Pager navigation (j/k/h/l keys) - covered by automated tests
- Pager state management - covered by automated tests
- Query execution - covered by integration tests

## Background: Sprint 29/30/31 Context

**Sprint 29:** Implemented pager, claimed working, 100% test pass rate → User reported broken
**Sprint 30:** Architectural refactor, 100% test pass rate → User reported still broken
**Sprint 31:** Two-pass truncation fix, automated tests pass → Manual validation pending
**Issue #14:** User confirms pager still produces garbled output with screenshot evidence

**Root Cause (Sprint 31 Analysis):**
Cell values truncated to MAX_CELL_LENGTH (100) but display_width capped at MAX_COLUMN_WIDTH (40). Rust's format! macro does NOT truncate, causing cell overflow.

**Sprint 31 Fix:**
Two-pass algorithm: Calculate display_width (pass 1), then truncate cell to display_width (pass 2).

**Sprint 33 Approach:**
- Disable pager by default (`pager_enabled: false`)
- Document manual validation procedure (this test case)
- Ship without claiming "pager works"
- User can opt-in with `/pager on` if they want to test

## Prerequisites

- tq binary compiled in release mode
- Live Teradata database connection (TQ_LOGON configured)
- Terminal emulator with adjustable width (iTerm2, Terminal.app, GNOME Terminal, etc.)
- `script` command available for evidence capture
- Test table: `dbc.databases` (system table, always available)

## Test Procedure

### Setup

**Terminal Width Configuration:**

Test at the following terminal widths (in order of priority):

1. **117 characters** (HIGH PRIORITY) - Issue #14 reported problems at this width
2. **80 characters** (STANDARD) - Minimum standard terminal width
3. **120 characters** (STANDARD) - Common default in modern terminals
4. **160 characters** (WIDE) - Wide monitor scenario

**Terminal Width Setup Instructions:**

```bash
# iTerm2 / Terminal.app (macOS)
# 1. Open Terminal Preferences → Profiles → Window
# 2. Set Columns to desired width (117, 80, 120, or 160)
# 3. Restart terminal or create new window

# GNOME Terminal (Linux)
# 1. Right-click → Preferences → Profiles → [Your Profile]
# 2. Set default size: Columns = desired width
# 3. Restart terminal

# Verify current terminal width
tput cols  # Should output target width
```

### Test Execution Matrix

Execute the following test for EACH terminal width:

#### Test 1: Basic Pager Rendering (Terminal Width: N)

**Step 1: Start Evidence Capture**

```bash
# Capture session to file for evidence
script /tmp/tq-pager-test-width-N.txt
```

**Step 2: Launch tq REPL**

```bash
# Start tq in REPL mode
tq repl

# Verify connection
# Expected: "Connected to <database>" message
```

**Step 3: Enable Pager**

```sql
tq> /pager on
```

**Expected Output:**
```
Pager enabled.
```

**Step 4: Execute Test Query**

```sql
tq> SELECT TOP 10 * FROM dbc.databases;
```

**Expected Behavior:**
- Query executes successfully
- Pager activates (alternate screen mode)
- Table displays with columns

**Step 5: Visual Inspection Checklist**

**CRITICAL VALIDATION POINTS:**

- [ ] **Column Headers Visible:** All column headers are readable (not truncated mid-word)
- [ ] **Column Alignment:** Data values appear directly under their respective headers
- [ ] **Vertical Separators:** All `┆` characters form straight vertical lines (not jagged)
- [ ] **No Line Wrapping:** Each table row occupies ONE line, not multiple wrapped lines
- [ ] **No Overflow:** Cell content does not overflow into adjacent columns
- [ ] **Horizontal Scrolling:** If table is wider than terminal, indicator shows "(+N cols)" at right edge
- [ ] **Cell Truncation:** Long values are truncated with ellipsis (...), not causing overflow
- [ ] **Border Integrity:** Top border (═) and column separators (┆) are continuous and aligned

**FAILURE INDICATORS (Issue #14 Symptoms):**

- [ ] **FAIL:** Column headers misaligned from data (e.g., "DatabaseName" header not above database name values)
- [ ] **FAIL:** Lines wrap to next terminal line (multiple physical lines per table row)
- [ ] **FAIL:** Vertical separators (┆) are jagged or broken
- [ ] **FAIL:** Cell content overflows past column boundary into next column
- [ ] **FAIL:** Table is unreadable or requires horizontal scrolling for narrow result set

**Step 6: Test Horizontal Navigation** (if table is wider than terminal)

```
# Press 'l' (lowercase L) to scroll right
# Press 'h' to scroll back left
```

**Expected:**
- Columns scroll smoothly
- Column position indicator updates (e.g., "Columns 3-6 of 12")
- No rendering artifacts or glitches

**Step 7: Exit Pager**

```
# Press 'q' to exit pager
```

**Expected:**
- Pager exits cleanly
- Returns to REPL prompt
- No terminal state corruption

**Step 8: Test with Pager Disabled (Baseline)**

```sql
tq> /pager off
tq> SELECT TOP 10 * FROM dbc.databases;
```

**Expected:**
- Table displays correctly WITHOUT pager
- This establishes baseline: if this fails, issue is NOT pager-specific

**Step 9: Stop Evidence Capture**

```bash
# Exit tq REPL
tq> /quit

# Stop script recording
exit

# Review captured output
less /tmp/tq-pager-test-width-N.txt
```

### Test Execution Schedule

Execute the above test procedure for each terminal width:

1. **Priority 1:** 117 characters (Issue #14 width)
2. **Priority 2:** 80 characters (minimum standard)
3. **Priority 3:** 120 characters (common default)
4. **Priority 4:** 160 characters (wide monitor)

**Time Estimate:** 15-20 minutes per width, 60-80 minutes total

## Expected Results

### PASS Criteria

**For EACH terminal width tested, ALL of the following must be true:**

1. ✅ Column headers align with data columns
2. ✅ Vertical separators (┆) form straight lines
3. ✅ Each table row occupies exactly ONE terminal line (no wrapping)
4. ✅ Cell content does not overflow into adjacent columns
5. ✅ Long values are truncated with ellipsis (...) when needed
6. ✅ Horizontal scrolling works smoothly (if applicable)
7. ✅ Pager exits cleanly with 'q' key
8. ✅ No terminal state corruption after pager exit

**Visual Example of PASSING Output (117-char terminal):**

```
╭──────────────┬─────────────┬───────────┬───────────┬─────────────┬──────────────╮
│ DatabaseName ┆ CreatorName ┆ OwnerName ┆ PermSpace ┆ JournalFlag ┆ SpoolSpace   │
╞══════════════╪═════════════╪═══════════╪═══════════╪═════════════╪══════════════╡
│ All          ┆ DBC         ┆ DBC       ┆         0 ┆ NN          ┆            0 │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ DBC          ┆ DBC         ┆ DBC       ┆   1000000 ┆ NN          ┆      5000000 │
...

[Row 1 of 10] [Columns 1-6 of 12] [Press h/l to scroll, q to quit]
```

### FAIL Criteria

**If ANY of the following occur at ANY terminal width, the test FAILS:**

1. ❌ Column headers misaligned from data
2. ❌ Vertical separators (┆) are jagged or broken
3. ❌ Table rows wrap to multiple terminal lines (Issue #14 symptom)
4. ❌ Cell content overflows into adjacent columns (Issue #14 symptom)
5. ❌ Horizontal scrolling navigation broken (h/l keys don't work)
6. ❌ Pager does not exit cleanly with 'q' key
7. ❌ Terminal state corrupted after pager exit (colors broken, cursor invisible, etc.)

**Visual Example of FAILING Output (Issue #14):**

```
╭──────────────────────────────────────────────────────────────┬─────────────
│ DatabaseName                                                 ┆ CreatorName
╞══════════════════════════════════════════════════════════════╪═════════════
│ All                                                          ┆ DBC
                                                                              ┆ DBC
                    ┆         0 ┆ NN          ┆            0 │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
```

**Issue:** Lines wrapping, separators broken, columns misaligned - this is what Issue #14 reported.

## Test Report Format

After completing all tests, document results in this format:

```markdown
# TC-033-PAGER-MANUAL Test Results

**Test Date:** YYYY-MM-DD
**Tester:** [Your Name]
**tq Version:** [Version from `tq --version`]
**Build:** Release
**OS:** [macOS / Linux / Windows]
**Terminal:** [iTerm2 / GNOME Terminal / etc.]

## Test Results by Terminal Width

### Terminal Width: 117 characters (Issue #14 width)

**Result:** PASS / FAIL

**Evidence:**
- Script output: `/tmp/tq-pager-test-width-117.txt`
- Screenshot: [Attach screenshot if available]

**Visual Inspection Checklist:**
- [ ] Column alignment: PASS / FAIL
- [ ] Vertical separators: PASS / FAIL
- [ ] Line wrapping: PASS / FAIL (no wrapping = PASS)
- [ ] Cell overflow: PASS / FAIL (no overflow = PASS)
- [ ] Horizontal scrolling: PASS / FAIL / N/A
- [ ] Clean exit: PASS / FAIL

**Issues Found:**
[Describe any visual rendering problems]

### [Repeat for each terminal width: 80, 120, 160]

## Overall Verdict

**PASS:** All terminal widths passed visual inspection
**FAIL:** One or more terminal widths failed visual inspection

**Recommendation:**
- If PASS: Pager can be enabled by default in future sprint
- If FAIL: Keep pager disabled by default, continue debugging

## Evidence Files

Attach the following files to test report:
1. `/tmp/tq-pager-test-width-117.txt` (script output)
2. `/tmp/tq-pager-test-width-80.txt` (script output)
3. `/tmp/tq-pager-test-width-120.txt` (script output)
4. `/tmp/tq-pager-test-width-160.txt` (script output)
5. Screenshots (if available)
```

## Sprint 33 Status

**Execution Status:** NOT EXECUTED

**Reason:** No human tester available for Sprint 33

**Mitigation:**
- Pager disabled by default (`pager_enabled: false` in `src/commands/repl/state.rs`)
- Users can opt-in with `/pager on` command if they want to test
- Automated tests (27 unit tests, 48 interactive tests) verify logic and state management
- Sprint review will NOT claim "pager works" without this manual validation

**Future Action:**
- When human tester becomes available, execute this test case
- If PASS: Update default to `pager_enabled: true` in future sprint
- If FAIL: Debug further, keep disabled by default

## Notes

### Why Manual Validation is Required

Per `docs/testing/approach.md` - Testing Limitations by Feature Type:

**Type 4: Interactive/Alternate Screen (Minimal Automated Coverage)**

Features: Pager, full-screen modes, interactive navigation

**Limitations:**
- Alternate screen buffer invisible to test framework
- PTY timing differs from real terminal
- User interaction sequences not reproducible
- Terminal resize behavior untestable

**Mitigation:** Limited PTY tests for state changes + **MANDATORY manual validation**

### Sprint 29/30 Lesson Learned

Two consecutive sprints achieved 100% test pass rates while delivering completely broken pager functionality:

| Sprint | Tests Passed | Feature Status | User Assessment |
|--------|--------------|----------------|-----------------|
| Sprint 29 | 386/386 (100%) | Broken | "absolutely not working" |
| Sprint 30 | 449/449 (100%) | Still broken | "exact same issue" |

**Root Cause:** Tests validated API contracts and logic, but NOT actual rendered output

**Sprint 31 Philosophy:** "Cannot claim pager works without manual validation"

**Sprint 33 Application:** Ship with pager disabled, document manual test, be honest about validation status

### Testing Philosophy

From `docs/testing/honest-assessment.md`:

> "Success is delivering working features, not passing tests. Remember Sprint 29 and 30. Never repeat that pattern."

This test case ensures that IF we claim "pager works" in a future sprint, we have actual evidence (not just passing automated tests).

## References

- **Issue #14:** https://github.com/remi-td/tq/issues/14
- **Sprint 31 Review:** `docs/sprints/sprint-31-review.md`
- **Testing Approach:** `docs/testing/approach.md`
- **Honest Assessment:** `docs/testing/honest-assessment.md`
- **Pager Implementation:** `src/commands/repl/pager.rs`
