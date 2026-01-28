# Project Documentation Specifications

## Overview

This document specifies requirements for user-facing project documentation, including the README, contribution guidelines, and other repository documentation. These documents serve as the first point of contact for users and contributors.

## README.md Structure and Content

**REQ-DOC-001: README Purpose and Audience**

The README.md file SHALL serve as the primary onboarding document for new users:

1. **REQ-DOC-001.1** - Target audience: Users who want to quickly understand and try the tool
2. **REQ-DOC-001.2** - Secondary audience: Developers considering contribution
3. **REQ-DOC-001.3** - Tone: Professional, concise, welcoming
4. **REQ-DOC-001.4** - Length target: Readable in under 3 minutes
5. **REQ-DOC-001.5** - Format: Markdown with clear section hierarchy

**REQ-DOC-002: README Section Order**

The README.md SHALL follow this section structure:

1. **REQ-DOC-002.1** - Section 1: TLDR Summary (What/Visual/Quick Start)
2. **REQ-DOC-002.2** - Section 2: AI Development Story
3. **REQ-DOC-002.3** - Section 3: Installation
4. **REQ-DOC-002.4** - Section 4: Usage Examples
5. **REQ-DOC-002.5** - Section 5: Features
6. **REQ-DOC-002.6** - Section 6: Documentation Links
7. **REQ-DOC-002.7** - Section 7: Development and Contribution
8. **REQ-DOC-002.8** - Section 8: License
9. **REQ-DOC-002.9** - Section 9: Trademarks and Disclaimers

### Section 1: TLDR Summary

**REQ-DOC-003: TLDR Summary Content**

The opening section SHALL provide instant value:

1. **REQ-DOC-003.1** - Project name and tagline (single sentence, <15 words)
2. **REQ-DOC-003.2** - Visual demonstration: Screenshot or animated GIF showing tool in action
3. **REQ-DOC-003.3** - What is it: 2-3 sentence description of purpose and target users
4. **REQ-DOC-003.4** - Key differentiator: 1 sentence highlighting unique value proposition
5. **REQ-DOC-003.5** - Quick start: Minimal 3-step getting started guide

**Example Structure:**

```markdown
# tq - Teradata Query CLI

> A lightweight, Rust-powered command-line client for Teradata databases with
> interactive REPL and modern output formatting.

![tq in action](docs/images/tq-screenshot.png)

## What is tq?

tq is a fast, user-friendly terminal client for Teradata databases. It provides
a powerful REPL (Read-Eval-Print Loop) with tab completion, syntax highlighting,
and beautiful table output. Perfect for DBAs, analysts, and developers who work
with Teradata from the command line.

**Why tq?** Modern CLI experience, instant startup, no Java dependencies.

## Quick Start

```bash
# Install
cargo install tq

# Connect
export TQ_LOGON="user:pass@host:1025/db"
tq repl

# Query
tq> SELECT * FROM employees LIMIT 10;
```
```

**REQ-DOC-004: Visual Demonstration Requirements**

The screenshot or demo SHALL:

1. **REQ-DOC-004.1** - Show actual tool output (not mockup)
2. **REQ-DOC-004.2** - Demonstrate key features: REPL prompt, table output, syntax highlighting
3. **REQ-DOC-004.3** - Use realistic sample data (not lorem ipsum)
4. **REQ-DOC-004.4** - Display in typical terminal width (80-120 columns)
5. **REQ-DOC-004.5** - High quality image or optimized GIF (<500KB)
6. **REQ-DOC-004.6** - Dark terminal theme (preferred for developer audience)
7. **REQ-DOC-004.7** - Include image alt text for accessibility

### Section 2: AI Development Story

**REQ-DOC-005: AI Development Section**

This section SHALL tell the unique story of AI-exclusive development:

