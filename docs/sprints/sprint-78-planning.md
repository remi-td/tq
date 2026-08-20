# Sprint 78 Planning: Parameterized SQL Script Execution

**Sprint:** 78
**Type:** Feature Sprint
**Objective:** Provide intuitive, standardized parameterization for SQL scripts executed by `tq`.

## Objectives & Scope

1. **Jinja2 & Shell Variable Syntax Support:**
   - Support `{{var}}`, `{{ var }}`, `${VAR}`, and `${ENV.VAR}` in SQL templates.
2. **CLI Definition Flag (`-D key=value` / `--define key=value`):**
   - Allow passing single parameters directly via CLI without YAML parameter files.
3. **Implicit Environment Fallback:**
   - Automatically fall back to process environment variables when markers are unmapped in parameter stores.
4. **Dry-Run Inspection (`--dry-run`):**
   - Output substituted SQL queries to stdout without database execution.

## Acceptance Criteria

- [ ] `{{ var }}` (with surrounding spaces) and `${VAR}` markers are correctly substituted.
- [ ] `-D key=value` / `--define key=value` flags parse and override parameter file values.
- [ ] Process environment variables are substituted automatically when markers exist.
- [ ] `--dry-run` prints substituted SQL query and exits with code 0 without connecting.
- [ ] `tq help params` and specifications are updated to reflect the design.
- [ ] 100% test pass rate on all unit tests.
