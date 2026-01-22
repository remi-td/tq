# tq (Teradata Query) - Specifications

**Version:** 1.7.0 (Sprint 19 Complete)
**Status:** Active Development - Sprint 19 Bug Fixes Complete
**Last Updated:** 2026-01-22

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
| Single query execution | ✅📝 Implemented and tested | `tq query "SELECT..."` | P0 |
| Connection testing | ✅📝 Implemented and tested | `tq ping` | P0 |
| Multiple output formats | ✅📝 Implemented and tested | `--format table\|json\|csv` | P0 |
| TD2 authentication | ✅📝 Implemented and tested | `--logmech TD2` | P0 |
| LDAP authentication | ✅📝 Implemented and tested | `--logmech LDAP` | P0 |
| Kerberos authentication | ✅📝 Implemented and tested | `--logmech KRB5` | P0 |
| Connection string parsing | ✅📝 Implemented and tested | `-l user:pass@host:port/db` | P0 |
| Environment variables | ✅📝 Implemented and tested | `TQ_LOGON` | P0 |
| Password file support | ✅📝 Implemented and tested | `--password-file` | P0 |
| Secure credential handling | ✅📝 Implemented and tested | N/A | P0 |

### Interactive Mode (REPL)

#### Phase 1 - MVP Foundation ✅ Complete

| Feature | Status | Command | Priority |
|---------|--------|---------|----------|
| Interactive prompt | ✅📝 Implemented and tested | `tq repl` | P0 |
| Multi-line SQL input | ✅📝 Implemented and tested | Continue until `;` | P0 |
| Command history (in-memory) | ✅📝 Implemented and tested | ↑/↓ arrows | P0 |
| `/session` metacommand | ✅📝 Implemented and tested | `/session` | P0 |
| `/quit` metacommand | ✅📝 Implemented and tested | `/quit` | P0 |
| `/help` metacommand | ✅❓ Implemented, needs testing | `/help` | P0 |

#### Phase 2 - Enhanced REPL (Sprint 4-5)

| Feature | Status | Command | Sprint | Priority |
|---------|--------|---------|--------|----------|
| `/describe` metacommand | ✅❓ Implemented, needs testing | `/describe table` | 4 | P0 |
| `/ping` metacommand | ✅❓ Implemented, needs testing | `/ping` | 4 | P0 |
| Persistent history | ✅❓ Implemented, needs testing | Auto-saved to `~/.tq_history` | 4 | P1 |
| Vi keybindings | ✅📝 Implemented and tested | `--editor-mode vi` | 4 | P1 |
| Emacs keybindings | ✅📝 Implemented and tested | `--editor-mode emacs` | 4 | P1 |
| SQL syntax highlighting | ✅📝 Implemented and tested | Auto-enabled in TTY | 5 | P1 |
| Result paging (horizontal) | ✅📝 Implemented and tested | Wide tables | 5 | P1 |
| Result paging (vertical) | ✅📝 Implemented and tested | Long results | 5 | P1 |
| Query timing display | ✅📝 Implemented and tested | Show execution time | 5 | P1 |

#### Phase 3 - Advanced REPL (Sprint 6-7)

| Feature | Status | Command | Sprint | Priority |
|---------|--------|---------|--------|----------|
| Table formatting | ✅📝 Implemented and tested | All table output | 6 | P0 |
| Tab completion (keywords) | ✅📝 Implemented and tested | Tab key | 6 | P1 |
| Tab completion (tables) | ✅📝 Implemented and tested | Tab key | 7,13 | P0 |
| Tab completion (columns) | ✅❓ Implemented, needs testing | Tab key | 7,13 | P1 |
| Tab completion (multi-line) | ✅📝 Implemented and tested | Across line breaks | 9,13 | P0 |
| `/export` metacommand | ✅📝 Implemented and tested | `/export <format> [dest]` | 12,13 | P1 |
| `/pager on\|off` metacommand | ✅📝 Implemented and tested | `/pager on\|off` | 6 | P2 |
| `/colors` metacommand | ✅📝 Implemented and tested | `/colors on\|off` | 6 | P2 |
| `/logon` metacommand | ✅❓ Implemented, needs testing | `/logon [connection-string]` | 7 | P1 |

#### Phase 4 - Quality & Branding (Sprint 11-13)

