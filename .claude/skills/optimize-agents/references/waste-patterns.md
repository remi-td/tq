# Token Waste Patterns Catalog

This reference provides a systematic catalog of common token waste patterns found in agent transcripts, organized for efficient analysis.

## How to Use This Catalog

**Analysis Approach:**
1. Read the transcript completely
2. Identify the **highest token operations** (largest file reads, most frequent tool calls)
3. For each high-token operation, check if it matches a waste pattern below
4. If match found, apply the documented solution
5. Focus on top 3-5 operations consuming most tokens - don't try to optimize everything

**Key Principle:** Analyze EVERY transcript regardless of total token count. Even a 30K transcript can have inefficiencies worth fixing.

---

## Pattern Catalog

### Pattern 1: Redundant File Reads

**Symptoms:**
- Same file read 3+ times in single agent invocation
- File already in context but read again later
- Reading entire file when partial read would suffice

**Detection:**
```bash
# Count reads per file in transcript
jq -r 'select(.message.content) | .message.content[] |
  select(.type == "tool_use" and .name == "Read") |
  .input.file_path' transcript.jsonl | sort | uniq -c | sort -rn
```

**Root Causes:**
- Agent doesn't realize file is already in context
- Missing architecture overview in documentation
- Exploring without clear direction
- File modified then re-read

**Solutions:**
- Add file/module overviews to architecture docs
- Update agent prompt: "Don't re-read files already in context"
- Pre-specify which files to modify in agent instructions
- Use Edit tool's preview feature instead of Read after editing

**Typical Savings:** 3-10K tokens per occurrence

**Example Proposal:**
```markdown
## Reduce Redundant Reads of metadata.rs

**Tokens Saved**: 4,000 per occurrence
**Frequency**: Once per sprint (rust-teradata-architect)
**Confidence**: High
**Implementation Effort**: Small

### Problem
Agent reads src/db/metadata.rs 5 times during feature implementation (lines 45, 123, 456, 789, 1012 in transcript).

### Proposed Solution
Add "Metadata System Architecture" section to rust-architecture.md explaining:
- MetadataCache design and purpose
- How components interact
- Common extension patterns

### Files Affected
- docs/builder/rust-architecture.md (add section 6.3)
```

---

### Pattern 2: Unnecessary Exploration

**Symptoms:**
- 20+ Read operations before first Edit
- Reading files unrelated to the task
- Grep searches that don't inform the work
- Exploring directory structures

**Detection:**
- Count Read operations before first Edit/Write
- Identify files read but never modified
- Check if exploration matched task scope

**Root Causes:**
- Agent instructions too vague
- "Files to modify" not specified upfront
- Missing architectural guidance
- Agent unsure where code lives

**Solutions:**
- Add "Files Involved" section to sprint planning
- Pre-specify target files in agent invocation
- Add module location guide to architecture docs
- Use Glob strategically before reading

**Typical Savings:** 5-15K tokens per sprint

---

### Pattern 3: Verbose Output

**Symptoms:**
- Agent responses > 1000 lines
- Full test logs pasted instead of summarized
- Repeating information already in context
- Overly detailed explanations

**Detection:**
- Measure length of agent text responses
- Check for repeated content
- Look for summarizable data presented in full

**Root Causes:**
- Agent prompt doesn't emphasize conciseness
- Trying to be thorough by being verbose
- Not using structured formats (tables, YAML)
- Including full tool output instead of summaries

**Solutions:**
- Add to agent prompt: "Be concise, user sees full context"
- Guidelines: "Summarize test results, don't paste full logs"
- Use structured formats (tables for comparisons, YAML for specs)
- Reference line numbers instead of quoting large passages

**Typical Savings:** 3-8K tokens per agent invocation

---

### Pattern 4: Rework Loops

**Symptoms:**
- Agent makes changes then reverts them
- Multiple Edit operations on same code block
- Trying approach A, failing, trying approach B
- Test failures requiring multiple fix iterations

**Detection:**
- Count Edit operations per file
- Identify failed tool uses followed by retries
- Track test-fix-test cycles

**Root Causes:**
- Insufficient design phase
- Missing test-before-edit practice
- Unclear requirements
- Agent making assumptions instead of asking

**Solutions:**
- Strengthen Phase 2 design specifications
- Agent must understand code before editing
- Add "run unit tests before marking complete" to workflow
- Encourage clarifying questions upfront

**Typical Savings:** 10-25K tokens per occurrence

---

### Pattern 5: Missing Documentation

**Symptoms:**
- Agent asks questions that docs should answer
- Multiple agents asking same questions
- Rediscovering information each sprint
- Lengthy exploration to understand system

**Detection:**
- Search for question patterns in transcripts
- Identify repeated questions across sprints
- Note topics agents struggle with

**Root Causes:**
- Documentation gaps
- Information exists but not discoverable
- Architecture not explained
- Common patterns not documented

