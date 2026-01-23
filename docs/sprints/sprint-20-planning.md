---
sprint: 20
start_date: 2026-01-23
status: Planning
---

# Sprint 20 Planning: Critical Bug Fixes - Logo & Tab Completion (Retry)

## Sprint Overview

**Sprint Goal:** Fix two critical production bugs that persist despite Sprint 19 attempts: lowercase logo design and tab completion pager output.

**Sprint Theme:** Maintenance Sprint - Critical bug fixes requiring root cause investigation and robust implementation.

---

## Reality Check Summary

- **Reviewed sprints:** 15, 17, 19
- **Patterns detected:**
  - Stuck issues: Logo and tab completion bugs appeared in Sprint 18 (failed), Sprint 19 (attempted fix)
  - False test confidence: Sprint 18 had 286/286 tests passing but bugs persisted
  - User validation gap: Sprint 19 marked manual validation as pending
- **Decision:** Feature Sprint (Bug Fix Focus)
- **Rationale:** Two specific, well-defined bugs with clear requirements from user. Need to verify Sprint 19 fixes or implement correct solutions.

---

## Objectives

1. **Verify or fix logo design** - Ensure lowercase ASCII art 'tq' logo displays correctly with 't' in Teradata orange
2. **Verify or fix tab completion** - Ensure tab completion after "select * from " does not show pager output
3. **Validate with user** - Obtain actual user validation before marking sprint complete

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Logo Design - Lowercase ASCII Art

**Description:** Implement or verify the lowercase 'tq' logo using block characters, with 't' in Teradata orange (RGB ≈ 255,95,0) and 'q' in default color, as specified in user's bug report.

**User's Exact Requirements (from incoming/open-bugs.md):**
```
 __
/\ \__
\ \ ,_\    __
 \ \ \/  /'__`\
  \ \ \_/\ \L\ \
   \ \__\ \___, \
    \/__/\/___/\ \
              \ \_\
               \/_/
```
"This is a lowercase 't' (left) in Teradata orange and lowercase 'q' (right) in default color, using block characters for clarity."

**Acceptance Criteria:**
- [ ] Logo uses the exact ASCII art provided by user
- [ ] 't' character (left) is colored in Teradata orange (RGB ≈ 255,95,0, color code 202)
- [ ] 'q' character (right) is in default terminal color
- [ ] Logo displays correctly on REPL startup
- [ ] Visually verified by reading actual REPL startup output
- [ ] Manual testing: User confirms logo looks correct in their terminal

**Reference:** `incoming/open-bugs.md`, `docs/specifications/branding-guidelines.md`

**Estimated Complexity:** Medium (need to verify Sprint 19 implementation or correct it)

---

#### Feature 2: Tab Completion - Suppress Pager Output

**Description:** Fix tab completion to prevent "Page 1: records 0 - 0  total: 0" pager output from appearing when pressing TAB after "select * from ".

**User's Exact Requirements (from incoming/open-bugs.md):**
> "If I press tab after `select * from ` I get:
> ```
> tq> ? select * from
> Page 1: records 0 - 0  total: 0
> ```
> You story about teradatarustapi is writing directly to TTY doesn't make any sense to me since the query functionality works well otherwise and uses the same drivers..."

**User's Recommended Solution:**
- Cache all database names at startup (`sel databasename from dbc.databases;`)
- Cache all database object names incrementally as databases are used (`sel tablename from dbc.tablesV where databasename = <databasename>;`)
- Implement proper menu-based completion with filtering and navigation
- Research how this is best implemented in other Rust tools
- Design a robust solution with test mechanism

**Acceptance Criteria:**
- [ ] Tab completion after "select * from " does NOT show pager output
- [ ] Database names are cached at REPL startup or first completion request
- [ ] Table names are cached incrementally as needed
- [ ] Completion menu shows databases, filters as user types
- [ ] After selecting database with '.', completion shows tables in that database
- [ ] Solution researched - understand how other Rust tools handle this
- [ ] Design document created explaining approach
- [ ] Test mechanism implemented and passing
- [ ] Manual testing: User confirms tab completion works without pager output

**Reference:** `incoming/open-bugs.md`, `docs/specifications/repl.md#tab-completion`

**Estimated Complexity:** High (requires investigation, design, and robust implementation)

---

### Explicitly Out of Scope

Things we are intentionally NOT doing in this sprint:

- New feature development - Focus is exclusively on fixing these two critical bugs
- Sprint 19 retrospective - Will be handled separately if needed
- Framework optimization - Will address after bug fixes are validated
- Other backlog items - All deferred until critical bugs are resolved

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] Both P0 bugs are fixed and verified working
- [ ] Logo displays exactly as specified by user
- [ ] Tab completion works without pager output
- [ ] User manually validates both fixes in their environment
- [ ] 100% test pass rate (unit + integration tests)
- [ ] Tests include validation for both bug fixes
- [ ] Design document created for tab completion solution
- [ ] Research conducted on Rust CLI tab completion patterns
- [ ] Zero technical debt introduced
- [ ] Code quality meets project standards
- [ ] All features validated by quality-validator agent

---

## Dependencies

### External Dependencies
- None - both bugs are self-contained

### Prerequisite Work
- Read Sprint 19 review to understand what was attempted
- Read Sprint 19 code changes to verify implementation
- Understand why Sprint 19 fixes didn't work (if they didn't)

### Blockers
- **Manual validation requirement:** User must test both fixes in their terminal
  - **Mitigation:** Design tests that can verify core functionality, request user validation as final gate

---

## Risks & Mitigation

