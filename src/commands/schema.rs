//! Schema command implementation
//!
//! Extracts a topological schema graph of database objects, primary indexes,
//! column definitions, row count estimates, and inferred relationship join paths.
//! Designed for token-optimized RAG in AI agent workflows as well as human exploration.

use crate::cli::{SchemaArgs, SchemaFormat};
use crate::commands::format_helpers::{
    classify_index, column_type_case_sql, csv_escape, map_table_kind,
};
use crate::commands::query_helpers;
use crate::db::{DatabaseClient, Value};
use crate::error::Result;
use crate::sql::escape_sql_string;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;

// =============================================================================
// Schema Graph Data Model
// =============================================================================

/// Column definition in the schema graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaColumn {
    pub name: String,
    pub col_type: String,
    pub nullable: bool,
    pub is_primary_index: bool,
    pub is_partition_key: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fk_candidate: Option<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub comment: String,
}

/// Index definition in the schema graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaIndex {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub index_type: String,
    pub short_label: String,
    pub is_primary: bool,
    pub columns: Vec<String>,
}

/// Table definition in the schema graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaTable {
    pub name: String,
    pub kind: String,
    pub table_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_count_est: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<i64>,
    pub primary_index: Vec<String>,
    pub partition_keys: Vec<String>,
    pub columns: Vec<SchemaColumn>,
    pub indexes: Vec<SchemaIndex>,
}

/// Inferred join path between two tables
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaJoinPath {
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    pub join_type: String,
    pub cost: String,
    pub reason: String,
}

/// Inner data structure for schema graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaGraphData {
    pub database: String,
    pub table_count: usize,
    pub relationship_count: usize,
    pub tables: Vec<SchemaTable>,
    pub join_paths: Vec<SchemaJoinPath>,
}

/// Top-level enveloped schema graph payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaGraphEnvelope {
    pub ok: bool,
    pub row_count: usize,
    pub data: SchemaGraphData,
}

// =============================================================================
// Public Execution Entry Points
// =============================================================================

/// Execute `tq schema` in batch mode
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &SchemaArgs,
    writer: &mut W,
    use_color: bool,
) -> Result<()> {
    let resolved_db = query_helpers::resolve_database(client, args.database.as_deref())?;
    let mut data = fetch_schema_graph(
        client,
        &resolved_db,
        args.pattern.as_deref(),
        args.include_views,
    )?;

    // Infer candidate relationships and score join paths
    infer_relationships_and_joins(&mut data, args.depth);

    let format = if args.json {
        SchemaFormat::Json
    } else {
        args.format
    };

    match format {
        SchemaFormat::Json => render_json(&data, writer)?,
        SchemaFormat::Table => render_table(&data, writer, use_color)?,
        SchemaFormat::Markdown | SchemaFormat::Md => render_markdown(&data, writer)?,
        SchemaFormat::Compact => render_compact(&data, writer)?,
        SchemaFormat::Toon => render_compact(&data, writer)?,
        SchemaFormat::Csv | SchemaFormat::Tsv => render_csv(&data, writer)?,
    }

    Ok(())
}

/// Execute `/schema` in REPL mode
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    database: Option<&str>,
    pattern: Option<&str>,
    writer: &mut W,
) -> Result<()> {
    let resolved_db = query_helpers::resolve_database(client, database)?;
    let mut data = fetch_schema_graph(client, &resolved_db, pattern, false)?;
    infer_relationships_and_joins(&mut data, 2);

    writeln!(writer)?;
    render_table(&data, writer, true)?;
    writeln!(writer)?;
    Ok(())
}

// =============================================================================
// Metadata Extraction Engine (Bulk Queries)
// =============================================================================

