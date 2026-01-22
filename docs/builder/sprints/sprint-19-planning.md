# Sprint 19 Planning: CRITICAL BUG FIXES - Logo & Tab Completion (Retry)

**Date:** 2026-01-22
**Type:** Maintenance Sprint (CRISIS)
**Priority:** P0 - BLOCKING PRODUCTION USE

---

## Reality Check Summary

**Reviewed Sprints:** 18 (incomplete), 17, 16

**CRITICAL PATTERN DETECTED:**
- **Sprint 18 was planned but NEVER EXECUTED** - planning exists but no review, no delivery
- **Same bugs reported TWICE** - User reported identical bugs again in `open-bugs.md`
- **Tab completion regression pattern** - Broken across Sprints 7, 8, 9, 11, 13, 18
- **Framework failure** - Sprint planning created but never delivered

**Decision:** MAINTENANCE SPRINT (CRISIS - RETRY SPRINT 18)

**Rationale:** User reports TWO BLOCKING bugs that Sprint 18 was supposed to fix but never did:
1. **Logo branding wrong**: ASCII art instead of lowercase "tq" with subtitle
2. **Tab completion broken**: Debug traces showing instead of completions

**Root Cause:** Sprint 18 failed to deliver. Must execute immediately with simplified scope.

---

## Sprint Objectives

### P0 - CRITICAL (BLOCKING)

#### 1. Fix ASCII Art Logo Display

**Problem:** Logo displays ASCII block art instead of simple lowercase "tq" text with information

**User Specification:**
```
The ASCII art `tq` LOGO should be written in lowercase with the 't' in
Teradata orange color (#F37021) and 'q' in white/black. This big ASCII
art is our logo, the first thing the user sees. NEXT to it (on the right)
should be the welcome and information messages.
```

**Expected Output:**
```
tq    Teradata Query Tool v 1.7
      Connected to mcp-vikzqtnd0db0nglk.env.clearscape.teradata.com:1025
      Teradata version 20.00.00.00
      User: demo_user
      Default row limit: 100
      Editor mode: emacs
```

Where "tq" is rendered in lowercase ASCII art with 't' in Teradata orange (#F37021, xterm-256 color 202).

**Acceptance Criteria:**
- [ ] Logo shows lowercase "tq" in ASCII art (NOT uppercase blocks)
- [ ] 't' displayed in Teradata orange (xterm-256 color 202)
- [ ] 'q' displayed in white/black
- [ ] Information messages displayed to the RIGHT of logo
- [ ] Banner matches branding guidelines from `open-bugs.md`

**Files to Modify:**
- `src/commands/repl/mod.rs` (print_banner function)

**Effort Estimate:** 1 hour

---

#### 2. Fix Tab Completion - Remove Debug Output

**Problem:** Tab completion shows debug traces instead of actual completions

**User Report:**
```
tq> ? sel * fr[TAB]
Page 1: records 0 - 0  total: 0  [FULL]
```

Expected: Should show completion menu or complete the word "from"

**User Report 2:**
```
tq> ? sel * from dbc.t[TAB]
Page 1: records 0 - 0  total: 0
```

Expected: Should show tables in DBC database

**Root Cause:** Debug traces left in completion code, or completion returning wrong data

**Acceptance Criteria:**
- [ ] Tab completion shows completion menu (NOT debug traces)
- [ ] Completion after "from " shows database/table names
- [ ] Completion after "dbc." shows tables in DBC
- [ ] NO debug output or page headers in completion
- [ ] Completion works as it did in earlier sprints

**Files to Investigate:**
- `src/commands/repl/metadata_completer.rs`
- `src/commands/repl/sql_context.rs`
- Any recent changes to completion code

**Approach:**
1. Find where "Page 1: records..." output comes from
2. Ensure completions return proper Suggestion objects
3. Test with live REPL
4. Validate each completion context works

**Effort Estimate:** 2-3 hours

---

## P1 - High Priority

#### 3. Test Both Fixes with Live Database

**Acceptance Criteria:**
- [ ] Launch tq in REPL mode
- [ ] Verify logo displays correctly with orange 't'
- [ ] Test tab completion after "SELECT * FROM "
- [ ] Test tab completion after "SELECT * FROM dbc."
- [ ] No debug traces appear
- [ ] User validates fixes work

**Effort Estimate:** 30 minutes

---

## Sprint Success Criteria

### Definition of Done
- [ ] Logo shows lowercase "tq" with info on right side
- [ ] Tab completion shows completion menu (not debug traces)
- [ ] Both bugs verified fixed with live database
- [ ] Zero regressions in existing functionality
- [ ] User validates fixes resolve blocking issues

### Quality Gates
- [ ] 100% existing test pass rate
- [ ] Zero build warnings
- [ ] Zero clippy warnings
- [ ] Manual validation with live database

---

## Sprint Scope: MAINTENANCE ONLY

This is a **Maintenance Sprint** focused on fixing TWO critical bugs. NO new features. NO scope expansion.

**In Scope:**
- Fix logo display (lowercase with info messages)
- Fix tab completion (remove debug output)
- Test and validate fixes

**Out of Scope:**
- New features
- Tab completion enhancements
- Performance optimizations
- Documentation updates

---

## Phase Execution Plan

### Phase 0: Reality Check ✅ COMPLETE
- Reviewed sprints 16, 17, 18 (incomplete)
- Identified Sprint 18 failure pattern
- Detected critical user-reported bugs
- **Decision:** Maintenance Sprint (Retry Sprint 18)

### Phase 1: Planning ✅ COMPLETE
- Created sprint-19-planning.md
- Defined 2 P0 objectives (logo + tab completion)
- Documented acceptance criteria
- Simplified scope for fast execution

### Phase 2: Design (SKIP - Emergency Bug Fixes)
- These are bug fixes with clear specifications
- User provided exact requirements in open-bugs.md
- Proceed directly to Phase 3 implementation

### Phase 3: Implementation & Testing
- rust-teradata-architect: Fix both bugs
- quality-validator: Test with live database
- Iterate until both P0 objectives complete

### Phase 4: Ship
- Validate against Definition of Done
- Git commit with bug fix descriptions
- Push to master

### Phase 5: Retrospective
- Use `/sprint-reviewer` skill
- Document why Sprint 18 failed
- Identify process improvements

---

## Sprint Risks

| Risk | Impact | Mitigation |
|------|--------|----------|
| Sprint 18 failure pattern repeats | CRITICAL | Simplified scope, clear specs |
| Tab completion complexity | HIGH | Focus on debug output only |
| Logo ASCII art rendering | MEDIUM | Test in actual terminal |

---

## Success Metrics

### Before Sprint 19 (Current State - BROKEN)
- Logo: Wrong ASCII art ❌
- Tab completion: Shows "Page 1: records..." debug output ❌
- User productivity: BLOCKED ❌

### After Sprint 19 (Target State - FIXED)
- Logo: Lowercase "tq" with orange 't', info on right ✅
- Tab completion: Shows actual completions ✅
- User productivity: UNBLOCKED ✅

---

## Notes

**CRITICAL:** Sprint 18 was planned but never executed. This is Sprint 19 executing Sprint 18's objectives.

**User Expectation:** ASAP fix - these bugs are blocking productive work.

**Quality Bar:** User must validate fixes work. No "code looks correct" - must test in real terminal.

**Simplified Scope:** Only fix these 2 bugs. Don't rebuild tab completion system, just remove debug output.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-22 | 1.0 | Sprint 19 planning - Critical bug fixes (Sprint 18 retry) | Sprint Coordinator |
