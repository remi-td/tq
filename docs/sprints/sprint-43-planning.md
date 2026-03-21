---
sprint: 43
start_date: 2026-03-21
status: Planning
---

# Sprint 43 Planning: Profile Management & Parser Polish

## Sprint Overview

**Sprint Goal:** Deliver profile management CLI commands and complete Sprint 42 parser remediation items.

**Sprint Theme:** Configuration UX & Parser Error Handling

---

## Reality Check Summary

- **Reviewed sprints:** 40 (Variable Substitution, 8.0/10), 41 (GitHub Releases, 8.0/10), 42 (SQL Parser Hardening, 8.5/10)
- **Patterns detected:** Spec/implementation alignment recurring but improving (caught in review phase each sprint)
- **Decision:** Feature Sprint
- **Rationale:** Healthy velocity, 100% test pass rates, no stuck issues, no crisis. Tech debt stable/reducing. Ready for next feature.

---

## Objectives

1. **Profile Management Commands** - Deliver `tq profile add`, `tq profile edit`, `tq profile delete` for non-interactive profile CRUD operations
2. **Sprint 42 Remediation** - Complete parser error handling (`Result` return type), spec clarifications, and missing tests

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Profile Management Commands

**Description:** Add CLI commands for managing connection profiles in `~/.tq/config.toml` without manual file editing. Non-interactive, flag-based approach (no interactive prompts) for scriptability.

**Acceptance Criteria:**
- [ ] AC-1: `tq profile add <name> --host <host> [--port <port>] [--database <db>] [--user <user>] [--logmech <mech>] [--password-file <path>]` creates a new profile
- [ ] AC-2: `tq profile edit <name> [--host <host>] [--port <port>] [--database <db>] [--user <user>] [--logmech <mech>] [--password-file <path>]` updates specified fields only
- [ ] AC-3: `tq profile delete <name>` removes a profile (with `--force` to skip confirmation prompt)
- [ ] AC-4: Error if profile already exists on `add` (unless `--force` to overwrite)
- [ ] AC-5: Error if profile doesn't exist on `edit` or `delete`
- [ ] AC-6: Creates `~/.tq/config.toml` and directory if they don't exist on `add`
- [ ] AC-7: Preserves existing config content (other profiles, defaults section) on add/edit/delete
- [ ] AC-8: Output confirms action taken (e.g., "Profile 'dev' added successfully")
- [ ] AC-9: Tab completion for profile names on edit/delete (REPL not required, just CLI)
- [ ] AC-10: `tq profile list` as alias for existing `tq profiles` command
- [ ] AC-11: Validates logmech values (TD2, LDAP, KRB5, TDNEGO)
- [ ] AC-12: Validates port is a valid number (1-65535)

**Reference:** `docs/specifications/configuration.md#profile-management` (to be created)

**Estimated Complexity:** Medium

---

#### Feature 2: Sprint 42 Parser Remediation

**Description:** Complete remediation items identified in Sprint 42 review.

**Acceptance Criteria:**
- [ ] AC-13: `parse_statements()` returns `Result<Vec<ParsedStatement>, ParseError>` for unterminated strings (REQ-PARSE-007)
- [ ] AC-14: `parse_statements()` returns `Result<Vec<ParsedStatement>, ParseError>` for unterminated block comments (REQ-PARSE-013)
- [ ] AC-15: `ParseError` includes line number and column for error location
- [ ] AC-16: All existing call sites updated for `Result` return type
- [ ] AC-17: `test_comment_marker_inside_string_is_not_comment` test added
- [ ] AC-18: REQ-PARSE-015 "begins accumulating" wording clarified in spec
- [ ] AC-19: Space-injection behavior documented in spec and design doc
- [ ] AC-20: Explanatory comment added for `unwrap()` at parser.rs:178

**Reference:** Sprint 42 review recommendations #1-5

**Estimated Complexity:** Low

---

### Explicitly Out of Scope