1. **REQ-DOC-005.1** - Headline: "Built Exclusively by AI Agents"
2. **REQ-DOC-005.2** - Tone: Tongue-in-cheek but professional
3. **REQ-DOC-005.3** - Content length: 2-3 paragraphs
4. **REQ-DOC-005.4** - Explain the agent-driven development process
5. **REQ-DOC-005.5** - Mention specialized agents (cli-ux-designer, rust-teradata-architect, etc.)
6. **REQ-DOC-005.6** - Explain contribution model (humans submit issues, agents implement)
7. **REQ-DOC-005.7** - Link to roadmap for current development status
8. **REQ-DOC-005.8** - Maintain credibility (avoid over-hyping, acknowledge limitations)

**Example Content:**

```markdown
## Built Exclusively by AI Agents

Here's something different: `tq` is developed entirely by AI agents using Claude
Code. No human has written a line of production code. Instead, specialized AI
agents collaborate through a sprint-driven workflow:

- **cli-ux-designer**: Designs the user experience and interface specifications
- **rust-teradata-architect**: Implements features and maintains code quality
- **quality-validator**: Writes and executes test suites
- **tq-project-manager**: Coordinates releases and manages technical debt

**How to contribute?** Humans are welcome to submit GitHub issues with feature
requests or bug reports. The AI agents triage, prioritize, and implement them
autonomously in sprint cycles. Think of it as a collaborative experiment in
AI-driven software development.

Current development status and roadmap: [docs/roadmap/status.md](docs/roadmap/status.md)

*Disclaimer: While the agents handle implementation, humans oversee the project
direction and validate releases. This is an experiment in AI capabilities, not
a replacement for human developers.*
```

**REQ-DOC-006: AI Development Section Guidelines**

When describing AI development:

1. **REQ-DOC-006.1** - Avoid claiming "sentience" or "consciousness"
2. **REQ-DOC-006.2** - Use "AI agents" or "agents", not "robots" or "AGI"
3. **REQ-DOC-006.3** - Acknowledge human oversight and validation
4. **REQ-DOC-006.4** - Frame as collaborative experiment, not marketing gimmick
5. **REQ-DOC-006.5** - Be honest about limitations and ongoing learning
6. **REQ-DOC-006.6** - Invite users to observe and provide feedback

### Section 3: Installation

**REQ-DOC-007: Installation Instructions**

Installation section SHALL be clear and comprehensive:

1. **REQ-DOC-007.1** - Primary method: cargo install (Rust ecosystem)
2. **REQ-DOC-007.2** - Prerequisites: Rust toolchain installation link
3. **REQ-DOC-007.3** - System requirements: Supported platforms (Linux, macOS, Windows)
4. **REQ-DOC-007.4** - License acceptance reminder (link to LICENSE file)
5. **REQ-DOC-007.5** - Verification step: `tq --version` to confirm installation
6. **REQ-DOC-007.6** - Alternative methods (if available): pre-built binaries, Docker, package managers

**Example Content:**

```markdown
## Installation

### Prerequisites

tq is written in Rust. Install the Rust toolchain if you haven't already:
https://rustup.rs

Supported platforms: Linux, macOS, Windows

### Install from crates.io

```bash
cargo install tq
```

### Verify Installation

```bash
tq --version
# tq 1.12.0
```

**License Notice:** By installing tq, you accept the license terms for bundled
dependencies (Teradata drivers, Go runtime). See [LICENSE](LICENSE) for details.

### Alternative Installation Methods

**Pre-built Binaries** (coming soon)

Download from [Releases](https://github.com/username/tq/releases)

**Docker** (coming soon)

```bash
docker run -it tq:latest
```
```

### Section 4: Usage Examples

**REQ-DOC-008: Usage Examples**

The usage section SHALL demonstrate common workflows:

1. **REQ-DOC-008.1** - Example 1: Basic connection and query (REPL mode)
2. **REQ-DOC-008.2** - Example 2: One-shot query (batch mode)
3. **REQ-DOC-008.3** - Example 3: Configuration file usage
4. **REQ-DOC-008.4** - Example 4: Output format selection (CSV, JSON)
5. **REQ-DOC-008.5** - Each example SHALL include command + expected output
6. **REQ-DOC-008.6** - Examples SHALL use realistic sample data
7. **REQ-DOC-008.7** - Examples SHALL demonstrate value (not trivial "hello world")

