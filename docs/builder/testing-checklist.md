# Testing Checklist for tq Sprints

**Version:** 1.0.0
**Last Updated:** 2026-01-21
**Owner:** CLI UX Designer
**Purpose:** Phase-specific testing requirements for Quality Validator agent

---

## Overview

This checklist ensures testing requirements are validated at each phase of the sprint workflow. The Quality Validator agent must use this checklist to validate testing readiness and execution.

**Key Principle:** "If a feature is specified, it has a test. If a test exists, it passes. If it passes, the spec is accurate."

---

## Quick Start: 3 Key Questions Per Phase

### Phase 2: Design
1. What test infrastructure does this feature need?
2. Does the infrastructure exist and work?
3. Can I write tests today for this feature class?

### Phase 3: Build & Test
1. Does each acceptance criterion have a test?
2. Are my tests high quality (independent, deterministic, clear)?
3. Did I test what users see (not just what code does)?

### Phase 4: Ship
1. Do all tests still pass (100% pass rate)?
2. Did I perform manual smoke testing?
3. Does the feature match the specification?

---

## Phase 2: Design - Test Infrastructure Availability Check

**Goal:** Ensure test infrastructure exists BEFORE implementation begins.

**Quality Validator Responsibilities:**
- Review feature requirements from sprint planning
- Identify what type of testing is required
- Verify appropriate test infrastructure exists
- Block Phase 3 if infrastructure is inadequate

### Checklist: Test Infrastructure Availability

- [ ] **Review Feature Requirements**
  - Read sprint planning document
  - Identify all acceptance criteria
  - Understand feature scope (REPL vs batch vs core)

- [ ] **Classify Testing Requirements**
  - [ ] Unit tests needed? (logic, algorithms, data transformations)
  - [ ] Integration tests needed? (database interactions, CLI workflows)
  - [ ] Interactive tests needed? (REPL features, user-facing behavior)

- [ ] **Verify Infrastructure Exists**
  - [ ] Unit test framework: `cargo test --lib` works
  - [ ] Integration test framework: `cargo test --test '*'` works
  - [ ] Interactive test framework: `tests/interactive/` exists with working harness
  - [ ] Test fixtures available: Mock data, recorded responses, test databases
  - [ ] Helper functions: Common test utilities documented in `tests/README.md`

- [ ] **REPL Feature Special Requirements**
  - [ ] Interactive test framework operational (expectrl or equivalent)
  - [ ] Can spawn tq REPL process programmatically
  - [ ] Can send keyboard input (Tab, Enter, arrows)
  - [ ] Can capture and parse REPL output
  - [ ] Test fixtures for common REPL scenarios exist

- [ ] **Database Feature Special Requirements**
  - [ ] Live Teradata database available for testing
  - [ ] Test database credentials configured
  - [ ] Sample tables/data available
  - [ ] Metadata queries (DBC views) accessible

- [ ] **Batch/CLI Feature Special Requirements**
  - [ ] File I/O test fixtures
  - [ ] stdin/stdout capture mechanisms
  - [ ] Exit code validation utilities
  - [ ] Pipeline integration test framework

### Decision Gates

**PROCEED to Phase 3 if:**
- All required test infrastructure exists
- Helpers/fixtures documented
- Can write tests immediately when feature is implemented

**BLOCK Phase 3 if:**
- Missing test infrastructure for feature class
- Interactive tests needed but framework not operational
- No clear path to test acceptance criteria

**Action on Block:**
- Document infrastructure gap in blocker document
- Create task to build missing infrastructure
- Escalate to Sprint Coordinator
- Delay feature implementation until infrastructure ready

---

## Phase 3: Build & Test - Test Writing Requirements

**Goal:** Write comprehensive tests that prove acceptance criteria are met.

**Quality Validator Responsibilities:**
- Write tests for each acceptance criterion
- Ensure tests follow testing guidelines
- Validate test quality (not just quantity)
- Run tests and document results

### Checklist: Test Design & Implementation

#### General Requirements

- [ ] **Test Coverage**
  - [ ] Every acceptance criterion has at least one test
  - [ ] Edge cases identified and tested
  - [ ] Error conditions tested (not just happy path)
  - [ ] Regression tests for previous bugs in same area

- [ ] **Test Quality**
  - [ ] Tests are independent (no shared state)
  - [ ] Tests are deterministic (no flaky tests)
  - [ ] Tests have clear names describing what they test
  - [ ] Test assertions are specific and meaningful
  - [ ] Test failures provide actionable error messages

- [ ] **Test Documentation**
  - [ ] Each test has a comment explaining purpose
  - [ ] Complex test setup is documented
  - [ ] Known limitations noted
  - [ ] Test data explained (why these values?)

#### Unit Tests

