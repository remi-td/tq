# Sprint 47 Test Strategy: Tech Debt Elimination & Command Enrichment

**Created:** 2026-03-23
**Author:** quality-validator
**Sprint:** Sprint 47
**Features:**
1. Bug #36: /inspect DDL & Column Type Fix for views
2. Shared Helpers Extraction (format_helpers.rs)
3. REPL Delegation to Batch Modules (/describe, /list)
4. Enrich `tq describe` Output (header, Comments, Indexes, JSON)
5. Enrich `tq list` Output (Owner/Type for databases, Rows/Size for tables, JSON)
6. Enrich `tq show-indexes` Output (Primary/Secondary sections, UPI/NUPI labels, JSON)

---

## Overview

Sprint 47 is a quality-consolidation sprint. The six features fall into three categories:

- **Pure logic features (no DB)**: The shared helpers extraction (Feature 2) and all formatting enrichment output logic (Features 4, 5, 6) are pure functions or writer-injectable render functions. These are 100% unit-testable without a live database.
- **REPL delegation (DB required, interactive)**: Feature 3 involves the REPL metacommand wiring. Testing requires a live database to drive the REPL and verify that the delegation path produces identical output to the batch modules. These tests use `#[ignore]` and expectrl.
- **Bug fix (DB required for full coverage)**: Bug #36 (Feature 1) has pure-logic helpers testable as unit tests, but end-to-end view DDL retrieval requires a live database where a view exists. These end-to-end tests use `#[ignore]`.

All six features are independently unit-testable through their formatting and logic functions. Database-dependent behaviors are cleanly separated into `#[ignore]` integration tests.

---

## Feature-by-Feature Test Strategy

---

### Feature 1: Bug #36 — /inspect DDL & Column Type Fix for Views

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-47-planning.md` Feature 1, AC-1 through AC-4
- Secondary: `docs/specifications/repl.md` — object inspection section
- Secondary: `src/commands/inspect.rs` — `query_definition()`, `query_columns()`, `query_object_type()`

**Requirements (from acceptance criteria):**
1. AC-1: `/inspect dbc.tables` (a view) shows complete view definition text — not garbled or truncated
2. AC-2: `/inspect` on a view shows column types from DBC.ColumnsV (not [NULL])
3. AC-3: `/inspect` on a macro shows full macro definition
4. AC-4: Unit tests cover DDL retrieval logic for views and macros

**Feature Characteristics:**

**User Interaction Type:** Mixed — Pure Logic (query_definition SQL construction) + Interactive PTY (REPL /inspect display)

**Explanation:** The root cause of Bug #36 is either in the SQL used to retrieve DDL (a pure-logic concern: the SHOW VIEW statement must concatenate multi-row results) or in the column type query for views (which reads from DBC.ColumnsV and must not return NULL). The SQL construction is testable via unit tests. Verifying the fix in a live REPL requires a DB connection and an actual view/macro object.

**Observable Behavior:**
- [x] Visual output in terminal (REPL /inspect output with full DDL)
- [x] Structured data output (tq inspect --format json includes definition)

**External Dependencies:**
- [x] Database connection — required for AC-1, AC-2, AC-3 (end-to-end)
- None for AC-4 (unit tests on logic helpers)

**Validation Challenges:**
- The DDL truncation bug may manifest in how multi-row SHOW VIEW results are concatenated. Unit tests must simulate multi-row results.
- Column type returning [NULL] for views is a SQL issue (wrong table or missing join). Unit tests verify the query string construction, but correctness requires a real view.
- Mac environment may have no live Teradata — end-to-end tests are `#[ignore]`.

**Critical Behaviors to Validate:**
1. `query_definition()` concatenates all rows from SHOW VIEW result (not just first row) — multi-row scenario
2. `query_definition()` uses correct SQL: `SHOW VIEW "db"."obj"` format with quoted identifiers
3. Column type query uses `DBC.ColumnsV` (not `DBC.Columns`) to avoid NULL on views
4. `/inspect macroname` produces full macro definition via `SHOW MACRO "db"."obj"`

#### 2. Test Strategy Derivation

**Decision Tree Results:**
- "Pure Logic" for SQL construction → Unit tests REQUIRED for AC-4
- "Database connection" for AC-1, AC-2, AC-3 → `#[ignore]` integration tests RECOMMENDED
- "Interactive PTY" for REPL display → `#[ignore]` interactive tests for full end-to-end

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** AC-4 — that `query_definition()` multi-row concatenation logic is correct; that [NULL] rows are filtered; that SQL construction uses quoted identifiers for view and macro.
- **Approach:** Construct mock result rows in Rust (Vec<Vec<Value>>) simulating multi-row SHOW output; call the row-concatenation logic; assert the combined string is correct. Assert SQL format via string contains checks on the generated query.
- **Rationale:** The row-concatenation logic is pure; mock rows are sufficient to verify the fix.
- **Gap if missing:** AC-4 explicitly requires unit tests. Multi-row join bug is not caught without explicit multi-row test.
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (DB, `#[ignore]`)**
- **Validates:** AC-1, AC-2, AC-3 — actual view/macro DDL is fully retrieved without truncation; column types are not NULL on views.
- **Approach:** Spawn `tq inspect dbc.tables` (a well-known Teradata view); assert stdout contains a complete CREATE VIEW text, not ending prematurely. Spawn `tq inspect` on a known macro and assert full macro definition appears.
- **Rationale:** Only a live database can verify that the SQL fix works end-to-end against real Teradata catalog data.
- **Gap if missing:** AC-1 through AC-3 uncovered; SQL fix might have a different bug in the call path not caught by unit tests.
- **Necessity:** RECOMMENDED (DB required, `#[ignore]`)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | Validates multi-row DDL concatenation and NULL filtering (AC-4) | AC-4 uncovered; concatenation bug not caught | MUST IMPLEMENT |
| Integration tests (DB) | RECOMMENDED | Validates end-to-end DDL retrieval on real view/macro (AC-1, AC-2, AC-3) | Complete DDL validation impossible without DB | SHOULD IMPLEMENT (ignored) |
| Interactive tests (PTY) | NOT NEEDED | REPL /inspect is a thin wrapper over the same logic; integration test is sufficient | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirements | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| AC-1 | `/inspect view` shows complete DDL (not truncated/garbled) | sprint-47-planning.md Feature 1 | Unit + Integration(ignored) | TC-047-001 |
| AC-2 | `/inspect view` shows column types (not [NULL]) | sprint-47-planning.md Feature 1 | Integration(ignored) | TC-047-001 |
| AC-3 | `/inspect macro` shows full macro definition | sprint-47-planning.md Feature 1 | Integration(ignored) | TC-047-001 |
| AC-4 | Unit tests cover DDL retrieval logic | sprint-47-planning.md Feature 1 | Unit | TC-047-001 |

