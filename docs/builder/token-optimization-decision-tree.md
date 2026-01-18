# Token Optimization Decision Tree

This document provides a systematic approach to analyzing sprint metrics and identifying concrete optimization actions.

## How to Use This Decision Tree

1. Run `./extract-sprint-metrics.sh <session-id> <sprint-num>` after each sprint
2. Review the generated metrics in `sprint-N-metrics.md`
3. Follow this decision tree to identify optimization opportunities
4. Document action items in sprint review
5. Implement optimizations in next sprint

---

## Decision Tree

### Start: Review Sprint Metrics

```
┌─────────────────────────────────────┐
│ Sprint N Metrics Generated          │
│ Review: sprint-N-metrics.md         │
└──────────────┬──────────────────────┘
               │
               ▼
      ┌────────────────────┐
      │ Compare to Baseline│
      │ (Sprint 6-7 avg)   │
      └────────┬───────────┘
               │
               ▼
```

---

## 1. High Token Usage Pattern

**Trigger:** Grand total > 150,000 tokens (baseline + 50%)

```
Grand Total > 150K?
├─ YES → Investigate cause
│   ├─ Which agent used most tokens?
│   │   ├─ rust-teradata-architect (Opus)?
│   │   │   └─ Go to Section 1.1
│   │   ├─ quality-validator (Sonnet)?
│   │   │   └─ Go to Section 1.2
│   │   ├─ cli-ux-designer (Sonnet)?
│   │   │   └─ Go to Section 1.3
│   │   └─ Multiple agents high?
│   │       └─ Go to Section 1.4
│   │
└─ NO → Go to Section 2 (Quality Metrics)
```

### 1.1 Rust-Teradata-Architect High Token Usage

**Diagnostic Questions:**

1. **Check cache hit rate**
   - < 30%? → Agent is re-reading context
   - Action: Add more stable context to prompt cache

2. **Check number of Edit operations**
   - Find in subagent transcript: `grep -c '"tool_name":"Edit"' agent-*.jsonl`
   - > 20 edits? → Agent struggling with code changes
   - Action: Provide better code context upfront

3. **Check Read operations**
   - Find: `grep -c '"tool_name":"Read"' agent-*.jsonl`
   - > 30 reads? → Agent exploring too much
   - Action: Pre-specify files to modify in agent prompt

**Common Patterns & Solutions:**

| Pattern | Symptom | Root Cause | Solution | File to Update |
|---------|---------|------------|----------|----------------|
| **Repeated file reads** | Same file read 3+ times | Missing context about file structure | Add file overview to rust-architecture.md | `docs/builder/rust-architecture.md` |
| **Many Edit failures** | Error logs in transcript | Incorrect old_string matching | Update rust-coder skill with Edit best practices | `.claude/skills/rust-coder/SKILL.md` |
| **Exploration phase** | 20+ Read before first Edit | Unclear which files to modify | Add "Files Involved" section to sprint planning | `sprint-N-planning.md` template |
| **Verbose prompts** | High input tokens on first message | Sprint coordinator over-explaining | Use document references instead of inline context | `.claude/skills/sprint-coordinator/SKILL.md` |

**Concrete Actions:**

```markdown
## Action Items for Next Sprint

### Optimize rust-teradata-architect prompts
- [ ] Add "Files to Modify" section to sprint planning (reduces exploration)
- [ ] Update rust-architecture.md with [specific module] overview (provides context)
- [ ] Add Edit best practice to rust-coder skill: "Read section before editing"
- [ ] Expected reduction: 10-15K tokens

**Implementation:**
1. Edit sprint-coordinator skill to include file list in planning
2. Add module documentation to rust-architecture.md
3. Update rust-coder/SKILL.md with Edit guidelines
```

---

### 1.2 Quality-Validator High Token Usage

**Diagnostic Questions:**

1. **Check test case count**
   - > 30 test cases designed? → Appropriate
   - < 15 test cases for complex feature? → Insufficient testing (quality issue)

