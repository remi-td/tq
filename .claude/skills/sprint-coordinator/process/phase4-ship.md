# Phase 4: Ship

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Validate, commit, and document the sprint.

## Prerequisites
- Phase 3 completed with 100% test **EXECUTION** rate + 100% pass rate
- Test report available in `tests/results/.../REPORT.md`
- **CRITICAL**: Tests must have been EXECUTED, not code reviewed

## Process

### Step 1: Validate Against Definition of Done

**FIRST: Read quality-validator report and verify execution proof in `tests/results/sprint-N/`** 
- Does report contain actual `cargo test` output?
- Were interactive tests run with `--ignored` flag?
- Are there execution timestamps/results, not just "code looks good"?

 `docs/sprints/sprint-N-planning.md`

**BLOCKING: If tests were NOT executed → STOP and go back to Phase 3**

**THEN: Read `definitions/done.md` and verify:**

| Criterion | Check | Required |
|-----------|-------|----------|
| **Tests EXECUTED** | All tests ran and produced output? NOT just code reviewed? | ✅ MUST |
| **Functional Correctness** | All EXECUTED tests pass (100%)? | ✅ MUST |
| **Interactive Tests Run** | Tests with `#[ignore]` executed with `--ignored` flag? | ✅ MUST |
| **Code Quality** | No new `TODO`/`FIXME` comments? `cargo clippy` clean? | ✅ MUST |
| **Documentation Sync** | Code matches specs? No drift? | ✅ MUST |
| **Architecture Compliance** | Implementation follows `docs/design/*.md`? | ✅ MUST |
| **Zero Technical Debt** | No workarounds introduced? | ✅ MUST |

### Step 2: Decision

**CRITICAL CHECKS:**
- **Tests NOT executed?** → BLOCKED: Cannot ship. Go back to Phase 3, fix environment, execute tests.
- **Quality report based on code review?** → REJECTED: Code review is not execution. Go back to Phase 3.
- **Interactive tests skipped?** → REJECTED: REPL features require interactive test execution. Go back to Phase 3.

**STANDARD CHECKS:**
- **All criteria met + tests executed?** → Proceed to Step 3
- **Any other criterion failed?** → Loop back to Phase 3. Document what needs fixing

### Step 3: Commit and Push

```bash
git add .
git commit -m "Complete Sprint N: [Summary of features]
git push origin main
```

### Step 3.5: Update GitHub Issues

**CRITICAL**: After successful push, use `/github-issues` skill to update completed issues.

For each GitHub issue addressed in this sprint:

**If fully implemented:**
```bash
gh issue close <number> --comment "$(cat <<'EOF'
## Implementation Complete ✅

**Implemented in Sprint N**

**Changes:**
- [Summary of implementation]

**Commits:**
- <commit-hash>: <commit-message>

**Documentation:**
- Updated: `docs/specifications/<file>.md`
- Design: `docs/design/<file>.md`
- Tests: `tests/cases/TC-XXX.md`

**Test Results:** ✅ All tests passing

**Sprint Review:** See docs/sprints/sprint-N-review.md for full details.

Thank you for the feature request!
EOF
)"
```

**If partially implemented or needs follow-up:**
```bash
gh issue comment <number> --body "$(cat <<'EOF'
## Progress Update - Sprint N

**Completed:**
- [What was implemented]

**Remaining:**
- [What's still pending]

**Status:** Keeping issue open for remaining work

**Sprint Review:** See docs/sprints/sprint-N-review.md
EOF
)"
```

**Verification:**
- All issues mentioned in sprint-N-planning.md are updated
- Closed issues have complete implementation details
- Open issues have progress comments

### Step 4: Update Roadmap (Required)

**CRITICAL**: These updates are MANDATORY. Roadmap tracking depends on this step.

#### Update `docs/roadmap/status.md`

1. **Read the file** to understand current status
2. **For each feature delivered in this sprint**:
   - Find the feature section in status.md
   - Change status from 🚧 or 📋 to ✅
   - Add version number: `✅ v1.X.0` (increment minor version)
   - Add sprint reference: `(Sprint N)`
3. **Update summary statistics** at the top:
   - Count total ✅, 🚧, 📋 features
   - Update percentages
   - Update "Last Updated" date

**Example**:
```markdown
Before:
## Tab Completion
📋 Planned - Context-aware suggestions

After:
## Tab Completion
✅ v1.2.0 (Sprint 15) - Context-aware suggestions
```

#### Update `docs/roadmap/backlog.md`

1. **Read the file** to see current backlog
2. **For each feature completed**:
   - Remove it from backlog (it's now in status.md)
   - If it was blocking other features, update dependencies
3. **Reprioritize if needed**:
   - Move items between P0/P1/P2 based on what's now possible
   - GitHub issues with `sprint-ready` label are potential backlog items

**Verification**:
- No feature should appear in BOTH status.md (as ✅) AND backlog.md
- Completed features fully removed from backlog

**NOTE:** Full sprint review will be created in Phase 5 using sprint-reviewer skill.

### Step 5: Proceed to Phase 5

**CRITICAL: Do NOT skip Phase 5**

Phase 4 commits the code. Phase 5 creates the comprehensive retrospective.

Read `process/phase5-review.md` and:
- Invoke the `/sprint-reviewer` skill
- Do NOT manually create sprint-N-review.md
- Sprint-reviewer will launch 3 parallel agent reviews
- Sprint-reviewer will collect token/cost metrics
- Sprint-reviewer will use proper template

## Output
- Code committed and pushed ✅
- Specifications.md updated with sprint completion ✅
- **Ready for Phase 5 (Retrospective)**
