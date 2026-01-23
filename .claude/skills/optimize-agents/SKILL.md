---
name: optimize-agents
description: Analyzes historical sprint metrics to identify framework optimization opportunities. Generates concrete actions to improve agents, tools, documentation, and workflows. Use after collecting metrics from 2+ sprints.
allowed-tools: Read, Bash, Write, Edit, Grep, Glob
model: claude-opus-4-5-20251101
context: fork
---

# Optimize Agents Skill

## Purpose

Deep analysis of sprint metrics to identify where time/tokens are wasted and generate concrete optimization actions.

**Input:** Historical sprint metrics files (`docs/sprints/sprint-N-metrics.md`)
**Output:** Specific file edits to improve agents, docs, tools, and workflows
**Model:** Opus (requires complex analysis and decision-making)

## When to Use

**Quarterly:** After every 3-4 sprints to identify systemic patterns
**On-Demand:** After quality failures or unusually high token usage
**Minimum Data:** At least 2 sprints with metrics collected

```
/optimize-agents
```

## What This Skill Does

### 1. Analyze Where Time/Tokens Were Wasted

Examines metrics to identify:
- **Inefficient agents:** High token usage relative to output quality
- **Redundant work:** Repeated file reads, duplicate searches
- **Missing context:** Agents asking questions that docs should answer
- **Workflow issues:** Sequential execution when parallel possible
- **Quality failures:** Rework due to bugs, missed requirements
- **Missing tools:** Tasks agents improvise that should be automated

### 2. Apply Pattern Analysis

Uses bundled `references/waste-patterns.md` to systematically:
- Match transcript operations to known waste patterns
- Identify root causes
- Map to documented solutions
- Estimate impact of fixes

### 3. Generate Concrete Optimization Actions

Produces specific, actionable improvements:
- **Agent prompt edits:** Exact changes to `.claude/subagents/*.md`
- **Documentation updates:** Sections to add to architecture/testing guides
- **Tool creation:** Scripts/utilities to automate repetitive tasks
- **Workflow improvements:** Changes to sprint-coordinator skill

### 4. Prioritize by Impact

Ranks optimizations by expected token reduction and implementation effort.

## Workflow

### Step 1: Gather Historical Data

```bash
# Find all sprint metrics files
ls -t docs/sprints/sprint-*-metrics.md | head -6
```

Read the most recent 3-6 sprint metrics files to establish patterns.

### Step 2: Load Waste Patterns Catalog

The bundled `references/waste-patterns.md` is automatically available when this skill loads. It contains:
- 12 common waste patterns with symptoms and solutions
- Detection techniques for each pattern
- Quantification methods
- Prioritization framework

Reference this catalog throughout your analysis to match observed behaviors to known patterns.

### Step 3: Identify Patterns Across Sprints

For each pattern in the waste patterns catalog, check if it appears in multiple sprints:

#### Pattern: High Token Agent

```bash
# Extract token usage per agent across sprints
for metrics in docs/sprints/sprint-{6,7,8}-metrics.md; do
    echo "=== $(basename $metrics) ==="
    grep -A 20 "## Sprint Summary" "$metrics" | grep "Total Tokens"
done
```

**Questions to answer:**
- Which agent consistently uses most tokens?
- Is token usage increasing or decreasing over time?
- Which sprints had unusually high usage?

#### Pattern: Quality Failures

```bash
# Check sprint reviews for quality issues
for review in docs/sprints/sprint-{6,7,8}-review.md; do
    echo "=== $(basename $review) ==="
    grep -i "issue\|bug\|failure\|rework\|manual test" "$review" | head -5
done
```

**Questions to answer:**
- Did any sprint have quality failures?
- Were manual tests skipped? (Sprint 8 pattern)
- Was there rework due to bugs?

#### Pattern: Low Cache Hit Rate

```bash
# Extract cache rates from metrics
grep -h "Cache Hit Rate" docs/sprints/sprint-*-metrics.md | sort
```

**Questions to answer:**
- Which agents have < 40% cache hit rate?
- Is this consistent across sprints?
- Why is caching ineffective?

#### Pattern: Missing Context

Read subagent transcripts to identify clarifying questions:

```bash
# Find most recent subagent transcript
transcript=$(ls -t ~/.claude/projects/-Users-remi-turpaud-Code-genAI-tq/*/subagents/*.jsonl | head -1)

# Search for question patterns
jq -r 'select(.message.content) | .message.content[] | select(.type == "text") | .text' "$transcript" | grep -E "What is|How do I|Where should|Should I" | head -10
```

**Questions to answer:**
- What information are agents frequently asking for?
- Which documentation is missing or unclear?
- What architectural decisions are not documented?

#### Pattern: Redundant Operations

Analyze tool usage in transcripts:

```bash
# Count Read operations per file
transcript=$(ls -t ~/.claude/projects/-Users-remi-turpaud-Code-genAI-tq/*/subagents/agent-*.jsonl | head -1)

jq -r 'select(.message.content) | .message.content[] | select(.type == "tool_use" and .name == "Read") | .input.file_path' "$transcript" | sort | uniq -c | sort -rn | head -10
```

**Questions to answer:**
- Which files are read 3+ times?
- Could they be pre-loaded in agent instructions?
- Are agents exploring or do they know what to read?

### Step 4: Map Patterns to Root Causes

For each pattern identified, use the waste patterns catalog to determine:

