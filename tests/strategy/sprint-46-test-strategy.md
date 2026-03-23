# Sprint 46 Test Strategy: Bug Fixes & /inspect Polish

**Created:** 2026-03-23
**Author:** quality-validator
**Sprint:** Sprint 46
**Features:**
1. Bug #35: Identifier quoting (`quote_identifier` uppercase + `extract_table_name` word boundaries)
2. Bug #34: New CLI batch commands (`tq describe`, `tq list`, `tq show-indexes`)
3. /inspect formatting compliance (section headers, defaults, footer, skew hint, TableKind NoPI, error prefix, usage prompt)

---

## Overview

Sprint 46 is a bug-heavy sprint with two P0 user-reported bugs and one P1 presentation polish task. The test strategy reflects this composition:

- **Bug #35** is two compounding pure-logic bugs in `src/sql/identifiers.rs` (`quote_identifier`) and `src/db/client.rs` (`extract_table_name`). Both functions are pure and fully unit-testable without a database. End-to-end verification (`/sample dbc.tables;` working) requires a live Teradata connection and is marked `#[ignore]`.
- **Bug #34** adds three new CLI subcommands (`tq describe`, `tq list`, `tq show-indexes`). Argument parsing via clap is pure-logic and 100% unit-testable. SQL generation and output formatting is unit-testable by injecting a mock writer. Actual database execution requires a live connection and is marked `#[ignore]`.
- **/inspect formatting** is pure presentation logic living entirely in `src/commands/inspect.rs`. All formatting helpers (`map_table_kind`, `calculate_skew`, section-header rendering, default display, column count footer, error prefix, usage prompt) are unit-testable without a database.

The sprint has a clear two-tier dependency profile: bugs #35 and #34 argument parsing, and all /inspect formatting tests need no database; end-to-end command execution tests require a live Teradata connection.

---

## Feature-by-Feature Test Strategy

---

### Feature 1: Bug #35 — Identifier Quoting (quote_identifier uppercase + extract_table_name word boundaries)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-46-planning.md` Bug #35, AC-1 through AC-8
- Secondary: `src/sql/identifiers.rs` — `quote_identifier()` function
- Secondary: `src/db/client.rs` — `extract_table_name()` function

**Requirements (from acceptance criteria):**
1. AC-7: Unit tests for `quote_identifier()` with uppercase behavior — quoted identifier must uppercase the input before wrapping in double quotes
2. AC-8: Unit tests for `extract_table_name()` word boundary matching — must not match "TABLE" substring within "TABLES"
3. AC-6: `extract_table_name` correctly extracts table from `SELECT * FROM "DBC"."TABLES" SAMPLE 10`
4. AC-5: Case-insensitive table names work: `dbc.TablesV`, `DBC.TABLESV`, `dbc.tablesv`
5. AC-1 through AC-4: `/sample`, `/peek`, `tq sample` work end-to-end (require DB, `#[ignore]`)

**Feature Characteristics:**

**User Interaction Type:** Pure Logic (string transformation algorithms)

**Explanation:** Both `quote_identifier()` and `extract_table_name()` are pure functions with no I/O. `quote_identifier` transforms a string to its SQL-safe double-quoted uppercase form. `extract_table_name` parses SQL text using keyword matching with word boundaries. Neither function requires a database connection, PTY, or file system access.

**Observable Behavior:**
- [x] Structured data output (function return values — strings / `Option<String>`)
- No visual side-effects

**External Dependencies:**
- [x] Database connection — only for end-to-end AC-1 through AC-5 (`#[ignore]`)
- [x] None for unit-level AC-6, AC-7, AC-8

**Validation Challenges:**
- The word-boundary fix for `extract_table_name` is subtle: `find("TABLE")` matches "TABLE" inside "TABLES". The test must use table names that contain keyword substrings to catch the bug.
- Uppercase quoting must not break SQL injection protection (embedded double quotes must still be doubled after uppercasing).