/// Fetch database tables, columns, indexes, and row count metrics in constant round-trips
pub fn fetch_schema_graph(
    client: &DatabaseClient,
    database: &str,
    pattern: Option<&str>,
    include_views: bool,
) -> Result<SchemaGraphData> {
    let db_escaped = escape_sql_string(database);

    // 1. Fetch tables from DBC.TablesV
    let kind_filter = if include_views {
        "'T', 'O', 'V'"
    } else {
        "'T', 'O'"
    };

    let sql_tables = format!(
        "SELECT TRIM(DatabaseName), TRIM(TableName), TRIM(TableKind) \
         FROM DBC.TablesV \
         WHERE DatabaseName = '{}' AND TableKind IN ({}) \
         ORDER BY TableName",
        db_escaped, kind_filter
    );

    let table_rows = client.execute(&sql_tables)?.rows;

    let regex_pattern = pattern.map(glob_pattern_to_regex);

    let mut target_tables: Vec<(String, String)> = Vec::new();
    for row in &table_rows {
        let name = row
            .get(1)
            .map(|v| v.display().trim().to_string())
            .unwrap_or_default();
        let kind = row
            .get(2)
            .map(|v| v.display().trim().to_string())
            .unwrap_or_default();

        if let Some(ref re) = regex_pattern {
            if !re.is_match(&name) {
                continue;
            }
        }
        target_tables.push((name, kind));
    }

    if target_tables.is_empty() {
        return Ok(SchemaGraphData {
            database: database.to_string(),
            table_count: 0,
            relationship_count: 0,
            tables: Vec::new(),
            join_paths: Vec::new(),
        });
    }

    // 2. Fetch row count estimates from DBC.StatsV (graceful fallback)
    let mut row_counts: HashMap<String, i64> = HashMap::new();
    let sql_stats = format!(
        "SELECT TRIM(TableName), MAX(RowCount) \
         FROM DBC.StatsV \
         WHERE DatabaseName = '{}' \
         GROUP BY 1",
        db_escaped
    );
    if let Ok(res) = client.execute(&sql_stats) {
        for row in res.rows {
            if let (Some(t), Some(c)) = (row.first(), row.get(1)) {
                let t_name = t.display().trim().to_uppercase();
                if let Ok(count) = c.display().trim().parse::<i64>() {
                    row_counts.insert(t_name, count);
                }
            }
        }
    }

    // 3. Fetch sizes in bytes from DBC.TableSizeV (graceful fallback)
    let mut table_sizes: HashMap<String, i64> = HashMap::new();
    let sql_sizes = format!(
        "SELECT TRIM(TableName), CAST(SUM(CurrentPerm) AS BIGINT) \
         FROM DBC.TableSizeV \
         WHERE DatabaseName = '{}' \
         GROUP BY 1",
        db_escaped
    );
    if let Ok(res) = client.execute(&sql_sizes) {
        for row in res.rows {
            if let (Some(t), Some(s)) = (row.first(), row.get(1)) {
                let t_name = t.display().trim().to_uppercase();
                if let Ok(size) = s.display().trim().parse::<i64>() {
                    table_sizes.insert(t_name, size);
                }
            }
        }
    }

    // 4. Fetch columns from DBC.ColumnsV
    let type_expr = column_type_case_sql();
    let sql_columns = format!(
        "SELECT TRIM(TableName), TRIM(ColumnName), ColumnId, Nullable, \
         COALESCE(TRIM(CommentString), ''), \
         {} AS ColType \
         FROM DBC.ColumnsV \
         WHERE DatabaseName = '{}' \
         ORDER BY TableName, ColumnId",
        type_expr, db_escaped
    );

    let mut columns_by_table: HashMap<String, Vec<SchemaColumn>> = HashMap::new();
    if let Ok(res) = client.execute(&sql_columns) {
        for row in res.rows {
            let t_name = row
                .first()
                .map(|v| v.display().trim().to_uppercase())
                .unwrap_or_default();
            let c_name = row
                .get(1)
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();
            let nullable_str = row
                .get(3)
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();
            let nullable = matches!(nullable_str.as_str(), "Y" | "YES" | "1");
            let comment = row
                .get(4)
                .map(|v| {
                    let s = v.display().trim().to_string();
                    if s == "[NULL]" {
                        String::new()
                    } else {
                        s
                    }
                })
                .unwrap_or_default();
            let col_type = row
                .get(5)
                .map(|v| {
                    let s = v.display().trim().to_string();
                    if s == "[NULL]" {
                        String::new()
                    } else {
                        s
                    }
                })
                .unwrap_or_default();

            columns_by_table
                .entry(t_name)
                .or_default()
                .push(SchemaColumn {
                    name: c_name,
                    col_type,
                    nullable,
                    is_primary_index: false,
                    is_partition_key: false,
                    fk_candidate: None,
                    comment,
                });
        }
    }

    // 5. Fetch indexes from DBC.IndicesV
    let sql_indexes = format!(
        "SELECT TRIM(TableName), TRIM(IndexName), IndexType, UniqueFlag, \
         TRIM(ColumnName), IndexNumber, ColumnPosition \
         FROM DBC.IndicesV \
         WHERE DatabaseName = '{}' \
         ORDER BY TableName, IndexNumber, ColumnPosition",
        db_escaped
    );

    // Group raw index rows by Table -> IndexNumber -> (IndexInfo, Vec<ColumnName>)
    struct RawIndexGroup {
        name: Option<String>,
        index_type: String,
        is_unique: bool,
        columns: Vec<String>,
    }

    let mut indexes_by_table: HashMap<String, HashMap<i32, RawIndexGroup>> = HashMap::new();
    if let Ok(res) = client.execute(&sql_indexes) {
        for row in res.rows {
            let t_name = row
                .first()
                .map(|v| v.display().trim().to_uppercase())
                .unwrap_or_default();
            let idx_name_raw = row
                .get(1)
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();
            let idx_name = if idx_name_raw.is_empty() || idx_name_raw == "[NULL]" {
                None
            } else {
                Some(idx_name_raw)
            };
            let idx_type = row
                .get(2)
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();
            let unique_flag = row
                .get(3)
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();
            let is_unique = matches!(unique_flag.as_str(), "Y" | "YES" | "1");
            let col_name = row
                .get(4)
                .map(|v| v.display().trim().to_string())
                .unwrap_or_default();
            let idx_num = match row.get(5) {
                Some(Value::Integer(n)) => *n as i32,
                Some(v) => v.display().trim().parse::<i32>().unwrap_or(1),
                None => 1,
            };

            let table_map = indexes_by_table.entry(t_name).or_default();
            let entry = table_map.entry(idx_num).or_insert_with(|| RawIndexGroup {
                name: idx_name,
                index_type: idx_type.clone(),
                is_unique,
                columns: Vec::new(),
            });
            entry.columns.push(col_name);
        }
    }

    // 6. Assemble Tables
    let mut tables: Vec<SchemaTable> = Vec::new();
    for (tbl_name, tbl_kind) in target_tables {
        let tbl_upper = tbl_name.to_uppercase();
        let row_count_est = row_counts.get(&tbl_upper).copied();
        let size_bytes = table_sizes.get(&tbl_upper).copied();

        // Convert index groups
        let mut indexes: Vec<SchemaIndex> = Vec::new();
        let mut primary_index: Vec<String> = Vec::new();
        let mut partition_keys: Vec<String> = Vec::new();

        if let Some(tbl_indexes) = indexes_by_table.remove(&tbl_upper) {
            let mut sorted_keys: Vec<i32> = tbl_indexes.keys().copied().collect();
            sorted_keys.sort();

            for k in sorted_keys {
                if let Some(raw) = tbl_indexes.get(&k) {
                    let (type_label, short_label) =
                        classify_index(&raw.index_type, raw.is_unique);
                    let is_primary =
                        raw.index_type.trim() == "P" || raw.index_type.trim() == "Q";

                    if is_primary && primary_index.is_empty() {
                        primary_index = raw.columns.clone();
                    }

                    if raw.index_type.trim() == "Q" {
                        partition_keys.extend(raw.columns.clone());
                    }

                    indexes.push(SchemaIndex {
                        name: raw.name.clone(),
                        index_type: type_label.to_string(),
                        short_label: short_label.to_string(),
                        is_primary,
                        columns: raw.columns.clone(),
                    });
                }
            }
        }

        // Attach primary index and partition flags to columns
        let mut columns = columns_by_table.remove(&tbl_upper).unwrap_or_default();
        for col in &mut columns {
            let col_upper = col.name.to_uppercase();
            if primary_index
                .iter()
                .any(|pi| pi.to_uppercase() == col_upper)
            {
                col.is_primary_index = true;
            }
            if partition_keys
                .iter()
                .any(|pk| pk.to_uppercase() == col_upper)
            {
                col.is_partition_key = true;
            }
        }

        tables.push(SchemaTable {
            name: tbl_name,
            kind: map_table_kind(&tbl_kind),
            table_kind: tbl_kind,
            row_count_est,
            size_bytes,
            primary_index,
            partition_keys,
            columns,
            indexes,
        });
    }

    Ok(SchemaGraphData {
        database: database.to_string(),
        table_count: tables.len(),
        relationship_count: 0,
        tables,
        join_paths: Vec::new(),
    })
}