| Feature | Status | Command | Sprint | Priority |
|---------|--------|---------|--------|----------|
| Professional branding | ✅📝 Implemented and tested | Logo, colors, naming | 12,13 | P0 |
| Interactive test framework | ✅📝 Implemented and tested | expectrl-based tests | 13 | P0 |
| Export syntax simplification | ✅📝 Implemented and tested | `/export <fmt> [dest]` | 13 | P1 |
| Export to clipboard | ✅📝 Implemented and tested | `/export <fmt> clipboard` | 12 | P1 |
| Export full dataset | ✅📝 Implemented and tested | Re-execute without limit | 12 | P1 |

### Batch Mode (Sprint 10) ✅ Foundation Complete

| Feature | Status | Sprint | Priority |
|---------|--------|--------|----------|
| Execute from file | ✅📝 Implemented and tested | 10 | P0 |
| Read SQL from stdin | ✅📝 Implemented and tested | 10 | P0 |
| Output to stdout | ✅📝 Implemented and tested | 10 | P0 |
| Multiple statement execution | ✅📝 Implemented and tested | 10 | P0 |
| Enhanced error messages | ✅📝 Implemented and tested | 10 | P1 |
| Batch mode output behavior | ✅📝 Implemented and tested | 10 | P1 |
| Output to file | 📋 Planned | 11+ | P1 |
| Streaming large results | 📋 Planned | 11+ | P1 |
| Transaction control | 📋 Planned | 11+ | P2 |
| Variable substitution | 📋 Planned | 11+ | P2 |

### Configuration (Sprint 16-17)

| Feature | Status | Priority | Sprint |
|---------|--------|----------|--------|
| User config file (`~/.tq/config.toml`) | ✅📝 Implemented and tested | P1 | 16 |
| Connection profiles | ✅📝 Implemented and tested | P1 | 16 |
| Default preferences (format, editor_mode, etc) | ✅📝 Implemented and tested | P1 | 16 |
| `--profile <name>` flag | ✅📝 Implemented and tested | P2 | 16 |
| `tq help config` subcommand | 🚧 In Progress | P0 | 17 |
| `tq help credentials` subcommand | 🚧 In Progress | P0 | 17 |
| `tq profiles` command | 🚧 In Progress | P1 | 17 |
| Password file permission enforcement | 🚧 In Progress | P1 | 17 |
| Security check ordering fix | 🚧 In Progress | P0 | 17 |
| Project config file (`.tq.toml`) | 📋 Planned | P1 | 18+ |
| Profile editing commands | 📋 Planned | P1 | 18+ |
| Keyring integration | 📋 Planned | P2 | 18+ |
| Config validation command | 📋 Planned | P2 | 18+ |

### Help System (Sprint 17) 🚧 In Progress

| Feature | Status | Priority |
|---------|--------|----------|
| `tq help` (general) | ✅📝 Implemented and tested | P0 |
| `tq help config` subcommand | 🚧 In Progress | P0 |
| `tq help credentials` subcommand | 🚧 In Progress | P0 |
| Help topic routing | 🚧 In Progress | P0 |

**Legend:**
- ✅📝 Implemented and tested
- ✅❓ Implemented, testing incomplete
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

### Sprint 7: Interactive Mode Phase 4 - Database-Aware Features (Quality Issues Found)
**Goal:** Enhance REPL with intelligent tab completion for database objects and dynamic connection management

**Delivered Features:**
1. **Tab Completion for Table Names (P0)** - Complete after FROM, JOIN, UPDATE keywords using database metadata
2. **Tab Completion for Column Names (P1)** - Complete after SELECT, WHERE, ORDER BY with SQL context awareness
3. **`/logon` Metacommand (P1)** - Dynamic connection switching without restarting REPL

**Status:** 🔧 Features Do Not Work - In Repair (Sprint 8)
**Completion Date:** 2026-01-18 (marked complete prematurely)
**Version Released:** v1.5.0

**Sprint Review:** [Sprint 7 Review](../sprints/sprint-7-review.md)

**Quality Issue:** Features passed unit tests but fail against real Teradata databases. Manual testing was not performed.

### Sprint 8: Quality Recovery - Critical Bug Fixes (Partial)
**Goal:** Fix all critical bugs from Sprints 5-7 and restore user trust through mandatory live database testing

