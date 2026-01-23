# Test Case Template

**File Location:** `tests/cases/TC###.md`

```markdown
---
id: TC###
title: Brief descriptive title
category: [Functionality|Usability|Error-Handling|Integration|Documentation]
priority: [Critical|High|Medium|Low]
created: YYYY-MM-DD
updated: YYYY-MM-DD
commit: <git-commit-hash>
---

# Test Case TC###: [Title]

## Purpose
Clear statement of what functionality/behavior this test validates.

## Scope
- What is being tested
- What is NOT being tested (exclusions)

## Prerequisites
- Required environment setup

## Test Procedure

### Setup
```bash
# Setup commands
```

### Execution Steps
1. Step 1 with expected outcome
2. Step 2 ...

### Verification
- What to check to confirm success

## Expected Results
Detailed description of expected behavior.
```
