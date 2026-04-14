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
| **All Targets Tested** | `cargo test --all-targets` passes (integration tests included)? | ✅ MUST |
| **Functional Correctness** | All EXECUTED tests pass (100%)? | ✅ MUST |
| **Interactive Tests Run** | Tests with `#[ignore]` executed with `--ignored` flag? | ✅ MUST |
| **Code Quality** | No new `TODO`/`FIXME` comments? `cargo clippy --all-targets` clean (zero warnings)? | ✅ MUST |
| **Documentation Sync** | Code matches specs? No drift? | ✅ MUST |
| **Architecture Compliance** | Implementation follows `docs/design/*.md`? | ✅ MUST |
| **Zero Technical Debt** | No workarounds introduced? | ✅ MUST |

### Step 1.5: Documentation Accuracy Verification

**PURPOSE:** Prevent documentation/implementation mismatches (Sprint 22 & 23 lesson)

**CRITICAL**: Verify that user-facing documentation matches what was actually delivered.

**Check User Guides** (`docs/user/*.md`):
- [ ] All code examples use correct syntax (e.g., glob patterns not SQL LIKE)
- [ ] All feature descriptions match actual implementation
- [ ] No deferred features documented as implemented
- [ ] All flag names match actual CLI flags (e.g., no `--force` if not implemented)
- [ ] Error messages in docs match actual error messages in code

**Check Specifications** (`docs/specifications/*.md`):
- [ ] Requirements marked as implemented are actually implemented
- [ ] No requirements documented that were deferred to future sprints
- [ ] Examples in specifications execute correctly with the tool

**Check CLI Help Text**:
- [ ] Run `cargo run --release -- --help` and verify all flags documented
- [ ] Run `cargo run --release -- <subcommand> --help` for each subcommand
- [ ] Help text matches user guide descriptions

**How to Verify**:
1. Read `docs/sprints/sprint-N-planning.md` "Scope" section
2. List all P0 and P1 features with DELIVERED status
3. For each delivered feature:
   - Find its documentation in `docs/user/*.md`
   - Verify description matches implementation
   - Test examples execute correctly
4. For each deferred feature:
   - Verify it's NOT documented in user guides
   - Verify specifications mark it as future work

**Common Issues to Check** (from Sprints 22, 23, 38, 39):
- Pattern syntax mismatch (SQL LIKE `%` vs glob `*`)
- Flags documented but not implemented (`--force`)
- Loading indicators described but not built
- Session types described incorrectly
- Qualified patterns documented but not supported
- **Output schema mismatch**: User guide/spec shows different columns or properties than actual implementation (Sprint 38: Node Count/PE Count; Sprint 39: Session/User/Query Text vs multi-query history)
- **Alias conflicts**: Short aliases documented but already used by another command (Sprint 39: `/q` conflicts with `/quit`)
- **Error message text divergence**: User guide shows one error message, code produces a different one
- **JSON/CSV field names mismatch**: Scripting examples use wrong field names for `jq` or parsing
- **Help text describing unimplemented features**: Extended help mentions features not in the code (Sprint 39: sysconfig "nodes, PEs")

**Output Schema Verification** (added Sprint 39):
For each new command, compare ACTUAL output against documented output:
1. Read the display function in source code (display_repl_table, display_csv, display_json)
2. List actual columns/properties from the code
3. Compare against user guide example output
4. Compare against specification (REQ-XXX) output examples
5. Fix any discrepancies BEFORE commit

**Verdict**:
- ✅ **PASS**: All documentation matches delivered features
- ❌ **FAIL**: Documentation mismatches found → Fix docs before commit

### Step 1.6: Minor Task Completion Check

**PURPOSE:** Prevent deferred minor tasks that waste tokens in follow-up sprints.

**CRITICAL**: Tasks estimated at <30 minutes MUST be completed NOW, not deferred.

**Check for Minor Gaps:**
- [ ] Any emoji badges or visual polish missing? Fix now (5-10 min)
- [ ] Any help text updates needed? Fix now (10-15 min)
- [ ] Any spec/implementation verification pending? Do now (10-15 min)
- [ ] Any warning message format issues? Fix now (15 min)

**Cost Calculation:**
- Deferring a 15-min task creates: task tracking overhead + context reload + execution = ~30 min equivalent
- Complete minor tasks NOW to avoid double-handling

**Rule:**
- Tasks <30 min → Complete in this sprint
- Tasks 30-60 min → Complete if time permits
- Tasks >60 min → Document for next sprint (acceptable deferral)

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

### Step 3.1: Tag Release and Trigger Build

**CRITICAL**: After successful push, create a version tag and push it to trigger the GitHub Actions release workflow. This builds cross-platform binaries and creates a GitHub Release.

1. **Read the version** from `Cargo.toml` (should already be bumped during implementation)
2. **Create and push the tag:**

```bash
# Get version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
git tag "v${VERSION}"
git push origin "v${VERSION}"
```

3. **Verify the release workflow started:**
```bash
gh run list --workflow=release.yml --limit=1
```

4. **Do NOT wait for the workflow to complete** — it runs in CI. Proceed to the next step. If the workflow fails, it will be caught in the next sprint's Phase 0 reality check or flagged by the user.

**NEVER skip this step.** Every sprint ships a tagged release with cross-compiled binaries.

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
