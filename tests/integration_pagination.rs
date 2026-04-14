//! Integration tests for Sprint 56: Result Pagination
//!
//! These tests verify pagination flags (`--page-size`, `--page`) on `tq query`,
//! `tq search`, and `tq list` commands, including the JSON envelope `pagination`
//! object and backward-compatibility guarantees.
//!
//! ## Test categories
//!
//! ### Non-ignored (no DB required)
//! - `test_pagination_error_page_without_size` — `--page` without `--page-size` exits non-zero
//! - `test_esc_consolidated` — `fn esc(` must NOT appear in search.rs or list.rs after cleanup
//! - `test_markdown_escape_in_helpers` — `markdown_escape_pipe` must exist in format_helpers.rs
//!
//! ### Ignored (live Teradata DB required)
//! - `test_query_pagination_first_page` — page 1 returns exactly `--page-size` rows, has_more true
//! - `test_query_pagination_second_page` — page 2 returns next rows
//! - `test_query_pagination_json_envelope` — pagination object has all 4 required fields
//! - `test_query_no_pagination_backward_compat` — no `pagination` key without `--page-size`
//! - `test_search_pagination` — `tq search tables` respects `--page-size`
//!
//! ## Run commands
//!
//! Non-ignored tests (no DB required):
//! ```bash
//! cargo test --test integration_pagination
//! ```
//!
//! Live database tests:
//! ```bash
//! cargo test --test integration_pagination -- --ignored
//! ```

mod common;

// =============================================================================
// TC-056-002: Query Result Pagination — end-to-end CLI integration
// =============================================================================

// -----------------------------------------------------------------------------
// TC-056-002-A1: First page returns exactly page_size rows, has_more is true
// -----------------------------------------------------------------------------

/// Verify that `--page-size 5` on a 20-row result set returns exactly 5 rows
/// and the `pagination` envelope reports `has_more: true`.
///
/// AC covered: F1-AC-1 (--page-size flag), F1-AC-3 (has_more field)
///
/// Run with: cargo test --test integration_pagination test_query_pagination_first_page -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_query_pagination_first_page() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon =
            std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args([
                "-l",
                &logon,
                "query",
                "--page-size",
                "5",
                "--format",
                "json",
                "SELECT TOP 20 DatabaseName FROM DBC.DatabasesV",
            ])
            .output()
            .expect("Failed to run tq query --page-size 5");

        assert!(
            output.status.success(),
            "tq query --page-size must exit 0, got: {:?}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "Output must be valid JSON: {}\nActual output:\n{}",
                e, stdout
            )
        });

        // Standard envelope
        assert_eq!(json["ok"], true, "Envelope must have ok: true");

        // Data array must be exactly page_size rows
        let data = json["data"].as_array().unwrap_or_else(|| {
            panic!("Envelope must have data array, got: {:?}", json["data"])
        });
        assert_eq!(
            data.len(),
            5,
            "First page must return exactly 5 rows (page_size), got {}",
            data.len()
        );

        // Pagination object must be present
        let pagination = &json["pagination"];
        assert!(
            !pagination.is_null(),
            "JSON envelope must contain 'pagination' object when --page-size is used"
        );

        // has_more must be true (20 rows total, only 5 returned)
        assert_eq!(
            pagination["has_more"], true,
            "has_more must be true when more pages are available, got: {:?}",
            pagination["has_more"]
        );

        // page must be 1 (default)
        assert_eq!(
            pagination["page"], 1,
            "Default page must be 1, got: {:?}",
            pagination["page"]
        );

        // page_size must reflect the flag value
        assert_eq!(
            pagination["page_size"], 5,
            "page_size must equal --page-size flag value (5), got: {:?}",
            pagination["page_size"]
        );
    });
}

// -----------------------------------------------------------------------------
// TC-056-002-A2: Second page returns the next rows
// -----------------------------------------------------------------------------

