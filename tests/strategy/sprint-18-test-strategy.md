# Sprint 18 Test Strategy: Critical Bug Fixes

**Created:** 2026-01-21
**Author:** quality-validator
**Sprint:** Sprint 18
**Features:** Logo Fix (P0), Tab Completion Rebuild (P0)

---

## Sprint Context

**Sprint Type:** Maintenance Sprint (CRISIS)
**Priority:** P0 - BLOCKING PRODUCTION USE

**Critical Bugs:**
1. Logo displays uppercase ASCII art instead of lowercase "tq" text with subtitle
2. Tab completion broken - text inserted at wrong position, keyword completion interfering

**User Impact:** Both bugs block productive use of the REPL tool.

---

## Feature-by-Feature Test Strategy

### Feature 1: Logo Fix

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/sprints/sprint-18-planning.md` Section "P0 - CRITICAL: Fix ASCII Art Logo"
- Requirements:
  1. "Logo shows lowercase 'tq' text (NOT ASCII block art)"
  2. "Logo includes subtitle 'Teradata Query tool v1.7.0'"
  3. "'tq' is displayed in Teradata orange (xterm-256 color 202)"
  4. "Text is simple and clean (no fancy ASCII art)"

**Feature Characteristics:**

**User Interaction Type:** Interactive PTY (REPL)
- ✅ Interactive PTY (REPL, terminal UI with cursor/colors/rendering)

**Explanation:** The logo is displayed when the REPL starts, requiring terminal color support and visual rendering.

**Observable Behavior:**
- ✅ Visual output in terminal (colors, formatting, layout)
- The logo uses ANSI color codes (xterm-256 color 202)
- Simple text output with newlines

**External Dependencies:**
- ✅ Terminal/PTY (terminal color sequences, color support)
- Requires terminal that supports xterm-256 colors

**Validation Challenges:**
- Visual color validation requires actual PTY with color support
- Color rendering varies by terminal emulator
- Need to verify text content and color codes

**Critical Behaviors to Validate:**
1. Logo text is lowercase "tq" (not uppercase ASCII art blocks)
2. Logo includes subtitle "Teradata Query tool v1.7.0"
3. Color code for "tq" is xterm-256 color 202 (Teradata orange)
4. No fancy ASCII art characters (████, etc.)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Unit tests cannot validate terminal output, colors, visual rendering

IF "Visual output in terminal" checked:
  → Interactive tests OR integration tests with output capture REQUIRED
  Reason: Unit tests cannot validate formatting, colors, layout
```

**Derived Test Types:**

**Test Type 1: Interactive Tests (expectrl)**
- **Validates:** Logo text content, color codes, visual appearance in terminal
- **Approach:** Spawn tq REPL in PTY, capture banner output, parse ANSI codes and text content
- **Rationale:** Only way to validate visual output with colors as users see it
- **Gap if missing:** Cannot verify logo displays correctly to users, color bugs would not be caught
- **Necessity:** ✅ REQUIRED

**Test Type 2: Unit Tests**
- **Validates:** Banner generation logic (text formatting, ANSI code construction)
- **Approach:** Test print_banner function directly, verify returned string contains correct text and ANSI codes
- **Rationale:** Fast validation of logic without spawning processes
- **Gap if missing:** Logic bugs in banner generation, but would be caught by interactive tests
- **Necessity:** ⚠️ RECOMMENDED (nice-to-have but not blocking)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Interactive tests (expectrl) | ✅ REQUIRED | Validates logo as users see it | Visual bugs, color issues, content errors | MUST IMPLEMENT |
| Unit tests | ⚠️ RECOMMENDED | Validates banner logic | Logic bugs (would be caught by interactive) | SHOULD IMPLEMENT if time allows |
| Manual tests | ✅ REQUIRED | Human validates visual quality and color appearance | Subjective color/appearance issues | DOCUMENT TEST CASES |

**Summary:**
- ✅ REQUIRED test types: 2 (Interactive, Manual)
- ⚠️ RECOMMENDED test types: 1 (Unit)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| LOGO-1 | "Logo shows lowercase 'tq' text" | sprint-18-planning.md | Interactive + Manual | Visual validation in PTY | TC-LOGO-001 |
| LOGO-2 | "Logo includes subtitle 'Teradata Query tool v1.7.0'" | sprint-18-planning.md | Interactive + Manual | Text content validation | TC-LOGO-001 |
| LOGO-3 | "'tq' is displayed in Teradata orange (color 202)" | sprint-18-planning.md | Interactive + Manual | ANSI color code validation | TC-LOGO-001 |
| LOGO-4 | "No ASCII art blocks (████)" | sprint-18-planning.md | Interactive + Manual | Anti-pattern validation | TC-LOGO-001 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements (missing test coverage)
- ✅ No unjustified test types (test types without requirement rationale)

