# Sprint 34 Test Strategy

**Created:** 2026-02-03
**Author:** quality-validator
**Sprint:** Sprint 34
**Features:** Code Quality Cleanup, Security Hardening, Documentation Synchronization

---

## Instructions for quality-validator

This strategy defines validation approach for Sprint 34, a maintenance sprint addressing technical debt from Sprint 33. Unlike feature sprints, this sprint requires verifying that cleanup activities do not introduce regressions while improving code quality and security.

**Key Challenge:** Validate improvements without breaking existing functionality (471 tests must continue to pass).

---

## Sprint 34 Objectives Summary

This maintenance sprint has three tracks:

1. **Track 1: Code Quality** - Extract duplicate `format_column_type()` function to shared module
2. **Track 2: Security** - Add SQL identifier quoting to prevent injection
3. **Track 3: Documentation** - Synchronize specifications with implementation (no code changes)

**Sprint Type:** MAINTENANCE (cleanup sprint, no new features)

---

## Feature-by-Feature Test Strategy

### Track 1: Code Quality - Extract `format_column_type()` to Shared Module

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-34-planning.md` - Acceptance Criteria AC-1 through AC-5
- Secondary: `docs/sprints/sprint-33-review.md` - Technical Review identifying duplication
- Requirements:
  1. "AC-1: `format_column_type()` extracted to shared module (`src/utils/teradata_types.rs`)"
  2. "AC-2: Both `sample.rs` and `metacommands.rs` use shared implementation"
  3. "AC-3: Unit tests pass for shared utility module"
  4. "AC-4: No code duplication detected in technical review"
  5. "AC-5: Zero regressions (all 471 tests continue to pass)"

**Current State:**
- `format_column_type()` exists in `src/commands/sample.rs` (lines 189-221)
- Same logic needs to be in `src/commands/repl/metacommands.rs` (not yet duplicated, per grep results)
- Sprint 33 review identified this as technical debt to address

**Feature Characteristics:**

**User Interaction Type:** Pure Logic
- This is internal refactoring with no user-visible changes
- Users should see identical behavior before and after refactoring
- Type formatting output must remain unchanged

**Observable Behavior:**
- No visual output changes
- No CLI behavior changes
- No REPL behavior changes
- Internal module structure changes only

**External Dependencies:**
- None - Pure logic, no I/O or database dependencies

**Validation Challenges:**
- Challenge 1: Must verify behavior is identical before and after extraction
- Challenge 2: Must ensure both consumers use shared implementation (no lingering duplicates)
- Challenge 3: Must validate all Teradata type codes still format correctly (VARCHAR, DECIMAL, etc.)

**Critical Behaviors to Validate:**
1. Behavior 1: Type formatting produces identical output after extraction (Sprint 34 AC-3)
2. Behavior 2: Both `sample.rs` and `metacommands.rs` import and use shared module (Sprint 34 AC-2)
3. Behavior 3: All 471 existing tests continue to pass (Sprint 34 AC-5 - regression prevention)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Pure Logic" checked:
  → Unit tests REQUIRED
  Reason: Pure functions with no I/O are perfectly suited for unit testing

IF "Zero regressions" requirement:
  → Regression tests REQUIRED (run full test suite)
  Reason: Must prove no existing functionality broken

IF "Code duplication" removal:
  → Code review verification REQUIRED
  Reason: Must confirm no duplicate implementations remain
```

**Derived Test Types:**

**Test Type 1: Unit Tests for Shared Module**
- **Validates:** Type formatting logic for all Teradata type codes (AC-3)
- **Approach:** Test each type code mapping (CV→VARCHAR, I→INTEGER, D→DECIMAL, etc.)
- **Rationale:** Pure function validation - input type code → output formatted string
- **Gap if missing:** Cannot verify type formatting correctness after extraction
- **Necessity:** ✅ REQUIRED

**Test Type 2: Regression Tests (Full Suite)**
- **Validates:** No existing functionality broken by refactoring (AC-5)
- **Approach:** Run entire test suite (471 tests) and verify 100% pass rate
- **Rationale:** Refactoring should be transparent - all tests should still pass
- **Gap if missing:** Cannot prove refactoring didn't introduce subtle breakage
- **Necessity:** ✅ REQUIRED

