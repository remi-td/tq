# TC-LICENSE-003: NOTICE or THIRD-PARTY-LICENSES File Check

**Test Case ID:** TC-LICENSE-003
**Feature:** LICENSE Third-Party Notices (#8)
**Test Type:** Integration (File Validation)
**Priority:** P1
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Verify that if a NOTICE or THIRD-PARTY-LICENSES file is created (as mentioned in AC-LICENSE-004), it exists and contains proper third-party attributions.

---

## Prerequisites

- [ ] tq project repository checked out
- [ ] Working directory is project root
- [ ] LICENSE file exists and validated (TC-LICENSE-001, TC-LICENSE-002)

---

## Test Steps

### Step 1: Check if NOTICE File Exists
**Action:** Look for NOTICE file at project root
```bash
ls -la NOTICE 2>/dev/null || echo "NOTICE file not found"
```

**Expected Result:**
- Either:
  - NOTICE file exists, OR
  - No NOTICE file (acceptable if attributions are in LICENSE)

### Step 2: Check if THIRD-PARTY-LICENSES File Exists
**Action:** Look for THIRD-PARTY-LICENSES file variants
```bash
ls -la THIRD-PARTY-LICENSES* 2>/dev/null || echo "THIRD-PARTY-LICENSES not found"
ls -la THIRD_PARTY_LICENSES* 2>/dev/null || echo "THIRD_PARTY_LICENSES not found"
```

**Expected Result:**
- Either:
  - THIRD-PARTY-LICENSES file exists, OR
  - No separate file (acceptable if attributions are in LICENSE)

### Step 3: If NOTICE Exists - Validate Content
**Action:** If NOTICE file exists, check content
```bash
if [ -f NOTICE ]; then
    cat NOTICE
    grep -i 'teradatarustapi' NOTICE
    grep -i 'copyright' NOTICE
fi
```

**Expected Result:**
- File is readable
- Contains third-party attributions
- teradatarustapi mentioned
- Copyright notices present

### Step 4: If THIRD-PARTY-LICENSES Exists - Validate Content
**Action:** If separate third-party file exists, check content
```bash
if [ -f THIRD-PARTY-LICENSES ]; then
    cat THIRD-PARTY-LICENSES
    grep -i 'teradatarustapi' THIRD-PARTY-LICENSES
    grep -i 'Go Authors' THIRD-PARTY-LICENSES
fi
```

**Expected Result:**
- File is readable
- Contains complete third-party licenses
- teradatarustapi license text included
- Go license text included

### Step 5: Verify Attributions Location
**Action:** Determine where third-party attributions are located
```bash
# Check if attributions are in LICENSE file
if grep -q 'teradatarustapi' LICENSE; then
    echo "Attributions in LICENSE file"
fi

# Or in separate NOTICE/THIRD-PARTY-LICENSES
if [ -f NOTICE ] && grep -q 'teradatarustapi' NOTICE; then
    echo "Attributions in NOTICE file"
fi

if [ -f THIRD-PARTY-LICENSES ] && grep -q 'teradatarustapi' THIRD-PARTY-LICENSES; then
    echo "Attributions in THIRD-PARTY-LICENSES file"
fi
```

**Expected Result:**
- Third-party attributions are in ONE of:
  - LICENSE file (all-in-one approach), OR
  - NOTICE file (Apache-style approach), OR
  - THIRD-PARTY-LICENSES file (explicit separation)
- Attributions are complete in whichever location chosen

### Step 6: Verify No Duplicate Attributions
**Action:** If multiple files exist, check for unnecessary duplication
```bash
# Count total attribution size
if [ -f LICENSE ]; then wc -l LICENSE; fi
if [ -f NOTICE ]; then wc -l NOTICE; fi
if [ -f THIRD-PARTY-LICENSES ]; then wc -l THIRD-PARTY-LICENSES; fi
```

**Expected Result:**
- No excessive duplication
- Clear separation of concerns if multiple files exist
- LICENSE = main license
- NOTICE = attribution notices (if needed)
- THIRD-PARTY-LICENSES = third-party full licenses (if needed)

---

## Expected Results

### Success Criteria
- [x] Third-party attributions are present in project
- [x] Attributions are in LICENSE, NOTICE, or THIRD-PARTY-LICENSES (or combination)
- [x] If separate NOTICE file exists, it contains proper attributions
- [x] If separate THIRD-PARTY-LICENSES exists, it contains full license text
- [x] No critical missing attributions
- [x] Clear organization (no confusion about where to find licenses)

### File Organization Patterns (All Valid)
**Pattern 1: All-in-One (Recommended for small projects)**
- LICENSE file contains: tq MIT + teradatarustapi BSD + Go BSD
- No NOTICE or THIRD-PARTY-LICENSES files

**Pattern 2: LICENSE + NOTICE**
- LICENSE file contains: tq MIT license
- NOTICE file contains: Third-party attribution notices

**Pattern 3: LICENSE + THIRD-PARTY-LICENSES**
- LICENSE file contains: tq MIT license
- THIRD-PARTY-LICENSES file contains: Full third-party license text

**Pattern 4: Three-File Split**
- LICENSE file contains: tq MIT license
- NOTICE file contains: Short attribution notices
- THIRD-PARTY-LICENSES file contains: Full third-party license text

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** quality-validator
**Build Version:** [Commit hash]

**Files Found:**
```
$ ls -la LICENSE NOTICE THIRD-PARTY-LICENSES 2>/dev/null
[Output - list which files exist]
```

**Attribution Location:**
```
Attributions found in:
- LICENSE: [YES/NO]
- NOTICE: [YES/NO or N/A - file does not exist]
- THIRD-PARTY-LICENSES: [YES/NO or N/A - file does not exist]
```

**File Organization Pattern:**
```
Pattern used: [Pattern 1 / Pattern 2 / Pattern 3 / Pattern 4]
Rationale: [Why this pattern makes sense for tq project]
```

**Content Validation (if separate files exist):**
```
NOTICE content: [Summary or "N/A"]
THIRD-PARTY-LICENSES content: [Summary or "N/A"]
```

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Pass Condition:**
- PASS: Third-party attributions present in clear location (any valid pattern)
- FAIL: Attributions missing or unclear organization
- BLOCKED: LICENSE file does not exist

**Observations:**
- [Note which file organization pattern is used]
- [Note if pattern is appropriate for tq project size]

---

## Notes

- AC-LICENSE-004 says "if needed" - not always required
- All-in-one LICENSE approach (Pattern 1) is valid and common for small projects
- Separate NOTICE file is more common in large projects or Apache-licensed projects
- tq is small project with 2-3 dependency licenses, so single LICENSE file is acceptable
- Key requirement: Attributions must be SOMEWHERE clear and complete

---

## Related Requirements

- AC-LICENSE-004: "NOTICE or THIRD-PARTY-LICENSES file created if needed" (sprint-27-planning.md:95)
- AC-LICENSE-002: "teradatarustapi license attribution included" (sprint-27-planning.md:93)
- AC-LICENSE-003: "Go license attribution included" (sprint-27-planning.md:94)
- GitHub Issue #8: LICENSE organization