**Solutions:**
- Add missing sections to architecture/testing docs
- Create "Common Patterns" sections
- Add cross-references between related docs
- Document architectural decisions

**Typical Savings:** 5-15K tokens per sprint (cumulative)

---

### Pattern 6: Unclear Instructions

**Symptoms:**
- Agent asks clarifying questions
- Multiple back-and-forth exchanges
- Agent makes incorrect assumptions
- Work doesn't match expectations

**Detection:**
- Count question-answer exchanges
- Identify ambiguous requirements
- Note where assumptions were wrong

**Root Causes:**
- Agent system prompt too vague
- Sprint planning lacks detail
- Requirements not explicit
- Context not provided upfront

**Solutions:**
- Add specificity to agent prompts
- Include "Pre-answered Questions" section
- Provide examples in instructions
- Reference relevant docs explicitly

**Typical Savings:** 3-10K tokens per sprint

---

### Pattern 7: Tool Misuse

**Symptoms:**
- Using expensive tools when simpler ones work
- Bash for tasks that dedicated tools handle
- Grep entire codebase when Glob would work
- Multiple small Reads instead of one large Read

**Detection:**
- Review tool usage patterns
- Identify inefficient tool choices
- Check for repeated small operations

**Root Causes:**
- Agent unaware of better tool options
- Habits from other contexts
- Missing tool usage guidelines
- Optimization not prioritized

**Solutions:**
- Add tool selection guidelines to agent prompts
- Prefer: Glob > Grep > Read > Bash for file ops
- Use Read with large ranges instead of multiple small reads
- Document tool efficiency patterns

**Typical Savings:** 2-5K tokens per sprint

---

### Pattern 8: Context Bloat

**Symptoms:**
- Loading large files completely when sections suffice
- Reading 1000+ line specs repeatedly
- Including unnecessary context
- Not using targeted reads

**Detection:**
- Identify large file reads (>500 lines)
- Check if entire file was needed
- Look for repeated large reads

**Root Causes:**
- Agent doesn't know which sections are relevant
- Documentation too long/unfocused
- Reading convenience over efficiency
- Lack of indexing/navigation

**Solutions:**
- Add section numbers to long docs
- Create summaries for large docs
- Use Read with offset/limit for targeted access
- Split large documents by topic

**Typical Savings:** 5-10K tokens per sprint

---

### Pattern 9: Quality Failures (Critical)

**Symptoms:**
- Tests pass but features don't work
- Manual tests skipped or not documented
- Database unavailable during testing
- Edge cases not tested
- User reports bugs after "completion"

**Detection:**
- Check sprint review for quality issues
- Look for "manual tests: NO" or missing test logs
- Search for rework in subsequent sprints
- Database connectivity issues mentioned

**Root Causes:**
- Phase 3.5 (Database Check) skipped
- Manual testing not enforced
- Over-reliance on unit tests
- Edge cases not considered

**Solutions (CRITICAL - P0):**
- Make Phase 3.5 database check MANDATORY - STOP sprint if fails
- Require manual test execution and documentation
- Update validation checklist to block completion without manual tests
- Add edge case testing guidelines

**Typical Savings:** 20-40K tokens per quality failure avoided (prevents rework)

---

### Pattern 10: Low Cache Hit Rate

**Symptoms:**
- Cache hit rate < 40%
- Cache created but rarely reused
- Prompts changing every invocation
- Volatile system context

**Detection:**
```bash
# Extract cache rates from metrics
grep "Cache Hit Rate" sprint-N-metrics.md
```

**Root Causes:**
- Agent prompts change every sprint
- Specifications rewritten frequently
- Variable content in cached region
- System prompts not stabilized