**Test Type 3: Code Review Verification**
- **Validates:** No duplicate implementations remain (AC-4), both modules use shared code (AC-2)
- **Approach:** Grep for `format_column_type` definitions, verify only one implementation exists
- **Rationale:** Must confirm cleanup objective achieved
- **Gap if missing:** Cannot verify technical debt actually eliminated
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (shared module) | ✅ REQUIRED | Validates type formatting logic correctness | Logic bugs, type mapping errors | MUST IMPLEMENT |
| Regression tests (full suite) | ✅ REQUIRED | Proves no functionality broken | Silent regressions, behavior changes | MUST RUN |
| Code review verification | ✅ REQUIRED | Confirms duplication eliminated | Cleanup objective not validated | MUST PERFORM |
| Integration tests (new) | ❌ NOT NEEDED | No new integration points, refactoring only | N/A | SKIP |
| Interactive tests (new) | ❌ NOT NEEDED | No user-facing changes, internal only | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 3 - Unit tests, regression suite, code review
- ⚠️ RECOMMENDED test types: 0
- ❌ NOT NEEDED test types: 2 - No new integration/interactive tests needed

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| AC-1 | "format_column_type() extracted to shared module (src/utils/teradata_types.rs)" | sprint-34-planning.md §AC-1 | Code review | Must verify module created and populated | Code inspection |
| AC-2 | "Both sample.rs and metacommands.rs use shared implementation" | sprint-34-planning.md §AC-2 | Code review | Must verify imports and usage | Grep verification |
| AC-3 | "Unit tests pass for shared utility module" | sprint-34-planning.md §AC-3 | Unit tests | Type formatting correctness | 8-12 tests (one per type code) |
| AC-4 | "No code duplication detected in technical review" | sprint-34-planning.md §AC-4 | Code review | Must verify no duplicate definitions remain | Grep for duplicates |
| AC-5 | "Zero regressions (all 471 tests continue to pass)" | sprint-34-planning.md §AC-5 | Regression tests | Full suite must pass at 100% | Run `cargo test` |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements (missing test coverage)
- [x] No unjustified test types (test types without requirement rationale)

**Coverage Gaps:**
- None identified for Track 1

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Integration Tests (new)**
- **Reason for omission:** Pure refactoring with no new integration points
- **What won't be validated:** Integration behavior (already covered by existing tests)
- **Risk assessment:** LOW - Existing integration tests provide coverage
- **Mitigation:** Run existing integration tests as part of regression suite
- **Revisit criteria:** N/A - not needed for refactoring sprints

**Interactive Tests (new)**
- **Reason for omission:** No user-facing changes, internal module extraction only
- **What won't be validated:** REPL behavior (unchanged)
- **Risk assessment:** LOW - Existing interactive tests cover REPL
- **Mitigation:** Run existing interactive tests as part of regression suite
- **Revisit criteria:** N/A - not needed for internal refactoring

#### 6. Test Implementation Plan

**Test Type: Unit Tests (Shared Module)**
- **Location:** `src/utils/teradata_types.rs` - inline `#[cfg(test)] mod tests` section
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 8-12 tests
- **Key scenarios to cover:**
  1. VARCHAR type (CV with length)
  2. CHAR type (CF with length)
  3. INTEGER types (I, I1, I2, I8)
  4. DECIMAL type (D with precision and scale)
  5. DATE/TIME types (DA, TS, TZ, AT)
  6. Binary types (BV, BF)
  7. LOB types (CO, BO)
  8. JSON type (JN)
  9. Unknown type code fallback
- **Mocking strategy:** No mocking needed - pure function testing

**Test Type: Regression Tests (Full Suite)**
- **Location:** All existing test locations (`cargo test`)
- **Framework:** Existing test infrastructure
- **Test count estimate:** 471 existing tests (Sprint 33 baseline: 384 lib tests + 87 integration/interactive tests)
- **Key scenarios to cover:**
  1. All unit tests (`cargo test --lib`) - 384 tests
  2. All integration tests - subset of remaining
  3. All interactive tests (requires database) - 48 ignored tests
- **Setup requirements:** Database connection for interactive tests (use `--ignored` flag)

**Test Type: Code Review Verification**
- **Location:** Manual verification via grep/code inspection
- **Framework:** Shell commands and code inspection
- **Test count estimate:** 3 verification checks
- **Key scenarios to cover:**
  1. Verify `src/utils/teradata_types.rs` exists and contains `format_column_type()`
  2. Verify `src/commands/sample.rs` imports and uses shared function
  3. Verify no duplicate definitions exist (grep for `fn format_column_type`)
