---
sprint: 9
start_date: 2026-01-18
target_completion: 2026-01-19
status: Planning
---

# Sprint 9 Planning: Complete Quality Recovery (Bug Fixes Only)

## Sprint Overview

**Sprint Goal:** Complete all remaining bug fixes from Sprint 8 with 100% quality validation. Zero new features.

**Sprint Theme:** Quality First - Restore user trust by delivering fully working, thoroughly tested bug fixes before considering any new features.

**Philosophy:** Better to deliver 3 fully-fixed bugs than 6 partially-fixed bugs. Quality over quantity.

---

## Strategic Context

### Why Sprint 9 Focuses Only on Bug Fixes

After Sprints 5-8, we have:
- ✅ Rich feature set (REPL, completion, paging, syntax highlighting, export, etc.)
- 🔧 Quality issues (tab completion, error messages, build warnings)
- 📉 User trust damaged by incomplete fixes

**Sprint 9 Priority:** Restore quality and trust BEFORE adding new features.

**What Success Looks Like:**
- Users can use ALL delivered features without frustration
- Clean builds with zero warnings
- Professional error messages
- v1.5.1 release that users can trust

---

## Objectives

High-level objectives for this sprint:

1. **Complete ALL remaining Sprint 8 bug fixes** with thorough validation
2. **Test each fix immediately** with live database (no batching)
3. **Get user validation** for every P0/P1 fix before moving to next bug
4. **Achieve clean build** with zero warnings
5. **Create v1.5.1 release** that restores user confidence

---

## Scope

### P0 - Critical (Must Fix)

These bugs BLOCK users from using core features effectively.

#### Bug 1: Tab Completion Shows Only 9 Databases

**Issue:** When typing `SELECT * FROM <Tab>`, only 9 databases are displayed. Scrolling loops through same 9 databases. Many more databases exist but aren't shown.

**User Impact:** CRITICAL - Users cannot discover and complete database names beyond the first 9

**Root Cause:** Likely a display/menu limitation in reedline completion system, not a caching issue

**Current Behavior:**
```
tq> SELECT * FROM <Tab>
DBC (database)
val (database)
TD_SYSAL (database)
... only 9 total ...
[Scrolling loops through same 9]
```

**Expected Behavior:**
```
tq> SELECT * FROM <Tab>
[Shows ALL databases, scrollable list with proper pagination]
OR
[Shows first 20, user can scroll to see more]
```

**Acceptance Criteria:**
- [ ] All databases are accessible via tab completion (100+ databases in test system)
- [ ] Scrolling through completion menu shows all available options
- [ ] If menu has size limit, implement proper pagination or filtering
- [ ] Visual indicator shows "X of Y results" if paginated
- [ ] Tested with live Teradata database with 100+ databases
- [ ] User validates fix works in real REPL session

**Estimated Complexity:** Medium - Requires understanding reedline completion menu limits

**Testing Requirements:** MANDATORY live database testing with 100+ databases

---

#### Bug 2: Multi-Line Tab Completion Broken

**Issue:** Tab completion fails after newline. Example:
```
tq> SELECT * FROM database_name.
...> <Tab>
```
Shows SQL keywords instead of tables in the specified database.

**User Impact:** CRITICAL - Multi-line queries are common, completion should work across lines

**Root Cause:** SQL context detection (sql_context.rs) doesn't preserve context across line boundaries

**Expected Behavior:**
```
tq> SELECT * FROM DBC.
...> <Tab>
[Shows tables in DBC database, not SQL keywords]
```

**Acceptance Criteria:**
- [ ] Tab completion works correctly after newline in multi-line SQL statements
- [ ] SQL context detection spans multiple lines correctly
- [ ] Schema-qualified completion works: `FROM database.<newline><Tab>`
- [ ] Tested with various multi-line query patterns
- [ ] User validates fix works in real REPL session

**Estimated Complexity:** Medium - Requires fixing SQL context parser to handle multi-line input

**Testing Requirements:** MANDATORY live database testing with multi-line queries

---

#### Bug 3: Error Messages Show Full Stack Traces

**Issue:** When SQL errors occur, users see full Go stack traces from the Teradata driver instead of clean error messages.

**User Impact:** HIGH - Unprofessional, confusing, obscures actual error message

