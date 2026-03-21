# Sprint 43 Test Strategy: Profile Management & Parser Polish

**Created:** 2026-03-21
**Author:** quality-validator
**Sprint:** Sprint 43
**Features:**
1. Profile Management Commands (`tq profile add/edit/delete/list`)
2. Sprint 42 Parser Remediation (`Result` return type, missing test)

---

## Overview

Sprint 43 delivers two distinct feature clusters with very different test profiles:

- **Feature 1 (Profile Management)** is a CLI batch feature with significant file system side effects. It manipulates `~/.tq/config.toml` - a TOML file shared with existing functionality. The primary risk is config file corruption: writing must preserve unrelated content. No database is required.
- **Feature 2 (Parser Remediation)** is a pure-logic change to a library function that affects only its return type and adds one missing test. This is the cleanest class of change - no external dependencies, fully verifiable by unit tests.

Neither feature requires a live database or interactive PTY tests.

---

## Feature-by-Feature Test Strategy

---

### Feature 1: Profile Management Commands (AC-1 through AC-12)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-43-planning.md` - AC-1 through AC-12
- Secondary: `docs/specifications/configuration.md` - Profile structure, TOML format, logmech values

**Requirements (from acceptance criteria):**

1. AC-1: `tq profile add <name> --host <host> [--port <port>] [--database <db>] [--user <user>] [--logmech <mech>] [--password-file <path>]` creates a new profile
2. AC-2: `tq profile edit <name> [--host] [--port] [--database] [--user] [--logmech] [--password-file]` updates specified fields only
3. AC-3: `tq profile delete <name>` removes a profile (with `--force` to skip confirmation prompt)
4. AC-4: Error if profile already exists on `add` (unless `--force` to overwrite)
5. AC-5: Error if profile doesn't exist on `edit` or `delete`
6. AC-6: Creates `~/.tq/config.toml` and directory if they don't exist on `add`
7. AC-7: Preserves existing config content (other profiles, defaults section) on add/edit/delete
8. AC-8: Output confirms action taken (e.g., "Profile 'dev' added successfully")
9. AC-9: Tab completion for profile names on edit/delete (CLI only, not REPL)
10. AC-10: `tq profile list` as alias for existing `tq profiles` command
11. AC-11: Validates logmech values (TD2, LDAP, KRB5, TDNEGO)
12. AC-12: Validates port is a valid number (1-65535)

**Feature Characteristics:**

**User Interaction Type:**
- CLI Batch (scripted, non-interactive command execution)

**Explanation:** `tq profile add/edit/delete/list` are one-shot CLI commands. They receive all input via flags (non-interactive by design - AC-3 mentions `--force` to skip confirmation, implying the default may prompt, but the overall design is scriptable/non-interactive). The primary mechanism is flag-based.

**Observable Behavior:**
- File system side effects: `~/.tq/config.toml` created/modified/deleted
- Structured text output: confirmation messages on success (AC-8)
- Exit codes: non-zero on error

**External Dependencies:**
- File system access (reads/writes `~/.tq/config.toml`)
- No database connection required
- No network access
- No PTY/terminal required

**Validation Challenges:**

1. **Config file isolation**: Tests must not corrupt the developer's real `~/.tq/config.toml`. Tests must use a temporary directory and point the tool to it via an environment variable or a test-only config path override.
2. **TOML preservation**: Verifying that `add`, `edit`, and `delete` preserve the rest of the config file requires round-trip assertion: write a known config, execute the command, parse the resulting TOML, assert all untouched sections survived.
3. **`--force` on delete**: AC-3 implies that `tq profile delete <name>` without `--force` prompts for confirmation. Testing the confirmation prompt requires interactive input. This is a risk area - if `--force` is always required in non-interactive mode, testing is straightforward; if a TTY check is performed, the test must handle both paths.
4. **Tab completion (AC-9)**: Shell tab completion registration is a build-artifact/shell integration concern, not directly unit-testable at runtime. This AC is validated by code inspection (is completion registered for profile names?) rather than a runtime test.

**Critical Behaviors to Validate:**

1. "Preserves existing config content (other profiles, defaults section) on add/edit/delete" (AC-7) - This is the highest-risk behavior. TOML write operations that lose existing data are silent data corruption bugs.
2. "Creates `~/.tq/config.toml` and directory if they don't exist" (AC-6) - The fresh-install path must be tested explicitly.
3. "Error if profile already exists on `add` (unless `--force` to overwrite)" (AC-4) - Guard against silent overwrites.
4. "Validates logmech values (TD2, LDAP, KRB5, TDNEGO)" (AC-11) - Case sensitivity must be tested (is "td2" accepted?).
5. "Validates port is a valid number (1-65535)" (AC-12) - Boundary values (0, 1, 65535, 65536) must all be tested.

