# Agent Coordination Guide

**Purpose:** This document provides guidance for the main Claude agent on when and how to coordinate specialized sub-agents.

**Last Updated:** 2026-01-18

---

## Overview

The tq project uses a **main agent coordination model** where the primary Claude agent orchestrates specialized sub-agents through parallel execution. The main agent owns the workflow, makes decisions, and synthesizes outputs from sub-agents.

**Key Principle:** The main agent is the coordinator. No sub-agent coordinates other sub-agents.

---

## Agent Roles Summary

| Agent | Model | Role | Launch When |
|-------|-------|------|-------------|
| **Main Agent** | Sonnet 4.5 | Coordinator & Decision Maker | Always active |
| **cli-ux-designer** | Sonnet | UX Authority & Specifications Owner | Designing features, updating specs |
| **rust-teradata-architect** | Opus | Implementation & Architecture Owner | Implementing features, refactoring |
| **quality-validator** | Sonnet | Test Designer & Executor | Designing tests, executing test suites |
| **tq-project-manager** | Haiku | Validator & Tech Debt Guardian | Validating completion, assessing quality |

---

## Main Agent Responsibilities

As the main agent, you:

1. **Own the Sprint Workflow**
   - Create sprint planning documents
   - Coordinate all phases of development
   - Make go/no-go decisions
   - Synthesize agent outputs into decisions

2. **Maximize Parallelism**
   - Launch independent agents in a single message with multiple Task calls
   - Example: Design + Feasibility review in parallel
   - Example: Implementation + Test design in parallel

3. **Maintain Context**
   - Keep main conversation clean and focused on decisions
   - Let sub-agents handle verbose work (code writing, test execution)
   - Ensure sub-agents return summaries, not full outputs

4. **Document Everything**
   - Create sprint planning documents (`sprint-N-planning.md`)
   - Create sprint review documents (`sprint-N-review.md`)
   - Update roadmap after each sprint
   - Track action items across sprints

---

## When to Launch Each Agent

### cli-ux-designer

**Launch When:**
- Starting a new sprint (Phase 2: Design)
- Designing new features or commands
- Refining user experience
- Updating specifications dashboard
- Creating detailed specifications

**Typical Prompts:**
- "Review sprint-N-planning.md and create detailed specifications for [features]"
- "Update specifications.md dashboard with 🚧 status for in-progress features"
- "Design the UX for [feature] following CLI best practices"

**Expected Outputs:**
- Updated `specifications.md`
- New or updated files in `detailed-specifications/`
- UX design recommendations

**Parallel With:**
- rust-teradata-architect (for technical feasibility assessment)

---

### rust-teradata-architect

**Launch When:**
- Assessing technical feasibility (Phase 2: Design)
- Implementing features (Phase 3: Implementation)
- Refactoring code
- Updating architecture documentation
- Addressing technical debt

**Typical Prompts:**
- "Review sprint-N-planning.md and assess technical feasibility for [features]"
- "Implement features from sprint-N-planning.md according to detailed specifications"
- "Refactor [module] to reduce technical debt"

**Expected Outputs:**
- Technical feasibility reports
- Working implementation
- Unit tests
- Updated `rust-architecture.md`
- Technical debt analysis

**Parallel With:**
- cli-ux-designer (during design phase)
- quality-validator (during implementation phase)

---

### quality-validator

**Launch When:**
- Designing test cases (Phase 3: Implementation)
- Executing test suites (Phase 4: Testing)
- Validating features before sprint closure
- Generating test reports

**Typical Prompts:**
- "Design comprehensive test cases for sprint-N features based on detailed specifications"
- "Execute all test suites (unit + integration) and generate test report"
- "Validate that [feature] meets all acceptance criteria"

**Expected Outputs:**
- Test case files in `tests/cases/TC###.md`
- Test execution reports in `tests/results/YYYYMMDD-HHMMSS/REPORT.md`
- Pass/fail analysis
- Coverage assessment

**Parallel With:**
- rust-teradata-architect (during implementation phase - while architect codes, validator designs tests)

---

### tq-project-manager

**Launch When:**
- Validating sprint completion (Phase 5: Closure)
- Assessing technical debt at any time
- Verifying quality standards
- Providing go/no-go decisions

**Typical Prompts:**
- "Validate that sprint-N is truly complete and ready for closure"
- "Assess technical debt in the current codebase"
- "Verify that all features meet quality standards"

**Expected Outputs:**
- Sprint completion validation report
- Technical debt assessment
- Go/no-go recommendation
- Recommendations for next sprint

