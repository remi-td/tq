# Phase 5: Retrospective

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Sprint retrospective using the sprint-reviewer skill.
**Process** Coordinate comprehensive sprint retrospectives, producing a single consolidated review with metrics and optimization recommendations.

## Prerequisites
- Phase 4 completed (code committed and pushed)
- Sprint planning document exists
- All features implemented and tested

## Process

## Prerequisites

Before starting:
1. All features implemented
2. All tests passing
3. Version updated in Cargo.toml
4. Git commits complete

### Step 1: Prepare Context

1. Read roadmap for sprint objectives
2. Check git history: `git log --oneline --since="[date]"`
3. Run tests: `cargo test --all-targets`
4. Read previous sprint review (if exists)

### Step 2: Collect Metrics

**CRITICAL: Use the `/collect-metrics` skill for accurate token data.**

1. **Invoke collect-metrics skill:**
   ```
   Use Skill tool with skill: "collect-metrics" to output the metrics in a file named `docs/builder/sprints/sprint-N-metrics.md`
   ```

2. **Read generated metrics:**
   - File: `docs/builder/sprints/sprint-N-metrics.md`
   - Contains actual token usage from sub-agent transcripts
   - Includes cache hit rates and cost estimates

3. **If metrics collection fails:**
   - Note explicitly: "Token metrics not collected - transcript data unavailable"
   - DO NOT make up estimates or guesses
   - Provide honest statement about missing data

4. **Collect other metrics:**
   - Features delivered: From `docs/builder/sprints/sprint-N-planning.md`
   - Test counts: From `tests/results/sprint-N/REPORT.md`
   - Feature status: Cross-check planning vs. test report

5. **Update the sprint review document**
    - Use [sprint review template](./references/sprint-planning-template.md) to initialise the document: `docs/builder/sprints/sprint-N-review.md`
    - Update the document with the metrics collected in the previous step

### Step 3: Launch Parallel Reviews

Launch 3 agents in ONE message with Task tool:
- `rust-teradata-architect`: Technical review
- `quality-validator`: Quality review
- `cli-ux-designer`: UX review

See [Agent Prompts](references/agent-prompts.md) for prompt templates.

### Step 4: Create Review Document

Create ONE file: `docs/builder/sprints/sprint-N-review.md`

Use template `references/template.md` for document structure.

**Cost Metrics Section:**
- If `sprint-N-metrics.md` exists: Use actual data from collect-metrics
- If metrics unavailable: State clearly "Token metrics not collected for this sprint"
- NEVER make up estimates or guess token counts
- Be honest about data availability

### Step 5: Update Roadmap

- Mark sprint as COMPLETED
- Add retrospective summary
- Update "Current Sprint" section

## Key Principles

| Principle | Rule |
|-----------|------|
| Single File | ONE review per sprint |
| Real Metrics | Use `/collect-metrics` for token data; NEVER estimate |
| Honest Data | If metrics unavailable, say so explicitly |
| Specific Actions | File/line references in recommendations |
| Comparison | Compare to previous sprint |
| Under 600 lines | Keep reviews scannable |

## Detailed References

- **[Template](references/template.md)**: Review document structure
- **[Agent Prompts](references/agent-prompts.md)**: Prompts for each reviewer


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