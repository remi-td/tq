# TC-LICENSE-001: LICENSE File Existence and Completeness

**Test Case ID:** TC-LICENSE-001
**Feature:** LICENSE File Validation (#8)
**Test Type:** Integration (File Validation)
**Priority:** P0
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Verify that the LICENSE file exists at project root, is readable, and contains complete license text without placeholders or TODO comments.

---

## Prerequisites

- [ ] tq project repository checked out
- [ ] Working directory is project root
- [ ] LICENSE file should be present after Sprint 27 implementation

---

## Test Steps

### Step 1: Verify LICENSE File Exists
**Action:** Check that LICENSE file exists at project root
```bash
ls -la LICENSE
```

**Expected Result:**
- File exists at ./LICENSE
- File is readable
- File has reasonable size (> 1000 bytes, < 50000 bytes)

### Step 2: Read LICENSE File
**Action:** Read LICENSE file contents
```bash
cat LICENSE
```

**Expected Result:**
- File is readable (no permission errors)
- Contains text content (not binary)
- Multiple sections visible

### Step 3: Verify No Placeholder Text
**Action:** Search for common placeholder patterns
```bash
grep -E '\[YEAR\]|\[COPYRIGHT HOLDER\]|\<COPYRIGHT HOLDER\>|\<YEAR\>' LICENSE
```

**Expected Result:**
- No placeholder patterns found
- Actual years and copyright holders specified
- Exit code 1 (no matches)

### Step 4: Verify No TODO/FIXME Comments
**Action:** Search for incomplete sections
```bash
grep -iE 'TODO|FIXME|XXX|TBD|PLACEHOLDER' LICENSE
```

**Expected Result:**
- No TODO, FIXME, XXX, TBD, or PLACEHOLDER text found
- License is production-ready
- Exit code 1 (no matches)

### Step 5: Verify Multiple License Sections
**Action:** Check that LICENSE contains multiple license sections (MIT + third-party)
```bash
# Count license section headers
grep -c -i 'license' LICENSE
```

**Expected Result:**
- Multiple license sections present (≥ 2)
- Indicates multi-license structure (tq MIT + third-party)

### Step 6: Verify Reasonable File Size
**Action:** Check LICENSE file size
```bash
wc -c LICENSE
```

**Expected Result:**
- File size > 1000 bytes (has substantial content)
- File size < 50000 bytes (reasonable for multi-license file)
- Not a stub file

### Step 7: Verify Multiple Copyright Holders
**Action:** Check for copyright statements
```bash
grep -i 'copyright' LICENSE
```

**Expected Result:**
- Multiple copyright statements (tq project + third parties)
- Year(s) specified (e.g., 2026, or range like 2019-2024)
- Copyright holders identified

---

## Expected Results

### Success Criteria
- [x] LICENSE file exists at project root
- [x] File is readable (no permission issues)
- [x] No placeholder text ([YEAR], <COPYRIGHT HOLDER>)
- [x] No TODO/FIXME/XXX comments
- [x] Multiple license sections present
- [x] Reasonable file size (indicates complete content)
- [x] Multiple copyright statements (tq + third parties)

### File Validation Checklist
- [ ] File exists: ./LICENSE
- [ ] File size: > 1000 bytes
- [ ] No placeholders: [YEAR], <COPYRIGHT HOLDER>
- [ ] No TODOs: TODO, FIXME, XXX, TBD
- [ ] Multi-license structure: ≥ 2 license sections
- [ ] Copyright statements: Multiple holders, actual years

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** quality-validator
**Build Version:** [Commit hash]

**File Existence:**
```
$ ls -la LICENSE
[Output]
```

**File Size:**
```
$ wc -c LICENSE
[Byte count] LICENSE
```

**Placeholder Check:**
```
$ grep -E '\[YEAR\]|\[COPYRIGHT HOLDER\]' LICENSE
[Output - should be empty]
```

**TODO Check:**
```
$ grep -iE 'TODO|FIXME|XXX' LICENSE
[Output - should be empty]
```

**License Section Count:**
```
$ grep -c -i 'license' LICENSE
[Count - should be ≥ 2]
```

**Copyright Statements:**
```
$ grep -i 'copyright' LICENSE
[List of copyright statements]
```

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Pass Condition:**
- PASS: All validation checks pass, file is complete and ready for release
- FAIL: Placeholders, TODOs, or missing content found
- BLOCKED: File does not exist

**Defects Found:**
- [If FAIL: List what is incomplete or invalid]

---

## Notes

- This test validates file structure and completeness, not legal correctness
- Legal compliance requires manual review (see TC-LICENSE-MANUAL)
- Automated tests cannot validate license compatibility or legal requirements
- This test ensures LICENSE file is production-ready (no draft artifacts)

---

## Related Requirements

- AC-LICENSE-001: "LICENSE file updated with complete terms" (sprint-27-planning.md:92)
- GitHub Issue #8: LICENSE - Proper licensing
- File Location: ./LICENSE (project root)
