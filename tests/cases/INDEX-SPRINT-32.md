# Test Case Index - Sprint 32

**Sprint:** 32
**Features:** Content-Based Column Width (P0) + Fix GitHub README Display (P1)
**Created:** 2026-02-03
**Author:** quality-validator

---

## Summary

**Total Test Cases:** 8 automated test cases + 2 manual validation protocols

**Test Distribution:**
- Unit Tests: 5 (TC-032-001 through TC-032-005)
- Integration Tests: 2 (TC-032-011, TC-032-012)
- Benchmark Tests: 1 (BENCH-032-001 - to be created by rust-teradata-architect)
- Manual Validation: 2 (TC-032-MANUAL - MANDATORY, TC-032-README)

**Acceptance Criteria Coverage:** 10/10 ACs for Feature #13, 4/4 ACs for Feature #12

---

## Feature 1: Content-Based Column Width (#13) [P0]

### Unit Tests (5 tests)

#### TC-032-001: Unit Test - Content-Based Column Width Calculation
- **Priority:** Critical
- **File:** `tests/cases/TC-032-001.md`
- **Coverage:** AC-1 (column width from content), AC-3 (header length minimum)
- **Purpose:** Verify column widths calculated from actual content, not schema type
- **Scenarios:** 4 tests
  - Width from content, not schema VARCHAR(N)
  - Header length respected as minimum
  - Max content length across rows
  - Multiple columns independent widths

#### TC-032-002: Unit Test - Maximum Column Width Cap
- **Priority:** Critical
- **File:** `tests/cases/TC-032-002.md`
- **Coverage:** AC-2 (maximum width cap at 100 chars)
- **Purpose:** Verify column widths capped at MAX_COLUMN_WIDTH (100 chars)
- **Scenarios:** 5 tests
  - Very long content (150 chars) capped at 100
  - Extremely long content (1000 chars) capped at 100
  - Content under cap (99 chars) not truncated
  - Content exactly at cap (100 chars)
  - Mixed columns: some capped, some not

#### TC-032-003: Unit Test - NULL Value Width Handling
- **Priority:** High
- **File:** `tests/cases/TC-032-003.md`
- **Coverage:** AC-8 (NULL value handling)
- **Purpose:** Verify NULL values represented as `[NULL]` (6 chars) in width calculations
- **Scenarios:** 5 tests
  - NULL value width calculation (mixed with content)
  - All-NULL column width
  - NULL longer than header
  - NULL shorter than content
  - Multiple columns with NULLs

#### TC-032-004: Unit Test - Numeric Column Right-Alignment Logic
- **Priority:** High
- **File:** `tests/cases/TC-032-004.md`
- **Coverage:** AC-9 (numeric alignment)
- **Purpose:** Verify numeric columns remain right-aligned with content-based widths
- **Scenarios:** 6 tests
  - Integer column width from digits
  - Negative integers include minus sign
  - Decimal width includes decimal point
  - Numeric alignment logic preserved
  - Large integer width (10+ digits)
  - Mixed numeric and text columns

#### TC-032-005: Unit Test - Empty String and Edge Cases
- **Priority:** Medium
- **File:** `tests/cases/TC-032-005.md`
- **Coverage:** AC-1 (edge cases), AC-3 (header minimum)
- **Purpose:** Verify edge cases handled correctly (empty strings, Unicode, etc.)
- **Scenarios:** 7 tests
  - Empty string width (uses other content or header)
  - All empty strings use header length
  - Single-char strings
  - Unicode display width (not byte count)
  - Zero-row tables use header only
  - Whitespace-only strings
  - Mixed edge cases

---

### Integration Tests (2 tests)

#### TC-032-011: Integration Test - End-to-End Table Formatting
- **Priority:** Critical
- **File:** `tests/cases/TC-032-011.md`
- **Coverage:** AC-1 (content-based width), AC-5 (existing tests pass)
- **Purpose:** Verify content-based width integrates correctly with table formatter
- **Scenarios:** 3 tests
  - End-to-end formatting with content-based widths (not schema widths)
  - Mixed column types formatted correctly
  - Existing table formatter regression check (AC-5)

#### TC-032-012: Integration Test - Terminal Width Truncation
- **Priority:** High
- **File:** `tests/cases/TC-032-012.md`
- **Coverage:** AC-1 (content width), AC-4 (more columns visible - implicit)
- **Purpose:** Verify content-based width with terminal width truncation
- **Scenarios:** 3 tests
  - Content-based widths with 80-char terminal limit
  - More columns visible than schema-based approach
  - Batch mode shows all columns (no truncation)

---

### Performance Benchmarks (1 benchmark)

#### BENCH-032-001: Column Width Calculation Performance
- **Priority:** Critical
- **File:** `benches/table_formatting.rs` (to be created)
- **Coverage:** AC-7 (no performance regression)
- **Purpose:** Measure column width calculation performance (content-based vs. schema-based)
- **Scenarios:** 5 benchmarks
  - Baseline: Schema-based width calculation
  - Content-based: 100x20 table (typical)
  - Content-based: 1000x50 table (stress test)
  - Content-based: Very long content (200+ char strings)
  - Comparison report: percentage change from baseline
