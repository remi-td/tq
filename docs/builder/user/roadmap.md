# tq Roadmap

**Last Updated:** 2026-01-17
**Version:** 1.3.0

---

## Current Sprint: Sprint 6 (Interactive Mode Phase 3)

**Completed:** Sprint 5
**In Progress:** Sprint 6
**Target Completion:** Q2 2026

Sprint 6 focuses on tab completion, export functionality, and connection switching.

### Released Features

| Feature | Description | Status |
|---------|-------------|--------|
| `tq ping` | Test database connectivity with timing | Shipped |
| `tq query` | Execute SQL queries from argument, file, or stdin | Shipped |
| `tq repl` | Interactive REPL mode for database exploration | Shipped |
| Table output | Human-readable table formatting | Shipped |
| JSON output | Machine-parseable JSON format | Shipped |
| CSV output | RFC 4180 compliant CSV format | Shipped |
| TD2 authentication | Teradata native authentication | Shipped |
| LDAP authentication | LDAP directory authentication | Shipped |
| KRB5 authentication | Kerberos authentication | Shipped |
| Connection string parsing | `user:password@host:port/database` format | Shipped |
| Environment variables | `TQ_LOGON`, `TQ_LOGMECH`, etc. | Shipped |
| Password file support | Secure credential storage | Shipped |
| Configuration file | TOML-based user configuration | Shipped |
| Multi-line SQL input | Statements accumulate until semicolon | Shipped |
| Command history | In-memory history with arrow key navigation | Shipped |
| `/help` command | Display help within REPL | Shipped |
| `/quit` command | Exit REPL cleanly | Shipped |
| `/session` command | Display session information | Shipped |
| Ctrl-C handling | Cancel current input gracefully | Shipped |
| Default 100-row limit | REPL auto-limits SELECT queries (configurable) | Shipped |
| Actual column names | Query results show real column names from database | Shipped |
| `/describe` metacommand | Show table structure (columns, types, nullable) | Shipped |
| `/ping` metacommand | Test connection within REPL with latency display | Shipped |
| Persistent history | Save command history to `~/.tq_history` | Shipped |
| Vim/Emacs keybindings | Configurable editor modes (--editor-mode flag) | Shipped |
| SQL syntax highlighting | Color-coded SQL keywords, strings, numbers, comments | Shipped |
| Enhanced timing display | Detailed query timing breakdown with rows/second | Shipped |
| Vertical result paging | Navigate long result sets with j/k, PageUp/Down | Shipped |
| Horizontal result paging | Scroll wide tables with h/l, arrow keys | Shipped |

### Technical Notes

- Uses `reedline` crate for line editing (same as Nushell)
- Persistent history saved to `~/.tq_history` with FileBackedHistory
- Editor mode configurable via `--editor-mode` flag or `TQ_EDITOR_MODE` env var
- Table output format for results
- Shows timing for each query
- Column metadata fetched from Teradata API for accurate column names
- Default 100-row limit for SELECT queries without explicit LIMIT (configurable via `--default-limit`)
- SQL syntax highlighting uses nu-ansi-term for terminal colors
- Result paging uses crossterm for terminal control
- Disable highlighting with `--no-syntax-highlight`
- Disable paging with `--no-pager`
- Enable enhanced timing with `--enhanced-timing`

---

## Sprint 5: Interactive Mode Phase 2 - Advanced Features (COMPLETED)

**Status:** Completed
**Completed:** 2026-01-17

### Goals

| Feature | Priority | Status | Description |
|---------|----------|--------|-------------|
| SQL syntax highlighting | P1 | Completed | Color-coded SQL input for keywords, strings, numbers, comments |
| Enhanced timing display | P1 | Completed | Detailed query timing with breakdown and rows/second |
| Vertical result paging | P2 | Completed | Navigate long result sets with less-like controls |
| Horizontal result paging | P2 | Completed | Scroll wide tables with arrow keys |

### Sprint 5 Success Criteria (All Met)
1. SQL input is syntax highlighted with keywords in cyan, strings in green, numbers in yellow
2. Comments (-- and /* */) are displayed in gray italic
3. Functions (COUNT, SUM, etc.) are displayed in magenta
4. Timing information shows total time, and optionally first-row latency and rows/second
5. Large result sets can be navigated with j/k, PageUp/Down, and arrow keys
6. Wide tables can be scrolled horizontally with h/l and arrow keys
7. All existing tests continue to pass (126 unit tests, 37 integration tests)
8. New unit tests added for syntax highlighting (13 tests) and paging (5 tests)

### Sprint 5 Retrospective Summary
- **Features Delivered:** 4/4 (100%)
- **Quality Assessment:** 4/5 (Good)
- **Technical Debt:** Zero introduced
- **Key Achievement:** Significant UX improvement with syntax highlighting and result paging
- **Full Review:** See [Sprint 5 Review](../sprints/sprint-5-review.md)

---

## Sprint 4: Interactive Mode Phase 2 - Foundation Features (COMPLETED)

**Status:** Completed
**Completed:** 2026-01-17

### Goals

| Feature | Priority | Status | Description |
|---------|----------|--------|-------------|
| `/describe` metacommand | P0 | Completed | Describe table structure (columns, types, nullable) |
| `/ping` metacommand | P0 | Completed | Test connection within REPL with latency display |
| Persistent history | P1 | Completed | Save command history to `~/.tq_history` |
| Vim/Emacs keybindings | P1 | Completed | Configurable editor modes (--editor-mode flag) |

### Sprint 4 Success Criteria (All Met)
1. `/describe <table>` shows table structure with column names, types, and nullable status
2. `/ping` tests database connection and displays latency
3. Command history persists across REPL sessions
4. Users can switch between vim and emacs keybinding modes
5. All existing tests continue to pass (104 unit tests)
6. New unit and integration tests added for new features

