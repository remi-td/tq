# Sprint 37 Test Cases Summary

## Overview

**Sprint:** 37 - External Editor Integration
**Date:** 2026-02-13
**Type:** Feature Sprint (`/edit` Command + Live-DB Test)

## Test Case Documents

### Feature 1: `/edit` Command - External Editor Integration - P0

| Test ID | Title | Category | ACs Covered | Test Count |
|---------|-------|----------|-------------|------------|
| TC-037-001 | Editor Resolution and Temp File Creation | Unit | AC-1, AC-4, AC-9 | 8 unit |
| TC-037-002 | Edit Modified Content Execution | Integration + Interactive | AC-2, AC-10 | 2 integration + 2 interactive |
| TC-037-003 | Edit Without Changes Skips Execution | Unit + Integration + Interactive | AC-3 | 4 unit + 2 integration + 2 interactive |
| TC-037-004 | Edit Tab Completion and Help Text | Interactive | AC-5, AC-6 | 9 interactive |
| TC-037-005 | Edit Error Handling | Unit + Interactive | AC-7, AC-8 | 3 unit + 5 interactive |
| TC-037-006 | Edit Full REPL Mode Only | Integration + Interactive | AC-11 | 3 integration + 3 interactive |

**Feature 1 Total:** 15 unit tests + 7 integration tests + 21 interactive tests = **43 automated tests**

**Meta-validation (not separate tests):**
- **AC-12**: Unit tests cover all paths (validated by TC-037-001, TC-037-003, TC-037-005)
- **AC-13**: Integration tests validate CLI behavior (validated by TC-037-002, TC-037-003, TC-037-006)

### Feature 2: `/show indexes` Live-DB Test - P1

