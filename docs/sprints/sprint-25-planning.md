---
sprint: 25
start_date: 2026-01-27
target_completion: 2026-01-27
status: Planning
---

# Sprint 25 Planning: Documentation & Issue Template Fixes

## Sprint Overview

**Sprint Goal:** Fix documentation organization issues and repair broken GitHub issue template configuration to improve user experience and contribution workflow.

**Sprint Theme:** Quick wins - High-impact documentation fixes with minimal complexity

---

## Reality Check Summary

- **Reviewed Sprints:** 24, 23, 22
- **Patterns Detected:**
  - ✅ Excellent feature delivery (100% P0 across 3 sprints)
  - ✅ Zero technical debt maintained
  - ✅ Strong test quality (100% pass rates)
  - ✅ Process improvements working (Sprint 24 doc verification)
  - ⚠️ Documentation issues resolved in Sprint 24 (verification added to Ship phase)
- **Decision:** Feature Sprint
- **Rationale:** Healthy velocity, no systemic issues, 4 sprint-ready GitHub issues (2 high-priority bugs)

---

## Objectives

High-level objectives for this sprint:

1. **Fix documentation organization** - Remove duplicate roadmap file creating user confusion
2. **Repair issue template** - Fix broken documentation issue template (404 error)
3. **Improve contribution workflow** - Ensure users can report issues properly

---

## Scope

### P0 - Critical (Must Have)

Features that are absolutely required for sprint success. These MUST be delivered.

#### Feature 1: Fix Duplicate Roadmap Documentation (#4)

**Description:** Delete `docs/user/roadmap.md` to eliminate duplicate roadmap documentation. Keep `docs/roadmap/roadmap.md` as single source of truth.

**Acceptance Criteria:**
- [ ] `docs/user/roadmap.md` deleted
- [ ] All cross-references updated to point to `docs/roadmap/roadmap.md`
- [ ] No broken links in documentation
- [ ] User guide index updated if needed

**Reference:** GitHub Issue #4, [Documentation Organization in CLAUDE.md](../../CLAUDE.md#documentation-organization)

**Estimated Complexity:** Low

**GitHub Issue:** #4 (priority-high, bug)

---

#### Feature 2: Fix Documentation Issue Template (#5)

**Description:** Repair broken documentation issue template causing 404 error when users try to create documentation issues.

**Acceptance Criteria:**
- [ ] Documentation issue template creates successfully (no 404)
- [ ] Template file path correct in `.github/ISSUE_TEMPLATE/config.yml`
- [ ] Template renders properly with all fields
- [ ] Test creating a documentation issue end-to-end

**Reference:** GitHub Issue #5, `.github/ISSUE_TEMPLATE/` configuration

**Estimated Complexity:** Low

**GitHub Issue:** #5 (priority-high, bug)

---

### P1 - High Priority (Should Have)

No P1 features planned for this sprint. Focus on quick P0 bug fixes.

---

### Explicitly Out of Scope

Things we are intentionally NOT doing in this sprint:

- Issue #6 (`/sessions` command) - Medium priority enhancement, deferred to future sprint
- Issue #7 (Horizontal paging) - Low priority enhancement, requires significant complexity
- Any new feature development - Focus on documentation fixes only
- Specification updates - No requirements changes, only fixes

---

## GitHub Issues

### Selected for Sprint 25
- #4: [BUG] Duplicate Roadmap documentation (priority-high, bug)
- #5: [BUG] Documentation issue not working (priority-high, bug)

### Deferred to Future Sprints
- #6: [FEATURE] Add /sessions command (priority-medium, enhancement) - Deferred: Focus on P0 bugs first
- #7: [FEATURE] Horizontal paging of resultsets (priority-low, enhancement) - Deferred: Low priority, high complexity

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] Both P0 features implemented and validated
- [ ] GitHub Issue #4 closed with implementation details
- [ ] GitHub Issue #5 closed with implementation details
- [ ] Documentation organized with single roadmap file
- [ ] Issue templates working correctly (verified by creating test issue)
- [ ] No broken links in documentation
- [ ] Zero technical debt introduced
- [ ] All changes committed and pushed to GitHub
- [ ] Sprint review document created

