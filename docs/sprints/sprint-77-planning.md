# Sprint 77 Planning

**Date:** 2026-08-20
**Type:** Maintenance / Tech Debt Sprint
**Current Version:** 1.55.0 → target 1.56.0

---

## Reality Check Summary

- **Reviewed sprints:** 74, 75, 76
- **Patterns detected:**
  - *High velocity & stability* — 100% test pass rate, 0 clippy warnings.
  - *Agent Ergonomics & Debt Repayment Focus* — User explicitly requested a dedicated tech debt repayment sprint focused on design, code implementation, user and agent ergonomics, and REPL vs batch command coherence.
- **Decision:** **Maintenance & Tech Debt Repayment Sprint**
- **Objective:** Make `tq` an **exemplary tool when it comes to CLI designed for agents**, resolving command asymmetries, adding `--json` shortcuts, expanding global agent safety controls, and providing structured JSON error handling.

---

## Objectives

1. **`--json` Flag Shortcut** — Provide dedicated `--json` flag across all subcommands as a shorthand for `--format json`.
2. **Command Coherence & Parity** — Add batch subcommands (`tq params`, `tq errorlevel`) and REPL metacommands (`/fastload`, `/fastexport`, `/profile`) plus intuitive aliases (`tq qi`, `tq di`).
3. **Global Agent Safety Control (`--agent-safe` & `TQ_AGENT_SAFE`)** — Enable global `--agent-safe` flag and `TQ_AGENT_SAFE=1` environment variable.
4. **Structured JSON Errors on `stdout`** — Guarantee single-stream structured JSON error responses when `--format json` or `--json` is set.
5. **Release v1.56.0** — Ship with updated documentation, test execution proof, and tagged release.

---

## Acceptance Criteria

- [x] Every subcommand accepting `--format` also accepts `--json` as a direct boolean shortcut for `--format json`.
- [x] `tq params <file>` command inspects and validates YAML parameter files.
- [x] `tq errorlevel` command inspects error severity classification mappings.
- [x] REPL mode supports `/fastload`, `/fastexport`, and `/profile` metacommands.
- [x] Command aliases `tq qi` (for `query-inspect`) and `tq di` (for `show-indexes`) work in CLI.
- [x] `TQ_AGENT_SAFE=1` environment variable and top-level `--agent-safe` flag enforce read-only & single-statement constraints.
- [x] When `--format json` or `--json` is enabled in batch mode, errors output structured JSON `{"ok": false, "error": {...}}` on `stdout`.
- [x] 100% test pass rate with zero clippy warnings.
- [x] Version bumped to 1.56.0 with updated `docs/roadmap/status.md` and `docs/specifications/cli-interface.md`.

---

## Selected Issues & Backlog Items

- **#37 (Part 7)** — [AGENT MODE] Top-level `--json` flag shortcut & unified agent-safe environment context.
- **Tech Debt Repayment** — REPL/Batch command parity, parameter file inspector, errorlevel inspect subcommand.
