# Token Optimization Decision Tree (Deprecated)

**This file has been replaced and is no longer maintained.**

## New Location

The token optimization analysis framework has been refactored and moved to:

**`.claude/skills/optimize-agents/references/waste-patterns.md`**

## What Changed

### Old Approach (Deprecated)
- Threshold-based analysis ("Grand Total > 150K?")
- Baseline comparisons (Sprint 6-7 average)
- Analyzed aggregate metrics after the fact

### New Approach (Current)
- **Per-transcript analysis** - Analyze every transcript regardless of size
- **Pattern-based** - Match operations to 12 known waste patterns
- **Parallel execution** - Launch `optimization-analyzer` agents for each transcript
- **Structured backlog** - Proposals tracked in `docs/builder/optimization-backlog/`

## New Workflow

The optimization system now follows this workflow (Phase 6 of sprint):

1. **Parallel Analysis**: Launch `optimization-analyzer` for each transcript
2. **Consolidation**: Main agent deduplicates proposals, assigns IDs
3. **Prioritization**: Calculate impact scores, select top 3-5
4. **Implementation**: Main agent implements selected proposals
5. **Validation**: Next sprint verifies expected improvements

## Key Resources

| Resource | Purpose |
|----------|---------|
| `.claude/subagents/optimization-analyzer.md` | Analyzes single transcript for waste patterns |
| `.claude/skills/optimize-agents/SKILL.md` | Analysis framework and workflow |
| `.claude/skills/optimize-agents/references/waste-patterns.md` | **Pattern catalog** (replaces this file) |
| `docs/builder/optimization-backlog/` | Structured backlog with proposals |

## Migration Note

If you were referencing this file:
- For waste patterns and analysis methodology: See `waste-patterns.md`
- For the optimization workflow: See `CLAUDE.md` Phase 6
- For proposal format and backlog: See `docs/builder/optimization-backlog/README.md`

---

**Last Updated:** Sprint 11 (2026-01-18)
**Status:** Deprecated in favor of skill-based pattern catalog