**Parallel With:**
- Generally launched alone during sprint closure
- Can be launched with other agents if doing mid-sprint quality checks

---

## Sprint Workflow Coordination

### Phase 1: Sprint Planning (Main Agent Leads)

**Main Agent Actions:**
1. Review `specifications.md` and previous sprint review
2. Create `sprint-N-planning.md` using template
3. Define scope (P0/P1/P2), objectives, acceptance criteria
4. Get user approval

**No Sub-Agents Launched**

---

### Phase 2: Parallel Design (Main Agent Coordinates)

**Main Agent Actions:**

Launch BOTH agents in a single message:

```
Task 1: cli-ux-designer
Prompt: "Review sprint-N-planning.md and create/update detailed specifications
for [features]. Update docs/builder/specifications.md dashboard with 🚧 status."

Task 2: rust-teradata-architect
Prompt: "Review sprint-N-planning.md and assess technical feasibility for [features].
Identify architectural considerations and tech debt opportunities."
```

**After Both Complete:**
- Review both outputs
- Resolve conflicts between UX vision and technical constraints
- Validate specifications are implementable
- Update sprint plan if scope changes

---

### Phase 3: Parallel Implementation (Main Agent Coordinates)

**Main Agent Actions:**

Launch BOTH agents in a single message:

```
Task 1: rust-teradata-architect
Prompt: "Implement features from sprint-N-planning.md according to detailed
specifications. Follow rust-architecture.md patterns. Write unit tests."

Task 2: quality-validator
Prompt: "Design comprehensive integration test cases for sprint-N features.
Create TC###.md files in tests/cases/. Cover happy path, edge cases, errors."
```

**After Both Complete:**
- Review code quality and architectural decisions
- Review test case coverage
- Validate alignment between implementation and specs
- Check for technical debt (should be zero)

---

### Phase 4: Test Execution (Main Agent Coordinates)

**Main Agent Actions:**

Launch quality-validator:

```
Task: quality-validator
Prompt: "Execute all test suites (unit + integration) for sprint-N.
Generate test results in tests/results/YYYYMMDD-HHMMSS/."
```

**After Completion:**
- Review test report
- If 100% pass rate: Proceed to Phase 5
- If failures: Launch rust-teradata-architect to fix, return to Phase 4

---

### Phase 5: Sprint Closure (Main Agent Coordinates)

**Main Agent Actions:**

1. Launch tq-project-manager:

```
Task: tq-project-manager
Prompt: "Validate sprint-N is truly complete. Verify: features work as specified,
documentation updated, zero technical debt, all acceptance criteria met."
```

2. After validation:
   - If NOT APPROVED: Address issues, iterate
   - If APPROVED: Create sprint review

3. Main Agent Creates:
   - `sprint-N-review.md` (retrospective, metrics, lessons)
   - Update `specifications.md` (change 🚧 to ✅)
   - Update `docs/builder/user/roadmap.md`

4. Commit changes when user approves

---

## Parallelism Best Practices

### When to Launch in Parallel

**Do launch in parallel:**
- Design + Feasibility assessment (Phase 2)
- Implementation + Test case design (Phase 3)
- Any independent tasks that don't depend on each other

**Don't launch in parallel:**
- Tasks where one depends on the output of another
- Example: Don't launch implementation before design is complete
- Example: Don't launch test execution before implementation is done

### How to Launch in Parallel

**Single message with multiple Task tool calls:**

```
I'm now launching cli-ux-designer and rust-teradata-architect in parallel to
maximize efficiency.

[Task tool call 1: cli-ux-designer with prompt]
[Task tool call 2: rust-teradata-architect with prompt]

I'll wait for both to complete and then synthesize their outputs.
```

---

## Communication Patterns

### When Launching Agents

Be clear about:
1. Which phase of the sprint you're in
2. Which agents you're launching
3. Why you're launching them
4. What you expect from each
5. Whether they're running in parallel

Example:
```
Phase 2: Parallel Design

I'm launching two agents in parallel:
1. cli-ux-designer: Create detailed specifications
2. rust-teradata-architect: Assess technical feasibility

Once both complete, I'll synthesize their outputs and validate
that the specifications are implementable.
```

### After Agents Complete

Synthesize and report:
1. What each agent delivered
2. Your assessment of their outputs
3. Any conflicts or issues discovered
4. Your decision on how to proceed