**Coverage Gaps:** None - all requirements covered

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Performance/Benchmark Tests**
- **Reason for omission:** Banner display has no performance requirements, happens once at startup
- **What won't be validated:** Banner generation speed
- **Risk assessment:** LOW - Logo display is instantaneous, no performance SLA
- **Mitigation:** N/A
- **Revisit criteria:** Never - not a performance-critical feature

#### 6. Test Implementation Plan

**Test Type: Interactive Tests (expectrl)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 1 test
- **Key scenarios to cover:**
  1. Start tq REPL, capture banner output
  2. Verify "tq" appears (lowercase) in output
  3. Verify "Teradata Query tool v1.7.0" subtitle appears
  4. Parse ANSI codes to verify color 202 is used for "tq"
  5. Verify no block ASCII art characters (████)
- **Implementation notes:** Need to parse ANSI escape sequences to validate color codes

**Test Type: Manual Tests**
- **Location:** `tests/cases/TC-LOGO-001.md`
- **Framework:** Human visual inspection
- **Test count estimate:** 1 test
- **Key scenarios to cover:**
  1. Start tq REPL in real terminal
  2. Visually inspect logo appearance
  3. Verify color appears orange (subjective)
  4. Verify text is clean and professional
- **Implementation notes:** Document with screenshots if possible

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Interactive tests validate: Logo text content, color codes, absence of ASCII art
- Manual tests validate: Visual quality, color appearance (subjective)
- Combined coverage: Comprehensive

**Gaps in combined coverage:** None

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

---

### Feature 2: Tab Completion Rebuild

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/builder/sprints/sprint-18-planning.md` Section "P0 - CRITICAL: Rebuild Tab Completion"
- Requirements:
  1. "Tab completion for database names after FROM/JOIN works correctly"
  2. "Tab completion for table names after FROM/JOIN works correctly"
  3. "Tab completion for column names in SELECT/WHERE works correctly"
  4. "NO keyword completion (dropped completely for now)"
  5. "Text inserted at CORRECT cursor position (not beginning of line)"
  6. "Span calculation fixed and tested"
  7. "All 3 completion contexts work in isolation"

**Feature Characteristics:**

**User Interaction Type:** Interactive PTY (REPL)
- ✅ Interactive PTY (REPL, terminal UI with cursor/colors/rendering)

**Explanation:** Tab completion is a core REPL interactive feature requiring keyboard input simulation (Tab key) and cursor position awareness.

**Observable Behavior:**
- ✅ Visual output in terminal (completion suggestions, inserted text)
- ✅ State management (completion context, metadata cache)
- Tab key triggers completion mechanism
- Completion text inserted at cursor position
- Visual suggestions displayed to user

**External Dependencies:**
- ✅ Database connection (requires live database for metadata queries)
- ✅ Terminal/PTY (terminal control sequences, cursor positioning)
- Metadata queries need real database (DBC.DatabasesV, DBC.TablesV, etc.)

**Validation Challenges:**
- Cursor position validation requires actual PTY
- Span calculation errors only visible in interactive environment
- Context detection depends on SQL parsing and cursor state
- Metadata completion requires live database with queryable objects

**Critical Behaviors to Validate:**
1. Tab after "FROM " shows database names and current DB tables (not keywords)
2. Tab after "FROM database." shows tables in that database
3. Tab after "SELECT " or "WHERE " shows column names (when context known)
4. Tab after qualified name "database.tab" completes table names
5. NO keyword suggestions appear in any context
6. Completion text inserted at cursor position (not beginning of line)
7. Span calculation correct for all contexts

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Unit tests cannot validate terminal output, cursor behavior, visual rendering

IF "Database connection" checked:
  → Integration tests with live database REQUIRED
  Reason: Mocks don't catch SQL syntax errors, metadata query issues

IF "Visual output in terminal" checked:
  → Interactive tests OR integration tests with output capture REQUIRED
  Reason: Unit tests cannot validate formatting, cursor position
```

**Derived Test Types:**

**Test Type 1: Interactive Tests (expectrl) - PRIMARY**
- **Validates:** Tab completion behavior as users experience it, cursor position, completion insertion
- **Approach:** Spawn tq REPL, send SQL text, press Tab, capture suggestions and inserted text
- **Rationale:** MANDATORY for REPL features - only way to test real user experience
- **Gap if missing:** Cannot verify completion works for users, cursor bugs would not be caught
- **Necessity:** ✅ REQUIRED

