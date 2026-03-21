# Sprint 42 Test Strategy: SQL Parser Hardening

**Created:** 2026-03-21
**Author:** quality-validator
**Sprint:** Sprint 42
**Features:** Quote-Aware SQL Statement Splitting (AC-1 to AC-12), Sprint 41 Remediation (AC-13 to AC-15)

---

## Overview

Sprint 42 fixes three critical bugs in the SQL file parser by replacing the naive `sql.split(';')` approach in `src/sql/parser.rs` with a proper character-by-character state-machine lexer. All three bugs share the same root cause and are fixed in a single targeted rewrite of `parse_statements()`.

### Feature Scope

1. **Feature 1: Quote-Aware SQL Statement Splitting** (P0) - Rewrite `parse_statements()` to correctly handle semicolons inside quoted strings, multi-line statements, and comment blocks. Addresses bugs #28, #29, #30.
2. **Feature 2: Sprint 41 Remediation** (P1) - Mark `test_repl_startup_and_quit` as `#[ignore]`, pin `cross-rs` version in release.yml, rename `TMPDIR` to `TQ_TMPDIR` in install.sh.

### Test Profile

This sprint has a straightforward test profile:

- **Feature 1** is **pure library logic** in `src/sql/parser.rs`. The parser takes a `&str` and returns `Vec<ParsedStatement>` with zero external dependencies. This is the ideal target for comprehensive unit tests.
- **Feature 2** is a mix of test-annotation changes (verifiable via `cargo test`), CI YAML changes (structural review), and shell script changes (shellcheck).
- No live database is required for any P0 test.
- No interactive PTY tests are required.

---

## Feature-by-Feature Test Strategy

---

### Feature 1: Quote-Aware SQL Statement Splitting (Bugs #28, #29, #30)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/specifications/batch-mode.md` - Section "SQL File Parser Requirements" (REQ-PARSE-001 through REQ-PARSE-017)
- Secondary: `docs/sprints/sprint-42-planning.md` - Acceptance Criteria AC-1 through AC-12
- Issues: GitHub #28 (semicolons in strings), #29 (multi-line SQL), #30 (comment blocks)

**Requirements:**

1. REQ-PARSE-001: Semicolons only terminate statements when outside quoted strings, line comments, and block comments
2. REQ-PARSE-002: Trailing content without a final semicolon is treated as an implicit last statement
3. REQ-PARSE-003: Empty/whitespace-only/comment-only statements are silently discarded
4. REQ-PARSE-004: Newlines and whitespace within a statement are preserved verbatim
5. REQ-PARSE-005: Single-quoted strings are recognized; inside a string, semicolons and comment markers are not special
6. REQ-PARSE-006: `''` inside a string is an escaped quote - does NOT end the string
7. REQ-PARSE-007: Unterminated string literal at EOF must produce a parse error with accurate line number
8. REQ-PARSE-008: `--` outside strings/block-comments begins a line comment extending to end-of-line
9. REQ-PARSE-009: Line comment content is stripped from statement text before assembly
10. REQ-PARSE-010: `/*` outside strings/line-comments begins a block comment ending at first `*/`
11. REQ-PARSE-011: Block comments may span multiple lines
12. REQ-PARSE-012: Nested block comments are NOT supported; first `*/` closes the block comment
13. REQ-PARSE-013: Unterminated block comment at EOF must produce a parse error with accurate line number
14. REQ-PARSE-014: Line number counter increments on every `\n` regardless of lexical context; starts at 1; does not reset between statements
15. REQ-PARSE-015: `start_line` of each `ParsedStatement` records the line where accumulation began
16. REQ-PARSE-016: Database error messages must include the `start_line` of the failing statement
17. REQ-PARSE-017: Parse errors (unterminated string/comment) must reference the line where the offending construct began, not the EOF line

**Acceptance Criteria from Planning:**

- AC-1: Semicolons inside single-quoted strings (`'...'`) do NOT split statements
- AC-2: Escaped quotes (`''`) inside strings are handled correctly
- AC-3: Multi-line SQL statements (newlines within a statement) are preserved as single statements
- AC-4: Line comments (`-- ...`) are stripped before statement assembly
- AC-5: Block comments (`/* ... */`) are stripped before statement assembly
- AC-6: Comments between statements do not contaminate adjacent statements
- AC-7: Empty lines between statements are handled correctly
- AC-8: Line number tracking remains accurate for error reporting
- AC-9: `has_multiple_statements()` works correctly with new parser
- AC-10: All existing parser tests pass (backwards compatible for simple cases)
- AC-11: New tests cover all 3 bug scenarios from issues #28, #29, #30
- AC-12: `ParsedStatement` struct unchanged (API compatible)

