# TC-LOGO-003 - Logo Display Verification (Sprint 20 - 9-Line ASCII Art)

**Test ID:** TC-LOGO-003
**Category:** Functionality (Visual)
**Priority:** Critical (P0 - BLOCKING)
**Sprint:** 20
**Type:** Hybrid (Interactive Automated + Manual Visual)
**Status:** PENDING

---

## Context

**Sprint 18 Failure:** TC-LOGO-001 APPROVED but user reports logo still wrong.
**Sprint 19 Failure:** TC-LOGO-002 attempted fix but user reports logo still not matching specification.
**Root Cause:** Logo doesn't match user's exact 9-line ASCII art specification with lowercase block characters.
**Sprint 20 Mission:** Verify EXACT 9-line ASCII art logo with 't' in orange, 'q' in default color.

---

## Objective

Verify that the tq REPL banner displays the user's exact 9-line lowercase ASCII art logo:
1. ASCII art matches user's specification line-by-line (9 lines total)
2. 't' portion (left) is colored in Teradata orange (xterm-256 color 202)
3. 'q' portion (right) is in default terminal color (no color override)
4. Welcome/info messages displayed TO THE RIGHT of logo (not below)
5. Logo uses LOWERCASE block characters (not uppercase)

---

## User Specification

From `incoming/open-bugs.md`:
> "This is a lowercase 't' (left) in Teradata orange and lowercase 'q' (right) in default color, using block characters for clarity."

**User's Exact 9-Line ASCII Art:**
```
 __
/\ \__
\ \ ,_\    __
 \ \ \/  /'__`\
  \ \ \_/\ \L\ \
   \ \__\ \___, \
    \/__/\/___/\ \
              \ \_\
               \/_/
```

**Expected Layout:**
```
 __
/\ \__                    Teradata Query Tool v 1.7.1
\ \ ,_\    __             Connected to: hostname:1025
 \ \ \/  /'__`\           Teradata version: 20.00.00.00
  \ \ \_/\ \L\ \          User: demo_user
   \ \__\ \___, \         Default row limit: 100
    \/__/\/___/\ \        Editor mode: emacs
              \ \_\
               \/_/

tq>
```

Where the 't' portion (left side) is orange and 'q' portion (right side) is default color.

---

## Prerequisites

- [ ] tq binary built: `cargo build --release`
- [ ] Database connection configured in `.env`
- [ ] Terminal with xterm-256 color support
- [ ] Screenshot capture tool available (macOS: Cmd+Shift+4, Linux: gnome-screenshot)

---

## Test Procedure

### Part 1: Automated Component (expectrl)

**Purpose:** Provide automated regression detection and safety net.

**Execution:** Run via `cargo test --test interactive_tests test_logo_display -- --ignored`

**What Automated Test Validates:**
- [ ] REPL starts successfully
- [ ] Startup output contains ASCII art characters (underscores, slashes, backslashes, parentheses)
- [ ] ANSI escape sequence for xterm-256 color 202 (orange) is present
- [ ] Info lines appear in output (version, connection, user)
- [ ] Logo has approximately 9 lines of ASCII art
- [ ] No crashes or errors during startup

**Automated Test Pass Criteria:**
- [ ] Test executes without errors
- [ ] All assertions pass (color codes, ASCII characters, info lines present)
- [ ] REPL reaches prompt successfully

**Note:** Automated test cannot verify visual appearance, exact layout, or subjective correctness. Manual validation is REQUIRED.

---

### Part 2: Manual Component (User Validation)

**CRITICAL:** This MUST be done in an ACTUAL terminal by a human, NOT automated PTY test.

#### Step 1: Start REPL in Real Terminal

```bash
./target/release/tq repl
```

Wait for banner to display and prompt to appear.

#### Step 2: Visual Inspection Checklist

**Question 1: How many lines is the ASCII art logo?**

**Your Observation:**
- [ ] Logo is 9 lines (CORRECT)
- [ ] Logo is different number of lines: _____ (FAIL)

**Question 2: Does the logo match the user's exact ASCII art?**

**Instructions:** Compare the displayed logo character-by-character with the user's specification:
```
 __
/\ \__
\ \ ,_\    __
 \ \ \/  /'__`\
  \ \ \_/\ \L\ \
   \ \__\ \___, \
    \/__/\/___/\ \
              \ \_\
               \/_/