**Test Type 2: Unit Tests**
- **Validates:** SQL context analysis logic, completion candidate filtering, span calculation
- **Approach:** Test sql_context.rs functions directly with various SQL inputs
- **Rationale:** Fast validation of parsing logic and context detection
- **Gap if missing:** Logic bugs in context analysis, but would be caught by interactive tests
- **Necessity:** ⚠️ RECOMMENDED

**Test Type 3: Manual Tests**
- **Validates:** Completion works in real-world scenarios, no unexpected behavior
- **Approach:** Human tests various SQL patterns with Tab key in live REPL
- **Rationale:** Catch edge cases and usability issues automated tests might miss
- **Gap if missing:** Edge cases, real-world usage patterns not covered
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Interactive tests (expectrl) | ✅ REQUIRED | Validates completion as users see it | Cannot verify cursor position, insertion point | MUST IMPLEMENT |
| Unit tests | ⚠️ RECOMMENDED | Validates context analysis logic | Logic bugs (would be caught by interactive) | EXISTING TESTS SUFFICIENT |
| Manual tests | ✅ REQUIRED | Human validates real-world usage | Edge cases, usability issues | DOCUMENT TEST CASES |

**Summary:**
- ✅ REQUIRED test types: 2 (Interactive, Manual)
- ⚠️ RECOMMENDED test types: 1 (Unit - already exists)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| COMP-1 | "Database completion after FROM/JOIN" | sprint-18-planning.md | Interactive + Manual | Only interactive can validate what user sees | TC-COMPLETION-001 |
| COMP-2 | "Table completion after FROM/JOIN" | sprint-18-planning.md | Interactive + Manual | Only interactive can validate what user sees | TC-COMPLETION-002 |
| COMP-3 | "Column completion in SELECT/WHERE" | sprint-18-planning.md | Interactive + Manual | Only interactive can validate what user sees | TC-COMPLETION-003 |
| COMP-4 | "Qualified name completion (database.table)" | sprint-18-planning.md | Interactive + Manual | Context-sensitive completion in PTY | TC-COMPLETION-004 |
| COMP-5 | "NO keyword completion" | sprint-18-planning.md | Interactive + Manual | Anti-pattern validation | TC-COMPLETION-005 |
| COMP-6 | "Text inserted at correct cursor position" | sprint-18-planning.md | Interactive + Manual | Cursor position only testable in PTY | ALL COMPLETION TESTS |
| COMP-7 | "Span calculation correct" | sprint-18-planning.md | Unit + Interactive | Unit tests logic, interactive validates result | Existing unit tests + ALL |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements (missing test coverage)
- ✅ No unjustified test types

**Coverage Gaps:** None - all requirements covered

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Performance/Benchmark Tests**
- **Reason for omission:** Sprint 18 is a bug fix sprint, not performance optimization
- **What won't be validated:** Completion speed, metadata query performance
- **Risk assessment:** LOW - Performance already validated in previous sprints (TC042, TC043)
- **Mitigation:** Re-run existing performance tests (TC042, TC043) after fixes
- **Revisit criteria:** If user reports slowness after fix

#### 6. Test Implementation Plan

**Test Type: Interactive Tests (expectrl)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 5 tests
- **Key scenarios to cover:**
  1. Tab after "SELECT * FROM " → shows databases and tables (NOT keywords)
  2. Tab after "SELECT * FROM my_database." → shows tables in my_database
  3. Tab after "SELECT " (with known table context) → shows columns
  4. Tab after "WHERE " (with known table context) → shows columns
  5. Verify completion inserts at cursor position (not line start)
- **Implementation notes:**
  - Need to verify text insertion position (critical for bug fix)
  - Must explicitly check for NO keyword suggestions
  - Requires live database with queryable objects

**Test Type: Manual Tests**
- **Location:** `tests/cases/TC-COMPLETION-*.md`
- **Framework:** Human testing with live REPL
- **Test count estimate:** 5 test cases
- **Key scenarios to cover:**
  1. Database completion after FROM
  2. Table completion after FROM
  3. Column completion in SELECT/WHERE
  4. Qualified name completion (database.table)
  5. Verify NO keyword completion in any context
- **Implementation notes:** Document expected vs actual behavior, capture screenshots if issues found

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Interactive tests validate: Completion triggers, suggestions shown, text insertion, cursor position
- Unit tests validate: Context analysis logic, span calculation (existing tests)
- Manual tests validate: Real-world usage, edge cases, no unexpected keywords
- Combined coverage: Comprehensive