**Critical Behaviors to Validate:**
1. `quote_identifier("dbc")` returns `"DBC"` (uppercase applied before quoting)
2. `quote_identifier("tables")` returns `"TABLES"` (lowercase input uppercased)
3. `quote_identifier("my\"table")` returns `"MY\"\"TABLE"` (embedded quote escaped, uppercase applied)
4. `extract_table_name("SELECT * FROM employees WHERE status = 'TABLES'")` returns `"employees"` (not "s" from TABLES substring)
5. `extract_table_name(`SELECT * FROM "DBC"."TABLES" SAMPLE 10`)` returns the full qualified ref
6. `extract_table_name("INSERT INTO tablesv VALUES (1)")` returns `"tablesv"` (INTO keyword, word boundary)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

- "Pure Logic" checked for both functions → Unit tests are sufficient for AC-6, AC-7, AC-8
- "Database connection" checked for end-to-end behavior → `#[ignore]` integration tests for AC-1 through AC-5

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** `quote_identifier()` uppercase behavior and `extract_table_name()` word-boundary matching (AC-6, AC-7, AC-8)
- **Approach:** Direct calls to the pure functions with carefully chosen inputs that expose the bugs and verify the fixes. Include regression cases (no-break for special chars, embedded quotes).
- **Rationale:** The functions are pure; no mocking needed. Unit tests are complete and fast.
- **Gap if missing:** The core bugs go undetected. AC-7 and AC-8 explicitly require unit tests.
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (DB, `#[ignore]`)**
- **Validates:** `/sample dbc.tables;` end-to-end execution (AC-1 through AC-5)
- **Approach:** Spawn `tq sample dbc.tables` process with `TQ_LOGON` set, capture stdout, assert non-error output.
- **Rationale:** End-to-end tests catch any remaining path issues not caught by unit tests (e.g., `quote_table_reference()` call site in `metacommands.rs`).
- **Gap if missing:** AC-1 through AC-5 are uncovered; the call site in metacommands.rs could have a different bug.
- **Necessity:** RECOMMENDED (DB required, `#[ignore]`)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | Validates pure function behavior for AC-6, AC-7, AC-8 | Bug fix correctness unverified | MUST IMPLEMENT |
| Integration tests (DB) | RECOMMENDED | Validates end-to-end sample command works | AC-1 through AC-5 untested | SHOULD IMPLEMENT (ignored) |
| Interactive tests (PTY) | NOT NEEDED | No PTY-specific behavior in quoting logic | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirements | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| AC-7 | Unit tests for `quote_identifier()` uppercase | sprint-46-planning.md Bug #35 | Unit | TC-046-001 |
| AC-8 | Unit tests for `extract_table_name()` word boundaries | sprint-46-planning.md Bug #35 | Unit | TC-046-002 |
| AC-6 | `extract_table_name` extracts from `SELECT * FROM "DBC"."TABLES" SAMPLE 10` | sprint-46-planning.md Bug #35 | Unit | TC-046-002 |
| AC-5 | Case-insensitive table names work | sprint-46-planning.md Bug #35 | Unit + Integration(ignored) | TC-046-001, TC-046-003 |
| AC-1 | `/sample dbc.tables;` works in REPL | sprint-46-planning.md Bug #35 | Integration (ignored) | TC-046-003 |
| AC-2 | `/sample dbc.tables` (no semicolon) works | sprint-46-planning.md Bug #35 | Integration (ignored) | TC-046-003 |
| AC-3 | `/peek dbc.tables;` works | sprint-46-planning.md Bug #35 | Integration (ignored) | TC-046-003 |
| AC-4 | `tq sample dbc.tables` works in batch | sprint-46-planning.md Bug #35 | Integration (ignored) | TC-046-003 |

#### 5. Gap Analysis

