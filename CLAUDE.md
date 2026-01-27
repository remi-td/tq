# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**tq** (Teradata Query) is a lightweight Rust command line client for Teradata databases. It follows a simple one-shot execution model: one tool call -> one connection -> close session when done.

## Claude Skills for this project
Use the following skills when working with code in this repository:

### Development Skills
- **teradata-rust**: Guides writing idiomatic Rust code for Teradata database interactions using the teradatarustapi
- **rust-coder**: Guides writing idiomatic, efficient, well-structured Rust code
- **rust-debugger**: Diagnoses and fixes Rust compile errors, borrow-checker issues, runtime panics, and logic bugs
- **cli-designer**: Skills to design CLI applications following industry best practices

### Sprint Management Skills
- **sprint-coordinator**: Coordinates sprint phases and manages the sprint-driven development workflow (use when starting a new sprint)
- **sprint-reviewer**: Coordinates comprehensive sprint retrospectives with specialized agents, producing consolidated review documents
- **github-issues**: Manages GitHub issues for sprint intake, triage, and lifecycle management (use when triaging new issues, selecting issues for sprints, or updating issue status)
- **collect-metrics**: Collects token usage metrics from sprint subagent transcripts for framework optimization
- **optimize-agents**: Analyzes historical sprint metrics to identify framework optimization opportunities

### Framework Development Skills
- **skill-builder**: Guides the creation of effective Claude skills following the Agent Skills Standard
- **subagent-builder**: Creates specialized Claude subagents through interactive interviews and configuration guidance

## Development methodology

This project is developed exclusively by Claude Code using the skills and agents mentioned above.

### Workflow: Sprint-Driven Development

The project follows a structured sprint-driven approach coordinated by the **main Claude agent** (not a sub-agent). The main agent orchestrates specialized sub-agents through parallel execution for maximum efficiency.

**IMPORTANT: Use the `/sprint-coordinator` skill when starting a new sprint or coordinating sprint phases.**

The sprint-coordinator skill (`.claude/skills/sprint-coordinator/SKILL.md`) defines the complete 6-phase workflow:

0. **Restrospective** - Review the at least last three sprints to identify opportunities for improvement.
1. **Sprint Planning** - Review context, create sprint plan, execute autonomously
2. **Parallel Design** - cli-ux-designer + rust-teradata-architect in parallel
3. **Parallel Implementation** - rust-teradata-architect + quality-validator in parallel
4. **Test Execution & Fix Loop** - quality-validator executes tests, iterate until 100% pass
5. **Sprint Closure** - tq-project-manager validates, create sprint review, update roadmap
6. **Framework Optimization** - Review retro for improvements, optional token analysis, implement optimizations

**Key Principles:**
- **Full Autonomy - HEADLESS LOOP:** Execute all sprint phases (0-5) automatically without stopping for approval. This is a versioned, safe sandbox environment. NEVER ask "Should I proceed?" or wait for user permission between phases.
- **Own All Decisions:** The sprint coordinator makes ALL executive decisions autonomously. No supervisor. No approval gates.
- **Maximize Parallelism:** Launch independent sub-agents in a single message with multiple Task calls
- **Context Isolation:** Sub-agents handle verbose work; main conversation stays clean
- **Quality Focus:** Zero technical debt tolerance, 100% test pass rate before sprint closure
- **Documentation-Driven:** Every sprint produces planning and review documents
- **Self Reflection:** The primary Claude agent must reflect on its own performance and identify opportunities for improvement
- **Continuous Improvement:** Phase 6 ensures framework learns from each sprint

#### Specialized Sub-Agent Roles

**cli-ux-designer** (Sonnet)
- Owns: `specifications.md`, `detailed-specifications/*.md`
- Creates: User-facing specifications, CLI interface designs
- Launch when: Designing features, refining UX, updating specifications

**rust-teradata-architect** (Opus)
- Owns: `docs/design/` (technical design documents), production code, unit tests
- Creates: Design documentation, production code, unit tests, architectural patterns
- Launch when: Implementing features, refactoring code, making architectural decisions, updating design docs

**quality-validator** (Sonnet)
- Owns: `testing-guidelines.md`, test cases, test execution
- Creates: Test cases (`tests/cases/TC###.md`), test results, validation reports
- Launch when: Designing tests, executing test suites, validating quality

**tq-project-manager** (Haiku)
- Owns: Completion validation, tech debt tracking, sprint finalization
- Creates: Sprint completion validation reports, final sprint commits
- Performs: Git commit and push to GitHub after validation passes
- Launch when: Validating sprint completion, assessing technical debt, verifying quality standards

**optimization-analyzer** (Opus)
- Owns: Framework optimization analysis, waste pattern identification
- Creates: Structured optimization proposals with impact metrics
- Uses: `/optimize-agents` skill with waste patterns catalog
- Launch when: Phase 6 (Framework Optimization) - one instance per transcript, all in parallel
- Note: Multiple instances launched simultaneously, each analyzing one transcript in isolation