- [ ] **Logic Testing**
  - [ ] Pure functions tested in isolation
  - [ ] All branches/paths covered
  - [ ] Boundary conditions tested
  - [ ] Type conversions validated

- [ ] **Mock External Dependencies**
  - [ ] Database calls mocked for unit tests
  - [ ] File I/O mocked or use temp files
  - [ ] Network calls mocked
  - [ ] System time mocked if time-dependent

#### Integration Tests

- [ ] **End-to-End Workflows**
  - [ ] Full command execution tested
  - [ ] Real database connections (if safe)
  - [ ] Actual file I/O
  - [ ] Pipeline integration validated

- [ ] **Data Validation**
  - [ ] Output format correctness (JSON valid, CSV RFC 4180)
  - [ ] Data type preservation
  - [ ] NULL handling
  - [ ] Special characters/escaping

- [ ] **Exit Codes**
  - [ ] Success: exit code 0
  - [ ] Runtime error: exit code 1
  - [ ] Usage error: exit code 2
  - [ ] Tested in both interactive and batch modes

#### Interactive Tests (REPL Features ONLY)

- [ ] **Test What Users See**
  - [ ] Visual layout validated (not just content presence)
  - [ ] Content semantically correct (not just mechanically present)
  - [ ] Context awareness verified
  - [ ] Real user interaction patterns simulated

- [ ] **Mandatory for REPL Features**
  - [ ] Tab completion tests verify actual completions (not just that Tab works)
  - [ ] Prompt rendering tests verify colors and format
  - [ ] Multi-line editing tests verify line preservation
  - [ ] History tests verify persistence and recall
  - [ ] Metacommand tests verify output and side effects
  - [ ] Display tests verify alignment and truncation

- [ ] **Live Database Required**
  - [ ] Tests run against real Teradata database
  - [ ] Metadata queries return actual data
  - [ ] Completions are queryable/usable
  - [ ] No mocks for integration-level REPL tests

- [ ] **Anti-Pattern Testing**
  - [ ] Explicitly test known failure modes
  - [ ] Document what should NOT happen
  - [ ] Regression tests for past bugs
  - [ ] Example: "Does NOT show '(SQL keyword)' repeated"

### Test Execution Requirements

- [ ] **Run All Tests**
  - [ ] Unit tests: `cargo test --lib` (100% pass required)
  - [ ] Integration tests: `cargo test --test '*'` (100% pass required)
  - [ ] Interactive tests: `cargo test --test interactive_tests` (100% pass required)

- [ ] **Build Quality**
  - [ ] Zero compiler warnings: `cargo build --all-targets`
  - [ ] Zero clippy warnings: `cargo clippy --all-targets --all-features`
  - [ ] Code formatting: `cargo fmt -- --check`

- [ ] **Performance Check**
  - [ ] Test suite completes in reasonable time (<30s for full suite)
  - [ ] No tests timeout or hang
  - [ ] Resource cleanup verified (no leaked connections/files)

### Test Report Requirements

Create test report documenting:

- [ ] **Test Statistics**
  - Total tests written
  - Tests passing/failing
  - Coverage percentage (if measurable)
  - Test execution time

- [ ] **Acceptance Criteria Validation**
  - Map each criterion to test(s) that prove it
  - Document any criteria not testable (and why)
  - Note any partial implementations

- [ ] **Issues Found**
  - Bugs discovered during testing
  - Test failures with root cause analysis
  - Blockers preventing feature completion

---

## Phase 4: Ship - Test Validation Checklist

**Goal:** Validate all quality gates before declaring sprint complete.

**Quality Validator Responsibilities:**
- Verify 100% test pass rate
- Validate against Definition of Done
- Perform manual smoke testing
- Issue APPROVED or REJECTED verdict

### Checklist: Final Validation

#### Automated Test Validation

- [ ] **All Tests Pass**
  - [ ] Unit tests: 100% pass rate
  - [ ] Integration tests: 100% pass rate
  - [ ] Interactive tests: 100% pass rate (if applicable)
  - [ ] No skipped tests
  - [ ] No flaky tests (run suite 3 times, all pass)

- [ ] **Build Quality**
  - [ ] Zero compiler warnings
  - [ ] Zero clippy warnings
  - [ ] `#![deny(warnings)]` enforced (if P0 requirement)
  - [ ] Code formatted: `cargo fmt -- --check`

- [ ] **Coverage Validation**
  - [ ] All acceptance criteria have tests
  - [ ] All new code has test coverage
  - [ ] Regression tests added for bug fixes
  - [ ] Coverage meets or exceeds target (>60% for new code)

#### Manual Validation

- [ ] **Smoke Testing**
  - [ ] Build release binary: `cargo build --release`
  - [ ] Run basic functionality manually
  - [ ] Verify feature works as user would use it
  - [ ] Check visual appearance (for REPL features)

