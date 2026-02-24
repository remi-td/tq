# Phase 3: Build & Test

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Implement features and validate them with tests—in parallel.

## Warmup (for Sub-Agents)
This phase follows Design (Phase 2). At this point:
- Specifications in `docs/specifications/*.md` are finalized.
- Design documents in `docs/design/*.md` are updated and approved.
- For each feature, you will task the following agents to run in parallel:
   - the `rust-teradata-architect` agent to implement the feature
   - the `quality-validator` agent to implement the test case

## Process

### Step 1: Launch Parallel Agents

Launch BOTH agents in a **single message with multiple Task calls**:

1. **`rust-teradata-architect`**:
   - Instruction: "Implement the features defined in `docs/sprints/sprint-N-planning.md` as per requirements in `docs/specifications/*.md` for Sprint N using the design outlined in `docs/design/*.md`. Follow patterns in `docs/design/vision.md`. Compile and run the tool to validate that your feature is implemented. Return a summary of what was implemented."

2. **`quality-validator`**:
   - Instruction: "Design and implement tests for the features in Sprint N as per requirements in `docs/sprints/sprint-N-planning.md`. Use the specifications in `docs/specifications/*.md`. Document your strategy in `tests/strategy/` based on `tests/strategy/test-strategy-template.md`. Add test cases in `tests/cases` and use `tests/README.md`

2. **`cli-ux-designer`**:
   - Instruction: "Update the documentation for the features in Sprint N as per `docs/sprints/sprint-N-planning.md`. Use the specifications in `docs/specifications/*.md`. The documentation should be placed in `docs/user`. Make sure that the documentation is accurate, intuitive and easy to navigate.
   - **IMPORTANT (Sprint 39 lesson):** Tell the documentation agent to verify output examples against the ACTUAL implementation source code, not just the specification. Read display_repl_table/display_csv/display_json functions in the source to get the correct column names, output format, and error messages. Specifications may diverge from implementation when the architect makes pragmatic improvements.

### Step 2: Collect Results

Wait for both agents to complete. Expect:
- From Architect: Summary of implemented code.
- From Validator: Confirmation that the test strategy for the sprint is developed and cases created.

### Step 3: Validate

Calidate that the strategy defined for the sprint in `tests/strategy/` addresses the features in this sprint. If not provide feedback to the `quality-validator` and request adjustments. Repeat this process until the strategy is correct.

### Step 4: Execute Tests

We need to execute all tests and address bug fixes. Use the **two-iteration strategy** for comprehensive validation.

#### Two-Iteration Test Strategy

**Iteration 1: Core Logic (Unit Tests)**
- Run unit tests first: `cargo test --lib`
- Validates: Functions, modules, internal logic
- Gate: Must pass before Iteration 2

**Iteration 2: CLI Integration**
- Run integration tests: `cargo test --test '*'`
- Run interactive tests: `cargo test -- --ignored` (if database available)
- Validates: End-to-end behavior, CLI parsing, error handling

**Benefits:**
- Early detection of logic errors
- Clear separation of failure causes
- Progressive confidence building

#### Test Loop

Until all test cases pass, run a test round (I denotes the round number, starting 1):
1. Run **`quality-validator`**:
   - Instruction: "Execute the test cases for Sprint N, iteration I as per the strategy you defined in `tests/strategy/` and executing the test cases in `tests/cases`. Produce a test evidence `tests/results/sprint-N/test-evidence-I.md` and write or update the test report in `tests/results/sprint-N/`
2. Validate test evidence: if all tests are passed, end loop.
3. Run **`rust-teradata-architect`**:
   - Instruction: "Fix bugs identified in test evidence `tests/results/sprint-N/test-evidence-I.md` or provide justification.
4. Wait for the `rust-teradata-architect` agent to complete its tasks. If justification is provided, pass it to the `quality-validator` next round with the following message: "Developer provided explanation {justification message} for issue {test case reference}. Revise the test case if you accept or add justification in test evidence'.

**CRITICAL VERIFICATION - Check the Validator's report:** `tests/results/sprint-N/`

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

### Step 4: Synthesize

Review all sub-agents outputs:
- **Gaps?** Identify failed tests or unclear outcomes. Ensure that documentation was updated.
- **Scope?** Ensure all features in scope were delivered, tested and documented. If some were not or only delivered, move them to the backlog in `specifications.md` and update `sprint-N-planning.md` with a special note on these features.
- **Ready?** Proceed to Phase 3.
- **Status Update** Update the `sprint-N-planning.md` document to mark this phase as complete

## Output
- Implemented code in `src/`.
- Test strategy in `tests/strategy/`.
- Test cases in `tests/cases/`.
- Test evidence and results in `tests/results/sprint-N/` 
- Proceed to Phase 4.
