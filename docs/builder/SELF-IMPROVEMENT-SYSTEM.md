# Self-Improvement System for TQ Framework

## Overview

This document describes the complete self-improvement system that enables the TQ project's Claude agent framework to **continuously optimize itself** by analyzing sprint metrics and generating concrete improvements.

**Primary Goal:** Reduce token consumption while maintaining quality through measured, iterative improvements.

---

## System Architecture

### Two-Phase Design

The system is deliberately split into two separate concerns:

#### Phase 1: **Metrics Collection** (Simple & Fast)
- **Tool:** `/collect-metrics` skill
- **When:** Every sprint during Phase 5 (Sprint Closure)
- **Duration:** 2-3 minutes
- **Purpose:** Extract factual token usage data
- **Output:** `sprint-N-metrics.md` with raw numbers

#### Phase 2: **Optimization Analysis** (Deep & Thorough)
- **Tool:** `/optimize-agents` skill
- **When:** Quarterly (every 3-4 sprints) or on-demand
- **Duration:** 30-60 minutes
- **Purpose:** Identify waste patterns and generate improvements
- **Output:** Concrete file edits to optimize agents/docs/tools

### Why This Separation?

**Sprint retrospectives stay fast:** Just collect numbers, no analysis paralysis

**Optimization gets proper time:** Dedicated session for deep thinking, not rushed

**Can batch-analyze:** Look at patterns across multiple sprints for stronger signals

**Clear responsibility:** Metrics = facts, Optimization = decisions

---

## Phase 1: Metrics Collection

### How It Works

During Sprint Closure (Phase 5), after creating the sprint review:

```bash
# In the main conversation
/collect-metrics 8
```

The skill:
1. Finds the current session ID
2. Runs `.claude/scripts/extract-sprint-metrics.sh`
3. Parses subagent transcripts (JSONL files)
4. Extracts token usage: input, output, cache creation, cache reads
5. Calculates costs and cache hit rates
6. Generates `docs/builder/sprints/sprint-8-metrics.md`
7. Adds metrics summary to sprint review

### What Gets Measured

**Per Agent:**
- Input tokens
- Output tokens
- Cache creation tokens
- Cache read tokens
- Total tokens
- Cache hit rate (%)
- Estimated cost ($)

**Sprint Summary:**
- Grand total tokens
- Total cost
- Overall cache efficiency
- Comparison to previous sprints

### Example Output

```markdown
## Sprint 8 - Token Usage Metrics

### Agent: rust-teradata-architect (ID: a7c8742)

| Metric | Value |
|--------|-------|
| Input Tokens | 35,769 |
| Output Tokens | 15 |
| Cache Creation | 129,670 |
| Cache Reads | 142,169 |
| **Total Tokens** | **307,623** |
| Cache Hit Rate | 46.2% |

### Sprint Summary

| Metric | Value |
|--------|-------|
| Total Input Tokens | 89,450 |
| Total Output Tokens | 12,340 |
| Total Cache Creation | 201,450 |
| Total Cache Reads | 189,220 |
| **Grand Total** | **492,460** |
| Overall Cache Hit Rate | 47.3% |

## Estimated Cost (Sonnet 4.5 pricing)

| Category | Cost |
|----------|------|
| Input Tokens | $0.87 |
| Output Tokens | $1.85 |
| Cache Reads | $0.06 |
| **Total** | **$2.78** |
```

### Technical Implementation

**Script:** `.claude/scripts/extract-sprint-metrics.sh`

**How it works:**
1. Locates subagent transcripts: `~/.claude/projects/<project>/<session>/subagents/agent-*.jsonl`
2. Uses `jq` to parse JSONL and extract usage data
3. Sums tokens across all API calls in each agent's transcript
4. Calculates aggregate statistics
5. Outputs formatted markdown

**Key insight:** Each line in the JSONL is a message with optional `.message.usage` field:
```json
{
  "message": {
    "usage": {
      "input_tokens": 35769,
      "output_tokens": 15,
      "cache_creation_input_tokens": 129670,
      "cache_read_input_tokens": 142169
    }
  }
}
```

