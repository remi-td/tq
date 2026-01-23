# Phase 1-M: Crisis Deliberation (Maintenance Sprint)

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Facilitate multi-agent deliberation to reach consensus on how to address the crisis.

## Prerequisites
- Phase 0 (Reality Check) completed.
- Sprint type decided: Maintainance

## Overview

You have elected to perform a maintainance sprint because you are facing a major issue with the way the team works or the technical debt (stuck issues, accumulating debt, framework problems). 

We cannot jump straight to planning, instead: 
1. We facilitate a structured deliberation where all domain experts contribute their perspective.
2. We investigate the issue in details to identify the root case and decide what needs to be done to resolve it.

```
Round 1: Problem Statement → All Agents (Parallel) → Perspectives
Synthesis 1: Coordinator merges perspectives
Round 2: Synthesis → All Agents (Parallel) → Reactions
Final Synthesis: Coordinator decides → Solutioning
Solution Proposed -> Proceed to Planning
```

## Process

### CRITICAL: Autonomous Execution

You are the ultimate authority, you decide on what should be the scope of this sprint and what aligns to your overall objectives.
There is no supervisor available to validate your planning decisions or actions.

**DO NOT:**
- Ask user "Should I proceed?"
- Wait for approval
- Request review of planning document

**DO:**
- Complete this phase and capture your decisions in the planning document
- Inform user "Phase 1 complete. Proceeding to Phase 2."
- Move directly to Phase 2

This is a versioned, safe environment. Execute autonomously.

### Step 1: Prepare Problem Statement

Before launching agents, document the situation clearly and stress the sevierity in the sprint planning document `docs/sprints/sprint-N-planning.md`:

```markdown
## Crisis Summary

**Patterns Detected:**
- [Pattern 1 from Phase 0]
- [Pattern 2 from Phase 0]

**Evidence:**
- Sprint X: [Issue]
- Sprint Y: [Same issue]
- Sprint Z: [Still not resolved]

**Impact:**
- [How this affects the project]
```

### Step 2: Round 1 - Parallel Perspectives

Launch ALL THREE agents in a **single message** with the same prompt structure:

**Prompt Template for All Agents:**
```
CRISIS DELIBERATION - Round 1

You are participating in a multi-agent deliberation to address a project crisis.

## Crisis Summary
[Insert the crisis summary from Step 1]

## Your Task
From your domain expertise as [Agent Role], provide:

1. **Root Cause Analysis**: What do you believe is the fundamental cause of this problem?
2. **Proposed Solution**: What specific actions would you recommend?
3. **Effort Estimate**: How much work is your solution (Small/Medium/Large)?
4. **Risk of Inaction**: What happens if we don't fix this?

Be specific and actionable. This is Round 1; you will see other agents' perspectives in Round 2.
```

### Step 3: Synthesis 1

After all agents return, create a synthesis document:

**File:** `docs/sprints/sprint-N-crisis-deliberation.md`

```markdown
# Sprint N Crisis Deliberation

## Round 1 Summary

### Problem Statement
[From Step 1]

### Agent Perspectives

#### cli-ux-designer
- **Root Cause:** [Summary]
- **Proposed Solution:** [Summary]
- **Effort:** [S/M/L]

#### rust-teradata-architect
- **Root Cause:** [Summary]
- **Proposed Solution:** [Summary]
- **Effort:** [S/M/L]

#### quality-validator
- **Root Cause:** [Summary]
- **Proposed Solution:** [Summary]
- **Effort:** [S/M/L]

### Synthesis

#### Areas of Agreement
- [Points where 2+ agents agree]

#### Areas of Disagreement
- [Points where agents differ]

#### Open Questions
- [Things that need clarification]

#### Emerging Consensus
- [The direction that seems to be forming]
```

### Step 4: Round 2 - Reactions

Launch ALL THREE agents again with the synthesis:

**Prompt Template for All Agents:**
```
CRISIS DELIBERATION - Round 2

You are continuing the multi-agent deliberation. Here is the synthesis from Round 1:

## Round 1 Synthesis
[Insert the synthesis from Step 3]

## Your Task
React to the emerging consensus:

1. **Agreement**: Do you agree with the emerging direction? Why or why not?
2. **Gaps**: What is missing from the synthesis?
3. **Priority**: What should the sprint focus on FIRST?
4. **Acceptance Criteria**: How will we know the crisis is resolved?

This is the final round. The Coordinator will make a decision based on your input.
```

### Step 5: Final Synthesis & Decision

Update the deliberation document with Round 2 reactions:

```markdown
## Round 2 Reactions

#### cli-ux-designer
- [Summary of reaction]

#### rust-teradata-architect
- [Summary of reaction]

#### quality-validator
- [Summary of reaction]

## Final Decision

**Sprint Focus:** [What the sprint will address]
**Rationale:** [Why this was chosen]
**Acceptance Criteria:**
- [ ] [Criterion 1]
- [ ] [Criterion 2]
```

### Step 6: Detail solution

Based on the final decision taks any of the folowing agents to perform a detailed analysis of the issue and propose a solution.

Based on the nature of the problem, you may launch any of the agents:
- cli-ux-designer: For usability concerns or interface design issues, misunderstood requirements
- rust-teradata-architect: For clear bugs in the software, need to develop additional tools for the framework
- quality-validator: For repeated issues with testing (eg. missed bugs)

If you have identified multiple distinct issues, run multiple agents in parallel with one issue each.

Run the agent(s) wit the following instructions:
"""
**Instructions** Our team is facing a crisis due to [issue name]. You need to perform a detailed analysis for the root cause of this issue and provide a sound and high confidence solution to the problem to ensure that this is permanently fixed.
**Input document** `docs/sprints/sprint-N-crisis-deliberation.md`
**Output document** `docs/sprints/sprint-N-analysis-[issue name].md`
**Output Structure**:
```Markdown
# Detailed analysis for issue [issue name]

## Description
[What is the problem we are trying to solve]

## Root cause analysis
[Clear analysis of the root cause behind the issue]

## Solution proposal
[How do we solve the issue, in details? With clear reference to code or documentation elements]

## Acceptance Criteria
[how do we validate that the solution works as expected?]
```
"""

Validate that the `docs/sprints/sprint-N-analysis-[issue name].md` are created with clear directions for the resolution.

### Step 7: Proceed to Planning

Now create `sprint-N-planning.md` based on `.claude/skills/sprint-coordinator/references/sprint-planning-template.md`, outlining the goals, objectives and acceptance criterias from the final deliberation.
The Features must reflect the list of items to implement in order to solve the identified issues, it may refer the technical analysis as and when appropriate.

Proceed to Phase 2 (Design) with this plan.

## Convergence Criteria

Deliberation is considered "converged" when:
- **Root Cause Agreement**: At least 2 agents identify the same root cause.
- **Priority Emerges**: A clear first action is identified.
- **No Blocking Disagreement**: No agent raises a fundamental objection in Round 2.

If after Round 2 there is still fundamental disagreement, the Coordinator makes an executive decision and documents the reasoning.

## Output
- `docs/sprints/sprint-N-crisis-deliberation.md`
- `docs/sprints/sprint-N-planning.md`
- Proceed to Phase 2 (Design) with Maintenance focus.