- **Implementation notes:** Automate with bash commands in test report

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: Type formatting logic correctness for all Teradata types
- Regression tests validate: No existing functionality broken by refactoring
- Code review validates: Duplication eliminated, shared module used

**Combined coverage:** COMPREHENSIVE

**Gaps in combined coverage:**
- No gaps identified - refactoring validation complete

**Acceptance criteria:**
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified"
- [x] Known gaps are documented and accepted

---

### Track 2: Security Hardening - SQL Identifier Quoting

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-34-planning.md` - Acceptance Criteria AC-6 through AC-10
- Secondary: `docs/sprints/sprint-33-review.md` - Technical Review identifying SQL injection risk
- Requirements:
  1. "AC-6: SQL identifiers quoted in `/sample` command (`\"database\".\"table\"`)"
  2. "AC-7: SQL identifiers quoted in `/peek` command"
  3. "AC-8: SQL identifiers quoted in batch mode (`tq sample`, `tq peek`)"
  4. "AC-9: Unit tests validate quoted identifier generation"
  5. "AC-10: Regression tests verify functionality with special characters in table names"

**Current State:**
- Current SQL generation: `SELECT * FROM database.table SAMPLE n` (unquoted)
- Risk: Table names with spaces, quotes, or special characters could cause SQL errors
- Example problematic names: `My Table`, `Table-2024`, `customer"data`

**Feature Characteristics:**

**User Interaction Type:** Pure Logic + Database Integration
- SQL generation is pure logic (string formatting)
- Actual execution requires database integration testing
- Users should see identical behavior unless using special characters

**Observable Behavior:**
- Structured data output (query results) should be unchanged
- SQL error handling for special character table names will improve

**External Dependencies:**
- Database connection required for integration testing
- Test database with specially-named tables for edge case validation

**Validation Challenges:**
- Challenge 1: Must test with actual special characters in table names (requires database setup)
- Challenge 2: Must verify quoting doesn't break normal table names
- Challenge 3: Must test both REPL and batch modes (different code paths)

**Critical Behaviors to Validate:**
1. Behavior 1: Normal table names continue to work (`customers` → `"customers"`) (AC-6, AC-7, AC-8)
2. Behavior 2: Table names with spaces work (`My Table` → `"My Table"`) (AC-10)
3. Behavior 3: Table names with quotes work (`customer"data` → `"customer""data"`) (AC-10)
4. Behavior 4: Qualified names quoted separately (`db.table` → `"db"."table"`) (AC-6, AC-7)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Pure logic" + "Database integration":
  → Unit tests REQUIRED for quote_identifier() logic
  → Integration tests REQUIRED for end-to-end validation

IF "Special character edge cases":
  → Edge case tests REQUIRED
  Reason: Must prove special characters handled correctly

IF "Regression prevention":
  → Regression tests REQUIRED
  Reason: Must prove normal table names still work