---

## Phase 2: Optimization Analysis

### How It Works

After collecting metrics from 2+ sprints:

```bash
# Typically run quarterly or after noticing issues
/optimize-agents
```

The skill (runs in Opus, forked context):
1. Reads historical `sprint-*-metrics.md` files (3-6 sprints)
2. Loads decision tree: `docs/builder/token-optimization-decision-tree.md`
3. Identifies patterns across sprints
4. Maps patterns to root causes
5. Generates concrete optimization actions
6. Prioritizes by impact and effort
7. Produces implementation plan with specific file edits

### What Gets Analyzed

#### 1. High Token Usage Patterns

**Questions:**
- Which agent consistently uses most tokens?
- Is usage increasing or decreasing over time?
- Which sprints had anomalies?

**Example Finding:**
```
rust-teradata-architect used 300K+ tokens in Sprint 7 and 8
→ Reads src/db/metadata.rs 4-5 times per sprint
→ Root cause: Missing module architecture documentation
→ Fix: Add "Metadata System Architecture" section to rust-architecture.md
→ Expected impact: 10-15K token reduction
```

#### 2. Quality Failure Patterns

**Questions:**
- Were there bugs found after "tests passed"?
- Were manual tests skipped?
- Was database unavailable during testing?

**Example Finding (Sprint 8):**
```
Features marked complete but broken
→ Manual tests were not executed
→ Root cause: Phase 3.5 (database check) was optional
→ Fix: Make Phase 3.5 MANDATORY in sprint-coordinator skill
→ Expected impact: Prevent 20-40K rework tokens per quality failure
```

#### 3. Low Cache Efficiency Patterns

**Questions:**
- Which agents have <40% cache hit rate?
- Why is caching ineffective?
- Is prompt content volatile?

**Example Finding:**
```
cli-ux-designer has 22% cache hit rate
→ Specification formats change every sprint
→ Root cause: No stable template structure
→ Fix: Create spec templates with stable sections
→ Expected impact: 5-8K token reduction + 60% cache improvement
```

#### 4. Missing Context Patterns

**Questions:**
- What questions do agents repeatedly ask?
- Which documentation is unclear?
- What decisions aren't documented?

**Technique:** Analyze subagent transcripts for question patterns:
```bash
# Search transcripts for clarifying questions
jq -r 'select(.message.content) | .message.content[] |
       select(.type == "text") | .text' transcript.jsonl |
grep -E "What is|How do I|Where should|Should I"
```

**Example Finding:**
```
rust-teradata-architect asks "How do completer and metadata interact?" in Sprint 7 and 8
→ This relationship is implemented but not documented
→ Root cause: Missing interaction diagrams in architecture docs
→ Fix: Add "Module Interaction Patterns" section with diagrams
→ Expected impact: 3-5K token reduction + faster implementation
```

#### 5. Redundant Operations Patterns

**Questions:**
- Which files are read 3+ times?
- Are agents exploring or do they know what to look for?
- Could context be pre-loaded?

**Technique:** Count Read operations per file:
```bash
jq -r 'select(.message.content) | .message.content[] |
       select(.type == "tool_use" and .name == "Read") |
       .input.file_path' transcript.jsonl |
sort | uniq -c | sort -rn
```

**Example Finding:**
```
src/db/metadata.rs read 5 times in Sprint 7, 4 times in Sprint 8
→ Agent explores file structure each sprint
→ Root cause: No upfront overview of module
→ Fix: Add module overview to rust-architecture.md
→ Expected impact: Reduce reads from 4-5 to 1-2, save 3-4K tokens
```

### The Decision Tree

Located at: `docs/builder/token-optimization-decision-tree.md`

Provides systematic framework:

