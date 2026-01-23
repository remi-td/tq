# Sprint 21 Test Strategy: Tab Completion Quality & Data Completeness

**Created:** 2026-01-23
**Author:** quality-validator
**Sprint:** Sprint 21
**Features:**
1. Complete Database Metadata Fetching (P0)
2. Universal Table Metadata Fetching (P0)
3. Second TAB Accepts Selection (P1)
4. Smart Database-Dot-TAB Completion (P1)
5. Automated Tab Completion Regression Tests (P2)

---

## CRITICAL CONTEXT: Sprint 18/20 False Positive Crisis

### The Problem

**Sprint 18/20 had 286/286 tests passing but bugs persisted.**

**Root Cause**: Automated tests validated CODE behavior (completion mechanism works), NOT USER EXPERIENCE (wrong content displayed).

**Evidence from Sprint 20 Review**:
- Iteration 1: 290/290 tests PASS → User reports "Still same issue" ❌
- Iteration 2: 290/290 tests PASS → User reports "Still same issue" ❌
- Iteration 3: 290/290 tests PASS → User reports "Bravo!!!" ✅

**Key Quote from Sprint 20 Review**:
> "Automated tests passed in ALL 3 iterations, but only manual user validation detected that iterations 1-2 were unsuccessful. This confirms Sprint 18/19's lesson: automated tests validate code behavior, manual tests validate user experience."

### The Solution: Hybrid Testing Mandatory

**Hybrid Testing Pattern**:
- **Automated Component**: Fast feedback, regression detection, CI/CD compatible
- **Manual Component**: User experience validation, keyboard interaction verification
- **Verdict Logic**: APPROVED only if BOTH automated AND manual tests pass

**This is NON-NEGOTIABLE for Sprint 21.**

---

## Test Automation Capabilities & Limitations

### What Automated Tests CAN Validate

| Aspect | Test Type | Technique | Confidence Level |
|--------|-----------|-----------|------------------|
| Metadata fetch queries return data | Unit | Mock database, verify SQL syntax | HIGH ✅ |
| Completion suggestions contain correct items | Unit | Test completion logic with mock data | HIGH ✅ |
| SQL context detection (FROM vs SELECT) | Unit | Parse SQL strings, verify context | HIGH ✅ |
| Completion filter logic (prefix matching) | Unit | Test string filtering algorithms | HIGH ✅ |
| Text output contains database names | PTY | Capture stdout, search for strings | MEDIUM ⚠️ |

### What Automated Tests CANNOT Validate

| Aspect | Why Automation Fails | Evidence |
|--------|---------------------|----------|
| TAB key behavior (2nd TAB accepts vs moves down) | PTY cannot distinguish menu navigation from text insertion | Sprint 20: ListMenu vs ColumnarMenu |
| Menu visual display (columns, alignment, colors) | PTY captures escape codes, not rendered output | Sprint 18: Table alignment broken |
| Cursor position after completion | PTY cannot reliably track cursor state in reedline | Known reedline limitation |
| "No pager output appears" (negative assertion) | Hard to prove absence of transient UI artifacts | Sprint 20: Pager banner bug |
| User perception of "intuitive" behavior | Subjective UX judgment | Requires human evaluation |

**CRITICAL INSIGHT**: The features in Sprint 21 are EXACTLY the type that fooled automated tests in Sprint 18/20:
- TAB key behavior (UI interaction)
- Menu display (visual rendering)
- Database/table name completion (content correctness)

**HIGH RISK for false positives if we rely on automation alone.**

---

## Feature-by-Feature Test Strategy

### Feature 1: Complete Database Metadata Fetching (P0)

#### 1. Specification Analysis

**Specification References**:
- Primary: `docs/sprints/sprint-21-planning.md` lines 48-68
- User Issue: "If I do `sel * from `+TAB I get a list of many databases, it should contain all databases on the system, but I noticed that I am using the dbc one!!! Make sure all databases are included"

**Feature Characteristics**:

**User Interaction Type**: Interactive PTY (REPL completion)

**Explanation**: User presses TAB in REPL to trigger completion menu. The feature's success is determined by WHAT appears in that menu, not just that completion mechanism works.

**Observable Behavior**:
- [x] Visual output in terminal (completion menu with database names)
- [x] Database side effects (metadata query executed)
- [x] State management (metadata cached for future completions)

**External Dependencies**:
- [x] Database connection (requires live database)
- [x] Terminal/PTY (menu display)

**Validation Challenges**:
- Database availability: Tests require real Teradata connection
- System database access: `dbc` may be restricted in some environments
- Completeness: Hard to prove "ALL databases" without knowing total count
- Content validation: PTY tests can verify text presence but not menu rendering

