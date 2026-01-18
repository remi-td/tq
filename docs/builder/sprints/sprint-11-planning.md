---
sprint: 11
start_date: 2026-01-18
target_completion: 2026-01-18
status: Planning
---

# Sprint 11 Planning: Critical Quality Recovery - Table Display & Tab Completion

## Sprint Overview

**Sprint Goal:** Fix critical regressions in tab completion and table display to restore user trust and tool usability.

**Sprint Theme:** Quality Recovery - This is a **critical bug fix sprint** addressing user-reported regressions that make the tool unusable. Zero tolerance for quality issues.

**User Feedback Context:**
- "Completion doesn't make sense AGAIN!"
- "Broken AGAIN with the padding!!!"
- "Please stop the padding for now and postpone it for much later as it just breaks everything"

---

## Objectives

1. **Restore tab completion functionality** - Fix regression causing SQL keywords to appear instead of context-aware completions
2. **Remove broken padding feature entirely** - Implement simpler terminal-width-aware column truncation
3. **Root cause analysis** - Understand WHY these regressions occurred despite Sprint 9/10 test passes
4. **Prevent future regressions** - Improve testing methodology to catch these issues before deployment

---

## Scope

### P0 - Critical (Must Have)

#### Bug Fix 1: Tab Completion Regression

**Description:** Tab completion shows "(SQL keyword)" repeated 25 times instead of context-aware completions (databases, tables, columns)

**Evidence:** Screenshot in `docs/builder/incoming/completion.png` shows completions broken when typing "tq> ? sel * from ["

**Root Cause (Hypothesis):** Recent changes in Sprint 10 may have broken the completion context detection logic

**Acceptance Criteria:**
- [ ] Context-aware completion works correctly (shows databases/tables, NOT generic "SQL keyword")
- [ ] Database name completion after FROM/JOIN keywords works
- [ ] Table name completion in qualified syntax (database.table) works
- [ ] Column name completion after SELECT/WHERE works
- [ ] Multi-line completion context preserved (Sprint 9 fix remains working)
- [ ] All completion scenarios tested with live database
- [ ] Root cause documented in sprint review

**Reference:**
- Bug report: `docs/builder/incoming/open-bugs.md`
- Original feature: `detailed-specifications/repl-mode.md#tab-completion`
- Sprint 7 implementation: `docs/builder/sprints/sprint-7-review.md`

**Estimated Complexity:** Medium (requires root cause analysis + fix + comprehensive testing)

---

#### Bug Fix 2: Table Display Completely Broken

**Description:** Table output shows scattered, unreadable text with excessive padding across the screen

**Evidence:** Screenshot in `docs/builder/incoming/table display-bug.png` shows table completely unusable

**User Directive:**
- REMOVE the padding feature entirely (it keeps breaking things)
- Implement simpler approach: detect terminal width, truncate columns to fit
- Show "(+n cols)" indicator in header for hidden columns
- Postpone proper padding research until we have proper visual testing framework

**Acceptance Criteria:**
- [ ] Padding logic completely removed from codebase
- [ ] Terminal width detection implemented using `terminal_size` crate
- [ ] Columns truncated to fit terminal width (prioritize leftmost columns)
- [ ] Header shows "| (+n cols) |" indicator when columns are hidden
- [ ] Body shows "| ... |" in last column when data is truncated
- [ ] Table remains readable and aligned
- [ ] Works in various terminal widths (80, 120, 160, 200+ columns)
- [ ] Batch mode (non-TTY) shows all columns (no truncation)
- [ ] Root cause documented: why did padding keep breaking?

**Reference:**
- Bug report: `docs/builder/incoming/open-bugs.md`
- Original feature: `detailed-specifications/output-formats.md#table-format`
- Sprint 6 implementation: `docs/builder/sprints/sprint-6-review.md`

**Estimated Complexity:** Medium (remove old code + implement simpler approach + extensive testing)

---

### P1 - High Priority (Should Have)

#### Root Cause Analysis & Prevention

**Description:** Comprehensive analysis of why these regressions occurred despite passing tests in Sprints 9-10

**Investigation Areas:**
1. **Why didn't tests catch tab completion regression?**
   - Were completion tests actually run against live database?
   - Did batch mode changes affect REPL completion logic?
   - Test coverage gaps?

2. **Why did padding break again?**
   - Was padding code modified in Sprint 10?
   - Did new formatters interfere with table display?
   - Batch mode vs REPL mode interaction?

3. **Testing methodology gaps:**
   - Are we testing the right scenarios?
   - Do we need visual regression testing?
   - Should we require manual validation for UI features?

**Acceptance Criteria:**
- [ ] Root cause identified for tab completion regression
- [ ] Root cause identified for table display regression
- [ ] Testing gaps documented
- [ ] Recommendations for preventing future regressions
- [ ] Updated testing-guidelines.md with lessons learned

