# Sprint 27 Test Strategy - Bug Fix and Documentation

**Created:** 2026-01-27
**Author:** quality-validator
**Sprint:** Sprint 27
**Features:** Bug fix for /sessions command (#10), LICENSE file validation (#8), README validation (#9)

---

## Feature-by-Feature Test Strategy

### Feature 1: Bug Fix - /sessions Command Incorrect Session Count (#10)

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/specifications/repl.md` sections REQ-SESS-001 through REQ-SESS-008
- Design: `docs/design/repl.md` Sessions Command section
- Issue: GitHub Issue #10 - User reproduction case showing 3 sessions exist but only 2 displayed
- Test Cases: Existing Sprint 26 test cases TC-SESS-001 through TC-SESS-010

**Bug Description:**
User reported that `/sessions` command returns 2 sessions when 3 actually exist. User provided comparison:
- Direct SQL query to MonitorSession(-1,'*',0) returns 3 rows (SessionNo 1230, 1231, 1232)
- `/sessions` command returns only 2 rows (SessionNo 1230, 1231 - missing 1232)
- Missing session is in DISPATCHING/ACTIVE state (actively running query)

**Requirements:**
1. REQ-SESS-002.1: "Data source: MonitorSession(-1,'*',0) table function" (repl.md:1560)
2. REQ-SESS-003.1: "Default output format: Table (box-drawing characters)" (repl.md:1584)
3. AC-3: "Output displays 10 columns: SessionNo, UserName, LogonTime, PEstate, AMPState, AMPCPUSec, AMPIO, ReqSpool, Amp CPU Skew %, Amp IO Skew %" (sprint-26-planning.md:108)
4. Bug Fix Requirement: "All 3 sessions from user example are displayed correctly" (sprint-27-planning.md:85)
5. Bug Fix Requirement: "Regression test added to prevent recurrence" (sprint-27-planning.md:86)
6. Bug Fix Requirement: "All existing tests pass (no regressions)" (sprint-27-planning.md:87)

**Feature Characteristics:**

**User Interaction Type:**
- [x] Interactive PTY (REPL, terminal UI with cursor/colors/rendering)
- [x] CLI Batch (scripted, piped, non-interactive command execution)
- [x] Pure Logic (internal algorithm, data filtering, row processing)

**Explanation:**
This is a BUG FIX in an existing feature that operates in three modes:
1. **Interactive PTY**: `/sessions` in REPL displays table in terminal
2. **CLI Batch**: `tq sessions` as standalone command
3. **Pure Logic**: Bug likely in row filtering/processing logic (missing row during parsing or filtering)

**Observable Behavior:**
- [x] Visual output in terminal (colors, formatting, layout)
- [x] Structured data output (JSON, CSV, table formats)
- [x] Database side effects (READ ONLY - query execution)

**External Dependencies:**
- [x] Database connection (requires live database)
- [x] Terminal/PTY (terminal control sequences, cursor positioning)

**Validation Challenges:**
1. **Bug reproduction**: Need to reproduce exact user scenario (3 sessions with specific states: IDLE, IDLE, DISPATCHING/ACTIVE)
2. **Root cause isolation**: Must identify why one session is filtered out (parsing error? state filtering? skew calculation bug?)
3. **Regression prevention**: Existing Sprint 26 tests passed, yet bug exists - need more comprehensive test coverage
4. **Session state variety**: Must test all combinations of PEState and AMPState (IDLE/IDLE, IDLE/ACTIVE, DISPATCHING/ACTIVE)
5. **Non-deterministic environment**: Cannot reliably create exact session states on demand

**Critical Behaviors to Validate:**
1. "All sessions returned by MonitorSession(-1,'*',0) SHALL be displayed in /sessions output" - Core requirement
2. "No filtering SHALL be applied to sessions based on state (IDLE, DISPATCHING, ACTIVE)" - Bug hypothesis
3. "Row count in /sessions footer SHALL match actual session count from database" - Verification mechanism
4. "Sessions in all states (IDLE/IDLE, IDLE/ACTIVE, DISPATCHING/ACTIVE) SHALL be displayed" - State coverage
5. "Existing Sprint 26 tests (TC-SESS-001 through TC-SESS-010) SHALL continue to pass" - No regression

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Bug fix in existing feature":
  → Regression tests REQUIRED
  Reason: Must verify bug is fixed AND existing functionality still works

IF "Interactive PTY" checked:
  → Interactive tests (expectrl) REQUIRED
  Reason: Bug affects REPL /sessions command output

IF "CLI Batch" checked:
  → Integration tests REQUIRED
  Reason: Bug may also affect batch mode `tq sessions`

IF "Pure Logic" (potential filtering/parsing bug):
  → Unit tests REQUIRED
  Reason: Isolate root cause in row processing logic

IF "Database connection" checked:
  → Integration tests with live database REQUIRED
  Reason: Need real MonitorSession data to reproduce bug
```

**Derived Test Types:**

**Test Type 1: Bug Reproduction Test**
- **Validates:** User's exact scenario - 3 sessions with states IDLE, IDLE, DISPATCHING/ACTIVE all appear
- **Approach:** Interactive test that queries MonitorSession directly, then executes /sessions, compares row counts
- **Rationale:** Must prove bug is fixed with user's exact scenario (not just "works for me" scenario)
- **Gap if missing:** Bug may still exist in edge cases not covered by general tests
- **Necessity:** ✅ REQUIRED

**Test Type 2: Regression Tests (Existing Sprint 26 Tests)**
- **Validates:** No regression in existing functionality (Sprint 26 features still work)
- **Approach:** Re-run all TC-SESS-001 through TC-SESS-010 tests from Sprint 26
- **Rationale:** Bug fix must not break existing functionality
- **Gap if missing:** Risk of fixing one bug but introducing new bugs
- **Necessity:** ✅ REQUIRED

**Test Type 3: Session State Coverage Tests**
- **Validates:** All combinations of PEState and AMPState are displayed correctly
- **Approach:** Unit tests with mock data covering all state combinations
- **Rationale:** Bug may be state-specific (e.g., DISPATCHING sessions filtered out)
- **Gap if missing:** State-specific bugs may go undetected
- **Necessity:** ✅ REQUIRED

**Test Type 4: Row Count Validation Tests**
- **Validates:** Session count in footer matches actual database row count
- **Approach:** Integration test that queries MonitorSession, counts rows, compares to /sessions footer
- **Rationale:** Provides automated verification that no rows are lost
- **Gap if missing:** Cannot automatically detect missing rows
- **Necessity:** ✅ REQUIRED

**Test Type 5: Unit Tests for Row Processing Logic**
- **Validates:** SessionInfo::from_row() correctly parses all valid row formats
- **Approach:** Unit tests with mock Value arrays representing different session states
- **Rationale:** Isolate root cause in parsing logic independent of database
- **Gap if missing:** Cannot identify if bug is in parsing vs filtering vs display
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Bug reproduction test | ✅ REQUIRED | Must verify user's exact scenario (3 sessions, specific states) | Bug may still exist in production scenario | MUST IMPLEMENT |
| Regression tests (Sprint 26) | ✅ REQUIRED | Ensure bug fix doesn't break existing functionality | Risk of introducing new bugs | MUST RE-RUN |
| Session state coverage tests | ✅ REQUIRED | Verify all state combinations work (bug may be state-specific) | State-specific bugs undetected | MUST IMPLEMENT |
| Row count validation tests | ✅ REQUIRED | Automated verification that no rows lost | Cannot detect missing rows automatically | MUST IMPLEMENT |
| Unit tests for row processing | ✅ REQUIRED | Isolate root cause in parsing/filtering logic | Cannot identify bug location | MUST IMPLEMENT |
| Manual verification | ⚠️ RECOMMENDED | Human validates fix with real database workload | Usability issues, edge cases | DOCUMENT TEST CASES |

**Summary:**
- ✅ REQUIRED test types: 5 (bug repro, regression, state coverage, row count, unit)
- ⚠️ RECOMMENDED test types: 1 (manual verification)
- ❌ NOT NEEDED test types: 0

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| BUG-FIX-001 | "All 3 sessions from user example are displayed correctly" | sprint-27-planning.md:85 | Bug Repro + Integration | Must reproduce user scenario | TC-SESS-BUG-001 |
| BUG-FIX-002 | "Regression test added to prevent recurrence" | sprint-27-planning.md:86 | State Coverage + Row Count | Prevent future regressions | TC-SESS-BUG-002 |
| BUG-FIX-003 | "All existing tests pass (no regressions)" | sprint-27-planning.md:87 | Regression (Sprint 26) | No functionality broken | TC-SESS-001 to 010 |
| REQ-SESS-002.1 | "Data source: MonitorSession(-1,'*',0)" | repl.md:1560 | Integration + Bug Repro | Verify all rows returned | TC-SESS-BUG-001 |
| BUG-ROOT-001 | "No filtering applied based on session state" | Bug hypothesis | State Coverage + Unit | Verify IDLE/DISPATCHING/ACTIVE all work | TC-SESS-BUG-003 |
| BUG-ROOT-002 | "Row parsing handles all valid formats" | Bug hypothesis | Unit | Verify SessionInfo::from_row() correct | TC-SESS-BUG-004 |
| BUG-ROOT-003 | "Session count in footer matches database count" | Bug hypothesis | Row Count Validation | Automated row loss detection | TC-SESS-BUG-002 |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements (missing test coverage)
- [x] No unjustified test types (test types without requirement rationale)

**Coverage Gaps:**
- **Root cause unknown**: Until bug is analyzed, tests are based on hypothesis. May need to adjust test strategy after root cause is identified.
- **Non-deterministic session states**: Cannot reliably create specific session states (IDLE vs ACTIVE) on demand. Will use existing sessions or mock data.

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Performance/Benchmark Tests**
- **Reason for omission:** Bug fix does not change performance characteristics
- **What won't be validated:** Query execution speed, memory usage
- **Risk assessment:** LOW
- **Mitigation:** Bug fix should not impact performance. Manual observation during testing.
- **Revisit criteria:** If performance regression is reported after bug fix

**Cross-platform Tests**
- **Reason for omission:** Bug is database query logic, not OS-specific
- **Risk assessment:** LOW
- **Mitigation:** Bug fix is in row processing logic (platform-independent)
- **Revisit criteria:** If platform-specific bugs are reported

#### 6. Test Implementation Plan

**Test Type: Bug Reproduction Test**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 1 test
- **Key scenarios to cover:**
  1. `test_sessions_bug_fix_three_sessions` - Execute /sessions, verify 3+ sessions displayed (matches user scenario)
- **Implementation notes:**
  - Query MonitorSession directly first: `SELECT COUNT(*) FROM TABLE(MonitorSession(-1,'*',0))`
  - Execute `/sessions` command
  - Compare row count in footer to actual database count
  - Verify all session states present (IDLE, DISPATCHING, ACTIVE)

**Test Type: Regression Tests (Sprint 26)**
- **Location:** `tests/cases/TC-SESS-001.md` through `TC-SESS-010.md`
- **Framework:** Existing test infrastructure (PTY, integration, unit)
- **Test count estimate:** 10 tests (existing)
- **Key scenarios to cover:**
  - Re-run all Sprint 26 tests without modification
  - Verify 100% pass rate
- **Setup requirements:** Live database connection via TQ_LOGON

**Test Type: Session State Coverage Tests**
- **Location:** `src/commands/sessions.rs` test module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 6 tests
- **Key scenarios to cover:**
  1. `test_session_info_from_row_idle_idle` - PEState=IDLE, AMPState=IDLE
  2. `test_session_info_from_row_idle_active` - PEState=IDLE, AMPState=ACTIVE
  3. `test_session_info_from_row_dispatching_idle` - PEState=DISPATCHING, AMPState=IDLE
  4. `test_session_info_from_row_dispatching_active` - PEState=DISPATCHING, AMPState=ACTIVE (user's missing session)
  5. `test_session_info_from_row_active_idle` - PEState=ACTIVE, AMPState=IDLE
  6. `test_session_info_from_row_active_active` - PEState=ACTIVE, AMPState=ACTIVE
- **Mocking strategy:** Create mock Value arrays for each state combination

**Test Type: Row Count Validation Tests**
- **Location:** `tests/integration_tests.rs`
- **Framework:** Built-in Rust integration test support
- **Test count estimate:** 2 tests
- **Key scenarios to cover:**
  1. `test_sessions_row_count_matches_database` - Count rows in MonitorSession, compare to /sessions output
  2. `test_sessions_footer_count_accurate` - Parse footer "N sessions found", verify matches actual row count
- **Setup requirements:** Live database connection via TQ_LOGON. Mark tests with `#[ignore]`.

**Test Type: Unit Tests for Row Processing Logic**
- **Location:** `src/commands/sessions.rs` test module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 4 tests
- **Key scenarios to cover:**
  1. `test_session_info_from_row_valid_complete` - All fields present and valid
  2. `test_session_info_from_row_valid_nulls` - Optional fields NULL (skew percentages)
  3. `test_session_info_from_row_invalid_short` - Row too short (< 12 columns)
  4. `test_session_info_from_row_invalid_types` - Wrong Value types in row
- **Mocking strategy:** Create mock Value arrays representing various row formats

**Test Type: Manual Verification**
- **Location:** `tests/cases/TC-SESS-BUG-001-MANUAL.md`
- **Framework:** Human tester
- **Test count estimate:** 1 manual test case
- **Key scenarios to cover:**
  1. Run user's exact SQL query: `SELECT * FROM TABLE(MonitorSession(-1,'*',0))`
  2. Count rows manually
  3. Run `/sessions` command
  4. Compare row counts and verify all sessions present
  5. Verify session states match (IDLE, DISPATCHING, ACTIVE)

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the bug is fixed and feature "works as specified"?**

**Analysis:**
- Bug reproduction test validates: User's exact scenario (3+ sessions with varied states) works
- Regression tests validate: Existing Sprint 26 functionality not broken
- State coverage tests validate: All state combinations (IDLE/DISPATCHING/ACTIVE) handled correctly
- Row count validation tests validate: Automated detection of missing rows
- Unit tests validate: Row processing logic handles all valid formats
- Manual verification validates: Real-world usage confirms fix
- Combined coverage: COMPREHENSIVE for bug fix

**Gaps in combined coverage:**
- Root cause unknown until bug is analyzed - tests may need adjustment after analysis
- Cannot create specific session states on demand - rely on existing sessions or mocks
- Non-deterministic session list - tests verify count matches, not specific sessions

**Acceptance criteria:**
- [x] User's bug scenario has dedicated test (bug reproduction)
- [x] All state combinations tested (state coverage)
- [x] Automated row count verification (row count validation)
- [x] Regression tests pass (Sprint 26 tests)
- [x] Root cause isolated and tested (unit tests)
- [x] Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Non-deterministic session states acceptable because: Tests verify count and state variety, not specific session IDs
- Root cause uncertainty acceptable because: Comprehensive test coverage addresses multiple hypotheses
- Cannot reproduce exact user environment acceptable because: Tests cover the general pattern (3+ sessions, varied states)

---

### Feature 2: LICENSE File Validation (#8)

#### 1. Specification Analysis

**Specification References:**
- Primary: GitHub Issue #8 - LICENSE: Proper licensing
- Current: LICENSE file contains only MIT license (incomplete)
- Required: Attribution for teradatarustapi dependencies
- External: https://github.com/Teradata/teradatarustapi/blob/main/LICENSE
- External: https://github.com/Teradata/teradatarustapi/blob/main/THIRDPARTYLICENSE

**Requirements:**
1. AC-LICENSE-001: "LICENSE file updated with complete terms" (sprint-27-planning.md:92)
2. AC-LICENSE-002: "teradatarustapi license attribution included" (sprint-27-planning.md:93)
3. AC-LICENSE-003: "Go license attribution included" (sprint-27-planning.md:94)
4. AC-LICENSE-004: "NOTICE or THIRD-PARTY-LICENSES file created if needed" (sprint-27-planning.md:95)
5. AC-LICENSE-005: "README licensing section added" (sprint-27-planning.md:96)
6. AC-LICENSE-006: "Compliance with Teradata redistribution terms verified" (sprint-27-planning.md:97)

**Feature Characteristics:**

**User Interaction Type:**
- [ ] Interactive PTY
- [ ] CLI Batch
- [ ] Web UI
- [ ] API
- [ ] Background Process
- [x] Pure Logic (documentation file validation)

**Explanation:**
This is DOCUMENTATION VALIDATION, not a runtime feature. Validation is:
1. File exists and is readable
2. Required attribution text is present
3. License terms are complete and accurate
4. Compliance with third-party redistribution terms

**Observable Behavior:**
- [x] File system side effects (LICENSE file created/modified)

**External Dependencies:**
- [ ] Database connection
- [x] File system access (reads/writes files)
- [ ] Network access (for fetching upstream license references)
- [ ] Terminal/PTY
- [ ] System clipboard
- [ ] Operating system specific features
- [ ] None

**Validation Challenges:**
1. **Legal compliance**: Not a legal expert - must follow established patterns from similar projects
2. **Attribution completeness**: Must identify ALL third-party dependencies that require attribution
3. **License compatibility**: MIT + Teradata licenses must be compatible
4. **Redistribution terms**: Must understand Teradata's redistribution requirements
5. **Format validation**: No standard format for multi-license files

**Critical Behaviors to Validate:**
1. "LICENSE file contains MIT license for tq project" - Base requirement
2. "LICENSE file contains teradatarustapi attribution with correct terms" - Third-party requirement
3. "LICENSE file contains Go language license attribution (from teradatarustapi THIRDPARTYLICENSE)" - Transitive dependency
4. "All attributions are complete (copyright holders, year, license text)" - Completeness
5. "No misleading claims (e.g., 'MIT only' when dependencies have different licenses)" - Accuracy

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "File system side effects":
  → File validation tests REQUIRED
  Reason: Need to verify file exists, is readable, contains required text

IF "Documentation validation":
  → Content validation tests REQUIRED
  Reason: Must verify specific attribution text is present

IF "Legal compliance":
  → Manual review REQUIRED
  Reason: Automated tests cannot validate legal correctness
```

**Derived Test Types:**

**Test Type 1: File Existence and Readability Tests**
- **Validates:** LICENSE file exists and is readable
- **Approach:** Automated test reads LICENSE file from disk
- **Rationale:** Basic smoke test - file must exist
- **Gap if missing:** Broken builds where LICENSE file is missing
- **Necessity:** ✅ REQUIRED

**Test Type 2: Content Validation Tests**
- **Validates:** Required attribution text is present
- **Approach:** Automated test searches LICENSE file for specific strings
- **Rationale:** Ensures all required attributions are included
- **Gap if missing:** Incomplete attributions, missing licenses
- **Necessity:** ✅ REQUIRED

**Test Type 3: License Text Completeness Tests**
- **Validates:** License text is complete (not truncated, no placeholders)
- **Approach:** Automated test checks for common placeholders ([YEAR], <COPYRIGHT HOLDER>)
- **Rationale:** Ensures LICENSE file is production-ready
- **Gap if missing:** Incomplete/invalid license text
- **Necessity:** ✅ REQUIRED

**Test Type 4: Manual Legal Review**
- **Validates:** Legal correctness, license compatibility, redistribution compliance
- **Approach:** Human reviewer (with legal knowledge) reviews LICENSE file
- **Rationale:** Automated tests cannot validate legal correctness
- **Gap if missing:** Legal compliance issues, license conflicts
- **Necessity:** ⚠️ RECOMMENDED (blocking for production release)

**Test Type 5: README Integration Tests**
- **Validates:** README contains licensing section with link to LICENSE file
- **Approach:** Automated test searches README for licensing section
- **Rationale:** Users should be informed about licensing in README
- **Gap if missing:** Users unaware of license terms
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| File existence tests | ✅ REQUIRED | Verify LICENSE file exists and is readable | Broken builds, missing file | MUST IMPLEMENT |
| Content validation tests | ✅ REQUIRED | Verify required attributions present | Incomplete attributions | MUST IMPLEMENT |
| License completeness tests | ✅ REQUIRED | Verify no placeholders, text complete | Invalid license text | MUST IMPLEMENT |
| Manual legal review | ⚠️ RECOMMENDED | Validate legal correctness, compliance | Legal issues, license conflicts | BLOCKING FOR RELEASE |
| README integration tests | ✅ REQUIRED | Verify README links to LICENSE | Users unaware of terms | MUST IMPLEMENT |

**Summary:**
- ✅ REQUIRED test types: 4 (existence, content, completeness, README)
- ⚠️ RECOMMENDED test types: 1 (legal review - BLOCKING)
- ❌ NOT NEEDED test types: 0

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| AC-LICENSE-001 | "LICENSE file updated with complete terms" | sprint-27-planning.md:92 | Existence + Completeness | File must exist with complete text | TC-LICENSE-001 |
| AC-LICENSE-002 | "teradatarustapi license attribution included" | sprint-27-planning.md:93 | Content Validation | Specific attribution required | TC-LICENSE-002 |
| AC-LICENSE-003 | "Go license attribution included" | sprint-27-planning.md:94 | Content Validation | Transitive dependency attribution | TC-LICENSE-002 |
| AC-LICENSE-004 | "NOTICE or THIRD-PARTY-LICENSES file created if needed" | sprint-27-planning.md:95 | File Existence | May need separate file | TC-LICENSE-003 |
| AC-LICENSE-005 | "README licensing section added" | sprint-27-planning.md:96 | README Integration | User awareness | TC-LICENSE-004 |
| AC-LICENSE-006 | "Compliance with Teradata redistribution terms verified" | sprint-27-planning.md:97 | Manual Legal Review | Legal expertise required | TC-LICENSE-MANUAL |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements (missing test coverage)
- [x] No unjustified test types

**Coverage Gaps:**
- **Legal expertise**: No automated test can validate legal compliance. Requires manual review by someone with legal knowledge.
- **License compatibility**: No automated test can verify MIT + Teradata licenses are compatible. Requires manual analysis.

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Automated Legal Compliance Tests**
- **Reason for omission:** Legal compliance requires human expertise, not automatable
- **What won't be validated:** License compatibility, redistribution terms correctness, legal conflicts
- **Risk assessment:** MEDIUM
- **Mitigation:** Manual legal review required before release
- **Revisit criteria:** If legal automation tools become available

**License Format Validation Tests**
- **Reason for omission:** No standard format for multi-license files
- **What won't be validated:** Specific formatting, section ordering, visual presentation
- **Risk assessment:** LOW
- **Mitigation:** Follow established patterns from similar projects (Rust, Go projects with third-party deps)
- **Revisit criteria:** If standard format emerges

#### 6. Test Implementation Plan

**Test Type: File Existence and Readability Tests**
- **Location:** `tests/integration_tests.rs`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 1 test
- **Key scenarios to cover:**
  1. `test_license_file_exists` - Verify LICENSE file exists at project root
- **Implementation notes:** Use `std::fs::metadata()` and `std::fs::read_to_string()`

**Test Type: Content Validation Tests**
- **Location:** `tests/integration_tests.rs`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 3 tests
- **Key scenarios to cover:**
  1. `test_license_contains_mit` - Verify MIT license text present
  2. `test_license_contains_teradatarustapi_attribution` - Verify teradatarustapi attribution
  3. `test_license_contains_go_attribution` - Verify Go license attribution
- **Implementation notes:** Search for specific strings (e.g., "MIT License", "teradatarustapi", "Go Authors")

**Test Type: License Text Completeness Tests**
- **Location:** `tests/integration_tests.rs`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 2 tests
- **Key scenarios to cover:**
  1. `test_license_no_placeholders` - Verify no [YEAR], <COPYRIGHT HOLDER> placeholders
  2. `test_license_no_todos` - Verify no TODO, FIXME, XXX comments
- **Implementation notes:** Regex search for placeholder patterns

**Test Type: Manual Legal Review**
- **Location:** `tests/cases/TC-LICENSE-MANUAL.md`
- **Framework:** Human reviewer
- **Test count estimate:** 1 manual review
- **Key scenarios to cover:**
  1. Verify MIT license text is complete and accurate
  2. Verify teradatarustapi attribution matches upstream LICENSE
  3. Verify Go license attribution matches teradatarustapi THIRDPARTYLICENSE
  4. Verify no license conflicts (MIT compatible with Teradata terms)
  5. Verify redistribution terms are met

**Test Type: README Integration Tests**
- **Location:** `tests/integration_tests.rs`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 1 test
- **Key scenarios to cover:**
  1. `test_readme_contains_licensing_section` - Verify README has "License" or "Licensing" section
- **Implementation notes:** Search README.md for licensing section header

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the LICENSE file is "compliant and complete"?**

**Analysis:**
- File existence tests validate: LICENSE file exists and is readable
- Content validation tests validate: Required attributions (MIT, teradatarustapi, Go) are present
- Completeness tests validate: No placeholders, text is complete
- Manual legal review validates: Legal correctness, compatibility, compliance
- README integration tests validate: Users are informed about licensing
- Combined coverage: ADEQUATE for documentation validation

**Gaps in combined coverage:**
- Legal expertise required - automated tests cannot validate legal correctness
- License compatibility requires manual analysis
- No format validation (but low risk)

**Acceptance criteria:**
- [x] All required attributions have automated tests
- [x] File existence verified automatically
- [x] Manual legal review is documented as required
- [x] README integration is tested
- [x] Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Legal expertise gap acceptable because: Manual review is explicitly required before release
- Format gap acceptable because: No standard format exists, following established patterns is sufficient
- License compatibility gap acceptable because: Manual review will verify compatibility

---

### Feature 3: README Validation (#9)

#### 1. Specification Analysis

**Specification References:**
- Primary: GitHub Issue #9 - README: User-focused documentation
- Current: README starts with "GitHub Configuration" (developer-focused)
- Required: User-focused TLDR format with screenshot

**Requirements:**
1. AC-README-001: "TLDR introduction section (What/Visual/Quick Start)" (sprint-27-planning.md:100)
2. AC-README-002: "AI-agent development story section (tongue-in-cheek tone)" (sprint-27-planning.md:101)
3. AC-README-003: "Screenshot of tq in action included" (sprint-27-planning.md:102)
4. AC-README-004: "Installation instructions clear and concise" (sprint-27-planning.md:103)
5. AC-README-005: "Links to roadmap and documentation" (sprint-27-planning.md:104)
6. AC-README-006: "Professional tone suitable for public project" (sprint-27-planning.md:105)
7. AC-README-007: "GitHub Configuration section moved to appropriate location" (sprint-27-planning.md:106)

**Feature Characteristics:**

**User Interaction Type:**
- [ ] Interactive PTY
- [ ] CLI Batch
- [ ] Web UI
- [ ] API
- [ ] Background Process
- [x] Pure Logic (documentation file validation)

**Explanation:**
This is DOCUMENTATION VALIDATION for user-facing README. Validation is:
1. File structure follows best practices (TLDR format)
2. Required sections are present
3. Screenshot is included and valid
4. Links are valid and functional
5. Tone is appropriate (professional yet tongue-in-cheek)

**Observable Behavior:**
- [x] File system side effects (README.md file modified, screenshot added)

**External Dependencies:**
- [ ] Database connection
- [x] File system access (reads/writes files, includes image)
- [ ] Network access (for link validation - optional)
- [ ] Terminal/PTY
- [ ] System clipboard
- [ ] Operating system specific features
- [ ] None

**Validation Challenges:**
1. **Tone validation**: Cannot automatically validate "professional" or "tongue-in-cheek" tone
2. **Screenshot quality**: Cannot automatically validate screenshot is "good" or "useful"
3. **Link validation**: Links may be valid today but break later (requires periodic re-validation)
4. **Subjective quality**: "Clear and concise" is subjective - requires human judgment
5. **First impression**: Cannot automatically validate README gives "good first impression"

**Critical Behaviors to Validate:**
1. "README starts with TLDR section (not developer configuration)" - Structure requirement
2. "Screenshot is present and valid (PNG/JPG, reasonable size)" - Visual requirement
3. "AI development story section is present" - Unique value proposition
4. "Installation instructions section is present" - User onboarding
5. "Links to roadmap and documentation are present and functional" - Navigation
6. "Professional tone throughout (no slang, emojis, or unprofessional language)" - Quality requirement

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Documentation validation":
  → Content validation tests REQUIRED
  Reason: Must verify specific sections and content are present

IF "File system side effects" (screenshot):
  → File validation tests REQUIRED
  Reason: Need to verify screenshot file exists and is valid image

IF "Subjective quality" (tone, clarity):
  → Manual review REQUIRED
  Reason: Automated tests cannot validate subjective quality
```

**Derived Test Types:**

**Test Type 1: File Existence and Format Tests**
- **Validates:** README.md exists, is readable, is valid markdown
- **Approach:** Automated test reads README.md, verifies basic markdown syntax
- **Rationale:** Basic smoke test - file must exist and be parseable
- **Gap if missing:** Broken builds, corrupted README
- **Necessity:** ✅ REQUIRED

**Test Type 2: Section Presence Tests**
- **Validates:** Required sections are present (TLDR, AI story, Installation, Links)
- **Approach:** Automated test searches README.md for section headers
- **Rationale:** Ensures README has complete structure
- **Gap if missing:** Missing sections, incomplete README
- **Necessity:** ✅ REQUIRED

**Test Type 3: Screenshot Validation Tests**
- **Validates:** Screenshot file exists, is valid image, reasonable size
- **Approach:** Automated test verifies image file exists and is PNG/JPG
- **Rationale:** Screenshot is key visual element
- **Gap if missing:** Missing screenshot, broken image
- **Necessity:** ✅ REQUIRED

**Test Type 4: Link Validation Tests**
- **Validates:** Links to roadmap and documentation are present and syntactically valid
- **Approach:** Automated test parses markdown links, verifies paths exist
- **Rationale:** Broken links hurt user experience
- **Gap if missing:** Broken navigation, user frustration
- **Necessity:** ⚠️ RECOMMENDED

**Test Type 5: Manual Tone and Quality Review**
- **Validates:** Professional tone, clarity, first impression, AI story appropriateness
- **Approach:** Human reviewer reads README and evaluates quality
- **Rationale:** Automated tests cannot validate subjective quality
- **Gap if missing:** Poor quality README, unprofessional tone
- **Necessity:** ⚠️ RECOMMENDED (blocking for production release)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| File existence/format tests | ✅ REQUIRED | Verify README.md exists and is valid markdown | Broken builds, corrupted file | MUST IMPLEMENT |
| Section presence tests | ✅ REQUIRED | Verify required sections present | Missing sections, incomplete | MUST IMPLEMENT |
| Screenshot validation tests | ✅ REQUIRED | Verify screenshot exists and is valid | Missing visual, broken image | MUST IMPLEMENT |
| Link validation tests | ⚠️ RECOMMENDED | Verify links are syntactically valid | Broken navigation | SHOULD IMPLEMENT |
| Manual tone/quality review | ⚠️ RECOMMENDED | Validate subjective quality, tone | Poor quality, unprofessional | BLOCKING FOR RELEASE |

**Summary:**
- ✅ REQUIRED test types: 3 (existence, sections, screenshot)
- ⚠️ RECOMMENDED test types: 2 (links, manual review - manual review BLOCKING)
- ❌ NOT NEEDED test types: 0

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text (quote from spec) | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|-----------------------------------|----------------|--------------|---------------|------------|
| AC-README-001 | "TLDR introduction section (What/Visual/Quick Start)" | sprint-27-planning.md:100 | Section Presence | Specific section required | TC-README-001 |
| AC-README-002 | "AI-agent development story section (tongue-in-cheek tone)" | sprint-27-planning.md:101 | Section Presence + Manual Review | Section required, tone is subjective | TC-README-002 |
| AC-README-003 | "Screenshot of tq in action included" | sprint-27-planning.md:102 | Screenshot Validation | Image file required | TC-README-003 |
| AC-README-004 | "Installation instructions clear and concise" | sprint-27-planning.md:103 | Section Presence + Manual Review | Section required, clarity is subjective | TC-README-004 |
| AC-README-005 | "Links to roadmap and documentation" | sprint-27-planning.md:104 | Link Validation | Navigation required | TC-README-005 |
| AC-README-006 | "Professional tone suitable for public project" | sprint-27-planning.md:105 | Manual Review | Subjective quality assessment | TC-README-MANUAL |
| AC-README-007 | "GitHub Configuration section moved" | sprint-27-planning.md:106 | Section Absence | Verify developer content moved | TC-README-006 |

**Coverage Validation:**
- [x] Every specification requirement appears in table
- [x] Every requirement maps to at least one test type
- [x] Every test type is justified by requirement
- [x] No orphaned requirements (missing test coverage)
- [x] No unjustified test types

**Coverage Gaps:**
- **Subjective quality**: Cannot automatically validate "clear", "concise", "professional" - requires manual review
- **First impression**: Cannot automatically validate README gives good first impression
- **Screenshot quality**: Can verify file exists, but not if it's "good" or "useful"

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Automated Tone Analysis**
- **Reason for omission:** Tone ("professional", "tongue-in-cheek") requires human judgment
- **What won't be validated:** Professional tone, appropriate humor, clarity, readability
- **Risk assessment:** MEDIUM
- **Mitigation:** Manual review required before release
- **Revisit criteria:** If NLP tools for tone analysis become practical

**Network Link Validation Tests**
- **Reason for omission:** Requires network access, links may be ephemeral
- **What won't be validated:** Whether links actually resolve (HTTP 200)
- **Risk assessment:** LOW
- **Mitigation:** Manual click-through during review, CI can optionally validate
- **Revisit criteria:** If CI supports network validation

**Screenshot Content Validation Tests**
- **Reason for omission:** Cannot automatically validate screenshot shows "useful" content
- **What won't be validated:** Screenshot quality, usefulness, clarity
- **Risk assessment:** LOW
- **Mitigation:** Manual review validates screenshot is appropriate
- **Revisit criteria:** If image recognition tools become practical

#### 6. Test Implementation Plan

**Test Type: File Existence and Format Tests**
- **Location:** `tests/integration_tests.rs`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 1 test
- **Key scenarios to cover:**
  1. `test_readme_exists_and_readable` - Verify README.md exists and can be read
- **Implementation notes:** Use `std::fs::read_to_string("README.md")`

**Test Type: Section Presence Tests**
- **Location:** `tests/integration_tests.rs`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 5 tests
- **Key scenarios to cover:**
  1. `test_readme_has_tldr_section` - Verify TLDR/What/Visual/Quick Start section present
  2. `test_readme_has_ai_story_section` - Verify AI development story section present
  3. `test_readme_has_installation_section` - Verify installation instructions present
  4. `test_readme_has_documentation_links` - Verify documentation links section present
  5. `test_readme_no_github_config_at_start` - Verify GitHub Configuration not in first sections
- **Implementation notes:** Search for section headers (e.g., "## What", "## Installation", etc.)

**Test Type: Screenshot Validation Tests**
- **Location:** `tests/integration_tests.rs`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 2 tests
- **Key scenarios to cover:**
  1. `test_readme_references_screenshot` - Verify README includes screenshot image reference
  2. `test_screenshot_file_exists` - Verify screenshot file exists and is valid image (PNG/JPG)
- **Implementation notes:** Parse markdown for image syntax `![alt](path)`, verify file at path exists

**Test Type: Link Validation Tests**
- **Location:** `tests/integration_tests.rs`
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 2 tests
- **Key scenarios to cover:**
  1. `test_readme_has_roadmap_link` - Verify link to roadmap (docs/roadmap/)
  2. `test_readme_has_documentation_link` - Verify link to documentation (docs/)
- **Implementation notes:** Parse markdown for link syntax `[text](path)`, verify path exists (file or directory)

**Test Type: Manual Tone and Quality Review**
- **Location:** `tests/cases/TC-README-MANUAL.md`
- **Framework:** Human reviewer
- **Test count estimate:** 1 manual review
- **Key scenarios to cover:**
  1. Verify TLDR section gives good first impression (clear, concise, compelling)
  2. Verify AI development story is appropriate (tongue-in-cheek but professional)
  3. Verify installation instructions are clear and complete
  4. Verify professional tone throughout (no slang, excessive emojis, unprofessional language)
  5. Verify screenshot is useful and high quality
  6. Verify links are functional (manual click-through)

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the README is "user-focused, professional, and complete"?**

**Analysis:**
- File existence tests validate: README.md exists and is readable
- Section presence tests validate: Required sections (TLDR, AI story, Installation) are present
- Screenshot validation tests validate: Screenshot is included and is valid image
- Link validation tests validate: Links to roadmap and docs are present and syntactically valid
- Manual tone/quality review validates: Professional tone, clarity, first impression
- Combined coverage: ADEQUATE for documentation validation

**Gaps in combined coverage:**
- Subjective quality (tone, clarity, usefulness) requires manual review
- Screenshot content quality requires manual review
- Link functionality (HTTP 200) not automatically validated (low risk)

**Acceptance criteria:**
- [x] All required sections have automated tests
- [x] Screenshot presence verified automatically
- [x] Link presence verified automatically
- [x] Manual quality review is documented as required
- [x] Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Subjective quality gap acceptable because: Manual review is explicitly required before release
- Screenshot content gap acceptable because: Automated tests verify file exists, manual review verifies quality
- Link functionality gap acceptable because: Syntactic validation catches most issues, manual review verifies navigation

---

## Strategy Summary

**Total Features Analyzed:** 3 (Bug fix, LICENSE, README)

**Test Types Required:**
- Unit tests: ✅ (Bug fix: row processing, state coverage)
- Integration tests: ✅ (Bug fix: row count validation, LICENSE: file validation, README: section validation)
- Interactive tests (PTY): ✅ (Bug fix: bug reproduction, regression tests)
- Regression tests: ✅ (Bug fix: Sprint 26 tests TC-SESS-001 through TC-SESS-010)
- File validation tests: ✅ (LICENSE: content, completeness; README: sections, screenshot)
- Manual reviews: ⚠️ (LICENSE: legal review; README: tone/quality review) - BLOCKING

**Estimated Test Count:**
- **Bug Fix (#10):**
  - Bug reproduction: 1 test
  - Regression (Sprint 26): 10 tests (existing)
  - State coverage: 6 tests
  - Row count validation: 2 tests
  - Unit (row processing): 4 tests
  - Manual: 1 test case
  - Subtotal: 24 tests (1 manual)

- **LICENSE (#8):**
  - File existence: 1 test
  - Content validation: 3 tests
  - Completeness: 2 tests
  - README integration: 1 test
  - Manual legal review: 1 review
  - Subtotal: 7 tests + 1 manual review

- **README (#9):**
  - File existence: 1 test
  - Section presence: 5 tests
  - Screenshot validation: 2 tests
  - Link validation: 2 tests
  - Manual tone/quality: 1 review
  - Subtotal: 10 tests + 1 manual review

**Total: 41 automated tests + 3 manual reviews**

**Risk Assessment:**
- HIGH risk gaps: None
- MEDIUM risk gaps:
  - LICENSE legal compliance (mitigated by manual review)
  - README subjective quality (mitigated by manual review)
- LOW risk gaps:
  - Bug fix non-deterministic session states (covered by existing sessions)
  - README link functionality (syntactic validation sufficient)

**Dependencies Required:**
- Live database: Yes (bug fix tests only - marked with `#[ignore]`)
- Network access: No (for core tests)
- Specific OS: No (portable Rust code)
- Other: Screenshot file (provided by user in issue #9)

**Manual Review Requirements (BLOCKING):**
- TC-SESS-BUG-001-MANUAL: Bug fix verification with real workload
- TC-LICENSE-MANUAL: Legal compliance review
- TC-README-MANUAL: Tone and quality review

**Testing Infrastructure Updates:**

No new testing tools or infrastructure updates are required. All tests use existing frameworks:
- Unit tests: Built-in Rust `#[test]` framework
- Integration tests: Built-in Rust integration test support
- Interactive tests: Existing `expectrl` PTY framework (from Sprint 26)
- File validation: Standard library `std::fs` for file operations
- Manual reviews: Test case documentation in `tests/cases/`

**Test Execution Strategy:**

1. **Automated Tests (CI/Local):**
   - Run unit tests: `cargo test --lib`
   - Run integration tests (no DB): `cargo test --test integration_tests`
   - Run bug fix tests (with DB): `cargo test --test interactive_tests -- --ignored`
   - Run regression tests: `cargo test --test interactive_tests -- --ignored` (Sprint 26 tests)

2. **Manual Reviews (Pre-Release):**
   - Bug fix manual verification (TC-SESS-BUG-001-MANUAL)
   - LICENSE legal review (TC-LICENSE-MANUAL) - BLOCKING
   - README quality review (TC-README-MANUAL) - BLOCKING

3. **Sprint Closure Criteria:**
   - 100% automated test pass rate (41 tests)
   - 100% manual review completion (3 reviews)
   - All acceptance criteria met (from sprint-27-planning.md)

---

## Strategy Validation Checklist

**Before submitting to sprint coordinator for review:**

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
- [x] Manual review requirements are explicit and blocking status is clear
- [x] Testing infrastructure updates documented (none required)

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-01-27
**Review Status:** DRAFT
**Submitted for Review:** [Awaiting coordinator approval]

**Reviewer:** sprint-coordinator
**Review Status:** PENDING
**Review Date:** [Pending]
**Review Comments:** [Awaiting review]

**Approval means:**
- ✅ Test strategy derived from specifications and issue requirements (not assumptions)
- ✅ All required test types identified with clear rationale
- ✅ Coverage gaps explicitly identified and assessed
- ✅ Implementation plan is detailed and achievable
- ✅ Manual review requirements are explicit with blocking status
- ✅ Ready to proceed with test execution (manual reviews) or test case creation

**Approval signature:** [sprint-coordinator agent ID and timestamp]
