---
sprint: 40
start_date: 2026-03-20
target_completion: 2026-03-20
status: Planning
---

# Sprint 40 Planning: Variable Substitution

## Sprint Overview

**Sprint Goal:** Implement YAML-based variable substitution for SQL templates, enabling parameterized queries in both batch and REPL modes.

**Sprint Theme:** SQL Templating & Parameterization

---

## Reality Check Summary

- Reviewed sprints: 37, 38, 39
- Patterns detected: None - healthy velocity, costs improving ($7.55/feature in Sprint 39)
- Decision: Feature Sprint
- Rationale: No crisis, no stuck issues, no accumulating debt. User has flagged Issue #26 (Variable Substitution) as top priority.

---

## Objectives

1. **Variable Substitution Engine** - Parse YAML parameter files and substitute variables in SQL text using `{{variable}}` markers
2. **Environment Variable Integration** - Support `$ENV.VAR_NAME` special dictionary for reading environment variables
3. **CLI Integration** - Add `-p`/`--params` flag for parameter file path, working in batch mode (`tq query`) and REPL mode
4. **Sprint 39 Remediation** - Update REQ-QUERY spec, clean up redundant utility tests

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: Variable Substitution Engine (Issue #26)

**Description:** Parse YAML parameter files and substitute `{{variable.path}}` markers in SQL text with corresponding values. Support nested YAML structures (e.g., `{{target.database.prefix}}`). Include `$ENV.*` special dictionary for environment variables (e.g., `{{$ENV.DATABASE_URI}}`).

**Acceptance Criteria:**
- [ ] AC-1: New `--params`/`-p` flag accepts path to a YAML file
- [ ] AC-2: `{{variable}}` markers in SQL are replaced with values from YAML
- [ ] AC-3: Nested YAML paths work with dot notation: `{{section.key}}` resolves `section: { key: value }`
- [ ] AC-4: `{{$ENV.VAR_NAME}}` reads from environment variables
- [ ] AC-5: Undefined variables produce clear error with variable name and available variables
- [ ] AC-6: Works with `tq query` (inline SQL), `tq query --file` (file input), and stdin
- [ ] AC-7: Works in REPL mode via `/params` metacommand to load/unload parameter files
- [ ] AC-8: Multiple `-p` flags merge parameters (later files override earlier)
- [ ] AC-9: `tq help params` topic explains variable substitution syntax and usage
- [ ] AC-10: Tab completion for `/params` metacommand in REPL
- [ ] AC-11: YAML parse errors produce actionable error messages with file path and line number

**Reference:** Issue #26, `docs/specifications/batch-mode.md#variables`

**Estimated Complexity:** High

---

#### Feature 2: Sprint 39 Remediation

**Description:** Address P0 recommendations from Sprint 39 review.

**Acceptance Criteria:**
- [ ] AC-12: REQ-QUERY specification updated to match multi-query implementation output
- [ ] AC-13: ~25 redundant utility tests removed from consumer modules (sessions.rs, sysconfig.rs, locks.rs, sample.rs) - functions already tested in monitoring_utils.rs

**Reference:** Sprint 39 review, recommendations #1 and #2

**Estimated Complexity:** Low

---

### Explicitly Out of Scope

- Script preprocessing (`@include`, `@if` directives) - separate future feature
- Variable definition via CLI flags (`--var key=value`) - may add in future sprint
- REPL inline variable definition (`:set var=value`) - future consideration
- Macro expansion - too complex for this sprint
- TOML/JSON parameter file formats - YAML only for now

---

## GitHub Issues

### Selected for Sprint
- #26: Variable Substitution (priority-high, enhancement) - **P0**

### Deferred
- #24: Query Drill-Down - partially complete, remaining items are P2
- #17, #19, #20: PMON features - medium priority, deferred
- #21, #22, #23, #25: PMON advanced features - low priority, deferred

---

## Dependencies

### External Dependencies
- `serde_yaml` crate for YAML parsing
- `serde` with derive feature (already in use)

### Prerequisite Work
- Sprint 39 complete (done)

### Blockers
- None identified

---

## Risks & Mitigation

### Risk 1: YAML library compatibility
- **Probability:** Low
- **Impact:** Medium
- **Mitigation:** `serde_yaml` is mature and widely used. Test with complex nested structures early.

### Risk 2: Variable syntax conflicts with SQL
- **Probability:** Low
- **Impact:** High
- **Mitigation:** `{{var}}` syntax (Mustache/Jinja-style) does not conflict with any SQL syntax. Double curly braces are unambiguous.

### Risk 3: Session budget for feature complexity
- **Probability:** Medium
- **Impact:** Medium
- **Mitigation:** Core substitution engine is straightforward (YAML parse + string replace). REPL integration (`/params`) can be simplified if needed.

---

## Action Items from Previous Sprint

- [ ] Update REQ-QUERY spec to match multi-query implementation (Sprint 39 rec #1)
- [ ] Remove ~25 redundant utility tests from consumer modules (Sprint 39 rec #2)

**Reference:** `docs/sprints/sprint-39-review.md`

---

## Agent Assignments

### cli-ux-designer (Sonnet)
**Responsibilities:**
- Design variable substitution UX: marker syntax, error messages, help text
- Update specifications: `docs/specifications/batch-mode.md`, `docs/specifications/cli-interface.md`, `docs/specifications/repl.md`
- Design `/params` metacommand UX

**Deliverables:**
- Updated specifications with variable substitution requirements
- `/params` metacommand specification
- `tq help params` content specification
- Error message specifications

---

### rust-teradata-architect (Opus)
**Responsibilities:**
- Implement variable substitution engine (`src/params.rs` or `src/commands/params.rs`)
- Add `--params`/`-p` CLI flag to GlobalOpts
- Integrate substitution into query execution pipeline
- Implement `/params` REPL metacommand
- Sprint 39 remediation: update specs, clean up redundant tests
- Write unit tests for all new code
- Update `docs/design/` with substitution architecture

**Deliverables:**
- Variable substitution module with comprehensive unit tests
- CLI integration
- REPL integration
- Design documentation

---

## Files Involved

### Objective 1: Variable Substitution
**Source Files:**
- `src/params.rs` (NEW) - Variable substitution engine
- `src/cli.rs` - Add `--params`/`-p` flag to GlobalOpts
- `src/main.rs` - Wire params into query execution
- `src/commands/repl/metacommands.rs` - `/params` metacommand
- `src/commands/repl/metadata_completer.rs` - Tab completion

**Test Files:**
- Unit tests in `src/params.rs` `#[cfg(test)]` module
- Integration tests for CLI flag behavior

**Documentation:**
- `docs/specifications/batch-mode.md` - Variable substitution spec
- `docs/specifications/cli-interface.md` - --params flag spec
- `docs/specifications/repl.md` - /params metacommand spec
- `docs/design/batch-mode.md` or `docs/design/params.md` - Technical design
- `docs/user/repl-guide.md` - User documentation

### Objective 2: Sprint 39 Remediation
**Source Files:**
- `src/commands/sessions.rs` - Remove redundant tests
- `src/commands/sysconfig.rs` - Remove redundant tests
- `src/commands/locks.rs` - Remove redundant tests
- `src/commands/sample.rs` - Remove redundant tests

**Documentation:**
- `docs/specifications/repl.md` - Update REQ-QUERY to match implementation

---

### quality-validator (Sonnet)
**Responsibilities:**
- Design test cases for variable substitution
- Execute all test suites
- Validate acceptance criteria

**Deliverables:**
- Test execution report
- 100% test pass rate
- Acceptance criteria validation

---

## Success Criteria

- [ ] All P0 features implemented, tested, and working
- [ ] 100% test pass rate (unit + integration)
- [ ] All acceptance criteria met
- [ ] Documentation updated
- [ ] Zero technical debt introduced
- [ ] Code quality meets project standards

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-03-20 | 1.0 | Initial sprint plan | Sprint Coordinator |