// =============================================================================
// Graph Relationship & Join Path Inference Engine
// =============================================================================

/// Infer logical foreign key relationships and score Teradata join paths
pub fn infer_relationships_and_joins(data: &mut SchemaGraphData, depth: usize) {
    if depth == 0 || data.tables.len() < 2 {
        data.relationship_count = 0;
        data.join_paths.clear();
        return;
    }

    let mut inferred_joins: Vec<SchemaJoinPath> = Vec::new();
    let mut seen_pairs: HashSet<(String, String, String, String)> = HashSet::new();

    // Map table name to index in data.tables
    let table_indices: HashMap<String, usize> = data
        .tables
        .iter()
        .enumerate()
        .map(|(idx, t)| (t.name.to_uppercase(), idx))
        .collect();

    // Inspect table pairs (i, j) where i != j
    let table_count = data.tables.len();
    for i in 0..table_count {
        for j in 0..table_count {
            if i == j {
                continue;
            }

            let t_from = &data.tables[i];
            let t_to = &data.tables[j];

            for col_from in &t_from.columns {
                let cand_from = ColumnCandidate {
                    table_name: &t_from.name,
                    col_name: &col_from.name,
                    col_type: &col_from.col_type,
                    primary_index: &t_from.primary_index,
                };
                for col_to in &t_to.columns {
                    let cand_to = ColumnCandidate {
                        table_name: &t_to.name,
                        col_name: &col_to.name,
                        col_type: &col_to.col_type,
                        primary_index: &t_to.primary_index,
                    };
                    if !is_candidate_foreign_key(&cand_from, &cand_to) {
                        continue;
                    }

                    // Avoid duplicate undirected edges
                    let key = (
                        t_from.name.to_uppercase(),
                        col_from.name.to_uppercase(),
                        t_to.name.to_uppercase(),
                        col_to.name.to_uppercase(),
                    );
                    let rev_key = (
                        t_to.name.to_uppercase(),
                        col_to.name.to_uppercase(),
                        t_from.name.to_uppercase(),
                        col_from.name.to_uppercase(),
                    );

                    if seen_pairs.contains(&key) || seen_pairs.contains(&rev_key) {
                        continue;
                    }
                    seen_pairs.insert(key);

                    // Score Teradata PI join optimization
                    let from_is_pi = t_from
                        .primary_index
                        .iter()
                        .any(|p| p.eq_ignore_ascii_case(&col_from.name));
                    let to_is_pi = t_to
                        .primary_index
                        .iter()
                        .any(|p| p.eq_ignore_ascii_case(&col_to.name));

                    let (join_type, cost, reason) = if from_is_pi && to_is_pi {
                        (
                            "COLOCATED_PI_JOIN".to_string(),
                            "MINIMAL".to_string(),
                            "Both join columns are Primary Indexes; collocated join on same AMP with zero row redistribution"
                                .to_string(),
                        )
                    } else if to_is_pi {
                        (
                            "RECOMMENDED_PI_JOIN".to_string(),
                            "LOW".to_string(),
                            format!(
                                "Target column {}.{} is Primary Index; single-table redistribution only",
                                t_to.name, col_to.name
                            ),
                        )
                    } else if from_is_pi {
                        (
                            "RECOMMENDED_PI_JOIN".to_string(),
                            "LOW".to_string(),
                            format!(
                                "Source column {}.{} is Primary Index; single-table redistribution only",
                                t_from.name, col_from.name
                            ),
                        )
                    } else {
                        (
                            "ALL_AMPS_JOIN".to_string(),
                            "MEDIUM".to_string(),
                            "Neither column is Primary Index; requires full redistribution or duplication across AMPs"
                                .to_string(),
                        )
                    };

                    inferred_joins.push(SchemaJoinPath {
                        from_table: t_from.name.clone(),
                        from_column: col_from.name.clone(),
                        to_table: t_to.name.clone(),
                        to_column: col_to.name.clone(),
                        join_type,
                        cost,
                        reason,
                    });
                }
            }
        }
    }

    // Populate fk_candidate on the matching column in data.tables
    for join in &inferred_joins {
        if let Some(&idx) = table_indices.get(&join.from_table.to_uppercase()) {
            if let Some(col) = data.tables[idx]
                .columns
                .iter_mut()
                .find(|c| c.name.eq_ignore_ascii_case(&join.from_column))
            {
                if col.fk_candidate.is_none() {
                    col.fk_candidate = Some(format!("{}.{}", join.to_table, join.to_column));
                }
            }
        }
    }

    data.relationship_count = inferred_joins.len();
    data.join_paths = inferred_joins;
}

