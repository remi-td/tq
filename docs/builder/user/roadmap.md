# tq Roadmap

**Last Updated:** 2026-01-17
**Version:** 1.2.0

---

## Current Sprint: Sprint 5 (Interactive Mode Phase 2 - Advanced)

**Completed:** Sprint 4
**In Progress:** Sprint 5
**Target Completion:** Q1-Q2 2026

Sprint 5 focuses on advanced REPL features including syntax highlighting and result paging.

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

### Technical Notes

- Uses `reedline` crate for line editing (same as Nushell)
- Persistent history saved to `~/.tq_history` with FileBackedHistory
- Editor mode configurable via `--editor-mode` flag or `TQ_EDITOR_MODE` env var
- Table output format for results
- Shows timing for each query
- Column metadata fetched from Teradata API for accurate column names
- Default 100-row limit for SELECT queries without explicit LIMIT (configurable via `--default-limit`)

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

## Next Up: Interactive Mode (Phase 2 - Advanced)

**Planned:** Q1-Q2 2026

| Feature | Priority | Description |
|---------|----------|-------------|
| SQL syntax highlighting | P1 | Color syntax in SQL input |
| Query timing display | P1 | Enhanced timing information |
| Result paging (horizontal) | P2 | Scroll through wide result sets |
| Result paging (vertical) | P2 | Scroll through long result sets |

---

## Future: Interactive Mode (Phase 3)

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
