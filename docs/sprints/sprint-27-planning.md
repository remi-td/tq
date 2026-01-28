# Sprint 27 Planning

**Date:** 2026-01-27
**Type:** Bug Fix + Documentation Sprint
**Sprint Coordinator:** Main Agent

---

## Reality Check Summary

- **Reviewed sprints:** 26, 25, 24
- **Patterns detected:** None - Healthy velocity
  - 100% feature delivery across all recent sprints
  - Excellent quality ratings (9.0-9.5/10)
  - Zero technical debt maintained
  - Sprint 26 achieved single-iteration success
- **Decision:** Feature Sprint (Bug Fix Priority)
- **Rationale:** Project shows excellent health. Sprint 26 introduced a critical bug in /sessions command that must be fixed immediately. Opportunity to also address high-priority licensing and README documentation.

---

## GitHub Issues Analysis

**Issues Triaged:** 4 total
- **#10** - BUG: Incorrect session count (NEW - CRITICAL)
- **#8** - LICENSE: Proper licensing (NEW - HIGH PRIORITY)
- **#9** - README: User-focused documentation (NEW - MEDIUM)
- **#7** - Horizontal paging (Already triaged - DEFER again)

**Triage Results:**
- Accepted: 3 issues (#10, #8, #9)
- Sprint-ready: 4 issues total
- Deferred: 1 issue (#7 - too complex for bug fix sprint)

---

## Sprint Focus

**Primary Focus:** Fix critical bug in Sprint 26 /sessions feature while addressing high-priority legal compliance and documentation needs.

**Sprint 26 Context:**
- Delivered `/sessions` command (v1.12.0)
- Issue discovered: Command returns incorrect session count (shows 2 when 3 exist)
- This is a regression that affects core monitoring functionality

**Strategic Value:**
- Maintains quality of recently shipped feature
- Addresses legal compliance (LICENSE)
- Improves user onboarding (README)
- Relatively small scope enables fast turnaround

---

## Objectives

### P0 Objectives (Must Deliver)

1. **Fix /sessions Command Bug (#10)**
   - Root cause: Command shows 2 sessions when 3 actually exist
   - Impact: Users cannot trust session counts for monitoring
   - Evidence: User-provided comparison shows SQL query returns 3 rows, `/sessions` returns 2
   - Deliverable: Correct session filtering/display logic

2. **Add Proper LICENSE File (#8)**
   - Current: MIT license only (incomplete/misleading)
   - Required: Attribution for teradatarustapi dependencies
   - Compliance: Teradata GoSQL Driver license + Go license
   - Deliverable: Complete LICENSE file with proper third-party attributions

### P1 Objectives (Should Deliver)

3. **Update README for Users (#9)**
   - Current: Starts with "GitHub Configuration" (developer-focused)
   - Required: User-focused TLDR format
   - Sections: What/Visual/Quick Start, AI Development Story, Additional Details
   - Deliverable: Professional README with screenshot and clear onboarding

---

## Acceptance Criteria

### Bug Fix (#10)
- [ ] Root cause identified and documented
- [ ] Fix implemented in `src/commands/sessions.rs`
- [ ] All 3 sessions from user example are displayed correctly
- [ ] Regression test added to prevent recurrence
- [ ] Design document updated with fix explanation
- [ ] All existing tests pass (no regressions)
- [ ] Manual verification with user's example query

### LICENSE (#8)
- [ ] LICENSE file updated with complete terms
- [ ] teradatarustapi license attribution included
- [ ] Go license attribution included
- [ ] NOTICE or THIRD-PARTY-LICENSES file created if needed
- [ ] README licensing section added
- [ ] Compliance with Teradata redistribution terms verified

### README (#9)
- [ ] TLDR introduction section (What/Visual/Quick Start)
- [ ] AI-agent development story section (tongue-in-cheek tone)
- [ ] Screenshot of tq in action included
- [ ] Installation instructions clear and concise
- [ ] Links to roadmap and documentation
- [ ] Professional tone suitable for public project
- [ ] GitHub Configuration section moved to appropriate location (CONTRIBUTING.md or developer docs)

---

## Scope

### In Scope

**P0 - Critical:**
- `/sessions` bug fix (#10)
- Proper LICENSE file (#8)

**P1 - High Priority:**
- User-focused README (#9)

### Out of Scope

- **Horizontal paging (#7)** - Too complex, requires dedicated sprint
  - Deferred from Sprint 26
  - High implementation complexity (interactive pager mode, arrow key navigation)
  - Will be considered for future "Advanced Paging" sprint

- **User guide update from Sprint 26** - While noted as gap in Sprint 26 review, focusing on critical bug first. Can be addressed in Sprint 28 if time allows.

---

## GitHub Issues

### Selected for Sprint 27

- **#10:** [BUG] Incorrect number of sessions (priority-high, bug) - **P0**
- **#8:** [FEATURE] License (priority-high, enhancement) - **P0**
- **#9:** [DOCS] Readme (priority-medium, documentation) - **P1**

### Deferred

- **#7:** [FEATURE] Horizontal paging of resultsets (priority-medium, enhancement)
  - **Reason:** High complexity (interactive pager mode, terminal state management, arrow key handling)
  - **Status:** Remains in backlog for future sprint
  - **Recommendation:** Consider for dedicated "Advanced Paging" sprint

---

## Dependencies

**Technical Dependencies:**
- `/sessions` bug fix requires understanding of Sprint 26 implementation
- Must read `src/commands/sessions.rs` (707 lines from Sprint 26)
- Must review MonitorSession SQL query logic
- Design document: `docs/design/repl.md` (Sessions Command section)

**External Dependencies:**
- LICENSE: Review teradatarustapi licensing terms
  - https://github.com/Teradata/teradatarustapi/blob/main/LICENSE
  - https://github.com/Teradata/teradatarustapi/blob/main/THIRDPARTYLICENSE

**Documentation Dependencies:**
- README screenshot provided by user (already attached to issue #9)
- Roadmap status for README summary

---

## Risk Assessment

**Low Risk Sprint:**
- Bug fix is localized to single module (`sessions.rs`)
- LICENSE is documentation-only (no code changes)
- README is documentation-only (no code changes)
- Clear acceptance criteria for all objectives
- Small scope enables fast iteration

**Known Risks:**
1. **Bug Root Cause Complexity** (Medium risk)
   - Mitigation: Thorough analysis in Phase 2 (Design)
   - User provided clear reproduction case

2. **LICENSE Legal Review** (Low risk)
   - Mitigation: Follow standard open-source licensing patterns
   - Reference established projects for guidance

3. **README Tone** (Low risk)
   - Mitigation: User specified "tongue-in-cheek" is acceptable
   - Clear examples provided

---

## Success Metrics

- **Bug Fix:** 100% of sessions displayed correctly (3/3 in user example)
- **LICENSE:** Legal compliance verified, no misleading claims
- **README:** Professional first impression for new users
- **Quality:** 100% test pass rate maintained
- **Debt:** Zero new technical debt
- **Iterations:** Target 1-2 iterations (bug fix may require debugging)

---

## Sprint Execution Plan

**Phase 2 - Design (Parallel):**
- cli-ux-designer: README structure, LICENSE content review
- rust-teradata-architect: Bug root cause analysis, fix design

**Phase 3 - Build & Test (Parallel):**
- rust-teradata-architect: Implement bug fix, verify with user example
- quality-validator: Regression tests, license verification, README review

**Phase 4 - Ship:**
- Validate all acceptance criteria
- Update issue #10, #8, #9 with implementation details
- Git commit and push
- Close completed issues

**Phase 5 - Retrospective:**
- Sprint review with metrics
- Lessons learned from bug fix process
- Framework optimization opportunities

---

## Notes

**Sprint 26 Gap:**
Sprint 26 review identified user guide gap (docs/user/repl-guide.md missing /sessions documentation). This is noted as P1 for future sprint but not included in Sprint 27 scope due to critical bug priority.

**AI Development Story:**
Issue #9 highlights unique aspect of tq project - exclusively developed by AI agents. This is a compelling story to tell in README and differentiates the project.

**Legal Compliance:**
Issue #8 is important for project credibility and legal clarity. Addressing early prevents future complications with users or Teradata.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-27 | 1.0 | Sprint 27 planning - Bug fix + documentation sprint | Sprint Coordinator |
