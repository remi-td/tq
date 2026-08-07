# Phase 5: Retrospective (Lean Mode)

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Produce a concise, data-backed sprint retrospective without launching parallel review sub-agents.

## Prerequisites
- Phase 4 completed (code committed, pushed, tagged)
- All tests passing

## Process

### Step 1: Collect Metrics
Invoke `/collect-metrics` skill to generate `docs/sprints/sprint-N-metrics.md`.
The script automatically calculates true per-sprint deltas if running in a shared multi-sprint session.

### Step 2: Write Concise Retrospective Document
Create `docs/sprints/sprint-N-review.md` using the lean template.
**Hard Constraint:** Keep document under 50 lines total.

**Include:**
1. **Executive Summary:** Sprint number, features delivered, version release tag.
2. **Verification & Quality:** Unit & integration test execution status, Clippy results.
3. **Sprint Delta Metrics:** Token delta, cost estimate, cache hit rate from `sprint-N-metrics.md`.
4. **Key Learnings / Friction:** 1–2 bullet points on what went well or blockers encountered.

Do NOT launch `rust-teradata-architect`, `quality-validator`, or `cli-ux-designer` sub-agents for retrospective reviews.

### Step 3: Update Roadmap
Mark completed features in `docs/roadmap/status.md`.