- **Acceptance Threshold:** <10% regression or <1ms absolute for typical tables
- **Implementation:** rust-teradata-architect will create using criterion crate

---

### Manual Validation (1 protocol - MANDATORY)

#### TC-032-MANUAL: Manual Validation - Content-Based Column Width
- **Priority:** **BLOCKING - MANDATORY**
- **File:** `tests/cases/TC-032-MANUAL.md`
- **Coverage:** AC-4 (8+ columns at 117-char), AC-6 (manual validation), AC-8 (NULL alignment), AC-9 (numeric alignment), AC-2 (truncation visual)
- **Purpose:** **MANDATORY manual validation** of visual column density improvement
- **Type 4 Feature:** Visual/interactive - automated tests CANNOT validate
- **Scenarios:** 7 tests
  1. **PRIMARY TEST**: 117-char terminal - MUST show 8+ columns (AC-4) - **BLOCKING**
  2. Narrow terminal (80 chars) - verify improvement
  3. Standard wide (120 chars) - verify efficient space use
  4. Very wide (160 chars) - verify many columns visible
  5. Visual alignment check (NULL and numeric) - AC-8, AC-9
  6. Truncation check (long content at 100-char cap) - AC-2 visual
  7. Before/after comparison (if available)
- **Evidence Required:**
  - Script command output from all terminal widths
  - Store in `tests/results/sprint-32/manual-validation/`
- **Sprint Coordinator Responsibility:** This test MUST be performed by sprint coordinator
- **Quality Validator Role:** ADVISORY verdict only, not blocking
- **Sprint 31 Lesson Applied:** Type 4 features require MANDATORY manual validation

---

## Feature 2: Fix GitHub README Display (#12) [P1]

### Manual Verification (1 protocol)

#### TC-032-README: Manual Verification - GitHub README Display Fix
- **Priority:** Low
- **File:** `tests/cases/TC-032-README.md`
- **Coverage:** AC-1 (root README displays), AC-2 (.github/ accessible), AC-3 (conventions), AC-4 (no broken links)
- **Purpose:** Verify root README displays on GitHub after renaming `.github/README.md`
- **Nature:** Documentation fix (file rename) - no code changes
- **Scenarios:** 5 checks
  1. Root README displays on repository landing page
  2. `.github/` directory still accessible with renamed file
  3. No broken links in root README
  4. No broken links in `.github/` content
  5. Solution follows GitHub conventions
- **Estimated Time:** ~2 minutes
- **Complexity:** Very Low

---

## Test Execution Strategy

### Phase 1: Unit Tests (rust-teradata-architect)
Run all unit tests in `src/format/table.rs`:
```bash
cargo test --lib table
```
Expected: 25-27 unit tests pass (existing + 20 new tests from TC-032-001 through TC-032-005)

### Phase 2: Integration Tests (rust-teradata-architect)
Run integration tests:
```bash
cargo test --test integration_tests
```
Expected: All existing tests + 6 new tests pass (TC-032-011, TC-032-012)

### Phase 3: Performance Benchmarks (rust-teradata-architect)
Run benchmarks:
```bash
cargo bench table_formatting
```
Expected: Content-based width calculation shows <10% regression vs. schema-based

### Phase 4: Manual Validation (sprint-coordinator)
Execute manual validation protocols:
1. **TC-032-MANUAL** (MANDATORY - BLOCKING)
   - Test at 4 terminal widths (80, 117, 120, 160)
   - PRIMARY TEST: 117-char terminal MUST show 8+ columns
   - Evidence capture: script command output
   - Store in `tests/results/sprint-32/manual-validation/`

2. **TC-032-README** (REQUIRED)
   - Verify on GitHub after push
   - Check root README display, links, `.github/` accessibility
   - ~2 minutes

---

## Acceptance Criteria Coverage Map

### Feature 1: Content-Based Column Width (#13)

| AC | Requirement | Test Cases | Test Type | Status |
|----|-------------|------------|-----------|--------|
| AC-1 | Column width from content, not schema | TC-032-001, TC-032-005, TC-032-011, TC-032-012 | Unit + Integration | Designed |
| AC-2 | Max width cap (100 chars) | TC-032-002, TC-032-MANUAL (visual) | Unit + Manual | Designed |
| AC-3 | Min width respects header | TC-032-001, TC-032-005 | Unit | Designed |
| AC-4 | 8+ columns at 117-char terminal | **TC-032-MANUAL (PRIMARY TEST)** | **Manual (BLOCKING)** | Designed |
| AC-5 | Existing tests still pass | TC-032-011 | Integration | Designed |
| AC-6 | Manual validation in REPL | **TC-032-MANUAL (MANDATORY)** | **Manual (BLOCKING)** | Designed |
| AC-7 | No performance regression | BENCH-032-001 | Benchmark | To be created |
| AC-8 | NULL values correct | TC-032-003, TC-032-MANUAL (visual) | Unit + Manual | Designed |
| AC-9 | Numeric alignment correct | TC-032-004, TC-032-MANUAL (visual) | Unit + Manual | Designed |
| AC-10 | Documentation updated | Manual review | Documentation | To be verified |

