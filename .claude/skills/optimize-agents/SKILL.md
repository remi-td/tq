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

**Input:** Historical sprint metrics files (`sprint-*-metrics.md`)
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
ls -t docs/builder/sprints/sprint-*-metrics.md | head -6
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
for metrics in docs/builder/sprints/sprint-{6,7,8}-metrics.md; do
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
for review in docs/builder/sprints/sprint-{6,7,8}-review.md; do
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
grep -h "Cache Hit Rate" docs/builder/sprints/sprint-*-metrics.md | sort
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
Add "Metadata Module Overview" section to rust-architecture.md explaining:
- MetadataCache purpose and design
- How sql_context uses metadata
- Common patterns for extending metadata features

### Implementation

**File:** `docs/builder/rust-architecture.md`
**Action:** Add new section after "6.2 REPL Module Organization"

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
```

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

Generate a concrete action list:

```markdown
# Framework Optimization Plan
**Generated:** <date>
**Based on:** Sprint 6-8 metrics analysis
**Total Expected Impact:** 35-50K token reduction per sprint (30-40%)

## P0 - Critical (Implement in Sprint 9)

### Action 1: Make Phase 3.5 Database Check Mandatory
**Files:** `.claude/skills/sprint-coordinator/SKILL.md`
**Change:** Update Phase 3.5 section
**Old behavior:** "Verify database connectivity (optional)"
**New behavior:** "Run `tq ping`. If fails, STOP sprint. Do not proceed."
**Expected impact:** Prevent quality failures (Sprint 8 pattern), save 20-40K rework tokens
**Effort:** 15 minutes
**Owner:** Main agent (update sprint-coordinator before next sprint)

### Action 2: Add Module Overviews to Rust Architecture
**Files:** `docs/builder/rust-architecture.md`
**Change:** Add sections 6.3 (Metadata), 6.4 (Completion), 6.5 (Connection Management)
**Expected impact:** 10-15K token reduction (fewer redundant reads)
**Effort:** 2-3 hours
**Owner:** Main agent or rust-teradata-architect
**Details:** [Full content provided in Optimization #1 above]

## P1 - High Priority (Implement in Sprint 9-10)

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

## P2 - Medium Priority (Implement in Sprint 10-11)

[Continue for all identified optimizations]

## Implementation Timeline

**Sprint 9:**
- [ ] P0 Action 1: Database check mandatory (15 min)
- [ ] P0 Action 2: Architecture docs module overviews (2-3 hours)
- [ ] P1 Action 3: Architect prompt optimization (30 min)
- [ ] Measure impact in Sprint 9 metrics

**Sprint 10:**
- [ ] P1 Action 4: Test case YAML template (1 hour)
- [ ] P2 Actions if Sprint 9 showed positive results
- [ ] Continue measuring

**Sprint 11:**
- [ ] Remaining P2 actions
- [ ] Quarterly review: Did optimizations work?
- [ ] Refine waste patterns catalog based on results
```

### Step 8: Present Findings to User

Summarize in a concise report:

```markdown
# Framework Optimization Analysis Complete

## Metrics Analyzed
- Sprint 6: [X] tokens, [issues]
- Sprint 7: [Y] tokens, [issues]
- Sprint 8: [Z] tokens, [quality failure]

## Key Findings

### 1. Quality Failure Pattern (P0 - Critical)
**Issue:** Sprint 8 features marked complete but broken
**Root Cause:** Manual tests skipped, database unavailable during testing
**Fix:** Make Phase 3.5 mandatory, enforce manual test documentation
**Impact:** Prevents 20-40K token rework per quality failure

### 2. Inefficient Agent: rust-teradata-architect (P0)
**Issue:** Reads same files 4-5 times per sprint (12-15K wasted tokens)
**Root Cause:** Missing module architecture documentation
**Fix:** Add module overviews to rust-architecture.md
**Impact:** 10-15K token reduction per sprint

### 3. Low Cache Hit Rate: cli-ux-designer (P1)
**Issue:** 22% cache hit rate (paying 10x more than necessary)
**Root Cause:** Volatile specification formats
**Fix:** Create stable spec templates, separate variable content
**Impact:** 5-8K token reduction per sprint

### 4. Sequential Execution (P1)
**Issue:** Phases run sequentially when parallel possible
**Root Cause:** Sprint-coordinator not launching agents in parallel
**Fix:** Update workflow with explicit parallelism instructions
**Impact:** 30-50% faster sprints (indirect token savings)

## Total Expected Impact

**Token Reduction:** 35-50K per sprint (30-40% reduction)
**Quality Improvement:** Zero rework sprints (Sprint 8 pattern eliminated)
**Time Savings:** 1-2 days faster sprint completion

## Recommended Next Steps

1. **Review implementation plan** (see full plan above)
2. **Prioritize P0 actions** for immediate implementation
3. **Implement in Sprint 9** and measure results
4. **Iterate based on Sprint 9 metrics**

Would you like me to:
- [ ] Implement P0 actions now (30 min - 3 hours)
- [ ] Create detailed content for documentation updates
- [ ] Generate specific agent prompt edits
- [ ] Set up validation metrics for Sprint 9
```

## Important Notes

### This Skill Uses Opus Model

Why Opus?
- Requires complex pattern recognition across large datasets
- Needs to generate detailed, accurate documentation content
- Must apply waste patterns catalog logic systematically
- Outputs are high-leverage (impact entire framework)

### Context: Fork

Runs in separate context to avoid polluting main conversation with verbose analysis.

### Human Approval Required

This skill generates **recommendations**, not automatic changes. User reviews and approves before implementation.

### Iterative Improvement

After each round of optimizations:
1. Measure impact in next sprint metrics
2. Validate expected reductions achieved
3. Refine waste patterns catalog with new learnings
4. Remove ineffective recommendations

## Expected Outcomes

After 2-3 optimization cycles:

- **Token Usage:** 30-50% reduction from baseline
- **Quality:** Zero rework sprints
- **Speed:** 30-50% faster sprint completion
- **Cache Efficiency:** >60% cache hit rate
- **Agent Efficiency:** <2 fix iterations per sprint

## Meta-Learning

This skill enables the framework to **optimize itself**:

- Sprint N: Collect metrics
- Analysis: Identify waste patterns
- Sprint N+1: Apply optimizations, measure impact
- Refinement: Keep what works, discard what doesn't
- Sprint N+2: New baseline, new optimization cycle

The framework continuously improves through measured experimentation.
