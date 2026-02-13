# Sprint 35 Test Strategy: Project Configuration + Quick Wins

**Created:** 2026-02-13
**Author:** quality-validator
**Sprint:** Sprint 35
**Features:** Project Config File (.tq.toml), Documentation Polish, Unicode Testing

---

## Overview

Sprint 35 introduces project-level configuration (`.tq.toml`) to enable team-shared connection profiles, complementing the existing user configuration. It also addresses two minor documentation polish items from Sprint 34 and adds proper Unicode testing for identifier quoting.

**Sprint Context:**
- Builds on Sprint 17's user config foundation (100% test pass rate)
- Sprint 34 established clean foundation (649/649 tests passing, zero tech debt)
- P0 focus on project config with precedence management
- P1 quick wins (documentation + Unicode test)
- All features testable without database (except precedence integration testing)

---

## Feature-by-Feature Test Strategy

### Feature 1: Project Config File (`.tq.toml`) - P0

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/specifications/configuration.md` (Project Config section)
- Planning: `docs/sprints/sprint-35-planning.md` lines 44-61
- Context: Sprint 17 user config implementation

**Requirements:**
1. "Parse `.tq.toml` from current directory (walks up to find)" (sprint-35-planning.md line 49)
2. "Load project config before user config (project overrides user)" (sprint-35-planning.md line 50)
3. "Support same TOML structure as user config (profiles, preferences)" (sprint-35-planning.md line 51)
4. "`tq profiles` command shows both user and project profiles" (sprint-35-planning.md line 52)
5. "`--profile` flag works with both user and project profiles" (sprint-35-planning.md line 53)
6. "Project profiles take precedence over user profiles with same name" (sprint-35-planning.md line 54)
7. "Comprehensive error handling (invalid TOML, permission errors)" (sprint-35-planning.md line 55)
8. "Test coverage: unit tests for config loading, integration tests for profile resolution" (sprint-35-planning.md line 57)

**Feature Characteristics:**

**User Interaction Type:** ✅ CLI Batch + Pure Logic
**Explanation:** Project config loading is automatic during CLI initialization. It affects both batch mode commands and REPL startup. The config discovery logic (walk up directory tree) and precedence resolution are pure logic, but their effect is observable in CLI behavior.

**Observable Behavior:**
- ✅ File system side effects (reads `.tq.toml`, walks directory tree)
- ✅ Configuration precedence (project overrides user)
- ✅ Profile resolution with name conflicts
- ✅ Error messages (invalid TOML, missing file, permission errors)

**External Dependencies:**
- ✅ File system access (reads `.tq.toml`, walks directory structure)
- ✅ TOML parsing library (existing `toml` crate)

**Validation Challenges:**
- **Directory walking**: Must test discovery from nested directories, stopping at filesystem root
- **Precedence logic**: Project > User requires testing with conflicting profiles
- **Error handling**: Invalid TOML, permission errors, malformed files
- **Integration**: Must verify `tq profiles` shows both sources, `--profile` resolves correctly

**Critical Behaviors to Validate:**
1. `.tq.toml` discovered by walking up directory tree from current directory
2. Project config loaded and merged with user config (project takes precedence)
3. `tq profiles` lists both user and project profiles with clear source indication
4. Profile name conflicts resolved (project wins)
5. Invalid TOML produces clear error message
6. Missing project config is not an error (optional by design)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Pure Logic" + "File system access" checked:
  → Unit tests REQUIRED
  Reason: Config discovery and precedence logic must be tested in isolation

IF "CLI Batch" + "Configuration behavior" checked:
  → Integration tests REQUIRED
  Reason: End-to-end profile resolution and command execution must be validated
```

**Derived Test Types:**

**Test Type 1: Unit Tests**
- **Validates:** Config discovery logic, directory walking, precedence resolution, TOML parsing
- **Approach:** Test config loading functions with temp directories and mock files
- **Rationale:** Pure logic components (walk up tree, merge configs) must be validated independently
- **Gap if missing:** Logic errors in discovery or precedence not caught until integration
- **Necessity:** ✅ REQUIRED