**Status:** Partially Complete - Continued in Sprint 9
**Completion Date:** 2026-01-18
**Version:** v1.5.0 (partial fixes)

**Sprint Review:** [Sprint 8 Review](../sprints/sprint-8-review.md)

---

### Sprint 9: Complete Quality Recovery ✅ Complete
**Goal:** Complete all remaining bug fixes from Sprint 8 with autonomous execution

**Bugs Fixed:**
1. ✅ Tab completion menu size (shows 25 items, not 9)
2. ✅ Multi-line tab completion (context preserved across lines)
3. ✅ Error messages (clean SQL errors, no Go stack traces)
4. ✅ LIMIT hint message (uses Teradata TOP/SAMPLE syntax)
5. ✅ Result paging re-enabled (with Sprint 8 fixes)
6. ✅ Build warnings cleaned up (zero warnings)

**Status:** ✅ Complete
**Completion Date:** 2026-01-18
**Version Released:** v1.5.1

**Sprint Planning:** [Sprint 9 Planning](../sprints/sprint-9-planning.md)
**Sprint Review:** [Sprint 9 Review](../sprints/sprint-9-review.md)

**Key Achievement:** 100% bug fix completion with autonomous execution

---

### Sprint 10: Batch Mode Foundation ✅ Complete
**Goal:** Implement foundational batch mode features for scripting and automation

**Delivered Features:**
1. **stdin Input Support (P0)** - Pipe SQL queries from stdin (`cat query.sql | tq query`)
2. **File Input Support (P0)** - Execute SQL scripts from files (`tq query --file script.sql`)
3. **Multiple Statement Execution (P0)** - Sequential execution with fail-fast error handling
4. **Enhanced Error Messages (P1)** - Statement number, line tracking, contextual errors
5. **Batch Mode Output Behavior (P1)** - Appropriate defaults for scripting use cases

**Status:** ✅ Complete
**Completion Date:** 2026-01-18
**Version Released:** v1.6.0

**Test Results:**
- Unit Tests: 195/195 passed (100%)
- Integration Tests: 37/37 passed (100%)
- Code Coverage: >95% for new code
- Build Warnings: 0

**Sprint Planning:** [Sprint 10 Planning](../sprints/sprint-10-planning.md)
**Sprint Review:** [Sprint 10 Review](../sprints/sprint-10-review.md)

**Key Achievement:** Full batch mode foundation enabling scripting, automation, and CI/CD integration

---

### Sprint 11: Critical Quality Recovery - Table Display & Tab Completion ✅ Complete
**Goal:** Fix critical regressions in tab completion and table display to restore user trust and tool usability

**Critical Fixes (P0):**
1. ✅ **Table Display Regression** - Padding completely removed, terminal-width-aware truncation implemented
2. ✅ **Pager Disabled** - No more panning mode per user directive
3. ✅ **Tab Completion Fixed** - Removed keyword fallback, shows context-aware completions

**Status:** ✅ Code Complete (user validation pending)
**Completion Date:** 2026-01-18
**Version Released:** v1.6.1

**Key Achievements:**
- **246/246 tests passing (100%)**
- **Zero technical debt**
- **Pager completely disabled** - Direct output, no paging intermediate
- **Table rewritten** - Clean, simple truncation algorithm (430 lines)
- **30 new table tests** - Comprehensive coverage of terminal widths, batch mode, edge cases
- **6 new completion tests** - No keyword fallback validation

**Root Cause Found:**
- Bugs NOT introduced in Sprint 10 (falsely accused)
- Existed earlier, not caught by test coverage gaps
- Interactive features need interactive tests (expectrl)

**Sprint Planning:** [Sprint 11 Planning](../sprints/sprint-11-planning.md)
**Sprint Review:** [Sprint 11 Review](../sprints/sprint-11-review.md)

**Key Achievement:** Responsive autonomous execution with user feedback, both bugs fixed at code level

---

### Sprint 12: Export Enhancements & Branding ✅ Complete
**Goal:** Add high-value export features and professional branding for client presentations

**Delivered Features:**
1. **Export to Clipboard (P1)** - Copy results directly to system clipboard
   - Supports table, CSV, JSON, sql formats
   - Cross-platform support (macOS, Linux, Windows) via arboard
   - Graceful error handling for unavailable clipboard

