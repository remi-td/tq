# Sprint 31 Planning: Framework Crisis Recovery

**Date:** 2026-02-03
**Type:** MAINTENANCE SPRINT (Crisis Mode)
**Sprint Coordinator:** Main Claude Agent

---

## Reality Check Summary

**Reviewed Sprints:** Sprint 30, 29, 28

**Critical Patterns Detected:**

1. **Testing Framework Fundamentally Broken**
   - Sprint 29: 386/386 tests pass (100%) → feature completely broken
   - Sprint 30: 449/449 tests pass (100%) → feature still broken, disabled
   - Pattern: Tests validate code structure, not user outcomes
   - Gap: Tests cannot capture/validate actual pager rendering

2. **Pager Feature Stuck (2 Sprints)**
   - Sprint 29: Claimed "production-ready", user: "absolutely not working"
   - Sprint 30: Architectural refactor, user: "failed again: exact same issue"
   - Cost: $81 invested, zero working functionality
   - Current state: Disabled by default, 973 lines of dead code

3. **User Trust Destroyed**
   - Sprint reviews claiming "9.5/10 excellent" for broken features
   - Pattern of false success claims based on test metrics
   - User frustration: "running in circle", "Value in every sprint is little"

4. **Accumulating Technical Debt**
   - 973 lines of disabled pager code (pager.rs)
   - 1,552 lines of Track 3 test utilities that can't validate rendering
   - Multiple "CRITICAL FIX" commits showing continued failed debugging

**Crisis Severity:** CRITICAL - Framework cannot distinguish between "tests pass" and "feature works"

**Decision Rationale:**
- Three consecutive sprint failures (28: planning error, 29: broken feature, 30: still broken)
- Testing framework fundamentally broken for interactive features
- User trust destroyed by pattern of claiming success for failures
- Must fix framework before attempting new features

**Sprint Type:** MAINTENANCE SPRINT - Framework Crisis Recovery

---

## Sprint 31 Objectives

### Primary Goal
**Restore Framework Integrity** - Fix the fundamental gap between test validation and feature functionality

### Critical Success Criteria

1. **Honest Assessment**
   - ✅ Acknowledge Sprint 29 and Sprint 30 as FAILURES
   - ✅ Update documentation to reflect reality, not test metrics
   - ✅ Stop claiming success based on test pass rates alone

2. **Manual Validation Gates**
   - ✅ Implement mandatory manual testing for interactive/visual features
   - ✅ Sprint Coordinator must manually verify features before closure
   - ✅ quality-validator verdict becomes ADVISORY, not blocking

3. **Pager Feature Resolution**
   - ✅ Either: Fix pager rendering and manually validate it works
   - ✅ Or: Remove pager code entirely (clean removal of dead code)
   - ✅ Do NOT leave feature disabled indefinitely

4. **Framework Documentation**
   - ✅ Document: "Automated tests cannot validate all feature types"
   - ✅ Document: Testing limitations and manual validation requirements
   - ✅ Update testing philosophy in docs/testing/

### Sprint Scope

**Track 1: Framework Documentation & Process Fixes**
- Update Sprint 29 review to reflect reality (FAILED, not 9.5/10)
- Update Sprint 30 review with honest assessment
- Document testing limitations in docs/testing/approach.md
- Implement manual validation gate in sprint-coordinator process
- Update Definition of Done to require manual verification

**Track 2: Pager Feature Resolution**
- Option A: Fix rendering bug with manual terminal validation
- Option B: Remove pager code entirely (clean removal)
- Decision criteria: Time-box 4 hours for fix attempt, then remove if not working

**Track 3: Test Infrastructure Assessment**
- Document what Track 3 utilities CAN and CANNOT validate
- Decide: Keep utilities (potential future value) or remove (disconnected from problem)
- If keeping: Document how to connect to actual rendering validation

**Track 4: GitHub Issues Triage**
- Triage issue #13 (Trim column names) - mark sprint-ready if in scope
- Triage issue #12 (README display) - documentation fix, quick win potential

---

## Track Prioritization

### MANDATORY (Must Complete)
1. **Track 1: Framework Documentation** (2-3 hours)
   - BLOCKING: Must complete before any future feature sprints
   - Updates sprint reviews, testing docs, process definitions

2. **Track 2: Pager Resolution** (4 hours max)
   - BLOCKING: Cannot leave disabled code indefinitely
   - Time-boxed: 4 hours to fix, else remove

### OPTIONAL (If Time Permits)
3. **Track 4: GitHub Issues** (1-2 hours)
   - Issue #12: README display fix (documentation, quick)
   - Issue #13: Trim column names (feature, evaluate scope)

4. **Track 3: Test Infrastructure** (1 hour)
   - Document Track 3 utility capabilities and limitations
   - Low priority: Can defer to future sprint

---

## Acceptance Criteria

### Track 1: Framework Documentation
- [ ] Sprint 29 review updated with honest assessment (FAILED rating)
- [ ] Sprint 30 review confirms critical failure status
- [ ] docs/testing/approach.md documents testing limitations
- [ ] docs/testing/approach.md documents manual validation requirements
- [ ] sprint-coordinator process updated with manual validation gate
- [ ] Definition of Done includes manual verification requirement

