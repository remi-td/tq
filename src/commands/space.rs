//! Space analysis commands (`tq space`, `tq dbspace`)
//!
//! Gives administrators a one-command view of permanent, spool and temporary
//! space usage for a database or an individual object, replacing hand-written
//! `DBC.DiskSpaceV` / `DBC.TableSizeV` queries.
//!
//! | Invocation | Result |
//! |------------|--------|
//! | `tq space <database>` | Database header row followed by one row per contained object |
//! | `tq space <database>.<object>` | Exactly one object row |
//! | `tq dbspace <database>` | Database-level perm/spool/temp metrics only |
//! | `tq dbspace <database>.<object>` | Usage error — `dbspace` operates on databases only |
//!
//! Both DBC views are per-AMP (one row per `Vproc`), so every metric is
//! aggregated across AMPs and skew is derived from the distribution across
//! those rows using the formula from issue #54:
//! `100 - (AVG(x) / NULLIFZERO(MAX(x)) * 100)`.

use super::monitoring_utils::{
    escape_csv, extract_f64_lenient, extract_i64_lenient, extract_trimmed_string,
};
use super::severity::{MonitoringContext, Severity};
use crate::cli::{DbspaceArgs, OutputFormat, SpaceArgs};
use crate::commands::format_helpers::{format_size, markdown_escape_pipe};
use crate::db::{DatabaseClient, Value};
use crate::error::{Result, TqError};
use crate::sql::identifiers::escape_sql_string;
use std::io::Write;

/// Rendered placeholder for a metric that was computed but is NULL
const NULL_DISPLAY: &str = "[--]";
/// Rendered placeholder for a metric that does not apply to a row
const NA_DISPLAY: &str = "-";
/// Rendered placeholder for `PermUsed%` when `MaxPerm = 0` (no perm limit)
const UNLIMITED_DISPLAY: &str = "[unlimited]";
/// Decimal places used by `format_size` for byte humanization
const SIZE_PRECISION: usize = 1;

// =============================================================================
// Target parsing
// =============================================================================

/// A validated `space` / `dbspace` target
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceTarget {
    /// `<database>`
    Database(String),
    /// `<database>.<object>`
    Object {
        /// Owning database or user
        database: String,
        /// Object (table, join index, stored procedure, ...) name
        object: String,
    },
}

/// Usage block appended to invalid-reference errors for `tq space`
const SPACE_USAGE: &str = "Usage: tq space <database>[.<object>]\n\n\
     Examples:\n  \
     tq space demo_user\n  \
     tq space demo_user.orders";

/// Split a target on `.`, ignoring dots inside double-quoted identifier parts.
///
/// Returns the raw parts with their surrounding quotes still attached.
fn split_target_parts(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in input.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            '.' if !in_quotes => {
                parts.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }
    parts.push(current);
    parts
}

/// Strip one layer of surrounding double quotes, un-doubling embedded ones
fn unquote_identifier(part: &str) -> String {
    let trimmed = part.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].replace("\"\"", "\"")
    } else {
        trimmed.to_string()
    }
}

/// Parse a positional target into [`SpaceTarget`]
///
/// No dot yields a database target; exactly one dot yields an object target.
/// More than one dot, or an empty component, is a usage error (exit code 2)
/// per REQ-SPACE-001.
pub fn parse_target(input: &str) -> Result<SpaceTarget> {
    let invalid = || TqError::InvalidObjectReference {
        reference: input.to_string(),
        expected: "<database> or <database>.<object>".to_string(),
        usage: SPACE_USAGE.to_string(),
    };

    let parts = split_target_parts(input);
    if parts.len() > 2 {
        return Err(invalid());
    }

    let cleaned: Vec<String> = parts.iter().map(|p| unquote_identifier(p)).collect();
    if cleaned.iter().any(|p| p.is_empty()) {
        return Err(invalid());
    }

    match cleaned.len() {
        1 => Ok(SpaceTarget::Database(cleaned[0].clone())),
        2 => Ok(SpaceTarget::Object {
            database: cleaned[0].clone(),
            object: cleaned[1].clone(),
        }),
        _ => Err(invalid()),
    }
}

/// Parse a `dbspace` target, rejecting the qualified form (REQ-DBSPACE-001)
pub fn parse_database_target(input: &str) -> Result<String> {
    match parse_target(input)? {
        SpaceTarget::Database(db) => Ok(db),
        SpaceTarget::Object { database, object } => Err(TqError::InvalidObjectReference {
            reference: input.to_string(),
            expected: "<database> (dbspace operates on databases only)".to_string(),
            usage: format!(
                "Hint: use 'tq space {database}.{object}' for object-level space,\n      \
                 or 'tq dbspace {database}' for the database.\n\n\
                 Usage: tq dbspace <database>"
            ),
        }),
    }
}

// =============================================================================
// SQL
// =============================================================================

/// Build the database-level aggregation over `DBC.DiskSpaceV`
///
/// `PermSkew` / `SpoolSkew` / `TempSkew` on the view are the *configured
/// permissible* skew limit, not a measurement, so they are deliberately unused;
/// skew is computed from the AVG/MAX distribution across AMPs instead.
fn build_database_sql(database: &str) -> String {
    format!(
        r#"SELECT
    DatabaseName,
    SUM(MaxPerm) AS MaxPerm,
    SUM(CurrentPerm) AS CurrentPerm,
    SUM(PeakPerm) AS PeakPerm,
    (100 - (AVG(CurrentPerm) / NULLIFZERO(MAX(CurrentPerm)) * 100)) AS PermSkewPct,
    SUM(MaxSpool) AS MaxSpool,
    SUM(CurrentSpool) AS CurrentSpool,
    SUM(PeakSpool) AS PeakSpool,
    (100 - (AVG(CurrentSpool) / NULLIFZERO(MAX(CurrentSpool)) * 100)) AS SpoolSkewPct,
    SUM(MaxTemp) AS MaxTemp,
    SUM(CurrentTemp) AS CurrentTemp,
    SUM(PeakTemp) AS PeakTemp,
    (100 - (AVG(CurrentTemp) / NULLIFZERO(MAX(CurrentTemp)) * 100)) AS TempSkewPct
FROM DBC.DiskSpaceV
WHERE UPPER(DatabaseName) = UPPER('{}')
GROUP BY 1"#,
        escape_sql_string(database)
    )
}

/// Build the object-level aggregation over `DBC.TableSizeV`
///
/// Note the capital `B` in `DataBaseName` — that is this view's own spelling.
/// `TableSizeV` exposes no `MaxPerm`: perm allocation is a database-level
/// property, so object rows carry current/peak/skew only.
///
/// Rows are ordered alphabetically by object name (REQ-SPACE-003).
fn build_object_sql(database: &str, object: Option<&str>) -> String {
    let object_predicate = match object {
        Some(name) => format!(
            "\n  AND UPPER(TableName) = UPPER('{}')",
            escape_sql_string(name)
        ),
        None => String::new(),
    };

    format!(
        r#"SELECT
    DataBaseName,
    TableName,
    SUM(CurrentPerm) AS CurrentPerm,
    SUM(PeakPerm) AS PeakPerm,
    (100 - (AVG(CurrentPerm) / NULLIFZERO(MAX(CurrentPerm)) * 100)) AS PermSkewPct
FROM DBC.TableSizeV
WHERE UPPER(DataBaseName) = UPPER('{}'){}
GROUP BY 1, 2
ORDER BY 2"#,
        escape_sql_string(database),
        object_predicate
    )
}

