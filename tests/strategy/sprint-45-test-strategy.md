# Sprint 45 Test Strategy: Helper Bug Fix & Object Inspection

**Created:** 2026-03-23
**Author:** quality-validator
**Sprint:** Sprint 45
**Features:**
1. Bug #32 Fix: Metacommand Semicolon Stripping
2. /inspect Command (Issue #33) - REPL and batch mode
3. Sprint 44 Deferred Items (--force help text, abort message with profile name, debug logging)

---

## Overview

Sprint 45 delivers a bug fix and a new feature:

- **Bug #32 (Feature 1)**: Metacommands fail when users append a trailing semicolon (SQL habit). The fix is a single `trim_end_matches(';')` call in `metacommands.rs`. This is pure-logic and 100% unit-testable without a database.
- **/inspect command (Feature 2)**: A new comprehensive object inspection command that queries multiple DBC views (`DBC.TablesV`, `DBC.ColumnsV`, `DBC.IndicesV`, `DBC.TableSizeV`) and renders a rich multi-section report. The pure-logic helpers (size formatting, skew calculation, name parsing, SQL construction) are unit-testable in isolation. The rendering and DBC queries require a live database. Both REPL and batch modes must be validated.
- **Sprint 44 deferred items (Feature 3)**: Three low-complexity housekeeping fixes (help text string, abort message string, debug log calls). All are static verifiable through `cargo test --lib` and code inspection.

The sprint has a clear two-tier dependency profile: Features 1 and 3 need no database; Feature 2 requires a live Teradata connection for integration tests.

---

## Feature-by-Feature Test Strategy

---

### Feature 1: Bug #32 — Metacommand Semicolon Stripping

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-45-planning.md` — AC-1 through AC-6, Feature 1
- Secondary: `src/commands/repl/metacommands.rs` — `handle_metacommand()` and `handle_metacommand_with_state()`

**Requirements (from acceptance criteria):**
1. AC-1: `/describe tablename;` resolves to table `tablename` (semicolon stripped)
2. AC-2: `/list tables;` shows tables (not "Unknown list subcommand: tables;")
3. AC-3: `/sample dbc.tables;` samples from `dbc.tables`
4. AC-4: `/show indexes tablename;` shows indexes
5. AC-5: All other metacommands with trailing semicolons work correctly
6. AC-6: Unit tests cover semicolon stripping for at least 4 commands

**Feature Characteristics:**

**User Interaction Type:** Pure Logic (internal string normalization algorithm)

**Explanation:** The fix is a string transformation applied to metacommand input before splitting into command parts. No database access is needed to validate that the stripping happens correctly; it can be validated by inspecting the parsed command name and argument that reach the dispatch logic.

**Observable Behavior:**
- Structured data output (the correct command and arguments are dispatched)
- No visual side-effects specific to this fix

**External Dependencies:**
- None (pure logic)

**Validation Challenges:**
- The two entry points (`handle_metacommand` and `handle_metacommand_with_state`) must both be covered; missing one means a code path is untested.
- Regression: ensure non-semicolon inputs still work identically.

**Critical Behaviors to Validate:**
1. Single trailing semicolon is stripped before `split_whitespace()` on the argument
2. Multiple trailing semicolons (`;;`) are stripped completely
3. No semicolon present — behaviour unchanged (regression)
4. Stripping applies to the argument, not the command word itself (e.g., `/list tables;` strips from `tables;`)
5. Both `handle_metacommand` and `handle_metacommand_with_state` apply stripping

#### 2. Test Strategy Derivation

**Decision Tree Results:**

- "Pure Logic" → Unit tests sufficient; no PTY or database needed
- "CLI Batch" path exists (commands dispatch DB calls) but the stripping logic itself is pre-DB; unit tests with mock/captured writer cover the argument parsing layer

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Correct stripping at the parsing boundary before dispatch
- **Approach:** Call the parsing/normalization function directly or test observable output via a writer capture; verify stripped argument reaches the dispatch match arm
- **Rationale:** The fix is a pure string transformation — unit tests are the correct, sufficient, and fastest tool
- **Gap if missing:** No automated coverage of semicolon stripping; the bug could silently regress
- **Necessity:** REQUIRED

**Test Type 2: Interactive Tests (expectrl)**
- **Validates:** REPL end-to-end — that `/describe tablename;` in a live REPL session triggers the describe handler (not an error)
- **Approach:** Start tq REPL via expectrl, send `/describe dbc.tables;`, expect a columns listing rather than an error message
- **Rationale:** Unit tests validate the stripping logic, but an interactive test confirms the wiring is correct end-to-end in the REPL loop
- **Gap if missing:** Integration wiring bug (e.g., stripping applied in wrong place) would not be caught
- **Necessity:** REQUIRED — live database needed for commands that execute DBC queries; the test must use `#[ignore]`

**Test Type 3: Integration Tests (batch)**
- **Validates:** That batch mode commands (e.g., `tq describe dbc.tables`) are not affected (out of scope for this bug — batch describe is a separate code path)
- **Approach:** N/A — the bug is REPL-only; batch mode does not use `metacommands.rs`
- **Necessity:** NOT NEEDED for this feature

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | Pure-logic string fix; fast, precise, no deps | Silent regression possible | MUST IMPLEMENT |
| Interactive tests (expectrl) | REQUIRED | End-to-end wiring in REPL with DB | Integration wiring bug undetected | MUST IMPLEMENT (ignored, DB required) |
| CLI batch integration | NOT NEEDED | Bug is REPL-only; batch has separate code path | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirement | N/A | SKIP |

**Summary:**
- REQUIRED: 2 (unit, interactive/integration)
- NOT NEEDED: 2

#### 4. Specification Coverage Map

| Req ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|--------|-----------------|----------------|--------------|------------|
| AC-1 | `/describe tablename;` strips semicolon | sprint-45-planning.md | Unit + Interactive | TC-045-001, TC-045-007 |
| AC-2 | `/list tables;` matches `tables` subcommand | sprint-45-planning.md | Unit | TC-045-002 |
| AC-3 | `/sample dbc.tables;` samples correctly | sprint-45-planning.md | Unit + Interactive | TC-045-003, TC-045-007 |
| AC-4 | `/show indexes table;` shows indexes | sprint-45-planning.md | Unit | TC-045-004 |
| AC-5 | All other metacommands with semicolons work | sprint-45-planning.md | Unit | TC-045-005 |
| AC-6 | Unit tests cover at least 4 commands | sprint-45-planning.md | Unit | TC-045-001 through TC-045-005 |
| REGRESSION | No semicolon — behavior unchanged | sprint-45-planning.md | Unit | TC-045-006 |
| EDGE | Multiple semicolons `;;` stripped | sprint-45-planning.md | Unit | TC-045-005 |

#### 5. Gap Analysis

**Interactive Tests (database-dependent)**
- **Reason for inclusion:** Required to validate REPL wiring is correct end-to-end
- **Risk if database unavailable:** BLOCKED for that specific test; unit tests still pass and cover the logic
- **Mitigation:** Mark with `#[ignore]`; unit tests provide primary coverage; BLOCKED verdict only affects interactive tests

**Performance/Benchmark Tests**
- **Reason for omission:** No performance requirement; this is a string trim on user input
- **Risk:** LOW
- **Mitigation:** N/A

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/repl/metacommands.rs` `#[cfg(test)]` module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 8 tests
- **Key scenarios to cover:**
  1. Single trailing semicolon on `/describe tablename;`
  2. Single trailing semicolon on `/list tables;` — subcommand matches
  3. Single trailing semicolon on `/sample dbc.tables;`
  4. Single trailing semicolon on `/show indexes table;`
  5. Single trailing semicolon on `/peek table;`
  6. Multiple semicolons `;;` fully stripped
  7. No semicolon — describe still works (regression)
  8. Semicolon on command name itself (not just argument) — only arg semicolons stripped, command detection unchanged
- **Mocking strategy:** Use a `Vec<u8>` writer to capture output; no real DB client needed for parsing-level tests; use `DatabaseClient` stub for commands that would call DB

**Test Type: Interactive / DB Integration Tests**
- **Location:** `tests/integration_tests.rs` (new section, `#[ignore]`)
- **Framework:** Rust integration test support; expectrl for PTY where needed
- **Test count estimate:** 3 tests
- **Key scenarios to cover:**
  1. `/describe dbc.tables;` in REPL returns column listing, not error
  2. `/list tables;` in REPL returns table list, not "unknown subcommand" error
  3. `/sample dbc.tables;` in REPL returns sample rows, not error
- **Setup requirements:** Live Teradata database via `TQ_LOGON` env var

---

### Feature 2: /inspect Command (Issue #33)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-45-planning.md` — AC-1 through AC-10, Feature 2
- Secondary: `src/commands/inspect.rs` (new module)

**Requirements (from acceptance criteria):**
1. AC-1: `/inspect <table>` shows object type (Table, View, Macro, etc.)
2. AC-2: `/inspect <table>` shows columns with types, nullable, default values
3. AC-3: `/inspect <table>` shows primary index structure (PI columns, PPI, NoPI)
4. AC-4: `/inspect <table>` shows secondary indexes if any
5. AC-5: `/inspect <table>` shows table size (CurrentPerm) and skew factor
6. AC-6: `/inspect <view>` shows column info and upstream dependencies
7. AC-7: `/inspect` supports qualified names (`database.object`)
8. AC-8: `tq inspect <object>` batch mode with table/CSV/JSON output
9. AC-9: Tab completion for `/inspect` in REPL
10. AC-10: Helpful error messages for non-existent objects or permission errors

**Feature Characteristics:**

**User Interaction Type:**
- Interactive PTY (REPL `/inspect` command)
- CLI Batch (`tq inspect` subcommand)

**Explanation:** The feature has two interaction surfaces. The REPL surface renders a rich multi-section text report that users read interactively. The batch surface outputs structured data (table/CSV/JSON) that can be piped or scripted.

**Observable Behavior:**
- Visual output in terminal (section headers, column table, size/skew values)
- Structured data output (JSON/CSV in batch mode)
- Database side effects: read-only DBC queries

**External Dependencies:**
- Database connection (DBC.TablesV, DBC.ColumnsV, DBC.IndicesV, DBC.TableSizeV)
- Terminal/PTY (REPL rendering)

**Validation Challenges:**
- "Graceful degradation" (AC-10) requires testing when DBC views are inaccessible — hard to simulate without a controlled database
- Skew calculation is a derived metric from per-AMP data — needs verified formula test
- Multiple DBC views queried independently; any may fail silently — each section must be independently testable
- Tab completion (AC-9) requires PTY simulation

**Critical Behaviors to Validate:**
1. Object type correctly mapped from `TableKind` single-character code to human-readable string
2. Size bytes formatted as human-readable (KB/MB/GB/TB)
3. Skew formula: `(max_amp_size - avg_amp_size) / avg_amp_size * 100`
4. Qualified name `db.obj` parsed into `(database, object)` pair; unqualified `obj` uses session default database
5. Each section (type, columns, indexes, size, deps) fetched and rendered independently — failure in one does not abort others
6. Batch mode respects `--output` flag (table/CSV/JSON)
7. Non-existent object returns clear error (not a Rust panic or raw DB error)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

- "Interactive PTY" checked → Interactive tests (expectrl) REQUIRED for REPL rendering
- "CLI Batch" checked → Integration tests REQUIRED for `tq inspect` batch path
- "Database connection" checked → Integration tests with live database REQUIRED; mark `#[ignore]`
- "Visual output in terminal" checked → Interactive tests OR integration tests with output capture

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Pure-logic helpers: TableKind mapping, size formatting, skew calculation, qualified name parsing, SQL query construction
- **Approach:** Test each helper function in isolation with known inputs and expected outputs
- **Rationale:** These functions have zero external dependencies and predictable outputs; unit tests are fast and precise
- **Gap if missing:** Logic bugs in formatting, skew formula, or name parsing would only surface at runtime
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests with Live Database**
- **Validates:** Full inspect pipeline against real DBC views: object type retrieved, columns listed, indexes listed, size/skew computed, graceful degradation on missing object
- **Approach:** `#[ignore]` tests that call `inspect_object()` or run `tq inspect` as a subprocess against a known test table/view
- **Rationale:** SQL queries against DBC views only work against a real Teradata instance; mocks would not catch SQL syntax errors or schema mismatches
- **Gap if missing:** DBC query errors (wrong column names, permission denials) would not be caught in CI
- **Necessity:** REQUIRED (conditionally — BLOCKED when no database available)

**Test Type 3: Interactive Tests (expectrl)**
- **Validates:** REPL `/inspect` renders the multi-section report correctly, tab completion works for `/inspect`
- **Approach:** Start tq REPL, type `/inspect dbc.tables`, expect section headers and column names to appear; separately test TAB after `/inspect ` produces object completions
- **Rationale:** Visual rendering and tab completion are only observable in a PTY session
- **Gap if missing:** Section header formatting bugs, missing newlines, broken completions would not be caught
- **Necessity:** REQUIRED (conditionally — BLOCKED when no database available)

**Test Type 4: Batch CLI Integration Tests**
- **Validates:** `tq inspect <object>` runs, respects `--output table/csv/json`, exits 0 on success, exits non-zero on missing object
- **Approach:** Spawn `tq inspect dbc.tables` as subprocess, capture stdout, assert format matches selected output flag
- **Rationale:** Batch mode is a separate code path from REPL; must be independently validated
- **Gap if missing:** Batch-only bugs (wrong output format, missing sections in JSON) undetected
- **Necessity:** REQUIRED (conditionally — BLOCKED when no database available)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | Logic helpers (format, skew, parse) have zero deps | Formula bugs, parsing bugs undetected | MUST IMPLEMENT |
| Integration (live DB) | REQUIRED | DBC queries only work against real Teradata | SQL errors, permission issues undetected | MUST IMPLEMENT (ignored, DB required) |
| Interactive (expectrl) | REQUIRED | Visual rendering and tab completion need PTY | Rendering bugs, completion bugs undetected | MUST IMPLEMENT (ignored, DB required) |
| Batch CLI integration | REQUIRED | Batch is a separate code path | Batch-only output bugs undetected | MUST IMPLEMENT (ignored, DB required) |
| Benchmark tests | NOT NEEDED | No performance SLA specified | N/A | SKIP |

**Summary:**
- REQUIRED: 4 (unit, integration, interactive, batch CLI)
- NOT NEEDED: 1

#### 4. Specification Coverage Map

| Req ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|--------|-----------------|----------------|--------------|------------|
| AC-1 | Shows object type | sprint-45-planning.md | Unit + Integration | TC-045-008, TC-045-014 |
| AC-2 | Shows columns with types, nullable, default | sprint-45-planning.md | Unit (SQL construction) + Integration | TC-045-009, TC-045-014 |
| AC-3 | Shows primary index structure | sprint-45-planning.md | Unit (SQL construction) + Integration | TC-045-010, TC-045-014 |
| AC-4 | Shows secondary indexes | sprint-45-planning.md | Integration | TC-045-014 |
| AC-5 | Shows table size and skew | sprint-45-planning.md | Unit (skew formula) + Integration | TC-045-011, TC-045-014 |
| AC-6 | Shows column info and deps for views | sprint-45-planning.md | Integration | TC-045-015 |
| AC-7 | Supports qualified names `db.obj` | sprint-45-planning.md | Unit (name parsing) + Integration | TC-045-012, TC-045-014 |
| AC-8 | Batch mode with table/CSV/JSON output | sprint-45-planning.md | Batch CLI integration | TC-045-017, TC-045-018 |
| AC-9 | Tab completion for /inspect | sprint-45-planning.md | Interactive (expectrl) | TC-045-019 |
| AC-10 | Helpful error for non-existent object | sprint-45-planning.md | Unit (error construction) + Integration | TC-045-013, TC-045-016 |
| GRACEFUL | Graceful degradation on DBC view failure | sprint-45-planning.md | Unit (error handling path) + Integration | TC-045-013 |

#### 5. Gap Analysis

**Database-dependent tests blocked without live connection**
- **Reason:** DBC views require a real Teradata instance
- **What won't be validated without DB:** SQL accuracy, actual data rendering, graceful degradation in production-like scenario
- **Risk:** MEDIUM — unit tests cover logic, but integration coverage is zero without DB
- **Mitigation:** Unit tests cover all pure-logic paths; integration tests are `#[ignore]`
- **Revisit:** Run integration tests whenever a Teradata instance is available

**Graceful degradation on DBC.TableSizeV being inaccessible**
- **Reason:** Simulating view permission denial requires a specifically configured database account
- **Risk:** LOW — the code path can be partially validated via unit tests that simulate an error return from the size query
- **Mitigation:** Unit test that exercises the error branch of size/skew fetching

**Performance/Benchmark Tests**
- **Reason:** No performance SLA defined; DBC query latency is Teradata-side
- **Risk:** LOW
- **Mitigation:** N/A

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/inspect.rs` `#[cfg(test)]` module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 12 tests
- **Key scenarios to cover:**
  1. `TableKind` → display name: `T` → "Table", `V` → "View", `M` → "Macro", unknown → "Unknown (X)"
  2. Size formatting: 0 bytes → "0 B", 500 → "500 B", 1024 → "1.0 KB", 1_048_576 → "1.0 MB", 1_073_741_824 → "1.0 GB", 1_099_511_627_776 → "1.0 TB"
  3. Skew calculation: balanced (all amps equal) → 0.0%, highly skewed → high percentage
  4. Skew with zero average (no data) → returns 0.0% or "N/A" (no divide-by-zero panic)
  5. Qualified name parsing: `"db.obj"` → `("db", "obj")`
  6. Unqualified name: `"obj"` → `(session_db, "obj")` or `("", "obj")`
  7. SQL construction for columns query uses correct DBC.ColumnsV predicate
  8. SQL construction for indexes query uses correct DBC.IndicesV predicate
  9. SQL construction for size query uses correct DBC.TableSizeV predicate
  10. Error returned when object not found (no rows in DBC.TablesV) — error message contains object name
  11. Graceful degradation: size section skipped when query returns error (does not propagate as fatal)
  12. Multiple semicolons at end of inspect argument stripped (reuse of Feature 1 fix)
- **Mocking strategy:** No database mock needed for pure-logic unit tests; error-path tests use `Err(...)` return simulation via direct function calls

**Test Type: Integration Tests with Live Database**
- **Location:** `tests/integration_tests.rs` (new `inspect` section, `#[ignore]`)
- **Framework:** Built-in Rust integration test support
- **Test count estimate:** 4 tests
- **Key scenarios to cover:**
  1. Inspect a known table (`dbc.dbcinfo` or similar) — all sections present
  2. Inspect a known view — shows column info, view definition visible
  3. Inspect non-existent object — returns error with object name, exit code non-zero
  4. Inspect qualified name `dbc.tables` — parses and executes correctly
- **Setup requirements:** Live Teradata via `TQ_LOGON`, `dbc.dbcinfo` or equivalent accessible table

**Test Type: Interactive Tests (expectrl)**
- **Location:** `tests/interactive_tests.rs` (new section, `#[ignore]`)
- **Framework:** expectrl crate
- **Test count estimate:** 2 tests
- **Key scenarios to cover:**
  1. Type `/inspect dbc.tables` in REPL — expect "Object Type" header and column listing to appear
  2. Type `/inspect ` then TAB in REPL — expect object name completions to appear
- **Implementation notes:** Use 5-second timeout for DB round-trips; expect section headers as anchors

**Test Type: Batch CLI Integration Tests**
- **Location:** `tests/integration_tests.rs` (new `inspect_batch` section, `#[ignore]`)
- **Framework:** `std::process::Command` to spawn `tq inspect`
- **Test count estimate:** 3 tests
- **Key scenarios to cover:**
  1. `tq inspect dbc.dbcinfo` (table output) — stdout contains column names, exit 0
  2. `tq inspect dbc.dbcinfo --output csv` — stdout is valid CSV, exit 0
  3. `tq inspect nonexistent_table_xyz` — exit code non-zero, stderr contains helpful message
- **Setup requirements:** Compiled `tq` binary in `target/debug/` or `target/release/`; live Teradata via `TQ_LOGON`

---

### Feature 3: Sprint 44 Deferred Items

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-45-planning.md` — Feature 3, AC-1 through AC-4

**Requirements:**
1. AC-1: `docs/design/connection-management.md` updated to match actual `resolve_driver_lib_dir` signature
2. AC-2: `--force` description changed to "Skip confirmation prompt"
3. AC-3: Abort message includes profile name: "Aborted. Profile 'NAME' was not deleted."
4. AC-4: `log::debug!` calls present at each fallback step in `resolve_driver_lib_dir`

**Feature Characteristics:**

**User Interaction Type:**
- AC-2, AC-3: CLI Batch (profile subcommand output)
- AC-4: Pure Logic (logging instrumentation)
- AC-1: Documentation (no runtime behavior)

**Observable Behavior:**
- Structured data output (help text string for AC-2)
- State management visible via log output for AC-4

**External Dependencies:**
- None for AC-2, AC-3 (string literals in source code)
- None for AC-4 (log macros compile unconditionally)
- No database needed

**Validation Challenges:**
- AC-1 is documentation drift; no automated test can verify doc accuracy — requires code inspection
- AC-4 debug log presence is verifiable via code inspection or by running with `RUST_LOG=debug` and checking stderr

**Critical Behaviors to Validate:**
1. Help text for `--force` contains "Skip confirmation prompt"
2. Abort output contains the profile name in single quotes
3. Debug log calls present at each fallback point in `resolve_driver_lib_dir`

#### 2. Test Strategy Derivation

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** AC-2 help string literal; AC-3 abort message format including profile name
- **Approach:** Parse CLI args with `--help` output capture OR test the abort message construction function directly
- **Necessity:** REQUIRED

**Test Type 2: Code Inspection (manual)**
- **Validates:** AC-1 doc accuracy; AC-4 debug log presence
- **Approach:** Read source code and verify `log::debug!` calls exist at each fallback; read doc and verify signature matches
- **Necessity:** REQUIRED (no automated alternative for doc accuracy)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | Help text and abort message are verifiable strings | String regressions undetected | MUST IMPLEMENT |
| Code inspection | REQUIRED | AC-1 doc drift and AC-4 log presence require reading source | Silently wrong doc or missing logs | DOCUMENT PROCEDURE |
| Integration tests | NOT NEEDED | No runtime behavior change requiring DB | N/A | SKIP |
| Interactive tests | NOT NEEDED | No visual rendering changes | N/A | SKIP |

#### 4. Specification Coverage Map

| Req ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|--------|-----------------|----------------|--------------|------------|
| AC-1 | Design doc matches `resolve_driver_lib_dir` signature | sprint-45-planning.md | Code Inspection | TC-045-020 |
| AC-2 | `--force` description is "Skip confirmation prompt" | sprint-45-planning.md | Unit | TC-045-021 |
| AC-3 | Abort message includes profile name | sprint-45-planning.md | Unit | TC-045-022 |
| AC-4 | `log::debug!` at each fallback step | sprint-45-planning.md | Code Inspection + Unit | TC-045-023 |

#### 5. Gap Analysis

**AC-1 Documentation Accuracy**
- **Reason for code-inspection-only:** No automated tool verifies prose accuracy
- **Risk:** LOW — documentation drift has no runtime impact
- **Mitigation:** Code inspection at test execution time

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/cli.rs::tests` (for `--force` help text); `src/commands/profile.rs::tests` (for abort message)
- **Framework:** Built-in Rust test framework
- **Test count estimate:** 3 tests
- **Key scenarios to cover:**
  1. `--force` help text on `tq profile delete` contains "Skip confirmation prompt"
  2. Abort message when deleting profile "myprofile" contains `'myprofile'`
  3. `resolve_driver_lib_dir` function body contains `log::debug!` macro calls (verified via compile — the log macro expands without a DB connection)
- **Mocking strategy:** None needed; string literals and log macros are compile-time verifiable

---

## Test Case Inventory

### TC-045-001: Semicolon Stripped from /describe Argument
- **Type:** Unit
- **Priority:** Critical (Bug fix regression)
- **Feature:** Bug #32
- **DB Required:** No
- **Description:** Verify that `/describe tablename;` parses to argument `"tablename"` (not `"tablename;"`)

### TC-045-002: Semicolon Stripped from /list Subcommand
- **Type:** Unit
- **Priority:** Critical (Bug fix regression)
- **Feature:** Bug #32
- **DB Required:** No
- **Description:** Verify that `/list tables;` parses subcommand as `"tables"` (not `"tables;"`) and does not produce "Unknown list subcommand" error

### TC-045-003: Semicolon Stripped from /sample Argument
- **Type:** Unit
- **Priority:** Critical (Bug fix regression)
- **Feature:** Bug #32
- **DB Required:** No
- **Description:** Verify that `/sample dbc.tables;` parses table argument as `"dbc.tables"` (not `"dbc.tables;"`)

### TC-045-004: Semicolon Stripped from /show indexes Argument
- **Type:** Unit
- **Priority:** High
- **Feature:** Bug #32
- **DB Required:** No
- **Description:** Verify that `/show indexes table;` parses table argument as `"table"` (not `"table;"`)

### TC-045-005: Multiple Trailing Semicolons Fully Stripped
- **Type:** Unit
- **Priority:** High
- **Feature:** Bug #32
- **DB Required:** No
- **Description:** Verify that `/describe a;;` parses argument as `"a"` with all trailing semicolons removed

### TC-045-006: No Semicolon — Regression Test
- **Type:** Unit
- **Priority:** Critical (Regression)
- **Feature:** Bug #32
- **DB Required:** No
- **Description:** Verify that `/describe tablename` (no semicolon) still parses correctly — behavior unchanged after fix

### TC-045-007: Semicolon Stripping End-to-End in REPL
- **Type:** Integration (DB required, `#[ignore]`)
- **Priority:** High
- **Feature:** Bug #32
- **DB Required:** Yes
- **Description:** In a live REPL session, verify that `/describe dbc.tables;` returns a column listing and not "Unknown command" or similar error

### TC-045-008: TableKind Code to Display Name Mapping
- **Type:** Unit
- **Priority:** Critical
- **Feature:** /inspect command
- **DB Required:** No
- **Description:** Verify that each TableKind code (`T`, `V`, `M`, `O`, etc.) maps to the correct human-readable display name; unknown codes produce a fallback string

### TC-045-009: Column Section SQL Construction
- **Type:** Unit
- **Priority:** High
- **Feature:** /inspect command
- **DB Required:** No
- **Description:** Verify that the SQL generated for the columns section queries `DBC.ColumnsV` with the correct `DatabaseName` and `TableName` predicates

### TC-045-010: Index Section SQL Construction
- **Type:** Unit
- **Priority:** High
- **Feature:** /inspect command
- **DB Required:** No
- **Description:** Verify that the SQL generated for the indexes section queries `DBC.IndicesV` with correct predicates and orders results correctly

### TC-045-011: Size Formatting and Skew Calculation
- **Type:** Unit
- **Priority:** High
- **Feature:** /inspect command
- **DB Required:** No
- **Description:** Verify byte-to-human-readable formatting (0B, KB, MB, GB, TB); verify skew formula `(max - avg) / avg * 100`; verify no divide-by-zero when avg is zero

### TC-045-012: Qualified Name Parsing
- **Type:** Unit
- **Priority:** High
- **Feature:** /inspect command
- **DB Required:** No
- **Description:** Verify `"db.obj"` parses to database=`"db"`, object=`"obj"`; verify unqualified `"obj"` uses session default; verify name with extra dots handled gracefully

### TC-045-013: Error Handling for Non-Existent Object and Missing DBC Views
- **Type:** Unit
- **Priority:** High
- **Feature:** /inspect command
- **DB Required:** No
- **Description:** Verify that when the object type query returns zero rows, a helpful error message including the object name is produced; verify size section failure does not abort the entire inspect output

### TC-045-014: Full /inspect on a Known Table (Integration)
- **Type:** Integration (DB required, `#[ignore]`)
- **Priority:** Critical
- **Feature:** /inspect command
- **DB Required:** Yes
- **Description:** Run `/inspect dbc.dbcinfo` (or equivalent accessible table) in REPL; verify output contains: object type header, column listing with type info, index section header, size section header

### TC-045-015: /inspect on a Known View (Integration)
- **Type:** Integration (DB required, `#[ignore]`)
- **Priority:** High
- **Feature:** /inspect command
- **DB Required:** Yes
- **Description:** Run `/inspect` on a known view; verify output contains column info and a "Dependencies" or "View Definition" section; verify no crash

### TC-045-016: /inspect Non-Existent Object (Integration)
- **Type:** Integration (DB required, `#[ignore]`)
- **Priority:** High
- **Feature:** /inspect command
- **DB Required:** Yes
- **Description:** Run `tq inspect nonexistent_xyz_table`; verify exit code is non-zero and stderr/stdout contains a helpful message with the object name

### TC-045-017: Batch Mode Table Output (Integration)
- **Type:** Integration (DB required, `#[ignore]`)
- **Priority:** Critical
- **Feature:** /inspect command — batch mode
- **DB Required:** Yes
- **Description:** Run `tq inspect dbc.dbcinfo` (default table output); verify stdout contains column headers and at least one data row; verify exit code 0

### TC-045-018: Batch Mode CSV and JSON Output (Integration)
- **Type:** Integration (DB required, `#[ignore]`)
- **Priority:** High
- **Feature:** /inspect command — batch mode
- **DB Required:** Yes
- **Description:** Run `tq inspect dbc.dbcinfo --output csv` and `--output json`; verify output is valid CSV/JSON; verify exit code 0

### TC-045-019: Tab Completion for /inspect in REPL (Interactive)
- **Type:** Interactive / expectrl (DB required, `#[ignore]`)
- **Priority:** Medium
- **Feature:** /inspect command — tab completion
- **DB Required:** Yes
- **Description:** In a live REPL, type `/inspect ` and press TAB; verify that database or object name completions appear

### TC-045-020: Design Doc Matches resolve_driver_lib_dir Signature (Code Inspection)
- **Type:** Code Inspection (manual)
- **Priority:** Low
- **Feature:** Sprint 44 deferred — doc drift
- **DB Required:** No
- **Description:** Read `docs/design/connection-management.md` and `src/db/client.rs`; verify documented function signature matches actual implementation

### TC-045-021: --force Help Text Verification
- **Type:** Unit
- **Priority:** Medium
- **Feature:** Sprint 44 deferred — --force description
- **DB Required:** No
- **Description:** Verify that the `--force` flag on `tq profile delete` has description text containing "Skip confirmation prompt"

### TC-045-022: Abort Message Includes Profile Name
- **Type:** Unit
- **Priority:** Medium
- **Feature:** Sprint 44 deferred — abort message
- **DB Required:** No
- **Description:** Verify that the abort message when user cancels profile deletion contains the profile name in single quotes (e.g., "Aborted. Profile 'myprofile' was not deleted.")

### TC-045-023: Debug Logging in resolve_driver_lib_dir
- **Type:** Unit / Code Inspection
- **Priority:** Medium
- **Feature:** Sprint 44 deferred — debug logging
- **DB Required:** No
- **Description:** Verify that `log::debug!` macro calls are present at each fallback step in `resolve_driver_lib_dir` by running with `RUST_LOG=debug` and inspecting stderr output (or via code inspection)

---

## Test Execution Plan

### Phase 1: Unit Tests (No Database Required)

Run all unit tests embedded in source:

```bash
cargo test --lib 2>&1
```

Expected: All tests pass, including TC-045-001 through TC-045-006 (semicolon stripping) and TC-045-008 through TC-045-013 (inspect unit logic) and TC-045-021 through TC-045-023 (deferred items).

### Phase 2: Integration Tests (No Database Required)

Run integration tests that do not require a live database:

```bash
cargo test --test integration_tests 2>&1
```

Expected: All non-ignored integration tests pass.

### Phase 3: Database Integration Tests (Live Database Required)

If a live Teradata database is available via `TQ_LOGON`:

```bash
export TQ_LOGON="user:password@host:1025/database"
cargo test --test integration_tests -- --ignored 2>&1
```

Covers: TC-045-007, TC-045-014 through TC-045-018.

Expected: All ignored integration tests pass.

### Phase 4: Interactive Tests (Live Database + PTY Required)

If a live Teradata database is available:

```bash
cargo test --test interactive_tests -- --ignored 2>&1
```

Covers: TC-045-019 (tab completion).

### Phase 5: Code Inspection (Manual)

Manually verify TC-045-020: compare `docs/design/connection-management.md` against `src/db/client.rs` `resolve_driver_lib_dir` function signature and fallback steps.

---

## New Testing Tools Required

No new testing tools are required for this sprint. The existing infrastructure is sufficient:
- Built-in Rust `#[test]` framework for unit tests
- `#[ignore]` annotation for database-dependent tests
- expectrl crate (already used in prior sprints) for interactive PTY tests
- `std::process::Command` for batch CLI subprocess tests

---

## Strategy Summary

**Total Features Analyzed:** 3

**Test Types Required:**
- Unit tests: REQUIRED — all three features
- Integration tests (live DB): REQUIRED — Feature 2 (/inspect), Feature 1 (REPL wiring)
- Interactive tests (expectrl, live DB): REQUIRED — Feature 2 (rendering, tab completion)
- Batch CLI integration: REQUIRED — Feature 2 (batch mode)
- Benchmark tests: NOT NEEDED — none of the features have performance SLAs

**Estimated Test Count:**
- Unit: 23 tests (TC-045-001 through TC-045-013, TC-045-021 through TC-045-023)
- Integration (DB, ignored): 7 tests (TC-045-007, TC-045-014 through TC-045-018)
- Interactive (PTY, ignored): 2 tests (TC-045-019)
- Code inspection (manual): 1 procedure (TC-045-020)
- **Total: 33 test cases**

**Database Dependency Split:**
- No database required: 24 test cases (TC-045-001 through TC-045-013, TC-045-020 through TC-045-023)
- Live database required: 9 test cases (TC-045-007, TC-045-014 through TC-045-019)

**Risk Assessment:**
- HIGH risk gaps: none
- MEDIUM risk gaps: integration tests for /inspect BLOCKED if no database available; unit tests provide logic coverage
- LOW risk gaps: doc drift (AC-1), graceful degradation edge case when DBC view access denied

**Dependencies Required:**
- Live database: Yes — for TC-045-007, TC-045-014 through TC-045-019
- Network access: No
- Specific OS: No
- Other: expectrl crate for TC-045-019; compiled `tq` binary for batch tests

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
