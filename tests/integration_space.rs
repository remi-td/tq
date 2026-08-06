//! Integration tests for `tq space` / `tq dbspace` (Issue #54, Sprint 76)
//!
//! Corresponds to TC109-I01 through TC109-I14 in
//! `tests/strategy/sprint-76-strategy.md`. Every test spawns the built `tq`
//! binary against the live database configured by `TQ_LOGON` (loaded from
//! `.env` via `dotenvy`) — exactly how a real user would invoke `space` /
//! `dbspace` — because only a live run can prove the SQL in issue #54 is
//! valid against a real `DBC.DiskSpaceV` / `DBC.TableSizeV` catalog (see
//! `docs/design/space-analysis.md`).
//!
//! Run with:
//! ```bash
//! cargo test --test integration_space -- --ignored
//! ```
//!
//! ## Sprint 76 Phase 2 rulings that shape these assertions
//!
//! - **Descoped** (planning doc, "Descoped in Phase 2"): no fuzzy-match /
//!   spelling-suggestion helper exists anywhere in `src/`. TC109-I06/I07
//!   assert only the project's standard not-found message, not a suggestion.
//! - `demo_user` and `DBC` are both Teradata *users* (`DBKind = 'U'`), not
//!   `DBKind = 'D'` databases, so `dbspace demo_user` must succeed — a naive
//!   "must be DBKind='D'" check would incorrectly reject it (ruling #6 probe
//!   against `DBC.DatabasesV`).
//! - Because `src/commands/space.rs` did not exist at the time this file was
//!   written (architect implementing in parallel), exact JSON key names,
//!   error wording, and table column headers are unknown. Assertions are
//!   deliberately black-box: they check exit codes, structural validity
//!   (valid JSON/CSV/markdown), row counts via line-splitting, and substring
//!   presence of domain values (table/database names) rather than exact
//!   field names. This is expected to need reconciliation once `space.rs`
//!   lands; see the coordinator's execution loop.

#![allow(deprecated)] // cargo_bin is the standard way to find binary in assert_cmd

#[path = "helpers/mod.rs"]
mod helpers;

use assert_cmd::Command;
use std::process::Output;

/// A live database/user known to exist on the configured Teradata system
/// (confirmed live in `docs/design/space-analysis.md`), used as the target
/// for `space`/`dbspace` happy-path tests.
const LIVE_DB: &str = "demo_user";

/// Build a `tq` subprocess wired to the live database from `.env`/`TQ_LOGON`.
fn tq_cmd() -> Command {
    dotenvy::dotenv().ok();
    let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");
    let mut cmd = Command::cargo_bin("tq").unwrap();
    cmd.env("TQ_LOGON", logon);
    cmd.env_remove("TQ_PROFILE");
    cmd
}

fn run(args: &[&str]) -> Output {
    tq_cmd().args(args).output().unwrap()
}