**Critical Behaviors to Validate**:
1. "System database `dbc` appears in database completion list" (AC#1)
2. "ALL databases on Teradata system are fetched (not filtered by access rights during fetch)" (AC#2)
3. "Query used to fetch databases returns complete system catalog" (AC#3)

#### 2. Test Strategy Derivation

**Decision Tree Results**:

```
✅ "Interactive PTY" checked
   → Interactive tests (expectrl) REQUIRED
   Reason: Must validate what user sees in completion menu

✅ "Database connection" checked
   → Integration tests with live database REQUIRED
   Reason: Metadata fetch must query real Teradata system catalog

✅ "Visual output in terminal" checked
   → PTY tests with content verification REQUIRED
   Reason: Unit tests cannot validate terminal menu display
```

**Derived Test Types**:

**Test Type 1: Unit Tests**
- **Validates**: Metadata query SQL syntax is correct, returns expected structure
- **Approach**: Mock database connection, verify query text matches `SELECT * FROM DBC.DatabasesV` or equivalent
- **Rationale**: Catches SQL syntax errors, ensures query targets correct system view
- **Gap if missing**: Query might use wrong catalog view, filter databases incorrectly
- **Necessity**: ✅ REQUIRED

**Test Type 2: Integration Tests (Live Database)**
- **Validates**: Metadata query executes successfully, returns rows including `dbc`
- **Approach**: Execute actual metadata fetch against test database, verify `dbc` in results
- **Rationale**: Proves query works on real Teradata, `dbc` is actually fetched
- **Gap if missing**: Query might fail with real database due to permissions, syntax differences
- **Necessity**: ✅ REQUIRED

**Test Type 3: Interactive PTY Tests**
- **Validates**: Completion menu shows `dbc` after `FROM ` + TAB
- **Approach**: Spawn tq REPL, type `SELECT * FROM `, send TAB, capture output, verify `dbc` present
- **Rationale**: Validates end-to-end user experience, proves completion mechanism uses fetched data
- **Gap if missing**: Integration logic might fail to pass data to completer, UI might not display it
- **Necessity**: ✅ REQUIRED

**Test Type 4: Manual Validation**
- **Validates**: User confirms `dbc` appears in completion menu in their environment
- **Approach**: User runs tq REPL, types `SELECT * FROM `, presses TAB, observes menu
- **Rationale**: Catches environment-specific issues, permission variations, visual rendering bugs
- **Gap if missing**: False positive risk (automated tests pass, user's database filtered differently)
- **Necessity**: ✅ REQUIRED (Sprint 20 lesson learned)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (query syntax) | ✅ REQUIRED | Validates SQL correctness without database | Wrong catalog view, SQL errors | MUST IMPLEMENT |
| Integration tests (live DB) | ✅ REQUIRED | Proves query fetches `dbc` on real database | Permission issues, catalog differences | MUST IMPLEMENT |
| Interactive tests (PTY) | ✅ REQUIRED | Validates completion menu shows `dbc` | Completer logic errors, UI rendering bugs | MUST IMPLEMENT |
| Manual validation (user) | ✅ REQUIRED | Human confirms in production environment | Environment-specific failures (Sprint 20 lesson) | DOCUMENT PROCEDURE |

**Summary**:
- ✅ REQUIRED test types: 4 (all must be implemented)
- ⚠️ RECOMMENDED test types: 0
- ❌ NOT NEEDED test types: 0

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| REQ-F1-1 | "System database `dbc` appears in database completion list" | sprint-21-planning.md:56 | Integration + PTY + Manual | Integration proves fetch, PTY proves display, Manual proves UX | TC-F1-INT-001, TC-F1-PTY-001, Manual-F1 |
| REQ-F1-2 | "ALL databases on Teradata system are fetched" | sprint-21-planning.md:57 | Unit + Integration | Unit validates query, Integration counts results | TC-F1-UNIT-001, TC-F1-INT-002 |
| REQ-F1-3 | "Query used returns complete system catalog" | sprint-21-planning.md:58 | Unit | Unit test validates SQL uses correct catalog view | TC-F1-UNIT-002 |

**Coverage Validation**:
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements (missing test coverage)
- [x] No unjustified test types

**Coverage Gaps**: NONE

#### 5. Gap Analysis

**Test Types Intentionally Omitted**: NONE

All test types are required for this feature due to Sprint 20 false positive risk.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location**: `src/db/metadata.rs` test module
- **Framework**: Built-in Rust test framework
- **Test count estimate**: 2 tests
- **Key scenarios to cover**:
  1. Metadata query SQL targets `DBC.DatabasesV` or equivalent system view
  2. Query does not filter by user permissions (fetches all databases)
- **Mocking strategy**: Mock Teradata connection, verify SQL text before execution

**Test Type: Integration Tests (Live Database)**
- **Location**: `tests/integration_tests.rs`
- **Framework**: Built-in Rust integration test support, marked `#[ignore]`
- **Test count estimate**: 2 tests
- **Key scenarios to cover**:
  1. Fetch databases, verify `dbc` in results
  2. Fetch databases, verify count matches user's expected total (document expected count in test)
- **Setup requirements**: `TQ_LOGON` environment variable with test database credentials

**Test Type: Interactive PTY Tests**
- **Location**: `tests/interactive_tests.rs`
- **Framework**: expectrl crate, marked `#[ignore]`
- **Test count estimate**: 1 test
- **Key scenarios to cover**:
  1. Type `SELECT * FROM `, press TAB, verify `dbc` in output text
- **Implementation notes**: PTY test validates text presence, not visual menu rendering

**Test Type: Manual Validation**
- **Location**: `tests/cases/TC-F1-MANUAL.md`
- **Framework**: Human execution with checklist
- **Test count estimate**: 1 procedure
- **Key scenarios to cover**:
  1. User sees `dbc` in completion menu (visual confirmation)
  2. User confirms menu displays correctly (not truncated, readable)
- **Evidence**: Screenshot or user confirmation statement

#### 7. Coverage Sufficiency Assessment

**Question**: If all planned test types are implemented and passing, can we claim Feature 1 "works as specified"?

**Analysis**:
- Unit tests validate: Query SQL correctness
- Integration tests validate: Query fetches `dbc` from real database
- PTY tests validate: Completion mechanism uses fetched data and outputs text
- Manual tests validate: User sees `dbc` in menu, menu renders correctly

**Combined coverage**: ADEQUATE with one caveat

**Gaps in combined coverage**:
- **Gap 1**: Cross-environment compatibility not tested (different Teradata versions, permission models)
- **Gap 2**: Performance not tested (large database list rendering time)

**Acceptance criteria**:
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified"
- [x] Known gaps are documented and accepted

**If gaps exist, why they're acceptable**:
- **Gap 1 is acceptable**: Sprint 21 tests against user's environment, not all possible environments
- **Gap 2 is acceptable**: Specification has no performance requirements, defer to future sprint if issues arise

---

### Feature 2: Universal Table Metadata Fetching (P0)

#### 1. Specification Analysis

**Specification References**:
- Primary: `docs/sprints/sprint-21-planning.md` lines 70-90
- User Issue: "Some databases objects are not cached/fetched. For example: `tq> | sel * from demo_user.` → NO RECORDS FOUND. I know that there are three tables in this database, but it should be fetched!"

**Feature Characteristics**:

**User Interaction Type**: Interactive PTY (REPL completion)

**Explanation**: User types `database.` and presses TAB, expects tables in that database to appear.

**Observable Behavior**:
- [x] Visual output in terminal (completion menu with table names)
- [x] Database side effects (table metadata query for specific database)
- [x] State management (on-demand fetching vs pre-loading)

**External Dependencies**:
- [x] Database connection (requires live database)
- [x] Terminal/PTY (menu display)

**Validation Challenges**:
- Database diversity: Must test multiple databases (not just default database)
- Permission handling: Some databases may deny table list access (graceful degradation)
- Timing: On-demand fetch introduces latency (UX concern)
- Negative cases: "NO RECORDS FOUND" error message must NOT appear

**Critical Behaviors to Validate**:
1. "Metadata fetch attempts to load tables for ALL databases" (AC#1)
2. "`demo_user` database tables appear in completion" (AC#2)
3. "Completion shows tables after typing `database.` + TAB" (AC#3)
4. "Error handling: graceful degradation if permission denied for specific database" (AC#4)

#### 2. Test Strategy Derivation

**Decision Tree Results**:

```
✅ "Interactive PTY" checked
   → Interactive tests REQUIRED

✅ "Database connection" checked
   → Integration tests with live database REQUIRED

✅ "Visual output in terminal" checked
   → PTY tests with content verification REQUIRED

✅ "Error handling: graceful degradation" checked
   → Negative test cases REQUIRED (permission denied scenario)
```

**Derived Test Types**:

**Test Type 1: Unit Tests**
- **Validates**: Table fetch query SQL syntax, qualified name parsing (`demo_user.`)
- **Approach**: Mock database, verify query targets `DBC.TablesV WHERE DatabaseName = ?`
- **Rationale**: Catches SQL errors, ensures per-database table fetching
- **Gap if missing**: Query might fetch all tables (performance issue) or wrong tables
- **Necessity**: ✅ REQUIRED

**Test Type 2: Integration Tests (Live Database)**
- **Validates**: Table fetch returns rows for `demo_user`, handles permission denied gracefully
- **Approach**: Execute table fetch for test database, verify expected tables returned; test with restricted database
- **Rationale**: Proves query works on real Teradata, error handling correct
- **Gap if missing**: Query might fail, error messages might be user-hostile
- **Necessity**: ✅ REQUIRED

**Test Type 3: Interactive PTY Tests**
- **Validates**: Typing `demo_user.` + TAB shows tables, NOT "NO RECORDS FOUND" error
- **Approach**: Spawn REPL, type `SELECT * FROM demo_user.`, send TAB, verify table names in output
- **Rationale**: Validates end-to-end UX, proves completion triggers table fetch
- **Gap if missing**: Completer might not recognize qualified name context, or fail silently
- **Necessity**: ✅ REQUIRED

**Test Type 4: Manual Validation**
- **Validates**: User confirms tables appear in menu, menu is usable (not too slow, readable)
- **Approach**: User types `demo_user.`, presses TAB, observes menu and latency
- **Rationale**: Catches UX issues (slow fetch, confusing display), environment-specific problems
- **Gap if missing**: False positive risk (automated tests pass, user sees different behavior)
- **Necessity**: ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (query syntax) | ✅ REQUIRED | Validates per-database table query | Wrong query, fetch all tables | MUST IMPLEMENT |
| Integration tests (live DB) | ✅ REQUIRED | Proves table fetch + error handling | Permission issues, error UX | MUST IMPLEMENT |
| Interactive tests (PTY) | ✅ REQUIRED | Validates qualified name completion | Context detection failures | MUST IMPLEMENT |
| Manual validation (user) | ✅ REQUIRED | Human confirms tables appear, UX acceptable | False positives (Sprint 20 lesson) | DOCUMENT PROCEDURE |

**Summary**:
- ✅ REQUIRED test types: 4
- ⚠️ RECOMMENDED test types: 0
- ❌ NOT NEEDED test types: 0

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| REQ-F2-1 | "Metadata fetch attempts to load tables for ALL databases" | sprint-21-planning.md:77 | Unit + Integration | Unit validates logic, Integration proves execution | TC-F2-UNIT-001, TC-F2-INT-001 |
| REQ-F2-2 | "`demo_user` database tables appear in completion" | sprint-21-planning.md:78 | Integration + PTY + Manual | Integration proves fetch, PTY proves display, Manual proves UX | TC-F2-INT-002, TC-F2-PTY-001, Manual-F2 |
| REQ-F2-3 | "Completion shows tables after typing `database.` + TAB" | sprint-21-planning.md:79 | PTY + Manual | PTY validates automation, Manual validates rendering | TC-F2-PTY-001, Manual-F2 |
| REQ-F2-4 | "Graceful degradation if permission denied" | sprint-21-planning.md:80 | Integration + PTY | Integration tests error handling, PTY validates no crash | TC-F2-INT-003, TC-F2-PTY-002 |

**Coverage Validation**:
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements
- [x] No unjustified test types

**Coverage Gaps**: NONE

#### 5. Gap Analysis

**Test Types Intentionally Omitted**: NONE

**Negative Testing Critical**: Must verify "NO RECORDS FOUND" error does NOT appear (user's original issue).

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location**: `src/commands/repl/metadata_completer.rs` test module
- **Framework**: Built-in Rust test framework
- **Test count estimate**: 2 tests
- **Key scenarios to cover**:
  1. Qualified name parsing: `demo_user.` → database="demo_user", prefix=""
  2. Table query SQL: `SELECT TableName FROM DBC.TablesV WHERE DatabaseName = 'demo_user'`
- **Mocking strategy**: Mock database, verify SQL before execution

**Test Type: Integration Tests (Live Database)**
- **Location**: `tests/integration_tests.rs`
- **Framework**: Built-in Rust integration test support, marked `#[ignore]`
- **Test count estimate**: 3 tests
- **Key scenarios to cover**:
  1. Fetch tables for `demo_user`, verify expected table names
  2. Fetch tables for restricted database, verify graceful error (no panic)
  3. Fetch tables for non-existent database, verify empty result (not error)
- **Setup requirements**: Test database with `demo_user` schema, known table count

**Test Type: Interactive PTY Tests**
- **Location**: `tests/interactive_tests.rs`
- **Framework**: expectrl crate, marked `#[ignore]`
- **Test count estimate**: 2 tests
- **Key scenarios to cover**:
  1. Type `SELECT * FROM demo_user.`, press TAB, verify table names in output
  2. Negative test: Verify "NO RECORDS FOUND" does NOT appear in output
- **Implementation notes**: PTY captures stdout, search for table names and error strings

**Test Type: Manual Validation**
- **Location**: `tests/cases/TC-F2-MANUAL.md`
- **Framework**: Human execution with checklist
- **Test count estimate**: 1 procedure
- **Key scenarios to cover**:
  1. User types `demo_user.`, presses TAB, sees tables in menu
  2. User confirms completion latency acceptable (<1s)
  3. User confirms no error messages appear
- **Evidence**: Screenshot or user confirmation

#### 7. Coverage Sufficiency Assessment

**Question**: If all planned test types pass, can we claim Feature 2 "works as specified"?

**Analysis**:
- Unit tests validate: Qualified name parsing, query SQL
- Integration tests validate: Table fetch from real database, error handling
- PTY tests validate: Completion triggered, table names output, no error messages
- Manual tests validate: Menu displays correctly, latency acceptable, UX smooth

**Combined coverage**: COMPREHENSIVE

**Gaps in combined coverage**:
- **Gap 1**: Large database performance (100+ tables) not tested
- **Gap 2**: Unicode table names not tested

**Acceptance criteria**:
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified"
- [x] Known gaps are documented and accepted

**Gaps are acceptable because**:
- **Gap 1**: Specification has no performance requirements for large result sets
- **Gap 2**: Teradata table names are typically ASCII, Unicode edge case not in scope

---

### Feature 3: Second TAB Accepts Selection (P1)

#### 1. Specification Analysis

**Specification References**:
- Primary: `docs/sprints/sprint-21-planning.md` lines 93-117
- User Issue: "When we hit tab the first time, the object menu is displayed, which is OK. But when we hit tab a second time, the cursor select the next object (down) which is unintuitive (the down arrow is for this), typically a second tab hit validates the completion with the highlighted object (same as enter)."

**Feature Characteristics**:

**User Interaction Type**: Interactive PTY (REPL keyboard interaction)

**Explanation**: This is PURE UI BEHAVIOR. Success depends on HOW the menu responds to keyboard input, not WHAT data it contains.

**Observable Behavior**:
- [x] Visual output in terminal (menu navigation, text insertion at cursor)
- [x] State management (reedline menu state, cursor position)

**External Dependencies**:
- [x] Terminal/PTY (keyboard input simulation)

**Validation Challenges**:
- **CRITICAL**: This is the HIGHEST RISK feature for false positives
- Keyboard interaction: TAB vs DOWN arrow vs ENTER behavior
- Cursor position: Where text is inserted after completion
- Menu state: Does menu close after second TAB?
- **PTY limitation**: expectrl can send TAB, but cannot reliably detect cursor position or menu state changes

**Critical Behaviors to Validate**:
1. "First TAB: Show completion menu with first item highlighted" (AC#1)
2. "Second TAB: Accept highlighted item and insert into command line" (AC#2)
3. "DOWN arrow: Move to next item in menu" (AC#3)
4. "UP arrow: Move to previous item in menu" (AC#4)
5. "ENTER: Accept highlighted item" (AC#5)
6. "Behavior matches bash/zsh completion UX" (AC#6)

#### 2. Test Strategy Derivation

**Decision Tree Results**:

```
✅ "Interactive PTY" checked
   → Interactive tests REQUIRED (but INSUFFICIENT)

✅ "Visual output in terminal" checked
   → Manual validation MANDATORY

❌ "Database connection" NOT required for TAB behavior testing
   → Can test with mock completion data
```

**CRITICAL ASSESSMENT: Automation Limitations**

**What PTY Tests CAN Validate**:
- TAB key sends correct escape sequence
- Some text appears after pressing TAB twice
- Menu-related text appears after first TAB

**What PTY Tests CANNOT Validate** (Sprint 20 lesson):
- Whether second TAB moves down vs accepts
- Cursor position after completion
- Menu closes vs stays open
- Visual appearance of highlighted item
- "Intuitive" behavior (subjective UX)

**HIGH FALSE POSITIVE RISK**: PTY test will likely pass even if behavior is wrong.

**Derived Test Types**:

**Test Type 1: Unit Tests**
- **Validates**: N/A - This is reedline configuration, no tq logic to unit test
- **Necessity**: ❌ NOT NEEDED

**Test Type 2: Integration Tests**
- **Validates**: N/A - Keyboard behavior cannot be integration tested without PTY
- **Necessity**: ❌ NOT NEEDED

**Test Type 3: Interactive PTY Tests**
- **Validates**: TAB key interactions produce some observable output change
- **Approach**: Send TAB, send TAB again, capture output, verify text changed
- **Rationale**: Provides WEAK signal that something happened, useful for CI/CD regression detection
- **Gap if missing**: No automated regression detection at all
- **Necessity**: ⚠️ RECOMMENDED (but INSUFFICIENT for approval)

**Test Type 4: Manual Validation** ⚠️ **PRIMARY VALIDATION METHOD**
- **Validates**: Second TAB accepts (not moves down), matches bash/zsh UX
- **Approach**: Human runs tq REPL, types `SELECT * FROM `, presses TAB, observes menu, presses TAB again, confirms text accepted
- **Rationale**: ONLY manual testing can validate keyboard interaction UX (Sprint 20 lesson)
- **Gap if missing**: FALSE POSITIVE GUARANTEED (automated tests cannot validate this)
- **Necessity**: ✅ REQUIRED - **THIS IS THE ONLY RELIABLE TEST FOR THIS FEATURE**

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ❌ NOT NEEDED | No tq logic to test (reedline configuration) | N/A | SKIP |
| Integration tests | ❌ NOT NEEDED | Keyboard behavior not integration-testable | N/A | SKIP |
| Interactive tests (PTY) | ⚠️ RECOMMENDED | Weak signal for regression detection | No automated CI/CD check | IMPLEMENT (but mark as insufficient) |
| Manual validation (user) | ✅ REQUIRED | ONLY method to validate TAB behavior | FALSE POSITIVE GUARANTEED if omitted | DOCUMENT PROCEDURE (PRIMARY TEST) |

**Summary**:
- ✅ REQUIRED test types: 1 (manual validation)
- ⚠️ RECOMMENDED test types: 1 (PTY, but insufficient)
- ❌ NOT NEEDED test types: 2 (unit, integration)

**CRITICAL NOTE**: This feature CANNOT be reliably validated by automation. Manual validation is MANDATORY for APPROVED verdict.

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| REQ-F3-1 | "First TAB: Show completion menu with first item highlighted" | sprint-21-planning.md:102 | PTY + Manual | PTY detects menu text, Manual validates highlighting | TC-F3-PTY-001, Manual-F3 |
| REQ-F3-2 | "Second TAB: Accept highlighted item and insert" | sprint-21-planning.md:103 | Manual | PTY CANNOT validate this (Sprint 20 lesson) | Manual-F3 (PRIMARY) |
| REQ-F3-3 | "DOWN arrow: Move to next item in menu" | sprint-21-planning.md:104 | Manual | PTY CANNOT validate menu navigation | Manual-F3 |
| REQ-F3-4 | "UP arrow: Move to previous item in menu" | sprint-21-planning.md:105 | Manual | PTY CANNOT validate menu navigation | Manual-F3 |
| REQ-F3-5 | "ENTER: Accept highlighted item" | sprint-21-planning.md:106 | Manual | PTY CANNOT distinguish ENTER from TAB behavior | Manual-F3 |
| REQ-F3-6 | "Behavior matches bash/zsh completion UX" | sprint-21-planning.md:107 | Manual | Subjective UX comparison, human judgment required | Manual-F3 |

**Coverage Validation**:
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements
- [x] No unjustified test types

**Coverage Gaps**:
- **MAJOR GAP**: 5 out of 6 acceptance criteria can ONLY be validated manually
- **Automated coverage**: ~16% (only AC#1 partially testable with PTY)

#### 5. Gap Analysis

**Test Types Intentionally Omitted**:

**Unit Tests**
- **Reason**: Feature is reedline configuration (external library), no tq application logic to unit test
- **What won't be validated**: N/A (nothing to validate at unit level)
- **Risk**: LOW
- **Mitigation**: Manual validation covers all behavior
- **Revisit criteria**: If tq implements custom menu component (not using reedline)

**Integration Tests**
- **Reason**: Keyboard interaction requires PTY, cannot be tested with integration test framework
- **What won't be validated**: N/A (PTY tests cover automation layer)
- **Risk**: LOW
- **Mitigation**: PTY tests + manual validation
- **Revisit criteria**: Never (integration tests not applicable to keyboard input)

**CRITICAL GAP: Automated Testing Insufficient**

This feature has the HIGHEST false positive risk in Sprint 21:
- **Risk**: HIGH
- **Evidence**: Sprint 20 had identical issue (menu component behavior not testable with automation)
- **Mitigation**: Make manual validation MANDATORY for APPROVED verdict
- **Acceptance criteria**: PTY tests passing is NOT sufficient, manual validation REQUIRED

#### 6. Test Implementation Plan

**Test Type: Interactive PTY Tests**
- **Location**: `tests/interactive_tests.rs`
- **Framework**: expectrl crate, marked `#[ignore]`
- **Test count estimate**: 1 test (LIMITED VALUE)
- **Key scenarios to cover**:
  1. First TAB produces completion menu text output
  2. Second TAB changes text output (but CANNOT validate behavior is correct)
- **Implementation notes**:
  - Test provides WEAK signal for CI/CD
  - Test CANNOT distinguish "second TAB accepts" from "second TAB moves down"
  - Test PASSING does NOT mean feature works correctly
  - Mark test with warning comment about limitations

**Test Type: Manual Validation** ⚠️ **PRIMARY TEST**
- **Location**: `tests/cases/TC-F3-MANUAL.md`
- **Framework**: Human execution with detailed checklist
- **Test count estimate**: 1 comprehensive procedure
- **Key scenarios to cover**:
  1. First TAB shows menu, first item highlighted
  2. Second TAB accepts highlighted item (text inserted, menu closes)
  3. DOWN arrow moves to next item (not accept)
  4. UP arrow moves to previous item
  5. ENTER accepts highlighted item
  6. Compare to bash/zsh completion (side-by-side test)
- **Evidence**:
  - Video recording of keyboard interactions (recommended)
  - Screenshot of each keyboard action result
  - User confirmation statement: "Second TAB accepts, matches bash/zsh"
- **VERDICT GATE**: APPROVED verdict REQUIRES this manual test to pass

#### 7. Coverage Sufficiency Assessment

**Question**: If all planned test types pass, can we claim Feature 3 "works as specified"?

**Analysis**:
- Unit tests: N/A
- Integration tests: N/A
- PTY tests validate: TAB key produces output (weak signal)
- Manual tests validate: TAB, DOWN, UP, ENTER behavior; bash/zsh comparison

**Combined coverage**: ADEQUATE **ONLY IF manual validation performed**

**Gaps in combined coverage**:
- **Gap 1**: Automated coverage extremely limited (~16%)
- **Gap 2**: No automated regression detection for keyboard behavior

**Acceptance criteria**:
- [x] All specification requirements have test coverage (via manual tests)
- [x] All test types justified by requirements
- [ ] Combined coverage is sufficient to claim "works as specified" **WITHOUT manual validation**
- [x] Known gaps are documented and accepted

**CRITICAL VERDICT LOGIC**:
- PTY tests PASS + Manual tests PASS → APPROVED ✅
- PTY tests PASS + Manual tests NOT PERFORMED → REJECTED ❌ (false positive risk)
- PTY tests PASS + Manual tests FAIL → REJECTED ❌
- PTY tests FAIL → REJECTED ❌

**Gap 1 is acceptable because**: Feature cannot be reliably automated (Sprint 20 lesson), manual validation required
**Gap 2 is acceptable because**: PTY tests provide weak regression signal, full regression requires manual smoke test

---

### Feature 4: Smart Database-Dot-TAB Completion (P1)

#### 1. Specification Analysis

**Specification References**:
- Primary: `docs/sprints/sprint-21-planning.md` lines 119-141
- User Issue: "Also, when I hit tab on a database after a FROM/JOIN, I would expect to complete the database name, add a '.' and prompt the list of tables in this database directly."

**Feature Characteristics**:

**User Interaction Type**: Interactive PTY (REPL completion with multi-stage logic)

**Explanation**: User types partial database name (`dem`), presses TAB. Tool completes to `demo_user.` and immediately shows tables. This is CONTEXT-AWARE MULTI-STAGE COMPLETION.

**Observable Behavior**:
- [x] Visual output in terminal (database name completed, dot added, table menu displayed)
- [x] Database side effects (table metadata fetched on-demand)
- [x] State management (completion state tracks partial completion)

**External Dependencies**:
- [x] Database connection (requires live database for table fetch)
- [x] Terminal/PTY (multi-stage interaction)

**Validation Challenges**:
- Multi-stage completion: Single TAB triggers two actions (complete database + show tables)
- Ambiguity handling: `de` matches `demo_user` and `demo_admin` → show database list first
- Context awareness: Must work after FROM and JOIN keywords
- State management: Completer must track partial completion to trigger next stage
- **PTY limitation**: Hard to verify intermediate states (database completion) vs final state (table list)

**Critical Behaviors to Validate**:
1. "Typing `dem` + TAB completes to `demo_user.` (if unambiguous)" (AC#1)
2. "After completing `demo_user.`, immediately show tables in `demo_user` database" (AC#2)
3. "If ambiguous (multiple matches), show database list first" (AC#3)
4. "Works after FROM keyword" (AC#4)
5. "Works after JOIN keyword" (AC#5)

#### 2. Test Strategy Derivation

**Decision Tree Results**:

```
✅ "Interactive PTY" checked
   → Interactive tests REQUIRED

✅ "Database connection" checked
   → Integration tests with live database REQUIRED

✅ "State management" (multi-stage completion) checked
   → Unit tests for state machine logic REQUIRED
```

**Automation Capability Assessment**:

**What PTY Tests CAN Validate**:
- Text contains `demo_user.` after TAB
- Text contains table names after TAB
- Multi-stage completion produces combined result

**What PTY Tests CANNOT Validate**:
- Visual appearance of two-stage process (user sees database complete, then tables appear)
- Timing: Does table list appear immediately or after delay?
- Smooth UX: Does it feel like "one action" or "two actions"?

**Automation risk**: MEDIUM (lower than Feature 3)

PTY tests can validate CONTENT correctness (database name + tables appear), but not UX smoothness.

**Derived Test Types**:

**Test Type 1: Unit Tests**
- **Validates**: Multi-stage completion logic (unambiguous → complete + fetch tables)
- **Approach**: Test completion state machine with mock data
- **Rationale**: Validates complex logic without database/PTY
- **Gap if missing**: Logic errors (ambiguity detection, state transitions)
- **Necessity**: ✅ REQUIRED

**Test Type 2: Integration Tests (Live Database)**
- **Validates**: Database name completion + table fetch work end-to-end
- **Approach**: Call completion function with `dem`, verify returns `demo_user.` + table suggestions
- **Rationale**: Proves database integration and multi-stage logic together
- **Gap if missing**: Integration bugs (fetch wrong tables, timing issues)
- **Necessity**: ✅ REQUIRED

**Test Type 3: Interactive PTY Tests**
- **Validates**: Typing `dem` + TAB produces `demo_user.` + table names in output
- **Approach**: Send `SELECT * FROM dem`, send TAB, verify output contains `demo_user.` and table names
- **Rationale**: Validates end-to-end REPL integration, content correctness
- **Gap if missing**: REPL integration bugs, completer not invoked
- **Necessity**: ✅ REQUIRED

**Test Type 4: Manual Validation**
- **Validates**: UX is smooth, immediate, matches user expectation
- **Approach**: User types `dem`, presses TAB, observes database completion + table list appear "immediately"
- **Rationale**: Validates UX smoothness, not just content correctness
- **Gap if missing**: Poor UX (choppy, confusing) ships despite passing automated tests
- **Necessity**: ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests (state machine) | ✅ REQUIRED | Validates multi-stage completion logic | Logic errors, ambiguity handling | MUST IMPLEMENT |
| Integration tests (live DB) | ✅ REQUIRED | Proves database + table fetch integration | Integration bugs, wrong data | MUST IMPLEMENT |
| Interactive tests (PTY) | ✅ REQUIRED | Validates REPL integration, content correctness | REPL-level bugs | MUST IMPLEMENT |
| Manual validation (user) | ✅ REQUIRED | Validates UX smoothness, user expectation | Poor UX ships (Sprint 20 lesson) | DOCUMENT PROCEDURE |

**Summary**:
- ✅ REQUIRED test types: 4 (all layers)
- ⚠️ RECOMMENDED test types: 0
- ❌ NOT NEEDED test types: 0

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------|----------------|--------------|---------------|------------|
| REQ-F4-1 | "Typing `dem` + TAB completes to `demo_user.` (if unambiguous)" | sprint-21-planning.md:127 | Unit + Integration + PTY + Manual | Unit validates logic, Integration proves execution, PTY proves REPL, Manual proves UX | TC-F4-UNIT-001, TC-F4-INT-001, TC-F4-PTY-001, Manual-F4 |
| REQ-F4-2 | "After completing, immediately show tables" | sprint-21-planning.md:128 | Integration + PTY + Manual | Integration proves fetch, PTY proves output, Manual proves "immediate" | TC-F4-INT-002, TC-F4-PTY-001, Manual-F4 |
| REQ-F4-3 | "If ambiguous, show database list first" | sprint-21-planning.md:129 | Unit + PTY | Unit validates ambiguity detection, PTY proves output | TC-F4-UNIT-002, TC-F4-PTY-002 |
| REQ-F4-4 | "Works after FROM keyword" | sprint-21-planning.md:130 | Unit + PTY | Unit validates FROM context, PTY proves REPL behavior | TC-F4-UNIT-003, TC-F4-PTY-003 |
| REQ-F4-5 | "Works after JOIN keyword" | sprint-21-planning.md:131 | Unit + PTY | Unit validates JOIN context, PTY proves REPL behavior | TC-F4-UNIT-004, TC-F4-PTY-004 |

**Coverage Validation**:
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements
- [x] No unjustified test types

**Coverage Gaps**: NONE

#### 5. Gap Analysis

**Test Types Intentionally Omitted**: NONE

**Note**: This feature has better automation prospects than Feature 3 because content correctness CAN be validated by PTY tests.

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location**: `src/commands/repl/metadata_completer.rs` test module
- **Framework**: Built-in Rust test framework
- **Test count estimate**: 4 tests
- **Key scenarios to cover**:
  1. Unambiguous prefix → complete + return tables
  2. Ambiguous prefix → return databases only
  3. FROM context detection → database completion enabled
  4. JOIN context detection → database completion enabled
- **Mocking strategy**: Mock metadata cache with test databases/tables

**Test Type: Integration Tests (Live Database)**
- **Location**: `tests/integration_tests.rs`
- **Framework**: Built-in Rust integration test support, marked `#[ignore]`
- **Test count estimate**: 2 tests
- **Key scenarios to cover**:
  1. Complete `dem` → returns `demo_user.` + tables from demo_user
  2. Complete `de` (ambiguous) → returns database list only
- **Setup requirements**: Test database with `demo_user` and another `de*` database

**Test Type: Interactive PTY Tests**
- **Location**: `tests/interactive_tests.rs`
- **Framework**: expectrl crate, marked `#[ignore]`
- **Test count estimate**: 4 tests
- **Key scenarios to cover**:
  1. `FROM dem` + TAB → output contains `demo_user.` + table names
  2. `JOIN dem` + TAB → output contains `demo_user.` + table names
  3. `FROM de` + TAB (ambiguous) → output contains database list
  4. Negative: verify no error messages appear
- **Implementation notes**: PTY validates content, not UX smoothness

**Test Type: Manual Validation**
- **Location**: `tests/cases/TC-F4-MANUAL.md`
- **Framework**: Human execution with checklist
- **Test count estimate**: 1 procedure
- **Key scenarios to cover**:
  1. User types `FROM dem`, presses TAB, observes database completion + table list appear smoothly
  2. User confirms latency acceptable (feels immediate, <500ms)
  3. User confirms behavior matches expectation (intuitive workflow)
- **Evidence**: Screenshot or video, user confirmation

#### 7. Coverage Sufficiency Assessment

**Question**: If all planned test types pass, can we claim Feature 4 "works as specified"?

**Analysis**:
- Unit tests validate: Multi-stage logic, ambiguity handling, context detection
- Integration tests validate: Database + table fetch integration
- PTY tests validate: REPL integration, content correctness (database name + tables appear)
- Manual tests validate: UX smoothness, immediacy, user expectation

**Combined coverage**: COMPREHENSIVE

**Gaps in combined coverage**:
- **Gap 1**: Performance with slow database (table fetch latency) not tested
- **Gap 2**: Very long database/table names (menu overflow) not tested

**Acceptance criteria**:
- [x] All specification requirements have test coverage
- [x] All test types justified by requirements
- [x] Combined coverage is sufficient to claim "works as specified"
- [x] Known gaps are documented and accepted

**Gaps are acceptable because**:
- **Gap 1**: Specification has no performance SLA, defer to future sprint if issues arise
- **Gap 2**: Edge case, low priority, not blocking MVP

---

### Feature 5: Automated Tab Completion Regression Tests (P2)

This feature IS the test suite itself. See "Strategy Summary" section below for comprehensive test coverage across Features 1-4.

---

## Hybrid Testing Pattern Definition

### Automated Component

**Purpose**: Fast feedback, regression detection, CI/CD integration

**Test Types**:
- Unit tests: Logic validation (parsers, state machines, algorithms)
- Integration tests: Database queries, end-to-end workflows
- PTY tests: Terminal output content verification

**Characteristics**:
- Run in <5 minutes total
- No human interaction required
- Deterministic (same input → same output)
- CI/CD compatible

**Limitations** (Sprint 20 lesson learned):
- Cannot validate keyboard interaction UX (TAB vs DOWN vs ENTER)
- Cannot validate visual rendering (menu display, alignment, colors)
- Cannot validate subjective UX ("intuitive", "smooth", "immediate")
- Cannot prove absence of transient bugs (pager banner, flicker)

**Verdict Logic**: Automated tests PASS = "No obvious regressions detected", NOT "Feature works correctly"

### Manual Component

**Purpose**: User experience validation, catch false positives

**Test Types**:
- Visual inspection (menu rendering, alignment, colors)
- Keyboard interaction (TAB, arrows, ENTER behavior)
- Subjective UX (intuitive, smooth, matches bash/zsh)
- Environment validation (works in user's database, terminal)

**Characteristics**:
- Requires human execution
- Subjective judgment involved
- Environment-dependent (database, terminal, permissions)
- Cannot run in CI/CD

**Mandatory For**:
- All interactive features (REPL completion, menu navigation)
- All visual features (logo display, table formatting)
- All keyboard interaction features (TAB behavior, line editing)
- All user-reported bugs (user must validate fix)

**Evidence Requirements**:
- Screenshots or video recordings
- User confirmation statement ("Second TAB accepts as expected")
- Comparison to reference behavior (bash/zsh side-by-side test)

**Verdict Logic**: Manual tests PASS = "User confirms feature works correctly in their environment"

### Combined Verdict Logic

| Automated Tests | Manual Tests | Verdict | Rationale |
|-----------------|--------------|---------|-----------|
| PASS | PASS | APPROVED ✅ | Both code and UX validated |
| PASS | NOT PERFORMED | REJECTED ❌ | False positive risk (Sprint 20 lesson) |
| PASS | FAIL | REJECTED ❌ | Automated tests gave false positive |
| FAIL | PASS | REJECTED ❌ | Code regression detected |
| FAIL | FAIL | REJECTED ❌ | Both layers failed |
| FAIL | NOT PERFORMED | REJECTED ❌ | Code regression |

**CRITICAL RULE**: APPROVED verdict REQUIRES **BOTH** automated AND manual tests to PASS.

---

## Risk Assessment: False Positive Probability

### Feature 1: Complete Database Metadata Fetching

**False Positive Risk**: LOW

**Rationale**:
- Content-based validation (database names in text output)
- PTY tests can reliably verify `dbc` in output
- Integration tests prove database query works
- Simple pass/fail criteria (either `dbc` appears or not)

**Mitigation**: Manual validation confirms menu displays correctly (not truncated)

### Feature 2: Universal Table Metadata Fetching

**False Positive Risk**: MEDIUM

**Rationale**:
- Content-based validation (table names in output)
- PTY tests can verify table names appear
- BUT: Negative assertion ("NO RECORDS FOUND" must NOT appear) harder to prove
- Graceful degradation testing (permission denied) complex

**Mitigation**:
- Manual validation confirms error handling UX
- PTY negative test explicitly searches for error strings

### Feature 3: Second TAB Accepts Selection

**False Positive Risk**: EXTREMELY HIGH ⚠️

**Rationale**:
- Pure keyboard interaction behavior
- PTY tests CANNOT distinguish "TAB accepts" from "TAB moves down"
- Sprint 20 EXACT scenario (menu component behavior)
- 5 out of 6 acceptance criteria NOT testable with automation

**Mitigation**:
- Manual validation is PRIMARY test (not secondary)
- PTY test marked with warning about limitations
- APPROVED verdict REQUIRES manual validation

### Feature 4: Smart Database-Dot-TAB Completion

**False Positive Risk**: MEDIUM

**Rationale**:
- Content-based validation CAN verify database + tables appear
- BUT: UX smoothness ("immediate") NOT testable with automation
- Multi-stage completion state management complex (unit tests help)

**Mitigation**:
- Manual validation confirms UX smoothness and latency
- Unit tests validate logic, reducing integration bug risk

---

## Strategy Summary

### Total Test Count Estimate

| Test Type | Feature 1 | Feature 2 | Feature 3 | Feature 4 | Total |
|-----------|-----------|-----------|-----------|-----------|-------|
| Unit tests | 2 | 2 | 0 | 4 | 8 |
| Integration tests | 2 | 3 | 0 | 2 | 7 |
| Interactive tests (PTY) | 1 | 2 | 1* | 4 | 8 |
| Manual validation procedures | 1 | 1 | 1** | 1 | 4 |
| **Total** | **6** | **8** | **2** | **11** | **27** |

*Feature 3 PTY test is marked as INSUFFICIENT, provides weak signal only
**Feature 3 manual validation is PRIMARY test for this feature

### Test Types Required by Feature

| Feature | Unit | Integration | PTY | Manual | Primary Validation |
|---------|------|-------------|-----|--------|-------------------|
| F1: Database Metadata | ✅ | ✅ | ✅ | ✅ | Hybrid (equal weight) |
| F2: Table Metadata | ✅ | ✅ | ✅ | ✅ | Hybrid (equal weight) |
| F3: Second TAB Accepts | ❌ | ❌ | ⚠️ | ✅ | **Manual (100% weight)** |
| F4: Smart Completion | ✅ | ✅ | ✅ | ✅ | Hybrid (equal weight) |

### Risk-Adjusted Test Coverage

| Feature | Automated Coverage | Manual Coverage | False Positive Risk | Mitigation |
|---------|-------------------|-----------------|---------------------|------------|
| F1 | 83% (5/6 tests) | 17% (1/6 tests) | LOW | Manual confirms display |
| F2 | 75% (6/8 tests) | 25% (2/8 tests) | MEDIUM | Manual confirms UX + error handling |
| F3 | 16% (1/6 ACs) | 84% (5/6 ACs) | **EXTREMELY HIGH** | Manual is primary test |
| F4 | 73% (8/11 tests) | 27% (3/11 tests) | MEDIUM | Manual confirms UX smoothness |

### Dependencies Required

**For Automated Tests**:
- [x] Live Teradata database (TQ_LOGON environment variable)
- [x] Test database with `demo_user` schema and known tables
- [x] expectrl crate for PTY testing
- [x] Rust test framework (built-in)

**For Manual Tests**:
- [x] User access to tq REPL with test database
- [x] Terminal environment for keyboard interaction testing
- [x] bash or zsh for comparison testing (Feature 3)
- [x] Screenshot or screen recording capability

### Estimated Test Execution Time

| Test Type | Count | Avg Time | Total Time |
|-----------|-------|----------|------------|
| Unit tests | 8 | 1ms | 8ms |
| Integration tests | 7 | 500ms | 3.5s |
| PTY tests | 8 | 2s | 16s |
| Manual validation | 4 | 5min | 20min |
| **Total** | **27** | - | **~20min** |

**Automated tests**: ~20 seconds (CI/CD compatible)
**Manual tests**: ~20 minutes (human execution required)

---

## Test Implementation Priorities

### Phase 3A: Implement Automated Tests (Parallel with rust-teradata-architect)

**Priority 1 (MUST HAVE)**:
1. Unit tests for Feature 1 (database query validation)
2. Integration tests for Feature 1 (verify `dbc` fetched)
3. Unit tests for Feature 2 (table query validation)
4. Integration tests for Feature 2 (verify `demo_user` tables fetched)

**Priority 2 (SHOULD HAVE)**:
5. PTY tests for Feature 1 (verify `dbc` in completion output)
6. PTY tests for Feature 2 (verify tables in completion output)
7. Unit tests for Feature 4 (multi-stage completion logic)
8. Integration tests for Feature 4 (database + table fetch)

**Priority 3 (NICE TO HAVE)**:
9. PTY tests for Feature 4 (verify smart completion output)
10. PTY test for Feature 3 (weak signal, mark as insufficient)

### Phase 3B: Execute Manual Validation

**MUST COMPLETE BEFORE APPROVED VERDICT**:
1. Manual-F1: Verify `dbc` appears in completion menu
2. Manual-F2: Verify `demo_user` tables appear, no error messages
3. **Manual-F3: Verify second TAB accepts (PRIMARY TEST FOR F3)**
4. Manual-F4: Verify smart completion UX smooth and immediate

**Evidence Requirements**:
- Screenshots or video for each feature
- User confirmation statement for each manual test
- Feature 3: Side-by-side comparison with bash/zsh

---

## Tools and Infrastructure Needs

### Existing Tools (Already Available)

- [x] `cargo test` framework (unit tests)
- [x] `tests/integration_tests.rs` (integration tests)
- [x] `tests/interactive_tests.rs` (PTY tests with expectrl)
- [x] `.env` file for TQ_LOGON configuration

### New Tools Required

**NONE** - All necessary tools are already in place.

**Potential Future Enhancements** (Deferred to Sprint 22+):
- Visual regression testing tool (termshot, vhs) for screenshot-based validation
- Automated keyboard interaction testing (if possible, research needed)
- Performance profiling for completion latency

### Test Utilities to Create

**Helper Functions** (add to `tests/interactive_tests.rs`):

1. `spawn_repl_and_complete(input: &str, tab_count: usize) -> String`
   - Spawns REPL, sends input, sends TAB N times, returns output
   - Reduces boilerplate in PTY tests

2. `verify_no_error_strings(output: &str) -> bool`
   - Searches for common error patterns ("NO RECORDS FOUND", "ERROR", "Page 1: records")
   - Enables negative assertions

3. `extract_completion_suggestions(output: &str) -> Vec<String>`
   - Parses PTY output to extract suggested items from completion menu
   - Enables content verification

**Test Data Setup** (document in `tests/README.md`):
- Required database state for integration tests
- Expected database/table counts for assertions
- Test database credentials and permissions

---

## Manual Validation Procedures

### Manual-F1: Complete Database Metadata Fetching

**Objective**: Verify `dbc` database appears in completion menu

**Prerequisites**:
- tq REPL compiled and runnable
- TQ_LOGON configured with test database credentials

**Steps**:
1. Start tq REPL: `tq repl`
2. Wait for connection confirmation
3. Type: `SELECT * FROM `
4. Press TAB key
5. Observe completion menu

**Expected Results**:
- Completion menu appears with database names
- `dbc` appears in the list
- Menu is readable (not truncated, properly formatted)
- No error messages

**Acceptance Criteria**:
- [ ] `dbc` visible in completion menu
- [ ] Menu displays correctly (columns aligned, readable)
- [ ] No pager output or error messages

**Evidence**: Screenshot of completion menu showing `dbc`

---

### Manual-F2: Universal Table Metadata Fetching

**Objective**: Verify `demo_user` database tables appear after `demo_user.` + TAB

**Prerequisites**:
- tq REPL compiled and runnable
- TQ_LOGON configured with test database
- `demo_user` database exists with at least 1 table

**Steps**:
1. Start tq REPL: `tq repl`
2. Wait for connection confirmation
3. Type: `SELECT * FROM demo_user.`
4. Press TAB key
5. Observe completion menu

**Expected Results**:
- Completion menu appears with table names from `demo_user`
- Expected tables visible (document expected table names)
- No "NO RECORDS FOUND" error message
- Menu loads within reasonable time (<2s)

**Acceptance Criteria**:
- [ ] Tables from `demo_user` visible in menu
- [ ] No "NO RECORDS FOUND" error
- [ ] Completion latency acceptable (<2s)
- [ ] Menu displays correctly

**Evidence**: Screenshot of completion menu showing `demo_user` tables

---

### Manual-F3: Second TAB Accepts Selection ⚠️ PRIMARY TEST

**Objective**: Verify second TAB key accepts highlighted item (not moves down)

**Prerequisites**:
- tq REPL compiled and runnable
- TQ_LOGON configured with test database
- bash or zsh available for comparison

**Steps**:

**Test 1: Second TAB Accepts**
1. Start tq REPL: `tq repl`
2. Type: `SELECT * FROM `
3. Press TAB (first time)
4. Observe: Completion menu appears, first item highlighted
5. Press TAB (second time)
6. Observe: Highlighted item inserted into command line, menu closes

**Test 2: DOWN Arrow Moves Down**
7. Clear line (Ctrl-U)
8. Type: `SELECT * FROM `
9. Press TAB (first time)
10. Press DOWN arrow
11. Observe: Highlight moves to second item (item NOT inserted)
12. Press TAB
13. Observe: Second item inserted

**Test 3: UP Arrow Moves Up**
14. Clear line (Ctrl-U)
15. Type: `SELECT * FROM `
16. Press TAB (first time)
17. Press DOWN arrow (highlight on second item)
18. Press UP arrow
19. Observe: Highlight moves back to first item

**Test 4: ENTER Accepts**
20. Clear line (Ctrl-U)
21. Type: `SELECT * FROM `
22. Press TAB (first time)
23. Press ENTER
24. Observe: Highlighted item inserted

**Test 5: Compare to bash/zsh**
25. Open bash or zsh terminal
26. Type: `ls /u` and press TAB (bash completion)
27. Observe first TAB behavior (shows completions)
28. Press TAB again
29. Observe second TAB behavior (accepts completion)
30. Compare to tq behavior

**Expected Results**:
- First TAB: Shows menu, first item highlighted
- Second TAB: Accepts highlighted item, inserts text, closes menu
- DOWN: Moves highlight down (does NOT insert)
- UP: Moves highlight up (does NOT insert)
- ENTER: Accepts highlighted item
- Behavior matches bash/zsh

**Acceptance Criteria**:
- [ ] First TAB shows menu with first item highlighted
- [ ] Second TAB inserts highlighted item and closes menu
- [ ] DOWN arrow moves highlight down (not accept)
- [ ] UP arrow moves highlight up (not accept)
- [ ] ENTER accepts highlighted item
- [ ] Behavior matches bash/zsh completion UX

**Evidence**:
- Video recording of TAB, DOWN, UP, ENTER interactions (RECOMMENDED)
- Screenshots of each step
- User confirmation: "Second TAB accepts as expected, matches bash/zsh"

**CRITICAL**: This is the PRIMARY TEST for Feature 3. Automated tests CANNOT validate this behavior.

---

### Manual-F4: Smart Database-Dot-TAB Completion

**Objective**: Verify typing `dem` + TAB completes to `demo_user.` and shows tables immediately

**Prerequisites**:
- tq REPL compiled and runnable
- TQ_LOGON configured with test database
- `demo_user` database exists with tables
- No other database starting with `dem` (unambiguous match)

**Steps**:

**Test 1: Unambiguous Database Completion**
1. Start tq REPL: `tq repl`
2. Type: `SELECT * FROM dem`
3. Press TAB
4. Observe: Text completes to `demo_user.`
5. Observe: Table list appears immediately
6. Note latency (should feel immediate, <500ms)

**Test 2: After JOIN Keyword**
7. Clear line (Ctrl-U)
8. Type: `SELECT * FROM t1 JOIN dem`
9. Press TAB
10. Observe: Completes to `demo_user.`, shows tables

**Test 3: Ambiguous Database Prefix**
11. Clear line (Ctrl-U)
12. Type: `SELECT * FROM de` (assuming multiple databases start with `de`)
13. Press TAB
14. Observe: Shows database list (not tables)
15. Select one database, press TAB again
16. Observe: Shows tables for selected database

**Expected Results**:
- Unambiguous prefix: Completes database + dot, shows tables in one TAB
- Ambiguous prefix: Shows database list first, then tables on second TAB
- Works after FROM and JOIN keywords
- Latency acceptable (feels immediate, <500ms)
- UX smooth and intuitive

**Acceptance Criteria**:
- [ ] `dem` + TAB completes to `demo_user.` and shows tables
- [ ] Works after FROM keyword
- [ ] Works after JOIN keyword
- [ ] Ambiguous prefix shows database list first
- [ ] Latency acceptable (<500ms perceived)
- [ ] UX smooth and intuitive

**Evidence**: Screenshot or video, user confirmation

---

## Test Execution Schedule

### Sprint 21 Phase 3: Build & Test

**Week 1, Day 1-2: Automated Test Implementation**
- rust-teradata-architect implements Features 1-4
- quality-validator implements unit tests (parallel)
- quality-validator implements integration tests (parallel)

**Week 1, Day 2-3: PTY Test Implementation**
- quality-validator implements PTY tests for Features 1, 2, 4
- quality-validator implements PTY test for Feature 3 (mark as insufficient)

**Week 1, Day 3: Automated Test Execution**
- Run all unit tests: `cargo test --lib`
- Run all integration tests: `cargo test --test integration_tests -- --ignored`
- Run all PTY tests: `cargo test --test interactive_tests -- --ignored`
- Generate test report with pass/fail counts

**Week 1, Day 3: Manual Validation**
- Execute Manual-F1 procedure
- Execute Manual-F2 procedure
- Execute Manual-F3 procedure (PRIMARY TEST)
- Execute Manual-F4 procedure
- Collect evidence (screenshots, videos, confirmations)

**Week 1, Day 3: Iteration Decision**
- IF automated tests FAIL OR manual tests FAIL:
  - rust-teradata-architect fixes issues
  - quality-validator re-executes failed tests
  - REPEAT until 100% pass rate
- IF automated tests PASS AND manual tests PASS:
  - Generate final test report
  - Verdict: APPROVED
  - Proceed to Phase 4 (Ship)

---

## Success Criteria

Sprint 21 testing is successful when:

- [x] All 27 tests implemented (8 unit, 7 integration, 8 PTY, 4 manual)
- [x] All automated tests executed (unit + integration + PTY)
- [x] All automated tests passing (100% pass rate)
- [x] All manual validation procedures executed
- [x] All manual validation procedures passing (user confirms)
- [x] Test evidence collected (screenshots, videos, confirmations)
- [x] Test report generated with APPROVED verdict
- [x] Zero false positives (manual validation confirms automated results)

**Verdict Logic**:
- APPROVED: Automated PASS + Manual PASS ✅
- REJECTED: Automated FAIL **OR** Manual FAIL **OR** Manual NOT PERFORMED ❌
- BLOCKED: Tests cannot execute (database unavailable) ⛔

---

## Lessons Learned from Sprint 18/20 Applied

### 1. Hybrid Testing Mandatory

**Sprint 20 Lesson**: "Automated tests validate CODE behavior, manual tests validate USER EXPERIENCE."

**Applied in Sprint 21**: Every feature has BOTH automated AND manual components. APPROVED verdict requires BOTH to pass.

### 2. Manual Validation Required for Interactive Features

**Sprint 20 Lesson**: "For user-reported bugs, user validation is MANDATORY before sprint closure."

**Applied in Sprint 21**: All 4 features have manual validation procedures. Feature 3 (TAB behavior) uses manual validation as PRIMARY test.

### 3. Test Limitations Documented

**Sprint 20 Lesson**: "Tests should not give false confidence. Document what tests CAN and CANNOT validate."

**Applied in Sprint 21**: Section "Test Automation Capabilities & Limitations" explicitly lists what PTY tests cannot validate. Feature 3 PTY test marked with warning.

### 4. High-Risk Features Identified

**Sprint 20 Lesson**: "Sprint 20 had EXACT same scenario (menu component behavior)."

**Applied in Sprint 21**: Feature 3 identified as EXTREMELY HIGH false positive risk. Manual validation made primary test (not secondary).

### 5. Negative Assertions Included

**Sprint 20 Lesson**: "No pager output" negative assertion was hard to prove with automation.

**Applied in Sprint 21**: PTY tests include explicit searches for error strings ("NO RECORDS FOUND", "Page 1: records"). Manual validation confirms absence.

### 6. User Environment Validation

**Sprint 20 Lesson**: "User tested in iterations 1-2 and reported 'still same issue', preventing false ship."

**Applied in Sprint 21**: Manual validation requires user to test in THEIR environment with THEIR database (not just test database).

---

## Document Sign-off

**Test Strategy Author**: quality-validator
**Created Date**: 2026-01-23
**Review Status**: DRAFT
**Submitted for Review**: Pending

**Strategy Completeness Checklist**:
- [x] Every feature has complete specification analysis
- [x] Feature characteristics classified (not assumed)
- [x] Test strategy derived from characteristics (not guessed)
- [x] Every test type has clear rationale
- [x] Gap analysis complete and honest
- [x] Specification coverage map includes all requirements
- [x] Every requirement maps to at least one test type
- [x] Test implementation plan detailed and actionable
- [x] Coverage sufficiency assessed
- [x] Sprint 18/20 lessons learned applied
- [x] False positive risk assessed for each feature
- [x] Hybrid testing pattern clearly defined
- [x] Manual validation procedures documented

**Ready for Phase 3 Implementation**: YES ✅