**Current Behavior:**
```
Error: SQL syntax error: [Version 20.0.49] [Session 1429] [Teradata Database] [Error 3707] Syntax error...
 at gosqldriver/teradatasql.formatError ErrorUtil.go:101
 at gosqldriver/teradatasql.(*teradataConnection).formatDatabaseError ErrorUtil.go:210
 at gosqldriver/teradatasql.(*teradataConnection).makeChainedDatabaseError ErrorUtil.go:226
 ... 15 more lines of stack trace ...
```

**Expected Behavior:**
```
Error: SQL syntax error - Expected something like an 'UDFCALLNAME' keyword between '.' and the 'AS' keyword.
[Error 3707] [Session 1429]
```

**Acceptance Criteria:**
- [ ] SQL errors show only the relevant error message and code
- [ ] Stack traces are suppressed for end users
- [ ] Error message is actionable and clear
- [ ] Session and error code included for debugging
- [ ] Tested with various SQL error scenarios
- [ ] User confirms error messages are professional and helpful

**Estimated Complexity:** Low - Error formatting and filtering

**Testing Requirements:** Generate various SQL errors and verify clean output

---

### P1 - High Priority (Should Fix)

Important quality issues that affect user experience.

#### Bug 4: Incorrect LIMIT Hint Message

**Issue:** When displaying large result sets, hint says "Add LIMIT clause" but Teradata uses TOP or SAMPLE syntax, not LIMIT.

**User Impact:** MEDIUM - Confuses users, suggests invalid syntax

**Current Message:**
```
Showing first 100 rows. Add LIMIT clause for different results.
```

**Expected Message:**
```
Showing first 100 rows. Use TOP N or SAMPLE N for different results.
```

**Additional Requirements:**
- Update all documentation references from LIMIT to TOP/SAMPLE
- Ensure help text uses correct Teradata syntax
- Examples should show Teradata patterns

**Acceptance Criteria:**
- [ ] All hint messages use "TOP N or SAMPLE N" instead of "LIMIT"
- [ ] Help text (`/help`, `--help`) updated with Teradata syntax
- [ ] Examples in error messages show correct syntax
- [ ] User confirms messages are clear and accurate

**Estimated Complexity:** Low - Text changes and search/replace

**Testing Requirements:** Visual inspection of all user-facing messages

---

#### Bug 5: Pager Navigation Needs Validation

**Issue:** Result paging with arrows was fixed in Sprint 8 Round 3, but hasn't been validated with live database queries.

**User Impact:** MEDIUM - If pager doesn't work, users can't navigate large result sets

