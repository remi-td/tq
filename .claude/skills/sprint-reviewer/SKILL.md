---
name: sprint-reviewer
description: Coordinates comprehensive sprint retrospectives with specialized agents, producing a single consolidated review document with metrics analysis, lessons learned, and agent optimization recommendations. Use at the end of each sprint when all features are complete and tested.
---

# Sprint Reviewer

## Overview

This skill guides the tq-project-manager agent through conducting a comprehensive sprint retrospective by coordinating reviews from technical, quality, and UX specialists, then consolidating findings into a single actionable document with emphasis on cost metrics and continuous improvement.

## When to Use

Use this skill when:
- A sprint has been completed (all features implemented and tested)
- User explicitly requests a sprint review or retrospective
- You need to document sprint outcomes and prepare for the next sprint
- You need to analyze agent performance and optimization opportunities

## Prerequisites

Before starting the review:
1. All sprint features must be implemented
2. All tests must be passing
3. Version should be updated in Cargo.toml
4. Git commits should be complete

## Sprint Review Process

### Step 1: Preparation and Context Gathering

First, gather essential sprint context:

1. **Read the roadmap** to identify:
   - Current sprint number and objectives
   - Features that were planned vs delivered
   - Previous sprint metrics for comparison

2. **Check git history** for the sprint:
   ```bash
   git log --oneline --since="[sprint-start-date]"
   git diff [previous-sprint-tag]..HEAD --stat
   ```

3. **Collect test metrics**:
   ```bash
   cargo test --all-targets --all-features
   ```

4. **Review previous sprint retrospective** (if exists) in docs/builder/sprints/
   - Note action items and check if addressed
   - Review previous agent optimization recommendations
   - Identify patterns or recurring issues

### Step 2: Launch Parallel Agent Reviews

Launch THREE agents in parallel using a SINGLE message with multiple Task tool calls:

#### Agent 1: rust-teradata-architect (Technical Review)

```
Task tool:
- subagent_type: "rust-teradata-architect"
- description: "Sprint [N] technical review"
- prompt: "Conduct a comprehensive technical review of Sprint [N].

          Review scope:
          1. Implementation approach and architectural decisions
          2. Code quality, modularity, and maintainability
          3. Technical challenges encountered and solutions
          4. Technical debt assessment (introduced vs resolved)
          5. Adherence to rust-architecture.md and design guidelines
          6. Lessons learned for future sprints

          Analyze:
          - New files created and their purpose
          - Modified files and scope of changes
          - Library dependencies added
          - Integration points with existing code

          Provide specific recommendations for:
          - Code improvements
          - Architectural refinements
          - rust-coder skill enhancements
          - rust-teradata-architect agent prompt improvements"
```

#### Agent 2: quality-validator (Quality Review)

```
Task tool:
- subagent_type: "quality-validator"
- description: "Sprint [N] quality review"
- prompt: "Conduct a comprehensive quality assurance review of Sprint [N].

          Review scope:
          1. Test coverage analysis (new tests vs total tests)
          2. Test pass rate and any failures or warnings
          3. Testing methodology effectiveness
          4. Manual testing scenarios and results
          5. Regression testing results
          6. Quality metrics trends

          Analyze:
          - Unit test coverage for new features
          - Integration test coverage
          - Edge cases and error scenarios tested
          - Test execution time

          Provide specific recommendations for:
          - Testing approach improvements
          - testing-guidelines.md updates
          - quality-validator agent prompt enhancements
          - Automated testing infrastructure"
```

#### Agent 3: cli-ux-designer (UX Review)

```
Task tool:
- subagent_type: "cli-ux-designer"
- description: "Sprint [N] UX review"
- prompt: "Conduct a comprehensive UX review of Sprint [N].

          Review scope:
          1. Feature usability and user experience
          2. CLI design consistency and conventions
          3. Flag naming and configuration options
          4. Help text and documentation quality
          5. Error messages and user feedback
          6. Comparison with industry standards

          Analyze:
          - User interaction patterns
          - Discoverability of features
          - Accessibility considerations
          - Default behaviors and their appropriateness

          Provide specific recommendations for:
          - UX improvements for next sprint
          - specifications.md and detailed-specifications/ clarifications
          - cli-ux-designer agent enhancements
          - Documentation updates needed"
```