#### Sprint Planning Documents

Each sprint produces two key documents:
- **`docs/builder/sprints/sprint-N-planning.md`**: Scope, objectives, acceptance criteria (created at sprint start)
- **`docs/builder/sprints/sprint-N-review.md`**: Retrospective, metrics, lessons learned, framework optimization opportunities (created at sprint end)

These documents provide shared context for all agents and track sprint progress.

See the sprint-coordinator skill for complete details on Phase 6: Framework Optimization, which implements continuous improvement through sprint retrospective analysis and optional token metrics.

#### GitHub Issues Integration

The project uses GitHub Issues for feature requests, bug reports, and user feedback. This replaces the legacy `incoming/` folder approach.

**IMPORTANT: Use the `/github-issues` skill for all GitHub issue management tasks.**

**Issue Lifecycle:**

1. **Intake**: Users create issues using templates (`.github/ISSUE_TEMPLATE/`)
2. **Triage**: `/github-issues` skill analyzes new issues and applies labels:
   - `sprint-ready` - Accepted and ready for sprint inclusion
   - `needs-info` - Requires clarification from issue author
   - `wont-fix` - Rejected as out of scope
   - `duplicate` - Duplicate of existing issue
3. **Planning** (Phase 1): Sprint coordinator fetches `sprint-ready` issues, includes selected ones in sprint plan
4. **Tracking**: Issues are commented with sprint inclusion notice in Phase 1
5. **Closure** (Phase 4): Issues are updated/closed with implementation details after successful commit and push

**Issue Labels:**

Workflow:
- `sprint-ready` - Triaged and ready for sprint inclusion
- `needs-info` - Requires user clarification
- `wont-fix` - Rejected as out of scope
- `duplicate` - Duplicate of existing issue

Type:
- `bug` - Something isn't working correctly
- `enhancement` - New feature or improvement request
- `documentation` - Documentation updates

Priority:
- `priority-high` - High priority, blocking or critical
- `priority-medium` - Medium priority, important but not blocking
- `priority-low` - Low priority, nice to have

**Sprint Integration Points:**

- **Phase 1 (Planning)**: Use `/github-issues` to fetch sprint-ready issues, select for sprint, comment on selected issues
- **Phase 4 (Ship)**: Use `/github-issues` to close completed issues with implementation details, or comment on partial implementations

**Autonomous Operation**: The github-issues skill operates autonomously. It makes triage decisions based on project scope and specifications without requiring user approval.

### Documentation Organization

The project documentation is organized into five clear categories:

#### 1. Pure Specifications (`docs/specifications/`)

**Purpose:** Timeless feature requirements - WHAT the tool should do

**Contents:**
- `vision.md` - Project vision, goals, and principles
- `user-personas.md` - Target users and use cases
- `cli-interface.md` - Command-line interface specification
- `repl.md` - Interactive REPL mode specification
- `batch-mode.md` - Batch mode execution
- `configuration.md` - Configuration files and profiles
- `output-formats.md` - Output format specifications
- `error-handling.md` - Error messages and exit codes
- `security.md` - Security requirements
- `performance.md` - Performance targets
- `branding-guidelines.md` - Visual identity and branding

**Owner:** `cli-ux-designer` agent

**Key Principle:** These files contain ONLY pure requirements with NO implementation status, sprint references, or dates. They are the single source of truth for feature behavior.

#### 2. Technical Design (`docs/design/`)

**Purpose:** Technical architecture and implementation approach - HOW features are implemented

**Contents:**
- `README.md` - Organization and usage guide for design docs
- `vision.md` - High-level technical architecture, design principles, component integration
- `cli-interface.md` - Command parsing, argument handling implementation
- `repl.md` - REPL loop, state management, interactive features
- `batch-mode.md` - Batch execution, file processing (when created)
- `connection-management.md` - Connection lifecycle, credential resolution, Teradata integration
- `configuration.md` - Config loading, profile management (when created)
- `output-formats.md` - Formatters architecture, rendering pipeline (when created)
- `error-handling.md` - Error types, propagation patterns (when created)
- `security.md` - Security implementation details (when created)
- `performance.md` - Optimization techniques, profiling (when created)

**Owner:** `rust-teradata-architect` agent

**Key Principle:** Design documents explain implementation approach with code references, architectural patterns, and design decisions. They mirror the structure of specifications but address technical "how" instead of user-facing "what". NO sprint references, status updates, or dates.

**Relationship to Specifications:** Each specification in `docs/specifications/` may have a corresponding design document in `docs/design/` that explains the technical implementation.

#### 3. Roadmap & Status (`docs/roadmap/`)

**Purpose:** Implementation tracking and planning - WHEN features are/will be implemented

