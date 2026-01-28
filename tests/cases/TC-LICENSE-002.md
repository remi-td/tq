# TC-LICENSE-002: LICENSE Attribution Validation

**Test Case ID:** TC-LICENSE-002
**Feature:** LICENSE Third-Party Attribution (#8)
**Test Type:** Integration (Content Validation)
**Priority:** P0
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Verify that LICENSE file contains required attributions for MIT license (tq project), teradatarustapi license, and Go language license as required by transitive dependencies.

---

## Prerequisites

- [ ] tq project repository checked out
- [ ] LICENSE file exists (see TC-LICENSE-001)
- [ ] Reference license URLs known:
  - https://github.com/Teradata/teradatarustapi/blob/main/LICENSE
  - https://github.com/Teradata/teradatarustapi/blob/main/THIRDPARTYLICENSE

---

## Test Steps

### Step 1: Verify MIT License for tq Project
**Action:** Check that LICENSE contains MIT license for tq
```bash
grep -i 'MIT License' LICENSE
```

**Expected Result:**
- "MIT License" text found
- Indicates tq project's base license

### Step 2: Verify tq Copyright Statement
**Action:** Search for tq project copyright
```bash
grep -i 'copyright.*tq' LICENSE
```

**Expected Result:**
- tq project copyright statement found
- Includes year (2026 or range)
- Identifies copyright holder

### Step 3: Verify teradatarustapi Attribution
**Action:** Search for teradatarustapi license attribution
```bash
grep -i 'teradatarustapi' LICENSE
```

**Expected Result:**
- teradatarustapi attribution found
- Indicates dependency license is included

### Step 4: Verify Go Language Attribution
**Action:** Search for Go language license attribution
```bash
grep -i 'Go Authors\|golang\|go language' LICENSE
```

**Expected Result:**
- Go Authors or similar attribution found
- Indicates transitive dependency license (via teradatarustapi)

### Step 5: Verify BSD License (teradatarustapi)
**Action:** Search for BSD license text (teradatarustapi uses BSD)
```bash
grep -i 'BSD' LICENSE
```

**Expected Result:**
- BSD license text found
- teradatarustapi is BSD-licensed
- Attribution is complete

### Step 6: Verify All Required License Types Present
**Action:** Check for both MIT and BSD license types
```bash
# Count different license types
grep -E 'MIT License|BSD.*License' LICENSE | wc -l
```

**Expected Result:**
- Both MIT and BSD license types present
- Indicates multi-license structure
- Count ≥ 2

### Step 7: Verify License Text Completeness
**Action:** Check that license text is substantial (not just headers)
```bash
# MIT license text should contain key phrases
grep 'Permission is hereby granted, free of charge' LICENSE
grep 'THE SOFTWARE IS PROVIDED "AS IS"' LICENSE

# BSD license text should contain key phrases
grep 'Redistribution and use in source and binary forms' LICENSE
```

**Expected Result:**
- MIT license full text present (permission grant, warranty disclaimer)
- BSD license full text present (redistribution terms)
- Not just license names, but complete license text

### Step 8: Verify Multiple Copyright Years/Holders
**Action:** Count unique copyright statements
```bash
grep -i 'copyright' LICENSE | grep -v '^#' | sort -u
```

**Expected Result:**
- Multiple copyright statements (tq + third parties)
- Different copyright holders identified
- Multiple years or year ranges

---

## Expected Results

### Success Criteria
- [x] MIT License text present (tq project)
- [x] tq project copyright statement present
- [x] teradatarustapi attribution present
- [x] Go Authors attribution present (transitive dependency)
- [x] BSD license text present (teradatarustapi)
- [x] Complete license text (not just headers)
- [x] Multiple copyright holders identified

### Attribution Checklist
| Attribution | Required Text | Found | Notes |
|-------------|---------------|-------|-------|
| tq MIT | "MIT License" | [Y/N] | tq project base license |
| tq Copyright | "Copyright" + year + holder | [Y/N] | tq copyright |
| teradatarustapi | "teradatarustapi" | [Y/N] | Direct dependency |
| Go Authors | "Go Authors" or "golang" | [Y/N] | Transitive via teradatarustapi |
| BSD License | "BSD" + full text | [Y/N] | teradatarustapi license |
| MIT Full Text | "Permission is hereby granted" | [Y/N] | Complete MIT text |
| BSD Full Text | "Redistribution and use" | [Y/N] | Complete BSD text |

---

## Actual Results

**Test Execution Date:** [To be filled during execution]
**Tester:** quality-validator
**Build Version:** [Commit hash]

**MIT License Check:**
```
$ grep -i 'MIT License' LICENSE
[Output]
```

**tq Copyright Check:**
```
$ grep -i 'copyright.*tq' LICENSE
[Output]
```

**teradatarustapi Attribution:**
```
$ grep -i 'teradatarustapi' LICENSE
[Output]
```

**Go Authors Attribution:**
```
$ grep -i 'Go Authors\|golang' LICENSE
[Output]
```

**BSD License Check:**
```
$ grep -i 'BSD' LICENSE
[Output]
```

**License Text Completeness:**
```
$ grep 'Permission is hereby granted' LICENSE
[Output - should find MIT text]

$ grep 'Redistribution and use' LICENSE
[Output - should find BSD text]
```

**All Copyright Statements:**
```
$ grep -i 'copyright' LICENSE | grep -v '^#' | sort -u
[List all unique copyright statements]
```

---

## Pass/Fail Status

**Status:** [PASS | FAIL | BLOCKED]

**Pass Condition:**
- PASS: All required attributions present (MIT for tq, BSD for teradatarustapi, Go Authors)
- FAIL: Any required attribution missing
- BLOCKED: LICENSE file does not exist

**Missing Attributions:**
- [If FAIL: List which attributions are missing]

---

## Notes

- teradatarustapi is BSD-licensed (see upstream LICENSE)
- teradatarustapi has THIRDPARTYLICENSE file including Go license
- Go language is BSD-licensed (3-clause BSD)
- This test validates attribution presence, not legal correctness
- Legal review required (see TC-LICENSE-MANUAL)

---

## Related Requirements

- AC-LICENSE-002: "teradatarustapi license attribution included" (sprint-27-planning.md:93)
- AC-LICENSE-003: "Go license attribution included" (sprint-27-planning.md:94)
- AC-LICENSE-001: "LICENSE file updated with complete terms" (sprint-27-planning.md:92)
- GitHub Issue #8: LICENSE - Current MIT-only is incomplete/misleading
- Upstream Reference: https://github.com/Teradata/teradatarustapi/blob/main/LICENSE
- Upstream Reference: https://github.com/Teradata/teradatarustapi/blob/main/THIRDPARTYLICENSE
