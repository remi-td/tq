# TC-032-README: Feature #12 - Fix GitHub README Display

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-032-README |
| **Title** | Manual Verification - GitHub README Display Fix |
| **Category** | Manual Verification |
| **Priority** | Low |
| **Feature** | Sprint 32 - Fix GitHub README Display (Feature #12) |
| **Test Type** | Manual Verification |
| **Created** | 2026-02-03 |

## Purpose

Verify that the root `README.md` displays correctly on GitHub repository landing page after renaming `.github/README.md`.

## Acceptance Criteria Coverage

- **AC-1**: Root `README.md` displays on GitHub repository landing page
- **AC-2**: `.github/` directory content remains accessible for GitHub configuration
- **AC-3**: Solution is GitHub convention-compliant
- **AC-4**: No broken links or references

## Nature of Feature

This is a **documentation fix** (file rename/move operation). No code changes, no runtime behavior, no logic to test.

**Test Approach:** Simple post-push verification on GitHub.

## Prerequisites

- [ ] `.github/README.md` renamed to `.github/GITHUB_CONFIG.md` (or similar)
- [ ] Changes committed and pushed to GitHub
- [ ] Access to GitHub repository web interface

## Test Procedure

### Step 1: Verify Root README Displays on Landing Page

**Action:** Navigate to GitHub repository landing page

```
URL: https://github.com/[user]/tq
```

**Expected Result:**
- Root `README.md` content displays on repository landing page
- Content shows project introduction (not "GitHub Configuration")
- Teradata logo visible (if included in root README)
- All sections render correctly

**Actual Result:**
- README displayed: [YES/NO]
- Content correct: [YES/NO]
- Visual rendering: [GOOD/ISSUES]

**Pass/Fail:**
- **PASS**: Root README displays with project introduction content
- **FAIL**: `.github/` README displays instead, or no README shown

---

### Step 2: Verify .github/ Directory Still Accessible

**Action:** Navigate to `.github/` directory on GitHub

```
URL: https://github.com/[user]/tq/tree/master/.github
```

**Expected Result:**
- `.github/` directory accessible
- Renamed file visible (e.g., `GITHUB_CONFIG.md`)
- Directory contents intact (issue templates, workflows, etc.)

**Actual Result:**
- Directory accessible: [YES/NO]
- Renamed file present: [YES/NO]
- Contents intact: [YES/NO]

**Pass/Fail:**
- **PASS**: Directory accessible, renamed file present
- **FAIL**: Directory missing or renamed file not found

---

### Step 3: Verify No Broken Links in Root README

**Action:** Click all links in root README

**Expected Result:**
- All links work (no 404 errors)
- Internal links (to docs/, LICENSE, etc.) resolve correctly
- External links (if any) resolve correctly

**Actual Result:**
- Broken links found: [NONE / LIST]

**Pass/Fail:**
- **PASS**: No broken links
- **FAIL**: One or more links return 404 or error

---

### Step 4: Verify No Broken Links in .github/ Content

**Action:** Open renamed file (e.g., `GITHUB_CONFIG.md`) and check links

**Expected Result:**
- All links in renamed file work
- References to project files still resolve

**Actual Result:**
- Broken links found: [NONE / LIST]

**Pass/Fail:**
- **PASS**: No broken links
- **FAIL**: Links broken after rename

---

### Step 5: Verify GitHub Convention Compliance

**Action:** Verify solution follows GitHub conventions

**GitHub Convention:**
- When both root `README.md` and `.github/README.md` exist, `.github/README.md` takes precedence
- Removing `.github/README.md` allows root `README.md` to display
- Renamed file (e.g., `GITHUB_CONFIG.md`) does not interfere

**Expected Result:**
- Root README displays (convention followed)
- No confusion or conflicts

**Actual Result:**
- Convention followed: [YES/NO]

**Pass/Fail:**
- **PASS**: Solution follows GitHub conventions
- **FAIL**: Non-standard approach used

---

## Overall Pass/Fail Criteria

**OVERALL PASS if:**
- ✅ Root `README.md` displays on repository landing page (AC-1)
- ✅ `.github/` directory accessible with renamed file (AC-2)
- ✅ Solution follows GitHub conventions (AC-3)
- ✅ No broken links in root README or `.github/` content (AC-4)

**OVERALL FAIL if:**
- ❌ Root README does not display
- ❌ `.github/` directory inaccessible or renamed file missing
- ❌ Broken links found
- ❌ Non-standard solution used

---

## Test Execution

**Tester:** [Name]

**Date:** [YYYY-MM-DD]

**Commit Hash:** [Git commit]

**Test Result:** [PASS / FAIL]

---

## Evidence

**Screenshots (optional but recommended):**
1. GitHub repository landing page showing root README
2. `.github/` directory listing showing renamed file
3. Link validation (if any issues found)

**Store in:** `tests/results/sprint-32/feature-12-verification/`

---

## Notes

**Estimated Time:** ~2 minutes

**Complexity:** Very Low (simple file rename verification)

**Risk:** Very Low (no code changes)

**No automated tests needed** - This is purely a GitHub UI verification task.

---

## Related Requirements

- **Feature #12**: Fix GitHub README Display
- **GitHub Issue #12**: [DOCS] Wrong readme display on GitHub
- **Sprint 32 Planning**: sprint-32-planning.md §Feature 2 (lines 96-130)
