---
sprint: 32
start_date: 2026-02-03
target_completion: 2026-02-03
status: Planning
---

# Sprint 32 Planning: Content-Based Column Width + Quick Wins

## Reality Check Summary

**Reviewed Sprints:** 31, 30, 29

**Patterns Detected:**
- Sprint 29-30: Framework crisis (tests passing, features broken)
- Sprint 31: Successful crisis recovery with honest assessment
- Framework health: RESTORED ✅
- Testing philosophy: Documented with manual validation requirements
- No stuck issues or accumulating debt

**Decision:** FEATURE SPRINT

**Rationale:**
- Framework integrity restored through Sprint 31
- Testing limitations clearly documented
- Manual validation requirements established
- Ready to resume feature delivery with honest assessment practices
- Two sprint-ready GitHub issues available (high and low priority)

---

## Sprint Overview

**Sprint Goal:** Improve table display density through content-based column width calculation, plus quick documentation fix

**Sprint Theme:** UX Enhancement - making wide table queries significantly more usable in REPL mode

---

## Objectives

1. Implement content-based column width calculation to maximize information density in table displays
2. Fix GitHub README display issue (quick win)
3. Apply Sprint 31 lessons: manual validation for visual features

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Content-Based Column Width (#13)

**Description:** Calculate column widths based on actual content length rather than schema-defined type, significantly improving table display density for VARCHAR columns

**GitHub Issue:** #13 (priority-high, enhancement)

**User Pain Point:**
- Current: `SELECT * FROM DBC.Databases` shows only 2 columns in standard terminal (DatabaseName and CreatorName take ~60 chars each despite content being ~15 chars)
- Desired: Same query shows 8+ columns by sizing to actual content

**Acceptance Criteria:**
- [ ] AC-1: Column width calculated from actual cell content, not schema type
- [ ] AC-2: Maximum column width capped at reasonable limit (e.g., 100 chars)
- [ ] AC-3: Minimum column width respects column header length
- [ ] AC-4: `SELECT * FROM DBC.Databases` displays 8+ columns in 117-char terminal
- [ ] AC-5: Existing table formatting tests still pass
- [ ] AC-6: Manual validation: REPL query shows improved density
- [ ] AC-7: No performance regression (measure column calc time)
- [ ] AC-8: Works correctly with NULL values
- [ ] AC-9: Works correctly with numeric alignment
- [ ] AC-10: Documentation updated with new column width behavior

**Reference:**
- Issue: #13
- Spec: `docs/specifications/output-formats.md#table-format`
- Design: `docs/design/table-formatting.md` (to be updated)
- Implementation: `src/format/table.rs`

**Estimated Complexity:** Medium
- Requires refactor of column width calculation logic
- Must preserve alignment and truncation behavior
- Needs comprehensive testing across data types
- Manual validation required per Sprint 31 testing philosophy

**Sprint 31 Lesson Application:**
- This is a Type 4 feature (visual/interactive) per `docs/testing/approach.md`
- MANDATORY manual validation required before sprint completion
- Must test in real REPL with actual queries (not just unit tests)
- Evidence: Script command output showing before/after column density

---

### P1 - High Priority (Should Have)

#### Feature 2: Fix GitHub README Display (#12)

**Description:** Fix `.github/README.md` preventing proper README display on GitHub repository landing page

**GitHub Issue:** #12 (priority-low, documentation)

**Problem:**
- Root `README.md` contains project introduction
- `.github/README.md` displays "GitHub Configuration" on repository landing page
- Users landing on repository see wrong content

**Acceptance Criteria:**
- [ ] AC-1: Root `README.md` displays on GitHub repository landing page
- [ ] AC-2: `.github/` directory content remains accessible for GitHub configuration
- [ ] AC-3: Solution is GitHub convention-compliant
- [ ] AC-4: No broken links or references

**Reference:**
- Issue: #12
- Files: `README.md`, `.github/README.md`

**Estimated Complexity:** Low
- Simple file rename or move
- ~5 minutes implementation
- Immediate verification via GitHub UI

**Solution Approach:**
- Rename `.github/README.md` to `.github/GITHUB_NOTES.md` or similar
- GitHub will default to root `README.md` when no `.github/README.md` exists

---

### Explicitly Out of Scope

- Pager manual validation (deferred from Sprint 31 - addressed if user reports issues)
- Track 3 test utilities assessment (deferred from Sprint 31 - no blocking issues)
- Any new features beyond #13 and #12
- Performance optimizations beyond preventing regression
- Additional output format enhancements

**Rationale:**
- Sprint 31 left pager enabled with strong automated test validation
- No user reports of pager issues since Sprint 31
- Pager validation becomes priority if issues reported
- Focus Sprint 32 on high-value UX enhancement (#13)

---

## GitHub Issues

### Selected for Sprint 32
- **#13**: [FEATURE] Trim column names (priority-high, enhancement) - P0
- **#12**: [DOCS] Wrong readme display on GitHub (priority-low, documentation) - P1

### Deferred
- None (backlog has no other sprint-ready issues)

---

## Success Criteria

The sprint is considered successful when ALL of the following are true:

- [ ] Feature #13 (Content-Based Column Width) implemented and validated
- [ ] Feature #12 (README Display) fixed and visible on GitHub
- [ ] 100% test pass rate (unit + integration tests)
- [ ] All 10 acceptance criteria met for Feature #13
- [ ] All 4 acceptance criteria met for Feature #12
- [ ] **MANDATORY: Manual validation performed for Feature #13** (per Sprint 31 testing philosophy)
- [ ] Evidence captured: Script command output showing improved column density
- [ ] Documentation updated: `docs/specifications/output-formats.md`, `docs/design/table-formatting.md`
- [ ] Zero technical debt introduced
- [ ] Both GitHub issues (#13, #12) closed with implementation details
- [ ] Completion validated by tq-project-manager

---

## Dependencies

### External Dependencies
- None (uses existing table formatting infrastructure)

### Prerequisite Work
- Sprint 31 testing philosophy must be applied (manual validation for visual features) ✅

### Blockers
- None identified

---

## Risks & Mitigation

### Risk 1: Column Width Calculation Breaks Existing Layouts

**Probability:** Medium
**Impact:** High (would break existing table rendering)

**Mitigation:**
- Comprehensive unit tests for edge cases
- Preserve existing tests, add new ones
- Test with various data types (VARCHAR, INTEGER, DECIMAL, DATE, NULL)
- Manual validation with diverse queries

### Risk 2: Performance Regression from Content Scanning

**Probability:** Low
**Impact:** Medium (slower table rendering)

**Mitigation:**
- Measure performance before and after
- Benchmark with large result sets (1000+ rows, 50+ columns)
- Implement early exit optimizations if needed
- Cache column widths during single table render

### Risk 3: Manual Validation Not Performed (Repeat Sprint 29/30 Pattern)

**Probability:** Low (Sprint 31 established process)
**Impact:** Critical (would repeat false success pattern)

**Mitigation:**
- Explicit acceptance criterion for manual validation (AC-6)
- quality-validator verdict is ADVISORY, not blocking
- Sprint coordinator must perform manual REPL test before closure
- Evidence required: script command output capture
- Sprint blocked if manual validation reveals issues

---

## Action Items from Sprint 31

From `docs/sprints/sprint-31-review.md`:

**MANDATORY:**

- [ ] **Perform Manual Pager Validation** (deferred - see "Out of Scope")
  - Status: Deferred to Sprint 32 only if user reports issues
  - Rationale: Strong automated validation, pager enabled, no user reports since Sprint 31
  - Will prioritize if issues emerge

- [ ] **Assess Track 3 Utilities** (deferred - see "Out of Scope")
  - Status: Deferred to future sprint
  - Rationale: No blocking issues, utilities are well-designed infrastructure
  - Can be connected to future visual validation needs

**APPLIED IN SPRINT 32:**

- [x] **Apply Manual Validation Requirements**
  - Feature #13 is Type 4 (visual/interactive) per `docs/testing/approach.md`
  - AC-6 explicitly requires manual validation
  - Evidence capture planned: script command output
  - Sprint coordinator responsible for validation

- [x] **Honest Assessment Practice**
  - Sprint planning acknowledges pager validation deferral
  - Clear distinction between automated and manual validation
  - No false success claims based on test metrics alone

**Reference:** `docs/sprints/sprint-31-review.md` Section 5 (Lessons Learned)

---

## Agent Assignments

### cli-ux-designer (Sonnet)

**Responsibilities:**
- Review Feature #13 requirements and validate UX goals
- Update `docs/specifications/output-formats.md` with content-based width behavior
- Ensure column width calculation aligns with terminal width awareness philosophy
- Validate that solution addresses user pain point from issue #13

**Deliverables:**
- Updated `docs/specifications/output-formats.md` (column width calculation section)
- UX validation: confirm content-based widths solve described pain point
- Specification review for Feature #13 acceptance criteria

---

### rust-teradata-architect (Opus)

**Responsibilities:**
- Implement content-based column width calculation in `src/format/table.rs`
- Update `docs/design/table-formatting.md` with implementation approach
- Write unit tests for all edge cases (NULL, numeric, long strings, etc.)
- Implement Feature #12 (README fix) - simple file operation
- Ensure no performance regression
- Measure performance: column calculation time before/after

**Deliverables:**
- Working implementation of Feature #13 (content-based column widths)
- Working implementation of Feature #12 (README display fix)
- Unit tests with 100% pass rate for table formatting
- Updated `docs/design/table-formatting.md`
- Performance benchmark results
- Technical debt assessment: zero new debt

---

### quality-validator (Sonnet)

**Responsibilities:**
- Design test cases for Feature #13 (content-based column widths)
- Execute all test suites (unit + integration)
- Generate test report in `tests/results/sprint-32/`
- Validate acceptance criteria
- **ADVISORY ROLE:** Provide recommendation, not blocking verdict
- **Note:** Manual validation by sprint coordinator is separate and mandatory

**Deliverables:**
- Test cases in `tests/cases/TC-032-XXX.md`
- Test execution report in `tests/results/sprint-32/REPORT.md`
- 100% automated test pass rate
- Advisory verdict on Feature #13 readiness
- Note that manual validation is required before sprint closure

---

### tq-project-manager (Haiku)

**Responsibilities:**
- Validate sprint completion against Definition of Done
- Verify manual validation was performed for Feature #13
- Assess technical debt status
- Verify documentation synchronization
- Close GitHub issues #13 and #12 with implementation details
- Provide go/no-go decision for sprint closure

**Deliverables:**
- Sprint completion validation report
- Confirmation of manual validation evidence
- GitHub issues closure with details
- Go/no-go recommendation
- Commit and push to GitHub
- Recommendations for Sprint 33

---

## Sprint Timeline

**Estimated Duration:** 1 day (single-day sprint)

### Phase Breakdown

- **Phase 0: Reality Check** ✅ (Complete)
  - Reviewed sprints 31, 30, 29
  - Decision: Feature Sprint
  - Framework health: RESTORED

- **Phase 1: Planning** ✅ (Complete)
  - Sprint planning document created
  - GitHub issues triaged (#13, #12)
  - Issues commented with sprint inclusion

- **Phase 2: Design** (Est. 1-2 hours)
  - Parallel execution: cli-ux-designer + rust-teradata-architect
  - Specifications finalized for content-based column widths
  - Technical approach documented

- **Phase 3: Build & Test** (Est. 3-4 hours)
  - Parallel execution: rust-teradata-architect (code) + quality-validator (tests)
  - Feature #13 implemented and tested
  - Feature #12 implemented (5 minutes)
  - Test iterations until 100% pass rate

- **Phase 4: Ship** (Est. 1 hour)
  - **MANDATORY: Manual validation of Feature #13**
  - tq-project-manager validates completion
  - Evidence review: script command output
  - Commit and push if validation passes
  - Close GitHub issues #13 and #12

- **Phase 5: Retrospective** (Est. 1 hour)
  - Launch sprint-reviewer skill (3 agents in parallel)
  - Collect metrics and create review document
  - Document lessons learned

---

## Notes

### Sprint 31 Lessons Applied

**Testing Philosophy:**
- Feature #13 is classified as Type 4 (visual/interactive) per `docs/testing/approach.md`
- Manual validation is MANDATORY, not optional
- Test metrics (100% pass rate) are advisory inputs, not conclusions
- Evidence required: captured output showing improved column density

**Honest Assessment:**
- Pager validation deferred (not claiming it's fully validated)
- Clear about what was tested (automated) vs. what requires manual validation
- No false success claims based on metrics alone

**Quality Validator Role:**
- Advisory verdict, not blocking
- Sprint coordinator owns manual validation decision
- Test pass rate is input, not conclusion

### GitHub Issues Management

Both issues triaged and sprint-ready:
- Issue #13 commented: "Included in Sprint 32"
- Issue #12 commented: "Included in Sprint 32"
- Both will be closed in Phase 4 with implementation details

### Feature #13 Implementation Notes

**Current Behavior (Problem):**
- Column width = schema type definition (e.g., VARCHAR(64) → 64 chars)
- Result: Wide columns consume excessive space even with short content
- Example: `DatabaseName VARCHAR(64)` takes 64 chars despite content being ~15 chars

**Desired Behavior (Solution):**
- Column width = max(content_length, header_length)
- Maximum cap: ~100 chars (prevent single column consuming entire screen)
- Result: Columns sized to actual content, more columns visible

**Example Impact:**
- Before: 2 columns visible (DatabaseName, CreatorName) + "14 cols hidden"
- After: 8+ columns visible (DatabaseName, CreatorName, OwnerName, AccountName, ProtectionType, JournalFlag, PermSpace, SpoolSpace)

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-03 | 1.0 | Initial Sprint 32 planning - Content-Based Column Width + README fix | Sprint Coordinator |