**Test Type 2: Integration Tests**
- **Validates:** Full CLI behavior with project config, `tq profiles` output, `--profile` resolution
- **Approach:** Create test directory structures with `.tq.toml`, execute commands, validate output
- **Rationale:** End-to-end user experience must be validated with real file structures
- **Gap if missing:** CLI integration bugs, output format issues, precedence in practice
- **Necessity:** ✅ REQUIRED

**Test Type 3: Interactive Tests**
- **Validates:** REPL mode with project config loaded
- **Approach:** Spawn REPL from directory with `.tq.toml`, verify config applied
- **Rationale:** REPL should honor project config just like batch mode
- **Gap if missing:** REPL-specific project config bugs
- **Necessity:** ⚠️ RECOMMENDED (if time permits)

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Unit tests | ✅ REQUIRED | Validates config discovery and precedence logic | Logic bugs in file walking, precedence resolution | MUST IMPLEMENT |
| Integration tests | ✅ REQUIRED | Validates end-to-end CLI behavior | Profile listing bugs, resolution issues | MUST IMPLEMENT |
| Interactive tests | ⚠️ RECOMMENDED | Validates REPL with project config | REPL-specific config loading issues | IMPLEMENT IF TIME |
| Manual tests | ⚠️ RECOMMENDED | Human validates usability | UX confusion with two config sources | DOCUMENT SCENARIOS |

**Summary:**
- ✅ REQUIRED test types: 2 (Unit, Integration)
- ⚠️ RECOMMENDED test types: 2 (Interactive, Manual)
- ❌ NOT NEEDED test types: 0

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| PROJ-1 | Parse `.tq.toml` from current directory (walks up) | sprint-35-planning.md line 49 | Unit + Integration | Unit tests discovery logic, integration tests real behavior | TC-035-001 |
| PROJ-2 | Project config loaded before user config | sprint-35-planning.md line 50 | Unit | Validates loading order and precedence | TC-035-002 |
| PROJ-3 | Same TOML structure as user config | sprint-35-planning.md line 51 | Unit | Validates parsing compatibility | TC-035-003 |
| PROJ-4 | `tq profiles` shows both user and project profiles | sprint-35-planning.md line 52 | Integration | Must validate actual command output | TC-035-004 |
| PROJ-5 | `--profile` works with both sources | sprint-35-planning.md line 53 | Integration | Must test profile resolution from both configs | TC-035-005 |
| PROJ-6 | Project profiles precedence over user profiles | sprint-35-planning.md line 54 | Unit + Integration | Unit tests logic, integration tests behavior | TC-035-006 |
| PROJ-7 | Error handling (invalid TOML, permissions) | sprint-35-planning.md line 55 | Unit + Integration | Both test error paths | TC-035-007 |

**Coverage Validation:**
- ✅ Every specification requirement appears in table
- ✅ Every requirement maps to at least one test type
- ✅ Every test type is justified by requirement
- ✅ No orphaned requirements
- ✅ No unjustified test types

**Coverage Gaps:** None identified

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Benchmark/Performance Tests**
- **Reason for omission:** Config loading has no performance requirements, happens once at startup
- **What won't be validated:** Config loading speed, directory walking performance
- **Risk assessment:** LOW - Config loading is fast, no user-reported issues expected
- **Mitigation:** Monitor in practice, add benchmarks if users report slowness
- **Revisit criteria:** If config loading takes >100ms or users report delays

**Cross-platform Path Tests (Windows vs Unix)**
- **Reason for omission:** Directory walking uses std::fs which handles platform differences
- **What won't be validated:** Windows-specific path resolution edge cases
- **Risk assessment:** LOW - Standard library abstracts platform differences
- **Mitigation:** Test on macOS/Linux, rely on std::fs portability
- **Revisit criteria:** If Windows users report path resolution issues

#### 6. Test Implementation Plan

