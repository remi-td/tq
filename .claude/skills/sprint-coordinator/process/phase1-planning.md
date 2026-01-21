# Phase 1: Planning

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Define the sprint scope, objectives, and acceptance criteria.

## Prerequisites
- Phase 0 (Reality Check) completed.
- Sprint type decided (Feature or Maintenance).

## Process

### Step 1: Create Planning Document

Create `docs/builder/sprints/sprint-N-planning.md`:

```markdown
# Sprint N Planning

**Date:** YYYY-MM-DD
**Type:** [Feature Sprint | Maintenance Sprint]

## Reality Check Summary
- Reviewed sprints: [List]
- Patterns detected: [None | List]
- Decision rationale: [Why this sprint type]

## Objectives
1. [Objective 1]
2. [Objective 2]

## Acceptance Criteria
- [ ] [Criterion 1]
- [ ] [Criterion 2]

## Scope
### In Scope
- [Feature/Fix 1]
- [Feature/Fix 2]

### Out of Scope
- [Explicitly excluded items]

## Dependencies
- [External dependencies or blockers]
```

### Step 2: Context Review

Read relevant context:
- `specifications.md` - Current feature status
- `rust-architecture.md` - Technical constraints
- Any open issues in `docs/builder/incoming/`

### Step 3: Validate Scope

Ensure objectives are:
- **Specific**: Clear, measurable outcomes.
- **Achievable**: Can be completed in one sprint.
- **Relevant**: Aligned with project goals.

## Output
- `docs/builder/sprints/sprint-N-planning.md` created.
- **Immediately proceed to Phase 2 (Design)** - No approval needed. Execute autonomously.

## CRITICAL: Autonomous Execution

**DO NOT:**
- Ask user "Should I proceed?"
- Wait for approval
- Request review of planning document

**DO:**
- Create planning document
- Inform user "Phase 1 complete. Proceeding to Phase 2."
- Move directly to Phase 2

This is a versioned, safe environment. Execute autonomously.
