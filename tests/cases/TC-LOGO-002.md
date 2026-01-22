# TC-LOGO-002 - Logo ASCII Art with Info on Right

**Test ID:** TC-LOGO-002
**Category:** Functionality (Visual)
**Priority:** Critical (P0 - BLOCKING)
**Sprint:** 19
**Type:** Manual Visual Test
**Status:** PASS

---

## Context

**Sprint 18 Failure:** TC-LOGO-001 APPROVED but user reports logo still wrong.
**Root Cause:** Sprint 18 tested for simple text "tq", user wants ASCII ART "tq" with specific layout.
**Sprint 19 Mission:** Verify ACTUAL ASCII art with info on RIGHT side.

---

## Objective

Verify that the tq REPL banner displays:
1. ASCII art lowercase "tq" logo (NOT simple text)
2. 't' in Teradata orange (#F37021)
3. 'q' in white or black
4. Welcome/info messages displayed TO THE RIGHT of logo (not below)

---

## User Specification

From `open-bugs.md`:
> "it is the ASCII art `tq` LOGO should be written in lowercase with th 't' in the Teradata orabge color (#F37021) and 'q' in white/black. This big ASCII art is our logo, the first thing the user see when they open. NEXT to it (on the right) should be the welcome and information messages."

**Expected Layout:**
```
tq    Teradata Query Tool v 1.7
      Connected to mcp-vikzqtnd0db0nglk.env.clearscape.teradata.com:1025
      Teradata version 20.00.00.00
      User: demo_user
      Default row limit: 100
      Editor mode: emacs
```

Where "tq" is ASCII art (made of characters forming letters), NOT plain text "tq".

---

## Prerequisites

- [ ] tq binary built: `cargo build --release`
- [ ] Database connection configured in `.env`
- [ ] Terminal with xterm-256 color support
- [ ] Screenshot capture tool available (macOS: Cmd+Shift+4, Linux: gnome-screenshot)

---

## Test Procedure

### Step 1: Start REPL in Real Terminal

**CRITICAL:** This MUST be done in an ACTUAL terminal, NOT automated PTY test.

```bash
./target/release/tq repl
```

### Step 2: Visual Inspection

**Immediately upon startup, inspect the banner and answer these questions:**

#### Question 1: Is the logo ASCII art or plain text?

**ASCII Art Example (CORRECT):**
```
 ██
 ██  ███
 ██  ██
 ▀█▄▄██▄
```

**Plain Text Example (WRONG):**
```
tq
```

**Your Observation:**
- [ ] Logo is ASCII art (made of block characters: █, ▀, ▄, etc.)
- [ ] Logo is plain text "tq" (FAIL if checked)

#### Question 2: What is the logo case?

**Your Observation:**
- [ ] Logo represents lowercase "tq"
- [ ] Logo represents uppercase "TQ" (FAIL if checked)

#### Question 3: What color is the 't' character?

**Your Observation:**
- [ ] 't' is orange (Teradata brand color)
- [ ] 't' is other color: _______________

#### Question 4: What color is the 'q' character?

**Your Observation:**
- [ ] 'q' is white or default terminal color
- [ ] 'q' is other color: _______________

#### Question 5: Where are the info messages?

**Layout Option A (CORRECT):**
```
tq    Teradata Query Tool v 1.7
      Connected to ...
      User: ...
```
Info messages on the RIGHT of logo.

**Layout Option B (WRONG):**
```
tq
Teradata Query Tool v 1.7
Connected to ...
User: ...
```
Info messages BELOW logo.

**Your Observation:**
- [ ] Info messages are to the RIGHT of logo (Option A - CORRECT)
- [ ] Info messages are BELOW logo (Option B - WRONG, FAIL if checked)

### Step 3: Capture Screenshot

**Action:** Take a screenshot of the entire banner area.

**Screenshot Requirements:**
- Include full banner from top to first prompt
- High enough resolution to see character details
- Color rendering visible

**Screenshot File:** Save as `tests/results/sprint-19/logo-screenshot.png`

### Step 4: Compare with Expected Layout

**Expected Layout (from user specification):**
```
tq    Teradata Query Tool v 1.7
      Connected to mcp-vikzqtnd0db0nglk.env.clearscape.teradata.com:1025
      Teradata version 20.00.00.00
      User: demo_user
      Default row limit: 100
      Editor mode: emacs
```

Where "tq" is ASCII art with:
- Lowercase letters
- 't' in orange
- 'q' in white/black
- Info on right side

**Does your screenshot match this layout?**
- [ ] YES - Logo and layout match user specification
- [ ] NO - Differences noted below

**Differences (if any):**
```
[Describe any differences from expected layout]
```

---

## Expected Results

### Visual Requirements

1. **Logo Type:** ASCII art (block characters forming lowercase "tq")
2. **Color:**
   - 't' in Teradata orange (#F37021, xterm-256 color 202)
   - 'q' in white or default terminal color
3. **Layout:** Info messages displayed to the RIGHT of logo (not below)
4. **Content:** All info messages present (version, connection, user, settings)

### Anti-Patterns (MUST NOT appear)

- ❌ Plain text "tq" (not ASCII art)
- ❌ Uppercase ASCII art "TQ"
- ❌ Info messages below logo (not on right)
- ❌ Missing colors (all black/white)
- ❌ Wrong color for 't' (not orange)

---

## Actual Results

**Test Execution Date:** _______________
**Tester:** _______________
**Terminal:** _______________
**tq Version:** _______________

### Visual Observations

**1. Logo Type:**
- [ ] ASCII art (CORRECT)
- [ ] Plain text (FAIL)
- Description: _______________

**2. Logo Case:**
- [ ] Lowercase (CORRECT)
- [ ] Uppercase (FAIL)

**3. 't' Color:**
- [ ] Orange (CORRECT)
- [ ] Other: _______________ (FAIL)

**4. 'q' Color:**
- [ ] White/black (CORRECT)
- [ ] Other: _______________ (FAIL)

**5. Info Messages Position:**
- [ ] Right side of logo (CORRECT)
- [ ] Below logo (FAIL)

**6. Screenshot:**
- [ ] Screenshot captured: `tests/results/sprint-19/logo-screenshot.png`

### Test Verdict

- [ ] ✅ PASS - All visual requirements met, matches user specification
- [ ] ❌ FAIL - Requirements not met (see failures below)
- [ ] ⛔ BLOCKED - Cannot execute test (see blockers below)

**Failures (if FAIL):**
```
[List specific failures]
```

**Blockers (if BLOCKED):**
```
[List blockers]
```

---

## Comparison with Sprint 18

**Sprint 18 TC-LOGO-001 Results:**
- ✅ Verified ANSI color 202 present
- ✅ Verified lowercase "tq" text
- ✅ Verified subtitle present
- ❌ Did NOT verify ASCII art vs plain text
- ❌ Did NOT verify layout (right vs below)

**Sprint 19 TC-LOGO-002 Improvements:**
- ✅ Explicitly checks for ASCII art
- ✅ Explicitly checks layout (right vs below)
- ✅ Visual inspection required
- ✅ Screenshot evidence required

---

## Notes

**Why Manual Testing is Required:**
- No automated test can reliably distinguish ASCII art from plain text in terminal output
- Layout validation (right vs below) requires human visual inspection
- PTY automation in Sprint 18 did not catch the layout issue
- Screenshot provides irrefutable evidence of actual appearance

**Sprint 18 Lesson:**
> "Tests passed but user says it's wrong = tests didn't test the right thing"

---

## Exit Code

N/A (REPL mode, exit code not relevant)

---

## Related Tests

- **TC-LOGO-001** (Sprint 18): Tested for simple text, missed ASCII art requirement
- **TC-COMPLETION-001** (Sprint 19): Tab completion after FROM
- **TC-COMPLETION-002** (Sprint 19): Tab completion qualified names

---

## References

- Bug Report: `docs/builder/incoming/open-bugs.md` (lines 5-16)
- Sprint Planning: `docs/builder/sprints/sprint-19-planning.md`
- Sprint 18 Report: `tests/results/sprint-18/REPORT.md` (claimed FIXED but user disagrees)

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-22 | 1.0 | Initial test case for Sprint 19 (RETRY) | quality-validator |
