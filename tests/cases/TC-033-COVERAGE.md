# Sprint 33 Test Coverage Matrix

## Sprint Overview

**Sprint:** 33 - Pager Bug Fix + Data Sampling Commands
**Date:** 2026-02-03
**Test Strategy:** `tests/strategy/sprint-33-test-strategy.md`

## Features Under Test

### Feature 1: Pager Bug Fix (Issue #14)
- **Scope:** Fix pager rendering, disable by default, validate with existing tests
- **Test Cases:** 1 new unit test + verification of 75 existing tests
- **Status:** Test implementation complete

### Feature 2: Data Sampling Commands
- **Scope:** Implement `/sample` and `/peek` metacommands for REPL and batch mode
- **Test Cases:** 8 new test case documents covering unit, integration, interactive, and batch modes
- **Status:** Test implementation complete

## Test Case Summary

| Test ID | Type | Feature | Priority | Description |
|---------|------|---------|----------|-------------|
| TC-033-001 | Unit | Pager (AC-3) | Critical | Pager disabled by default |
| TC-033-002 | Unit | Sample (AC-1-4) | Critical | /sample command parsing and SQL generation |
| TC-033-003 | Unit | Peek (AC-5-6) | Critical | /peek command parsing and SQL generation |
| TC-033-004 | Integration | Sample (AC-4,8,9,13) | Critical | /sample execution against live database |
| TC-033-005 | Integration | Peek (AC-5,6,8,9) | Critical | /peek execution with metadata |
| TC-033-006 | Interactive | Sample (AC-7,10) | Critical | /sample in REPL with tab completion |
| TC-033-007 | Interactive | Peek (AC-7,10) | Critical | /peek in REPL with tab completion |
| TC-033-008 | Batch | Sample (AC-11) | Critical | tq sample CLI command |
| TC-033-009 | Batch | Peek (AC-11) | Critical | tq peek CLI command |
| TC-033-PAGER-MANUAL | Manual | Pager (AC-6) | Critical | Manual visual validation (documented, not executable) |

**Total Test Cases:** 10 (9 automated + 1 manual documented)

## Acceptance Criteria Coverage

### Feature 1: Pager Bug Fix (10 ACs)

| AC | Requirement | Test Coverage | Status |
|----|-------------|---------------|--------|
| AC-1 | Root cause identified | Code review / Analysis | N/A - Analysis |
| AC-2 | Fix implemented | Existing 27 pager unit tests | Verify Pass |
| AC-3 | Default disabled | TC-033-001 | New Test |
| AC-4 | Unit tests pass | Existing 27 pager tests | Verify Pass |
| AC-5 | Integration tests pass | Existing 48 interactive tests | Verify Pass |
| AC-6 | Manual test case documented | TC-033-PAGER-MANUAL | Documented |
| AC-7 | User can enable | Existing test_horizontal_paging_* | Verify Pass |
| AC-8 | Documentation updated | Manual review | N/A |
| AC-9 | GitHub issue updated | Manual task | N/A |
| AC-10 | Zero regressions | All existing tests (355 unit + 48 interactive) | Verify Pass |

**Coverage:** 10/10 ACs covered (8 automated, 1 documented manual, 1 analysis)

### Feature 2: Data Sampling Commands (15 ACs)

| AC | Requirement | Test Coverage | Status |
|----|-------------|---------------|--------|
| AC-1 | /sample implemented | TC-033-002, TC-033-004, TC-033-006, TC-033-008 | Full Coverage |
| AC-2 | Default 10 rows | TC-033-002 (unit), TC-033-004 (integration) | Full Coverage |
| AC-3 | Max 1000 validation | TC-033-002 (unit), TC-033-008 (batch) | Full Coverage |
| AC-4 | SAMPLE clause | TC-033-002 (unit), TC-033-004 (integration) | Full Coverage |
| AC-5 | /peek implemented | TC-033-003, TC-033-005, TC-033-007, TC-033-009 | Full Coverage |
| AC-6 | Column metadata | TC-033-003 (unit), TC-033-005 (integration), TC-033-007 (interactive) | Full Coverage |
| AC-7 | Tab completion | TC-033-006 (sample), TC-033-007 (peek) | Full Coverage |
| AC-8 | Error handling | TC-033-002/003 (unit), TC-033-004/005 (integration), TC-033-006/007 (interactive) | Full Coverage |
| AC-9 | Multi-format | TC-033-004/005 (integration), TC-033-008/009 (batch) | Full Coverage |
| AC-10 | Help text | TC-033-006 (sample), TC-033-007 (peek) | Full Coverage |
| AC-11 | Batch mode | TC-033-008 (sample), TC-033-009 (peek) | Full Coverage |
| AC-12 | Qualified names | TC-033-002/003 (unit), TC-033-004/005 (integration), TC-033-008/009 (batch) | Full Coverage |
| AC-13 | Performance | TC-033-004 (integration observation) | Observational |
| AC-14 | Documentation | Manual review | N/A |
| AC-15 | 100% test coverage | All TC-033-* tests | Full Coverage |

**Coverage:** 15/15 ACs covered (13 automated, 1 observational, 1 manual review)

## Test Type Distribution