#### 5. Gap Analysis

**Interactive PTY Tests**
- **Reason for omission:** `/inspect` in REPL delegates to the same `inspect_object()` function as batch mode. Unit + integration tests sufficiently cover the logic path. Adding PTY tests would duplicate coverage without catching new bugs.
- **Risk assessment:** LOW
- **Mitigation:** Integration test spawns `tq inspect` batch command, which exercises the same code path.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/inspect.rs` `#[cfg(test)]` module (extend existing)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 6 tests
- **Key scenarios:**
  1. Multi-row DDL: simulate SHOW VIEW returning 3 rows; assert all 3 are concatenated in output
  2. NULL row filtering: SHOW VIEW row with "[NULL]" value is excluded from concatenation
  3. Empty result: SHOW VIEW returns 0 rows; `query_definition` returns empty string (no panic)
  4. View SQL format: generated SQL contains `SHOW VIEW` (not SHOW TABLE)
  5. Macro SQL format: generated SQL contains `SHOW MACRO`
  6. Unknown kind: `query_definition` called with kind="T" returns empty string (tables have no definition)

**Test Type: Integration Tests (DB, `#[ignore]`)**
- **Location:** `tests/integration_tests.rs` or `tests/inspect_ddl_47.rs`
- **Framework:** `std::process::Command`
- **Test count estimate:** 4 tests
- **Key scenarios:**
  1. `tq inspect dbc.tables` exits 0, stdout contains "CREATE VIEW" or "REPLACE VIEW" (AC-1)
  2. `tq inspect dbc.tables` stdout does not contain "[NULL]" in the column type section (AC-2)
  3. `tq inspect dbc.tables --format json` contains non-empty "columns" array with actual type strings (AC-2)
  4. `tq inspect <known_macro>` stdout contains macro body text (AC-3)

#### 7. Coverage Sufficiency Assessment

- Unit tests validate: DDL concatenation logic, NULL filtering, SQL format correctness
- Integration tests validate: end-to-end DDL retrieval and column type resolution on real data
- Combined coverage: comprehensive (unit tests cover the root-cause logic; integration tests cover end-to-end)

**Gaps:** Without a DB, AC-1 through AC-3 are not directly executable. This is documented and acceptable.

---

### Feature 2: Shared Helpers Extraction (format_helpers.rs)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-47-planning.md` Feature 2, AC-1 through AC-6
- Secondary: Sprint 46 review recommendations #7 and #8
- Secondary: `src/commands/inspect.rs` — existing `json_escape`, `csv_escape`, `truncate_str`

**Requirements (from acceptance criteria):**
1. AC-1: `json_escape()` exists once in format_helpers.rs, used by all 4 command modules
2. AC-2: `csv_escape()` exists once in format_helpers.rs, used by all 4 command modules
3. AC-3: `parse_table_name()` exists once in format_helpers.rs (or identifiers.rs)
4. AC-4: `truncate_str()` exists once with proper UTF-8 `char_indices()` handling
5. AC-5: Zero code duplication of these functions across the codebase
6. AC-6: All existing tests pass after extraction (no regression)

**Feature Characteristics:**

**User Interaction Type:** Pure Logic (string transformation utilities)

**Explanation:** All four functions are pure string transformations. They accept `&str` and return `String` or `(String, String)`. No I/O, no database, no terminal. The extraction refactoring must not change behavior.

**Observable Behavior:**
- [x] Structured data output (function return values — strings)

**External Dependencies:**
- None for any AC — pure logic only

**Validation Challenges:**
- `truncate_str` has a UTF-8 byte-boundary bug in the existing implementation (`&s[..max_len - 3]` may slice mid-codepoint). The fix uses `char_indices()`. Test must exercise multi-byte Unicode characters that would previously panic.
- Zero-duplication (AC-5) is verified by `grep` over the codebase — it's a structural check, not a logic test.
- AC-6 (no regression) requires running the full test suite after extraction.

**Critical Behaviors to Validate:**
1. `json_escape` correctly escapes `\`, `"`, `\n`, `\r`, `\t`
2. `csv_escape` wraps in quotes when value contains comma, quote, or newline; doubles embedded quotes
3. `parse_table_name` splits on first dot; handles unqualified names; returns correct db/obj parts
4. `truncate_str` truncates ASCII strings at exact character boundary with `...`
5. `truncate_str` with a 3-byte UTF-8 character (e.g., `é`, CJK) does not panic and does not produce invalid UTF-8
6. `truncate_str` with max_len=3 returns exactly 3 characters (the ellipsis itself)
7. `truncate_str` with max_len=2 returns exactly 2 dots (edge case: max_len <= 3)

#### 2. Test Strategy Derivation

**Decision Tree Results:**
- "Pure Logic" for all 4 helpers → Unit tests REQUIRED, no other test types needed

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** All 7 ACs. Direct function calls asserting return values. Includes Unicode boundary test for `truncate_str`.
- **Approach:** In `src/commands/format_helpers.rs` `#[cfg(test)]` module, call each function with typical, empty, maximum-length, and pathological inputs. For Unicode: use a string composed of 3-byte UTF-8 characters (`é`, CJK) and truncate to a length that falls mid-codepoint under the old implementation.
- **Rationale:** Pure functions; full coverage with unit tests.
- **Gap if missing:** UTF-8 boundary bug in `truncate_str` is the reported regression — without the Unicode test, the bug may silently persist.
- **Necessity:** REQUIRED