#### 2. Test Strategy Derivation

**Decision Tree Results:**

- "Interactive PTY" NOT checked - commands are flag-based, non-interactive
- "CLI Batch" checked - these are one-shot CLI commands
- "File system access" checked - `~/.tq/config.toml` is the primary side effect
- "Database connection" NOT checked - no database needed for profile CRUD
- "Visual output in terminal" NOT checked at the level needing PTY - confirmation messages are plain text captured via stdout

**Derived Test Types:**

**Test Type 1: Unit Tests (internal logic)**
- **Validates:** TOML serialization/deserialization logic, validation functions (logmech enum, port range), profile merge logic
- **Approach:** Call internal functions directly with crafted inputs; assert return values and error types
- **Rationale:** The TOML preservation logic and validation functions are pure/near-pure and testable in isolation without spawning a process
- **Gap if missing:** Validation edge cases (port=0, port=65536, unknown logmech) not caught until integration test runtime; harder to debug
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (CLI process execution)**
- **Validates:** All AC-1 through AC-12 observable behaviors - the actual commands as the user would run them
- **Approach:** Spawn `tq profile add/edit/delete/list` as child processes using `std::process::Command`. Use `TMPDIR`-scoped fake home or override config path. Read the resulting TOML file to assert structural correctness. Capture stdout/stderr for confirmation messages and error messages.
- **Rationale:** Unit tests cannot validate the full CLI wiring (argument parsing, command dispatch, file path resolution, error message formatting). Integration tests prove the end-to-end path works. File system side effects (AC-6, AC-7) require reading the file after the command runs.
- **Gap if missing:** Argument parsing bugs, path resolution errors, and file creation races would not be caught
- **Necessity:** REQUIRED

**Test Type 3: Interactive Tests (PTY/expectrl)**
- **Validates:** `tq profile delete <name>` confirmation prompt behavior (if it exists without `--force`)
- **Approach:** Spawn the REPL in a PTY, send `n`/`y` to the confirmation prompt, verify appropriate behavior
- **Rationale:** If delete prompts for confirmation without `--force`, the prompt is interactive and cannot be tested via `std::process::Command` (stdin is not a TTY)
- **Gap if missing:** Delete confirmation UX not validated
- **Necessity:** CONDITIONALLY REQUIRED - only if delete without `--force` shows a TTY prompt. If the implementation always requires `--force` (pure non-interactive), this is NOT NEEDED. The architect's design determines this.
- **Decision:** Design OPTIONAL interactive test for confirmation prompt; wait for architect's API decision. If `--force` is required (no interactive prompt in non-TTY mode), skip interactive test.

**Test Type 4: Code Inspection**
- **Validates:** AC-9 (tab completion for profile names registered in shell completion logic)
- **Approach:** Grep for profile name completion registration in `src/cli.rs` or completion generator
- **Rationale:** Runtime tests cannot verify shell completion - it's a build artifact validated by code structure
- **Necessity:** REQUIRED (code inspection, not a cargo test)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (validation logic) | REQUIRED | Validates port/logmech validation, pure logic | Edge case validation bugs | MUST IMPLEMENT |
| Integration tests (CLI process) | REQUIRED | Validates all AC behaviors end-to-end, file system effects | CLI wiring bugs, TOML write bugs | MUST IMPLEMENT |
| Interactive tests (confirmation prompt) | CONDITIONAL | Only if delete prompts interactively without `--force` | Confirmation UX broken | IMPLEMENT IF NEEDED |
| Code inspection (tab completion) | REQUIRED | AC-9 not runtime-testable | Completion silently broken | INSPECT SOURCE |
| Benchmark tests | NOT NEEDED | No performance requirements for profile CRUD | N/A | SKIP |

**Summary:**
- REQUIRED test types: 3 (unit, integration, code inspection)
- CONDITIONAL: 1 (interactive confirmation prompt)
- NOT NEEDED: 1 (benchmark)

#### 4. Specification Coverage Map

