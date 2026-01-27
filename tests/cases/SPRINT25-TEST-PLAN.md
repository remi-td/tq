# Sprint 25 Test Plan: Documentation & Issue Template Fixes

**Sprint:** 25
**Created:** 2026-01-27
**Author:** quality-validator
**Test Type:** Manual Validation + Automated Verification
**Status:** READY FOR EXECUTION

---

## Overview

Sprint 25 addresses two high-priority documentation bugs that require primarily manual validation with supporting automated checks. No Rust code changes are involved.

**Features Under Test:**
1. Feature 1: Fix Duplicate Roadmap Documentation (#4)
2. Feature 2: Fix Documentation Issue Template (#5)

**Test Approach:** Manual validation with automated verification commands

---

## Feature 1: Fix Duplicate Roadmap Documentation

### Acceptance Criteria
- [ ] `docs/user/roadmap.md` deleted
- [ ] All cross-references updated to point to `docs/roadmap/roadmap.md`
- [ ] No broken links in documentation
- [ ] User guide index updated if needed

### Test Procedures

#### Automated Verification

**Test 1.1: Verify File Deletion**
```bash
test ! -f docs/user/roadmap.md && echo "✅ PASS: File deleted" || echo "❌ FAIL: File still exists"
```

**Expected:** PASS - File does not exist

---

**Test 1.2: Verify No Stale References**
```bash
grep -r "docs/user/roadmap.md" docs/ CLAUDE.md README.md 2>/dev/null
echo "Exit code: $?"
```

**Expected:** Exit code 1 (no matches found) = PASS

---

#### Manual Verification

**Test 1.3: Documentation Links Work**

Checklist:
- [ ] Open `README.md` in browser/editor
  - [ ] Verify any roadmap links point to `docs/roadmap/roadmap.md`
  - [ ] Click links to verify they work (no 404)
- [ ] Open `CLAUDE.md` in browser/editor
  - [ ] Find "Documentation Organization" section
  - [ ] Verify it references correct roadmap location
  - [ ] Verify accuracy of documentation structure description
- [ ] Open `docs/roadmap/roadmap.md` directly
  - [ ] Verify file renders correctly
  - [ ] Verify content is intact

---

**Test 1.4: User Guide References**

Checklist:
- [ ] Open `docs/user/repl-guide.md`
  - [ ] Search for "roadmap" references
  - [ ] Verify any references point to correct location
- [ ] Open `docs/user/batch-mode-guide.md`
  - [ ] Search for "roadmap" references
  - [ ] Verify any references point to correct location

---

**Test 1.5: Directory Structure**

```bash
ls -la docs/user/
```

Checklist:
- [ ] Confirm `docs/user/roadmap.md` does NOT appear in listing
- [ ] Verify only expected files remain (repl-guide.md, batch-mode-guide.md)

---

### Pass/Fail Criteria

**PASS if ALL of the following are true:**
- ✅ File `docs/user/roadmap.md` does NOT exist
- ✅ Zero matches when searching for `docs/user/roadmap.md` references
- ✅ All roadmap links in documentation work (no 404s)
- ✅ User guide navigation consistent

**FAIL if ANY of the following are true:**
- ❌ File `docs/user/roadmap.md` still exists
- ❌ Stale references to `docs/user/roadmap.md` found
- ❌ Broken links after change
- ❌ User guide references incorrect

---

## Feature 2: Fix Documentation Issue Template

### Acceptance Criteria
- [ ] Documentation issue template creates successfully (no 404)
- [ ] Template file path correct in `.github/ISSUE_TEMPLATE/config.yml`
- [ ] Template renders properly with all fields
- [ ] Test creating a documentation issue end-to-end

### Test Procedures

#### Automated Verification

**Test 2.1: Validate YAML Syntax**
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/ISSUE_TEMPLATE/config.yml'))" && echo "✅ PASS: YAML valid" || echo "❌ FAIL: YAML syntax error"
```

**Expected:** PASS - YAML parses without errors

---

**Test 2.2: Inspect Configuration**
```bash
cat .github/ISSUE_TEMPLATE/config.yml
```

**Manual Verification:**
- [ ] Verify `blank_issues_enabled: false` or `true` (as designed)
- [ ] Verify `contact_links` array present
- [ ] Verify each contact link has `name`, `url`, `about` fields
- [ ] Verify URLs are correct (no typos)
- [ ] Verify repository references correct (remi-td/tq)

---

**Test 2.3: Verify Template Files Exist**
```bash
ls -la .github/ISSUE_TEMPLATE/
```

**Manual Verification:**
- [ ] Confirm `bug_report.md` exists
- [ ] Confirm `feature_request.md` exists
- [ ] Confirm `config.yml` exists
- [ ] Verify no other unexpected files

---

#### Manual GitHub UI Testing

**Test 2.4: End-to-End Issue Creation Flow**

**Prerequisites:**
- Web browser
- GitHub account with access to repository
- Internet connection

**Steps:**
1. Navigate to https://github.com/remi-td/tq/issues/new/choose
2. Observe the issue template selection page
3. Verify the following options appear:
   - [ ] Bug Report option
   - [ ] Feature Request option
   - [ ] Contact links for Documentation and Discussions (if configured)
4. If "Documentation" is a contact link:
   - [ ] Click on "Documentation" link
   - [ ] Verify it redirects to correct URL (likely README.md)
   - [ ] Verify NO 404 error occurs
5. If "Documentation" is a template:
   - [ ] Click "Get started" for Documentation template
   - [ ] Verify template loads successfully (no 404)
   - [ ] Verify all expected fields appear
6. Take screenshot as proof of successful test

**Expected Results:**
- No 404 errors when clicking any option
- Documentation link/template works correctly
- All templates render properly

---

**Test 2.5: Template Content Validation**

For each template file (`bug_report.md`, `feature_request.md`):

**bug_report.md:**
```bash
cat .github/ISSUE_TEMPLATE/bug_report.md
```

Checklist:
- [ ] Verify YAML frontmatter present (between `---` delimiters)
- [ ] Verify `name`, `about`, `title`, `labels` fields present
- [ ] Verify template body has appropriate sections
- [ ] Verify no broken markdown formatting

**feature_request.md:**
```bash
cat .github/ISSUE_TEMPLATE/feature_request.md
```

Checklist:
- [ ] Verify YAML frontmatter present
- [ ] Verify `name`, `about`, `title`, `labels` fields present
- [ ] Verify template body has appropriate sections
- [ ] Verify no broken markdown formatting

---

### Pass/Fail Criteria

**PASS if ALL of the following are true:**
- ✅ YAML config parses without errors
- ✅ All file paths in config exist
- ✅ Creating documentation issue via GitHub does NOT produce 404
- ✅ Template/contact link renders correctly
- ✅ End-to-end issue creation flow works

**FAIL if ANY of the following are true:**
- ❌ YAML syntax error in config
- ❌ File paths in config do not exist
- ❌ GitHub UI shows 404 error when accessing documentation
- ❌ Templates do not render
- ❌ Missing expected fields

---

## Test Execution Checklist

### Prerequisites
- [ ] Git repository clean (no uncommitted changes before testing)
- [ ] Access to GitHub repository (for UI testing)
- [ ] Python 3 available (for YAML validation)
- [ ] Web browser available (for manual testing)

### Execution Order

1. **Feature 1: Roadmap Documentation**
   - [ ] Run automated file deletion check (Test 1.1)
   - [ ] Run automated reference search (Test 1.2)
   - [ ] Perform manual link verification (Test 1.3)
   - [ ] Verify user guide references (Test 1.4)
   - [ ] Inspect directory structure (Test 1.5)

2. **Feature 2: Issue Template**
   - [ ] Run YAML validation (Test 2.1)
   - [ ] Inspect configuration content (Test 2.2)
   - [ ] Verify template files exist (Test 2.3)
   - [ ] Perform GitHub UI testing (Test 2.4)
   - [ ] Validate template content (Test 2.5)

3. **Documentation**
   - [ ] Create test report in `tests/results/sprint-25/REPORT.md`
   - [ ] Include all test results (pass/fail)
   - [ ] Include screenshots from GitHub UI testing
   - [ ] Document any issues found

---

## Test Report Structure

Create `tests/results/sprint-25/REPORT.md` with:

```markdown
---
verdict: APPROVED | REJECTED | BLOCKED
sprint: 25
date: YYYY-MM-DD
tester: quality-validator
---

# Sprint 25 Test Report: Documentation & Issue Template Fixes

## Executive Summary
[Overall verdict and key findings]

## Feature 1: Fix Duplicate Roadmap Documentation

### Test Results
- Test 1.1 (File Deletion): ✅ PASS / ❌ FAIL
- Test 1.2 (Reference Search): ✅ PASS / ❌ FAIL
- Test 1.3 (Link Verification): ✅ PASS / ❌ FAIL
- Test 1.4 (User Guide): ✅ PASS / ❌ FAIL
- Test 1.5 (Directory Structure): ✅ PASS / ❌ FAIL

### Evidence
[Paste command output and observations]

## Feature 2: Fix Documentation Issue Template

### Test Results
- Test 2.1 (YAML Validation): ✅ PASS / ❌ FAIL
- Test 2.2 (Config Inspection): ✅ PASS / ❌ FAIL
- Test 2.3 (File Existence): ✅ PASS / ❌ FAIL
- Test 2.4 (GitHub UI): ✅ PASS / ❌ FAIL
- Test 2.5 (Template Content): ✅ PASS / ❌ FAIL

### Evidence
[Paste command output, screenshots, observations]

## Overall Verdict

**Verdict:** [APPROVED | REJECTED | BLOCKED]

**Rationale:** [Why this verdict was assigned]

## Issues Found
[List any issues discovered, or "None"]

## Recommendations
[Any suggestions for improvements]
```

---

## Notes

**Sprint Characteristics:**
- No Rust code changes (documentation/configuration only)
- No database required for testing
- No interactive (expectrl) tests needed
- Primarily manual verification with automated supporting checks

**Testing Tools Used:**
- Bash (grep, test, ls)
- Python 3 (YAML validation)
- Web browser (GitHub UI testing)
- Text editor (manual inspection)

**Estimated Testing Time:** 30-45 minutes
