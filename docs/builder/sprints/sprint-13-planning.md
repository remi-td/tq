---
sprint: 13
start_date: 2026-01-19
target_completion: 2026-01-19
status: Planning
---

# Sprint 13 Planning: Quality Crisis Recovery + Interactive Testing Framework

## Sprint Overview

**Sprint Goal:** Fix critical tab completion and branding bugs with proper interactive testing, restore user trust, and ship quality improvements.

**Sprint Theme:** "Test What Users See, Not Just What Code Does" - This sprint implements systematic interactive testing to catch bugs that unit tests miss.

**Critical Context:** Tab completion has been reported broken **FOUR times** across four sprints despite 100% unit test pass rates. The root cause is a fundamental test coverage gap: unit tests verify logic, but interactive features need interactive tests. This sprint fixes both the testing framework AND the bugs.

---

## Objectives

1. **Implement Interactive Testing Framework** - Add expectrl-based tests to catch real UX bugs
2. **Fix Tab Completion Properly** - Fix all three reported issues with real validation
3. **Fix Branding Issues** - Create proper branding guidelines and implement correctly
4. **Verify Export Functionality** - Ensure full dataset export actually works
5. **Build Quality Improvements** - Clean up build warnings from Sprint 12
6. **Ship Value** - Add one small high-value feature to balance bug fixes

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Interactive Testing Framework

**Description:** Implement expectrl-based integration tests for REPL features to catch bugs that unit tests miss. This is the foundation for fixing tab completion and preventing future regressions.

**Acceptance Criteria:**
- [ ] At least 5 interactive tests implemented in `tests/interactive_tests.rs`
- [ ] Test: Database/table completion shows correct visual output (not keywords)
- [ ] Test: Completion inserts at cursor position (not line start)
- [ ] Test: Reserved word completion (`sel `→`SELECT`, `fr`→`FROM`)
- [ ] Test: Multi-line context preservation
- [ ] Test: Schema-qualified completion (`database.<Tab>` shows tables)
- [ ] All interactive tests passing with real tq binary
- [ ] Test framework documented in `testing-guidelines.md`

**Reference:** `tab-completion-failure-analysis.md` (lines 430-459)

**Estimated Complexity:** High

**Critical:** This is BLOCKING - cannot claim tab completion fixed without these tests.

---

#### Feature 2: Fix Tab Completion (All Three Issues)

**Description:** Fix the three critical tab completion bugs reported by user using interactive tests to validate fixes.

**Issue 1: Shows Keywords Instead of Tables**
```
User types: select * from <Tab>
Current: Shows (SQL keyword) (SQL keyword) ...
Expected: Shows database_name1, database_name2, ...
```

**Issue 2: Cursor Insertion at Beginning of Line**
```
User types: select * from database_name<Tab>
Current: Completion inserts at line start
Expected: Inserts at cursor position
```

**Issue 3: Reserved Word Completion Doesn't Work**
```
User types: sel * fr<Tab>
Current: Shows all keywords or doesn't complete
Expected: Auto-completes to "FROM" (only valid keyword)
```

**Acceptance Criteria:**
- [ ] Issue 1: After `SELECT * FROM `, Tab shows database names (verified by interactive test)
- [ ] Issue 2: Completion inserts at cursor position (verified by interactive test)
- [ ] Issue 3: `sel `+Tab completes to `SELECT`, `fr`+Tab completes to `FROM` (verified by interactive test)
- [ ] All existing unit tests still passing
- [ ] All new interactive tests passing
- [ ] Manual test cases TC027, TC028 executed and documented
- [ ] User validation completed and approved

**Reference:** `docs/builder/incoming/open-bugs.md` (lines 24-34), `tab-completion-failure-analysis.md` (lines 179-221)

**Estimated Complexity:** High

**Critical:** Feature broken for 4 sprints, must be fixed properly this time.

---

#### Feature 3: Fix Logo Branding Issues

**Description:** Create comprehensive branding guidelines and implement logo correctly per user specifications.