- Interactive prompts for profile add/edit (flag-based only for v1)
- Profile management in REPL mode (CLI only)
- Project config (`.tq.toml`) profile management (user config only)
- Double-quoted identifier handling in parser (Sprint 42 backlog item)
- Profile import/export between machines

---

## GitHub Issues

### Selected for Sprint
- No new GitHub issues selected. Sprint focuses on top P1 backlog item (Profile Management) and Sprint 42 remediation.

### Deferred
- #24: Query Drill-Down (partially complete, /explain and /skew remaining - larger scope)
- #17: Performance Summary and Resource Usage (PMON - requires ResUsage)
- #19-#25: Other PMON features (future sprints)

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working as specified
- [ ] 100% test pass rate (unit + integration tests)
- [ ] All acceptance criteria met for delivered features
- [ ] Documentation updated to reflect new features
- [ ] Zero technical debt introduced
- [ ] Code quality meets project standards

---

## Action Items from Previous Sprint

- [ ] Implement `Result<Vec<ParsedStatement>, ParseError>` for unterminated constructs (Sprint 42 #1)
- [ ] Add `test_comment_marker_inside_string_is_not_comment` test (Sprint 42 #2)
- [ ] Clarify REQ-PARSE-015 wording (Sprint 42 #3)
- [ ] Document space-injection behavior (Sprint 42 #4)
- [ ] Add comment for unwrap() at parser.rs:178 (Sprint 42 #5)

**Reference:** `docs/sprints/sprint-42-review.md` Section 7

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Write profile management specification in `docs/specifications/configuration.md`
- Define CLI interface, error messages, output formats
- Clarify REQ-PARSE-015 and document space-injection behavior in batch-mode spec

**Deliverables:**
- Profile management requirements (REQ-PROFILE-xxx)
- Updated CLI interface specification
- Parser spec clarifications

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement profile add/edit/delete commands
- Implement parser `Result` return type change
- Write unit tests for all new code
- Update design docs

**Deliverables:**
- `src/commands/profile.rs` - Profile management implementation
- Updated `src/sql/parser.rs` - Result return type
- Updated `src/cli.rs` - Profile subcommands
- Unit tests with 100% pass rate
- Updated `docs/design/` documents

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design test cases for profile management
- Execute all test suites
- Validate acceptance criteria

**Deliverables:**
- Test cases in `tests/cases/TC-043-*.md`
- Test execution report
- 100% test pass rate

---

## Files Involved

### Objective 1: Profile Management Commands
**Source Files:**
- `src/commands/profile.rs` - New: Profile management implementation
- `src/cli.rs` - Add profile subcommands
- `src/main.rs` - Wire profile commands
- `src/config.rs` - Config read/write utilities (may need write support)

**Test Files:**
- Unit tests in `src/commands/profile.rs`
- Integration tests if applicable

**Documentation:**
- `docs/specifications/configuration.md` - Profile management section
- `docs/design/configuration.md` - Technical design
- `docs/user/repl-guide.md` - User guide updates

### Objective 2: Sprint 42 Parser Remediation
**Source Files:**
- `src/sql/parser.rs` - Result return type, tests
- All call sites of `parse_statements()`

**Documentation:**
- `docs/specifications/batch-mode.md` - REQ-PARSE-015, space-injection
- `docs/design/batch-mode.md` - Space-injection documentation

---

## Risks & Mitigation

### Risk 1: TOML Serialization Complexity
- **Probability:** Medium
- **Impact:** Medium
- **Mitigation:** Use `toml` crate's serialization. If preserving comments/formatting is hard, accept that write operations may reformat the file (document this behavior).

### Risk 2: Parser API Change Blast Radius
- **Probability:** Low
- **Impact:** Low
- **Mitigation:** `parse_statements` call sites are limited. The change from `Vec` to `Result<Vec, E>` is mechanical.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-03-21 | 1.0 | Initial sprint plan | Sprint Coordinator |