**Reference:** `docs/builder/testing-guidelines.md`

**Estimated Complexity:** Medium (requires investigation + documentation)

---

### P2 - Medium Priority (Nice to Have)

#### Improve Test Coverage for REPL Features

**Description:** Add integration tests that better validate REPL UI features

**Potential Improvements:**
- Visual output validation (capture and compare table rendering)
- More comprehensive completion scenarios
- Terminal width simulation tests
- Automated REPL interaction tests

**Acceptance Criteria:**
- [ ] At least 5 new integration tests for completion scenarios
- [ ] At least 5 new tests for table display with various terminal widths
- [ ] Tests use expectrl for REPL automation
- [ ] All tests pass with live database

**Reference:** `docs/builder/testing-guidelines.md`

**Estimated Complexity:** Medium

---

### Explicitly Out of Scope

- **Any new features** - This is a bug fix sprint only
- **Proper padding implementation** - User explicitly postponed this
- **Advanced table formatting** - Keep it simple for now
- **Configuration files** - Deferred to Sprint 12+
- **Performance optimization** - Focus on correctness first

**Rationale:** User has lost patience with features that break basic functionality. We must restore quality and trust before adding new capabilities.

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] Tab completion works correctly for all context-aware scenarios (databases, tables, columns)
- [ ] Table display is readable with appropriate terminal width handling
- [ ] "(+n cols)" indicator shows when columns are truncated
- [ ] All P0 bugs fixed and validated with live database
- [ ] Root cause analysis complete and documented
- [ ] 100% test pass rate (unit + integration tests)
- [ ] Zero build warnings
- [ ] Zero technical debt introduced
- [ ] User validates fixes work correctly
- [ ] Completion validated by tq-project-manager agent
- [ ] Testing methodology improvements documented

---

## Dependencies

### External Dependencies
- Live Teradata database for testing (user confirms availability)
- Terminal with configurable width for testing truncation logic
- `terminal_size` crate (already in dependencies)

### Prerequisite Work
- None - Sprint 10 complete, ready to proceed

### Blockers
- **Database Availability:** User must have test database running for validation
  - **Mitigation:** User confirmed database available, verify with `./target/release/tq ping` before starting

---

## Risks & Mitigation

### Risk 1: Root Cause Not Identifiable
- **Probability:** Low
- **Impact:** Medium (delays prevention measures)
- **Mitigation:** Focus on fixing bugs first, root cause analysis is P1 (not blocking)

### Risk 2: Fixes Break Other Features
- **Probability:** Medium
- **Impact:** High (more regressions)
- **Mitigation:**
  - Comprehensive test suite execution after each fix
  - Sequential bug fixing (not parallel)
  - Manual validation with live database
  - quality-validator agent validates all scenarios

### Risk 3: User Expectations Misaligned
- **Probability:** Low
- **Impact:** High (rework required)
- **Mitigation:**
  - Get explicit user approval on sprint plan
  - Show user the simpler table truncation approach before implementing
  - Get confirmation that removing padding entirely is acceptable

---

## Action Items from Previous Sprint

Items carried over from Sprint 10 review:

- [ ] **Fix interactive test** - `test_repl_help_command` requires live database (Priority: Low)
  - Status: Will address if time permits, not blocking

- [ ] **Add performance benchmarks** - For batch operations (Priority: Low)
  - Status: Deferred to Sprint 12+, focus on quality first

**Reference:** `docs/builder/sprints/sprint-10-review.md#action-items`

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Design the simplified table truncation UX (terminal width detection + column truncation)
- Specify the "(+n cols)" indicator format and behavior
- Update `detailed-specifications/output-formats.md` with new approach
- Document why padding is being removed
- Update `specifications.md` to mark features as 🔧 In Repair

**Deliverables:**
- Updated `specifications.md` with 🔧 status for broken features
- Updated `detailed-specifications/output-formats.md` with simplified table truncation spec
- UX design for "(+n cols)" indicator
- Validation that new approach meets usability standards

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Root cause analysis: Why did completion break? Why did padding break?
- Fix tab completion regression
- Remove all padding logic from codebase
- Implement terminal width detection and column truncation
- Write comprehensive unit tests for both fixes
- Update `rust-architecture.md` if patterns change
- Zero technical debt tolerance

**Deliverables:**
- Root cause analysis document (in sprint review)
- Working tab completion (all scenarios validated)
- Working table display with terminal width awareness
- All padding code removed
- Unit tests with 100% pass rate
- Clean build (zero warnings)
- Technical debt report (should be zero)

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design comprehensive test cases for tab completion scenarios
- Design test cases for table display with various terminal widths
- Execute all test suites (unit + integration + manual)
- Generate detailed test report
- Validate acceptance criteria for both bug fixes
- Test with live Teradata database
- Recommend testing methodology improvements