### Unit Tests
- **Count:** 3 test case documents (TC-033-001, TC-033-002, TC-033-003)
- **Estimated Test Functions:** 15-20 individual test functions
- **Coverage:** Command parsing, SQL generation, parameter validation, default values
- **Database Required:** No
- **Run Command:** `cargo test --lib`

### Integration Tests (#[ignore])
- **Count:** 2 test case documents (TC-033-004, TC-033-005)
- **Estimated Test Functions:** 18 individual test functions
- **Coverage:** Query execution, error handling, output formats, performance
- **Database Required:** Yes
- **Run Command:** `cargo test --test integration_tests -- --ignored`

### Interactive Tests (#[ignore])
- **Count:** 2 test case documents (TC-033-006, TC-033-007)
- **Estimated Test Functions:** 13 individual test functions
- **Coverage:** REPL integration, tab completion, help text, PTY behavior
- **Database Required:** Yes
- **PTY Required:** Yes
- **Run Command:** `cargo test --test interactive_tests -- --ignored`

### Batch Mode Tests (#[ignore])
- **Count:** 2 test case documents (TC-033-008, TC-033-009)
- **Estimated Test Functions:** 15 individual test functions
- **Coverage:** CLI argument parsing, subprocess execution, exit codes
- **Database Required:** Yes
- **Run Command:** `cargo test --test integration_tests test_batch -- --ignored`

### Manual Tests
- **Count:** 1 test case document (TC-033-PAGER-MANUAL)
- **Purpose:** Visual validation of pager rendering at specific terminal widths
- **Status:** Documented but not executable (no human tester available)
- **Acceptance:** Acknowledged gap, pager disabled by default for safety

**Total Estimated Test Functions:** 61-66 automated tests + 1 manual test case

## Test Execution Strategy

### Phase 1: Unit Tests (No Database Required)
```bash
cargo test --lib test_pager_disabled_by_default
cargo test --lib test_sample_
cargo test --lib test_peek_
```
**Expected:** ~18 unit tests pass

### Phase 2: Existing Test Verification (Database Required)
```bash
cargo test --lib  # All 355 unit tests
cargo test --test interactive_tests -- --ignored  # All 48 interactive tests
```
**Expected:** 100% pass rate (403 tests)

### Phase 3: Integration Tests (Database Required)
```bash
cargo test --test integration_tests test_sample_command -- --ignored
cargo test --test integration_tests test_peek_command -- --ignored
```
**Expected:** ~18 integration tests pass

### Phase 4: Interactive Tests (Database + PTY Required)
```bash
cargo test --test interactive_tests test_sample_ -- --ignored
cargo test --test interactive_tests test_peek_ -- --ignored
```
**Expected:** ~13 interactive tests pass

### Phase 5: Batch Mode Tests (Database Required)
```bash
cargo test --test integration_tests test_batch_sample -- --ignored
cargo test --test integration_tests test_batch_peek -- --ignored
```
**Expected:** ~15 batch mode tests pass

### Total Test Execution
- **Unit Tests:** 355 + 18 new = 373 tests
- **Integration Tests:** Existing + 18 new + 15 batch = 33+ tests
- **Interactive Tests:** 48 + 13 new = 61 tests
- **Grand Total:** ~467 automated tests

## Risk Assessment

### HIGH Risk (Acknowledged and Mitigated)
- **Pager Manual Validation Not Executed:**
  - **Risk:** Visual rendering bug (Issue #14) cannot be confirmed fixed
  - **Mitigation:** Pager disabled by default (AC-3), users protected
  - **Acceptance:** Sprint planning explicitly acknowledges shipping without manual validation

### MEDIUM Risk (Monitored)
- **Performance Not Benchmarked:**
  - **Risk:** SAMPLE queries may be slow on large tables
  - **Mitigation:** Observational testing during integration tests, SAMPLE is Teradata-native optimization
  - **Acceptance:** No specific SLA defined in AC-13

### LOW Risk
- None identified

## Gaps and Limitations

### Documented Gaps
1. **Manual pager validation:** Documented in TC-033-PAGER-MANUAL but not executed
2. **Performance benchmarks:** No criterion benchmarks for sampling commands
3. **Cross-platform testing:** Tests run on macOS only (development environment)

### Accepted Limitations
1. **Pager fix unverified:** Shipping with pager disabled by default
2. **No human UX validation:** Data sampling commands tested programmatically only
3. **Limited error scenarios:** Testing with accessible tables only, not all permission scenarios

## Success Criteria

**Sprint 33 is APPROVED if:**
- ✅ All 373 unit tests pass (355 existing + 18 new)
- ✅ All 61 interactive tests pass (48 existing + 13 new)
- ✅ All integration tests pass (existing + 33 new)
- ✅ TC-033-001 (pager disabled by default) passes
- ✅ No regressions in existing functionality
- ⚠️ TC-033-PAGER-MANUAL documented (execution blocked by no human tester)

**Quality Gate:** 100% automated test pass rate required for APPROVED verdict

## References

- **Test Strategy:** `tests/strategy/sprint-33-test-strategy.md`
- **Sprint Planning:** `docs/sprints/sprint-33-planning.md`
- **Test Cases:** `tests/cases/TC-033-*.md`
- **Test Documentation:** `docs/testing/`
- **GitHub Issue:** #14 - [BUG] Pager broken and on by default
