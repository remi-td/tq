# Honest Assessment Principles

This document establishes principles for honest sprint assessment and review, learned from the Sprint 29/30 pager crisis.

## The Problem: False Success Metrics

### Sprint 29 Assessment (What Was Written)

```
Overall Assessment: 9.5/10 (Excellent)
Test Pass Rate: 386/386 (100%)
Status: COMPLETE - ONE substantial feature delivered
v1.13.0 is production-ready
```

### Sprint 29 Reality (What Users Experienced)

- User: "this feature really doesn't exist!!!"
- User: "absolutely not working, same as before!!!"
- Feature: Completely broken, garbled output

### The Gap

A sprint rated 9.5/10 "Excellent" delivered a completely broken feature. The assessment was based on test metrics, not functionality.

## Assessment Principles

### Principle 1: Function Over Metrics

**Wrong approach:**
- "100% test pass rate = feature works"
- "Code compiles and tests pass = production-ready"
- "No clippy warnings = high quality"

**Correct approach:**
- "Does the feature actually work when a user tries it?"
- "Would I be comfortable demonstrating this to the user?"
- "If the user ran this right now, what would they experience?"

### Principle 2: User Experience is the Measure

**Assessment question:** "What does the USER see and experience?"

Not:
- What do the tests report?
- What does the code structure look like?
- How many tests passed?

But:
- Does the user see correct output?
- Can the user accomplish their task?
- Is the experience what was promised?

### Principle 3: Honest Failure Acknowledgment

When a feature does not work:

**Wrong:** "Partially implemented with minor issues"
**Correct:** "Feature does not work"

**Wrong:** "Needs additional testing"
**Correct:** "Feature broken, tests failed to detect it"

**Wrong:** "Technical challenges encountered"
**Correct:** "We delivered a broken feature"

### Principle 4: Rating Must Reflect Reality

| Scenario | Appropriate Rating |
|----------|-------------------|
| Feature works as specified | 8-10/10 |
| Feature works with minor issues | 6-7/10 |
| Feature partially works | 4-5/10 |
| Feature does not work | 1-3/10 |
| Feature broken despite tests passing | 1-2/10 (framework failure) |

**Sprint 29 should have been rated:** 2/10 (feature broken despite tests)

### Principle 5: Test Metrics are Inputs, Not Outcomes

Test pass rate informs assessment but does not determine it:

- 100% pass rate + working feature = success
- 100% pass rate + broken feature = **testing framework failure**
- 80% pass rate + working feature = acceptable (fix failing tests)
- 80% pass rate + broken feature = failure

The relevant metric is: **Does it work?**

## Sprint Review Rating Guidelines

### When to Assign High Ratings (8+)

- Feature works correctly from user perspective
- Manual validation confirms functionality
- Edge cases handled appropriately
- Documentation accurate
- No workarounds or disabled code

### When to Assign Medium Ratings (5-7)

- Feature works with known limitations
- Minor issues that don't block core functionality
- Documentation exists but may have gaps
- Some edge cases need attention

### When to Assign Low Ratings (1-4)

- Feature does not work as specified
- Core functionality broken
- Tests pass but functionality broken (indicates testing failure)
- Feature disabled by default
- User cannot accomplish intended task

### Automatic Low Rating Triggers

1. **Feature disabled by default** = Maximum 3/10
2. **User reports "not working"** = Maximum 4/10
3. **100% test pass but feature broken** = 2/10 (framework crisis)
4. **Same issue in consecutive sprints** = 1/10

## Retrospective Sprint Assessment

### Sprint 29: Horizontal Paging

**Original rating:** 9.5/10 (Excellent)

**Corrected rating:** 2/10 (Critical Failure)

**Rationale:**
- Feature completely broken despite 100% test pass rate
- User immediately reported non-functional
- Testing framework failed to validate actual rendering
- Assessment based on metrics, not functionality

### Sprint 30: Pager Refactor

**Original rating:** 2/10 (Critical Failure)

**Confirmed rating:** 2/10 (Critical Failure)

**Rationale:**
- Correct identification of root cause
- Sound architectural solution
- Implementation still broken
- Feature disabled rather than fixed
- $61.78 invested for negative value

## Going Forward

### Sprint Closure Checklist

Before assigning sprint rating:

1. **Functionality verified:** Have I manually confirmed the feature works?
2. **User perspective considered:** Would user be satisfied with this?
3. **Honesty check:** Am I rating based on what works, not what tests say?
4. **Trust calibration:** If I'm wrong, would user lose trust?

### Warning Signs of False Success