Example:
```
Both agents have completed:

cli-ux-designer delivered:
- Updated specifications.md with 🚧 status
- Detailed spec for tab completion in detailed-specifications/repl-mode.md
- UX design follows CLI best practices

rust-teradata-architect delivered:
- Technical feasibility confirmed
- Identified use of reedline completer trait
- No architectural concerns
- Estimated complexity: Medium

Synthesis: Both outputs align. Specifications are implementable.
Proceeding to Phase 3: Implementation.
```

---

## Quality Standards

The main agent enforces these standards:

**Before Phase 2 → Phase 3:**
- [ ] Specifications are clear, complete, and unambiguous
- [ ] Technical feasibility confirmed by architect
- [ ] UX design meets CLI best practices
- [ ] No conflicts between design and technical constraints

**Before Phase 3 → Phase 4:**
- [ ] Implementation complete and follows rust-architecture.md
- [ ] Unit tests written and passing
- [ ] Test cases designed with comprehensive coverage
- [ ] No technical debt introduced

**Before Phase 4 → Phase 5:**
- [ ] 100% test pass rate achieved
- [ ] All acceptance criteria validated
- [ ] No test failures or blockers

**Before Sprint Closure:**
- [ ] tq-project-manager validation: APPROVED
- [ ] All documentation synchronized
- [ ] Zero technical debt
- [ ] All P0 features complete
- [ ] All P1 features complete or explicitly deferred

---

## Troubleshooting

### Agent Outputs Conflict

**Example:** Designer wants feature A, architect says it's technically problematic.

**Resolution:**
1. Review both perspectives
2. Make architectural decision based on project principles
3. Document decision in sprint-N-planning.md
4. Update one or both agents' outputs as needed

### Tests Fail in Phase 4

**Resolution:**
1. Review test failures with quality-validator report
2. Launch rust-teradata-architect to fix issues
3. Return to Phase 4 (re-test)
4. Iterate until 100% pass rate

### Scope Too Large During Planning

**Resolution:**
1. Reduce scope: move features to P2 or next sprint
2. Update sprint-N-planning.md
3. Get user approval for scope change
4. Document decision

### Technical Debt Discovered

**Resolution:**
1. If in scope and can be fixed quickly: address immediately
2. If out of scope or complex: document as action item for next sprint
3. Never defer critical technical debt
4. tq-project-manager will flag this in validation

---

## Common Mistakes to Avoid

### ❌ Don't Do This

1. **Sequential when parallel is possible**
   - Launching designer, waiting, then launching architect
   - Should launch both in parallel during Phase 2

2. **Skipping validation**
   - Moving to next phase without synthesizing outputs
   - Not checking for conflicts or issues

3. **Launching wrong agent for task**
   - Using cli-ux-designer for implementation
   - Using rust-teradata-architect for test execution

4. **Not documenting decisions**
   - Making decisions without updating sprint-N-planning.md
   - Not creating sprint review at closure

5. **Accepting incomplete work**
   - Moving forward with <100% test pass rate
   - Closing sprint without tq-project-manager validation

### ✅ Do This Instead

1. **Maximize parallelism**
   - Launch independent agents in single message
   - Design + Feasibility in parallel
   - Implementation + Test Design in parallel

2. **Always synthesize**
   - Review all agent outputs
   - Make clear decisions
   - Document conflicts and resolutions

3. **Use right agent for task**
   - Refer to "When to Launch Each Agent" section
   - Follow agent specializations

4. **Document everything**
   - Update sprint planning docs
   - Create comprehensive sprint reviews
   - Track action items across sprints

5. **Maintain quality standards**
   - Require 100% test pass rate
   - Always validate with tq-project-manager
   - Never compromise on technical debt

---

## Quick Reference

### Agent Launch Checklist

Before launching any agent:
- [ ] Clear on which phase I'm in
- [ ] Know what I need from this agent
- [ ] Have I provided enough context (sprint plan, specs)?
- [ ] Can this run in parallel with another agent?
- [ ] What will I do with the output?

After agent completes:
- [ ] Review output for quality and completeness
- [ ] Check for issues or conflicts
- [ ] Synthesize with other outputs if parallel
- [ ] Make clear decision on how to proceed
- [ ] Document any important decisions

---

## References

- Sprint Coordinator Skill: `~/.claude/skills/sprint-coordinator.md`
- Agent Definitions: `.claude/agents/*.md`
- Sprint Template: `docs/builder/sprints/sprint-template-planning.md`
- CLAUDE.md: Project development methodology
- Specifications: `docs/builder/specifications.md`

---

**Remember:** You are the conductor. The agents are your orchestra. Your job is to coordinate, synthesize, decide, and document. Keep the workflow moving efficiently while maintaining the highest quality standards.