### Risk 1: Sprint 19 Already Fixed These Bugs
- **Probability:** Medium
- **Impact:** Low (good outcome - just need user validation)
- **Mitigation:** First step is to verify Sprint 19 implementation against user's exact requirements

### Risk 2: Tab Completion Fix is Complex and Time-Consuming
- **Probability:** High (user specifically requested research and robust design)
- **Impact:** High (could extend sprint duration)
- **Mitigation:**
  - Dedicate rust-teradata-architect to research phase first
  - Create detailed design document before implementation
  - Consider phased approach: Phase 1 = suppress pager output, Phase 2 = full caching solution

### Risk 3: teradatarustapi Library Limitations
- **Probability:** Medium (external Go library may have constraints)
- **Impact:** Medium (may need workarounds or alternative approaches)
- **Mitigation:**
  - Research library thoroughly during design phase
  - Consider alternative query approaches for metadata
  - Explore stdout/stderr redirection options (Sprint 19 approach)

### Risk 4: False Confidence from Automated Tests (Repeat of Sprint 18)
- **Probability:** Medium (history shows this happened before)
- **Impact:** High (bugs persist despite "passing" tests)
- **Mitigation:**
  - Require manual validation as mandatory acceptance criterion
  - Design hybrid tests (automated + manual verification)
  - User must confirm fixes work in their actual environment

---

## Action Items from Previous Sprint

Items from Sprint 19 review that are relevant to this sprint:

- [ ] User manual validation of tab completion - Sprint 19 left this pending
- [ ] Verify logo implementation matches user's exact ASCII art specification
- [ ] Investigate root cause of why pager output appears during completion

**Reference:** `docs/sprints/sprint-19-review.md`

---

## Investigation Tasks (Phase 2)

Before implementation, we must investigate:

### Logo Investigation
1. Read current logo implementation in `src/commands/repl/mod.rs`
2. Compare against user's exact ASCII art specification
3. Verify color implementation (is it actually orange 202?)
4. Test actual display by reading REPL output

### Tab Completion Investigation
1. Read Sprint 19 StdoutSuppressor implementation in `src/db/metadata.rs`
2. Understand why user says "pager output still appears"
3. Research how teradatarustapi handles query output
4. Research how other Rust CLI tools implement database object completion
5. Design caching strategy for database/table names
6. Create test plan for validation

---

## Agent Assignments

### rust-teradata-architect (Opus)
**Responsibilities:**
- **Phase 2:** Investigate both bugs, research solutions, create design document
- **Phase 3:** Implement fixes for both bugs
- Write unit tests for all new code
- Update `docs/design/` with tab completion caching architecture

**Deliverables:**
- Investigation report on current state vs. user requirements
- Research document on Rust CLI tab completion patterns
- Design document for tab completion caching solution
- Working implementation of both bug fixes
- Unit tests with 100% pass rate
- Updated `docs/design/repl.md` with completion architecture

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design comprehensive test cases for both bugs
- Create hybrid tests (automated + manual verification)
- Execute all test suites
- Validate acceptance criteria
- Coordinate user manual validation

**Deliverables:**
- Test strategy document for Sprint 20
- Test cases: TC-LOGO-003 (verify new design), TC-TAB-COMPLETION-003 (verify no pager)
- Test execution report with automated results
- Manual test instructions for user validation
- Final validation that all acceptance criteria are met

---

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Verify logo specification matches user requirements
- Review tab completion UX design
- Ensure error messages are clear and helpful
- Update specifications if needed

**Deliverables:**
- Logo specification verification report
- Tab completion UX review
- Updated `docs/specifications/branding-guidelines.md` if needed
- Updated `docs/specifications/repl.md#tab-completion` if needed

---

## Sprint Timeline

**Estimated Duration:** 1-2 days

### Phase Breakdown
- **Phase 0: Reality Check** (Complete)
  - Sprint history reviewed
  - Patterns identified
  - Sprint type decided

- **Phase 1: Planning** (Complete)
  - Sprint planning document created
  - User requirements captured verbatim

- **Phase 2: Investigation & Design** (Est. 4-6 hours)
  - Parallel execution: rust-teradata-architect (investigation) + cli-ux-designer (specification verification)
  - Investigation report delivered
  - Design document created
  - Specifications verified

- **Phase 3: Implementation & Testing** (Est. 6-8 hours)
  - Parallel execution: rust-teradata-architect (implementation) + quality-validator (test design)
  - Bug fixes implemented
  - Tests designed and ready

- **Phase 4: Validation** (Est. 2-3 hours)
  - quality-validator executes automated tests
  - 100% pass rate achieved
  - Manual test instructions provided to user

- **Phase 5: User Validation & Closure** (Est. 30 minutes + user time)
  - User manually validates both fixes
  - Sprint review created
  - Roadmap updated

---

## Notes

### Key Lessons from Sprint 18/19

**What Went Wrong:**
- Sprint 18: 286/286 tests passed but bugs persisted
- Sprint 18: Misinterpreted user requirements (plain text vs. ASCII art)
- Sprint 19: Manual validation left pending, user still reports issues

**What We Must Do Differently:**
- Quote user requirements verbatim (already done in this doc)
- Verify implementation against EXACT user specification
- Require actual user validation before marking complete
- Design hybrid tests that catch user-facing issues

### Critical Success Factor

**USER MUST VALIDATE.** No matter how good our tests look, no matter how confident we are, this sprint is NOT complete until the user confirms both fixes work in their actual terminal.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-23 | 1.0 | Initial sprint plan - Critical bug fixes retry | Sprint Coordinator |