2. **Check iteration count**
   - Find: Count how many times tests were re-run
   - > 3 iterations? → Implementation quality problem
   - Action: Improve Phase 2 design thoroughness

3. **Check test execution logs**
   - Long test output logs? → Too verbose
   - Action: Summarize test results instead of full logs

**Common Patterns & Solutions:**

| Pattern | Symptom | Root Cause | Solution | File to Update |
|---------|---------|------------|----------|----------------|
| **Multiple test/fix iterations** | quality-validator invoked 3+ times | Implementation bugs | Improve Phase 2 design specs | `detailed-specifications/*.md` |
| **Verbose test case design** | Each test case > 500 tokens | Prose-heavy documentation | Use structured format (YAML/table) | `testing-guidelines.md` |
| **Re-running passing tests** | Same tests executed multiple times | Inefficient workflow | Only run failed tests in iterations | `.claude/skills/sprint-coordinator/SKILL.md` |
| **Full test logs in output** | Output includes all test stdout | Unnecessary verbosity | Summarize: "X passed, Y failed" | `.claude/skills/quality-validator.md` |

**Concrete Actions:**

```markdown
## Action Items for Next Sprint

### Optimize quality-validator efficiency
- [ ] Update testing-guidelines with structured test case format
- [ ] Add to sprint-coordinator: "Run only failed tests in fix iterations"
- [ ] Update quality-validator prompt: "Summarize results, don't paste full logs"
- [ ] Expected reduction: 5-8K tokens

**Implementation:**
1. Create test case YAML template in testing-guidelines.md
2. Update sprint-coordinator Phase 4 with selective re-testing
3. Edit quality-validator agent prompt for concise reporting
```

---

### 1.3 CLI-UX-Designer High Token Usage

**Diagnostic Questions:**

1. **Check specification length**
   - Find: `wc -l docs/builder/detailed-specifications/[new-file].md`
   - > 800 lines? → Too verbose
   - Action: Use tabular formats, reduce prose

2. **Check if duplicating content**
   - Does spec repeat content from other specs?
   - Action: Cross-reference instead of duplicating

3. **Check example count**
   - > 10 code examples in one spec? → Excessive
   - Action: Provide 2-3 representative examples only

**Common Patterns & Solutions:**

| Pattern | Symptom | Root Cause | Solution | File to Update |
|---------|---------|------------|----------|----------------|
| **Verbose specifications** | Spec > 800 lines | Prose-heavy writing style | Add conciseness guideline to agent prompt | `.claude/subagents/cli-ux-designer.md` |
| **Duplicate content** | Same concepts explained multiple places | Not cross-referencing | Use markdown links to reference existing docs | Agent prompt + CLAUDE.md |
| **Over-specifying** | Details implementation internals | Confusing UX spec with architecture | Clarify boundary: UX = user-visible only | `.claude/subagents/cli-ux-designer.md` |
| **Too many examples** | 10+ code blocks per feature | Trying to cover all cases | Guideline: 2-3 representative examples max | Agent prompt |

**Concrete Actions:**

```markdown
## Action Items for Next Sprint

### Optimize cli-ux-designer output
- [ ] Add to agent prompt: "Keep specs under 500 lines, use tables not prose"
- [ ] Add guideline: "Cross-reference existing docs instead of duplicating"
- [ ] Add: "2-3 examples max per feature, not exhaustive"
- [ ] Expected reduction: 8-12K tokens (in agent output + subsequent reads)

**Implementation:**
1. Edit .claude/subagents/cli-ux-designer.md frontmatter and instructions
2. Add example of good vs bad spec to reference docs
3. Update sprint-coordinator to check spec length before accepting
```

---

### 1.4 Multiple Agents High Token Usage

**Trigger:** All agents showing elevated token usage

**Likely Cause:** Sprint scope too large or workflow inefficiency

**Diagnostic Questions:**

1. **Check feature count**
   - > 5 features in sprint? → Too ambitious
   - Action: Reduce scope to 2-3 features per sprint