**Integration Tests (end-to-end, DB required)**
- **Reason for inclusion:** AC-1 through AC-5 are user-observable behaviors. They are marked `#[ignore]` and only run with `--ignored` when DB is available.
- **What won't be validated without DB:** Whether the fixed `quote_identifier` is actually called through the full `/sample` and `tq sample` paths.
- **Risk assessment:** MEDIUM — the unit tests cover the fix, but the call site in `metacommands.rs:quote_table_reference()` might pass the identifier differently.
- **Mitigation:** Unit tests cover the function; code review confirms call site uses `quote_identifier` directly.
- **Revisit:** Run with `--ignored` in CI when a Teradata test instance is available.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/sql/identifiers.rs` `#[cfg(test)]` module (new tests added)
- **Location:** `src/db/client.rs` `#[cfg(test)]` module (new tests added)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 12 tests
- **Key scenarios:**
  1. `quote_identifier("dbc")` → `"\"DBC\""` (lowercase uppercased)
  2. `quote_identifier("tables")` → `"\"TABLES\""`
  3. `quote_identifier("TablesV")` → `"\"TABLESV\""` (mixed case uppercased)
  4. `quote_identifier("my\"table")` → `"\"MY\"\"TABLE\""` (embedded quote escaped after uppercasing)
  5. `extract_table_name("SELECT * FROM employees WHERE status IN ('TABLES')")` → `Some("employees")` (word boundary)
  6. `extract_table_name("INSERT INTO tablesv VALUES (1)")` → `Some("tablesv")`
  7. `extract_table_name("UPDATE tablesv SET x=1")` → `Some("tablesv")`
  8. `extract_table_name("SELECT * FROM \"DBC\".\"TABLES\" SAMPLE 10")` → Some form of DBC/TABLES extraction
  9. Regression: `quote_identifier` still escapes embedded double quotes
  10. Regression: `quote_qualified_name` still formats correctly (both parts uppercased)

**Test Type: Integration Tests (DB, `#[ignore]`)**
- **Location:** `tests/integration_tests.rs` or `tests/bug35_sample_integration.rs`
- **Framework:** `std::process::Command` to spawn `tq` binary
- **Test count estimate:** 4 tests
- **Key scenarios:**
  1. `tq sample dbc.tables` exits 0 and produces rows
  2. `tq sample dbc.tablesv` exits 0 with lowercase input
  3. `tq peek dbc.tables` exits 0
  4. `tq sample DBC.TABLESV` exits 0 with uppercase input

#### 7. Coverage Sufficiency Assessment

- Unit tests validate: `quote_identifier` uppercase, embedded-quote safety, `extract_table_name` word-boundary correctness
- Integration tests validate: end-to-end path through sample/peek commands with real DB
- Combined coverage: adequate for sprint acceptance

**Gaps in combined coverage:**
- Without DB, AC-1 through AC-5 are not directly executed. This is documented and acceptable; the unit tests validate the root-cause fix.

---

### Feature 2: Bug #34 — New CLI Batch Commands (tq describe, tq list, tq show-indexes)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-46-planning.md` Bug #34, AC-1 through AC-8
- Secondary: `src/cli.rs` — `Command` enum (needs new variants: `Describe`, `List`, `ShowIndexes`)
- Secondary: `src/main.rs` — command dispatch
- Secondary: `docs/specifications/cli-interface.md` — updated by cli-ux-designer

**Requirements (from acceptance criteria):**
1. AC-8: Unit tests for argument parsing
2. AC-7: All new commands appear in `tq --help` and `tq help`
3. AC-6: All new commands support `--format table|csv|json` flag
4. AC-1: `tq describe <table>` outputs column info
5. AC-2: `tq list databases` outputs database list
6. AC-3: `tq list tables [pattern]` outputs table list with optional glob filter
7. AC-4: `tq list views` outputs view list
8. AC-5: `tq show-indexes <table>` outputs index information

**Feature Characteristics:**

**User Interaction Type:** CLI Batch (scripted, non-interactive command execution) + Pure Logic (argument parsing)

**Explanation:** The new subcommands are standard clap CLI subcommands. Argument parsing is pure logic. The SQL generation and output formatting helpers are also testable in isolation. Actual database queries require a live connection.