```

**Derived Test Types:**

**Test Type 1: Unit Tests for `quote_identifier()` Function**
- **Validates:** Identifier quoting logic (AC-9)
- **Approach:** Test quoting algorithm with various inputs (normal, spaces, quotes, special chars)
- **Rationale:** Pure function - input identifier → output quoted identifier
- **Gap if missing:** Cannot verify quoting algorithm correctness
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests with Database**
- **Validates:** End-to-end query execution with quoted identifiers (AC-10)
- **Approach:** Execute sample/peek commands against test tables with special character names
- **Rationale:** Only way to prove SQL actually works with Teradata
- **Gap if missing:** Cannot prove quoted SQL is accepted by database
- **Necessity:** ✅ REQUIRED

**Test Type 3: Regression Tests (Full Suite)**
- **Validates:** Normal table names still work (AC-6, AC-7, AC-8)
- **Approach:** Run full test suite, verify no breakage
- **Rationale:** Quoting should not break existing functionality
- **Gap if missing:** Cannot prove change didn't break normal table access
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (quote function) | ✅ REQUIRED | Validates quoting algorithm logic | Algorithm bugs, edge cases missed | MUST IMPLEMENT |
| Integration tests (database) | ✅ REQUIRED | Proves SQL works with Teradata | Quoting syntax errors, DB rejection | MUST IMPLEMENT |
| Regression tests (full suite) | ✅ REQUIRED | Proves normal names still work | Silent breakage of existing features | MUST RUN |
| Interactive tests (new) | ⚠️ RECOMMENDED | Validates REPL `/sample`, `/peek` with special chars | REPL-specific issues | SHOULD IMPLEMENT |

**Summary:**
- ✅ REQUIRED test types: 3 - Unit tests, integration tests, regression tests
- ⚠️ RECOMMENDED test types: 1 - Interactive tests for REPL validation
- ❌ NOT NEEDED test types: 0

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| AC-6 | "SQL identifiers quoted in /sample command" | sprint-34-planning.md §AC-6 | Unit + Integration | SQL generation correctness | TC-SQL-01 to TC-SQL-03 |
| AC-7 | "SQL identifiers quoted in /peek command" | sprint-34-planning.md §AC-7 | Unit + Integration | SQL generation correctness | TC-SQL-04 to TC-SQL-06 |
| AC-8 | "SQL identifiers quoted in batch mode" | sprint-34-planning.md §AC-8 | Integration | CLI invocation | TC-SQL-07, TC-SQL-08 |
| AC-9 | "Unit tests validate quoted identifier generation" | sprint-34-planning.md §AC-9 | Unit | Algorithm validation | TC-SQL-09 to TC-SQL-14 |
| AC-10 | "Regression tests verify special characters work" | sprint-34-planning.md §AC-10 | Integration (database) | Edge case validation | TC-SQL-15 to TC-SQL-18 |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements (missing test coverage)
- [x] No unjustified test types (test types without requirement rationale)

**Coverage Gaps:**
- Interactive REPL tests are RECOMMENDED but not REQUIRED (existing interactive tests may provide some coverage)

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

None - All necessary test types included.

**Test Types at RECOMMENDED (not REQUIRED) level:**

**Interactive Tests (REPL mode)**
- **Reason for deferral:** Existing interactive tests may already cover `/sample` and `/peek`
- **What won't be validated:** REPL-specific edge cases with special character table names
- **Risk assessment:** MEDIUM - REPL and batch modes have different code paths
- **Mitigation:** Run existing interactive tests; add specific test if gaps found
- **Revisit criteria:** If integration tests reveal REPL-specific issues

#### 6. Test Implementation Plan

**Test Type: Unit Tests (quote_identifier function)**
- **Location:** `src/utils/` (wherever quote_identifier is implemented) - inline tests
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 6-8 tests
- **Key scenarios to cover:**
  1. Normal identifier (no special chars): `customers` → `"customers"`
  2. Identifier with spaces: `My Table` → `"My Table"`
  3. Identifier with quotes: `customer"data` → `"customer""data"` (double-quote escaping)
  4. Identifier with hyphens: `table-2024` → `"table-2024"`
  5. Already quoted identifier (idempotence): `"customers"` → `"customers"` OR error
  6. Empty identifier (edge case): `` → `""` OR error
  7. Qualified name handling: ensure function handles single identifiers only
- **Mocking strategy:** No mocking - pure string function

**Test Type: Integration Tests (Database)**
- **Location:** `tests/integration_tests.rs` with `#[ignore]` attribute
- **Framework:** Built-in Rust integration test support
- **Test count estimate:** 4-6 tests
- **Key scenarios to cover:**
  1. Sample command with normal table name (regression check)
  2. Sample command with space in table name (edge case)
  3. Peek command with normal table name (regression check)
  4. Peek command with special characters in table name (edge case)
  5. Batch mode `tq sample` with normal table
  6. Batch mode `tq peek` with normal table
- **Setup requirements:**
  - Test database connection (use TQ_LOGON from .env)
  - Create test tables with special character names (setup script needed)
  - Example: `CREATE TABLE "My Test Table" (id INTEGER, name VARCHAR(100))`

**Test Type: Regression Tests (Full Suite)**
- **Location:** All existing test locations
- **Framework:** Existing test infrastructure
- **Test count estimate:** 471 existing tests (same as Track 1)
- **Key scenarios to cover:**
  1. All existing unit tests
  2. All existing integration tests
  3. All existing interactive tests (with database)
- **Setup requirements:** Database connection for ignored tests

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: Quoting algorithm correctness for all edge cases
- Integration tests validate: SQL execution works with Teradata database
- Regression tests validate: No breakage of existing functionality

**Combined coverage:** ADEQUATE (with gap noted below)

**Gaps in combined coverage:**
- Gap 1: Interactive REPL tests with special characters not explicitly included (RECOMMENDED level)
- Gap 2: Cross-database compatibility not tested (Teradata-only testing)

