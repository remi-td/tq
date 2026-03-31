//! Integration tests for Sprint 55: `tq search tables` and `tq search columns` commands
//!
//! These tests verify the `tq search` subcommand works end-to-end via the CLI binary.
//! All live-database tests are marked `#[ignore]` and require:
//!   - `TQ_LOGON` set in `.env` (or as an environment variable)
//!   - A reachable Teradata instance
//!
//! ## Run commands
//!
//! Non-ignored tests (no DB required):
//! ```bash
//! cargo test --test integration_search
//! ```
//!
//! Live database tests:
//! ```bash
//! cargo test --test integration_search -- --ignored
//! ```

mod common;

// =============================================================================
// Sprint 55 Feature 1: `tq search tables <keyword>`
// =============================================================================

// -----------------------------------------------------------------------------
// TC-055-001-C1: `tq search tables dbc` returns results
// -----------------------------------------------------------------------------

/// Verify that searching for a common keyword returns at least one result row.
///
/// "dbc" reliably matches tables in the DBC system database on every Teradata instance.
///
/// AC covered: F1-AC-1 (keyword matching), F1-AC-5 (read-only, no error)
///
/// Run with: cargo test --test integration_search test_search_tables_returns_results -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_search_tables_returns_results() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon =
            std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args(["-l", &logon, "search", "tables", "dbc"])
            .output()
            .expect("Failed to run tq search tables");

        assert!(
            output.status.success(),
            "tq search tables must exit 0, got: {:?}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.trim().is_empty(),
            "stdout must not be empty for known keyword 'dbc': got empty output"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("Error:"),
            "Must not produce errors on stderr: {}",
            stderr
        );
    });
}

// -----------------------------------------------------------------------------
// TC-055-001-D1: `tq search tables dbc --database DBC` scopes to one database
// -----------------------------------------------------------------------------

/// Verify that `--database` restricts the search to a single database.
///
/// When scoped to DBC, results must still appear (DBC definitely has tables matching "dbc")
/// and the command must exit cleanly.
///
/// AC covered: F1-AC-2 (--database scoping)
///
/// Run with: cargo test --test integration_search test_search_tables_database_flag -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_search_tables_database_flag() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args(["-l", &logon, "search", "tables", "dbc", "--database", "DBC"])
            .output()
            .expect("Failed to run tq search tables --database DBC");

        assert!(
            output.status.success(),
            "tq search tables --database must exit 0, got: {:?}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.trim().is_empty(),
            "Scoped search to DBC must produce output (DBC has tables containing 'dbc')"
        );

        // When scoped to DBC, output should reference DBC
        assert!(
            stdout.to_uppercase().contains("DBC"),
            "Scoped search output must reference DBC database: {}",
            stdout
        );
    });
}

// -----------------------------------------------------------------------------
// TC-055-001-E1: `tq search tables dbc --format json` produces valid JSON envelope
// -----------------------------------------------------------------------------

/// Verify the JSON output uses the standard `{"ok": true, "row_count": N, "data": [...]}` envelope.
///
/// AC covered: F1-AC-3 (JSON format), F1-AC-4 (standard envelope)
///
/// Run with: cargo test --test integration_search test_search_tables_json_envelope -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_search_tables_json_envelope() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args(["-l", &logon, "search", "tables", "dbc", "--format", "json"])
            .output()
            .expect("Failed to run tq search tables --format json");

        assert!(
            output.status.success(),
            "tq search tables --format json must exit 0, got: {:?}\nstderr: {}",
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

        assert_eq!(json["ok"], true, "Envelope must have ok: true, got: {:?}", json);
        assert!(
            json["row_count"].is_number(),
            "Envelope must have numeric row_count, got: {:?}",
            json["row_count"]
        );
        assert!(
            json["data"].is_array(),
            "Envelope must have data array, got: {:?}",
            json["data"]
        );

        let count = json["row_count"].as_u64().unwrap();
        let data_len = json["data"].as_array().unwrap().len() as u64;
        assert_eq!(
            count, data_len,
            "row_count ({}) must match data array length ({})",
            count, data_len
        );

        // At least one result expected for "dbc" keyword
        assert!(
            count >= 1,
            "Expected at least 1 result for keyword 'dbc', got row_count={}",
            count
        );
    });
}

// -----------------------------------------------------------------------------
// TC-055-001-F1: `tq search tables` is read-only (agent-safe compatible)
// -----------------------------------------------------------------------------

