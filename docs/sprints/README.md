# Sprint History

This directory contains the historical record of sprints for the tq project. Each sprint produces exactly two documents that capture planning and outcomes.

## Document Types

### Planning Documents (`sprint-N-planning.md`)

**Purpose**: Define scope, objectives, and acceptance criteria before work begins

**Created**: At sprint start

**Contains**:
- Sprint goal and theme
- Objectives (what we're trying to achieve)
- Scope breakdown by priority (P0/P1/P2)
- Feature descriptions with acceptance criteria
- Dependencies and risks
- Success criteria

**Example**: `sprint-15-planning.md`

### Review Documents (`sprint-N-review.md`)

**Purpose**: Retrospective analysis of what was delivered and learned

**Created**: At sprint completion

**Contains**:
- What was delivered
- What was learned (successes and challenges)
- Metrics (token usage, test results, time estimates)
- Technical decisions and trade-offs
- Recommendations for future sprints
- Framework optimization opportunities

**Example**: `sprint-15-review.md`

## Naming Convention

All documents follow the pattern `sprint-N-[type].md`:
- `sprint-15-planning.md` - Planning document for sprint 15
- `sprint-15-review.md` - Review document for sprint 15

## Structure

Sprints are numbered sequentially starting from 1. Each sprint directory contains exactly these two document types - no more, no less. This keeps the structure simple and predictable.

## Usage

### When Planning a Sprint
1. Review recent sprint reviews to understand lessons learned
2. Check `docs/roadmap/status.md` for what's implemented
3. Check `docs/roadmap/backlog.md` for prioritized features
4. Create `sprint-N-planning.md` with clear objectives and scope

### When Completing a Sprint
1. Verify all acceptance criteria met
2. Document what was delivered
3. Capture lessons learned and recommendations
4. Create `sprint-N-review.md` with retrospective analysis
5. Update `docs/roadmap/status.md` with new implementation status

### For Historical Reference
- Review past sprints to understand project evolution
- Study patterns of success and challenges
- Learn from technical decisions documented in reviews
- Understand scope and delivery trends

## Relationship to Other Documentation

```
docs/
├── specifications/   # WHAT features should do (requirements)
├── design/          # HOW features are implemented (architecture)
├── testing/         # Testing approach and methodology
├── sprints/         # WHEN features were delivered (history)
│   ├── sprint-N-planning.md
│   └── sprint-N-review.md
└── roadmap/         # Implementation status and future plans
```

Sprint documents reference specifications and design docs but remain focused on planning and retrospective analysis. They provide the historical narrative of how the project evolved sprint by sprint.

## Archive Policy

All sprint documents are kept indefinitely as historical record. They provide valuable context for understanding architectural decisions, feature evolution, and lessons learned over time.
