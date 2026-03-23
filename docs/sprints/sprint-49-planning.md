# Sprint 49 Planning: Session Control Functions

**Sprint Duration:** 2026-03-23 (Single-session feature sprint)
**Status:** IN PROGRESS
**Target Version:** v1.30.0
**Issue:** #20 - PMON: Session Control Functions

---

## Objectives

Implement DBA session control commands for managing problematic sessions:

1. `/abort <session_id>` — Abort a session (with confirmation)
2. `/abort query <session_id>` — Abort only the running query
3. `/priority <session_id> <level>` — Change session priority (RUSH/MEDIUM/LOW)
4. `tq abort <session_id>` — Batch mode abort
5. `tq priority <session_id> <level>` — Batch mode priority change

---

## Scope

### P0 — Must Have
- **Abort session**: Execute `ABORT SESSION <session_id>` via SQL
- **Abort query**: Execute `ABORT REQUEST <session_id>` via SQL
- **Safety confirmation**: REPL mode requires `[y/N]` confirmation before destructive ops
- **Batch mode**: `tq abort` and `tq priority` CLI commands with `--force` for non-interactive
- **Error handling**: Privilege errors, invalid session IDs, already-terminated sessions

### P1 — Should Have
- **Priority change**: Execute `SET SESSION PRIORITY <session_id> = <level>`
- **Multi-format output**: Table/CSV/JSON for batch mode results
- **Tab completion**: New commands in metacommand completion menu

### P2 — Nice to Have
- Abort by user: `/abort user <username>` (deferred — complex safety implications)
- Abort by host: `/abort host <hostname>` (deferred)
- Logoff idle: `/logoff idle --older-than <duration>` (deferred)

---

## Acceptance Criteria

### AC-1: Abort Session
- `tq abort 1234` sends `ABORT SESSION 1234` to Teradata
- REPL `/abort 1234` prompts `Abort session 1234? [y/N]` before executing
- Batch mode requires `--force` flag (no TTY prompt)
- Success message: `Session 1234 aborted.`
- Error for invalid session: `Error: Session 1234 not found or already terminated.`

### AC-2: Abort Query
- `tq abort --query 1234` sends abort request for current query only
- REPL `/abort query 1234` prompts confirmation
- Success: `Running query on session 1234 aborted.`

### AC-3: Change Priority
- `tq priority 1234 rush` changes session priority
- Valid levels: RUSH, MEDIUM, LOW
- REPL `/priority 1234 rush` executes directly (non-destructive, no confirmation needed)
- Success: `Session 1234 priority changed to RUSH.`

### AC-4: Safety
- All abort operations require confirmation in REPL mode
- Batch mode abort requires `--force` flag
- Priority changes do NOT require confirmation (non-destructive)
- Privilege errors show helpful grant instructions

### AC-5: Tab Completion
- `/abort` and `/priority` appear in metacommand completion menu

### AC-6: Tests
- Unit tests for SQL generation, output formatting, confirmation logic
- Error handling tests for privilege errors and invalid sessions

---

## Technical Approach

### SQL Commands
- Abort session: Direct SQL execution of control statements
- Priority: Uses Teradata priority management

### Module Structure
- `src/commands/abort.rs` — Abort session/query implementation
- `src/commands/priority.rs` — Priority change implementation
- CLI args in `src/cli.rs`
- REPL handlers in `src/commands/repl/metacommands.rs`
- Tab completion in `src/commands/repl/metadata_completer.rs`

### Safety Model
- REPL: Interactive `[y/N]` prompt via stdin (only for abort, not priority)
- Batch: `--force` flag required for abort operations
- No-op without confirmation (safe default)
