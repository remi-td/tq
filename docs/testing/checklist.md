# Test Implementation Checklist

This checklist prevents test implementation gaps like the one in Sprint 22 Iteration 1, where test strategy specified integration/PTY tests but only unit tests were implemented.

## Purpose

**Problem**: Test strategy documents don't guarantee tests are actually implemented.

**Solution**: Mandatory pre-review verification before requesting quality-validator review.

**Sprint 22 Example**: Iteration 1 rejected because Feature 2 strategy specified integration + PTY tests, but rust-teradata-architect only implemented unit tests.

---

## Pre-Review Verification (MANDATORY)

Before requesting quality-validator review, rust-teradata-architect MUST complete this checklist:

### 1. Test Strategy Alignment ✅

- [ ] **Read test strategy document**: `tests/strategy/sprint-N-test-strategy.md`
- [ ] **Verify feature coverage**: All features listed in strategy have corresponding tests
- [ ] **Check test type requirements**: For each feature, confirm required test types are implemented

### 2. Test Type Implementation ✅

For each feature in the sprint, verify:

#### Unit Tests
- [ ] All new functions have unit tests
- [ ] All new modules have test modules (`#[cfg(test)] mod tests`)
- [ ] Edge cases covered (empty input, NULL values, boundaries)
- [ ] Error paths tested (invalid input, failures)

#### Integration Tests (if strategy requires)
- [ ] Tests exist in `tests/integration_tests.rs` or feature-specific file
- [ ] Database-dependent tests marked with `#[ignore]` and run with `--ignored`
- [ ] Tests use live database connection (not mocks)
- [ ] Tests validate end-to-end behavior (CLI → database → output)
- [ ] Tests cover all output formats if applicable (table, CSV, JSON)

#### PTY/Interactive Tests (if strategy requires)
- [ ] Tests exist in `tests/interactive_tests.rs`
- [ ] Tests marked with `#[ignore]` and run with `--test-threads=1`
- [ ] Tests spawn real REPL process
- [ ] Tests validate user-visible output (not just internal state)
- [ ] Tests cover keyboard interactions if applicable (Tab, Enter, arrows)

#### Manual Tests (if strategy requires)
- [ ] Test procedures documented in `tests/cases/` with `MANUAL-` prefix
- [ ] Procedures include clear step-by-step instructions
- [ ] Expected results clearly defined
- [ ] Evidence requirements specified (screenshots, command output)

### 3. Local Test Execution ✅

Run ALL test types locally before submitting:

```bash
# Unit tests
cargo test --lib

# Integration tests (requires database)
cargo test --test integration_tests -- --ignored --test-threads=1

# PTY tests (requires database)
cargo test --test interactive_tests -- --ignored --test-threads=1

# All tests together
cargo test && cargo test -- --ignored --test-threads=1
```

Verify:
- [ ] All unit tests pass (100%)
- [ ] All integration tests pass (100%) or failures documented with explanation
- [ ] All PTY tests pass (100%) or failures documented with explanation
- [ ] No new clippy warnings
- [ ] No new compiler warnings

### 4. Test Coverage Verification ✅

Check test count against strategy:

- [ ] **Count unit tests**: `grep -r "#\[test\]" src/ | wc -l`
- [ ] **Count integration tests**: `grep -r "#\[test\]" tests/integration_*.rs | wc -l`
- [ ] **Count PTY tests**: `grep -r "#\[test\]" tests/interactive_tests.rs | wc -l`
- [ ] **Compare to strategy**: Test counts match or exceed strategy estimates

### 5. Documentation Updates ✅

- [ ] Test case documentation created/updated in `tests/cases/`
- [ ] Test case INDEX.md updated with new tests
- [ ] Design documentation updated if architecture changed
- [ ] User documentation matches implementation (no deferred features documented)

---

## Test Type Decision Matrix

Use this to determine which test types are required for each feature:

