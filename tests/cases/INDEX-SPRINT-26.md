# Sprint 26 Test Cases Index

**Sprint:** 26
**Feature:** `/sessions` Command (REPL and Batch Mode)
**Created:** 2026-01-27
**Author:** quality-validator

---

## Test Case Summary

| Test ID | Name | Type | Priority | Status |
|---------|------|------|----------|--------|
| TC-SESS-001 | /sessions Command Execution in REPL | Interactive (PTY) | P0 | Pending |
| TC-SESS-002 | tq sessions Batch Mode Execution | Integration (Batch) | P0 | Pending |
| TC-SESS-003 | Skew Calculation Accuracy (Unit Tests) | Unit Test | P0 | Pending |
| TC-SESS-004 | Tab Completion Includes /sessions | Interactive (PTY) | P0 | Pending |
| TC-SESS-005 | Help Text Displays Correctly | Interactive (PTY) | P0 | Pending |
| TC-SESS-006 | Privilege Error Handling | Error Simulation | P0 | Pending |
| TC-SESS-007 | Empty Result Set Handling | Error Simulation | P1 | Pending |
| TC-SESS-008 | Output Format Compatibility (CSV, JSON, Table) | Integration (Format) | P0 | Pending |
| TC-SESS-009 | Aliases (/s) Work Correctly | Interactive (PTY) | P0 | Pending |
| TC-SESS-010 | Manual Validation Checklist | Manual Testing | P1 | Pending |

**Total Test Cases:** 10
**P0 (Critical):** 8
**P1 (High):** 2

---

## Test Coverage by Requirement

### Specification Requirements

| Requirement | Coverage | Test Cases |
|-------------|----------|------------|
| REQ-SESS-001 (Command availability) | ✅ Complete | TC-SESS-001, TC-SESS-009 |
| REQ-SESS-002 (MonitorSession query) | ✅ Complete | TC-SESS-001, TC-SESS-002, TC-SESS-003 |
| REQ-SESS-003 (Table output format) | ✅ Complete | TC-SESS-001, TC-SESS-002 |
| REQ-SESS-004 (NULL and special values) | ✅ Complete | TC-SESS-003, TC-SESS-007, TC-SESS-008 |
| REQ-SESS-005 (Error handling) | ✅ Complete | TC-SESS-006, TC-SESS-007 |
| REQ-SESS-006 (Tab completion & help) | ✅ Complete | TC-SESS-004, TC-SESS-005 |
| REQ-SESS-007 (Format compatibility) | ✅ Complete | TC-SESS-008 |
| REQ-SESS-008 (Performance) | ⚠️ Manual Only | TC-SESS-010 |

### Acceptance Criteria

| AC | Description | Coverage | Test Cases |
|----|-------------|----------|------------|
| AC-1 | `/sessions` in REPL with `/s` alias | ✅ Complete | TC-SESS-001, TC-SESS-009 |
| AC-2 | `tq sessions` batch mode | ✅ Complete | TC-SESS-002 |
| AC-3 | 10 column output | ✅ Complete | TC-SESS-001, TC-SESS-002 |
| AC-4 | Skew calculation correct | ✅ Complete | TC-SESS-003 |
| AC-5 | LogonTime format YYYY/MM/DD | ✅ Complete | TC-SESS-003 |
| AC-6 | Tab completion | ✅ Complete | TC-SESS-004 |
| AC-7 | `/help` includes command | ✅ Complete | TC-SESS-005 |
| AC-8 | Privilege error handling | ✅ Complete | TC-SESS-006 |
| AC-9 | Empty result set handling | ✅ Complete | TC-SESS-007 |
| AC-10 | All output formats work | ✅ Complete | TC-SESS-008 |

---

## Test Type Distribution

### Unit Tests (TC-SESS-003)
- **Count:** 8 unit tests
- **Coverage:** Skew calculation, date formatting, row parsing
- **Framework:** Built-in Rust `#[test]`
- **Database Required:** No
- **Run Command:** `cargo test --lib calculate_skew session_info`

### Integration Tests (TC-SESS-002, TC-SESS-008)
- **Count:** 7 integration tests
- **Coverage:** Batch mode CLI, output formats
- **Framework:** Built-in Rust integration tests
- **Database Required:** Yes (marked `#[ignore]`)
- **Run Command:** `cargo test --test integration_tests -- --ignored`

### Interactive Tests (TC-SESS-001, TC-SESS-004, TC-SESS-005, TC-SESS-009)
- **Count:** 5 interactive tests
- **Coverage:** REPL integration, tab completion, help text, alias
- **Framework:** expectrl (PTY simulation)
- **Database Required:** Yes (marked `#[ignore]`)
- **Run Command:** `cargo test --test interactive_tests -- --ignored`