/// Probe `DBC.DatabasesV` to distinguish "holds no space" from "unknown name".
///
/// `DBKind` is `'D'` for a database and `'U'` for a user; both own space, so
/// both are accepted.
fn build_database_probe_sql(database: &str) -> String {
    format!(
        "SELECT DatabaseName, DBKind FROM DBC.DatabasesV \
         WHERE UPPER(DatabaseName) = UPPER('{}')",
        escape_sql_string(database)
    )
}

/// Probe `DBC.TablesV` for a specific object under a specific database
fn build_object_probe_sql(database: &str, object: &str) -> String {
    format!(
        "SELECT DataBaseName, TableName FROM DBC.TablesV \
         WHERE UPPER(DataBaseName) = UPPER('{}') AND UPPER(TableName) = UPPER('{}')",
        escape_sql_string(database),
        escape_sql_string(object)
    )
}

/// Probe `DBC.TablesV` for an object of this name in any database
///
/// Used only to turn `tq dbspace <table>` into a "not a database" error that
/// names the right command, instead of a bare not-found (REQ-DBSPACE-003).
fn build_any_object_probe_sql(name: &str) -> String {
    format!(
        "SELECT TOP 1 DataBaseName, TableName FROM DBC.TablesV \
         WHERE UPPER(TableName) = UPPER('{}')",
        escape_sql_string(name)
    )
}

// =============================================================================
// Types
// =============================================================================

/// One space class (perm, spool or temp) aggregated across AMPs
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceMetrics {
    /// `SUM(Max<class>)` — the allocation
    pub max: i64,
    /// `SUM(Current<class>)`
    pub current: i64,
    /// `SUM(Peak<class>)`
    pub peak: i64,
    /// Skew percentage, `None` when the class holds nothing on every AMP
    pub skew_pct: Option<f64>,
}

impl SpaceMetrics {
    /// Percentage of the allocation currently consumed.
    ///
    /// `None` when `max == 0`, which in Teradata means "no limit" rather than
    /// "zero capacity" — see [`SpaceMetrics::is_unlimited`].
    pub fn pct_used(&self) -> Option<f64> {
        if self.max <= 0 {
            None
        } else {
            Some(self.current as f64 / self.max as f64 * 100.0)
        }
    }

    /// Whether the allocation is unlimited (`max == 0`)
    pub fn is_unlimited(&self) -> bool {
        self.max == 0
    }
}

/// A database's perm, spool and temp footprint
#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseSpace {
    /// Database or user name as stored in the catalog
    pub database: String,
    /// Permanent space
    pub perm: SpaceMetrics,
    /// Spool space
    pub spool: SpaceMetrics,
    /// Temporary space
    pub temp: SpaceMetrics,
}

impl DatabaseSpace {
    /// Build from a `build_database_sql` result row
    ///
    /// Returns `None` when the row is too short or carries no database name.
    /// Absent numerics default to `0`; absent skew stays `None` rather than
    /// being coerced to `0.0`, which would misreport "no data" as "perfectly
    /// distributed".
    pub fn from_row(row: &[Value]) -> Option<Self> {
        if row.len() < 13 {
            return None;
        }
        if matches!(row[0], Value::Null) {
            return None;
        }
        let database = extract_trimmed_string(&row[0], "");
        if database.is_empty() {
            return None;
        }

        let num = |idx: usize| extract_i64_lenient(&row[idx]).unwrap_or(0);

        Some(Self {
            database,
            perm: SpaceMetrics {
                max: num(1),
                current: num(2),
                peak: num(3),
                skew_pct: extract_f64_lenient(&row[4]),
            },
            spool: SpaceMetrics {
                max: num(5),
                current: num(6),
                peak: num(7),
                skew_pct: extract_f64_lenient(&row[8]),
            },
            temp: SpaceMetrics {
                max: num(9),
                current: num(10),
                peak: num(11),
                skew_pct: extract_f64_lenient(&row[12]),
            },
        })
    }

    /// A zero-usage record for a database that exists but holds no space
    pub fn empty(database: &str) -> Self {
        let zero = || SpaceMetrics {
            max: 0,
            current: 0,
            peak: 0,
            skew_pct: None,
        };
        Self {
            database: database.to_string(),
            perm: zero(),
            spool: zero(),
            temp: zero(),
        }
    }
}

/// One object's permanent-space footprint
///
/// `MaxPerm` is absent by construction: `DBC.TableSizeV` does not expose it.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectSpace {
    /// Owning database
    pub database: String,
    /// Object name
    pub object: String,
    /// `SUM(CurrentPerm)`
    pub current_perm: i64,
    /// `SUM(PeakPerm)`
    pub peak_perm: i64,
    /// Perm skew percentage, `None` when the object holds nothing
    pub perm_skew_pct: Option<f64>,
}

impl ObjectSpace {
    /// Build from a `build_object_sql` result row
    pub fn from_row(row: &[Value]) -> Option<Self> {
        if row.len() < 5 {
            return None;
        }
        let database = extract_trimmed_string(&row[0], "");
        let object = extract_trimmed_string(&row[1], "");
        if database.is_empty() || object.is_empty() {
            return None;
        }

        Some(Self {
            database,
            object,
            current_perm: extract_i64_lenient(&row[2]).unwrap_or(0),
            peak_perm: extract_i64_lenient(&row[3]).unwrap_or(0),
            perm_skew_pct: extract_f64_lenient(&row[4]),
        })
    }
}

/// What a `space` / `dbspace` invocation produced
///
/// Modelling the three shapes as one enum means each renderer is a single
/// `match` and "which columns exist" is answered by the type system rather than
/// by conditionally meaningful `Option` fields.
#[derive(Debug, Clone, PartialEq)]
pub enum SpaceReport {
    /// `tq space <db>`: header row plus one row per contained object
    Database {
        /// The database's own space
        header: DatabaseSpace,
        /// Objects directly under the database, alphabetically ordered
        objects: Vec<ObjectSpace>,
    },
    /// `tq space <db>.<obj>`
    Object(ObjectSpace),
    /// `tq dbspace <db>`
    DatabaseOnly(DatabaseSpace),
}

// =============================================================================
// Query orchestration
// =============================================================================

/// Fetch the database-level record, distinguishing "no space" from "no such name"
///
/// The catalog probe runs only when the space query returned nothing, so the
/// common case still costs a single round trip.
fn query_database_space(client: &DatabaseClient, database: &str) -> Result<DatabaseSpace> {
    let result = client.execute(&build_database_sql(database))?;

    if let Some(space) = result.rows.iter().find_map(|row| DatabaseSpace::from_row(row)) {
        return Ok(space);
    }

    let probe = client.execute(&build_database_probe_sql(database))?;
    if probe.rows.is_empty() {
        return Err(TqError::ObjectNotFound {
            object_type: "Database".to_string(),
            name: database.to_string(),
            hint: Some(
                "Check the database name spelling, and that you have SELECT \
                 privilege on DBC.DiskSpaceV."
                    .to_string(),
            ),
        });
    }

    // The database exists but holds no space: a valid answer, not a failure.
    Ok(DatabaseSpace::empty(database))
}

