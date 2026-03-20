# Sprint 40 Test Strategy: Variable Substitution

**Created:** 2026-03-20
**Author:** quality-validator
**Sprint:** Sprint 40
**Features:** Variable Substitution Engine (Issue #26), Sprint 39 Remediation (redundant test removal)

---

## Overview

Sprint 40 delivers two deliverables:

1. **Feature 1: Variable Substitution Engine** (P0) - YAML-based parameter file parsing and `{{variable}}` marker substitution in SQL, including `{{$ENV.VAR_NAME}}` for environment variables, multi-file merging, CLI `--params`/`-p` flag, and REPL `/params` metacommand.

2. **Feature 2: Sprint 39 Remediation** (P0) - Remove ~25 redundant utility tests from sessions.rs, sysconfig.rs, locks.rs, and sample.rs that now duplicate coverage already established in monitoring_utils.rs.

**Note on specification completeness:** At strategy definition time the cli-ux-designer has not yet written the new specification sections. This strategy is derived from the acceptance criteria in `docs/sprints/sprint-40-planning.md` (AC-1 through AC-13) and the existing `docs/specifications/batch-mode.md` Variable Substitution section, which currently documents only the shell-workaround approach. The strategy assumes the specifications written by cli-ux-designer will be consistent with the ACs. Test case details will be finalized against the completed specifications before test execution.

---

## Feature-by-Feature Test Strategy

---

### Feature 1: Variable Substitution Engine

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-40-planning.md` lines 44-56 (AC-1 through AC-11)
- Secondary: `docs/specifications/batch-mode.md` §9 Variable Substitution (currently documents workarounds; will be updated by cli-ux-designer)
- Secondary: `docs/specifications/cli-interface.md` (will be updated with `--params` flag)
- Secondary: `docs/specifications/repl.md` (will be updated with `/params` metacommand)

**Requirements from Acceptance Criteria:**

1. AC-1: New `--params`/`-p` flag accepts path to a YAML file
2. AC-2: `{{variable}}` markers in SQL are replaced with values from YAML
3. AC-3: Nested YAML paths work with dot notation: `{{section.key}}` resolves `section: { key: value }`
4. AC-4: `{{$ENV.VAR_NAME}}` reads from environment variables
5. AC-5: Undefined variables produce clear error with variable name and available variables
6. AC-6: Works with `tq query` (inline SQL), `tq query --file` (file input), and stdin
7. AC-7: Works in REPL mode via `/params` metacommand to load/unload parameter files
8. AC-8: Multiple `-p` flags merge parameters (later files override earlier)
9. AC-9: `tq help params` topic explains variable substitution syntax and usage
10. AC-10: Tab completion for `/params` metacommand in REPL
11. AC-11: YAML parse errors produce actionable error messages with file path and line number

**Feature Characteristics:**

**User Interaction Types:**
- Pure Logic (variable substitution engine: YAML parsing, marker scanning, string replacement)
- CLI Batch (tq query with `--params` flag - scripted, non-interactive)
- Interactive PTY (REPL `/params` metacommand, tab completion)

**Explanation:** The substitution engine itself is a pure algorithm: parse YAML into a map, scan SQL for `{{...}}` markers, replace each with the resolved value. The CLI batch surface adds a file path argument. The REPL surface adds an interactive metacommand with tab completion.

**Observable Behavior:**
- Structured data output: SQL sent to the database is modified (markers replaced with values)
- File system side effects: YAML parameter files are read from disk
- State management: REPL mode holds loaded parameter context across queries
- Visual output in terminal (REPL): `/params show` displays current parameters

**External Dependencies:**
- File system access: reads YAML parameter files from disk
- Terminal/PTY: REPL metacommand requires interactive session for tab completion validation
- None for unit tests (pure logic: YAML parsing is done via `serde_yaml`, string replacement is deterministic)

**Validation Challenges:**

- **REPL state persistence**: Loaded parameters must survive across multiple queries in the same session. Only an interactive test can validate this cross-query persistence.
- **Multiple `-p` merge ordering**: Override behavior (later file wins) must be tested with specifically crafted fixture files.
- **Environment variable isolation**: Tests for `{{$ENV.VAR_NAME}}` must set/unset environment variables carefully to avoid test pollution.
- **Error message quality**: AC-5 and AC-11 require actionable error messages with specific content (variable name, available variables, file path, line number). Unit tests can assert on message content.
- **SQL passthrough**: SQL with no `{{` markers must be passed through unchanged. Critical for backwards compatibility.

**Critical Behaviors to Validate:**

1. AC-2: `{{variable}}` in SQL text is replaced with the string value from the YAML map
2. AC-3: `{{section.key}}` resolves multi-level nested YAML: `section: { key: value }` → `value`
3. AC-4: `{{$ENV.DATABASE}}` reads from `std::env::var("DATABASE")` at substitution time
4. AC-5: Undefined variable error includes the variable name and lists all variables available
5. AC-8: When two files define the same key, the later file's value wins (merge override)
6. AC-11: YAML parse error includes the file path and serde_yaml line/column information

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Pure Logic" checked:
  → Unit tests REQUIRED
  Reason: Substitution engine is a deterministic algorithm - fastest and most precise at
  AC-2, AC-3, AC-4, AC-5, AC-8, AC-11 edge cases

IF "CLI Batch" checked:
  → Integration tests REQUIRED
  Reason: AC-1 (--params flag), AC-6 (inline/file/stdin), AC-9 (help topic) require
  end-to-end CLI execution validation

IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: AC-7 (/params metacommand), AC-10 (tab completion) only validatable with PTY

IF "File system access" checked:
  → Need fixture YAML files for tests
  Reason: YAML parsing requires real files; unit tests can use in-memory strings;
  integration tests need real files on disk
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** AC-2 (marker substitution), AC-3 (nested paths), AC-4 (env var resolution), AC-5 (undefined variable error), AC-8 (multi-file merge), AC-11 (YAML parse error messages), passthrough (no markers), edge cases (`{{` without closing `}}`, empty YAML, special chars in values)
- **Approach:** Test `src/params.rs` functions directly with in-memory YAML strings and SQL fragments. Use `std::env::set_var` / `std::env::remove_var` for environment variable tests.
- **Rationale:** The substitution engine is pure logic. Unit tests give precise control over inputs and exact assertion of outputs/errors without any external dependencies.
- **Gap if missing:** Logic bugs in marker scanning, path resolution, merge ordering, and error formatting would only surface in integration tests with much less diagnostic precision.
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (CLI Batch)**
- **Validates:** AC-1 (`--params`/`-p` flag accepted by CLI), AC-6 (works with inline SQL, `--file`, stdin), AC-8 (multiple `-p` flags at CLI level), AC-9 (`tq help params` topic)
- **Approach:** Spawn `tq` binary via `std::process::Command` with YAML fixture files on disk. Assert on exit code and stdout content. No database required for substitution tests (SQL can be a trivially valid constant or the test database can be skipped by checking the substituted SQL is correct before connection).
- **Rationale:** AC-1 and AC-6 are CLI surface requirements. The `--params` flag must be wired into argument parsing (clap). The integration test validates the flag is accepted, the file is read, and substitution happens before query execution.
- **Gap if missing:** Flag typos in clap definition, wrong argument position, flag not wired to execution pipeline - none visible to unit tests.
- **Necessity:** REQUIRED

**Test Type 3: Interactive Tests (REPL, expectrl)**
- **Validates:** AC-7 (`/params load`, `/params unload`, `/params show` metacommands), AC-10 (tab completion for `/params`), state persistence across queries in REPL session
- **Approach:** Spawn REPL via expectrl. Load a YAML file via `/params load ./fixtures/test.yaml`, execute a SQL query using a `{{variable}}` marker, verify the correct SQL is sent (or verify REPL output). Test `/params show` displays loaded variables. Test `/params unload` clears them. Tab-complete `/par` and verify `/params` appears.
- **Rationale:** AC-7 and AC-10 are REPL-only behaviors. Tab completion cannot be tested without a PTY. Cross-query parameter persistence requires an interactive session.
- **Gap if missing:** `/params` metacommand broken, tab completion missing, state not persisting - none visible to unit or batch integration tests.
- **Necessity:** REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | Core substitution logic; AC-2, AC-3, AC-4, AC-5, AC-8, AC-11 edge cases | Logic bugs, edge case failures invisible | MUST IMPLEMENT |
| Integration tests (CLI) | REQUIRED | AC-1 flag wiring; AC-6 SQL source types; AC-9 help topic | CLI wiring bugs, flag not accepted | MUST IMPLEMENT |
| Interactive tests (REPL) | REQUIRED | AC-7 metacommand; AC-10 tab completion; PTY-only behaviors | Metacommand broken, tab completion missing | MUST IMPLEMENT |
| Benchmark tests | NOT NEEDED | No performance requirements specified for substitution | N/A | SKIP |

**Summary:**
- REQUIRED test types: 3
- NOT NEEDED: 1

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| AC-1 | `--params`/`-p` flag accepts YAML file path | sprint-40 AC-1 | Integration | CLI flag must be validated via binary invocation | TC-040-002 |
| AC-2 | `{{variable}}` markers replaced with YAML values | sprint-40 AC-2 | Unit | Pure substitution logic - fastest/most precise in unit | TC-040-001 |
| AC-3 | `{{section.key}}` dot notation for nested YAML | sprint-40 AC-3 | Unit | Nested path resolution is pure logic | TC-040-001 |
| AC-4 | `{{$ENV.VAR_NAME}}` reads environment variables | sprint-40 AC-4 | Unit | Env var resolution is pure logic, controlled in unit tests | TC-040-001 |
| AC-5 | Undefined variable produces clear error with name and available vars | sprint-40 AC-5 | Unit + Integration | Unit validates message content; integration validates CLI exit code | TC-040-001, TC-040-002 |
| AC-6 | Works with inline SQL, `--file`, and stdin | sprint-40 AC-6 | Integration | Three input sources require CLI invocation to validate | TC-040-002 |
| AC-7 | REPL `/params` metacommand (load/unload/show) | sprint-40 AC-7 | Interactive | PTY-only behavior; state persistence across queries | TC-040-003 |
| AC-8 | Multiple `-p` flags merge (later overrides) | sprint-40 AC-8 | Unit + Integration | Unit validates merge logic; integration validates multi-flag CLI | TC-040-001, TC-040-002 |
| AC-9 | `tq help params` topic | sprint-40 AC-9 | Integration | Help text accessible via CLI binary | TC-040-002 |
| AC-10 | Tab completion for `/params` in REPL | sprint-40 AC-10 | Interactive | PTY-only tab completion validation | TC-040-003 |
| AC-11 | YAML parse errors include file path and line number | sprint-40 AC-11 | Unit + Integration | Unit validates error content; integration validates CLI error output | TC-040-001, TC-040-002 |

**Coverage Validation:**
- Every acceptance criterion maps to at least one test type
- Every test type justified by requirement
- No orphaned requirements

#### 5. Gap Analysis

**Live Database for Substitution Tests**
- **Reason:** Variable substitution happens before query execution. The substituted SQL text is what gets sent to the database. We can test that the correct SQL text is produced without a live database by testing the substitution output in isolation. Integration tests that require a database connection (to verify end-to-end) are marked `#[ignore]`.
- **What won't be validated without DB:** That the substituted SQL executes correctly against a real Teradata system.
- **Risk assessment:** LOW - Substitution correctness (unit-tested) is independent of SQL validity. SQL correctness is already covered by existing query tests.
- **Mitigation:** Unit tests validate exact substitution output. Integration tests validate CLI wiring without DB.

**REPL State Persistence Across Session Restart**
- **Reason:** The specification does not require parameter files to be persisted across REPL session restarts (only that they work within a session). This simplifies state management.
- **Risk assessment:** LOW - Session state is in-memory only; no persistence requirement identified.

**Concurrent REPL Sessions with Different Params**
- **Reason:** Out of scope; tq is a single-session tool.
- **Risk assessment:** N/A

#### 6. Test Implementation Plan

**Test Type: Unit Tests (`src/params.rs`)**
- **Location:** `src/params.rs` - `#[cfg(test)] mod tests`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 30 tests
- **Key scenarios:**

  *YAML parsing:*
  1. `test_parse_yaml_simple_flat` - `{ key: value }` produces flat map
  2. `test_parse_yaml_nested_two_levels` - `{ a: { b: val } }` accessible as `a.b`
  3. `test_parse_yaml_nested_three_levels` - `{{a.b.c}}` resolves three-level nesting
  4. `test_parse_yaml_array_value` - Array value is converted to string representation
  5. `test_parse_yaml_integer_value` - Integer `42` coerced to string `"42"`
  6. `test_parse_yaml_boolean_value` - Boolean `true` coerced to string `"true"`
  7. `test_parse_yaml_empty_file` - Empty YAML produces empty parameter map (not an error)
  8. `test_parse_yaml_invalid_syntax` - Invalid YAML returns error with file path context
  9. `test_parse_yaml_special_chars_in_value` - Values with SQL special chars (quotes, semicolons) are preserved exactly

  *Variable resolution:*
  10. `test_substitute_simple_variable` - `SELECT {{schema}}` → `SELECT mydb`
  11. `test_substitute_nested_path` - `{{target.db}}` resolves `target: { db: mydb }`
  12. `test_substitute_multiple_markers` - Multiple `{{var}}` in one SQL all replaced
  13. `test_substitute_same_variable_twice` - `{{x}} AND {{x}}` both replaced
  14. `test_substitute_env_var` - `{{$ENV.MY_VAR}}` reads `std::env::var("MY_VAR")`
  15. `test_substitute_env_var_missing` - `{{$ENV.UNDEFINED_VAR}}` returns error
  16. `test_substitute_undefined_variable_error_contains_name` - Error message contains the variable name
  17. `test_substitute_undefined_variable_error_lists_available` - Error message lists known variables
  18. `test_substitute_no_markers_passthrough` - SQL without `{{` passes through unchanged
  19. `test_substitute_partial_marker_no_close` - `{{ without }}` is passed through or produces clear error

  *Multiple file merging:*
  20. `test_merge_two_files_non_overlapping` - Keys from both files present in merged result
  21. `test_merge_override_later_wins` - Same key defined in both; later file value wins
  22. `test_merge_three_files_priority` - Three files; last definition always wins
  23. `test_merge_nested_override` - Nested key `a.b` from later file overrides earlier

  *Edge cases:*
  24. `test_empty_sql_passthrough` - Empty SQL string produces empty output, no error
  25. `test_sql_with_single_curly_brace` - `{not_a_marker}` passed through unchanged
  26. `test_value_with_curly_braces` - Value containing `{` in a YAML string is safe
  27. `test_whitespace_in_marker` - `{{ var }}` with spaces: treated as undefined or stripped
  28. `test_null_yaml_value` - YAML explicit `null` as value produces clear error or empty string
  29. `test_env_prefix_case_sensitive` - `{{$env.VAR}}` (lowercase) does not match `$ENV.` prefix
  30. `test_parse_yaml_error_includes_line_number` - serde_yaml error exposes line/col in formatted message

- **Mocking strategy:** In-memory YAML strings via `serde_yaml::from_str`. Environment variables via `std::env::set_var`/`remove_var` (note: test isolation requires care; use unique env var names per test or sequential test execution).

**Test Type: Integration Tests (CLI Batch) - `tests/params_integration.rs`**
- **Location:** `tests/params_integration.rs` (new file)
- **Framework:** `std::process::Command`
- **Test count estimate:** 12 tests (9 no-DB, 3 live-DB marked `#[ignore]`)
- **Fixture files needed:** `tests/fixtures/params/` directory with YAML files:
  - `basic.yaml` - `{ table: employees, schema: hr }`
  - `override_a.yaml` - `{ env: staging }`
  - `override_b.yaml` - `{ env: production }`
  - `nested.yaml` - `{ target: { db: mydb, schema: myschema } }`
  - `invalid.yaml` - Intentionally malformed YAML
  - `envvar.yaml` - `{ query: "SELECT {{$ENV.TQ_TEST_TABLE}}" }`
- **Key scenarios (no-DB):**
  1. `test_params_flag_accepted` - `tq --params basic.yaml query "SELECT 1"` exits non-zero only for DB failure, not arg parse error (verifies flag is recognized)
  2. `test_params_short_flag_accepted` - `tq -p basic.yaml query "SELECT 1"` (verifies short flag)
  3. `test_params_file_not_found` - `tq --params nonexistent.yaml query "SELECT 1"` exits non-zero with file-not-found error on stderr
  4. `test_params_invalid_yaml` - `tq --params invalid.yaml query "SELECT 1"` exits non-zero with YAML parse error mentioning file path on stderr
  5. `test_params_undefined_variable` - SQL with `{{undefined_var}}` and known params produces error naming the undefined variable
  6. `test_params_multiple_p_flags` - Two `-p` flags with override; verify later-wins behavior in error output or help text
  7. `test_help_params_topic` - `tq help params` exits 0 and stdout contains variable substitution documentation
  8. `test_params_no_markers_passthrough` - YAML file + SQL with no markers; query runs normally (no substitution error)
  9. `test_params_inline_sql_substituted` - With fixture basic.yaml and SQL `SELECT * FROM {{schema}}.{{table}}`, verifies substitution happened (check error message when DB absent, or verify SQL in verbose output)
- **Key scenarios (live-DB marked `#[ignore]`):**
  10. `test_params_inline_sql_executes` - Full end-to-end: `tq --params basic.yaml query "SELECT 1"` succeeds
  11. `test_params_file_sql_substituted` - SQL file with markers + YAML file; query executes
  12. `test_params_stdin_sql_substituted` - SQL via stdin with markers; substitution happens before execution

**Test Type: Interactive Tests (REPL, expectrl) - `tests/interactive_tests.rs`**
- **Location:** `tests/interactive_tests.rs` (append to existing file)
- **Framework:** expectrl crate (existing infrastructure)
- **Test count estimate:** 6 tests (all marked `#[ignore]`)
- **Key scenarios:**
  1. `test_params_tab_completion_shows_metacommand` - Type `/par` + Tab; verify `/params` appears in completion
  2. `test_params_load_command` - `/params load tests/fixtures/params/basic.yaml`; verify confirmation message
  3. `test_params_show_displays_variables` - After load, `/params show` displays key-value pairs from YAML
  4. `test_params_variable_used_in_query` - Load basic.yaml; execute `SELECT * FROM {{schema}}.{{table}};`; verify query executes (success or DB error, not substitution error)
  5. `test_params_unload_clears_state` - Load, then `/params unload`; subsequent query with `{{var}}` produces "undefined variable" error
  6. `test_params_load_nonexistent_file` - `/params load /does/not/exist.yaml`; verify error message displayed without REPL crash

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Unit tests validate: all substitution algorithm paths (30 tests), including AC-2, AC-3, AC-4, AC-5, AC-8, AC-11, passthrough, edge cases
- Integration tests validate: CLI flag wiring AC-1, all three SQL input modes AC-6, multi-flag merge AC-8, help topic AC-9, error exit codes
- Interactive tests validate: REPL metacommand AC-7, tab completion AC-10, state persistence

**Known gaps:**
1. Live-DB end-to-end is optional (marked `#[ignore]`). Risk: LOW. Substitution correctness is unit-validated; query execution is tested by existing query tests.
2. `test_params_multiple_p_flags` without a live DB can only verify error output or verbose log output; it cannot assert on the actual SQL sent to the database. Risk: LOW - merge logic is unit-tested directly.

**Question: If all planned tests pass, can we claim the feature "works as specified"?**
**Answer: YES.** Unit tests provide comprehensive algorithmic coverage. Integration tests validate the CLI surface. Interactive tests validate the REPL surface. The live-DB gap is acceptable because substitution is tested independently of database execution.

---

### Feature 2: Sprint 39 Remediation - Redundant Test Removal

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-40-planning.md` lines 63-72 (AC-12, AC-13)
- Secondary: Sprint 39 review recommendations #1 and #2

**Requirements:**
1. AC-12: REQ-QUERY spec updated to match multi-query implementation
2. AC-13: ~25 redundant utility tests removed from consumer modules

**Context:** Sprint 39 extracted `extract_integer`, `extract_decimal`, `extract_trimmed_string`, and `escape_csv` from sessions.rs, sysconfig.rs, locks.rs, and sample.rs into a shared `monitoring_utils.rs`. The shared module now has authoritative tests for these utility functions. The consumer module tests that test only these utility functions (not the consuming module's own logic) are redundant.

#### 2. Redundant Test Identification

**Criterion for redundancy:** A test is redundant if it tests a function that is now defined in `monitoring_utils.rs` and the test adds no coverage beyond what is already in `monitoring_utils.rs` tests. Tests that test the consuming module's own logic (e.g., `SessionInfo::from_row()`, `LockInfo::from_row()`, display formatting) are NOT redundant.

##### sessions.rs redundant tests (6 tests)

| Test Name | Reason Redundant | monitoring_utils.rs Coverage |
|-----------|-----------------|------------------------------|
| `test_escape_csv_simple` (line 521) | Tests `escape_csv("hello")` == `"hello"` | `test_escape_csv_simple` in monitoring_utils |
| `test_escape_csv_with_comma` (line 526) | Tests `escape_csv("hello,world")` | `test_escape_csv_with_comma` in monitoring_utils |
| `test_escape_csv_with_quotes` (line 531) | Tests `escape_csv("say \"hello\"")` | `test_escape_csv_with_quotes` in monitoring_utils |
| `test_extract_integer_from_integer` (line 640) | Tests `extract_integer` on `Value::Integer` | `test_extract_integer_from_integer` in monitoring_utils |
| `test_extract_integer_from_decimal` (line 646) | Tests `extract_integer` on `Value::Decimal` | `test_extract_integer_from_decimal` in monitoring_utils |
| `test_extract_integer_from_null` (line 652) | Tests `extract_integer` on `Value::Null` | `test_extract_integer_from_null` in monitoring_utils |
| `test_extract_decimal_from_decimal` (line 658) | Tests `extract_decimal` on `Value::Decimal` | `test_extract_decimal_from_decimal` in monitoring_utils |
| `test_extract_decimal_from_integer` (line 666) | Tests `extract_decimal` on `Value::Integer` | `test_extract_decimal_from_integer` in monitoring_utils |
| `test_extract_decimal_from_null` (line 674) | Tests `extract_decimal` on `Value::Null` | `test_extract_decimal_from_null` in monitoring_utils |

**Total redundant in sessions.rs: 9 tests**

Note: `test_format_logon_time`, `test_format_skew_*`, `test_format_spool_*`, `test_session_info_from_row*`, `test_calculate_skew_*` are NOT redundant - they test sessions-specific logic.

##### sysconfig.rs redundant tests (8 tests)

| Test Name | Reason Redundant | monitoring_utils.rs Coverage |
|-----------|-----------------|------------------------------|
| `test_extract_trimmed_string_from_string` (line 265) | Tests `extract_trimmed_string` on string | `test_extract_trimmed_string_from_string` in monitoring_utils |
| `test_extract_trimmed_string_from_null` (line 271) | Tests `extract_trimmed_string` on null | `test_extract_trimmed_string_from_null_with_null_display` in monitoring_utils |
| `test_extract_trimmed_string_from_integer` (line 280) | Tests `extract_trimmed_string` on integer | `test_extract_trimmed_string_from_integer` in monitoring_utils |
| `test_extract_integer_from_integer` (line 286) | Tests `extract_integer` on `Value::Integer` | `test_extract_integer_from_integer` in monitoring_utils |
| `test_extract_integer_from_decimal` (line 292) | Tests `extract_integer` on `Value::Decimal` | `test_extract_integer_from_decimal` in monitoring_utils |
| `test_extract_integer_from_null` (line 298) | Tests `extract_integer` on `Value::Null` | `test_extract_integer_from_null` in monitoring_utils |
| `test_escape_csv_simple` (line 383) | Tests `escape_csv("hello")` | `test_escape_csv_simple` in monitoring_utils |
| `test_escape_csv_with_comma` (line 388) | Tests `escape_csv("hello,world")` | `test_escape_csv_with_comma` in monitoring_utils |
| `test_escape_csv_with_quotes` (line 393) | Tests `escape_csv("say \"hello\"")` | `test_escape_csv_with_quotes` in monitoring_utils |

**Note on borderline cases:** `test_escape_csv_with_parentheses_and_colon` (line 398) and `test_escape_csv_release_with_comma` (line 406) use sysconfig-domain-specific values (version strings) to verify edge cases of `escape_csv` in the context of how sysconfig uses it. These are borderline. However, `monitoring_utils.rs` has `test_escape_csv_no_special_chars` (same semantic) and `test_escape_csv_with_comma_in_parentheses` (same value pattern). Both are therefore redundant.

Adding those 2: **Total redundant in sysconfig.rs: 11 tests** (including `test_escape_csv_with_parentheses_and_colon` and `test_escape_csv_release_with_comma`)

##### locks.rs redundant tests (5 tests)

| Test Name | Reason Redundant | monitoring_utils.rs Coverage |
|-----------|-----------------|------------------------------|
| `test_escape_csv_simple` (line 896) | Tests `escape_csv("hello")` | `test_escape_csv_simple` in monitoring_utils |
| `test_escape_csv_with_comma` (line 901) | Tests `escape_csv("1045, 1067")` - same semantic | `test_escape_csv_with_comma` in monitoring_utils |
| `test_extract_trimmed_string_from_string` (line 906) | Tests `extract_trimmed_string` on string | `test_extract_trimmed_string_from_string` in monitoring_utils |
| `test_extract_trimmed_string_from_null` (line 912) | Tests `extract_trimmed_string` on null | `test_extract_trimmed_string_from_null_with_null_display` in monitoring_utils |
| `test_extract_integer_from_integer` (line 918) | Tests `extract_integer` on `Value::Integer` | `test_extract_integer_from_integer` in monitoring_utils |
| `test_extract_integer_from_decimal` (line 924) | Tests `extract_integer` on `Value::Decimal` | `test_extract_integer_from_decimal` in monitoring_utils |
| `test_extract_integer_from_null` (line 930) | Tests `extract_integer` on `Value::Null` | `test_extract_integer_from_null` in monitoring_utils |

**Total redundant in locks.rs: 7 tests**

Note: All the `test_lock_info_from_row_*`, `test_build_display_rows_*`, `test_identify_blocking_chains_*`, `test_display_*`, `test_format_waiting_sessions_*`, and `test_error_classification_*` tests are NOT redundant - they test locks-specific logic.

##### sample.rs redundant tests (4 tests)

| Test Name | Reason Redundant | monitoring_utils.rs Coverage |
|-----------|-----------------|------------------------------|
| `test_escape_csv_simple` (line 481) | Tests `escape_csv("hello")` | `test_escape_csv_simple` in monitoring_utils |
| `test_escape_csv_with_comma` (line 486) | Tests `escape_csv("hello,world")` | `test_escape_csv_with_comma` in monitoring_utils |
| `test_escape_csv_with_quotes` (line 491) | Tests `escape_csv("say \"hello\"")` | `test_escape_csv_with_quotes` in monitoring_utils |
| `test_escape_csv_with_newline` (line 496) | Tests `escape_csv` with newline | `test_escape_csv_with_newline` in monitoring_utils |

**Total redundant in sample.rs: 4 tests**

Note: `test_parse_table_name_*` and `test_constants` are NOT redundant.

#### 3. Summary of Redundant Tests to Remove

| Module | Redundant Tests | Count |
|--------|----------------|-------|
| sessions.rs | test_escape_csv_simple, test_escape_csv_with_comma, test_escape_csv_with_quotes, test_extract_integer_from_integer, test_extract_integer_from_decimal, test_extract_integer_from_null, test_extract_decimal_from_decimal, test_extract_decimal_from_integer, test_extract_decimal_from_null | 9 |
| sysconfig.rs | test_extract_trimmed_string_from_string, test_extract_trimmed_string_from_null, test_extract_trimmed_string_from_integer, test_extract_integer_from_integer, test_extract_integer_from_decimal, test_extract_integer_from_null, test_escape_csv_simple, test_escape_csv_with_comma, test_escape_csv_with_quotes, test_escape_csv_with_parentheses_and_colon, test_escape_csv_release_with_comma | 11 |
| locks.rs | test_escape_csv_simple, test_escape_csv_with_comma, test_extract_trimmed_string_from_string, test_extract_trimmed_string_from_null, test_extract_integer_from_integer, test_extract_integer_from_decimal, test_extract_integer_from_null | 7 |
| sample.rs | test_escape_csv_simple, test_escape_csv_with_comma, test_escape_csv_with_quotes, test_escape_csv_with_newline | 4 |
| **Total** | | **31 tests** |

#### 4. Test Strategy for Remediation Validation

**Decision Tree Results:**
- Pure refactor (test removal, not behavior change) → Regression suite is the primary validation gate
- No new user-facing behavior → No interactive or integration tests needed

**Derived Test Types:**

**Test Type 1: Regression Suite (cargo test)**
- **Validates:** AC-13 (tests removed without regression); AC-12 indirectly (spec updated without breaking tests)
- **Approach:** After removing tests, run `cargo test --lib`. All remaining tests must pass. The baseline count minus 31 is the expected new count.
- **Rationale:** Test removal should not affect any existing behavior. The regression suite proves the removal was safe.
- **Necessity:** REQUIRED

**Test Type 2: Clippy Check**
- **Validates:** No dead imports introduced by removing the test functions that used utility imports
- **Approach:** `cargo clippy -- -D warnings`
- **Rationale:** Removing tests may leave unused `use` statements in `#[cfg(test)]` blocks if those functions were the only users of the import.
- **Necessity:** REQUIRED

---

## Test Case Documents to Produce

| Test Case ID | Title | Feature | Type |
|--------------|-------|---------|------|
| TC-040-001 | Variable Substitution Engine - Unit Tests | Feature 1 | Unit |
| TC-040-002 | Variable Substitution - CLI Batch Integration Tests | Feature 1 | Integration |
| TC-040-003 | Variable Substitution - REPL Metacommand Interactive Tests | Feature 1 | Interactive |
| TC-040-004 | Sprint 39 Remediation - Redundant Test Removal | Feature 2 | Regression |

---

## Strategy Summary

**Total Features Analyzed:** 2

**Test Types Required:**

| Feature | Unit | Integration | Interactive | Regression | Clippy |
|---------|------|-------------|-------------|------------|--------|
| Feature 1: Variable Substitution | REQUIRED (30 tests) | REQUIRED (12 tests) | REQUIRED (6 tests) | N/A | N/A |
| Feature 2: Sprint 39 Remediation | N/A | N/A | N/A | REQUIRED | REQUIRED |

**Estimated Test Count (new tests added by Sprint 40):**

| Type | Feature 1 | Feature 2 | Total New |
|------|-----------|-----------|-----------|
| Unit tests | 30 | 0 | **30** |
| Integration tests (CLI) | 12 (9 no-DB + 3 live-DB) | 0 | **12** |
| Interactive tests (REPL) | 6 (all `#[ignore]`) | 0 | **6** |
| **Total added** | **48** | **0** | **48** |
| Tests removed (redundant) | 0 | **-31** | **-31** |

**Baseline:** ~790 tests (Sprint 39 target; confirmed at test execution time)
**Target:** ~790 + 48 - 31 = **~807 tests**

**Risk Assessment:**
- HIGH risk gaps: None
- MEDIUM risk gaps:
  - Live database needed for 3 integration + 6 interactive tests; these are marked `#[ignore]` and REPL tests require Teradata connection. Mitigated by comprehensive unit tests covering all substitution logic.
  - Multiple `-p` flag merge behavior is verifiable without DB only in error scenarios or verbose output; core merge logic is unit-tested.
- LOW risk gaps:
  - `{{` without `}}` edge case behavior depends on implementation decision (passthrough vs error); unit test covers both cases once the implementation choice is known.
  - Environment variable test isolation requires sequential test execution or unique variable names.

**Dependencies Required:**
- Live database: YES (for `#[ignore]` integration and interactive tests only)
- Network access: NO
- Specific OS: NO
- YAML fixture files: YES - create `tests/fixtures/params/` directory with test YAML files
- External tools: NO (serde_yaml is the only new dependency; already identified in sprint plan)

**New Testing Tools Required:** None. `serde_yaml` is sufficient for unit tests. Existing `expectrl` infrastructure covers interactive tests. Existing `std::process::Command` covers integration tests.

---

## Test Execution Strategy

### Phase 1: Regression Baseline Verification (before sprint work)

```bash
cargo test 2>&1 | grep "test result:"
# Record baseline count (should be ~790 from Sprint 39)
```

### Phase 2: Variable Substitution Engine (after params.rs implemented)

```bash
# Unit tests for substitution engine
cargo test --lib params::tests

# Full regression gate
cargo test --lib
```

### Phase 3: CLI Integration (after --params flag wired)

```bash
# Integration tests (no-DB subset)
cargo test --test params_integration -- --skip live_db

# Full regression
cargo test --lib
```

### Phase 4: REPL Integration (after /params metacommand implemented)

```bash
# Interactive tests (requires database)
cargo test --test interactive_tests params -- --ignored --test-threads=1
```

### Phase 5: Sprint 39 Remediation (after redundant tests removed)

```bash
# Clippy check (may reveal dead imports)
cargo clippy -- -D warnings

# Full regression (critical gate - must still pass at reduced count)
cargo test --lib
# Expected: ~790 - 31 + 30 = ~789 unit tests passing
```

### Phase 6: Full Final Regression

```bash
# All unit tests
cargo test --lib

# All integration tests (no-DB)
cargo test --test params_integration

# All interactive tests (requires database)
cargo test --test interactive_tests -- --ignored --test-threads=1

# Final clippy
cargo clippy -- -D warnings

# Expected: ~807 total tests passing
```

---

## Coverage Sufficiency Assessment

**Question: If all planned tests pass, can we claim the features "work as specified"?**

**Feature 1 (Variable Substitution):**
- Unit tests validate: all 11 ACs algorithmic requirements with 30 targeted tests covering YAML parsing, variable resolution, env var integration, merge behavior, error messages, edge cases
- Integration tests validate: CLI flag wiring, SQL input source compatibility, help topic, error exit codes
- Interactive tests validate: REPL metacommand lifecycle, tab completion, state persistence
- Combined coverage: **Comprehensive for all acceptance criteria**

**Feature 2 (Sprint 39 Remediation):**
- Regression suite validates: no behavioral regression from test removal
- Clippy validates: no dead code from removed tests
- Combined coverage: **Sufficient for a pure test-removal operation**

**Answer: YES for both features, with the accepted gap that live-DB end-to-end substitution execution depends on a connected Teradata system.**

---

## Strategy Validation Checklist

- Every feature has complete specification analysis section
- Feature characteristics are classified (not assumed)
- Test strategy derived from characteristics (not guessed)
- Every test type has clear rationale
- Gap analysis complete and honest
- Specification coverage map includes all ACs
- Every AC maps to at least one test type
- Test implementation plan detailed and actionable with test names
- Coverage sufficiency assessed
- Redundant test list identified with exact test names and line numbers
- Fixture file requirements documented
- New testing tools assessed (none required)
- Environment variable test isolation risk documented

**Strategy Status:** READY FOR REVIEW

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-03-20
**Review Status:** DRAFT
**Sprint:** 40 - Variable Substitution
**Submitted for Review:** 2026-03-20

**Reviewer:** tq-project-manager
**Review Status:** PENDING
**Review Date:** (pending)
**Review Comments:** (pending)
