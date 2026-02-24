# Sprint 38 Test Cases Summary

## Overview

**Sprint:** 38 - PMON Foundation: System Config & Lock Monitoring
**Date:** 2026-02-24
**Type:** Feature Sprint (two PMON DBA commands)

## Test Case Documents

### Feature 1: System Configuration Summary (`/sysconfig`) - P0

| Test ID | Title | Category | ACs Covered | Test Count |
|---------|-------|----------|-------------|------------|
| TC-038-001 | SysconfigInfo SQL, Struct, and Formatting Unit Tests | Unit | AC-1, AC-2, AC-3, AC-8, AC-9 | 12 unit |
| TC-038-002 | Sysconfig Batch Mode CLI Integration Tests | Integration | AC-4 | 3 no-DB + 2 live-DB |
| TC-038-003 | Sysconfig REPL Tab Completion and Help Text | Interactive | AC-5, AC-6 | 3 interactive |
| TC-038-004 | Sysconfig REPL Command Execution and Alias | Interactive | AC-1, AC-2, AC-3, AC-9 | 3 interactive |
| TC-038-005 | Sysconfig Error Handling | Unit + Interactive | AC-7 | 4 unit + 1 interactive |

**Feature 1 Total:** 16 unit tests + 5 integration tests + 7 interactive tests = **28 tests**

### Feature 2: Session Blocking & Lock Information (`/locks`) - P0

| Test ID | Title | Category | ACs Covered | Test Count |
|---------|-------|----------|-------------|------------|
| TC-038-006 | LockInfo SQL, Parsing, Lock Type Mapping Unit Tests | Unit | AC-1, AC-2, AC-3, AC-8, AC-9 | 15 unit |
| TC-038-007 | Locks Batch Mode CLI Integration Tests | Integration | AC-4 | 3 no-DB + 2 live-DB |
| TC-038-008 | Locks REPL Tab Completion and Help Text | Interactive | AC-5, AC-6 | 3 interactive |
| TC-038-009 | Locks REPL Command Execution and Alias | Interactive | AC-1, AC-2, AC-3, AC-9 | 3 interactive |
| TC-038-010 | Locks Error Handling | Unit + Interactive | AC-7 | 5 unit + 1 interactive |

**Feature 2 Total:** 20 unit tests + 5 integration tests + 7 interactive tests = **32 tests**

## Total Test Coverage

### New Tests to Implement

**Feature 1 (/sysconfig):**
- Unit tests: 16 tests (SQL constants, struct parsing, formatters, error messages, alias)
- Integration tests: 5 tests (3 no-DB wiring + 2 live-DB `#[ignore]`)
- Interactive tests: 7 tests (all `#[ignore]`, require live database)
- **Total Feature 1:** 28 tests

**Feature 2 (/locks):**
- Unit tests: 20 tests (SQL, struct parsing x4 lock types, lock type mapping x4, empty state, formatters, alias)
- Integration tests: 5 tests (3 no-DB wiring + 2 live-DB `#[ignore]`)
- Interactive tests: 7 tests (all `#[ignore]`, require live database)
- **Total Feature 2:** 32 tests (Feature 2 is more complex due to lock type mapping)

**Sprint 38 Total New Tests:** 36 unit + 10 integration + 14 interactive = **60 automated tests**

### Existing Tests to Run (Regression)

- **Regression Suite:** ~721 tests (Sprint 37 baseline)

**Sprint 38 Target Test Count:**
- **Baseline:** ~721 tests (Sprint 37)
- **New tests:** +60 tests
- **Total Target:** ~781 tests

**Note:** Live-DB integration tests (4) and all interactive tests (14) are marked `#[ignore]`. They require a database connection.

## Acceptance Criteria Coverage Map

### Feature 1: System Configuration Summary (9 ACs)

| AC | Description | Test Cases | Test Type |
|----|-------------|------------|-----------|
| AC-1 | Queries DBC.DBCInfoV for version/release | TC-038-001, TC-038-004 | Unit + Interactive |
| AC-2 | AMP count via HASHAMP()+1 | TC-038-001, TC-038-004 | Unit + Interactive |
| AC-3 | Displays version, node count, AMP/PE topology | TC-038-001, TC-038-002, TC-038-004 | Unit + Integration + Interactive |
| AC-4 | `tq sysconfig` with table/csv/json | TC-038-002 | Integration |
| AC-5 | Tab completion includes `/sysconfig` | TC-038-003 | Interactive |
| AC-6 | Help text (compact + extended) | TC-038-003 | Unit + Interactive |
| AC-7 | Error handling for privilege errors | TC-038-005 | Unit + Interactive |
| AC-8 | Unit tests for SQL, formatting, parsing | TC-038-001 | Unit (meta) |
| AC-9 | `/sc` short alias | TC-038-001, TC-038-004 | Unit + Interactive |

**Coverage:** 9/9 ACs (100%)

### Feature 2: Session Blocking & Lock Information (9 ACs)