### Step 3: Analyze Token and Time Metrics

While agents are working, prepare cost analysis:

1. **Check task output files** for token usage from completed agent runs:
   - Look in /private/tmp/claude/-Users-remi-turpaud-Code-genAI-tq/tasks/ for agent output files
   - Each agent run logs token usage in its output

2. **Calculate sprint totals**:
   - Total input tokens (including cache hits)
   - Total output tokens
   - Approximate cost (based on Claude pricing)
   - Time spent per agent
   - Average tokens per feature delivered

3. **Identify optimization opportunities**:
   - Agents with highest token consumption
   - Redundant context loading
   - Opportunities for skill refinement to reduce tokens

### Step 4: Compile Comprehensive Review Document

Create a SINGLE file: `docs/builder/sprints/sprint-[N]-review.md`

**Document Structure:**

```markdown
# Sprint [N] Review: [Sprint Name]

**Sprint Duration:** [Start Date] - [End Date]
**Status:** COMPLETED
**Version Released:** v[X.Y.Z]

---

## 1. Executive Summary

**Overall Assessment:** [Rating]/10 - [Excellent/Good/Fair]

**Key Achievements:**
- [Achievement 1]
- [Achievement 2]

**Sprint Health:** [Summary statement]

---

## 2. Sprint Metrics

### 2.1 Feature Delivery

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Features Planned | [N] | [N] | ✅/⚠️/❌ |
| Features Delivered | - | [N] | [%] |
| Tests Added | - | [N] | - |
| Total Tests | - | [N] | - |

### 2.2 Code Metrics

| Metric | Value |
|--------|-------|
| Files Added | [N] |
| Files Modified | [N] |
| Lines of Code Added | ~[N] |
| Dependencies Added | [N] |

### 2.3 Cost and Efficiency Metrics

| Agent | Tasks | Input Tokens | Output Tokens | Time Spent | Avg Tokens/Task |
|-------|-------|--------------|---------------|------------|-----------------|
| rust-teradata-architect | [N] | [N] | [N] | [time] | [N] |
| quality-validator | [N] | [N] | [N] | [time] | [N] |
| cli-ux-designer | [N] | [N] | [N] | [time] | [N] |
| tq-project-manager | [N] | [N] | [N] | [time] | [N] |
| **TOTAL** | [N] | [N] | [N] | [time] | [N] |

**Cost Analysis:**
- Estimated total cost: $[X.XX]
- Cost per feature: $[X.XX]
- Token efficiency: [X] tokens per feature
- Most expensive agent: [agent-name] ([%] of total)

**Optimization Opportunities:**
- [Specific recommendation 1]
- [Specific recommendation 2]

### 2.4 Quality Metrics

| Metric | Value | Trend |
|--------|-------|-------|
| Test Pass Rate | [%] | ↑/→/↓ |
| Code Coverage | [%] | ↑/→/↓ |
| Technical Debt | [Zero/Low/Medium] | ↓/→/↑ |
| Regressions | [N] | - |

---

## 3. Technical Review

[Consolidate rust-teradata-architect review]

**Implementation Approach:**
- [Summary of technical decisions]

**Challenges Encountered:**
1. [Challenge 1 and solution]
2. [Challenge 2 and solution]

**Code Quality Assessment:** [Rating]/5
- Architecture: [Rating]/5
- Maintainability: [Rating]/5
- Test Coverage: [Rating]/5

**Technical Debt:**
- Introduced: [List or None]
- Resolved: [List or None]
- Net Change: [Positive/Neutral/Negative]

---

## 4. Quality Assurance Review

[Consolidate quality-validator review]

**Test Results:**
- Unit Tests: [N] passed, [N] failed
- Integration Tests: [N] passed, [N] failed
- Doc Tests: [N] passed, [N] failed

**Testing Effectiveness:**
- [Assessment of testing approach]

**Issues Found:**
- Critical: [N]
- Major: [N]
- Minor: [N]

---

## 5. User Experience Review

[Consolidate cli-ux-designer review]

**UX Rating:** [N]/10

**Feature Usability:**
- [Feature 1]: [Rating] - [Brief assessment]
- [Feature 2]: [Rating] - [Brief assessment]

**CLI Design Quality:**
- Flag naming: [Rating]
- Default behaviors: [Rating]
- Help text: [Rating]

**Industry Comparison:**
- [How tq compares to competitors]

---

## 6. Sprint Comparison

| Metric | Sprint [N-1] | Sprint [N] | Change |
|--------|--------------|------------|--------|
| Features Delivered | [N] | [N] | [%] |
| Tests Added | [N] | [N] | [%] |
| Total Tokens | [N] | [N] | [%] |
| Total Cost | $[X] | $[X] | [%] |

---

## 7. Lessons Learned

### 7.1 What Worked Well

1. [Lesson 1]
2. [Lesson 2]

### 7.2 What Could Be Improved

1. [Improvement 1]
2. [Improvement 2]

### 7.3 Surprises and Discoveries

- [Unexpected finding 1]
- [Unexpected finding 2]

---

## 8. Recommendations

### 8.1 For Sprint [N+1] (Next Sprint)

**Feature Priorities:**
1. [Priority 1 with rationale]
2. [Priority 2 with rationale]

**Technical Priorities:**
1. [Technical item 1]
2. [Technical item 2]

**Quality Priorities:**
1. [Quality item 1]
2. [Quality item 2]

### 8.2 Agent Optimization Recommendations

**High Priority:**

1. **rust-coder skill improvements:**
   - [Specific skill enhancement needed]
   - [Rationale and expected impact]

2. **Agent prompt refinements:**
   - Agent: [agent-name]
   - Change: [Specific prompt improvement]
   - Reason: [Why this will help]

3. **Documentation updates:**
   - File: [documentation-file]
   - Update: [What to add/change]
   - Benefit: [Expected improvement]

**Medium Priority:**

4. **CLAUDE.md updates:**
   - [Project-level guidance to add]

5. **Testing guidelines enhancements:**
   - [Testing methodology improvement]

**Low Priority:**

6. **Infrastructure improvements:**
   - [Tool or process improvement]

---

## 9. Action Items

| Action | Owner | Priority | Status |
|--------|-------|----------|--------|
| [Action 1] | [Agent] | High | To Do |
| [Action 2] | [Agent] | Medium | To Do |

---

## 10. Previous Sprint Action Items Review

[Check previous sprint review's action items]

| Action | Status | Notes |
|--------|--------|-------|
| [Previous action 1] | ✅/⚠️/❌ | [Outcome] |

---

## Appendices

### A. Detailed Feature Analysis

[Feature-by-feature breakdown with implementation details]

### B. Test Coverage Details

[Detailed test analysis if needed]

### C. Token Usage Breakdown

[Detailed token analysis by agent task if valuable]

---

**Review Completed:** [Date]
**Next Sprint:** Sprint [N+1] - [Name]
**Next Sprint Focus:** [Brief description]
```

