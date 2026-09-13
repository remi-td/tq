# Technical Design: Schema Graph Topology & RAG (`tq schema`)

## Architectural Overview

`tq schema` (with aliases `tq schema-graph` and `tq sg`) extracts database schema metadata, primary index structures, column attributes, and row estimates, and computes a directed relationship graph with candidate join paths.

## Key Components

### 1. Metadata Extraction Engine
To maintain `tq`'s sub-100ms execution, metadata is retrieved in bulk rather than table-by-table:
- **`DBC.TablesV`**: Extracted for target `DatabaseName` (filtered for `TableKind IN ('T', 'O', 'V')` or TableName pattern).
- **`DBC.TableSizeV`**: Queried for estimated row counts grouped by table name.
- **`DBC.ColumnsV`**: Bulk-queried with standard Teradata type conversion `CASE` expressions.
- **`DBC.IndicesV`**: Bulk-queried to determine Primary Index (`IndexType IN ('P', 'Q')`), Unique status (`UniqueFlag`), and partitioned status (`PPI`).

### 2. Graph & Join Path Inference Model

```
SchemaGraph
├── database: String
├── tables: Vec<TableNode>
│   ├── name: String
│   ├── kind: TableKind
│   ├── row_count_est: Option<i64>
│   ├── primary_index: Vec<String>
│   ├── partition_keys: Vec<String>
│   └── columns: Vec<ColumnNode>
└── join_paths: Vec<JoinPath>
    ├── from_table: String
    ├── from_column: String
    ├── to_table: String
    ├── to_column: String
    ├── join_type: JoinType (COLOCATED_PI_JOIN | RECOMMENDED_PI_JOIN | ALL_AMPS_JOIN)
    └── cost: JoinCost (MINIMAL | LOW | MEDIUM)
```

### 3. Join Inference Heuristics
1. **Identical Column Names**: Columns across tables sharing identical names and compatible data types.
2. **Entity Suffix Conventions**: Matching `<table>_id` in child table to `id` in parent table.
3. **Teradata Optimization Scoring**:
   - `COLOCATED_PI_JOIN` (Cost: Minimal): Both columns participate in their respective tables' Primary Indexes. AMPs execute join locally with zero row redistribution.
   - `RECOMMENDED_PI_JOIN` (Cost: Low): Target table column is in Primary Index. Only the source table needs hash redistribution.
   - `ALL_AMPS_JOIN` (Cost: Medium): Neither column is a Primary Index. Both tables must be redistributed or duplicated across AMPs.

### 4. Serialization Formats
- **Table**: Formatted ASCII table summarizing objects, primary indexes, and inferred relations.
- **JSON**: Machine-readable enveloped payload matching `misc/agentic_cli_research_and_roadmap.md`.
- **Markdown**: Formatted markdown tables with generated Mermaid `erDiagram`.
- **Compact**: Dense token-optimized format (~80% token reduction for LLM prompts).
- **CSV**: Standard CSV rows for tabular ingestion.