---

## Dependencies

### External Dependencies
- GitHub repository access for issue template configuration
- Git access for committing and pushing changes

### Prerequisite Work
- None - Both features are standalone fixes

### Blockers
- None identified

---

## Risks & Mitigation

Identify potential risks and how we'll mitigate them:

### Risk 1: Roadmap Cross-References
- **Probability:** Low
- **Impact:** Medium (broken links)
- **Mitigation:** Search codebase for all references to `docs/user/roadmap.md` before deletion

### Risk 2: Issue Template Configuration
- **Probability:** Low
- **Impact:** Low (easy to debug)
- **Mitigation:** Review `.github/ISSUE_TEMPLATE/config.yml` and verify file paths, test after fix

---

## Action Items from Previous Sprint

Items carried over from Sprint 24 retrospective:

- [ ] Add database pre-check to Phase 3 (Sprint 24 recommendation) - Deferred to Sprint 26
- [ ] Clarify multi-line navigation in user guide (Sprint 24 recommendation) - Deferred to Sprint 26
- [ ] AI testing boundaries documentation (Sprint 24 recommendation) - Deferred to Sprint 26

**Note:** Sprint 25 focuses on urgent documentation bugs. Sprint 24 process improvements will be addressed in Sprint 26.

**Reference:** [Sprint 24 Review](sprint-24-review.md#7-recommendations)

---

## Agent Assignments

Clear assignment of responsibilities to specialized agents:

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Review documentation organization after fix
- Verify issue template user experience
- Validate that documentation structure is clear

**Deliverables:**
- Validation report on documentation fixes
- UX assessment of issue template

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Investigate issue template configuration
- Fix `.github/ISSUE_TEMPLATE/` configuration if needed
- Search and update cross-references to roadmap file

**Deliverables:**
- Delete `docs/user/roadmap.md`
- Fix issue template configuration
- Update any cross-references

---

### quality-validator (Sonnet)
**Responsibilities:**
- Test issue template creation end-to-end
- Verify no broken documentation links
- Validate both P0 features work correctly

**Deliverables:**
- Validation report confirming both fixes work
- Test evidence (screenshots if applicable)

---

### tq-project-manager (Haiku)
**Responsibilities:**
- Validate sprint completion
- Verify documentation organization is clean
- Close GitHub issues with implementation details
- Create Sprint 25 review document

**Deliverables:**
- Sprint completion validation
- GitHub issue closure
- Sprint 25 review document

---

## Sprint Timeline

**Estimated Duration:** < 1 day (quick wins)

### Phase Breakdown
- **Phase 0: Reality Check** (Complete)
  - Reviewed Sprints 22-24
  - Triaged 4 GitHub issues
  - Decision: Feature Sprint

- **Phase 1: Planning** (Complete)
  - Sprint planning document created
  - 2 P0 bugs selected

- **Phase 2: Design** (Est. 15-30 minutes)
  - Parallel execution: cli-ux-designer + rust-teradata-architect
  - Minimal design needed (straightforward fixes)

- **Phase 3: Build & Test** (Est. 30-60 minutes)
  - Parallel execution: rust-teradata-architect + quality-validator
  - Delete file, fix configuration, validate

- **Phase 4: Ship** (Est. 15 minutes)
  - tq-project-manager validates completion
  - Commit and push changes
  - Close GitHub issues

- **Phase 5: Retrospective** (Est. 30-45 minutes)
  - Sprint review created
  - Token metrics collected
  - Lessons learned documented

---

## Notes

Additional context, considerations, or information:

- **Sprint Rationale:** Quick wins to clean up documentation and improve contribution workflow
- **Low Risk:** Both features are straightforward fixes with minimal complexity
- **High Impact:** Improves user experience and removes confusion
- **No Code Changes:** Only documentation and configuration changes (no Rust code modified)
- **Fast Execution:** Should complete in < 1 day due to low complexity

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-27 | 1.0 | Initial Sprint 25 plan - Documentation fixes | Sprint Coordinator |
