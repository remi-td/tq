# Sprint 36 Test Cases Summary

## Overview

**Sprint:** 36 - Help Text Update + REPL Enhancements
**Date:** 2026-02-13
**Type:** Feature Sprint (Config Polish + `/repeat` + `/show indexes`)

## Test Case Documents

### Feature 1: Config Help Text & UX Polish - P0

| Test ID | Title | Category | ACs Covered | Test Count |
|---------|-------|----------|-------------|------------|
| TC-036-001 | Config Help Text Content Validation | Unit + Integration | AC-1, AC-2 | 5 unit + 3 integration |
| TC-036-002 | Profile Command with Project Config | Integration | AC-3, AC-4 | 6 integration |
| TC-036-003 | Invalid Project Config Warning | Unit + Integration | AC-5 | 3 unit + 6 integration |

**Feature 1 Total:** 8 unit tests + 15 integration tests = **23 automated tests**

### Feature 2: `/repeat` Command - P1

| Test ID | Title | Category | ACs Covered | Test Count |
|---------|-------|----------|-------------|------------|
| TC-036-004 | `/repeat` Command Basic Behavior | Unit + Interactive | AC-8, AC-9, AC-13, AC-14 | 4 unit + 6 interactive |
| TC-036-005 | `/repeat` with Various SQL Types | Interactive | AC-10 | 8 interactive |
| TC-036-006 | `/repeat` Completion and Help | Interactive | AC-11, AC-12 | 9 interactive |

**Feature 2 Total:** 4 unit tests + 23 interactive tests = **27 automated tests**

### Feature 3: `/show indexes` Command - P1

| Test ID | Title | Category | ACs Covered | Test Count |
|---------|-------|----------|-------------|------------|
| TC-036-007 | `/show indexes` SQL Generation and Parsing | Unit | AC-15, AC-16, AC-17, AC-23 | 10 unit |
| TC-036-008 | `/show indexes` Output Format | Integration + Interactive | AC-18, AC-24 | 7 interactive |
| TC-036-009 | `/show indexes` Error Handling | Integration + Interactive | AC-19, AC-20 | 8 interactive |
| TC-036-010 | `/show indexes` Completion and Help | Interactive | AC-21, AC-22 | 9 interactive |

**Feature 3 Total:** 10 unit tests + 24 interactive tests = **34 automated tests**

## Total Test Coverage

### New Tests to Implement

**Config Polish Track:**
- **Unit tests:** 8 tests (help text generation, error formatting)
- **Integration tests:** 15 tests (CLI behavior, profiles output, warnings)
- **Total new automated tests:** 23 tests

**`/repeat` Command Track:**
- **Unit tests:** 4 tests (command parsing, state handling)
- **Interactive tests:** 23 tests (REPL behavior, various SQL types, completion)
- **Total new automated tests:** 27 tests

**`/show indexes` Command Track:**
- **Unit tests:** 10 tests (SQL generation, argument parsing)
- **Interactive tests:** 24 tests (output format, error handling, completion)
- **Total new automated tests:** 34 tests

**Sprint 36 Total New Tests:** 22 unit + 15 integration + 47 interactive = **84 automated tests**

### Existing Tests to Run

- **Regression Suite:** 710 tests (Sprint 35 baseline)
  - Unit tests: ~456
  - Integration/interactive tests: ~254

**Sprint 36 Target Test Count:**
- **Baseline:** 710 tests (Sprint 35)
- **New tests:** +84 tests
- **Total:** 794 tests

**Note:** Interactive tests marked `#[ignore]` require database access (run with `--ignored --test-threads=1`).

## Acceptance Criteria Coverage Map

### Feature 1: Config Help Text & UX Polish (7 ACs)

| AC | Description | Test Cases | Test Type |
|----|-------------|------------|-----------|
| AC-1 | `tq help config` includes project config section | TC-036-001 | Unit + Integration |
| AC-2 | `tq help config` shows 5-level precedence hierarchy | TC-036-001 | Unit + Integration |
| AC-3 | `tq profiles` shows project config path when present | TC-036-002 | Integration |
| AC-4 | `tq profiles` shows tip when no profiles exist | TC-036-002 | Integration |
| AC-5 | Invalid `.tq.toml` produces stderr warning | TC-036-003 | Unit + Integration |
| AC-6 | All existing tests pass (zero regressions) | Full suite | Regression |
| AC-7 | New unit + integration tests for all sub-features | TC-036-001-003 | Unit + Integration |

