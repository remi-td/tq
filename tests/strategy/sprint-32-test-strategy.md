# Sprint 32 Test Strategy

**Created:** 2026-02-03
**Author:** quality-validator
**Sprint:** Sprint 32
**Features:** Content-Based Column Width (P0) + Fix GitHub README Display (P1)

---

## Instructions for quality-validator

This strategy applies Sprint 31 lessons learned to properly classify Feature #13 as Type 4 (visual/interactive) requiring MANDATORY manual validation.

**Key Principles:**
1. Test strategy derives from feature characteristics (not assumptions)
2. Every test type must be justified by specification requirement
3. Type 4 features require manual validation (not optional)
4. Quality validator verdict is ADVISORY for visual features
5. Sprint coordinator must perform manual validation before closure

---

## Feature-by-Feature Test Strategy

### Feature 1: Content-Based Column Width (#13) [P0]

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-32-planning.md` §Feature 1: Content-Based Column Width (lines 52-90)
- Secondary: `docs/specifications/output-formats.md` §Table Format (lines 22-93)
- Implementation: `src/format/table.rs` (column width calculation logic)
- Related: GitHub Issue #13

**Requirements (10 Acceptance Criteria):**
1. **AC-1**: Column width calculated from actual cell content, not schema type
2. **AC-2**: Maximum column width capped at reasonable limit (e.g., 100 chars)
3. **AC-3**: Minimum column width respects column header length
4. **AC-4**: `SELECT * FROM DBC.Databases` displays 8+ columns in 117-char terminal
5. **AC-5**: Existing table formatting tests still pass
6. **AC-6**: Manual validation: REPL query shows improved density
7. **AC-7**: No performance regression (measure column calc time)
8. **AC-8**: Works correctly with NULL values
9. **AC-9**: Works correctly with numeric alignment
10. **AC-10**: Documentation updated with new column width behavior

**Feature Characteristics:**

**User Interaction Type:** ✅ **Type 4: Interactive/Visual Feature (Per Sprint 31 Testing Philosophy)**

**Explanation:** This feature is classified as Type 4 per `docs/testing/approach.md` §Feature Types and Their Testing Limitations (lines 426-436). While the underlying logic can be unit tested, the PRIMARY user-observable behavior is VISUAL TABLE RENDERING in a real terminal. Users will judge success by seeing MORE COLUMNS VISIBLE in actual REPL sessions. The specification explicitly states the user pain point: "Current: shows only 2 columns... Desired: shows 8+ columns" (sprint-32-planning.md lines 59-61). This is a VISUAL DENSITY improvement that CANNOT be fully validated by automated tests alone.

**Critical Sprint 31 Lesson:** Sprint 29/30 post-mortem showed that 100% automated test pass rates delivered completely broken visual features. Per `docs/testing/philosophy.md` §Testing Limitations and Manual Validation (lines 229-307), Type 4 features REQUIRE manual validation before claiming success.

**Observable Behavior:** [Check all that apply]
- ✅ **Visual output in terminal** (table layout, column spacing, density) - PRIMARY OBSERVABLE
- ✅ Structured data output (JSON, CSV) - Secondary (not affected by visual changes)
- ❌ File system side effects
- ❌ Database side effects
- ❌ Network interactions
- ⚠️ Performance characteristics (column calculation time must not regress)
- ❌ State management

**External Dependencies:**
- ✅ **Terminal/PTY** (table rendering, column width calculations based on terminal size)
- ✅ **Database connection** (for realistic testing with actual data types and lengths)
- ❌ File system access
- ❌ Network access
- ❌ System clipboard
- ❌ Operating system specific features

**Validation Challenges:**
- **Challenge 1**: Visual column density can only be verified by human looking at actual terminal output
- **Challenge 2**: Automated tests can verify column count but NOT whether layout "looks good" or "is usable"
- **Challenge 3**: The user pain point (only 2 columns visible) requires testing at SPECIFIC terminal width (117 chars from issue #13)
- **Challenge 4**: NULL value rendering must preserve visual alignment (automated tests check text, not visual alignment)
- **Challenge 5**: Numeric alignment must be preserved (automated tests check logic, not visual appearance)
- **Challenge 6**: Maximum width cap (100 chars) behavior requires visual verification to ensure truncation is graceful

**Critical Behaviors to Validate:**
1. **Column width from content** - "Column width calculated from actual cell content, not schema type" (AC-1)
2. **Maximum width capping** - "Maximum column width capped at reasonable limit (e.g., 100 chars)" (AC-2)
3. **Header length minimum** - "Minimum column width respects column header length" (AC-3)
4. **Density improvement** - "`SELECT * FROM DBC.Databases` displays 8+ columns in 117-char terminal" (AC-4)
5. **NULL handling** - "Works correctly with NULL values" (AC-8)
6. **Numeric alignment** - "Works correctly with numeric alignment" (AC-9)
7. **Performance preservation** - "No performance regression (measure column calc time)" (AC-7)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Type 4: Interactive/Visual Feature" classified:
  → Unit tests REQUIRED (for logic correctness)
  → Integration tests REQUIRED (for end-to-end correctness)
  → MANDATORY manual validation REQUIRED (for user-observable behavior)
  Reason: Automated tests cannot validate visual table density improvements

IF "Visual output in terminal" checked:
  → Manual validation REQUIRED at specific terminal widths (80, 117, 120, 160)
  Reason: User-reported issue #13 specifically mentions 117-char terminal

IF "Performance characteristics" checked:
  → Benchmark tests REQUIRED (measure column calculation time before/after)
  Reason: AC-7 explicitly requires "No performance regression"

IF "Database connection" checked:
  → Integration tests with live database RECOMMENDED
  Reason: Realistic data types and lengths needed for comprehensive validation
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Column width calculation logic, max width capping, header length minimum, NULL handling, numeric alignment
- **Approach:** Test `calculate_column_width()` function in isolation with mocked data
  - Test width calculation from content (short strings, long strings)
  - Test maximum width cap (100 chars or configured limit)
  - Test minimum width respects header length
  - Test NULL value handling (width calculation treats "[NULL]" as content)
  - Test numeric alignment logic preservation
  - Test edge cases: empty strings, very long strings (1000+ chars), Unicode characters
- **Rationale:** Pure logic can be unit tested without terminal - catches calculation bugs, off-by-one errors, edge cases
- **Gap if missing:** Logic bugs could cause incorrect widths, overflow issues, NULL rendering problems
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests**
- **Validates:** End-to-end table formatting with content-based widths, integration with existing formatting pipeline
- **Approach:** Test complete table rendering with `QueryResult` objects
  - Create `QueryResult` with various column types (VARCHAR(64) with 10-char content, INTEGER, DATE, NULL)
  - Verify formatted table output has correct structure
  - Verify column count in output (can count `│` separators)
  - Verify existing tests still pass (AC-5)
  - Test batch mode vs. TTY mode (terminal width detection)
- **Rationale:** Validates integration with existing table formatter, ensures no regressions in table structure
- **Gap if missing:** Integration issues with existing formatter, broken table structure, batch mode problems
- **Necessity:** ✅ REQUIRED

**Test Type 3: Performance Benchmark Tests**
- **Validates:** Column calculation time does not regress (AC-7)
- **Approach:** Use criterion crate to benchmark column width calculation
  - Baseline: Current width calculation (from schema type definition)
  - New: Content-based width calculation (scan cell values)
  - Measure with various table sizes: 10 rows x 10 cols, 100 rows x 50 cols, 1000 rows x 20 cols
  - Acceptable threshold: <10% regression (or <1ms absolute for typical tables)
- **Rationale:** AC-7 explicitly requires performance validation - content scanning could be slower than schema lookup
- **Gap if missing:** Performance regression could cause slow table rendering, poor UX for large result sets
- **Necessity:** ✅ REQUIRED (explicit AC)

**Test Type 4: Manual Validation - MANDATORY**
- **Validates:** Actual visual column density improvement at user-reported terminal width (AC-4, AC-6)
- **Approach:** Human tester performs REPL validation at multiple terminal widths:
  1. **Terminal width: 117 chars** (from issue #13) - CRITICAL TEST
     - Execute: `SELECT * FROM DBC.Databases`
     - Expected: 8+ columns visible (vs. 2 columns before)
     - Capture: `script` command output or screenshot
  2. **Terminal width: 80 chars** - Narrow terminal
     - Verify: Table still renders correctly, columns don't overflow
     - Expected: More columns than before (even if fewer than 117-char terminal)
  3. **Terminal width: 120 chars** - Standard wide terminal
     - Verify: Table uses available space efficiently
  4. **Terminal width: 160 chars** - Very wide terminal
     - Verify: Table shows many columns without excessive whitespace
  5. **Visual alignment check**:
     - Verify: NULL values aligned correctly
     - Verify: Numeric columns right-aligned correctly
     - Verify: Text columns left-aligned correctly
     - Verify: Headers aligned with column content
  6. **Truncation check**:
     - Query table with very long VARCHAR content (200+ chars)
     - Verify: Truncation at 100-char cap is graceful and readable
- **Rationale:** MANDATORY per Sprint 31 testing philosophy - visual features REQUIRE human verification. AC-4 and AC-6 explicitly require manual validation. User pain point is VISUAL ("only 2 columns visible").
- **Gap if missing:** Automated tests could pass while visual density is NOT improved, repeating Sprint 29/30 pattern
- **Necessity:** ✅ **MANDATORY - BLOCKING FOR SPRINT CLOSURE**
- **Evidence Required:** Script command output or screenshots showing before/after column density

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates column width calculation logic, edge cases, NULL handling | Logic bugs, incorrect widths, overflow issues | MUST IMPLEMENT (15+ tests) |
| Integration tests | ✅ REQUIRED | Validates table formatting pipeline integration, no regressions | Table structure broken, batch mode broken, existing tests fail | MUST IMPLEMENT (10+ tests) |
| Performance benchmarks | ✅ REQUIRED | AC-7 explicitly requires performance validation | Performance regression, slow rendering | MUST IMPLEMENT (criterion) |
| Manual validation | ✅ **MANDATORY** | Type 4 feature - visual density only verifiable by human at actual terminal widths | Visual improvement NOT achieved, repeat Sprint 29/30 false success | **BLOCKING - MUST PERFORM** |
| Interactive tests (expectrl) | ⚠️ RECOMMENDED | Could automate column count verification in PTY | Limited value - manual validation more effective for visual features | OPTIONAL |

**Summary:**
- ✅ REQUIRED test types: 4 (Unit, Integration, Benchmark, Manual) - MUST implement all
- ⚠️ RECOMMENDED test types: 0
- ❌ NOT NEEDED test types: 0
- 🔴 **MANDATORY manual validation: 1 - BLOCKING for sprint closure**

#### 4. Specification Coverage Map

**Map each specification requirement (AC) to test type(s) that validate it:**

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-1 | "Column width calculated from actual cell content, not schema type" | sprint-32-planning.md §63 | Unit + Integration | Unit validates calculation logic, integration validates end-to-end | TC-032-001, TC-032-011 |
| AC-2 | "Maximum column width capped at reasonable limit (e.g., 100 chars)" | sprint-32-planning.md §64 | Unit + Manual | Unit validates cap logic, manual validates truncation is graceful | TC-032-002, MANUAL-1 |
| AC-3 | "Minimum column width respects column header length" | sprint-32-planning.md §65 | Unit + Integration | Unit validates min width logic, integration validates in table context | TC-032-003, TC-032-012 |
| AC-4 | "`SELECT * FROM DBC.Databases` displays 8+ columns in 117-char terminal" | sprint-32-planning.md §66 | **Manual ONLY** | BLOCKING - visual column density at specific terminal width | **MANUAL-1** |
| AC-5 | "Existing table formatting tests still pass" | sprint-32-planning.md §67 | Integration | Regression validation - run existing test suite | TC-032-013 |
| AC-6 | "Manual validation: REPL query shows improved density" | sprint-32-planning.md §68 | **Manual ONLY** | BLOCKING - human verifies visual improvement in REPL | **MANUAL-1** |
| AC-7 | "No performance regression (measure column calc time)" | sprint-32-planning.md §69 | Benchmark | Explicit performance requirement - criterion benchmarks | BENCH-032-001 |
| AC-8 | "Works correctly with NULL values" | sprint-32-planning.md §70 | Unit + Integration + Manual | Unit validates logic, integration validates formatting, manual validates alignment | TC-032-004, TC-032-014, MANUAL-1 |
| AC-9 | "Works correctly with numeric alignment" | sprint-32-planning.md §71 | Unit + Integration + Manual | Unit validates logic, integration validates formatting, manual validates alignment | TC-032-005, TC-032-015, MANUAL-1 |
| AC-10 | "Documentation updated with new column width behavior" | sprint-32-planning.md §72 | Manual Review | Documentation review - verify specs updated | DOC-REVIEW |

**Coverage Validation:**
- ✅ Every specification requirement (10 ACs) appears in table
- ✅ Every requirement maps to at least one test type
- ✅ AC-4 and AC-6 map to MANDATORY manual validation (Type 4 feature)
- ✅ No orphaned requirements (all have test coverage)
- ✅ Type 4 classification drives manual validation requirement

**Coverage Gaps:**
- None identified - all 10 acceptance criteria have explicit test coverage
- AC-4 and AC-6 correctly classified as manual-only (Type 4 feature characteristic)

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Interactive Tests (expectrl)**
- **Reason for omission:** Manual validation is more effective for Type 4 visual features. Interactive tests could verify column count but NOT visual density or alignment quality. Manual testing at specific terminal widths (80, 117, 120, 160) provides higher value.
- **What won't be validated:** Automated PTY-based column count verification
- **Risk assessment:** LOW - Manual validation covers the same ground more effectively. Unit + integration tests validate logic correctness. The critical validation is VISUAL, which only manual testing can provide.
- **Mitigation:** Comprehensive manual validation protocol with evidence capture (script command). Multiple terminal widths tested. Visual alignment explicitly verified.
- **Revisit criteria:** If manual validation becomes too time-consuming or if we need CI automation (but would still require manual approval for Type 4 features)

**Cross-Platform Tests (Windows/macOS/Linux)**
- **Reason for omission:** Content-based width calculation is platform-agnostic (string length, integer math). Terminal width detection uses crossterm (already cross-platform tested). No platform-specific behavior.
- **What won't be validated:** Platform-specific terminal width detection edge cases
- **Risk assessment:** LOW - Logic is pure Rust (string operations), crossterm handles platform differences, existing table formatter works cross-platform
- **Mitigation:** Development testing on macOS, CI on Linux, community testing during release
- **Revisit criteria:** If platform-specific bugs reported in column width calculation

#### 6. Test Implementation Plan

**For each REQUIRED test type, document implementation approach:**

**Test Type: Unit Tests**
- **Location:** `src/format/table.rs` test module (inline `#[cfg(test)]`)
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 15-20 tests
- **Key scenarios to cover:**
  1. `calculate_column_width()` with short content (10 chars) in VARCHAR(64) column
  2. `calculate_column_width()` with long content (150 chars) - verify 100-char cap
  3. `calculate_column_width()` with header longer than content - verify header length used
  4. `calculate_column_width()` with NULL values - verify "[NULL]" length used
  5. `calculate_column_width()` with numeric values - verify alignment logic preserved
  6. `calculate_column_width()` with empty strings - verify minimum width
  7. `calculate_column_width()` with Unicode characters - verify correct byte vs. char counting
  8. `calculate_column_width()` with mixed content lengths - verify max content length selected
  9. Edge case: Very long content (1000+ chars) - verify cap works, no overflow
  10. Edge case: Zero-length header and content - verify fallback width
  11. Edge case: All NULL column - verify width calculation
  12. Regression: Existing alignment logic tests still pass
  13. Regression: Terminal width detection still works
  14. Regression: Column selection logic still works
  15. Performance: Width calculation completes in reasonable time (<1ms for typical table)