1. **Why is this happening?**
   - Missing documentation?
   - Unclear agent instructions?
   - Missing tools?
   - Workflow inefficiency?

2. **What's the root cause?**
   - Agent prompt too vague
   - Documentation gap
   - Lack of automation
   - Scope too large

3. **What's the fix?**
   - Add specific section to docs
   - Clarify agent instruction
   - Create automation script
   - Adjust sprint scope

### Step 5: Generate Concrete Actions

For each identified optimization, create a detailed action plan:

```markdown
## Optimization #1: [Pattern Name]

### Evidence
- Sprint 7: rust-teradata-architect read `src/db/metadata.rs` 5 times (5K tokens wasted)
- Sprint 8: rust-teradata-architect read `src/db/metadata.rs` 4 times (4K tokens wasted)
- Pattern consistent across 2 sprints

### Root Cause
Agent doesn't understand metadata module structure upfront. Explores file multiple times during implementation.

### Solution
Add "Metadata Module Overview" section to design documentation explaining:
- MetadataCache purpose and design
- How sql_context uses metadata
- Common patterns for extending metadata features

### Implementation

**File:** `docs/design/repl.md`
**Action:** Add new section explaining metadata system architecture

**Content to add:**
```markdown
### 6.3 Metadata System Architecture

The metadata system provides database introspection for tab completion:

**Core Components:**
- `src/db/metadata.rs` - MetadataCache with lazy loading
- `src/commands/repl/sql_context.rs` - SQL statement parsing
- `src/commands/repl/metadata_completer.rs` - Completion logic

**Design Pattern:**
1. User presses Tab
2. sql_context analyzes SQL statement (determines context: table? column?)
3. metadata_completer queries MetadataCache (lazy-loads if first request)
4. Cache returns results with 300-500ms timeout
5. Completer formats suggestions for reedline

**Common Extension Pattern:**
When adding new completion features:
1. Extend CompletionContext enum in sql_context
2. Add detection logic in analyze_completion_context()
3. Add cache query method in metadata.rs (follow table/column pattern)
4. Wire up in metadata_completer.rs complete() method

**Key Invariants:**
- Never block REPL startup (lazy loading)
- Always timeout metadata queries (300-500ms)
- Cache is session-scoped (clear on /logon)
- Graceful degradation on query failures
```

**Expected Impact:**
- Reduces metadata.rs reads from 4-5 per sprint to 1
- Saves 3-4K tokens per sprint
- Faster implementation (less exploration)

**Validation:**
In Sprint 9 metrics, check that metadata.rs reads decrease to 1-2.

### Step 6: Prioritize Actions

Rank all identified optimizations by:

**Priority Matrix:**
| Priority | Criteria | Example |
|----------|----------|---------|
| **P0** | Prevents quality failures | Make Phase 3.5 database check mandatory |
| **P0** | >10K token reduction | Add module overviews to architecture docs |
| **P1** | 5-10K token reduction | Optimize agent prompts for conciseness |
| **P1** | Enables parallel execution | Update sprint-coordinator workflow |
| **P2** | Improve cache hit rate | Stabilize agent system prompts |
| **P2** | <5K token reduction | Minor prompt clarifications |
| **P3** | Quality of life | Improved error messages |

### Step 7: Create Implementation Plan

Generate a concrete action list, save it in `docs/sprints/sprint-N-planning.md` (where N is the sprint number), for example:

```markdown
# Framework Optimization Plan
**Generated:** <date>
**Based on:** Sprint 6-8 metrics analysis
**Total Expected Impact:** 35-50K token reduction per sprint (30-40%)

## P0 - Critical (Implement now)

### Action 1: Make Phase 3.5 Database Check Mandatory
**Files:** `.claude/skills/sprint-coordinator/SKILL.md`
**Change:** Update Phase 3.5 section
**Old behavior:** "Verify database connectivity (optional)"
**New behavior:** "Run `tq ping`. If fails, STOP sprint. Do not proceed."
**Expected impact:** Prevent quality failures (Sprint 8 pattern), save 20-40K rework tokens
**Effort:** 15 minutes
**Owner:** Main agent (update sprint-coordinator before next sprint)

### Action 2: Add Module Overviews to Design Documentation
**Files:** `docs/design/repl.md`, `docs/design/connection-management.md`
**Change:** Add detailed sections for Metadata System, Tab Completion Architecture, Connection Lifecycle
**Expected impact:** 10-15K token reduction (fewer redundant reads)
**Effort:** 2-3 hours
**Owner:** Main agent or rust-teradata-architect
**Details:** [Full content provided in Optimization #1 above]

## P1 - High Priority (Implement asap)

### Action 3: Optimize rust-teradata-architect Prompt
**Files:** `.claude/subagents/rust-teradata-architect.md`
**Change:** Add "Files Involved" guidance
**Expected impact:** 5-8K token reduction (less exploration)
**Effort:** 30 minutes

### Action 4: Create Test Case YAML Template
**Files:** `docs/builder/testing-guidelines.md`, `tests/cases/TC-TEMPLATE.yaml`
**Change:** Replace prose-heavy test cases with structured YAML
**Expected impact:** 5-7K token reduction (more efficient test documentation)
**Effort:** 1 hour

## P2 - Medium Priority (plan to implement)

[Continue for all identified optimizations]
```

### Step 7: Implementation
Implement changes in agent prompts, skills or directly Claude.md for Critical and High Priority action items.