### Step 5: Update Roadmap

Update `docs/builder/user/roadmap.md`:

1. **Mark sprint as COMPLETED** in the sprint section
2. **Add retrospective summary** with link to full review:
   ```markdown
   ### Sprint [N] Retrospective Summary
   - **Features Delivered:** [N]/[N] ([%])
   - **Quality Assessment:** [Rating]/5
   - **Token Efficiency:** [N] tokens per feature
   - **Key Achievement:** [One-sentence highlight]
   - **Full Review:** See [Sprint [N] Review](../sprints/sprint-[N]-review.md)
   ```

3. **Update "Current Sprint" section** to next sprint
4. **Update metrics** in technical notes if relevant

### Step 6: Create Summary for User

Provide a concise summary highlighting:
- Sprint completion status
- Key metrics (features, tests, tokens, cost)
- Major achievements
- Critical recommendations
- Agent optimization opportunities identified
- Link to full review document

## Guidelines

### Token Efficiency Focus

The cost metrics section is CRITICAL. Always include:
- Token usage per agent with breakdown
- Cost calculation (approximate based on current pricing)
- Identification of most expensive operations
- Specific recommendations to reduce token consumption
- Comparison to previous sprints

### Agent Optimization Emphasis

Every review MUST include actionable agent optimization recommendations:
- **Skill enhancements**: What should be added to rust-coder, teradata-rust, cli-designer, etc.
- **Prompt improvements**: How agent prompts can be more specific to reduce iteration
- **Documentation updates**: What should be added to CLAUDE.md, specifications, guidelines
- **Process improvements**: How the development workflow can be more efficient