**Acceptance criteria:**
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified"
- [x] Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Gap 1 is acceptable because: Integration tests cover batch mode, and REPL uses same underlying logic. If issues arise, can add interactive tests later.
- Gap 2 is acceptable because: tq is Teradata-only tool; no other databases in scope.

---

### Track 3: Documentation Synchronization

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-34-planning.md` - Acceptance Criteria AC-11 through AC-15
- Secondary: `docs/sprints/sprint-33-review.md` - UX Review identifying spec/impl discrepancies
- Requirements:
  1. "AC-11: `/peek` specification updated to allow `[N]` parameter (REQ-SAMPLE-004.1)"
  2. "AC-12: Pager status badges added to `docs/specifications/repl.md` section headers"
  3. "AC-13: Specification matches implementation behavior"
  4. "AC-14: User documentation reflects accurate `/peek` syntax"
  5. "AC-15: No specification/implementation discrepancies remain"

**Current State:**
- Specification says `/peek <table>` (no row count parameter)
- Implementation supports `/peek <table> [N]` (optional row count)
- Pager sections lack status badges (experimental/stable indicators)

**Feature Characteristics:**

**User Interaction Type:** Documentation Only
- NO code changes
- NO user-facing behavior changes
- Documentation accuracy improvements only

**Observable Behavior:**
- No observable behavior changes
- Documentation becomes more accurate

**External Dependencies:**
- None - Pure documentation updates

**Validation Challenges:**
- Challenge 1: How to "test" documentation accuracy?
- Challenge 2: No automated tests can verify doc/code alignment
- Challenge 3: Requires human verification (code review)

**Critical Behaviors to Validate:**
1. Behavior 1: Specification documents optional `[N]` parameter for `/peek` (AC-11, AC-14)
2. Behavior 2: Pager section headers include status badges (AC-12)
3. Behavior 3: No discrepancies remain between spec and implementation (AC-15)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Documentation only" (no code changes):
  → Manual verification REQUIRED
  → Automated tests NOT APPLICABLE
  Reason: Cannot unit test documentation accuracy

IF "Spec/implementation alignment":
  → Code review REQUIRED
  Reason: Must manually verify docs match code behavior
```

**Derived Test Types:**

**Test Type 1: Manual Documentation Review**
- **Validates:** Documentation accuracy and completeness (AC-11, AC-12, AC-14)
- **Approach:** Human reads updated documentation, verifies syntax, badges, completeness
- **Rationale:** No automated way to validate prose documentation
- **Gap if missing:** Cannot claim documentation is accurate
- **Necessity:** ✅ REQUIRED

**Test Type 2: Code Review (Spec/Impl Alignment)**
- **Validates:** Specification matches actual implementation behavior (AC-13, AC-15)
- **Approach:** Compare spec requirements to code implementation, identify discrepancies
- **Rationale:** Only way to verify spec and code are synchronized
- **Gap if missing:** Cannot prove alignment achieved
- **Necessity:** ✅ REQUIRED

**Test Type 3: Regression Tests (Existing Suite)**
- **Validates:** No code changes introduced by accident (AC-15)
- **Approach:** Run full test suite, verify 100% pass rate (no changes = no failures)
- **Rationale:** Guard against accidental code modifications
- **Gap if missing:** Cannot prove documentation-only sprint didn't introduce code changes
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Manual documentation review | ✅ REQUIRED | Only way to validate prose documentation | Doc errors, inaccuracies | MUST PERFORM |
| Code review (alignment) | ✅ REQUIRED | Validates spec matches implementation | Spec/impl discrepancies | MUST PERFORM |
| Regression tests (full suite) | ✅ REQUIRED | Proves no accidental code changes | Accidental modifications | MUST RUN |
| Unit tests (new) | ❌ NOT NEEDED | No code changes to test | N/A | SKIP |
| Integration tests (new) | ❌ NOT NEEDED | No integration changes | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 3 - Manual review, code review, regression tests
- ⚠️ RECOMMENDED test types: 0
- ❌ NOT NEEDED test types: 2 - No new unit/integration tests needed

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| AC-11 | "/peek specification updated to allow [N] parameter" | sprint-34-planning.md §AC-11 | Manual review | Verify REQ-SAMPLE-004.1 updated | Doc inspection |
| AC-12 | "Pager status badges added to repl.md section headers" | sprint-34-planning.md §AC-12 | Manual review | Verify badges present | Doc inspection |
| AC-13 | "Specification matches implementation behavior" | sprint-34-planning.md §AC-13 | Code review | Compare spec to code | Alignment check |
| AC-14 | "User documentation reflects accurate /peek syntax" | sprint-34-planning.md §AC-14 | Manual review | Verify examples show [N] | Doc inspection |
| AC-15 | "No specification/implementation discrepancies remain" | sprint-34-planning.md §AC-15 | Code review + Regression | Comprehensive alignment | Full verification |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements (missing test coverage)
- [x] No unjustified test types (test types without requirement rationale)

