# TC-LICENSE-MANUAL: Legal Compliance Manual Review

**Test Case ID:** TC-LICENSE-MANUAL
**Feature:** LICENSE Legal Compliance Review (#8)
**Test Type:** Manual (Legal Review)
**Priority:** P0 (BLOCKING for release)
**Created:** 2026-01-27
**Sprint:** Sprint 27

---

## Objective

Manually review LICENSE file for legal compliance, license compatibility, and adherence to Teradata redistribution terms. This review requires human judgment and legal knowledge that cannot be automated.

---

## Prerequisites

- [ ] LICENSE file exists and automated tests pass (TC-LICENSE-001, TC-LICENSE-002)
- [ ] Reviewer has basic understanding of open source licensing
- [ ] Reference licenses available:
  - MIT License: https://opensource.org/licenses/MIT
  - BSD 3-Clause: https://opensource.org/licenses/BSD-3-Clause
  - teradatarustapi LICENSE: https://github.com/Teradata/teradatarustapi/blob/main/LICENSE
  - teradatarustapi THIRDPARTYLICENSE: https://github.com/Teradata/teradatarustapi/blob/main/THIRDPARTYLICENSE

---

## Test Steps

### Step 1: Verify MIT License Accuracy for tq Project
**Action:** Compare tq's MIT license text against official MIT license

**Review:**
- [ ] MIT license text is complete (not truncated)
- [ ] Permission grant section present: "Permission is hereby granted, free of charge..."
- [ ] Warranty disclaimer present: "THE SOFTWARE IS PROVIDED 'AS IS'..."
- [ ] Copyright notice includes correct year(s): 2026 or appropriate range
- [ ] Copyright holder is identified correctly
- [ ] No modifications to MIT license text (must be verbatim)

**Expected Result:**
- tq MIT license matches official MIT template
- Copyright notice is accurate
- No custom modifications that invalidate license

### Step 2: Verify teradatarustapi License Attribution
**Action:** Compare teradatarustapi attribution against upstream LICENSE

**Review:**
- [ ] teradatarustapi attribution matches upstream LICENSE file
- [ ] BSD 3-Clause license text is complete
- [ ] Copyright notice from upstream is preserved
- [ ] All three BSD clauses present:
  1. Redistribution in source form must retain copyright notice
  2. Redistribution in binary form must reproduce copyright notice
  3. Neither name may be used to endorse without permission
- [ ] Attribution is accurate and not modified

**Expected Result:**
- teradatarustapi attribution is complete and accurate
- Matches upstream LICENSE exactly
- All required copyright notices preserved

### Step 3: Verify Go Language License Attribution
**Action:** Compare Go license attribution against upstream

**Review:**
- [ ] Go Authors copyright present (from teradatarustapi THIRDPARTYLICENSE)
- [ ] Go's BSD 3-Clause license text present
- [ ] Go license attribution matches Go project's LICENSE file
- [ ] Transitive dependency chain is clear (tq → teradatarustapi → Go)

**Expected Result:**
- Go license attribution is complete
- Matches Go project's official license
- Transitive nature is documented

### Step 4: Verify License Compatibility
**Action:** Check that all licenses are compatible with each other

**Review:**
- [ ] MIT (tq) is compatible with BSD (teradatarustapi) - **YES (both permissive)**
- [ ] BSD (teradatarustapi) is compatible with BSD (Go) - **YES (same license family)**
- [ ] No copyleft licenses (GPL, LGPL) that conflict with MIT/BSD - **Verify none present**
- [ ] Attribution requirements are met for all licenses
- [ ] No conflicting redistribution terms

**Expected Result:**
- All licenses are compatible
- No licensing conflicts
- tq can be redistributed under MIT while including BSD dependencies

### Step 5: Verify Redistribution Compliance
**Action:** Check compliance with teradatarustapi and Go redistribution terms

**Review:**
- [ ] BSD redistribution requirements met:
  - Source code redistribution: Copyright notices preserved ✓
  - Binary redistribution: Copyright notices in LICENSE file ✓
  - No use of names for endorsement without permission ✓
- [ ] teradatarustapi attribution is complete
- [ ] Go attribution is complete
- [ ] LICENSE file will be included in tq releases

**Expected Result:**
- All redistribution requirements satisfied
- tq can legally redistribute with teradatarustapi and Go dependencies
- No missing attributions or copyright notices

### Step 6: Verify Completeness and Clarity
**Action:** Read LICENSE file as a user would

**Review:**
- [ ] LICENSE file is easy to understand
- [ ] Clear separation between tq license and third-party licenses
- [ ] No confusing or contradictory statements
- [ ] No misleading claims (e.g., "MIT only" when dependencies are BSD)
- [ ] Professional presentation

**Expected Result:**
- LICENSE file is clear and user-friendly
- No legal confusion
- Transparent about multi-license nature

### Step 7: Verify No Red Flags
**Action:** Check for common licensing mistakes

**Review:**
- [ ] No dual licensing without clear explanation
- [ ] No patent termination clauses that conflict with MIT
- [ ] No additional restrictions beyond original licenses
- [ ] No trademark restrictions beyond BSD clause 3
- [ ] No license text modifications that invalidate licenses

**Expected Result:**
- No red flags found
- LICENSE is legally sound
- No custom modifications that create legal issues

### Step 8: Final Legal Compliance Assessment
**Action:** Overall assessment of legal compliance

**Review:**
- [ ] tq MIT license: COMPLIANT / NON-COMPLIANT
- [ ] teradatarustapi BSD license: COMPLIANT / NON-COMPLIANT
- [ ] Go BSD license: COMPLIANT / NON-COMPLIANT
- [ ] License compatibility: COMPATIBLE / INCOMPATIBLE
- [ ] Redistribution terms: MET / NOT MET
- [ ] Overall compliance: APPROVED / NEEDS REVISION

**Expected Result:**
- All aspects COMPLIANT
- Overall assessment: APPROVED for release
- No legal blockers

---

## Expected Results

### Success Criteria
- [x] MIT license for tq is accurate and complete
- [x] teradatarustapi BSD license attribution is accurate
- [x] Go BSD license attribution is accurate
- [x] All licenses are compatible (MIT + BSD + BSD)
- [x] Redistribution requirements are met
- [x] LICENSE file is clear and professional
- [x] No legal red flags
- [x] Overall legal compliance: APPROVED

### Compliance Checklist
| Aspect | Status | Notes |
|--------|--------|-------|
| tq MIT license accuracy | [PASS/FAIL] | |
| teradatarustapi attribution | [PASS/FAIL] | |
| Go attribution | [PASS/FAIL] | |
| License compatibility | [PASS/FAIL] | |
| Redistribution compliance | [PASS/FAIL] | |
| Clarity and completeness | [PASS/FAIL] | |
| No red flags | [PASS/FAIL] | |
| **Overall Compliance** | [APPROVED/NEEDS REVISION] | **BLOCKING** |

---

## Actual Results

**Review Date:** [To be filled by reviewer]
**Reviewer:** [Name and qualifications]
**Build Version:** [Commit hash]

**tq MIT License Review:**
```
Accurate: [YES/NO]
Complete: [YES/NO]
Copyright correct: [YES/NO]
Issues found: [None / List]
```

**teradatarustapi Attribution Review:**
```
Matches upstream: [YES/NO]
BSD text complete: [YES/NO]
Copyright preserved: [YES/NO]
Issues found: [None / List]
```

**Go Attribution Review:**
```
Matches upstream: [YES/NO]
BSD text complete: [YES/NO]
Transitive chain clear: [YES/NO]
Issues found: [None / List]
```

**License Compatibility:**
```
MIT + BSD compatible: [YES/NO]
No GPL conflicts: [YES/NO]
Issues found: [None / List]
```

**Redistribution Compliance:**
```
BSD requirements met: [YES/NO]
Attributions complete: [YES/NO]
Issues found: [None / List]
```

**Legal Red Flags:**
```
[List any concerns or "None found"]
```

**Overall Assessment:**
```
Legal Compliance: [APPROVED / NEEDS REVISION]

If NEEDS REVISION:
- Issue 1: [Description]
- Issue 2: [Description]
- Recommended changes: [List]

If APPROVED:
- LICENSE is legally compliant and ready for release
```

---

## Pass/Fail Status

**Status:** [APPROVED | NEEDS REVISION | BLOCKED]

**Pass Condition:**
- APPROVED: All licenses accurate, compatible, compliant - READY FOR RELEASE
- NEEDS REVISION: Issues found that must be corrected
- BLOCKED: Major legal issues, consult legal counsel

**Critical Issues Found:**
- [If NEEDS REVISION: List blocking legal issues]

**Recommendations:**
- [Suggestions for improvement, even if approved]

---

## Notes

- **This is a BLOCKING review** - Sprint 27 cannot be released without APPROVED status
- Automated tests (TC-LICENSE-001, TC-LICENSE-002) check structure, not legal validity
- This manual review addresses legal compliance that cannot be automated
- Reviewer should have basic understanding of MIT and BSD licenses
- If major legal issues found, consider consulting legal counsel
- This test validates compliance with AC-LICENSE-006: "Compliance with Teradata redistribution terms verified"

---

## Related Requirements

- AC-LICENSE-006: "Compliance with Teradata redistribution terms verified" (sprint-27-planning.md:97)
- AC-LICENSE-001: "LICENSE file updated with complete terms" (sprint-27-planning.md:92)
- AC-LICENSE-002: "teradatarustapi license attribution included" (sprint-27-planning.md:93)
- AC-LICENSE-003: "Go license attribution included" (sprint-27-planning.md:94)
- GitHub Issue #8: LICENSE - Legal compliance and transparency
- Sprint 27 Success: Legal compliance is P0 requirement