**Test Type 2: Structural Duplication Check (Bash)**
- **Validates:** AC-5 — zero duplicate function definitions across `describe.rs`, `list.rs`, `show_indexes.rs`, `inspect.rs`
- **Approach:** Run `grep -c "fn json_escape" src/commands/*.rs` and assert count is 1 (only in format_helpers.rs). Repeat for `csv_escape`, `parse_table_name`, `truncate_str`.
- **Rationale:** Code review alone is insufficient; automated grep catches accidental re-introduction.
- **Gap if missing:** AC-5 silently fails if a duplicate is accidentally left in a module.
- **Necessity:** REQUIRED (runs in test execution phase as shell assertion)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | Validates all 4 helper functions including Unicode safety | UTF-8 boundary bug undetected, behavior regression | MUST IMPLEMENT |
| Structural grep check | REQUIRED | Validates zero-duplication (AC-5) | Duplicate definitions not caught | MUST IMPLEMENT (in test report) |
| Integration tests (DB) | NOT NEEDED | No DB interaction for these pure functions | N/A | SKIP |
| Interactive tests (PTY) | NOT NEEDED | No terminal interaction | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| AC-1 | `json_escape()` in format_helpers.rs used by all 4 modules | sprint-47-planning.md Feature 2 | Unit + Structural grep | TC-047-002 |
| AC-2 | `csv_escape()` in format_helpers.rs used by all 4 modules | sprint-47-planning.md Feature 2 | Unit + Structural grep | TC-047-002 |
| AC-3 | `parse_table_name()` in format_helpers.rs once | sprint-47-planning.md Feature 2 | Unit + Structural grep | TC-047-002 |
| AC-4 | `truncate_str()` with UTF-8 char_indices() | sprint-47-planning.md Feature 2 | Unit (Unicode test) | TC-047-002 |
| AC-5 | Zero duplication | sprint-47-planning.md Feature 2 | Structural grep | TC-047-002 |
| AC-6 | All existing tests pass | sprint-47-planning.md Feature 2 | Full `cargo test --lib` | TC-047-002 |

#### 5. Gap Analysis

**Semantic equivalence verification**
- After extraction, the implementations might be subtly changed (e.g., different regex, different escape order). Unit tests verify each function's behavior explicitly, preventing silent semantic drift.
- **Risk assessment:** LOW — functions are simple string transforms; unit tests cover all branches.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/format_helpers.rs` `#[cfg(test)]` module (new file)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 20 tests
- **Key scenarios:**
  1. `json_escape("")` → `""` (empty)
  2. `json_escape("hello")` → `"hello"` (no special chars)
  3. `json_escape("he\"llo")` → `"he\\\"llo"` (embedded quote)
  4. `json_escape("line\nnew")` → `"line\\nnew"` (newline)
  5. `json_escape("tab\there")` → `"tab\\there"` (tab)
  6. `json_escape("back\\slash")` → `"back\\\\slash"` (backslash)
  7. `csv_escape("hello")` → `"hello"` (no special chars)
  8. `csv_escape("hello,world")` → `"\"hello,world\""` (comma)
  9. `csv_escape("say \"hi\"")` → `"\"say \"\"hi\"\"\""` (embedded quotes)
  10. `csv_escape("line\nnew")` → quoted (newline)
  11. `parse_table_name("mytable")` → `(None, "mytable")` (unqualified)
  12. `parse_table_name("mydb.mytable")` → `(Some("mydb"), "mytable")` (qualified)
  13. `parse_table_name("a.b.c")` → `(Some("a"), "b.c")` (first dot wins)
  14. `truncate_str("short", 10)` → `"short"` (no truncation needed)
  15. `truncate_str("exactly10c", 10)` → `"exactly10c"` (exact fit)
  16. `truncate_str("this is a long string", 10)` → `"this is..."` (ASCII truncation)
  17. `truncate_str("éàü_long_string", 7)` → truncates at char boundary (not byte), no panic, valid UTF-8
  18. `truncate_str("中文很长的字符串", 6)` → valid UTF-8 result (CJK 3-byte chars)
  19. `truncate_str("ab", 3)` → `"ab"` (shorter than max_len, no ellipsis)
  20. `truncate_str("abc", 2)` → `".."` (max_len <= 3, all dots)

**Structural Grep Check**
- **Location:** Documented in TC-047-002.md; executed during test report phase
- **Commands:**
  ```bash
  grep -c "fn json_escape" src/commands/describe.rs src/commands/list.rs src/commands/show_indexes.rs src/commands/inspect.rs
  grep -c "fn csv_escape" src/commands/describe.rs src/commands/list.rs src/commands/show_indexes.rs src/commands/inspect.rs
  grep -c "fn truncate_str" src/commands/describe.rs src/commands/list.rs src/commands/show_indexes.rs src/commands/inspect.rs
  ```
  Each count must be 0 (no duplicates in the command modules).

#### 7. Coverage Sufficiency Assessment

- Unit tests validate: all 4 helper functions with ASCII, empty, edge-case, and Unicode inputs
- Structural grep validates: zero duplication (AC-5)
- Full test suite validates: no regression (AC-6)
- Combined coverage: comprehensive — no gaps for this feature

---

### Feature 3: REPL Delegation to Batch Modules

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-47-planning.md` Feature 3, AC-1 through AC-4
- Secondary: Sprint 46 review recommendation #6
- Secondary: `src/commands/repl/metacommands.rs` — `/describe` and `/list` handlers

**Requirements (from acceptance criteria):**
1. AC-1: `/describe <table>` in REPL calls `describe::execute_for_repl()`
2. AC-2: `/list databases`, `/list tables`, `/list views` in REPL calls `list::execute_for_repl()`
3. AC-3: REPL output is identical to previous behavior (no regression)
4. AC-4: Existing REPL tests pass without modification

**Feature Characteristics:**

**User Interaction Type:** Interactive PTY (REPL metacommand execution in a live terminal session)

**Explanation:** The `/describe` and `/list` metacommands run inside the REPL loop, which requires a live PTY, a database connection, and actual user input. The delegation refactoring must not change user-observable output. Because the REPL is interactive and stateful, the only way to verify AC-3 ("identical output") is to run the REPL with both the old code path (before delegation) and the new code path (after delegation) against a real database. Since we cannot run both simultaneously, the test strategy compares the new REPL output against the same output produced by the batch commands (`tq describe`, `tq list`), which now share the same code path.

**Observable Behavior:**
- [x] Visual output in terminal (REPL metacommand output)
- [x] State management (REPL session connection)

**External Dependencies:**
- [x] Database connection — required for all 4 ACs
- [x] Terminal/PTY — REPL requires a real PTY process

**Validation Challenges:**
- Cannot mock the database for delegation tests — the whole point is that the same function is called.
- Output identity comparison is fragile if the REPL adds extra newlines or prompts around the output. Tests must strip REPL chrome (prompts, blank lines) before comparing.
- Flakiness risk from PTY timing — tests must use explicit `wait_for_prompt()` patterns.

**Critical Behaviors to Validate:**
1. `/describe dbc.tables` in REPL produces column list output (not an error, not empty)
2. `/list databases` in REPL produces database list with entries (not an error)
3. `/list tables` in REPL produces table list (not an error)
4. `/list views` in REPL produces view list (not an error)
5. No regression: `/describe` and `/list` still work (AC-4 — existing tests pass)

#### 2. Test Strategy Derivation

**Decision Tree Results:**
- "Interactive PTY" checked → Interactive tests (expectrl) REQUIRED
- "Database connection" checked → `#[ignore]` required

