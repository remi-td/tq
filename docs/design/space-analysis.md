# Space Analysis Design (`tq space`, `tq dbspace`)

## Overview

`tq space` and `tq dbspace` give administrators a one-command view of permanent, spool and
temporary space usage for a database or an individual object, replacing hand-written
`DBC.DiskSpaceV` / `DBC.TableSizeV` queries.

| Invocation | Result |
|------------|--------|
| `tq space <database>` | One database header row followed by one row per object directly under the database |
| `tq space <database>.<object>` | Exactly one object row |
| `tq dbspace <database>` | Database-level perm/spool/temp metrics only |
| `tq dbspace <database>.<object>` | Usage error — `dbspace` operates on databases only |

All four output formats (`table`, `json`, `csv`, `markdown`) are supported, following the
structure established by `src/commands/skew.rs`.

---

## Data Sources

Two DBC views are used. Both are per-AMP: each returns one row per `Vproc`, so every metric
must be aggregated across AMPs, and skew is derived from the distribution across those rows.

### `DBC.DiskSpaceV` — database level

Verified column set:

```
Vproc, DatabaseName, AccountName,
MaxPerm, MaxSpool, MaxTemp,
CurrentPerm, CurrentSpool, CurrentPersistentSpool, CurrentTemp,
PeakPerm, PeakSpool, PeakPersistentSpool, PeakTemp,
MaxProfileSpool, MaxProfileTemp,
TrustUserName, AppProxyUser,
AllocatedPerm, AllocatedSpool, AllocatedTemp,
PermSkew, SpoolSkew, TempSkew
```

Two notes that materially affect the design:

- The naming is `Max*` / `Current* `/ `Peak*` / `Allocated*` for each of `Perm`, `Spool`,
  `Temp`. There is no `MaxPermSpace`-style suffix.
- `PermSkew`, `SpoolSkew` and `TempSkew` are **not** measured skew. Per the view's own column
  comments they return "the permissible skew limit percent … at the Global level" — a
  configured limit, not an observation. They are deliberately unused; skew is computed from
  the `AVG`/`MAX` distribution instead.

### `DBC.TableSizeV` — object level

Verified column set — deliberately small:

```
Vproc, DataBaseName, AccountName, TableName, CurrentPerm, PeakPerm
```

Two consequences:

- The database column is spelled **`DataBaseName`** (capital `B`), unlike `DiskSpaceV`'s
  `DatabaseName`. Teradata identifier resolution is case-insensitive so either spelling
  executes, but the SQL uses each view's own spelling to stay faithful to the catalog.
- There is **no `MaxPerm`** at object level. Perm allocation is a database-level property, so
  object rows expose `CurrentPerm`, `PeakPerm` and perm skew only, and the `MaxPerm` /
  `% used` columns are populated for the database header row alone.

`TableSizeV` covers every object that consumes perm space, including stored procedures
(`TableKind = 'P'`), so no `TableKind` filter is applied — filtering would silently omit rows
that legitimately hold space.

---

## SQL

Both statements below were executed against a live Teradata system and returned the expected
shapes.

### Database level

```sql
SELECT
    DatabaseName,
    SUM(MaxPerm)                                                    AS MaxPerm,
    SUM(CurrentPerm)                                                AS CurrentPerm,
    SUM(PeakPerm)                                                   AS PeakPerm,
    (100 - (AVG(CurrentPerm)  / NULLIFZERO(MAX(CurrentPerm))  * 100)) AS PermSkewPct,
    SUM(MaxSpool)                                                   AS MaxSpool,
    SUM(CurrentSpool)                                               AS CurrentSpool,
    SUM(PeakSpool)                                                  AS PeakSpool,
    (100 - (AVG(CurrentSpool) / NULLIFZERO(MAX(CurrentSpool)) * 100)) AS SpoolSkewPct,
    SUM(MaxTemp)                                                    AS MaxTemp,
    SUM(CurrentTemp)                                                AS CurrentTemp,
    SUM(PeakTemp)                                                   AS PeakTemp,
    (100 - (AVG(CurrentTemp)  / NULLIFZERO(MAX(CurrentTemp))  * 100)) AS TempSkewPct
FROM DBC.DiskSpaceV
WHERE UPPER(DatabaseName) = UPPER('<database>')
GROUP BY 1
```

### Object level

