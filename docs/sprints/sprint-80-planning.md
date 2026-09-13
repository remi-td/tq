# Sprint 80 Planning: Token Compression Schemes & Agent Ergonomics

**Sprint:** 80
**Type:** Feature Sprint
**Specification:** `misc/token_compression_deep_dive.md`
**Objective:** Implement universal token compression schemes (`toon`, `compact`, `tsv`, cell compression, token budgeting) and eliminate AI agent choice overhead with an ergonomic `--agent` mode and `TQ_AGENT=1`.

## Reality Check Summary
- Reviewed sprints: 77, 78, 79
- Patterns detected: None (100% test pass rate, 0 clippy warnings)
- Decision: Feature Sprint (universal token compression & agent mode)

## Objectives & Scope

1. **New Token-Optimized Output Formats:**
   - Implement `toon` (Token-Oriented Object Notation, ~79% token reduction vs JSON).
   - Implement `compact` (Columnar JSON, ~68% token reduction vs JSON).
   - Implement `tsv` (Tab-Separated Values, ~72% token reduction vs JSON).
2. **Fast BPE Token Estimator & Cell Compression:**
   - Lightweight $O(N)$ allocation-free BPE token estimator.
   - Cell-level compression: float capping to 2 decimals, `NULL` as `~`, string truncation > 100 chars.
   - Dynamic token budgeting (`--token-budget <N>`): row truncation with clear overflow notice.
3. **Ergonomic AI Agent Mode (`--agent` / `TQ_AGENT=1`):**
   - Eliminate agent choice overhead by bundling optimal defaults: `toon` default (or `compact` if `--json`), `--compress-tokens`, 4000 token budget, `--agent-safe` restrictions, `--show-tokens`.
   - Preserve standard human CLI behavior when `--agent` is absent.
4. **Subcommand & Formatting Dispatch:**
   - Wire `Toon`, `Compact`, `Tsv`, and `--agent` across all queries and inspection commands.
5. **Companion Skill & Documentation:**
   - Update `agentic/skills/teradata-query/SKILL.md` with formats, options, defaults, and recommendations.

## Acceptance Criteria

- [ ] `tq query --format toon` emits dense token-oriented syntax with minimal quotes.
- [ ] `tq query --format compact` emits valid columnar JSON (`{"ok":true,"cols":[...],"rows":[[...]]}`).
- [ ] `tq query --format tsv` emits tab-delimited records.
- [ ] `--agent` and `TQ_AGENT=1` automatically activate token compression, safe guardrails, 4k budget, and `toon` format.
- [ ] `--agent --json` automatically upgrades to `compact` JSON.
- [ ] Dynamic token budget truncates row output when budget is reached.
- [ ] Live Teradata database verification passes against cloud test instance.
- [ ] 100% test pass rate and zero clippy warnings.
