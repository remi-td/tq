# Phase 0: Reality Check

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Reflect on long-term performance before planning new work.

## Purpose
This phase is the "stop and think" moment. Before planning new features, we must ensure we're not repeating past mistakes or ignoring systemic problems.

## Process

### Step 1: Read Recent History
Read the last **3 sprint reviews** (or all available if fewer):
- `docs/sprints/sprint-N-review.md`
- `docs/sprints/sprint-(N-1)-review.md`
- `docs/sprints/sprint-(N-2)-review.md`

### Step 2: Pattern Detection
Look for these warning signs:

| Pattern | Symptom | Action |
|---------|---------|--------|
| **Stuck Issue** | Same bug/feature appears across 2+ sprints | Escalate to Maintenance Sprint |
| **Accumulating Debt** | Repeated mentions of "TODO", "tech debt", "workaround" | Escalate to Maintenance Sprint |
| **Framework Issues** | Agents failing, prompts ineffective, workflow bottlenecks | Escalate to Maintenance Sprint |
| **Healthy Velocity** | Features shipping, tests passing, no repeating issues | Proceed to Feature Sprint |

If you notice any of these issues, you should declare that your team is facing a crisis that needs to be addressed before resuming with features delivery. 
If thre is no severe issue and your delivery is progressing and improving, you may proceed with delivering new features.

### Step 3: Decide Sprint Type
Based on pattern analysis, decide if we what type of sprint we should proceed to:

- **Feature Sprint** with new feature objectives.
- **Maintenance Sprint**: to address a crisis with objectives focused on:
  - Fixing root causes of repeated issues
  - Paying down technical debt
  - Improving tooling, documentation, or workflow
  - Refactoring problematic code

### Step 4: Initiate sprint planning and document decision
Create the sprint planning document ``docs/sprints/sprint-N-planning.md`` with the reality check findings:

```markdown
# Sprint N Planning
**Date:** YYYY-MM-DD
**Type:** [Feature Sprint | Maintenance Sprint]

## Reality Check Summary
- Reviewed sprints: N-1, N-2, N-3
- Patterns detected: [None | List of patterns]
- Decision: [Feature Sprint | Maintenance Sprint]
- Rationale: [Why this decision was made]

## Objectives for this sprint
...
```

### Step 5: Quarterly Roadmap Review (Optional)

**Check if quarterly review is due** (every ~12 sprints or 3 months):

If it's time for a quarterly review:
1. **Read `docs/roadmap/roadmap.md`**
2. **Assess strategic progress**:
   - Are we on track for v2.0, v3.0 milestones?
   - Have priorities shifted based on user feedback?
   - Should success metrics be updated?
3. **Update roadmap.md if needed**:
   - Adjust release milestones
   - Update strategic priorities
   - Document major direction changes
4. **Document the review** in sprint planning

**Note**: This is infrequent. Most sprints skip this step.

## Output
Proceed to Phase 1 with a clear sprint type decision.
- If feature sprint, follow the steps in `process/phase1-feature-planning.md`
- If Maintenance sprint, follow the steps in `phase1-maintainance-planning.md`
