---
name: collect-metrics
description: Collects token usage metrics from subagent transcripts. Use to add factual metrics to analyze your agent performance. Simple data extraction only, no analysis.
allowed-tools: Bash, Read, Write
---

# Collect Metrics Skill

## Purpose

Simple, fast extraction of token usage data from subagent transcripts. Adds factual metrics to sprint review document.

**What this does:** Data collection only
**What this does NOT do:** Analysis, interpretation, or recommendations

## Setup (One-time)

The project uses a SessionStart hook to automatically capture session information:
- Current session ID is stored in `.claude/current-session-id.txt`
- Session history with timestamps is appended to `.claude/session-history.txt`

This is configured in `.claude/settings.json` and requires no manual intervention.

## Workflow

### Single-Session Sprints

For sprints completed in a single session:

```bash
# Extract metrics for current session (captured by hook)
./.claude/skills/collect-metrics/scripts/extract-sprint-metrics.sh <sprint-number>

# Example: Extract metrics for sprint 22
./.claude/skills/collect-metrics/scripts/extract-sprint-metrics.sh 22
```

Or for a specific past session:

```bash
# Extract metrics for a specific session
./.claude/skills/collect-metrics/scripts/extract-sprint-metrics.sh <session-id> <sprint-number>

# Example
./.claude/skills/collect-metrics/scripts/extract-sprint-metrics.sh f599ef4e-6741-40b9-8b70-54c6e6d7272e 18
```

### Multi-Session Sprints

Sprint work often spans multiple Claude sessions. To collect complete metrics:

**Step 1: List recent sessions**

```bash
# List sessions from last N days (default: 7)
./.claude/skills/collect-metrics/scripts/list-recent-sessions.sh [days]

# Example: List sessions from last 3 days
./.claude/skills/collect-metrics/scripts/list-recent-sessions.sh 3
```

This shows session IDs, timestamps, and subagent counts to help identify sprint-related sessions.

**Step 2: Combine metrics from multiple sessions**

```bash
# Combine metrics from multiple sessions
./.claude/skills/collect-metrics/scripts/combine-sprint-metrics.sh <sprint-number> <session-id-1> <session-id-2> [...]

# Example: Sprint 22 used two sessions
./.claude/skills/collect-metrics/scripts/combine-sprint-metrics.sh 22 \
  27f6d7b5-e9d3-4034-8903-bee5e292dcf3 \
  93583c02-d3a3-4c61-8b4f-49d9b4aac8ac
```

This produces a combined metrics file with:
- Aggregated token usage across all sessions
- Overall cache hit rate and costs
- Per-session breakdown

## Output

Creates `docs/sprints/sprint-<N>-metrics.md` with:
- Token usage by agent (input, output, cache creation, cache reads)
- Overall cache hit rates
- Total token counts
- Estimated costs

## Important Notes

- This skill only collects data
- This skill does NOT analyze or recommend changes
- Fast execution (~2-3 minutes)
- Provides factual baseline for optimization

## Multi-Sprint Sessions (Sprint 69 Note)

When multiple sprints run in a single Claude session, metrics are **session-cumulative**. The "Grand Total" includes all sprints in that session.

**To compute per-sprint cost:**

1. Find the previous sprint's "Grand Total" tokens from its metrics file
2. Subtract from current sprint's "Grand Total"
3. Apply same formula to compute cost

**Example (Sprint 69):**
```
Session Grand Total: 63,653,746 tokens ($33.18)
Sprint 68 baseline:  23,078,954 tokens ($12.23)
Sprint 69 delta:     40,574,792 tokens (~$21)
```

**Best Practice:** When starting a new sprint in an existing session, note the current cumulative totals so delta computation is straightforward.

**Future Enhancement:** Consider adding `--after <timestamp>` flag to filter agents by creation time for cleaner per-sprint metrics.