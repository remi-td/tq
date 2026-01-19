# Sprint 13 Test Strategy

**Created:** 2026-01-19
**Author:** quality-validator
**Sprint:** Sprint 13
**Features:** Interactive Testing Framework, Tab Completion Fixes (3 issues), Logo Branding, Export Full Dataset, Export Syntax Simplification, Build Warning Cleanup

---

## Executive Summary

Sprint 13 addresses a critical quality crisis: tab completion has failed **four consecutive sprints** despite 100% unit test pass rates. The root cause is a fundamental test coverage gap—**unit tests verify logic, but interactive features need interactive tests**.

**Key Insight from Failure Analysis:**
> "The issue is NOT code bugs - it's a fundamental gap between what we test and what users experience." (tab-completion-failure-analysis.md)

This strategy derives test types from feature characteristics using the decision tree approach, ensuring we test **what users see**, not just **what code does**.

**Strategy Changes from Previous Sprints:**
1. **Interactive tests (expectrl) MANDATORY** for all REPL features
2. **User validation REQUIRED** for all UX/visual features
3. **Manual validation** for features where automated tests have limitations
4. **Honest gap analysis** documenting what we cannot test

---

## Feature-by-Feature Test Strategy

### Feature 1: Interactive Testing Framework (Meta-Feature)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/sprints/sprint-13-planning.md` lines 36-54
- Secondary: `docs/builder/testing-guidelines.md` lines 719-798
- Failure Analysis: `docs/builder/sprints/tab-completion-failure-analysis.md` lines 430-459

**Requirements:**
1. "At least 5 interactive tests implemented in `tests/interactive_tests.rs`"
2. "Test: Database/table completion shows correct visual output (not keywords)"
3. "Test: Completion inserts at cursor position (not line start)"
4. "Test: Reserved word completion (`sel `→`SELECT`, `fr`→`FROM`)"
5. "Test: Multi-line context preservation"
6. "Test: Schema-qualified completion (`database.<Tab>` shows tables)"
7. "All interactive tests passing with real tq binary"
8. "Test framework documented in `testing-guidelines.md`"

**Feature Characteristics:**

**User Interaction Type:** ✅ **Pure Logic + Framework Infrastructure**
- This is not a user-facing feature itself
- Enables testing of interactive features
- Provides test infrastructure for PTY simulation

**Explanation:** This feature creates the *testing infrastructure* for validating interactive PTY features. It's a meta-feature that doesn't have user-observable behavior itself, but enables validation of features that do.

**Observable Behavior:**
- ✅ Test execution output (pass/fail results)
- ❌ Not user-facing (infrastructure only)

**External Dependencies:**
- ✅ Terminal/PTY (expectrl simulates PTY for testing)
- ✅ Database connection (tests run against live database)
- ✅ File system access (test binary execution)

**Validation Challenges:**
- "PTY simulation may not perfectly match real terminal behavior"
- "Timing-sensitive interactions (Tab key press detection)"
- "Cross-platform terminal differences (macOS vs Linux)"
- "Framework must detect real bugs while minimizing false positives"

**Critical Behaviors to Validate:**
1. Framework can spawn tq REPL in PTY environment (from expectrl requirements)
2. Framework can send keystrokes (Tab, Enter, Ctrl-C) and observe output (from expectrl requirements)
3. Framework can capture visual terminal output including completion suggestions (Sprint 13 requirement)
4. Framework can validate cursor position and insertion point (Sprint 13 requirement - Issue 2)
5. Tests execute reliably without flakiness (stability requirement)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Database connection" checked:
  → Integration tests with live database REQUIRED
  Reason: Framework tests must validate against real Teradata database behavior

IF "Framework infrastructure" (special case):
  → Unit tests REQUIRED
  Reason: Framework code itself needs logic validation

IF "Test execution output" checked:
  → Integration tests REQUIRED
  Reason: Framework must be validated by running actual tests
```

**Derived Test Types:**

**Test Type 1: Unit Tests (Framework Code)**
- **Validates:** Internal framework logic (test helpers, PTY setup, utilities)
- **Approach:** Standard Rust unit tests in framework module
- **Rationale:** Framework code needs validation like any other code
- **Gap if missing:** Framework bugs could cause false positives/negatives in interactive tests
- **Necessity:** ⚠️ RECOMMENDED

**Test Type 2: Integration Tests (Framework Validation)**
- **Validates:** Framework can actually run interactive tests successfully
- **Approach:** Create minimal "smoke test" interactive tests that validate framework works
- **Rationale:** Proves framework itself functions correctly before using it to test features
- **Gap if missing:** Framework might be broken, giving misleading test results
- **Necessity:** ✅ REQUIRED

**Test Type 3: Interactive Tests (Example Tests)**
- **Validates:** Framework can detect real bugs in REPL features
- **Approach:** Implement the 5 required interactive tests specified in acceptance criteria
- **Rationale:** These tests validate tab completion AND prove the framework works
- **Gap if missing:** Framework exists but doesn't catch the bugs it was designed to find
- **Necessity:** ✅ REQUIRED

**Test Type 4: Manual Validation**
- **Validates:** Framework documentation is complete and accurate
- **Approach:** Human reads testing-guidelines.md updates, follows examples
- **Rationale:** Documentation quality can't be fully automated
- **Gap if missing:** Framework might work but be undocumented or poorly explained
- **Necessity:** ⚠️ RECOMMENDED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (framework) | ⚠️ RECOMMENDED | Validates framework helper code | Framework logic bugs | SHOULD IMPLEMENT |
| Integration tests (framework validation) | ✅ REQUIRED | Proves framework actually works | Framework silently broken | MUST IMPLEMENT |
| Interactive tests (5 example tests) | ✅ REQUIRED | Validates framework catches real bugs | Framework exists but useless | MUST IMPLEMENT |
| Manual validation (documentation) | ⚠️ RECOMMENDED | Human validates documentation quality | Documentation incomplete/wrong | DOCUMENT CHECKLIST |

**Summary:**
- ✅ REQUIRED test types: 2 - Integration tests (framework validation), Interactive tests (5 examples)
- ⚠️ RECOMMENDED test types: 2 - Unit tests (framework code), Manual validation (docs)
- ❌ NOT NEEDED test types: 0

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| FW-REQ-1 | "At least 5 interactive tests implemented" | sprint-13-planning.md:40 | Interactive | Count of tests in tests/interactive_tests.rs | IC001-IC005 |
| FW-REQ-2 | "Test: Database/table completion shows correct visual output" | sprint-13-planning.md:41 | Interactive | Validates framework can observe terminal output | IC001 |
| FW-REQ-3 | "Test: Completion inserts at cursor position" | sprint-13-planning.md:42 | Interactive | Validates framework can detect cursor issues | IC002 |
| FW-REQ-4 | "Test: Reserved word completion" | sprint-13-planning.md:43 | Interactive | Validates framework can test keyword completion | IC003 |
| FW-REQ-5 | "Test: Multi-line context preservation" | sprint-13-planning.md:44 | Interactive | Validates framework handles multi-line PTY state | IC004 |
| FW-REQ-6 | "Test: Schema-qualified completion" | sprint-13-planning.md:45 | Interactive | Validates framework can test qualified names | IC005 |
| FW-REQ-7 | "All interactive tests passing" | sprint-13-planning.md:46 | Integration | Validates framework itself works | Framework smoke test |
| FW-REQ-8 | "Test framework documented" | sprint-13-planning.md:47 | Manual | Human validates documentation quality | Documentation review |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements
- ✅ No unjustified test types

**Coverage Gaps:**
- None identified for this feature

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Benchmark/Performance Tests**
- **Reason for omission:** Framework execution speed not specified in requirements
- **What won't be validated:** Framework overhead, test execution timing
- **Risk assessment:** LOW - Framework is for testing, not production use
- **Mitigation:** Monitor test execution time manually, optimize if tests become slow
- **Revisit criteria:** If interactive tests take >5 seconds each to execute

#### 6. Test Implementation Plan

**Test Type: Integration Tests (Framework Validation)**
- **Location:** `tests/interactive_tests.rs` (same file as interactive tests)
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 1-2 smoke tests
- **Key scenarios to cover:**
  1. Framework can spawn tq REPL successfully
  2. Framework can send commands and receive output
  3. Framework can detect connection to database
- **Implementation notes:** These run first to validate framework before other tests

**Test Type: Interactive Tests (5 Required Examples)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 5 tests (per acceptance criteria)
- **Key scenarios to cover:**
  1. **IC001:** Type `SELECT * FROM ` then Tab → shows database names, NOT "(SQL keyword)"
  2. **IC002:** Type `SELECT * FROM db` then Tab → completion inserts at cursor, not line start
  3. **IC003:** Type `sel ` then Tab → autocompletes to `SELECT`; type `fr` then Tab → autocompletes to `FROM`
  4. **IC004:** Multi-line SQL: line 1 `SELECT *`, line 2 `FROM ` then Tab → shows databases (context preserved)
  5. **IC005:** Type `DBC.` then Tab → shows tables in DBC database
- **Implementation notes:**
  - Requires live database connection (TQ_LOGON environment variable)
  - May need timing delays for Tab completion to process
  - Must validate actual terminal output, not just internal state
  - Anti-pattern validation: explicitly check "(SQL keyword)" does NOT appear

**Test Type: Manual Validation (Documentation)**
- **Location:** Checklist in this document
- **Framework:** Human review
- **Test count estimate:** 1 documentation review session
- **Key scenarios to cover:**
  1. testing-guidelines.md updated with interactive test section
  2. Examples are clear and runnable
  3. Framework limitations documented
  4. Setup instructions complete
- **Implementation notes:** Performed by quality-validator after implementation

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Integration tests validate: Framework can spawn PTY, send commands, capture output
- Interactive tests validate: Framework detects real tab completion bugs (the whole point)
- Manual validation ensures: Documentation explains how to use framework
- Combined coverage: **COMPREHENSIVE**

**Gaps in combined coverage:**
- Framework tested on single platform only (development machine OS)
- Cross-platform terminal differences not validated (Linux/macOS/Windows)
- Performance characteristics not measured

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**Gaps are acceptable because:**
- Gap 1 (single platform): Development primarily on macOS, cross-platform testing in future sprints
- Gap 2 (performance): No performance requirements specified, framework just needs to work
- Gap 3 (terminal differences): expectrl handles most portability, major terminals work similarly

---

### Feature 2: Fix Tab Completion (All Three Issues)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/detailed-specifications/repl-mode.md` §5.6 Tab Completion
- Sprint Planning: `docs/builder/sprints/sprint-13-planning.md` lines 58-96
- Failure Analysis: `docs/builder/sprints/tab-completion-failure-analysis.md` lines 179-221