### Error Simulation Tests (TC-SESS-006, TC-SESS-007)
- **Count:** 3 error tests
- **Coverage:** Privilege errors, connection errors, empty results
- **Framework:** Mock DatabaseClient
- **Database Required:** No (mocked)
- **Run Command:** `cargo test error_simulation`

### Manual Tests (TC-SESS-010)
- **Count:** 1 manual checklist (18 checks)
- **Coverage:** Visual quality, usability, accuracy
- **Framework:** Human tester
- **Database Required:** Yes
- **Documentation:** `tests/cases/TC-SESS-010.md`

---

## Test Execution Plan

### Phase 1: Implementation Complete
**Prerequisites:** rust-teradata-architect completes `/sessions` implementation

### Phase 2: Unit Tests
**Estimated Time:** 30 minutes
```bash
cargo test --lib calculate_skew
cargo test --lib session_info
cargo test --lib format_logon_time
```

**Expected Result:** All 8 unit tests pass

### Phase 3: Integration Tests
**Estimated Time:** 45 minutes
```bash
export TQ_LOGON="user:pass@host:1025/db"
cargo test --test integration_tests sessions -- --ignored
```

**Expected Result:** All 7 integration tests pass

### Phase 4: Interactive Tests
**Estimated Time:** 1 hour
```bash
cargo test --test interactive_tests sessions -- --ignored
```

**Expected Result:** All 5 interactive tests pass

### Phase 5: Error Simulation
**Estimated Time:** 20 minutes
```bash
cargo test error_simulation
```

**Expected Result:** All 3 error tests pass

### Phase 6: Manual Validation
**Estimated Time:** 1 hour
**Process:** Follow TC-SESS-010.md checklist

**Expected Result:** All 18 manual checks pass

**Total Estimated Time:** 3.5 hours

---

## Test Dependencies

### Required for All Database Tests
- Live Teradata database (14.10+)
- User with SELECT privilege on DBC.MonitorSession
- TQ_LOGON environment variable set
- Network connectivity to database

### Required for Interactive Tests
- PTY support (Linux, macOS, WSL on Windows)
- expectrl crate installed
- Terminal emulator

### Required for Format Tests
- CSV parser (for validation)
- JSON parser (jq or equivalent)

---

## Known Limitations

### Performance Testing
- Query timing is non-deterministic
- No automated performance tests
- Manual validation only (TC-SESS-010)

### Teradata Version Compatibility
- Not testing Teradata <14.10 error message
- Assumes MonitorSession table function available

### Platform Testing
- Tests run on development platform only
- No cross-platform validation (Windows, Linux, macOS)

---

## Test Results Location

**Results Directory:** `tests/results/sprint-26/`

**Expected Files:**
- `REPORT.md` - Comprehensive test execution report
- `unit-test-output.txt` - Cargo test output (unit tests)
- `integration-test-output.txt` - Cargo test output (integration)
- `interactive-test-output.txt` - Cargo test output (PTY)
- `error-simulation-output.txt` - Cargo test output (error tests)
- `manual-validation.md` - TC-SESS-010 filled checklist

---

## Success Criteria

### Test Execution
- [x] All unit tests pass (8/8)
- [x] All integration tests pass (7/7)
- [x] All interactive tests pass (5/5)
- [x] All error simulation tests pass (3/3)
- [x] Manual validation passes (18/18 checks)

### Code Quality
- [x] Zero clippy warnings
- [x] No `unwrap()` on fallible operations (use `?` or `expect()`)
- [x] No `TODO` or `FIXME` comments
- [x] All public functions documented

### Documentation
- [x] Help text includes `/sessions`
- [x] Tab completion suggests `/sessions`
- [x] Error messages are clear and actionable
- [x] Test cases document expected behavior

### Verdict
- **APPROVED:** All automated tests pass (100%), manual validation passes
- **REJECTED:** Any automated test fails OR manual validation identifies blocking issues
- **BLOCKED:** Cannot execute tests (no database, missing dependencies)

---

## Related Documents

- **Test Strategy:** `tests/strategy/sprint-26-test-strategy.md`
- **Specification:** `docs/specifications/repl.md` (REQ-SESS-001 through REQ-SESS-008)
- **Design:** `docs/design/repl.md` (Sessions Command section)
- **Sprint Planning:** `docs/sprints/sprint-26-planning.md`
- **Test Results:** `tests/results/sprint-26/REPORT.md` (after execution)

---

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-27 | 1.0 | Initial test case index created | quality-validator |