| Requirement | Requirement Text (abbreviated) | Test Type(s) | Test Cases |
|-------------|-------------------------------|--------------|------------|
| AC-1 | `tq profile add <name> --host <host> [opts]` creates new profile | Integration | TC-043-001 |
| AC-2 | `tq profile edit <name> [opts]` updates specified fields only | Integration | TC-043-002 |
| AC-3 | `tq profile delete <name>` removes profile (`--force` skips prompt) | Integration | TC-043-003 |
| AC-4 | Error if profile exists on `add` (unless `--force`) | Integration | TC-043-001 |
| AC-5 | Error if profile not found on `edit` or `delete` | Integration | TC-043-002, TC-043-003 |
| AC-6 | Creates `~/.tq/` and `config.toml` if they don't exist | Integration | TC-043-001 |
| AC-7 | Preserves existing config content on add/edit/delete | Integration | TC-043-001, TC-043-002, TC-043-003 |
| AC-8 | Output confirms action taken | Integration | TC-043-001, TC-043-002, TC-043-003 |
| AC-9 | Tab completion for profile names on edit/delete | Code inspection | TC-043-004 |
| AC-10 | `tq profile list` is alias for `tq profiles` | Integration | TC-043-004 |
| AC-11 | Validates logmech (TD2, LDAP, KRB5, TDNEGO) | Unit + Integration | TC-043-005, TC-043-001 |
| AC-12 | Validates port 1-65535 | Unit + Integration | TC-043-005, TC-043-001 |

**Coverage Validation:**
- [x] Every acceptance criterion appears in the table
- [x] Every criterion maps to at least one test type
- [x] Every test type is justified by criterion
- [x] No orphaned requirements

**Coverage Gap: AC-9 (Tab Completion)**

Tab completion correctness cannot be verified via `cargo test` alone. The test (TC-043-004) is a source code inspection: check that `complete_profile_names` (or equivalent) is registered as the completer for the `edit` and `delete` subcommands in `src/cli.rs`. This is a structural check, not a runtime validation. Risk is LOW - completion failures are cosmetic and immediately visible to users.

**Coverage Gap: AC-3 Confirmation Prompt**

The behavior of `tq profile delete <name>` without `--force` in a non-TTY environment (how CI runs) is architecture-dependent. If the tool silently fails or shows an error in non-TTY mode, integration tests can cover this. If it requires a TTY prompt, an optional interactive test is needed. This gap will be resolved once the architect's implementation is known.

#### 5. Gap Analysis

**Interactive Confirmation Prompt Test**
- **Reason for omission (provisional):** Architecture unknown at strategy-design time. If delete requires `--force` in non-TTY (the recommended non-interactive design), no interactive test is needed.
- **What won't be validated:** Interactive confirmation UX if not `--force`-only
- **Risk assessment:** LOW - the sprint planning states "Non-interactive, flag-based" design; `--force` is the designated non-interactive path
- **Mitigation:** Document that manual verification of the confirmation prompt should be done once
- **Revisit:** If architect implements TTY prompt for delete without `--force`

**Live Database Tests**
- **Reason for omission:** Profile management requires no database connection; all operations are pure config file manipulation
- **Risk assessment:** N/A - database not a dependency

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/profile.rs` `#[cfg(test)]` module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 12-15 unit tests
- **Key scenarios to cover:**
  1. Valid logmech values: `TD2`, `LDAP`, `KRB5`, `TDNEGO` all accepted
  2. Invalid logmech values: `"td2"` (lowercase), `"KERBEROS"`, `""`, `"OTHER"` all rejected
  3. Valid port values: `1`, `1025`, `65535` all accepted
  4. Invalid port values: `0`, `65536`, `99999` all rejected
  5. Profile struct serializes to expected TOML structure
  6. Profile field merge logic: edit with partial flags updates only those fields
  7. Config preservation: read-modify-write cycle doesn't lose unrelated sections
- **Mocking strategy:** Use `tempfile` crate for temporary config directories; no database mocking needed

**Test Type: Integration Tests (CLI process)**
- **Location:** `tests/integration_tests.rs` (new `profile_management` section) or `tests/profile_tests.rs`
- **Framework:** `std::process::Command` + `tempfile` crate
- **Test count estimate:** 18-22 tests
- **Key scenarios to cover:**

  *Profile Add (TC-043-001):*
  1. Add profile to non-existent config: directory and file created, profile written correctly
  2. Add profile to existing config with `[defaults]` section: `[defaults]` preserved
  3. Add profile to existing config with other profiles: other profiles preserved
  4. Add profile with all fields specified: all fields written to TOML
  5. Add profile with only `--host` (minimum required): other fields absent from TOML
  6. Add duplicate profile: exits non-zero, error message mentions profile name
  7. Add duplicate profile with `--force`: profile overwritten, success message
  8. Add profile with invalid logmech: exits non-zero, error message lists valid values
  9. Add profile with invalid port (0): exits non-zero, error message indicates valid range
  10. Add profile with invalid port (65536): exits non-zero
  11. Add profile with valid port boundaries (1 and 65535): success

  *Profile Edit (TC-043-002):*
  12. Edit existing profile changing one field: only that field changes, others preserved
  13. Edit existing profile changing multiple fields: all specified fields change
  14. Edit non-existent profile: exits non-zero, error message mentions profile name
  15. Edit profile with invalid logmech: exits non-zero
  16. Edit profile with invalid port: exits non-zero

  *Profile Delete (TC-043-003):*
  17. Delete existing profile with `--force`: profile removed, other profiles preserved, success message
  18. Delete non-existent profile: exits non-zero, error message mentions profile name
  19. Delete last profile in config: config file remains valid (possibly with empty `[profiles]` section or just `[defaults]`)

  *Profile List (TC-043-004):*
  20. `tq profile list` with existing profiles: same output as `tq profiles`
  21. `tq profile list` with no config file: appropriate error or empty output

