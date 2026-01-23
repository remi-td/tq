# Phase 2: Design

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Create detailed specifications and assess technical feasibility—in parallel.

## Warmup (for Sub-Agents)
This phase follows Planning (Phase 1). At this point:
- `sprint-N-planning.md` defines what features you are designing.
- `cli-ux-designer`: Will update `docs/specifications/*.md` with pure, timeless requirements (no status badges).
- `rust-teradata-architect`: Will assess feasibility and update `docs/design/*.md` to document solution design.

## Process

### Step 1: Launch Parallel Agents

Launch BOTH agents in a **single message with multiple Task calls**:

1. **`cli-ux-designer`**:
   - Instruction: "Create or update pure specifications for features in Sprint N. Update `docs/specifications/*.md` with ONLY timeless requirements (no status badges, no sprint references). Break down into numbered requirements with zero ambiguity. Developers, testers, and writers will rely on this as the single source of truth. Return a summary of spec changes."

2. **`rust-teradata-architect`**:
   - Instruction: "Assess technical feasibility for Sprint N. Read `docs/sprints/sprint-N-planning.md` for objectives and `docs/specifications/*.md` for requirements. Update `docs/design/*.md` to document the solution design for each feature. Use `docs/design/vision.md` as architectural reference and update it if patterns change. Create or update feature-specific design docs (e.g., `docs/design/repl.md`, `docs/design/cli-interface.md`) explaining HOW features are implemented with code references. Return a feasibility assessment and any concerns."

2. **`quality-validator`**:
   - Instruction: "Define your test strategy to test features in Sprint N as per per requirements in `docs/sprints/sprint-N-planning.md`. Document your strategy in `tests/strategy/` based on `tests/strategy/test-strategy-template.md`. If you estimate that new tools are required to execute tests (or existing tools need to be updated), specify them in detail in the `tests/README.md` and raise requests this request to the coordinator.


### Step 2: Collect Results

Wait for all agents to complete. Expect:
- From Designer: Updated specs, summary of changes.
- From Architect: Feasibility assessment, architecture updates.
- From Quality Validator: Feasibility assessment, strategy and request for tool

### Step 4: Update testing framework

I the Quality Validator has requested new testing tools, add these to the sprint planning document and immediately develop it:
1. Run the  **`rust-teradata-architect`**: Instruction - The Quality Validator has requested the creation of testing tools {test tool names}, as fully specified in `tests/README.md`. Please implement them immediately and test them functionally.
2. Run the  **`quality-validator`**:  Instruction - The requested testing tools {test tool names} have been developed as specified in `tests/README.md`. Please valitate their functionality immediately and approve or reject the implementation

If the **`quality-validator`** rejects the implementation, provide the context and comments to the **`rust-teradata-architect`** and request fix. Loop until validation from **`quality-validator`**.



### Step 4: Synthesize

Review both outputs:
- **Gaps?** Identify missing specifications or unclear architecture.
- **Agreement?** Ensure that both accepted to deliver the features in this sprint (or move them to the backlog in `specifications.md` amd update `sprint-N-planning.md`)
- **Ready?** Proceed to Phase 3.

## Output
- Updated `docs/specifications/*.md` (pure requirements only).
- Updated `docs/design/*.md` (technical design documentation).
- Proceed to Phase 3 (Build & Test).
