---
sprint: 72
start_date: 2026-07-10
target_completion: 2026-07-10
status: Completed
---

# Sprint 72 Planning: Custom Severity Levels (#49)

## Sprint Overview

**Sprint Goal:** Implement custom severity overrides for Teradata error codes, allowing SQL script and REPL execution to continue on warning-level errors and exit with appropriate BTEQ-compatible return codes.

**Sprint Theme:** BTEQ Compatibility and Error Resilience.

---

## Objectives

1. **Custom Severity Levels CLI support**: Parse `--errorlevel CODE [CODE...] SEVERITY` globally and apply to batch/single query execution.
2. **Interactive REPL metacommand**: Implement `/errorlevel` to view, set, or clear overrides.
3. **Resilient Batch Mode Execution**: Enable batch execution to continue on warning-level errors.

---

## Scope

### P0 - Critical (Must Have)

#### Feature 1: CLI support for custom severity overrides

- **Description**: Add `--errorlevel` flag globally. Support BTEQ severities: `warning` (4), `error` (8), `severe` (12), `fatal` (16).
- **Acceptance Criteria**:
  - [ ] Maps specified error codes to demoted severity.
  - [ ] Demoted errors print as warnings to stderr and do not halt batch script.
  - [ ] Script returns exit code 4 if warning-level errors occurred (and no higher severity error).

#### Feature 2: REPL metacommand `/errorlevel`

- **Description**: Implement `/errorlevel` inside interactive prompt.
- **Acceptance Criteria**:
  - [ ] `/errorlevel` displays active overrides.
  - [ ] `/errorlevel clear` clears active overrides.
  - [ ] `/errorlevel CODE [CODE...] SEVERITY` dynamically adds overrides.
  - [ ] Warnings during query execution in REPL print as warnings and do not abort the input or transaction.

---

## Success Criteria

- [ ] All features implemented, tested, and working as specified.
- [ ] 100% test pass rate (unit + integration).
- [ ] No regressions.

---

## Agent Assignments

### cli-ux-designer (Sonnet)
- Update `docs/specifications/cli-interface.md` with `--errorlevel` and `/errorlevel` specs.
- Ensure UX consistency.

### rust-teradata-architect (Opus)
- Implement severity levels mapping and error code extraction.
- Wire overrides into batch and single-query execution.
- Wire offsets into REPL execution.

### quality-validator (Sonnet)
- Verify test coverage and pass rate.

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-07-10 | 1.0 | Initial Sprint 72 plan | Sprint Coordinator |