/// Fetch the objects directly under a database
fn query_objects(client: &DatabaseClient, database: &str) -> Result<Vec<ObjectSpace>> {
    let result = client.execute(&build_object_sql(database, None))?;
    Ok(result
        .rows
        .iter()
        .filter_map(|row| ObjectSpace::from_row(row))
        .collect())
}

/// Fetch a single object, probing the catalog when the space query is empty
fn query_single_object(
    client: &DatabaseClient,
    database: &str,
    object: &str,
) -> Result<ObjectSpace> {
    let result = client.execute(&build_object_sql(database, Some(object)))?;

    if let Some(found) = result.rows.iter().find_map(|row| ObjectSpace::from_row(row)) {
        return Ok(found);
    }

    let probe = client.execute(&build_object_probe_sql(database, object))?;
    if probe.rows.is_empty() {
        return Err(TqError::ObjectNotFound {
            object_type: "Object".to_string(),
            name: format!("{database}.{object}"),
            hint: Some(
                "Check the object name spelling and the database qualification."
                    .to_string(),
            ),
        });
    }

    // The object exists but occupies no perm space (e.g. an empty table).
    Ok(ObjectSpace {
        database: database.to_string(),
        object: object.to_string(),
        current_perm: 0,
        peak_perm: 0,
        perm_skew_pct: None,
    })
}

/// Build the report for `tq space <target>`
pub fn build_space_report(client: &DatabaseClient, target: &SpaceTarget) -> Result<SpaceReport> {
    match target {
        SpaceTarget::Database(database) => {
            let header = query_database_space(client, database)?;
            let objects = query_objects(client, &header.database)?;
            Ok(SpaceReport::Database { header, objects })
        }
        SpaceTarget::Object { database, object } => Ok(SpaceReport::Object(
            query_single_object(client, database, object)?,
        )),
    }
}

/// Build the report for `tq dbspace <database>`
pub fn build_dbspace_report(client: &DatabaseClient, database: &str) -> Result<SpaceReport> {
    match query_database_space(client, database) {
        Ok(space) => Ok(SpaceReport::DatabaseOnly(space)),
        Err(TqError::ObjectNotFound { .. }) => {
            // Not a database — is it an object? Name the right command if so.
            let probe = client.execute(&build_any_object_probe_sql(database))?;
            if let Some(row) = probe.rows.first() {
                let owner = extract_trimmed_string(&row[0], "");
                return Err(TqError::WrongObjectKind {
                    name: database.to_string(),
                    expected_kind: "database".to_string(),
                    hint: Some(format!(
                        "'{database}' is an object in database '{owner}'.\n\n\
                         Hint: use 'tq space {owner}.{database}' for object-level space."
                    )),
                });
            }
            Err(TqError::ObjectNotFound {
                object_type: "Database".to_string(),
                name: database.to_string(),
                hint: Some(
                    "Check the database name spelling, and that you have SELECT \
                     privilege on DBC.DiskSpaceV."
                        .to_string(),
                ),
            })
        }
        Err(e) => Err(e),
    }
}

// =============================================================================
// Entry points
// =============================================================================

/// Execute `tq space` in batch mode
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &SpaceArgs,
    writer: &mut W,
    ctx: &MonitoringContext,
) -> Result<()> {
    let target = parse_target(&args.target)?;
    let report = build_space_report(client, &target)?;
    render(&report, args.format, writer, ctx)
}

/// Execute `tq dbspace` in batch mode
pub fn execute_dbspace<W: Write>(
    client: &DatabaseClient,
    args: &DbspaceArgs,
    writer: &mut W,
    ctx: &MonitoringContext,
) -> Result<()> {
    let database = parse_database_target(&args.database)?;
    let report = build_dbspace_report(client, &database)?;
    render(&report, args.format, writer, ctx)
}

/// Render a report in the requested format
pub fn render<W: Write>(
    report: &SpaceReport,
    format: OutputFormat,
    writer: &mut W,
    ctx: &MonitoringContext,
) -> Result<()> {
    match format {
        OutputFormat::Table => display_table(report, writer, ctx),
        OutputFormat::Csv => display_csv(report, writer),
        OutputFormat::Json => display_json(report, writer),
        OutputFormat::Markdown | OutputFormat::Md => display_markdown(report, writer, ctx),
    }
}

/// Execute `/space` or `/dbspace` inside the REPL
pub fn execute_for_repl<W: Write>(
    client: &DatabaseClient,
    target: &str,
    database_only: bool,
    writer: &mut W,
    ctx: &MonitoringContext,
) -> Result<()> {
    writeln!(writer)?;

    let report = if database_only {
        parse_database_target(target).and_then(|db| build_dbspace_report(client, &db))
    } else {
        parse_target(target).and_then(|t| build_space_report(client, &t))
    };

    match report {
        Ok(report) => display_table(&report, writer, ctx)?,
        Err(e) => writeln!(writer, "{}", e.user_message())?,
    }

    writeln!(writer)?;
    Ok(())
}

// =============================================================================
// Cell formatting helpers
// =============================================================================

/// Humanized byte count for table/markdown output (REQ-SPACE-HUMAN-001)
fn size_cell(bytes: i64) -> String {
    format_size(bytes, SIZE_PRECISION)
}

/// A percentage for table/markdown output, `[--]` when NULL (REQ-SPACE-NULL-001)
fn pct_cell(pct: Option<f64>) -> String {
    match pct {
        Some(v) => format!("{v:.1}"),
        None => NULL_DISPLAY.to_string(),
    }
}

/// `PermUsed%` for table/markdown, distinguishing "unlimited" from NULL
fn used_cell(metrics: &SpaceMetrics) -> String {
    if metrics.is_unlimited() {
        UNLIMITED_DISPLAY.to_string()
    } else {
        pct_cell(metrics.pct_used())
    }
}

/// A percentage for csv output: empty field when NULL
fn pct_csv(pct: Option<f64>) -> String {
    pct.map(|v| format!("{v:.1}")).unwrap_or_default()
}

/// Classify a skew percentage, `None` when there is no measurement
fn skew_severity(ctx: &MonitoringContext, pct: Option<f64>) -> Option<Severity> {
    pct.map(|v| ctx.thresholds.skew(v))
}

/// Classify a space-consumption percentage, `None` when unlimited or NULL
fn space_severity(ctx: &MonitoringContext, metrics: &SpaceMetrics) -> Option<Severity> {
    metrics.pct_used().map(|v| ctx.thresholds.space(v))
}

// =============================================================================
// Table rendering
// =============================================================================