**Example Content:**

```markdown
## Usage

### Interactive REPL Mode

Start an interactive session:

```bash
export TQ_LOGON="myuser:mypass@tdhost:1025/mydb"
tq repl
```

Inside the REPL:

```sql
tq> SELECT employee_id, first_name, last_name
    FROM employees
    WHERE department = 'Engineering'
    LIMIT 5;

┌─────────────┬────────────┬───────────┐
│ employee_id │ first_name │ last_name │
├─────────────┼────────────┼───────────┤
│ 1001        │ Alice      │ Anderson  │
│ 1002        │ Bob        │ Brown     │
│ 1003        │ Charlie    │ Chen      │
│ 1004        │ Diana      │ Davis     │
│ 1005        │ Eve        │ Evans     │
└─────────────┴────────────┴───────────┘

5 rows (0.123s)
```

### One-Shot Queries

Execute a single query:

```bash
tq query "SELECT COUNT(*) FROM orders WHERE date = CURRENT_DATE"
```

### Export to CSV

```bash
tq query "SELECT * FROM sales_summary" --format csv > report.csv
```

### Using Configuration File

Create `~/.config/tq/config.toml`:

```toml
[profiles.production]
host = "prod-td.company.com"
port = 1025
database = "sales_db"
username = "analyst"

[profiles.staging]
host = "staging-td.company.com"
port = 1025
database = "test_db"
username = "analyst"
```

Connect using profile:

```bash
tq repl --profile production
```
```

### Section 5: Features

**REQ-DOC-009: Features Section**

The features section SHALL highlight key capabilities:

1. **REQ-DOC-009.1** - Organized by category (REPL features, output formats, performance, etc.)
2. **REQ-DOC-009.2** - Bulleted list format for scannability
3. **REQ-DOC-009.3** - Emphasize user benefits, not technical implementation
4. **REQ-DOC-009.4** - Include both implemented and planned features (mark clearly)
5. **REQ-DOC-009.5** - Link to detailed documentation for complex features

**Example Content:**

```markdown
## Features

### Interactive REPL
- Multi-line SQL editing with syntax highlighting
- Tab completion for tables, columns, and SQL keywords
- Command history with search (Ctrl-R)
- Emacs and Vi editing modes
- Metacommands: `/describe`, `/list`, `/export`, `/sessions`, and more

### Output Formats
- Beautiful ASCII table output with box-drawing characters
- CSV export for data analysis tools
- JSON output for programmatic processing
- Automatic column truncation for wide results
- Result paging for large datasets

### Performance
- Instant startup (no JVM warmup)
- Efficient memory usage
- Fast result rendering
- Configurable result limits

### Developer Experience
- Configuration profiles for multiple environments
- Secure credential management
- Detailed error messages
- Batch mode for scripting

See [full feature documentation](docs/specifications/) for detailed specifications.
```

### Section 6: Documentation Links

**REQ-DOC-010: Documentation Navigation**

The documentation section SHALL provide clear entry points:

1. **REQ-DOC-010.1** - Link to user guide (if available)
2. **REQ-DOC-010.2** - Link to feature specifications
3. **REQ-DOC-010.3** - Link to roadmap/status
4. **REQ-DOC-010.4** - Link to examples/tutorials
5. **REQ-DOC-010.5** - Link to troubleshooting guide (if available)
6. **REQ-DOC-010.6** - Link to API/technical documentation (if available)

**Example Content:**

```markdown
## Documentation

- **[User Guide](docs/user/)** - Comprehensive usage documentation
- **[Feature Specifications](docs/specifications/)** - Detailed feature specs
- **[Roadmap](docs/roadmap/status.md)** - Current implementation status
- **[Architecture](docs/design/)** - Technical design documents
- **[Examples](examples/)** - Sample queries and workflows

**Need help?** Check the [FAQ](docs/FAQ.md) or open a GitHub issue.
```

