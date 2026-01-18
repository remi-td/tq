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
- **collect-metrics**: Collects token usage metrics from sprint subagent transcripts for framework optimization
- **optimize-agents**: Analyzes historical sprint metrics to identify framework optimization opportunities

### Framework Development Skills
- **skill-builder**: Guides the creation of effective Claude skills following the Agent Skills Standard
- **subagent-builder**: Creates specialized Claude subagents through interactive interviews and configuration guidance

## Development methodology

This project is developed exclusively by Claude Code using the skills and agents mentioned above.

### Workflow: Sprint-Driven Development

The project follows a structured sprint-driven approach coordinated by the **main Claude agent** (not a sub-agent). The main agent orchestrates specialized sub-agents through parallel execution for maximum efficiency.

**Use the `/sprint-coordinator` skill** when starting a new sprint or coordinating sprint phases.

#### Sprint Workflow Overview

```
1. Sprint Planning (Main Agent)
   - Check docs/builder/incoming/ for new feature requests/bugs
   - Review previous sprint retrospective
   ↓ Creates: sprint-N-planning.md

2. Parallel Design (Main Agent Coordinates)
   ├─→ cli-ux-designer: Detailed specs
   └─→ rust-teradata-architect: Technical feasibility
   ↓ Main agent synthesizes outputs

2.5. Database Connectivity Check (Main Agent)
   - Verify .env file exists and TQ_LOGON is configured
   - Run: ./target/release/tq ping
   - If fails: STOP and ask user to start test database
   ↓ Only proceed when database is verified

3. Parallel Implementation (Main Agent Coordinates)
   ├─→ rust-teradata-architect: Implementation
   └─→ quality-validator: Test case design
   ↓ Main agent reviews outputs

4. Test Execution & Fix Loop (Main Agent Coordinates)
   └─→ quality-validator: Execute tests
   ↓ If failures:
       ├─→ rust-teradata-architect: Fix issues
       └─→ quality-validator: Re-run tests
       ↓ Loop until 100% pass rate

5. Sprint Closure (Main Agent Coordinates)
   ├─→ tq-project-manager: Validate completion + commit/push to GitHub
   ├─→ Main agent: Sprint review + specifications update
   ├─→ /collect-metrics: Extract token usage data (2-3 min)
   └─→ Main agent: Update docs/user/roadmap.md

6. Framework Optimization (End of every sprint)
   ├─→ Step 1: Close sprint work
   │   └─→ tq-project-manager: Commit & push feature code
   │
   ├─→ Step 2: Parallel optimization analysis (using Task tool)
   │   ├─→ Task(optimization-analyzer): Analyze transcript-001.md for sprint N
   │   ├─→ Task(optimization-analyzer): Analyze transcript-002.md for sprint N
   │   ├─→ Task(optimization-analyzer): Analyze transcript-003.md for sprint N
   │   └─→ ... (Variable number based on agent invocations in sprint)
   │   Note: Each agent analyzes one transcript in isolated context
   │
   ├─→ Step 3: Consolidate proposals into backlog
   │   └─→ Main agent: Review outputs, deduplicate, assign IDs (P###)
   │       Add to docs/builder/optimization-backlog/pending/
   │
   ├─→ Step 4: Prioritize & implement optimizations
   │   ├─→ Main agent: Calculate impact scores for all proposals
   │   ├─→ Main agent: Select top 3-5 high-impact proposals
   │   ├─→ Main agent: Compact conversation history
   │   └─→ Main agent: Implement selected proposals
   │       Move implemented proposals to optimization-backlog/implemented/
   │
   └─→ Step 5: Commit framework improvements
       └─→ Main agent: Commit & push with descriptive message
```

#### Key Principles

1. **Main Agent Coordinates:** The primary Claude agent owns the workflow and makes all decisions
2. **Maximize Parallelism:** Launch independent sub-agents in a single message with multiple Task calls
3. **Context Isolation:** Sub-agents handle verbose work; main conversation stays clean
4. **Quality Focus:** Zero technical debt tolerance, 100% test pass rate before sprint closure
5. **Documentation-Driven:** Every sprint produces planning and review documents

#### Specialized Sub-Agent Roles

**cli-ux-designer** (Sonnet)
- Owns: `specifications.md`, `detailed-specifications/*.md`
- Creates: User-facing specifications, CLI interface designs
- Launch when: Designing features, refining UX, updating specifications

**rust-teradata-architect** (Opus)
- Owns: `rust-cli-design-general.md`, `rust-architecture.md`, implementation
- Creates: Production code, unit tests, architecture docs
- Launch when: Implementing features, refactoring code, making architectural decisions

**quality-validator** (Sonnet)
- Owns: `testing-guidelines.md`, test cases, test execution
- Creates: Test cases (`tests/cases/TC###.md`), test results, validation reports
- Launch when: Designing tests, executing test suites, validating quality

**tq-project-manager** (Haiku)
- Owns: Completion validation, tech debt tracking, sprint finalization
- Creates: Sprint completion validation reports, final sprint commits
- Performs: Git commit and push to GitHub after validation approval
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
- **`docs/builder/sprints/sprint-N-review.md`**: Retrospective, metrics, lessons learned (created at sprint end)