**Requirements:**
- Vertical scrolling (j/k, Page Up/Down)
- Horizontal scrolling (h/l, arrow keys) for wide tables
- Exit pager with 'q' returns to REPL (doesn't exit program)
- Position indicators show current location

**Acceptance Criteria:**
- [ ] Test with query returning 1000+ rows - vertical scrolling works
- [ ] Test with query returning 20+ columns - horizontal scrolling works
- [ ] Test 'q' exit returns to REPL prompt (not exit program)
- [ ] Position indicators display correctly
- [ ] User validates pager UX is smooth and intuitive

**Estimated Complexity:** Low - Likely already works, needs validation only

**Testing Requirements:** MANDATORY live database testing with large result sets

---

### P2 - Code Quality (Nice to Have)

Technical debt that doesn't directly impact users but affects maintainability.

#### Bug 6: Build Warnings (Unused Imports)

**Issue:** Build produces warnings for unused imports, affecting code quality perception.

**Warnings:**
```
warning: unused imports: `PagerConfig`, `display_with_pager`, and `should_page`
  --> src/commands/repl/executor.rs:11:20

warning: unused import: `TableInfo`
  --> src/commands/repl/metadata_completer.rs:18:60
```

**User Impact:** LOW - Not user-facing, but indicates technical debt

**Acceptance Criteria:**
- [ ] Run `cargo fix --lib -p tq` to auto-remove unused imports
- [ ] Build completes with zero warnings
- [ ] No functionality broken by import removal

**Estimated Complexity:** Trivial - Automated fix

**Testing Requirements:** Run build, verify zero warnings, run test suite

---

### Explicitly Out of Scope

**NO new features in Sprint 9:**
- ❌ Batch mode (file input, stdin)
- ❌ Configuration files
- ❌ Connection profiles
- ❌ Additional completion features
- ❌ Performance optimization
- ❌ Architectural improvements (e.g., pager refactor)

**Rationale:** Focus 100% on completing bug fixes with high quality. New features will be considered in Sprint 10+ after quality is restored.

---

## Success Criteria

Sprint 9 is considered successful when ALL of the following are true:

### Code Quality
- [ ] All P0 bugs fixed and validated with live database
- [ ] All P1 bugs fixed and validated with live database
- [ ] P2 bug fixed (clean build, zero warnings)
- [ ] 100% unit test pass rate
- [ ] Zero technical debt introduced
- [ ] Clean build with zero warnings

### Testing
- [ ] Each bug fix tested immediately with live database
- [ ] User validates EACH fix before moving to next bug
- [ ] Comprehensive test results documented for all fixes
- [ ] Manual REPL testing confirms all features work smoothly

### User Validation
- [ ] User confirms Bug 1 fixed (all databases accessible)
- [ ] User confirms Bug 2 fixed (multi-line completion works)
- [ ] User confirms Bug 3 fixed (clean error messages)
- [ ] User confirms Bug 4 fixed (correct hint messages)
- [ ] User confirms Bug 5 validated (pager works correctly)
- [ ] User approves v1.5.1 release

### Documentation
- [ ] Sprint review created with honest assessment
- [ ] specifications.md updated (all 🔧 → ✅)
- [ ] Release notes written for v1.5.1

---

## Methodology: Fix-Test-Validate Loop

**CRITICAL:** Sprint 9 uses a strict one-bug-at-a-time approach:

### The Loop

```
For each bug in priority order:
  1. Analyze root cause
  2. Implement fix
  3. Build and run unit tests
  4. Test with live database IMMEDIATELY
  5. Document test results
  6. Get user validation
  7. If user approves: Mark COMPLETE, move to next bug
  8. If user rejects: Iterate fix, return to step 2
```

### Why This Works

- **Fast feedback:** Catch issues immediately, not at end of sprint
- **No wasted effort:** Don't fix bugs 4-6 if bugs 1-3 need rework
- **User confidence:** User sees incremental, verified progress
- **Quality focus:** Each bug completed thoroughly before moving on

### What This Means

- Sprint Coordinator pauses after each fix for user validation
- User MUST test each fix in their REPL before sprint continues
- No "batch testing at the end" - testing is continuous
- Sprint duration may vary based on fix complexity and iteration

---

## Dependencies

### External Dependencies
- **Live Teradata database** - MANDATORY for all testing
- **User availability** - Required for validation after each fix
- **TQ_LOGON configured** - Database connection must work

### Prerequisite Work
- Database connectivity verified (run `./target/release/tq ping`)
- User has time to test each fix (estimate 10-15 min per fix)
- Test database has sufficient data:
  - 100+ databases for Bug 1 testing
  - Large tables (1000+ rows) for Bug 5 testing
  - Wide tables (20+ columns) for Bug 5 testing

### Blockers
- **Blocker 1:** Database unavailable
  - **Mitigation:** Verify connectivity at sprint start, user keeps database accessible
- **Blocker 2:** User unavailable for validation
  - **Mitigation:** Sprint pauses, resumes when user available

---

## Risks & Mitigation

### Risk 1: Reedline Completion Menu Has Hard Limits

- **Probability:** Medium
- **Impact:** High (Bug 1 may be unfixable without library changes)
- **Mitigation:**
  - Research reedline documentation for menu size limits
  - Consider alternative UX (e.g., fuzzy filtering instead of scrolling)
  - If unfixable: Document limitation, provide workaround (prefix filtering)

### Risk 2: Multi-Line Context Detection Complex

- **Probability:** Medium
- **Impact:** Medium (Bug 2 may require significant refactoring)
- **Mitigation:**
  - Analyze sql_context.rs carefully before starting
  - Consider passing full buffer instead of just last line
  - Budget extra time for this fix

### Risk 3: More Bugs Discovered During Testing

- **Probability:** Medium
- **Impact:** Medium (extends sprint duration)
- **Mitigation:**
  - Document new bugs for Sprint 10, don't expand Sprint 9 scope
  - Focus on planned bugs only
  - User validation helps catch issues early

### Risk 4: Fixes Break Existing Functionality

- **Probability:** Low
- **Impact:** High (regressions)
- **Mitigation:**
  - Run full test suite after each fix
  - Test not just the bug but surrounding functionality
  - User validates holistically, not just the specific bug

---

## Agent Assignments

### cli-ux-designer (Sonnet)

**Responsibilities:**
- Design clean error message format for Bug 3
- Write updated hint messages for Bug 4
- Review all user-facing text for Teradata syntax correctness
- Update specifications.md after sprint completion

**Deliverables:**
- Error message format specification
- Updated hint message text
- Documentation updates
- Updated specifications.md (🔧 → ✅ transitions)

---

### rust-teradata-architect (Opus)

**Responsibilities:**
- Fix Bug 1 (database list display limit)
- Fix Bug 2 (multi-line tab completion)
- Fix Bug 3 (error message formatting)
- Implement Bug 4 (hint message changes)
- Run `cargo fix` for Bug 6 (unused imports)
- Document root cause for each bug
- Update rust-architecture.md if needed

**Deliverables:**
- Working fixes for all bugs with code comments
- Root cause documentation
- Updated unit tests (100% pass rate)
- Clean build with zero warnings
- Technical implementation notes

---

### quality-validator (Sonnet)

**Responsibilities:**
- Execute unit + integration tests after each fix
- **CRITICAL:** Test each fix with live database immediately
- Create test cases if needed
- Document test results for each bug fix
- Provide user with clear test instructions
- Generate final test report at sprint end

**Deliverables:**
- Test execution results for each bug (1-6)
- Live database test validation for P0/P1 bugs
- Clear test instructions for user validation
- Final sprint test report in tests/results/
- 100% test pass rate across all test types

---

### tq-project-manager (Haiku)

**Responsibilities:**
- Validate sprint completion at closure
- Verify user validated all fixes
- Assess final code quality
- Provide go/no-go decision for v1.5.1 release
- Review sprint execution effectiveness

**Deliverables:**
- Sprint completion validation report
- Code quality assessment (should be Grade A)
- User validation confirmation
- Go/no-go recommendation for release
- Process improvement recommendations

---

## Sprint Timeline

**Estimated Duration:** 1-2 days (depends on fix complexity and user availability)

**Philosophy:** No time pressure. Complete bugs thoroughly, even if takes longer.

### Phase Breakdown

#### Phase 1: Planning ✅ COMPLETE
- Sprint planning document created
- User approval obtained

#### Phase 2: Design Phase (Est. 2-3 hours)
- **Parallel execution:** cli-ux-designer + rust-teradata-architect
- cli-ux-designer: Design error messages and hint text
- rust-teradata-architect: Analyze root causes for all 6 bugs
- **Output:** Clear fix strategy for each bug, prioritized order

#### Phase 3: Implementation Phase (Est. 6-10 hours)
- **Sequential execution:** One bug at a time, with user validation
- **For each bug (P0 first, then P1, then P2):**
  1. Rust-teradata-architect implements fix
  2. Build and run unit tests
  3. Quality-validator tests with live database
  4. Provide user with test instructions
  5. User validates fix in real REPL
  6. If approved: Move to next bug
  7. If issues: Iterate fix
- **No parallelism in Phase 3:** Sequential ensures quality

#### Phase 3.5: Database Connectivity Check (MANDATORY)
- Run `./target/release/tq ping` to verify database connection
- If fails: STOP and wait for user to fix database
- Only proceed when database confirmed working

#### Phase 4: Final Validation (Est. 1-2 hours)
- Quality-validator: Run comprehensive test suite
- Quality-validator: Execute any missed test cases
- User: Final acceptance testing of all fixes together
- Verify no regressions, all fixes still working
- 100% test pass rate required

#### Phase 5: Sprint Closure (Est. 1-2 hours)
- tq-project-manager: Validate completion
- Sprint Coordinator: Create sprint review
- Update specifications.md (all 🔧 → ✅)
- Update roadmap
- Create v1.5.1 release notes
- Get user approval for release

---

## Quality Gate: Continuous User Validation

**NEW for Sprint 9:** User validation is CONTINUOUS, not end-of-sprint.

### Validation Checkpoints

After implementing each fix:
1. **Automated tests** run (unit + integration)
2. **Live database test** by quality-validator
3. **User validation** with real REPL session
4. **Approval gate:** User says "fixed" or "not fixed"
5. **If fixed:** Document and move to next bug
6. **If not fixed:** Iterate immediately, don't move on

### User Validation Requirements

For each P0/P1 bug, user must:
- Test fix in their real Teradata environment
- Confirm bug is truly fixed (not just "better")
- Report any remaining issues or edge cases
- Provide explicit "approved" or "needs work" decision

### Why This Works

- Prevents "supposedly fixed" situations from Sprints 5-7
- Fast feedback enables quick iteration
- Builds user trust through visible progress
- Ensures sprint doesn't close with broken "fixes"

---

## Sprint Workflow Improvements

Based on Sprint 8 learnings:

### Changes from Sprint 8

1. **Sequential fixes, not parallel**
   - Sprint 8: Tried to fix all bugs at once
   - Sprint 9: Fix one bug completely, then move to next

2. **Immediate testing, not batched**
   - Sprint 8: Implement all fixes → test at end
   - Sprint 9: Implement one fix → test immediately

3. **User validation as gate, not optional**
   - Sprint 8: User testing at end of sprint
   - Sprint 9: User validates each fix before next starts

4. **Conservative scope, higher quality**
   - Sprint 8: 4 bugs, partially fixed
   - Sprint 9: 6 bugs, but okay if only complete 3-4 thoroughly

### Success Metrics

- **Quality over quantity:** Better to fix 3 bugs completely than 6 bugs partially
- **User satisfaction:** User explicitly approves each fix
- **Zero regressions:** Full test suite runs after every fix
- **Professional output:** Clean builds, clean errors, clean UX

---

## Action Items from Sprint 8

Implementing lessons learned:

- ✅ Test each fix with live database immediately (not batch at end)
- ✅ Get user validation for each fix before moving to next
- ✅ Run `cargo fix` and `cargo clippy` before sprint closure
- ✅ Focus on small scope, complete thoroughly
- ✅ Document test results for each fix iteration

---

## Notes

### Critical Context

Sprint 9 is about redemption. Sprints 5-8 damaged user trust by shipping incomplete or broken features. Sprint 9 must restore that trust by:

1. **Finishing what we started** - Complete all Sprint 8 remaining bugs
2. **Thorough validation** - User confirms every fix works
3. **Professional quality** - Clean builds, clean errors, clean code
4. **No shortcuts** - Test properly, document properly, validate properly

### Definition of "Fixed"

A bug is "fixed" when:
- Code implements the fix
- Unit tests pass
- Live database testing passes
- User validates in real REPL session
- User explicitly says "approved"
- No regressions detected

### Definition of "Sprint Complete"

Sprint 9 is complete when:
- All P0 bugs are FIXED (as defined above)
- All P1 bugs are FIXED (as defined above)
- P2 bug is fixed (clean build)
- User approves v1.5.1 release
- Sprint review documents quality and learnings
- Specifications.md updated with ✅ status

---

## Expected Outcomes

### For Users
- Tab completion works perfectly (all databases, multi-line support)
- Error messages are clean and professional
- Pager works smoothly for large result sets
- Hints use correct Teradata syntax
- Confidence restored in tq quality

### For Project
- v1.5.1 release that users can trust
- Clean codebase with zero warnings
- Comprehensive test coverage
- Quality-first culture established
- Foundation for Sprint 10+ new features

### For Process
- Proof that fix-test-validate loop works
- User validation as quality gate proven effective
- Sequential approach delivers higher quality than parallel
- Foundation for future sprint execution

---

## Approval

**Status:** PENDING USER APPROVAL

**Questions for User:**

1. ✅ Is the bug prioritization correct (P0: bugs 1-3, P1: bugs 4-5, P2: bug 6)?
2. ✅ Are you available to validate fixes as we go (not batch at end)?
3. ✅ Is your test database accessible and ready?
4. ✅ Any other bugs we missed that should be in Sprint 9?
5. ✅ Agree that Sprint 9 should be bug-fixes only, no new features?

**User Approval Required Before Proceeding to Phase 2**

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-18 | 1.0 | Initial Sprint 9 plan - Complete Quality Recovery | Sprint Coordinator |