```
1. High Token Usage?
   ├─ Which agent?
   │  ├─ rust-teradata-architect → Section 1.1
   │  ├─ quality-validator → Section 1.2
   │  └─ cli-ux-designer → Section 1.3

2. Quality Failures?
   ├─ Manual tests skipped? → Section 2.1 (CRITICAL)
   ├─ Database unavailable? → Section 2.2
   └─ Edge cases not tested? → Section 2.3

3. Low Cache Hit Rate?
   └─ < 40%? → Section 3.1

4. Workflow Inefficiency?
   ├─ Sequential when parallel possible? → Section 4.1
   ├─ Multiple fix iterations? → Section 4.2
   └─ Back-and-forth communication? → Section 4.3
```

Each section provides:
- **Diagnostic questions** to confirm the pattern
- **Root cause analysis** explaining why it happens
- **Concrete solutions** with specific file changes
- **Expected impact** in token reduction

### Output Format: Concrete Actions

The skill generates an implementation plan with specific file edits:

```markdown
## Optimization #1: Add Metadata Module Overview

### Evidence
- Sprint 7: rust-teradata-architect read metadata.rs 5 times (5K tokens)
- Sprint 8: rust-teradata-architect read metadata.rs 4 times (4K tokens)
- Pattern consistent across 2 sprints

### Root Cause
Agent doesn't understand metadata module structure upfront.
Explores file multiple times during implementation.

### Solution
Add "Metadata Module Overview" to rust-architecture.md

### Implementation

**File:** `docs/builder/rust-architecture.md`
**Action:** Add new section 6.3 after "6.2 REPL Module Organization"

**Content to add:**
```markdown
### 6.3 Metadata System Architecture

The metadata system provides database introspection for tab completion:

**Core Components:**
- `src/db/metadata.rs` - MetadataCache with lazy loading
- `src/commands/repl/sql_context.rs` - SQL statement parsing
- `src/commands/repl/metadata_completer.rs` - Completion logic

[... full content provided ...]
```

**Expected Impact:**
- Reduces metadata.rs reads from 4-5 to 1-2
- Saves 3-4K tokens per sprint
- Faster implementation (less exploration)

**Validation:**
In Sprint 9 metrics, verify metadata.rs reads ≤ 2
```

### Prioritization Matrix

All optimizations are ranked by impact and effort:

| Priority | Criteria | Example |
|----------|----------|---------|
| **P0** | Prevents quality failures | Make database check mandatory |
| **P0** | >10K token reduction | Add module overviews |
| **P1** | 5-10K token reduction | Optimize agent prompts |
| **P1** | Enables parallelism | Update workflow |
| **P2** | Cache efficiency | Stabilize prompts |
| **P2** | <5K token reduction | Minor improvements |
| **P3** | Quality of life | Better error messages |

**Implementation order:** P0 → P1 → P2 → P3

---

## Complete Workflow Example

### Sprint 6-8: Baseline Establishment

```bash
# Sprint 6 Closure
/collect-metrics 6
# Metrics collected: 380K tokens, $3.10

# Sprint 7 Closure
/collect-metrics 7
# Metrics collected: 420K tokens, $3.45

# Sprint 8 Closure (quality failure discovered)
/collect-metrics 8
# Metrics collected: 490K tokens, $4.02
# Note: High tokens due to rework from quality failure
```

### First Optimization Cycle

```bash
# After Sprint 8, run deep analysis
/optimize-agents

# Output:
## Key Findings
1. Quality failure: Manual tests skipped (Sprint 8)
   → Fix: Make Phase 3.5 mandatory
   → Impact: Prevent 20-40K rework tokens

2. rust-teradata-architect inefficiency
   → Fix: Add module overviews to architecture docs
   → Impact: 10-15K token reduction

3. Low cache hit rate (cli-ux-designer: 22%)
   → Fix: Create stable spec templates
   → Impact: 5-8K token reduction

## Total Expected Impact: 35-63K tokens (30-50% reduction)

## Implementation Plan
- P0 Action 1: Update sprint-coordinator Phase 3.5 (15 min)
- P0 Action 2: Add architecture docs sections (2-3 hours)
- P1 Action 3: Create spec templates (1 hour)
```

