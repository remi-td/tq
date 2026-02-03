# Sprint 32 Test Cases Summary

**Sprint:** 32
**Created:** 2026-02-03
**Author:** quality-validator
**Status:** Test Design Complete - Ready for Implementation

---

## Executive Summary

Test cases designed for Sprint 32 features:
- **Feature #13**: Content-Based Column Width (P0) - 8 test cases + MANDATORY manual validation
- **Feature #12**: Fix GitHub README Display (P1) - 1 verification protocol

**Total Test Artifacts:** 8 test case documents + 1 test index + 1 summary

**Critical Path:**
1. rust-teradata-architect implements tests and features
2. quality-validator executes automated tests → ADVISORY verdict
3. **sprint-coordinator executes TC-032-MANUAL → FINAL verdict (BLOCKING)**

---

## Test Case Breakdown

### Feature #13: Content-Based Column Width

#### Unit Tests (5 test cases)
**Location:** `src/format/table.rs` test module

1. **TC-032-001**: Content-Based Width Calculation (4 tests)
   - Width from content, not schema
   - Header length minimum
   - Max across rows
   - Independent column widths

2. **TC-032-002**: Maximum Width Cap (5 tests)
   - Cap at 100 chars for long content
   - Under-cap content not truncated
   - Mixed columns with/without capping

3. **TC-032-003**: NULL Value Handling (5 tests)
   - NULL as `[NULL]` (6 chars)
   - All-NULL columns
   - Mixed NULL/non-NULL

4. **TC-032-004**: Numeric Alignment (6 tests)
   - Integer widths from digits
   - Negative numbers with minus sign
   - Decimal widths with decimal point
   - Alignment logic preserved

5. **TC-032-005**: Edge Cases (7 tests)
   - Empty strings
   - Unicode display width
   - Zero-row tables
   - Whitespace-only strings

**Total Unit Tests:** ~27 tests (existing + 20 new)

---

#### Integration Tests (2 test cases)
**Location:** `tests/integration_tests.rs`

6. **TC-032-011**: End-to-End Table Formatting (3 tests)
   - Content-based widths in full pipeline
   - Mixed column types
   - Regression check (AC-5)

7. **TC-032-012**: Terminal Width Truncation (3 tests)
   - 80-char terminal with content widths
   - More columns visible vs. schema-based
   - Batch mode (no truncation)

**Total Integration Tests:** 6 new tests

---

#### Performance Benchmarks (1 benchmark)
**Location:** `benches/table_formatting.rs` (to be created)

8. **BENCH-032-001**: Width Calculation Performance (5 benchmarks)
   - Baseline: schema-based
   - Content-based: 100x20 table
   - Content-based: 1000x50 table (stress)
   - Content-based: long strings (200+ chars)
   - Comparison report

**Acceptance Threshold:** <10% regression or <1ms absolute

---

#### Manual Validation (MANDATORY - BLOCKING)
**Location:** `tests/cases/TC-032-MANUAL.md`

9. **TC-032-MANUAL**: Visual Column Density Validation
   **Type 4 Feature:** Visual/interactive - automated tests CANNOT validate
   **Priority:** **BLOCKING FOR SPRINT CLOSURE**

   **7 Test Scenarios:**
   1. **PRIMARY TEST**: 117-char terminal → MUST show 8+ columns (AC-4) - **BLOCKING**
   2. 80-char terminal → verify improvement
   3. 120-char terminal → efficient space use
   4. 160-char terminal → many columns visible
   5. Visual alignment (NULL and numeric)
   6. Truncation at 100-char cap (visual quality)
   7. Before/after comparison

   **Evidence Required:**
   - Script command output from all terminal widths
   - Store in `tests/results/sprint-32/manual-validation/`

   **Responsibility:** sprint-coordinator MUST execute this test
   **Quality Validator Role:** ADVISORY verdict only (not blocking)

---

### Feature #12: Fix GitHub README Display

#### Manual Verification (1 protocol)
**Location:** `tests/cases/TC-032-README.md`