**Feature Characteristics:**

**User Interaction Type:**
- Pure Logic - The parser function `parse_statements(&str) -> Vec<ParsedStatement>` is a pure function with no user interaction, no I/O, and no external dependencies. Input and output are in-memory data structures.

**Explanation:** The parser operates on a `&str` slice and returns a `Vec`. There is no file I/O within the parser function itself (file reading happens at the call site). This is the cleanest possible unit-testable surface.

**Observable Behavior:**
- No terminal output, no file system effects, no database effects, no network interactions.
- All observable behavior is captured in the return value: `Vec<ParsedStatement>` or an error type.

**External Dependencies:**
- None (pure logic, no external dependencies)

**Validation Challenges:**

1. **Comment stripping behavior change**: The existing tests expect comments to be PRESERVED in the output SQL (e.g., `test_parse_preserves_comments` and `test_parse_multiline_comment`). The new spec (REQ-PARSE-009) requires line comments to be STRIPPED. This is a deliberate spec-driven behavior change. Existing tests that assert comment preservation will need to be updated as part of the rewrite.
2. **Line number accuracy with new lexer**: The new lexer tracks line numbers character-by-character. Line numbers must remain accurate when comments, quoted strings, and blank lines precede a statement. The existing `test_parse_multiline_statements` and `test_line_tracking_accuracy` tests will validate this.
3. **Error return type**: REQ-PARSE-007 and REQ-PARSE-013 require unterminated-string and unterminated-block-comment errors. The current `parse_statements()` signature returns `Vec<ParsedStatement>` with no error path. The architect may need to change the signature to `Result<Vec<ParsedStatement>, ParseError>`. Test cases must accommodate whichever signature the architect designs.

**Critical Behaviors to Validate:**
1. "A semicolon terminates the current statement if and only if it appears outside of a single-quoted string literal, a line comment, or a block comment." (REQ-PARSE-001) - This is the core of bug #28.
2. "A doubled single-quote (`''`) inside a quoted string ... does not end the string." (REQ-PARSE-006) - The escaped-quote edge case.
3. "A line comment is stripped from the statement text before it is sent to the database." (REQ-PARSE-009) - New behavior, different from old parser. This is the core of bug #30.
4. "A block comment may extend across any number of lines." (REQ-PARSE-011) - The multi-line block comment spanning statements.
5. "The parser must maintain an accurate line number counter throughout the entire input... does not reset between statements." (REQ-PARSE-014) - Critical for error reporting.

#### 2. Test Strategy Derivation

**Decision Tree Results:**

- "Interactive PTY" NOT checked - no REPL or terminal involved
- "CLI Batch" NOT checked at the parser level - the parser is a library function
- "Pure Logic" checked - the parser is a deterministic function of its input
- "Database connection" NOT checked - no database
- "File system access" NOT checked - parser operates on strings only
- "None" external dependencies - pure logic

**Derived Test Types:**

**Test Type 1: Unit Tests (in `src/sql/parser.rs` `#[cfg(test)]`)**
- **Validates:** All REQ-PARSE-001 through REQ-PARSE-017, all AC-1 through AC-12
- **Approach:** Call `parse_statements()` directly with carefully crafted SQL strings; assert on the returned `Vec<ParsedStatement>` fields (count, `.sql`, `.statement_number`, `.start_line`); for error cases, assert the error type and line number reported.
- **Rationale:** The parser is pure logic. Unit tests are the definitive, most direct, and fastest way to validate all parsing rules. Every permutation of the lexer's state machine can be tested in isolation.
- **Gap if missing:** No validation at all - cannot ship the fix without unit tests.
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (CLI `--file` flag)**
- **Validates:** AC-1, AC-3, AC-6 - that the parser is correctly wired into the `--file` execution path and the results reach the database executor
- **Approach:** Write a SQL file to disk, run `tq query --file <file>` via `Command::new()` process execution, capture stdout/stderr, assert correct number of statements were executed and that no parsing errors appear.
- **Rationale:** Unit tests validate the parser function in isolation. Integration tests validate that the `--file` path correctly calls `parse_statements()` and handles the result. The wiring could be broken even if the parser function is correct.
- **Gap if missing:** Could miss a regression where `--file` bypasses the new parser.
- **Necessity:** REQUIRED for wiring validation - but BLOCKED if no database is available, since `tq` requires a live connection for actual execution. However, the file-not-found error path and parse error paths can be tested without a database.
- **Note:** Database-dependent integration tests will be marked `#[ignore]`.