**Test Type: Unit Tests**
- **Location:** `src/config.rs` test module
- **Framework:** Built-in Rust test framework (`#[test]`)
- **Test count estimate:** 8-10 tests
- **Key scenarios to cover:**
  1. Find `.tq.toml` in current directory
  2. Find `.tq.toml` in parent directory (walk up)
  3. Find `.tq.toml` multiple levels up
  4. Stop at filesystem root (no infinite loop)
  5. Project config overrides user config (same profile name)
  6. Project config merges with user config (different profile names)
  7. Invalid TOML parsing error
  8. Missing `.tq.toml` is not an error (returns None)
  9. Both configs have defaults section (project wins)
  10. Profile field precedence (project profile field overrides user profile field)
- **Mocking strategy:** Use tempfile crate to create test directory structures

**Test Type: Integration Tests**
- **Location:** `tests/integration_tests.rs`
- **Framework:** Built-in Rust integration test support with std::process::Command
- **Test count estimate:** 6-8 tests
- **Key scenarios to cover:**
  1. `tq profiles` with only user config (baseline)
  2. `tq profiles` with only project config
  3. `tq profiles` with both configs (shows both, indicates source)
  4. `tq profiles` with name conflict (project profile shown, user profile hidden or marked)
  5. `tq --profile <project-profile>` resolves correctly
  6. `tq --profile <user-profile>` resolves correctly
  7. Invalid `.tq.toml` produces error message
  8. Profile field override (project host overrides user host)
- **Setup requirements:** Create temp directories with `.tq.toml` and config files, set working directory

**Test Type: Interactive Tests (if time permits)**
- **Location:** `tests/interactive_tests.rs`
- **Framework:** expectrl crate for PTY simulation
- **Test count estimate:** 2 tests
- **Key scenarios to cover:**
  1. Start REPL from directory with `.tq.toml`, verify connection uses project profile
  2. `/profiles` metacommand shows both user and project profiles
- **Implementation notes:** Requires live database for connection testing

#### 7. Coverage Sufficiency Assessment

**Question: If all planned test types are implemented and passing, can we claim the feature "works as specified"?**

**Analysis:**
- Unit tests validate: Config discovery, directory walking, precedence logic, TOML parsing
- Integration tests validate: CLI behavior, profile listing, profile resolution, error messages
- Interactive tests validate: REPL integration with project config
- Combined coverage: **Comprehensive**

**Gaps in combined coverage:**
- None identified - unit and integration tests cover all critical behaviors

**Acceptance criteria:**
- ✅ All specification requirements have test coverage
- ✅ All test types justified by requirements
- ✅ Combined coverage is sufficient to claim "works as specified"
- ✅ Known gaps are documented and accepted

**If gaps exist, document why they're acceptable:**
- Performance testing deferred (LOW risk, no requirements)
- Windows-specific testing relies on std::fs portability (LOW risk)

---

### Feature 2: Sprint 34 Documentation Polish - P1

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-35-planning.md` lines 69-81
- Context: Sprint 34 Review section 9 (Lessons Learned)

**Requirements:**
1. "Add emoji badge (🧪 EXPERIMENTAL) to pager section in specifications" (sprint-35-planning.md line 72)
2. "Verify `/peek` default count in code and update documentation if needed" (sprint-35-planning.md line 73)

**Feature Characteristics:**

**User Interaction Type:** ✅ Documentation Update (no code changes)
**Explanation:** Pure documentation updates to specifications. No functional changes, no user-observable behavior changes.

**Observable Behavior:**
- ❌ None (documentation only)

**External Dependencies:**
- ❌ None

**Validation Challenges:**
- Documentation accuracy (emoji added, /peek default verified)
- No code changes needed unless /peek default is wrong

**Critical Behaviors to Validate:**
1. Pager section in specifications has 🧪 EXPERIMENTAL badge
2. `/peek` default count in documentation matches code implementation

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Documentation Update" + "No code changes":
  → Manual validation REQUIRED
  → Automated tests NOT APPLICABLE (no code behavior to test)
```

