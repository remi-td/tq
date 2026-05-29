# Sprint 69 Framework Optimization Plan

**Generated:** 2026-05-29
**Based on:** Sprint 67-69 metrics and retro analysis
**Total Expected Impact:** ~$5-8 per sprint reduction, improved quality gates

---

## Summary of Sprint 69 Insights

| Insight | Pattern Match | Root Cause |
|---------|---------------|------------|
| Session-cumulative metrics hard to read | N/A (tooling gap) | collect-metrics sums all agents in session |
| PTY guard heuristic too narrow | Pattern 9b (PTY Phantom Pass) | Grep-based detection misses non-standard guards |
| Retro found spec/code divergence + 5 missing tests | Pattern 4 (Rework Loops) | Phase 2 spec review lacks implementation cross-check |
| PTY fix rewrite cycle ($21 vs $15 expected) | Pattern 4 (Rework Loops) | First implementation approach failed |

---

## P0 - Critical (Implement Now)

### Action 1: Replace PTY Guard Heuristics with Hard .expect() Rule

**Evidence:**
- Sprint 68: TC104 "PASSED" when early-return guard fired (caught in Phase 5)
- Sprint 69: TC104 pager-activation guard converted to `.expect()` as retro fix #4
- Pattern: Grep-based detection for "cursor position" text missed the pager-activation guard

**Root Cause:**
PTY tests have multiple potential guards (cursor detection, pager activation, etc.). Heuristic grep detection will always miss some. The Sprint 69 fix — converting guards to hard `.expect()` failures — is the correct solution: make silent bypasses impossible.

**Solution:**
Codify the rule that new PTY tests must use `.expect()` (or equivalent hard failure) rather than early-return guards. Existing tests with early-return guards should be converted.

**Implementation:**

**File:** `docs/testing/honest-assessment.md`
**Action:** Add new section after "PTY Tests with Early-Return Guards"

```markdown
### Hard-Failure Pattern for PTY Tests

**Rule:** New PTY tests MUST NOT use early-return guards that exit silently on infrastructure issues. Instead, use hard `.expect()` failures that make the test fail explicitly.

**Wrong (early-return guard):**
```rust
if pager_activation_result.is_err() {
    return; // Silent pass, no assertions executed
}
```

**Right (hard failure):**
```rust
pager_activation_result.expect("Pager must activate for this test");
```

**Rationale:** Early-return guards let tests report "PASSED" without exercising assertions. Even with perfect detection heuristics, this pattern creates phantom coverage. Hard failures are unambiguous — the test either exercises its assertions or explicitly fails.

**Migration:** Existing PTY tests with early-return guards should be converted to hard `.expect()` as they are touched. TC104's Sprint 69 fix demonstrates the pattern.
```

**Expected Impact:**
- Eliminates the "PTY phantom pass" pattern entirely
- Saves 10-20K tokens per sprint in Phase 5 detection and retro fixes
- Prevents quality regressions from shipping

**Effort:** 15 minutes

---

### Action 2: Phase 2 Spec Review Must Include Implementation Cross-Check

**Evidence:**
- Sprint 69: Retro found `search_seg` truncation gap — spec (009.4) required truncation but code didn't implement it
- Sprint 69: 5 of 7 unit tests in TC108 not implemented
- Pattern: Spec/code divergence discovered in Phase 5, forcing retro fixes

**Root Cause:**
Phase 2 spec review focuses on spec completeness but doesn't cross-check whether existing implementation matches spec. Similarly, test strategy counts tests to author but doesn't verify all planned tests have numbers.

**Solution:**
Add cross-check steps to Phase 2 process.

**Implementation:**

**File:** `.claude/skills/sprint-coordinator/process/phase2-design.md`
**Action:** Add new section after design synthesis

