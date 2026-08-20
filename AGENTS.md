# AGENTS.md

This file provides system guidance and workflow rules for **Antigravity** when working with code in this repository.

## Project Overview

**tq** (Teradata Query) is a fast, lightweight Rust command line client for Teradata databases. It follows a simple one-shot execution model: one tool call -> one connection -> close session when done.

---

## Antigravity Skill & Persona Discovery

Antigravity automatically discovers skills and rules under `.agents/`. When performing tasks, you MUST read and follow the instructions in the relevant files:

### Available Development Skills
- **`teradata-rust`** (`.agents/skills/teradata-rust/SKILL.md`): Idiomatic Rust code for Teradata database interactions using `teradatarustapi`.
- **`rust-coder`** (`.agents/skills/rust-coder/SKILL.md`): Idiomatic, efficient, well-structured Rust code modeling and patterns.
- **`rust-debugger`** (`.agents/skills/rust-debugger/SKILL.md`): Diagnosing and fixing Rust compile errors, borrow-checker issues, and runtime panics.
- **`cli-designer`** (`.agents/skills/cli-designer/SKILL.md`): Designing CLI applications following `clig.dev` best practices.

### Available Sprint Management Skills
- **`sprint-coordinator`** (`.agents/skills/sprint-coordinator/SKILL.md`): Orchestrating the 7-phase sprint workflow.
- **`sprint-reviewer`** (`.agents/skills/sprint-reviewer/SKILL.md`): Retrospective analysis and review documents.
- **`github-issues`** (`.agents/skills/github-issues/SKILL.md`): GitHub issue intake, triage, and status updates.

### Persona Role Descriptions (`.agents/agents/`)
When coordinating sprint phases, inspect and adopt the specific persona checklists and guidelines from these files:
- **`cli-ux-designer`** (`.agents/agents/cli-ux-designer.md`): UX specifications and CLI interface design.
- **`rust-teradata-architect`** (`.agents/agents/rust-teradata-architect.md`): Technical design, production code, unit tests, and architecture.
- **`quality-validator`** (`.agents/agents/quality-validator.md`): Test design, test execution (`tests/`), and quality standards verification.
- **`tq-project-manager`** (`.agents/agents/tq-project-manager.md`): Tech debt tracking, sprint validation, git commits, and version release tagging.

---

## Sprint-Driven Development Workflow

Execute the sprint-driven approach through the following 7-phase lifecycle:

### Phase 0: Reality Check
- Scan recent sprint state (`docs/roadmap/status.md` and past sprint reviews in `docs/sprints/`) to decide whether to execute a **Feature Sprint** or **Maintenance Sprint**.

### Phase 1: Lean Planning
- Create/update `docs/sprints/sprint-N-planning.md` (keep concise, 40-50 lines max) detailing scope, objectives, and acceptance criteria.
- If using GitHub issues, use `github-issues` skill to select `sprint-ready` issues.

### Phase 2: Design (Persona: `cli-ux-designer` & `rust-teradata-architect`)
- Inspect `.agents/agents/cli-ux-designer.md` and `.agents/agents/rust-teradata-architect.md` using `view_file`.
- Update CLI specifications in `docs/specifications/cli-interface.md` and technical designs in `docs/design/`.

### Phase 3: Build & Test (Persona: `rust-teradata-architect` & `quality-validator`)
- Inspect `.agents/agents/rust-teradata-architect.md` and `.agents/skills/rust-coder/SKILL.md`.
- Implement production code in `src/`.
- Inspect `.agents/agents/quality-validator.md` and `.agents/skills/rust-debugger/SKILL.md`.
- Write unit tests in `src/` and integration tests in `tests/`.
- Verify compilation and code quality:
  - `cargo check --tests`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

### Phase 4: Ship (Persona: `tq-project-manager`)
- Inspect `.agents/agents/tq-project-manager.md`.
- Verify 100% test pass rate across unit and integration tests.
- Bump version in `Cargo.toml`.
- Stage files, commit with conventional commit format (`feat(...)` / `fix(...)`), create release tag (`vX.Y.Z`), and push to GitHub.

### Phase 5: Retrospective & Review
- Create `docs/sprints/sprint-N-review.md` (keep under 50 lines) summarizing accomplishments, verification metrics, and delta metrics.
- Update `docs/roadmap/status.md`.

### Phase 6: Action-Only Optimization
- If workflow friction occurred, directly apply prompt or code edits to optimize skills/rules.

---

## Key Principles & Execution Guardrails

1. **Full Autonomy**: Execute all sprint phases (0-6) automatically without stopping for approval between phases. Own all technical and design decisions.
2. **Quality Gate**: Zero technical debt tolerance. `cargo clippy --all-targets -- -D warnings` MUST pass with 0 warnings before shipping.
3. **Never Guess Code Logic or Schemas**: Use `view_file` and `grep_search` to verify existing source code before editing.
4. **Log Inspection**: Fetch and inspect full error logs before diagnosing runtime or test failures.
5. **No Superficial Patches**: Never mask symptoms, comment out assertions, or delete failing tests. Fix the underlying root cause.