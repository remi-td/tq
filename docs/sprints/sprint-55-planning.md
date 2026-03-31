# Sprint 55 Planning: Search/Discovery Commands

**Date:** 2026-03-31
**Type:** Feature Sprint
**Status:** Planning

---

## Reality Check Summary
- Reviewed sprints: 52, 53, 54
- Patterns detected: None - healthy velocity, three consecutive feature sprints delivered (8.5, 8.8, 9.0)
- Decision: Feature Sprint
- Rationale: Issue #37 (agent mode) trajectory continues smoothly. Parts 1-4 complete. Search/discovery (part 5) is the next logical step for agent-friendly schema exploration.

---

## Sprint Goal

Deliver cross-database search commands that allow agents (and humans) to discover tables and columns by keyword without knowing exact object names or locations.

## Sprint Theme

Agent-mode search/discovery - completing Issue #37 part 5.

---

## Objectives

1. Implement `tq search tables <keyword>` batch command for cross-database table search
2. Implement `tq search columns <keyword>` batch command for cross-database column search
3. Implement `/search tables <keyword>` and `/search columns <keyword>` REPL metacommands
4. Support all output formats (table, JSON, CSV, markdown) with JSON envelope
5. Support `--database` flag to scope search to a specific database

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: `tq search tables <keyword>`

**Description:** Search for tables across all accessible databases by name pattern. Uses SQL LIKE matching with wildcards. Returns database, table name, type, estimated rows, and size.

**Acceptance Criteria:**
- [ ] `tq search tables emp` finds tables containing "emp" across all databases
- [ ] `--database` flag scopes search to a single database
- [ ] All four output formats supported (table, JSON, CSV, markdown)
- [ ] JSON output uses standard envelope (`{"ok": true, "row_count": N, "data": [...]}`)
- [ ] Agent-safe mode compatible (read-only query)
- [ ] Handles no-results gracefully

**Reference:** Issue #37 (Search/Discovery), `docs/specifications/cli-interface.md`

**Estimated Complexity:** Medium

#### Feature 2: `tq search columns <keyword>`

**Description:** Search for columns across all accessible tables/databases by name pattern. Returns database, table, column name, type, and nullable status.

**Acceptance Criteria:**
- [ ] `tq search columns salary` finds columns containing "salary" across databases
- [ ] `--database` flag scopes search to a single database
- [ ] All four output formats supported
- [ ] JSON output uses standard envelope
- [ ] Agent-safe mode compatible
- [ ] Handles no-results gracefully

**Reference:** Issue #37 (Search/Discovery)

**Estimated Complexity:** Medium

#### Feature 3: REPL integration (`/search`)

**Description:** Add `/search tables <keyword>` and `/search columns <keyword>` metacommands to the REPL with tab completion.

**Acceptance Criteria:**
- [ ] `/search tables <keyword>` works in REPL
- [ ] `/search columns <keyword>` works in REPL
- [ ] Tab completion for `/search` and subcommands
- [ ] Help text via `/search` without arguments

**Reference:** Existing REPL metacommand patterns

**Estimated Complexity:** Low

### Explicitly Out of Scope

- `tq relations <table>` (foreign key graph) - deferred to future sprint
- `tq join-path <table-a> <table-b>` - deferred to future sprint
- Result pagination (`--page`, `--page-size`) - deferred to Sprint 56
- Search by column value/content - not in scope

---

## GitHub Issues

### Selected for Sprint
- #37: Agent mode: stable JSON contracts, structured errors, safe execution, and richer introspection (part 5: search/discovery)

### Deferred
- #37 part 6 (result pagination) - next sprint
- #25: PMON Dynamic Session Monitoring - requires TUI framework
- #17: PMON Performance Summary - medium priority, deferred

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working
- [ ] 100% test pass rate (unit + integration)
- [ ] All acceptance criteria met
- [ ] Documentation updated (specifications, design)
- [ ] Zero technical debt introduced
- [ ] Zero clippy warnings

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Design search command UX (argument structure, output layout)
- Update `docs/specifications/cli-interface.md` with search commands

**Deliverables:**
- Updated CLI specification with search command details

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement `src/commands/search.rs` (table search + column search)
- Add `Search` variant to CLI command enum
- Wire into `main.rs` dispatch
- Add REPL `/search` metacommand
- Add tab completion for `/search`
- Write unit tests

**Deliverables:**
- Working search implementation with all output formats
- Unit tests for search logic and rendering

### quality-validator (Sonnet)
**Responsibilities:**
- Design test cases for search commands
- Execute all tests (unit + interactive)
- Validate acceptance criteria

**Deliverables:**
- Test execution report with 100% pass rate

---

## Files Involved

### Feature 1 & 2: Search commands
**Source Files:**
- `src/commands/search.rs` - New: search implementation
- `src/commands/mod.rs` - Add search module
- `src/cli.rs` - Add Search command variant and SearchArgs
- `src/main.rs` - Wire search dispatch

### Feature 3: REPL integration
**Source Files:**
- `src/commands/repl/mod.rs` or equivalent - Add /search handling
- `src/commands/repl/metadata_completer.rs` - Tab completion for /search

**Documentation:**
- `docs/specifications/cli-interface.md` - Search command spec
- `docs/design/cli-interface.md` - Search design notes

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-03-31 | 1.0 | Initial sprint plan | Sprint Coordinator |