### Single File Principle

**CRITICAL**: Create ONE review file per sprint in `docs/builder/sprints/`, not multiple files per agent.

File naming: `sprint-[N]-review.md` where N is the sprint number.

### Previous Review Analysis

ALWAYS read the previous sprint's review before creating the new one:
- Check if action items were addressed
- Review agent optimization recommendations
- Identify patterns across sprints
- Track improvements or regressions

### Comparison Analysis

Always compare current sprint to previous sprint:
- Feature delivery rate
- Test additions
- Token usage trends
- Cost efficiency
- Quality metrics

### Be Specific in Recommendations

Avoid vague recommendations like "improve documentation." Instead:
- ❌ "Improve rust-coder skill"
- ✅ "Add section to rust-coder skill on Teradata-specific error handling patterns (lines 234-245 of src/db/mod.rs show the pattern)"

### Cost-Benefit Analysis

For each recommendation, explain:
- What problem it solves
- Expected token/cost savings
- Implementation effort
- Priority level

## Anti-Patterns to Avoid

**Don't:**
- Create multiple review files per sprint (UX review, technical review, etc.)
- Skip token/cost analysis
- Provide generic agent optimization recommendations
- Forget to compare to previous sprint
- Ignore previous sprint action items
- Create reviews longer than 600 lines (consolidate!)
- Use absolute file paths (always relative to project root)

**Do:**
- Consolidate all reviews into one file
- Emphasize cost metrics and optimization
- Provide specific, actionable recommendations with file/line references
- Track trends across sprints
- Keep reviews focused and scannable
- Link to relevant documentation and code

## Success Criteria

A successful sprint review:
- ✅ Single consolidated file in docs/builder/sprints/
- ✅ Comprehensive token and cost analysis
- ✅ Specific agent optimization recommendations with rationale
- ✅ Comparison to previous sprint with trends
- ✅ Previous action items tracked and reviewed
- ✅ Clear recommendations for next sprint
- ✅ Roadmap updated with retrospective summary
- ✅ Under 600 lines total length

## Example Agent Optimization Recommendation

**Good Example:**

```markdown
### High Priority Optimization #1: Enhance rust-coder skill with Teradata SQL patterns

**Problem:** rust-teradata-architect spent 45% more tokens (12,450 vs 8,567 in Sprint 4)
on implementing SQL result formatting due to lack of Teradata-specific patterns in
rust-coder skill.

**Solution:** Add section to .claude/skills/rust-coder/SKILL.md:
- Teradata metadata format handling (map-of-arrays vs array-of-objects)
- Column type mapping from Teradata native types to Rust types
- Example from src/db/result.rs:152-178 showing correct pattern

**Expected Impact:**
- Reduce iteration cycles by ~2-3 (saving ~4,000 tokens per Teradata integration task)
- Estimated savings: 15-20% on database integration features

**Implementation Effort:** Low (1-2 hours to document pattern)

**Priority:** High (affects all future database features)
```

**Poor Example:**

```markdown
### Optimization: Improve rust-coder skill

The skill should be better at Rust code. Add more examples.
```

## Notes

- Run this review process at the END of each sprint before starting the next
- The project manager should schedule reviews as part of sprint closure
- Reviews are living documents - update if significant insights emerge later
- Share review highlights with the user to maintain transparency
