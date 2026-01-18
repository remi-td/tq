# P000: Example Optimization Proposal

**Status**: Implemented ✅
**Created**: Sprint 0 (example)
**Implemented**: Sprint 0 (example)
**Impact Score**: 8,000

## Metrics
- **Tokens Saved**: 4,000 per occurrence
- **Frequency**: 2 times per sprint
- **Confidence**: High
- **Estimated Implementation Effort**: Small

## Problem

Agent repeatedly reads the same large specification file multiple times within a single invocation, wasting tokens.

**Evidence from Sprint X transcript (lines 123-456):**
```
Line 123: Read specifications.md (2,300 lines)
Line 456: Read specifications.md again (2,300 lines) - file already in context
Line 789: Read specifications.md third time (2,300 lines) - still in context
```

Total waste: ~6,900 lines × 15 tokens/line = ~103,500 tokens wasted

## Proposed Solution

Add explicit instruction to agent prompt reminding it not to re-read files already in context:

**File:** `.claude/subagents/example-agent.md`
**Location:** After line 15 (before main instructions)
**Change:** Add new section

```markdown
## Context Awareness

Files loaded at the start of your invocation remain in your context throughout the conversation. Do NOT re-read files that are already in your context unless:
1. You explicitly modified the file and need to see changes
2. User specifically requests a fresh read

**Common mistake to avoid:**
❌ Reading specifications.md multiple times in one invocation
✅ Reference your existing context of specifications.md
```

## Files Affected
- `.claude/subagents/example-agent.md` (line 16: add context awareness section)

## Validation Criteria

**Success indicators:**
- Next sprint transcript shows 0-1 reads of specifications.md (down from 3+)
- Token usage for example-agent decreases by ~100K tokens
- No quality degradation (agent still produces good output)

**How to measure:**
```bash
# Count reads of specifications.md in next sprint
jq -r 'select(.message.content) | .message.content[] |
  select(.type == "tool_use" and .name == "Read" and .input.file_path == "specifications.md")' \
  transcript.jsonl | wc -l

# Expected: 0-1 (down from 3+)
```

## Evidence from Transcripts

**Sprint X - example-agent transcript:**
- Line 123: First read of specifications.md (justified - needs initial context)
- Line 456: Second read of specifications.md (WASTEFUL - file already in context from line 123)
- Line 789: Third read of specifications.md (WASTEFUL - file still in context)

**Root cause:** Agent doesn't realize file is already in its context, reads redundantly.

## Implementation Notes

**Implemented:** Sprint 0
**Changes made:**
- Added "Context Awareness" section to `.claude/subagents/example-agent.md` at line 16
- Exact wording as proposed above
- Tested agent invocation to verify new section is loaded

**Deviations from plan:** None

## Validation Results

**Measured in Sprint Y:**
- example-agent read specifications.md 1 time (down from 3)
- Token reduction: 103,500 tokens saved (matched prediction)
- Quality: Output quality maintained, no regressions

**Conclusion:** ✅ Optimization successful, proposal validated

**Lessons learned:**
- Simple prompt additions can have large impact
- Agents benefit from explicit reminders about context awareness
- Validation is critical - measure actual impact

---

## Usage Note

This is an **example proposal** demonstrating the format. Real proposals should follow this structure with actual evidence from sprint transcripts.
