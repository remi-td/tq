---
sprint: 8
start_date: 2026-01-18
target_completion: 2026-01-19
status: Planning
---

# Sprint 8 Planning: Critical Bug Fixes & Quality Recovery

## Sprint Overview

**Sprint Goal:** Fix all critical bugs from production user reports and establish mandatory live database testing to prevent future quality failures.

**Sprint Theme:** Quality Recovery - This sprint addresses a catastrophic quality failure where Sprints 5, 6, and 7 were marked "complete with 100% test pass rate" but multiple features don't work against real Teradata databases.

**Severity:** CRITICAL - Users are experiencing broken core features that were supposedly delivered and tested.

---

## Root Cause Analysis

**Quality Failure:** Our test strategy relied exclusively on unit tests without validating features against live Teradata databases. This allowed us to ship:
- Broken table formatting (Sprint 6 - supposedly fixed)
- Non-functional tab completion (Sprint 7 - main feature)
- Broken result paging (Sprint 5 - main feature)
- Incorrect SQL syntax hints (multiple sprints)

**Action Required:** Implement mandatory live database integration testing for all future sprints. Unit tests alone are insufficient for database client tools.

---

## Objectives

High-level objectives for this sprint:

1. **Fix all P0 bugs** preventing users from using core features
2. **Fix all P1 bugs** causing confusion and poor UX
3. **Establish live database testing** as mandatory quality gate
4. **Restore user trust** by delivering working, tested features
5. **Update sprint workflow** to prevent this from happening again

---

## Scope

### P0 - Critical (Must Have)

These bugs completely break user workflows and MUST be fixed.

#### Bug 1: Table Padding Completely Broken

**Issue:** Table output has no alignment, columns completely misaligned, unreadable output.

**User Impact:** CRITICAL - Users cannot read query results in table format. This is the default output format.

**Example:**
```
tq> sel * from dbc.databases;
╭──────────────────────────────────────────────────────────────┬──────────────────────────────────────────────────────────────┬──────────────────────────────────────────────────────────────┬──────────────────────────────────────────────────────────────┬────────────────┬─────────────┬─────────────┬─────────────┬─────────────┬───────────────────────────────────────────────────────────────────────────┬─────────────────────┬──────────────────────────────────────────────────────────────┬─────────────────────┬────────┬─────────────┬─────────────────────╮
│ DatabaseName                                                 ┆ CreatorName                                                  ┆ OwnerName                                                    ┆ AccountName                                                  ┆ ProtectionType ┆ JournalFlag ┆ PermSpace   ┆ SpoolSpace  ┆ TempSpace   ┆ CommentString                                                             ┆ CreateTimeStamp     ┆ LastAlterName                                                ┆ LastAlterTimeStamp  ┆ DBKind ┆ AccessCount ┆ LastAccessTimeStamp │
╞══════════════════════════════════════════════════════════════╪══════════════════════════════════════════════════════════════╪══════════════════════════════════════════════════════════════╪══════════════════════════════════════════════════════════════╪════════════════╪═════════════╪═════════════╪═════════════╪═════════════╪═══════════════════════════════════════════════════════════════════════════╪═════════════════════╪══════════════════════════════════════════════════════════════╪═════════════════════╪════════╪═════════════╪═════════════════════╡
│ val                                                          ┆ DBC                                                          ┆ system                                                       ┆ DBC                                                          ┆ F              ┆ NN          ┆   500000000 ┆ 99230829772 ┆ 99230829772 ┆ [NULL]                                                                    ┆ 2025-10-09 17:45:37 ┆ DBC                                                          ┆ 2025-10-09 17:45:37 ┆ U      ┆      [NULL] ┆ [NULL]              │
```
(Values not aligned with headers at all)

**Acceptance Criteria:**
- [ ] Column values align perfectly with column headers
- [ ] Table borders render correctly
- [ ] NULL values display as [NULL] and align properly
- [ ] Wide tables handle padding correctly
- [ ] Tested with real Teradata query results (DBC.TablesV, DBC.ColumnsV, etc.)
- [ ] Manual validation with live database confirms proper formatting

**Reference:** Sprint 6 supposedly fixed this but it's still broken

**Estimated Complexity:** High - This was supposedly "fixed" before, need to understand why it's still broken