**Derived Test Types:**

**Test Type 1: Unit Tests (delegation call path)**
- **Validates:** That `metacommands.rs` handler function bodies call `describe::execute_for_repl` or `list::execute_for_repl` rather than inline logic — verified through code inspection and compile-time test.
- **Approach:** Check that the function name `execute_for_repl` is called from within the metacommand handlers. This is primarily a code-structure test, best verified as part of the compilation check.
- **Rationale:** Low-value as a standalone test since the compile will fail if the function doesn't exist, but useful as documentation of the requirement.
- **Gap if missing:** Delegation wiring might silently call the wrong function variant.
- **Necessity:** NOT NEEDED (compile-time verification is sufficient; behavioral tests are in interactive tests)

**Test Type 2: Interactive Tests (expectrl, `#[ignore]`)**
- **Validates:** AC-1 through AC-3 — REPL `/describe` and `/list` produce valid output after delegation wiring
- **Approach:** Spawn `tq repl`, send `/describe dbc.tables`, wait for prompt, assert output contains column-like content (column names, types). Repeat for `/list databases`, `/list tables`, `/list views`.
- **Rationale:** Only PTY tests can verify what the user actually sees in the REPL. Batch-mode tests do not cover the REPL code path in metacommands.rs.
- **Gap if missing:** AC-1 through AC-3 are fully uncovered. The delegation wiring might call the function incorrectly or format output differently.
- **Necessity:** REQUIRED (DB required, `#[ignore]`)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | NOT NEEDED | Compile-time verification covers delegation wiring | N/A | SKIP |
| Interactive tests (expectrl) | REQUIRED | Only test type that validates REPL output seen by user | AC-1 through AC-3 fully uncovered | MUST IMPLEMENT (ignored) |
| Integration tests (batch CLI) | NOT NEEDED | Batch commands tested in Features 4-6; REPL path needs PTY | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirements | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| AC-1 | `/describe <table>` in REPL calls `describe::execute_for_repl()` | sprint-47-planning.md Feature 3 | Interactive (ignored) | TC-047-003 |
| AC-2 | `/list *` in REPL calls `list::execute_for_repl()` | sprint-47-planning.md Feature 3 | Interactive (ignored) | TC-047-003 |
| AC-3 | REPL output identical to previous behavior | sprint-47-planning.md Feature 3 | Interactive (ignored) | TC-047-003 |
| AC-4 | Existing REPL tests pass without modification | sprint-47-planning.md Feature 3 | `cargo test --test interactive_tests -- --ignored` | TC-047-003 |

#### 5. Gap Analysis

**Without DB (blocked scenario)**
- **Reason:** Interactive tests require a real Teradata connection — there is no mock for the REPL session.
- **What won't be validated:** The entire delegation behavior (AC-1 through AC-4).
- **Risk assessment:** HIGH if DB is unavailable — report BLOCKED.
- **Mitigation:** Require `TQ_LOGON` to be set. If not set, mark tests BLOCKED.

#### 6. Test Implementation Plan

**Test Type: Interactive Tests (expectrl, `#[ignore]`)**
- **Location:** `tests/interactive_tests.rs` (extend existing)
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 5 tests
- **Key scenarios:**
  1. Spawn REPL, send `/describe dbc.tables`, wait for prompt; assert output contains "Column" or column name (delegation produced output)
  2. Spawn REPL, send `/list databases`, wait for prompt; assert output contains a database name (DBC is always present)
  3. Spawn REPL, send `/list tables`, wait for prompt; assert output contains at least one table name or "no tables found" (not an error crash)
  4. Spawn REPL, send `/list views`, wait for prompt; assert output contains at least one view name or "no views found" (not an error crash)
  5. Regression: existing `/describe` and `/list` test cases pass (AC-4)

#### 7. Coverage Sufficiency Assessment

- Interactive tests validate: delegation path produces visible output in REPL
- Combined coverage: adequate for the sprint goal (prove delegation wiring works)

**Gaps in combined coverage:**
- Exact output identity comparison (new vs old code path) is not automated. The test validates "produces reasonable output" not "produces byte-identical output." This is acceptable because the batch modules now define the canonical output — the REPL simply delegates to them.

---