| Test ID | Title | Category | ACs Covered | Test Count |
|---------|-------|----------|-------------|------------|
| TC-037-007 | Show Indexes Live Database Test | Integration (#[ignore]) | AC-14, AC-15 | 4 integration (#[ignore]) |

**Feature 2 Total:** 4 integration tests (#[ignore]) = **4 automated tests**

## Total Test Coverage

### New Tests to Implement

**`/edit` Command Track:**
- **Unit tests:** 15 tests (editor resolution, temp files, content comparison, command parsing, error handling)
- **Integration tests:** 7 tests (mock editor workflow, mode detection)
- **Interactive tests:** 21 tests (REPL behavior, tab completion, help text, error messages)
- **Total new automated tests:** 43 tests

**`/show indexes` Live-DB Test Track:**
- **Integration tests (#[ignore]):** 4 tests (live database validation, output format)
- **Total new automated tests:** 4 tests

**Sprint 37 Total New Tests:** 15 unit + 11 integration + 21 interactive = **47 automated tests**

### Existing Tests to Run

- **Regression Suite:** 674 tests (Sprint 36 baseline)
  - Unit tests: ~456
  - Integration/interactive tests: ~218

**Sprint 37 Target Test Count:**
- **Baseline:** 674 tests (Sprint 36)
- **New tests:** +47 tests
- **Total:** 721 tests

**Note:**
- Interactive tests marked `#[ignore]` require database access (run with `--ignored --test-threads=1`)
- Live-DB tests for `/show indexes` marked `#[ignore]` (run with `--ignored`)
- Integration tests with mock editors do NOT require database

## Acceptance Criteria Coverage Map

### Feature 1: `/edit` Command (13 ACs)

| AC | Description | Test Cases | Test Type |
|----|-------------|------------|-----------|
| AC-1 | Opens temp `.sql` file using $EDITOR/$VISUAL/vi fallback | TC-037-001 | Unit |
| AC-2 | On save and exit, edited SQL executed automatically | TC-037-002 | Integration + Interactive |
| AC-3 | On exit without changes, no execution occurs | TC-037-003 | Unit + Integration + Interactive |
| AC-4 | Alias `\e` works identically to `/edit` | TC-037-001 | Unit |
| AC-5 | Tab completion includes `/edit` and `\e` | TC-037-004 | Interactive |
| AC-6 | `/help` includes `/edit` command | TC-037-004 | Interactive |
| AC-7 | Error when no previous query | TC-037-005 | Unit + Interactive |
| AC-8 | Error when $EDITOR not set and vi not found | TC-037-005 | Unit + Interactive |
| AC-9 | Temp file uses `.sql` extension | TC-037-001 | Unit |
| AC-10 | Edited query stored as `last_sql` | TC-037-002 | Integration + Interactive |
| AC-11 | Works in full REPL mode only | TC-037-006 | Integration + Interactive |
| AC-12 | Unit tests cover all paths | TC-037-001, 003, 005 | Meta-validation |
| AC-13 | Integration tests validate CLI behavior | TC-037-002, 003, 006 | Meta-validation |

**Coverage:** 13/13 ACs (100%)

### Feature 2: `/show indexes` Live-DB Test (2 ACs)

| AC | Description | Test Cases | Test Type |
|----|-------------|------------|-----------|
| AC-14 | `#[ignore]` test with real Teradata connection | TC-037-007 | Integration (#[ignore]) |
| AC-15 | Validates output format and column headers | TC-037-007 | Integration (#[ignore]) |

**Coverage:** 2/2 ACs (100%)

**Overall Sprint Coverage:** 15/15 ACs (100%)

## Test Execution Plan

### Phase 0: Mock Editor Setup (15 minutes) - PREREQUISITE

**Priority:** Critical (blocks Feature 1 tests)

**Sequence:**
1. Create `tests/fixtures/mock_editors/` directory
2. Write mock editor scripts:
   - `mock_editor_modify.sh` - Appends to file, exits 0
   - `mock_editor_no_change.sh` - Exits 0 without modifying
   - `mock_editor_error.sh` - Exits with code 1
   - `mock_editor_empty.sh` - Empties file, exits 0
3. Make scripts executable: `chmod +x tests/fixtures/mock_editors/*.sh`
4. Verify scripts work: `bash tests/fixtures/mock_editors/mock_editor_modify.sh /tmp/test.sql`

**Expected results:** 4 executable mock editor scripts ready for use

### Phase 1: Unit Tests (1-2 hours)

**Run tests in sequence:**

1. **Unit tests for `/edit`:**
   ```bash
   cargo test --lib repl::metacommands::edit
   # Expected: 15 new tests (TC-037-001, 003, 005)
   ```

**Total Phase 1:** 15 tests

### Phase 2: Integration Tests (1-2 hours)

**Run tests in sequence:**

1. **Integration tests for `/edit` (with mock editors):**
   ```bash
   cargo test --test integration_edit_command
   # Expected: 7 new tests (TC-037-002, 003, 006)
   ```

2. **Integration tests for `/show indexes` (live-DB, optional):**
   ```bash
   cargo test --test integration_show_indexes -- --ignored
   # Expected: 4 new tests (TC-037-007)
   # Note: Only if database available
   ```

**Total Phase 2:** 7 integration tests (mock) + 4 integration tests (live-DB, optional)

### Phase 3: Interactive Tests (2-3 hours)

**Run tests in sequence:**

1. **Interactive tests for `/edit` (requires database + mock editor):**
   ```bash
   cargo test --test interactive_tests edit -- --ignored --test-threads=1
   # Expected: 21 new tests (TC-037-002, 003, 004, 005, 006)
   ```

**Total Phase 3:** 21 tests

### Phase 4: Full Regression (30 minutes)

**Verify no regressions:**
```bash
# Run all unit tests
cargo test --lib

# Run all integration tests
cargo test --test integration_*

# Run all interactive tests (requires database)
cargo test --test interactive_tests -- --ignored --test-threads=1

# Expected: 721 tests passing (674 baseline + 47 new)
# Note: 4 live-DB tests optional if database available
```

### Phase 5: Test Evidence Collection

**Create test evidence document:**
- File: `tests/results/sprint-37/test-evidence-1.md`
- Contents:
  - Full `cargo test` output (all 721 tests)
  - Test execution timestamps
  - Pass/fail summary by test case
  - Database connection verification
  - Mock editor verification

### Phase 6: Test Report

**Create test report:**
- File: `tests/results/sprint-37/REPORT.md`
- Contents:
  - Verdict: APPROVED / REJECTED / BLOCKED
  - Test execution summary (721/721 passed)
  - AC coverage validation (15/15 satisfied)
  - Issues found (if any)
  - Recommendations (if REJECTED)

## Success Criteria

### Must Achieve (BLOCKING for APPROVED)

- ✅ All 47 new automated tests pass (100%)
- ✅ All 674 existing tests pass (100% - zero regressions)
- ✅ All 15 ACs satisfied
- ✅ Mock editor scripts created and functional
- ✅ Database connection available for interactive tests
- ✅ No panics or crashes

### Quality Standards

- **100% test pass rate required** (no failures allowed)
- **100% AC coverage required** (all 15 ACs tested)
- **Zero regressions tolerated** (all existing tests must pass)
- **Clear error messages** (validation of AC-7, AC-8)
- **Mock editor approach** (real editors cannot be automated)
- **Manual validation recommended** (test with real editors: vim, nano, VS Code)

## Risk Assessment

### Low Risk Areas

- **Editor resolution:** Clear precedence ($VISUAL → $EDITOR → vi)
- **Temp file creation:** Using standard tempfile crate
- **Command parsing:** Simple metacommand, no arguments
- **Unit test coverage:** Comprehensive for all logic paths

### Medium Risk Areas

- **Mock editor testing:** Mock editors don't cover all real editor quirks
  - **Mitigation:** Manual validation checklist with common editors
- **Interactive tests:** Require database connection, PTY simulation
  - **Mitigation:** Use expectrl (proven in previous sprints), mark with `#[ignore]`
- **Content comparison:** Whitespace handling edge cases
  - **Mitigation:** Unit tests cover various scenarios, document trim strategy

### High Risk Areas

- None identified

## Database Requirements

**Feature 1 (`/edit` Command):**
- Database required: **YES** (interactive tests only)
- Unit tests: No database (pure logic)
- Integration tests with mock editor: No database (CLI commands only)
- Interactive tests: Require live database connection

**Feature 2 (`/show indexes` Live-DB Test):**
- Database required: **YES** (all tests)
- Integration tests: Require live database with DBC.IndicesV access

**Summary:**
- **Unit tests:** 15 tests (no database required)
- **Integration tests (mock editor):** 7 tests (no database required)
- **Interactive tests:** 21 tests (require database connection)
- **Live-DB integration tests:** 4 tests (require database connection)
- **Total tests requiring database:** 25/47 (53%)

## Baseline Comparison

| Metric | Sprint 36 Baseline | Sprint 37 Target | Delta |
|--------|-------------------|------------------|-------|
| Unit Tests | 456 | 471 | +15 |
| Integration Tests | 218 | 229 | +11 |
| Interactive Tests | (included above) | (included above) | +21 |
| Total Tests | 674 | 721 | +47 |
| Test Pass Rate | 100% | 100% | 0% |
| Features Tested | 36 sprints | 37 sprints | +1 |

**Note:** Sprint 37 adds moderate test coverage (47 new tests, 7% increase).

## Dependencies and Execution Order

### Implementation Dependencies

- **Phase 0 (Mock Editor Setup):** quality-validator creates mock editor scripts
- **Phase 2 (Design):** cli-ux-designer + rust-teradata-architect (parallel)
- **Phase 3 (Implementation):** rust-teradata-architect completes code
- **Phase 4 (Test Execution):** quality-validator executes tests

### Test Execution Dependencies

1. **BLOCKING:** Phase 0 (mock editor setup) must complete before any Feature 1 tests
2. **Parallel:** Unit tests (Feature 1) - independent
3. **Sequential:** Unit tests before integration/interactive (verify logic first)
4. **Sequential:** Integration tests require mock editors
5. **Sequential:** Interactive tests require database setup + mock editors
6. **Parallel:** All regression tests once implementation complete

### Blocking Conditions

- Mock editor scripts MUST exist before Feature 1 integration/interactive tests
- Unit tests MUST pass before running interactive tests
- Implementation MUST be complete before test execution
- Database MUST be available for interactive tests (21 tests will be BLOCKED otherwise)
- Live-DB tests optional (can skip if database unavailable)

## Test Artifacts

### Input Artifacts (Prerequisites)

- Sprint 37 Planning: `docs/sprints/sprint-37-planning.md` ✅
- Sprint 37 Test Strategy: `tests/strategy/sprint-37-test-strategy.md` ✅
- Sprint 36 Review: `docs/sprints/sprint-36-review.md` ✅

### Output Artifacts (Deliverables)

- Test Case Docs: `tests/cases/TC-037-*.md` (7 files) ✅ CREATED
- Test Summary: `tests/cases/TC-037-SUMMARY.md` ✅ CREATED
- Mock Editor Scripts: `tests/fixtures/mock_editors/*.sh` (4 files) (pending creation)
- Test Evidence: `tests/results/sprint-37/test-evidence-1.md` (pending execution)
- Test Report: `tests/results/sprint-37/REPORT.md` (pending execution)
- Updated Index: `tests/cases/INDEX.md` (pending update)

## Lessons Learned Integration

### From Sprint 36 Review

- **Lesson:** Clear test case documentation improves execution efficiency
  - **Applied:** 7 detailed test case documents created
- **Lesson:** Database-dependent tests marked with #[ignore]
  - **Applied:** All 25 database tests documented with `#[ignore]` markers
- **Lesson:** Mock approach needed when real dependencies cannot be automated
  - **Applied:** Mock editor scripts enable automated testing of external editor workflow

### From Sprint 35 Review

- **Lesson:** Unit tests before integration tests (validate logic first)
  - **Applied:** Unit tests in Phase 1, integration/interactive in Phases 2-3
- **Lesson:** Graceful error handling improves UX
  - **Applied:** AC-7, AC-8 focus on clear error messages with actionable guidance

## Test Strategy Highlights

### Decision-Making Process

All tests derived from feature characteristics using test strategy decision tree.

**Example: Editor Resolution (TC-037-001)**
```
Feature characteristic: External process + Environment variables + File I/O
↓
Decision tree: IF "External process" → Mock approach REQUIRED
              IF "Environment variables" → Unit tests REQUIRED
↓
Result: 8 unit tests (resolution logic, temp files, parsing)
```

**Example: Edit Execution (TC-037-002)**
```
Feature characteristic: Interactive PTY + REPL + External process + Database
↓
Decision tree: IF "Interactive PTY" → Interactive tests REQUIRED
              IF "External process" → Mock editor REQUIRED
↓
Result: 2 integration tests (mock editor) + 2 interactive tests (REPL + mock)
```

**Example: Live-DB Test (TC-037-007)**
```
Feature characteristic: Database-dependent + System catalog query
↓
Decision tree: IF "Database-dependent" → Integration test #[ignore] REQUIRED
↓
Result: 4 integration tests (live database, optional)
```

### Coverage Sufficiency

**Question:** If all planned tests pass, can we claim features "work as specified"?

**Answer:** YES (with documented gap for real editor compatibility)

- **`/edit` Command:** Unit tests validate logic, integration tests validate workflow with mock editor, interactive tests validate REPL UX
- **Live-DB Test:** Integration tests validate real database behavior, output format
- **Combined coverage:** **Comprehensive with MEDIUM-risk gap (real editor compatibility)**
- **Gap mitigation:** Manual validation checklist covers common editors (vim, nano, VS Code)

### Gaps Identified and Accepted

- **Real editor compatibility:** Mock editors cannot test vim, nano, emacs, VS Code
  - **Risk:** MEDIUM
  - **Mitigation:** Manual validation checklist, community testing
  - **Acceptable:** Mock editor covers all core logic paths

- **Cross-platform editor quirks:** Unix editors may behave differently on Windows
  - **Risk:** LOW
  - **Mitigation:** Standard env vars ($EDITOR, $VISUAL) are cross-platform
  - **Acceptable:** Community testing will identify rare quirks

- **Performance tests:** Not required (no performance requirements)
  - **Risk:** LOW
  - **Mitigation:** Monitor in practice
  - **Acceptable:** File I/O is fast, editor launch time out of our control

## New Testing Infrastructure

### Mock Editor Scripts

**Created in Phase 0:**
- Location: `tests/fixtures/mock_editors/`
- Scripts:
  1. `mock_editor_modify.sh` - Simulates editor that modifies file
  2. `mock_editor_no_change.sh` - Simulates editor that exits without changes
  3. `mock_editor_error.sh` - Simulates editor failure (exit code 1)
  4. `mock_editor_empty.sh` - Simulates editor that empties file
- Purpose: Enable automated testing of `/edit` workflow without real editor interaction
- Usage: Set $EDITOR to mock script path in tests

## Next Steps

1. **PHASE 0 (PREREQUISITE):** Create mock editor scripts - 15 minutes
2. **WAIT:** For rust-teradata-architect to complete implementation
3. **EXECUTE:** Unit tests (TC-037-001, 003, 005) - 1-2 hours
4. **EXECUTE:** Integration tests (TC-037-002, 003, 006, 007) - 1-2 hours
5. **EXECUTE:** Interactive tests (TC-037-002, 003, 004, 005, 006) - 2-3 hours
6. **EXECUTE:** Full regression (721 tests) - 30 minutes
7. **COLLECT:** Test evidence (cargo test output) - 30 minutes
8. **REPORT:** Create test report with verdict - 30 minutes
9. **ITERATE:** If REJECTED, work with architect to resolve, re-test

## References

- Sprint 37 Planning: `docs/sprints/sprint-37-planning.md`
- Sprint 37 Test Strategy: `tests/strategy/sprint-37-test-strategy.md`
- Sprint 36 Review: `docs/sprints/sprint-36-review.md`
- REPL Specifications: `docs/specifications/repl.md`
- Test Approach: `docs/testing/approach.md`
- Test Strategy Template: `tests/strategy/test-strategy-template.md`
