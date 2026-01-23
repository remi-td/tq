---
sprint: 21
start_date: 2026-01-23
target_completion: 2026-01-24
status: Planning
---

# Sprint 21 Planning: Tab Completion Quality & Data Completeness

## Sprint Overview

**Sprint Goal:** Enhance tab completion UX, ensure complete metadata fetching, and establish automated regression testing for tab completion features.

**Sprint Theme:** Tab Completion Quality - Following Sprint 20's successful fix of the pager banner bug, this sprint addresses three newly discovered quality-of-life issues in tab completion: missing database metadata, unintuitive TAB key behavior, and incomplete table fetching across all databases.

---

## Reality Check Summary

**Reviewed Sprints:** Sprint 17, 19, 20

**Patterns Detected:**
- Sprint 20 successfully fixed the pager banner bug after 3 iterations ("congratulations for fixing it after 10 sprints!")
- User now exploring deeper functionality and discovering UX/data issues
- User explicitly requests automated regression testing capability
- Healthy progression: critical bug fixed → user explores → finds improvements

**Decision:** Feature Sprint

**Rationale:** Sprint 20 resolved the critical pager output bug (user validated). The three issues reported are NEW quality-of-life improvements discovered during normal use, not regressions. This represents healthy user engagement with the feature. The user's request for automated regression testing indicates maturity requirement, not crisis.

---

## Objectives

High-level objectives for this sprint:

1. **Complete Database Metadata Coverage** - Ensure ALL databases (including system databases like `dbc`) are included in tab completion
2. **Improve Tab Completion UX** - Make second TAB accept selection (like bash/zsh), implement smart database.table completion
3. **Establish Automated Regression Testing** - Create comprehensive automated tests to prevent future tab completion regressions

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Complete Database Metadata Fetching

**Description:** Ensure ALL databases on the system are fetched and cached for tab completion, including system databases like `dbc` that were previously missing.

**User Issue:**
> "If I do `sel * from `+TAB I get a list of many databases, it should contain all databases on the system, but I noticed that I am using the dbc one!!! Make sure all databases are included"

**Acceptance Criteria:**
- [ ] System database `dbc` appears in database completion list
- [ ] ALL databases on Teradata system are fetched (not filtered by access rights during fetch)
- [ ] Query used to fetch databases returns complete system catalog
- [ ] Manual validation: User confirms `dbc` appears in completion menu
- [ ] Automated test: Verify `dbc` present in test database completion list

**Reference:** `docs/specifications/repl.md#database-completion`

**Estimated Complexity:** Low (SQL query modification)

**Root Cause:** Current query may filter system databases or use wrong system catalog view.

---

#### Feature 2: Universal Table Metadata Fetching

**Description:** Ensure table metadata is fetched for ALL databases, not just some. Fix "NO RECORDS FOUND" issue for databases like `demo_user` that have tables but aren't cached.

**User Issue:**
> "Some databases objects are not cached/fetched. For example: `tq> | sel * from demo_user.` → NO RECORDS FOUND. I know that there are three tables in this database, but it should be fetched!"

**Acceptance Criteria:**
- [ ] Metadata fetch attempts to load tables for ALL databases (not just a subset)
- [ ] `demo_user` database tables appear in completion (user validation)
- [ ] Completion shows tables after typing `database.` + TAB
- [ ] Error handling: graceful degradation if permission denied for specific database
- [ ] Automated test: Verify tables fetched for multiple databases including edge cases

**Reference:** `docs/specifications/repl.md#table-completion`

**Estimated Complexity:** Medium (may require architecture change to on-demand fetching)

**Root Cause:** Metadata loading may be limited to specific databases or failing silently for some databases.

---

### P1 - High Priority (Should Have)

#### Feature 3: Second TAB Accepts Selection

**Description:** Change TAB key behavior to match bash/zsh standards: first TAB shows menu, second TAB accepts highlighted item (currently second TAB moves down).

**User Issue:**
> "When we hit tab the first time, the object menu is displayed, which is OK. But when we hit tab a second time, the cursor select the next object (down) which is unintuitive (the down arrow is for this), typically a second tab hit validates the completion with the highlighted object (same as enter)."

**Acceptance Criteria:**
- [ ] First TAB: Show completion menu with first item highlighted
- [ ] Second TAB: Accept highlighted item and insert into command line
- [ ] DOWN arrow: Move to next item in menu
- [ ] UP arrow: Move to previous item in menu
- [ ] ENTER: Accept highlighted item
- [ ] Behavior matches bash/zsh completion UX
- [ ] Manual validation: User confirms intuitive behavior
- [ ] Automated test: Verify TAB key behavior in PTY simulation