2. **Check if agents ran in parallel**
   - Look at timestamps in transcripts
   - Sequential when could be parallel? → Coordination issue
   - Action: Update sprint-coordinator parallelism rules

3. **Check for repeated context**
   - Do multiple agents read same 1000+ line specs?
   - Action: Create spec summaries for agents

**Concrete Actions:**

```markdown
## Action Items for Next Sprint

### Reduce overall sprint scope
- [ ] Limit to 2-3 P0/P1 features maximum
- [ ] Create 200-line summaries of detailed-specifications for agent consumption
- [ ] Update sprint-coordinator: Always launch design agents in parallel
- [ ] Expected reduction: 20-30K tokens across all agents

**Implementation:**
1. Add sprint scope guideline to CLAUDE.md workflow
2. Create detailed-specifications/*-summary.md files
3. Update sprint-coordinator skill with parallelism checklist
```

---

## 2. Quality Metrics Pattern

**Trigger:** Tests passed but quality issues found later (Sprint 8 scenario)

```
Quality failures found?
├─ YES → Identify root cause
│   ├─ Manual tests skipped?
│   │   └─ Go to Section 2.1
│   ├─ Database unavailable during testing?
│   │   └─ Go to Section 2.2
│   ├─ Edge cases not tested?
│   │   └─ Go to Section 2.3
│   └─ Requirements unclear?
│       └─ Go to Section 2.4
│
└─ NO → Go to Section 3 (Cache Efficiency)
```

### 2.1 Manual Tests Skipped

**THE MOST CRITICAL PATTERN** (Sprint 8 root cause)

**Symptoms:**
- Unit/integration tests pass 100%
- Features marked complete
- User reports features don't work
- No manual test execution logs in sprint review

**Root Cause:**
- Phase 3.5 (Database Check) was skipped
- Phase 4 manual testing was not executed
- Agents assumed "tests pass = features work"

**Solution - MANDATORY WORKFLOW CHANGES:**

```markdown
## CRITICAL Action Items (Implement Immediately)

### Make Phase 3.5 MANDATORY
- [ ] Update sprint-coordinator skill Phase 3.5:
      "Database check is MANDATORY. If ping fails, STOP sprint immediately."
- [ ] Add explicit STOP instruction: "DO NOT proceed to Phase 4 without verified database"
- [ ] Add user notification: "Database unavailable - sprint paused until fixed"

### Make Manual Testing MANDATORY
- [ ] Update quality-validator agent:
      "Design manual test cases for each feature"
      "Execute manual tests against live database"
      "Document test results with screenshots/logs"
- [ ] Add to sprint review template: "Manual tests executed: YES/NO (must be YES)"
- [ ] Update tq-project-manager validation:
      "Cannot mark sprint complete without manual test documentation"

### Expected Impact
- Prevents 100% of "tests pass but features broken" quality failures
- Zero token impact (prevents costly rework from quality failures)
- Estimated rework savings: 20-40K tokens per quality failure avoided
```

**Files to Update:**
1. `.claude/skills/sprint-coordinator/SKILL.md` - Phase 3.5 and Phase 4
2. `.claude/subagents/quality-validator.md` - Testing requirements
3. `.claude/subagents/tq-project-manager.md` - Validation checklist
4. `docs/builder/sprints/sprint-template-review.md` - Manual test documentation section

---

### 2.2 Database Unavailable During Testing

**Symptoms:**
- Tests designed but not executed
- Skipped tests in CI/CD logs
- "Requires live database" notes without execution

**Solution:**

```markdown
## Action Items

### Enforce Database Availability
- [ ] Phase 3.5 in sprint-coordinator: Run `./target/release/tq ping` before Phase 4
- [ ] If ping fails: STOP sprint, notify user, do not proceed
- [ ] Add to sprint planning: "Database availability confirmed: YES/NO"
- [ ] Expected reduction in rework: 10-20K tokens per failed sprint
```

---

### 2.3 Edge Cases Not Tested

