# Quality Validation Report Template

**File Location:** `tests/results/YYYYMMDD-HHMMSS/REPORT.md`

```markdown
---
report_type: Quality Validation Report
executed: YYYY-MM-DD HH:MM:SS
commit: <git-commit-hash>
tester: quality-validator
total_tests: X
passed: X
failed: X
blocked: X
skipped: X
---

# Quality Validation Report

**Date**: YYYY-MM-DD HH:MM:SS
**Commit**: `<hash>`
**Test Coverage**: X test cases executed

## Executive Summary

[2-3 paragraphs summarizing overall quality assessment, key findings, and recommendation priority]

**Overall Assessment**: [Production Ready|Needs Minor Fixes|Needs Major Fixes|Not Ready]

## Test Type Coverage (MANDATORY)

**Test Strategy Reference:** `tests/strategy/sprint-N-test-strategy.md`

| Test Type | Strategy Status | Implemented | Executed | Results | Gap Impact |
|-----------|-----------------|-------------|----------|---------|------------|
| Unit tests | ✅ REQUIRED | ✅ Yes | ✅ Yes | 246/246 pass | N/A |
| Interactive tests (expectrl) | ✅ REQUIRED | ❌ No | ❌ No | N/A | HIGH - User behavior not validated |
| Integration tests | ⚠️ RECOMMENDED | ✅ Yes | ✅ Yes | 12/12 pass | N/A |
| Manual tests | ⚠️ RECOMMENDED | ⚠️ Partial | ❌ No | N/A | MEDIUM - UX validation incomplete |

## Findings

### Critical Issues
- **[TC###] Issue Title**
  - **Severity**: Critical
  - **Description**: ...
  - **Recommendation**: ...

### Major Issues
- ...

## Recommendations
1. [Immediate Action]
2. [Short Term]
```