**Derived Test Types:**

**Test Type 1: Manual Validation**
- **Validates:** Documentation accuracy, emoji added, /peek default verified
- **Approach:** Read specifications, grep for emoji, check code for /peek default
- **Rationale:** Documentation changes require human review
- **Gap if missing:** Documentation inaccuracy not caught
- **Necessity:** ✅ REQUIRED

**Test Type 2: Code Inspection**
- **Validates:** `/peek` default value in code matches documentation
- **Approach:** Read code, verify default parameter value
- **Rationale:** Ensures documentation reflects reality
- **Gap if missing:** Documentation-code mismatch
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Manual validation | ✅ REQUIRED | Documentation changes need human review | Inaccurate documentation | MUST PERFORM |
| Code inspection | ✅ REQUIRED | Verify /peek default matches docs | Documentation-code mismatch | MUST PERFORM |
| Unit tests | ❌ NOT NEEDED | No code changes to test | N/A | SKIP |
| Integration tests | ❌ NOT NEEDED | No functional changes | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 2 (Manual validation, Code inspection)
- ❌ NOT NEEDED test types: 2 (Unit, Integration)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| DOC-1 | Add emoji badge to pager section | sprint-35-planning.md line 72 | Manual | Documentation review | Manual checklist |
| DOC-2 | Verify /peek default in code | sprint-35-planning.md line 73 | Code Inspection | Compare code to docs | Manual checklist |

**Coverage Validation:**
- ✅ All requirements covered
- ✅ Appropriate validation method

**Coverage Gaps:** None

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Automated Tests**
- **Reason:** Documentation changes have no code behavior to test
- **Risk:** NONE
- **Mitigation:** Manual review by quality-validator

#### 6. Test Implementation Plan

**Manual Validation Checklist:**
- [ ] Open `docs/specifications/repl.md`
- [ ] Search for pager section
- [ ] Verify 🧪 EXPERIMENTAL emoji badge present
- [ ] Open code for `/peek` implementation
- [ ] Verify default count value (likely 10 or 20)
- [ ] Check documentation mentions correct default
- [ ] Update documentation if mismatch found

**Estimated time:** 5-10 minutes

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Manual validation validates: Documentation accuracy
- Code inspection validates: Documentation-code consistency
- Combined coverage: **Sufficient for documentation updates**

**Acceptance criteria:**
- ✅ All documentation changes verified
- ✅ Code-documentation consistency confirmed

---

### Feature 3: Enhanced Unicode Testing - P1

#### 1. Specification Analysis

**Specification References:**
- Primary: `docs/sprints/sprint-35-planning.md` lines 86-98
- Context: Sprint 34 Review section 9 (Lessons Learned)
- Code reference: `src/sql/identifiers.rs`

**Requirements:**
1. "Create `test_quote_identifier_unicode_actual()` in `src/sql/identifiers.rs`" (sprint-35-planning.md line 89)
2. "Test Unicode characters: 中文 (Chinese), العربية (Arabic), emoji, etc." (sprint-35-planning.md line 90)
3. "Verify double-quote escaping works with all Unicode" (sprint-35-planning.md line 91)
4. "All tests pass (649/649 → 650/650)" (sprint-35-planning.md line 92)

**Feature Characteristics:**

**User Interaction Type:** ✅ Pure Logic (internal test addition)
**Explanation:** This is adding a unit test for existing functionality (SQL identifier quoting). No functional changes, just test coverage improvement.

**Observable Behavior:**
- ❌ None (test addition only, existing functionality already works)

**External Dependencies:**
- ❌ None

**Validation Challenges:**
- Test implementation (write test that validates Unicode handling)
- Test must pass (validates existing implementation is correct)

**Critical Behaviors to Validate:**
1. New test exists in `src/sql/identifiers.rs`
2. Test covers Unicode characters (Chinese, Arabic, emoji)
3. Test validates double-quote escaping with Unicode
4. Test passes (650/650 total)

#### 2. Test Strategy Derivation