**Coverage:** 7/7 ACs (100%)

### Feature 2: `/repeat` Command (7 ACs)

| AC | Description | Test Cases | Test Type |
|----|-------------|------------|-----------|
| AC-8 | `/repeat` re-executes last SQL statement | TC-036-004 | Unit + Interactive |
| AC-9 | Clear message when no previous query | TC-036-004 | Unit + Interactive |
| AC-10 | Works after any SQL type (SELECT, INSERT, DDL) | TC-036-005 | Interactive |
| AC-11 | Tab completion includes `/repeat` with description | TC-036-006 | Interactive |
| AC-12 | `/help` includes `/repeat` | TC-036-006 | Interactive |
| AC-13 | Short alias `\r` works | TC-036-004 | Unit + Interactive |
| AC-14 | Unit tests validate all behaviors | TC-036-004 | Unit |

**Coverage:** 7/7 ACs (100%)

### Feature 3: `/show indexes` Command (10 ACs)

| AC | Description | Test Cases | Test Type |
|----|-------------|------------|-----------|
| AC-15 | Displays index info from DBC.IndicesV | TC-036-007, TC-036-008 | Unit + Interactive |
| AC-16 | Qualified name support `database.table` | TC-036-007, TC-036-008 | Unit + Interactive |
| AC-17 | Short alias `\di` works | TC-036-007 | Unit |
| AC-18 | Table shows IndexName, IndexType, ColumnName, ColumnPosition | TC-036-008 | Interactive |
| AC-19 | Error for non-existent table | TC-036-009 | Interactive |
| AC-20 | Error for permission denied | TC-036-009 | Interactive |
| AC-21 | Tab completion includes `/show indexes` | TC-036-010 | Interactive |
| AC-22 | `/help` includes `/show indexes` | TC-036-010 | Interactive |
| AC-23 | Unit tests for SQL generation and parsing | TC-036-007 | Unit |
| AC-24 | Integration tests for CLI behavior | TC-036-008, TC-036-009 | Interactive |

**Coverage:** 10/10 ACs (100%)

**Overall Sprint Coverage:** 24/24 ACs (100%)

## Test Execution Plan

### Phase 1: Config Polish Tests (2-3 hours)

**Run tests in sequence:**

1. **Unit tests:**
   ```bash
   cargo test --lib config::help_text
   cargo test --lib config::warnings
   # Expected: 8 new tests
   ```

2. **Integration tests:**
   ```bash
   cargo test --test integration_help_text
   cargo test --test integration_profiles_project_config
   # Expected: 15 new tests
   ```

**Total Phase 1:** 23 tests

### Phase 2: `/repeat` Command Tests (2-3 hours)

**Run tests in sequence:**

1. **Unit tests:**
   ```bash
   cargo test --lib repl::metacommands::repeat
   # Expected: 4 new tests
   ```

2. **Interactive tests (requires database):**
   ```bash
   cargo test --test interactive_tests repeat -- --ignored --test-threads=1
   # Expected: 23 new tests
   ```

**Total Phase 2:** 27 tests

### Phase 3: `/show indexes` Tests (3-4 hours)

**Run tests in sequence:**

1. **Unit tests:**
   ```bash
   cargo test --lib repl::metacommands::show_indexes
   # Expected: 10 new tests
   ```

2. **Interactive tests (requires database):**
   ```bash
   cargo test --test interactive_tests show_indexes -- --ignored --test-threads=1
   # Expected: 24 new tests
   ```

**Total Phase 3:** 34 tests

### Phase 4: Full Regression (30 minutes)

**Verify no regressions:**
```bash
# Run all unit tests
cargo test --lib

# Run all integration tests
cargo test --test integration_*

# Run all interactive tests (requires database)
cargo test --test interactive_tests -- --ignored --test-threads=1

# Expected: 794 tests passing (710 baseline + 84 new)
```

### Phase 5: Test Evidence Collection

**Create test evidence document:**
- File: `tests/results/sprint-36/test-evidence-1.md`
- Contents:
  - Full `cargo test` output (all 794 tests)
  - Test execution timestamps
  - Pass/fail summary by test case
  - Database connection verification