**Observable Behavior:**
- [x] Structured data output (JSON, CSV, table)
- [x] Visual output in terminal (table formatting)

**External Dependencies:**
- [x] Database connection — for AC-1 through AC-5 (actual data retrieval)
- None for AC-6, AC-7, AC-8 (argument parsing, help text)

**Validation Challenges:**
- clap help text is tested via `--help` output capture (process spawn, no DB)
- `--format` flag acceptance is testable without a DB by verifying that clap parses the enum correctly
- Actual SQL generation is testable with a mock/captured writer if commands accept `impl Write`

**Critical Behaviors to Validate:**
1. `tq describe --help` includes the subcommand and `--format` flag description (AC-7)
2. `tq list --help` shows databases/tables/views subcommands (AC-7)
3. `tq show-indexes --help` includes `--format` flag (AC-7)
4. Clap parses `--format json`, `--format csv`, `--format table` without error (AC-6)
5. `tq list` without a subcommand shows error or help (edge case)
6. `tq describe` without a table name shows error (edge case)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

- "CLI Batch" checked → Integration tests (process spawn with `--help`, no DB needed) REQUIRED
- "Pure Logic" for argument parsing → Unit tests REQUIRED (clap struct parsing)
- "Database connection" checked → `#[ignore]` integration tests for AC-1 through AC-5

**Derived Test Types:**

**Test Type 1: Unit Tests (clap argument parsing)**
- **Validates:** The `Describe`, `List`, `ShowIndexes` variants exist in `Command` enum with correct fields (AC-8)
- **Approach:** Use `Cli::try_parse_from(["tq", "describe", "my.table"])` and assert variant + field values. Test `--format` parsing.
- **Rationale:** Catches typos in argument names, missing variants, wrong field types before any binary is built.
- **Gap if missing:** AC-8 uncovered; argument errors only found at runtime.
- **Necessity:** REQUIRED

**Test Type 2: CLI Integration Tests (process spawn, no DB)**
- **Validates:** Help text content (AC-7), `--format` flag presence (AC-6), error messages for missing arguments
- **Approach:** Spawn `tq describe --help`, `tq list --help`, `tq show-indexes --help` and assert strings in stdout. Spawn `tq describe` with no args and assert non-zero exit with error.
- **Rationale:** Help text can only be verified via actual process output; unit tests won't catch missing flag documentation.
- **Gap if missing:** Help text bugs and missing `--format` in help go undetected.
- **Necessity:** REQUIRED

**Test Type 3: Integration Tests (DB, `#[ignore]`)**
- **Validates:** AC-1 through AC-5 (actual data returned from Teradata)
- **Approach:** Spawn `tq describe dbc.tables`, `tq list databases`, `tq list tables`, `tq list views`, `tq show-indexes dbc.tables` with `TQ_LOGON` set.
- **Rationale:** Unit and help-text tests cannot validate that the SQL queries are correct and results are rendered.
- **Gap if missing:** AC-1 through AC-5 untested; SQL bugs go undetected.
- **Necessity:** RECOMMENDED (DB required, `#[ignore]`)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (clap parsing) | REQUIRED | Validates argument struct for AC-8 | Parsing bugs, missing variants | MUST IMPLEMENT |
| CLI integration (--help, no DB) | REQUIRED | Validates help text AC-7 and --format flag AC-6 | Help text bugs, missing flags | MUST IMPLEMENT |
| Integration tests (DB) | RECOMMENDED | Validates data retrieval AC-1 through AC-5 | SQL bugs, rendering bugs | SHOULD IMPLEMENT (ignored) |
| Interactive tests (PTY) | NOT NEEDED | No REPL-specific behavior | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirements | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| AC-8 | Unit tests for argument parsing | sprint-46-planning.md Bug #34 | Unit (clap) | TC-046-004 |
| AC-7 | Commands appear in `tq --help` | sprint-46-planning.md Bug #34 | CLI integration | TC-046-005 |
| AC-6 | All commands support `--format` flag | sprint-46-planning.md Bug #34 | Unit (clap) + CLI integration | TC-046-004, TC-046-005 |
| AC-1 | `tq describe` outputs column info | sprint-46-planning.md Bug #34 | Integration (ignored) | TC-046-006 |
| AC-2 | `tq list databases` outputs list | sprint-46-planning.md Bug #34 | Integration (ignored) | TC-046-006 |
| AC-3 | `tq list tables [pattern]` works | sprint-46-planning.md Bug #34 | Integration (ignored) | TC-046-006 |
| AC-4 | `tq list views` works | sprint-46-planning.md Bug #34 | Integration (ignored) | TC-046-006 |
| AC-5 | `tq show-indexes` works | sprint-46-planning.md Bug #34 | Integration (ignored) | TC-046-006 |

