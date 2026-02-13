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

3. **Review GitHub Issues**:
   - Use `/github-issues` skill to fetch sprint-ready issues
   - Focus on issues labeled `sprint-ready`
   - Review by priority: `priority-high` first, then `priority-medium`
   - Check mix of bugs vs enhancements vs documentation

4. **Scan `docs/specifications/*.md`**:
   - Understand full requirements for selected features
   - Check for any specification updates needed
   - Verify issues align with specifications

5. **Update backlog if needed**:
   - Add newly accepted GitHub issues to `docs/roadmap/backlog.md`
   - Include GitHub issue numbers (e.g., "CSV Export (#42)")
   - Reprioritize based on recent learnings
   - Mark urgent bugs as P0

### Step 2: Define Scope

Ensure objectives are:
- **Specific**: Clear, measurable outcomes.
- **Achievable**: Can be completed in one sprint.
- **Relevant**: Aligned with project goals.
- **Session-bounded**: Can complete ALL phases (0-5) in a single session.

#### Session Budget Rule (CRITICAL)

**A smaller single-session sprint is MORE efficient than a larger multi-session sprint.**

Cost analysis from Sprints 30-36:
- Single-session sprints: $10-20 (Sprints 32, 34, 35)
- Multi-session sprints: $36-62 (Sprints 30, 36)

**Before finalizing scope, verify:**
- Can design phase complete quickly? (Usually <20 min)
- Can implementation complete? (Estimate based on complexity)
- Can testing complete? (Usually 20-40 min)
- Buffer for Phase 4-5? (Usually 15-20 min)

**If scope exceeds session budget:**
- Move P2 items to backlog
- Reduce P1 items to essentials
- If still too large, defer P1 items
- NEVER split a sprint across sessions

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
- [Feature/Fix 1] (#IssueNumber if applicable)
- [Feature/Fix 2] (#IssueNumber if applicable)

### Out of Scope
- [Explicitly excluded items]

## GitHub Issues
### Selected for Sprint
- #XX: [Issue title] (priority-high, bug)
- #YY: [Issue title] (priority-medium, enhancement)

### Deferred
- #ZZ: [Issue title] - [Reason for deferral]

## Dependencies
- [External dependencies or blockers]
```

### Step 4: Update Selected GitHub Issues

After planning document is complete, use `/github-issues` skill to comment on selected issues:

For each issue included in the sprint:
```bash
gh issue comment <number> --body "Included in Sprint N. See planning document: docs/sprints/sprint-N-planning.md"
```

This creates traceability between GitHub issues and sprint execution.

## Output
- `docs/sprints/sprint-N-planning.md` updated with objectives, scope, and GitHub issue references
- Selected GitHub issues commented with sprint inclusion notice
- **Immediately proceed to Phase 2 (Design)** - No approval needed. Execute autonomously.