**Test Type 3: Interactive Tests (PTY/expectrl)**
- **Validates:** N/A - the parser does not affect REPL behavior (REPL uses its own multi-line input mechanism per the sprint planning exclusions)
- **Approach:** Not applicable
- **Gap if missing:** N/A
- **Necessity:** NOT NEEDED - sprint plan explicitly states "REPL changes out of scope; REPL already handles multi-line correctly via validator"

**Test Type 4: Performance/Benchmark Tests**
- **Validates:** Parser speed
- **Approach:** criterion benchmarks
- **Gap if missing:** Could miss performance regression on large SQL files
- **Necessity:** NOT NEEDED - no performance requirement specified; character-by-character scan is O(n) and suitable for typical SQL files

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (parser module) | REQUIRED | Validates all 17 REQ-PARSE rules directly | No validation of parser correctness | MUST IMPLEMENT |
| CLI integration tests (--file flag, no-DB path) | REQUIRED | Validates parser-to-CLI wiring for error paths | Wiring bugs undetected | MUST IMPLEMENT |
| CLI integration tests (--file flag, live-DB path) | RECOMMENDED | End-to-end validation with actual SQL execution | Undetected query assembly errors | IMPLEMENT as `#[ignore]` |
| Interactive tests (expectrl) | NOT NEEDED | REPL not in scope for this sprint | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirements stated | N/A | SKIP |

**Summary:**
- REQUIRED test types: 2 (unit + CLI no-DB integration)
- RECOMMENDED: 1 (CLI live-DB integration, as `#[ignore]`)
- NOT NEEDED: 2 (interactive, benchmark)

#### 4. Specification Coverage Map

