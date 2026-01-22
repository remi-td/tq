# Sprint 19 Test Strategy: CRITICAL BUG FIXES (RETRY - Sprint 18 Failed)

**Created:** 2026-01-22
**Author:** quality-validator
**Sprint:** Sprint 19
**Sprint Type:** Maintenance Sprint (CRISIS - RETRY)
**Features:** Logo Fix (P0), Tab Completion Fix (P0)

---

## Critical Context: Why Sprint 19 Exists

**Sprint 18 Status:** APPROVED by quality-validator (2026-01-22 00:06)
- All 286 tests PASSED (100% pass rate)
- Logo bug marked as FIXED
- Tab completion bug marked as FIXED

**User Reality Check (2026-01-22 22:13):**
- Logo STILL showing wrong format
- Tab completion STILL showing "Page 1: records 0 - 0 total: 0 [FULL]" instead of completions

**ROOT CAUSE:** Sprint 18 tests gave FALSE POSITIVES. Tests passed but bugs NOT actually fixed.

**Sprint 19 Mission:** Create tests that ACTUALLY validate what the user sees, not what the code THINKS it does.

---

## Specification Analysis

### Bug 1: Logo Display

**User Report (`open-bugs.md` line 7-16):**
> "it is the ASCII art `tq` LOGO should be written in lowercase with th 't' in the Teradata orabge color (#F37021) and 'q' in white/black. This big ASCII art is our logo, the first thing the user see when they open. NEXT to it (on the right) should be the welcome and information messages."

**Expected Output:**
```
tq    Teradata Query Tool v 1.7
      Connected to mcp-vikzqtnd0db0nglk.env.clearscape.teradata.com:1025
      Teradata version 20.00.00.00
      User: demo_user
      Default row limit: 100
      Editor mode: emacs
```

