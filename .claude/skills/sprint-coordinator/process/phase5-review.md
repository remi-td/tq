# Phase 5: Retrospective

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Sprint retrospective using the sprint-reviewer skill.

## Prerequisites
- Phase 4 completed (code committed and pushed)
- Sprint planning document exists
- All features implemented and tested

## Process

**CRITICAL: You MUST invoke the sprint-reviewer skill. Do NOT manually create a review document.**

### Step 1: Invoke Sprint Reviewer Skill

**MANDATORY:** Use the Skill tool to invoke sprint-reviewer:

```
Use Skill tool with skill: "sprint-reviewer"
```

**DO NOT:**
- Manually create sprint-N-review.md
- Skip the parallel agent reviews
- Create your own review format
- Skip token/cost metrics

### Step 2: Sprint Reviewer Will Handle

The sprint-reviewer skill will:
1. Launch 3 agents in parallel (rust-teradata-architect, quality-validator, cli-ux-designer)
2. Collect their domain-specific reviews
3. Gather token/cost metrics
4. Use the proper template from `sprint-reviewer/references/template.md`
5. Create ONE consolidated `sprint-N-review.md` document
6. Update roadmap

### Step 3: Verify Output

After sprint-reviewer completes, verify:
- [ ] `sprint-N-review.md` created using proper template
- [ ] Contains sections from all 3 agent reviews
- [ ] Includes token/cost metrics
- [ ] Includes "Estimated Cost" and "Cost per Feature"
- [ ] Roadmap updated

## Output
- `docs/builder/sprints/sprint-N-review.md` created by sprint-reviewer skill
- Roadmap updated with sprint completion
- Ready for Phase 6 (Framework Optimization) if needed

## Why This Matters

**The sprint-reviewer skill ensures:**
- Parallel agent reviews capture domain expertise
- Token/cost tracking for framework optimization
- Consistent review format across all sprints
- Comprehensive retrospective from multiple perspectives

**Skipping the skill means:**
- Missing token/cost data (needed for optimization)
- Missing agent-specific insights
- Inconsistent review format
- Incomplete retrospective