- **Mocking strategy:**
  - Mock `QueryResult` with known column types and content
  - Mock terminal width for width calculation tests
  - No database mocking needed (unit tests don't query database)

**Test Type: Integration Tests**
- **Location:** `tests/integration_tests.rs` or new `tests/table_formatting_integration.rs`
- **Framework:** Built-in Rust integration test support
- **Test count estimate:** 10-12 tests
- **Key scenarios to cover:**
  1. End-to-end: Create `QueryResult` with VARCHAR(64) columns containing 10-char strings, verify table output
  2. End-to-end: Mixed column types (VARCHAR, INTEGER, DATE, NULL), verify table structure correct
  3. Batch mode: Verify all columns shown (no terminal width truncation)
  4. TTY mode: Mock terminal width, verify column selection logic works
  5. Regression: Run existing table formatting tests (`cargo test table`) - verify 100% pass rate
  6. Regression: Verify table borders render correctly
  7. Regression: Verify row count and timing footer still works
  8. NULL handling: Table with NULL values renders correctly
  9. Numeric alignment: Table with numeric columns renders with correct alignment
  10. Large table: 100 rows x 50 columns renders without crash or corruption
  11. Edge case: Single column table renders correctly
  12. Edge case: Single row table renders correctly
- **Setup requirements:** Mock `QueryResult` objects with various data types and content

**Test Type: Performance Benchmark Tests**
- **Location:** `benches/table_formatting.rs` (new file)
- **Framework:** criterion crate
- **Test count estimate:** 3-5 benchmarks
- **Key scenarios to cover:**
  1. **Baseline benchmark**: Current width calculation (schema-based) with 100x20 table
  2. **New benchmark**: Content-based width calculation with 100x20 table (short content)
  3. **Stress benchmark**: Content-based with 1000x50 table (measure scaling)
  4. **Edge benchmark**: Content-based with very long content (200+ char strings)
  5. **Comparison report**: Show percentage change from baseline
- **Implementation notes:**
  - Add criterion to Cargo.toml dev-dependencies
  - Create benchmark harness in `benches/` directory
  - Measure both total table formatting time AND isolated width calculation time
  - Acceptable threshold: <10% regression (or <1ms absolute for 100x20 table)
  - Report results in test execution output

**Test Type: Manual Validation - MANDATORY**
- **Location:** Human tester in development environment
- **Framework:** Manual REPL testing with evidence capture
- **Test count estimate:** 1 comprehensive test protocol (5 terminal widths + 3 alignment checks)
- **Key scenarios to cover:**
  1. **Primary test - 117-char terminal** (from issue #13):
     - Resize terminal to exactly 117 characters width
     - Execute: `SELECT * FROM DBC.Databases`
     - Capture: `script /tmp/sprint32-test-117.txt` before and after
     - Expected: 8+ columns visible (vs. 2 before)
     - Evidence: Script output showing column headers and improved density
  2. **Narrow terminal - 80-char**:
     - Execute same query in 80-char terminal
     - Verify: More columns visible than before (even if fewer than 117-char)
     - Evidence: Screenshot or script output
  3. **Standard terminal - 120-char**:
     - Execute same query in 120-char terminal
     - Verify: Efficient use of space, no excessive whitespace
  4. **Wide terminal - 160-char**:
     - Execute same query in 160-char terminal
     - Verify: Many columns visible, table looks professional
  5. **Visual alignment check**:
     - Execute: `SELECT * FROM DBC.Databases` (mixed column types)
     - Verify: NULL values ("[NULL]") aligned correctly in columns
     - Verify: Numeric columns (like PermSpace, SpoolSpace) right-aligned
     - Verify: Text columns (like DatabaseName, OwnerName) left-aligned
     - Verify: Column headers aligned with content
  6. **Truncation check**:
     - Create test table with very long VARCHAR(500) containing 200+ char strings
     - Execute: `SELECT * FROM test_table`
     - Verify: Long columns truncated at 100 chars (or configured cap)
     - Verify: Truncation is graceful (no visual corruption)
  7. **Before/After comparison**:
     - Capture table output BEFORE content-based width implementation
     - Capture table output AFTER content-based width implementation
     - Compare side-by-side: count visible columns in each
     - Evidence: Both outputs in sprint test results
- **Implementation notes:**
  - Use `script` command to capture terminal session: `script /tmp/sprint32-manual-test.txt`
  - Test against real Teradata database (from TQ_LOGON in .env)
  - Document all evidence in `tests/results/sprint-32/MANUAL-VALIDATION.md`
  - Sprint coordinator must review evidence before approving sprint closure
  - BLOCKING: If visual improvement not observed, sprint CANNOT be approved

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- **Unit tests validate:** Column width calculation logic correctness, edge cases, NULL handling, numeric alignment logic
- **Integration tests validate:** Table formatting pipeline integration, no structural regressions, batch mode correctness
- **Benchmark tests validate:** Performance does not regress (AC-7 requirement)
- **Manual validation validates:** ACTUAL VISUAL COLUMN DENSITY IMPROVEMENT at user-reported terminal width (AC-4, AC-6)
- **Combined coverage:** COMPREHENSIVE - unit/integration prove correctness, benchmarks prove performance, manual validation proves USER-OBSERVABLE IMPROVEMENT

**Gaps in combined coverage:**
- **Gap 1**: Cross-platform testing (only tested on development platform + CI Linux)
  - **Acceptable because:** Logic is platform-agnostic, crossterm handles platform differences, low risk
- **Gap 2**: Automated PTY column count verification (no expectrl tests)
  - **Acceptable because:** Manual validation is more effective for Type 4 visual features, provides higher-quality validation

**Acceptance criteria:**
- ✅ All 10 specification requirements (ACs) have test coverage
- ✅ AC-4 and AC-6 have MANDATORY manual validation (Type 4 feature)
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted with justification
- ✅ Manual validation protocol is detailed and evidence-based

**CRITICAL: Type 4 Classification Drives Manual Validation Requirement**

Per `docs/testing/approach.md` §Feature Types and Their Testing Limitations (lines 426-436), Type 4 features have:
- **Limitations:** "Alternate screen buffer invisible to test framework, PTY timing differs from real terminal, User interaction sequences not reproducible, Terminal resize behavior untestable"
- **Mitigation:** "Limited PTY tests for state changes + **MANDATORY manual validation**"

Sprint 31 lesson: "quality-validator verdict is ADVISORY for visual features. The sprint coordinator must manually verify before approval." (docs/testing/philosophy.md line 298)

**If gaps exist, document why they're acceptable:**
- **Gap 1 (cross-platform)** is acceptable because: Content-based width calculation is pure string length operations (platform-agnostic), crossterm provides terminal abstraction, development + CI testing covers primary platforms
- **Gap 2 (no expectrl tests)** is acceptable because: Manual validation at multiple terminal widths provides superior verification for Type 4 visual features. Automated PTY tests would verify column count but NOT visual quality or alignment, which is the core user requirement.

---

### Feature 2: Fix GitHub README Display (#12) [P1]

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-32-planning.md` §Feature 2: Fix GitHub README Display (lines 96-130)
- Files: `README.md`, `.github/README.md`
- Related: GitHub Issue #12

**Requirements (4 Acceptance Criteria):**
1. **AC-1**: Root `README.md` displays on GitHub repository landing page
2. **AC-2**: `.github/` directory content remains accessible for GitHub configuration
3. **AC-3**: Solution is GitHub convention-compliant
4. **AC-4**: No broken links or references

**Feature Characteristics:**

**User Interaction Type:** ❌ Not Interactive - **Documentation Fix (File Operation)**

**Explanation:** This is a simple file rename/move operation. No code changes, no logic, no runtime behavior. Solution: Rename `.github/README.md` to `.github/GITHUB_NOTES.md` (or similar). GitHub will default to root `README.md` when no `.github/README.md` exists.

**Observable Behavior:**
- ❌ Visual output in terminal
- ❌ Structured data output
- ✅ **File system side effects** (file renamed/moved)
- ❌ Database side effects
- ❌ Network interactions
- ❌ Performance characteristics
- ❌ State management

**External Dependencies:**
- ❌ Database connection
- ✅ **File system access** (rename operation)
- ❌ Network access
- ❌ Terminal/PTY
- ❌ System clipboard
- ❌ Operating system specific features

**Validation Challenges:**
- **Challenge 1**: Must verify change on actual GitHub repository (not testable in CI without push)
- **Challenge 2**: Must ensure `.github/` content remains accessible (no broken internal links)

**Critical Behaviors to Validate:**
1. **Root README displays** - "Root `README.md` displays on GitHub repository landing page" (AC-1)
2. **No broken links** - "No broken links or references" (AC-4)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "File system side effects" checked:
  → Manual verification REQUIRED
  Reason: Must verify on actual GitHub repository after push

IF "Simple file operation" with NO code changes:
  → Unit tests NOT NEEDED (no code to test)
  → Integration tests NOT NEEDED (no runtime behavior)
  → Manual verification SUFFICIENT
  Reason: File rename is atomic operation, no logic to test
```

**Derived Test Types:**

**Test Type 1: Manual Verification - REQUIRED**
- **Validates:** Root README displays on GitHub, `.github/` content accessible, no broken links
- **Approach:** After file rename and push to GitHub:
  1. Navigate to GitHub repository landing page
  2. Verify root `README.md` content displays (not `.github/README.md`)
  3. Navigate to `.github/` directory on GitHub
  4. Verify renamed file (e.g., `GITHUB_NOTES.md`) is accessible
  5. Check all links in both README files (no 404s)
- **Rationale:** Only way to verify GitHub's file resolution behavior - cannot be tested in CI before push
- **Gap if missing:** Could push broken change that doesn't display README correctly
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ❌ NOT NEEDED | No code changes, no logic to test | N/A | SKIP |
| Integration tests | ❌ NOT NEEDED | No runtime behavior, file operation only | N/A | SKIP |
| Manual verification | ✅ REQUIRED | Must verify on actual GitHub after push | Broken README display, broken links | MUST PERFORM |

**Summary:**
- ✅ REQUIRED test types: 1 (Manual verification after push)
- ❌ NOT NEEDED test types: 2 (Unit, Integration)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| AC-1 | "Root `README.md` displays on GitHub repository landing page" | sprint-32-planning.md §108 | Manual Verification | Must verify on actual GitHub | MANUAL-2 |
| AC-2 | "`.github/` directory content remains accessible for GitHub configuration" | sprint-32-planning.md §109 | Manual Verification | Must verify on actual GitHub | MANUAL-2 |
| AC-3 | "Solution is GitHub convention-compliant" | sprint-32-planning.md §110 | Manual Verification | Verify file naming follows conventions | MANUAL-2 |
| AC-4 | "No broken links or references" | sprint-32-planning.md §111 | Manual Verification | Must verify on actual GitHub | MANUAL-2 |

**Coverage Validation:**
- ✅ All 4 specification requirements have test coverage
- ✅ Manual verification is sufficient (no code changes)

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Unit Tests**
- **Reason for omission:** No code changes - purely file rename/move operation
- **What won't be validated:** N/A (no logic to validate)
- **Risk assessment:** NONE - file rename is atomic operation
- **Mitigation:** Manual verification after push
- **Revisit criteria:** Never (file operation, not code)

**Integration Tests**
- **Reason for omission:** No runtime behavior - documentation change only
- **What won't be validated:** N/A (no runtime behavior)
- **Risk assessment:** NONE - no integration points
- **Mitigation:** Manual verification after push
- **Revisit criteria:** Never (documentation, not code)

#### 6. Test Implementation Plan

**Test Type: Manual Verification**
- **Location:** GitHub repository (after push)
- **Framework:** Human verification in browser
- **Test count estimate:** 1 verification protocol (4 checks)
- **Key scenarios to cover:**
  1. Navigate to https://github.com/user/tq (repository landing page)
  2. Verify root `README.md` content displays (project introduction, not GitHub config)
  3. Navigate to https://github.com/user/tq/tree/master/.github
  4. Verify `.github/` directory accessible and contains renamed file
  5. Click links in root `README.md` - verify no 404s
  6. Click links in `.github/GITHUB_NOTES.md` (or renamed file) - verify no 404s
- **Implementation notes:**
  - Perform after tq-project-manager pushes to GitHub
  - Document verification in sprint review
  - Takes ~2 minutes total
  - No tooling required (just web browser)

#### 7. Coverage Sufficiency Assessment

**Question: If manual verification passes, can we claim the feature "works as specified"?**

**Analysis:**
- **Manual verification validates:** Root README displays on GitHub, `.github/` content accessible, no broken links, GitHub convention compliance
- **Combined coverage:** SUFFICIENT - manual verification is the ONLY necessary validation for documentation file operations

**Gaps in combined coverage:**
- None - manual verification covers all 4 acceptance criteria

**Acceptance criteria:**
- ✅ All 4 specification requirements have test coverage (manual verification)
- ✅ No code changes mean no unit/integration tests needed
- ✅ Manual verification is sufficient for documentation changes

---

## Strategy Summary

**Total Features Analyzed:** 2

**Feature 1: Content-Based Column Width (#13)**
- Type: Type 4 (Visual/Interactive) - MANDATORY manual validation required
- Test Types: Unit (✅), Integration (✅), Benchmark (✅), Manual (✅ MANDATORY)
- Test Count: 25-32 unit tests, 10-12 integration tests, 3-5 benchmarks, 1 comprehensive manual protocol
- Risk: MEDIUM if manual validation skipped (repeat Sprint 29/30 pattern)

**Feature 2: Fix GitHub README Display (#12)**
- Type: Documentation Fix (File Operation)
- Test Types: Manual Verification (✅)
- Test Count: 1 verification protocol (4 checks)
- Risk: LOW (simple file rename, verifiable on GitHub)

**Test Types Required:**
- ✅ Unit tests: Feature 1 (column width calculation logic)
- ✅ Integration tests: Feature 1 (table formatting pipeline)
- ✅ Benchmark tests: Feature 1 (AC-7 performance requirement)
- ✅ **Manual validation: Feature 1 (MANDATORY - Type 4 visual feature)** 🔴 BLOCKING
- ✅ Manual verification: Feature 2 (GitHub README display)

**Estimated Test Count:**
- Unit: 25-32 tests (Feature 1)
- Integration: 10-12 tests (Feature 1)
- Benchmark: 3-5 benchmarks (Feature 1)
- Manual validation: 1 comprehensive protocol (Feature 1) - **BLOCKING**
- Manual verification: 1 protocol (Feature 2)
- **Total automated: 38-49 tests + 3-5 benchmarks**
- **Total manual: 2 protocols (1 MANDATORY, 1 required)**

**Risk Assessment:**
- **HIGH risk gaps**: None (if manual validation performed)
- **MEDIUM risk gaps**: Skipping manual validation for Feature 1 (would repeat Sprint 29/30 false success pattern)
- **LOW risk gaps**: None

**Dependencies Required:**
- Live database: ⚠️ RECOMMENDED (for realistic integration testing with DBC.Databases)
- Terminal access: ✅ YES (for manual validation at multiple widths: 80, 117, 120, 160 chars)
- GitHub access: ✅ YES (for Feature 2 manual verification after push)

**Tool Requirements:**
- Existing tools: Built-in Rust test framework, criterion (add to dev-dependencies)
- New tools: None required
- Manual tools: `script` command for evidence capture, terminal resizing

**Quality Validator Role:**
- Design and implement automated tests (unit, integration, benchmark)
- Execute automated test suite
- Generate test report with pass/fail results
- **Provide ADVISORY verdict for Feature 1** (Type 4 - manual validation required)
- Note in report: "Manual validation REQUIRED before sprint closure"
- Sprint coordinator performs manual validation and makes final approval decision

---

## Critical Sprint 31 Lessons Applied

### Type 4 Feature Classification

**Feature #13 is Type 4 per `docs/testing/approach.md` §Feature Types:**

From docs/testing/approach.md lines 426-436:
> #### Type 4: Interactive/Alternate Screen (Minimal Automated Coverage)
>
> Features: Pager, full-screen modes, interactive navigation
>
> **Limitations:**
> - Alternate screen buffer invisible to test framework
> - PTY timing differs from real terminal
> - User interaction sequences not reproducible
> - Terminal resize behavior untestable
>
> **Mitigation:** Limited PTY tests for state changes + **MANDATORY manual validation**

**Feature #13 exhibits Type 4 characteristics:**
- Visual table rendering (terminal output)
- Terminal width-dependent behavior (80, 117, 120, 160 chars)
- User-observable density improvement (core requirement)
- Alignment and truncation (visual quality)

### Manual Validation is MANDATORY

From docs/testing/philosophy.md lines 263-275:
> ### When Manual Validation is Mandatory
>
> Manual validation is **REQUIRED** (not optional) for:
>
> 1. **Pager/alternate screen features** - Cannot capture alternate buffer content
> 2. **Terminal width-dependent rendering** - Must test at multiple actual widths
> 3. **Visual formatting** - Table alignment, column widths, borders
> 4. **Interactive navigation** - Real terminal input/response cycles
> 5. **Any feature where automated tests cannot capture actual output**

**Feature #13 requires manual validation because:**
- Terminal width-dependent (user reported 117-char issue)
- Visual formatting (column density, alignment)
- Table rendering (core user-observable behavior)

### Quality Validator Verdict is ADVISORY

From docs/testing/philosophy.md line 298:
> **quality-validator verdict is ADVISORY for visual features.** The sprint coordinator must manually verify before approval.

**Sprint 32 Process:**
1. quality-validator implements and executes automated tests
2. quality-validator generates test report with ADVISORY verdict
3. quality-validator notes: "Manual validation REQUIRED for AC-4 and AC-6"
4. Sprint coordinator performs manual validation (REPL testing at multiple terminal widths)
5. Sprint coordinator captures evidence (script command output)
6. Sprint coordinator makes final approval decision based on manual validation results

### Evidence Capture Required

From sprint-32-planning.md lines 216-219:
> - Evidence required: script command output capture
> - Sprint blocked if manual validation reveals issues

**Evidence protocol:**
1. Use `script /tmp/sprint32-manual-validation.txt` to capture terminal session
2. Test at terminal widths: 80, 117, 120, 160 characters
3. Execute: `SELECT * FROM DBC.Databases` at each width
4. Capture before/after comparison (if available)
5. Document in `tests/results/sprint-32/MANUAL-VALIDATION.md`
6. Sprint coordinator reviews evidence before approval

---

## Strategy Validation Checklist

**Before submitting to sprint-coordinator for execution:**

- ✅ Every feature has complete specification analysis section
- ✅ Feature #13 classified as Type 4 (visual/interactive) per Sprint 31 lessons
- ✅ Feature characteristics are classified (not assumed)
- ✅ Test strategy is derived from characteristics (decision tree applied)
- ✅ Every test type has clear rationale (justified by requirements)
- ✅ Gap analysis is complete and honest (2 low-risk gaps documented)
- ✅ Specification coverage map includes all requirements (10 ACs for Feature 1, 4 ACs for Feature 2)
- ✅ Every requirement maps to at least one test type (AC-4 and AC-6 map to MANDATORY manual validation)
- ✅ Test implementation plan is detailed and actionable (framework, location, scenario count, evidence capture)
- ✅ Coverage sufficiency is assessed (automated + manual = comprehensive)
- ✅ No hand-waving or vague justifications (specific test scenarios, terminal widths, evidence protocol)
- ✅ Sprint 31 lessons explicitly applied (Type 4 classification, MANDATORY manual validation, ADVISORY verdict)
- ✅ Manual validation protocol is detailed with evidence requirements

**All checkboxes checked** - Strategy is complete and ready for execution.

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-02-03
**Review Status:** READY FOR EXECUTION
**Sprint:** 32

**Key Decisions:**
- Feature #13: Type 4 classification → MANDATORY manual validation required
- Quality validator verdict: ADVISORY (not blocking)
- Sprint coordinator: Performs manual validation and makes final approval decision
- Evidence required: Script command output showing column density improvement at 117-char terminal width

**Approval means:**
- ✅ Test strategy derived from specifications and Sprint 31 lessons
- ✅ Type 4 feature correctly classified with manual validation requirement
- ✅ All required test types identified with clear rationale (Unit, Integration, Benchmark, Manual)
- ✅ Coverage gaps explicitly identified and assessed (2 low-risk gaps)
- ✅ Implementation plan is detailed and achievable (38-49 tests + 2 manual protocols)
- ✅ Ready to proceed with test implementation and execution

**Framework Improvements Applied:**
- Sprint 31 testing philosophy integrated
- Type 4 feature classification applied
- Manual validation protocol detailed with evidence requirements
- Clear distinction between ADVISORY (automated tests) and BLOCKING (manual validation) verdicts