fn stdout_str(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_str(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// A unique-per-run name so parallel/repeated test executions never collide
/// on a leftover fixture table.
fn unique_name(prefix: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{prefix}_{nanos}")
}

/// Create a small permanent table under [`LIVE_DB`] so `tq space <db>` has a
/// real child object to enumerate. Panics (does not silently skip) if
/// creation fails — a fixture setup failure is a real signal, not a reason
/// to skip the test.
fn create_fixture_table(name: &str) {
    let qualified = format!("{LIVE_DB}.{name}");
    let create_sql =
        format!("CREATE TABLE {qualified} (id INTEGER, note VARCHAR(32)) PRIMARY INDEX (id)");
    let out = run(&["query", &create_sql]);
    assert!(
        out.status.success(),
        "fixture table creation failed for {qualified}: stdout={} stderr={}",
        stdout_str(&out),
        stderr_str(&out)
    );
}

fn drop_fixture_table(name: &str) {
    let qualified = format!("{LIVE_DB}.{name}");
    let _ = run(&["query", &format!("DROP TABLE {qualified}")]);
}

/// Panics do not "silently" fail a test the way a returned `Err` might be
/// swallowed — but stderr containing "panicked at" would mean the *tq*
/// process itself panicked, which is always a bug regardless of what the
/// test intended to check.
fn assert_no_panic(out: &Output) {
    let stderr = stderr_str(out);
    assert!(
        !stderr.contains("panicked at"),
        "tq process panicked, this is a bug regardless of test outcome: {stderr}"
    );
}

/// Count of non-empty CSV lines in `s`, excluding the header line (line 0).
fn csv_data_row_count(s: &str) -> usize {
    let lines: Vec<&str> = s.lines().filter(|l| !l.trim().is_empty()).collect();
    lines.len().saturating_sub(1)
}

// =============================================================================
// TC109-I01: `tq space <db>` returns a header row + one row per object
// =============================================================================

/// TC109-I01: `tq space <db>` returns a database header row plus one row per
/// object directly under it (AC bullet 1).
#[test]
#[ignore]
fn tc109_i01_space_database_returns_header_and_object_rows() {
    let table = unique_name("tq_space_i01");
    create_fixture_table(&table);

    let out = run(&["space", LIVE_DB, "--format", "csv"]);
    drop_fixture_table(&table);

    assert!(
        out.status.success(),
        "tq space {LIVE_DB} failed: stderr={}",
        stderr_str(&out)
    );
    let stdout = stdout_str(&out);
    // One database header row plus at least the fixture object row.
    assert!(
        csv_data_row_count(&stdout) >= 2,
        "expected at least 2 data rows (database header + >=1 object): {stdout}"
    );
    assert!(
        stdout.to_lowercase().contains(&table.to_lowercase()),
        "expected fixture table {table} among the object rows: {stdout}"
    );
}

// =============================================================================
// TC109-I02: `tq space <db>.<obj>` returns exactly one row
// =============================================================================

/// TC109-I02: `tq space <db>.<obj>` returns exactly one row for that object
/// (AC bullet 2).
#[test]
#[ignore]
fn tc109_i02_space_object_returns_exactly_one_row() {
    let table = unique_name("tq_space_i02");
    create_fixture_table(&table);

    let out = run(&["space", &format!("{LIVE_DB}.{table}"), "--format", "csv"]);
    drop_fixture_table(&table);

    assert!(
        out.status.success(),
        "tq space {LIVE_DB}.{table} failed: stderr={}",
        stderr_str(&out)
    );
    let stdout = stdout_str(&out);
    assert_eq!(
        csv_data_row_count(&stdout),
        1,
        "expected exactly one data row for a single-object query: {stdout}"
    );
    assert!(
        stdout.to_lowercase().contains(&table.to_lowercase()),
        "the one row must be the requested object: {stdout}"
    );
}

// =============================================================================
// TC109-I03: `tq dbspace <db>` returns database-level metrics only
// =============================================================================

/// TC109-I03: `tq dbspace <db>` returns database-level perm/spool/temp
/// metrics only — no per-object rows (AC bullet 3).
#[test]
#[ignore]
fn tc109_i03_dbspace_returns_database_level_only() {
    // Ensure at least one object exists so a buggy implementation that
    // accidentally enumerates objects would be caught.
    let table = unique_name("tq_dbspace_i03");
    create_fixture_table(&table);

    let out = run(&["dbspace", LIVE_DB, "--format", "csv"]);
    drop_fixture_table(&table);

    assert!(
        out.status.success(),
        "tq dbspace {LIVE_DB} failed: stderr={}",
        stderr_str(&out)
    );
    let stdout = stdout_str(&out);
    assert_eq!(
        csv_data_row_count(&stdout),
        1,
        "dbspace must return exactly one (database-level) row, no object rows: {stdout}"
    );
}

// =============================================================================
// TC109-I04: `tq dbspace <db>.<obj>` fails with a clear, actionable error
// =============================================================================

/// TC109-I04: `tq dbspace <db>.<obj>` fails with a clear, actionable error
/// naming the qualified-name problem (AC bullet 4).
#[test]
#[ignore]
fn tc109_i04_dbspace_rejects_qualified_name() {
    let out = run(&["dbspace", &format!("{LIVE_DB}.some_object")]);

    assert!(
        !out.status.success(),
        "dbspace must reject a qualified db.object target"
    );
    assert_no_panic(&out);
    let stderr = stderr_str(&out).to_lowercase();
    assert!(
        stderr.contains("dbspace") || stderr.contains("database") || stderr.contains("object"),
        "error should name the qualified-name problem for dbspace: {stderr}"
    );
}

// =============================================================================
// TC109-I05: `tq dbspace <table>` (a real object, not a database) fails
// distinctly from the "unknown name" case
// =============================================================================

/// TC109-I05: `tq dbspace <table_name>` where `table_name` is a real object
/// that is not itself a database fails with a distinct "not a database"
/// error (Issue #54 body).
#[test]
#[ignore]
fn tc109_i05_dbspace_on_real_table_fails_distinctly() {
    let table = unique_name("tq_dbspace_i05");
    create_fixture_table(&table);

    // Pass the bare table name (unqualified) as if it were a database name.
    let out = run(&["dbspace", &table]);
    drop_fixture_table(&table);

    assert!(
        !out.status.success(),
        "dbspace on a real (non-database) object must fail"
    );
    assert_no_panic(&out);
    let stderr = stderr_str(&out).to_lowercase();
    // This must NOT be the generic "does not exist" not-found message —
    // the object *does* exist, it's just the wrong kind. We accept any
    // wording that signals a type mismatch rather than absence.
    assert!(
        !stderr.contains("does not exist") && !stderr.contains("not found"),
        "a real (non-database) object must not be reported as not-found: {stderr}"
    );
}

// =============================================================================
// TC109-I06 / I07: unknown database / unknown object → standard not-found
// (spelling suggestion descoped in Phase 2, see module doc comment)
// =============================================================================

/// TC109-I06: `tq space <unknown_db>` produces the project's standard
/// not-found error. No spelling suggestion is asserted — descoped in Phase 2
/// (no fuzzy-match helper exists in `src/`).
#[test]
#[ignore]
fn tc109_i06_space_unknown_database_standard_not_found() {
    let unknown = unique_name("tq_definitely_absent_db");
    let out = run(&["space", &unknown]);

    assert!(!out.status.success(), "unknown database must fail");
    assert_no_panic(&out);
    let stderr = stderr_str(&out).to_lowercase();
    assert!(
        stderr.contains("does not exist") || stderr.contains("not found") || stderr.contains("no such"),
        "expected the standard not-found error for an unknown database: {stderr}"
    );
}

/// TC109-I07: `tq space <db>.<unknown_obj>` produces the standard not-found
/// error for the object. No spelling suggestion asserted (descoped).
#[test]
#[ignore]
fn tc109_i07_space_unknown_object_standard_not_found() {
    let unknown = unique_name("tq_definitely_absent_obj");
    let out = run(&["space", &format!("{LIVE_DB}.{unknown}")]);

    assert!(!out.status.success(), "unknown object must fail");
    assert_no_panic(&out);
    let stderr = stderr_str(&out).to_lowercase();
    assert!(
        stderr.contains("does not exist") || stderr.contains("not found") || stderr.contains("no such"),
        "expected the standard not-found error for an unknown object: {stderr}"
    );
}

// =============================================================================
// TC109-I08 - I11: all four output formats
// =============================================================================

/// TC109-I08: `--format json` produces valid, parseable JSON.
#[test]
#[ignore]
fn tc109_i08_format_json_is_valid() {
    let out = run(&["space", LIVE_DB, "--format", "json"]);
    assert!(out.status.success(), "stderr={}", stderr_str(&out));
    let _: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout must be valid JSON");
}

/// TC109-I09: `--format csv` produces well-formed CSV (a header line plus at
/// least one data line, comma-separated).
#[test]
#[ignore]
fn tc109_i09_format_csv_is_well_formed() {
    let out = run(&["space", LIVE_DB, "--format", "csv"]);
    assert!(out.status.success(), "stderr={}", stderr_str(&out));
    let stdout = stdout_str(&out);
    let mut lines = stdout.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next().expect("CSV must have a header line");
    assert!(header.contains(','), "CSV header must be comma-separated: {header}");
}

/// TC109-I10: `--format markdown` produces a valid markdown table (header
/// row followed by a `---`-style separator row, both pipe-delimited).
#[test]
#[ignore]
fn tc109_i10_format_markdown_is_valid_table() {
    let out = run(&["space", LIVE_DB, "--format", "markdown"]);
    assert!(out.status.success(), "stderr={}", stderr_str(&out));
    let stdout = stdout_str(&out);
    let mut lines = stdout.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next().expect("markdown output must have a header row");
    let separator = lines.next().expect("markdown output must have a separator row");
    assert!(header.contains('|'), "header row must be pipe-delimited: {header}");
    assert!(
        separator.contains('-') && separator.contains('|'),
        "separator row must be a markdown table rule: {separator}"
    );
}

/// TC109-I11: `--format table` (the default) produces human-readable table
/// output — i.e. NOT machine-format output (not raw JSON/CSV-shaped).
#[test]
#[ignore]
fn tc109_i11_default_format_is_human_readable_table() {
    let out = run(&["space", LIVE_DB]);
    assert!(out.status.success(), "stderr={}", stderr_str(&out));
    let stdout = stdout_str(&out);
    let trimmed = stdout.trim_start();
    assert!(
        !trimmed.starts_with('{') && !trimmed.starts_with('['),
        "default format must not look like JSON: {stdout}"
    );
}

// =============================================================================
// TC109-I12: database with zero contained objects → header row only
// =============================================================================

/// TC109-I12: A database with zero contained objects returns the header row
/// only, with no crash. Requires the test user to have `CREATE DATABASE`
/// privilege; if that privilege is absent this test fails loudly at setup
/// (not silently skipped), which is the correct signal for the execution
/// loop to either grant the privilege or substitute a known-empty fixture.
#[test]
#[ignore]
fn tc109_i12_empty_database_returns_header_only_no_crash() {
    let db_name = unique_name("tq_empty_db_i12");
    let create = run(&[
        "query",
        &format!("CREATE DATABASE {db_name} FROM {LIVE_DB} AS PERM = 1000000"),
    ]);
    assert!(
        create.status.success(),
        "setup: CREATE DATABASE {db_name} failed (requires CREATE DATABASE privilege on {LIVE_DB}): stderr={}",
        stderr_str(&create)
    );

    let out = run(&["space", &db_name, "--format", "csv"]);
    let _ = run(&["query", &format!("DROP DATABASE {db_name}")]);

    assert!(out.status.success(), "stderr={}", stderr_str(&out));
    assert_no_panic(&out);
    let stdout = stdout_str(&out);
    assert_eq!(
        csv_data_row_count(&stdout),
        1,
        "an empty database must still produce exactly its own header row: {stdout}"
    );
}

// =============================================================================
// TC109-I13: skew percentages are always in [0, 100] or explicitly absent
// =============================================================================

/// TC109-I13: Live skew % values are within `[0, 100]` or explicitly
/// null/absent — never NaN or negative.
#[test]
#[ignore]
fn tc109_i13_skew_percentages_in_range_or_absent() {
    let out = run(&["space", LIVE_DB, "--format", "json"]);
    assert!(out.status.success(), "stderr={}", stderr_str(&out));
    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout must be valid JSON");

    fn check_numbers(value: &serde_json::Value, key_hint: &str) {
        match value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    check_numbers(v, k);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    check_numbers(item, key_hint);
                }
            }
            serde_json::Value::Number(n) => {
                let lower = key_hint.to_lowercase();
                if lower.contains("skew") || lower.contains("pct") || lower.contains("percent") {
                    let f = n.as_f64().expect("skew/pct field must be a finite number");
                    assert!(!f.is_nan(), "skew/pct value must not be NaN (key: {key_hint})");
                    assert!(
                        (0.0..=100.0).contains(&f),
                        "skew/pct value {f} out of [0, 100] range (key: {key_hint})"
                    );
                }
            }
            _ => {}
        }
    }
    check_numbers(&json, "");
}