#### 5. Gap Analysis

**Interactive PTY Tests**
- **Reason for omission:** The new commands are batch-mode only, no REPL interaction involved.
- **What won't be validated:** Color rendering (not specified for these commands)
- **Risk assessment:** LOW
- **Mitigation:** CLI integration tests with --help capture cover observable output
- **Revisit:** If future spec adds interactive features to these commands

#### 6. Test Implementation Plan

**Test Type: Unit Tests (clap argument parsing)**
- **Location:** `src/cli.rs` `#[cfg(test)]` module
- **Framework:** Built-in Rust test framework + clap `try_parse_from`
- **Test count estimate:** 10 tests
- **Key scenarios:**
  1. `Cli::try_parse_from(["tq", "describe", "mydb.employees"])` → `Command::Describe` with table = "mydb.employees"
  2. `Cli::try_parse_from(["tq", "describe", "--format", "json", "employees"])` → format = Json
  3. `Cli::try_parse_from(["tq", "list", "databases"])` → `Command::List` with subcommand = Databases
  4. `Cli::try_parse_from(["tq", "list", "tables"])` → `Command::List` with subcommand = Tables
  5. `Cli::try_parse_from(["tq", "list", "tables", "emp*"])` → pattern = Some("emp*")
  6. `Cli::try_parse_from(["tq", "list", "views"])` → subcommand = Views
  7. `Cli::try_parse_from(["tq", "show-indexes", "mydb.employees"])` → `Command::ShowIndexes`
  8. `Cli::try_parse_from(["tq", "show-indexes", "--format", "csv", "employees"])` → format = Csv
  9. `Cli::try_parse_from(["tq", "describe"])` → error (missing required table argument)
  10. `Cli::try_parse_from(["tq", "list"])` → error or shows help (missing subcommand)

**Test Type: CLI Integration Tests (process spawn, no DB)**
- **Location:** `tests/cli_commands_46.rs`
- **Framework:** `std::process::Command`
- **Test count estimate:** 6 tests
- **Key scenarios:**
  1. `tq describe --help` stdout contains "describe" and "--format"
  2. `tq list --help` stdout contains "databases", "tables", "views"
  3. `tq show-indexes --help` stdout contains "show-indexes" and "--format"
  4. `tq --help` stdout contains "describe", "list", "show-indexes" (AC-7 global help)
  5. `tq describe` exits non-zero, stderr contains error about missing argument
  6. `tq list invalid-subcommand` exits non-zero with error

**Test Type: Integration Tests (DB, `#[ignore]`)**
- **Location:** `tests/cli_commands_46.rs`
- **Framework:** `std::process::Command` with `TQ_LOGON`
- **Test count estimate:** 5 tests
- **Key scenarios:**
  1. `tq describe dbc.tables` exits 0, output contains column names
  2. `tq list databases` exits 0, output contains "DBC"
  3. `tq list tables` exits 0, produces rows
  4. `tq list views` exits 0, produces rows
  5. `tq show-indexes dbc.tables` exits 0, produces index info

#### 7. Coverage Sufficiency Assessment