/// Column headers for the database form of `tq space`
const DATABASE_FORM_HEADERS: [&str; 15] = [
    "Kind",
    "Object",
    "CurrentPerm",
    "PeakPerm",
    "PermSkew%",
    "MaxPerm",
    "PermUsed%",
    "SpoolCurrent",
    "SpoolMax",
    "SpoolPeak",
    "SpoolSkew%",
    "TempCurrent",
    "TempMax",
    "TempPeak",
    "TempSkew%",
];

/// Column headers for `tq dbspace` (the database row, minus `Kind`)
const DBSPACE_HEADERS: [&str; 14] = [
    "Database",
    "CurrentPerm",
    "PeakPerm",
    "PermSkew%",
    "MaxPerm",
    "PermUsed%",
    "SpoolCurrent",
    "SpoolMax",
    "SpoolPeak",
    "SpoolSkew%",
    "TempCurrent",
    "TempMax",
    "TempPeak",
    "TempSkew%",
];

/// Column headers for the single-object form of `tq space`
const OBJECT_FORM_HEADERS: [&str; 5] = [
    "Database",
    "Object",
    "CurrentPerm",
    "PeakPerm",
    "PermSkew%",
];

/// The database row's metric cells, already humanized and colored
///
/// `include_kind` prepends the `Kind`/`Object` pair used by `tq space`; when
/// false the database name leads instead, as `tq dbspace` requires.
fn database_cells(db: &DatabaseSpace, ctx: &MonitoringContext, include_kind: bool) -> Vec<String> {
    let mut cells = Vec::with_capacity(15);
    if include_kind {
        cells.push("DATABASE".to_string());
    }
    cells.push(db.database.clone());
    cells.push(size_cell(db.perm.current));
    cells.push(size_cell(db.perm.peak));
    cells.push(ctx.styler.paint_optional(
        skew_severity(ctx, db.perm.skew_pct),
        &pct_cell(db.perm.skew_pct),
    ));
    cells.push(size_cell(db.perm.max));
    cells.push(
        ctx.styler
            .paint_optional(space_severity(ctx, &db.perm), &used_cell(&db.perm)),
    );
    cells.push(size_cell(db.spool.current));
    cells.push(size_cell(db.spool.max));
    cells.push(size_cell(db.spool.peak));
    cells.push(ctx.styler.paint_optional(
        skew_severity(ctx, db.spool.skew_pct),
        &pct_cell(db.spool.skew_pct),
    ));
    cells.push(size_cell(db.temp.current));
    cells.push(size_cell(db.temp.max));
    cells.push(size_cell(db.temp.peak));
    cells.push(ctx.styler.paint_optional(
        skew_severity(ctx, db.temp.skew_pct),
        &pct_cell(db.temp.skew_pct),
    ));
    cells
}

/// An object row inside the database form: database-only columns read `-`
fn object_cells_in_database_form(obj: &ObjectSpace, ctx: &MonitoringContext) -> Vec<String> {
    let mut cells = vec![
        "TABLE".to_string(),
        obj.object.clone(),
        size_cell(obj.current_perm),
        size_cell(obj.peak_perm),
        ctx.styler.paint_optional(
            skew_severity(ctx, obj.perm_skew_pct),
            &pct_cell(obj.perm_skew_pct),
        ),
    ];
    cells.extend(std::iter::repeat_n(NA_DISPLAY.to_string(), 10));
    cells
}

/// Build a `comfy_table` with the given headers, numeric columns right-aligned
fn build_table(headers: &[&str], rows: Vec<Vec<String>>, name_columns: usize) -> comfy_table::Table {
    use comfy_table::{presets, Cell, CellAlignment, ContentArrangement, Table};

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(headers.to_vec());

    for row in rows {
        let cells: Vec<Cell> = row
            .into_iter()
            .enumerate()
            .map(|(i, value)| {
                let cell = Cell::new(value);
                if i < name_columns {
                    cell
                } else {
                    cell.set_alignment(CellAlignment::Right)
                }
            })
            .collect();
        table.add_row(cells);
    }
    table
}

/// Render a report as a human-readable table
fn display_table<W: Write>(
    report: &SpaceReport,
    writer: &mut W,
    ctx: &MonitoringContext,
) -> Result<()> {
    match report {
        SpaceReport::Database { header, objects } => {
            writeln!(writer, "Space Analysis — Database: {}", header.database)?;
            writeln!(writer)?;

            let mut rows = vec![database_cells(header, ctx, true)];
            rows.extend(objects.iter().map(|o| object_cells_in_database_form(o, ctx)));

            let table = build_table(&DATABASE_FORM_HEADERS, rows, 2);
            writeln!(writer, "{table}")?;
            writeln!(writer)?;

            let total: i64 = objects.iter().map(|o| o.current_perm).sum();
            writeln!(
                writer,
                "{} rows (1 database, {} objects) | Total object CurrentPerm: {}",
                objects.len() + 1,
                objects.len(),
                size_cell(total)
            )?;
        }
        SpaceReport::DatabaseOnly(db) => {
            writeln!(writer, "Space Analysis — Database: {}", db.database)?;
            writeln!(writer)?;
            let table = build_table(&DBSPACE_HEADERS, vec![database_cells(db, ctx, false)], 1);
            writeln!(writer, "{table}")?;
        }
        SpaceReport::Object(obj) => {
            writeln!(
                writer,
                "Space Analysis — Object: {}.{}",
                obj.database, obj.object
            )?;
            writeln!(writer)?;
            let row = vec![
                obj.database.clone(),
                obj.object.clone(),
                size_cell(obj.current_perm),
                size_cell(obj.peak_perm),
                ctx.styler.paint_optional(
                    skew_severity(ctx, obj.perm_skew_pct),
                    &pct_cell(obj.perm_skew_pct),
                ),
            ];
            let table = build_table(&OBJECT_FORM_HEADERS, vec![row], 2);
            writeln!(writer, "{table}")?;
        }
    }
    Ok(())
}

// =============================================================================
// Markdown rendering
// =============================================================================

/// Emit a markdown row from pre-rendered cells
fn markdown_row<W: Write>(writer: &mut W, cells: &[String]) -> Result<()> {
    let escaped: Vec<String> = cells.iter().map(|c| markdown_escape_pipe(c)).collect();
    writeln!(writer, "| {} |", escaped.join(" | "))?;
    Ok(())
}

/// Emit a markdown header plus alignment row
fn markdown_header<W: Write>(writer: &mut W, headers: &[&str], name_columns: usize) -> Result<()> {
    let head: Vec<String> = headers.iter().map(|h| markdown_escape_pipe(h)).collect();
    writeln!(writer, "| {} |", head.join(" | "))?;
    let align: Vec<&str> = (0..headers.len())
        .map(|i| if i < name_columns { ":---" } else { "---:" })
        .collect();
    writeln!(writer, "| {} |", align.join(" | "))?;
    Ok(())
}

