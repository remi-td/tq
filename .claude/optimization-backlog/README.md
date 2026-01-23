# Framework Optimization Backlog

This directory contains the framework's self-improvement backlog: concrete, data-driven proposals to reduce token usage, improve quality, and enhance developer experience.

## Directory Structure

```
optimization-backlog/
├── README.md           # This file - explains the system
├── pending/            # Proposals awaiting implementation
├── implemented/        # Completed proposals (moved from pending/)
└── rejected/           # Proposals decided against (with rationale)
```

## How the System Works

### 1. Analysis Phase (End of Every Sprint)

After feature code is committed, the main agent launches parallel optimization analysis:

```
Task(optimization-analyzer) for transcript-001.md
Task(optimization-analyzer) for transcript-002.md
Task(optimization-analyzer) for transcript-003.md
...
```

Each `optimization-analyzer` agent:
- Analyzes one transcript from the sprint
- Identifies token waste patterns using `/optimize-agents` skill
- Generates structured proposals with metrics
- Returns raw proposals to main agent

### 2. Consolidation Phase

The main agent:
- Collects all proposals from parallel analyzers
- Deduplicates similar proposals
- Assigns proposal IDs (P001, P002, etc.)
- Adds proposals to `pending/` directory

### 3. Prioritization Phase

The main agent calculates impact scores:

```
Impact Score = (Tokens Saved × Frequency Weight × Confidence Weight) / Implementation Effort

Where:
- Frequency Weight: Daily=30, Per sprint=10, Per feature=3, Rare=1
- Confidence Weight: High=1.0, Medium=0.6, Low=0.3
- Implementation Effort: Small=1, Medium=3, Large=5
```

High-impact proposals (score > 500 OR high confidence + small effort) are prioritized.

### 4. Implementation Phase

The main agent:
- Selects top 3-5 proposals
- Compacts conversation history
- Implements selected proposals
- Moves proposals from `pending/` to `implemented/`
- Commits changes to git

### 5. Validation Phase (Next Sprint)

After implementation:
- Next sprint metrics should show expected reductions
- If successful: Document in proposal file
- If failed: Investigate why, refine approach or reject

## Proposal File Format

Each proposal file follows this template:

```markdown
# P###: [Proposal Name]

**Status**: Pending | Implemented | Rejected
**Created**: Sprint X
**Implemented**: Sprint Y (if applicable)
**Impact Score**: [calculated score]

## Metrics
- **Tokens Saved**: X per occurrence
- **Frequency**: "3 times per sprint" | "once per feature" | etc.
- **Confidence**: High | Medium | Low
- **Estimated Implementation Effort**: Small | Medium | Large

## Problem
[What inefficiency/waste was observed? Include specific evidence.]

## Proposed Solution
[Detailed, actionable steps. Be specific about what changes to make.]

## Files Affected
- `.claude/subagents/foo.md` (line XX: add/modify/remove...)
- `docs/builder/bar.md` (section YY: add content...)

## Validation Criteria
[How do we know this optimization worked? What should decrease in future sprints?]

## Evidence from Transcripts
[Quote specific passages from sprint transcripts that demonstrate the waste, with line numbers]

## Implementation Notes
[Added when implemented - what was actually done, any deviations from plan]

## Validation Results
[Added after next sprint - did it work as expected?]
```

## Proposal Statuses

### Pending
- Analyzed and documented
- Not yet implemented
- In the backlog for consideration

### Implemented
- Changes have been made
- Moved from `pending/` to `implemented/`
- Awaiting validation in next sprint

### Rejected
- Decided not to implement
- Moved from `pending/` to `rejected/`
- Must include rationale for rejection

## Prioritization Criteria

### P0 - Critical (Implement Immediately)
- Prevents quality failures
- Saves >10K tokens per sprint
- Fixes recurring issues

**Example:** Make database connectivity check mandatory

### P1 - High Priority (Implement Soon)
- Saves 5-10K tokens per sprint
- Enables parallel execution
- Improves agent efficiency

**Example:** Add module overviews to reduce exploration

### P2 - Medium Priority (Implement When Capacity)
- Saves 2-5K tokens per sprint
- Improves cache hit rate
- Minor workflow improvements

**Example:** Stabilize agent system prompts

### P3 - Low Priority (Backlog)
- Saves <2K tokens per sprint
- Quality of life improvements
- Nice-to-have optimizations

**Example:** Improved error messages