- **Setup requirements:**
  - `tempfile` crate for temp directories
  - Set config path override via environment variable or process argument (the architect must expose a mechanism for tests to override `~/.tq/config.toml` path - likely via `TQ_CONFIG_DIR` env var or `--config-dir` flag, or tests use `HOME` override)
  - No database, no `TQ_LOGON` needed

**Test Type: Code Inspection (AC-9)**
- **Approach:** Grep `src/cli.rs` for completer registration on `profile edit` and `profile delete` subcommands
- **Expected:** Profile name completer registered for `<name>` positional argument on `edit` and `delete`

#### 7. Coverage Sufficiency Assessment

**If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

- Unit tests validate: logmech/port validation logic, TOML serialization, field merge logic
- Integration tests validate: CLI dispatch, file creation/modification, error messages, exit codes, content preservation, all AC behaviors end-to-end
- Code inspection validates: tab completion registration (AC-9)
- Combined coverage: **Comprehensive for all testable behaviors**

**Gaps in combined coverage:**
- Interactive confirmation prompt (AC-3 non-force path) - LOW risk, sprint design is non-interactive
- Actual shell tab completion execution (AC-9) - LOW risk, structural check is sufficient

**Acceptance criteria:**
- [x] All 12 acceptance criteria have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified" with noted gaps
- [x] Known gaps are documented and accepted

---

### Feature 2: Sprint 42 Parser Remediation (AC-13 through AC-20)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-43-planning.md` - AC-13 through AC-20
- Secondary: `docs/sprints/sprint-42-review.md` - Remediation items #1-5
- Relevant prior strategy: `tests/strategy/sprint-42-test-strategy.md` - REQ-PARSE-007, REQ-PARSE-013

**Requirements:**

1. AC-13: `parse_statements()` returns `Result<Vec<ParsedStatement>, ParseError>` for unterminated strings (REQ-PARSE-007)
2. AC-14: `parse_statements()` returns `Result<Vec<ParsedStatement>, ParseError>` for unterminated block comments (REQ-PARSE-013)
3. AC-15: `ParseError` includes line number and column for error location
4. AC-16: All existing call sites updated for `Result` return type
5. AC-17: `test_comment_marker_inside_string_is_not_comment` test added
6. AC-18: REQ-PARSE-015 "begins accumulating" wording clarified in spec (not testable - spec-only change)
7. AC-19: Space-injection behavior documented in spec and design doc (not testable - documentation change)
8. AC-20: Explanatory comment added for `unwrap()` at parser.rs:178 (not testable - code comment)

**Testable ACs:** AC-13, AC-14, AC-15, AC-16, AC-17
**Non-testable ACs:** AC-18 (spec wording), AC-19 (documentation), AC-20 (code comment) - validated by code inspection only

**Feature Characteristics:**

**User Interaction Type:**
- Pure Logic - `parse_statements()` is a deterministic function of its string input; the remediation changes only its return type from `Vec<ParsedStatement>` to `Result<Vec<ParsedStatement>, ParseError>`

**Explanation:** This is a pure library function change. The return type change propagates to call sites (AC-16) but the core logic is unchanged - new behavior is only triggered on malformed SQL (unterminated string/comment), which was previously undefined behavior.

**Observable Behavior:**
- Return value change: `Result<Vec<ParsedStatement>, ParseError>` instead of `Vec<ParsedStatement>`
- `ParseError` struct carries line number and column
- One new test case verifies comment-marker-in-string behavior

**External Dependencies:**
- None (pure logic, no external dependencies)

**Validation Challenges:**

1. **API shape dependent on architect design**: AC-13/14/15 require `ParseError` to contain line and column. The exact struct shape (field names, column type) depends on the architect's implementation. Test cases assert on the observable behavior (error returned, line number correct, column correct) and must adapt to the actual struct layout.
2. **Column definition ambiguity**: "column" could mean byte offset, character offset, or 1-based column position within the line. Tests must verify the column is useful for error reporting, not just that a field exists.
3. **Call site compilation (AC-16)**: If all call sites are updated, `cargo build` succeeds. If any call site is missed, compilation fails. This is validated implicitly by `cargo test` (compilation is a prerequisite).

**Critical Behaviors to Validate:**