---

## Next Up: Interactive Mode (Phase 3)

**Planned:** Q2 2026

| Feature | Priority | Description |
|---------|----------|-------------|
| Tab completion (keywords) | P1 | Complete SQL keywords |
| Tab completion (tables) | P1 | Complete table names from schema |
| Tab completion (columns) | P2 | Complete column names |
| `/export` command | P1 | Export last result to file |
| `/logon` command | P1 | Switch connections within REPL |
| Theming | P2 | Customizable color schemes |

---

## Backlog

Features under consideration for future releases:

| Feature | Notes |
|---------|-------|
| Multiple statement execution | Execute multiple SQL statements from file |
| Transaction control | BEGIN/COMMIT/ROLLBACK support |
| Variable substitution | Template queries with parameters |
| Query caching | Cache frequently used queries |
| SSL/TLS support | Encrypted connections |
| Keyring integration | OS-native credential storage |
| Shell completions | bash/zsh/fish completion scripts |
| Homebrew formula | Easy installation on macOS |

---

## Rejected / Won't Implement

Features that have been considered and rejected:

| Feature | Reason |
|---------|--------|
| Multiple background sessions | Adds significant complexity; users can run multiple tq instances |
| GUI interface | Out of scope - tq is CLI-only by design |
| Connection pooling | Not needed for one-shot execution model |
| Data transformation | Use external tools (jq, csvkit) per UNIX philosophy |
| Schema migration | Use dedicated tools (Liquibase, Flyway) |

---

## Release History

### v1.3.0 (2026-01-17) - Interactive Mode Phase 2 Advanced

**Sprint 5 Highlights:**
- **SQL Syntax Highlighting**: Real-time color-coded SQL input
  - Keywords (SELECT, FROM, WHERE, etc.) in cyan bold
  - String literals in green
  - Numbers in yellow
  - Comments (-- and /* */) in gray italic
  - Functions (COUNT, SUM, AVG, etc.) in magenta
  - Supports Teradata-specific keywords (QUALIFY, SEL, TOP, SAMPLE)
  - Disable with `--no-syntax-highlight` flag
- **Enhanced Query Timing**: Detailed timing breakdown
  - Total execution time
  - First row latency (when available)
  - Transfer time (when available)
  - Rows per second throughput
  - Enable with `--enhanced-timing` flag
- **Result Paging**: Navigate large result sets interactively
  - Vertical paging for long results (j/k, PageUp/Down, Space)
  - Horizontal scrolling for wide tables (h/l, arrow keys)
  - Status line shows position and scroll indicators
  - less-like navigation with q to quit
  - Disable with `--no-pager` flag

**New Dependencies:**
- nu-ansi-term 0.50: Terminal color support
- crossterm 0.28: Terminal input/output control
- unicode-width 0.2: Unicode character width calculation

**New Tests:**
- 13 new unit tests for syntax highlighting
- 5 new unit tests for result paging
- Tests cover keyword detection, number parsing, string handling, scroll behavior

### v1.2.0 (2026-01-17) - Interactive Mode Phase 2 Foundation

**Sprint 4 Highlights:**
- **`/describe <table>` metacommand**: Inspect table structure without writing SQL queries
  - Shows column names, data types, nullable status, and default values
  - Supports qualified names (database.table) and unqualified names
  - Queries DBC.ColumnsV for comprehensive column information
- **`/ping` metacommand**: Test database connection within REPL
  - Displays connection latency in milliseconds
  - Shows session duration and connection details
  - Provides helpful suggestions on connection failure
- **Persistent command history**: Command history saved across sessions
  - Default location: `~/.tq_history` (10,000 entries)
  - Custom location via `--history-file` or `TQ_HISTORY_FILE` env var
  - Disable with `--no-history` flag
  - Metacommands (starting with `/`) excluded from history
- **Vim/Emacs keybinding modes**: Configurable editor modes
  - Emacs mode (default): Standard readline-style keybindings
  - Vi mode: Modal editing with insert/normal modes
  - Configure via `--editor-mode` flag or `TQ_EDITOR_MODE` env var

**New Tests:**
- 5 new unit tests for Sprint 4 functionality
- Tests for escape_sql_string, format_nullable, truncate_string
- CLI tests for editor mode and history options

### v1.1.0 (2026-01-17) - Interactive Mode MVP

**Sprint 3 Highlights:**
- Fixed critical column naming bug: Query results now display actual column names from the database instead of generic "col1", "col2"
- Fixed REPL default row limit: SELECT queries in REPL mode now default to 100 rows (configurable via `--default-limit`)
- Added live database integration tests to prevent future regressions

**Technical Debt Paid:**
- Added 5 new unit tests for metadata parsing (map-of-arrays format)
- Added 2 live database integration tests for column name validation
- Comprehensive validation reports generated in tests/results/

**Bug Fix Journey:**
- Initial implementation had incorrect metadata parsing (expected array-of-objects, API returns map-of-arrays)
- quality-validator caught the regression during validation (0% live query success rate)
- rust-teradata-architect fixed metadata parsing with proper format handling
- Revalidation confirmed all tests pass and column names display correctly

### v1.0.0 (2026-01-16)
- Initial MVP release
- Core query execution functionality
- Multiple output formats
- All authentication mechanisms
- Configuration management

---

## Feedback

We welcome feedback on the roadmap and feature requests. Please:
1. Check if your request aligns with tq's design principles (simple, fast, composable)
2. Submit requests via the project's issue tracker
3. Include specific use cases and examples

---

**Document maintained by:** AI Project Manager
