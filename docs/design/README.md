# Design Documentation

This directory contains the **technical design** documentation for tq, explaining **HOW** features are implemented.

## Purpose

While `docs/specifications/` defines **WHAT** tq should do (user requirements and behavior), `docs/design/` explains **HOW** it works (architecture, patterns, implementation approaches).

## Structure

Design documents mirror the structure of specifications where applicable:

| Specification | Design Document | Purpose |
|--------------|-----------------|---------|
| `specifications/vision.md` | `design/vision.md` | High-level architecture and design principles |
| `specifications/cli-interface.md` | `design/cli-interface.md` | Command parsing, argument handling implementation |
| `specifications/repl.md` | `design/repl.md` | REPL loop, state management, interactive features |
| `specifications/batch-mode.md` | `design/batch-mode.md` | Batch execution, file processing |
| `specifications/configuration.md` | `design/configuration.md` | Config file parsing, profile management |
| `specifications/output-formats.md` | `design/output-formats.md` | Formatters architecture, rendering pipeline |
| `specifications/error-handling.md` | `design/error-handling.md` | Error types, propagation patterns, recovery |
| `specifications/performance.md` | `design/performance.md` | Optimization techniques, profiling approaches |
| `specifications/security.md` | `design/security.md` | Security implementation details |

Some design documents have no one-to-one specification counterpart because they describe a
cross-cutting mechanism rather than a single user-facing feature:

| Design Document | Purpose |
|-----------------|---------|
| `design/monitoring.md` | Threshold configuration, severity classification and color rendering shared by all monitoring commands |
| `design/space-analysis.md` | `tq space` / `tq dbspace`: DBC sources, SQL, aggregation and output shapes |

## What Design Documents Contain

Each design document should include:

1. **Functional Building Blocks**: Key modules, structs, traits involved
2. **Design Patterns**: Architectural patterns applied (e.g., Builder, Strategy, Command)
3. **Solution Approach**: How the feature is implemented at a high level
4. **Code Linkage**: File paths, module names, key function signatures
5. **Design Decisions**: Why this approach was chosen over alternatives
6. **Integration Points**: How this component interacts with others

## What Design Documents Should NOT Contain

- **Sprint references** (e.g., "Sprint 8 implementation")
- **Status updates** (e.g., "TODO", "Completed", "In Progress")
- **Dates** (design docs are timeless)
- **Bug tracking** (use issue tracker)
- **Requirements** (those belong in specifications)

## Ownership

**Owner**: `rust-teradata-architect` agent

The rust-teradata-architect agent maintains these documents as the single source of truth for technical implementation guidance.

## How to Use Design Documents

### When Implementing Features
1. Read `docs/specifications/[feature].md` to understand **what** to build
2. Read `docs/design/[feature].md` to understand **how** to build it
3. Follow existing patterns and architecture
4. Update design docs if you introduce new patterns

### When Fixing Bugs
1. Check `docs/specifications/` for expected behavior
2. Check `docs/design/` for implementation approach
3. Identify where implementation diverges from design
4. Fix code to match design (or update design if requirements changed)

### When Refactoring
1. Understand current design from `docs/design/`
2. Propose new design approach
3. Get approval for design changes
4. Update design docs before implementing
5. Implement according to updated design

### When Reviewing Code
1. Verify code follows patterns in `docs/design/`
2. Check that design docs are up-to-date
3. Suggest design doc updates if patterns have evolved

## Relationship to Other Documentation

```
docs/
├── specifications/     # WHAT the tool should do (requirements)
│   └── vision.md      # Product vision, goals, principles
├── design/            # HOW the tool works (architecture)
│   └── vision.md      # Technical architecture, component integration
├── roadmap/           # WHEN features are implemented (status)
│   └── status.md      # Current implementation status
└── builder/           # Framework for Claude agents
    └── testing-guidelines.md  # Testing methodology
```

## Getting Started

Start with `vision.md` to understand the overall technical architecture, then dive into specific feature design documents as needed.
