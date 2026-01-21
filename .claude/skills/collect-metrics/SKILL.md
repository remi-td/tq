---
name: collect-metrics
description: Collects token usage metrics from sprint subagent transcripts. Use during Phase 5 (Sprint Closure) to add factual metrics to sprint review. Simple data extraction only, no analysis.
allowed-tools: Bash, Read, Write
model: claude-sonnet-4-5-20250929
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
# Make script executable
chmod +x .claude/scripts/extract-sprint-metrics.sh

# Extract metrics
./.claude/scripts/extract-sprint-metrics.sh "$SESSION_ID" <sprint-number>
```

This creates: `docs/builder/sprints/sprint-N-metrics.md`

### Step 3: Add Metrics to Sprint Review

Read the generated metrics file and create a fully detailed report in: `docs/builder/sprints/sprint-N-metrics.md`

```markdown
## Token Usage Metrics

**Data Source:** Session `<session-id>`
**Collection Date:** <date>

[Sprint summary]

### By Agent

[summary by agent]

### By Agent and Phase
[summary by agent and phase]

### Details
[Full details from the report file]
```

### Step 4: Report Completion

Inform the user:

```
✅ Metrics collected for Sprint N

**Metrics file created:** docs/builder/sprints/sprint-N-metrics.md

**Key numbers:**
- Total tokens: [X]
- Estimated cost: $[Y]
- Cache hit rate: [Z]%
```

## Important Notes

- ✅ This skill only collects data
- ❌ This skill does NOT analyze or recommend changes
- ⚡ Fast execution (~2-3 minutes)
- 📊 Provides factual baseline for optimization

## Next Step

After collecting metrics, optionally run:
```
/optimize-agents
```

This separate skill will:
- Analyze metrics across multiple sprints
- Apply decision tree to identify patterns
- Generate concrete optimization actions
- Propose specific file edits

See `optimize-agents/SKILL.md` for details.
