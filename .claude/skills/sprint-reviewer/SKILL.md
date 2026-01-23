---
name: sprint-reviewer
description: Conducts sprint retrospectives with parallel agent reviews. Use at the end of each sprint when all features are complete and tested.
---

# Sprint Reviewer

Coordinate comprehensive sprint retrospectives, producing a single consolidated review with metrics and optimization recommendations.

## Prerequisites

Before starting:
1. All features implemented
2. All tests passing
3. Version updated in Cargo.toml
4. Git commits complete

## Review Process

### Step 1: Prepare Context

1. Read roadmap for sprint objectives
2. Check git history: `git log --oneline --since="[date]"`
3. Run tests: `cargo test --all-targets`
4. Read previous sprint review (if exists)

### Step 2: Collect Metrics

**CRITICAL: Use the `/collect-metrics` skill for accurate token data.**

1. **Invoke collect-metrics skill:**
   ```
   Use Skill tool with skill: "collect-metrics" to output the metrics in a file named `docs/sprints/sprint-N-metrics.md`
   ```

2. **Read generated metrics:**
   - File: `docs/sprints/sprint-N-metrics.md`
   - Contains actual token usage from sub-agent transcripts
   - Includes cache hit rates and cost estimates

3. **If metrics collection fails:**
   - Note explicitly: "Token metrics not collected - transcript data unavailable"
   - DO NOT make up estimates or guesses
   - Provide honest statement about missing data

4. **Collect other metrics:**
   - Features delivered: From `docs/sprints/sprint-N-planning.md`
   - Test counts: From `tests/results/sprint-N/REPORT.md`
   - Feature status: Cross-check planning vs. test report

### Step 3: Launch Parallel Reviews

Launch 3 agents in ONE message with Task tool:
- `rust-teradata-architect`: Technical review
- `quality-validator`: Quality review
- `cli-ux-designer`: UX review

See [Agent Prompts](references/agent-prompts.md) for prompt templates.

### Step 4: Create Review Document

Create ONE file: `docs/sprints/sprint-N-review.md`

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
