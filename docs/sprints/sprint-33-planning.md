---
sprint: 33
start_date: 2026-02-03
target_completion: 2026-02-03
status: Planning
---

# Sprint 33 Planning: Pager Bug Fix + Data Sampling

## Reality Check Summary
- **Reviewed sprints:** 32, 31, 29
- **Patterns detected:** Framework healthy, Sprint 32 demonstrated maturity
- **Critical finding:** Issue #14 reports pager still broken (contradicts Sprint 31/32 claims)
- **Decision:** MIXED SPRINT (P0 Bug Fix + P0 Business Feature)
- **Rationale:** Must fix pager bug AND deliver user value to maintain momentum

## Sprint Overview

**Sprint Goal:** Fix pager rendering bug and deliver data exploration feature for business value

**Sprint Theme:** Bug fix + user value - Stabilize pager (disabled by default) while adding fast data sampling commands for exploratory workflows

**Sprint Type:** Mixed (Bug Fix + Feature)

---

## Objectives

1. **Fix Pager Rendering Bug** - Resolve column alignment/line break issues reported in Issue #14
2. **Disable Pager by Default** - Set pager_enabled: false regardless of fix to prevent bad user experience
3. **Deliver Data Sampling Feature** - Add `/sample` and `/peek` commands for fast data exploration
4. **Maintain Quality Standards** - 100% test pass rate, comprehensive validation, zero regressions

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Pager Bug Fix (Issue #14)

**GitHub Issue:** #14 - [BUG] Pager broken and on by default

**Description:**
Fix column alignment and line break issues in interactive pager. Sprint 31 implemented two-pass truncation fix, but Issue #14 reports pager still produces garbled output. Root cause analysis required.

**Acceptance Criteria:**
- [ ] **AC-1:** Root cause identified - Debug why Sprint 31's two-pass truncation didn't resolve the issue
- [ ] **AC-2:** Fix implemented - Correct rendering logic to prevent column overflow
- [ ] **AC-3:** Default disabled - `pager_enabled: false` in `src/commands/repl/state.rs` regardless of fix status
- [ ] **AC-4:** Unit tests pass - All existing pager tests (27 tests) pass at 100%
- [ ] **AC-5:** Integration tests pass - No regressions in interactive tests (48 tests)
- [ ] **AC-6:** Manual test case documented - Create TC-033-PAGER-MANUAL.md with validation steps (even if not executed)
- [ ] **AC-7:** User can enable if desired - `/pager on` command still works for users who want to test
- [ ] **AC-8:** Documentation updated - Update user guide to note pager is experimental, off by default
- [ ] **AC-9:** GitHub issue updated - Comment on #14 with findings and fix details
- [ ] **AC-10:** Zero new regressions - Existing functionality unaffected

**Reference:**
- Specification: `docs/specifications/repl.md#horizontal-column-navigation`
- Design: `docs/design/repl.md#pager-architecture`
- Sprint 31 Fix: `docs/sprints/sprint-31-review.md` (two-pass truncation)
- Sprint 30 Crisis: `docs/sprints/sprint-30-review.md` (architectural refactor)

**Estimated Complexity:** MEDIUM (root cause analysis required, but fix likely small)

**Constraints:**
- No human testing available - Ship based on automated tests + code review
- Must disable by default regardless of confidence in fix
- Cannot claim "pager works" without manual validation (Sprint 31 lesson)

---

#### Feature 2: Data Sampling Commands

**Description:**
Add fast data exploration commands for sampling table data without writing full SQL queries. Targets data analyst and DBA personas who need quick data inspection during REPL sessions.

**Commands:**
- `/sample <table> [n]` - Show random N rows from table (default: 10)
- `/peek <table>` - Show first 5 rows with column info (quick preview)

**Acceptance Criteria:**
- [ ] **AC-1:** `/sample` command implemented - Accepts table name, optional row count
- [ ] **AC-2:** Default sample size - 10 rows if count not specified
- [ ] **AC-3:** Sample size validation - Max 1000 rows (prevent accidental large queries)
- [ ] **AC-4:** Random sampling - Use Teradata SAMPLE clause for true random sampling
- [ ] **AC-5:** `/peek` command implemented - Shows first 5 rows + column metadata
- [ ] **AC-6:** Column info display - Show data types, nullable, precision for `/peek`
- [ ] **AC-7:** Tab completion - Both commands in metacommand completion menu
- [ ] **AC-8:** Error handling - Clear messages for invalid tables, permissions, syntax
- [ ] **AC-9:** Multi-format support - Respect current output format (table/csv/json)
- [ ] **AC-10:** Help text updated - `/help` shows both commands with examples
- [ ] **AC-11:** Batch mode integration - `tq sample <table>` and `tq peek <table>` commands
- [ ] **AC-12:** Qualified names - Support database.tablename syntax
- [ ] **AC-13:** Performance - Fast execution even on large tables (SAMPLE is efficient)
- [ ] **AC-14:** Documentation complete - User guide, specifications, design docs updated
- [ ] **AC-15:** 100% test coverage - Unit tests + interactive tests for both commands

**Reference:**
- Specification: `docs/specifications/repl.md#data-sampling` (to be created)
- Backlog: `docs/roadmap/backlog.md` (P2 - Data Sampling Commands)

**Estimated Complexity:** MEDIUM

**User Value:** HIGH - Addresses data exploration use case, complements `/list` commands from Sprint 22

---

### Explicitly Out of Scope

Items intentionally NOT included in Sprint 33:

- **Pager removal** - Not removing feature, just disabling by default and fixing if possible
- **Pager full manual validation** - No human testing available, shipping based on automated validation
- **Search in pager** - Deferred to future sprint (P2 backlog item)
- **Additional sampling options** - No stratified sampling, filtering, or advanced options (keep simple)
- **Data export from sample** - Use `/export` on the sample query instead
- **Performance benchmarking** - No criterion benchmarks (low priority for sampling commands)

**Rationale:** Sprint 33 must deliver ONE bug fix + ONE feature for user value. Scope is constrained to ensure both objectives are met with high quality.

---

## GitHub Issues

### Selected for Sprint
- **#14:** [BUG] Pager broken and on by default (priority-high, bug)
  - Status: sprint-ready
  - Will be fixed in this sprint (disable by default + attempt repair)

### Deferred
- No other open issues at this time

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] **Pager disabled by default** - User experience protected from broken feature
- [ ] **Pager bug root cause identified** - Understanding of why Sprint 31 fix didn't work
- [ ] **Fix attempted and validated** - Code changes made, tested, documented honestly
- [ ] **Data sampling commands working** - `/sample` and `/peek` fully functional
- [ ] **100% test pass rate** - All 394+ tests passing (unit + integration + new tests)
- [ ] **All 25 acceptance criteria met** - 10 for pager, 15 for data sampling
- [ ] **Documentation complete** - Specs, design, user guide, test cases all updated
- [ ] **Zero technical debt introduced** - Clean implementation, no TODOs or workarounds
- [ ] **GitHub issue #14 updated** - Status commented with findings and resolution
- [ ] **Honest assessment** - Sprint review reflects actual status, not aspirational claims