```markdown
### Step 4: Implementation Cross-Check

Before proceeding to Phase 3, verify:

1. **Spec-Code Alignment:** For each requirement updated or added in this sprint:
   - Grep the codebase for the relevant function/feature
   - Verify existing code matches the spec
   - Flag divergences for Phase 3 architect attention
   
2. **Test Enumeration:** Quality-validator's test strategy must:
   - Number all planned tests (e.g., "TC108-U01..U07")
   - Count total: "7 tests planned"
   - Phase 3 validation will check all numbered tests are implemented

**Why this matters:** Sprint 69 retro found spec 009.4 required truncation that code didn't implement. Early detection in Phase 2 would have prevented 5 retro fixes.
```

**Expected Impact:**
- Catches spec/code divergence before Phase 3 implementation
- Prevents missing test scenarios (Sprint 69: 5 of 7 tests missing)
- Saves 5-10K tokens in retro fix iterations

**Effort:** 30 minutes

---

## P1 - High Priority (Implement ASAP)

### Action 3: Add Sprint Boundary Markers to Metrics Collection

**Evidence:**
- Sprint 69: Session c68a1c89 ran Sprint 68 review ($12.23) then Sprint 69 ($20.95)
- Metrics are session-cumulative; Sprint 69 cost derived as delta from Sprint 68 baseline
- Sprint 68-metrics.md: "Grand total: 23,078,954 tokens"
- Sprint 69-metrics.md: "Grand total: 63,653,746 tokens" (cumulative, not sprint-only)

**Root Cause:**
collect-metrics script sums all subagents in a session. When multiple sprints run in one session, the later sprint's metrics include the earlier sprint's work.

**Solution:**
Add optional time-range filtering to collect-metrics, or record sprint boundaries.

**Implementation:**

**File:** `.claude/skills/collect-metrics/scripts/extract-sprint-metrics.sh`
**Action:** Add `--after` flag to filter by timestamp

```bash
# New usage:
# extract-sprint-metrics.sh <sprint-number> [--after <timestamp>]
#
# Example: Extract Sprint 69, filtering agents created after Sprint 68 baseline
# ./extract-sprint-metrics.sh 69 --after "2026-05-29T10:00:00"
```

**Alternative (simpler):**
Add a note to the metrics template explaining session-cumulative behavior and how to compute deltas.

**File:** `.claude/skills/collect-metrics/references/metrics-template.md`
**Action:** Add section

```markdown
## Multi-Sprint Sessions

When multiple sprints run in a single session, metrics are cumulative. To compute per-sprint cost:

1. Find the previous sprint's "Grand Total" tokens
2. Subtract from current sprint's "Grand Total"
3. Apply same formula to compute cost

**Example (Sprint 69):**
- Session Grand Total: 63,653,746 tokens
- Sprint 68 baseline: 23,078,954 tokens  
- Sprint 69 delta: 40,574,792 tokens (~$21)
```

**Expected Impact:**
- Clearer per-sprint cost attribution
- Easier sprint-over-sprint comparisons
- No code change needed if documentation approach chosen

**Effort:** 30-60 minutes (script change) or 15 minutes (documentation only)

---

### Action 4: Quality-Validator Test Strategy Must Enumerate Tests by Number

**Evidence:**
- Sprint 69: TC108 strategy said "7 unit tests" but only 2 were implemented initially
- 5 tests added as retro fixes (U03, U05, U06, U07 plus assertion strengthening)
- Pattern: Test count mentioned but not tracked individually

**Root Cause:**
Test strategy specifies test count but doesn't enumerate each test with a number. Phase 3 QV has no checklist to verify all planned tests exist.

**Solution:**
Require numbered test enumeration in strategy and verification in Phase 3.

**Implementation:**

**File:** `.claude/agents/quality-validator.md`
**Action:** Add to Step 2 (Design Tests)

```markdown
### Step 2: Design Tests

For each feature, design one or more tests that prove it works.

**CRITICAL: Enumerate all tests by number.**

Your test strategy must list each planned test with a unique identifier:
- TC108-U01: Wide terminal composed format
- TC108-U02: Narrow terminal drops row context  
- TC108-U03: Row context no `%`, separator exact
- TC108-U04: Scroll persistence
- TC108-U05: Very narrow truncation
- TC108-U06: Not-found no row context
- TC108-U07: Search segment width guard

**Total planned: 7 tests**

During Phase 3 execution, verify every numbered test is implemented. If a test is skipped, document the reason in evidence.
```