**Decision Tree Results:**

```
IF "Test Addition" (not feature addition):
  → Meta-validation REQUIRED (test the test)
  → Run new test and verify it passes
```

**Derived Test Types:**

**Test Type 1: Test Execution**
- **Validates:** New test exists and passes
- **Approach:** Run `cargo test test_quote_identifier_unicode_actual`
- **Rationale:** Ensures test is implemented correctly
- **Gap if missing:** New test might not exist or might fail
- **Necessity:** ✅ REQUIRED

**Test Type 2: Code Review**
- **Validates:** Test covers Unicode scenarios comprehensively
- **Approach:** Read test code, verify Unicode characters tested
- **Rationale:** Ensures test quality and coverage
- **Gap if missing:** Test might exist but be incomplete
- **Necessity:** ✅ REQUIRED

#### 3. Test Type Necessity Matrix

| Test Type | Necessary? | Rationale | Gap if Omitted | Decision |
|-----------|------------|-----------|----------------|----------|
| Test execution | ✅ REQUIRED | Must verify test exists and passes | Test not implemented or failing | MUST RUN |
| Code review | ✅ REQUIRED | Must verify test quality | Incomplete test coverage | MUST REVIEW |
| Additional tests | ❌ NOT NEEDED | Feature already works, just adding test | N/A | SKIP |

**Summary:**
- ✅ REQUIRED test types: 2 (Test execution, Code review)
- ❌ NOT NEEDED test types: 1 (Additional tests)

#### 4. Specification Coverage Map

| Requirement ID | Requirement Text | Spec Reference | Test Type(s) | Justification | Test Cases |
|----------------|------------------|----------------|--------------|---------------|------------|
| UNICODE-1 | Test exists in identifiers.rs | sprint-35-planning.md line 89 | Test Execution | Verify test implemented | Test run |
| UNICODE-2 | Test covers Unicode (Chinese, Arabic, emoji) | sprint-35-planning.md line 90 | Code Review | Verify test comprehensiveness | Code inspection |
| UNICODE-3 | Test validates double-quote escaping | sprint-35-planning.md line 91 | Code Review | Verify test logic | Code inspection |
| UNICODE-4 | All tests pass (650/650) | sprint-35-planning.md line 92 | Test Execution | Verify no regressions | Full test run |

**Coverage Validation:**
- ✅ All requirements covered
- ✅ Appropriate validation methods

**Coverage Gaps:** None

#### 5. Gap Analysis

**Test Types Intentionally Omitted:**

**Additional Unicode Tests**
- **Reason:** Feature already implemented and working, just adding one test for coverage
- **Risk:** NONE
- **Mitigation:** Existing implementation already handles Unicode correctly

#### 6. Test Implementation Plan

**Validation Approach:**
1. **Code Review** - Inspect new test in `src/sql/identifiers.rs`:
   - [ ] Test function `test_quote_identifier_unicode_actual()` exists
   - [ ] Test includes Chinese characters (中文)
   - [ ] Test includes Arabic characters (العربية)
   - [ ] Test includes emoji (😀, ✅, etc.)
   - [ ] Test validates double-quote escaping (e.g., `"含""引号"` for `含"引号`)
   - [ ] Test has clear assertions

2. **Test Execution** - Run test suite:
   ```bash
   # Run new test specifically
   cargo test --lib test_quote_identifier_unicode_actual

   # Run all tests to verify 650/650
   cargo test --lib
   ```

**Estimated time:** 2 minutes

#### 7. Coverage Sufficiency Assessment

**Analysis:**
- Test execution validates: New test exists and passes
- Code review validates: Test quality and comprehensiveness
- Combined coverage: **Sufficient for test addition**

**Acceptance criteria:**
- ✅ New test exists
- ✅ New test passes
- ✅ Test covers required Unicode scenarios
- ✅ Total test count increases to 650

---

## Strategy Summary

**Total Features Analyzed:** 3