| Feature Type | Unit | Integration | PTY | Manual |
|--------------|------|-------------|-----|--------|
| Pure logic/algorithms | ✅ Required | ❌ Skip | ❌ Skip | ❌ Skip |
| Database commands | ✅ Required | ✅ Required | ⚠️ If REPL | ❌ Skip |
| REPL keyboard UX | ✅ Logic only | ⚠️ Optional | ⚠️ Optional | ✅ PRIMARY |
| REPL visual output | ✅ Logic only | ❌ Skip | ✅ Required | ⚠️ Recommended |
| CLI batch commands | ✅ Required | ✅ Required | ❌ Skip | ❌ Skip |
| File I/O operations | ✅ Required | ✅ Required | ❌ Skip | ❌ Skip |
| Configuration parsing | ✅ Required | ⚠️ Optional | ❌ Skip | ❌ Skip |
| Output formatting | ✅ Required | ⚠️ Verify visually | ⚠️ If REPL | ❌ Skip |

**Legend:**
- ✅ **Required**: Must implement before review
- ⚠️ **Conditional**: Check test strategy document
- ❌ **Skip**: Not needed for this feature type

---

## Common Test Implementation Gaps

### Gap 1: Strategy Says Integration, Only Unit Implemented
**Example**: Sprint 22 Iteration 1 - Feature 2 (schema commands)
- **Strategy**: "Integration tests required to validate SQL queries"
- **Implementation**: Only unit tests for glob pattern matching
- **Result**: REJECTED - missing 6 integration tests

**Prevention**: Check test strategy for each feature, verify all test types implemented.

### Gap 2: PTY Tests Missing for REPL Features
**Example**: Sprint 20 Iterations 1-2 - Tab completion pager output
- **Strategy**: "PTY tests required to validate menu display"
- **Implementation**: PTY tests present but validated wrong layer (data, not UI)
- **Result**: False positive - automated tests passed, bug persisted

**Prevention**: For REPL features, ALWAYS implement PTY tests that validate user-visible output.

### Gap 3: Manual Tests Not Documented
**Example**: Sprint 21 - Feature 3 (second TAB accepts)
- **Strategy**: "Manual validation PRIMARY (extremely high false positive risk)"
- **Implementation**: Manual procedure documented, but not executed
- **Result**: PENDING verdict - cannot approve without manual validation

**Prevention**: Document manual test procedures in `tests/cases/MANUAL-*.md` before requesting review.

### Gap 4: Deferred Features Documented
**Example**: Sprint 22 - Loading indicator
- **Strategy**: "Feature 3 deferred to future sprint"
- **Documentation**: User guide describes loading indicator feature
- **Result**: False user expectations - documented undelivered feature

**Prevention**: Review user documentation before ship, verify only delivered features documented.

---

## Verdict Criteria

Quality-validator uses these criteria to approve/reject:

### APPROVED ✅
- All test types from strategy are implemented
- All automated tests pass (100%)
- Manual tests documented (if required)
- Test counts match or exceed strategy estimates
- Zero regressions (existing tests still pass)

### REJECTED ❌
- Missing test types specified in strategy
- Automated tests fail
- Manual tests not documented (if required by strategy)
- Test coverage gaps for critical paths

### BLOCKED ⛔
- Tests cannot execute (missing database, credentials)
- Infrastructure issues (driver conflicts, environment setup)
- Upstream dependency issues (library bugs)

---

## Before Requesting Review

**Final checklist** - mark each item before submitting to quality-validator:

- [ ] ✅ Read test strategy document completely
- [ ] ✅ Verified all required test types implemented
- [ ] ✅ Ran all test types locally (unit, integration, PTY)
- [ ] ✅ All tests pass or failures documented with explanation
- [ ] ✅ Test counts match strategy estimates
- [ ] ✅ Test case documentation created/updated
- [ ] ✅ No deferred features documented in user guide
- [ ] ✅ Ready for quality-validator review

**If ANY checkbox unchecked**: Do NOT request quality-validator review. Complete missing items first.

---

## Sprint 22 Lesson Applied

**Root Cause**: Test strategy creation ≠ test implementation verification

**Solution**: This checklist forces explicit verification that strategy requirements are met.

**Expected Benefit**: Reduce iterations by catching test gaps before quality review (Sprint 22: 2 iterations → Sprint 23 goal: 1 iteration).

---

## References

- Test Strategy Template: `tests/strategy/test-strategy-template.md`
- Testing Approach: `docs/testing/approach.md`
- Testing Guidelines: `docs/testing/guidelines.md`
- Sprint 22 Review: `docs/sprints/sprint-22-review.md` (Section 6.1: Test Strategy ≠ Test Implementation)
