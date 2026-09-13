# Sprint 79 Planning: Topological Schema RAG (`tq schema`)

**Sprint:** 79
**Type:** Feature Sprint
**Issue:** #55
**Objective:** Provide dense, token-compressed schema graph mapping of database objects, table relationships, primary/secondary indexes, and candidate join paths.

## Reality Check Summary
- Reviewed sprints: 76, 77, 78
- Patterns detected: None (100% test pass rate, 0 clippy warnings)
- Decision: Feature Sprint (implement `tq schema` / `schema-graph`)

## Objectives & Scope

1. **CLI Subcommand & REPL Integration:**
   - Add `tq schema [DATABASE] [TABLE_PATTERN]` (canonical, single command).
   - Add REPL metacommand `/schema`.
   - Add options: `--depth <N>`, `--format <table|json|csv|markdown|compact>`, `--json`, `--include-views`.
2. **Bulk Metadata Extraction Engine:**
   - Bulk query DBC tables (`DBC.TablesV`, `DBC.ColumnsV`, `DBC.IndicesV`, `DBC.TableSizeV`) in constant round trips.
3. **Graph & Join Path Inference Engine:**
   - Detect logical foreign keys across tables via column name & type compatibility.
   - Infer Teradata PI join recommendations (`COLOCATED_PI_JOIN`, `RECOMMENDED_PI_JOIN`, `ALL_AMPS_JOIN`).
4. **Token-Optimized Renderers:**
   - Render `table`, `json`, `markdown` (with Mermaid ER), `compact` (token-optimized adjacency), and `csv`.
5. **Quality & Validation:**
   - Zero clippy warnings, unit tests, and live Teradata integration tests against test instance.

## Acceptance Criteria

- [ ] `tq schema` runs against current database or specified database.
- [ ] REPL metacommand `/schema` displays schema summary.
- [ ] Candidate foreign keys and Teradata PI join paths are detected and scored.
- [ ] `--format json` outputs structured envelope matching specification.
- [ ] `--format compact` emits dense single-line representation saving >70% tokens.
- [ ] `--format markdown` outputs valid Markdown with Mermaid `erDiagram`.
- [ ] Live integration test passes against live Teradata instance.
- [ ] 100% test pass rate and 0 clippy warnings.
