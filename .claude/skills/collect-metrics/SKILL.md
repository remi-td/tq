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

The project uses a SessionStart hook to automatically capture the current session ID. This is configured in `.claude/settings.json` and requires no manual intervention.

## Workflow

### Option 1: Use Current Session (Recommended)

```bash
# Extract metrics for current session (captured by hook)
./.claude/skills/collect-metrics/scripts/extract-sprint-metrics.sh <sprint-number>

# Example: Extract metrics for sprint 22
./.claude/skills/collect-metrics/scripts/extract-sprint-metrics.sh 22
```

The session ID is automatically read from `.claude/current-session-id.txt` (populated by the SessionStart hook).

### Option 2: Use Specific Session

```bash
# Extract metrics for a specific past session
./.claude/skills/collect-metrics/scripts/extract-sprint-metrics.sh <session-id> <sprint-number>

# Example: Extract metrics for sprint 18 from a specific session
./.claude/skills/collect-metrics/scripts/extract-sprint-metrics.sh f599ef4e-6741-40b9-8b70-54c6e6d7272e 18
```

### Finding Past Session IDs

```bash
# List recent sessions for this project
ls -t ~/.claude/projects/$(pwd | sed 's|/|-|g; s|\.|-|g')/*.jsonl | head -5
```

## Output

Creates `docs/sprints/sprint-<N>-metrics.md` with:
- Token usage by agent (input, output, cache creation, cache reads)
- Overall cache hit rates
- Total token counts
- Estimated costs

## Important Notes

- ✅ This skill only collects data
- ❌ This skill does NOT analyze or recommend changes
- ⚡ Fast execution (~2-3 minutes)
- 📊 Provides factual baseline for optimization