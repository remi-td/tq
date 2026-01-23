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

## Workflow

### Step 1: Find Current Session ID

```bash
# Get current session (you should already know this from context)
SESSION_ID="<current-session-id>"

# Or find most recent session
SESSION_ID=$(ls -t ~/.claude/projects/-Users-remi-turpaud-Code-genAI-tq/ | grep -E '^[0-9a-f-]+$' | head -1)

echo "Using session: $SESSION_ID"
```

### Step 2: Run Metrics Extraction

```bash
# Extract metrics
./skills/collect-metrics/scripts/extract-sprint-metrics.sh "$SESSION_ID" <output-file.md>
```

This creates a markdown file at the location specified.

## Important Notes

- ✅ This skill only collects data
- ❌ This skill does NOT analyze or recommend changes
- ⚡ Fast execution (~2-3 minutes)
- 📊 Provides factual baseline for optimization