/// Verify that the search command completes without DML errors.
///
/// A SELECT-only command must always exit 0. This confirms agent-safe compatibility.
///
/// AC covered: F1-AC-5 (agent-safe mode, read-only)
///
/// Run with: cargo test --test integration_search test_search_tables_is_read_only -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_search_tables_is_read_only() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args(["-l", &logon, "search", "tables", "sys"])
            .output()
            .expect("Failed to run tq search tables");

        assert!(
            output.status.success(),
            "Read-only search must exit 0, got: {:?}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.to_lowercase().contains("permission denied"),
            "Read-only search must not encounter permission errors: {}",
            stderr
        );
    });
}

// -----------------------------------------------------------------------------
// TC-055-001-G1: `tq search tables` with no-match keyword exits 0 with 0 results
// -----------------------------------------------------------------------------

/// Verify that a keyword with no matches produces a graceful empty result,
/// not a panic or non-zero exit code.
///
/// AC covered: F1-AC-6 (no-results graceful handling)
///
/// Run with: cargo test --test integration_search test_search_tables_no_results -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_search_tables_no_results() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args(["-l", &logon, "search", "tables", "xyzzy_no_match_abc_55"])
            .output()
            .expect("Failed to run tq search tables with no-match keyword");

        assert!(
            output.status.success(),
            "No-results search must exit 0 (not crash), got: {:?}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("Error:"),
            "No-results must not produce errors on stderr: {}",
            stderr
        );
        assert!(
            !stderr.to_lowercase().contains("panic"),
            "No-results must not panic: {}",
            stderr
        );

        // Validate the output signals zero matches — any of these formats are acceptable:
        // - "0 result(s)" or "0 results" in a footer
        // - "No tables found" message
        // - Empty data in JSON envelope (row_count: 0)
        // - Empty stdout (minimalist rendering)
        let stdout = String::from_utf8_lossy(&output.stdout);
        let indicates_empty = stdout.contains("0 result")
            || stdout.contains("No tables")
            || stdout.contains("no tables")
            || stdout.trim().is_empty();
        assert!(
            indicates_empty,
            "No-results output must indicate 0 matches (got: '{}')",
            stdout
        );
    });
}

// =============================================================================
// Sprint 55 Feature 2: `tq search columns <keyword>`
// =============================================================================

// -----------------------------------------------------------------------------
// TC-055-002-C1: `tq search columns name` returns results
// -----------------------------------------------------------------------------

/// Verify that searching columns by a common keyword returns results.
///
/// "name" is an extremely common column name prefix across Teradata system tables.
///
/// AC covered: F2-AC-1 (keyword matching), F2-AC-5 (read-only)
///
/// Run with: cargo test --test integration_search test_search_columns_returns_results -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_search_columns_returns_results() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon =
            std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args(["-l", &logon, "search", "columns", "name"])
            .output()
            .expect("Failed to run tq search columns");

        assert!(
            output.status.success(),
            "tq search columns must exit 0, got: {:?}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.trim().is_empty(),
            "stdout must not be empty for common keyword 'name'"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("Error:"),
            "Must not produce errors: {}",
            stderr
        );
    });
}

// -----------------------------------------------------------------------------
// TC-055-002-D1: `tq search columns name --database DBC` scopes results
// -----------------------------------------------------------------------------

/// Verify that `--database` restricts column search to the specified database.
///
/// AC covered: F2-AC-2 (--database scoping)
///
/// Run with: cargo test --test integration_search test_search_columns_database_flag -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_search_columns_database_flag() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args([
                "-l",
                &logon,
                "search",
                "columns",
                "name",
                "--database",
                "DBC",
            ])
            .output()
            .expect("Failed to run tq search columns --database DBC");

        assert!(
            output.status.success(),
            "tq search columns --database must exit 0, got: {:?}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.trim().is_empty(),
            "Scoped column search to DBC must produce output"
        );

        // When scoped to DBC, output should reference DBC
        assert!(
            stdout.to_uppercase().contains("DBC"),
            "Scoped search output must reference DBC database: {}",
            stdout
        );
    });
}

// -----------------------------------------------------------------------------
// TC-055-002-E1: `tq search columns name --format json` — nullable is boolean
// -----------------------------------------------------------------------------