| Requirement | Requirement Text (abbreviated) | Test Type(s) | Test Cases |
|-------------|-------------------------------|--------------|------------|
| REQ-PARSE-001 / AC-1 | Semicolons outside quoted strings/comments are terminators | Unit | TC-042-001 |
| REQ-PARSE-002 | Trailing content without final `;` is implicit last statement | Unit | TC-042-004 |
| REQ-PARSE-003 / AC-7 | Empty/whitespace/comment-only statements are discarded | Unit | TC-042-003, TC-042-004 |
| REQ-PARSE-004 / AC-3 | Newlines and whitespace within statements are preserved | Unit | TC-042-002 |
| REQ-PARSE-005 / AC-1 | Single-quoted strings: semicolons and comment markers inside are not special | Unit | TC-042-001 |
| REQ-PARSE-006 / AC-2 | `''` inside string does not end the string | Unit | TC-042-001 |
| REQ-PARSE-007 | Unterminated string literal produces error with correct line number | Unit | TC-042-001 |
| REQ-PARSE-008 / AC-4 | `--` outside strings/block-comments begins line comment to EOL | Unit | TC-042-003 |
| REQ-PARSE-009 / AC-4, AC-6 | Line comment content is stripped from statement text | Unit | TC-042-003 |
| REQ-PARSE-010 / AC-5 | `/*` outside strings/line-comments begins block comment | Unit | TC-042-003 |
| REQ-PARSE-011 / AC-5 | Block comments may span multiple lines | Unit | TC-042-003 |
| REQ-PARSE-012 | Nested block comments not supported; first `*/` closes | Unit | TC-042-003 |
| REQ-PARSE-013 | Unterminated block comment produces error with correct line number | Unit | TC-042-003 |
| REQ-PARSE-014 / AC-8 | Line number counter increments on `\n`; never resets | Unit | TC-042-002, TC-042-003 |
| REQ-PARSE-015 / AC-8 | `start_line` records line where statement accumulation began | Unit | TC-042-002, TC-042-003 |
| REQ-PARSE-016 | DB error messages include `start_line` | Integration (wiring) | TC-042-005 |
| REQ-PARSE-017 | Parse error messages reference line where construct began | Unit | TC-042-001, TC-042-003 |
| AC-6 | Comments between statements do not contaminate adjacent statements | Unit + Integration | TC-042-003, TC-042-004 |
| AC-9 | `has_multiple_statements()` works correctly with new parser | Unit | TC-042-004 |
| AC-10 | All existing parser tests pass | Unit (regression) | TC-042-005 (run all tests) |
| AC-11 | New tests cover all 3 bug scenarios | Unit | TC-042-001 (#28), TC-042-002 (#29), TC-042-003 (#30) |
| AC-12 | `ParsedStatement` struct unchanged (API compatible) | Unit (compile) | TC-042-005 |
| AC-13 | `test_repl_startup_and_quit` marked `#[ignore]` | Cargo test run | TC-042-005 |
| AC-14 | `cross-rs` version pinned in release.yml | Code inspection | TC-042-005 |
| AC-15 | `TMPDIR` renamed to `TQ_TMPDIR` in install.sh | Code inspection | TC-042-005 |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements

**Coverage Gap: REQ-PARSE-016 (DB error with start_line)**

REQ-PARSE-016 specifies that database error messages include the `start_line` field. This is validated at the error formatting layer (where the `ParsedStatement` struct is used to build the error message), not inside the parser itself. The unit tests validate that `start_line` is accurate. The wiring from `start_line` to the error message formatter is validated by inspecting the error handler in `src/commands/query/mod.rs` (or equivalent). A live-DB integration test could verify this end-to-end but is blocked without a database. This gap is LOW risk - the `start_line` field exists and is accurate; the error formatting is straightforward.

#### 5. Gap Analysis

**Interactive/E2E Tests With Live Database**
- **Reason for omission:** No live Teradata database in the local development environment
- **What won't be validated:** AC-1 through AC-6 in actual end-to-end execution where SQL is sent to Teradata
- **Risk assessment:** LOW - the parser's correctness is fully verifiable without a live DB; the integration is a simple pass-through
- **Mitigation:** Unit tests cover 100% of the parser logic; integration tests verify CLI wiring for error paths; live-DB tests are marked `#[ignore]` for when a database is available
- **Revisit criteria:** When a test database is available, run `cargo test -- --ignored`

**REQ-PARSE-007 and REQ-PARSE-013 (Parse Error Handling)**
- **Note:** The current `parse_statements()` signature returns `Vec<ParsedStatement>` with no error path. The architect must extend the API (likely `Result<Vec<ParsedStatement>, ParseError>`) to satisfy REQ-PARSE-007 and REQ-PARSE-013. Test cases TC-042-001 and TC-042-003 include error scenarios; the exact assertion form depends on the API chosen by the architect.
- **Risk assessment:** LOW - the test cases document both the expected behavior and the assertion pattern; the architect controls the API shape

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/sql/parser.rs` `#[cfg(test)]` module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 35-40 unit tests across TC-042-001 through TC-042-004
- **Key scenarios to cover:**
  1. Semicolons inside `'single-quoted strings'` do NOT split (bug #28)
  2. Escaped quotes `''` inside strings stay in-string
  3. Multi-line statements spanning multiple lines (bug #29)
  4. Line comments `--` stripped from output; semicolons in them ignored
  5. Block comments `/* */` stripped; semicolons in them ignored; multi-line blocks
  6. Comments between statements produce clean output (bug #30)
  7. Mixed scenarios combining all three features
  8. Line number accuracy through all contexts
  9. Error cases: unterminated strings and block comments
  10. Regression: all 17 existing named tests still pass
- **Mocking strategy:** No mocking needed - pure function

**Test Type: CLI Integration Tests (no-DB path)**
- **Location:** `tests/integration_tests.rs` (new section)
- **Framework:** `std::process::Command` + `std::fs::write`
- **Test count estimate:** 3-5 tests
- **Key scenarios to cover:**
  1. `tq query --file <file>` with a parse error (unterminated string) - should print error without requiring DB
  2. `tq query --file nonexistent.sql` - file not found error (existing behavior)
  3. Variable substitution with `--file` (if applicable - regression check)
- **Setup requirements:** Temp dir via `std::env::temp_dir()`, no database

**Test Type: CLI Integration Tests (live-DB, `#[ignore]`)**
- **Location:** `tests/integration_tests.rs` (new section, all `#[ignore]`)
- **Framework:** `std::process::Command` + `TQ_LOGON` env var from `.env`
- **Test count estimate:** 3 tests
- **Key scenarios to cover:**
  1. SQL file with semicolons inside quoted strings executes correctly (bug #28 end-to-end)
  2. SQL file with multi-line statements executes correctly (bug #29 end-to-end)
  3. SQL file with comment blocks between statements executes correctly (bug #30 end-to-end)
- **Setup requirements:** Live Teradata database, `TQ_LOGON` environment variable

#### 7. Coverage Sufficiency Assessment

**If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

- Unit tests validate: All 17 REQ-PARSE requirements, all parser state transitions, all edge cases, line number accuracy, error cases, API compatibility
- CLI integration tests (no-DB) validate: Parser wiring into the `--file` execution path, error propagation to the user
- Combined coverage: Comprehensive for parser correctness. Adequate for the `--file` wiring. Incomplete for live-DB end-to-end (accepted gap, low risk, covered by `#[ignore]` tests when DB available)

**Acceptance criteria:**
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified" (with noted DB gap)
- [x] Known gaps are documented and accepted

---

### Feature 2: Sprint 41 Remediation

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-42-planning.md` - AC-13 through AC-15

**Requirements:**

1. AC-13: `test_repl_startup_and_quit` marked as `#[ignore]` in `src/commands/repl/mod.rs`
2. AC-14: `cross-rs` version pinned in `.github/workflows/release.yml`
3. AC-15: `TMPDIR` renamed to `TQ_TMPDIR` in `install.sh`

**Feature Characteristics:**

**User Interaction Type:**
- Pure code/configuration changes - no new user interaction
- AC-13 affects test execution behavior (test no longer runs by default)
- AC-14 is a CI YAML change - not locally executable
- AC-15 is a shell script change verifiable via shellcheck and code review

**External Dependencies:**
- None for AC-13 (test annotation, verified by running `cargo test`)
- GitHub Actions runners for AC-14 at runtime (local: code review only)
- `shellcheck` for full AC-15 validation (POSIX compliance)

#### 2. Test Strategy Derivation

**Test Type 1: `cargo test` (regression suite)**
- **Validates:** AC-13 - test is marked `#[ignore]` and no longer causes failures in default test run
- **Approach:** Run `cargo test` without `--ignored` and verify `test_repl_startup_and_quit` is NOT in the default test output (it appears as `ignored` not `failed`)
- **Necessity:** REQUIRED

**Test Type 2: Code inspection**
- **Validates:** AC-13 (correct annotation), AC-14 (version pinning), AC-15 (TMPDIR rename)
- **Approach:** Grep for `#[ignore]` on the test function, grep for `cross-rs` version pin in YAML, grep for `TQ_TMPDIR` in install.sh
- **Necessity:** REQUIRED

#### 3. Specification Coverage Map

| Requirement | Test Type | Status |
|-------------|-----------|--------|
| AC-13: `test_repl_startup_and_quit` as `#[ignore]` | `cargo test` + code inspection | LOCALLY TESTABLE |
| AC-14: `cross-rs` version pinned | Code inspection of release.yml | LOCALLY TESTABLE (structure only) |
| AC-15: `TQ_TMPDIR` in install.sh | Code inspection + shellcheck | LOCALLY TESTABLE |

---

## Consolidated Test Implementation Plan

### Test Suite 1: Parser Unit Tests

**Location:** `src/sql/parser.rs` `#[cfg(test)]` module
**Command:** `cargo test --lib sql::parser`
**Tests:** All tests in TC-042-001, TC-042-002, TC-042-003, TC-042-004

| Test ID | Test Name | Validates | Bug |
|---------|-----------|-----------|-----|
| TS42-U01 | `test_semicolon_in_single_quoted_string` | AC-1, REQ-PARSE-001, REQ-PARSE-005 | #28 |
| TS42-U02 | `test_multiple_semicolons_in_string` | AC-1, REQ-PARSE-001 | #28 |
| TS42-U03 | `test_escaped_quote_in_string` | AC-2, REQ-PARSE-006 | #28 |
| TS42-U04 | `test_string_with_escaped_quote_and_semicolon` | AC-1, AC-2 | #28 |
| TS42-U05 | `test_insert_with_semicolon_in_value` | AC-1 (realistic SQL) | #28 |
| TS42-U06 | `test_unterminated_string_error` | REQ-PARSE-007 | #28 |
| TS42-U07 | `test_multiline_select_single_statement` | AC-3, REQ-PARSE-004 | #29 |
| TS42-U08 | `test_multiline_insert_single_statement` | AC-3, REQ-PARSE-004 | #29 |
| TS42-U09 | `test_multiline_with_blank_lines_in_statement` | AC-3, AC-7 | #29 |
| TS42-U10 | `test_line_number_preserved_for_multiline_statement` | AC-8, REQ-PARSE-015 | #29 |
| TS42-U11 | `test_line_comment_stripped` | AC-4, REQ-PARSE-009 | #30 |
| TS42-U12 | `test_line_comment_semicolon_not_terminator` | REQ-PARSE-008 | #30 |
| TS42-U13 | `test_block_comment_stripped` | AC-5, REQ-PARSE-010 | #30 |
| TS42-U14 | `test_block_comment_multiline` | REQ-PARSE-011 | #30 |
| TS42-U15 | `test_block_comment_semicolon_not_terminator` | REQ-PARSE-010 | #30 |
| TS42-U16 | `test_block_comment_no_nesting` | REQ-PARSE-012 | #30 |
| TS42-U17 | `test_unterminated_block_comment_error` | REQ-PARSE-013 | #30 |
| TS42-U18 | `test_comment_between_statements_no_contamination` | AC-6 | #30 |
| TS42-U19 | `test_line_comment_only_between_statements` | AC-6, REQ-PARSE-003 | #30 |
| TS42-U20 | `test_block_comment_only_between_statements` | AC-6, REQ-PARSE-003 | #30 |
| TS42-U21 | `test_combined_all_three_bugs` | AC-1, AC-3, AC-4, AC-5, AC-6 | #28+#29+#30 |
| TS42-U22 | `test_has_multiple_statements_with_comments` | AC-9 | #30 |
| TS42-U23 | `test_has_multiple_statements_with_quoted_semicolons` | AC-9 | #28 |
| TS42-U24 | `test_line_numbers_through_comments` | AC-8, REQ-PARSE-014 | #30 |
| TS42-U25 | `test_line_numbers_never_reset_between_statements` | REQ-PARSE-014 | all |
| TS42-U26 | `test_trailing_content_no_semicolon` | REQ-PARSE-002 | regression |
| TS42-U27..N | All 17 existing named tests still pass | AC-10 (regression) | regression |

### Test Suite 2: CLI Integration Tests (no-DB)

**Location:** `tests/integration_tests.rs`
**Command:** `cargo test --test integration_tests`

| Test ID | Test Name | Validates |
|---------|-----------|-----------|
| TS42-I01 | `test_file_with_parse_error_reports_line_number` | REQ-PARSE-017 wiring |
| TS42-I02 | `test_file_not_found_error` | Existing file-error path (regression) |

### Test Suite 3: CLI Integration Tests (live-DB, `#[ignore]`)

**Location:** `tests/integration_tests.rs`
**Command:** `cargo test --test integration_tests -- --ignored`
**Prerequisites:** `TQ_LOGON` set

| Test ID | Test Name | Validates |
|---------|-----------|-----------|
| TS42-D01 | `test_file_semicolon_in_string_executes_correctly` | AC-1 end-to-end |
| TS42-D02 | `test_file_multiline_sql_executes_correctly` | AC-3 end-to-end |
| TS42-D03 | `test_file_comments_between_statements_executes_correctly` | AC-6 end-to-end |

### Test Suite 4: Regression - Full Test Suite

**Command:** `cargo test`
**Validates:** AC-10 (all existing tests pass), AC-13 (repl test now ignored)

| Test ID | Command | Expected Result |
|---------|---------|-----------------|
| TS42-R01 | `cargo test` | 100% pass rate, zero failures |
| TS42-R02 | `cargo test` | `test_repl_startup_and_quit` appears as `ignored`, not `failed` |
| TS42-R03 | `cargo clippy -- -D warnings` | Zero warnings |

### Test Suite 5: Sprint 41 Remediation - Code Inspection

| Test ID | Check | Validates | Expected Result |
|---------|-------|-----------|-----------------|
| TS42-C01 | `#[ignore]` annotation on `test_repl_startup_and_quit` | AC-13 | Present |
| TS42-C02 | Pinned version for `cross-rs` in `release.yml` | AC-14 | Version pin present (e.g., `v0.2.5`) |
| TS42-C03 | `TQ_TMPDIR` in `install.sh` (not bare `TMPDIR`) | AC-15 | `TQ_TMPDIR` present; bare `TMPDIR` absent |

---

## Strategy Summary

**Total Features Analyzed:** 2 (Feature 1: Parser Rewrite, Feature 2: Sprint 41 Remediation)

**Test Types Required:**
- Unit tests: REQUIRED for Feature 1 (parser pure logic)
- CLI integration tests (no-DB): REQUIRED for Feature 1 wiring validation
- CLI integration tests (live-DB, `#[ignore]`): RECOMMENDED for Feature 1 end-to-end
- Cargo test regression suite: REQUIRED for Feature 2 (AC-13) and Feature 1 (AC-10)
- Code inspection: REQUIRED for Feature 2 (AC-13, AC-14, AC-15)
- Interactive tests: NOT NEEDED
- Benchmark tests: NOT NEEDED

**Estimated Test Count:**
- Unit: ~27-35 tests (including existing regression tests)
- CLI integration (no-DB): 2 tests
- CLI integration (live-DB, ignored): 3 tests
- Code inspection checks: 3 items
- Full regression: 1 run (all existing tests)
- Total: ~36-44 new/updated tests + full regression suite

**Risk Assessment:**
- HIGH risk gaps: None - the parser is pure logic, fully testable without external dependencies
- MEDIUM risk gaps: Live-DB end-to-end validation (covered by `#[ignore]` tests when DB available); parse error API shape depends on architect decision
- LOW risk gaps: REQ-PARSE-016 (DB error line number in output - wired through existing error formatter; `start_line` accuracy validated by unit tests)

**Dependencies Required:**
- Live database: NO for unit tests; YES for `#[ignore]` integration tests
- Network access: NO
- Specific OS: NO
- Other: None

**Key Risk: Comment Stripping Behavior Change**

The existing parser tests `test_parse_preserves_comments` and `test_parse_multiline_comment` assert that comments are INCLUDED in the output SQL. The new spec (REQ-PARSE-009) requires line comments to be STRIPPED. This is an intentional behavior change to fix bug #30.

Expected resolution: The architect will update these two tests to assert that comment-stripped SQL is passed through. The quality-validator test cases (TC-042-003) document the new expected behavior. If the architect decides to keep comments in the SQL (relying on Teradata to handle them), the test cases note both possibilities and will be adjusted accordingly.

**Key Risk: Parse Error API Shape**

REQ-PARSE-007 and REQ-PARSE-013 require error reporting for unterminated strings/comments. The current `parse_statements()` returns `Vec<ParsedStatement>` with no error path. The new API must be `Result<Vec<ParsedStatement>, ParseError>` (or similar). Test cases document this requirement but use a placeholder `// assert error is returned` pattern. The architect defines the exact error type.

---

## Strategy Validation Checklist

- [x] Every feature has complete specification analysis section
- [x] Feature characteristics are classified (not assumed)
- [x] Test strategy is derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest
- [x] Specification coverage map includes all 17 REQ-PARSE requirements plus all ACs
- [x] Every requirement maps to at least one test type
- [x] Test implementation plan is detailed and actionable
- [x] Coverage sufficiency is assessed
- [x] Known gaps are documented and accepted (DB-dependent tests marked `#[ignore]`)
- [x] Comment-stripping behavior change risk documented
- [x] Parse error API shape risk documented

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-03-21
**Review Status:** DRAFT

**Known Blockers / Risks:**
1. Parse error API: `parse_statements()` must return a `Result` for REQ-PARSE-007 and REQ-PARSE-013. Awaiting architect's API design decision.
2. Comment-stripping behavior: Two existing tests (`test_parse_preserves_comments`, `test_parse_multiline_comment`) will need updating to match new spec (REQ-PARSE-009). Architect to update these as part of the rewrite.
3. Live-DB tests (TS42-D01 to TS42-D03) are `#[ignore]` - blocked without database. Not a blocker for the APPROVED verdict.