**Gaps in combined coverage:**
- Cross-platform testing (macOS/Linux/Windows) not explicitly covered - will test on available platform only
- Extremely long SQL statements (1000+ characters) not covered - low priority edge case

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**Gaps are acceptable because:**
- Cross-platform: Tab completion uses reedline (cross-platform library), platform-specific issues unlikely
- Long SQL: Edge case, not blocking production use

---

## Strategy Summary

**Total Features Analyzed:** 2

**Test Types Required:**
- Interactive tests: ✅ [Logo, Tab Completion]
- Manual tests: ✅ [Logo, Tab Completion]
- Unit tests: ⚠️ [Existing tests sufficient]

**Estimated Test Count:**
- Interactive: 6 tests (1 logo + 5 completion)
- Manual: 6 test cases (1 logo + 5 completion)
- Total: 12 tests

**Risk Assessment:**
- HIGH risk gaps: None
- MEDIUM risk gaps: None
- LOW risk gaps: 2 (cross-platform, long SQL edge case)

**Dependencies Required:**
- Live database: Yes (for tab completion metadata queries)
- Network access: No
- Specific OS: No (will test on available platform)
- Other: Terminal with xterm-256 color support for logo color validation

---

## Test Execution Approach

### Phase 1: Pre-Implementation (Ready for Execution)

**Test Case Documentation:**
- ✅ Create TC-LOGO-001.md
- ✅ Create TC-COMPLETION-001.md through TC-COMPLETION-005.md
- ✅ Update tests/cases/INDEX.md

### Phase 2: Post-Implementation (Wait for Fixes)

**Once rust-teradata-architect completes fixes:**

1. **Build and verify compilation**
   ```bash
   cargo build --release
   cargo clippy --all-targets
   ```

2. **Run existing test suite (regression check)**
   ```bash
   cargo test --lib
   cargo test --test integration_tests
   ```

3. **Run new interactive tests**
   ```bash
   cargo test --test interactive_tests test_logo_display -- --ignored
   cargo test --test interactive_tests test_tab_completion -- --ignored
   ```

4. **Execute manual test cases**
   - Start tq REPL manually
   - Follow each test case procedure
   - Document actual results
   - Capture screenshots if issues found

5. **Create test report**
   - Document: `tests/results/sprint-18/TEST-REPORT.md`
   - Include: Test execution proof (cargo test output)
   - Include: Manual test results
   - Include: Screenshots of logo and completion behavior
   - Verdict: APPROVED / REJECTED / BLOCKED

### Phase 3: Iteration (If Tests Fail)

**If tests fail:**
1. Document failure details clearly
2. Provide rust-teradata-architect with reproduction steps
3. Re-test after fixes
4. Repeat until 100% pass rate

### Verdict Criteria

**APPROVED:** Both critical bugs fixed, all tests pass
- ✅ Logo shows lowercase "tq" with subtitle
- ✅ Tab completion works (databases, tables, columns)
- ✅ Tab completion inserts at correct position
- ✅ NO keyword completion appears
- ✅ All interactive tests pass
- ✅ All manual tests pass
- ✅ Zero regressions (existing tests still pass)

**REJECTED:** Fixes incomplete or introduce regressions
- ❌ Logo still shows ASCII art OR missing subtitle
- ❌ Tab completion still broken (wrong position, keywords appear)
- ❌ New bugs introduced
- ❌ Existing tests fail (regressions)

**BLOCKED:** Cannot execute tests
- ⛔ Database not available
- ⛔ Build fails
- ⛔ Terminal environment issues

---

## Strategy Validation Checklist

**Before submitting to tq-project-manager for review:**

- ✅ Every feature has complete specification analysis section
- ✅ Feature characteristics are classified (not assumed)
- ✅ Test strategy is derived from characteristics (not guessed)
- ✅ Every test type has clear rationale
- ✅ Gap analysis is complete and honest
- ✅ Specification coverage map includes all requirements
- ✅ Every requirement maps to at least one test type
- ✅ Test implementation plan is detailed and actionable
- ✅ Coverage sufficiency is assessed
- ✅ No hand-waving or vague justifications

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-01-21
**Review Status:** READY FOR REVIEW
**Submitted for Review:** 2026-01-21

**Next Steps:**
1. Create test case documents (TC-LOGO-001, TC-COMPLETION-001 through 005)
2. Wait for rust-teradata-architect to complete fixes
3. Execute tests with live database
4. Create test report