**User Requirements:**
- Tool name `tq` in lowercase
- Letter 't' in Teradata orange (#F37021)
- Use block character █ (simpler than | and _)
- Logo renders correctly (last two lines not offset)
- Interactive prompt `tq>` in Teradata orange (not green)

**Acceptance Criteria:**
- [ ] Create `docs/builder/detailed-specifications/branding-guidelines.md` with complete design
- [ ] Logo redesigned using █ block character per user specification
- [ ] Logo last two lines properly aligned (no offset)
- [ ] Tool name displayed as lowercase `tq` with 't' in Teradata orange
- [ ] Interactive prompt `tq>` colored in Teradata orange (not green)
- [ ] User validates and approves logo design
- [ ] Implementation matches branding guidelines document

**Reference:** `docs/builder/incoming/open-bugs.md` (lines 6-22)

**Estimated Complexity:** Medium

---

### P1 - High Priority (Should Have)

#### ~~Feature 4: Verify and Fix Export Full Dataset~~ ✅ VERIFIED WORKING

**Status:** User confirmed that export full dataset is working correctly as of 2026-01-19.

**Original Issue:** User initially reported export only saving 100 rows instead of full dataset.

**Resolution:** Feature was working correctly. User has now verified the functionality works as expected.

**No action required in this sprint.**

---

#### Feature 5: Simplify Export Command Syntax

**Description:** Simplify `/export` command semantics to reduce user confusion.

**Current Confusing Syntax:**
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

**Acceptance Criteria:**
- [ ] Syntax simplified to `/export <format> [destination]`
- [ ] `destination` can be filename or literal `clipboard`
- [ ] Help text updated to show new syntax
- [ ] Examples: `/export csv results.csv`, `/export json clipboard`
- [ ] Backward compatibility maintained (old syntax still works with deprecation notice)
- [ ] All export tests passing with new syntax
- [ ] Documentation updated

**Reference:** `docs/builder/incoming/open-bugs.md` (lines 38-58)

**Estimated Complexity:** Low

---

#### Feature 6: Build Warning Cleanup

**Description:** Fix 4 build warnings from Sprint 12 branding code.

**Acceptance Criteria:**
- [ ] Fix unused Result warnings in `src/commands/repl/mod.rs` (lines 239-242)
- [ ] Use proper error handling pattern: `let _ = writeln!(...)`
- [ ] Zero build warnings after fix
- [ ] Logo still displays correctly after changes

**Reference:** Sprint 12 review (lines 219-242)

**Estimated Complexity:** Low

---

### P2 - Medium Priority (Nice to Have)

#### Feature 7: Connection String Validation (New Feature)

**Description:** Add connection string validation with helpful error messages to improve user experience when configuring connections.

**Rationale:** Ship a small new feature to add value beyond just bug fixes. Connection validation is quick to implement and provides immediate user benefit.

**Acceptance Criteria:**
- [ ] Validate connection string format before attempting connection
- [ ] Clear error messages for common mistakes:
  - Missing host, port, or database
  - Invalid port number
  - Malformed connection string syntax
- [ ] Helpful suggestions in error messages (e.g., "Expected format: user:pass@host:port/db")
- [ ] Unit tests for validation logic
- [ ] Integration tests for error scenarios

**Estimated Complexity:** Low

---

### Explicitly Out of Scope

Things we are intentionally NOT doing in this sprint:

- Configuration files (user config, connection profiles) - Deferred to Sprint 14+
- Transaction control features - Planned for future
- Variable substitution - Planned for future
- Streaming large results - Planned for future
- Additional completion features (functions, schemas) - After tab completion proven working

**Rationale:** Focus on fixing critical quality issues before adding complexity.

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] **Interactive testing framework implemented and working**
- [ ] **All 3 tab completion issues fixed and validated by user**
- [ ] **Logo branding fixed and approved by user**
- [ ] **Export full dataset verified working (or fixed if broken)**
- [ ] **Export syntax simplified**
- [ ] **Zero build warnings**
- [ ] 100% test pass rate (unit + integration + interactive tests)
- [ ] All acceptance criteria met for P0 and P1 features
- [ ] Documentation updated to reflect new features
- [ ] Zero technical debt introduced
- [ ] Code quality meets project standards (per rust-architecture.md)
- [ ] All features validated by quality-validator agent
- [ ] User validation completed for all REPL/UX features
- [ ] Completion validated by tq-project-manager agent

