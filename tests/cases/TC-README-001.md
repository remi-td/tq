# TC-README-001: README Structure and TLDR Section

**Test Case ID:** TC-README-001
**Feature:** README User-Focused Structure (#9)
**Test Type:** Integration (Content Validation)
**Priority:** P1
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Verify that README.md follows user-focused TLDR structure with What/Visual/Quick Start sections at the beginning, not developer configuration.

---

## Prerequisites

- [ ] tq project repository checked out
- [ ] README.md file exists at project root

---

## Test Steps

### Step 1: Verify README.md Exists
**Action:** Check that README.md file exists
```bash
ls -la README.md
```

**Expected Result:**
- README.md exists at project root
- File is readable

### Step 2: Verify README Starts with User-Facing Content
**Action:** Read first 50 lines of README
```bash
head -50 README.md
```

**Expected Result:**
- README does NOT start with "GitHub Configuration" (developer-focused)
- README DOES start with user-facing content (What/Visual/Quick Start)
- First section is about tq project, not development setup

### Step 3: Verify TLDR or Introduction Section
**Action:** Check for TLDR-style introduction
```bash
grep -E '^## What|^## tq|^# tq' README.md | head -5
```

**Expected Result:**
- README has clear introduction section
- Section explains what tq is (TLDR format)
- Common patterns: "## What is tq?", "## tq", "# tq - Teradata Query"

### Step 4: Verify Visual Element (Screenshot)
**Action:** Check for screenshot or visual element in early sections
```bash
grep -i '!\[.*\](.*.png\|.*.jpg)' README.md | head -5
```

**Expected Result:**
- README includes screenshot or visual
- Screenshot is referenced in first ~100 lines
- Image shows tq in action

### Step 5: Verify Quick Start Section
**Action:** Look for Quick Start or Getting Started section
```bash
grep -i '^## Quick Start\|^## Getting Started\|^## Installation' README.md
```

**Expected Result:**
- README has Quick Start or Installation section
- Section appears early in README (user onboarding)
- Provides clear steps to get started

### Step 6: Verify GitHub Configuration is NOT at Top
**Action:** Check that GitHub Configuration moved to appropriate location
```bash
grep -n -i 'GitHub Configuration' README.md
```

**Expected Result:**
- "GitHub Configuration" section exists (still needed for contributors)
- Section is NOT in first 100 lines
- Section is moved to contributing/developer section

### Step 7: Verify Section Order is User-Focused
**Action:** List all major section headers in order
```bash
grep '^## ' README.md
```

**Expected Result:**
- Sections ordered for user journey:
  1. What (introduction)
  2. Visual (screenshot)
  3. Quick Start / Installation
  4. Usage / Features
  5. Documentation links
  6. Contributing / Development (GitHub Configuration here)
  7. License
- Developer content (GitHub Configuration) is near end, not beginning

---

## Expected Results

### Success Criteria
- [x] README starts with user-facing content (not GitHub Configuration)
- [x] TLDR introduction section present (What is tq?)
- [x] Visual element (screenshot) included
- [x] Quick Start / Installation section present
- [x] GitHub Configuration moved to appropriate location (not at top)
- [x] Section order is user-focused

### Section Order Examples (Valid Patterns)
**Pattern 1: TLDR Format**
```
# tq - Teradata Query
## What is tq?
## Screenshot
## Quick Start
## Features
## Documentation
## Contributing
## License
```

**Pattern 2: Traditional Format**
```
# tq
## Introduction
## Installation
## Usage
## Features
## Development (GitHub Configuration)
## License
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** quality-validator
**Build Version:** [Commit hash]

**First 50 Lines Content:**
```
$ head -50 README.md
[Paste first 50 lines - should show user-facing content]
```

**Section Headers in Order:**
```
$ grep '^## ' README.md
[List all section headers]
```

**GitHub Configuration Location:**
```
$ grep -n 'GitHub Configuration' README.md
[Line number - should be > 100]
```

**Screenshot Reference:**
```
$ grep -i '!\[.*\](.*.png)' README.md
[Screenshot markdown syntax]
```

**Quick Start Section:**
```
$ grep -n -i '^## Quick Start\|^## Installation' README.md
[Line number - should be early, < 100]
```

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Pass Condition:**
- PASS: README starts with user content, has TLDR structure, GitHub Configuration moved
- FAIL: README still starts with GitHub Configuration, missing key sections
- BLOCKED: README.md does not exist

**Defects Found:**
- [If FAIL: List what is missing or misplaced]

---

## Notes

- AC-README-001: "TLDR introduction section (What/Visual/Quick Start)" (sprint-27-planning.md:100)
- AC-README-007: "GitHub Configuration section moved to appropriate location" (sprint-27-planning.md:106)
- User complaint: README currently starts with "GitHub Configuration" (developer-focused)
- Goal: Professional first impression for new users
- GitHub Configuration is still needed, just moved to developer section

---

## Related Requirements

- AC-README-001: "TLDR introduction section (What/Visual/Quick Start)" (sprint-27-planning.md:100)
- AC-README-003: "Screenshot of tq in action included" (sprint-27-planning.md:102)
- AC-README-007: "GitHub Configuration section moved" (sprint-27-planning.md:106)
- GitHub Issue #9: README - User-focused documentation
- User Request: README should give good first impression, not start with developer config
