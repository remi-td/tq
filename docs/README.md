# Documentation Overview

This directory contains all documentation for the `tq` project, organized for clarity and minimal confusion.

---

## Directory Structure

```
docs/
├── specifications/        # WHAT - Pure feature requirements
├── roadmap/              # WHEN - Implementation status and planning
├── sprints/              # ARCHIVE - Sprint history
└── builder/              # HOW - Architecture and testing guides
```

---

## Quick Navigation

### I want to know... what a feature should do
→ **Read [`specifications/`](specifications/)**
- Pure, timeless feature requirements
- No status badges, no sprint references
- Single source of truth for expected behavior

### I want to know... if a feature is implemented
→ **Read [`roadmap/status.md`](roadmap/status.md)**
- Current implementation status (✅ 🚧 📋)
- Links to specifications for each feature
- Version history

### I want to know... what to implement next
→ **Read [`roadmap/backlog.md`](roadmap/backlog.md)**
- Prioritized feature backlog
- Dependencies and planning info

### I want to know... the strategic direction
→ **Read [`roadmap/roadmap.md`](roadmap/roadmap.md)**
- High-level product roadmap
- Release history and future phases

### I want to know... how to implement something
→ **Read [`builder/rust-architecture.md`](builder/rust-architecture.md)**
- Architecture decisions
- Implementation patterns
- Technical design

### I want to know... how to test features
→ **Read [`builder/testing-guidelines.md`](builder/testing-guidelines.md)**
- Testing methodology
- Test design patterns
- Execution guidelines

### I want to know... what happened in past sprints
→ **Read [`sprints/`](sprints/)**
- Sprint planning documents
- Retrospectives and reviews
- Historical context

---

## Documentation Philosophy

### Separation of Concerns

Each documentation category has a clear purpose:

| Category | Purpose | Updated When | Owner |
|----------|---------|--------------|-------|
| **specifications/** | Define WHAT features do | Requirements change | cli-ux-designer |
| **roadmap/** | Track WHEN features ship | After each sprint | sprint-coordinator |
| **sprints/** | Document sprint context | During sprints | sprint-coordinator |
| **builder/** | Guide HOW to implement | Architecture evolves | rust-teradata-architect |

### Single Source of Truth

- **Feature behavior**: See `specifications/` only
- **Implementation status**: See `roadmap/status.md` only
- **What's next**: See `roadmap/backlog.md` only
- **Architecture**: See `builder/rust-architecture.md` only

### No Redundancy

- Feature details live in `specifications/` - not duplicated elsewhere
- Status tracking lives in `roadmap/` - not in specifications
- Sprint context lives in `sprints/` - not in current docs

---

## For Developers

### Starting a new feature?
1. Read `roadmap/backlog.md` to see what's prioritized
2. Read `specifications/[feature].md` for pure requirements
3. Read `builder/rust-architecture.md` for implementation patterns
4. Create sprint plan in `sprints/sprint-N-planning.md`

### Fixing a bug?
1. Read `specifications/[feature].md` for expected behavior
2. Compare code to specification
3. Fix code to match specification
4. Don't update specifications unless requirements actually changed

### Implementing a feature?
1. Read `specifications/` for requirements (WHAT)
2. Read `builder/rust-architecture.md` for patterns (HOW)
3. Implement according to specifications
4. After completion, update `roadmap/status.md` with ✅

---

## For Agents

### cli-ux-designer
- **Owns**: `specifications/*.md`
- **Updates**: When requirements change
- **Never writes**: Status badges, sprint references, implementation dates

### rust-teradata-architect
- **Owns**: `builder/rust-architecture.md`
- **Reads**: `specifications/` for requirements
- **Writes**: Architecture decisions and patterns

### quality-validator
- **Reads**: `specifications/` for acceptance criteria
- **Owns**: `builder/testing-guidelines.md`
- **Creates**: Test cases and reports in `../tests/`

### sprint-coordinator
- **Owns**: `roadmap/*.md` and `sprints/*.md`
- **Updates**: Status after sprints, backlog during planning
- **Never touches**: `specifications/` (that's designer's job)

---

## Migration Note

**Date:** 2026-01-23

This structure was reorganized to separate pure specifications from implementation tracking.

**Old structure** (archived in `sprints/archive/`):
- Mixed specs with status badges and sprint context
- `specifications.md` combined vision + status + roadmap
- `detailed-specifications/` had sprint summaries in specs

**New structure**:
- Pure specifications without any status tracking
- Status and planning in separate `roadmap/` directory
- Clear separation of concerns

See [`sprints/archive/README.md`](sprints/archive/README.md) for full migration details.

---

## Related Documentation

- **[Specifications README](specifications/README.md)** - Detailed spec navigation
- **[Roadmap README](roadmap/README.md)** - Status and planning docs
- **[CLAUDE.md](../CLAUDE.md)** - Project instructions for Claude Code

---

## Need Help?

- **Can't find a feature spec?** → Check `specifications/README.md`
- **Don't know if it's implemented?** → Check `roadmap/status.md`
- **Confused about structure?** → You're reading the right doc!
- **Need historical context?** → Check `sprints/sprint-N-review.md`