**Expected Impact:**
- Prevents "5 of 7 tests not implemented" scenario
- Creates accountability for planned vs. actual tests
- Saves 5-10K tokens in retro test additions

**Effort:** 20 minutes

---

## P2 - Medium Priority (Plan to Implement)

### Action 5: Document PTY Rewrite Risk for Complex Infrastructure Fixes

**Evidence:**
- Sprint 69: PTY fix cost ~$21 vs expected ~$15 for clean feature sprint
- First implementation (CPR in drain_pending) failed; required second pass
- Rewrite cycle added ~$6 to sprint cost

**Root Cause:**
PTY infrastructure fixes involve trial-and-error when the failure mode is not fully understood upfront. Sprint 69 correctly investigated options in Phase 2 but the first implementation approach didn't work.

**Solution:**
Add guidance for "infrastructure fix" sprints that may require implementation iteration.

**Implementation:**

**File:** `.claude/skills/sprint-coordinator/process/phase1-feature-planning.md`
**Action:** Add subsection on infrastructure fix planning

```markdown
### Infrastructure Fix Planning

When a sprint's primary objective is fixing infrastructure (e.g., PTY harness, test framework, CI):

1. **Budget for iteration:** Infrastructure fixes often require 2 implementation passes. Plan for 1.5x the normal feature cost.

2. **Prove approach first:** Before full implementation, validate the fix approach with a minimal proof-of-concept. Sprint 69's live-DB probe confirming CPR injection works was correct.

3. **Document fallback:** If the first approach fails, what's the backup? Sprint 69 pivoted from "CPR in drain_pending" to "CPR in hot read path" within the same phase.

4. **Accept higher cost:** A $21 infrastructure fix that unlocks 4 sprints of blocked tests is high-value. Don't force false efficiency.
```

**Expected Impact:**
- Sets realistic expectations for infrastructure sprints
- Documents the "prove then implement" pattern
- Reduces surprise when iteration is needed

**Effort:** 15 minutes

---

### Action 6: Split-Chunk CPR Detection (Sprint 70 P1)

**Evidence:**
- Sprint 69 review C1: `absorb_chunk` detects CPR query per-chunk only
- If `\x1b` arrives in one chunk and `[6n` in the next, the 4-byte sequence is split
- Low probability but latent reintroduction of P1 bug

**Solution:**
Sprint 70 should implement overlap-window scanning in `absorb_chunk`.

**Implementation:**
Document as Sprint 70 P1 candidate (already in Sprint 69 review follow-up items).

---

## Validation

After Sprint 70:

| Optimization | Validation Criteria |
|--------------|---------------------|
| Action 1 (hard .expect()) | No "skipped for reason: guard fired" in PTY test evidence |
| Action 2 (Phase 2 cross-check) | Spec/code divergences caught in Phase 2, not Phase 5 |
| Action 3 (sprint boundaries) | Sprint 70 metrics show direct cost, not delta computation |
| Action 4 (test enumeration) | All TC### tests numbered in strategy; 0 "tests not implemented" retro fixes |
| Action 5 (infra planning) | Infrastructure sprints budgeted at 1.5x; no surprise overruns |

---

## Implementation Order

1. **Action 1** (hard .expect() rule) — implement immediately, 15 min
2. **Action 4** (test enumeration) — implement immediately, 20 min  
3. **Action 2** (Phase 2 cross-check) — implement immediately, 30 min
4. **Action 3** (sprint boundaries) — implement before Sprint 70 metrics collection
5. **Action 5** (infra planning) — add to planning docs, 15 min

**Total implementation time:** ~90 minutes

**Expected savings:** $5-8 per sprint (fewer retro fixes, no PTY phantom passes, clearer cost tracking)