---

## Dependencies

### External Dependencies
- arboard (clipboard library) - Already added in Sprint 12
- expectrl (interactive testing) - Already available, needs implementation

### Prerequisite Work
- Sprint 12 codebase as baseline
- Tab completion failure analysis completed (already done)

### Blockers
- **User availability for validation** - CRITICAL: Cannot close sprint without user validation of REPL features
- **Test database access** - Required for interactive tests (assume available per .env setup)

---

## Risks & Mitigation

### Risk 1: Tab Completion Root Cause Elusive
- **Probability:** Medium
- **Impact:** High (delays sprint completion)
- **Mitigation:**
  - Start with interactive tests to expose exact failure mode
  - Debug with real REPL session, not just unit tests
  - Use `println!()` debugging to trace reedline integration
  - Review reedline documentation for completion behavior

### Risk 2: User Validation Unavailable
- **Probability:** Low (user engaged this sprint)
- **Impact:** High (cannot close sprint)
- **Mitigation:**
  - Proactively request user validation early in sprint
  - Provide clear validation checklist for user
  - Have interactive tests as backup validation

### Risk 3: Export Feature Actually Broken
- **Probability:** Medium (user says it's broken, Sprint 12 says it works)
- **Impact:** Medium (requires additional fix work)
- **Mitigation:**
  - Test early in sprint with real database
  - Document actual behavior vs. expected
  - Budget time for fix if needed

---

## Action Items from Previous Sprint

Items from Sprint 12 review:

- [x] Build Warning Cleanup - Addressed in Feature 6 (P1)
- [x] Interactive Test Framework - Identified in failure analysis, now P0 in this sprint
- [ ] User validation of REPL features - Will be mandatory in this sprint

**Reference:** [Sprint 12 Review](sprint-12-review.md)

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Create comprehensive branding guidelines document (Feature 3)
- Design logo using block character per user specifications
- Update REPL specifications with new interactive testing requirements
- Simplify export command syntax design (Feature 5)
- Update `specifications.md` with 🚧 status for in-progress features

**Deliverables:**
- `docs/builder/detailed-specifications/branding-guidelines.md` (complete design)
- Updated `specs.md` with branding and testing requirements
- Export syntax simplification design document
- UX validation for all visual changes

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement interactive testing framework (Feature 1) - FIRST PRIORITY
- Debug and fix all 3 tab completion issues (Feature 2)
- Implement logo redesign per branding guidelines (Feature 3)
- Verify/fix export full dataset functionality (Feature 4)
- Simplify export command syntax (Feature 5)
- Fix build warnings (Feature 6)
- Implement connection validation (Feature 7, if time permits)
- Update `rust-architecture.md` if patterns change
- Write unit tests for all new code

**Deliverables:**
- 5+ interactive tests in `tests/interactive_tests.rs`
- Fixed tab completion code with all tests passing
- Redesigned logo implementation
- Working export full dataset feature
- Simplified export syntax implementation
- Zero build warnings
- Updated architecture documentation

**Critical:** Must implement interactive tests BEFORE claiming tab completion fixed.

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design comprehensive test strategy for interactive features
- Execute manual test cases TC027, TC028 (tab completion)
- Execute manual export tests with large datasets (Feature 4)
- Validate logo rendering in real terminal (Feature 3)
- Run all test suites (unit + integration + interactive)
- Generate test reports in `tests/results/`
- Validate acceptance criteria for all features
- **Mandatory:** User validation checklist for REPL features

**Deliverables:**
- Test strategy document in `tests/strategy/sprint-13-test-strategy.md`
- Updated test cases with actual results documented
- Test execution report in `tests/results/YYYYMMDD-HHMMSS/REPORT.md`
- User validation checklist for tab completion, logo, export
- 100% test pass rate for all test types
- Documented user validation results

**Critical:** Cannot approve sprint without user validation of REPL features.

---

### tq-project-manager (Haiku)
**Responsibilities:**
- Validate sprint completion at closure
- Verify user validation completed for all REPL features
- Assess technical debt status
- Verify all documentation synchronized
- Validate that testing framework gaps are closed
- Provide go/no-go decision for sprint closure

**Deliverables:**
- Sprint completion validation report
- User validation verification
- Technical debt assessment
- Testing framework validation
- Go/no-go recommendation with rationale
- Recommendations for Sprint 14

---

## Sprint Timeline

**Estimated Duration:** 1 day (autonomous execution)

### Phase Breakdown

- **Phase 1: Planning** (Complete)
  - Sprint planning document created
  - Autonomous execution approved by user

- **Phase 2: Design** (Est. 2 hours)
  - Parallel execution: cli-ux-designer + rust-teradata-architect feasibility
  - Branding guidelines created
  - Testing framework design finalized
  - Export syntax simplification designed

- **Phase 3: Implementation** (Est. 6-8 hours)
  - Sequential (testing framework must come first):
    1. rust-teradata-architect implements interactive tests FIRST
    2. THEN fixes tab completion using those tests to validate
  - Parallel where independent:
    - Logo redesign + Export syntax + Build warnings can run in parallel
  - quality-validator designs test strategy in parallel

- **Phase 4: Testing** (Est. 2-3 hours)
  - **Stage 1:** quality-validator creates test strategy, tq-project-manager validates strategy
  - **Stage 2:** quality-validator executes all tests (unit + integration + interactive)
  - Manual test execution (TC027, TC028, export tests)
  - User validation of REPL features (MANDATORY)
  - 100% pass rate achieved

- **Phase 5: Closure** (Est. 1-2 hours)
  - tq-project-manager validates completion (including user validation)
  - Sprint review created
  - Roadmap updated
  - Commit and push changes

- **Phase 6: Framework Optimization** (Est. 1 hour)
  - Review sprint retrospective for framework improvements
  - Update testing guidelines with interactive test requirements
  - Update sprint-coordinator skill with UX validation requirements
  - Commit framework improvements

---

## Notes

### Critical Success Factors

1. **Interactive Tests Before Fixes:** Must implement interactive testing framework BEFORE attempting to fix tab completion. Testing framework is the foundation.

2. **User Validation Mandatory:** Cannot close sprint without user validation of:
   - Tab completion (all 3 issues fixed)
   - Logo branding (design approved)
   - Export full dataset (verified working)

3. **Test What Users See:** Unit tests verify logic, interactive tests verify UX. Both are required.

4. **Follow Branding Guidelines:** Must create guidelines BEFORE implementing logo. No more guesswork.

5. **Document Failures:** If tests fail, document actual behavior vs. expected. No "it should work" without proof.

### Framework Improvements This Sprint

This sprint implements critical framework improvements:

1. **Interactive Testing Requirement:** All REPL features must have expectrl tests (new)
2. **User Validation Requirement:** UX features cannot close without user sign-off (new)
3. **Specification Completeness:** Design details must be complete before implementation (reinforced)
4. **Test Coverage Analysis:** Different feature types need different testing approaches (new insight)

### Why This Sprint Will Succeed

**Previous Sprints Failed Because:**
- Relied only on unit tests for interactive features
- Didn't validate with real user workflows
- Specifications incomplete (branding guidelines missing)
- Lessons learned but not applied

**This Sprint Will Succeed Because:**
- Interactive testing framework implemented first
- User validation mandatory before closure
- Branding guidelines created before implementation
- Framework updated to prevent recurrence
- Test strategy validated before execution

---

## Approval

**Status:** Approved (Autonomous Execution)

**Approved By:** User (implicit - requested autonomous execution)
**Approval Date:** 2026-01-19

**Sprint Coordinator Note:** Executing autonomously per user request. Will deliver 100% working features with proper validation this time.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-19 | 1.0 | Initial Sprint 13 plan - Quality crisis recovery | Sprint Coordinator |