```

**Your Observation:**
- [ ] Logo matches user's specification EXACTLY (CORRECT)
- [ ] Logo has differences (FAIL - describe below)

**Differences (if any):**
```
[Describe character-level differences]
```

**Question 3: Does the logo represent lowercase or uppercase letters?**

**Visual Guide:**
- **Lowercase:** Characters suggest lowercase 't' and 'q' shapes
- **Uppercase:** Characters suggest uppercase 'T' and 'Q' shapes

**Your Observation:**
- [ ] Logo clearly represents LOWERCASE "tq" (CORRECT)
- [ ] Logo represents UPPERCASE "TQ" (FAIL)
- [ ] Ambiguous/unclear: _______________

**Question 4: What color is the 't' portion (left side of logo)?**

**Your Observation:**
- [ ] 't' is orange (Teradata brand color) (CORRECT)
- [ ] 't' is other color: _______________ (FAIL)
- [ ] 't' is not colored (all default/white/black) (FAIL)

**Question 5: What color is the 'q' portion (right side of logo)?**

**Your Observation:**
- [ ] 'q' is in default terminal color (white/black, not colored) (CORRECT)
- [ ] 'q' is colored: _______________ (FAIL)

**Question 6: Where are the info messages located?**

**Layout Option A (CORRECT):**
```
tq-logo-art    Teradata Query Tool v X.X
(left side)    Connected to ...
               User: ...
```
Info messages on the RIGHT of logo.

**Layout Option B (WRONG):**
```
tq-logo-art
(centered or left)

Teradata Query Tool v X.X
Connected to ...
User: ...
```
Info messages BELOW logo.

**Your Observation:**
- [ ] Info messages are to the RIGHT of logo (Option A - CORRECT)
- [ ] Info messages are BELOW logo (Option B - FAIL)

**Question 7: Do all the info lines appear?**

**Expected Info Lines:**
- [ ] "Teradata Query Tool v X.X.X"
- [ ] "Connected to: [hostname:port]"
- [ ] "Teradata version: X.X.X.X"
- [ ] "User: [username]"
- [ ] "Default row limit: [number]"
- [ ] "Editor mode: [emacs/vi]"

**Your Observation:**
- [ ] All info lines present (CORRECT)
- [ ] Missing info lines (FAIL - list missing ones): _______________

#### Step 3: Capture Screenshot

**Action:** Take a screenshot of the entire banner area.

**Screenshot Requirements:**
- Include full banner from top to first prompt (`tq>`)
- High enough resolution to see character details
- Color rendering visible (orange 't' should be clearly orange)
- Entire 9-line logo visible

**Screenshot File Path:** `tests/results/sprint-20/screenshots/logo-display.png`

**Capture Method:**
- **macOS:** Cmd+Shift+4, select banner area, save to above path
- **Linux:** `gnome-screenshot -a` or `scrot -s`, save to above path
- **Windows:** Snipping Tool, save to above path

**Screenshot Captured:**
- [ ] Screenshot saved to `tests/results/sprint-20/screenshots/logo-display.png`

---

## Expected Results

### Visual Requirements (MUST ALL BE TRUE)

1. **Logo Structure:**
   - Exactly 9 lines of ASCII art
   - Matches user's specification character-by-character
   - Uses block characters (slashes, backslashes, underscores, parentheses)

2. **Logo Case:**
   - Clearly represents LOWERCASE "tq" (not uppercase "TQ")
   - Visual shape suggests lowercase letter forms

3. **Colors:**
   - 't' portion (left) in Teradata orange (RGB ≈ 255,95,0, xterm-256 color 202)
   - 'q' portion (right) in default terminal color (no color override)

4. **Layout:**
   - Info messages displayed TO THE RIGHT of logo
   - NOT below logo
   - All 6 info lines present (version, connection, Teradata version, user, row limit, editor mode)

5. **Rendering:**
   - No crashes or errors
   - Logo displays cleanly without corruption
   - Colors render correctly in terminal

### Anti-Patterns (MUST NOT APPEAR)

- ❌ Logo is not 9 lines (too few, too many, or plain text "tq")
- ❌ Logo doesn't match user's specification (wrong characters, wrong layout)
- ❌ Logo represents UPPERCASE "TQ" instead of lowercase "tq"
- ❌ 't' is not orange (wrong color or no color)
- ❌ 'q' is colored (should be default color)
- ❌ Info messages below logo (not on right)
- ❌ Missing info lines

---

## Actual Results

**Test Execution Date:** _______________
**Tester:** _______________
**Terminal:** _______________ (e.g., iTerm2, Terminal.app, gnome-terminal)
**OS:** _______________ (e.g., macOS 13.5, Ubuntu 22.04)
**tq Version:** _______________

### Automated Test Results

**Execution Command:**
```bash
cargo test --test interactive_tests test_logo_display -- --ignored
```

**Test Output:**
```
[Paste test execution output here]
```

**Automated Test Verdict:**
- [ ] ✅ PASS - All automated assertions passed
- [ ] ❌ FAIL - Automated test failed (see output above)
- [ ] ⛔ BLOCKED - Cannot execute automated test

### Manual Test Results

**1. Logo Line Count:**
- [ ] 9 lines (CORRECT)
- [ ] Other: _____ lines (FAIL)

**2. Logo Matches Specification:**
- [ ] EXACT match (CORRECT)
- [ ] Differences (FAIL): _______________

**3. Logo Case:**
- [ ] Lowercase (CORRECT)
- [ ] Uppercase (FAIL)

**4. 't' Color:**
- [ ] Orange (CORRECT)
- [ ] Other: _______________ (FAIL)

**5. 'q' Color:**
- [ ] Default color (CORRECT)
- [ ] Colored: _______________ (FAIL)

**6. Info Messages Position:**
- [ ] Right side of logo (CORRECT)
- [ ] Below logo (FAIL)

**7. All Info Lines Present:**
- [ ] All 6 lines present (CORRECT)
- [ ] Missing lines (FAIL): _______________

**8. Screenshot:**
- [ ] Screenshot captured: `tests/results/sprint-20/screenshots/logo-display.png`

### Final Verdict

**Hybrid Test Verdict:**
- [ ] ✅ PASS - Both automated AND manual tests passed
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

## Comparison with Previous Sprints

**Sprint 18 TC-LOGO-001:**
- ✅ Verified ANSI color 202 present
- ✅ Verified lowercase "tq" text
- ✅ Verified subtitle present
- ❌ Did NOT verify ASCII art structure
- ❌ Did NOT verify exact line count or character-level match

**Sprint 19 TC-LOGO-002:**
- ✅ Verified ASCII art vs plain text
- ✅ Verified layout (right vs below)
- ✅ Required visual inspection
- ❌ Did NOT verify exact 9-line specification match
- ❌ Did NOT verify character-level correctness

**Sprint 20 TC-LOGO-003 Improvements:**
- ✅ Verifies EXACT 9-line specification
- ✅ Character-by-character comparison required
- ✅ Hybrid testing (automated + manual)
- ✅ Screenshot evidence mandatory
- ✅ Explicit line count verification
- ✅ All info lines checked

---

## Unit Test Coverage

**Automated Unit Tests (Recommended):**

While full visual validation requires manual testing, unit tests can validate logo data structures:

**Test Location:** `src/commands/repl/mod.rs` test module

**Test Cases:**
1. **Verify logo_t array structure:**
   - Contains 9 strings (one per line)
   - Contains expected ASCII characters (/, \, _, etc.)
   - No empty lines or malformed data

2. **Verify logo_q array structure:**
   - Contains 9 strings (one per line)
   - Contains expected ASCII characters ((, ), ', \, etc.)
   - Aligns with logo_t line count

3. **Verify color application:**
   - Orange color (202) applied to logo_t
   - No color applied to logo_q (default color used)

**Unit Test Commands:**
```bash
# Run logo unit tests
cargo test --lib logo

