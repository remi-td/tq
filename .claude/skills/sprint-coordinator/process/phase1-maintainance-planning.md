# Phase 1-M: Maintenance Sprint Planning (Lean Single-Pass)

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Investigate technical debt, process friction, or bug backlog and create a lean maintenance plan in one pass.

## Process

### Step 1: Single-Pass Investigation
Instead of 2-round multi-agent deliberation, launch a single targeted investigation:
- Launch `rust-teradata-architect` (for code/architecture issues), `quality-validator` (for test issues), or `cli-ux-designer` (for UX issues) with the issue description.
- Ask the agent to return a concise root-cause analysis and proposed fix plan in one step.

### Step 2: Create Lean Planning Document
Create `docs/sprints/sprint-N-planning.md` (Max 40-50 lines total).

```markdown
# Sprint N Planning (Maintenance)

## Maintenance Objectives
1. Fix root cause of [Issue/Debt]
2. Implement regression test / process safeguard

## Scope
- [Code change 1]
- [Process/doc change 2]

## Acceptance Criteria
- [ ] 100% test pass rate
- [ ] Issue verified fixed
```

### Step 3: Proceed to Phase 2 (Design) / Phase 3 (Build)
Execute autonomously without stopping for approval.