/// Context for checking candidate foreign key relationships
pub struct ColumnCandidate<'a> {
    pub table_name: &'a str,
    pub col_name: &'a str,
    pub col_type: &'a str,
    pub primary_index: &'a [String],
}

/// Determine whether two columns form a candidate logical foreign key relationship
pub fn is_candidate_foreign_key(from: &ColumnCandidate, to: &ColumnCandidate) -> bool {
    if !are_types_compatible(from.col_type, to.col_type) {
        return false;
    }

    let cf = from.col_name.to_ascii_lowercase();
    let ct = to.col_name.to_ascii_lowercase();
    let tf = from.table_name.to_ascii_lowercase();
    let tt = to.table_name.to_ascii_lowercase();

    // Check if column name has an identifier signature
    let is_key_name = cf.ends_with("_id")
        || cf.ends_with("_no")
        || cf.ends_with("_num")
        || cf.ends_with("_code")
        || cf.ends_with("_key")
        || cf == "id"
        || ct.ends_with("_id")
        || ct == "id";

    if !is_key_name {
        return false;
    }

    let from_has_pi = from
        .primary_index
        .iter()
        .any(|p| p.eq_ignore_ascii_case(from.col_name));
    let to_has_pi = to
        .primary_index
        .iter()
        .any(|p| p.eq_ignore_ascii_case(to.col_name));

    // Pattern 1: Exact column name match (e.g. orders.customer_id == customers.customer_id)
    if cf == ct {
        // If one table has it as PI, the other table references it
        if to_has_pi && !from_has_pi {
            return true;
        }
        if from_has_pi && !to_has_pi {
            return false;
        }

        // Entity stem check (e.g. table is "customers", column is "customer_id")
        let stem_to = strip_entity_plural(&tt);
        let stem_from = strip_entity_plural(&tf);
        if cf.starts_with(&stem_to) && !cf.starts_with(&stem_from) {
            return true;
        }
        if cf.starts_with(&stem_from) && !cf.starts_with(&stem_to) {
            return false;
        }

        // Canonical alphabetical ordering to avoid duplicate reverse edge
        return tf < tt;
    }

    // Pattern 2: Entity suffix to id (e.g. orders.customer_id == customers.id)
    let stem_to = strip_entity_plural(&tt);
    if ct == "id" && (cf == format!("{}_id", stem_to) || cf == format!("{}id", stem_to)) {
        return true;
    }

    false
}

/// Check if two Teradata column types are functionally compatible for joins
pub fn are_types_compatible(type_a: &str, type_b: &str) -> bool {
    let a = type_a.to_ascii_uppercase();
    let b = type_b.to_ascii_uppercase();

    // Direct match
    if a == b {
        return true;
    }

    // Integer family
    let is_int_a = a.starts_with("INT")
        || a.starts_with("BIGINT")
        || a.starts_with("SMALLINT")
        || a.starts_with("BYTEINT")
        || a == "I"
        || a == "I1"
        || a == "I2"
        || a == "I8";
    let is_int_b = b.starts_with("INT")
        || b.starts_with("BIGINT")
        || b.starts_with("SMALLINT")
        || b.starts_with("BYTEINT")
        || b == "I"
        || b == "I1"
        || b == "I2"
        || b == "I8";
    if is_int_a && is_int_b {
        return true;
    }

    // String family
    let is_str_a = a.starts_with("VARCHAR") || a.starts_with("CHAR") || a == "CV" || a == "CF";
    let is_str_b = b.starts_with("VARCHAR") || b.starts_with("CHAR") || b == "CV" || b == "CF";
    if is_str_a && is_str_b {
        return true;
    }

    // Decimal family
    let is_dec_a = a.starts_with("DECIMAL") || a.starts_with("NUMBER") || a == "D" || a == "N";
    let is_dec_b = b.starts_with("DECIMAL") || b.starts_with("NUMBER") || b == "D" || b == "N";
    if is_dec_a && is_dec_b {
        return true;
    }

    false
}