### Implement P0 Optimizations

```bash
# Main agent or user implements the P0 actions
# 1. Edit .claude/skills/sprint-coordinator/SKILL.md
# 2. Add sections to docs/builder/rust-architecture.md
# 3. Update workflow in CLAUDE.md
```

### Sprint 9: Measure Impact

```bash
# Sprint 9 Closure
/collect-metrics 9
# Metrics collected: 315K tokens, $2.58

# Validation:
# - 175K token reduction (36% improvement!)
# - No quality failures
# - metadata.rs reads: 2 (down from 4-5)
# - Cache hit rate: 58% (up from 22%)

# SUCCESS! Optimizations worked.
```

### Sprint 10-12: Continue Optimizing

```bash
# Implement P1 actions based on Sprint 9 success

# Sprint 12: Second optimization cycle
/optimize-agents

# Output:
## Key Findings
1. Token usage stable at 300-320K per sprint
2. New pattern: quality-validator runs 3 iterations per test phase
   → Fix: Add validation to Phase 2 design specs
   → Impact: 8-12K token reduction

## Total Expected Impact: 8-12K additional reduction
```

### Long-term: Continuous Improvement

After 3-4 optimization cycles:
- **Token usage:** 250-280K per sprint (40-50% reduction from baseline)
- **Quality:** Zero rework sprints
- **Speed:** 1-2 days faster sprint completion
- **Cache efficiency:** 65-75% cache hit rate
- **Agent effectiveness:** Highly optimized prompts and docs

---

## Key Success Factors

### 1. Consistent Metrics Collection

**Critical:** Collect metrics every single sprint without exception.

**Why:** Without baseline data, can't measure improvement.

**How:** Make `/collect-metrics` mandatory in Phase 5 workflow.

### 2. Regular Analysis Cadence

**Recommended:** Every 3-4 sprints for deep analysis.

**Why:** Need multiple data points to identify reliable patterns.

**How:** Calendar reminder after every 3rd sprint closure.

### 3. Action Item Discipline

**Critical:** Actually implement the P0/P1 optimizations identified.

**Why:** Analysis is worthless if actions aren't executed.

**How:** Add action items to next sprint planning, track completion.

### 4. Measure Validate Iterate

**Critical:** Always measure impact of optimizations.

**Why:** Some optimizations work, some don't. Learn from data.

**How:** Compare Sprint N+1 metrics to Sprint N after implementing changes.

### 5. Update Decision Tree

**Important:** Document new patterns discovered.

**Why:** Framework learns from experience, gets smarter over time.

**How:** Add new patterns to decision tree when found.

---

## Technical Details

### Subagent Transcript Format

Transcripts stored at: `~/.claude/projects/<project>/<session-id>/subagents/agent-<id>.jsonl`

Each line is a JSON object:
```json
{
  "agentId": "a7c8742",
  "sessionId": "f599ef4e-6741-40b9-8b70-54c6e6d7272e",
  "timestamp": "2026-01-18T14:19:23.456Z",
  "type": "assistant",
  "message": {
    "id": "msg_123",
    "role": "assistant",
    "content": [...],
    "usage": {
      "input_tokens": 35769,
      "output_tokens": 15,
      "cache_creation_input_tokens": 129670,
      "cache_read_input_tokens": 142169
    }
  }
}
```

### Token Cost Calculation

**2026 Claude API Pricing:**

| Model | Input | Output | Cache Write | Cache Read |
|-------|-------|--------|-------------|------------|
| Sonnet 4.5 | $3/1M | $15/1M | $3.75/1M | $0.30/1M |
| Opus 4.5 | $15/1M | $75/1M | $18.75/1M | $1.50/1M |
| Haiku 4.5 | $1/1M | $5/1M | $1.25/1M | $0.10/1M |