**Coverage:** 10/10 ACs have test coverage

**BLOCKING Requirements:**
- **AC-4**: Manual validation at 117-char terminal - MUST show 8+ columns
- **AC-6**: Manual validation in REPL - MUST confirm visual improvement
- **Both validated by TC-032-MANUAL** (MANDATORY test)

---

### Feature 2: Fix GitHub README Display (#12)

| AC | Requirement | Test Cases | Test Type | Status |
|----|-------------|------------|-----------|--------|
| AC-1 | Root README displays on GitHub | TC-032-README | Manual Verification | Designed |
| AC-2 | .github/ content accessible | TC-032-README | Manual Verification | Designed |
| AC-3 | GitHub conventions followed | TC-032-README | Manual Verification | Designed |
| AC-4 | No broken links | TC-032-README | Manual Verification | Designed |

**Coverage:** 4/4 ACs have test coverage

---

## Sprint 31 Lessons Applied

### Type 4 Feature Classification

**Feature #13 correctly classified as Type 4** (Visual/Interactive) per `docs/testing/approach.md`:
- User pain point is VISUAL: "only 2 columns visible" → "8+ columns visible"
- Terminal width-dependent behavior (80, 117, 120, 160 chars)
- Alignment quality cannot be validated by automated tests
- Manual validation is MANDATORY, not optional

### Quality Validator Role

Per `docs/testing/philosophy.md` line 298:
> **quality-validator verdict is ADVISORY for visual features.** The sprint coordinator must manually verify before approval.

**Sprint 32 Implementation:**
- quality-validator: Designs tests, implements automated tests, executes unit/integration tests
- quality-validator: Generates test report with ADVISORY verdict
- sprint-coordinator: Performs TC-032-MANUAL validation (MANDATORY)
- sprint-coordinator: Makes final approval decision based on manual validation

### Evidence Requirements

**From sprint-32-planning.md lines 216-219:**
> - Evidence required: script command output capture
> - Sprint blocked if manual validation reveals issues

**Sprint 32 Evidence Protocol:**
- Use `script` command to capture terminal sessions
- Test at 4 terminal widths: 80, 117, 120, 160 characters
- Store all evidence in `tests/results/sprint-32/manual-validation/`
- Sprint coordinator reviews evidence before approval

### Honest Assessment

**No false success claims:**
- 100% automated test pass rate is ADVISORY input, not conclusion
- AC-4 and AC-6 can ONLY be validated by manual testing
- Sprint CANNOT be approved without manual validation
- If visual improvement not observed, sprint is REJECTED (not approved based on metrics)

---

## Test Case Files

All test case files located in: `tests/cases/`

**Unit Tests:**
- `TC-032-001.md` - Content-based width calculation
- `TC-032-002.md` - Maximum width cap
- `TC-032-003.md` - NULL value handling
- `TC-032-004.md` - Numeric alignment
- `TC-032-005.md` - Edge cases (empty strings, Unicode)

**Integration Tests:**
- `TC-032-011.md` - End-to-end table formatting
- `TC-032-012.md` - Terminal width truncation

**Manual Validation:**
- `TC-032-MANUAL.md` - Content-based width REPL validation (MANDATORY - BLOCKING)
- `TC-032-README.md` - GitHub README display verification

**Benchmarks:**
- `BENCH-032-001` - To be created in `benches/table_formatting.rs` by rust-teradata-architect

---

## Next Steps

1. **rust-teradata-architect**: Implement unit tests (TC-032-001 through TC-032-005) in `src/format/table.rs`
2. **rust-teradata-architect**: Implement integration tests (TC-032-011, TC-032-012) in `tests/integration_tests.rs`
3. **rust-teradata-architect**: Create benchmark (BENCH-032-001) in `benches/table_formatting.rs` using criterion
4. **rust-teradata-architect**: Implement Feature #13 (content-based width calculation)
5. **rust-teradata-architect**: Implement Feature #12 (rename `.github/README.md`)
6. **quality-validator**: Execute all automated tests (unit + integration + benchmarks)
7. **quality-validator**: Generate test report with ADVISORY verdict
8. **sprint-coordinator**: Execute TC-032-MANUAL (MANDATORY - BLOCKING)
9. **sprint-coordinator**: Execute TC-032-README (after push)
10. **sprint-coordinator**: Make final approval decision based on manual validation

---

## Critical Success Factors

**For Sprint Approval:**
1. ✅ All unit tests pass (25-27 tests)
2. ✅ All integration tests pass (6 new tests)
3. ✅ Performance benchmarks show <10% regression (AC-7)
4. ✅ **TC-032-MANUAL PRIMARY TEST: 8+ columns visible at 117-char terminal (AC-4) - BLOCKING**
5. ✅ **TC-032-MANUAL: Visual improvement confirmed in REPL (AC-6) - BLOCKING**
6. ✅ TC-032-README: GitHub README displays correctly (Feature #12)

**If any BLOCKING test fails, sprint CANNOT be approved.**

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-03 | 1.0 | Initial test case index for Sprint 32 | quality-validator |