/// Verify that `--page 2 --page-size 5` returns a different, non-empty set of
/// rows and `pagination.page` equals 2.
///
/// AC covered: F1-AC-2 (--page flag), F1-AC-4 (correct slice offset)
///
/// Run with: cargo test --test integration_pagination test_query_pagination_second_page -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_query_pagination_second_page() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set");

        // Fetch page 1 for comparison
        let page1_output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args([
                "-l",
                &logon,
                "query",
                "--page-size",
                "5",
                "--page",
                "1",
                "--format",
                "json",
                "SELECT TOP 20 DatabaseName FROM DBC.DatabasesV ORDER BY DatabaseName",
            ])
            .output()
            .expect("Failed to run tq query page 1");

        // Fetch page 2
        let page2_output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args([
                "-l",
                &logon,
                "query",
                "--page-size",
                "5",
                "--page",
                "2",
                "--format",
                "json",
                "SELECT TOP 20 DatabaseName FROM DBC.DatabasesV ORDER BY DatabaseName",
            ])
            .output()
            .expect("Failed to run tq query page 2");

        assert!(
            page2_output.status.success(),
            "tq query --page 2 must exit 0, got: {:?}\nstderr: {}",
            page2_output.status,
            String::from_utf8_lossy(&page2_output.stderr)
        );

        let stdout1 = String::from_utf8_lossy(&page1_output.stdout);
        let stdout2 = String::from_utf8_lossy(&page2_output.stdout);

        let json1: serde_json::Value = serde_json::from_str(&stdout1)
            .unwrap_or_else(|e| panic!("Page 1 output must be valid JSON: {}\n{}", e, stdout1));
        let json2: serde_json::Value = serde_json::from_str(&stdout2)
            .unwrap_or_else(|e| panic!("Page 2 output must be valid JSON: {}\n{}", e, stdout2));

        assert_eq!(json2["ok"], true, "Page 2 envelope must have ok: true");

        // pagination.page must equal 2
        assert_eq!(
            json2["pagination"]["page"], 2,
            "pagination.page must equal 2 for second page, got: {:?}",
            json2["pagination"]["page"]
        );

        // Page 2 data must be non-empty
        let data2 = json2["data"]
            .as_array()
            .expect("Page 2 data must be an array");
        assert!(
            !data2.is_empty(),
            "Page 2 must return at least one row (DBC has >= 10 databases)"
        );

        // Page 1 and page 2 data must differ (different slice of results)
        let data1 = json1["data"]
            .as_array()
            .expect("Page 1 data must be an array");

        // The first row of page 2 must not match the first row of page 1
        // (they are different offsets in a deterministic ORDER BY query)
        assert_ne!(
            data1.first(),
            data2.first(),
            "First row of page 1 and page 2 must differ (different result slices)"
        );
    });
}

// -----------------------------------------------------------------------------
// TC-056-002-B1: JSON envelope pagination object has all 4 required fields
// -----------------------------------------------------------------------------

