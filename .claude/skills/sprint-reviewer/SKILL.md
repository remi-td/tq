---
name: sprint-reviewer
description: Conducts single-pass sprint retrospectives with token delta metrics. Use at the end of each sprint when all features are complete and tested.
---

# Sprint Reviewer (Lean Mode)

Generate a single concise sprint retrospective (<50 lines) with actual token delta metrics and test verification summary.

## Prerequisites
1. All features implemented
2. All unit and integration tests passing (`cargo test`)
3. Version updated in Cargo.toml
4. Git commits and release tag complete

## Review Process

### Step 1: Collect Metrics
Invoke `/collect-metrics` skill to generate `docs/sprints/sprint-N-metrics.md`.
The script computes true per-sprint deltas automatically.

### Step 2: Create Concise Review Document
Create `docs/sprints/sprint-N-review.md` using `references/template.md`.
**HARD CONSTRAINT:** Keep document under 50 lines total. Do NOT launch sub-agents (`architect`, `quality`, `ux`) for retrospective reviews.

### Step 3: Update Roadmap
- Mark sprint as COMPLETED in `docs/roadmap/status.md`.
- Add 1-line retrospective summary.

## Key Principles

| Principle | Rule |
|-----------|------|
| Single File | ONE concise review per sprint (`sprint-N-review.md`) |
| Under 50 lines | Keep reviews tight to preserve prompt context window |
| Real Metrics | Use `/collect-metrics` for token delta data |
| No Sub-Agents | Single-pass by coordinator; zero parallel review bloat |
