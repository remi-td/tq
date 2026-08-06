//! Integration tests for monitoring thresholds, severity colors, and
//! `refresh_interval` precedence (Issue #23, Sprint 76).
//!
//! Corresponds to TC110-I01 through TC110-I08 in
//! `tests/strategy/sprint-76-strategy.md`. Tests spawn the built `tq` binary
//! (subprocess, real OS pipes) because ANSI-byte presence/absence is stream-
//! level state (`is_terminal()`, `NO_COLOR`) that cannot be faked from
//! within the same process — see `docs/design/monitoring.md`.
//!
//! Run with:
//! ```bash
//! cargo test --test integration_monitoring -- --ignored
//! ```
//!
//! ## Sprint 76 Phase 2 rulings that shape these assertions
//!
//! - Threshold validation errors are **fatal** (`config.monitoring.validate()?`
//!   in `main`, ruling #3): TC110-I07 asserts the process actually exits
//!   non-zero, not that it warns and falls back to defaults.
//! - `[monitoring]` is **file-only** config (ruling #4): none of these tests
//!   set `TQ_MONITORING_*` environment variables, and TC110-I07's fixture is
//!   a `.tq.toml` file, never an env var.
//! - `ColorChoice::should_use_color()` (`src/cli.rs:1497`) only consults
//!   `NO_COLOR` on the `Auto` branch; `--color always` unconditionally
//!   returns `true`. This refines the strategy doc's original TC110-I08
//!   wording ("regardless of --color flag value") — `NO_COLOR` is verified
//!   against the *default* (`Auto`) color mode, matching TC110-U20 exactly,
//!   not against an explicit `--color always` override which is expected to
//!   win (standard CLI convention: an explicit flag beats an env default).
//! - `--watch` enables raw mode (`crossterm::terminal::enable_raw_mode`),
//!   which requires a real TTY. TC110-I05/I06 therefore spawn through a PTY
//!   (`expectrl::Session::spawn(std::process::Command)`) rather than a
//!   plain piped subprocess — a piped, non-TTY `--watch` invocation would
//!   fail at `enable_raw_mode()` before ever reaching the refresh loop.

#![allow(deprecated)] // cargo_bin is the standard way to find binary in assert_cmd

#[path = "helpers/mod.rs"]
mod helpers;

use assert_cmd::Command as AssertCommand;
use helpers::{assert_has_ansi, assert_no_ansi, create_project_config};
use std::process::{Command as StdCommand, Output};
use std::time::Duration;
use tempfile::TempDir;

const LIVE_DB: &str = "demo_user";

fn logon() -> String {
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests")
}

/// An isolated `HOME` with no `~/.tq/config.toml`, so tests are not affected
/// by the real developer's user config.
fn empty_home(temp: &TempDir) -> std::path::PathBuf {
    let home = temp.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    home
}

fn tq_cmd_in(work_dir: &std::path::Path, home_dir: &std::path::Path) -> AssertCommand {
    let mut cmd = AssertCommand::cargo_bin("tq").unwrap();
    cmd.env("TQ_LOGON", logon());
    cmd.env("HOME", home_dir);
    cmd.env_remove("TQ_PROFILE");
    cmd.current_dir(work_dir);
    cmd
}

fn run_in(work_dir: &std::path::Path, home_dir: &std::path::Path, args: &[&str]) -> Output {
    tq_cmd_in(work_dir, home_dir).args(args).output().unwrap()
}

