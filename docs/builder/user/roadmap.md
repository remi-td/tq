# tq Roadmap

**Last Updated:** 2026-01-17
**Version:** 1.0.0

---

## Current Release: v1.0.0 (MVP)

The initial release provides core functionality for executing SQL queries against Teradata databases.

### Released Features

| Feature | Description | Status |
|---------|-------------|--------|
| `tq ping` | Test database connectivity with timing | Shipped |
| `tq query` | Execute SQL queries from argument, file, or stdin | Shipped |
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

---

## In Progress: Interactive Mode (Phase 1)

**Sprint:** 2026-01-17
**Status:** Implementation Complete, Testing Pending

### Features Being Delivered

| Feature | Description | Status |
|---------|-------------|--------|
| `tq repl` | Interactive REPL mode | Implementation Complete |
| Multi-line SQL input | Statements accumulate until semicolon | Implementation Complete |
| Command history | In-memory history with arrow key navigation | Implementation Complete |
| `/help` command | Display help within REPL | Implementation Complete |
| `/quit` command | Exit REPL cleanly | Implementation Complete |
| `/session` command | Display session information | Implementation Complete |
| Ctrl-C handling | Cancel current input gracefully | Implementation Complete |

### Technical Notes

- Uses `reedline` crate for line editing (same as Nushell)
- In-memory history (persistent history in Phase 2)
- Table output format for results
- Shows timing for each query

### Testing Status

- Unit tests: All passing (78 tests)
- Integration tests: All passing (37 tests)
- Manual testing: Pending live database connection

---

## Next Up: Interactive Mode (Phase 2)

**Planned:** Q1 2026

| Feature | Priority | Description |
|---------|----------|-------------|
| Persistent history | P1 | Save history to `~/.tq_history` |
| SQL syntax highlighting | P1 | Color syntax in SQL input |
| Vim/Emacs keybindings | P1 | Configurable editor modes |
| `/describe` command | P0 | Describe table structure |
| `/ping` command | P0 | Test connection within REPL |
| Query timing display | P1 | Enhanced timing information |
| Result paging | P1 | Scroll through large result sets |

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