// =============================================================================
// TC109-I14: live execution confirms DBC.DiskSpaceV / DBC.TableSizeV queries
// run without column/object errors (validates the SQL from #54)
// =============================================================================

/// TC109-I14: A live run against `DBC.DiskSpaceV` (database-level) and
/// `DBC.TableSizeV` (object-level) completes without any SQL column/object
/// error, proving the queries in issue #54 are valid against the real
/// catalog (AC bullet 13 — no fabricated DBC objects/columns).
#[test]
#[ignore]
fn tc109_i14_live_sql_runs_without_column_or_object_errors() {
    let table = unique_name("tq_space_i14");
    create_fixture_table(&table);

    let db_level = run(&["dbspace", LIVE_DB]);
    let obj_level = run(&["space", &format!("{LIVE_DB}.{table}")]);
    drop_fixture_table(&table);

    for (label, out) in [("dbspace (DiskSpaceV)", &db_level), ("space object (TableSizeV)", &obj_level)] {
        assert!(out.status.success(), "{label} failed: stderr={}", stderr_str(out));
        let stderr = stderr_str(out).to_lowercase();
        assert!(
            !stderr.contains("column") && !stderr.contains("3707") && !stderr.contains("does not exist"),
            "{label} must not surface a SQL column/object error: {stderr}"
        );
    }
}
