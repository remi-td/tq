# Roadmap Documentation

This directory contains implementation tracking and planning documents for the `tq` project.

---

## Documents

### [status.md](status.md) - Implementation Status Dashboard
**Purpose:** Track current implementation status of all features

**Contents:**
- Feature-by-feature status (✅ implemented, 🚧 in progress, 📋 planned)
- Links to pure specifications
- Version history (when each feature was added)
- Test statistics and metrics

**Updated:** After each sprint completion (Phase 4)

**Use Cases:**
- Check if a feature is implemented
- Find which version introduced a feature
- Link implementation status to specifications

---

### [backlog.md](backlog.md) - Feature Backlog
**Purpose:** Prioritized list of features to implement

**Contents:**
- Features organized by priority (P0, P1, P2, P3)
- Brief description and specification links
- Dependencies between features
- Future considerations

**Updated:** During sprint planning (Phase 0)

**Use Cases:**
- Sprint planning - select next features to implement
- Understand feature priorities
- Track dependencies

---

### [roadmap.md](roadmap.md) - Product Roadmap
**Purpose:** High-level strategic direction

**Contents:**
- Release history and milestones
- Future phases (v2.0, v3.0)
- Guiding principles
- Success metrics
- Version strategy

**Updated:** Quarterly or when strategic direction changes

**Use Cases:**
- Understand product vision and direction
- Communicate plans to stakeholders
- Strategic planning

---

## Relationship to Other Documentation

```
docs/
├── specifications/        # WHAT - Pure feature requirements (timeless)
│   ├── vision.md
│   ├── cli.md
│   ├── repl.md
│   └── ...
│
├── roadmap/              # WHEN - Planning and status tracking
│   ├── status.md         ← Current implementation status
│   ├── backlog.md        ← What's next to implement
│   └── roadmap.md        ← Strategic direction
│
└── sprints/              # ARCHIVE - Sprint history
    ├── sprint-N-planning.md
    └── sprint-N-review.md
```

---

## Workflow Integration

### During Sprint Planning (Phase 0)
1. Review `backlog.md` for prioritized features
2. Check `status.md` to see what's already implemented
3. Read specifications from `../specifications/` for requirements
4. Create `../sprints/sprint-N-planning.md` with selected features

### During Sprint Execution (Phase 1-3)
- Reference specifications for requirements (not roadmap docs)
- Roadmap docs are NOT updated during sprint

### After Sprint Completion (Phase 4)
1. Update `status.md` with newly implemented features
2. Mark features as ✅ and add version number
3. Update `backlog.md` to remove completed features
4. Create sprint review in `../sprints/`

### Quarterly Reviews
- Review `roadmap.md` for strategic alignment
- Assess progress toward v2.0, v3.0 goals
- Update success metrics
- Adjust priorities based on retrospectives

---

## Key Principles

### Separation of Concerns
- **Specifications** define WHAT features should do (timeless)
- **Roadmap** tracks WHEN features were/will be implemented
- **Sprint docs** provide historical context and retrospectives

### Single Source of Truth
- **Feature behavior**: See specifications
- **Implementation status**: See status.md
- **What's next**: See backlog.md
- **Strategic direction**: See roadmap.md

### Minimal Redundancy
- Feature details live in specifications only
- Roadmap documents link to specifications, don't duplicate them
- Status badges and sprint info live here, not in specifications

---

## Maintenance

**Who Updates:**
- `status.md` - Updated by sprint-coordinator after Phase 4
- `backlog.md` - Updated by sprint-coordinator during Phase 0
- `roadmap.md` - Updated by project lead quarterly

**When to Update:**
- `status.md` - After every sprint
- `backlog.md` - Before every sprint (Phase 0)
- `roadmap.md` - Quarterly or when direction changes

---

## Related Documentation

- **[Specifications](../specifications/)** - Pure feature requirements
- **[Sprint History](../sprints/)** - Historical sprint documents
- **[CLAUDE.md](../../CLAUDE.md)** - Project instructions for Claude Code