/// Render a report as a GitHub-flavored markdown table
///
/// Severity coloring applies here as it does to `table` (REQ-COLOR-007); the
/// styler is disabled whenever color mode resolves to off, including for
/// `--output` file writes.
fn display_markdown<W: Write>(
    report: &SpaceReport,
    writer: &mut W,
    ctx: &MonitoringContext,
) -> Result<()> {
    match report {
        SpaceReport::Database { header, objects } => {
            markdown_header(writer, &DATABASE_FORM_HEADERS, 2)?;
            markdown_row(writer, &database_cells(header, ctx, true))?;
            for obj in objects {
                markdown_row(writer, &object_cells_in_database_form(obj, ctx))?;
            }
        }
        SpaceReport::DatabaseOnly(db) => {
            markdown_header(writer, &DBSPACE_HEADERS, 1)?;
            markdown_row(writer, &database_cells(db, ctx, false))?;
        }
        SpaceReport::Object(obj) => {
            markdown_header(writer, &OBJECT_FORM_HEADERS, 2)?;
            markdown_row(
                writer,
                &[
                    obj.database.clone(),
                    obj.object.clone(),
                    size_cell(obj.current_perm),
                    size_cell(obj.peak_perm),
                    ctx.styler.paint_optional(
                        skew_severity(ctx, obj.perm_skew_pct),
                        &pct_cell(obj.perm_skew_pct),
                    ),
                ],
            )?;
        }
    }
    Ok(())
}

// =============================================================================
// CSV rendering
// =============================================================================

/// Raw byte counts and unhumanized percentages, per REQ-SPACE-HUMAN-002.
///
/// Both NULL and "not applicable" render as an empty field.
fn display_csv<W: Write>(report: &SpaceReport, writer: &mut W) -> Result<()> {
    /// The database row's CSV fields, without the leading name column(s)
    fn db_fields(db: &DatabaseSpace) -> Vec<String> {
        vec![
            db.perm.current.to_string(),
            db.perm.peak.to_string(),
            pct_csv(db.perm.skew_pct),
            db.perm.max.to_string(),
            pct_csv(db.perm.pct_used()),
            db.spool.current.to_string(),
            db.spool.max.to_string(),
            db.spool.peak.to_string(),
            pct_csv(db.spool.skew_pct),
            db.temp.current.to_string(),
            db.temp.max.to_string(),
            db.temp.peak.to_string(),
            pct_csv(db.temp.skew_pct),
        ]
    }

    match report {
        SpaceReport::Database { header, objects } => {
            writeln!(writer, "{}", DATABASE_FORM_HEADERS.join(","))?;

            let mut row = vec!["DATABASE".to_string(), escape_csv(&header.database)];
            row.extend(db_fields(header));
            writeln!(writer, "{}", row.join(","))?;

            for obj in objects {
                let mut row = vec![
                    "TABLE".to_string(),
                    escape_csv(&obj.object),
                    obj.current_perm.to_string(),
                    obj.peak_perm.to_string(),
                    pct_csv(obj.perm_skew_pct),
                ];
                row.extend(std::iter::repeat_n(String::new(), 10));
                writeln!(writer, "{}", row.join(","))?;
            }
        }
        SpaceReport::DatabaseOnly(db) => {
            writeln!(writer, "{}", DBSPACE_HEADERS.join(","))?;
            let mut row = vec![escape_csv(&db.database)];
            row.extend(db_fields(db));
            writeln!(writer, "{}", row.join(","))?;
        }
        SpaceReport::Object(obj) => {
            writeln!(writer, "{}", OBJECT_FORM_HEADERS.join(","))?;
            writeln!(
                writer,
                "{},{},{},{},{}",
                escape_csv(&obj.database),
                escape_csv(&obj.object),
                obj.current_perm,
                obj.peak_perm,
                pct_csv(obj.perm_skew_pct)
            )?;
        }
    }
    Ok(())
}

// =============================================================================
// JSON rendering
// =============================================================================

/// Database-level JSON keys shared by `space` and `dbspace`
///
/// Byte counts are JSON numbers, not the driver's strings: the lenient
/// extractors already converted them. Skew serializes as `null` when absent.
/// `perm_unlimited` is present only when `MaxPerm = 0` (REQ-SPACE-NULL-003).
fn database_json(db: &DatabaseSpace) -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    map.insert("current_perm_bytes".into(), db.perm.current.into());
    map.insert("peak_perm_bytes".into(), db.perm.peak.into());
    map.insert("perm_skew_pct".into(), json_pct(db.perm.skew_pct));
    map.insert("max_perm_bytes".into(), db.perm.max.into());
    map.insert("perm_used_pct".into(), json_pct(db.perm.pct_used()));
    if db.perm.is_unlimited() {
        map.insert("perm_unlimited".into(), true.into());
    }
    map.insert("spool_current_bytes".into(), db.spool.current.into());
    map.insert("spool_max_bytes".into(), db.spool.max.into());
    map.insert("spool_peak_bytes".into(), db.spool.peak.into());
    map.insert("spool_skew_pct".into(), json_pct(db.spool.skew_pct));
    map.insert("temp_current_bytes".into(), db.temp.current.into());
    map.insert("temp_max_bytes".into(), db.temp.max.into());
    map.insert("temp_peak_bytes".into(), db.temp.peak.into());
    map.insert("temp_skew_pct".into(), json_pct(db.temp.skew_pct));
    map
}

/// Serialize an optional percentage, preserving NULL as `null`
fn json_pct(pct: Option<f64>) -> serde_json::Value {
    match pct {
        Some(v) => serde_json::json!(v),
        None => serde_json::Value::Null,
    }
}

/// Object rows omit database-only keys entirely rather than nulling them,
/// so a consumer can distinguish "not applicable" from "computed but NULL".
fn object_json(obj: &ObjectSpace, include_database: bool) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if include_database {
        map.insert("database".into(), obj.database.clone().into());
    } else {
        map.insert("kind".into(), "TABLE".into());
    }
    map.insert("object".into(), obj.object.clone().into());
    map.insert("current_perm_bytes".into(), obj.current_perm.into());
    map.insert("peak_perm_bytes".into(), obj.peak_perm.into());
    map.insert("perm_skew_pct".into(), json_pct(obj.perm_skew_pct));
    serde_json::Value::Object(map)
}