**Coverage Gaps:**
- None identified for Track 3

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Unit Tests**
- **Reason for omission:** No code changes to test
- **What won't be validated:** N/A - documentation only
- **Risk assessment:** NONE - no code changes
- **Mitigation:** Regression tests ensure no accidental code changes
- **Revisit criteria:** N/A

**Integration Tests**
- **Reason for omission:** No behavior changes to integrate
- **What won't be validated:** N/A - documentation only
- **Risk assessment:** NONE - no code changes
- **Mitigation:** Regression tests ensure no accidental code changes
- **Revisit criteria:** N/A

#### 6. Test Implementation Plan

**Test Type: Manual Documentation Review**
- **Location:** `docs/specifications/repl.md` - human inspection
- **Framework:** N/A (manual review)
- **Test count estimate:** 3 verification checks
- **Key scenarios to cover:**
  1. Verify REQ-SAMPLE-004.1 syntax shows `/peek <table> [N]`
  2. Verify pager section headers include status badges (🧪 Experimental, ✅ Stable)
  3. Verify examples and usage text accurate
- **Implementation notes:** Quality validator performs review, documents findings in test report

**Test Type: Code Review (Spec/Impl Alignment)**
- **Location:** Compare `docs/specifications/repl.md` to `src/commands/sample.rs`, `src/commands/repl/metacommands.rs`
- **Framework:** Manual code inspection
- **Test count estimate:** 2 alignment checks
- **Key scenarios to cover:**
  1. `/peek` command: Verify spec matches code behavior (optional N parameter)
  2. Pager status: Verify spec reflects experimental/stable status correctly
- **Implementation notes:** Line-by-line comparison, document any remaining discrepancies

**Test Type: Regression Tests (Full Suite)**
- **Location:** All existing test locations
- **Framework:** Existing test infrastructure
- **Test count estimate:** 471 existing tests (same as Tracks 1 and 2)
- **Key scenarios to cover:**
  1. All existing unit tests
  2. All existing integration tests
  3. All existing interactive tests
- **Setup requirements:** Standard test environment

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Manual review validates: Documentation accuracy, syntax correctness, badge presence
- Code review validates: Spec/impl alignment achieved
- Regression tests validate: No accidental code changes

**Combined coverage:** COMPREHENSIVE

**Gaps in combined coverage:**
- No gaps identified - documentation validation complete

**Acceptance criteria:**
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified"
- [x] Known gaps are documented and accepted

---

## Strategy Summary

**Total Features Analyzed:** 3 (one per track)

**Test Types Required:**

**Track 1 (Code Quality):**
- Unit tests: ✅ (shared module validation)
- Regression tests: ✅ (471 tests)
- Code review: ✅ (duplication check)

**Track 2 (Security):**
- Unit tests: ✅ (quote_identifier function)
- Integration tests: ✅ (database with special chars)
- Regression tests: ✅ (471 tests)
- Interactive tests: ⚠️ RECOMMENDED (REPL edge cases)

**Track 3 (Documentation):**
- Manual review: ✅ (doc accuracy)
- Code review: ✅ (spec/impl alignment)
- Regression tests: ✅ (no code changes)

**Estimated Test Count:**

**New Tests to Create:**
- Track 1: 8-12 unit tests (type formatting)
- Track 2: 6-8 unit tests (quote_identifier) + 4-6 integration tests (database)
- Track 3: 0 new tests (manual review only)
- **Total New Tests:** 18-26 tests

**Existing Tests to Run:**
- Regression suite: 471 tests (384 lib + 87 integration/interactive)
- **Total Test Execution:** 489-497 tests

**Risk Assessment:**

**HIGH risk gaps:**
- None

**MEDIUM risk gaps:**
- Interactive REPL tests for SQL quoting edge cases (Track 2) - RECOMMENDED but not REQUIRED
  - Mitigation: Can add interactive tests if issues discovered during integration testing