- High test pass rate but no manual verification
- "Tests pass" as the primary evidence
- "Code complete" without functionality demonstration
- Defensive language ("mostly works", "edge cases remain")
- **A test rated REQUIRED in the sprint strategy shows up as `skipped for reason: not authored` in the evidence** (Sprint 67 lesson — PTY tests for AC-1/AC-11 and ANSI-byte test for AC-8 were strategy-REQUIRED but went unwritten; the authoring was deprioritised, not technically blocked)
- **A test labeled `run and passed` covered a different code path than the AC it claims to prove** (Sprint 67 lesson — AC-7 horizontal-scroll was "proven" by a vertical-scroll test until QV self-critique caught the mislabel)

### The "REQUIRED-is-not-optional" Rule

When Phase 2 strategy classifies a test as **REQUIRED**, that classification is binding. It can only be satisfied by authoring the test, not by any of the following substitutions:

- Code inspection with a paragraph of prose
- A "structural verification" that greps for the right call in the source
- A different test that covers a related but not-quite-the-same code path
- A claim that the harness is not ready (if the harness IS ready — as the Sprint 66 tiered PTY harness was for Sprint 67 — deferring authoring is a choice, not a blocker)

If a REQUIRED test cannot be authored within the sprint, the evidence doc must:

1. Label the AC as `skipped for reason: REQUIRED test not authored — <explicit reason>` (not `run and passed`, not `skipped for reason: accepted fallback`).
2. Rate the gap at **MEDIUM severity** at minimum. Code inspection is supplementary evidence only; it does not downgrade a REQUIRED gap to LOW.
3. Add a P2 follow-up item to the sprint review with the exact test to author and a time estimate.

This rule exists because every time it has been relaxed (Sprint 65 AC-4..AC-9, Sprint 67 AC-1/AC-7/AC-8/AC-11), the next sprint inherits the gap as phantom coverage — the evidence record says "passed" or "accepted", the AC goes unexercised, and a bug in that code path can ship unnoticed.

### PTY Tests with Early-Return Guards

PTY tests often include early-return guards that bail out when infrastructure limitations are detected (e.g., reedline's `[6n` cursor-position queries going unanswered). When a guard fires, the test framework may report `1 passed` even though no assertions were exercised.

**Rule:** A PTY test that exits via an early-return guard **must be labeled `skipped for reason: <guard name> fired` in test evidence**, not `passed` or `run and passed`.

**Detection:** After any PTY test run, check the PTY dump (if produced) or test output for guard activation. Common indicators:
- Test completes in <5 seconds when normal execution takes 30-60 seconds
- PTY dump shows incomplete interaction (e.g., `[6n` sequence with no response)
- Test log mentions "guard fired" or "early return"

**Evidence Label:**
- **Wrong:** `run and passed` (when guard fired)
- **Right:** `skipped for reason: PTY cursor detection fired` with PTY dump reference

This rule prevents phantom coverage where a test "passes" without exercising the code path it claims to validate. The Sprint 68 TC104 case demonstrated that this pattern can escape detection until Phase 5 review.

### Per-AC Assertion Citation

An AC is proven by a *specific assertion on a specific code path*, not by a test function whose name happens to contain a related keyword. The test-evidence format must cite, for each AC, the exact assertion that exercises the code path the AC specifies — not just the test function name. If no such assertion exists in any authored test, the AC is `skipped for reason`, regardless of how many neighbouring tests passed.

Example (Sprint 67 AC-7 mislabel):
- **Wrong:** AC-7 (horizontal scroll to matched column) → `run and passed` via `submit_search_scrolls_to_first_match`.
  - Test does exist and passes. But its fixture has 3 cols, all visible — the `col_offset` branch in `scroll_to_match_index` never fires. The AC is untouched.
- **Right:** AC-7 → `skipped for reason: no multi-column fixture test authored; horizontal col_offset branch not exercised by any passing test`.

### Building Trust Through Honesty

User trust is built by:
- Acknowledging failures honestly
- Not claiming success for broken features
- Prioritizing user experience over metrics
- Demonstrating self-awareness about limitations

Trust is destroyed by:
- Claiming success for broken features
- Ratings that don't match reality
- Repeated failures without acknowledgment
- Prioritizing metrics over outcomes

## Conclusion

The Sprint 29/30 pager crisis demonstrated that:

1. Test pass rates can be completely disconnected from feature functionality
2. Sprint assessments based on metrics rather than reality destroy user trust
3. Honest acknowledgment of failure is necessary for improvement

**New standard:** Sprint ratings must reflect actual user experience, not test metrics. If a feature doesn't work, the sprint failed, regardless of test pass rate.