/// Render a report as JSON in the project's `{ok, row_count, data}` envelope
fn display_json<W: Write>(report: &SpaceReport, writer: &mut W) -> Result<()> {
    let data: Vec<serde_json::Value> = match report {
        SpaceReport::Database { header, objects } => {
            let mut header_map = serde_json::Map::new();
            header_map.insert("kind".into(), "DATABASE".into());
            header_map.insert("object".into(), header.database.clone().into());
            header_map.extend(database_json(header));

            let mut rows = vec![serde_json::Value::Object(header_map)];
            rows.extend(objects.iter().map(|o| object_json(o, false)));
            rows
        }
        SpaceReport::DatabaseOnly(db) => {
            let mut map = serde_json::Map::new();
            map.insert("database".into(), db.database.clone().into());
            map.extend(database_json(db));
            vec![serde_json::Value::Object(map)]
        }
        SpaceReport::Object(obj) => vec![object_json(obj, true)],
    };

    let output = serde_json::json!({
        "ok": true,
        "row_count": data.len(),
        "data": data,
    });
    writeln!(writer, "{}", serde_json::to_string_pretty(&output)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::severity::{SeverityStyler, Thresholds};

    fn plain_ctx() -> MonitoringContext {
        MonitoringContext::default()
    }

    fn colored_ctx() -> MonitoringContext {
        MonitoringContext::new(
            &crate::config::MonitoringThresholds::default(),
            &crate::config::MonitoringColors::default(),
            true,
        )
    }

    fn sample_database() -> DatabaseSpace {
        DatabaseSpace {
            database: "demo_user".to_string(),
            perm: SpaceMetrics {
                max: 10_737_418_240,
                current: 7_301_444_780,
                peak: 7_645_041_818,
                skew_pct: Some(4.2),
            },
            spool: SpaceMetrics {
                max: 2_147_483_648,
                current: 125_829_120,
                peak: 356_515_840,
                skew_pct: Some(12.5),
            },
            temp: SpaceMetrics {
                max: 1_073_741_824,
                current: 0,
                peak: 0,
                skew_pct: None,
            },
        }
    }

    fn sample_object(name: &str, current: i64, skew: Option<f64>) -> ObjectSpace {
        ObjectSpace {
            database: "demo_user".to_string(),
            object: name.to_string(),
            current_perm: current,
            peak_perm: current,
            perm_skew_pct: skew,
        }
    }

    fn sample_report() -> SpaceReport {
        SpaceReport::Database {
            header: sample_database(),
            objects: vec![
                sample_object("customers", 2_254_857_830, Some(2.0)),
                sample_object("orders", 3_435_973_836, Some(5.1)),
            ],
        }
    }

    // =========================================================================
    // parse_target
    // =========================================================================

    #[test]
    fn test_parse_target_bare_database() {
        assert_eq!(
            parse_target("demo_user").unwrap(),
            SpaceTarget::Database("demo_user".to_string())
        );
    }

    #[test]
    fn test_parse_target_qualified_object() {
        assert_eq!(
            parse_target("demo_user.orders").unwrap(),
            SpaceTarget::Object {
                database: "demo_user".to_string(),
                object: "orders".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_target_rejects_two_dots() {
        let err = parse_target("a.b.c").unwrap_err();
        assert_eq!(err.exit_code(), 2);
        assert!(err.user_message().contains("Invalid object reference 'a.b.c'"));
        assert!(err.user_message().contains("tq space demo_user.orders"));
    }

    #[test]
    fn test_parse_target_rejects_leading_dot() {
        assert!(parse_target(".orders").is_err());
    }

    #[test]
    fn test_parse_target_rejects_trailing_dot() {
        assert!(parse_target("demo_user.").is_err());
    }

    #[test]
    fn test_parse_target_rejects_empty() {
        assert!(parse_target("").is_err());
    }

    #[test]
    fn test_parse_target_quoted_name_containing_dot() {
        assert_eq!(
            parse_target("\"my.db\".orders").unwrap(),
            SpaceTarget::Object {
                database: "my.db".to_string(),
                object: "orders".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_target_strips_quotes() {
        assert_eq!(
            parse_target("\"Mixed Case\"").unwrap(),
            SpaceTarget::Database("Mixed Case".to_string())
        );
    }

    #[test]
    fn test_parse_database_target_accepts_bare_name() {
        assert_eq!(parse_database_target("demo_user").unwrap(), "demo_user");
    }

    #[test]
    fn test_parse_database_target_rejects_qualified_name() {
        let err = parse_database_target("demo_user.orders").unwrap_err();
        assert_eq!(err.exit_code(), 2);
        let msg = err.user_message();
        assert!(msg.contains("dbspace operates on databases only"));
        assert!(msg.contains("tq space demo_user.orders"));
        assert!(msg.contains("tq dbspace demo_user"));
    }

    // =========================================================================
    // SQL builders
    // =========================================================================

    #[test]
    fn test_build_database_sql_uses_diskspacev() {
        let sql = build_database_sql("demo_user");
        assert!(sql.contains("FROM DBC.DiskSpaceV"));
        assert!(sql.contains("UPPER(DatabaseName) = UPPER('demo_user')"));
        assert!(sql.contains("NULLIFZERO(MAX(CurrentPerm))"));
        // PermSkew/SpoolSkew/TempSkew are permissible limits, never selected.
        assert!(!sql.contains(" PermSkew,"));
    }

    #[test]
    fn test_build_object_sql_without_object_has_no_table_predicate() {
        let sql = build_object_sql("demo_user", None);
        assert!(sql.contains("FROM DBC.TableSizeV"));
        assert!(sql.contains("UPPER(DataBaseName) = UPPER('demo_user')"));
        assert!(!sql.contains("TableName) = UPPER("));
        // Alphabetical ordering per REQ-SPACE-003
        assert!(sql.trim_end().ends_with("ORDER BY 2"));
    }

    #[test]
    fn test_build_object_sql_with_object_adds_predicate() {
        let sql = build_object_sql("demo_user", Some("orders"));
        assert!(sql.contains("UPPER(TableName) = UPPER('orders')"));
    }

    #[test]
    fn test_sql_escapes_single_quotes() {
        let sql = build_database_sql("O'Brien");
        assert!(sql.contains("UPPER('O''Brien')"));
        let sql = build_object_sql("db", Some("it's"));
        assert!(sql.contains("UPPER('it''s')"));
    }

    #[test]
    fn test_probe_sql_shapes() {
        let sql = build_database_probe_sql("demo_user");
        assert!(sql.contains("DBC.DatabasesV"));
        assert!(sql.contains("DBKind"));

        let sql = build_object_probe_sql("demo_user", "orders");
        assert!(sql.contains("DBC.TablesV"));
        assert!(sql.contains("UPPER(TableName) = UPPER('orders')"));

        let sql = build_any_object_probe_sql("orders");
        assert!(sql.contains("TOP 1"));
        assert!(!sql.contains("DataBaseName) = UPPER"));
    }

    // =========================================================================
    // Row parsing — the driver delivers SUM(BIGINT) as Value::String
    // =========================================================================

    fn database_row_from_driver() -> Vec<Value> {
        vec![
            Value::String("demo_user".to_string()),
            Value::String("35829234636".to_string()), // MaxPerm
            Value::String("2260992".to_string()),     // CurrentPerm
            Value::String("2392064".to_string()),     // PeakPerm
            Value::Decimal(5.47945205479452),         // PermSkewPct
            Value::String("19327352832".to_string()), // MaxSpool
            Value::String("0".to_string()),           // CurrentSpool
            Value::String("1719795712".to_string()),  // PeakSpool
            Value::Null,                              // SpoolSkewPct
            Value::String("19327352832".to_string()), // MaxTemp
            Value::String("0".to_string()),           // CurrentTemp
            Value::String("0".to_string()),           // PeakTemp
            Value::Null,                              // TempSkewPct
        ]
    }

    #[test]
    fn test_database_from_row_parses_driver_strings() {
        let db = DatabaseSpace::from_row(&database_row_from_driver()).unwrap();
        assert_eq!(db.database, "demo_user");
        assert_eq!(db.perm.max, 35_829_234_636);
        assert_eq!(db.perm.current, 2_260_992);
        assert_eq!(db.perm.peak, 2_392_064);
        assert!((db.perm.skew_pct.unwrap() - 5.479_452_054_794_52).abs() < 1e-9);
        assert_eq!(db.spool.peak, 1_719_795_712);
        // NULL skew must stay None, never 0.0
        assert_eq!(db.spool.skew_pct, None);
        assert_eq!(db.temp.skew_pct, None);
    }

    #[test]
    fn test_database_from_row_rejects_short_row() {
        assert!(DatabaseSpace::from_row(&[Value::String("db".into())]).is_none());
    }

    #[test]
    fn test_database_from_row_rejects_null_name() {
        let mut row = database_row_from_driver();
        row[0] = Value::Null;
        assert!(DatabaseSpace::from_row(&row).is_none());
    }

    #[test]
    fn test_object_from_row_parses_driver_strings() {
        let row = vec![
            Value::String("demo_user".to_string()),
            Value::String("get_data".to_string()),
            Value::String("294912".to_string()),
            Value::String("294912".to_string()),
            Value::Decimal(35.714285714285715),
        ];
        let obj = ObjectSpace::from_row(&row).unwrap();
        assert_eq!(obj.database, "demo_user");
        assert_eq!(obj.object, "get_data");
        assert_eq!(obj.current_perm, 294_912);
        assert!((obj.perm_skew_pct.unwrap() - 35.714_285_714_285_715).abs() < 1e-9);
    }

    #[test]
    fn test_object_from_row_integer_skew() {
        // Teradata returns an exact 0 skew as an integer, not a decimal.
        let row = vec![
            Value::String("demo_user".to_string()),
            Value::String("points".to_string()),
            Value::String("0".to_string()),
            Value::String("0".to_string()),
            Value::Integer(0),
        ];
        let obj = ObjectSpace::from_row(&row).unwrap();
        assert_eq!(obj.perm_skew_pct, Some(0.0));
        assert_eq!(obj.current_perm, 0);
    }

    #[test]
    fn test_object_from_row_null_skew_is_none() {
        let row = vec![
            Value::String("demo_user".to_string()),
            Value::String("empty_tbl".to_string()),
            Value::String("0".to_string()),
            Value::String("0".to_string()),
            Value::Null,
        ];
        let obj = ObjectSpace::from_row(&row).unwrap();
        assert_eq!(obj.perm_skew_pct, None);
    }

    #[test]
    fn test_object_from_row_rejects_short_row() {
        assert!(ObjectSpace::from_row(&[Value::String("db".into())]).is_none());
    }

    // =========================================================================
    // SpaceMetrics
    // =========================================================================

    #[test]
    fn test_pct_used_none_when_max_zero() {
        let m = SpaceMetrics {
            max: 0,
            current: 100,
            peak: 100,
            skew_pct: None,
        };
        assert_eq!(m.pct_used(), None);
        assert!(m.is_unlimited());
        assert_eq!(used_cell(&m), UNLIMITED_DISPLAY);
    }

    #[test]
    fn test_pct_used_computes() {
        let m = SpaceMetrics {
            max: 200,
            current: 150,
            peak: 180,
            skew_pct: None,
        };
        assert!((m.pct_used().unwrap() - 75.0).abs() < 1e-9);
        assert!(!m.is_unlimited());
    }

    #[test]
    fn test_database_space_empty_is_all_zero() {
        let db = DatabaseSpace::empty("nospace");
        assert_eq!(db.perm.current, 0);
        assert_eq!(db.perm.skew_pct, None);
        assert!(db.perm.is_unlimited());
    }

    // =========================================================================
    // Cell helpers
    // =========================================================================

    #[test]
    fn test_pct_cell_null_marker() {
        assert_eq!(pct_cell(None), NULL_DISPLAY);
        assert_eq!(pct_cell(Some(4.25)), "4.2");
    }

    #[test]
    fn test_pct_csv_null_is_empty_field() {
        assert_eq!(pct_csv(None), "");
        assert_eq!(pct_csv(Some(12.5)), "12.5");
    }

    // =========================================================================
    // Table rendering
    // =========================================================================

    #[test]
    fn test_display_table_database_form_separates_header_row() {
        let mut out = Vec::new();
        display_table(&sample_report(), &mut out, &plain_ctx()).unwrap();
        let text = String::from_utf8(out).unwrap();

        assert!(text.contains("Space Analysis — Database: demo_user"));
        assert!(text.contains("DATABASE"));
        assert!(text.contains("TABLE"));
        assert!(text.contains("customers"));
        // Humanized bytes, not raw integers
        assert!(text.contains("GB"));
        assert!(!text.contains("7301444780"));
        // Footer
        assert!(text.contains("3 rows (1 database, 2 objects)"));
        assert!(text.contains("Total object CurrentPerm:"));
        // Database-only columns are "not applicable" on object rows
        assert!(text.contains(" - "));
    }

    #[test]
    fn test_display_table_object_form_single_row() {
        let report = SpaceReport::Object(sample_object("orders", 3_435_973_836, Some(5.1)));
        let mut out = Vec::new();
        display_table(&report, &mut out, &plain_ctx()).unwrap();
        let text = String::from_utf8(out).unwrap();

        assert!(text.contains("Space Analysis — Object: demo_user.orders"));
        assert!(text.contains("PermSkew%"));
        assert!(!text.contains("MaxPerm"));
        assert!(!text.contains("SpoolCurrent"));
    }

    #[test]
    fn test_display_table_dbspace_has_no_kind_column() {
        let report = SpaceReport::DatabaseOnly(sample_database());
        let mut out = Vec::new();
        display_table(&report, &mut out, &plain_ctx()).unwrap();
        let text = String::from_utf8(out).unwrap();

        assert!(text.contains("Database"));
        assert!(!text.contains("Kind"));
        assert!(text.contains("TempSkew%"));
        // No object rows
        assert!(!text.contains("TABLE"));
    }

    #[test]
    fn test_display_table_null_skew_renders_marker() {
        let report = SpaceReport::DatabaseOnly(sample_database());
        let mut out = Vec::new();
        display_table(&report, &mut out, &plain_ctx()).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains(NULL_DISPLAY));
    }

    #[test]
    fn test_display_table_unlimited_perm() {
        let mut db = sample_database();
        db.perm.max = 0;
        let report = SpaceReport::DatabaseOnly(db);
        let mut out = Vec::new();
        display_table(&report, &mut out, &plain_ctx()).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains(UNLIMITED_DISPLAY));
    }

    #[test]
    fn test_display_table_no_ansi_when_color_disabled() {
        let mut out = Vec::new();
        display_table(&sample_report(), &mut out, &plain_ctx()).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(!text.contains('\x1b'));
    }

    #[test]
    fn test_display_table_colors_critical_skew() {
        // Default skew thresholds are 40/70, so 85% skew is Critical.
        let report = SpaceReport::Object(sample_object("hot", 1024, Some(85.0)));
        let mut out = Vec::new();
        display_table(&report, &mut out, &colored_ctx()).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains('\x1b'));
    }

    #[test]
    fn test_display_table_does_not_color_null_skew() {
        let report = SpaceReport::Object(sample_object("idle", 0, None));
        let mut out = Vec::new();
        display_table(&report, &mut out, &colored_ctx()).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(!text.contains('\x1b'));
    }

    // =========================================================================
    // CSV rendering
    // =========================================================================

    #[test]
    fn test_display_csv_database_form() {
        let mut out = Vec::new();
        display_csv(&sample_report(), &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        let lines: Vec<&str> = text.lines().collect();

        assert_eq!(lines.len(), 4);
        assert!(lines[0].starts_with("Kind,Object,CurrentPerm,PeakPerm,PermSkew%,MaxPerm"));
        assert!(lines[1].starts_with("DATABASE,demo_user,7301444780,7645041818,4.2"));
        assert!(lines[2].starts_with("TABLE,customers,2254857830,2254857830,2.0"));
        // Object rows leave the database-only columns empty
        assert!(lines[2].ends_with(",,,,,,,,,,"));
        // Every row has the same field count as the header
        let cols = lines[0].split(',').count();
        for line in &lines[1..] {
            assert_eq!(line.split(',').count(), cols, "ragged row: {line}");
        }
    }

    #[test]
    fn test_display_csv_null_skew_is_empty_field() {
        let report = SpaceReport::DatabaseOnly(sample_database());
        let mut out = Vec::new();
        display_csv(&report, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        // TempSkew% is NULL and is the last field
        assert!(text.lines().nth(1).unwrap().ends_with(','));
    }

    #[test]
    fn test_display_csv_escapes_names() {
        let report = SpaceReport::Object(ObjectSpace {
            database: "db,with,commas".to_string(),
            object: "obj\"quoted".to_string(),
            current_perm: 10,
            peak_perm: 20,
            perm_skew_pct: Some(1.0),
        });
        let mut out = Vec::new();
        display_csv(&report, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("\"db,with,commas\""));
        assert!(text.contains("\"obj\"\"quoted\""));
    }

    #[test]
    fn test_display_csv_never_emits_ansi() {
        let mut out = Vec::new();
        display_csv(&sample_report(), &mut out).unwrap();
        assert!(!String::from_utf8(out).unwrap().contains('\x1b'));
    }

    #[test]
    fn test_display_csv_uses_raw_bytes_not_humanized() {
        let mut out = Vec::new();
        display_csv(&sample_report(), &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("7301444780"));
        assert!(!text.contains("GB"));
    }

    // =========================================================================
    // JSON rendering
    // =========================================================================

    #[test]
    fn test_display_json_database_form() {
        let mut out = Vec::new();
        display_json(&sample_report(), &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();

        assert_eq!(json["ok"], true);
        assert_eq!(json["row_count"], 3);
        let data = json["data"].as_array().unwrap();

        assert_eq!(data[0]["kind"], "DATABASE");
        assert_eq!(data[0]["object"], "demo_user");
        assert_eq!(data[0]["current_perm_bytes"], 7_301_444_780i64);
        assert!(data[0]["current_perm_bytes"].is_number());
        assert!(data[0]["temp_skew_pct"].is_null());
        // MaxPerm > 0, so the unlimited flag is absent entirely
        assert!(data[0].get("perm_unlimited").is_none());

        assert_eq!(data[1]["kind"], "TABLE");
        // Database-only keys are omitted, not nulled, on object rows
        assert!(data[1].get("max_perm_bytes").is_none());
        assert!(data[1].get("spool_current_bytes").is_none());
    }

    #[test]
    fn test_display_json_unlimited_flag() {
        let mut db = sample_database();
        db.perm.max = 0;
        let mut out = Vec::new();
        display_json(&SpaceReport::DatabaseOnly(db), &mut out).unwrap();
        let json: serde_json::Value =
            serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
        let row = &json["data"][0];
        assert_eq!(row["perm_unlimited"], true);
        assert!(row["perm_used_pct"].is_null());
    }

    #[test]
    fn test_display_json_object_form() {
        let report = SpaceReport::Object(sample_object("orders", 3_435_973_836, Some(5.1)));
        let mut out = Vec::new();
        display_json(&report, &mut out).unwrap();
        let json: serde_json::Value =
            serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
        let row = &json["data"][0];
        assert_eq!(row["database"], "demo_user");
        assert_eq!(row["object"], "orders");
        assert_eq!(row["current_perm_bytes"], 3_435_973_836i64);
    }

    #[test]
    fn test_display_json_never_emits_ansi() {
        let mut out = Vec::new();
        display_json(&sample_report(), &mut out).unwrap();
        assert!(!String::from_utf8(out).unwrap().contains('\x1b'));
    }

    // =========================================================================
    // Markdown rendering
    // =========================================================================

    #[test]
    fn test_display_markdown_database_form() {
        let mut out = Vec::new();
        display_markdown(&sample_report(), &mut out, &plain_ctx()).unwrap();
        let text = String::from_utf8(out).unwrap();
        let lines: Vec<&str> = text.lines().collect();

        assert!(lines[0].starts_with("| Kind | Object |"));
        assert!(lines[1].contains("---"));
        assert!(lines[2].contains("| DATABASE | demo_user |"));
        assert_eq!(lines.len(), 5);
        assert!(text.contains("GB"));
    }

    #[test]
    fn test_display_markdown_escapes_pipes() {
        let report = SpaceReport::Object(ObjectSpace {
            database: "db".to_string(),
            object: "we|ird".to_string(),
            current_perm: 1,
            peak_perm: 1,
            perm_skew_pct: None,
        });
        let mut out = Vec::new();
        display_markdown(&report, &mut out, &plain_ctx()).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("we\\|ird"));
    }

    #[test]
    fn test_display_markdown_no_ansi_when_disabled() {
        let mut out = Vec::new();
        display_markdown(&sample_report(), &mut out, &plain_ctx()).unwrap();
        assert!(!String::from_utf8(out).unwrap().contains('\x1b'));
    }

    // =========================================================================
    // render() dispatch
    // =========================================================================

    #[test]
    fn test_render_dispatches_all_four_formats() {
        for format in [
            OutputFormat::Table,
            OutputFormat::Csv,
            OutputFormat::Json,
            OutputFormat::Markdown,
            OutputFormat::Md,
        ] {
            let mut out = Vec::new();
            render(&sample_report(), format, &mut out, &plain_ctx()).unwrap();
            assert!(!out.is_empty(), "{format:?} produced no output");
        }
    }

    #[test]
    fn test_structured_formats_stay_clean_with_color_enabled() {
        for format in [OutputFormat::Json, OutputFormat::Csv] {
            let mut out = Vec::new();
            render(&sample_report(), format, &mut out, &colored_ctx()).unwrap();
            let text = String::from_utf8(out).unwrap();
            assert!(!text.contains('\x1b'), "{format:?} leaked ANSI codes");
        }
    }

    #[test]
    fn test_context_parts_are_reachable() {
        // Guards the public surface used by main.rs and the REPL.
        let ctx = MonitoringContext {
            thresholds: Thresholds::default(),
            styler: SeverityStyler::disabled(),
            refresh_interval: 6,
        };
        assert!(!ctx.styler.is_enabled());
    }
}