/// Normalize an entity plural table name to its singular stem (e.g. "customers" -> "customer")
fn strip_entity_plural(name: &str) -> String {
    if name.len() > 3 && name.ends_with('s') && !name.ends_with("ss") {
        name[..name.len() - 1].to_string()
    } else {
        name.to_string()
    }
}

/// Convert a shell glob pattern (e.g. `order*`, `*cust*`) to a regex
fn glob_pattern_to_regex(pattern: &str) -> regex::Regex {
    let mut re = String::from("^");
    for c in pattern.chars() {
        match c {
            '*' => re.push_str(".*"),
            '?' => re.push('.'),
            '.' | '(' | ')' | '+' | '|' | '^' | '$' | '@' | '%' | '[' | ']' | '{' | '}' | '\\' => {
                re.push('\\');
                re.push(c);
            }
            other => re.push(other),
        }
    }
    re.push('$');
    regex::RegexBuilder::new(&re)
        .case_insensitive(true)
        .build()
        .unwrap_or_else(|_| regex::Regex::new(".*").unwrap())
}

// =============================================================================
// Output Format Renderers
// =============================================================================

/// Render schema graph as formatted terminal tables
pub fn render_table<W: Write>(
    data: &SchemaGraphData,
    writer: &mut W,
    use_color: bool,
) -> Result<()> {
    if data.tables.is_empty() {
        writeln!(writer, "No tables found in database '{}'.", data.database)?;
        return Ok(());
    }

    writeln!(
        writer,
        "Database Schema: {} ({} tables, {} relationships)",
        data.database, data.table_count, data.relationship_count
    )?;
    writeln!(writer)?;

    // Section 1: Tables and Indexes
    writeln!(writer, "── Tables & Indexes ──")?;
    let mut table_chart = Table::new();
    table_chart
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table_chart.set_header(vec![
        Cell::new("Table Name"),
        Cell::new("Type"),
        Cell::new("Rows (Est.)"),
        Cell::new("Primary Index"),
        Cell::new("Partition Key"),
        Cell::new("Columns"),
    ]);

    for t in &data.tables {
        let rows_display = t
            .row_count_est
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string());
        let pi_display = if t.primary_index.is_empty() {
            "NoPI".to_string()
        } else {
            t.primary_index.join(", ")
        };
        let ppi_display = if t.partition_keys.is_empty() {
            "-".to_string()
        } else {
            t.partition_keys.join(", ")
        };
        let col_summary = format!(
            "{} cols ({})",
            t.columns.len(),
            t.columns
                .iter()
                .map(|c| c.name.as_str())
                .take(4)
                .collect::<Vec<_>>()
                .join(", ")
                + if t.columns.len() > 4 { ", …" } else { "" }
        );

        let mut row_cells = vec![
            Cell::new(&t.name),
            Cell::new(&t.kind),
            Cell::new(&rows_display),
            Cell::new(&pi_display),
            Cell::new(&ppi_display),
            Cell::new(&col_summary),
        ];

        if use_color && !t.primary_index.is_empty() {
            row_cells[3] = row_cells[3].clone().fg(Color::Cyan);
        }

        table_chart.add_row(row_cells);
    }

    writeln!(writer, "{}", table_chart)?;

    // Section 2: Inferred Relationships & Join Paths
    if !data.join_paths.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "── Inferred Relationships & Join Paths ──")?;
        let mut join_chart = Table::new();
        join_chart
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        join_chart.set_header(vec![
            Cell::new("Relationship Path"),
            Cell::new("Join Type"),
            Cell::new("Cost"),
            Cell::new("Teradata Optimization Note"),
        ]);

        for j in &data.join_paths {
            let path_str = format!(
                "{}.{} -> {}.{}",
                j.from_table, j.from_column, j.to_table, j.to_column
            );
            let mut row = vec![
                Cell::new(&path_str),
                Cell::new(&j.join_type),
                Cell::new(&j.cost),
                Cell::new(&j.reason),
            ];

            if use_color {
                match j.cost.as_str() {
                    "MINIMAL" => {
                        row[1] = row[1].clone().fg(Color::Green);
                        row[2] = row[2].clone().fg(Color::Green);
                    }
                    "LOW" => {
                        row[1] = row[1].clone().fg(Color::Cyan);
                        row[2] = row[2].clone().fg(Color::Cyan);
                    }
                    _ => {
                        row[1] = row[1].clone().fg(Color::Yellow);
                        row[2] = row[2].clone().fg(Color::Yellow);
                    }
                }
            }

            join_chart.add_row(row);
        }

        writeln!(writer, "{}", join_chart)?;
    }

    Ok(())
}