**Requirements (User-Reported Issues):**
1. **Issue 1:** "After `SELECT * FROM `, Tab shows database names (verified by interactive test)" (sprint-13-planning.md:83)
2. **Issue 2:** "Completion inserts at cursor position (verified by interactive test)" (sprint-13-planning.md:84)
3. **Issue 3:** "`sel `+Tab completes to `SELECT`, `fr`+Tab completes to `FROM` (verified by interactive test)" (sprint-13-planning.md:85)
4. "All existing unit tests still passing" (sprint-13-planning.md:86)
5. "All new interactive tests passing" (sprint-13-planning.md:87)
6. "Manual test cases TC027, TC028 executed and documented" (sprint-13-planning.md:88)
7. "User validation completed and approved" (sprint-13-planning.md:89)

**Specification Requirements from repl-mode.md:**
- §5.6.2 line 550: "Enable users to discover and navigate database tables through tab completion"
- §5.6.2 line 558: "Press Tab after typing partial table name following these SQL keywords: FROM, JOIN, UPDATE, INTO"
- §5.6.1 line 486: "Press Tab key after typing partial SQL keyword"

**Feature Characteristics:**

**User Interaction Type:** ✅ **Interactive PTY**
- REPL feature with terminal UI
- Cursor control and positioning
- Visual rendering of completion suggestions
- Real-time response to Tab key press

**Explanation:** Tab completion is a quintessential interactive feature. The user types in a terminal, presses Tab, and sees completion suggestions appear. The cursor position, visual output, and context detection are all critical user-observable behaviors that only exist in an interactive PTY environment.

