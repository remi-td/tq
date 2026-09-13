# Sprint 79 Test Strategy: Topological Schema RAG (`tq schema`)

**Sprint:** 79
**Feature:** `tq schema` (aliases: `schema-graph`, `sg`)
**Target:** Batch & REPL schema graph introspection and join path inference

## Test Matrix

### Unit Tests (`tests/cases/TC111` / internal module tests)

| Test ID | Description | Scope |
|---------|-------------|-------|
| `TC111-U01` | Database schema model serialization to JSON envelope | Verify `ok`, `data.database`, `data.tables`, `data.join_paths` structure |
| `TC111-U02` | Foreign key candidate inference with identical column names | Detect FK when table A and table B share column name and compatible type |
| `TC111-U03` | Foreign key candidate inference with table entity name suffix (`<table_singular>_id` -> `<table_singular>.id`) | Detect FK when `orders.customer_id` maps to `customers.id` or `customers.customer_id` |
| `TC111-U04` | Teradata Primary Index join cost scoring (`COLOCATED_PI_JOIN`, `RECOMMENDED_PI_JOIN`, `ALL_AMPS_JOIN`) | Verify join path scoring based on primary index participation |
| `TC111-U05` | Table format rendering of schema summary and join paths | Verify ASCII/ANSI table display format |
| `TC111-U06` | Markdown format rendering with Mermaid `erDiagram` | Verify valid markdown tables and syntax of Mermaid `erDiagram` |
| `TC111-U07` | Compact format rendering (dense token-optimized adjacency format) | Verify single-line notation and >70% token savings |
| `TC111-U08` | CSV format rendering of schema and relationships | Verify CSV rows and comma escaping |

### Integration Tests (Live Teradata against `trial-vikzqtnd0db0nglk.env.trial.teradata.com`)

| Test ID | Description | Scope |
|---------|-------------|-------|
| `TC111-I01` | Live `tq schema` against test database (`demo_user` / system) | Verify execution without errors against DBC views |
| `TC111-I02` | Live `tq schema-graph` and `tq sg` aliases | Verify aliases execute identically to `tq schema` |
| `TC111-I03` | Live `tq schema --format json` envelope validation | Verify valid JSON output on stdout |
| `TC111-I04` | Live `tq schema --format markdown` Mermaid validation | Verify Mermaid `erDiagram` block generated |
| `TC111-I05` | Live `tq schema --format compact` validation | Verify compact format generated |
| `TC111-I06` | End-to-end multi-table relationship test with created test tables (`test_customers`, `test_orders`) | Create tables with PIs, run `tq schema`, verify detected FK and PI join recommendation, drop tables |