1. "Returns `Result<Vec<ParsedStatement>, ParseError>` for unterminated strings" (AC-13/REQ-PARSE-007) - Unterminated string must produce `Err`, not a partial result or panic.
2. "Returns `Result<Vec<ParsedStatement>, ParseError>` for unterminated block comments" (AC-14/REQ-PARSE-013) - Same for block comments.
3. "`ParseError` includes line number and column" (AC-15) - Error location must be accurate, not just present.
4. "`test_comment_marker_inside_string_is_not_comment`" (AC-17) - `--` inside a single-quoted string must NOT start a line comment.

#### 2. Test Strategy Derivation

**Decision Tree Results:**

- "Interactive PTY" NOT checked - parser is a library function
- "CLI Batch" NOT checked at parser level - pure function
- "Pure Logic" checked - deterministic function, no external dependencies
- "Database connection" NOT checked
- "File system access" NOT checked

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** AC-13, AC-14, AC-15, AC-17 - all testable behaviors
- **Approach:** Call `parse_statements()` with crafted malformed SQL inputs; assert `Err(e)` is returned with correct line number and column in `e`; add the specific `test_comment_marker_inside_string_is_not_comment` test
- **Rationale:** Pure function - unit tests are the definitive validation method
- **Gap if missing:** No validation of the critical error path change
- **Necessity:** REQUIRED

**Test Type 2: Compilation check (implicit)**
- **Validates:** AC-16 (all call sites updated for `Result` return type)
- **Approach:** `cargo build` succeeds - compilation verifies call sites handle `Result`
- **Rationale:** Mishandled call sites cause compile errors; if the build passes, all sites are updated
- **Gap if missing:** Would miss the issue before it manifests as a runtime crash
- **Necessity:** REQUIRED (implicit in `cargo test`)

**Test Type 3: Code Inspection (non-testable ACs)**
- **Validates:** AC-18 (spec wording), AC-19 (documentation), AC-20 (code comment)
- **Approach:** Read `docs/specifications/batch-mode.md` for REQ-PARSE-015 update, read `docs/design/batch-mode.md` for space-injection documentation, read `src/sql/parser.rs:178` for explanatory comment
- **Necessity:** REQUIRED for completeness (documentation verification)

**Test Type 4: Interactive Tests (PTY)**
- **Validates:** N/A - parser change does not affect REPL
- **Necessity:** NOT NEEDED

**Test Type 5: Integration Tests (CLI `--file` error propagation)**
- **Validates:** That parse errors from `parse_statements()` surface correctly to the user via the `--file` execution path
- **Approach:** Write a SQL file with an unterminated string, run `tq query --file <file>`, verify stderr contains the error message with line/column
- **Rationale:** Unit tests validate the parser function; this validates the error propagates through the CLI to the user
- **Gap if missing:** Error could be swallowed or reformatted incorrectly at the call site
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (parser error cases) | REQUIRED | Validates AC-13, AC-14, AC-15, AC-17 directly | Error path not validated | MUST IMPLEMENT |
| Compilation check (`cargo build`) | REQUIRED | Validates AC-16 (call sites updated) | Compile error if sites missed | IMPLICIT in test run |
| Integration tests (CLI error propagation) | REQUIRED | Validates parse errors reach user | Error swallowed at call site | MUST IMPLEMENT |
| Code inspection (AC-18, AC-19, AC-20) | REQUIRED | Documentation changes not auto-testable | Documentation gaps | INSPECT FILES |
| Interactive tests | NOT NEEDED | Parser is not REPL-facing | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirements added | N/A | SKIP |

**Summary:**
- REQUIRED test types: 3 (unit, integration, code inspection)
- NOT NEEDED: 2 (interactive, benchmark)

#### 4. Specification Coverage Map

| Requirement | Requirement Text (abbreviated) | Test Type(s) | Test Cases |
|-------------|-------------------------------|--------------|------------|
| AC-13 / REQ-PARSE-007 | `parse_statements()` returns `Err` for unterminated string | Unit + Integration | TC-043-006 |
| AC-14 / REQ-PARSE-013 | `parse_statements()` returns `Err` for unterminated block comment | Unit + Integration | TC-043-006 |
| AC-15 | `ParseError` includes line number and column | Unit | TC-043-006 |
| AC-16 | All call sites updated for `Result` return type | Compilation (`cargo build`) | Implicit in cargo test |
| AC-17 | `test_comment_marker_inside_string_is_not_comment` test added | Unit | TC-043-007 |
| AC-18 | REQ-PARSE-015 wording clarified in spec | Code inspection | TC-043-007 |
| AC-19 | Space-injection documented in spec and design | Code inspection | TC-043-007 |
| AC-20 | Explanatory comment at parser.rs:178 | Code inspection | TC-043-007 |