/// Render schema graph as structured JSON envelope
pub fn render_json<W: Write>(data: &SchemaGraphData, writer: &mut W) -> Result<()> {
    let envelope = SchemaGraphEnvelope {
        ok: true,
        row_count: data.tables.len(),
        data: data.clone(),
    };
    let json_str = serde_json::to_string_pretty(&envelope)?;
    writeln!(writer, "{}", json_str)?;
    Ok(())
}

/// Render schema graph as Markdown with Mermaid ER diagram
pub fn render_markdown<W: Write>(data: &SchemaGraphData, writer: &mut W) -> Result<()> {
    writeln!(writer, "# Schema Graph: `{}`", data.database)?;
    writeln!(writer)?;
    writeln!(
        writer,
        "- **Tables:** {}",
        data.table_count
    )?;
    writeln!(
        writer,
        "- **Inferred Relationships:** {}",
        data.relationship_count
    )?;
    writeln!(writer)?;

    // Mermaid ER Diagram
    if !data.tables.is_empty() {
        writeln!(writer, "```mermaid")?;
        writeln!(writer, "erDiagram")?;

        // Render relationships in Mermaid
        for j in &data.join_paths {
            let from_clean = sanitize_mermaid_ident(&j.from_table);
            let to_clean = sanitize_mermaid_ident(&j.to_table);
            writeln!(
                writer,
                "    {} }}|..|| {} : \"{}\"",
                from_clean, to_clean, j.from_column
            )?;
        }

        // Render table blocks in Mermaid
        for t in &data.tables {
            let t_clean = sanitize_mermaid_ident(&t.name);
            writeln!(writer, "    {} {{", t_clean)?;
            for col in &t.columns {
                let col_clean = sanitize_mermaid_ident(&col.name);
                let type_clean = col
                    .col_type
                    .split('(')
                    .next()
                    .unwrap_or(&col.col_type)
                    .to_ascii_lowercase();
                let key_tag = if col.is_primary_index {
                    " PK"
                } else if col.fk_candidate.is_some() {
                    " FK"
                } else {
                    ""
                };
                writeln!(writer, "        {} {}{}", type_clean, col_clean, key_tag)?;
            }
            writeln!(writer, "    }}")?;
        }
        writeln!(writer, "```")?;
        writeln!(writer)?;
    }

    // Markdown Table of Tables
    writeln!(writer, "## Tables")?;
    writeln!(writer)?;
    writeln!(
        writer,
        "| Table | Type | Estimated Rows | Primary Index | Partition Key | Columns |"
    )?;
    writeln!(
        writer,
        "| :--- | :--- | :---: | :--- | :--- | :--- |"
    )?;
    for t in &data.tables {
        let rows = t
            .row_count_est
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string());
        let pi = if t.primary_index.is_empty() {
            "NoPI"
        } else {
            &t.primary_index.join(", ")
        };
        let ppi = if t.partition_keys.is_empty() {
            "-"
        } else {
            &t.partition_keys.join(", ")
        };
        let cols = t
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            writer,
            "| {} | {} | {} | {} | {} | {} |",
            t.name, t.kind, rows, pi, ppi, cols
        )?;
    }
    writeln!(writer)?;

    // Markdown Table of Join Paths
    if !data.join_paths.is_empty() {
        writeln!(writer, "## Join Paths & Optimization")?;
        writeln!(writer)?;
        writeln!(
            writer,
            "| From | To | Join Type | Cost | Optimization Note |"
        )?;
        writeln!(
            writer,
            "| :--- | :--- | :--- | :---: | :--- |"
        )?;
        for j in &data.join_paths {
            let from_str = format!("{}.{}", j.from_table, j.from_column);
            let to_str = format!("{}.{}", j.to_table, j.to_column);
            writeln!(
                writer,
                "| {} | {} | {} | {} | {} |",
                from_str, to_str, j.join_type, j.cost, j.reason
            )?;
        }
        writeln!(writer)?;
    }

    Ok(())
}

/// Render schema graph in dense, token-optimized compact format (for AI agents)
pub fn render_compact<W: Write>(data: &SchemaGraphData, writer: &mut W) -> Result<()> {
    writeln!(
        writer,
        "# database: {} ({} tables, {} join paths)",
        data.database, data.table_count, data.relationship_count
    )?;

    for t in &data.tables {
        let rows_str = match t.row_count_est {
            Some(n) if n >= 1_000_000 => format!("{:.1}M", n as f64 / 1_000_000.0),
            Some(n) if n >= 1_000 => format!("{:.1}K", n as f64 / 1_000.0),
            Some(n) => n.to_string(),
            None => "?".to_string(),
        };

        let mut header = format!("{} [{} rows]", t.name, rows_str);
        if !t.primary_index.is_empty() {
            header.push_str(&format!(" PI({})", t.primary_index.join(",")));
        }
        if !t.partition_keys.is_empty() {
            header.push_str(&format!(" PPI({})", t.partition_keys.join(",")));
        }
        header.push_str(": ");

        let col_strs: Vec<String> = t
            .columns
            .iter()
            .map(|c| {
                let compact_type = abbreviate_type(&c.col_type);
                let pi_marker = if c.is_primary_index { "*" } else { "" };
                if let Some(ref fk) = c.fk_candidate {
                    format!("{}{}:{}->{}", c.name, pi_marker, compact_type, fk)
                } else {
                    format!("{}{}:{}", c.name, pi_marker, compact_type)
                }
            })
            .collect();

        writeln!(writer, "{}{}", header, col_strs.join(", "))?;
    }

    for j in &data.join_paths {
        writeln!(
            writer,
            "JOIN: {}.{} -> {}.{} [{}, cost={}]",
            j.from_table, j.from_column, j.to_table, j.to_column, j.join_type, j.cost
        )?;
    }

    Ok(())
}