**LOW risk gaps:**
- None

**Dependencies Required:**

- **Live database:** YES (for Track 2 integration tests, Track 2 regression tests)
- **Test database with special-named tables:** YES (for Track 2 edge case testing)
  - Setup required: Create test tables like `"My Test Table"`, `"table-2024"`, etc.
- **Network access:** NO
- **Specific OS:** NO
- **Other:** None

---

## Tool Requirements Assessment

### Existing Tools Evaluation

**Current Test Infrastructure (from `tests/README.md`):**

1. **Unit Test Framework:** Built-in Rust `#[test]` - ✅ SUFFICIENT
2. **Integration Test Framework:** `tests/integration_tests.rs` - ✅ SUFFICIENT
3. **Interactive Test Framework:** `tests/interactive_tests.rs` with expectrl - ✅ SUFFICIENT
4. **Database Connection:** TQ_LOGON environment variable support - ✅ SUFFICIENT

**Track 1 Requirements:** ✅ All tools available
- Need: Unit test framework for type formatting
- Have: Built-in Rust testing
- Gap: NONE

**Track 2 Requirements:** ⚠️ Database setup tool needed
- Need: Unit test framework for quote_identifier
- Have: Built-in Rust testing ✅
- Need: Integration tests with special character table names
- Have: Integration test framework ✅
- **Gap: Test database setup script** - Need to create tables with special characters

**Track 3 Requirements:** ✅ All tools available
- Need: Manual documentation review process
- Have: Quality validator review protocol
- Gap: NONE

### New Tools Required

**Tool #1: Test Database Setup Script**

**Purpose:** Create test tables with special character names for Track 2 integration testing

**Specification:**
```bash
# tests/tools/setup_special_tables.sql
# Creates test tables with special characters in names

CREATE TABLE "My Test Table" (
    id INTEGER,
    name VARCHAR(100),
    created_date DATE
);

CREATE TABLE "table-2024" (
    id INTEGER,
    value DECIMAL(10,2)
);

CREATE TABLE "customer_data" (  -- Normal name for regression
    id INTEGER,
    email VARCHAR(200)
);

-- Note: Tables with quotes in names ("customer""data") may not be
-- supported by Teradata or may require special handling
-- Test quote handling via unit tests, not integration tests
```

**Usage:**
```bash
# Run setup before integration tests
tq -l "$TQ_LOGON" < tests/tools/setup_special_tables.sql

# Run integration tests
cargo test --test integration_tests -- --ignored

# Cleanup (optional)
tq -l "$TQ_LOGON" < tests/tools/cleanup_special_tables.sql
```

**Alternative Approach:**
Integration tests can create their own tables at test startup and drop them at test end (preferred for isolation).

**Implementation Priority:** MEDIUM
- Required for Track 2 integration tests
- Can use inline table creation in tests instead of separate script
- Not blocking if tests handle table lifecycle internally

### Tool Sufficiency Conclusion

**Are existing tools sufficient for Sprint 34 validation?**

**Answer:** YES, with minor enhancement

**Existing tools cover:**
- ✅ Unit testing (Rust built-in)
- ✅ Integration testing (existing framework)
- ✅ Interactive testing (expectrl)
- ✅ Regression testing (cargo test)
- ✅ Code review (manual inspection + grep)

**Enhancement needed:**
- ⚠️ Test database setup for special character table names (Track 2)
  - Priority: MEDIUM
  - Can be handled inline in tests (not blocking)

**Recommendation to Coordinator:**
Proceed with test implementation using existing tools. Integration tests should create/cleanup their own test tables with special characters. No new testing framework needed.

---

## Strategy Validation Checklist

**Before submitting to tq-project-manager for review:**

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

**If ANY checkbox unchecked:** Strategy is incomplete, do not submit.

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-02-03
**Review Status:** DRAFT
**Submitted for Review:** [Pending submission to sprint coordinator]

**Reviewer:** sprint-coordinator
**Review Status:** PENDING
**Review Date:** [Pending]
**Review Comments:** [Awaiting coordinator feedback]

**Approval means:**
- ✅ Test strategy derived from specifications (not assumptions)
- ✅ All required test types identified with clear rationale
- ✅ Coverage gaps explicitly identified and assessed
- ✅ Implementation plan is detailed and achievable
- ✅ Ready to proceed with test implementation

**Approval signature:** [Pending sprint-coordinator review]