**Observable Behavior:**
- ✅ **Visual output in terminal** (completion suggestions displayed to user)
- ✅ **State management** (completion context, cursor position, multi-line buffer)
- ❌ Structured data output (completion is visual, not data export)
- ❌ File system side effects
- ❌ Database side effects (reads metadata but doesn't modify)

**External Dependencies:**
- ✅ **Database connection** (queries DBC.TablesV, DBC.DatabasesV for completions)
- ✅ **Terminal/PTY** (Tab key handling, cursor positioning, visual rendering)
- ❌ File system access
- ❌ Network access (beyond database)

**Validation Challenges:**
1. "Visual rendering in terminal requires actual PTY, not just string comparison" - Can't test with unit tests alone
2. "Cursor position detection requires real terminal control sequences" - Must observe actual insertion point
3. "Context detection spans multiple lines, stored in reedline state" - Needs multi-line PTY simulation
4. "Timing sensitivity: Tab key must trigger completion within reasonable time" - Interactive tests may need delays
5. "Database metadata queries must return actual results" - Requires live database, not mocks

**Critical Behaviors to Validate (from specifications):**
1. "Press Tab after `SELECT * FROM ` shows database names" (repl-mode.md §5.6.2 line 559)
2. "Completion inserts text at current cursor position" (implied by Issue 2 report)
3. "Partial keyword `sel ` autocompletes to `SELECT`" (Issue 3 requirement)
4. "No keyword fallback in table context" (repl-mode.md §5.6.2, Sprint 8 fix)
5. "Multi-line context preserved" (Sprint 9 fix, requirement from failure analysis)
6. "Schema-qualified completion `database.<Tab>` shows tables" (repl-mode.md §5.6.2 Sprint 8 behavior)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
✅ "Interactive PTY" characteristic:
  → Interactive tests (expectrl) REQUIRED
  Reason: Unit tests cannot validate terminal output, cursor behavior, visual rendering
  GAP IF OMITTED: Visual bugs (wrong output shown), cursor position errors, rendering issues

✅ "Visual output in terminal" behavior:
  → Interactive tests OR integration tests with output capture REQUIRED
  Reason: Unit tests cannot validate formatting, colors, layout, alignment
  GAP IF OMITTED: Formatting bugs, layout issues, completion menu rendering problems

✅ "Database connection" dependency:
  → Integration tests with live database REQUIRED
  Reason: Mocks don't catch SQL syntax errors, query performance, real metadata issues
  GAP IF OMITTED: Metadata queries broken, DBC views not accessible, permission errors

✅ "State management" behavior:
  → Interactive tests with state verification REQUIRED
  Reason: Multi-line buffer state only exists in real PTY session
  GAP IF OMITTED: Context lost across line breaks, buffer corruption

ALSO (from existing codebase):
  → Unit tests REQUIRED (already exist)
  Reason: Internal completion logic needs validation (context detection, caching)
  GAP IF OMITTED: Logic bugs in completion algorithms
```

**Critical Insight from Failure Analysis (lines 155-176):**
> "Unit tests create a false sense of security for interactive features:
> 1. Mock Database: Tests use MockDatabaseClient, not real Teradata
> 2. Mock Terminal: Tests call complete() directly, not through reedline PTY
> 3. Mock Context: Tests provide perfect context strings, not real multi-line buffers
> 4. No Cursor Position: Tests don't verify where completion inserts text
> 5. No Rendering: Tests don't see what user sees (keywords vs. table names)"

**Derived Test Types:**

**Test Type 1: Unit Tests (Already Exist)**
- **Validates:** Internal completion logic, context detection, caching, filtering
- **Approach:** Existing unit tests in `src/commands/repl/metadata_completer.rs` tests module
- **Rationale:** Validates algorithms work correctly in isolation
- **Gap if missing:** Logic bugs in completion algorithms, caching issues
- **Necessity:** ✅ REQUIRED (already implemented, must keep passing)

**Test Type 2: Interactive Tests (expectrl) - NEW, BLOCKING**
- **Validates:** What user actually sees in terminal, cursor position, visual rendering
- **Approach:** expectrl PTY simulation in `tests/interactive_tests.rs`
- **Rationale:** **ONLY way to validate user-observable behavior** - unit tests failed to catch these bugs for 4 sprints
- **Gap if missing:** All 3 reported issues would remain undetected (as they were for 4 sprints)
- **Necessity:** ✅ REQUIRED - **BLOCKING** - Cannot claim tab completion works without these

**Test Type 3: Integration Tests with Live Database**
- **Validates:** Real database metadata queries work, DBC views accessible, permissions OK
- **Approach:** Tests that spawn REPL with real database, query metadata
- **Rationale:** Mock database doesn't catch SQL syntax errors, view access issues
- **Gap if missing:** Metadata queries might be broken against real Teradata
- **Necessity:** ✅ REQUIRED - Interactive tests cover this (they use live database)

**Test Type 4: Manual Test Cases (TC027, TC028)**
- **Validates:** Human validation of completion UX quality, subjective usability
- **Approach:** Execute existing manual test cases, document results
- **Rationale:** Some UX aspects need human judgment (is completion helpful? responsive? intuitive?)
- **Gap if missing:** Automated tests pass but UX is poor
- **Necessity:** ⚠️ RECOMMENDED - Provides additional validation beyond automated tests

**Test Type 5: User Validation - MANDATORY**
- **Validates:** Actual user (not agent) confirms all 3 issues are fixed
- **Approach:** User tests in real workflow, validates each issue resolved
- **Rationale:** User reported bugs; only user can confirm they're actually fixed
- **Gap if missing:** Agent thinks it's fixed, user still experiences bugs (has happened 4 times)
- **Necessity:** ✅ REQUIRED - **Cannot close sprint without user sign-off**

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Internal logic validation (already exist) | Algorithm bugs, logic errors | MUST KEEP PASSING |
| Interactive tests (expectrl) | ✅ REQUIRED | **Only way to validate what user sees** | Visual bugs, cursor errors, rendering issues - **ALL 3 REPORTED BUGS** | MUST IMPLEMENT |
| Integration tests (live DB) | ✅ REQUIRED | Real metadata queries validation | SQL errors, DBC view issues, permissions | COVERED BY INTERACTIVE TESTS |
| Manual tests (TC027, TC028) | ⚠️ RECOMMENDED | Human UX validation | Subjective usability issues | SHOULD EXECUTE |
| User validation | ✅ REQUIRED | User confirms bugs actually fixed | False positive (agents think fixed, user disagrees) | MUST OBTAIN |

**Summary:**
- ✅ REQUIRED test types: 3 - Unit tests (existing), Interactive tests (NEW), User validation
- ⚠️ RECOMMENDED test types: 1 - Manual tests
- ❌ NOT NEEDED test types: 0

**CRITICAL NOTE:** Feature 1 (Interactive Testing Framework) is a **BLOCKING DEPENDENCY** for this feature. Cannot implement interactive tests without the framework.

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| TC-ISSUE-1 | "After SELECT * FROM, Tab shows database names" | sprint-13-planning.md:83 | Interactive (expectrl) | Only interactive test observes terminal output | IC001 |
| TC-ISSUE-1-NEG | "Does NOT show '(SQL keyword)' after FROM" | Failure analysis anti-pattern | Interactive (expectrl) | Explicit negative test for known failure mode | IC001 |
| TC-ISSUE-2 | "Completion inserts at cursor position" | sprint-13-planning.md:84 | Interactive (expectrl) | Cursor position only testable in PTY | IC002 |
| TC-ISSUE-3A | "sel + Tab completes to SELECT" | sprint-13-planning.md:85 | Interactive (expectrl) | Keyword completion visual output | IC003 |
| TC-ISSUE-3B | "fr + Tab completes to FROM" | sprint-13-planning.md:85 | Interactive (expectrl) | Keyword completion visual output | IC003 |
| TC-REQ-4 | "All existing unit tests still passing" | sprint-13-planning.md:86 | Unit | No regressions in internal logic | Existing tests |
| TC-REQ-5 | "All new interactive tests passing" | sprint-13-planning.md:87 | Interactive | Framework and tests work | IC001-IC005 |
| TC-REQ-6 | "Manual test cases TC027, TC028 executed" | sprint-13-planning.md:88 | Manual | Human validation documented | TC027, TC028 |
| TC-REQ-7 | "User validation completed and approved" | sprint-13-planning.md:89 | User Validation | User confirms bugs fixed | User sign-off |
| TC-SPEC-1 | "Press Tab after FROM shows database names" | repl-mode.md §5.6.2:559 | Unit + Interactive | Unit tests logic, interactive tests UX | Existing unit + IC001 |
| TC-SPEC-2 | "No keyword fallback in table context" | repl-mode.md §5.6.2 Sprint 8 | Unit + Interactive | Unit tests logic, interactive validates no keywords shown | Existing unit + IC001 |
| TC-SPEC-3 | "Multi-line context preserved" | Sprint 9 requirement | Interactive | Multi-line PTY state only in interactive test | IC004 |
| TC-SPEC-4 | "Schema-qualified completion works" | repl-mode.md §5.6.2 Sprint 8 | Interactive | Teradata-specific database.table pattern | IC005 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements
- ✅ Anti-patterns explicitly tested (negative tests)

**Coverage Gaps:**
- None identified - comprehensive coverage with multiple test types

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Benchmark/Performance Tests**
- **Reason for omission:** No performance requirements in specification (completion must be "responsive" but no SLA)
- **What won't be validated:** Exact timing of Tab completion response, metadata query performance
- **Risk assessment:** LOW - Responsiveness is subjective, tested manually
- **Mitigation:** Manual test cases (TC027, TC028) include "feels responsive" assessment
- **Revisit criteria:** If users report completion is slow (>1 second delay)

**Cross-Platform Testing (Windows/Linux)**
- **Reason for omission:** Development environment is macOS, expectrl should be portable
- **What won't be validated:** Terminal behavior on Windows Terminal, Linux GNOME Terminal
- **Risk assessment:** MEDIUM - Terminal control sequences may differ slightly
- **Mitigation:** expectrl library handles most cross-platform differences
- **Revisit criteria:** If users on other platforms report bugs

**Requirements with Partial Coverage:**

*None identified - all requirements fully covered by combined test types*

**Known Testing Limitations:**

1. **PTY Cursor Position Validation:**
   - expectrl can observe output but cursor position detection may be unreliable
   - Mitigation: Test insertion by checking where text appears in output, not raw cursor coordinates
   - Impact: May not catch all cursor position bugs, but will catch obvious ones (Issue 2)

2. **Timing-Sensitive Interactions:**
   - Tab key completion may require brief delay to process
   - Tests may need `sleep()` calls, making them slower and potentially flaky
   - Mitigation: Use generous timeouts in interactive tests, accept slower execution

3. **Live Database Dependency:**
   - Tests require TQ_LOGON connection to real Teradata database
   - Tests will fail if database unavailable or credentials wrong
   - Mitigation: Clear setup instructions, skip tests gracefully if no connection

#### 6. Test Implementation Plan

**Test Type: Unit Tests (Existing - Keep Passing)**
- **Location:** `src/commands/repl/metadata_completer.rs` test module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** ~15-20 existing tests (exact count from codebase)
- **Key scenarios covered:** (already implemented)
  - Context detection logic
  - Completion filtering
  - Cache management
  - Mock database interactions
- **Action Required:** Ensure all existing tests still pass, no regressions

**Test Type: Interactive Tests (expectrl) - NEW**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 5 tests (as specified in Feature 1 acceptance criteria)
- **Key scenarios to cover:**
  1. **IC001: Issue 1 - Database Names After FROM**
     - User actions: Type `SELECT * FROM `, press Tab
     - Expected output: Completion menu shows database names (DBC, SYSUDTLIB, etc.)
     - Anti-pattern: Does NOT show "(SQL keyword)" repeated
     - Implementation: `p.send("SELECT * FROM \t")`, `p.expect("DBC")`, `p.expect_none("(SQL keyword)")`

  2. **IC002: Issue 2 - Cursor Position Insertion**
     - User actions: Type `SELECT * FROM database_name`, move cursor back, press Tab
     - Expected output: Completion inserts at cursor position, not line start
     - Implementation: Difficult - may need to observe where text appears in output
     - Workaround: Test simpler case - type partial word, Tab, verify completion at end of partial word

  3. **IC003: Issue 3 - Reserved Word Completion**
     - User actions: Type `sel `, press Tab → should complete to `SELECT`
     - User actions: Type `fr`, press Tab → should complete to `FROM`
     - Expected output: Full keyword appears after Tab
     - Implementation: `p.send("sel \t")`, `p.expect("SELECT")`

  4. **IC004: Multi-line Context Preservation**
     - User actions: Line 1: `SELECT *` + Enter, Line 2: `FROM ` + Tab
     - Expected output: Shows database names (context preserved from line 1)
     - Implementation: `p.send("SELECT *\n")`, `p.send("FROM \t")`, `p.expect("DBC")`

  5. **IC005: Schema-Qualified Completion**
     - User actions: Type `DBC.`, press Tab
     - Expected output: Shows tables in DBC database
     - Implementation: `p.send("DBC.\t")`, `p.expect("TablesV")` or similar DBC table

- **Implementation notes:**
  - All tests require live database (TQ_LOGON environment variable)
  - May need timing delays after Tab press: `std::thread::sleep(Duration::from_millis(100))`
  - Use `p.expect_any()` for partial matches, `p.expect_none()` for anti-patterns
  - Tests should be resilient to exact output format (completion menu rendering may vary)

**Test Type: Manual Tests (TC027, TC028)**
- **Location:** `tests/cases/TC027.md`, `tests/cases/TC028.md`
- **Framework:** Human execution with documentation
- **Test count estimate:** 2 existing test cases
- **Key scenarios to cover:**
  - TC027: Tab completion after JOIN clause
  - TC028: Tab completion after UPDATE statement
  - Both test cases already defined, just need execution and results documentation
- **Implementation notes:** Execute after interactive tests pass, document results in `tests/results/YYYYMMDD-HHMMSS/`

**Test Type: User Validation**
- **Location:** User validation checklist (to be created)
- **Framework:** User performs validation, agent documents
- **Test count estimate:** 3 validation checks (one per issue)
- **Key scenarios to cover:**
  1. User validates Issue 1 fixed (database names shown)
  2. User validates Issue 2 fixed (cursor position correct)
  3. User validates Issue 3 fixed (keyword completion works)
- **Implementation notes:** Create simple checklist for user, wait for sign-off

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- **Unit tests validate:** Internal logic, algorithms, context detection, caching (NECESSARY but INSUFFICIENT)
- **Interactive tests validate:** Visual output user sees, cursor position, real PTY behavior (CRITICAL - what was missing)
- **Manual tests validate:** Human subjective UX quality (additional confidence)
- **User validation ensures:** Actual user confirms bugs are fixed (final gate)
- **Combined coverage:** **COMPREHENSIVE and SUFFICIENT**

**Gaps in combined coverage:**
- Cross-platform terminal differences not tested (macOS only)
- Exact cursor position validation may be limited by expectrl capabilities
- Performance/timing not formally measured

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**Why combined coverage is sufficient:**

This is the **first sprint** where tab completion will have **proper test coverage**. Previous sprints had:
- Sprint 7: Unit tests only → bugs not caught
- Sprint 8: Unit tests only → bugs not caught
- Sprint 9: Unit tests only → bugs not caught
- Sprint 11: Unit tests only → bugs not caught
- Sprint 12: Unit tests only → bugs STILL not caught

Sprint 13 will have:
- ✅ Unit tests (internal logic)
- ✅ Interactive tests (visual output, cursor, real terminal)
- ✅ Manual tests (human UX validation)
- ✅ User validation (user confirms fixed)

**If all these pass, we can confidently claim tab completion works.**

The gaps (cross-platform, exact cursor position) are acceptable because:
- Gap 1 (platform): expectrl is portable, major terminals work similarly
- Gap 2 (cursor): We can validate insertion point indirectly by observing output
- Gap 3 (performance): Manual tests assess responsiveness subjectively

---

### Feature 3: Fix Logo Branding Issues

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/detailed-specifications/branding-guidelines.md` (entire document)
- Sprint Planning: `docs/builder/sprints/sprint-13-planning.md` lines 99-122

**Requirements:**
1. "Create `docs/builder/detailed-specifications/branding-guidelines.md` with complete design" (sprint-13-planning.md:111)
2. "Logo redesigned using █ block character per user specification" (sprint-13-planning.md:112)
3. "Logo last two lines properly aligned (no offset)" (sprint-13-planning.md:113)
4. "Tool name displayed as lowercase `tq` with 't' in Teradata orange" (sprint-13-planning.md:114)
5. "Interactive prompt `tq>` colored in Teradata orange (not green)" (sprint-13-planning.md:115)
6. "User validates and approves logo design" (sprint-13-planning.md:116)
7. "Implementation matches branding guidelines document" (sprint-13-planning.md:117)

**Specification Requirements from branding-guidelines.md:**
- Line 19: "Official Name: tq (all lowercase, always)"
- Line 35: "First letter 't' in Teradata orange (#F37021)"
- Line 88: "Character Set: Unicode block character █ (U+2588 Full Block)"
- Line 128: "All logo lines must be perfectly aligned (no offset)"
- Line 157: "Entire prompt tq> in Teradata orange (#F37021)"

**Feature Characteristics:**

**User Interaction Type:** ✅ **Interactive PTY (Visual/Terminal UI)**
- Logo displays in terminal on REPL startup
- Prompt appears in terminal during interactive session
- Visual rendering with colors and Unicode characters
- Terminal-specific rendering (colors, character encoding)

**Explanation:** Logo branding is a visual feature that users see in their terminal. The color rendering, character display, and alignment are all visual properties that only exist when viewing the terminal output. This is inherently an interactive visual feature.

**Observable Behavior:**
- ✅ **Visual output in terminal** (logo appearance, colors, alignment, prompt color)
- ❌ Structured data output
- ❌ File system side effects
- ❌ Database side effects
- ❌ Performance characteristics (logo just displays, no performance concern)
- ❌ State management (logo is stateless display)

**External Dependencies:**
- ✅ **Terminal/PTY** (color support, Unicode rendering, character encoding)
- ✅ **Operating system specific features** (color support varies by terminal emulator)
- ❌ Database connection (logo displays regardless of connection status)
- ❌ File system access
- ❌ Network access

**Validation Challenges:**
1. "Visual rendering differs across terminal emulators (iTerm2 vs Terminal.app vs GNOME)"
2. "Color rendering depends on terminal color support (truecolor vs 256-color vs 8-color)"
3. "Unicode block character █ may not render identically in all fonts"
4. "Alignment is visual - automated tests can check string length but not visual appearance"
5. "User's subjective judgment required - 'does it look professional?'"

**Critical Behaviors to Validate (from specifications):**
1. "Logo uses only █ block character (not | or _)" (branding-guidelines.md:88)
2. "Logo last two lines NOT offset" (branding-guidelines.md:128)
3. "Tool name always lowercase tq" (branding-guidelines.md:19)
4. "'t' letter in Teradata orange (#F37021)" (branding-guidelines.md:35)
5. "Prompt tq> in Teradata orange (not green)" (branding-guidelines.md:157)
6. "Logo lines perfectly aligned" (branding-guidelines.md:128)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
✅ "Interactive PTY" characteristic:
  → Interactive tests OR manual validation REQUIRED
  Reason: Visual terminal output cannot be validated by unit tests
  GAP IF OMITTED: Logo appears broken, offset, wrong colors, but tests pass

✅ "Visual output in terminal" behavior:
  → Manual validation REQUIRED (human visual inspection)
  Reason: Automated tests cannot judge "looks professional" or "visually aligned"
  GAP IF OMITTED: Logo passes string checks but looks ugly in terminal

✅ "Operating system specific features" dependency:
  → Manual tests on multiple terminal emulators REQUIRED
  Reason: Color and Unicode rendering varies by terminal
  GAP IF OMITTED: Logo works on development machine, broken on user's terminal

USER REQUIREMENT:
  → User validation MANDATORY
  Reason: User reported branding issues, user must approve design
  GAP IF OMITTED: Agent thinks logo is correct, user still unhappy
```

**Derived Test Types:**

**Test Type 1: Unit Tests (Logo String Content)**
- **Validates:** Logo string contains only █ character, no | or _, correct dimensions
- **Approach:** String validation tests on logo constant
- **Rationale:** Can catch obvious errors (wrong characters, wrong dimensions)
- **Gap if missing:** Logo might use wrong characters or be wrong size
- **Necessity:** ⚠️ RECOMMENDED

**Test Type 2: Integration Tests (Logo Display)**
- **Validates:** Logo displays on REPL startup, prompt appears with color
- **Approach:** Spawn REPL, capture output, verify logo appears
- **Rationale:** Proves logo actually renders (doesn't crash, isn't blank)
- **Gap if missing:** Logo code exists but doesn't actually display
- **Necessity:** ✅ REQUIRED

**Test Type 3: Manual Validation (Visual Inspection)**
- **Validates:** Logo looks correct visually (alignment, colors, appearance)
- **Approach:** Human views logo in terminal, checks against branding guidelines
- **Rationale:** **Only way to validate visual quality** - automated tests can't judge appearance
- **Gap if missing:** Logo displays but looks bad (offset, wrong colors, ugly)
- **Necessity:** ✅ REQUIRED - **BLOCKING for visual feature**

**Test Type 4: Manual Validation (Multi-Terminal)**
- **Validates:** Logo renders correctly in multiple terminal emulators
- **Approach:** Test in iTerm2, Terminal.app, and optionally Linux/Windows terminals
- **Rationale:** Color and Unicode support varies by terminal
- **Gap if missing:** Logo works on dev machine, broken elsewhere
- **Necessity:** ⚠️ RECOMMENDED

**Test Type 5: User Validation - MANDATORY**
- **Validates:** User approves logo design and implementation
- **Approach:** User views logo, confirms matches their vision
- **Rationale:** User requested specific branding, only user can approve
- **Gap if missing:** Agent-approved design doesn't match user expectations
- **Necessity:** ✅ REQUIRED - **Cannot close feature without user approval**

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (string content) | ⚠️ RECOMMENDED | Validates logo uses correct characters | Wrong characters in logo | SHOULD IMPLEMENT |
| Integration tests (display) | ✅ REQUIRED | Logo actually renders on startup | Logo doesn't display at all | MUST IMPLEMENT |
| Manual validation (visual) | ✅ REQUIRED | **Only way to validate appearance** | Logo displays but looks bad | MUST PERFORM |
| Manual validation (terminals) | ⚠️ RECOMMENDED | Cross-terminal compatibility | Works on dev machine, fails elsewhere | SHOULD TEST |
| User validation | ✅ REQUIRED | User approves branding | User still unhappy with design | MUST OBTAIN |

**Summary:**
- ✅ REQUIRED test types: 3 - Integration tests (display), Manual validation (visual), User validation
- ⚠️ RECOMMENDED test types: 2 - Unit tests (string), Manual validation (multi-terminal)
- ❌ NOT NEEDED test types: 0

**CRITICAL NOTE:** Visual features like branding **cannot be fully validated by automated tests**. Manual human inspection is required.

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| LOGO-REQ-1 | "Logo redesigned using █ block character" | sprint-13-planning.md:112 | Unit + Manual | Unit checks character, manual validates appearance | Unit test + Visual inspection |
| LOGO-REQ-2 | "Logo last two lines properly aligned (no offset)" | sprint-13-planning.md:113 | Manual | **Only human can judge visual alignment** | Visual inspection checklist |
| LOGO-REQ-3 | "Tool name displayed as lowercase tq" | sprint-13-planning.md:114 | Unit + Integration | Unit checks string, integration verifies display | Unit test + Startup test |
| LOGO-REQ-4 | "t in Teradata orange" | sprint-13-planning.md:114 | Manual | Color rendering is visual | Visual inspection checklist |
| LOGO-REQ-5 | "Interactive prompt tq> colored in Teradata orange (not green)" | sprint-13-planning.md:115 | Manual + Integration | Integration captures output, manual validates color | Interactive test + Visual check |
| LOGO-REQ-6 | "User validates and approves logo design" | sprint-13-planning.md:116 | User Validation | User approval mandatory | User sign-off |
| LOGO-REQ-7 | "Implementation matches branding guidelines" | sprint-13-planning.md:117 | Manual | Human compares code to specification | Document review |
| LOGO-SPEC-1 | "Official Name: tq (all lowercase, always)" | branding-guidelines.md:19 | Unit | String constant validation | Unit test |
| LOGO-SPEC-2 | "Character Set: Unicode block character █" | branding-guidelines.md:88 | Unit + Manual | Unit checks character code, manual checks rendering | Unit test + Visual check |
| LOGO-SPEC-3 | "All logo lines must be perfectly aligned" | branding-guidelines.md:128 | Manual | **Visual alignment only validateable by human** | Visual inspection checklist |
| LOGO-SPEC-4 | "Entire prompt tq> in Teradata orange" | branding-guidelines.md:157 | Manual + Integration | Integration tests prompt exists, manual validates color | Interactive test + Visual check |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ Visual requirements explicitly marked as "manual only"
- ✅ No orphaned requirements

**Coverage Gaps:**
- None identified - comprehensive coverage with appropriate test types for visual feature

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Benchmark/Performance Tests**
- **Reason for omission:** Logo display is instantaneous, no performance requirements
- **What won't be validated:** Time to render logo
- **Risk assessment:** LOW - Logo is static text, no performance concern
- **Mitigation:** None needed
- **Revisit criteria:** Never (logo is just text output)

**Automated Color Validation**
- **Reason for omission:** ANSI color codes can be checked, but *perceived color* requires human eyes
- **What won't be validated:** Whether Teradata orange actually looks orange in user's terminal
- **Risk assessment:** MEDIUM - Terminal color rendering varies (dark vs light themes)
- **Mitigation:** Manual visual inspection in multiple terminal themes
- **Revisit criteria:** If we find a way to automate color perception testing

**Requirements with Partial Coverage:**

**Logo Alignment (LOGO-REQ-2, LOGO-SPEC-3):**
- Unit tests can check string lengths are equal (crude alignment check)
- Only manual inspection can validate visual alignment
- **Acceptable:** Visual features inherently require human validation

**Color Rendering (LOGO-REQ-4, LOGO-REQ-5, LOGO-SPEC-4):**
- Integration tests can verify ANSI color codes are present in output
- Only manual inspection can validate colors *look* correct
- **Acceptable:** Color perception is subjective and terminal-dependent

**Known Testing Limitations:**

1. **Terminal Emulator Variability:**
   - Logo may render differently in iTerm2 vs Terminal.app vs GNOME Terminal
   - Unicode █ character rendering depends on font
   - Limitation: Can't test all terminal emulators
   - Mitigation: Test on 2-3 popular terminals (iTerm2, Terminal.app)

2. **Color Theme Dependency:**
   - Teradata orange looks different on light vs dark backgrounds
   - Some users may have custom color schemes
   - Limitation: Can't test all possible themes
   - Mitigation: Test on standard dark and light themes

3. **Font Rendering:**
   - Block character █ appearance varies by font
   - Some fonts may have alignment issues
   - Limitation: Can't control user's font choice
   - Mitigation: Test with standard monospace fonts (Monaco, Courier)

#### 6. Test Implementation Plan

**Test Type: Unit Tests (String Content)**
- **Location:** `src/commands/repl/mod.rs` or `src/branding.rs` test module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 3-5 tests
- **Key scenarios to cover:**
  1. Logo constant contains only █ character (no | or _)
  2. Logo has correct dimensions (lines count, character count per line)
  3. Tool name constant is "tq" (all lowercase)
  4. No uppercase "TQ" or "Tq" anywhere in branding code
- **Implementation notes:** Simple string validation tests

**Test Type: Integration Tests (Logo Display)**
- **Location:** `tests/interactive_tests.rs` (uses expectrl)
- **Framework:** expectrl for PTY simulation
- **Test count estimate:** 2-3 tests
- **Key scenarios to cover:**
  1. **Logo displays on REPL startup:**
     - Spawn tq REPL
     - Expect logo text to appear in output
     - Verify "Teradata Query Tool" subtitle appears
  2. **Prompt displays with color:**
     - Spawn tq REPL
     - Expect prompt "tq>" appears
     - Verify ANSI color codes present (e.g., `\x1b[38;2;243;112;33m`)
  3. **Logo contains block character:**
     - Capture startup output
     - Verify █ (U+2588) character present
     - Verify | and _ characters NOT present in logo
- **Implementation notes:**
  - Tests verify logo *exists* and *displays*, not that it *looks good*
  - Color code validation proves color is applied, not that it looks correct

**Test Type: Manual Validation (Visual Inspection)**
- **Location:** Manual test case `tests/cases/TC-BRANDING.md` (to be created)
- **Framework:** Human visual inspection with checklist
- **Test count estimate:** 1 comprehensive visual inspection
- **Checklist to cover:**
  - [ ] Logo displays on REPL startup
  - [ ] Logo uses only █ block character (visually confirm)
  - [ ] Logo last two lines are NOT offset (perfectly aligned)
  - [ ] All logo lines are perfectly aligned (vertical alignment)
  - [ ] Tool name appears as lowercase "tq" (no uppercase)
  - [ ] 't' letter appears in orange color (Teradata orange)
  - [ ] 'q' letter appears in default terminal color
  - [ ] Interactive prompt "tq>" appears in orange color (not green)
  - [ ] Logo looks professional and well-formed
  - [ ] No visual artifacts, scrambled characters, or rendering issues
- **Implementation notes:**
  - Execute in multiple terminal emulators (iTerm2, Terminal.app minimum)
  - Test in both dark and light terminal themes
  - Document results with screenshots

**Test Type: User Validation**
- **Location:** User validation request and sign-off documentation
- **Framework:** User performs visual inspection, provides approval
- **Test count estimate:** 1 user approval session
- **Validation request to user:**
  1. Please start tq REPL and review the logo
  2. Verify it matches your branding expectations:
     - Lowercase "tq" with 't' in Teradata orange
     - Block character █ used throughout
     - Logo lines perfectly aligned (no offset)
     - Prompt "tq>" in Teradata orange (not green)
  3. Confirm the design is approved
- **Implementation notes:**
  - Create simple validation checklist for user
  - Wait for explicit user approval before marking feature complete

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- **Unit tests validate:** Logo uses correct characters, correct dimensions, lowercase naming
- **Integration tests validate:** Logo actually displays, colors are applied (codes present)
- **Manual validation ensures:** Logo looks correct visually, alignment is perfect, colors appear right
- **User validation confirms:** User is happy with branding design
- **Combined coverage:** **COMPREHENSIVE for visual feature**

**Gaps in combined coverage:**
- Logo not tested on all terminal emulators (just 2-3 popular ones)
- Logo not tested on all operating systems (development machine only)
- Color rendering not tested programmatically (subjective validation only)

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**Why combined coverage is sufficient:**

Visual features like logo branding **cannot be fully automated**. The nature of visual design requires human judgment:
- "Does this look professional?"
- "Are the lines aligned?"
- "Does the color look good on my terminal?"

These are subjective questions that automated tests cannot answer.

Our strategy combines:
1. ✅ Unit tests - catch obvious mistakes (wrong characters, wrong dimensions)
2. ✅ Integration tests - prove logo displays and has color codes
3. ✅ Manual validation - human confirms it looks good
4. ✅ User validation - user (who requested branding) approves

**This is appropriate coverage for a visual feature.**

The gaps are acceptable because:
- Gap 1 (all terminals): Testing 2-3 popular terminals covers majority of users
- Gap 2 (all OS): expectrl is portable, major terminals behave similarly
- Gap 3 (automated color): Color is subjective, human validation is correct approach

**If all test types pass AND user approves, branding is correct.**

---

### Feature 4: Verify and Fix Export Full Dataset

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/detailed-specifications/repl-mode.md` §5.8.4 Export Results (lines 2183-2295)
- Sprint Planning: `docs/builder/sprints/sprint-13-planning.md` lines 126-152
- User Report: User states feature "STILL DOESN'T WORK PROPERLY"

**Requirements:**
1. "Manual test: Execute `SELECT * FROM large_table;` (displays 100 rows)" (sprint-13-planning.md:140)
2. "Manual test: `/export csv output.csv` (verify file contains ALL rows, not 100)" (sprint-13-planning.md:141)
3. "Manual test: Query with `TOP 50` exports exactly 50 rows (respects user limit)" (sprint-13-planning.md:142)
4. "Interactive test: Verify re-execution query logic" (sprint-13-planning.md:143)
5. "Test with table containing 1000+ rows to prove it works" (sprint-13-planning.md:144)
6. "User validation completed and approved" (sprint-13-planning.md:145)
7. "Document actual behavior in test results" (sprint-13-planning.md:146)

**Specification Requirements from repl-mode.md:**
- Line 2212: "SELECT * FROM employees WHERE dept = 'IT'" followed by "/export csv employees_it.csv"
- Line 2217: "Exported 10 rows to employees_it.csv"
- Line 2274: "SELECT * FROM huge_table" exports "1,000,000 rows" (implying ALL rows, not display limit)
- No explicit specification of "export ALL rows, not just displayed rows"

**User Report Context:**
> "THIS STILL DOESN'T WORK PROPERLY: Export should allow to export ALL the dataset to a file: if I do a `select * from mytable;` you will limit the dataset to 100 rows... However, if I want to export to a file, I want to export ALL the dataset, not just the first 100 rows..."

**Feature Characteristics:**

**User Interaction Type:** ✅ **Interactive PTY + Backend**
- REPL metacommand `/export` (interactive)
- File writing (backend operation)
- Database query re-execution (backend operation)

**Explanation:** Export is triggered interactively (`/export` command in REPL) but performs backend operations (re-execute query, write file). The user interacts in REPL, but the core functionality is backend data processing.

**Observable Behavior:**
- ✅ **File system side effects** (CSV/JSON file created with data)
- ✅ **Database side effects** (query re-executed to get full dataset)
- ✅ Visual output in terminal (success message, row count)
- ❌ Structured data output (file contents, not terminal output)
- ❌ Performance characteristics (not specified)

**External Dependencies:**
- ✅ **Database connection** (must re-execute query to get all rows)
- ✅ **File system access** (write CSV/JSON file)
- ✅ **Terminal/PTY** (interactive command entry)
- ❌ Network access
- ❌ Operating system specific features

**Validation Challenges:**
1. "Must verify file contents, not just that file exists" - Requires reading and parsing exported file
2. "Display limit (100 rows) vs export limit (ALL rows) distinction" - Core issue user reports
3. "Query re-execution logic must be correct" - Backend behavior not visible in REPL
4. "Large dataset testing requires table with 1000+ rows" - Test data setup
5. "User's actual use case unknown" - What table/query is user trying to export?

**Critical Behaviors to Validate:**
1. "Export re-executes query to get ALL rows (not just displayed 100)" - Core user requirement
2. "Display shows 100 rows, but export file contains all rows" - Expected behavior
3. "Query with TOP N exports exactly N rows (respects user limit)" - User control
4. "Large dataset (1000+ rows) exports successfully" - Scalability
5. "Success message shows actual row count exported" - User feedback

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
✅ "Database connection" dependency:
  → Integration tests with live database REQUIRED
  Reason: Must verify query re-execution gets all rows
  GAP IF OMITTED: Backend logic might be broken, only displays 100 but exports 100 too

✅ "File system side effects" behavior:
  → Integration tests with file validation REQUIRED
  Reason: Must read exported file and verify row count
  GAP IF OMITTED: File created but contains wrong data

✅ "Interactive PTY" (command entry):
  → Interactive tests OR manual tests REQUIRED
  Reason: /export command entered in REPL, must test real workflow
  GAP IF OMITTED: Command might not work in real REPL session

USER REQUIREMENT:
  → Manual tests REQUIRED (user reported bug, need real verification)
  Reason: User says it's broken, automated tests may not catch their use case
  GAP IF OMITTED: Tests pass but user's actual scenario still fails
```

**Derived Test Types:**

**Test Type 1: Unit Tests (Export Logic)**
- **Validates:** Export logic correctly determines whether to re-execute query
- **Approach:** Unit tests on export command implementation
- **Rationale:** Core logic needs validation (display limit vs export limit)
- **Gap if missing:** Logic bugs in when to re-execute query
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests (Full Workflow)**
- **Validates:** Query execution + export + file creation with correct row count
- **Approach:** Execute query, run export, read file, verify row count
- **Rationale:** End-to-end validation of complete export workflow
- **Gap if missing:** Components work in isolation but not together
- **Necessity:** ✅ REQUIRED

**Test Type 3: Manual Tests (User Scenario)**
- **Validates:** Real user workflow with large table (1000+ rows)
- **Approach:** Human executes SELECT, then /export, verifies file contents
- **Rationale:** **User reported bug** - must test real scenario to confirm fix
- **Gap if missing:** Tests pass but user's use case still broken
- **Necessity:** ✅ REQUIRED - **BLOCKING - user validation depends on this**

**Test Type 4: Interactive Tests (REPL Command)**
- **Validates:** /export command works in actual REPL session
- **Approach:** expectrl test that executes query, then /export, verifies success message
- **Rationale:** Validates command entry and REPL integration
- **Gap if missing:** Command works in tests but not real REPL
- **Necessity:** ⚠️ RECOMMENDED

**Test Type 5: User Validation - MANDATORY**
- **Validates:** User confirms export now works for their use case
- **Approach:** User tests with their actual table and query
- **Rationale:** User reported bug, only user can confirm it's fixed
- **Gap if missing:** Agent thinks it works, user still experiences issue
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (export logic) | ✅ REQUIRED | Core re-execution logic validation | Logic bugs, wrong row count | MUST IMPLEMENT |
| Integration tests (workflow) | ✅ REQUIRED | End-to-end query + export + file | Components don't integrate | MUST IMPLEMENT |
| Manual tests (large table) | ✅ REQUIRED | Real user scenario with 1000+ rows | User's use case not tested | MUST EXECUTE |
| Interactive tests (REPL) | ⚠️ RECOMMENDED | REPL command integration | Command might not work in REPL | SHOULD IMPLEMENT |
| User validation | ✅ REQUIRED | User confirms bug actually fixed | False positive (works in test, fails for user) | MUST OBTAIN |

**Summary:**
- ✅ REQUIRED test types: 3 - Unit tests, Integration tests, Manual tests, User validation
- ⚠️ RECOMMENDED test types: 1 - Interactive tests
- ❌ NOT NEEDED test types: 0

**CRITICAL NOTE:** User explicitly stated "STILL DOESN'T WORK PROPERLY" - this is a **user-reported bug** that requires **real validation**, not just automated tests.

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| EXPORT-REQ-1 | "Execute SELECT * FROM large_table (displays 100 rows)" | sprint-13-planning.md:140 | Manual | Establish baseline: REPL shows 100 rows | Manual test step 1 |
| EXPORT-REQ-2 | "/export csv output.csv (verify file contains ALL rows)" | sprint-13-planning.md:141 | Manual + Integration | **Core issue: file must have all rows** | Manual test step 2 + INT-EXPORT-1 |
| EXPORT-REQ-3 | "Query with TOP 50 exports exactly 50 rows" | sprint-13-planning.md:142 | Unit + Integration | Validates user limit respected | Unit test + INT-EXPORT-2 |
| EXPORT-REQ-4 | "Verify re-execution query logic" | sprint-13-planning.md:143 | Unit + Integration | Backend logic correctness | Unit test + INT-EXPORT-1 |
| EXPORT-REQ-5 | "Test with table containing 1000+ rows" | sprint-13-planning.md:144 | Manual | Scalability and real user scenario | Manual test with large table |
| EXPORT-REQ-6 | "User validation completed and approved" | sprint-13-planning.md:145 | User Validation | User confirms bug fixed | User sign-off |
| EXPORT-REQ-7 | "Document actual behavior in test results" | sprint-13-planning.md:146 | Manual | Transparency about what actually happens | Manual test documentation |
| EXPORT-SPEC-1 | "Export last result to CSV" | repl-mode.md:2293 | Unit + Integration | Basic export functionality | Unit + INT-EXPORT-1 |
| EXPORT-SPEC-2 | "Large result handling (1,000,000 rows)" | repl-mode.md:2274-2281 | Manual (scaled) | Proves ALL rows exported, not display limit | Manual test (1000+ rows) |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ Core user issue (ALL rows, not 100) explicitly covered
- ✅ No orphaned requirements

**Coverage Gaps:**
- None identified - comprehensive coverage with focus on user-reported issue

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Benchmark/Performance Tests (Very Large Datasets)**
- **Reason for omission:** Specification shows 1,000,000 row example but no performance SLA
- **What won't be validated:** Export speed for million-row datasets
- **Risk assessment:** LOW - Functional correctness more important than speed
- **Mitigation:** Manual test with 1000+ rows validates scalability at smaller scale
- **Revisit criteria:** If users report export is too slow for large datasets

**Cross-Format Testing (JSON, SQL formats)**
- **Reason for omission:** Focus on CSV (user's reported use case), other formats likely similar
- **What won't be validated:** JSON and SQL export formats with full datasets
- **Risk assessment:** MEDIUM - Formats might have different code paths
- **Mitigation:** If CSV works, JSON/SQL likely work (same backend logic)
- **Revisit criteria:** If user reports issues with other formats

**Requirements with Partial Coverage:**

*None identified - all requirements have comprehensive coverage*

**Known Testing Limitations:**

1. **Test Database Data:**
   - Need table with 1000+ rows for realistic testing
   - May need to create test table if none exists
   - Limitation: Test data setup may be manual
   - Mitigation: Document test data requirements clearly

2. **File Size Validation:**
   - Large exports (100K+ rows) may create large files
   - Tests need to handle large file reading/parsing
   - Limitation: May slow down test execution
   - Mitigation: Use moderate size (1000-5000 rows) for manual test

3. **User's Actual Use Case Unknown:**
   - User didn't specify which table or query is failing
   - Tests use generic scenario, may not match user's exact case
   - Limitation: Tests might pass but user's scenario fails
   - Mitigation: User validation MANDATORY to confirm fix

#### 6. Test Implementation Plan

**Test Type: Unit Tests (Export Logic)**
- **Location:** `src/commands/repl/export.rs` or similar (wherever export is implemented)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 5-8 tests
- **Key scenarios to cover:**
  1. Export logic detects when to re-execute query (not just use cached results)
  2. Display limit (100) does NOT apply to export
  3. User-specified TOP N limit IS respected in export
  4. Row count calculation correct
  5. File writing logic handles large row counts
- **Implementation notes:** Focus on re-execution logic (core user issue)

**Test Type: Integration Tests (Full Workflow)**
- **Location:** `tests/integration_tests.rs` or `tests/export_tests.rs`
- **Framework:** Built-in Rust integration test
- **Test count estimate:** 3-5 tests
- **Key scenarios to cover:**
  1. **INT-EXPORT-1: Full dataset export**
     - Setup: Table with 200 rows (more than display limit of 100)
     - Execute: SELECT * FROM test_table
     - Export: /export csv test_output.csv
     - Verify: File contains 200 rows (not 100)
     - Verify: Success message shows "Exported 200 rows"

  2. **INT-EXPORT-2: User limit respected**
     - Setup: Table with 1000 rows
     - Execute: SELECT TOP 50 * FROM test_table
     - Export: /export csv limited_output.csv
     - Verify: File contains exactly 50 rows

  3. **INT-EXPORT-3: Small result set (< 100 rows)**
     - Setup: Table with 10 rows
     - Execute: SELECT * FROM small_table
     - Export: /export csv small_output.csv
     - Verify: File contains 10 rows (no issue when under display limit)

- **Implementation notes:**
  - Requires live database with test tables
  - Must read and parse CSV files to count rows
  - Clean up test files after execution

**Test Type: Manual Tests (User Scenario)**
- **Location:** `tests/cases/TC-EXPORT-FULL.md` (to be created)
- **Framework:** Human execution with documentation
- **Test count estimate:** 1 comprehensive manual test
- **Test procedure:**
  1. **Setup:** Identify or create table with 1000+ rows in test database
  2. **Execute Query:** Run `SELECT * FROM large_table;` in tq REPL
  3. **Observe Display:** Verify REPL shows "Showing first 100 rows" message
  4. **Export:** Run `/export csv full_export.csv`
  5. **Verify File:** Open full_export.csv and count rows (should be 1000+, not 100)
  6. **Verify Message:** Check success message shows actual row count (e.g., "Exported 1234 rows")
  7. **Document:** Record actual row counts, screenshot if helpful
- **Expected Results:**
  - Display shows 100 rows
  - Export file contains ALL rows (1000+)
  - Success message shows full count
- **Implementation notes:**
  - This is the **critical test** that validates user's reported issue is fixed
  - Must document exact numbers (table row count, exported row count)
  - If this fails, feature is still broken

**Test Type: Interactive Tests (REPL Command) - RECOMMENDED**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl
- **Test count estimate:** 1-2 tests
- **Key scenarios to cover:**
  1. Execute query in REPL, run /export command, verify success message
  2. Verify /export command doesn't crash or hang
- **Implementation notes:**
  - Validates REPL command integration
  - File verification done in integration tests (simpler)

**Test Type: User Validation**
- **Location:** User validation request and documentation
- **Framework:** User tests with their actual data
- **Test count estimate:** 1 user validation session
- **Validation request to user:**
  1. Please test export with your actual table that was failing
  2. Execute SELECT * FROM [your table]
  3. Run /export csv [output file]
  4. Verify the exported file contains ALL rows, not just 100
  5. Confirm export now works as expected
- **Implementation notes:**
  - Critical - user reported the bug, only user can confirm fix
  - May need to help user if they have trouble testing

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- **Unit tests validate:** Export re-execution logic, limit handling
- **Integration tests validate:** End-to-end workflow with file verification
- **Manual tests validate:** Real large dataset scenario (1000+ rows)
- **User validation ensures:** User's actual use case now works
- **Combined coverage:** **COMPREHENSIVE**

**Gaps in combined coverage:**
- Very large datasets (100K+ rows) not tested (performance unknown)
- JSON and SQL formats not explicitly tested
- User's specific table/query not tested (unknown what it is)

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**Why combined coverage is sufficient:**

The user's core complaint is:
> "if I do a `select * from mytable;` you will limit the dataset to 100 rows... However, if I want to export to a file, I want to export ALL the dataset"

Our tests directly address this:
1. ✅ Unit tests validate re-execution logic (backend)
2. ✅ Integration tests verify file has all rows, not 100 (end-to-end)
3. ✅ Manual test proves it works with large table (real scenario)
4. ✅ User validation confirms user's actual case works (final proof)

**If all these pass, export works correctly.**

The gaps are acceptable because:
- Gap 1 (very large): 1000 rows proves concept, 100K is just scale
- Gap 2 (other formats): CSV is most common, logic is shared
- Gap 3 (user's table): User validation covers this

---

### Feature 5: Simplify Export Command Syntax

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/sprints/sprint-13-planning.md` lines 156-185
- Secondary: `docs/builder/detailed-specifications/repl-mode.md` §5.8.4 (lines 2183-2295)

**Requirements:**
1. "Syntax simplified to `/export <format> [destination]`" (sprint-13-planning.md:174)
2. "`destination` can be filename or literal `clipboard`" (sprint-13-planning.md:175)
3. "Help text updated to show new syntax" (sprint-13-planning.md:176)
4. "Examples: `/export csv results.csv`, `/export json clipboard`" (sprint-13-planning.md:177)
5. "Backward compatibility maintained (old syntax still works with deprecation notice)" (sprint-13-planning.md:178)
6. "All export tests passing with new syntax" (sprint-13-planning.md:179)
7. "Documentation updated" (sprint-13-planning.md:180)

**Current Confusing Syntax (from spec):**
```
/export <format> [file]
/export <format> clipboard
/export clipboard [format]
/export <format> --append [file]
```

**Proposed Simplified Syntax:**
```
/export <format> [file|clipboard]
```

**Feature Characteristics:**

**User Interaction Type:** ✅ **Interactive PTY (Command Interface)**
- REPL metacommand syntax change
- User types command, observes help text
- Interactive feedback (error messages, deprecation warnings)

**Explanation:** This is a command syntax redesign. The user interacts by typing commands in REPL and reading help text. It's an interactive interface feature, not a backend data processing feature.

**Observable Behavior:**
- ✅ Visual output in terminal (help text, deprecation warnings, error messages)
- ❌ Structured data output (syntax change, not data export)
- ❌ File system side effects (same as before, just different syntax)
- ❌ Database side effects
- ❌ Performance characteristics

**External Dependencies:**
- ✅ Terminal/PTY (command parsing, help display)
- ❌ Database connection (syntax change doesn't affect backend)
- ❌ File system access (same export functionality)

**Validation Challenges:**
1. "Backward compatibility requires testing old syntax still works" - More test cases
2. "Deprecation warnings must be clear and helpful" - UX validation
3. "Help text must accurately reflect new syntax" - Documentation validation
4. "Error messages must guide users to new syntax" - User experience

**Critical Behaviors to Validate:**
1. "New syntax `/export csv file.csv` works" (sprint-13-planning.md:177)
2. "New syntax `/export json clipboard` works" (sprint-13-planning.md:177)
3. "Old syntax still works (backward compatibility)" (sprint-13-planning.md:178)
4. "Deprecation notice shown for old syntax" (sprint-13-planning.md:178)
5. "Help text shows new syntax" (sprint-13-planning.md:176)
6. "/help command displays updated syntax" (implied)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
✅ "Interactive PTY" characteristic:
  → Interactive tests OR manual tests REQUIRED
  Reason: Command syntax is typed by user in REPL
  GAP IF OMITTED: Syntax might not work in real REPL session

✅ "Visual output in terminal" (help text, warnings):
  → Manual validation RECOMMENDED
  Reason: Help text quality and clarity are subjective
  GAP IF OMITTED: Help text might be confusing or wrong

ALSO (command parsing):
  → Unit tests REQUIRED
  Reason: Command parsing logic needs validation
  GAP IF OMITTED: Parser bugs, syntax variations not handled
```

**Derived Test Types:**

**Test Type 1: Unit Tests (Command Parsing)**
- **Validates:** Command parser correctly handles new and old syntax
- **Approach:** Unit tests on export command parser
- **Rationale:** Parser logic needs comprehensive test coverage
- **Gap if missing:** Parser bugs, edge cases not handled
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests (Command Execution)**
- **Validates:** Commands actually work end-to-end
- **Approach:** Execute export commands with new and old syntax, verify behavior
- **Rationale:** Proves syntax changes work in real execution
- **Gap if missing:** Parser works but execution broken
- **Necessity:** ✅ REQUIRED

**Test Type 3: Interactive Tests (REPL Session)**
- **Validates:** Commands work in actual REPL interactive session
- **Approach:** expectrl tests typing commands, observing output
- **Rationale:** Validates REPL integration and user-visible messages
- **Gap if missing:** Works in tests but not real REPL
- **Necessity:** ⚠️ RECOMMENDED

**Test Type 4: Manual Validation (Help Text Quality)**
- **Validates:** Help text is clear, accurate, helpful
- **Approach:** Human reads help text, confirms it makes sense
- **Rationale:** Documentation quality is subjective
- **Gap if missing:** Help text might be confusing
- **Necessity:** ⚠️ RECOMMENDED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (parsing) | ✅ REQUIRED | Command parser validation | Parser bugs, syntax errors | MUST IMPLEMENT |
| Integration tests (execution) | ✅ REQUIRED | End-to-end command execution | Syntax works but execution fails | MUST IMPLEMENT |
| Interactive tests (REPL) | ⚠️ RECOMMENDED | Real REPL session validation | Command might not work in REPL | SHOULD IMPLEMENT |
| Manual validation (help text) | ⚠️ RECOMMENDED | Documentation quality | Help text confusing or wrong | SHOULD PERFORM |

**Summary:**
- ✅ REQUIRED test types: 2 - Unit tests, Integration tests
- ⚠️ RECOMMENDED test types: 2 - Interactive tests, Manual validation
- ❌ NOT NEEDED test types: 0

**Note:** User validation NOT required for this feature (syntax simplification is internal improvement, not user-reported bug).

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| SYNTAX-REQ-1 | "Syntax simplified to /export <format> [destination]" | sprint-13-planning.md:174 | Unit + Integration | Parser handles new syntax | Unit tests + INT-SYNTAX-1 |
| SYNTAX-REQ-2 | "destination can be filename or literal clipboard" | sprint-13-planning.md:175 | Unit + Integration | Parser distinguishes file vs clipboard | Unit tests + INT-SYNTAX-2 |
| SYNTAX-REQ-3 | "Help text updated to show new syntax" | sprint-13-planning.md:176 | Manual | Human validates help text content | Manual review |
| SYNTAX-REQ-4 | "Example: /export csv results.csv" | sprint-13-planning.md:177 | Integration | New syntax works | INT-SYNTAX-1 |
| SYNTAX-REQ-5 | "Example: /export json clipboard" | sprint-13-planning.md:177 | Integration | Clipboard destination works | INT-SYNTAX-2 |
| SYNTAX-REQ-6 | "Backward compatibility maintained" | sprint-13-planning.md:178 | Unit + Integration | Old syntax still works | Unit tests + INT-SYNTAX-3 |
| SYNTAX-REQ-7 | "Old syntax shows deprecation notice" | sprint-13-planning.md:178 | Integration + Interactive | Deprecation warning displayed | INT-SYNTAX-3 + Interactive |
| SYNTAX-REQ-8 | "All export tests passing with new syntax" | sprint-13-planning.md:179 | Integration | Existing export functionality preserved | All export tests |
| SYNTAX-REQ-9 | "Documentation updated" | sprint-13-planning.md:180 | Manual | Human validates documentation | Doc review |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements

**Coverage Gaps:**
- None identified

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**User Validation**
- **Reason for omission:** This is an internal syntax improvement, not a user-reported bug
- **What won't be validated:** Whether users prefer new syntax over old
- **Risk assessment:** LOW - New syntax is objectively simpler (fewer variations)
- **Mitigation:** Backward compatibility ensures no user disruption
- **Revisit criteria:** If users report confusion about new syntax

**Requirements with Partial Coverage:**

*None identified*

**Known Testing Limitations:**

1. **Deprecation Warning Clarity:**
   - Automated tests verify warning appears
   - Cannot fully validate warning is helpful to users
   - Limitation: Warning quality is subjective
   - Mitigation: Manual review of warning message

2. **Help Text Comprehensiveness:**
   - Tests verify help text exists and includes examples
   - Cannot validate help is clear to new users
   - Limitation: Documentation quality needs human judgment
   - Mitigation: Manual review of help text

#### 6. Test Implementation Plan

**Test Type: Unit Tests (Command Parsing)**
- **Location:** `src/commands/repl/export.rs` test module (or wherever export command is)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 10-15 tests
- **Key scenarios to cover:**
  1. New syntax: `/export csv file.csv` parses correctly
  2. New syntax: `/export json clipboard` parses correctly
  3. Old syntax: `/export csv [file]` still works (backward compat)
  4. Old syntax: `/export clipboard [format]` still works
  5. Old syntax triggers deprecation flag (not shown in unit test, but logic exists)
  6. Invalid syntax: `/export invalidformat file.csv` errors appropriately
  7. Missing arguments: `/export csv` errors with helpful message
  8. Edge cases: filenames with spaces, special characters
- **Implementation notes:** Comprehensive parser validation

**Test Type: Integration Tests (Command Execution)**
- **Location:** `tests/integration_tests.rs` or `tests/export_syntax_tests.rs`
- **Framework:** Built-in Rust integration test
- **Test count estimate:** 5-7 tests
- **Key scenarios to cover:**
  1. **INT-SYNTAX-1: New syntax with file**
     - Execute query
     - Run `/export csv results.csv`
     - Verify file created, contains data
     - Verify no deprecation warning

  2. **INT-SYNTAX-2: New syntax with clipboard**
     - Execute query
     - Run `/export json clipboard`
     - Verify success message (clipboard testing may be platform-dependent)

  3. **INT-SYNTAX-3: Old syntax with deprecation warning**
     - Execute query
     - Run `/export csv [file]` (old syntax)
     - Verify file created
     - **Verify deprecation warning displayed**

  4. **INT-SYNTAX-4: Old clipboard syntax**
     - Execute query
     - Run `/export clipboard csv` (old syntax)
     - Verify works but shows deprecation

  5. **INT-SYNTAX-5: Error handling**
     - Test invalid format, missing args
     - Verify helpful error messages

- **Implementation notes:**
  - Deprecation warning validation is key differentiator
  - May need to capture stdout/stderr separately

**Test Type: Interactive Tests (REPL Session) - RECOMMENDED**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl
- **Test count estimate:** 2-3 tests
- **Key scenarios to cover:**
  1. Type new syntax in REPL, verify works
  2. Type old syntax, verify deprecation warning visible in terminal
  3. Type `/help` or `/export --help`, verify help text updated
- **Implementation notes:** Validates user-visible behavior

**Test Type: Manual Validation (Help Text)**
- **Location:** Manual checklist in test results
- **Framework:** Human review
- **Test count estimate:** 1 documentation review
- **Checklist:**
  - [ ] `/help` shows export command
  - [ ] Help text shows new syntax: `/export <format> [destination]`
  - [ ] Examples included: `/export csv file.csv`, `/export json clipboard`
  - [ ] Deprecation of old syntax mentioned (if appropriate for help text)
  - [ ] Error messages guide users to correct syntax
  - [ ] Documentation files (repl-mode.md) updated
- **Implementation notes:** Quick manual review after implementation

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- **Unit tests validate:** Parser handles new and old syntax correctly
- **Integration tests validate:** Commands execute end-to-end, deprecation warnings shown
- **Interactive tests validate:** Real REPL session, user-visible messages
- **Manual validation ensures:** Help text is accurate and clear
- **Combined coverage:** **COMPREHENSIVE**

**Gaps in combined coverage:**
- User preference for new syntax not validated (assumed better due to simplicity)
- Long-term deprecation plan not tested (when to remove old syntax)

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**Why combined coverage is sufficient:**

Syntax simplification is a well-defined change:
- Old syntax: 4 variations (confusing)
- New syntax: 1 pattern (simple)
- Backward compatibility: Old syntax still works

Our tests cover:
1. ✅ Parser handles both syntaxes (unit tests)
2. ✅ Both syntaxes work end-to-end (integration tests)
3. ✅ Deprecation warnings shown (integration + interactive tests)
4. ✅ Help text updated (manual validation)

**This is complete coverage for a syntax change.**

Gaps are acceptable because:
- Gap 1 (user preference): New syntax is objectively simpler (fewer variations)
- Gap 2 (deprecation timeline): Backward compat means no rush to remove old syntax

---

### Feature 6: Build Warning Cleanup

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/sprints/sprint-13-planning.md` lines 188-201
- Secondary: Sprint 12 review (mentioned but not in our current documents)

**Requirements:**
1. "Fix unused Result warnings in `src/commands/repl/mod.rs` (lines 239-242)" (sprint-13-planning.md:193)
2. "Use proper error handling pattern: `let _ = writeln!(...)`" (sprint-13-planning.md:194)
3. "Zero build warnings after fix" (sprint-13-planning.md:195)
4. "Logo still displays correctly after changes" (sprint-13-planning.md:196)

**Feature Characteristics:**

**User Interaction Type:** ✅ **Pure Logic (Code Quality)**
- Not user-facing (internal code fix)
- No user-observable behavior change
- Build-time issue, not runtime issue

**Explanation:** Build warnings are compiler messages. Fixing them changes code quality but not functionality. Users don't see warnings; developers do.

**Observable Behavior:**
- ❌ No user-observable behavior (internal code fix)
- ✅ Build output changes (warnings disappear)

**External Dependencies:**
- ❌ None (pure code fix)

**Validation Challenges:**
1. "Must verify fix doesn't break logo display" - Regression testing
2. "Must verify build is clean (zero warnings)" - Simple check

**Critical Behaviors to Validate:**
1. "Build completes with zero warnings" (sprint-13-planning.md:195)
2. "Logo display unchanged (no regression)" (sprint-13-planning.md:196)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
✅ "Pure Logic" (code quality fix):
  → Unit tests REQUIRED (if logic changes)
  Reason: Ensure error handling pattern doesn't introduce bugs
  GAP IF OMITTED: Error handling might be broken

✅ "No user-observable behavior" (internal fix):
  → Regression tests REQUIRED
  Reason: Verify fix doesn't break existing functionality
  GAP IF OMITTED: Logo might break

BUILD VERIFICATION:
  → Compile-time test REQUIRED
  Reason: Verify warnings actually disappear
  GAP IF OMITTED: Might not fix all warnings
```

**Derived Test Types:**

**Test Type 1: Compilation Verification**
- **Validates:** Build completes with zero warnings
- **Approach:** `cargo build` or `cargo clippy`, check for warnings
- **Rationale:** Must verify warnings are actually fixed
- **Gap if missing:** Warnings might still exist
- **Necessity:** ✅ REQUIRED

**Test Type 2: Regression Tests (Logo Display)**
- **Validates:** Logo still displays correctly after code changes
- **Approach:** Run existing logo tests, visual verification
- **Rationale:** Ensure error handling change doesn't break logo
- **Gap if missing:** Logo might break silently
- **Necessity:** ✅ REQUIRED

**Test Type 3: Unit Tests (Error Handling Pattern)**
- **Validates:** New error handling pattern works correctly
- **Approach:** Unit tests for write operations (if applicable)
- **Rationale:** Verify `let _ = writeln!(...)` doesn't swallow errors
- **Gap if missing:** Error handling might be wrong
- **Necessity:** ⚠️ RECOMMENDED (only if pattern changes logic)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Compilation verification | ✅ REQUIRED | Verify warnings actually fixed | Warnings might still exist | MUST PERFORM |
| Regression tests (logo) | ✅ REQUIRED | No functionality broken | Logo display might break | MUST EXECUTE |
| Unit tests (error handling) | ⚠️ RECOMMENDED | Error pattern works correctly | Error handling bugs | IMPLEMENT IF NEEDED |

**Summary:**
- ✅ REQUIRED test types: 2 - Compilation verification, Regression tests
- ⚠️ RECOMMENDED test types: 1 - Unit tests (if applicable)
- ❌ NOT NEEDED test types: All others (user validation, manual tests, etc.)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| BUILD-REQ-1 | "Fix unused Result warnings in src/commands/repl/mod.rs" | sprint-13-planning.md:193 | Compilation | Verify warnings fixed | Cargo build check |
| BUILD-REQ-2 | "Use proper error handling pattern: let _ = writeln!(...)" | sprint-13-planning.md:194 | Unit (optional) | Verify pattern works | Unit test if logic changes |
| BUILD-REQ-3 | "Zero build warnings after fix" | sprint-13-planning.md:195 | Compilation | Verify clean build | Cargo build + clippy |
| BUILD-REQ-4 | "Logo still displays correctly after changes" | sprint-13-planning.md:196 | Regression | No broken functionality | Existing logo tests + visual |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements

**Coverage Gaps:**
- None identified

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**All User-Facing Test Types (Interactive, Manual, User Validation)**
- **Reason for omission:** This is an internal code quality fix, not user-facing feature
- **What won't be validated:** User experience (unchanged)
- **Risk assessment:** NONE - Users don't see build warnings
- **Mitigation:** Regression tests ensure no functionality broken
- **Revisit criteria:** Never (not a user-facing feature)

**Requirements with Partial Coverage:**

*None*

**Known Testing Limitations:**

1. **Platform-Specific Warnings:**
   - Warnings might only appear on certain platforms (macOS/Linux/Windows)
   - Tests run on single platform
   - Limitation: Might not catch all platform-specific warnings
   - Mitigation: CI runs on multiple platforms (if available)

#### 6. Test Implementation Plan

**Test Type: Compilation Verification**
- **Location:** Build process, CI/CD
- **Framework:** `cargo build`, `cargo clippy`
- **Test count estimate:** 1 build verification
- **Execution:**
  1. Run `cargo build --release` → verify zero warnings
  2. Run `cargo clippy -- -D warnings` → verify clippy clean
  3. Check exit code is 0 (success)
- **Implementation notes:** Simple build verification, no code to write

**Test Type: Regression Tests (Logo Display)**
- **Location:** Existing tests + visual check
- **Framework:** Existing test suite + manual verification
- **Test count estimate:** Rerun existing logo tests
- **Execution:**
  1. Run all existing unit tests: `cargo test`
  2. Run logo-specific tests (if separate)
  3. Start REPL, visually verify logo displays
  4. Compare to previous version (should be identical)
- **Implementation notes:** No new tests needed, just run existing tests

**Test Type: Unit Tests (Error Handling) - OPTIONAL**
- **Location:** `src/commands/repl/mod.rs` test module (if needed)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 0-2 tests (only if pattern changes logic)
- **Scenarios:**
  - Only if `let _ = writeln!(...)` pattern could hide bugs
  - Likely not needed (standard Rust pattern)
- **Implementation notes:** Probably not necessary for simple warning fix

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- **Compilation verification ensures:** Warnings actually fixed
- **Regression tests ensure:** Logo still works
- **Combined coverage:** **SUFFICIENT for code quality fix**

**Gaps in combined coverage:**
- Platform-specific warnings not tested (single platform)

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**Why combined coverage is sufficient:**

This is a **trivial code quality fix**. The requirements are simple:
1. ✅ Fix warnings (verified by clean build)
2. ✅ Don't break logo (verified by regression tests)

**If build is clean and logo works, fix is complete.**

No user validation needed (internal fix), no complex testing needed (trivial change).

---

## Strategy Summary

**Total Features Analyzed:** 6

**Test Types Required (Across All Features):**
- ✅ Unit tests: Features 1, 2, 4, 5, 6 (5 features)
- ✅ Interactive tests (expectrl): Features 1, 2 (2 features) - **NEW, CRITICAL**
- ✅ Integration tests: Features 1, 2, 3, 4, 5 (5 features)
- ✅ Manual validation: Features 2, 3, 4 (3 features)
- ✅ User validation: Features 2, 3, 4 (3 features) - **MANDATORY**
- ✅ Compilation verification: Feature 6 (1 feature)

**Estimated Test Count:**
- Unit tests: ~40-50 tests (mostly existing + new for features 4, 5)
- Interactive tests (expectrl): 5 required (Feature 1) + coverage for Feature 2 (same tests)
- Integration tests: ~15-20 tests (various features)
- Manual tests: 5-7 test cases (TC027, TC028, branding, export, syntax)
- Compilation checks: 1 build verification

**Total Estimated New Tests:** ~25-30 new automated tests + 5-7 manual test cases

**Risk Assessment:**

**HIGH Risk Gaps:**
- None identified with approved strategy

**MEDIUM Risk Gaps:**
- Cross-platform terminal differences (Features 2, 3) - Mitigation: Test on 2-3 terminals
- Automated color validation impossible (Feature 3) - Mitigation: Manual validation
- User's exact export scenario unknown (Feature 4) - Mitigation: User validation MANDATORY

**LOW Risk Gaps:**
- Performance not tested (Features 1, 2, 4) - No performance requirements in specs
- Very large datasets not tested (Feature 4) - 1000 rows proves concept
- Platform-specific build warnings (Feature 6) - CI should catch

**Dependencies Required:**
- ✅ Live database: YES (Features 1, 2, 4 require TQ_LOGON)
- ✅ Network access: NO (database is only network dependency)
- ⚠️ Specific OS: macOS for development, expectrl should be portable
- ✅ expectrl crate: YES (already available, needs implementation)

---

## Critical Insights from Failure Analysis

### Why Previous Sprints Failed

From `tab-completion-failure-analysis.md`:

**The Issue is NOT Code Bugs:**
> "The tab completion failure is a systematic testing and validation gap, not a code quality issue." (line 516)

**What We Were Testing:**
> "Unit tests verify logic, but interactive features need interactive tests." (lines 13-14)

**What We Should Have Been Testing:**
1. ✅ **Visual output** - What user actually sees in terminal
2. ✅ **Cursor position** - Where completion inserts text
3. ✅ **Context detection** - Multi-line buffer state in real PTY
4. ✅ **User validation** - User confirms bugs are actually fixed

**Unit Test Illusion (lines 155-176):**
> "Unit tests create a false sense of security for interactive features:
> 1. Mock Database
> 2. Mock Terminal
> 3. Mock Context
> 4. No Cursor Position
> 5. No Rendering"

### This Sprint's Strategy Changes

**What's Different This Time:**

1. **✅ Interactive Testing Framework (Feature 1) - BLOCKING**
   - Must be implemented FIRST
   - Cannot claim tab completion works without interactive tests
   - Framework enables validation of real user experience

2. **✅ Test Types Derived from Feature Characteristics**
   - Interactive features → Interactive tests REQUIRED
   - Visual features → Manual validation REQUIRED
   - Used decision tree, not guesswork

3. **✅ User Validation MANDATORY**
   - Features 2, 3, 4 cannot close without user sign-off
   - Agent approval is insufficient for user-reported bugs

4. **✅ Honest Gap Analysis**
   - Explicitly document what we cannot test
   - Assess risk honestly
   - No hand-waving about coverage

5. **✅ Test Strategy BEFORE Implementation**
   - This document created before writing any test code
   - tq-project-manager validates strategy
   - Implementation follows approved strategy

---

## Strategy Validation Checklist

**Before submitting to tq-project-manager for review:**

- [x] Every feature has complete specification analysis section
- [x] Feature characteristics are classified (not assumed)
- [x] Test strategy is derived from characteristics using decision tree (not guessed)
- [x] Every test type has clear rationale with "Gap if Omitted"
- [x] Gap analysis is complete and honest
- [x] Specification coverage map includes all requirements
- [x] Every requirement maps to at least one test type
- [x] Test implementation plan is detailed and actionable
- [x] Coverage sufficiency is assessed for each feature
- [x] No hand-waving or vague justifications
- [x] Interactive tests REQUIRED for interactive features (Features 1, 2)
- [x] Manual validation REQUIRED for visual features (Feature 3)
- [x] User validation REQUIRED for user-reported bugs (Features 2, 3, 4)

**All checkboxes checked - Strategy is ready for review.**

---

## Implementation Dependencies

**Blocking Dependencies:**

1. **Feature 1 (Interactive Testing Framework) BLOCKS Feature 2 (Tab Completion)**
   - Feature 2 requires interactive tests
   - Interactive tests require Feature 1 framework
   - **Implementation Order:** Feature 1 MUST be completed before Feature 2

**Parallel Implementation Possible:**

- Feature 3 (Branding) - Independent
- Feature 4 (Export) - Independent
- Feature 5 (Export Syntax) - Dependent on Feature 4 (same code)
- Feature 6 (Build Warnings) - Independent, trivial

**Recommended Implementation Order:**

1. **Phase 1:** Feature 1 (Interactive Testing Framework) - BLOCKING
2. **Phase 2 (Parallel):**
   - Feature 2 (Tab Completion) - Uses Feature 1
   - Feature 6 (Build Warnings) - Trivial, quick win
   - Feature 3 (Branding) - Visual, independent
3. **Phase 3:** Feature 4 + 5 (Export features) - Related, can be together

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-01-19
**Review Status:** DRAFT - Awaiting tq-project-manager validation

**Submitted for Review:** [To be filled when sent to tq-project-manager]

**Reviewer:** tq-project-manager
**Review Status:** [PENDING | APPROVED | REJECTED]
**Review Date:** [Date]
**Review Comments:** [tq-project-manager's feedback]

**Approval Requirements:**
- ✅ Test strategy derived from specifications (not assumptions)
- ✅ All required test types identified with clear rationale
- ✅ Coverage gaps explicitly identified and assessed
- ✅ Implementation plan is detailed and achievable
- ✅ Interactive tests REQUIRED for interactive features
- ✅ User validation REQUIRED for user-reported bugs

**Approval signature:** [tq-project-manager agent ID and timestamp when approved]

---

## Next Steps After Approval

1. **Phase 1: Implement Feature 1 (Interactive Testing Framework)**
   - Create 5 interactive tests in `tests/interactive_tests.rs`
   - Validate framework works with smoke tests
   - Document framework in testing-guidelines.md

2. **Phase 2: Implement All Other Tests**
   - Unit tests for Features 2, 4, 5
   - Integration tests for Features 2, 3, 4, 5
   - Regression tests for Feature 6
   - Manual test case execution for Features 2, 3, 4

3. **Phase 3: Execute Test Evidence Documentation**
   - Create `tests/strategy/sprint-13-test-evidence.md`
   - Document which test types were actually executed
   - Map requirements to test evidence
   - Perform honest gap assessment

4. **Phase 4: Generate REPORT.md**
   - Include "Test Type Coverage" section (NEW)
   - Document which test types were required vs implemented
   - Assess overall test strategy compliance
   - Provide recommendations

5. **Phase 5: User Validation**
   - Create user validation checklists for Features 2, 3, 4
   - Wait for user sign-off on each feature
   - Document user validation results

**BLOCKING CONDITION:** Sprint cannot close without:
- ✅ All REQUIRED test types implemented and executed
- ✅ Test evidence document completed
- ✅ User validation obtained for Features 2, 3, 4
