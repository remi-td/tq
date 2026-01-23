# TC-F3-MANUAL: Loading Indicator for Tab Completion

**Feature:** Loading Indicator for Tab Completion (Sprint 22, Feature 3, P1)
**Test Type:** Manual Validation (TIMING-BASED UI)
**Priority:** P1 (OPTIONAL FOR APPROVED VERDICT)
**Author:** quality-validator
**Created:** 2026-01-23
**Sprint:** Sprint 22

---

## Objective

Verify that loading indicator appears during slow metadata fetch operations (>500ms) and provides clear feedback to the user.

**Note:** This is a **P1 (Priority 1) feature**. Manual validation is **RECOMMENDED** but **NOT MANDATORY** for APPROVED verdict. If unit tests pass, feature can ship without manual validation.

---

## Prerequisites

- `tq` REPL compiled and runnable
- Live Teradata database connection
- Test environment with:
  - **Slow metadata query scenario:** Large database or slow network (to trigger >500ms fetch)
  - **Fast metadata query scenario:** Cached data or small database (<50ms fetch)

**Setup Tip:** To test slow metadata, use a large database with many tables, or introduce artificial network latency.

---

## Test Procedure

### Test 1: Loading Indicator Appears for Slow Metadata Fetch

**Steps:**
1. Start `tq` REPL: `tq repl`
2. Type: `SELECT * FROM large_database.` (substitute with a database that has many tables and isn't cached yet)
3. Press: `TAB` immediately (trigger metadata fetch)
4. **Observe:** Loading indicator appears within 500ms

**Expected Result:**
- Loading message appears: `"Loading tables from <database>..."`
- Spinner animation is visible (e.g., rotating braille characters: ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧)
- Indicator stays visible while metadata is being fetched
- After fetch completes: Indicator disappears, completion menu appears

**Pass Criteria:**
- [ ] Loading indicator appears (within 500ms of TAB press)
- [ ] Message format: "Loading tables from <database>..."
- [ ] Spinner animation is visible
- [ ] Indicator clears when completion menu appears
- [ ] User receives clear feedback that system is working

**Evidence:** Video recording (recommended to capture timing) or screenshot

---

### Test 2: No Indicator for Fast/Cached Metadata

**Steps:**
1. In the same REPL session (metadata now cached)
2. Type: `SELECT * FROM large_database.` (same database as Test 1)
3. Press: `TAB` (fetch from cache)
4. **Observe:** Completion menu appears instantly, NO loading indicator

**Expected Result:**
- Completion menu appears immediately (<50ms)
- NO loading indicator shown
- Cached fetch is instant and seamless

**Pass Criteria:**
- [ ] Completion menu appears instantly
- [ ] NO loading indicator shown
- [ ] User experiences fast, responsive completion

**Evidence:** Screenshot or description

---

### Test 3: Indicator Timing Threshold

**Goal:** Verify 500ms threshold is implemented correctly.

**Steps:**
1. Repeat Test 1 with different databases
2. Observe timing of indicator appearance
3. **Assess:** Does indicator appear only for queries taking >500ms?

**Expected Result:**
- Fast queries (<500ms): No indicator
- Slow queries (>500ms): Indicator appears

**Pass Criteria:**
- [ ] Indicator does NOT appear for queries <500ms
- [ ] Indicator DOES appear for queries >500ms
- [ ] Threshold logic works as specified

**Note:** Precise timing validation is difficult manually. This test provides best-effort assessment.

---

### Test 4: User Can Cancel During Loading

**Steps:**
1. Trigger slow metadata fetch (Test 1 scenario)
2. While loading indicator is visible, press: `Ctrl-C`
3. **Observe:** Fetch is cancelled, returns to prompt

**Expected Result:**
- `Ctrl-C` cancels the metadata fetch
- Loading indicator disappears
- REPL returns to prompt (no completion)
- No error or crash

**Pass Criteria:**
- [ ] `Ctrl-C` cancels the fetch
- [ ] Indicator disappears
- [ ] REPL remains functional
- [ ] User can continue working

**Evidence:** Video or written description

---

### Test 5: Error During Fetch

**Steps:**
1. Type: `SELECT * FROM restricted_database.` (database with access denied)
2. Press: `TAB`
3. **Observe:** Loading indicator → error message

**Expected Result:**
- Loading indicator appears initially
- After error occurs: Indicator replaced with error message
- Error message: "Access denied to database 'restricted_database'" (or similar)
- REPL remains functional

**Pass Criteria:**
- [ ] Loading indicator appears before error
- [ ] Error message replaces indicator
- [ ] Error message is clear and actionable
- [ ] REPL continues to work

**Evidence:** Screenshot

---

## Evidence Collection

**Recommended Evidence:**
- [ ] **Video recording** showing loading indicator during slow fetch (captures timing)
- [ ] Screenshot of loading indicator message
- [ ] Screenshot of instant completion (no indicator, cached)
- [ ] User confirmation: "Indicator provides helpful feedback"

**How to Collect:**
- **Video:** Use screen recording tool (e.g., QuickTime on macOS, OBS on Linux)
- **Screenshots:** Capture loading indicator and completion menu
- **Written Notes:** Describe timing observations

---

## Acceptance Criteria Summary

✅ **PASS** if ALL of the following are true:
- [ ] Loading indicator appears for slow metadata queries (>500ms)
- [ ] Message format: "Loading tables from <database>..."
- [ ] Spinner animation is visible
- [ ] Indicator clears when completion menu appears
- [ ] NO indicator for cached/fast queries (<50ms)
- [ ] `Ctrl-C` cancels fetch gracefully

⚠️ **CONDITIONAL PASS** if:
- Indicator appears but timing is slightly off (e.g., 400ms instead of 500ms)
- Minor UX issues but functionality works

❌ **FAIL** if:
- Loading indicator never appears
- Indicator doesn't clear after fetch completes
- Indicator breaks terminal display
- User experience is confusing

---

## Notes

**P1 Feature (Non-Blocking):**
This is a P1 (Priority 1) feature, meaning manual validation is **RECOMMENDED** but **NOT MANDATORY** for Sprint 22 APPROVED verdict. If unit tests pass, feature can ship without manual validation.

**Automation Limitation:**
Timing-based async UI is VERY difficult to automate reliably. PTY tests cannot validate indicator appearance/disappearance timing accurately. Manual validation is the ONLY reliable method.

**False Positive Risk:** VERY HIGH for automated tests.

---

## Related Tests

- **Unit Tests:** `test_loading_indicator_threshold_*` in `src/commands/repl/loading_indicator.rs`
- **Specification:** `docs/specifications/repl.md` lines 617-692

---

## Test Result

**Date Executed:** _____________
**Tester:** _____________
**Verdict:** [ ] PASS  [ ] CONDITIONAL PASS  [ ] FAIL  [ ] NOT TESTED
**Notes:**

```
[Record timing observations, any issues, or suggestions]
```

**Evidence Files:**
- Video (loading indicator): _____________
- Screenshot (indicator message): _____________
- Screenshot (cached, no indicator): _____________

**Timing Assessment:**
- Indicator appears for slow fetch: [ ] YES  [ ] NO
- Indicator absent for fast fetch: [ ] YES  [ ] NO
- Timing threshold ~500ms: [ ] YES  [ ] NO  [ ] UNCLEAR

**User Experience Assessment:**
- Indicator provides helpful feedback: [ ] YES  [ ] NO
- UX is smooth and non-disruptive: [ ] YES  [ ] NO