Where "tq" is ASCII art (lowercase) with 't' in Teradata orange (#F37021).

**Key Requirements:**
1. Logo is ASCII art (not just text "tq")
2. Lowercase "tq" (not uppercase)
3. 't' in Teradata orange (#F37021, xterm-256 color 202)
4. 'q' in white/black
5. Information messages displayed TO THE RIGHT of logo (not below)

**CRITICAL: Sprint 18 tested for simple text "tq", but user wants ASCII ART lowercase "tq"**

### Bug 2: Tab Completion Debug Output

**User Report (`open-bugs.md` line 18-32):**
> "The tab completion is still not working properly. This time it looks like some debug traces are left in.."

**Examples:**
```
tq> ? sel * fr[TAB]
Page 1: records 0 - 0  total: 0  [FULL]
```

```
tq> ? sel * from dbc.t[TAB]
Page 1: records 0 - 0  total: 0
```

**Expected Behavior:**
- Show completion menu with databases/tables
- NO debug output
- NO pager status lines

**Key Requirements:**
1. Tab completion shows completion menu
2. No "Page 1: records..." output
3. No pager status bar
4. Completions are actual databases/tables
5. Text completes at cursor position

**CRITICAL: Pager status bar is being shown during tab completion**

---

## Test Strategy Derivation

### Why Sprint 18 Tests Failed

**Sprint 18 Logo Test:**
- ✅ Verified ANSI color 202 in output
- ✅ Verified lowercase "tq" text
- ✅ Verified subtitle present
- ❌ Did NOT verify ASCII art (vs simple text)
- ❌ Did NOT verify layout (info on right vs below)

**Sprint 18 Tab Completion Tests:**
- ✅ Verified Tab triggers completion
- ✅ Verified databases/tables in suggestions
- ✅ Verified no keywords
- ❌ Did NOT verify no pager output during completion
- ❌ Did NOT verify completion menu actually displays
- ❌ Did NOT test in actual terminal (used PTY automation)

**Lesson:** Automated tests validated CODE behavior, not USER experience.

### Sprint 19 Test Requirements

**ABSOLUTE REQUIREMENTS:**
1. **Test in ACTUAL terminal** - PTY automation doesn't show pager bugs
2. **Visual inspection MANDATORY** - Human must see what user sees
3. **Test exact user scenarios** - Type "sel * fr[TAB]" exactly as user does
4. **Capture screenshots** - Visual proof of behavior
5. **No code review substitutes** - Must EXECUTE in real environment

### Feature 1: Logo Display - REVISED

**Test Type Necessity Matrix:**

| Test Type | Necessary? | Why | Gap if Omitted |
|-----------|------------|-----|----------------|
| Manual visual test in REAL terminal | ✅ REQUIRED | Only way to see ASCII art layout | Cannot verify art vs text, cannot verify layout (right vs below) |
| Screenshot capture | ✅ REQUIRED | Visual proof for user validation | Cannot prove what was actually seen |
| ANSI code validation | ⚠️ INSUFFICIENT | Sprint 18 showed this doesn't catch layout bugs | Validates color but not appearance |
| Unit tests | ❌ INADEQUATE | Sprint 18 unit tests passed, bug still there | Would repeat Sprint 18 mistake |

**Test Implementation:**

**TC-LOGO-001: Logo ASCII Art Layout (MANUAL ONLY)**
- Start tq in ACTUAL terminal (not automated PTY)
- Take screenshot of banner
- Human validator checks:
  1. Is it ASCII art? (Not simple text)
  2. Is it lowercase "tq"?
  3. Is 't' orange and 'q' white/black?
  4. Are info messages on the RIGHT (not below)?
  5. Does it match user's expected layout?
- BLOCKER: No automated test can validate this reliably

### Feature 2: Tab Completion - REVISED

**Test Type Necessity Matrix:**

| Test Type | Necessary? | Why | Gap if Omitted |
|-----------|------------|-----|----------------|
| Manual test in REAL terminal | ✅ REQUIRED | Pager bug only shows in real terminal | PTY automation doesn't trigger pager output |
| Type exact user sequences | ✅ REQUIRED | Must reproduce "sel * fr[TAB]" exactly | Automation may not trigger same code path |
| Visual inspection of output | ✅ REQUIRED | Must see if "Page 1: records..." appears | Code inspection won't find pager call |
| Screenshot capture | ✅ REQUIRED | Proof of what actually happened | Cannot prove pager bug absent |
| Interactive PTY tests | ⚠️ INSUFFICIENT | Sprint 18 showed these give false positives | Miss pager bugs, miss visual issues |

**Test Implementation:**

**TC-TAB-COMPLETION-001: Database/Table Completion After FROM (MANUAL)**
- Start tq in ACTUAL terminal
- Type exactly: `sel * fr` (no enter)
- Press TAB
- Capture screenshot
- Human validator checks:
  1. Does completion menu appear?
  2. Are databases/tables shown?
  3. Is there ANY "Page 1: records..." output?
  4. Does completion work correctly?

**TC-TAB-COMPLETION-002: Qualified Table Completion (MANUAL)**
- Start tq in ACTUAL terminal
- Type exactly: `sel * from dbc.t` (no enter)
- Press TAB
- Capture screenshot
- Human validator checks:
  1. Does completion menu appear?
  2. Are tables in DBC shown?
  3. Is there ANY "Page 1: records..." output?
  4. Does completion work correctly?

---

## Sprint 19 Test Execution Approach

### Phase 1: Pre-Implementation

**DO NOT CREATE TEST CASES YET**

Sprint 19 is testing whether architect can fix bugs that Sprint 18 THOUGHT were fixed.

### Phase 2: Post-Implementation

Once rust-teradata-architect claims fixes are complete:

**Step 1: Manual Visual Testing (MANDATORY)**

```bash
# In ACTUAL terminal (NOT automated):
./target/release/tq repl

# Test 1: Logo
# - Take screenshot of banner
# - Verify ASCII art layout
# - Verify colors
# - Verify info messages on right

# Test 2: Tab completion after FROM
tq> sel * fr[TAB]
# - Take screenshot
# - Verify completion menu (no pager output)

# Test 3: Tab completion qualified
tq> sel * from dbc.t[TAB]
# - Take screenshot
# - Verify completion menu (no pager output)
```

**Step 2: Document Evidence**

Create `tests/results/sprint-19/test-evidence-1.md` with:
- Screenshots of banner
- Screenshots of tab completion
- Detailed descriptions of what was seen
- PASS/FAIL for each test

**Step 3: Verdict**

Create `tests/results/sprint-19/REPORT.md`:

**APPROVED Criteria (ALL must be met):**
- ✅ Logo is ASCII art (lowercase "tq")
- ✅ Logo 't' is orange, 'q' is white/black
- ✅ Info messages appear on RIGHT of logo
- ✅ Tab completion shows completion menu
- ✅ Tab completion NO "Page 1: records..." output
- ✅ Tab completion works for databases/tables
- ✅ Screenshots prove all above

**REJECTED Criteria (ANY fails sprint):**
- ❌ Logo is simple text (not ASCII art)
- ❌ Info messages below logo (not on right)
- ❌ Tab completion shows pager output
- ❌ Tab completion doesn't work
- ❌ Cannot reproduce bug fixes

**BLOCKED Criteria:**
- ⛔ Database not available
- ⛔ Build fails
- ⛔ Terminal environment broken

---

## Key Differences from Sprint 18

**Sprint 18 Approach (FAILED):**
- ✅ Ran automated tests
- ✅ Verified ANSI codes
- ✅ Checked unit tests
- ❌ Did NOT verify visual appearance
- ❌ Did NOT test in real terminal
- ❌ Did NOT catch pager bug

**Sprint 19 Approach (MUST SUCCEED):**
- ✅ Manual testing in REAL terminal
- ✅ Screenshot capture
- ✅ Human visual validation
- ✅ Test exact user scenarios
- ✅ No code review substitutes
- ⚠️ Automated tests are INSUFFICIENT

---

## Test Coverage Requirements

**Logo Fix:**
- [ ] Manual visual test EXECUTED in real terminal
- [ ] Screenshot captured
- [ ] ASCII art layout verified
- [ ] Color verified visually
- [ ] Info messages position verified (RIGHT not BELOW)

**Tab Completion Fix:**
- [ ] Manual test EXECUTED: "sel * fr[TAB]"
- [ ] Manual test EXECUTED: "sel * from dbc.t[TAB]"
- [ ] Screenshots captured for both
- [ ] Pager output ABSENT verified
- [ ] Completion menu PRESENT verified

**Coverage Gap if Manual Tests Omitted:**
- Sprint 18 proved automated tests CANNOT catch these bugs
- NO automated test can validate ASCII art layout
- NO automated test caught pager bug in Sprint 18
- Manual testing is NOT optional, it is REQUIRED

---

## Verdict Criteria

### APPROVED

**Requirements:**
1. ✅ Logo is ASCII art lowercase "tq" with correct colors
2. ✅ Info messages on right side of logo
3. ✅ Tab completion shows completion menu
4. ✅ NO "Page 1: records..." in tab completion
5. ✅ All manual tests executed with screenshots
6. ✅ Human validator confirms bugs fixed

### REJECTED

**Triggers:**
1. ❌ Logo is simple text (not ASCII art)
2. ❌ Info messages below logo (wrong layout)
3. ❌ Pager output appears during tab completion
4. ❌ Tab completion doesn't show menu
5. ❌ Cannot reproduce user's expected behavior

### BLOCKED

**Triggers:**
1. ⛔ Cannot build tq
2. ⛔ No database available for testing
3. ⛔ Terminal environment broken

---

## Lessons Learned from Sprint 18

**What Went Wrong:**
1. Over-reliance on automated tests
2. No visual validation of output
3. PTY automation doesn't catch all bugs
4. ANSI code verification doesn't prove appearance
5. Tests validated CODE, not USER EXPERIENCE

**What Sprint 19 Must Do Better:**
1. MANDATORY manual testing in real terminal
2. MANDATORY screenshot capture
3. MANDATORY human visual validation
4. Test EXACTLY what user reported
5. No code review substitutes for execution

**Critical Insight:**
> "100% automated test pass rate means NOTHING if tests don't validate what user sees."

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-01-22
**Sprint Context:** RETRY Sprint 18 (failed validation)
**Test Approach:** Manual visual testing MANDATORY
**Confidence Level:** MEDIUM (depends on architect understanding the REAL bugs)

**Next Steps:**
1. Wait for rust-teradata-architect to implement fixes
2. Execute manual visual tests in REAL terminal
3. Capture screenshots as evidence
4. Create comprehensive test report
5. REJECT if user's bugs not actually fixed

---

## Strategy Validation Checklist

- ✅ Identified why Sprint 18 tests gave false positives
- ✅ Derived test approach from Sprint 18 failure analysis
- ✅ Manual testing requirements clearly stated
- ✅ Screenshot requirements documented
- ✅ Verdict criteria based on user's actual report
- ✅ No hand-waving about what "should" work
- ✅ Honest assessment of Sprint 18 failure
- ✅ Clear gap analysis (manual vs automated testing)