```sql
SELECT
    DataBaseName,
    TableName,
    SUM(CurrentPerm)                                                AS CurrentPerm,
    SUM(PeakPerm)                                                   AS PeakPerm,
    (100 - (AVG(CurrentPerm) / NULLIFZERO(MAX(CurrentPerm)) * 100))   AS PermSkewPct
FROM DBC.TableSizeV
WHERE UPPER(DataBaseName) = UPPER('<database>')
  [AND UPPER(TableName) = UPPER('<object>')]
GROUP BY 1, 2
ORDER BY 2
```

The `AND UPPER(TableName) = ...` predicate is present only for the `<db>.<obj>` form.
Ordering is alphabetical by object name (REQ-SPACE-003), matching `tq list tables`, rather
than by size — the command is a reference view, not a "biggest offenders" report.

`AllocatedPerm` / `AllocatedSpool` / `AllocatedTemp` exist on the view but are not selected:
`Max*` is the allocation the user asked about, and carrying a second near-synonymous column
per space class would have widened an already 15-column table for no decision-support gain.
`COUNT(DISTINCT Vproc)` is likewise not selected — the AMP count is a system property already
available from `tq sysconfig`, not a property of this database's space.

### Skew formula and NULL safety

`100 - (AVG(x) / NULLIFZERO(MAX(x)) * 100)` is the formula from the feature request. It
measures how far the average AMP's usage falls below the busiest AMP's usage:

- Perfectly even distribution → `0`.
- One AMP holding everything on an *n*-AMP system → `100 * (n-1) / n`.

`NULLIFZERO` on the denominator is what makes it safe: when an object or a space class holds
nothing on any AMP, `MAX(x)` is 0, the divisor becomes `NULL`, and the whole expression
evaluates to `NULL` rather than raising a divide-by-zero. This was confirmed live — a
database with `CurrentSpool = 0` returns `SpoolSkewPct: null`, not an error. `NULL` is
carried through as `Option<f64> = None` and rendered as `-` in table output, an empty field
in CSV, and `null` in JSON. It is explicitly *not* coerced to `0.0`, which would misreport
"no data" as "perfect distribution".

`PermUsed%` is computed client-side rather than in SQL, guarding `MaxPerm == 0` by yielding
`None`. That guard carries a semantic distinction the renderers must preserve: in Teradata
`MaxPerm = 0` conventionally means *no perm limit*, not *zero capacity*. `pct_used()` returns
`None` in both the "unlimited" and the "genuinely absent" case, so `is_unlimited()` exists to
tell them apart. Table and markdown render `[unlimited]` rather than `[--]`; JSON keeps
`perm_used_pct: null` and adds a sibling `"perm_unlimited": true`, present only in that case
(REQ-SPACE-NULL-003).

---

## Numeric Extraction

The Teradata driver delivers `DECIMAL`, `NUMBER` and `BIGINT` as JSON strings, which the
result mapper surfaces as `Value::String`. This was confirmed against the live system: the
database-level query returns `"MaxPerm": "35829234636"` — a quoted string — while the derived
skew percentage returns an unquoted `5.47945205479452`.

Every column in these queries is a `SUM` over `BIGINT`, so **all byte counts arrive as
`Value::String`**. The existing helpers in `src/commands/monitoring_utils.rs` do not handle
this: `extract_integer` (`src/commands/monitoring_utils.rs:15`) matches only
`Value::Integer` / `Value::Decimal` / `Value::Null` and returns `None` for `Value::String`,
a behaviour pinned by `test_extract_integer_from_string`. Using it here would silently zero
out every metric.

Two lenient variants are therefore added to the same shared module, leaving the existing
functions and their contracts untouched:

```rust
/// Like `extract_integer`, but additionally parses numeric `Value::String`
/// payloads as produced by the driver for BIGINT / DECIMAL / NUMBER columns.
pub fn extract_i64_lenient(value: &Value) -> Option<i64>;

/// Like `extract_decimal`, but additionally parses numeric `Value::String` payloads.
pub fn extract_f64_lenient(value: &Value) -> Option<f64>;
```

Both trim the string before parsing (Teradata pads fixed-width character output) and return
`None` on parse failure, so a malformed cell degrades to "unavailable" rather than panicking.
`extract_i64_lenient` parses via `f64` when a decimal point is present, then truncates,
matching `extract_integer`'s existing `Decimal → i64` semantics.