### Section 7: Development and Contribution

**REQ-DOC-011: Development Section**

The development section SHALL explain the unique contribution model:

1. **REQ-DOC-011.1** - Explain AI-driven development workflow
2. **REQ-DOC-011.2** - How humans can contribute (submit GitHub issues)
3. **REQ-DOC-011.3** - Issue templates and triage process
4. **REQ-DOC-011.4** - Sprint cycle overview (optional, high-level)
5. **REQ-DOC-011.5** - Link to CONTRIBUTING.md (if available)
6. **REQ-DOC-011.6** - Code of conduct link (if available)

**Example Content:**

```markdown
## Development & Contribution

This project uses an AI-driven development workflow. Instead of traditional pull
requests, we accept contributions through GitHub Issues:

### How to Contribute

1. **Submit a GitHub Issue** using our templates:
   - Bug reports
   - Feature requests
   - Documentation improvements

2. **AI agents triage and implement** your issue autonomously in sprint cycles

3. **Track progress** via issue comments and roadmap updates

### Issue Labels

- `sprint-ready` - Accepted and queued for implementation
- `needs-info` - Requires clarification from issue author
- `in-progress` - Currently being implemented
- `completed` - Implemented and released

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

### Local Development

While agents handle implementation, you can explore the codebase:

```bash
git clone https://github.com/username/tq.git
cd tq
cargo build
cargo test
```

**Note:** Direct code contributions (pull requests) are not currently accepted
as this would interfere with the AI development experiment. However, issue
reports and feature requests are highly valued.
```

### Section 8: License

**REQ-DOC-012: License Section**

The license section SHALL clearly communicate licensing:

1. **REQ-DOC-012.1** - State primary license (MIT)
2. **REQ-DOC-012.2** - Note dependency licenses (Teradata, Go)
3. **REQ-DOC-012.3** - Link to LICENSE file
4. **REQ-DOC-012.4** - User acceptance statement
5. **REQ-DOC-012.5** - Keep concise (detailed terms in LICENSE file)

**Example Content:**

```markdown
## License

The `tq` tool source code is licensed under the **MIT License**.

**Important:** This tool depends on third-party software with separate licenses:
- Teradata GoSQL Driver (Teradata proprietary license)
- Go runtime (BSD-style Go license)

By installing and using tq, you accept all dependency license terms.

See [LICENSE](LICENSE) for complete license text and attributions.
```

### Section 9: Trademarks and Disclaimers

**REQ-DOC-013: Trademarks Section**

The trademarks section SHALL include legal notices:

1. **REQ-DOC-013.1** - Teradata trademark notice
2. **REQ-DOC-013.2** - Clarify not an official Teradata product
3. **REQ-DOC-013.3** - Clarify not endorsed by Teradata
4. **REQ-DOC-013.4** - Other relevant disclaimers

**Example Content:**

```markdown
## Trademarks

Teradata is a registered trademark of Teradata Corporation.

This project is **not affiliated with, endorsed by, or sponsored by Teradata
Corporation**. The name "Teradata" is used solely to indicate compatibility
with Teradata database systems.
```

## Additional Documentation Files

**REQ-DOC-014: CONTRIBUTING.md**

If a CONTRIBUTING.md file exists, it SHALL:

1. **REQ-DOC-014.1** - Explain AI-driven development workflow in detail
2. **REQ-DOC-014.2** - Provide issue submission guidelines
3. **REQ-DOC-014.3** - Explain triage process and timelines
4. **REQ-DOC-014.4** - List issue templates and when to use each
5. **REQ-DOC-014.5** - Clarify that pull requests are not accepted
6. **REQ-DOC-014.6** - Provide code of conduct (respectful interaction)

**REQ-DOC-015: CHANGELOG.md**

If a CHANGELOG file exists, it SHALL:

