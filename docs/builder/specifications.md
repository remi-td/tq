# tq (Teradata Query) - Specifications

**Version:** 1.5.0-dev
**Status:** Active Development - Sprint 7 (In Progress)
**Last Updated:** 2026-01-19

---

## Quick Navigation

- [Project Overview](#project-overview)
- [Feature Status Dashboard](#feature-status-dashboard)
- [Sprint Roadmap](#sprint-roadmap)
- [Detailed Specifications](#detailed-specifications)

---

## Project Overview

### Vision

`tq` is a best-in-class, lightweight command-line client for Teradata databases, designed to be fast, intuitive, and composable. It follows UNIX philosophy while providing a rich interactive experience comparable to `psql` for PostgreSQL.

### Goals

- **Simplicity**: Zero-configuration for basic use cases
- **Composability**: Works seamlessly in scripts and pipelines
- **Performance**: Fast startup, efficient query execution, minimal memory footprint
- **Security**: Secure credential handling, no password leaks
- **Cross-platform**: Works identically on Linux, macOS, and Windows
- **Self-contained**: Single static binary with no runtime dependencies

### Design Principles

1. **Convention over Configuration**: Sensible defaults for 80% of use cases
2. **Progressive Disclosure**: Simple things easy, complex things possible
3. **Fail Fast**: Clear error messages with actionable suggestions
4. **Respect UNIX Conventions**: `-h/--help`, `-V/--version`, stdin/stdout, exit codes
5. **Terminal Context Awareness**: Human output for TTY, machine output for pipes

### Architecture

- **Execution Model**: One-shot execution (connect → query → disconnect)
- **Language**: Rust (for performance and safety)
- **Driver**: Teradata teradatarustapi native driver
- **Authentication**: TD2, LDAP, Kerberos (KRB5), TDNEGO

---

## Feature Status Dashboard

### Core Features (MVP) ✅ Complete

| Feature | Status | Command | Priority |
|---------|--------|---------|----------|
| Single query execution | ✅ Implemented | `tq query "SELECT..."` | P0 |
| Connection testing | ✅ Implemented | `tq ping` | P0 |
| Multiple output formats | ✅ Implemented | `--format table\|json\|csv` | P0 |
| TD2 authentication | ✅ Implemented | `--logmech TD2` | P0 |
| LDAP authentication | ✅ Implemented | `--logmech LDAP` | P0 |
| Kerberos authentication | ✅ Implemented | `--logmech KRB5` | P0 |
| Connection string parsing | ✅ Implemented | `-l user:pass@host:port/db` | P0 |
| Environment variables | ✅ Implemented | `TQ_LOGON` | P0 |
| Password file support | ✅ Implemented | `--password-file` | P0 |
| Secure credential handling | ✅ Implemented | N/A | P0 |

### Interactive Mode (REPL)

#### Phase 1 - MVP Foundation ✅ Complete

| Feature | Status | Command | Priority |
|---------|--------|---------|----------|
| Interactive prompt | ✅ Implemented | `tq repl` | P0 |
| Multi-line SQL input | ✅ Implemented | Continue until `;` | P0 |
| Command history (in-memory) | ✅ Implemented | ↑/↓ arrows | P0 |
| `/session` metacommand | ✅ Implemented | `/session` | P0 |
| `/quit` metacommand | ✅ Implemented | `/quit` | P0 |
| `/help` metacommand | ✅ Implemented | `/help` | P0 |

#### Phase 2 - Enhanced REPL ✅ Complete (Sprint 4-5)

| Feature | Status | Command | Sprint | Priority |
|---------|--------|---------|--------|----------|
| `/describe` metacommand | ✅ Implemented | `/describe table` | 4 | P0 |
| `/ping` metacommand | ✅ Implemented | `/ping` | 4 | P0 |
| Persistent history | ✅ Implemented | Auto-saved to `~/.tq_history` | 4 | P1 |
| Vi keybindings | ✅ Implemented | `--editor-mode vi` | 4 | P1 |
| Emacs keybindings | ✅ Implemented | `--editor-mode emacs` | 4 | P1 |
| SQL syntax highlighting | ✅ Implemented | Auto-enabled in TTY | 5 | P1 |
| Result paging (horizontal) | ✅ Implemented | Wide tables | 5 | P1 |
| Result paging (vertical) | ✅ Implemented | Long results | 5 | P1 |
| Query timing display | ✅ Implemented | Show execution time | 5 | P1 |

#### Phase 3 - Advanced REPL ✅ Complete (Sprint 6)

| Feature | Status | Command | Sprint | Priority |
|---------|--------|---------|--------|----------|
| Table formatting fix | ✅ Sprint 6 | All table output | 6 | P0 |
| Tab completion (keywords) | ✅ Sprint 6 | Tab key | 6 | P1 |
| Tab completion (tables) | 🚧 Sprint 7 | Tab key | 7 | P0 |
| Tab completion (columns) | 🚧 Sprint 7 | Tab key | 7 | P1 |
| `/export` metacommand | ✅ Sprint 6 | `/export [format] [file]` | 6 | P1 |
| `/pager on\|off` metacommand | ✅ Sprint 6 | `/pager on\|off` | 6 | P2 |
| `/colors` metacommand | ✅ Sprint 6 | `/colors on\|off` | 6 | P2 |
| `/logon` metacommand | 🚧 Sprint 7 | `/logon [connection-string]` | 7 | P1 |

### Batch Mode 📋 Planned

| Feature | Status | Priority |
|---------|--------|----------|
| Execute from file | 📋 Planned | P0 |
| Read SQL from stdin | 📋 Planned | P0 |
| Output to stdout | ✅ Implemented | P0 |
| Output to file | 📋 Planned | P1 |
| Streaming large results | 📋 Planned | P1 |
| Multiple statement execution | 📋 Planned | P1 |
| Transaction control | 📋 Planned | P2 |
| Variable substitution | 📋 Planned | P2 |

### Configuration 📋 Planned

| Feature | Status | Priority |
|---------|--------|----------|
| User config file | 📋 Planned | P1 |
| Project config file | 📋 Planned | P1 |
| Connection profiles | 📋 Planned | P1 |
| Default format preference | 📋 Planned | P2 |
| Keyring integration | 📋 Planned | P2 |

**Legend:**
- ✅ Implemented and tested
- 🚧 In progress (current sprint)
- 📋 Planned (future sprint)
- 🔲 Deferred

---

## Sprint Roadmap

### Sprint 1-3: MVP ✅ Complete
**Goal:** Core functionality - query execution, connection testing, multiple output formats

**Delivered:**
- `tq query` command with table/JSON/CSV output
- `tq ping` command for connectivity testing
- TD2, LDAP, Kerberos authentication
- Connection string parsing and environment variables
- Password file support and secure credential handling

### Sprint 4: Interactive Mode Phase 1 - MVP ✅ Complete
**Goal:** Basic REPL functionality

**Delivered:**
- Interactive prompt with multi-line SQL support
- In-memory command history (↑/↓ navigation)
- `/session`, `/help`, `/quit` metacommands
- Graceful error handling and session management

**Sprint Review:** [Sprint 4 Review](../sprints/sprint-4-review.md)

### Sprint 5: Interactive Mode Phase 2 - Advanced Features ✅ Complete
**Goal:** Rich interactive experience with syntax highlighting and paging

**Delivered Features:**
- SQL syntax highlighting - Real-time color-coded SQL input
- Result paging (vertical) - Navigate long result sets with j/k/PageUp/Down
- Result paging (horizontal) - Scroll wide tables with h/l arrow keys
- Enhanced query timing - Detailed performance metrics display

**Status:** Complete
**Completion Date:** 2026-01-17
**Version Released:** v1.3.0

**Sprint Review:** [Sprint 5 Complete Review](../sprints/sprint-5-review.md)

### Sprint 6: Interactive Mode Phase 3 - Bug Fixes & Advanced Features ✅ Complete
**Goal:** Fix critical formatting bug and add tab completion, export, and display control features

**Delivered Features:**
1. **Critical Fix (P0):** Table formatting/padding bug fix (columns properly aligned)
2. **Tab Completion (P1):** SQL keyword auto-completion with Tab key (50+ keywords)
3. **Export Command (P1):** `/export [format] [file]` metacommand (JSON, CSV, SQL formats)
4. **Pager Control (P2):** `/pager on|off` metacommand for runtime pagination control
5. **Color Control (P2):** `/colors on|off` metacommand for syntax highlighting toggle

**Status:** Complete
**Completion Date:** 2026-01-18
**Version Released:** v1.4.0

**Sprint Review:** [Sprint 6 Complete Review](../sprints/sprint-6-review.md)

### Sprint 7: Interactive Mode Phase 4 - Database-Aware Features 🚧 In Progress
**Goal:** Enhance REPL with intelligent tab completion for database objects and dynamic connection management

**Planned Features:**
1. **Tab Completion for Table Names (P0)** - After FROM, JOIN, UPDATE keywords
2. **Tab Completion for Column Names (P1)** - After SELECT, WHERE, ORDER BY keywords
3. **`/logon` Metacommand (P1)** - Dynamic connection switching

**Status:** Design Phase (UX specifications complete)
**Target Completion:** 2026-01-20
**Version Target:** v1.5.0

**Sprint Planning:** [Sprint 7 Planning](../sprints/sprint-7-planning.md)

### Sprint 8+: Batch Mode & Configuration 📋 Future
**Goals:**
- File input (`--file`, stdin)
- Streaming large results
- Configuration files and connection profiles
- Additional completion features (functions, schemas)

---

## Detailed Specifications

Complete technical specifications are organized by domain:

### User & Design
- **[User Personas & Use Cases](detailed-specifications/user-personas.md)**
  Target users, usage patterns, and primary use cases

### Core Functionality
- **[CLI Interface Design](detailed-specifications/cli-interface.md)**
  Command structure, flags, help text, and command-line behavior

- **[REPL Mode](detailed-specifications/repl-mode.md)**
  Interactive mode specifications, metacommands, keybindings, and features

- **[Batch Mode](detailed-specifications/batch-mode.md)**
  Non-interactive execution, file input, piping, and scripting integration

### Configuration & Security
- **[Configuration Management](detailed-specifications/configuration.md)**
  Config files, connection profiles, environment variables, and credential management

- **[Security Requirements](detailed-specifications/security.md)**
  Password handling, credential storage, permissions, and security best practices

### Output & Error Handling
- **[Output Formats](detailed-specifications/output-formats.md)**
  Table, JSON, CSV formatting and terminal-aware output

- **[Error Handling](detailed-specifications/error-handling.md)**
  Error messages, troubleshooting guidance, and exit codes

### Performance
- **[Performance Considerations](detailed-specifications/performance.md)**
  Startup time, query execution, memory usage, and streaming

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-19 | 1.5.0-dev | Sprint 7 design phase: marked table/column completion and /logon as 🚧 In Progress | CLI UX Designer Agent |
| 2026-01-18 | 1.2.0 | Restructured into main spec + detailed specs | CLI UX Designer Agent |
| 2026-01-17 | 1.1.0 | Added Sprint 4 specifications | CLI UX Designer Agent |
| 2026-01-16 | 1.0.0 | Initial comprehensive specifications | Development Team |
| 2026-01-10 | 0.1.0 | Initial MVP specifications | Development Team |

---

## How to Use This Document

### For Product Planning
- Review the [Feature Status Dashboard](#feature-status-dashboard) for current implementation status
- Check the [Sprint Roadmap](#sprint-roadmap) for upcoming features and priorities
- Reference detailed specs when planning new features

### For Development
- Start with the relevant detailed specification document
- Follow the specifications exactly as written
- Propose updates to specs when you identify gaps or improvements
- Update sprint roadmap when features are completed

### For Testing
- Use feature status dashboard to identify what needs testing
- Reference detailed specs for acceptance criteria
- Update status when features pass validation

### For Documentation
- Link to detailed specs from user documentation
- Keep examples aligned with specifications
- Update help text to match CLI interface specs