The space module uses the lenient variants exclusively for numeric columns and the existing
`extract_trimmed_string` for names.

---

## Module Structure

New module `src/commands/space.rs`, modelled on `src/commands/skew.rs`.

### Types

```rust
/// One object's perm footprint. `MaxPerm` is absent by construction —
/// TableSizeV does not expose it.
pub struct ObjectSpace {
    pub database: String,
    pub object: String,
    pub current_perm: i64,
    pub peak_perm: i64,
    pub perm_skew_pct: Option<f64>,
}

/// A database's perm, spool and temp footprint.
pub struct DatabaseSpace {
    pub database: String,
    pub perm:  SpaceMetrics,
    pub spool: SpaceMetrics,
    pub temp:  SpaceMetrics,
}

/// One space class, aggregated across AMPs.
pub struct SpaceMetrics {
    pub max: i64,
    pub current: i64,
    pub peak: i64,
    pub skew_pct: Option<f64>,
}

impl SpaceMetrics {
    /// None when `max == 0` — see `is_unlimited`.
    pub fn pct_used(&self) -> Option<f64>;
    /// `max == 0` means "no limit" in Teradata, not "zero capacity".
    pub fn is_unlimited(&self) -> bool;
}

/// What a `space` invocation produced — drives which renderer shape is used.
pub enum SpaceReport {
    /// `tq space <db>`: header + members
    Database { header: DatabaseSpace, objects: Vec<ObjectSpace> },
    /// `tq space <db>.<obj>`
    Object(ObjectSpace),
    /// `tq dbspace <db>`
    DatabaseOnly(DatabaseSpace),
}
```

Modelling the three shapes as one enum rather than three parallel render paths means each
formatter is a single `match` and the "which columns exist" question is answered by the type
system rather than by `Option` fields that are conditionally meaningful.

### Row constructors

`ObjectSpace::from_row(&[Value]) -> Option<Self>` and
`DatabaseSpace::from_row(&[Value]) -> Option<Self>` follow the `SkewInfo::from_row` pattern
(`src/commands/skew.rs:83`): check arity first, return `None` on a missing key column, and
default absent numerics to `0`. Callers use `.filter_map(...)` over `result.rows`, so a
malformed row is dropped rather than aborting the command.

`Row` is `Vec<Value>` (`src/db/types.rs`), accessed positionally via indexing after the
arity check — there is no named-field accessor.

---

## Target Parsing and Validation

A shared parser turns the positional argument into a validated target:

```rust
enum SpaceTarget {
    Database(String),
    Object { database: String, object: String },
}

fn parse_target(input: &str) -> Result<SpaceTarget>;
```

Rules:

- Zero dots → `Database`.
- Exactly one dot, both sides non-empty → `Object`.
- More than one dot, or an empty side → parse error naming the expected form
  `<database>[.<object>]`.

`dbspace` calls the same parser and rejects `SpaceTarget::Object` with an actionable message:

```
Error: 'dbspace' operates on databases only, but 'demo_user.orders' names an object.
Hint: use 'tq space demo_user.orders' for object-level space, or
      'tq dbspace demo_user' for the database.
```

Quoted identifiers are handled by the project's existing identifier-quoting helper (see
`docs/design/cli-interface.md`, *Identifier Quoting Fix*) so that a name containing a dot can
be expressed as `"my.db".tbl`.

### Existence checking

A database that simply holds no space returns zero rows from `DiskSpaceV`, which is
indistinguishable from a misspelled name at the query level. Before reporting "not found",
the command probes the catalog:

```sql
SELECT DatabaseName, DBKind FROM DBC.DatabasesV
WHERE UPPER(DatabaseName) = UPPER('<database>')
```