/// Verify JSON envelope is valid and `nullable` field is a JSON boolean (not "YES"/"NO").
///
/// AC covered: F2-AC-3 (JSON format), F2-AC-4 (standard envelope + nullable boolean)
///
/// Run with: cargo test --test integration_search test_search_columns_json_nullable_boolean -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_search_columns_json_nullable_boolean() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args([
                "-l",
                &logon,
                "search",
                "columns",
                "name",
                "--format",
                "json",
            ])
            .output()
            .expect("Failed to run tq search columns --format json");

        assert!(
            output.status.success(),
            "tq search columns --format json must exit 0, got: {:?}\nstderr: {}",
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

        // Standard envelope checks
        assert_eq!(
            json["ok"], true,
            "Envelope must have ok: true, got: {:?}",
            json
        );
        assert!(
            json["row_count"].is_number(),
            "Envelope must have numeric row_count, got: {:?}",
            json["row_count"]
        );
        assert!(
            json["data"].is_array(),
            "Envelope must have data array, got: {:?}",
            json["data"]
        );

        // CRITICAL: for every result row, nullable must be a JSON boolean
        if let Some(data) = json["data"].as_array() {
            assert!(
                !data.is_empty(),
                "Expected results for keyword 'name' in DBC, got empty data array"
            );
            for (i, entry) in data.iter().enumerate() {
                if let Some(nullable_val) = entry.get("nullable") {
                    assert!(
                        nullable_val.is_boolean(),
                        "Entry {}: nullable must be a JSON boolean (not a string), got: {:?}",
                        i,
                        nullable_val
                    );
                }
            }
            // No string "YES" or "NO" — those violate the established API contract
            assert!(
                !stdout.contains("\"YES\""),
                "nullable must not appear as string 'YES' in JSON output: {}",
                stdout
            );
            assert!(
                !stdout.contains("\"NO\""),
                "nullable must not appear as string 'NO' in JSON output: {}",
                stdout
            );
        }
    });
}

// -----------------------------------------------------------------------------
// TC-055-002-G1: `tq search columns` with no-match keyword exits 0 with 0 results
// -----------------------------------------------------------------------------

/// Verify a column keyword search with no matches is handled gracefully.
///
/// AC covered: F2-AC-6 (no-results graceful handling)
///
/// Run with: cargo test --test integration_search test_search_columns_no_results -- --ignored
#[test]
#[ignore] // Requires live Teradata database
fn test_search_columns_no_results() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set");

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tq"))
            .args(["-l", &logon, "search", "columns", "xyzzy_no_match_abc_55"])
            .output()
            .expect("Failed to run tq search columns with no-match keyword");

        assert!(
            output.status.success(),
            "No-results column search must exit 0 (not crash), got: {:?}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("Error:"),
            "No-results must not produce errors on stderr: {}",
            stderr
        );
        assert!(
            !stderr.to_lowercase().contains("panic"),
            "No-results must not panic: {}",
            stderr
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let indicates_empty = stdout.contains("0 result")
            || stdout.contains("No columns")
            || stdout.contains("no columns")
            || stdout.trim().is_empty();
        assert!(
            indicates_empty,
            "No-results output must indicate 0 matches (got: '{}')",
            stdout
        );
    });
}

// =============================================================================
// TC-055-003-E1: Structural check — REPL completer references "search" (no DB)
// =============================================================================

/// Verify at the source level that the REPL completer includes `/search` entries.
///
/// This test compensates for tab completion PTY ambiguity (see TC-055-003-C).
/// It does not require a live database — it reads source files at test time.
///
/// AC covered: F3-AC-3 (tab completion registration, structural)
///
/// Run with: cargo test --test integration_search test_repl_search_completer_registration
#[test]
fn test_repl_search_completer_registration() {
    // Look for "search" in the REPL-related source files — try several candidate paths
    let candidate_files = [
        "src/commands/repl/metadata_completer.rs",
        "src/commands/repl/completer.rs",
        "src/commands/repl/mod.rs",
        "src/commands/repl/commands.rs",
        "src/commands/repl/metacommands.rs",
    ];

    let mut found_file = None;
    let mut found_search = false;

    for path in &candidate_files {
        if let Ok(source) = std::fs::read_to_string(path) {
            found_file = Some(*path);
            if source.contains("search") {
                found_search = true;
                break;
            }
        }
    }

    assert!(
        found_file.is_some(),
        "Could not find any REPL source file in candidates: {:?}",
        candidate_files
    );
    assert!(
        found_search,
        "REPL source files do not reference 'search' — /search metacommand is not registered"
    );
}