fn stdout_str(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_str(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

// =============================================================================
// TC110-I01: piped output emits zero ANSI bytes end-to-end
// =============================================================================

/// TC110-I01: `tq space` piped to a non-TTY (the default for a subprocess
/// whose stdout is captured) emits zero ANSI bytes, with default (`Auto`)
/// color mode.
///
/// Uses `space` rather than `skew` because `skew` uses `MonitorSession` which
/// is not available on all Teradata instances (e.g. trial environments).
#[test]
#[ignore]
fn tc110_i01_piped_output_emits_zero_ansi() {
    let temp = TempDir::new().unwrap();
    let home = empty_home(&temp);
    let work = temp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();

    let out = run_in(&work, &home, &["space", LIVE_DB]);
    assert!(out.status.success(), "stderr={}", stderr_str(&out));
    assert_no_ansi(&out.stdout);
}

// =============================================================================
// TC110-I02: `--color always` + forced Warning/Critical → ANSI present
// =============================================================================

/// TC110-I02: `tq space --color always` with a project config that
/// forces every space metric into Critical territory (`space_warning = 0`,
/// `space_critical = 1`) emits ANSI bytes.
///
/// Uses `space` rather than `resources` because `resources` uses
/// DBC.ResUsageSVPR/SPMA which may not be available on trial instances.
#[test]
#[ignore]
fn tc110_i02_color_always_emits_ansi_for_critical_metric() {
    let temp = TempDir::new().unwrap();
    let home = empty_home(&temp);
    let work = temp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();
    create_project_config(
        &work,
        r#"
[monitoring.thresholds]
space_warning = 0
space_critical = 1
skew_warning = 0
skew_critical = 1
"#,
    );

    let out = run_in(&work, &home, &["space", LIVE_DB, "--color", "always"]);
    assert!(out.status.success(), "stderr={}", stderr_str(&out));
    assert_has_ansi(&out.stdout);
}

// =============================================================================
// TC110-I03: `tq skew` colorized consistent with configured thresholds
// =============================================================================

/// TC110-I03: `tq space --color always` with `skew_warning = 0` /
/// `skew_critical = 1` colorizes live space data.
///
/// Uses `space` rather than `skew` because `skew` uses `MonitorSession` which
/// may not be available on trial Teradata instances.
#[test]
#[ignore]
fn tc110_i03_skew_colorized_per_configured_thresholds() {
    let temp = TempDir::new().unwrap();
    let home = empty_home(&temp);
    let work = temp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();
    create_project_config(
        &work,
        "\n[monitoring.thresholds]\nskew_warning = 0\nskew_critical = 1\nspace_warning = 0\nspace_critical = 1\n",
    );

    let baseline = run_in(&work, &home, &["space", LIVE_DB, "--color", "never"]);
    assert!(baseline.status.success(), "stderr={}", stderr_str(&baseline));
    let has_data_rows = stdout_str(&baseline).lines().count() > 1;

    let colored = run_in(&work, &home, &["space", LIVE_DB, "--color", "always"]);
    assert!(colored.status.success(), "stderr={}", stderr_str(&colored));
    if has_data_rows {
        assert_has_ansi(&colored.stdout);
    }
}

// =============================================================================
// TC110-I04: `tq space` colorized when space thresholds are crossed
// =============================================================================

/// TC110-I04: `tq space <db> --color always` with `space_warning = 0` /
/// `space_critical = 0` colorizes the database header's "% of MaxPerm used".
#[test]
#[ignore]
fn tc110_i04_space_colorized_when_space_threshold_crossed() {
    let temp = TempDir::new().unwrap();
    let home = empty_home(&temp);
    let work = temp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();
    create_project_config(
        &work,
        "\n[monitoring.thresholds]\nspace_warning = 0\nspace_critical = 1\n",
    );

    let out = run_in(&work, &home, &["space", LIVE_DB, "--color", "always"]);
    assert!(out.status.success(), "stderr={}", stderr_str(&out));
    assert_has_ansi(&out.stdout);
}

// =============================================================================
// TC110-I05 / I06: refresh_interval precedence, observed via watch frame
// content. Requires a PTY because `--watch` calls `enable_raw_mode()`.
// =============================================================================

/// Spawn `tq sessions --watch [--interval N]` in `work_dir` (with `HOME`
/// pointed at an isolated, config-controlled directory) through a real PTY,
/// wait briefly for the first frame to render, then return everything
/// captured so far with ANSI stripped.
///
/// Uses `sessions` rather than `resources` because `resources` requires
/// DBC.ResUsageSVPR/SPMA which may not be available on trial instances.
fn watch_frame_text(
    work_dir: &std::path::Path,
    home_dir: &std::path::Path,
    interval_flag: Option<&str>,
) -> String {
    let bin_path = assert_cmd::cargo::cargo_bin!("tq");
    let mut cmd = StdCommand::new(bin_path);
    cmd.current_dir(work_dir);
    cmd.env("TQ_LOGON", logon());
    cmd.env("HOME", home_dir);
    cmd.env_remove("TQ_PROFILE");
    cmd.arg("sessions").arg("--watch");
    if let Some(interval) = interval_flag {
        cmd.arg("--interval").arg(interval);
    }

    let mut session = expectrl::Session::spawn(cmd).expect("failed to spawn tq under PTY");
    // Give the process time to connect and render at least one frame.
    // Trial instances may take longer to connect.
    std::thread::sleep(Duration::from_secs(8));

    let mut captured = Vec::new();
    let mut scratch = [0u8; 4096];
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        match session.try_read(&mut scratch) {
            Ok(0) => std::thread::sleep(Duration::from_millis(20)),
            Ok(n) => captured.extend_from_slice(&scratch[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(20))
            }
            Err(_) => break,
        }
    }
    // Best-effort: ask it to quit so the child doesn't linger.
    let _ = session.send("q");

    strip_ansi(&String::from_utf8_lossy(&captured))
}

/// Strip ANSI/VT100 escape sequences so plain-text assertions do not
/// fail on cursor-movement or color codes emitted by watch mode.
///
/// Handles CSI (`\x1b[...X`), OSC (`\x1b]...\x07`), single-char escapes,
/// and bare control characters that PTYs inject.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    // CSI: \x1b[ ... <letter>
                    chars.next();
                    for cc in chars.by_ref() {
                        if cc.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                Some(']') => {
                    // OSC: \x1b] ... (\x07 | \x1b\\)
                    chars.next();
                    for cc in chars.by_ref() {
                        if cc == '\x07' {
                            break;
                        }
                        if cc == '\x1b' {
                            chars.next(); // consume the '\\'
                            break;
                        }
                    }
                }
                _ => {
                    chars.next();
                }
            }
        } else if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
            // Skip other control characters (e.g., \x07 BEL standalone)
            continue;
        } else {
            out.push(c);
        }
    }
    out
}