**Symptoms:**
- Main functionality works
- Error conditions not handled
- Null/empty input crashes application

**Solution:**

```markdown
## Action Items

### Improve Test Coverage
- [ ] Add to testing-guidelines: "Always test: null, empty, invalid, boundary values"
- [ ] Update quality-validator: "Design negative test cases, not just happy path"
- [ ] Add error handling checklist to rust-architecture.md
- [ ] Expected impact: Catch 80% of edge case bugs before shipping
```

---

## 3. Cache Efficiency Pattern

**Trigger:** Cache hit rate < 40%

```
Cache hit rate < 40%?
├─ YES → Improve caching
│   ├─ Which agent has low cache rate?
│   │   └─ Go to Section 3.1
│   │
└─ NO → Go to Section 4 (Workflow Efficiency)
```

### 3.1 Low Cache Hit Rate

**Why This Matters:**
- Cached tokens cost 10% of normal price
- Low cache rate = paying 10x more than necessary

**Diagnostic Questions:**

1. **Check what's being cached**
   - Look at first message in agent transcript
   - Is it stable (same across sprints)?
   - If yes but low cache rate → Context changing unnecessarily

2. **Check cache invalidation**
   - Are prompts changing every sprint?
   - Are specifications being rewritten?
   - Action: Stabilize shared context

**Common Patterns & Solutions:**

| Pattern | Symptom | Root Cause | Solution |
|---------|---------|------------|----------|
| **Volatile prompts** | Cache rate < 20% | Agent prompts change every sprint | Separate stable context from variable context |
| **Large volatile context** | Cache created but never reused | Specifications change completely each sprint | Use stable architecture docs, variable feature specs |
| **No prompt caching** | cache_creation = 0 | Prompts too short to cache | Consolidate system prompts, add stable context |

**Concrete Actions:**

```markdown
## Action Items

### Improve Prompt Caching
- [ ] Separate agent prompts into: 1) Stable system context 2) Variable sprint context
- [ ] Create stable reference docs that don't change (rust-patterns.md, testing-patterns.md)
- [ ] Update sprint-coordinator to reference stable docs instead of regenerating context
- [ ] Expected reduction: 30-50% cost reduction via caching
```

---

## 4. Workflow Efficiency Pattern

**Trigger:** Review sprint timeline and agent interactions

```
Review sprint execution timeline
├─ Agents ran sequentially when parallel possible?
│   └─ Go to Section 4.1
├─ Multiple fix iterations in Phase 4?
│   └─ Go to Section 4.2
├─ Back-and-forth between main agent and subagents?
│   └─ Go to Section 4.3
└─ All good? → Document success patterns
```

### 4.1 Sequential When Parallel Was Possible

**Symptoms:**
- Phase 2: cli-ux-designer finished, then rust-teradata-architect started
- Phase 3: Implementation finished, then quality-validator started

**Token Impact:**
- No direct token waste
- Indirect: Longer sprint = more context maintenance

**Solution:**

```markdown
## Action Items

### Maximize Parallelism
- [ ] Update sprint-coordinator Phase 2: "Launch BOTH agents in single message"
- [ ] Update sprint-coordinator Phase 3: "Launch architect AND validator in parallel"
- [ ] Add reminder: "Use multiple <invoke name='Task'> blocks in one message"
- [ ] Expected impact: 30-50% faster sprint completion, indirect token savings
```

---

### 4.2 Multiple Fix Iterations

**Symptoms:**
- Quality-validator ran 3+ times
- Each iteration fixed 1-2 test failures
- Pattern: Test → Fix → Test → Fix → Test

**Token Impact:**
- Each iteration: 5-10K tokens
- 3 iterations: 15-30K wasted tokens

**Solution:**

```markdown
## Action Items

### Reduce Fix Iterations
- [ ] Phase 2: Add more detail to specifications (prevent ambiguity)
- [ ] Phase 3: Rust-teradata-architect must run tests before marking complete
- [ ] Phase 4: If >5 failures, batch-fix by component not one-by-one
- [ ] Expected reduction: 10-20K tokens per sprint
```