### Feature 4: Enrich `tq describe` Output

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-47-planning.md` Feature 4, AC-1 through AC-5
- Secondary: `docs/specifications/cli-interface.md` §describe (lines 1452-1654)
- Secondary: Sprint 46 review recommendations #1 and #5

**Requirements (from acceptance criteria):**
1. AC-1: Output includes object header (database, table name, type) — "Table: DATABASE.tablename" and "Type: Table"
2. AC-2: Columns table includes CommentString column
3. AC-3: Indexes section shows primary and secondary indexes (format: "Primary Index (UPI): col")
4. AC-4: JSON output uses structured `{"object", "columns[]", "indexes[]}` wrapper (not flat)
5. AC-5: Unit tests for describe.rs formatting functions

**Spec excerpt for output format:**
```
Table: PRODUCTION.employees
Type:  Table
Rows (Est.): 42,573

Columns:
┌─...─┬─...─┬──────────┬─────────┬──────────┐
│ Column │ Type │ Nullable │ Default │ Comments │
...
Indexes:
  Primary Index (UPI): employee_id
  Secondary Index (NUSI): department_id
```

**Feature Characteristics:**

**User Interaction Type:** CLI Batch (scripted, non-interactive) + Pure Logic (formatting)

**Observable Behavior:**
- [x] Structured data output (JSON, CSV)
- [x] Visual output in terminal (table with header block, Indexes section)

**External Dependencies:**
- None for formatting logic (writer injection)
- [x] Database connection — for end-to-end AC-1 through AC-4 validation

**Validation Challenges:**
- The output structure changed substantially (new header block, new Comments column, new Indexes section). Unit tests need to verify each new section independently.
- JSON structure change (from flat to structured wrapper) is breaking; assert exact key names.
- "Rows (Est.)" and "Comments" require different DBC queries; unit tests must mock the output data.

**Critical Behaviors to Validate:**
1. Header block: output starts with "Table: " followed by qualified name
2. "Type:" line follows header
3. "Rows (Est.):" line is present for tables, absent for views (REQ-DESCRIBE-008)
4. Columns table has exactly 5 columns: Column, Type, Nullable, Default, Comments (REQ-DESCRIBE-005)
5. CommentString column renders correctly (empty string when no comment, actual text when present)
6. Indexes section header "Indexes:" is present
7. Primary index shown with UPI/NUPI label: "Primary Index (UPI): col1"
8. Secondary index shown with USI/NUSI label: "Secondary Index (NUSI): col2"
9. Indexes section absent for views (REQ-DESCRIBE-007)
10. JSON: `{"object": "...", "type": "...", "estimated_rows": N, "columns": [...], "indexes": [...]}`
11. JSON: `nullable` is boolean (not string), `default` is null (not "-") (REQ-DESCRIBE-009)

#### 2. Test Strategy Derivation

**Decision Tree Results:**
- "CLI Batch" checked → Integration tests REQUIRED (help text, exit codes)
- "Pure Logic" for formatting → Unit tests REQUIRED for AC-5
- "Database connection" checked → `#[ignore]` integration tests RECOMMENDED

**Derived Test Types:**

**Test Type 1: Unit Tests (formatting functions)**
- **Validates:** AC-2, AC-3, AC-4, AC-5 — describe.rs formatting functions produce correct output for all new sections
- **Approach:** Create `DescribeResult` structs with known values; call render functions with a `Vec<u8>` writer; assert output string contains correct sections, column headers, index formatting, and JSON structure.
- **Rationale:** Formatting functions are pure (writer injection); unit tests are complete and fast.
- **Gap if missing:** AC-5 explicitly requires unit tests. Formatting bugs would only surface with a DB.
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (DB, `#[ignore]`)**
- **Validates:** AC-1 through AC-4 — actual `tq describe` output against a real table
- **Approach:** Spawn `tq describe dbc.tables`; assert stdout contains "Table:", "Type:", "Columns:", and "Indexes:" sections. Spawn with `--format json`; parse output as JSON; assert structure.
- **Rationale:** Unit tests validate formatting logic but cannot verify that the new DBC queries (for Comments, Rows, Indexes) return data correctly.
- **Gap if missing:** DBC query bugs for Comments, Rows, Indexes not caught.
- **Necessity:** RECOMMENDED (DB required, `#[ignore]`)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | AC-5 requires them; validates formatting of all new sections | AC-5 uncovered; header/Comments/Indexes/JSON bugs undetected | MUST IMPLEMENT |
| Integration tests (DB) | RECOMMENDED | Validates DBC query results and full rendering | DBC query errors not caught | SHOULD IMPLEMENT (ignored) |
| Interactive tests (PTY) | NOT NEEDED | `tq describe` is batch-only; no REPL interaction | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirements | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| AC-1 | Output includes object header | sprint-47-planning.md Feature 4; cli-interface.md §describe | Unit | TC-047-004 |
| AC-2 | Columns table includes Comments column | sprint-47-planning.md Feature 4; REQ-DESCRIBE-005 | Unit | TC-047-004 |
| AC-3 | Indexes section shows primary and secondary indexes | sprint-47-planning.md Feature 4; cli-interface.md §describe output | Unit | TC-047-004 |
| AC-4 | JSON uses structured wrapper | sprint-47-planning.md Feature 4; REQ-DESCRIBE-009 | Unit + Integration(ignored) | TC-047-004 |
| AC-5 | Unit tests for describe.rs formatting | sprint-47-planning.md Feature 4 | Unit | TC-047-004 |
| REQ-DESCRIBE-007 | Indexes section absent for views | cli-interface.md §describe | Unit | TC-047-004 |
| REQ-DESCRIBE-008 | Rows (Est.) absent for views | cli-interface.md §describe | Unit | TC-047-004 |
| REQ-DESCRIBE-009 | JSON nullable is boolean, default is null | cli-interface.md §describe | Unit | TC-047-004 |

#### 5. Gap Analysis

**Without DB (blocked scenario)**
- Unit tests validate all formatting logic. DB tests validate the DBC query implementation.
- **Risk assessment:** MEDIUM — the DBC join for Comments and the estimated rows query are new SQL; only DB tests verify them.
- **Mitigation:** Unit tests cover the formatting functions; code review confirms DBC queries are correct.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/describe.rs` `#[cfg(test)]` module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 18 tests
- **Key scenarios:**
  1. Object header format: `format_header("PRODUCTION", "employees", "Table", Some(42573))` contains `"Table: PRODUCTION.employees"`
  2. Object header for view: `format_header(...)` omits Rows line for view type
  3. Comments column present: table output contains "Comments" header
  4. Comments empty: column with no comment renders empty string in Comments cell
  5. Comments with text: column with comment renders comment text in Comments cell
  6. Index section header "Indexes:" present in table output
  7. Primary UPI: "Primary Index (UPI): employee_id" in indexes section
  8. Primary NUPI: "Primary Index (NUPI): config_key" in indexes section
  9. Secondary NUSI: "Secondary Index (NUSI): department_id" in indexes section
  10. Secondary USI: "Secondary Index (USI): email" in indexes section
  11. View: Indexes section absent from output
  12. View: Rows (Est.) absent from output
  13. JSON object key present: output starts with `{"object":`
  14. JSON columns array: `"columns":[` present with entries
  15. JSON indexes array: `"indexes":[` present with entries
  16. JSON nullable is boolean true: `"nullable":true` (not `"nullable":"YES"`)
  17. JSON default is null: `"default":null` when no default set (not `"default":"-"`)
  18. No secondary indexes: Indexes section shows "No indexes defined" for table with only PI

**Test Type: Integration Tests (DB, `#[ignore]`)**
- **Location:** `tests/integration_tests.rs` or `tests/describe_47.rs`
- **Test count estimate:** 5 tests
- **Key scenarios:**
  1. `tq describe dbc.tables` exits 0, stdout contains "Table:", "Columns:", "Indexes:"
  2. `tq describe dbc.tables --format json` exits 0, output is valid JSON with "object", "columns", "indexes" keys
  3. `tq describe dbc.tables --format json` columns have `"nullable":true` or `"nullable":false` (boolean, not string)
  4. `tq describe dbc.columns` (a view) exits 0, stdout does not contain "Indexes:" or "Rows (Est.)"
  5. `tq describe nonexistent_table_xyz` exits 1, stderr or stdout contains "Error:"

#### 7. Coverage Sufficiency Assessment

- Unit tests validate: all 5 ACs and all spec requirements through formatting function testing
- Integration tests validate: DBC queries and full rendering with real data
- Combined coverage: comprehensive

---

### Feature 5: Enrich `tq list` Output

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-47-planning.md` Feature 5, AC-1 through AC-4
- Secondary: `docs/specifications/cli-interface.md` §list (lines 1657-2016)
- Secondary: Sprint 46 review recommendations #2, #3, #5

**Requirements (from acceptance criteria):**
1. AC-1: `tq list databases` shows Owner, Type columns (REQ-LIST-002, REQ-LIST-003)
2. AC-2: `tq list tables` shows Rows (Est.), Size columns (REQ-LIST-005, REQ-LIST-009)
3. AC-3: JSON output uses structured objects (not flat string arrays) (REQ-LIST-016)
4. AC-4: Unit tests for list.rs formatting functions

**Key spec requirements:**
- REQ-LIST-002: `list databases` columns: Database, Owner, Type
- REQ-LIST-003: Type is "System" for DBC-owned, "User" for others
- REQ-LIST-004: System databases first, then User (alphabetical within each group)
- REQ-LIST-005: `list tables` columns: Table, Type, Rows (Est.), Size
- REQ-LIST-009: Size in human-readable format (table), raw bytes (CSV/JSON)
- REQ-LIST-016: JSON returns array `[]`; empty result is `[]` not null

**Feature Characteristics:**

**User Interaction Type:** CLI Batch + Pure Logic (formatting)

**Observable Behavior:**
- [x] Structured data output (JSON, CSV, table)
- [x] Visual output in terminal (new columns)

**External Dependencies:**
- None for formatting logic
- [x] Database connection — for end-to-end validation

**Validation Challenges:**
- REQ-LIST-003 (System vs User type classification) requires knowing the owner — testable with mock data.
- REQ-LIST-009 (size in human-readable vs raw bytes per format) requires testing both the table format renderer and the CSV/JSON renderer with the same data.
- REQ-LIST-016 (empty result is `[]`) is a structural JSON test — important edge case.

**Critical Behaviors to Validate:**
1. `list databases` table output has "Owner" and "Type" columns
2. DBC-owned database shows Type = "System"
3. Non-DBC-owned database shows Type = "User"
4. `list tables` table output has "Rows (Est.)" and "Size" columns
5. Size in table format: human-readable ("2.1 MB", "890 KB")
6. Size in JSON format: raw bytes integer (`"size_bytes": 2201783`)
7. JSON `list databases` returns `[{"database": ..., "owner": ..., "type": ...}]` array
8. JSON `list tables` returns `[{"table": ..., "type": ..., "estimated_rows": ..., "size_bytes": ...}]` array
9. Empty result in JSON returns `[]` (not null, not omitted)

#### 2. Test Strategy Derivation

**Decision Tree Results:**
- "CLI Batch" checked → Integration tests REQUIRED
- "Pure Logic" for formatting → Unit tests REQUIRED for AC-4
- "Database connection" checked → `#[ignore]` integration tests RECOMMENDED

**Derived Test Types:**

**Test Type 1: Unit Tests (formatting functions)**
- **Validates:** AC-1, AC-2, AC-3, AC-4 — list.rs formatting functions for all new columns and JSON structure
- **Approach:** Create mock `DatabaseRow` / `TableRow` / `ViewRow` structs; call render functions with `Vec<u8>` writer; assert output structure.
- **Rationale:** Formatting is pure; unit tests are complete and fast.
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (DB, `#[ignore]`)**
- **Validates:** AC-1 through AC-4 end-to-end with real data
- **Approach:** Spawn `tq list databases`, `tq list tables`, `tq list views`; assert output columns and JSON structure.
- **Necessity:** RECOMMENDED (DB required, `#[ignore]`)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | AC-4 requires them; validates all new column logic | AC-4 uncovered; formatting bugs undetected | MUST IMPLEMENT |
| Integration tests (DB) | RECOMMENDED | Validates DBC query results and full rendering | DBC query errors not caught | SHOULD IMPLEMENT (ignored) |
| Interactive tests (PTY) | NOT NEEDED | `tq list` is batch-only | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirements | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| AC-1 | `list databases` shows Owner, Type | sprint-47-planning.md Feature 5; REQ-LIST-002 | Unit | TC-047-005 |
| AC-2 | `list tables` shows Rows (Est.), Size | sprint-47-planning.md Feature 5; REQ-LIST-005 | Unit | TC-047-005 |
| AC-3 | JSON uses structured objects | sprint-47-planning.md Feature 5; REQ-LIST-016 | Unit | TC-047-005 |
| AC-4 | Unit tests for list.rs formatting | sprint-47-planning.md Feature 5 | Unit | TC-047-005 |
| REQ-LIST-003 | Type classification: System/User | cli-interface.md §list | Unit | TC-047-005 |
| REQ-LIST-009 | Size: human-readable in table, raw bytes in JSON/CSV | cli-interface.md §list | Unit | TC-047-005 |
| REQ-LIST-016 | Empty result is `[]` not null | cli-interface.md §list | Unit | TC-047-005 |

#### 5. Gap Analysis

**Database-connected validation**
- New DBC columns (Owner in DatabasesV, Rows/Size in TableSizeV join) require real queries. Unit tests verify formatting given mock data.
- **Risk assessment:** MEDIUM — Owner and Size data may require different joins than previously implemented.
- **Mitigation:** Unit tests validate formatting; DB tests validate data retrieval.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/list.rs` `#[cfg(test)]` module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 16 tests
- **Key scenarios:**
  1. `list databases` table output: column headers include "Database", "Owner", "Type"
  2. DBC-owned database: Type column shows "System"
  3. Non-DBC-owned database: Type column shows "User"
  4. System before User ordering: System databases appear before User databases in output
  5. `list tables` table output: column headers include "Table", "Type", "Rows (Est.)", "Size"
  6. Size in table format: 890 KB for 911974 bytes ("890 KB")
  7. Size in table format: 2.1 MB for 2201783 bytes ("2.1 MB")
  8. Size in table format: 45.2 MB for 47395274 bytes ("45.2 MB")
  9. JSON `list databases`: output is array `[{...}]` with "database", "owner", "type" keys
  10. JSON `list tables`: output is array `[{...}]` with "table", "type", "estimated_rows", "size_bytes" keys
  11. JSON `size_bytes` is integer: raw bytes value in JSON (not human-readable string)
  12. JSON empty result: `[]` returned (not null, not `[null]`)
  13. CSV `list databases`: header row is `database,owner,type`
  14. CSV `list tables`: header row is `table,type,estimated_rows,size_bytes`
  15. CSV `size_bytes` is raw integer (not formatted string)
  16. Pattern filter: tables are filtered by LIKE pattern (test with mock filtered result set)

**Test Type: Integration Tests (DB, `#[ignore]`)**
- **Location:** `tests/integration_tests.rs` or `tests/list_47.rs`
- **Test count estimate:** 6 tests
- **Key scenarios:**
  1. `tq list databases` exits 0, stdout contains "DBC" in database column
  2. `tq list databases` stdout contains "Owner" and "Type" column headers
  3. `tq list databases --format json` is valid JSON array with "database", "owner", "type" keys
  4. `tq list tables` exits 0, stdout contains "Rows (Est.)" and "Size" column headers
  5. `tq list tables --format json` is valid JSON array with "estimated_rows" and "size_bytes" keys
  6. `tq list databases --format json | jq 'length'` outputs a number > 0

#### 7. Coverage Sufficiency Assessment

- Unit tests validate: all new column logic, type classification, size formatting, JSON structure
- Integration tests validate: DBC queries and full rendering
- Combined coverage: comprehensive

---

### Feature 6: Enrich `tq show-indexes` Output

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-47-planning.md` Feature 6, AC-1 through AC-4
- Secondary: `docs/specifications/cli-interface.md` §show-indexes (lines 2019-2150)
- Secondary: Sprint 46 review recommendations #4, #5

**Requirements (from acceptance criteria):**
1. AC-1: Output has separate Primary Index and Secondary Indexes sections
2. AC-2: Index types labeled as UPI/NUPI/USI/NUSI (not raw Teradata type codes)
3. AC-3: JSON output uses structured `{"primary_index", "secondary_indexes[]}` wrapper
4. AC-4: Unit tests for show_indexes.rs formatting functions

**Key spec output format:**
```
Index structure for PRODUCTION.employees:

  Primary Index
    Type:     Unique Primary Index (UPI)
    Columns:  employee_id

  Secondary Indexes
    #1  Non-Unique Secondary Index (NUSI)  (department_id)
    #2  Unique Secondary Index (USI)       (email)
```

**JSON format:**
```json
{
  "object": "PRODUCTION.employees",
  "primary_index": {"type": "UPI", "type_label": "Unique Primary Index (UPI)", "columns": ["employee_id"], "no_pi": false},
  "secondary_indexes": [
    {"index_no": 1, "type": "NUSI", "type_label": "Non-Unique Secondary Index (NUSI)", "columns": ["department_id"]}
  ]
}
```

**Feature Characteristics:**

**User Interaction Type:** CLI Batch + Pure Logic (formatting)

**Observable Behavior:**
- [x] Structured data output (JSON, CSV)
- [x] Visual output in terminal (two-section Primary/Secondary layout)

**External Dependencies:**
- None for formatting logic
- [x] Database connection — for end-to-end validation

**Validation Challenges:**
- Primary vs Secondary split requires understanding Teradata's `IndexType` values. The mapping (P=Primary, S=Secondary) must be correctly implemented.
- UPI/NUPI/USI/NUSI labeling requires combining `IndexType` (P/S) and `UniqueFlag` (Y/N). Test all 4 combinations.
- NoPI table is a special case: no primary index rows in DBC.IndicesV; output must show "No Primary Index (NoPI)".
- JSON `no_pi: false` vs `no_pi: true` is a critical structural distinction.

**Critical Behaviors to Validate:**
1. Output starts with "Index structure for DATABASE.tablename:"
2. "Primary Index" section header is present
3. Primary index type: UPI when IndexType="P" and UniqueFlag="Y"
4. Primary index type: NUPI when IndexType="P" and UniqueFlag="N"
5. Secondary index type: USI when IndexType="S" and UniqueFlag="Y"
6. Secondary index type: NUSI when IndexType="S" and UniqueFlag="N"
7. NoPI table: "Primary Index" section shows "No Primary Index (NoPI)"
8. "Secondary Indexes" section shows numbered list (#1, #2, ...)
9. No secondary indexes: "No secondary indexes defined." message shown
10. JSON: `primary_index` key with `type`, `type_label`, `columns`, `no_pi` subkeys
11. JSON: `secondary_indexes` key is array
12. JSON: NoPI table has `"no_pi": true, "type": null, "columns": []`
13. CSV: header row is `kind,index_no,type,columns`

#### 2. Test Strategy Derivation

**Decision Tree Results:**
- "CLI Batch" checked → Integration tests REQUIRED
- "Pure Logic" for formatting → Unit tests REQUIRED for AC-4
- "Database connection" checked → `#[ignore]` integration tests RECOMMENDED

**Derived Test Types:**

**Test Type 1: Unit Tests (formatting functions)**
- **Validates:** AC-1, AC-2, AC-3, AC-4 — show_indexes.rs formatting for all label types, both sections, JSON structure
- **Approach:** Create mock index data; call render functions with `Vec<u8>` writer; assert all label combinations and JSON keys.
- **Rationale:** Formatting is pure; unit tests are complete and fast.
- **Necessity:** REQUIRED

**Test Type 2: Integration Tests (DB, `#[ignore]`)**
- **Validates:** AC-1 through AC-4 end-to-end with real DBC.IndicesV data
- **Necessity:** RECOMMENDED (DB required, `#[ignore]`)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | REQUIRED | AC-4 requires them; validates all 4 label types and JSON structure | AC-4 uncovered; label/JSON bugs undetected | MUST IMPLEMENT |
| Integration tests (DB) | RECOMMENDED | Validates DBC query results and full rendering | DBC query errors not caught | SHOULD IMPLEMENT (ignored) |
| Interactive tests (PTY) | NOT NEEDED | `tq show-indexes` is batch-only | N/A | SKIP |
| Benchmark tests | NOT NEEDED | No performance requirements | N/A | SKIP |

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Test Cases |
|----------------|-----------------|----------------|--------------|------------|
| AC-1 | Separate Primary/Secondary sections | sprint-47-planning.md Feature 6; cli-interface.md §show-indexes | Unit | TC-047-006 |
| AC-2 | Labels: UPI/NUPI/USI/NUSI | sprint-47-planning.md Feature 6; cli-interface.md §show-indexes | Unit | TC-047-006 |
| AC-3 | JSON: `{primary_index, secondary_indexes[]}` | sprint-47-planning.md Feature 6; cli-interface.md §show-indexes | Unit | TC-047-006 |
| AC-4 | Unit tests for show_indexes.rs | sprint-47-planning.md Feature 6 | Unit | TC-047-006 |
| NoPI format | "No Primary Index (NoPI)" for NoPI tables | cli-interface.md §show-indexes | Unit | TC-047-006 |
| No secondary | "No secondary indexes defined." message | cli-interface.md §show-indexes | Unit | TC-047-006 |

#### 5. Gap Analysis

**NoPI table detection**
- NoPI tables have `TableKind = 'O'`. The show-indexes command may need to query `DBC.TablesV` to detect NoPI in addition to querying `DBC.IndicesV`. Unit tests can test the NoPI display rendering but the detection logic requires a real NoPI table or the `TableKind` being passed from the parent query.
- **Risk assessment:** MEDIUM — if NoPI detection is implemented purely by absence of primary index rows in IndicesV (not by TableKind), this is testable with an empty primary row set.
- **Mitigation:** Unit test uses empty primary index set to verify NoPI rendering.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/commands/show_indexes.rs` `#[cfg(test)]` module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 15 tests
- **Key scenarios:**
  1. Output header: "Index structure for DATABASE.employees:" present
  2. Primary section header: "Primary Index" present
  3. UPI label: IndexType="P", UniqueFlag="Y" → "Unique Primary Index (UPI)"
  4. NUPI label: IndexType="P", UniqueFlag="N" → "Non-Unique Primary Index (NUPI)"
  5. USI label: IndexType="S", UniqueFlag="Y" → "Unique Secondary Index (USI)"
  6. NUSI label: IndexType="S", UniqueFlag="N" → "Non-Unique Secondary Index (NUSI)"
  7. NoPI: no primary index rows → "No Primary Index (NoPI)" rendered
  8. Secondary section header: "Secondary Indexes" present
  9. Secondary index numbering: "#1", "#2", "#3" in output
  10. No secondary indexes: "No secondary indexes defined." rendered
  11. JSON: output starts with `{"object":`
  12. JSON: `"primary_index":` key present with `"type"`, `"type_label"`, `"columns"`, `"no_pi"`
  13. JSON: `"secondary_indexes":` key is an array
  14. JSON NoPI: `"no_pi":true,"type":null,"columns":[]`
  15. CSV: header row is `kind,index_no,type,columns`

**Test Type: Integration Tests (DB, `#[ignore]`)**
- **Location:** `tests/integration_tests.rs` or `tests/show_indexes_47.rs`
- **Test count estimate:** 4 tests
- **Key scenarios:**
  1. `tq show-indexes dbc.tables` exits 0, stdout contains "Primary Index" and "Secondary Indexes"
  2. `tq show-indexes dbc.tables` stdout contains one of UPI/NUPI/USI/NUSI labels
  3. `tq show-indexes dbc.tables --format json` is valid JSON with "primary_index" and "secondary_indexes" keys
  4. `tq show-indexes nonexistent_table_xyz` exits 1, stdout contains "Error:"

#### 7. Coverage Sufficiency Assessment

- Unit tests validate: all 4 label types, section layout, NoPI handling, JSON structure, CSV headers
- Integration tests validate: DBC query results and full rendering
- Combined coverage: comprehensive

---

## Strategy Summary

**Total Features Analyzed:** 6

**Test Types Required:**
- Unit tests: REQUIRED for Features 1, 2, 4, 5, 6
- Interactive tests (expectrl): REQUIRED for Feature 3 (REPL delegation)
- Integration tests (DB, `#[ignore]`): RECOMMENDED for Features 1, 3, 4, 5, 6
- Structural grep check: REQUIRED for Feature 2 (zero duplication AC-5)
- Benchmark tests: NOT NEEDED for any feature

**Estimated Test Count:**
- Unit: ~75 tests (6 F1 + 20 F2 + 18 F4 + 16 F5 + 15 F6)
- Interactive (DB, `#[ignore]`): 5 tests (Feature 3)
- Integration (DB, `#[ignore]`): ~23 tests (4 F1 + 5 F4 + 6 F5 + 4 F6 + 4 F3-REPL-batch-parity)
- Total: ~103 tests

**Risk Assessment:**
- HIGH risk gaps: Feature 3 (REPL delegation) — BLOCKED if no database available; no unit-testable fallback
- MEDIUM risk gaps: Features 1, 4, 5, 6 — DB-dependent queries (new DBC joins) not covered without DB
- LOW risk gaps: Feature 2 (shared helpers) — pure logic, fully unit-testable

**Dependencies Required:**
- Live database: Yes — required for `#[ignore]` tests in Features 1, 3, 4, 5, 6. Unit tests for Features 2, 4, 5, 6 (formatting logic) have no DB dependency.
- Network access: No
- Specific OS: No (Darwin for development, Linux for CI)
- Environment variable: `TQ_LOGON` must be set for all `#[ignore]` tests
- Crate: `expectrl` for interactive tests (Feature 3)

---

## Test Case Files

| Test Case File | Feature | DB Required | Test Count |
|---------------|---------|-------------|------------|
| `tests/cases/TC-047-001.md` | Bug #36 — /inspect DDL & column type fix | No (unit) / Yes (integration, ignored) | 6 unit + 4 integration |
| `tests/cases/TC-047-002.md` | Shared helpers extraction (format_helpers.rs) | No | 20 unit + 1 structural grep |
| `tests/cases/TC-047-003.md` | REPL delegation (/describe and /list) | Yes (`#[ignore]`) | 5 interactive |
| `tests/cases/TC-047-004.md` | Enrich `tq describe` output | No (unit) / Yes (integration, ignored) | 18 unit + 5 integration |
| `tests/cases/TC-047-005.md` | Enrich `tq list` output | No (unit) / Yes (integration, ignored) | 16 unit + 6 integration |
| `tests/cases/TC-047-006.md` | Enrich `tq show-indexes` output | No (unit) / Yes (integration, ignored) | 15 unit + 4 integration |

---

## Infrastructure Note

No new testing tools are required for this sprint. All test types are covered by existing infrastructure:
- Rust built-in `#[test]` for unit tests
- `std::process::Command` for CLI integration tests
- `expectrl` crate (already in project) for interactive REPL tests
- Shell `grep -c` for structural duplication check (Feature 2, AC-5)

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
