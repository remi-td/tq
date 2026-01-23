# TC-F4-MANUAL: Integration Test Infrastructure Fix

**Feature:** Integration Test Infrastructure Fix (Sprint 22, Feature 4, P1)
**Test Type:** Manual Validation (CI/CD COMPATIBILITY)
**Priority:** P1 (OPTIONAL FOR APPROVED VERDICT)
**Author:** quality-validator
**Created:** 2026-01-23
**Sprint:** Sprint 22

---

## Objective

Verify that integration tests run without connection conflicts and are compatible with CI/CD environments.

**Note:** This is a **P1 (Priority 1) feature**. Manual validation is **RECOMMENDED** but **NOT MANDATORY** for Sprint 22 APPROVED verdict. If all integration tests pass (100% pass rate), feature can ship without manual validation.

---

## Prerequisites

- `tq` project cloned locally
- Rust toolchain installed (`cargo` available)
- Live Teradata database connection configured in `.env` file:
  ```
  TQ_LOGON=username:password@host:1025/database
  ```

---

## Test Procedure

### Test 1: Run All Integration Tests

**Steps:**
1. Open terminal
2. Navigate to `tq` project root
3. Ensure `.env` file is configured with valid `TQ_LOGON`
4. Run: `cargo test --test integration_tests -- --ignored`
5. **Observe:** Test execution output

**Expected Result:**
- All integration tests execute successfully
- **100% pass rate** (no failures)
- **No "Driver only supports one connection at a time" errors**
- Tests complete within reasonable time (< 5 minutes)

**Pass Criteria:**
- [ ] All integration tests run without crashing
- [ ] 100% pass rate (no test failures)
- [ ] No connection conflict errors
- [ ] Test output is clear and readable
- [ ] Total execution time is acceptable

**Evidence:** Copy test output to evidence section below

---

### Test 2: Verify Test Isolation

**Goal:** Confirm each test gets clean connection state (no state leakage between tests).

**Steps:**
1. Review test output from Test 1
2. Look for:
   - Tests executing in sequence
   - No "connection already in use" errors
   - No unexpected state-related failures
3. **Assess:** Do tests appear isolated?

**Expected Result:**
- Each test runs independently
- No state leakage between tests
- Tests can run in any order without conflicts

**Pass Criteria:**
- [ ] No evidence of state leakage
- [ ] Tests execute independently
- [ ] Order of test execution doesn't matter

---

### Test 3: Run Specific Test Multiple Times

**Goal:** Verify test reliability and repeatability.

**Steps:**
1. Pick one integration test (e.g., `test_list_databases`)
2. Run 3 times in sequence:
   ```bash
   cargo test --test integration_tests test_list_databases -- --ignored
   cargo test --test integration_tests test_list_databases -- --ignored
   cargo test --test integration_tests test_list_databases -- --ignored
   ```
3. **Observe:** All runs pass consistently

**Expected Result:**
- Test passes all 3 times
- No flaky behavior (pass/fail randomly)
- Consistent results across runs

**Pass Criteria:**
- [ ] Test passes 3/3 times
- [ ] Results are consistent
- [ ] No flaky failures

---

### Test 4: Check CI/CD Compatibility (Optional)

**Goal:** Verify tests can run in automated CI/CD environment.

**Steps (if CI/CD is set up):**
1. Push code to GitHub (or CI platform)
2. Trigger CI/CD pipeline
3. Check test execution in CI logs
4. **Observe:** Tests run successfully in CI

**Expected Result:**
- Tests execute in CI environment
- 100% pass rate in CI
- No environment-specific failures

**Pass Criteria:**
- [ ] Tests run in CI without manual intervention
- [ ] All tests pass in CI
- [ ] No environment-specific issues

**Note:** If CI/CD is not set up, this test can be skipped.

---

### Test 5: Error Message Clarity (On Failure)

**Goal:** Verify error messages are helpful when tests fail.

**Steps:**
1. Intentionally break a test (e.g., disconnect database during test)
2. Run test: `cargo test --test integration_tests -- --ignored`
3. **Observe:** Error message quality

**Expected Result:**
- Error messages are clear and actionable
- Error indicates what went wrong (e.g., "Connection lost")
- No cryptic error codes or stack traces (unless necessary)

**Pass Criteria:**
- [ ] Error messages are clear
- [ ] User can understand what failed
- [ ] Error messages help with debugging

**Note:** This test is optional. Skip if all tests pass in Test 1.

---

## Evidence Collection

**Required Evidence:**
- [ ] Test output from Test 1 (all integration tests)
- [ ] Test count: Total tests, Passed, Failed
- [ ] Confirmation: "All integration tests passed (100% pass rate)"

**Optional Evidence:**
- [ ] CI/CD logs (if available)
- [ ] Test output from multiple runs (Test 3)

---

### Evidence: Test Execution Output

**Test Command:** `cargo test --test integration_tests -- --ignored`

**Test Output:**
```
[Paste test execution output here]

Expected format:
running X tests
test test_name_1 ... ok
test test_name_2 ... ok
...
test result: ok. X passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Test Count:**
- Total tests: _____
- Passed: _____
- Failed: _____
- Pass rate: _____%

---

## Acceptance Criteria Summary

✅ **PASS** if ALL of the following are true:
- [ ] All integration tests pass (100% pass rate)
- [ ] No "Driver only supports one connection at a time" errors
- [ ] Test isolation works (no state leakage)
- [ ] Tests are repeatable (consistent results)
- [ ] Error messages (if any) are clear

⚠️ **CONDITIONAL PASS** if:
- 90-99% pass rate (1-2 tests fail but not due to infrastructure)
- Tests pass but have minor timing issues

❌ **FAIL** if:
- Multiple tests fail due to connection conflicts
- "Driver only supports one connection" error persists
- Tests are flaky (pass/fail randomly)
- Test infrastructure is broken

---

## Notes

**P1 Feature (Non-Blocking):**
This is a P1 feature. If all integration tests pass (Test 1), manual validation is **NOT MANDATORY** for APPROVED verdict.

**Primary Validation Method:**
Running the tests IS the primary validation. This manual test mostly confirms:
1. Tests pass
2. Test output is readable
3. No infrastructure issues

**Sprint 21 Context:**
Sprint 21 had 50% integration test pass rate due to driver connection conflicts. Sprint 22 fixes this issue. Target: 100% pass rate.

---

## Related Tests

- **Integration Tests:** All tests in `tests/integration_tests.rs`
- **Specification:** `docs/sprints/sprint-22-planning.md` lines 99-106

---

## Test Result

**Date Executed:** _____________
**Tester:** _____________
**Verdict:** [ ] PASS  [ ] CONDITIONAL PASS  [ ] FAIL  [ ] NOT TESTED
**Notes:**

```
[Record any issues, test failures, or observations]
```

**Test Statistics:**
- Total integration tests: _____
- Tests passed: _____
- Tests failed: _____
- Pass rate: _____%

**Infrastructure Assessment:**
- Connection conflicts resolved: [ ] YES  [ ] NO
- Test isolation working: [ ] YES  [ ] NO
- CI/CD compatible: [ ] YES  [ ] NO  [ ] NOT TESTED