**Reference:** `docs/specifications/repl.md#tab-completion-behavior`

**Estimated Complexity:** Medium (reedline configuration or custom completer behavior)

**Technical Note:** This is a ColumnarMenu behavior configuration issue. Need to investigate reedline's menu navigation settings.

---

#### Feature 4: Smart Database-Dot-TAB Completion

**Description:** When user types `database.` and hits TAB, automatically complete the database name (if unambiguous), add the dot, and immediately show tables in that database.

**User Issue:**
> "Also, when I hit tab on a database after a FROM/JOIN, I would expect to complete the database name, add a '.' and prompt the list of tables in this database directly."

**Acceptance Criteria:**
- [ ] Typing `dem` + TAB completes to `demo_user.` (if unambiguous)
- [ ] After completing `demo_user.`, immediately show tables in `demo_user` database
- [ ] If ambiguous (multiple matches), show database list first
- [ ] Works after FROM keyword
- [ ] Works after JOIN keyword
- [ ] Manual validation: User confirms smooth workflow
- [ ] Automated test: Verify multi-stage completion behavior

**Reference:** `docs/specifications/repl.md#qualified-name-completion`

**Estimated Complexity:** High (requires context-aware completion logic)

**Technical Note:** May require changes to completer state management to track partial completion state.

---

### P2 - Medium Priority (Nice to Have)

#### Feature 5: Automated Tab Completion Regression Tests

**Description:** Create comprehensive automated test suite for tab completion to prevent future regressions, as explicitly requested by user.

**User Request:**
> "Make sure that you know how to test this for regression automatically."

**Acceptance Criteria:**
- [ ] Unit tests for metadata fetching logic (databases, tables)
- [ ] Integration tests for completion suggestions at various SQL positions
- [ ] PTY-based tests for menu display and navigation (where possible)
- [ ] Negative tests: verify no pager output, no error messages during completion
- [ ] Test coverage: database completion, table completion, column completion, qualified names
- [ ] CI/CD compatible test suite
- [ ] Documentation: `docs/testing/approach.md` updated with tab completion test patterns
- [ ] Test execution in Sprint 21 Phase 3: 100% pass rate

**Reference:** `docs/testing/approach.md#interactive-feature-testing`

**Estimated Complexity:** High (comprehensive test design across multiple layers)

**Sprint 20 Lesson:** Hybrid testing required (automated + manual) because automated tests gave false positives in Sprints 18-20.

---

### Explicitly Out of Scope

Things we are intentionally NOT doing in this sprint:

- **Column completion enhancements** - Existing column completion works, not touched in this sprint
- **Keyword completion changes** - Working as designed, no changes needed
- **Metacommand completion** - Deferred to future sprint (backlog item)
- **Completion performance optimization** - Current performance acceptable, focus on correctness first
- **Alternative completion menu styles** - ColumnarMenu is correct choice (Sprint 20 validation)

**Rationale:** Focus on the three specific user-reported issues plus regression testing. Avoid scope creep.

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] All P0 features are implemented, tested, and working as specified
- [ ] All P1 features are implemented and tested (or explicitly moved to next sprint)
- [ ] User manually validates all three reported issues are resolved
- [ ] 100% automated test pass rate (unit + integration tests)
- [ ] All acceptance criteria met for delivered features
- [ ] `docs/specifications/repl.md` updated with new completion behaviors
- [ ] `docs/design/repl.md` updated with implementation details
- [ ] Zero technical debt introduced
- [ ] Code quality meets project standards (per `docs/design/*.md`)
- [ ] quality-validator APPROVED verdict (automated + manual validation)
- [ ] tq-project-manager completion validation passed

---

## Dependencies

### External Dependencies
- **reedline library** (v0.37.0+) - Tab completion menu behavior configuration
- **Teradata system catalog** - DBC.DatabasesV, DBC.TablesV views for metadata
- **PTY testing framework** - For automated tab completion behavior tests

### Prerequisite Work
- ✅ Sprint 20: Tab completion pager banner bug fixed (ColumnarMenu implementation)
- ✅ Sprint 20: Metadata caching infrastructure in place (`src/db/metadata.rs`)
- ✅ Sprint 19/20: `OutputSuppressor` pattern for database query isolation

### Blockers
- **NONE IDENTIFIED** - All prerequisites complete, no known blockers