**Testing Requirements:** MANDATORY live database testing with various table widths and data types

---

#### Bug 2: Tab Completion Doesn't Work At All

**Issue:** Tab completion for tables and columns completely non-functional. No feedback to user about what's happening.

**User Impact:** CRITICAL - Sprint 7's main feature (database-aware completion) doesn't work at all.

**Current Behavior:** User presses Tab, nothing happens. No completions, no error messages, no indication of activity.

**Expected Behavior:** Tab completion should show relevant database objects based on SQL context.

**Root Cause Analysis Needed:**
- Is the metadata query failing silently?
- Are we querying the right Teradata system tables?
- Are timeouts too aggressive?
- Is the SQL context detection working?

**Teradata-Specific Requirements (from user):**
- Tables in Teradata are fully qualified: `database.table`
- When unqualified, tables resolve to current database (`SELECT DATABASE`)
- Best practice: Always use fully qualified names
- **Smart completion strategy:**
  - After `FROM`: Show database names first, then tables in current database
  - After `FROM database.`: Show tables in that specific database
  - Lazy load: Only fetch database list + current DB tables initially
  - Cache per-database as user explores (typically 2-3 databases per session)
  - Refresh cache after successful DDL statements

**Acceptance Criteria:**
- [ ] Tab completion after FROM shows database names + current DB tables
- [ ] Tab completion after `FROM database.` shows tables in that database
- [ ] Lazy loading: Minimal metadata queries on REPL startup
- [ ] Intelligent caching: Per-database metadata cache
- [ ] Cache refresh after DDL statements (CREATE, DROP, ALTER)
- [ ] Visual feedback when loading metadata (spinner, status message)
- [ ] Graceful degradation if metadata query fails (show error, don't crash)
- [ ] Tested with real Teradata database with 1000+ tables
- [ ] Manual validation: User tests in real REPL session

**Reference:** Sprint 7 feature, needs complete redesign for Teradata's naming model

**Estimated Complexity:** High - Requires architectural changes to metadata caching strategy

**Testing Requirements:** MANDATORY live database testing with various database schemas

---

#### Bug 3: Dataset Paging with Arrows Doesn't Work

**Issue:** Result paging (horizontal/vertical navigation) completely non-functional.

**User Impact:** CRITICAL - Sprint 5's main feature (result paging) doesn't work. Users can't navigate large result sets or wide tables.

**Expected Behavior:**
- Vertical paging: j/k/PageUp/PageDown navigate long result sets
- Horizontal paging: h/l/arrow keys scroll wide tables

**Current Behavior:** Arrow keys don't work. No paging interface appears.

**Acceptance Criteria:**
- [ ] Vertical paging works for result sets > 25 rows
- [ ] Horizontal paging works for tables wider than terminal width
- [ ] Keyboard shortcuts work: j/k/PageUp/PageDown (vertical), h/l/arrows (horizontal)
- [ ] Pager shows position indicator (e.g., "Row 50 of 200", "Col 5 of 12")
- [ ] Exit pager with q or Esc
- [ ] `/pager on|off` metacommand controls paging behavior
- [ ] Tested with real query returning 1000+ rows
- [ ] Tested with query returning 20+ columns
- [ ] Manual validation: User confirms paging works in real REPL session

**Reference:** Sprint 5 feature, supposedly complete but doesn't work

**Estimated Complexity:** High - Core feature completely broken, needs investigation

**Testing Requirements:** MANDATORY live database testing with large result sets

---

### P1 - High Priority (Should Have)

#### Bug 4: Incorrect LIMIT Hint

**Issue:** When displaying 100+ rows, hint says "Add LIMIT clause" but Teradata doesn't support LIMIT syntax.

**User Impact:** HIGH - Confuses users, suggests invalid syntax. Teradata uses TOP or SAMPLE, not LIMIT.

**Current Message:**
```
Showing first 100 rows. Add LIMIT clause for different results.
```

**Correct Message:**
```
Showing first 100 rows. Use TOP N or SAMPLE N for different results.
```

**Additional Context:** Teradata SQL uses:
- `SELECT TOP 10 * FROM table` (not `SELECT * FROM table LIMIT 10`)
- `SELECT * FROM table SAMPLE 50` (alternative to TOP)

**Acceptance Criteria:**
- [ ] Hint message uses "TOP N or SAMPLE N" instead of "LIMIT"
- [ ] Message is clear and actionable
- [ ] Examples show Teradata syntax
- [ ] Help text (`/help`, `--help`) updated to use TOP/SAMPLE
- [ ] Documentation updated
- [ ] Manual validation with live database

**Reference:** Multiple sprints, incorrect assumption about SQL syntax

**Estimated Complexity:** Low - Simple text change, but need to verify all locations

**Testing Requirements:** Visual inspection + manual validation

---

### P2 - Medium Priority (Nice to Have)

No P2 items in this sprint. Focus is on fixing critical bugs only.

---

### Explicitly Out of Scope

Things we are intentionally NOT doing in this sprint:

- **New features** - No new feature development until quality is restored
- **Performance optimization** - Focus is on correctness, not performance
- **Batch mode features** - Deferred to future sprint
- **Configuration file support** - Deferred to future sprint
- **Additional completion features** (functions, schemas) - Fix core completion first

**Rationale:** We must restore user trust by fixing broken features before adding new ones.

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] All P0 bugs are fixed and validated with live Teradata database
- [ ] All P1 bugs are fixed and validated with live Teradata database
- [ ] **NEW:** All test cases executed manually against live database and results documented
- [ ] 100% test pass rate (unit + integration tests)
- [ ] **NEW:** Manual integration tests with live database pass 100%
- [ ] Documentation updated to reflect fixes
- [ ] Zero technical debt introduced
- [ ] Code quality meets project standards (per rust-architecture.md)
- [ ] All features validated by quality-validator agent
- [ ] Completion validated by tq-project-manager agent
- [ ] **NEW:** User confirms all bugs are fixed in production environment