10. **TC-032-README**: GitHub README Display Fix (5 checks)
    - Root README displays on landing page
    - `.github/` directory accessible
    - No broken links in root README
    - No broken links in `.github/` content
    - GitHub conventions followed

**Nature:** Documentation fix (file rename) - no code changes
**Estimated Time:** ~2 minutes
**When:** After push to GitHub

---

## Test Coverage Analysis

### Feature #13 Acceptance Criteria (10 ACs)

| AC | Requirement | Automated Tests | Manual Tests | Coverage |
|----|-------------|----------------|--------------|----------|
| AC-1 | Width from content, not schema | TC-032-001, 005, 011, 012 | - | ✅ Unit + Integration |
| AC-2 | Max width cap (100 chars) | TC-032-002 | TC-032-MANUAL (visual) | ✅ Unit + Manual |
| AC-3 | Min width respects header | TC-032-001, 005 | - | ✅ Unit |
| AC-4 | 8+ cols at 117-char terminal | - | **TC-032-MANUAL (PRIMARY)** | ✅ **Manual ONLY - BLOCKING** |
| AC-5 | Existing tests pass | TC-032-011 | - | ✅ Integration |
| AC-6 | Manual REPL validation | - | **TC-032-MANUAL (MANDATORY)** | ✅ **Manual ONLY - BLOCKING** |
| AC-7 | No performance regression | BENCH-032-001 | - | ✅ Benchmark |
| AC-8 | NULL values correct | TC-032-003 | TC-032-MANUAL (alignment) | ✅ Unit + Manual |
| AC-9 | Numeric alignment correct | TC-032-004 | TC-032-MANUAL (alignment) | ✅ Unit + Manual |
| AC-10 | Documentation updated | - | Manual review | ✅ Review |

**Coverage: 10/10 ACs** ✅

**CRITICAL:** AC-4 and AC-6 can ONLY be validated by TC-032-MANUAL (manual validation)

---

### Feature #12 Acceptance Criteria (4 ACs)

| AC | Requirement | Test Type | Coverage |
|----|-------------|-----------|----------|
| AC-1 | Root README displays | TC-032-README | ✅ Manual Verification |
| AC-2 | .github/ accessible | TC-032-README | ✅ Manual Verification |
| AC-3 | Conventions followed | TC-032-README | ✅ Manual Verification |
| AC-4 | No broken links | TC-032-README | ✅ Manual Verification |

**Coverage: 4/4 ACs** ✅

---

## Sprint 31 Lessons Applied

### 1. Type 4 Feature Classification

**Feature #13 is Type 4** (Visual/Interactive) per `docs/testing/approach.md`:
- User pain point is VISUAL: "only 2 columns visible" → "8+ columns visible"
- Terminal width-dependent rendering
- Alignment quality cannot be validated by automated tests
- **Manual validation is MANDATORY, not optional**

### 2. Quality Validator Role - ADVISORY Only

Per `docs/testing/philosophy.md` line 298:
> **quality-validator verdict is ADVISORY for visual features.** The sprint coordinator must manually verify before approval.

**Sprint 32 Implementation:**
- quality-validator: Designs tests, executes automated tests, generates ADVISORY report
- sprint-coordinator: Executes TC-032-MANUAL (MANDATORY), makes FINAL approval decision

### 3. Evidence Capture Required

From `sprint-32-planning.md`:
- Use `script` command to capture terminal sessions
- Test at 4 terminal widths: 80, 117, 120, 160 chars
- Store evidence in `tests/results/sprint-32/manual-validation/`
- Sprint coordinator reviews evidence before approval

### 4. Honest Assessment

**No false success claims:**
- 100% automated test pass rate is ADVISORY input, not conclusion
- AC-4 (8+ columns at 117-char) can ONLY be validated manually
- AC-6 (REPL visual improvement) can ONLY be validated manually
- Sprint CANNOT be approved without manual validation
- If visual improvement not observed → sprint is REJECTED

---

## Test Execution Workflow