`DBKind` is `'D'` for a database and `'U'` for a user; both are accepted, since users own
space exactly as databases do (the live test system's `demo_user` and `DBC` are both `'U'`).
When the probe returns no row, the command emits `TqError::ObjectNotFound` (exit code 1).
No spelling suggestion is offered: no fuzzy-match helper exists anywhere in `src/`, and
inventing one was descoped. When the probe succeeds but the space query is empty, the command
reports zero usage rather than an error — an empty database is a valid answer, not a failure.

The object form probes `DBC.TablesV` on `(DataBaseName, TableName)` for the same reason.

`dbspace` adds one further step. When the database probe fails, it re-probes `DBC.TablesV`
for an object of that name in *any* database. If one is found, the error names the right
command rather than merely reporting "not found" (REQ-DBSPACE-003):

```
Error: Database 'evals_employees' not found.

'evals_employees' is an object in database 'demo_user', not a database.

Hint: use 'tq space demo_user.evals_employees' for object-level space.
```

Every probe is skipped on the happy path: they run only when the space query returned no
rows, so the common case costs one round trip, not two.

### Error types

Two variants were added to `src/error.rs` rather than reusing `TableNotFound` or
`InvalidConfig`, whose existing user-facing messages ("Table ... does not exist", "Invalid
configuration") would have been actively misleading here:

| Variant | Exit code | Used for |
|---------|-----------|----------|
| `TqError::ObjectNotFound { object_type, name, hint }` | 1 | An unknown database or object; `hint` carries the actionable follow-up |
| `TqError::InvalidObjectReference { reference, expected, usage }` | 2 | A malformed target (`a.b.c`) or a qualified name given to `dbspace` |

Both are general rather than space-specific, so other commands can adopt them.

---

## Output Formats

Four renderers, mirroring `src/commands/skew.rs:167`:

```rust
fn display_table<W: Write>(report: &SpaceReport, w: &mut W, ctx: &MonitoringContext) -> Result<()>;
fn display_markdown<W: Write>(report: &SpaceReport, w: &mut W, ctx: &MonitoringContext) -> Result<()>;
fn display_json<W: Write>(report: &SpaceReport, w: &mut W) -> Result<()>;
fn display_csv<W: Write>(report: &SpaceReport, w: &mut W) -> Result<()>;
```

Only the two human-facing renderers receive the `MonitoringContext`. `display_json` and
`display_csv` have no parameter through which a styler could reach them, which is what
structurally prevents ANSI escapes from reaching machine-readable output (see
`docs/design/monitoring.md`).

### Table

Byte counts are rendered through the existing `format_size(bytes, precision)` helper
(`src/commands/format_helpers.rs`) at one decimal place, so `tq space` reports `1.2 GB` in the
same style as `tq list tables`. Tables are built with `comfy_table` using the `UTF8_FULL`
preset and `ContentArrangement::Dynamic`, as in `src/commands/skew.rs`, with numeric columns
set to `CellAlignment::Right`.

Three column sets exist, one per `SpaceReport` variant:

| Variant | Columns |
|---------|---------|
| `Database` | `Kind`, `Object`, then the 13 metric columns |
| `DatabaseOnly` | `Database`, then the same 13 metric columns (no `Kind` — it is always a database) |
| `Object` | `Database`, `Object`, `CurrentPerm`, `PeakPerm`, `PermSkew%` |

In the `Database` variant the header row is distinguished by its `Kind = DATABASE` cell and by
the fact that object rows render `-` in all ten database-only columns, so the database total
cannot be mistaken for another object. A footer reports
`N rows (1 database, M objects) | Total object CurrentPerm: X` (REQ-SPACE-007).

### JSON

The envelope matches the project convention `{ "ok": true, "row_count": N, "data": [...] }`,
as emitted by `tq list`, `tq inspect` and `tq skew`. The specification illustrates this
command's JSON with a bare array; the enveloped form is used instead so that `tq space` is not
the single command in the tool with a different top-level shape. Row *contents* — snake_case
keys, the `_bytes` suffix, key omission, `perm_unlimited` — follow the specification exactly,
and those are the parts a consumer scripts against.

Byte counts serialize as JSON numbers, not the driver's strings: the lenient extractors have
already converted them, so the API surface is correctly typed (consistent with the *JSON API
Type Correctness* section of `docs/design/cli-interface.md`). `Option<f64>` skew serializes
as `null` when absent.

Rows are heterogeneous by design. The header row carries `kind: "DATABASE"` plus the full
perm/spool/temp key set; object rows carry `kind: "TABLE"` and **omit** the database-only keys
entirely rather than setting them to `null`. That is what lets a consumer distinguish "not
applicable to this row" (key absent) from "computed but NULL" (`key: null`) — a distinction
that would be lost if the rows were null-padded to a uniform shape. `row_count` counts all
emitted rows, header included.

### CSV and Markdown

CSV cannot express key omission, so it uses one fixed column set — the database form's 15
columns — and leaves both NULL and inapplicable cells as empty fields. Rows are therefore
never ragged, which is what `csv`-consuming tools require. Byte counts are raw integers, never
humanized (REQ-SPACE-HUMAN-002), so `awk`/`jq` comparisons are unambiguous.

Markdown mirrors the table format exactly, including humanized byte sizes and severity color.

`escape_csv` (`src/commands/monitoring_utils.rs`) and `markdown_escape_pipe`
(`src/commands/format_helpers.rs`) are reused for escaping.

---

## Severity Integration

Space output participates in the shared severity layer described in
`docs/design/monitoring.md`:

- Perm / spool / temp skew percentages classify against the `skew` threshold family.
- The database header's `PermUsed%` classifies against the `space` family.
- `None` metrics are rendered unstyled — absent data has no severity (REQ-COLOR-002).
- An unlimited allocation (`MaxPerm = 0`) is likewise unstyled: there is no percentage to
  classify.

---

## CLI Integration

Per the new-command checklist, the following are touched:

| File | Change |
|------|--------|
| `src/cli.rs` | `Command::Space(SpaceArgs)` and `Command::Dbspace(DbspaceArgs)` variants; both arg structs; format dispatch |
| `src/commands/space.rs` | New module: SQL, types, `execute`, `execute_for_repl`, renderers |
| `src/commands/mod.rs` | `pub mod space;` and `pub use space::execute as space;` |
| `src/main.rs` | Dispatch arms for both commands, including the `--output` file branch |
| `src/lib.rs` | Re-export `SpaceArgs`, `DbspaceArgs` |
| `src/commands/repl/metacommands.rs` | `/space` and `/dbspace` handlers in both `handle_metacommand` and `handle_metacommand_with_state` |
| `src/error.rs` | `ObjectNotFound` and `InvalidObjectReference` variants |
| `src/commands/repl/metadata_completer.rs` | `MetacommandDef` entries for tab completion |
| `src/commands/repl/metacommands.rs` | Entries in `print_help_extended` |

Argument shapes:

```rust
pub struct SpaceArgs {
    /// <database> or <database>.<object>
    pub target: String,
    pub format: OutputFormat,
    pub output: Option<PathBuf>,
}

pub struct DbspaceArgs {
    /// <database>
    pub database: String,
    pub format: OutputFormat,
    pub output: Option<PathBuf>,
}
```

Both commands share one implementation; `dbspace` is `space` with the target constrained to a
database and the member query skipped.

---

## Testing Strategy

Unit tests in `src/commands/space.rs`:

- `parse_target`: bare name, qualified name, two dots rejected, leading/trailing dot rejected,
  quoted identifier containing a dot.
- `dbspace` rejects a qualified target with the actionable message.
- `ObjectSpace::from_row` / `DatabaseSpace::from_row` with `Value::String` byte counts — the
  regression test that guards against the driver's string delivery.
- `from_row` with too few columns returns `None`.
- `from_row` with `Value::Null` skew yields `None`, not `Some(0.0)`.
- `SpaceMetrics::pct_used` returns `None` when `max == 0`.
- SQL builders: object predicate present only for the qualified form; database name is
  correctly escaped.
- Each renderer against a fixed `SpaceReport` for all three variants, asserting header
  presence, column order, `null` handling, and absence of `\x1b` in the non-table formats.

Integration tests exercise all three invocation shapes and the error paths against the live
system configured in `.env`.

---

## Code Linkage

| Element | Location |
|---------|----------|
| New module | `src/commands/space.rs` |
| Pattern reference (args, SQL, four renderers) | `src/commands/skew.rs` |
| `SkewInfo::from_row` pattern | `src/commands/skew.rs:83` |
| Four-format dispatch pattern | `src/commands/skew.rs:167` |
| `extract_integer` (strict, `Value::String` -> `None`) | `src/commands/monitoring_utils.rs` |
| `extract_i64_lenient` / `extract_f64_lenient` | `src/commands/monitoring_utils.rs` |
| `extract_trimmed_string`, `escape_csv` | `src/commands/monitoring_utils.rs` |
| `format_size` | `src/commands/format_helpers.rs` |
| `escape_sql_string` | `src/sql/identifiers.rs` |
| `Value` enum, `Row = Vec<Value>` | `src/db/types.rs` |
| Severity layer | `docs/design/monitoring.md` |