# Or run all REPL unit tests
cargo test --lib repl
```

**Note:** Unit tests validate data structures, NOT visual appearance. Manual validation is still REQUIRED.

---

## Notes

**Why Hybrid Testing is Required:**

**Automated Component:**
- Provides regression detection (catches if logo breaks in future)
- Validates technical requirements (color codes, ASCII characters present)
- Can run in CI/CD pipelines
- Fast feedback loop

**Manual Component:**
- Only humans can judge subjective visual correctness
- Validates exact character-by-character match to specification
- Confirms colors render correctly in actual terminal
- Verifies layout and visual appearance
- Screenshot provides irrefutable evidence

**Together:** Automated tests prevent regressions, manual tests confirm user requirements are met.

**Critical Success Factor:**
This test is NOT complete until BOTH automated and manual components PASS. Automated-only pass is NOT sufficient (learned from Sprint 18).

---

## Exit Code

N/A (REPL mode, exit code not relevant for logo display)

---

## Related Tests

- **TC-LOGO-001** (Sprint 18): Tested for simple text, missed ASCII art requirement
- **TC-LOGO-002** (Sprint 19): Tested for ASCII art, missed exact 9-line specification
- **TC-TAB-COMPLETION-003** (Sprint 20): Tab completion without pager output

---

## References

- Bug Report: `incoming/open-bugs.md` (lines 6-22)
- Sprint Planning: `docs/sprints/sprint-20-planning.md` (lines 41-70)
- Test Strategy: `tests/strategy/sprint-20-test-strategy.md` (lines 30-290)
- Branding Guidelines: `docs/specifications/branding-guidelines.md`

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-23 | 1.0 | Initial test case for Sprint 20 - 9-line ASCII art verification | quality-validator |