### Phase 3: Build & Test (Current Phase)

**Step 1: rust-teradata-architect implements automated tests**
```bash
# Implement unit tests in src/format/table.rs
# Implement integration tests in tests/integration_tests.rs
# Create benchmark in benches/table_formatting.rs
```

**Step 2: rust-teradata-architect implements features**
```bash
# Feature #13: Content-based column width calculation
# Feature #12: Rename .github/README.md to .github/GITHUB_CONFIG.md
```

**Step 3: quality-validator executes automated tests**
```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench table_formatting

# Generate test report with ADVISORY verdict
# Store in tests/results/sprint-32/REPORT.md
```

**Step 4: sprint-coordinator executes TC-032-MANUAL (MANDATORY)**
```bash
# Manual validation at 4 terminal widths
# PRIMARY TEST: 117-char terminal MUST show 8+ columns
# Capture evidence with script command
# Store in tests/results/sprint-32/manual-validation/
```

**Step 5: tq-project-manager validates and ships**
```bash
# Review quality-validator ADVISORY verdict
# Review sprint-coordinator manual validation results
# Make go/no-go decision
# Commit and push to GitHub
```

**Step 6: sprint-coordinator executes TC-032-README**
```bash
# Verify GitHub README display (after push)
# ~2 minutes verification
```

---

## Critical Success Factors

**For Sprint Approval (ALL required):**

✅ **Automated Tests:**
- All unit tests pass (27 tests)
- All integration tests pass (6 tests)
- Benchmarks show <10% regression (AC-7)

✅ **Manual Validation (BLOCKING):**
- **TC-032-MANUAL PRIMARY TEST: 8+ columns visible at 117-char terminal (AC-4)**
- **TC-032-MANUAL: Visual improvement confirmed in REPL (AC-6)**
- Evidence captured and stored

✅ **Feature #12:**
- TC-032-README: GitHub README displays correctly

**If TC-032-MANUAL fails (fewer than 8 columns at 117-char), sprint MUST be REJECTED.**

---

## Risk Assessment

### High-Risk Areas

**Risk 1: Manual Validation Shows <8 Columns at 117-Char Terminal**
- **Impact:** CRITICAL - AC-4 failure, sprint must be rejected
- **Probability:** Low (if implementation correct)
- **Mitigation:** Thorough unit/integration tests catch logic errors early
- **Contingency:** Fix implementation, re-test, iterate until AC-4 met

**Risk 2: Visual Quality Issues (Alignment, Truncation)**
- **Impact:** HIGH - Poor UX, sprint should be rejected
- **Probability:** Medium (visual features are complex)
- **Mitigation:** Comprehensive edge case testing, manual validation at multiple widths
- **Contingency:** Fix visual issues, re-test manual validation

### Medium-Risk Areas

**Risk 3: Performance Regression (>10%)**
- **Impact:** MEDIUM - AC-7 failure
- **Probability:** Low (width calc is simple string operations)
- **Mitigation:** Benchmark early, optimize if needed
- **Contingency:** Implement caching or early-exit optimizations

**Risk 4: Existing Tests Break (AC-5)**
- **Impact:** HIGH - Regression in table formatting
- **Probability:** Low (integration tests validate)
- **Mitigation:** Run full test suite continuously
- **Contingency:** Fix regressions before manual validation

---

## Test Artifacts Location

**Test Case Documents:**
```
/Users/remi.turpaud/Code/genAI/tq/tests/cases/
├── TC-032-001.md  (Unit: Content-based width calculation)
├── TC-032-002.md  (Unit: Max width cap)
├── TC-032-003.md  (Unit: NULL handling)
├── TC-032-004.md  (Unit: Numeric alignment)
├── TC-032-005.md  (Unit: Edge cases)
├── TC-032-011.md  (Integration: End-to-end formatting)
├── TC-032-012.md  (Integration: Terminal width truncation)
├── TC-032-MANUAL.md  (Manual: REPL validation - MANDATORY)
├── TC-032-README.md  (Manual: GitHub README verification)
├── INDEX-SPRINT-32.md  (Test case index)
└── SPRINT-32-TEST-SUMMARY.md  (This document)
```