---

### 4.3 Back-and-Forth Communication

**Symptoms:**
- Agent asks clarifying questions
- Main agent provides answer
- Agent asks another question
- Pattern repeats 3+ times

**Token Impact:**
- Each round-trip: 2-5K tokens
- Indicates missing context

**Solution:**

```markdown
## Action Items

### Provide Better Initial Context
- [ ] Update [agent] prompt to include [specific information]
- [ ] Add [missing section] to [documentation file]
- [ ] Pre-answer common questions in agent system prompt
- [ ] Expected reduction: 5-10K tokens per sprint
```

---

## 5. Success Patterns (Document and Repeat)

**Trigger:** Sprint completed with low token usage and high quality

```
Sprint metrics look good!
└─ Document what worked
    ├─ Which agent prompts were effective?
    ├─ Which documentation was helpful?
    ├─ Which workflow decisions saved tokens?
    └─ Add to "Best Practices" section
```

**Template:**

```markdown
## Sprint N - Success Pattern Identified

### What Worked
- [Specific practice that was efficient]

### Why It Worked
- [Root cause of efficiency]

### How to Repeat
- [Concrete steps to replicate in future sprints]

### Expected Impact
- [Token reduction when repeated]

### Status
- [ ] Documented in relevant docs
- [ ] Added to sprint-coordinator workflow
- [ ] Trained into agent prompts
```

---

## Quick Reference: Action Priority Matrix

After analyzing metrics, prioritize actions using this matrix:

| Priority | Trigger | Expected Reduction | Effort | Do First |
|----------|---------|-------------------|--------|----------|
| **P0** | Quality failure | Prevents rework (20-40K) | 1-2 hours | ✅ YES |
| **P0** | Cache rate < 20% | 30-50% cost reduction | 2-3 hours | ✅ YES |
| **P1** | Agent >50K tokens | 10-20K reduction | 1-2 hours | ✅ YES |
| **P1** | Multiple fix iterations | 10-20K reduction | 2-3 hours | ✅ YES |
| **P2** | Sequential execution | Faster, indirect savings | 30 min | Later |
| **P2** | Verbose specs | 5-10K reduction | 1 hour | Later |
| **P3** | Minor optimizations | <5K reduction | 1 hour | Backlog |

---

## How to Implement Actions

For each action identified:

1. **Document in sprint review:**
   ```markdown
   ## Action Items for Next Sprint
   - [ ] [Specific action] - Owner: [Agent/Main] - Expected: [X]K token reduction
   ```

2. **Update relevant file:**
   - Agent prompts: `.claude/subagents/*.md`
   - Skills: `.claude/skills/*/SKILL.md`
   - Documentation: `docs/builder/*.md`
   - Workflow: `CLAUDE.md`

3. **Validate in next sprint:**
   - Run metrics again after Sprint N+1
   - Compare token usage to Sprint N
   - Confirm expected reduction achieved
   - If not, investigate why

4. **Iterate:**
   - Keep what works
   - Discard what doesn't
   - Refine continuously

---

## Summary: The Process

```
1. Complete Sprint N
2. Extract metrics: ./extract-sprint-metrics.sh <session-id> N
3. Review sprint-N-metrics.md
4. Follow decision tree above
5. Document 3-5 action items in sprint-N-review.md
6. Implement actions before Sprint N+1
7. Measure improvement in Sprint N+1 metrics
8. Repeat
```

**Expected Timeline to Significant Improvement:**
- Sprint 1: Baseline measurement
- Sprint 2: First optimizations applied, 10-15% reduction
- Sprint 3: Refined optimizations, 20-25% reduction
- Sprint 4+: Sustained 25-40% reduction from baseline

**Success Criteria:**
- Token usage trending downward sprint-over-sprint
- Quality failures eliminated (zero rework sprints)
- Action items from decision tree consistently implemented
- Cache hit rates > 60%
- Fix iterations < 2 per sprint