These documents provide shared context for all agents and track sprint progress.

#### Self-Improvement System

The framework continuously optimizes itself through metrics-driven analysis and a structured optimization backlog:

**Every Sprint - Collect Metrics (Phase 5)**
- Use `/collect-metrics <sprint-num>` during Sprint Closure
- Extracts token usage from subagent transcripts
- Generates `sprint-N-metrics.md` with factual data
- Duration: 2-3 minutes

**Every Sprint - Analyze & Optimize (Phase 6)**
1. **Parallel Analysis:** Launch `optimization-analyzer` agents for each transcript
   - Each agent identifies waste patterns using `/optimize-agents` skill
   - Generates structured proposals with metrics (tokens saved, frequency, confidence)
   - Returns proposals to main agent

2. **Consolidation:** Main agent processes all proposals
   - Deduplicates similar findings across transcripts
   - Assigns proposal IDs (P001, P002, etc.)
   - Adds to `docs/builder/optimization-backlog/pending/`

3. **Prioritization:** Calculate impact scores
   ```
   Impact Score = (Tokens Saved × Frequency Weight × Confidence Weight) / Implementation Effort
   ```
   - Select top 3-5 proposals for immediate implementation
   - Prioritize: High impact OR (high confidence + small effort)

4. **Implementation:** Main agent implements selected proposals
   - Makes concrete file changes (agent prompts, documentation, workflow)
   - Moves proposals from `pending/` to `implemented/`
   - Commits framework improvements

5. **Validation:** Next sprint verifies impact
   - Metrics should show expected token reductions
   - Quality maintained or improved
   - Document results in proposal files

**What Gets Optimized:**
- **Agent prompts:** Eliminate redundant reads, add missing context
- **Documentation:** Fill gaps causing exploration/questions
- **Workflow:** Improve parallelism, reduce rework loops
- **Quality processes:** Prevent recurring bugs/failures

**Key Components:**
- `.claude/subagents/optimization-analyzer.md` - Analyzes single transcript for waste
- `.claude/skills/optimize-agents/SKILL.md` - Analysis framework and workflow
- `.claude/skills/optimize-agents/references/waste-patterns.md` - Pattern catalog
- `docs/builder/optimization-backlog/` - Structured backlog with proposals
- `.claude/skills/collect-metrics/SKILL.md` - Metrics extraction

**Expected Outcomes:**
- 30-50% token reduction after 3-4 optimization cycles
- Zero quality failures through improved processes
- Faster sprint execution via better parallelism
- Continuously improving agent efficiency
- Data-driven, measurable improvements

### Master specification documents

The project is governed by authoritative specification documents located in `docs/builder/`:

#### Main Specifications

1. **`specifications.md`** - Main specifications dashboard
   - High-level feature status dashboard with visual indicators (✅ 🚧 📋 🔲)
   - Sprint roadmap showing delivered, current, and planned work
   - Quick navigation to detailed specifications
   - Owned by the `cli-ux-designer` agent
   - Shows WHAT is implemented and WHAT is planned

2. **`detailed-specifications/*.md`** - Detailed technical specifications
   - Comprehensive specifications organized by domain
   - Each file is self-contained and covers a specific area:
     - `user-personas.md` - Target users and use cases
     - `cli-interface.md` - Command structure, flags, help text
     - `repl-mode.md` - Interactive mode specifications
     - `batch-mode.md` - Non-interactive execution
     - `configuration.md` - Config files and credentials
     - `output-formats.md` - Table, JSON, CSV formatting
     - `error-handling.md` - Error messages and exit codes
     - `security.md` - Security requirements
     - `performance.md` - Performance considerations
   - Owned by the `cli-ux-designer` agent
   - Defines WHAT the tool should do and HOW users interact with it

#### Architecture and Testing

3. **`rust-cli-design-general.md`** - General Rust CLI design guidelines
   - General Rust CLI design principles and best practices
   - Owned by the `rust-teradata-architect` agent
   - Provides patterns and principles for CLI tool development

4. **`rust-architecture.md`** - Rust architecture for tq
   - Architecture document specific to the tq tool
   - Owned by the `rust-teradata-architect` agent
   - Defines HOW the tool is implemented internally

5. **`testing-guidelines.md`** - Testing methodology and best practices
   - Testing approach, patterns, and execution techniques
   - Owned by the `quality-validator` agent
   - Defines HOW to design and execute quality validation tests
   - Provides templates, checklists, and lessons learned

### Document authority and precedence

**IMPORTANT**: The content of these specification documents is authoritative and overrides any other information, best practices, or general knowledge when working on this project.

When designing, coding, or testing:
1. **Always consult** the relevant specification documents first
2. **Follow** the specifications exactly as written
3. **Propose updates** to the specifications when you identify gaps or improvements
4. **Never deviate** from the specifications without explicit approval

### Updating specifications

- Any significant change to project specifications or guidelines MUST be reflected in these documents
- Changes to these documents MUST be carefully evaluated and approved by the project subject matter expert (the user)
- When proposing changes, clearly explain the rationale and impact

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