/// Render schema graph as CSV
pub fn render_csv<W: Write>(data: &SchemaGraphData, writer: &mut W) -> Result<()> {
    writeln!(
        writer,
        "section,database,table_name,kind,row_count_est,primary_index,from_col,to_table,to_col,join_type,cost"
    )?;

    for t in &data.tables {
        let rows = t
            .row_count_est
            .map(|n| n.to_string())
            .unwrap_or_default();
        let pi = t.primary_index.join(";");
        writeln!(
            writer,
            "table,{},{},{},{},{},,,,,",
            csv_escape(&data.database),
            csv_escape(&t.name),
            csv_escape(&t.kind),
            rows,
            csv_escape(&pi),
        )?;
    }

    for j in &data.join_paths {
        writeln!(
            writer,
            "join,{},{},,,,,{},{},{},{},{}",
            csv_escape(&data.database),
            csv_escape(&j.from_table),
            csv_escape(&j.from_column),
            csv_escape(&j.to_table),
            csv_escape(&j.to_column),
            csv_escape(&j.join_type),
            csv_escape(&j.cost),
        )?;
    }

    Ok(())
}

// =============================================================================
// Helper Functions
// =============================================================================

fn sanitize_mermaid_ident(s: &str) -> String {
    s.replace(['-', ' ', '.', '$'], "_")
}

fn abbreviate_type(full_type: &str) -> String {
    let t = full_type.to_ascii_lowercase();
    if t.starts_with("varchar") {
        "varchar".to_string()
    } else if t.starts_with("char") {
        "char".to_string()
    } else if t.starts_with("integer") || t.starts_with("int") {
        "int".to_string()
    } else if t.starts_with("smallint") {
        "smallint".to_string()
    } else if t.starts_with("bigint") {
        "bigint".to_string()
    } else if t.starts_with("byteint") {
        "byteint".to_string()
    } else if t.starts_with("decimal") || t.starts_with("number") {
        "dec".to_string()
    } else if t.starts_with("date") {
        "date".to_string()
    } else if t.starts_with("timestamp") {
        "ts".to_string()
    } else {
        t
    }
}

