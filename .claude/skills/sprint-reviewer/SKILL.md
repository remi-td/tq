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

### Step 2: Launch Parallel Reviews

Launch 3 agents in ONE message with Task tool:
- `rust-teradata-architect`: Technical review
- `quality-validator`: Quality review
- `cli-ux-designer`: UX review

See [Agent Prompts](references/agent-prompts.md) for prompt templates.

### Step 3: Collect Metrics

- Token usage per agent
- Estimated cost
- Features delivered
- Test counts

### Step 4: Create Review Document

Create ONE file: `docs/builder/sprints/sprint-N-review.md`

Use template `references/template.md` for document structure.

### Step 5: Update Roadmap

- Mark sprint as COMPLETED
- Add retrospective summary
- Update "Current Sprint" section

## Key Principles

| Principle | Rule |
|-----------|------|
| Single File | ONE review per sprint |
| Cost Focus | Always include token/cost analysis |
| Specific Actions | File/line references in recommendations |
| Comparison | Compare to previous sprint |
| Under 600 lines | Keep reviews scannable |

## Detailed References

- **[Template](references/template.md)**: Review document structure
- **[Agent Prompts](references/agent-prompts.md)**: Prompts for each reviewer
