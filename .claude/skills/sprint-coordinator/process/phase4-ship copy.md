# Phase 4: Ship

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Validate, commit, and document the sprint.

## Prerequisites
- Phase 3 completed with 100% test **EXECUTION** rate + 100% pass rate
- Test report available in `tests/results/.../REPORT.md`
- **CRITICAL**: Tests must have been EXECUTED, not code reviewed

## Process

### Step 1: Validate Against Definition of Done

**FIRST: Read quality-validator report and verify execution proof**
- Does report contain actual `cargo test` output?
- Were interactive tests run with `--ignored` flag?
- Are there execution timestamps/results, not just "code looks good"?

**BLOCKING: If tests were NOT executed → STOP and go back to Phase 3**

**THEN: Read `definitions/done.md` and verify:**

| Criterion | Check | Required |
|-----------|-------|----------|
| **Tests EXECUTED** | All tests ran and produced output? NOT just code reviewed? | ✅ MUST |
| **Functional Correctness** | All EXECUTED tests pass (100%)? | ✅ MUST |
| **Interactive Tests Run** | Tests with `#[ignore]` executed with `--ignored` flag? | ✅ MUST |
| **Code Quality** | No new `TODO`/`FIXME` comments? `cargo clippy` clean? | ✅ MUST |
| **Documentation Sync** | Code matches specs? No drift? | ✅ MUST |
| **Architecture Compliance** | Implementation follows `rust-architecture.md`? | ✅ MUST |
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

Co-Authored-By: Claude <noreply@anthropic.com>"
git push origin main
```

### Step 4: Update Roadmap (Quick Status Update)

Update `specifications.md`:
- Mark sprint status as ✅ Complete
- Update version status

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