**Test Types Required:**
- Unit tests: ✅ [Project Config] (required)
- Integration tests: ✅ [Project Config] (required)
- Interactive tests: ⚠️ [Project Config REPL] (recommended)
- Manual validation: ✅ [Documentation Polish, Unicode Test] (required)
- Code review: ✅ [Documentation Polish, Unicode Test] (required)

**Estimated Test Count:**
- Unit: 8-10 tests (project config discovery, precedence)
- Integration: 6-8 tests (tq profiles, --profile resolution)
- Interactive: 2 tests (REPL with project config - if time)
- Manual: 2 checklists (documentation, Unicode test review)
- **Total new automated tests: 14-20**
- **Baseline: 649 tests → Target: 663-669 tests**

**Test Cases to Document:**
- TC-035-001: Project config discovery (walk up directory tree)
- TC-035-002: Project config precedence (project > user)
- TC-035-003: Project config TOML structure compatibility
- TC-035-004: `tq profiles` shows both user and project profiles
- TC-035-005: `--profile` resolves from both configs
- TC-035-006: Profile name conflicts (project wins)
- TC-035-007: Error handling (invalid TOML, permissions)

**Risk Assessment:**
- **HIGH risk gaps:** None
- **MEDIUM risk gaps:** None
- **LOW risk gaps:**
  - Interactive tests for REPL (recommended but not critical)
  - Performance testing for config loading (no requirements)
  - Windows-specific path testing (relies on std::fs)

**Dependencies Required:**
- Live database: ⚠️ CONDITIONAL (only for interactive tests if implemented)
- Network access: ❌ NO
- Specific OS: ❌ NO (std::fs handles platform differences)
- File system: ✅ YES (tempfile for test directories)

**Sprint 35 Specific Notes:**
- **Primary focus:** Project config feature (P0 - critical)
- **Quick wins:** Documentation polish (5 min), Unicode test (5 min)
- **No database required** - Unit and integration tests work without database
- **Database optional** - Interactive tests need database but are lower priority
- **Test strategy mature** - Following established patterns from Sprint 17

---

## Tool Requirements Assessment

### Current Testing Tools (from `tests/README.md`)

**Available:**
- ✅ Unit test framework (built-in Rust)
- ✅ Integration test framework (std::process::Command)
- ✅ Interactive test framework (expectrl + PTY)
- ✅ Tempfile crate for test fixtures
- ✅ Environment variable management (dotenvy)

**Needed for Sprint 35:**
- ✅ tempfile - Already available (for creating test directory structures)
- ✅ std::fs - Built-in (for file operations)
- ✅ std::env - Built-in (for setting working directory in tests)

**New Tools Required:** NONE

All necessary testing infrastructure already exists. Project config testing will use:
1. **tempfile::TempDir** - Create test directory structures
2. **std::env::set_current_dir** - Change working directory for config discovery tests
3. **std::fs::write** - Create `.tq.toml` and config files
4. **std::process::Command** - Execute tq CLI for integration tests

### Tool Assessment Summary

**Can current tools test project config features?** ✅ YES

**New tools needed for config file discovery?** ❌ NO
- tempfile provides directory structure creation
- std::env provides working directory control
- std::fs provides file creation

**New tools for precedence testing?** ❌ NO
- Existing config loading code handles precedence
- Unit tests validate logic
- Integration tests validate behavior

**Recommendation:** Proceed with existing tools. No new tool development required.

---

## Test Execution Strategy

### Phase 1: Feature 3 - Unicode Test (5 minutes)
Quick win - complete first:
1. Review new test in `src/sql/identifiers.rs`
2. Run `cargo test --lib test_quote_identifier_unicode_actual`
3. Verify test passes and covers Unicode scenarios
4. ✅ Mark Feature 3 complete

### Phase 2: Feature 2 - Documentation Polish (5-10 minutes)
Quick win - complete second:
1. Verify pager emoji badge added
2. Check `/peek` default in code
3. Update documentation if needed
4. ✅ Mark Feature 2 complete