---

## Dependencies

### External Dependencies
- Teradata database connection (for testing sampling commands)
- teradatarustapi crate (already integrated)
- No new external dependencies required

### Prerequisite Work
- Sprint 32 content-based column width feature (✅ Complete)
- Sprint 31 pager bug fix attempt (✅ Complete, but didn't work)
- Tab completion framework (✅ Complete, Sprint 22)

### Blockers
- **No manual pager testing available** - Shipping without human validation
  - **Mitigation:** Disable by default, document limitation, rely on user feedback
- **Pager root cause may be complex** - Could require deep debugging
  - **Mitigation:** Time-box investigation, disable if fix too complex

---

## Risks & Mitigation

### Risk 1: Pager root cause not found in time
- **Probability:** MEDIUM
- **Impact:** LOW (pager disabled by default anyway)
- **Mitigation:** Time-box investigation to 3-4 hours. If not resolved, document findings and defer to future sprint. User experience protected by default-off setting.

### Risk 2: Data sampling commands more complex than expected
- **Probability:** LOW
- **Impact:** MEDIUM (feature incomplete)
- **Mitigation:** Start with minimal implementation (`/sample` only), defer `/peek` to P1 if needed. Leverage existing `/describe` code for column metadata.

### Risk 3: Shipping pager without manual validation (again)
- **Probability:** HIGH (no human testing)
- **Impact:** LOW (disabled by default)
- **Mitigation:** Apply Sprint 31 lesson - be honest about validation status. Document that pager is experimental, disabled by default, user feedback needed.

---

## Action Items from Previous Sprint

Items from Sprint 32 retrospective:

- [ ] **Verify GitHub README display** (Feature #12) - Visit https://github.com/remi-td/tq and confirm root README displays
  - **Sprint 33 Action:** Quick verification during Phase 1 (2 minutes)
  - **Reference:** `sprint-32-review.md` Section 8 (Actions Required)

- [ ] **Add Unicode width fix to backlog** - Unify Unicode handling (table.rs vs pager.rs)
  - **Sprint 33 Action:** Not in scope for Sprint 33, remains in backlog as P2
  - **Reference:** `sprint-32-review.md` Section 8 (Technical debt identified)

- [ ] **Create Type 4 Feature Testing Checklist** - Standard protocol for visual features
  - **Sprint 33 Action:** Not required for Sprint 33 (data sampling is Type 1, pager already has tests)
  - **Reference:** `sprint-32-review.md` Section 6 (Quality recommendations)

**Reference:** `docs/sprints/sprint-32-review.md`

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Design data sampling commands UX (`/sample`, `/peek`)
- Update `docs/specifications/repl.md` with data sampling requirements
- Define command syntax, error messages, help text
- Ensure consistency with existing schema commands (`/list`, `/describe`)

**Deliverables:**
- Updated `docs/specifications/repl.md` with REQ-SAMPLE-001 through REQ-SAMPLE-015
- Pager status documentation update (experimental, off by default)
- UX design validation for data sampling commands

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Debug pager rendering issue (root cause analysis)
- Implement pager fix (if achievable in time)
- Disable pager by default (`pager_enabled: false`)
- Implement data sampling commands (`/sample`, `/peek`)
- Write unit tests for all new code
- Update `docs/design/repl.md` with pager findings and sampling implementation

**Deliverables:**
- Pager root cause analysis document
- Pager fix implementation (or documentation of why deferred)
- Data sampling commands fully implemented
- Unit tests with 100% pass rate
- Updated `docs/design/repl.md`
- Technical debt report

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design test cases for data sampling commands
- Execute all test suites (unit + integration)
- Validate pager fix (automated tests only, no manual)
- Generate test reports in `tests/results/sprint-33/`
- Validate all 25 acceptance criteria

**Deliverables:**
- Test strategy: `tests/strategy/sprint-33-test-strategy.md`
- Test cases: `tests/cases/TC-033-*.md`
- Test execution report: `tests/results/sprint-33/REPORT.md`
- Manual validation document: `tests/cases/TC-033-PAGER-MANUAL.md` (not executed)
- 100% test pass rate
- Honest validation status (no false claims about pager working)

---

### tq-project-manager (Haiku)
**Responsibilities:**
- Validate sprint completion against Definition of Done
- Assess technical debt status
- Verify documentation synchronized
- Update GitHub issue #14 with resolution details
- Provide go/no-go decision for sprint closure

**Deliverables:**
- Sprint completion validation report
- Technical debt assessment
- GitHub issue #14 closure or status update
- Go/no-go recommendation
- Recommendations for Sprint 34

---

## Sprint Timeline

**Estimated Duration:** 1 day (single-day sprint)

### Phase Breakdown
- **Phase 0: Reality Check** (✅ Complete)
  - Reviewed last 3 sprints
  - Decision: Mixed sprint (bug + feature)

- **Phase 1: Planning** (✅ Complete)
  - Sprint planning document created
  - GitHub issue #14 triaged and included

- **Phase 2: Design** (Est. 1-2 hours)
  - Parallel: cli-ux-designer (data sampling specs) + rust-teradata-architect (pager analysis)
  - Specifications finalized

- **Phase 3: Build & Test** (Est. 4-6 hours)
  - Parallel: rust-teradata-architect (implementation) + quality-validator (test design)
  - Code + tests delivered

- **Phase 4: Ship** (Est. 1 hour)
  - quality-validator validates completion
  - tq-project-manager approves
  - Git commit and push

- **Phase 5: Retrospective** (Est. 1-2 hours)
  - Use `/sprint-reviewer` skill
  - Collect metrics
  - Create sprint-33-review.md

---

## Notes

### Critical Sprint 31/32 Context

**Sprint 31 (Framework Recovery):**
- Implemented two-pass cell truncation to fix pager overflow
- Enabled pager by default (`pager_enabled: true`)
- Acknowledged manual validation was pending
- Assessment: "Cannot claim pager works without manual validation"

**Sprint 32 (Content-Based Width):**
- Continued with pager enabled by default
- No pager changes made
- Assumed Sprint 31 fix was sufficient

**Issue #14 Reality:**
- User reports pager still produces garbled output
- Screenshot shows clear rendering problems
- Confirms Sprint 31's honest assessment that manual validation was needed
- Validates Sprint 31 philosophy: automated tests don't prove visual features work

### Sprint 33 Approach

**Honest Assessment:**
- Sprint 31's fix didn't resolve the issue
- Automated tests passed but feature was still broken
- This sprint will debug, attempt fix, but disable by default regardless
- Will not claim "pager works" without manual validation

**User Protection:**
- Disable pager by default immediately
- Users can opt-in with `/pager on` if they want to test
- Future sprint can re-enable if proper validation performed

**Business Value:**
- Data sampling commands provide immediate user value
- Complements existing schema exploration features
- Addresses data analyst persona needs

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-03 | 1.0 | Initial Sprint 33 plan - Pager bug fix + data sampling | Sprint Coordinator |