| AC | Description | Test Cases | Test Type |
|----|-------------|------------|-----------|
| AC-1 | Queries DBC.LockInfoV | TC-038-006, TC-038-009 | Unit + Interactive |
| AC-2 | Displays locked object, lock type, sessions | TC-038-006, TC-038-007, TC-038-009 | Unit + Integration + Interactive |
| AC-3 | Blocking chain identification | TC-038-006 | Unit |
| AC-4 | `tq locks` with table/csv/json | TC-038-007 | Integration |
| AC-5 | Tab completion includes `/locks` | TC-038-008 | Interactive |
| AC-6 | Help text (compact + extended) | TC-038-008 | Unit + Interactive |
| AC-7 | Error handling for privilege errors | TC-038-010 | Unit + Interactive |
| AC-8 | Unit tests for SQL, formatting, mapping, parsing | TC-038-006 | Unit (meta) |
| AC-9 | `/lk` short alias | TC-038-006, TC-038-009 | Unit + Interactive |

**Coverage:** 9/9 ACs (100%)

**Overall Sprint Coverage:** 18/18 ACs (100%)

## Test Execution Plan

### Phase 1: Unit Tests (no database required)

```bash
# Feature 1 unit tests
cargo test --lib commands::sysconfig::tests

# Feature 2 unit tests
cargo test --lib commands::locks::tests

# Full unit regression
cargo test --lib
```

**Expected:** 36 new unit tests passing

### Phase 2: Integration Tests - No Database (CLI wiring)

```bash
cargo test --test integration_sysconfig
cargo test --test integration_locks
```

**Expected:** 6 new no-DB integration tests passing (3 per feature)

### Phase 3: Integration Tests - Live Database

```bash
cargo test --test integration_sysconfig -- --ignored
cargo test --test integration_locks -- --ignored
```

**Expected:** 4 new live-DB integration tests passing (2 per feature)

### Phase 4: Interactive Tests - REPL

```bash
cargo test --test interactive_tests sysconfig -- --ignored --test-threads=1
cargo test --test interactive_tests locks -- --ignored --test-threads=1
```

**Expected:** 14 new interactive tests passing (7 per feature)

### Phase 5: Full Regression

```bash
# All unit tests
cargo test --lib

# All integration tests (no DB)
cargo test --test integration_tests
cargo test --test integration_sysconfig
cargo test --test integration_locks

# All interactive tests (requires database)
cargo test --test interactive_tests -- --ignored --test-threads=1

# Expected: ~781 tests passing (721 baseline + 60 new)
```

## Database Requirements

| Test Category | Feature 1 | Feature 2 | Requires DB |
|---------------|-----------|-----------|-------------|
| Unit tests | 16 | 20 | No |
| Integration (no-DB) | 3 | 3 | No |
| Integration (live-DB) | 2 | 2 | Yes (#[ignore]) |
| Interactive | 7 | 7 | Yes (#[ignore]) |
| **Total** | **28** | **32** | 18 need DB |

## Risk Assessment

### Low Risk Areas

- **Unit tests for /sysconfig:** Straightforward struct parsing following sessions.rs pattern
- **Unit tests for /locks empty state:** "No locks" path is the most common and easily tested
- **CLI wiring tests:** No database needed, simple argument validation

### Medium Risk Areas

- **Lock type mapping completeness:** Teradata may use lock type codes not anticipated (e.g., beyond RD/WR/EX/SR)
  - **Mitigation:** `test_lock_type_mapping_unknown_preserved` validates graceful handling of unknown codes
- **DBC.LockInfoV availability:** View may not exist on older Teradata versions
  - **Mitigation:** `test_view_not_found_error_produces_helpful_message` validates graceful handling
- **Interactive tests with no active locks:** Test environment may have no locks to display
  - **Mitigation:** Tests validate "no locks" path explicitly, not lock-present path

### High Risk Areas

- None identified

## Tool Requirements

No new testing tools required. All testing infrastructure already available:
- Unit test framework (Rust built-in)
- Integration tests (`std::process::Command`, `assert_cmd`)
- Interactive tests (`expectrl` + PTY)
- `DatabaseClient::mock()` for unit test signatures
- `Value::*` enum for mock row construction

## Baseline Comparison

| Metric | Sprint 37 Baseline | Sprint 38 Target | Delta |
|--------|-------------------|------------------|-------|
| Unit Tests | ~481 | ~517 | +36 |
| Integration Tests (no-DB) | ~240 | ~246 | +6 |
| Integration Tests (live-DB) | ~4 | ~8 | +4 |
| Interactive Tests | ~196 | ~210 | +14 |
| Total Tests | ~721 | ~781 | +60 |
| Test Pass Rate | 100% | 100% | 0% |
| Features Tested | 37 sprints | 38 sprints | +2 |

## References

- Sprint 38 Planning: `docs/sprints/sprint-38-planning.md`
- Sprint 38 Test Strategy: `tests/strategy/sprint-38-test-strategy.md`
- Sprint 37 Review: `docs/sprints/sprint-37-review.md`
- sessions.rs Pattern: `src/commands/sessions.rs`
- Admin User Stories: `docs/specifications/admin-user-stories.md`
