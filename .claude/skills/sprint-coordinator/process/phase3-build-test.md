# Phase 3: Build & Test

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Implement features and validate them with tests—in parallel.

## Warmup (for Sub-Agents)
This phase follows Design (Phase 2). At this point:
- Specifications in `detailed-specifications/*.md` are finalized.
- Architecture in `rust-architecture.md` is approved.
- You are working in parallel with another agent.

## Process

### Step 1: Launch Parallel Agents

Launch BOTH agents in a **single message with multiple Task calls**:

1. **`rust-teradata-architect`**:
   - Instruction: "Implement the features defined in `detailed-specifications/*.md` for Sprint N. Follow patterns in `rust-architecture.md`. Run `cargo check` and `cargo clippy` before returning. Return a summary of what was implemented."

2. **`quality-validator`**:
   - Instruction: "Design and implement tests for the features in Sprint N. Use the specifications in `detailed-specifications/*.md`. **CRITICAL: EXECUTE ALL TESTS including ignored tests with --ignored flag. Include test execution output as proof. Code review is NOT execution.** Return a report with pass/fail counts. Use the template in `.claude/templates/quality-report-template.md`."

### Step 2: Collect Results

Wait for both agents to complete. Expect:
- From Architect: Summary of implemented code.
- From Validator: Test report with structured verdict.

### Step 3: Evaluate

**CRITICAL VERIFICATION - Check the Validator's report:**

**FIRST: Verify tests were EXECUTED, not just reviewed**
- Does report include actual test execution output?
- Were interactive tests run with `--ignored` flag?
- Is there proof of execution (cargo test output)?

**BLOCKING CONDITIONS:**
- If verdict is **BLOCKED** (tests couldn't run): Sprint cannot ship. Fix blockers first.
- If tests were NOT EXECUTED: Treat as BLOCKED. Tests must run.
- If report says "validated via code review": **REJECT**. Code review is not execution.

**THEN: Check pass rate**
- **100% execution + 100% pass rate?** → Proceed to Phase 4 (Ship)
- **Failures?** → Loop: Re-launch Architect to fix, then Validator to re-test
- **Not executed?** → BLOCKED: Fix environment (database, credentials) and re-run

## Output
- Implemented code in `src/`.
- Test report in `tests/results/.../REPORT.md`.
- Proceed to Phase 4 if all tests pass.