### Phase 3: Feature 1 - Project Config Unit Tests (1-2 hours)
Critical path - implement unit tests:
1. Test config discovery (current dir, walk up, stop at root)
2. Test precedence logic (project > user)
3. Test TOML structure compatibility
4. Test error handling (invalid TOML)
5. Run: `cargo test --lib config::tests::test_project_config`
6. Verify all unit tests pass

### Phase 4: Feature 1 - Project Config Integration Tests (1-2 hours)
Critical path - implement integration tests:
1. Test `tq profiles` with both configs
2. Test `--profile` resolution from both sources
3. Test profile name conflicts
4. Test profile field overrides
5. Run: `cargo test --test integration_tests test_project_config`
6. Verify all integration tests pass

### Phase 5: Feature 1 - Interactive Tests (Optional, 30-60 minutes)
If time permits:
1. Test REPL with project config
2. Test `/profiles` metacommand
3. Run: `cargo test --test interactive_tests -- --ignored`
4. Verify REPL integration works

### Phase 6: Full Regression (15-30 minutes)
Verify no regressions:
```bash
# Run all unit tests
cargo test --lib

# Run all integration tests
cargo test --test integration_tests

# Expected: 663-669 tests passing (649 baseline + 14-20 new)
```

### Phase 7: Test Report Generation
Document results in `tests/results/sprint-35/REPORT.md`

---

## Coverage Sufficiency Assessment

### Overall Coverage Analysis

**Feature 1 (Project Config):**
- Unit tests validate: Config discovery, precedence logic, TOML parsing
- Integration tests validate: CLI behavior, profile listing, profile resolution
- Coverage: **Comprehensive** (14-18 tests cover all critical paths)

**Feature 2 (Documentation Polish):**
- Manual validation validates: Documentation accuracy
- Coverage: **Sufficient** (documentation-only changes)

**Feature 3 (Unicode Test):**
- Test execution validates: New test exists and passes
- Code review validates: Test quality
- Coverage: **Sufficient** (test addition, not feature addition)

**Combined Sprint Coverage:**
- All P0 requirements have automated tests
- All P1 requirements have validation approach
- No critical gaps identified
- **Overall: Comprehensive coverage for Sprint 35 deliverables**

---

## Success Criteria

Sprint 35 test strategy is successful if:

1. **Test Coverage Complete:**
   - ✅ All 3 features have test strategy defined
   - ✅ All P0 and P1 requirements mapped to tests
   - ✅ Test types derived from feature characteristics

2. **Test Implementation Achievable:**
   - ✅ 14-20 automated tests (clear scope, well-defined)
   - ✅ No new tools required (existing infrastructure sufficient)
   - ✅ Test setup straightforward (tempfile + std::fs)

3. **Quality Assurance Robust:**
   - ✅ Project config has unit + integration coverage
   - ✅ Documentation changes have manual validation
   - ✅ Unicode test has execution + review validation

4. **Gaps Identified and Accepted:**
   - ✅ Interactive tests optional (LOW risk)
   - ✅ Performance tests deferred (LOW risk)
   - ✅ All gaps have risk assessment

5. **Execution Plan Clear:**
   - ✅ Test phases defined with dependencies
   - ✅ Priority order established (quick wins first)
   - ✅ Success criteria for each phase

---

## Strategy Validation Checklist

**Before submitting for review:**

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

**Strategy Status:** READY FOR REVIEW

---

## Sign-off

**Test Strategy Author:** quality-validator
**Created Date:** 2026-02-13
**Review Status:** DRAFT
**Sprint:** 35 - Configuration Management + Quick Wins
**Submitted for Review:** 2026-02-13

**Reviewer:** tq-project-manager (pending)
**Review Status:** PENDING
**Review Date:** (pending)
**Review Comments:** (pending)

**Approval means:**
- ✅ Test strategy derived from specifications (not assumptions)
- ✅ All required test types identified with clear rationale
- ✅ Coverage gaps explicitly identified and assessed
- ✅ Implementation plan is detailed and achievable
- ✅ Ready to proceed with test implementation
