# Phase 1: Planning

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Define the sprint scope, objectives, and acceptance criteria.

## Prerequisites
- Phase 0 (Reality Check) completed.
- Sprint type decided: Feature

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

### Step 1: Context Review

Read relevant context to inform sprint planning:

1. **Read `docs/roadmap/status.md`**:
   - What features are already ✅ Complete?
   - What features are 🚧 In Progress?
   - What's the current version number?

2. **Read `docs/roadmap/backlog.md`**:
   - What P0 (critical) features are waiting?
   - What P1 (high priority) features should be next?
   - Any dependencies that are now unblocked?

3. **Review `incoming/`**:
   - Any new bug reports to address?
   - Any new feature requests to consider?
   - Any user feedback to incorporate?

4. **Scan `docs/specifications/*.md`**:
   - Understand full requirements for selected features
   - Check for any specification updates needed

5. **Update backlog if needed**:
   - Add new items from `incoming/` to `docs/roadmap/backlog.md`
   - Reprioritize based on recent learnings
   - Mark urgent bugs as P0

### Step 2: Define Scope

Ensure objectives are:
- **Specific**: Clear, measurable outcomes.
- **Achievable**: Can be completed in one sprint.
- **Relevant**: Aligned with project goals.

### Step 3: Update Planning Document

Append to the sprint planning document `docs/sprints/sprint-N-planning.md`:

```markdown

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

## Output
- `docs/sprints/sprint-N-planning.md` updated.
- **Immediately proceed to Phase 2 (Design)** - No approval needed. Execute autonomously.