- Unit tests validate: clap argument struct correctness, format flag parsing
- CLI integration tests validate: help text, error messages, subcommand routing visible in --help
- DB integration tests validate: end-to-end data retrieval
- Combined coverage: comprehensive for the unit-testable surface; adequate for the DB-dependent surface (marked `#[ignore]`)

---

### Feature 3: /inspect Formatting Compliance

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-46-planning.md` /inspect formatting AC-1 through AC-8
- Secondary: `src/commands/inspect.rs` — all formatting helpers and render functions

**Requirements (from acceptance criteria):**
1. AC-1: Section headers use `── Section Name ──` format (not `=== Section Name ===`)
2. AC-2: Default column shows `-` instead of empty string when no default is defined
3. AC-3: Column count footer: `N columns` displayed after column table
4. AC-4: Skew interpretation hint: `(low)`, `(moderate)`, `(high)` after skew percentage
5. AC-5: `O` TableKind mapped to "Table (NoPI)" instead of "Table"
6. AC-6: Error message uses `Error:` prefix for not-found (currently just `"Object '...' not found."`)
7. AC-7: Usage prompt shows examples when `/inspect` called with no argument
8. AC-8: Direct row indexing fixed in `inspect.rs:649-660` (panic risk for empty result sets)

**Feature Characteristics:**

**User Interaction Type:** Pure Logic (presentation-layer formatting helpers)

**Explanation:** All acceptance criteria are about how strings are formatted or how logic branches in pure functions. Section headers are strings in `writeln!` calls. `map_table_kind` is a pure match function. `calculate_skew` returns a `f64` and the hint label is pure logic on that value. Default display is in the column rendering loop. None of these require a database connection.

**Observable Behavior:**
- [x] Visual output in terminal (formatting, section headers)
- [x] Structured data output (written to `impl Write`)

**External Dependencies:**
- None for all 8 ACs (pure formatting logic)
- Database connection only for end-to-end rendering validation (marked `#[ignore]`)

**Validation Challenges:**
- Section header format requires exact string matching including box-drawing characters (`──`).
- AC-8 (panic risk) needs to be tested by constructing the scenario that previously caused a panic — calling the index access with an empty result set.
- AC-7 (usage prompt) requires testing the code path where `object_name` is empty or whitespace.

**Critical Behaviors to Validate:**
1. `map_table_kind("O")` returns `"Table (NoPI)"` (not `"Table"`)
2. `map_table_kind("T")` still returns `"Table"` (regression)
3. Skew hint: 0-5% → `(low)`, 6-20% → `(moderate)`, >20% → `(high)`
4. Section header for "Object Info" renders as `── Object Info ──` (not `=== Object Info ===`)
5. Default column value: `None` → `"-"` (not `""`)
6. Column count footer: after 3 columns, footer shows `3 columns`
7. Error message for not-found: `"Error: Object 'x' not found."` (with `Error:` prefix)
8. Empty-argument call shows usage prompt with examples
9. Direct row indexing replaced with safe `.get()` calls (no panic on empty result)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

- "Pure Logic" checked for all formatting helpers → Unit tests are sufficient for AC-1 through AC-8
- No PTY needed (output is captured via `impl Write`)
- No database needed for formatting logic tests

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** All 8 acceptance criteria via direct calls to formatting helpers or by writing to `Vec<u8>` and asserting output strings.
- **Approach:**
  - Call `map_table_kind("O")` directly and assert return value
  - Call `calculate_skew` with various inputs and assert the hint label logic
  - Construct a `Vec<u8>` writer and call the rendering path for section headers, default display, column count footer
  - Construct empty result set scenario and assert no panic