**Contents:**
- `status.md` - Current implementation status dashboard (✅ 🚧 📋)
- `backlog.md` - Prioritized feature backlog
- `roadmap.md` - High-level strategic direction

**Owner:** `sprint-coordinator`

**Updated:** After each sprint (status.md), during planning (backlog.md), quarterly (roadmap.md)

#### 4. Sprint History (`docs/sprints/`)

**Purpose:** Historical planning and retrospectives - Sprint context and lessons learned

**Contents:**
- `sprint-N-planning.md` - Sprint objectives and scope
- `sprint-N-review.md` - Retrospective with metrics and lessons

**Owner:** `sprint-coordinator`

**Note:** For reference only, not used during active development

#### 5. Testing Documentation (`docs/testing/`)

**Purpose:** Testing methodology and validation approach - HOW we validate implementations

**Contents:**
- `README.md` - Testing documentation organization and quick reference
- `philosophy.md` - Core testing principles and quality philosophy
- `approach.md` - Testing strategy, test types, design patterns
- `execution.md` - Running tests, best practices, debugging
- `tools.md` - Testing infrastructure, tools, and utilities

**Owner:** `quality-validator` agent

**Key Principle:** Testing documents explain HOW to validate features, independent of sprint execution. Test methodology is timeless, test results are per-sprint (in `tests/results/`).

### Document Authority and Usage

**IMPORTANT**: Documentation has different purposes and authority levels:

#### When Implementing Features
1. **Read specifications first**: `docs/specifications/` defines WHAT to build
2. **Read design docs**: `docs/design/` explains HOW to build it
3. **Check architecture**: `docs/design/vision.md` for high-level architecture
4. **Follow patterns**: Design docs for specific component patterns
5. **Design tests**: `docs/testing/` for test methodology and validation approach

#### When Planning Sprints
1. **Check status**: `docs/roadmap/status.md` for what's implemented
2. **Review backlog**: `docs/roadmap/backlog.md` for prioritized features
3. **Read specifications**: `docs/specifications/` for feature requirements
4. **Review design**: `docs/design/` for technical approach
5. **Create plan**: `docs/sprints/sprint-N-planning.md` for this sprint's scope

#### When Fixing Bugs
1. **Read specifications**: `docs/specifications/` for expected behavior
2. **Check design**: `docs/design/` for implementation approach
3. **Compare code**: Verify code matches design
4. **Never update status**: Bug fixes don't change specification requirements

#### When Refactoring
1. **Understand current design**: Read relevant docs in `docs/design/`
2. **Propose new approach**: Document design changes
3. **Update design docs**: Keep design docs synchronized with code
4. **Update specifications if needed**: Only if user-facing behavior changes

### Updating Documentation

**Specifications (`docs/specifications/`):**
- Updated by: `cli-ux-designer` agent
- When: Requirements change (NOT when implementation completes)
- Content: ONLY timeless requirements (no status, no sprint refs)
- Approval: Required from user for significant changes

**Design (`docs/design/`):**
- Updated by: `rust-teradata-architect` agent
- When: Architecture changes, new patterns added, or implementation approach changes
- Content: Technical design, architectural patterns, code references (no status, no sprint refs, no dates)
- Approval: Technical review during sprint
- Important: Keep synchronized with code - update design docs when refactoring

**Testing (`docs/testing/`):**
- Updated by: `quality-validator` agent
- When: Testing methodology evolves, new testing approaches discovered
- Content: Testing philosophy, approach, execution guidelines, tools (no status, no sprint refs, no dates)
- Approval: Technical review during sprint
- Important: Keep methodology timeless - sprint-specific test strategies go in `tests/strategy/`

**Roadmap (`docs/roadmap/`):**
- Updated by: `sprint-coordinator`
- When:
  - `status.md` - After sprint completion (Phase 4) - mark features ✅, update version numbers
  - `backlog.md` - During planning (Phase 1) - add new items, reprioritize, remove completed features
  - `roadmap.md` - Quarterly (Phase 0) - update strategic direction and milestones
- Content: Implementation status and planning info (no feature details - those live in specifications)
- Approval: Autonomous updates during sprint workflow

**Sprint History (`docs/sprints/`):**
- Updated by: `sprint-coordinator`
- When: At sprint start (planning.md) and sprint completion (review.md)
- Content: Sprint objectives, scope, retrospectives, lessons learned
- Approval: Autonomous creation during sprint workflow

### Environment configuration
The project uses a `.env` file to store development and test configuration that should not be committed to git.

1. Copy `.env.example` to `.env` in the project root
2. Edit `.env` to set your test connection details:
   ```
   TQ_LOGON=username:password@host:port/database
   ```
3. The `.env` file is automatically excluded from git tracking
4. Test agents and development workflows will automatically use values from `.env`

**Important**: The `.env` file should contain test credentials only, never production credentials.

## Guidelines
Never use absolute paths in the code or documentation. Use relative paths instead (to this project root, user's home directory, etc.).