---

## Risks & Mitigation

### Risk 1: reedline May Not Support Custom TAB Behavior
- **Probability:** Medium
- **Impact:** High (blocks P1 Feature 3)
- **Mitigation:** Early investigation in Phase 2 (Design). If reedline doesn't support, evaluate: (1) fork reedline, (2) submit upstream PR, (3) defer feature to next sprint with explanation to user.

### Risk 2: System Database Access May Be Restricted
- **Probability:** Low
- **Impact:** Medium (affects P0 Feature 1)
- **Mitigation:** Graceful degradation - if `dbc` access denied, log warning but don't fail. Test with user's actual Teradata environment during manual validation.

### Risk 3: On-Demand Table Fetching May Impact Performance
- **Probability:** Medium
- **Impact:** Low (UX delay)
- **Mitigation:** If Feature 2 requires on-demand fetching (vs. startup caching), implement with async background fetch and show "Loading..." indicator. Measure performance impact during testing.

### Risk 4: Automated Tests May Give False Positives Again
- **Probability:** High (Sprint 18/20 pattern)
- **Impact:** High (ship wrong fixes)
- **Mitigation:** Hybrid testing mandatory (Sprint 20 lesson learned). Every automated test must have corresponding manual validation procedure. User validation required before sprint closure.

---

## Action Items from Previous Sprint

Items carried over from Sprint 20 retrospective:

- [x] **Standardize Hybrid Testing Pattern** - Sprint 21 will implement this through Feature 5 (automated tests) + mandatory manual validation gates
- [x] **Add User Validation Gate to Definition of Done** - Added to Success Criteria above: "User manually validates all three reported issues are resolved"
- [ ] **Create Visual Specification Capture Protocol** - NOT APPLICABLE to Sprint 21 (no visual specs needed for tab completion logic)
- [ ] **Document Multi-Layer Debugging Approach** - DEFERRED to Sprint 22+ (no active debugging crisis)
- [ ] **Investigate Visual Regression Testing Tools** - PARTIALLY ADDRESSED through Feature 5 (automated PTY tests), full visual regression deferred
- [ ] **Add UI Component Tests** - ADDRESSED through Feature 5 (tab completion menu behavior tests)

**Reference:** `docs/sprints/sprint-20-review.md` Section 7 (Recommendations)

**Note:** Hybrid testing and user validation recommendations directly inform Sprint 21's approach.

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Design Features: ALL (1-5) require UX specification updates
- Update `docs/specifications/repl.md` with new completion behaviors
- Define intuitive TAB key interaction patterns (Feature 3)
- Specify smart database.table completion workflow (Feature 4)
- Ensure consistency with bash/zsh completion UX standards

**Deliverables:**
- Updated `docs/specifications/repl.md` (v2.x) with:
  - Database completion requirements (include system databases)
  - Table fetching requirements (all databases, graceful degradation)
  - TAB key behavior specification (second TAB accepts)
  - Qualified name completion workflow (database.TAB → tables)
  - Completion menu interaction patterns
- UX validation report for all features

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement Features: 1, 2, 3, 4 (P0/P1)
- Investigate reedline TAB behavior configuration (Risk 1 mitigation)
- Update metadata fetching queries (Feature 1: include `dbc`)
- Design on-demand or universal table fetching (Feature 2)
- Configure ColumnarMenu for custom TAB behavior (Feature 3)
- Implement context-aware qualified name completion (Feature 4)
- Write unit tests for all new completion logic
- Update `docs/design/repl.md` with architecture changes

**Deliverables:**
- Working implementation of Features 1-4
- Unit tests with 100% pass rate
- Updated `docs/design/repl.md` with:
  - Metadata fetching architecture (database/table queries)
  - ColumnarMenu configuration details
  - Completer state management for multi-stage completion
  - Performance considerations for on-demand fetching
- Technical debt assessment (should be ZERO)

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design comprehensive test strategy (Feature 5)
- Create test cases for Features 1-4 (automated + manual components)
- Execute all test suites (unit + integration + manual)
- Generate hybrid test reports with automated results + manual validation evidence
- Validate all acceptance criteria
- Provide APPROVED/REJECTED/BLOCKED verdict with clear justification