---

## Dependencies

### External Dependencies
- **CRITICAL:** Live Teradata database instance for testing
- User must provide database connection details in `.env` file
- Database must be accessible and responsive

### Prerequisite Work
- Database connectivity verification (Phase 3.5 of sprint workflow)
- `.env` file configured with `TQ_LOGON` environment variable
- Test database with sufficient data for realistic testing:
  - Multiple databases (3+)
  - Multiple tables per database (100+)
  - Large tables (1000+ rows)
  - Wide tables (20+ columns)

### Blockers
- **Blocker 1:** No live database available
  - **Mitigation:** User MUST provide test database access before Phase 4 (Testing)
  - **Action:** Verify database connectivity in Phase 3.5

- **Blocker 2:** Current implementation fundamentally incompatible with Teradata
  - **Mitigation:** Rust-teradata-architect will investigate and propose architectural changes if needed
  - **Action:** Early investigation in Phase 2 (Design)

---

## Risks & Mitigation

### Risk 1: Bugs Cannot Be Reproduced in Test Environment

- **Probability:** Medium
- **Impact:** High
- **Mitigation:**
  - User provides specific reproduction steps
  - Test against same Teradata version as production
  - User validates fixes before sprint closure

### Risk 2: Architectural Changes Required

- **Probability:** High (especially for tab completion)
- **Impact:** High (extends implementation time)
- **Mitigation:**
  - Rust-teradata-architect assesses in Phase 2
  - Adjust scope if needed, prioritize P0 bugs
  - May need to split tab completion fix into separate sprint if too complex

### Risk 3: More Undiscovered Bugs

- **Probability:** High (if these bugs slipped through, others might too)
- **Impact:** Medium (damages user trust further)
- **Mitigation:**
  - Comprehensive manual testing of ALL features
  - User performs acceptance testing before closure
  - Create comprehensive bug regression test suite

### Risk 4: Database Unavailable for Testing

- **Probability:** Low
- **Impact:** Critical (sprint cannot complete)
- **Mitigation:**
  - Verify database connectivity at sprint start (Phase 3.5)
  - User commits to keeping database available during sprint
  - Have fallback test database ready

---

## Action Items from Previous Sprint

Items from Sprint 7 review that are now CRITICAL:

- [x] **Execute Manual Interactive Tests** - This was deferred, causing the quality failure
  - TC026-TC043 designed but never executed
  - This MUST happen in Sprint 8 for all features
  - Reference: `sprint-7-review.md` section "What Could Be Improved"