### Phase 6: Test Report

**Create test report:**
- File: `tests/results/sprint-36/REPORT.md`
- Contents:
  - Verdict: APPROVED / NEEDS FIXES / BLOCKED
  - Test execution summary (794/794 passed)
  - AC coverage validation (24/24 satisfied)
  - Issues found (if any)
  - Recommendations (if NEEDS FIXES)

## Success Criteria

### Must Achieve (BLOCKING for APPROVED)

- ✅ All 84 new automated tests pass (100%)
- ✅ All 710 existing tests pass (100% - zero regressions)
- ✅ All 24 ACs satisfied
- ✅ Database connection available for interactive tests
- ✅ No panics or crashes

### Quality Standards

- **100% test pass rate required** (no failures allowed)
- **100% AC coverage required** (all 24 ACs tested)
- **Zero regressions tolerated** (all existing tests must pass)
- **Clear error messages** (validation of AC-5, AC-9, AC-19, AC-20)
- **UX consistency** (help text, completion, error handling)

## Risk Assessment

### Low Risk Areas

- **Config help text:** Unit/integration tests, no database required
- **Config warnings:** Clear error handling requirements
- **`/repeat` parsing:** Simple command, existing `last_sql` field
- **SQL generation:** Unit tests validate correctness without database

### Medium Risk Areas

- **Interactive tests:** Require database connection, PTY simulation
  - **Mitigation:** Use expectrl (proven in previous sprints), mark with `#[ignore]`
- **DBC.IndicesV queries:** System catalog access may vary by Teradata version
  - **Mitigation:** Standard catalog view, fallback strategy documented
- **Tab completion:** PTY-dependent behavior
  - **Mitigation:** Existing completion tests provide patterns

### High Risk Areas

- None identified

## Database Requirements

**Feature 1 (Config Polish):**
- Database required: **NO**
- All tests: Pure CLI testing, no database connection

**Feature 2 (`/repeat` Command):**
- Database required: **YES** (interactive tests only)
- Unit tests: No database (pure logic)
- Interactive tests: Require live database connection

**Feature 3 (`/show indexes` Command):**
- Database required: **YES** (interactive tests only)
- Unit tests: No database (SQL generation logic)
- Interactive tests: Require live database with DBC.IndicesV access

**Summary:**
- **Unit tests:** 22 tests (no database required)
- **Integration tests:** 15 tests (no database required)
- **Interactive tests:** 47 tests (require database connection)
- **Total tests requiring database:** 47/84 (56%)

## Baseline Comparison

| Metric | Sprint 35 Baseline | Sprint 36 Target | Delta |
|--------|-------------------|------------------|-------|
| Unit Tests | 456 | 478 | +22 |
| Integration Tests | 254 | 269 | +15 |
| Interactive Tests | (included above) | (included above) | +47 |
| Total Tests | 710 | 794 | +84 |
| Test Pass Rate | 100% | 100% | 0% |
| Features Tested | 35 sprints | 36 sprints | +1 |

**Note:** Sprint 36 adds significant interactive test coverage (47 new tests).

## Dependencies and Execution Order

### Implementation Dependencies

- **Phase 2 (Design):** cli-ux-designer + rust-teradata-architect (parallel)
- **Phase 3 (Implementation):** rust-teradata-architect completes code
- **Phase 4 (Test Execution):** quality-validator executes tests

### Test Execution Dependencies

1. **Parallel:** Feature 1 (config polish) tests - independent
2. **Sequential:** Unit tests before interactive tests (verify logic first)
3. **Sequential:** Interactive tests require database setup
4. **Parallel:** All regression tests once implementation complete

### Blocking Conditions

- Unit tests MUST pass before running interactive tests
- Implementation MUST be complete before test execution
- Database MUST be available for interactive tests (47 tests will be BLOCKED otherwise)

## Test Artifacts

### Input Artifacts (Prerequisites)

- Sprint 36 Planning: `docs/sprints/sprint-36-planning.md` ✅
- Sprint 36 Test Strategy: `tests/strategy/sprint-36-test-strategy.md` ✅
- Sprint 35 Review: `docs/sprints/sprint-35-review.md` ✅