- **Rationale:** All logic is pure and injectable via `impl Write`; unit tests are complete.
- **Gap if missing:** Formatting bugs go undetected until DB-connected run.
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (DB, `#[ignore]`)**
- **Validates:** That the formatted output looks correct on actual Teradata data
- **Approach:** Spawn `tq inspect dbc.tables`, assert output contains `──` header format and `(low)/(moderate)/(high)` hint
- **Rationale:** Validates the full rendering pipeline with real data
- **Gap if missing:** Rendering pipeline integration issues not caught
- **Necessity:** RECOMMENDED (DB required, `#[ignore]`)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | All formatting helpers are pure logic | Formatting bugs undetected | MUST IMPLEMENT |
| Integration tests (DB) | RECOMMENDED | Validates rendering with real data | End-to-end rendering gap | SHOULD IMPLEMENT (ignored) |
| Interactive tests (PTY) | NOT NEEDED | Output written to writer, no PTY needed | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirements | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| AC-1 | Section headers use `── Section Name ──` format | sprint-46-planning.md /inspect | Unit | TC-046-007 |
| AC-2 | Default column shows `-` for None | sprint-46-planning.md /inspect | Unit | TC-046-007 |
| AC-3 | Column count footer `N columns` | sprint-46-planning.md /inspect | Unit | TC-046-007 |
| AC-4 | Skew hint `(low)/(moderate)/(high)` | sprint-46-planning.md /inspect | Unit | TC-046-007 |
| AC-5 | `O` → "Table (NoPI)" | sprint-46-planning.md /inspect | Unit | TC-046-007 |
| AC-6 | Error prefix `Error:` for not-found | sprint-46-planning.md /inspect | Unit | TC-046-007 |
| AC-7 | Usage prompt with examples on no-arg | sprint-46-planning.md /inspect | Unit | TC-046-007 |
| AC-8 | Direct row indexing fixed (no panic) | sprint-46-planning.md /inspect | Unit | TC-046-007 |

#### 5. Gap Analysis

**Database-connected end-to-end rendering**
- **Reason for partial coverage:** The rendering functions require a live `DatabaseClient` to actually run queries. Unit tests cover the formatting helpers in isolation.
- **What won't be validated without DB:** Whether the section headers and skew hint appear in actual `tq inspect` output
- **Risk assessment:** LOW — formatting helpers are pure; if they return the correct string, the `writeln!` call produces correct output.
- **Mitigation:** Unit tests validate return values; integration tests run when DB is available.
- **Revisit:** Run DB tests in CI when Teradata instance is available.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/inspect.rs` `#[cfg(test)]` module (extend existing test module)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 15 tests
- **Key scenarios:**
  1. `map_table_kind("O")` → `"Table (NoPI)"` (AC-5 fix)
  2. `map_table_kind("T")` → `"Table"` (AC-5 regression)
  3. `skew_hint(0.0)` → `"(low)"` (AC-4, boundary)
  4. `skew_hint(5.0)` → `"(low)"` (AC-4, boundary upper)
  5. `skew_hint(5.1)` → `"(moderate)"` (AC-4, boundary)
  6. `skew_hint(20.0)` → `"(moderate)"` (AC-4, boundary upper)
  7. `skew_hint(20.1)` → `"(high)"` (AC-4, boundary)
  8. `skew_hint(100.0)` → `"(high)"` (AC-4)
  9. Section header rendering: `render_section_header("Object Info")` → `"── Object Info ──"` (AC-1)
  10. Default display: `None` → `"-"` (AC-2)
  11. Default display: `Some("42")` → `"42"` (AC-2 regression)
  12. Column count footer: given 5 columns, footer contains `"5 columns"` (AC-3)
  13. Not-found error: output starts with `"Error:"` (AC-6)
  14. No-arg usage prompt: contains example strings (AC-7)
  15. Empty result set: `query_object_type` returns `None` gracefully without panic (AC-8 — use safe `.get()`)

**Test Type: Integration Tests (DB, `#[ignore]`)**
- **Location:** `tests/inspect_integration_46.rs`
- **Framework:** `std::process::Command`
- **Test count estimate:** 3 tests
- **Key scenarios:**
  1. `tq inspect dbc.tables` stdout contains `──` header format
  2. `tq inspect dbc.tables` stdout contains skew hint `(low)` or `(moderate)` or `(high)`
  3. `tq inspect nonexistent_table` stdout contains `Error:` prefix