**Deliverables:**
- Test strategy document: `tests/strategy/sprint-21-test-strategy.md`
- Test cases: `tests/cases/TC-TAB-COMPLETION-*.md` (at least 5 new test cases)
- Automated test execution report: `tests/results/sprint-21/REPORT.md`
- Manual validation evidence: `tests/results/sprint-21/manual-validation.md` (screenshots, user confirmation)
- 100% automated test pass rate
- User validation confirmation for all three reported issues
- APPROVED verdict only if BOTH automated AND manual tests pass

---

### tq-project-manager (Haiku)
**Responsibilities:**
- Validate sprint completion at closure (Phase 4)
- Verify all user-reported issues resolved (dbc missing, TAB behavior, demo_user tables)
- Assess technical debt status (should be ZERO)
- Verify documentation synchronized (specs, design, tests)
- Provide go/no-go decision for sprint closure based on Definition of Done

**Deliverables:**
- Sprint completion validation report
- User issue resolution confirmation (all 3 issues)
- Technical debt assessment (ZERO target)
- Documentation sync verification
- Go/no-go recommendation with clear rationale
- Recommendations for Sprint 22 (if any issues deferred)

---

## Sprint Timeline

**Estimated Duration:** 1-2 days

### Phase Breakdown

- **Phase 0: Reality Check** (✅ COMPLETE)
  - Sprint 17, 19, 20 reviews analyzed
  - Pattern detection: healthy progression, not crisis
  - Decision: Feature Sprint

- **Phase 1: Planning** (✅ COMPLETE)
  - Sprint planning document created
  - Proceeding autonomously to Phase 2

- **Phase 2: Design** (Est. 4-6 hours)
  - **PARALLEL EXECUTION:**
    - `cli-ux-designer`: Update specifications for all 5 features
    - `rust-teradata-architect`: Investigate reedline TAB behavior (Risk 1), design metadata fetching changes
  - Specifications finalized with clear acceptance criteria
  - Risk 1 (reedline limitations) assessed with mitigation plan

- **Phase 3: Build & Test** (Est. 8-12 hours)
  - **PARALLEL EXECUTION:**
    - `rust-teradata-architect`: Implement Features 1-4, write unit tests
    - `quality-validator`: Design test cases (Feature 5), prepare manual validation procedures
  - **ITERATION LOOP (if needed):**
    - quality-validator executes automated tests
    - quality-validator performs manual validation (or requests user validation)
    - If FAIL: rust-teradata-architect fixes issues
    - Repeat until 100% pass rate achieved
  - Target: 100% automated test pass + user validation of all 3 issues

- **Phase 4: Ship** (Est. 2-3 hours)
  - tq-project-manager validates sprint completion
  - Verify Definition of Done checklist (all boxes checked)
  - Git commit with comprehensive message
  - Update `docs/roadmap/status.md` to reflect v1.7.2 or v1.8.0 (decision in Phase 2)
  - Git push to master

- **Phase 5: Retrospective** (Est. 3-4 hours)
  - **PARALLEL EXECUTION:**
    - Technical review by rust-teradata-architect
    - Quality review by quality-validator
    - UX review by cli-ux-designer
  - Create consolidated `docs/sprints/sprint-21-review.md`
  - Identify lessons learned and recommendations for Sprint 22

---

## Notes

**Key Insights from Sprint 20:**

1. **Hybrid Testing is Mandatory** - Sprint 20 required 3 iterations because automated tests passed but bugs persisted. Sprint 21 MUST implement hybrid testing (automated + manual) from the start.

2. **User Validation is Non-Negotiable** - For user-reported issues, user must validate fixes before sprint closure. Sprint 21 will require user to test all three issues.

3. **False Positives Are Real** - Automated tests validate CODE behavior, not USER EXPERIENCE. Sprint 21's automated tests must be designed with this limitation in mind.

4. **Persistence Pays Off** - Sprint 20 took 3 iterations but delivered correct fixes. Sprint 21 should iterate if needed, never ship wrong fixes.

**User Context:**

- User is technically sophisticated (Teradata expert)
- User is actively using the tool and providing detailed feedback
- User understands the difficulty ("congratulations for fixing it after 10 sprints!")
- User explicitly requests automated regression testing (showing maturity requirement)
- User's feedback is constructive, not critical

**Version Numbering Decision:**

- Sprint 20 was v1.7.1 (patch for bug fixes)
- Sprint 21 adds new UX behaviors (second TAB accepts, smart database.table completion)
- Decision: v1.8.0 (minor version bump for behavior changes)
- Will be finalized in Phase 2 based on scope delivered

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-23 | 1.0 | Initial sprint plan - Tab Completion Quality & Data Completeness | Sprint Coordinator |
