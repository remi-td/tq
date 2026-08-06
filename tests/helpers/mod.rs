//! Shared test helpers for tq's `tests/` integration suite.
//!
//! Include into a test binary with:
//! ```ignore
//! #[path = "helpers/mod.rs"]
//! mod helpers;
//! ```
//! (mirrors the existing `#[path = "helpers/pager_fixtures.rs"]` pattern used
//! by `tests/pager_dimensional_tests.rs`).
//!
//! Sprint 76: introduced for the space-analysis (#54) and monitoring
//! thresholds/colors (#23) test suites. Two things live here:
//!
//! 1. ANSI-escape detection (`contains_ansi` / `assert_no_ansi` /
//!    `assert_has_ansi`) — no existing helper scanned raw output for ANSI
//!    bytes; every color-suppression (`NO_COLOR`, `--color never`, piped) and
//!    color-emission (`--color always` under Warning/Critical) test needs it.
//! 2. `create_user_config` / `create_project_config` — promoted out of
//!    `tests/integration_project_config_edge_cases.rs`, where they were
//!    private, so `tests/integration_monitoring.rs` can reuse them verbatim
//!    for `[monitoring.thresholds]` / `[monitoring.colors]` fixtures instead
//!    of duplicating file-writing boilerplate.
//!
//! Not every test file needs every helper; `#[allow(dead_code)]` keeps the
//! per-binary "unused function" warning silent for whichever half a given
//! file doesn't use — the same tolerance already applied to
//! `tests/common/mod.rs`'s `pty_harness` re-export.

#![allow(dead_code)]

use std::fs;
use std::path::Path;

/// Returns `true` if `bytes` contains an ANSI CSI escape sequence
/// (`ESC [`, i.e. the two bytes `0x1b 0x5b`).
///
/// This is a raw byte scan rather than a `str` search so it also works on
/// subprocess stdout/stderr captured as `Vec<u8>` before (or instead of)
/// UTF-8 validation, and cannot be fooled by a lossy `String` conversion
/// mangling an escape sequence.
pub fn contains_ansi(bytes: &[u8]) -> bool {
    bytes.windows(2).any(|w| w == [0x1b, b'['])
}

/// Assert that `bytes` contains zero ANSI escape sequences.
///
/// Use for `--color never`, `NO_COLOR=1`, piped/non-TTY stdout, and for
/// every structured format (`json`, `csv`, `markdown`) regardless of the
/// active color mode — those formats must never carry ANSI even when color
/// is "always" for the same command's table output.
pub fn assert_no_ansi(bytes: &[u8]) {
    assert!(
        !contains_ansi(bytes),
        "unexpected ANSI escape sequence found in output: {:?}",
        String::from_utf8_lossy(bytes)
    );
}

/// Assert that `bytes` contains at least one ANSI escape sequence.
///
/// Use for `--color always` assertions against output that should contain a
/// severity-styled (Warning/Critical) value.
pub fn assert_has_ansi(bytes: &[u8]) {
    assert!(
        contains_ansi(bytes),
        "expected an ANSI escape sequence, found none in output: {:?}",
        String::from_utf8_lossy(bytes)
    );
}

/// Write `content` to `<home_dir>/.tq/config.toml`, creating the directory
/// tree if needed.
///
/// Promoted (Sprint 76) from the private helper of the same name and
/// signature duplicated in `tests/integration_project_config_edge_cases.rs`
/// and `tests/integration_profile_resolution.rs`.
pub fn create_user_config(home_dir: &Path, content: &str) {
    let config_dir = home_dir.join(".tq");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.toml");
    fs::write(&config_path, content).unwrap();
}

/// Write `content` to `<dir>/.tq.toml`.
///
/// Companion to [`create_user_config`] for project-level config fixtures.
pub fn create_project_config(dir: &Path, content: &str) {
    let config_path = dir.join(".tq.toml");
    fs::write(&config_path, content).unwrap();
}
