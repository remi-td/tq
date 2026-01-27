---
sprint: 24
start_date: 2026-01-27
target_completion: 2026-01-27
status: Planning
---

# Sprint 24 Planning: REPL History Enhancement & Process Improvements

## Reality Check Summary
- **Reviewed sprints:** 21, 22, 23
- **Patterns detected:**
  - Documentation accuracy issues (Sprints 22 & 23) - procedural gap, needs Ship phase verification
  - Test infrastructure mature and improving
  - Process improvements working (Sprint 23 checklist prevented Sprint 22 iteration gap)
- **Decision:** Feature Sprint
- **Rationale:** Healthy velocity, zero technical debt, no stuck issues. Documentation gap addressable within normal feature workflow.

## Sprint Overview

**Sprint Goal:** Implement multi-line command history for REPL mode and add documentation accuracy verification to Ship phase

**Sprint Theme:** Close gap between REPL specification and implementation for multi-line SQL history, while strengthening quality gates to prevent documentation mismatches

---

## Objectives

1. Implement multi-line command history in REPL mode (#3) - close gap between specification and implementation
2. Add documentation accuracy verification step to Ship phase process (Sprint 22 & 23 lesson)
3. Maintain 100% test pass rate and zero technical debt
4. Apply Sprint 23 testing methodology (checklist, consolidated guidelines)

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Multi-line Command History (#3)

**Description:** Update REPL command history to store and recall complete multi-line SQL statements (until `;` terminator) as single history entries, not line-by-line

**Problem:** Current implementation stores each line individually. Users pressing ↑ get only one line of a multi-line query, making it difficult to edit and replay complex queries.

**User Request:** "Currently the command history in REPL mode displays each line. However a simple command (SQL statement) may span until multiple lines (until the `;`), and a single line doesn't make sense."

**Acceptance Criteria:**
- [ ] Multi-line SQL statements stored as single history entry (grouped until `;` terminator)
- [ ] ↑/↓ arrows recall complete multi-line commands, not individual lines
- [ ] Cursor navigation works within recalled multi-line commands (↑/↓ moves between lines within command)
- [ ] History file format unchanged (backward compatible with existing `~/.tq_history`)
- [ ] Specification requirements met: "Multi-line SQL stored as single history entry" and "Recalled as complete multi-line block on ↑ arrow"
- [ ] All existing history features still work (search, deduplication, exclusions)
- [ ] Unit tests verify multi-line grouping logic
- [ ] Integration tests verify history persistence and recall
- [ ] PTY tests verify actual REPL behavior (keyboard navigation)

**Reference:** [REPL Mode - Command History](docs/specifications/repl.md#command-history) (lines 62-143)

**GitHub Issue:** #3 (priority-high, enhancement)

**Estimated Complexity:** High
- Requires changes to history storage logic (reedline integration)
- Complex cursor navigation within multi-line recalls
- Backward compatibility with existing history files
- Hybrid testing required (automated + manual validation)

---

#### Feature 2: Documentation Accuracy Verification in Ship Phase

**Description:** Add documentation verification step to Phase 4 (Ship) process to prevent documentation/implementation mismatches

**Problem:**
- Sprint 22: Pattern syntax documented as SQL LIKE (`%`, `_`) but implemented as glob (`*`, `?`)
- Sprint 22: Deferred loading indicator documented in user guide
- Sprint 23: `--force` flag documented but not implemented
- Pattern: Documentation written during planning, not verified against actual delivery

**Acceptance Criteria:**
- [ ] Phase 4 process document updated with documentation verification checklist
- [ ] Verification covers user guides, specifications, and examples
- [ ] Sprint coordinator executes verification before final commit
- [ ] Process prevents shipping with doc/implementation mismatches

**Reference:** Sprint 22 Review (Section 5 - UX Review), Sprint 23 Review (Section 7 - Lessons Learned)

**Estimated Complexity:** Low
- Process documentation update only
- No code changes required

---

### P1 - High Priority (Should Have)

#### Feature 3: Fix Sprint 23 Documentation Issues

**Description:** Address documented but unimplemented features from Sprint 23

**Issues:**
1. `--force` flag documented in specifications and user guide but not implemented
2. Teradata session type compatibility needs better documentation

**Acceptance Criteria:**
- [ ] Remove `--force` flag documentation from `docs/specifications/batch-mode.md` and `docs/user/batch-mode-guide.md` (deferred to future sprint)
- [ ] Add Teradata session type compatibility section to user guide (DBC/SQL vs BTEQ vs TeraSQL)
- [ ] Update error messages for transaction control to explain session limitations

**Reference:** Sprint 23 Review (Section 5 - UX Review, Section 7 - Recommendations)

**Estimated Complexity:** Low
- Documentation updates only
- Error message improvement

---

### Explicitly Out of Scope

Things we are intentionally NOT doing in this sprint:

- Implementing the deferred `--force` flag feature (defer to Sprint 25+)
- Second TAB accepts selection (blocked by reedline library Issue #624)
- Loading indicator for schema commands (requires complex threading design)
- Test infrastructure parallel execution fix (workaround documented, acceptable)
- Session type detection for proactive warnings (defer to Sprint 25+)
- Additional schema commands (`/show indexes`)
- Configuration management features (project config, profile editing)

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] All P0 features are implemented, tested, and working as specified
- [ ] Feature 1: Multi-line history works correctly in REPL (manual validation passed)
- [ ] Feature 2: Ship phase process updated with documentation verification
- [ ] Feature 3 (P1): Sprint 23 documentation issues resolved
- [ ] 100% test pass rate (unit + integration + PTY tests)
- [ ] All acceptance criteria met for delivered features
- [ ] Documentation updated to reflect new features
- [ ] Zero technical debt introduced
- [ ] Code quality meets project standards (per docs/design/*.md)
- [ ] All features validated by quality-validator agent
- [ ] Documentation accuracy verification executed in Ship phase
- [ ] Issue #3 closed with implementation details

---

## Dependencies

### External Dependencies
- reedline library (existing dependency, no version changes needed)
- No new dependencies required

### Prerequisite Work
- Sprint 23 complete (testing infrastructure, guidelines in place) ✅
- Test implementation checklist available (Sprint 23) ✅
- Consolidated testing guidelines available (Sprint 23) ✅

### Blockers
- None identified

---

## Risks & Mitigation

### Risk 1: Multi-line history complexity in reedline integration
- **Probability:** Medium
- **Impact:** High (could extend to multiple iterations if complex)
- **Mitigation:**
  - Read reedline documentation thoroughly before implementation
  - Create detailed test strategy upfront (Sprint 21/23 lesson)
  - Use hybrid testing (automated + manual) from start
  - Consider simpler fallback if reedline doesn't support desired behavior

### Risk 2: Backward compatibility with existing history files
- **Probability:** Medium
- **Impact:** Medium (user inconvenience if history lost)
- **Mitigation:**
  - Maintain file format compatibility
  - Test with existing history files from current version
  - Document migration path if format changes required
  - Provide warning message if incompatibility detected

### Risk 3: False positive risk in PTY tests (Sprint 20/21 lesson)
- **Probability:** Medium (keyboard behavior testing)
- **Impact:** Medium (wasted iteration like Sprint 22)
- **Mitigation:**
  - Make manual validation PRIMARY for keyboard navigation
  - Document automation limitations upfront in test strategy
  - Use Sprint 21's hybrid testing pattern as template

---

## Action Items from Previous Sprint

Items carried over from Sprint 23 retrospective:

- [ ] **Fix Documentation/Implementation Mismatches** (P1 Feature 3) - Remove `--force` from docs, add Teradata session guidance
- [ ] **Add Documentation Verification to Ship Phase** (P0 Feature 2) - Update Phase 4 process document
- [ ] **Apply Test Implementation Checklist** - Use checklist from Sprint 23 to prevent test gaps

**Reference:** [Sprint 23 Review - Section 7: Recommendations](sprint-23-review.md)

---

## GitHub Issues

### Selected for Sprint
- #3: [FEATURE] Multi-line command history (priority-high, enhancement) - P0 Feature 1

### Deferred
- No issues deferred (only one sprint-ready issue available)

---

## Agent Assignments

Clear assignment of responsibilities to specialized agents:

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Review and validate Feature 1 UX design (multi-line history navigation)
- Update specifications if needed for Feature 1
- Fix Sprint 23 documentation issues (Feature 3)
- Ensure UX consistency with existing REPL features

**Deliverables:**
- Updated `docs/specifications/repl.md` if needed (clarify multi-line history behavior)
- Fixed `docs/specifications/batch-mode.md` and `docs/user/batch-mode-guide.md` (remove `--force`)
- Added Teradata session compatibility section to user guide
- UX design validation for Feature 1

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement Feature 1 (multi-line command history)
- Write unit tests for history grouping logic
- Update `docs/design/repl.md` with multi-line history architecture
- Improve error messages for transaction control (Feature 3)
- Use test implementation checklist before quality review

**Deliverables:**
- Working implementation of multi-line command history
- Unit tests with 100% pass rate
- Updated `docs/design/repl.md` with implementation details
- Enhanced error messages for Teradata session limitations

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design comprehensive test strategy for Feature 1 (hybrid testing)
- Execute all test suites (unit + integration + PTY)
- Execute manual validation procedures for keyboard navigation
- Generate test reports in `tests/results/sprint-24/`
- Validate acceptance criteria for all features

**Deliverables:**
- Test strategy document in `tests/strategy/sprint-24-test-strategy.md`
- Test cases in `tests/cases/TC-XXX-*.md`
- Test execution report with evidence
- 100% automated test pass rate
- Manual validation evidence (screenshots/recordings)
- Validation that all acceptance criteria are met

---

### tq-project-manager (Haiku)
**Responsibilities:**
- Validate sprint completion at closure
- Execute documentation accuracy verification (Feature 2)
- Assess technical debt status
- Verify all documentation is synchronized
- Provide go/no-go decision for sprint closure
- Create final sprint commit and push to GitHub

**Deliverables:**
- Sprint completion validation report
- Documentation accuracy verification checklist results
- Technical debt assessment
- Go/no-go recommendation
- Git commit with sprint deliverables

---

## Sprint Timeline

**Estimated Duration:** 1 day

### Phase Breakdown
- **Phase 0: Reality Check** (Complete)
  - Reviewed Sprints 21, 22, 23
  - Detected documentation accuracy pattern
  - Decided: Feature Sprint

- **Phase 1: Planning** (Complete)
  - Sprint planning document created
  - Issue #3 triaged and accepted
  - GitHub issue commented with sprint inclusion

- **Phase 2: Design** (Est. 2-3 hours)
  - Parallel execution: cli-ux-designer + rust-teradata-architect
  - Specifications reviewed/updated
  - Design approach documented

- **Phase 3: Build & Test** (Est. 6-8 hours)
  - Parallel execution: rust-teradata-architect (code) + quality-validator (tests)
  - Implementation of Feature 1
  - Comprehensive testing (automated + manual)
  - Target: Single iteration (apply Sprint 23 lessons)

- **Phase 4: Ship** (Est. 1-2 hours)
  - tq-project-manager validates completion
  - **NEW:** Documentation accuracy verification executed
  - Issue #3 closed with implementation details
  - Git commit and push to GitHub

- **Phase 5: Retrospective** (Est. 1-2 hours)
  - Use `/sprint-reviewer` skill for comprehensive review
  - Sprint review document created
  - Lessons learned documented
  - Framework optimization opportunities identified

---

## Notes

### Design Considerations for Feature 1

**Multi-line History Grouping:**
- Group lines into statements until `;` terminator encountered
- Handle edge cases: comments, strings containing `;`, escaped characters
- Maintain compatibility with existing `~/.tq_history` file format

**Cursor Navigation:**
- ↑/↓ between history entries (complete statements)
- Within a recalled multi-line statement, allow line-by-line editing
- Maintain current cursor position when navigating within statement

**reedline Integration:**
- Review reedline's History trait and implementation
- Check if custom history implementation needed
- Ensure compatibility with Ctrl-R search

### Testing Approach (Sprint 21/23 Pattern)

**Feature 1 is HIGH FALSE POSITIVE RISK** (keyboard behavior):
- Manual validation PRIMARY
- Automated tests validate logic only (grouping, storage, recall)
- PTY tests for integration but NOT keyboard behavior verification
- Document automation limitations upfront in test strategy

### Sprint 23 Lessons Applied

1. ✅ Use test implementation checklist (prevent Sprint 22 gap)
2. ✅ Document automation limitations upfront (Sprint 21 pattern)
3. ✅ Add documentation verification to Ship phase (Feature 2)
4. ✅ Fix previous sprint documentation issues (Feature 3)

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-27 | 1.0 | Initial Sprint 24 plan | Sprint Coordinator |