**Coverage Validation:**
- [x] Every acceptance criterion appears in the table
- [x] Every criterion maps to at least one test type
- [x] All test types justified
- [x] No orphaned requirements

#### 5. Gap Analysis

**Non-testable Documentation ACs (AC-18, AC-19, AC-20)**
- **Reason for omission:** Specification wording changes and code comments are not machine-testable
- **What won't be validated:** Correctness of the wording itself
- **Risk assessment:** LOW - these are documentation improvements with no behavioral impact
- **Mitigation:** Code inspection during test execution phase

#### 6. Test Implementation Plan

**Test Type: Unit Tests (parser error cases)**
- **Location:** `src/sql/parser.rs` `#[cfg(test)]` module (extending existing test module)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 8-10 new unit tests
- **Key scenarios to cover:**
  1. Unterminated string starting on line 1: `Err(ParseError)` with `line == 1`
  2. Unterminated string starting on line 3 (after two complete statements): `Err(ParseError)` with `line == 3`
  3. Unterminated string: column reflects position where string literal opened
  4. Unterminated block comment starting on line 1: `Err(ParseError)` with `line == 1`
  5. Unterminated block comment starting on line 5: `Err(ParseError)` with `line == 5`
  6. Unterminated block comment: column reflects position where `/*` opened
  7. Valid input still returns `Ok(vec)` (regression check - `Result` change didn't break happy path)
  8. `test_comment_marker_inside_string_is_not_comment`: SQL `'this -- is not a comment'` returns one statement with the string content intact, not split or truncated
  9. Comment marker after close quote still starts comment: `'string' -- real comment` strips the comment
  10. Nested single-quote with comment marker: `'it''s -- not a comment'` handled correctly
- **Mocking strategy:** No mocking needed - pure function

**Test Type: Integration Tests (CLI error propagation)**
- **Location:** `tests/integration_tests.rs`
- **Framework:** `std::process::Command` + temp file
- **Test count estimate:** 2-3 tests
- **Key scenarios to cover:**
  1. `tq query --file <file_with_unterminated_string>`: stderr contains error message with line number; exit code non-zero
  2. `tq query --file <file_with_unterminated_block_comment>`: stderr contains error message with line number; exit code non-zero
  3. `tq query --file <valid_file>`: still succeeds (regression check) - can run without DB if file not found behavior tested, or use `#[ignore]` for actual query execution

**Test Type: Code Inspection Checklist**
- Verify `docs/specifications/batch-mode.md` updated for REQ-PARSE-015 wording (AC-18)
- Verify `docs/design/batch-mode.md` or equivalent documents space-injection behavior (AC-19)
- Verify `src/sql/parser.rs` line ~178 has explanatory comment for the `unwrap()` (AC-20)

#### 7. Coverage Sufficiency Assessment

**If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

- Unit tests validate: Error return for unterminated strings/comments, accurate line/column in error, comment-marker-in-string behavior, no regression on happy path
- Integration tests validate: CLI surfaces parse errors to user (error wiring through call stack)
- Compilation validates: All call sites handle `Result`
- Combined coverage: **Comprehensive for all behavioral requirements**

**Acceptance criteria:**
- [x] All testable ACs (AC-13 through AC-17) have test coverage
- [x] All test types justified
- [x] Combined coverage sufficient
- [x] Non-testable ACs (AC-18, AC-19, AC-20) covered by code inspection

---

## Consolidated Test Implementation Plan

### Test Suite 1: Profile Management - Unit Tests

**Location:** `src/commands/profile.rs` `#[cfg(test)]` module
**Command:** `cargo test --lib commands::profile`

| Test ID | Test Name | Validates |
|---------|-----------|-----------|
| TS43-U01 | `test_validate_logmech_td2_accepted` | AC-11 |
| TS43-U02 | `test_validate_logmech_ldap_accepted` | AC-11 |
| TS43-U03 | `test_validate_logmech_krb5_accepted` | AC-11 |
| TS43-U04 | `test_validate_logmech_tdnego_accepted` | AC-11 |
| TS43-U05 | `test_validate_logmech_lowercase_rejected` | AC-11 |
| TS43-U06 | `test_validate_logmech_unknown_rejected` | AC-11 |
| TS43-U07 | `test_validate_port_min_boundary_accepted` | AC-12 (port=1) |
| TS43-U08 | `test_validate_port_max_boundary_accepted` | AC-12 (port=65535) |
| TS43-U09 | `test_validate_port_zero_rejected` | AC-12 (port=0) |
| TS43-U10 | `test_validate_port_above_max_rejected` | AC-12 (port=65536) |
| TS43-U11 | `test_toml_preservation_add_profile_keeps_defaults` | AC-7 |
| TS43-U12 | `test_toml_preservation_add_profile_keeps_other_profiles` | AC-7 |
| TS43-U13 | `test_toml_preservation_edit_profile_keeps_unedited_fields` | AC-2, AC-7 |
| TS43-U14 | `test_toml_preservation_delete_profile_keeps_other_profiles` | AC-7 |

### Test Suite 2: Profile Management - Integration Tests

**Location:** `tests/profile_tests.rs` (new file) or new section in `tests/integration_tests.rs`
**Command:** `cargo test --test profile_tests` (or integration_tests)
**Prerequisites:** `tempfile` crate available, binary built (`cargo build`)

| Test ID | Test Name | Validates |
|---------|-----------|-----------|
| TS43-I01 | `profile_add_creates_config_dir_and_file` | AC-6 |
| TS43-I02 | `profile_add_writes_correct_toml_fields` | AC-1 |
| TS43-I03 | `profile_add_with_all_optional_fields` | AC-1 |
| TS43-I04 | `profile_add_with_only_host_minimum` | AC-1 |
| TS43-I05 | `profile_add_prints_success_message` | AC-8 |
| TS43-I06 | `profile_add_preserves_defaults_section` | AC-7 |
| TS43-I07 | `profile_add_preserves_other_profiles` | AC-7 |
| TS43-I08 | `profile_add_duplicate_fails_without_force` | AC-4 |
| TS43-I09 | `profile_add_duplicate_succeeds_with_force` | AC-4 |
| TS43-I10 | `profile_add_invalid_logmech_fails` | AC-11 |
| TS43-I11 | `profile_add_port_zero_fails` | AC-12 |
| TS43-I12 | `profile_add_port_above_max_fails` | AC-12 |
| TS43-I13 | `profile_add_port_boundaries_succeed` | AC-12 (1, 65535) |
| TS43-I14 | `profile_edit_updates_specified_field_only` | AC-2 |
| TS43-I15 | `profile_edit_updates_multiple_fields` | AC-2 |
| TS43-I16 | `profile_edit_preserves_other_profiles` | AC-7 |
| TS43-I17 | `profile_edit_nonexistent_fails` | AC-5 |
| TS43-I18 | `profile_edit_prints_success_message` | AC-8 |
| TS43-I19 | `profile_delete_with_force_removes_profile` | AC-3 |
| TS43-I20 | `profile_delete_preserves_other_profiles` | AC-7 |
| TS43-I21 | `profile_delete_nonexistent_fails` | AC-5 |
| TS43-I22 | `profile_delete_prints_success_message` | AC-8 |
| TS43-I23 | `profile_list_alias_matches_profiles_output` | AC-10 |

### Test Suite 3: Parser Remediation - Unit Tests

**Location:** `src/sql/parser.rs` `#[cfg(test)]` module
**Command:** `cargo test --lib sql::parser`

| Test ID | Test Name | Validates |
|---------|-----------|-----------|
| TS43-P01 | `test_unterminated_string_returns_error` | AC-13, REQ-PARSE-007 |
| TS43-P02 | `test_unterminated_string_error_has_correct_line` | AC-15 |
| TS43-P03 | `test_unterminated_string_error_has_column` | AC-15 |
| TS43-P04 | `test_unterminated_string_after_two_statements_line_tracking` | AC-13, AC-15 |
| TS43-P05 | `test_unterminated_block_comment_returns_error` | AC-14, REQ-PARSE-013 |
| TS43-P06 | `test_unterminated_block_comment_error_has_correct_line` | AC-15 |
| TS43-P07 | `test_unterminated_block_comment_error_has_column` | AC-15 |
| TS43-P08 | `test_valid_input_still_returns_ok` | AC-13/AC-14 regression |
| TS43-P09 | `test_comment_marker_inside_string_is_not_comment` | AC-17 |
| TS43-P10 | `test_comment_after_string_close_is_stripped` | AC-17 (contrast case) |

### Test Suite 4: Parser Remediation - Integration Tests (no-DB)

**Location:** `tests/integration_tests.rs`
**Command:** `cargo test --test integration_tests`

| Test ID | Test Name | Validates |
|---------|-----------|-----------|
| TS43-I-P01 | `test_file_with_unterminated_string_shows_error_with_line` | AC-13, AC-15 wiring |
| TS43-I-P02 | `test_file_with_unterminated_comment_shows_error_with_line` | AC-14, AC-15 wiring |

### Test Suite 5: Regression - Full Test Suite

**Command:** `cargo test`
**Validates:** AC-16 (call site compilation), no regressions in existing parser tests

| Test ID | Command | Expected Result |
|---------|---------|-----------------|
| TS43-R01 | `cargo test` | 100% pass rate, zero failures |
| TS43-R02 | `cargo clippy -- -D warnings` | Zero warnings |

### Test Suite 6: Code Inspection Checklist

| Check ID | Target | Validates | Expected Finding |
|----------|--------|-----------|-----------------|
| TS43-C01 | `src/cli.rs` - profile edit/delete completers | AC-9 | Profile name completer registered |
| TS43-C02 | `docs/specifications/batch-mode.md` | AC-18 | REQ-PARSE-015 updated with clarified wording |
| TS43-C03 | `docs/design/batch-mode.md` (or equivalent) | AC-19 | Space-injection behavior documented |
| TS43-C04 | `src/sql/parser.rs` ~line 178 | AC-20 | Explanatory comment present for `unwrap()` |

---

## Strategy Summary

**Total Features Analyzed:** 2 (Feature 1: Profile Management, Feature 2: Parser Remediation)

**Test Types Required:**
- Unit tests: REQUIRED for both features
- Integration tests (CLI, no-DB): REQUIRED for both features
- Code inspection: REQUIRED for AC-9 (completion), AC-18, AC-19, AC-20
- Interactive tests: NOT NEEDED (non-interactive CLI feature; parser is pure logic)
- Benchmark tests: NOT NEEDED

**Estimated Test Count:**

| Suite | Type | Estimated Tests |
|-------|------|-----------------|
| Profile Management - Unit | Unit | 14 tests |
| Profile Management - Integration | Integration | 23 tests |
| Parser Remediation - Unit | Unit | 10 tests |
| Parser Remediation - Integration | Integration | 2 tests |
| Regression suite | `cargo test` | 1 run (all existing tests) |
| Code inspections | Inspection | 4 checks |
| **Total** | | **49-53 tests + 4 inspections** |

**Risk Assessment:**

- HIGH risk gaps: None - both features are fully testable without external dependencies
- MEDIUM risk gaps:
  - Config path override mechanism: integration tests require a way to point the binary at a temp dir instead of `~/.tq/`. If the architect does not expose a config path override (env var or flag), integration tests cannot run safely without risk of corrupting the developer's config. This must be resolved before test implementation.
  - Delete confirmation prompt (AC-3): If implemented with interactive TTY prompt (non-`--force` path), additional test infrastructure needed.
- LOW risk gaps:
  - AC-9 tab completion validated by code inspection only (behavioral validation not possible via `cargo test`)
  - AC-18, AC-19, AC-20 are documentation-only (no behavioral impact)

**Dependencies Required:**
- Live database: NO
- Network access: NO
- Specific OS: NO
- Config path override mechanism: YES - needed for profile management integration tests to be safe and repeatable. The architect must expose a way to override the config directory (recommended: `TQ_CONFIG_DIR` environment variable that overrides `~/.tq/` location).

---

## Key Risk: Config Path Override Mechanism

Profile management integration tests **cannot safely run against the developer's real `~/.tq/config.toml`**. If the implementation reads `~/.tq/config.toml` unconditionally without supporting a config directory override, integration tests would either:
- Risk corrupting the developer's real profiles
- Need to mock the entire filesystem (complex, fragile)
- Be impossible to run in isolation

**Requirement for architect:** Expose a `TQ_CONFIG_DIR` environment variable (or equivalent) that overrides the `~/.tq/` directory lookup. This is a testability requirement, not just a nice-to-have. Integration tests will set `TQ_CONFIG_DIR=/tmp/tq-test-XXXX` to operate in a safe, isolated sandbox.

If this override is not implemented, profile management integration tests will be classified as **BLOCKED** and the sprint cannot be APPROVED.

---

## Strategy Validation Checklist

- [x] Every feature has complete specification analysis section
- [x] Feature characteristics are classified (not assumed)
- [x] Test strategy is derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest
- [x] Specification coverage map includes all acceptance criteria
- [x] Every acceptance criterion maps to at least one test type
- [x] Test implementation plan is detailed and actionable
- [x] Coverage sufficiency is assessed
- [x] Key risk (config path override) explicitly identified and escalated
- [x] Non-testable ACs (AC-18, AC-19, AC-20) explicitly called out

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-03-21
**Review Status:** DRAFT

**Known Blockers / Risks:**
1. **Config path override (HIGH priority):** Profile management integration tests require `TQ_CONFIG_DIR` (or equivalent) override mechanism to avoid touching the developer's real config. Architect must design this in.
2. **Delete confirmation prompt architecture:** If `tq profile delete <name>` without `--force` shows an interactive TTY prompt, an optional interactive test (expectrl) may be needed. Await architect's design.
3. **ParseError struct shape:** AC-15 requires line and column in the error. The exact field names and types are architect-defined. Test cases will assert on observable behavior; exact API assertions will be finalized after implementation.