**Solutions:**
- Separate stable context from variable context
- Create stable reference docs (don't modify)
- Use consistent prompt structure
- Reference stable docs instead of regenerating content

**Typical Savings:** 30-50% cost reduction via caching (10x price difference)

---

### Pattern 11: Sequential When Parallel Possible

**Symptoms:**
- Agents run one after another
- Phase 2: cli-ux-designer finishes, then rust-teradata-architect starts
- Phase 3: Implementation finishes, then quality-validator starts
- Timeline shows sequential execution

**Detection:**
- Review timestamps in transcripts
- Check if agents could have run simultaneously
- Note dependencies that require sequencing

**Root Causes:**
- Sprint coordinator doesn't launch in parallel
- Not using multiple Task invocations in one message
- Misunderstanding of parallelism capability
- Over-cautious workflow

**Solutions:**
- Update sprint-coordinator: "Launch agents in parallel"
- Document: "Use multiple Task blocks in single message"
- Identify truly independent phases
- Add parallelism checklist

**Typical Savings:** No direct token savings, but 30-50% faster sprint completion

---

### Pattern 12: Multiple Fix Iterations

**Symptoms:**
- Quality-validator runs 3+ times
- Pattern: Test → Fix 2 bugs → Test → Fix 1 bug → Test
- Each iteration only fixes subset of issues
- Incremental fixes instead of batch fixes

**Detection:**
- Count quality-validator invocations
- Track test pass rate per iteration
- Note if same tests fail multiple times

**Root Causes:**
- Incomplete fix implementation
- Not analyzing all failures together
- Rushing to "try something"
- Insufficient root cause analysis

**Solutions:**
- If >5 failures, require batch fix by component
- rust-teradata-architect must run unit tests before completion
- Add "analyze all failures together" to workflow
- Strengthen Phase 2 design to prevent ambiguity

**Typical Savings:** 10-20K tokens per sprint

---

## Analysis Methodology

### Step 1: Identify High-Token Operations

For each transcript, find the operations consuming the most tokens:

```bash
# Top file reads by size
jq -r 'select(.message.content) | .message.content[] |
  select(.type == "tool_use" and .name == "Read") |
  "\(.input.file_path)"' transcript.jsonl | sort | uniq -c | sort -rn | head -10

# Tool usage frequency
jq -r 'select(.message.content) | .message.content[] |
  select(.type == "tool_use") | .name' transcript.jsonl | sort | uniq -c | sort -rn

# Verbose responses (count text length)
jq -r 'select(.message.role == "assistant") | .message.content[] |
  select(.type == "text") | .text | length' transcript.jsonl | sort -rn | head -10
```

### Step 2: Match to Patterns

For each high-token operation:
1. Check if it matches a pattern from catalog above
2. Verify the pattern is actually wasteful (not necessary work)
3. Identify root cause using pattern's diagnostic questions
4. Design solution using pattern's documented approaches

### Step 3: Quantify Impact

Calculate tokens saved:
- **File reads**: file size (lines × 15 tokens/line) × occurrences avoided
- **Verbose output**: current length - target length
- **Rework**: total rework tokens if pattern eliminated
- **Cache improvement**: cached tokens × 0.9 (90% savings on cached tokens)

### Step 4: Generate Proposal

Use the template from optimization-analyzer agent prompt:
- **Tokens Saved**: Calculated above
- **Frequency**: Based on observed pattern
- **Confidence**: High (proven), Medium (likely), Low (speculative)
- **Implementation Effort**: Small (<50 lines), Medium (50-200 lines), Large (>200 lines)
- **Problem**: Evidence from transcript with line numbers
- **Proposed Solution**: Specific, actionable changes
- **Files Affected**: Exact files and changes needed
- **Validation Criteria**: How to verify it worked

---

## Prioritization Framework

After identifying all patterns in a transcript, prioritize proposals by impact score:

```
Impact Score = (Tokens Saved × Frequency Weight × Confidence Weight) / Implementation Effort

Frequency Weights:
- Multiple times per sprint: 10
- Once per sprint: 3
- Once per feature: 5
- Rare: 1

Confidence Weights:
- High: 1.0 (clear cause-effect, proven pattern)
- Medium: 0.6 (likely to help, needs validation)
- Low: 0.3 (speculative, uncertain)

Implementation Effort:
- Small: 1 (<50 lines, single file)
- Medium: 3 (50-200 lines, multiple files)
- Large: 5 (>200 lines, architecture change)
```

**Focus on:**
1. High impact score (>500)
2. High confidence + small effort (quick wins)
3. Quality failure prevention (P0 regardless of tokens)

---

## Success Patterns

When a sprint shows LOW token usage with HIGH quality, document what worked:

### Pattern: Efficient Implementation

**Characteristics:**
- Agent made <15 Read operations
- 0-1 fix iterations
- <3K tokens in agent responses
- Manual tests executed successfully

**What Enabled This:**
- Clear sprint planning with files specified
- Good architecture documentation
- Well-defined requirements
- Appropriate scope

**How to Repeat:**
- Document specific docs that helped
- Note which agent prompt sections were effective
- Identify workflow decisions that saved tokens
- Add to best practices

---

## Common Anti-Patterns to Avoid

**Over-Optimization:**
- Don't optimize operations that only happen once
- Don't add complexity to save <1K tokens
- Don't break workflows to achieve metrics

**Premature Optimization:**
- Need at least 2 sprints to identify real patterns
- Don't optimize based on single occurrence
- Validate root causes before implementing solutions

**Metric Gaming:**
- Don't reduce scope to reduce tokens
- Don't skip necessary work to hit targets
- Quality > token count always

---

## Validation

After implementing optimizations, validate in next sprint:

**Success Criteria:**
- Token usage decreased as predicted
- Quality maintained or improved
- No new problems introduced
- Patterns confirmed eliminated

**If Optimization Failed:**
- Investigate why prediction was wrong
- Document lessons learned
- Refine pattern understanding
- Try different approach

**If Optimization Succeeded:**
- Document success in sprint review
- Add to standard practices
- Consider applying to other agents
- Update this catalog with refined solution