### Output Artifacts (Deliverables)

- Test Case Docs: `tests/cases/TC-036-*.md` (10 files) ✅ CREATED
- Test Summary: `tests/cases/TC-036-SUMMARY.md` ✅ CREATED
- Test Evidence: `tests/results/sprint-36/test-evidence-1.md` (pending execution)
- Test Report: `tests/results/sprint-36/REPORT.md` (pending execution)
- Updated Index: `tests/cases/INDEX.md` (pending update)

## Lessons Learned Integration

### From Sprint 35 Review

- **Lesson:** Clear test case documentation improves execution efficiency
  - **Applied:** 10 detailed test case documents created
- **Lesson:** Database-dependent tests marked with #[ignore]
  - **Applied:** All 47 interactive tests documented with `#[ignore]` markers
- **Lesson:** Tests must execute and prove behavior, not just exist
  - **Applied:** Comprehensive test execution plan with evidence collection

### From Sprint 34 Review

- **Lesson:** Unit tests before integration tests (validate logic first)
  - **Applied:** Unit tests in Phase 1, interactive tests in Phases 2-3
- **Lesson:** Graceful error handling improves UX
  - **Applied:** AC-5, AC-9, AC-19, AC-20 focus on clear error messages

## Test Strategy Highlights

### Decision-Making Process

All tests derived from feature characteristics using test strategy decision tree.

**Example: Config Help Text (TC-036-001)**
```
Feature characteristic: CLI Batch + Structured output
↓
Decision tree: IF "CLI Batch" → Integration tests REQUIRED
              IF "Structured output" → Unit tests REQUIRED
↓
Result: 5 unit tests (content generation) + 3 integration tests (CLI execution)
```

**Example: `/repeat` Command (TC-036-004)**
```
Feature characteristic: Interactive PTY + REPL metacommand
↓
Decision tree: IF "Interactive PTY" → Interactive tests REQUIRED
              IF "State management" → Unit tests REQUIRED
↓
Result: 4 unit tests (parsing/state) + 6 interactive tests (REPL behavior)
```

**Example: `/show indexes` SQL (TC-036-007)**
```
Feature characteristic: SQL generation + Argument parsing
↓
Decision tree: IF "SQL generation" → Unit tests REQUIRED
↓
Result: 10 unit tests (SQL correctness, argument parsing)
```

### Coverage Sufficiency

**Question:** If all planned tests pass, can we claim features "work as specified"?

**Answer:** YES

- **Config Polish:** Unit tests validate logic, integration tests validate CLI behavior
- **`/repeat` Command:** Unit tests validate parsing/state, interactive tests validate REPL UX
- **`/show indexes` Command:** Unit tests validate SQL, interactive tests validate end-to-end behavior
- **Combined coverage:** **Comprehensive** (all critical paths tested)

### Gaps Identified and Accepted

- **Performance tests:** Not required (no performance requirements for any feature)
- **Cross-version tests:** Deferred (DBC.IndicesV is standard Teradata catalog)
- **Manual UX tests:** Recommended but optional (automated tests validate behavior)

All gaps have risk assessment (LOW) and mitigation strategy.

## Next Steps

1. **WAIT:** For rust-teradata-architect to complete implementation
2. **EXECUTE:** Config polish tests (TC-036-001-003) - 2-3 hours
3. **EXECUTE:** `/repeat` tests (TC-036-004-006) - 2-3 hours
4. **EXECUTE:** `/show indexes` tests (TC-036-007-010) - 3-4 hours
5. **EXECUTE:** Full regression (794 tests) - 30 minutes
6. **COLLECT:** Test evidence (cargo test output) - 30 minutes
7. **REPORT:** Create test report with verdict - 30 minutes
8. **ITERATE:** If NEEDS FIXES, work with architect to resolve, re-test

## References

- Sprint 36 Planning: `docs/sprints/sprint-36-planning.md`
- Sprint 36 Test Strategy: `tests/strategy/sprint-36-test-strategy.md`
- Sprint 35 Review: `docs/sprints/sprint-35-review.md`
- REPL Specifications: `docs/specifications/repl.md`
- Configuration Specifications: `docs/specifications/configuration.md`
- Test Approach: `docs/testing/approach.md`