#### 7. Coverage Sufficiency Assessment

- Unit tests validate: all 8 formatting ACs through pure function testing and writer injection
- Integration tests validate: rendered output in process stdout
- Combined coverage: comprehensive

**Gaps in combined coverage:**
- PTY rendering (colors, layout) not tested — acceptable because /inspect output is monochrome text
- End-to-end DB coverage requires `--ignored` run

---

## Strategy Summary

**Total Features Analyzed:** 3

**Test Types Required:**
- Unit tests: REQUIRED for all 3 features
- CLI integration tests (no DB): REQUIRED for Feature 2 (help text, argument errors)
- Integration tests (DB, `#[ignore]`): RECOMMENDED for all 3 features
- Interactive tests (PTY): NOT NEEDED for any feature
- Benchmark tests: NOT NEEDED for any feature

**Estimated Test Count:**
- Unit: ~37 tests (12 Bug #35 + 10 Bug #34 clap + 15 /inspect formatting)
- CLI integration (no DB): 6 tests (Bug #34 help text)
- Integration (DB, `#[ignore]`): 12 tests (4 Bug #35 + 5 Bug #34 + 3 /inspect)
- Total: ~55 tests

**Risk Assessment:**
- HIGH risk gaps: none
- MEDIUM risk gaps: End-to-end Bug #35 paths through `metacommands.rs` call site (unit tests cover the functions; call site correctness is partially code-review dependent)
- LOW risk gaps: DB-connected rendering for /inspect formatting

**Dependencies Required:**
- Live database: Yes, for `#[ignore]` integration tests only. Unit tests and CLI integration tests have no DB dependency.
- Network access: No
- Specific OS: No (Darwin used in development; Linux is CI target)
- Other: `TQ_LOGON` environment variable must be set for `#[ignore]` tests

---

## Test Case Files

| Test Case File | Feature | DB Required | Test Count |
|---------------|---------|-------------|------------|
| `tests/cases/TC-046-001.md` | Bug #35 — `quote_identifier` uppercase | No | 10 unit tests |
| `tests/cases/TC-046-002.md` | Bug #35 — `extract_table_name` word boundaries | No | 8 unit tests |
| `tests/cases/TC-046-003.md` | Bug #35 — end-to-end sample/peek (ignored) | Yes | 4 integration tests |
| `tests/cases/TC-046-004.md` | Bug #34 — clap argument parsing | No | 10 unit tests |
| `tests/cases/TC-046-005.md` | Bug #34 — CLI help text & error messages | No | 6 CLI integration tests |
| `tests/cases/TC-046-006.md` | Bug #34 — end-to-end describe/list/show-indexes (ignored) | Yes | 5 integration tests |
| `tests/cases/TC-046-007.md` | /inspect formatting compliance | No | 15 unit tests |
| `tests/cases/TC-046-INSPECT-INTEGRATION.md` | /inspect end-to-end formatting (ignored) | Yes | 3 integration tests |

---

## Strategy Validation Checklist

- [x] Every feature has complete specification analysis section
- [x] Feature characteristics are classified (not assumed)
- [x] Test strategy is derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis is complete and honest
- [x] Specification coverage map includes all requirements
- [x] Every requirement maps to at least one test type
- [x] Test implementation plan is detailed and actionable
- [x] Coverage sufficiency is assessed
- [x] No hand-waving or vague justifications

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-03-23
**Review Status:** DRAFT
**Submitted for Review:** 2026-03-23

**Reviewer:** tq-project-manager
**Review Status:** PENDING
**Review Date:** —
**Review Comments:** —

**Approval means:**
- Test strategy derived from specifications (not assumptions)
- All required test types identified with clear rationale
- Coverage gaps explicitly identified and assessed
- Implementation plan is detailed and achievable
- Ready to proceed with test implementation
