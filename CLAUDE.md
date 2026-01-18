# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**tq** (Teradata Query) is a lightweight Rust command line client for Teradata databases. It follows a simple one-shot execution model: one tool call -> one connection -> close session when done.

## Claude Skills for this project
Use the following skills when working with code in this repository:
- teradata-rust: Guides writing idiomatic Rust code for Teradata database interactions using the teradatarustapi 
- rust-coder: for writing idiomatic Rust code
- rust-debugger: for debugging Rust code
- cli-ux-       designer: Skills to design CLI applications

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

6. Framework Optimization (Periodic - Every 3-4 Sprints)
   └─→ /optimize-agents: Analyze metrics, generate improvement actions (30-60 min)
       ↓ Produces concrete file edits to optimize agents/docs/tools
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

#### Sprint Planning Documents

Each sprint produces two key documents:
- **`docs/builder/sprints/sprint-N-planning.md`**: Scope, objectives, acceptance criteria (created at sprint start)
- **`docs/builder/sprints/sprint-N-review.md`**: Retrospective, metrics, lessons learned (created at sprint end)

These documents provide shared context for all agents and track sprint progress.

#### Self-Improvement System

The framework continuously optimizes itself through metrics-driven analysis:

**Step 1: Collect Metrics (Every Sprint)**
- Use `/collect-metrics <sprint-num>` during Phase 5 (Sprint Closure)
- Extracts token usage from subagent transcripts
- Generates `sprint-N-metrics.md` with factual data
- Adds metrics section to sprint review
- Duration: 2-3 minutes

**Step 2: Analyze & Optimize (Every 3-4 Sprints)**
- Use `/optimize-agents` after collecting metrics from multiple sprints
- Analyzes patterns across sprints to identify waste
- Applies decision tree analysis (see `docs/builder/token-optimization-decision-tree.md`)
- Generates concrete optimization actions (specific file edits)
- Duration: 30-60 minutes (Opus-powered deep analysis)

**What Gets Optimized:**
- **Agent prompts:** More efficient instructions, better context
- **Documentation:** Fill gaps that cause agent confusion
- **Workflow:** Improve parallelism, reduce rework
- **Tools:** Automate repetitive agent tasks
- **Quality processes:** Prevent Sprint 8-style quality failures

**Key Files:**
- `.claude/scripts/extract-sprint-metrics.sh` - Bash script for metrics extraction
- `docs/builder/token-optimization-decision-tree.md` - Systematic analysis framework
- `.claude/skills/collect-metrics/SKILL.md` - Simple data collection
- `.claude/skills/optimize-agents/SKILL.md` - Deep analysis and improvements

**Expected Outcomes:**
- 30-50% token reduction after 3-4 optimization cycles
- Zero quality failures through improved processes
- Faster sprint execution via better parallelism
- Continuously improving agent efficiency

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