**Deliverables:**
- Test cases in `tests/cases/TC###.md` for:
  - Tab completion scenarios (context-aware, multi-line, etc.)
  - Table display with terminal widths: 80, 120, 160, 200+ columns
  - Batch mode vs REPL mode table output
- Test execution report in `tests/results/YYYYMMDD-HHMMSS/REPORT.md`
- 100% test pass rate
- Recommendations for preventing future regressions
- Updated `testing-guidelines.md` with lessons learned

---

### tq-project-manager (Haiku)
**Responsibilities:**
- Validate sprint completion at closure
- Verify both bugs are truly fixed (not just passing tests)
- Assess whether testing methodology improvements are adequate
- Verify documentation is synchronized
- Provide go/no-go decision for sprint closure
- Assess user trust restoration

**Deliverables:**
- Sprint completion validation report
- Quality assessment (zero tolerance for issues)
- Go/no-go recommendation
- Recommendations for Sprint 12
- Assessment of whether we've addressed root causes

---

## Sprint Timeline

**Estimated Duration:** 1 day

### Phase Breakdown

- **Phase 1: Planning** (Complete after user approval)
  - Sprint planning document created ✅
  - User approval obtained ⏳

- **Phase 2: Design** (Est. 1-2 hours)
  - Parallel execution: cli-ux-designer + rust-teradata-architect (root cause analysis)
  - Simplified table truncation UX designed
  - Root causes identified

- **Phase 2.5: Database Connectivity Check** (Est. 5 minutes)
  - Verify `.env` configured
  - Run `./target/release/tq ping` to verify database connectivity
  - **CRITICAL:** Do not proceed to Phase 3 if database unavailable

- **Phase 3: Implementation** (Est. 3-4 hours)
  - Parallel execution: rust-teradata-architect (implement fixes) + quality-validator (design tests)
  - Fix completion regression
  - Remove padding, implement terminal width detection
  - Comprehensive test cases designed

- **Phase 4: Testing** (Est. 1-2 hours)
  - quality-validator executes all tests
  - Live database validation
  - If failures: rust-teradata-architect fixes, return to Phase 4
  - Loop until 100% pass rate

- **Phase 5: Closure** (Est. 1 hour)
  - tq-project-manager validates completion
  - Sprint review created
  - Roadmap updated
  - User confirms fixes work

---

## Notes

### Critical Context for Agents

1. **User Frustration Level: HIGH**
   - These are recurring issues (broken "AGAIN")
   - User has lost patience with padding feature
   - Explicit directive to remove it entirely
   - This sprint is about restoring trust

2. **Quality Over Features**
   - Do NOT add any new features
   - Do NOT try to "improve" beyond fixing the bugs
   - Simple, working solution is better than complex, broken solution
   - User prefers simpler table truncation over fancy padding

3. **Testing Philosophy Change Needed**
   - Tests passed in Sprint 9/10 but bugs still shipped
   - Need better validation methodology
   - Manual testing with live database is MANDATORY
   - Visual validation for UI features is critical

4. **Padding Feature History**
   - Implemented in Sprint 6
   - Broke things, fixed in Sprint 8
   - Broke again somehow in Sprint 10
   - User directive: STOP trying to make it work, remove it
   - Future implementation requires visual testing framework

### Technical Approach

**Table Display Simplification:**
```
Old approach (REMOVE):
- Calculate padding for each column
- Try to fit data with padding
- Complex logic that keeps breaking

New approach (IMPLEMENT):
- Detect terminal width using terminal_size crate
- Calculate which columns fit without padding
- Show leftmost columns that fit
- Add "| (+n cols) |" header for hidden columns
- Add "| ... |" in body for truncated columns
- In batch mode (non-TTY): show all columns, no truncation
```

**Tab Completion Fix:**
- Investigate what changed in Sprint 10 that broke completion
- Likely culprit: batch mode changes affecting REPL state
- Test hypothesis: Does completion state get corrupted?
- Fix: Restore Sprint 9 working behavior
- Validate: Test every completion scenario with live database

---

## Approval

**Status:** Pending User Approval

**Approved By:** [Awaiting user approval]
**Approval Date:** [Pending]

**Questions for User:**
1. Does the simplified table truncation approach meet your needs?
   - Terminal width detection
   - Show "(+n cols)" for hidden columns
   - Remove all padding logic

2. Is database available for testing? (Need to run `tq ping` before Phase 3)

3. Any other issues or concerns to address in this sprint?

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-18 | 1.0 | Initial sprint plan - Critical quality recovery | Sprint Coordinator |