- [x] **Monitor Production Performance** - Irrelevant until features actually work
  - Deferred until bugs are fixed
  - Reference: `sprint-7-review.md` recommendations

- [ ] **Update Sprint Workflow** - Add mandatory live database testing phase
  - Current workflow has Phase 3.5 (Database Check) but testing is optional
  - MUST make live database testing mandatory, not optional
  - Update `.claude/skills/sprint-coordinator/SKILL.md`

**Reference:** [Sprint 7 Review](sprint-7-review.md)

---

## Agent Assignments

### cli-ux-designer (Sonnet)

**Responsibilities:**
- Design fix for Bug 4 (LIMIT hint) - P1
- Review and improve error messages for tab completion (show what's happening)
- Design visual feedback for metadata loading
- Update specifications to reflect Teradata-specific completion behavior
- Update `specifications.md` to mark Sprint 5, 6, 7 features as 🔧 In Repair

**Deliverables:**
- Updated `specifications.md` with 🔧 status for broken features
- Detailed specification for Teradata-specific tab completion
- Updated hint messages in `detailed-specifications/repl-mode.md`
- UX guidelines for metadata loading feedback

---

### rust-teradata-architect (Opus)

**Responsibilities:**
- Investigate and fix Bug 1 (table padding) - P0
- Investigate and fix Bug 2 (tab completion) - P0
- Investigate and fix Bug 3 (result paging) - P0
- Implement Bug 4 fix (LIMIT hint) - P1
- Write/fix unit tests for all bug fixes
- Update `rust-architecture.md` if architectural changes needed
- Document root cause for each bug

**Deliverables:**
- Working fixes for all 4 bugs
- Root cause analysis document for each bug
- Updated unit tests with 100% pass rate
- Updated `rust-architecture.md` if needed
- Technical debt report (should be zero)

---

### quality-validator (Sonnet)

**Responsibilities:**
- Design comprehensive manual test cases for all 4 bugs
- Execute all test suites (unit + integration) against live database
- **NEW:** Execute ALL manual test cases from Sprints 5, 6, 7 against live database
- Generate test reports in `tests/results/`
- Validate acceptance criteria for all bug fixes
- Create regression test suite to prevent these bugs from recurring

**Deliverables:**
- Test cases for Bug 1-4 in `tests/cases/TC###.md`
- **NEW:** Execution results for TC026-TC043 (Sprint 7 tests never run)
- **NEW:** Execution results for Sprint 5 paging tests
- **NEW:** Execution results for Sprint 6 formatting tests
- Test execution report in `tests/results/YYYYMMDD-HHMMSS/REPORT.md`
- 100% test pass rate (unit + integration + manual)
- Regression test suite document

---

### tq-project-manager (Haiku)

**Responsibilities:**
- Validate sprint completion at closure
- Assess technical debt status
- Verify all bugs are actually fixed (not just "tests pass")
- **NEW:** Verify manual testing was comprehensive and documented
- Provide go/no-go decision for sprint closure
- Recommend sprint workflow improvements

**Deliverables:**
- Sprint completion validation report
- Technical debt assessment
- Manual testing validation (confirms quality-validator executed all tests)
- Go/no-go recommendation
- Recommendations for preventing future quality failures

---

## Sprint Timeline

**Estimated Duration:** 2-3 days (longer due to complexity of fixes)

### Phase Breakdown

- **Phase 1: Planning** (Complete)
  - Sprint planning document created
  - User approval obtained

- **Phase 2: Design & Investigation** (Est. 4-6 hours)
  - Parallel execution: cli-ux-designer + rust-teradata-architect
  - **Critical:** Rust-teradata-architect investigates root cause of each bug
  - Determine if architectural changes needed
  - Finalize fix approach for each bug

- **Phase 3: Implementation** (Est. 8-12 hours)
  - Parallel execution: rust-teradata-architect + quality-validator
  - Rust-teradata-architect implements fixes
  - Quality-validator designs comprehensive test cases

- **Phase 3.5: Database Connectivity Check** (MANDATORY)
  - Verify live database is accessible
  - Run `./target/release/tq ping`
  - If fails: STOP and wait for user to provide working database
  - Only proceed when database confirmed working

- **Phase 4: Testing** (Est. 6-8 hours)
  - Quality-validator executes all automated tests (unit + integration)
  - **NEW:** Execute ALL deferred manual tests from Sprints 5, 6, 7
  - **CRITICAL:** If manual tests cannot be executed programmatically:
    - Sprint Coordinator MUST pause workflow
    - Provide user with detailed test instructions (step-by-step)
    - Wait for user to execute tests and report results
    - Document user's test results
    - Sprint CANNOT proceed without user's test completion
  - **NEW:** User performs acceptance testing
  - 100% pass rate required before closure
  - Fix loop: If any test fails, return to Phase 3

- **Phase 5: Closure** (Est. 2-3 hours)
  - Tq-project-manager validates completion
  - Sprint review created
  - Roadmap updated
  - User confirms all bugs fixed in production

---

## Quality Gate: Mandatory Live Database Testing

**NEW REQUIREMENT:** No sprint can be marked "complete" without executing manual integration tests against a live Teradata database.

**Testing Requirements:**
1. All test cases in `tests/cases/` must be executed and results documented
2. **CRITICAL:** If tests require manual execution (REPL interaction, visual validation):
   - Sprint Coordinator MUST pause workflow at Phase 4
   - Provide user with step-by-step test instructions
   - Wait for user to execute tests and provide results
   - Document user's test results in test report
   - Sprint CANNOT proceed until user completes tests
3. User must validate fixes in their production environment
4. Quality-validator must document all test execution (automated + user-executed)
5. Tq-project-manager must verify manual testing occurred before approving closure

**Consequences of Skipping:** If manual testing is skipped, sprint CANNOT be marked complete.

---

## Sprint Workflow Improvements

Based on this quality failure, we will update the sprint workflow:

**Changes to `.claude/skills/sprint-coordinator/SKILL.md`:**

1. **Phase 3.5 (Database Check):** Change from optional to MANDATORY
   - Current: "If database is unavailable, ask user to start it"
   - NEW: "Sprint CANNOT proceed to Phase 4 without verified database connectivity"

2. **Phase 4 (Testing):** Add manual testing requirement
   - Current: "quality-validator executes test suites"
   - NEW: "quality-validator executes test suites (unit + integration + MANUAL)"
   - NEW: "All manual test results must be documented with screenshots/logs"
   - NEW: "User must validate fixes in production environment"

3. **Phase 5 (Closure):** Add manual testing validation
   - Current: "tq-project-manager validates completion"
   - NEW: "tq-project-manager validates completion AND manual testing was performed"
   - NEW: "User must confirm bugs are fixed before sprint closure"

4. **Success Criteria:** Add manual testing checkpoint
   - NEW: "100% manual test pass rate documented"
   - NEW: "User acceptance testing completed"

---

## Notes

**Critical Context:**

This sprint is about restoring trust and quality. We've failed our users by shipping broken features marked as "complete with 100% test pass rate."

**Lessons for the Team:**
1. Unit tests alone are insufficient for database client tools
2. Integration testing against real databases is mandatory, not optional
3. "All tests passing" doesn't mean features work if tests don't cover real usage
4. Manual validation by users is critical for quality

**Accountability:**
- Quality-validator: Should have insisted on manual testing before marking sprints complete
- Rust-teradata-architect: Should have validated implementation against real database
- Tq-project-manager: Should have caught the gap between "tests pass" and "features work"
- Sprint Coordinator (me): Should have enforced mandatory live database testing

**Going Forward:**
- Live database testing is now a mandatory quality gate
- No sprint is complete without manual test execution and documentation
- User acceptance testing is required before closure

---

## Approval

**Status:** Approved

**Approved By:** User
**Approval Date:** 2026-01-18

**Questions for User:**
1. Can you provide live Teradata database access for testing? (Required)
2. Are there other bugs we haven't identified yet?
3. What's your priority order if we can't fix all bugs in one sprint?
4. Can you perform acceptance testing when fixes are ready?

**Revisions Requested:**
- [Awaiting user feedback]

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-18 | 1.0 | Initial sprint 8 plan - Quality Recovery | Sprint Coordinator (Main Agent) |