// =============================================================================
// Unit Tests (TC111-U01 through U08)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_graph() -> SchemaGraphData {
        let customers = SchemaTable {
            name: "customers".to_string(),
            kind: "Table".to_string(),
            table_kind: "T".to_string(),
            row_count_est: Some(500000),
            size_bytes: Some(65536000),
            primary_index: vec!["customer_id".to_string()],
            partition_keys: Vec::new(),
            columns: vec![
                SchemaColumn {
                    name: "customer_id".to_string(),
                    col_type: "INTEGER".to_string(),
                    nullable: false,
                    is_primary_index: true,
                    is_partition_key: false,
                    fk_candidate: None,
                    comment: String::new(),
                },
                SchemaColumn {
                    name: "customer_name".to_string(),
                    col_type: "VARCHAR(50)".to_string(),
                    nullable: false,
                    is_primary_index: false,
                    is_partition_key: false,
                    fk_candidate: None,
                    comment: String::new(),
                },
                SchemaColumn {
                    name: "region_id".to_string(),
                    col_type: "SMALLINT".to_string(),
                    nullable: true,
                    is_primary_index: false,
                    is_partition_key: false,
                    fk_candidate: None,
                    comment: String::new(),
                },
            ],
            indexes: vec![SchemaIndex {
                name: None,
                index_type: "Primary Index (UPI)".to_string(),
                short_label: "UPI".to_string(),
                is_primary: true,
                columns: vec!["customer_id".to_string()],
            }],
        };

        let orders = SchemaTable {
            name: "orders".to_string(),
            kind: "Table".to_string(),
            table_kind: "T".to_string(),
            row_count_est: Some(15000000),
            size_bytes: Some(524288000),
            primary_index: vec!["order_id".to_string()],
            partition_keys: vec!["order_date".to_string()],
            columns: vec![
                SchemaColumn {
                    name: "order_id".to_string(),
                    col_type: "INTEGER".to_string(),
                    nullable: false,
                    is_primary_index: true,
                    is_partition_key: false,
                    fk_candidate: None,
                    comment: String::new(),
                },
                SchemaColumn {
                    name: "customer_id".to_string(),
                    col_type: "INTEGER".to_string(),
                    nullable: false,
                    is_primary_index: false,
                    is_partition_key: false,
                    fk_candidate: None,
                    comment: String::new(),
                },
                SchemaColumn {
                    name: "order_date".to_string(),
                    col_type: "DATE".to_string(),
                    nullable: false,
                    is_primary_index: false,
                    is_partition_key: true,
                    fk_candidate: None,
                    comment: String::new(),
                },
                SchemaColumn {
                    name: "total_amount".to_string(),
                    col_type: "DECIMAL(18,2)".to_string(),
                    nullable: false,
                    is_primary_index: false,
                    is_partition_key: false,
                    fk_candidate: None,
                    comment: String::new(),
                },
            ],
            indexes: vec![SchemaIndex {
                name: None,
                index_type: "Primary Index (UPI)".to_string(),
                short_label: "UPI".to_string(),
                is_primary: true,
                columns: vec!["order_id".to_string()],
            }],
        };

        SchemaGraphData {
            database: "financials".to_string(),
            table_count: 2,
            relationship_count: 0,
            tables: vec![customers, orders],
            join_paths: Vec::new(),
        }
    }

    #[test]
    fn test_tc111_u01_json_envelope_serialization() {
        let mut data = create_sample_graph();
        infer_relationships_and_joins(&mut data, 2);

        let mut buf = Vec::new();
        render_json(&data, &mut buf).unwrap();
        let json_str = String::from_utf8(buf).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["ok"], true);
        assert_eq!(parsed["row_count"], 2);
        assert_eq!(parsed["data"]["database"], "financials");
        assert_eq!(parsed["data"]["tables"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["data"]["join_paths"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_tc111_u02_foreign_key_inference_identical_names() {
        let mut data = create_sample_graph();
        infer_relationships_and_joins(&mut data, 2);

        assert_eq!(data.relationship_count, 1);
        let join = &data.join_paths[0];
        assert_eq!(join.from_table, "orders");
        assert_eq!(join.from_column, "customer_id");
        assert_eq!(join.to_table, "customers");
        assert_eq!(join.to_column, "customer_id");
    }

    #[test]
    fn test_tc111_u03_foreign_key_inference_entity_suffix() {
        let mut data = create_sample_graph();
        // Rename customers.customer_id to customers.id
        data.tables[0].columns[0].name = "id".to_string();
        data.tables[0].primary_index = vec!["id".to_string()];

        infer_relationships_and_joins(&mut data, 2);

        assert_eq!(data.relationship_count, 1);
        let join = &data.join_paths[0];
        assert_eq!(join.from_table, "orders");
        assert_eq!(join.from_column, "customer_id");
        assert_eq!(join.to_table, "customers");
        assert_eq!(join.to_column, "id");
    }

    #[test]
    fn test_tc111_u04_teradata_pi_join_scoring() {
        let mut data = create_sample_graph();
        infer_relationships_and_joins(&mut data, 2);

        let join = &data.join_paths[0];
        assert_eq!(join.join_type, "RECOMMENDED_PI_JOIN");
        assert_eq!(join.cost, "LOW");
        assert!(join.reason.contains("customers.customer_id is Primary Index"));

        // Case: Both PI -> COLOCATED_PI_JOIN
        data.tables[1].primary_index = vec!["customer_id".to_string()];
        infer_relationships_and_joins(&mut data, 2);
        let join_colocated = &data.join_paths[0];
        assert_eq!(join_colocated.join_type, "COLOCATED_PI_JOIN");
        assert_eq!(join_colocated.cost, "MINIMAL");
    }

    #[test]
    fn test_tc111_u05_table_rendering() {
        let mut data = create_sample_graph();
        infer_relationships_and_joins(&mut data, 2);

        let mut buf = Vec::new();
        render_table(&data, &mut buf, false).unwrap();
        let table_str = String::from_utf8(buf).unwrap();

        assert!(table_str.contains("Database Schema: financials"));
        assert!(table_str.contains("customers"));
        assert!(table_str.contains("orders"));
        assert!(table_str.contains("RECOMMENDED_PI_JOIN"));
    }

    #[test]
    fn test_tc111_u06_markdown_mermaid_rendering() {
        let mut data = create_sample_graph();
        infer_relationships_and_joins(&mut data, 2);

        let mut buf = Vec::new();
        render_markdown(&data, &mut buf).unwrap();
        let md_str = String::from_utf8(buf).unwrap();

        assert!(md_str.contains("# Schema Graph: `financials`"));
        assert!(md_str.contains("```mermaid"));
        assert!(md_str.contains("erDiagram"));
        assert!(md_str.contains("orders }|..|| customers : \"customer_id\""));
    }

    #[test]
    fn test_tc111_u07_compact_token_optimized_rendering() {
        let mut data = create_sample_graph();
        infer_relationships_and_joins(&mut data, 2);

        let mut buf = Vec::new();
        render_compact(&data, &mut buf).unwrap();
        let compact_str = String::from_utf8(buf).unwrap();

        assert!(compact_str.contains("# database: financials (2 tables, 1 join paths)"));
        assert!(compact_str.contains("customers [500.0K rows] PI(customer_id):"));
        assert!(compact_str.contains("customer_id*:int"));
        assert!(compact_str.contains("JOIN: orders.customer_id -> customers.customer_id"));
    }

    #[test]
    fn test_tc111_u08_csv_rendering() {
        let mut data = create_sample_graph();
        infer_relationships_and_joins(&mut data, 2);

        let mut buf = Vec::new();
        render_csv(&data, &mut buf).unwrap();
        let csv_str = String::from_utf8(buf).unwrap();

        assert!(csv_str.starts_with("section,database,table_name"));
        assert!(csv_str.contains("table,financials,customers"));
        assert!(csv_str.contains("join,financials,orders"));
    }
}
