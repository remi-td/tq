# Sprint Review Document Template

Use this template for `docs/sprints/sprint-N-review.md`:

```markdown
# Sprint [N] Review: [Sprint Name]

**Sprint Duration:** [Start] - [End]
**Status:** COMPLETED
**Version:** v[X.Y.Z]

---

## 1. Executive Summary

**Overall Assessment:** [Rating]/10
**Key Achievements:** [List]
**Sprint Health:** [Summary]

---

## 2. Sprint Metrics

### Feature Delivery

| Metric | Target | Actual |
|--------|--------|--------|
| Features Planned | N | N |
| Features Delivered | - | N |
| Tests Added | - | N |

### Cost Metrics

**Option A: When metrics are available (from `/collect-metrics` skill):**

**Data Source:** Session `<session-id>` via `/collect-metrics` skill
**Collection Date:** [Date]

| Agent | Input Tokens | Output Tokens | Total | Cache Hits | Estimated Cost |
|-------|--------------|---------------|-------|------------|----------------|
| Main (coordinator) | N | N | N | N% | $X.XX |
| rust-teradata-architect | N | N | N | N% | $X.XX |
| quality-validator | N | N | N | N% | $X.XX |
| cli-ux-designer | N | N | N | N% | $X.XX |
| **TOTAL** | **N** | **N** | **N** | **N%** | **$X.XX** |

**Cost per Feature:** $X.XX (N features delivered)

[Optional: Include phase-by-phase breakdown if available from metrics file]

**Note:** See `docs/sprints/sprint-N-metrics.md` for detailed breakdown.

---

**Option B: When metrics are NOT available:**

**Token metrics not collected for this sprint.**

Reason: [Transcript data unavailable / collect-metrics script not run / session ended before collection]

To enable metrics for future sprints:
- Invoke `/collect-metrics <sprint-number>` during Phase 5
- Requires sub-agent transcript files in session directory
- Provides actual token counts, cache hit rates, and cost estimates

**Context visible:** Main agent used ~[X]k tokens (from /context command at sprint end)
**Sub-agents:** Not measured (requires transcript analysis)

---

## 3. Technical Review
[From rust-teradata-architect]

## 4. Quality Review
[From quality-validator]

## 5. UX Review
[From cli-ux-designer]

---

## 6. Lessons Learned

### What Worked Well
1. [Item]

### What Could Improve
1. [Item]

---

## 7. Recommendations

### For Sprint [N+1]
1. [Priority]

### Agent Optimizations
1. [Skill/prompt improvement]

---

## 8. Action Items

| Action | Owner | Priority |
|--------|-------|----------|
| [Action] | [Agent] | High |

---

**Review Completed:** [Date]
**Next Sprint:** [N+1] - [Name]
```