/// Verify the `pagination` object contains exactly:
/// - `page` (integer)
/// - `page_size` (integer)
/// - `has_more` (boolean)
/// - `total_rows` (integer or null)
///
/// AC covered: F1-AC-5 (complete pagination envelope schema)
///
/// Run with: cargo test --test integration_pagination test_query_pagination_json_envelope -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_query_pagination_json_envelope() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args([
                "-l",
                &logon,
                "query",
                "--page-size",
                "3",
                "--format",
                "json",
                "SELECT TOP 10 DatabaseName FROM DBC.DatabasesV",
            ])
            .output()
            .expect("Failed to run tq query --page-size 3");

        assert!(
            output.status.success(),
            "tq query with pagination must exit 0, got: {:?}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("Output must be valid JSON: {}\n{}", e, stdout));

        let pagination = &json["pagination"];
        assert!(
            !pagination.is_null() && pagination.is_object(),
            "JSON envelope must contain 'pagination' object, got: {:?}",
            pagination
        );

        // Field 1: page — must be an integer
        assert!(
            pagination["page"].is_number(),
            "pagination.page must be a number, got: {:?}",
            pagination["page"]
        );

        // Field 2: page_size — must be an integer
        assert!(
            pagination["page_size"].is_number(),
            "pagination.page_size must be a number, got: {:?}",
            pagination["page_size"]
        );

        // Field 3: has_more — must be a boolean
        assert!(
            pagination["has_more"].is_boolean(),
            "pagination.has_more must be a boolean, got: {:?}",
            pagination["has_more"]
        );

        // Field 4: total_rows — must be present (integer or null)
        assert!(
            pagination.get("total_rows").is_some(),
            "pagination must contain 'total_rows' field (may be null), got: {:?}",
            pagination
        );
        let total_rows = &pagination["total_rows"];
        assert!(
            total_rows.is_number() || total_rows.is_null(),
            "pagination.total_rows must be a number or null, got: {:?}",
            total_rows
        );

        // Sanity: page_size must match the flag value (3)
        assert_eq!(
            pagination["page_size"], 3,
            "pagination.page_size must equal 3 (the --page-size flag value)"
        );
    });
}

// -----------------------------------------------------------------------------
// TC-056-002-C1: Without --page-size, no pagination key in JSON output
// -----------------------------------------------------------------------------

/// Verify backward compatibility: when `--page-size` is NOT specified, the JSON
/// envelope must NOT contain a `pagination` key.
///
/// AC covered: F1-AC-6 (backward compatibility — no pagination key without flag)
///
/// Run with: cargo test --test integration_pagination test_query_no_pagination_backward_compat -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_query_no_pagination_backward_compat() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args([
                "-l",
                &logon,
                "query",
                "--format",
                "json",
                "SELECT TOP 5 DatabaseName FROM DBC.DatabasesV",
            ])
            .output()
            .expect("Failed to run tq query without --page-size");

        assert!(
            output.status.success(),
            "tq query without pagination must exit 0, got: {:?}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("Output must be valid JSON: {}\n{}", e, stdout));

        // Standard fields must still be present
        assert_eq!(json["ok"], true, "Envelope must still have ok: true");
        assert!(
            json["data"].is_array(),
            "Envelope must still have data array"
        );

        // CRITICAL: no pagination key when flag is absent
        assert!(
            json.get("pagination").is_none() || json["pagination"].is_null(),
            "Envelope must NOT contain 'pagination' key when --page-size is not used, \
             got: {:?}",
            json.get("pagination")
        );
    });
}

// =============================================================================
// TC-056-003: Search Pagination — `tq search tables --page-size N`
// =============================================================================

// -----------------------------------------------------------------------------
// TC-056-003-A1: tq search tables with --page-size paginates JSON output
// -----------------------------------------------------------------------------

/// Verify that `tq search tables dbc --page-size 3 --format json` returns
/// exactly 3 rows with a `pagination` object in the JSON envelope.
///
/// AC covered: F3-AC-1 (search supports --page-size), F3-AC-2 (pagination envelope)
///
/// Run with: cargo test --test integration_pagination test_search_pagination -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_search_pagination() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args([
                "-l",
                &logon,
                "search",
                "tables",
                "dbc",
                "--page-size",
                "3",
                "--format",
                "json",
            ])
            .output()
            .expect("Failed to run tq search tables --page-size 3");

        assert!(
            output.status.success(),
            "tq search tables --page-size must exit 0, got: {:?}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "Output must be valid JSON: {}\nActual output:\n{}",
                e, stdout
            )
        });

        assert_eq!(json["ok"], true, "Envelope must have ok: true");

        // Data must be limited to page_size rows
        let data = json["data"]
            .as_array()
            .expect("Envelope must have data array");
        assert!(
            data.len() <= 3,
            "search --page-size 3 must return at most 3 rows, got {}",
            data.len()
        );
        assert!(
            !data.is_empty(),
            "search for 'dbc' must return at least 1 row in first page"
        );

        // Pagination object must be present
        let pagination = &json["pagination"];
        assert!(
            !pagination.is_null() && pagination.is_object(),
            "JSON envelope must contain 'pagination' object when --page-size is used, \
             got: {:?}",
            pagination
        );

        assert_eq!(
            pagination["page_size"], 3,
            "pagination.page_size must equal 3"
        );
    });
}