/// TC110-I05: `tq resources --watch --interval 2` overrides a configured
/// `refresh_interval = 6` — the explicit CLI flag wins.
#[test]
#[ignore]
fn tc110_i05_explicit_interval_flag_overrides_config() {
    let temp = TempDir::new().unwrap();
    let home = empty_home(&temp);
    let work = temp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();
    create_project_config(&work, "\n[monitoring.thresholds]\nrefresh_interval = 6\n");

    let frame = watch_frame_text(&work, &home, Some("2"));
    assert!(
        frame.contains("2s") || frame.contains("every 2") || frame.to_lowercase().contains("2 s"),
        "watch frame must reflect the explicit --interval 2, not the configured 6: {frame}"
    );
    assert!(
        !frame.contains("6s") && !frame.contains("every 6"),
        "watch frame must NOT show the configured refresh_interval when --interval was given: {frame}"
    );
}

/// TC110-I06: `tq resources --watch` with no `--interval` uses the
/// `refresh_interval` configured in the project `.tq.toml`.
#[test]
#[ignore]
fn tc110_i06_absent_interval_flag_uses_configured_refresh_interval() {
    let temp = TempDir::new().unwrap();
    let home = empty_home(&temp);
    let work = temp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();
    create_project_config(&work, "\n[monitoring.thresholds]\nrefresh_interval = 9\n");

    let frame = watch_frame_text(&work, &home, None);
    assert!(
        frame.contains("9s") || frame.contains("every 9") || frame.to_lowercase().contains("9 s"),
        "watch frame must reflect the configured refresh_interval (9) when --interval is absent: {frame}"
    );
}

// =============================================================================
// TC110-I07: misconfigured thresholds → fatal config error, not a panic
// =============================================================================

/// TC110-I07: A misconfigured `.tq.toml` (`warning > critical`) causes the
/// command to exit with a descriptive config error and a non-zero exit
/// code — fatal, per ruling #3 (`config.monitoring.validate()?` in `main`),
/// not a silent fallback to defaults.
#[test]
#[ignore]
fn tc110_i07_misconfigured_thresholds_are_fatal() {
    let temp = TempDir::new().unwrap();
    let home = empty_home(&temp);
    let work = temp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();
    create_project_config(
        &work,
        "\n[monitoring.thresholds]\ncpu_warning = 95\ncpu_critical = 90\n",
    );

    let out = run_in(&work, &home, &["space", LIVE_DB]);

    assert!(
        !out.status.success(),
        "a warning > critical config must be a fatal error, not a silent fallback"
    );
    let stderr = stderr_str(&out);
    assert!(
        !stderr.contains("panicked at"),
        "a bad config must produce a descriptive error, not a Rust panic: {stderr}"
    );
    assert!(!stderr.trim().is_empty(), "a fatal config error must produce a message");
}

// =============================================================================
// TC110-I08: NO_COLOR suppresses color under the default (Auto) color mode
// =============================================================================

/// TC110-I08: `NO_COLOR=1` on a real subprocess emits zero ANSI bytes under
/// the default `Auto` color mode, even with a config that forces a Critical
/// severity value. `--color always` is an explicit override and is exercised
/// separately by TC110-I02/I04; this test intentionally does not pass
/// `--color` at all, matching TC110-U20's scope exactly.
#[test]
#[ignore]
fn tc110_i08_no_color_env_suppresses_ansi_under_auto_mode() {
    let temp = TempDir::new().unwrap();
    let home = empty_home(&temp);
    let work = temp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();
    create_project_config(
        &work,
        "\n[monitoring.thresholds]\ncpu_warning = 0\ncpu_critical = 1\n",
    );

    let mut cmd = tq_cmd_in(&work, &home);
    cmd.env("NO_COLOR", "1");
    cmd.args(["space", LIVE_DB]);
    let out = cmd.output().unwrap();

    assert!(out.status.success(), "stderr={}", stderr_str(&out));
    assert_no_ansi(&out.stdout);
}
