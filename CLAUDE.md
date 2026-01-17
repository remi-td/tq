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

### Master specification documents

The project is governed by three authoritative specification documents located in `docs/builder/`:

1. **`specifications.md`** - Master specifications
   - Documents all current and future features for the tool
   - Owned by the `cli-ux-designer` agent
   - Defines WHAT the tool should do and HOW users interact with it

2. **`rust-cli-design-general.md`** - General Rust CLI design guidelines
   - General Rust CLI design principles and best practices
   - Owned by the `rust-teradata-architect` agent
   - Provides patterns and principles for CLI tool development

3. **`rust-architecture.md`** - Rust architecture for tq
   - Architecture document specific to the tq tool
   - Owned by the `rust-teradata-architect` agent
   - Defines HOW the tool is implemented internally

4. **`testing-guidelines.md`** - Testing methodology and best practices
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