- [ ] **REPL Feature Manual Validation** (if applicable)
  - [ ] Start REPL: `./target/release/tq repl`
  - [ ] Test feature interactively with real database
  - [ ] Verify visual layout (alignment, colors, formatting)
  - [ ] Check responsiveness and performance
  - [ ] Validate error handling with invalid input
  - [ ] Confirm matches specification exactly

- [ ] **Documentation Validation**
  - [ ] Help text updated: `tq --help`, `tq <cmd> --help`
  - [ ] Examples in help text work
  - [ ] Specification documents updated
  - [ ] Known issues documented (if any)

#### Definition of Done Validation

- [ ] **Feature Complete**
  - [ ] All acceptance criteria met
  - [ ] No known bugs in new features
  - [ ] No regressions in existing features
  - [ ] Performance acceptable

- [ ] **Quality Gates**
  - [ ] 100% test pass rate
  - [ ] Interactive tests pass (for REPL features - BLOCKING)
  - [ ] Zero build warnings
  - [ ] Code review completed (self-review by architect)

- [ ] **Documentation Complete**
  - [ ] Specifications updated with actual behavior
  - [ ] Tests document feature behavior
  - [ ] Known limitations documented
  - [ ] Migration notes (if breaking changes)

- [ ] **Process Compliance**
  - [ ] Sprint planning followed
  - [ ] Design specifications created/updated
  - [ ] Test report generated
  - [ ] All artifacts committed to repo

### Verdict Decision

**Issue APPROVED if:**
- ALL checklist items complete
- 100% test pass rate
- Manual smoke test confirms functionality
- Meets Definition of Done
- No blocking issues

**Issue REJECTED if:**
- Any test failures
- Build warnings present (if enforcement enabled)
- Manual testing reveals issues
- Acceptance criteria not met
- Documentation incomplete

**Action on REJECT:**
- Document all issues in test report
- Create tasks for fixes
- Return to Phase 3 (Build & Test)
- Re-run validation after fixes

---

## Special Considerations

### Interactive Features (REPL)

**CRITICAL:** Interactive features require interactive tests. Unit tests alone are insufficient.

**Requirements:**
1. Interactive test framework must be operational
2. Tests must verify semantic correctness (not just mechanical operation)
3. Live database testing mandatory
4. Visual validation required (manual smoke test)
5. Anti-pattern testing (known failure modes)

**Example:**
- Bad Test: "Tab key triggers completion mechanism"
- Good Test: "Tab after FROM shows database names (not SQL keywords)"

### Batch/Scripting Features

**Requirements:**
1. Exit code correctness critical
2. stdout/stderr separation validated
3. Pipeline integration tested
4. Error message quality verified

### Database Features

**Requirements:**
1. Test with live Teradata database
2. Verify actual SQL execution
3. Validate data type handling
4. Check NULL handling
5. Test error conditions (connection loss, permission denied)

---

## Templates

### Test Report Template

```markdown
# Test Report - Sprint N

**Date:** YYYY-MM-DD
**Tester:** quality-validator agent
**Verdict:** APPROVED | REJECTED

## Test Statistics
- Unit tests: X/X passed (100%)
- Integration tests: Y/Y passed (100%)
- Interactive tests: Z/Z passed (100%)
- Total: N tests, N passed, 0 failed

## Coverage
- Acceptance criteria: X/X validated (100%)
- Code coverage: N% (target: >60%)
- Regression tests: Y tests added

## Manual Validation
- Smoke test: PASS
- Visual inspection: PASS
- Performance: Acceptable
- User experience: Matches specification

## Issues Found
[None | List of issues]

## Recommendations
[APPROVED - Ready to ship | REJECTED - Fix issues listed above]
```

---

## Quick Reference

### Phase 2 Questions
- What test infrastructure does this feature need?
- Does the infrastructure exist and work?
- Can I write tests today for this feature class?
- Should I block Phase 3?

### Phase 3 Questions
- Does each acceptance criterion have a test?
- Are my tests high quality (independent, deterministic, clear)?
- Did I test what users see (not just what code does)?
- Did I run ALL tests? (100% pass rate?)

### Phase 4 Questions
- Do all tests still pass?
- Did I perform manual smoke testing?
- Does the feature match the specification?
- Should I issue APPROVED or REJECTED?

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-21 | 1.0.0 | Initial checklist created for Sprint 14 maintenance sprint | CLI UX Designer |

---

**See Also:**
- [Testing Guidelines](testing-guidelines.md) - Detailed testing methodology and patterns
- [Definition of Done](definitions/done.md) - Sprint completion criteria
- Quality Validator agent instructions: `.claude/agents/quality-validator.md`
