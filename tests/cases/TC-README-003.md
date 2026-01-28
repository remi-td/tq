# TC-README-003: README Screenshot Validation

**Test Case ID:** TC-README-003
**Feature:** README Screenshot of tq in Action (#9)
**Test Type:** Integration (File + Content Validation)
**Priority:** P1
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Verify that README.md includes a screenshot of tq in action, the screenshot file exists, is a valid image, and is referenced correctly in the README.

---

## Prerequisites

- [ ] tq project repository checked out
- [ ] README.md file exists
- [ ] Screenshot file provided by user (issue #9) or generated

---

## Test Steps

### Step 1: Search for Screenshot Reference in README
**Action:** Look for image markdown syntax in README
```bash
grep '!\[.*\](.*\.png\|.*\.jpg\|.*\.jpeg\|.*\.gif)' README.md
```

**Expected Result:**
- Image markdown syntax found
- Pattern: `![alt text](path/to/image.png)`
- Image is referenced in README

### Step 2: Extract Screenshot File Path
**Action:** Extract image path from markdown
```bash
grep -o '!\[.*\]([^)]*\.\(png\|jpg\|jpeg\))' README.md | grep -o '([^)]*)' | tr -d '()'
```

**Expected Result:**
- Image path extracted (e.g., docs/images/screenshot.png)
- Path is relative to project root or absolute URL

### Step 3: Verify Screenshot File Exists
**Action:** Check that screenshot file exists at specified path
```bash
# If path is relative (common for local files):
ls -la docs/images/*.png 2>/dev/null || ls -la *.png 2>/dev/null || echo "Check extracted path"
```

**Expected Result:**
- Screenshot file exists at specified path
- File is not missing or broken link

### Step 4: Verify Screenshot is Valid Image
**Action:** Check file type and basic validity
```bash
file docs/images/screenshot.png  # Or whatever path extracted
```

**Expected Result:**
- File type is image (PNG, JPEG, GIF)
- Not a corrupted or text file
- Output: "PNG image data" or "JPEG image data"

### Step 5: Verify Screenshot File Size is Reasonable
**Action:** Check screenshot file size
```bash
ls -lh docs/images/screenshot.png  # Or extracted path
```

**Expected Result:**
- File size is reasonable for screenshot (typically 50 KB - 2 MB)
- Not too small (< 10 KB suggests broken/placeholder)
- Not too large (> 5 MB suggests non-optimized)

### Step 6: Verify Screenshot Placement in README
**Action:** Check where screenshot appears in README
```bash
grep -n '!\[.*\](.*\.png)' README.md
```

**Expected Result:**
- Screenshot appears in first ~100 lines of README
- Placement: After "What is tq?" section, before detailed usage
- Part of TLDR visual element

### Step 7: Verify Screenshot Alt Text
**Action:** Check that screenshot has descriptive alt text
```bash
grep -o '!\[[^\]]*\]' README.md | grep screenshot
```

**Expected Result:**
- Alt text is descriptive (not empty, not just "image")
- Examples: "tq screenshot", "tq in action", "tq REPL showing table output"
- Accessible for screen readers

---

## Expected Results

### Success Criteria
- [x] README references a screenshot image
- [x] Screenshot file exists at specified path
- [x] Screenshot is valid image (PNG, JPEG, or GIF)
- [x] Screenshot file size is reasonable (50 KB - 2 MB)
- [x] Screenshot appears early in README (< 100 lines)
- [x] Screenshot has descriptive alt text

### Screenshot Reference Example
```markdown
![tq in action showing REPL with table output](docs/images/tq-screenshot.png)
```

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** quality-validator
**Build Version:** [Commit hash]

**Screenshot Reference in README:**
```
$ grep '!\[.*\](.*\.png)' README.md
[Full markdown image syntax]
```

**Screenshot File Path:**
```
Extracted path: [path/to/screenshot.png]
```

**Screenshot File Exists:**
```
$ ls -la [extracted path]
[File details]
```

**Screenshot File Type:**
```
$ file [extracted path]
[File type - should be image]
```

**Screenshot File Size:**
```
$ ls -lh [extracted path]
[File size - should be 50KB-2MB range]
```

**Screenshot Placement:**
```
$ grep -n '!\[.*\](.*\.png)' README.md
[Line number - should be < 100]
```

**Screenshot Alt Text:**
```
$ grep -o '!\[[^\]]*\]' README.md
[Alt text content]
```

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Pass Condition:**
- PASS: Screenshot exists, valid image, referenced in README, appropriate placement
- FAIL: Screenshot missing, invalid, or not referenced
- BLOCKED: Cannot verify (README missing)

**Defects Found:**
- [If FAIL: Specify what is missing or invalid]

---

## Notes

- AC-README-003: "Screenshot of tq in action included" (sprint-27-planning.md:102)
- User provided screenshot in issue #9 (already attached to issue)
- Screenshot should show tq's key feature: REPL with table output
- Visual element is important for giving users quick understanding
- Screenshot should be optimized (not 10 MB raw PNG)
- Alt text is important for accessibility

---

## Related Requirements

- AC-README-003: "Screenshot of tq in action included" (sprint-27-planning.md:102)
- AC-README-001: "TLDR introduction section (What/Visual/Quick Start)" - Visual component
- GitHub Issue #9: README should include screenshot (user provided one)
- Dependencies: Screenshot file provided by user in issue #9
