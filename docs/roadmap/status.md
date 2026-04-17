# Implementation Status Dashboard

**Last Updated:** 2026-04-17
**Current Version:** 1.46.0
**Latest Sprint:** Sprint 64 Complete (Bug Fixes: SPL body parser + stdin detection)

---

## Status Legend

- ✅ **Implemented and tested** - Feature complete, passing tests
- 🚧 **In progress** - Currently being developed
- 📋 **Planned** - In backlog, not yet started
- 🔲 **Deferred** - Lower priority, may be implemented later

---

## Core Features (MVP)

All core features are complete and tested.

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| Single query execution | ✅ | [CLI Interface](../specifications/cli-interface.md#query-command) | v1.0.0 |
| Connection testing | ✅ | [CLI Interface](../specifications/cli-interface.md#ping-command) | v1.0.0 |
| Multiple output formats | ✅ | [Output Formats](../specifications/output-formats.md) | v1.0.0 |
| TD2 authentication | ✅ | [CLI Interface](../specifications/cli-interface.md#authentication) | v1.0.0 |
| LDAP authentication | ✅ | [CLI Interface](../specifications/cli-interface.md#authentication) | v1.0.0 |
| Kerberos authentication | ✅ | [CLI Interface](../specifications/cli-interface.md#authentication) | v1.0.0 |
| Connection string parsing | ✅ | [CLI Interface](../specifications/cli-interface.md#connection-string) | v1.0.0 |
| Environment variables | ✅ | [Configuration](../specifications/configuration.md#environment-variables) | v1.0.0 |
| Password file support | ✅ | [Security](../specifications/security.md#password-files) | v1.0.0 |
| Secure credential handling | ✅ | [Security](../specifications/security.md) | v1.0.0 |

---

## Interactive Mode (REPL)

### Phase 1 - MVP Foundation (Complete)

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| Interactive prompt | ✅ | [REPL Mode](../specifications/repl.md#starting-repl) | v1.2.0 |
| Multi-line SQL input | ✅ | [REPL Mode](../specifications/repl.md#multi-line-sql) | v1.2.0 |
| Command history (in-memory) | ✅ | [REPL Mode](../specifications/repl.md#command-history) | v1.2.0 |
| Multi-line command history | ✅ | [REPL Mode](../specifications/repl.md#command-history) | v1.11.0 (Sprint 24) |
| `/session` metacommand | ✅ | [REPL Mode](../specifications/repl.md#session-commands) | v1.2.0 |
| `/quit` metacommand | ✅ | [REPL Mode](../specifications/repl.md#utility-commands) | v1.2.0 |
| `/help` metacommand | ✅ | [REPL Mode](../specifications/repl.md#utility-commands) | v1.2.0 |

### Phase 2 - Enhanced Features (Complete)

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `/describe` metacommand | ✅ | [REPL Mode](../specifications/repl.md#schema-inspection) | v1.3.0 |
| `/ping` metacommand | ✅ | [REPL Mode](../specifications/repl.md#connection-commands) | v1.3.0 |
| Persistent history | ✅ | [REPL Mode](../specifications/repl.md#persistent-history) | v1.3.0 |
| Vi keybindings | ✅ | [REPL Mode](../specifications/repl.md#vi-mode) | v1.3.0 |
| Emacs keybindings | ✅ | [REPL Mode](../specifications/repl.md#emacs-mode) | v1.3.0 |
| SQL syntax highlighting | ✅ | [REPL Mode](../specifications/repl.md#syntax-highlighting) | v1.3.0 |
| Result paging (horizontal) | ✅ | [REPL Mode](../specifications/repl.md#horizontal-column-navigation) | v1.3.0, refactored v1.13.0 (Sprint 30, disabled by default) |
| Result paging (vertical) | ✅ | [REPL Mode](../specifications/repl.md#vertical-paging) | v1.3.0 |
| Pager exit snapshot | ✅ | [REPL Mode](../specifications/repl.md#pager-exit-behavior) | v1.45.0 (Sprint 63) |
| Query timing display | ✅ | [REPL Mode](../specifications/repl.md#timing-display) | v1.3.0 |

### Phase 3 - Advanced Features (Complete)

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| Table formatting | ✅ | [Output Formats](../specifications/output-formats.md#table-format) | v1.4.0 |
| Tab completion (keywords) | ✅ | [REPL Mode](../specifications/repl.md#keyword-completion) | v1.4.0 |
| Tab completion (tables) | ✅ | [REPL Mode](../specifications/repl.md#table-completion) | v1.5.0 |
| Tab completion (columns) | ✅ | [REPL Mode](../specifications/repl.md#column-completion) | v1.5.0 |
| Tab completion (multi-line) | ✅ | [REPL Mode](../specifications/repl.md#tab-completion) | v1.5.1 |
| `/export` metacommand | ✅ | [REPL Mode](../specifications/repl.md#export-commands) | v1.6.0 |
| `/pager on\|off` metacommand | ✅ | [REPL Mode](../specifications/repl.md#pager-control) | v1.4.0 |
| `/colors on\|off` metacommand | ✅ | [REPL Mode](../specifications/repl.md#color-control) | v1.4.0 |
| `/logon` metacommand | ✅ | [REPL Mode](../specifications/repl.md#logon-command) | v1.5.0 |

### Phase 4 - Quality & Branding (Complete)

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| Professional branding | ✅ | [Branding Guidelines](../specifications/branding-guidelines.md) | v1.6.1, fixed v1.7.1 (Sprint 20) |
| Interactive test framework | ✅ | Testing infrastructure | v1.6.1 |
| Export syntax simplification | ✅ | [REPL Mode](../specifications/repl.md#export-commands) | v1.6.1 |
| Export to clipboard | ✅ | [REPL Mode](../specifications/repl.md#export-to-clipboard) | v1.6.1 |
| Export full dataset | ✅ | [REPL Mode](../specifications/repl.md#export-full-dataset) | v1.6.1 |

**Sprint 20 Bug Fixes (v1.7.1):**
- Fixed logo display: Implemented correct 9-line lowercase ASCII art (was 3-line blocks)
- Fixed tab completion: Eliminated "Page 1: records 0 - 0" output (changed from ListMenu to ColumnarMenu)

**Sprint 21 Enhancements (v1.8.0):**
- Tab completion quality: Include 'dbc' system database in completion menu
- Tab completion data completeness: Universal table fetching for all databases (fixed "NO RECORDS FOUND" issue)
- Smart qualified name completion: Single database match appends '.' and shows tables immediately
- Automated regression tests: Comprehensive test suite with hybrid testing pattern

**Sprint 22 Enhancements (v1.9.0):**
- Metacommand tab completion: Type `/` + TAB to see all available metacommands with descriptions
- Enhanced schema commands: `/list databases`, `/list tables [pattern]`, `/list views` for quick schema exploration

**Sprint 23 Enhancements (v1.10.0):**
- Testing infrastructure improvements: Test implementation checklist, consolidated testing guidelines
- Output to file: `--output` flag with atomic file writes using tempfile crate
- Transaction control: `--atomic` flag for automatic BEGIN/COMMIT/ROLLBACK in batch mode
- Integration test driver synchronization: Mutex-based locking for parallel test execution
- Glob pattern support: Filter tables with patterns like `dbc.t*` or `*_archive`
- Short command aliases: `/l` (databases), `/dt` (tables), `/dv` (views)
- Comprehensive user documentation: New REPL guide with examples and best practices

**Sprint 24 Enhancements (v1.11.0):**
- Multi-line command history: SQL statements stored and recalled as complete units (not line-by-line)
- Documentation accuracy verification: Added Ship phase checklist to prevent doc/implementation mismatches
- Enhanced error messages: Better guidance for Teradata session mode transaction limitations
- SqlStatementValidator: Leverages reedline Validator trait for multi-line input handling
- Process improvements: Sprint 22 & 23 lessons applied to prevent documentation errors

**Sprint 26 Enhancements (v1.12.0):**
- `/sessions` command: List active Teradata sessions with performance metrics
- Session monitoring: Display SessionNo, UserName, LogonTime, PEstate, AMPState
- Performance metrics: AMPCPUSec, AMPIO, ReqSpool with thousand separators
- Skew calculation: CPU and I/O distribution across AMPs for bottleneck detection
- Batch mode integration: `tq sessions` standalone command
- Tab completion: `/sessions` and `/s` alias in completion menu
- Multi-format support: Table, CSV, and JSON output for session data

**Sprint 27 Bug Fixes & Documentation (v1.12.1):**
- `/sessions` bug fix: Correctly display ALL sessions regardless of PEstate/AMPState value types (fixes issue #10)
- LICENSE file: Comprehensive licensing with MIT + third-party attributions (teradatarustapi, Go SDK)
- README restructure: User-focused TLDR format with AI development story and quick start guide
- User documentation: Added `/sessions` command documentation to REPL guide (Sprint 26 gap addressed)
- Regression tests: Added tests to prevent session filtering bugs in future

**Sprint 28 Enhancements (v1.12.1):**
- Pager UX improvements: Enhanced column position indicators for better discoverability (feature existed since v1.3.0)
- Help system: Added comprehensive pager documentation to `/help` command
- Build cleanup: Removed build.rs success warning for cleaner development experience
- Status bar improvements: Better navigation hints in pager mode

**Sprint 29 Implementation (v1.13.0):**
- Interactive horizontal paging: Full implementation with Left/Right arrow navigation for wide result sets
- Vim keybindings: h/l for horizontal scrolling, H/L for jump to first/last column
- Column indicators: Visual feedback with `(+N cols) ←` and `(+N cols) →` indicators
- Status bar: Dynamic "Columns X-Y of Z" display showing current column range
- Help text: ? key shows comprehensive pager navigation controls including horizontal
- Position preservation: Column position maintained during vertical scrolling
- Pager integration: Re-enabled in executor with proper state management
- 23 interactive tests: Comprehensive test coverage for all horizontal paging features

**Sprint 30 Maintenance (v1.13.0):**
- Pager architectural refactor: Pager now accepts QueryResult directly instead of pre-formatted strings
- Column width calculation at render time: Eliminates pre-formatting pipeline that caused Sprint 29 issues
- Test infrastructure (Track 3): Created dimensional testing utilities (visual_validator, terminal_simulator)
- Dimensional test suite: 28 tests validating terminal width constraints
- Test fixtures: Helper functions for creating QueryResult test data
- 92 utility tests: Comprehensive validation of testing infrastructure
- Pager disabled by default: Feature has architectural improvements but still has rendering issues
- Dead code removal: Cleaned up write_output_for_pager(), write_all_columns() functions
- Crisis deliberation: Multi-agent analysis documented in sprint-30-crisis-deliberation.md

**Sprint 31 Maintenance (v1.13.0):**
- Framework crisis recovery: Honest documentation of testing limitations
- Pager bug fix: Two-pass cell truncation (root cause: cell value exceeds display width)
- Testing philosophy updates: Manual validation requirements for visual features
- Sprint 29 review correction: Downgraded 9.5/10 to 2/10 (honest assessment)
- Quality validator role clarified: Advisory verdict, not blocking
- Framework integrity restored through transparency

**Sprint 32 Features (v1.14.0):**
- **Content-based column width** (Issue #13): Columns sized to actual content, not schema types
- MAX_COLUMN_WIDTH constant (100 chars): Prevents columns from dominating display
- Table information density: 4.5x improvement (2 cols → 9 cols at 117-char terminal)
- 15 comprehensive unit tests: NULL, empty strings, numeric, Unicode, cap boundaries
- GitHub README fix (Issue #12): Root README now displays on repository landing page
- Documentation updates: Specifications, design, user guides
- Sprint 31 lessons applied: Type 4 classification, manual validation protocol

**Sprint 36 Enhancements (v1.17.0):**
- **Config help text polish**: Project configuration section in `tq help config`, 5-level precedence hierarchy
- **Config UX improvements**: Project config path shown in `tq profiles`, empty state tip, invalid TOML warning to stderr
- **`/repeat` command**: Re-execute last SQL query (alias `\r`), follows psql convention
- **`/show indexes <table>`**: Display table indexes from DBC.IndicesV (alias `\di`), qualified name support
- **Tab completion**: New commands in metacommand completion menu
- 40 new tests (674 total, 100% pass rate), zero clippy warnings

**Sprint 33 Features (v1.15.0):**
- **Pager bug fix** (Issue #14): Fixed Unicode width mismatch causing garbled output
- **Pager disabled by default**: User protection from rendering issues (opt-in with `/pager on`)
- **Data sampling commands**: `/sample` and `/peek` for fast data exploration
- **Random sampling**: `/sample <table> [n]` with Teradata SAMPLE clause (default 10, max 1000 rows)
- **Table preview**: `/peek <table>` shows first 5 rows + column metadata
- **Batch mode integration**: `tq sample` and `tq peek` CLI commands
- **Tab completion**: Data sampling commands in metacommand completion menu
- 22 new unit tests, 471 total tests (100% pass rate)

---

## Batch Mode

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| Execute from file | ✅ | [Batch Mode](../specifications/batch-mode.md#file-input) | v1.6.0 |
| Read SQL from stdin | ✅ | [Batch Mode](../specifications/batch-mode.md#stdin-input) | v1.6.0 |
| Output to stdout | ✅ | [Batch Mode](../specifications/batch-mode.md#output-behavior) | v1.6.0 |
| Multiple statement execution | ✅ | [Batch Mode](../specifications/batch-mode.md#multiple-statements) | v1.6.0 |
| Enhanced error messages | ✅ | [Error Handling](../specifications/error-handling.md) | v1.6.0 |
| Batch mode output behavior | ✅ | [Batch Mode](../specifications/batch-mode.md#output-behavior) | v1.6.0 |
| Output to file (--output flag) | ✅ | [Batch Mode](../specifications/batch-mode.md#output-to-file) | v1.10.0 (Sprint 23) |
| Atomic file writes | ✅ | [Batch Mode](../specifications/batch-mode.md#atomic-writes) | v1.10.0 (Sprint 23) |
| Transaction control (--atomic flag) | ✅ | [Batch Mode](../specifications/batch-mode.md#transactions) | v1.10.0 (Sprint 23) |
| Streaming large results | 📋 | [Performance](../specifications/performance.md#streaming) | Future |
| Variable substitution | ✅ | [Batch Mode](../specifications/batch-mode.md#variable-substitution) | v1.21.0 (Sprint 40) |

---

## Configuration

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| User config file (`~/.tq/config.toml`) | ✅ | [Configuration](../specifications/configuration.md#user-config) | v1.6.1 |
| Connection profiles | ✅ | [Configuration](../specifications/configuration.md#profiles) | v1.6.1 |
| Default preferences | ✅ | [Configuration](../specifications/configuration.md#preferences) | v1.6.1 |
| `--profile <name>` flag | ✅ | [CLI Interface](../specifications/cli-interface.md#profile-flag) | v1.6.1 |
| `tq help config` subcommand | ✅ | [CLI Interface](../specifications/cli-interface.md#help-command) | v1.7.0 |
| `tq help credentials` subcommand | ✅ | [CLI Interface](../specifications/cli-interface.md#help-command) | v1.7.0 |
| `tq profiles` command | ✅ | [CLI Interface](../specifications/cli-interface.md#profiles-command) | v1.7.0 |
| Password file permission enforcement | ✅ | [Security](../specifications/security.md#file-permissions) | v1.7.0 |
| Security check ordering fix | ✅ | [Security](../specifications/security.md) | v1.7.0 |
| Project config file (`.tq.toml`) | ✅ | [Configuration](../specifications/configuration.md#project-config) | v1.16.0 (Sprint 35) |
| Profile editing commands | ✅ | [Configuration](../specifications/configuration.md#profile-management) | v1.24.0 (Sprint 43) |
| Keyring integration | 📋 | [Security](../specifications/security.md#keyring) | Future |
| Config validation command | 📋 | [Configuration](../specifications/configuration.md#validation) | Future |

---

## Help System

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `tq help` (general) | ✅ | [CLI Interface](../specifications/cli-interface.md#help-command) | v1.0.0 |
| `tq help config` subcommand | ✅ | [CLI Interface](../specifications/cli-interface.md#help-topics) | v1.7.0 |
| `tq help credentials` subcommand | ✅ | [CLI Interface](../specifications/cli-interface.md#help-topics) | v1.7.0 |
| Help topic routing | ✅ | [CLI Interface](../specifications/cli-interface.md#help-command) | v1.7.0 |

---

## System Monitoring

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `/sessions` command (REPL) | ✅ | [REPL Mode](../specifications/repl.md#session-monitoring) | v1.12.0 (Sprint 26) |
| `tq sessions` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#sessions-command) | v1.12.0 (Sprint 26) |
| Session performance metrics | ✅ | [REPL Mode](../specifications/repl.md#session-monitoring) | v1.12.0 (Sprint 26) |
| CPU/IO skew calculation | ✅ | [REPL Mode](../specifications/repl.md#session-monitoring) | v1.12.0 (Sprint 26) |
| `/sysconfig` command (REPL) | ✅ | [REPL Mode](../specifications/repl.md#system-configuration) | v1.19.0 (Sprint 38) |
| `tq sysconfig` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#sysconfig-command) | v1.19.0 (Sprint 38) |
| `/locks` command (REPL) | ✅ | [REPL Mode](../specifications/repl.md#lock-monitoring) | v1.19.0 (Sprint 38) |
| `tq locks` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#locks-command) | v1.19.0 (Sprint 38) |
| Blocking chain identification | ✅ | [REPL Mode](../specifications/repl.md#lock-monitoring) | v1.19.0 (Sprint 38) |
| `/query` command (REPL) | ✅ | [REPL Mode](../specifications/repl.md#query-inspection) | v1.20.0 (Sprint 39) |
| `tq query-inspect` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#query-inspect) | v1.20.0 (Sprint 39) |
| Shared monitoring utilities | ✅ | Internal refactor | v1.20.0 (Sprint 39) |

---

## Data Sampling

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `/sample` command (REPL) | ✅ | [REPL Mode](../specifications/repl.md#data-sampling) | v1.15.0 (Sprint 33) |
| `/peek` command (REPL) | ✅ | [REPL Mode](../specifications/repl.md#data-sampling) | v1.15.0 (Sprint 33) |
| `tq sample` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#sample-command) | v1.15.0 (Sprint 33) |
| `tq peek` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#peek-command) | v1.15.0 (Sprint 33) |
| Random sampling (SAMPLE clause) | ✅ | [REPL Mode](../specifications/repl.md#data-sampling) | v1.15.0 (Sprint 33) |
| Column metadata display | ✅ | [REPL Mode](../specifications/repl.md#data-sampling) | v1.15.0 (Sprint 33) |
| Qualified name support | ✅ | [REPL Mode](../specifications/repl.md#data-sampling) | v1.15.0 (Sprint 33) |
| Tab completion integration | ✅ | [REPL Mode](../specifications/repl.md#data-sampling) | v1.15.0 (Sprint 33) |

---

## Schema Inspection

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `/describe <table>` | ✅ | [REPL Mode](../specifications/repl.md#schema-inspection) | v1.3.0 |
| `/list databases` | ✅ | [REPL Mode](../specifications/repl.md#schema-inspection) | v1.9.0 (Sprint 22) |
| `/list tables [pattern]` | ✅ | [REPL Mode](../specifications/repl.md#schema-inspection) | v1.9.0 (Sprint 22) |
| `/list views` | ✅ | [REPL Mode](../specifications/repl.md#schema-inspection) | v1.9.0 (Sprint 22) |
| `/show indexes <table>` | ✅ | [REPL Mode](../specifications/repl.md#schema-inspection) | v1.17.0 (Sprint 36) |
| `/inspect <object>` | ✅ | [REPL Mode](../specifications/repl.md#object-inspection) | v1.26.0 (Sprint 45) |
| `tq inspect` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#inspect-command) | v1.26.0 (Sprint 45) |
| `tq describe` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#describe-command) | v1.27.0 (Sprint 46) |
| `tq list` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#list-command) | v1.27.0 (Sprint 46) |
| `tq show-indexes` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#show-indexes-command) | v1.27.0 (Sprint 46) |

---

## Query Editing

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `/repeat` command | ✅ | [REPL Mode](../specifications/repl.md#query-editing) | v1.17.0 (Sprint 36) |
| `/edit` command | ✅ | [REPL Mode](../specifications/repl.md#query-editing) | v1.18.0 (Sprint 37) |

---

**Sprint 38 Enhancements (v1.19.0):**
- **`/sysconfig` command**: Display Teradata system topology (version, AMP count) via DBC.DBCInfoV
- **`tq sysconfig`**: Batch mode with table/CSV/JSON output
- **`/locks` command**: Display current lock contention from DBC.LockInfoV with blocking chain identification
- **`tq locks`**: Batch mode with table/CSV/JSON output
- **Blocking chain analysis**: Automatic identification of which sessions block which
- **Tab completion**: Both commands in metacommand completion menu
- Pre-existing clippy warnings fixed in interactive tests
- 748 total tests (100% pass rate), zero clippy warnings

**Sprint 39 Enhancements (v1.20.0):**
- **Monitoring utilities extraction**: Shared `monitoring_utils.rs` module eliminates 4x code duplication across sessions.rs, sysconfig.rs, locks.rs, sample.rs
- **Sprint 38 bug fix**: CSV output for locks with no waiters now uses empty string (was "(none)")
- **Sprint 38 doc alignment**: Design docs synced with DBC.LockInfoV implementation, user guide matches actual features
- **Error handling tests**: Added for sysconfig.rs and locks.rs
- **`/query` command**: Inspect recent SQL queries for a session via DBC.QryLogV (alias `/qi`)
- **`tq query-inspect`**: Batch mode with table/CSV/JSON output
- **Tab completion**: `/query` and `/qi` in metacommand completion menu
- 830 total tests (100% pass rate), zero clippy warnings

**Sprint 40 Enhancements (v1.21.0):**
- **Variable substitution**: YAML parameter files with `--params`/`-p` flag for SQL templating
- **`{{variable}}` markers**: Dot-notation nested access, `{{$ENV.VAR}}` for environment variables
- **`/params` metacommand**: Load/unload/show parameter files in REPL mode
- **`tq help params`**: Comprehensive help topic for variable substitution
- **Multiple file merging**: Deep merge with last-writer-wins semantics
- **Sprint 39 remediation**: REQ-QUERY spec updated, 31 redundant utility tests removed
- 855 total tests (100% pass rate), zero clippy warnings

**Sprint 41 Enhancements (v1.22.0):**
- **GitHub Actions release workflow**: Automated CI/CD pipeline triggered by `v*` tags
- **Cross-platform binaries**: Builds for Linux x86_64, Linux aarch64, macOS x86_64, macOS aarch64, Windows x86_64
- **Install script**: POSIX-compatible one-liner installer with checksum verification
- **Cross-compilation build.rs**: Uses `CARGO_CFG_TARGET_OS`/`CARGO_CFG_TARGET_ARCH` for correct library selection
- **Sprint 40 remediation**: Eliminated execute/execute_with_params duplication, LazyLock regex, /p alias documented
- 855 total tests (100% pass rate), zero clippy warnings

**Sprint 42 Bug Fixes (v1.23.0):**
- **SQL parser hardening**: Replaced naive `split(';')` with proper state-machine lexer (Issues #28, #29, #30)
- **Quote-aware splitting**: Semicolons inside single-quoted strings no longer split statements
- **Multi-line statement support**: Newlines within statements handled correctly in `--file` mode
- **Comment stripping**: Line (`--`) and block (`/* */`) comments stripped to prevent contamination
- **Sprint 41 remediation**: Pinned cross-rs v0.2.5, renamed TMPDIR in install.sh, marked flaky test as `#[ignore]`
- 674 unit tests + 179 integration tests (100% pass rate), zero clippy warnings

**Sprint 43 Enhancements (v1.24.0):**
- **Profile management commands**: `tq profile add`, `tq profile edit`, `tq profile delete`, `tq profile list`
- **Non-interactive CLI**: Flag-based profile CRUD for scriptability (no interactive prompts)
- **Validation**: Logmech (TD2/LDAP/KRB5/TDNEGO) and port (1-65535) validation with clear error messages
- **Config preservation**: Atomic writes with existing config content preservation
- **Parser error handling**: `parse_statements()` returns `Result` with `ParseError` (line/column) for unterminated strings and block comments
- **Sprint 42 remediation**: Parser spec clarifications (REQ-PARSE-015, REQ-PARSE-018), space-injection documentation
- 705 unit tests + 191 integration tests (100% pass rate), zero clippy warnings

**Sprint 44 Bug Fixes & Polish (v1.25.0):**
- **Runtime driver resolution** (Issue #31): Binary finds teradatasql library relative to executable path, not hardcoded CI build path
- **Driver search fallback chain**: --driver-lib-dir → TERADATA_LIB_DIR env var → executable directory → current directory
- **License acceptance**: Install script displays Teradata license and requires acceptance (--accept-license for non-interactive)
- **Profile flag naming fix**: Profile subcommands now use `--logmech`/`--password-file` (was `--auth`/`--pass-file`)
- **Profile delete confirmation**: TTY-interactive `[y/N]` prompt, non-TTY requires `--force`
- **SqlParseError struct variant**: Preserves line/column from ParseError for better error reporting
- **Shared display_profile() helper**: Eliminated handle_list/handle_profiles duplication
- 715 unit tests + 178 integration tests (100% pass rate), zero clippy warnings

**Sprint 45 Features (v1.26.0):**
- **Bug #32 fix**: Metacommand semicolon stripping — `/describe a;`, `/list tables;`, `/sample dbc.tables;` now work correctly
- **`/inspect` command** (Issue #33): Comprehensive object inspection showing type, columns, indexes, storage/skew, and view/macro definitions
- **`tq inspect`**: Batch mode with table/CSV/JSON output for scripting
- **Tab completion**: `/inspect` and `\i` alias in metacommand completion menu
- **Sprint 44 deferred**: `--force` help text, abort message with profile name, debug logging in driver resolution, design doc drift fix
- 742 unit tests + 191 integration tests (100% pass rate), zero clippy warnings

**Sprint 46 Bug Fixes & Polish (v1.27.0):**
- **Bug #35 fix**: Identifier quoting now uppercases before quoting, matching Teradata case-insensitive behavior. Also fixed `extract_table_name()` word boundary matching.
- **Bug #34 fix**: New batch CLI commands: `tq describe`, `tq list databases|tables|views`, `tq show-indexes` with table/CSV/JSON output
- **/inspect formatting polish**: Section headers use `──` format, default column `-`, column count footer, skew interpretation hints, `O`→"Table (NoPI)", Error: prefix, usage examples, safe row indexing
- 765 unit tests + 191 integration tests (100% pass rate), zero clippy warnings

**Sprint 47 Tech Debt Elimination & Command Enrichment (v1.28.0):**
- **Bug #36 fix**: `/inspect` now shows full DDL for views/macros (was garbled/truncated) and resolves column types from type codes (was [NULL])
- **Shared helpers extraction**: `format_helpers.rs` module eliminates 4x duplication of json_escape, csv_escape, parse_table_name, truncate_str across command modules
- **UTF-8 safety**: `truncate_str()` uses `char_indices()` for proper Unicode boundary handling (was byte-slicing)
- **REPL delegation**: `/describe`, `/list`, `/show indexes` now delegate to batch modules (~400 lines of duplicated code removed)
- **`tq describe` enrichment**: Object header block, Comments column, Indexes section, structured JSON output
- **`tq list` enrichment**: Owner/Type columns for databases, Rows/Size for tables, structured JSON objects
- **`tq show-indexes` enrichment**: Two-section Primary/Secondary layout, UPI/NUPI/USI/NUSI labels, structured JSON
- **Error consistency**: `Error:` prefix on all error messages, `<OBJECT>` in help text
- 799 unit tests + 178 integration tests (100% pass rate), zero clippy warnings

**Sprint 48 Query Layer Consolidation & Spec Alignment (v1.29.0):**
- **Shared query layer**: `query_helpers.rs` consolidates query_indexes (3→1), query_columns (2→1), resolve_database (2→1), query_object_header
- **Shared types**: `ColumnInfo`, `IndexGroup`, `ObjectHeader` defined once, used by inspect/describe/show_indexes
- **format_size unified**: Single parameterized function replaces 2 variants (inspect precision=2, list precision=1)
- **JSON API types fixed**: describe nullable as boolean, default as null; list tables rows/size as integers
- **Bug fixes**: summarize_error UTF-8 (uses truncate_str), show-indexes TABLE→OBJECT, list databases System/User labels, Error: prefix, DescribeArgs.object rename
- **Edge cases**: "No indexes defined.", "No Primary Index (NoPI)", "No secondary indexes.", Rows (Est.) in describe header
- **List views enriched**: Owner column added to display, CSV, and JSON output
- **Missing tests delivered**: 6 DDL tests, writer-injection rendering tests, column_type_case_sql completeness
- **Spec canonicalized**: `──` headers, glob patterns, inline index format, conditional Comment column
- 833 unit tests + 178 integration tests (100% pass rate), zero clippy warnings

**Sprint 49 Session Control Functions (v1.30.0):**
- **`/abort <session_id> [yes]`**: Abort session with interactive confirmation
- **`/abort query <session_id> [yes]`**: Abort running query only (keeps session alive)
- **Batch mode**: `tq abort --force` with table/CSV/JSON output
- **Safety model**: REPL requires explicit 'yes', batch requires `--force` flag
- **Tab completion**: Both commands in metacommand completion menu
- 855 unit tests + 178 integration tests (100% pass rate), zero clippy warnings

**Sprint 50 Query Drill-Down & Explain Plans (v1.31.0):**
- **`/explain <sql>`**: Show Teradata EXPLAIN execution plan step-by-step
- **`/skew [session_id]`**: Analyze AMP-level CPU/IO resource distribution
- **Batch mode**: `tq explain`, `tq skew` with table/CSV/JSON output
- **Skew interpretation**: Automatic hints (good/moderate/high/severe) based on thresholds
- **Top sessions**: `/skew` without session_id shows top-10 by CPU skew
- **EXPLAIN detection**: Automatically avoids double-prefixing EXPLAIN keyword
- 882 unit tests + 178 integration tests (100% pass rate), zero clippy warnings

**Sprint 51 Session History & Trends (v1.32.0):**
- **`/history [--last <dur>] [--user <name>]`**: View session logon/logoff activity
- **`tq history --last 24h`**: Batch mode with table/CSV/JSON output
- **Duration parsing**: 30m, 1h, 24h, 7d with validation and upper limits
- **Summary statistics**: Logons, logoffs, auth failures, unique users
- **User filtering**: `--user <name>` with SQL injection prevention
- **Event mapping**: L→Logon, O→Logoff, A→Auth Fail from DBC.LogOnOffV
- 906 unit tests + 178 integration tests (100% pass rate), zero clippy warnings

**Sprint 52 Enhancements (v1.33.0):**
- **Markdown output format**: New `markdown`/`md` format (`src/format/markdown.rs`) supported across all 14+ commands
- **Comment column**: Added to inspect/describe output in table, JSON, CSV, and markdown formats
- **Format documentation**: `--format` argument documented in all command `--help` text
- **Issues closed**: #38 (markdown output), #39 (comment column), #40 (format docs)
- 893 unit tests + 178 integration tests (100% pass rate), zero clippy warnings

**Sprint 53 Enhancements (v1.34.0):**
- **JSON envelope**: All JSON output uses consistent `{"ok": true, "row_count": N, "data": [...]}` structure
- **Structured JSON errors**: Error output with code, category, retryable, message, hint fields
- **12 error codes**: Across 9 categories for comprehensive error taxonomy
- **Command.format() method**: Centralized format resolution in CLI
- **JSON errors to stdout**: When `--format json`, errors output to stdout (not stderr) for single-stream parsing
- **Issue #37 parts 1-2**: Agent mode JSON envelope and structured errors
- 901 unit tests + 178 integration tests (100% pass rate), zero clippy warnings

**Sprint 54 Enhancements (v1.35.0):**
- **`--agent-safe` flag**: Blocks DDL/DML, enforces single-statement, limits rows
- **`--allow-dml` flag**: Fine-grained override to permit DML in agent-safe mode
- **`--max-rows` flag**: Limit result set size for agent context windows
- **Statement classification**: ReadOnly/DML/DDL with Teradata-specific support (LOCKING, COLLECT, SEL, INS, etc.)
- **Richer inspect JSON**: Indexes, storage, definition (DDL), and dependency graph in inspect output
- **Error variants**: `AgentSafeBlocked` and `AgentSafeMaxRows` for agent-specific errors
- **Issue #37 parts 3-4**: Agent-safe mode and richer introspection
- 913 unit tests + 178 integration tests (100% pass rate), zero clippy warnings

---

## Session Control

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `/abort` command (REPL) | ✅ | [Admin User Stories](../specifications/admin-user-stories.md#control-functions) | v1.30.0 (Sprint 49) |
| `tq abort` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#abort-command) | v1.30.0 (Sprint 49) |
| Safety confirmation (REPL) | ✅ | Internal requirement | v1.30.0 (Sprint 49) |
| Batch --force flag | ✅ | Internal requirement | v1.30.0 (Sprint 49) |

---

## Query Analysis

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `/explain` command (REPL) | ✅ | [Admin User Stories](../specifications/admin-user-stories.md#query-drill-down) | v1.31.0 (Sprint 50) |
| `tq explain` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#explain-command) | v1.31.0 (Sprint 50) |
| `/skew` command (REPL) | ✅ | [Admin User Stories](../specifications/admin-user-stories.md#query-drill-down) | v1.31.0 (Sprint 50) |
| `tq skew` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#skew-command) | v1.31.0 (Sprint 50) |
| Skew interpretation hints | ✅ | Internal enhancement | v1.31.0 (Sprint 50) |

---

## Session History

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `/history` command (REPL) | ✅ | [Admin User Stories](../specifications/admin-user-stories.md#session-history) | v1.32.0 (Sprint 51) |
| `tq history` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#history-command) | v1.32.0 (Sprint 51) |
| Time range filtering | ✅ | Internal requirement | v1.32.0 (Sprint 51) |
| User filtering | ✅ | Internal requirement | v1.32.0 (Sprint 51) |
| Summary statistics | ✅ | Internal requirement | v1.32.0 (Sprint 51) |

---

## Output Formats

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| Table format | ✅ | [Output Formats](../specifications/output-formats.md#table-format) | v1.0.0 |
| CSV format | ✅ | [Output Formats](../specifications/output-formats.md#csv-format) | v1.0.0 |
| JSON format | ✅ | [Output Formats](../specifications/output-formats.md#json-format) | v1.0.0 |
| Markdown format | ✅ | [Output Formats](../specifications/output-formats.md#markdown-format) | v1.33.0 (Sprint 52) |
| Format documentation in --help | ✅ | Internal requirement | v1.33.0 (Sprint 52) |
| Comment column in inspect/describe | ✅ | Internal enhancement | v1.33.0 (Sprint 52) |

---

## Agent Mode

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| JSON envelope (`ok`, `row_count`, `data`) | ✅ | Issue #37 part 1 | v1.34.0 (Sprint 53) |
| Structured JSON errors | ✅ | Issue #37 part 2 | v1.34.0 (Sprint 53) |
| `--agent-safe` flag | ✅ | Issue #37 part 3 | v1.35.0 (Sprint 54) |
| `--allow-dml` flag | ✅ | Issue #37 part 3 | v1.35.0 (Sprint 54) |
| `--max-rows` flag | ✅ | Issue #37 part 3 | v1.35.0 (Sprint 54) |
| Statement classification (ReadOnly/DML/DDL) | ✅ | Issue #37 part 3 | v1.35.0 (Sprint 54) |
| Richer inspect JSON (indexes, storage, DDL, deps) | ✅ | Issue #37 part 4 | v1.35.0 (Sprint 54) |
| Search/discovery commands | ✅ | Issue #37 part 5 | v1.36.0 (Sprint 55) |
| Result pagination | ✅ | Issue #37 part 6 | v1.37.0 (Sprint 56) |
| Search views subcommand | ✅ | Sprint 57 | v1.38.0 (Sprint 57) |
| Serde JSON in search renderers | ✅ | Sprint 57 tech debt | v1.38.0 (Sprint 57) |
| Context-aware IntelliSense | ✅ | Sprint 58 | v1.40.0 (Sprint 58) |

---

## PMON Resource Monitoring

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `/resources` command (REPL) | ✅ | [Admin User Stories](../specifications/admin-user-stories.md#performance-summary) | v1.41.0 (Sprint 59) |
| `tq resources` (batch mode) | ✅ | [CLI Interface](../specifications/cli-interface.md#resources-command) | v1.41.0 (Sprint 59) |
| Virtual mode (per-VPROC) | ✅ | Internal requirement | v1.41.0 (Sprint 59) |
| Physical mode (per-node) | ✅ | Internal requirement | v1.41.0 (Sprint 59) |
| CPU/IO skew calculation | ✅ | Internal requirement | v1.41.0 (Sprint 59) |

---

## Watch Mode

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `--watch` flag (sessions) | ✅ | [CLI Interface](../specifications/cli-interface.md#sessions-command) | v1.42.0 (Sprint 60) |
| `--watch` flag (locks) | ✅ | [CLI Interface](../specifications/cli-interface.md#locks-command) | v1.42.0 (Sprint 60) |
| `--watch` flag (resources) | ✅ | [CLI Interface](../specifications/cli-interface.md#resources-command) | v1.42.0 (Sprint 60) |
| `--interval` configuration | ✅ | Internal requirement | v1.42.0 (Sprint 60) |
| REPL watch mode | ✅ | [REPL Mode](../specifications/repl.md#watch-mode) | v1.42.0 (Sprint 60) |

---

## Extended Session Control

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `/abort user <username>` (REPL) | ✅ | [Admin User Stories](../specifications/admin-user-stories.md#control-functions) | v1.43.0 (Sprint 61) |
| `tq abort --user <username>` (batch) | ✅ | [CLI Interface](../specifications/cli-interface.md#abort-command) | v1.43.0 (Sprint 61) |
| `/abort host <hostname>` (REPL) | ✅ | [Admin User Stories](../specifications/admin-user-stories.md#control-functions) | v1.43.0 (Sprint 61) |
| `tq abort --host <hostname>` (batch) | ✅ | [CLI Interface](../specifications/cli-interface.md#abort-command) | v1.43.0 (Sprint 61) |
| `/logoff idle` (REPL) | ✅ | [Admin User Stories](../specifications/admin-user-stories.md#control-functions) | v1.43.0 (Sprint 61) |
| `tq logoff-idle` (batch) | ✅ | [CLI Interface](../specifications/cli-interface.md#logoff-idle-command) | v1.43.0 (Sprint 61) |

---

## Search & Discovery

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| `tq search tables` | ✅ | [CLI Interface](../specifications/cli-interface.md#search-command) | v1.36.0 (Sprint 55) |
| `tq search columns` | ✅ | [CLI Interface](../specifications/cli-interface.md#search-command) | v1.36.0 (Sprint 55) |
| `tq search views` | ✅ | [CLI Interface](../specifications/cli-interface.md#search-command) | v1.38.0 (Sprint 57) |
| `tq search procedures` | ✅ | [CLI Interface](../specifications/cli-interface.md#search-command) | v1.43.0 (Sprint 61) |

---

## Security Hardening

| Feature | Status | Spec Reference | Since |
|---------|--------|----------------|-------|
| JSON injection fix (`to_json_string`) | ✅ | [Security](../specifications/security.md#output-integrity) | v1.44.0 (Sprint 62) |
| SQL LIKE wildcard escaping (`escape_sql_like`) | ✅ | [Security](../specifications/security.md#sql-injection-prevention) | v1.44.0 (Sprint 62) |
| Password file permission enforcement | ✅ | [Security](../specifications/security.md#credential-security) | v1.44.0 (Sprint 62) |
| Error JSON structured serialization | ✅ | [Security](../specifications/security.md#output-integrity) | v1.44.0 (Sprint 62) |
| Git dependency pinning | ✅ | [Security](../specifications/security.md#supply-chain) | v1.44.0 (Sprint 62) |
| CI security workflow | ✅ | [Security](../specifications/security.md#ci-cd-security) | v1.44.0 (Sprint 62) |

---

## Summary Statistics

- **Total Features**: 129
- **Implemented**: 129 (100%)
- **Planned**: 0 (0%)
- **Test Pass Rate**: 100% (1058 unit tests pass, 92 integration tests pass)
- **Latest Sprint**: Sprint 63 - Pager Exit Snapshot

---

## Related Documents

- **[Backlog](backlog.md)** - Prioritized feature backlog for future sprints
- **[Roadmap](roadmap.md)** - High-level product roadmap
- **[Specifications](../specifications/)** - Pure feature specifications
- **[Sprint History](../sprints/)** - Historical sprint planning and reviews