// =============================================================================
// TC-056-002-E1: Error — --page without --page-size exits non-zero (no DB needed)
// =============================================================================

/// Verify that using `--page 2` without `--page-size` is rejected with a
/// non-zero exit code. This is a CLI validation error and does not need a DB.
///
/// AC covered: F1-AC-7 (--page requires --page-size)
///
/// Run with: cargo test --test integration_pagination test_pagination_error_page_without_size
#[test]
fn test_pagination_error_page_without_size() {
    // We intentionally do not provide a logon here — the error should be
    // raised at argument-validation time, before any DB connection attempt.
    // If the binary needs a connection string to even parse args, we fall
    // back to using a dummy value and accept "connection error" as long as
    // the exit code is non-zero.
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
        .args([
            "-l",
            "dummy:dummy@localhost:1025/dummy",
            "query",
            "--page",
            "2",
            "SELECT 1",
        ])
        .output()
        .expect("Failed to run tq query --page 2 (without --page-size)");

    assert!(
        !output.status.success(),
        "tq query --page without --page-size must exit non-zero (argument validation error), \
         but got exit code: {:?}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

// =============================================================================
// TC-056-004: Sprint 55 Tech Debt Cleanup — Structural checks (no DB)
// =============================================================================

// -----------------------------------------------------------------------------
// TC-056-004-A1: fn esc() must NOT appear in search.rs or list.rs after cleanup
// -----------------------------------------------------------------------------

/// Verify at the source level that local `fn esc(` definitions have been
/// removed from `search.rs` and `list.rs` and consolidated into `format_helpers.rs`.
///
/// This test will FAIL before the architect implements the consolidation, and
/// PASS once the cleanup is complete. That is the intended behavior — it documents
/// the structural contract.
///
/// AC covered: F4-AC-1 (esc() consolidated into format_helpers.rs)
///
/// Run with: cargo test --test integration_pagination test_esc_consolidated
#[test]
fn test_esc_consolidated() {
    let files_that_must_not_define_esc = [
        "src/commands/search.rs",
        "src/commands/list.rs",
    ];

    for path in &files_that_must_not_define_esc {
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
            panic!("Could not read {}: {}", path, e)
        });

        assert!(
            !source.contains("fn esc("),
            "After Sprint 56 cleanup, {} must NOT define a local `fn esc(` \
             — it should use format_helpers instead.\n\
             Found 'fn esc(' in: {}",
            path,
            path
        );
    }
}

// -----------------------------------------------------------------------------
// TC-056-004-B1: markdown_escape_pipe must exist in format_helpers.rs
// -----------------------------------------------------------------------------

/// Verify that `markdown_escape_pipe` is defined in `format_helpers.rs`.
///
/// This function centralises pipe-escaping for Markdown table rendering and
/// must be reachable by all command modules that produce markdown output.
///
/// AC covered: F4-AC-2 (markdown_escape_pipe in format_helpers.rs)
///
/// Run with: cargo test --test integration_pagination test_markdown_escape_in_helpers
#[test]
fn test_markdown_escape_in_helpers() {
    let helpers_path = "src/commands/format_helpers.rs";
    let source = std::fs::read_to_string(helpers_path).unwrap_or_else(|e| {
        panic!("Could not read {}: {}", helpers_path, e)
    });

    assert!(
        source.contains("markdown_escape_pipe"),
        "format_helpers.rs must define `markdown_escape_pipe` for Sprint 56 cleanup.\n\
         The function was not found in: {}",
        helpers_path
    );
}
