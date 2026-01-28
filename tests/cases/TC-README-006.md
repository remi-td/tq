# TC-README-006: README GitHub Configuration Section Moved

**Test Case ID:** TC-README-006
**Feature:** README GitHub Configuration Section Relocation (#9)
**Test Type:** Integration (Content Validation)
**Priority:** P1
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Verify that the "GitHub Configuration" section is moved from the beginning of README.md to an appropriate location (contributing/developer section or separate CONTRIBUTING.md file), and is not the first thing users see.

---

## Prerequisites

- [ ] tq project repository checked out
- [ ] README.md file exists
- [ ] Understanding of original README structure (started with GitHub Configuration)

---

## Test Steps

### Step 1: Verify GitHub Configuration is NOT at Top
**Action:** Check first 50 lines of README
```bash
head -50 README.md | grep -i 'GitHub Configuration'
```

**Expected Result:**
- "GitHub Configuration" NOT found in first 50 lines
- README starts with user-facing content instead
- Exit code 1 (no match)

### Step 2: Verify GitHub Configuration Still Exists
**Action:** Search entire README for GitHub Configuration
```bash
grep -i 'GitHub Configuration' README.md
```

**Expected Result:**
- "GitHub Configuration" section still exists (not deleted)
- Section is preserved but relocated
- Found somewhere in README

### Step 3: Verify GitHub Configuration Placement
**Action:** Get line number of GitHub Configuration section
```bash
grep -n -i 'GitHub Configuration' README.md
```

**Expected Result:**
- Line number > 100 (well into README)
- Typical placement: After user content, in developer/contributing section
- Not in first 20% of README

### Step 4: Check if Moved to CONTRIBUTING.md
**Action:** Check if CONTRIBUTING.md exists with GitHub Configuration
```bash
if [ -f CONTRIBUTING.md ]; then
    grep -i 'GitHub Configuration' CONTRIBUTING.md
else
    echo "CONTRIBUTING.md does not exist"
fi
```

**Expected Result:**
- Either:
  - GitHub Configuration in README (developer section), OR
  - GitHub Configuration moved to CONTRIBUTING.md
- Content exists somewhere appropriate

### Step 5: Verify Context of GitHub Configuration
**Action:** Read section around GitHub Configuration
```bash
grep -B 5 -A 10 -i 'GitHub Configuration' README.md
```

**Expected Result:**
- Section is in developer/contributing context
- Surrounded by other developer content (not user content)
- May be under "## Contributing" or "## Development" section

### Step 6: Verify User-Facing Content at Top
**Action:** Check what README starts with now
```bash
head -20 README.md
```

**Expected Result:**
- README starts with user content:
  - What is tq?
  - Screenshot
  - Quick start
- NOT developer configuration

---

## Expected Results

### Success Criteria
- [x] GitHub Configuration is NOT in first 50 lines of README
- [x] GitHub Configuration section still exists (not deleted)
- [x] GitHub Configuration is in developer/contributing section (line > 100)
- [x] README starts with user-facing content
- [x] Developer content appropriately grouped

### README Structure Before Sprint 27 (WRONG)
```
# tq
## GitHub Configuration  ← WRONG: Developer content first
...
## What is tq?  ← User content buried
```

### README Structure After Sprint 27 (CORRECT)
```
# tq
## What is tq?  ← Correct: User content first
## Screenshot
## Quick Start
...
## Contributing  ← Developer section
### GitHub Configuration  ← Correct: Developer content in appropriate section
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** quality-validator
**Build Version:** [Commit hash]

**First 50 Lines Check:**
```
$ head -50 README.md | grep -i 'GitHub Configuration'
[Output should be empty - no match]
```

**GitHub Configuration Still Exists:**
```
$ grep -i 'GitHub Configuration' README.md
[Output should show the section header]
```

**GitHub Configuration Line Number:**
```
$ grep -n -i 'GitHub Configuration' README.md
[Line number - should be > 100]

Total README lines: [X]
GitHub Config at line: [Y]
Percentage: [Y/X * 100]% through README (should be > 60%)
```

**CONTRIBUTING.md Check:**
```
$ [ -f CONTRIBUTING.md ] && grep -i 'GitHub Configuration' CONTRIBUTING.md || echo "In README only"
[Output - shows where content is located]
```

**Context Around GitHub Configuration:**
```
$ grep -B 5 -A 2 -i 'GitHub Configuration' README.md
[Section context - should be in developer/contributing area]
```

**README Start:**
```
$ head -20 README.md
[First 20 lines - should show user content, not GitHub Configuration]
```

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Pass Condition:**
- PASS: GitHub Configuration moved from top, now in developer section (line > 100)
- FAIL: GitHub Configuration still at top (first 50 lines)
- BLOCKED: README.md does not exist

**Placement Evaluation:**
- [Appropriate: YES/NO]
- [In developer/contributing context: YES/NO]

---

## Notes

- AC-README-007: "GitHub Configuration section moved to appropriate location" (sprint-27-planning.md:106)
- Original user complaint: README starts with "GitHub Configuration" (developer-focused)
- GitHub Configuration is still needed for contributors
- Solution: Move to developer/contributing section, not delete
- README should give good first impression to users, not developers
- Developer content belongs at end or in CONTRIBUTING.md

---

## Related Requirements

- AC-README-007: "GitHub Configuration section moved to appropriate location (CONTRIBUTING.md or developer docs)" (sprint-27-planning.md:106)
- GitHub Issue #9: README currently starts with developer config (wrong first impression)
- User Request: README should be user-focused
- Best Practice: Developer content near end of README or in CONTRIBUTING.md