2. **Full Dataset Export (P1)** - Export complete dataset to files
   - Re-executes query without row limit for file exports
   - Respects user-specified limits (TOP, SAMPLE)
   - Clear messaging on export behavior

3. **Professional Branding (P1)** - ASCII logo and Teradata orange
   - Welcome banner on REPL startup
   - Teradata orange color (#F37021) throughout
   - Session information display
   - Professional appearance for presentations

4. **Process Improvement** - Proper deployment workflow
   - Version bumped: 1.6.0 → 1.6.1
   - Binary rebuilt and verified
   - Sprint 11 fixes now properly deployed

**Status:** ✅ Complete
**Completion Date:** 2026-01-19
**Version Released:** v1.6.1

**Test Results:**
- Unit Tests: 216/216 passed (100%)
- Integration Tests: 37/37 passed (100%)
- Code Quality: Excellent (4 minor cosmetic warnings)
- Build Warnings: 4 (cosmetic, non-blocking)

**Sprint Planning:** [Sprint 12 Planning](../sprints/sprint-12-planning.md)
**Sprint Review:** [Sprint 12 Review](../sprints/sprint-12-review.md)

**Key Achievement:** User-requested export enhancements shipped with professional branding for client presentations

---

### Sprint 13: Tab Completion Context & Branding Fixes ✅ Complete
**Goal:** Fix critical tab completion context awareness and branding issues with proper interactive testing

**Sprint Theme:** "Test What Users See, Not Just What Code Does"

**Status:** ✅ Complete
**Start Date:** 2026-01-19
**Completion Date:** 2026-01-19
**Version Released:** v1.6.1

**Delivered Features:**

1. **Tab Completion Context Awareness (P0)** - Fixed after 5 sprints
   - Keyword abbreviation recognition (sel→SELECT, fr→FROM)
   - Context-aware completions (databases/tables after FROM, not keywords)
   - Empty prefix no longer shows all keywords

2. **Professional Branding (P0)** - Correct implementation
   - Logo displays 'tq' (lowercase) not 'Teradata Query Tool'
   - Prompt uses correct Teradata orange RGB(243,112,33)
   - Branding Guidelines v2.0.0 specification complete

3. **Interactive Test Framework (P0)** - Foundation established
   - expectrl-based test infrastructure
   - 14 interactive tests for tab completion validation
   - Tests verify content semantics, not just mechanisms

4. **Export Syntax Simplification (P1)** - Completed
   - `/export <format> [destination]` syntax
   - Supports file paths and 'clipboard' keyword

**Test Results:**
- Unit Tests: 216/216 passed (100%)
- Interactive Tests: 14/14 passed (100%)
- Code Coverage: Comprehensive for REPL features

**Sprint Review:** [Sprint 13 Commit](https://github.com/.../commit/2f369bc) - Full details in git history

**Key Achievement:** Tab completion issues from Sprints 7-12 fully resolved with interactive test validation

---

### Sprint 14: Quality Infrastructure Foundation ✅ Complete (Maintenance Sprint)
**Goal:** Establish quality infrastructure for interactive features to prevent UX regressions

**Status:** ✅ Complete
**Completion Date:** 2026-01-21
**Type:** Maintenance Sprint (Crisis Response)

**Delivered:**
1. ✅ **Clean Build Foundation** - Fixed 21 build warnings, enforced `#![deny(warnings)]`
2. ✅ **Specification Synchronization** - Resolved Sprint 13 confusion, audited all specs
3. ✅ **Test Infrastructure Documentation** - Created tests/README.md, comprehensive test guide
4. ✅ **Sprint 13 Validation** - Validated 253/253 tests passing, identified test gaps
5. ✅ **Process Documentation** - Created DoD, testing-checklist.md, updated testing-guidelines.md

**Quality Metrics:**
- Unit Tests: 216/216 passing (100%)
- Integration Tests: 37/37 passing (100%)
- Build Warnings: 0 (down from 21)
- Technical Debt: 0 new debt

**Key Achievement:** Transformed stuck issue (interactive test framework) into operational quality infrastructure foundation.

**Sprint Planning:** [Sprint 14 Planning](../sprints/sprint-14-planning.md)
**Sprint Review:** [Sprint 14 Review](../sprints/sprint-14-review.md)
**Crisis Deliberation:** [Sprint 14 Deliberation](../sprints/sprint-14-crisis-deliberation.md)

---

### Sprint 15: Sprint 13 Validation & Test Infrastructure Enhancement ✅ Complete
**Goal:** Complete Sprint 13 feature validation by adding missing tests, generating coverage baseline, and establishing comprehensive test infrastructure for REPL features

**Status:** ✅ Complete
**Completion Date:** 2026-01-21
**Type:** Feature Sprint (with validation focus)

**Objectives Delivered:**
1. **P0: Complete Sprint 13 Test Coverage** ✅ - Added 5 interactive tests (354 lines)
   - ✅ `/help` metacommand test
   - ✅ History persistence test
   - ✅ Multi-line history preservation test
   - ✅ SQL error format test
   - ✅ Column completion test

2. **P0: Coverage Baseline Generation** ✅ - Installed cargo-tarpaulin, baseline established
   - Baseline: 40.07% (1384/3454 lines) - informational
   - HTML coverage report generated

3. **P1: Documentation Improvements** ✅ - All P0 fixes from Sprint 14 UX review complete
   - ✅ Add implementation status badges to repl-mode.md
   - ✅ Add test status indicators to specifications.md
   - ✅ Add Quick Start section to testing-checklist.md

4. **P1: Test Infrastructure Validation** ✅ - Quality gates validated and operational

**Key Achievement:** Closed Sprint 13 validation gap. All Sprint 13 features now fully tested.

**Sprint Planning:** [Sprint 15 Planning](../sprints/sprint-15-planning.md)
**Sprint Review:** [Sprint 15 Review](../sprints/sprint-15-review.md)

---

### Sprint 16: Interactive Test Validation & Configuration Foundation ✅ Complete
**Goal:** Validate Sprint 13-15 interactive tests with live database, then establish configuration file foundation for connection profiles and user preferences

**Sprint Theme:** "Validation First, Then Configuration" - Complete test validation work from Sprint 15, then return to feature development with full confidence.

**Status:** ✅ Complete
**Start Date:** 2026-01-21
**Completion Date:** 2026-01-21

**Delivered:**
1. ✅ Interactive Test Execution Validation - All 20 interactive tests passing with live database
2. ✅ Coverage Metrics Documentation - Automated vs total coverage documented
3. ✅ User Configuration File - `~/.tq/config.toml` with connection profiles and defaults
4. ✅ Configuration Specification Completion - Detailed specification complete (v2.0.0)
5. ✅ Profile Selection CLI Flag - `--profile <name>` flag implemented and tested

**Test Results:**
- Unit Tests: 272/272 passed (100%)
- Integration Tests: All passed
- Code Quality: Zero warnings
- Technical Debt: Zero

**Sprint Planning:** [Sprint 16 Planning](../sprints/sprint-16-planning.md)
**Sprint Review:** [Sprint 16 Review](../sprints/sprint-16-review.md)

---

### Sprint 17: Configuration UX Completion ✅ Complete
**Goal:** Complete the configuration user experience by implementing help subcommands, fixing security issues, and adding profile management commands

**Sprint Theme:** "Configuration UX Polish" - Building on Sprint 16's configuration foundation to deliver a complete, secure, and user-friendly configuration experience.

**Status:** ✅ Complete (2026-01-21)
**Quality:** 9.5/10 (Exceptional)

**Delivered:**
1. ✅ **Help Subcommands** - `tq help config` and `tq help credentials` with comprehensive documentation
2. ✅ **Security Check Ordering Fix** - Fixed security check ordering in password file reading
3. ✅ **Password File Permission Enforcement** - Changed from warning to error for files with permissions != 0600 (breaking change)
4. ✅ **Profile Listing Command** - Added `tq profiles` command to list available profiles
5. ✅ **Logmech Parsing Refactoring** - Eliminated code duplication

**Key Achievement:** First sprint to deliver comprehensive help system with embedded content. 100% test pass rate (285/285 tests). Zero technical debt.

**Sprint Planning:** [Sprint 17 Planning](../sprints/sprint-17-planning.md)
**Sprint Review:** [Sprint 17 Review](../sprints/sprint-17-review.md)

---

### Sprint 18: Critical Bug Fixes - Logo & Tab Completion ✅ Complete
**Goal:** Fix two critical user-facing bugs blocking productive use

**Sprint Type:** Maintenance Sprint (Crisis)
**Status:** ✅ Complete (2026-01-22)
**Note:** Partially reverted in Sprint 19 (logo implementation incorrect)

**Delivered:**
1. ✅ **Logo Fix (INCORRECT)** - Changed to plain text "tq" (user wanted ASCII art, not plain text)
2. ✅ **Tab Completion Fix** - Rebuilt tab completion system, removed keyword completion, fixed span calculation

**Issue:** Sprint 18 misinterpreted user requirement for logo. User wanted "ASCII art in lowercase" but Sprint 18 delivered "plain text lowercase". This was corrected in Sprint 19.

**Sprint Planning:** [Sprint 18 Planning](../sprints/sprint-18-planning.md)

---

### Sprint 19: Critical Bug Fixes - Logo & Tab Completion (Retry) ✅ Complete
**Goal:** Correct Sprint 18 miscommunication and fully resolve both critical bugs

**Sprint Type:** Maintenance Sprint (Sprint 18 Retry)
**Status:** ✅ Complete (2026-01-22)
**Quality:** 9/10 (Excellent)

**Delivered:**
1. ✅ **Logo Fix (CORRECT)** - Implemented lowercase ASCII art "tq" with info messages to the RIGHT of logo (as user requested)
2. ✅ **Tab Completion Fix (COMPLETE)** - Added StdoutSuppressor to prevent teradatarustapi debug output during completions

**Key Achievement:** Correctly interpreted user requirements. Logo now displays lowercase ASCII art "tq" with 't' in Teradata orange and information messages positioned to the right of the logo on the same lines.

**UX Rating:** 9/10 (Excellent)

**Sprint Planning:** [Sprint 19 Planning](../sprints/sprint-19-planning.md)
**Sprint Review:** [Sprint 19 UX Review](../sprints/sprint-19-ux-review.md)

---

### Sprint 20+: Advanced Features 📋 Future
**Goals:**
- Transaction control (`--atomic` flag)
- Variable substitution
- Streaming large results
- Project-level config file (`.tq.toml`)
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

### Branding & Visual Identity
- **[Branding Guidelines](detailed-specifications/branding-guidelines.md)** 🚧
  Logo design, color specifications, naming conventions, and terminal rendering standards
  - v2.0.0: Complete unambiguous specification with exact logo design and prompt colors

### Performance
- **[Performance Considerations](detailed-specifications/performance.md)**
  Startup time, query execution, memory usage, and streaming

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-21 | 1.7.0-dev | Sprint 17 Phase 2: Marked Sprint 16 as Complete, added Sprint 17 roadmap section, marked help subcommands and profiles command as 🚧 In Progress, added Help System section | cli-ux-designer |
| 2026-01-21 | 1.6.1 | Sprint 16 Phase 2: Configuration features marked 🚧 In Progress, added Sprint 16 roadmap section with detailed objectives, updated version status | cli-ux-designer |
| 2026-01-21 | 1.6.1 | Sprint 15 Complete: Marked Sprint 15 as ✅ Complete with all objectives delivered, updated status to Ready for Sprint 16, added Sprint 15 Review link | Sprint Coordinator |
| 2026-01-21 | 1.6.1 | Sprint 15 Phase 2: Added test status indicators (✅📝 implemented+tested, ✅❓ needs testing), marked Sprint 15 as In Progress, added Sprint 15 roadmap section | CLI UX Designer Agent |
| 2026-01-21 | 1.6.1 | Sprint 14 Maintenance: Resolved Sprint 13 confusion, marked Sprint 13 as Complete, updated feature statuses to reflect reality, added Sprint 14 roadmap | CLI UX Designer Agent |
| 2026-01-19 | 1.7.0-dev | Sprint 13 Phase 2: Branding Guidelines v2.0.0 complete, export syntax design complete, features marked 🚧 | CLI UX Designer Agent |
| 2026-01-18 | 1.6.0 | Sprint 11: Mark table display and tab completion as 🔧 In Repair, add Sprint 11 roadmap section | CLI UX Designer Agent |
| 2026-01-18 | 1.5.1-dev | Sprint 8: Mark broken features as 🔧 In Repair, add Sprint 8 roadmap, add 🔧 to legend | CLI UX Designer Agent |
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