**Effective cost formula:**
```
Cost = (input_tokens × input_rate) +
       (output_tokens × output_rate) +
       (cache_creation_tokens × cache_write_rate) +
       (cache_read_tokens × cache_read_rate)
```

**Cache savings:**
Cache reads cost 10% of normal input tokens.
With 60% cache hit rate, effective input cost drops by ~54%.

### Dependencies

**Required tools:**
- `bash` - Script execution
- `jq` - JSON parsing
- Standard Unix utilities: `grep`, `awk`, `sort`, `uniq`

**Optional tools:**
- `bc` - Floating point arithmetic (for percentages)

---

## Troubleshooting

### "No subagent transcripts found"

**Cause:** Sprint didn't use any subagents.

**Solution:** Metrics collection only works for sprints that launched agents (cli-ux-designer, rust-teradata-architect, etc.). If the main agent did all the work, there are no subagent metrics to collect.

### "JSONL format changed"

**Cause:** Claude Code updated transcript format.

**Solution:** Update the extraction script's `jq` queries to match new format. Check structure:
```bash
head -2 <transcript>.jsonl | jq .
```

### "Metrics don't match expected values"

**Cause:** Multi-turn agent conversations, or subagents spawning subagents.

**Solution:** The script sums ALL usage records in the transcript. Verify by manual inspection:
```bash
jq 'select(.message.usage) | .message.usage' <transcript>.jsonl
```

### "Optimize-agents skill takes too long"

**Cause:** Analyzing many sprints or large transcripts.

**Solution:**
- Limit to 3-6 most recent sprints
- Use `context: fork` to run in background
- Opus model is intentionally used for quality analysis

---

## Future Enhancements

### Potential Additions

1. **Automated Implementation**
   - Generate PRs with optimization changes
   - A/B test prompt variations automatically
   - Rollback ineffective optimizations

2. **Predictive Analysis**
   - Estimate sprint token budget before starting
   - Warn if scope seems too large
   - Suggest simpler approaches

3. **Cross-Project Learning**
   - Share anonymized patterns across projects
   - Build library of common optimization patterns
   - Community-contributed decision tree entries

4. **Real-Time Monitoring**
   - Track token usage during sprint (not just at end)
   - Alert if anomaly detected mid-sprint
   - Course-correct before wasting tokens

5. **Visual Dashboards**
   - Web UI for metrics visualization
   - Trend graphs over time
   - Optimization impact validation

### But Start Simple

The current system (metrics collection + periodic analysis) is deliberately minimal:
- ✅ Easy to understand
- ✅ Easy to maintain
- ✅ Delivers clear value
- ✅ Proven effective

Add complexity only when simple system plateaus.

---

## Summary

**The self-improvement system enables the TQ framework to optimize itself:**

1. **Every sprint:** Collect factual token metrics (`/collect-metrics`)
2. **Every 3-4 sprints:** Deep analysis and optimization (`/optimize-agents`)
3. **Continuous:** Measure impact, validate improvements, iterate

**Expected outcomes:**
- 30-50% token reduction within 3-4 optimization cycles
- Zero quality failures through improved processes
- Faster sprints via better parallelism and less rework
- Continuously improving agent efficiency

**Key insight:** The framework learns from experience, identifies waste patterns, and generates concrete improvements—a true self-improving system.

---

## Quick Start

### First Time Setup

```bash
# 1. Ensure script exists and is executable
chmod +x .claude/scripts/extract-sprint-metrics.sh

# 2. Read the decision tree
cat docs/builder/token-optimization-decision-tree.md

# 3. Ready to use!
```

### After Each Sprint

```bash
# In sprint closure (Phase 5)
/collect-metrics <sprint-number>

# Review metrics in sprint-N-review.md
```

### Every 3-4 Sprints

```bash
# Deep analysis and optimization
/optimize-agents

# Review recommendations
# Implement P0 actions
# Measure impact in next sprint
```

That's it! The system is designed to be simple, effective, and continuously improving.