### Track 2: Pager Feature Resolution
- [ ] Decision made: Fix or Remove
- [ ] If Fix: Pager renders correctly in 80, 117, 120, 160 char terminals (manual validation)
- [ ] If Fix: Manual test with real database confirms functionality
- [ ] If Fix: Pager enabled by default (pager_enabled: true)
- [ ] If Remove: pager.rs removed (or stub with "not supported")
- [ ] If Remove: References removed from executor.rs, state.rs
- [ ] If Remove: Documentation updated (feature not supported)

### Track 3: Test Infrastructure (Optional)
- [ ] Document what visual_validator.rs CAN validate
- [ ] Document what visual_validator.rs CANNOT validate
- [ ] Document how to connect utilities to render capture (future work)
- [ ] Decision: Keep or remove Track 3 utilities

### Track 4: GitHub Issues (Optional)
- [ ] Issue #12 triaged (sprint-ready or needs-info)
- [ ] Issue #13 triaged (sprint-ready or deferred)
- [ ] If sprint-ready: Complete fix and close issue

---

## Sprint Execution Strategy

### Phase 1: Honest Assessment (30 minutes)
1. Read Sprint 29 and Sprint 30 reviews in full
2. Document honest findings (features were broken)
3. Update sprint review ratings to reflect reality
4. Commit updates with clear rationale

### Phase 2: Framework Documentation (2-3 hours)
1. Update docs/testing/approach.md
   - Section: "Testing Limitations"
   - Section: "When Manual Validation Required"
   - Section: "Interactive Feature Testing Strategy"
2. Update sprint-coordinator process
   - Add manual validation gate in Phase 4 (Ship)
   - quality-validator verdict becomes advisory
   - Coordinator must manually test before approval
3. Update Definition of Done
   - Manual verification requirement for visual/interactive features
   - Test pass rate necessary but not sufficient

### Phase 3: Pager Resolution (4 hours max, time-boxed)
1. **First Hour:** Debug rendering bug
   - Add Pager::render_to_buffer() method for testability
   - Capture actual output to file
   - Compare with expected terminal width
   - Identify mismatch in render_border(), render_header(), or render_row()

2. **Hours 2-3:** Fix and validate
   - Implement fix based on debug findings
   - Test in real terminal at multiple widths (80, 117, 120, 160)
   - Manual validation with live database
   - Use `script` command to capture output for evidence

3. **Hour 4:** Decision point
   - If working: Enable by default, document validation
   - If not working: Remove code, document as not supported

### Phase 4: Optional Work (If Time Permits)
1. Triage GitHub issues
2. Fix issue #12 if quick win
3. Document Track 3 utilities

---

## Success Metrics

### Minimum Success (Track 1 + Track 2)
- Framework documentation updated with honest assessment
- Testing philosophy documented with limitations
- Manual validation gates implemented in process
- Pager either working OR cleanly removed

### Full Success (All Tracks)
- All mandatory work complete
- GitHub issues triaged and/or fixed
- Track 3 utilities documented
- Clear path forward for future sprints

---

## Risk Assessment

### High Risk
1. **Pager fix might not be achievable in 4 hours**
   - Mitigation: Time-box strictly, remove if not fixable
   - Fallback: Clean removal is success outcome

2. **User expectation of new features**
   - Mitigation: Communicate clearly this is maintenance/crisis resolution
   - User feedback: "Value in every sprint is little" requires framework fixes first

### Medium Risk
3. **Scope creep into new features**
   - Mitigation: STRICT adherence to maintenance scope
   - No new features until framework integrity restored

---

## Out of Scope

**Explicitly NOT in Sprint 31:**
- ❌ New features (transaction support, query history, etc.)
- ❌ More test infrastructure (Track 3 expansion)
- ❌ Dimensional test expansion
- ❌ Feature enhancements or polish

**Rationale:** Framework must be fixed before feature development resumes

---

## Definition of Done

Sprint 31 is complete when:

1. ✅ Sprint 29 and Sprint 30 reviews honestly reflect failures
2. ✅ Testing philosophy documented with limitations
3. ✅ Manual validation gates implemented in process
4. ✅ Pager either working (manual validation) OR removed (clean)
5. ✅ All changes committed and pushed
6. ✅ Sprint review documents honest outcomes
7. ✅ User trust begins restoration through transparency

---

## Expected Duration

**Estimated Effort:** 6-8 hours total
- Track 1 (Framework Docs): 2-3 hours
- Track 2 (Pager Resolution): 4 hours (time-boxed)
- Track 4 (GitHub Issues): 1-2 hours (optional)

**Sprint Duration:** 1 day (single-day maintenance sprint)

---

## Next Steps

After Sprint 31 completion:
1. Validate framework fixes with small feature sprint (Sprint 32)
2. Apply manual validation gates to all future interactive features
3. Resume feature development with restored confidence in quality validation
4. User trust restoration through pattern of honest, working deliveries

---

**Sprint 31 is a crisis recovery sprint. Success means honesty, not features.**