1. **REQ-DOC-015.1** - Follow Keep a Changelog format (https://keepachangelog.com)
2. **REQ-DOC-015.2** - Group changes by version
3. **REQ-DOC-015.3** - Categorize changes: Added, Changed, Deprecated, Removed, Fixed, Security
4. **REQ-DOC-015.4** - Include release dates
5. **REQ-DOC-015.5** - Link to GitHub releases
6. **REQ-DOC-015.6** - Note AI agent involvement in each release

**REQ-DOC-016: FAQ.md**

If a FAQ file exists, it SHALL:

1. **REQ-DOC-016.1** - Address common user questions
2. **REQ-DOC-016.2** - Include troubleshooting tips
3. **REQ-DOC-016.3** - Explain AI development approach (if users ask)
4. **REQ-DOC-016.4** - Provide solutions to common errors
5. **REQ-DOC-016.5** - Link to relevant specification sections

## Documentation Quality Standards

**REQ-DOC-017: Writing Style**

All documentation SHALL follow these style guidelines:

1. **REQ-DOC-017.1** - Clear, concise language (avoid jargon unless necessary)
2. **REQ-DOC-017.2** - Active voice preferred ("Run the command" not "The command should be run")
3. **REQ-DOC-017.3** - Present tense for features ("tq provides" not "tq will provide")
4. **REQ-DOC-017.4** - Second person for instructions ("You can configure" not "One can configure")
5. **REQ-DOC-017.5** - Consistent terminology (use same term for same concept)
6. **REQ-DOC-017.6** - Code blocks with syntax highlighting
7. **REQ-DOC-017.7** - Proper Markdown formatting (headers, lists, tables)

**REQ-DOC-018: Accessibility**

Documentation SHALL be accessible:

1. **REQ-DOC-018.1** - Image alt text for all screenshots
2. **REQ-DOC-018.2** - Clear link text (not "click here")
3. **REQ-DOC-018.3** - Proper heading hierarchy (no skipped levels)
4. **REQ-DOC-018.4** - Tables with headers
5. **REQ-DOC-018.5** - Color not used as sole indicator of meaning

**REQ-DOC-019: Maintenance**

Documentation SHALL be kept up-to-date:

1. **REQ-DOC-019.1** - Update README when features are added
2. **REQ-DOC-019.2** - Update screenshots when UI changes significantly
3. **REQ-DOC-019.3** - Review documentation quarterly for accuracy
4. **REQ-DOC-019.4** - Fix broken links within 1 sprint of detection
5. **REQ-DOC-019.5** - Add "last updated" date to major documents

## Repository Organization

**REQ-DOC-020: Documentation Directory Structure**

The repository SHALL organize documentation clearly:

1. **REQ-DOC-020.1** - `README.md` - Project overview (root)
2. **REQ-DOC-020.2** - `LICENSE` - License and attributions (root)
3. **REQ-DOC-020.3** - `CONTRIBUTING.md` - Contribution guidelines (root)
4. **REQ-DOC-020.4** - `CHANGELOG.md` - Version history (root)
5. **REQ-DOC-020.5** - `docs/specifications/` - Feature specifications
6. **REQ-DOC-020.6** - `docs/design/` - Technical design documents
7. **REQ-DOC-020.7** - `docs/user/` - User guides
8. **REQ-DOC-020.8** - `docs/roadmap/` - Implementation status and planning
9. **REQ-DOC-020.9** - `docs/images/` - Screenshots and diagrams

## Acceptance Criteria

Documentation is complete when:

1. README.md includes all required sections in correct order
2. TLDR section provides instant value (screenshot, quick start)
3. AI development story is clear, accurate, and appropriately toned
4. Installation instructions are tested and verified
5. Usage examples demonstrate real value with realistic data
6. Features section is comprehensive and accurate
7. License section clearly communicates all license obligations
8. Trademark notices are present and legally appropriate
9. All links work and point to correct destinations
10. Documentation follows style guidelines consistently
11. Screenshots are current and high quality
12. New users can understand and install the tool in under 5 minutes

---

**Last Updated:** 2026-01-27
