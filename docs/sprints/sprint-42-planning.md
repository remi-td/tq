# Sprint 42 Planning: SQL Parser Hardening

## Sprint Overview

**Sprint Goal:** Fix critical SQL parsing bugs in `--file` mode by replacing the naive semicolon splitter with a proper quote-aware, comment-aware SQL lexer.

**Sprint Theme:** Bug Fix - SQL Parser Hardening

**Date:** 2026-03-21
**Type:** Feature Sprint (bug-fix focused)

---

## Reality Check Summary

- Reviewed sprints: 39, 40, 41
- Patterns detected: 3 NEW critical bugs (#28, #29, #30) all in SQL file parser
- Decision: Feature Sprint with bug fixes as P0
- Rationale: All 3 bugs share the same root cause (naive `sql.split(';')` in `src/sql/parser.rs`). This is core functionality that blocks real-world usage of `--file` mode. Healthy velocity from sprints 39-41 means we can focus on quality.

---

## Objectives

1. **Replace naive SQL splitter** with a proper lexer that tracks quoted strings, line comments, and block comments
2. **Fix all 3 reported bugs** (#28, #29, #30) in a single parser rewrite
3. **Sprint 41 remediation** - mark `test_repl_startup_and_quit` as `#[ignore]`, pin cross-rs version

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Quote-Aware SQL Statement Splitting (#28, #29, #30)

**Description:** Rewrite `src/sql/parser.rs::parse_statements()` to use a character-by-character lexer that correctly handles SQL lexical rules instead of naive `split(';')`.

**Acceptance Criteria:**
- [ ] AC-1: Semicolons inside single-quoted strings (`'...'`) do NOT split statements
- [ ] AC-2: Escaped quotes (`''`) inside strings are handled correctly
- [ ] AC-3: Multi-line SQL statements (newlines within a statement) are preserved as single statements
- [ ] AC-4: Line comments (`-- ...`) are stripped before statement assembly (not passed to Teradata as part of next statement)
- [ ] AC-5: Block comments (`/* ... */`) are stripped before statement assembly
- [ ] AC-6: Comments between statements do not contaminate adjacent statements
- [ ] AC-7: Empty lines between statements are handled correctly
- [ ] AC-8: Line number tracking remains accurate for error reporting
- [ ] AC-9: `has_multiple_statements()` works correctly with new parser
- [ ] AC-10: All existing parser tests pass (backwards compatible for simple cases)
- [ ] AC-11: New tests cover all 3 bug scenarios from issues #28, #29, #30
- [ ] AC-12: `ParsedStatement` struct unchanged (API compatible)

**Reference:** Issues #28, #29, #30 on GitHub

**Estimated Complexity:** Medium

---

### P1 - High Priority (Should Have)

#### Feature 2: Sprint 41 Remediation

**Description:** Address P0 items from Sprint 41 review.

**Acceptance Criteria:**
- [ ] AC-13: `test_repl_startup_and_quit` marked as `#[ignore]`
- [ ] AC-14: `cross-rs` version pinned in `.github/workflows/release.yml`
- [ ] AC-15: `TMPDIR` renamed to `TQ_TMPDIR` in `install.sh`

**Reference:** Sprint 41 Review, Section 7

**Estimated Complexity:** Low

---

### Explicitly Out of Scope

- Full SQL grammar parsing (we only need lexical awareness for splitting)
- Block comment nesting (`/* /* */ */` - not standard SQL)
- Dollar-quoted strings (PostgreSQL syntax, not Teradata)
- REPL changes (REPL already handles multi-line correctly via validator)
- New features from backlog

---

## GitHub Issues

### Selected for Sprint
- #28: Semicolons inside quoted strings split statements incorrectly (bug, P0)
- #29: Multi-line SQL statements fail in file execution mode (bug, P0)
- #30: SQL comment blocks cause parser misalignment (bug, P0)

### Deferred
- #24: Query Drill-Down (/explain, /skew) - Not this sprint, bug fixes first
- #25: Dynamic Session Monitoring - Requires async architecture

---

## Success Criteria

- [ ] All P0 acceptance criteria met
- [ ] All P1 acceptance criteria met
- [ ] 100% test pass rate (unit + integration)
- [ ] Zero regressions in existing tests
- [ ] Zero clippy warnings
- [ ] Design doc updated for SQL parser

---

## Agent Assignments

### cli-ux-designer (Sonnet)
- Update `docs/specifications/batch-mode.md` with SQL parsing requirements
- Document expected behavior for quoted strings, comments, multi-line in specs

### rust-teradata-architect (Opus)
- Rewrite `src/sql/parser.rs::parse_statements()` with proper lexer
- Write comprehensive unit tests for all bug scenarios
- Update `docs/design/batch-mode.md` with parser design
- Apply Sprint 41 remediation items

### quality-validator (Sonnet)
- Design test cases covering all 3 bug scenarios
- Execute full test suite
- Validate all acceptance criteria

---

## Files Involved

### Objective 1: SQL Parser Rewrite
**Source Files:**
- `src/sql/parser.rs` - Core parser rewrite (PRIMARY)

**Test Files:**
- Unit tests in `src/sql/parser.rs` `#[cfg(test)]` module
- `tests/` - Integration tests if needed

**Documentation:**
- `docs/specifications/batch-mode.md` - SQL parsing requirements
- `docs/design/batch-mode.md` - Parser design (create if needed)

### Objective 2: Sprint 41 Remediation
**Source Files:**
- `src/commands/repl/mod.rs` - Mark test as `#[ignore]`
- `.github/workflows/release.yml` - Pin cross-rs version
- `install.sh` - Rename TMPDIR

---

## Risks & Mitigation

### Risk 1: Comment stripping changes behavior
- **Probability:** Medium
- **Impact:** Medium
- **Mitigation:** Teradata handles comments itself. Stripping comments before sending is safe and prevents the bug. Test both approaches.

### Risk 2: Line number tracking becomes complex
- **Probability:** Low
- **Impact:** Low
- **Mitigation:** Track line numbers during lexing, not after. Each character scan updates line count.

---

## Action Items from Previous Sprint

- [ ] Mark `test_repl_startup_and_quit` as `#[ignore]` (Sprint 41 P0 #2)
- [ ] Pin `cross-rs` version in release.yml (Sprint 41 P0 #3)
- [ ] Rename `TMPDIR` to `TQ_TMPDIR` in install.sh (Sprint 41 P1 #4)

**Reference:** `docs/sprints/sprint-41-review.md` Section 7

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-03-21 | 1.0 | Initial sprint plan - SQL Parser Hardening | Sprint Coordinator |