## Impact Score Examples

**High Impact (Score > 1000):**
```
Tokens Saved: 5,000
Frequency: 3 times per sprint (weight: 10)
Confidence: High (weight: 1.0)
Effort: Small (divisor: 1)

Score = (5000 × 10 × 1.0) / 1 = 50,000
```

**Quick Win (High confidence, small effort):**
```
Tokens Saved: 1,500
Frequency: Once per sprint (weight: 3)
Confidence: High (weight: 1.0)
Effort: Small (divisor: 1)

Score = (1500 × 3 × 1.0) / 1 = 4,500
```

**Uncertain, High Effort (Lower priority):**
```
Tokens Saved: 10,000
Frequency: Once per sprint (weight: 3)
Confidence: Low (weight: 0.3)
Effort: Large (divisor: 5)

Score = (10000 × 3 × 0.3) / 5 = 1,800
```

## Best Practices

### When Adding Proposals

**DO:**
- Provide concrete evidence from transcripts
- Calculate realistic token savings
- Specify exact file changes needed
- Include validation criteria
- Reference waste patterns from optimize-agents skill

**DON'T:**
- Create vague proposals ("make agents better")
- Guess token savings without evidence
- Propose solutions without root cause analysis
- Optimize for metrics instead of value

### When Implementing Proposals

**DO:**
- Implement in priority order
- Measure impact in next sprint
- Document what was actually done
- Update proposal status

**DON'T:**
- Implement everything at once
- Skip validation phase
- Ignore failed optimizations
- Forget to move files between directories

### When Validating Results

**Success Indicators:**
- Token usage decreased as predicted
- Quality maintained or improved
- No new problems introduced
- Pattern confirmed eliminated

**Failure Indicators:**
- No measurable improvement
- New issues introduced
- Prediction was significantly off

**If Failed:**
- Investigate root cause
- Document lessons learned
- Consider rejecting or refining
- Don't repeat same approach

## Workflow Integration

This backlog is integrated into the sprint workflow at Phase 6 (Framework Optimization):

```
6. Framework Optimization (End of every sprint)
   ├─→ Step 1: Close sprint work
   │   └─→ tq-project-manager: Commit & push feature code
   │
   ├─→ Step 2: Parallel optimization analysis
   │   ├─→ Task(optimization-analyzer) for transcript-001.md
   │   ├─→ Task(optimization-analyzer) for transcript-002.md
   │   ├─→ Task(optimization-analyzer) for transcript-003.md
   │   └─→ Task(optimization-analyzer) for transcript-00N.md
   │
   ├─→ Step 3: Merge proposals into backlog
   │   └─→ Main agent: Deduplicate, assign IDs, add to pending/
   │
   ├─→ Step 4: Prioritize & implement
   │   ├─→ Main agent: Calculate impact scores, select top 3-5
   │   ├─→ Main agent: Compact history
   │   └─→ Main agent: Implement selected proposals
   │
   └─→ Step 5: Commit framework improvements
       └─→ Main agent: Commit & push optimization changes
```

## Example Proposal Lifecycle

### Sprint 11: Proposal Created

Agent transcript shows metadata.rs read 5 times (20K tokens wasted).

**Action:** Create `pending/P042-reduce-metadata-reads.md`

### Sprint 12: Proposal Prioritized

```
Impact Score = (4000 × 3 × 1.0) / 1 = 12,000  # High priority!
```

**Action:** Selected for implementation

### Sprint 12: Proposal Implemented

Add metadata system overview to design documentation (docs/design/repl.md).

**Action:** Move to `implemented/P042-reduce-metadata-reads.md`

### Sprint 13: Validation

Sprint 13 metrics show metadata.rs reads decreased from 5 to 1.

**Result:** Success! Document in proposal file.

## Success Metrics

After 3-4 optimization cycles, expect:

- **Token Reduction:** 30-50% from baseline
- **Quality Improvement:** Zero rework sprints
- **Speed Improvement:** 30-50% faster sprint completion
- **Cache Efficiency:** >60% cache hit rate
- **Fix Iterations:** <2 per sprint

## Questions?

See:
- `.claude/subagents/optimization-analyzer.md` - How proposals are generated
- `.claude/skills/optimize-agents/SKILL.md` - Optimization framework
- `.claude/skills/optimize-agents/references/waste-patterns.md` - Pattern catalog
- `CLAUDE.md` - Full sprint workflow including optimization phase
