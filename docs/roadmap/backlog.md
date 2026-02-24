# Feature Backlog

**Last Updated:** 2026-02-24
**Next Sprint:** Sprint 40

---

## Overview

This document contains the prioritized backlog of features to be implemented in future sprints. Features are organized by priority level and include dependencies where applicable.

---

## P0 - Critical (Must Have)

No P0 features currently in backlog. All critical features have been implemented.

---

## P1 - High Priority (Should Have)

### Configuration Management

**Profile Editing Commands**
- `tq profile add <name>` - Add new profile interactively
- `tq profile edit <name>` - Edit existing profile
- `tq profile delete <name>` - Remove profile
- Spec: [Configuration - Profile Management](../specifications/configuration.md#profile-management)

### REPL Enhancements

**Second TAB Accepts Selection**
- Match bash/zsh behavior: second TAB accepts highlighted completion item
- Blocked by reedline library limitation (Issue #624 - no MenuAccept event)
- Current workaround: Press ENTER to accept selection
- Awaiting upstream reedline fix or contribution opportunity
- Spec: [REPL Mode - Tab Completion Behavior](../specifications/repl.md#tab-completion-behavior)
- Deferred from Sprint 21 due to technical limitation

---

## P2 - Medium Priority (Nice to Have)

### Performance Optimizations

**Streaming Large Results**
- Stream query results without loading all into memory
- Support datasets >1GB
- Progress indicators for long-running queries
- Spec: [Performance - Streaming](../specifications/performance.md#streaming)

**Query Result Caching**
- Cache recent query results for re-export
- Configurable cache size and TTL
- `/cache clear` to manually clear cache
- Spec: [Performance - Caching](../specifications/performance.md#caching)

### Batch Mode Enhancements

**Variable Substitution**
- `${VAR}` syntax for variable substitution in SQL
- Command-line variable passing: `--var name=value`
- Environment variable expansion
- Spec: [Batch Mode - Variables](../specifications/batch-mode.md#variables)

**Script Preprocessing**
- Include files: `-- @include common.sql`
- Conditional execution: `-- @if ENVIRONMENT=prod`
- Macro expansion for DRY scripts
- Spec: [Batch Mode - Preprocessing](../specifications/batch-mode.md#preprocessing)

### Security Enhancements

**Keyring Integration**
- Store credentials in system keyring
- OS-native credential storage (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- `tq keyring add <profile>` command
- Spec: [Security - Keyring](../specifications/security.md#keyring)

**Config Validation Command**
- `tq config validate` - Check config file syntax
- Validate profile connectivity
- Security audit (check file permissions)
- Spec: [Configuration - Validation](../specifications/configuration.md#validation)

### REPL Enhancements

**Search in Pager**
- `/pattern` - Search forward in paged results
- `n` - Next match
- `N` - Previous match
- Spec: [REPL Mode - Pager Search](../specifications/repl.md#pager-search)

### PMON - Performance Monitor Features

**Performance Summary and Resource Usage** (Issue #17)
- CPU, memory, I/O metrics per node and VPROC
- Physical vs virtual resource monitoring
- Requires ResUsage collection to be enabled
- Spec: [Admin User Stories](../specifications/admin-user-stories.md) Section 2

**Session History** (Issue #19)
- Historical session activity analysis
- Session count trends over time
- Peak usage period identification
- Spec: [Admin User Stories](../specifications/admin-user-stories.md) Section 4

**Session Control Functions** (Issue #20)
- Abort sessions, release locks, change priority
- Safety confirmations for destructive operations
- Spec: [Admin User Stories](../specifications/admin-user-stories.md) Section 5

**Query Drill-Down and Analysis** (Issue #24) - PARTIALLY COMPLETE
- ~~View SQL text for running sessions~~ ✅ `/query` command (Sprint 39)
- Explain plan step analysis
- AMP skew identification
- Spec: [Admin User Stories](../specifications/admin-user-stories.md) Section 9

**Dynamic Session Monitoring** (Issue #25)
- Auto-refresh session display (6-second intervals)
- Configurable refresh frequency
- Requires async/TUI architecture
- Spec: [Admin User Stories](../specifications/admin-user-stories.md) Section 10

---

## P3 - Low Priority (Future)

### Advanced REPL Features

**Transaction Indicators**
- `tq(tx)>` prompt when in transaction
- Visual indication of uncommitted changes
- Automatic rollback warning on exit
- Spec: [REPL Mode - Transactions](../specifications/repl.md#transactions)

**Autocorrect Suggestions**
- Detect common typos in SQL keywords
- "Did you mean: SELECT?" suggestions
- Optional auto-fix with confirmation
- Spec: [REPL Mode - Autocorrect](../specifications/repl.md#autocorrect)

**Query Cancellation**
- Ctrl-C to cancel running query
- Double Ctrl-C to force quit
- Progress feedback during long queries
- Spec: [REPL Mode - Query Cancellation](../specifications/repl.md#query-cancellation)

### PMON - Advanced Features

**Graphical Resource Displays** (Issue #21)
- CPU/IO charts, color-coded thresholds
- Requires TUI framework (ratatui or similar)
- Spec: [Admin User Stories](../specifications/admin-user-stories.md) Section 6

**Graphical Session Displays** (Issue #22)
- Session count charts, state distribution
- Requires TUI framework
- Spec: [Admin User Stories](../specifications/admin-user-stories.md) Section 7

**Alerting and Threshold Configuration** (Issue #23)
- Configurable alert thresholds for resource metrics
- Color indicators for warning conditions
- Spec: [Admin User Stories](../specifications/admin-user-stories.md) Section 8

### Output Format Extensions

**Additional Export Formats**
- Parquet format for big data tools
- HTML tables for reports
- Markdown tables for documentation
- Spec: [Output Formats - Additional Formats](../specifications/output-formats.md#additional-formats)

**Format Customization**
- Custom delimiters for CSV
- JSON array vs. newline-delimited JSON
- Custom NULL representation
- Spec: [Output Formats - Customization](../specifications/output-formats.md#customization)

---

## Dependencies

Some features depend on others being implemented first:

- **Project Config** depends on **User Config** (✅ Complete)
- **Keyring Integration** depends on **Config Validation** (for secure storage)
- **Variable Substitution** may benefit from **Preprocessing** (can share parser)
- **Transaction Indicators** depends on **Transaction Control** implementation

---

## Future Considerations

Features under consideration but not yet committed to backlog:

- **Query Performance Analysis**: EXPLAIN plan integration
- **SQL Formatting**: Auto-format SQL queries
- **Multi-Connection REPL**: Switch between multiple connections
- **Query Templates**: Saved queries with parameter placeholders
- **Diff Mode**: Compare query results across databases
- **Collaborative Features**: Share queries via gists/URLs

These will be evaluated based on user feedback and demand.

---

## Backlog Management

This backlog is reviewed during **Phase 0 (Reality Check)** of each sprint. The sprint coordinator:

1. Reviews recent sprint retrospectives
2. Identifies highest-priority features from backlog
3. Considers bug fixes and technical debt
4. Decides: Feature Sprint or Maintenance Sprint

---

## Related Documents

- **[Status Dashboard](status.md)** - Current implementation status
- **[Roadmap](roadmap.md)** - High-level product direction
- **[Specifications](../specifications/)** - Detailed feature specifications