**Test Strategy:**
```
/Users/remi.turpaud/Code/genAI/tq/tests/strategy/
└── sprint-32-test-strategy.md  (Comprehensive strategy document)
```

**Test Results (to be created during execution):**
```
/Users/remi.turpaud/Code/genAI/tq/tests/results/sprint-32/
├── REPORT.md  (quality-validator test execution report)
└── manual-validation/
    ├── test-117.txt  (PRIMARY TEST evidence)
    ├── test-80.txt
    ├── test-120.txt
    ├── test-160.txt
    └── VALIDATION-SUMMARY.md  (sprint-coordinator summary)
```

---

## Next Actions for Sprint Coordinator

**Immediate:**
1. Review this test summary
2. Review test strategy: `tests/strategy/sprint-32-test-strategy.md`
3. Review test cases: `tests/cases/TC-032-*.md`
4. Proceed to Phase 3: Build & Test (parallel execution of rust-teradata-architect)

**After Implementation:**
1. Review quality-validator test report (ADVISORY)
2. **Execute TC-032-MANUAL** (MANDATORY - BLOCKING)
   - Test at 4 terminal widths
   - Capture evidence
   - PRIMARY TEST: Verify 8+ columns at 117-char
3. Make final approval decision based on manual validation
4. After push: Execute TC-032-README verification

**Decision Gates:**
- If automated tests fail → Fix, iterate
- If manual validation fails → Fix, iterate (DO NOT approve)
- If manual validation passes → Proceed to Phase 4: Ship

---

## Questions for Sprint Coordinator

**Before Proceeding:**
1. Is the Type 4 classification clear for Feature #13?
2. Is the quality-validator ADVISORY role vs. sprint-coordinator FINAL verdict understood?
3. Are the manual validation requirements (TC-032-MANUAL) clear?
4. Is the evidence capture protocol (script command) understood?
5. Is the BLOCKING nature of AC-4 (8+ columns at 117-char) understood?

**Any concerns or questions about the test approach?**

---

## Document Status

**Status:** ✅ COMPLETE - Ready for Implementation

**Deliverables:**
- ✅ Test strategy created (`tests/strategy/sprint-32-test-strategy.md`)
- ✅ 5 unit test cases designed (TC-032-001 through TC-032-005)
- ✅ 2 integration test cases designed (TC-032-011, TC-032-012)
- ✅ 1 benchmark specified (BENCH-032-001)
- ✅ 1 MANDATORY manual validation protocol (TC-032-MANUAL)
- ✅ 1 manual verification protocol (TC-032-README)
- ✅ Test case index created (`INDEX-SPRINT-32.md`)
- ✅ Test summary created (this document)

**Total Work Products:** 11 documents

**Next Step:** Proceed to Phase 3 Build (rust-teradata-architect implementation)

---

## Appendix: Quick Reference

### Test Case Counts
- Unit tests: 20 new (in 5 test case docs)
- Integration tests: 6 new (in 2 test case docs)
- Benchmarks: 5 new (in 1 benchmark)
- Manual validation: 7 scenarios (in 1 MANDATORY protocol)
- Manual verification: 5 checks (in 1 protocol)

### Coverage Summary
- Feature #13: 10/10 ACs covered (2 BLOCKING manual tests)
- Feature #12: 4/4 ACs covered (simple verification)

### Critical Path
1. Implement tests → 2. Implement features → 3. Execute automated tests (ADVISORY) → 4. **Execute manual tests (BLOCKING)** → 5. Approve or reject

### BLOCKING Tests
- **TC-032-MANUAL PRIMARY TEST**: 8+ columns at 117-char terminal (AC-4)
- **TC-032-MANUAL**: REPL visual improvement (AC-6)
- Both executed by sprint-coordinator, FINAL verdict

---

**Document History**

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-03 | 1.0 | Initial test summary for Sprint 32 | quality-validator |
