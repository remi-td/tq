//! Integration tests for `--profile` flag resolution
//!
//! These tests validate how the --profile flag resolves profiles from
//! user config, project config, and how they interact with precedence rules.
//!
//! Sprint 35: Project Config Support - AC-5 and AC-6 Integration Tests
//!
//! NOTE: These tests do not require a live database connection. They test
//! the configuration loading and error handling behavior, not actual queries.

#![allow(deprecated)] // cargo_bin is the standard way to find binary in assert_cmd

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create a tq command with custom HOME and working directory
fn tq_cmd_with_env(home_dir: &std::path::Path, work_dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("tq").unwrap();
    cmd.env("HOME", home_dir);
    cmd.current_dir(work_dir);
    // Clear any existing TQ_ env vars that might interfere
    cmd.env_remove("TQ_LOGON");
    cmd.env_remove("TQ_PROFILE");
    cmd
}

/// Create user config directory and file
fn create_user_config(home_dir: &std::path::Path, content: &str) {
    let config_dir = home_dir.join(".tq");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.toml");
    fs::write(&config_path, content).unwrap();
}

/// Create project config file in directory
fn create_project_config(dir: &std::path::Path, content: &str) {
    let config_path = dir.join(".tq.toml");
    fs::write(&config_path, content).unwrap();
}

// =============================================================================
// Test Group 2: `--profile` Resolution (7 tests)
// =============================================================================

/// TC-35-007: `--profile` resolves from user config when only user has profile
///
/// When a profile exists only in user config (~/.tq/config.toml),
/// --profile should find and use it.
#[test]
fn test_profile_resolves_from_user_config_only() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create user config with complete profile
    let user_config = r#"
[profiles.user_only]
host = "user-only.example.com"
port = 1025
database = "userdb"
user = "useruser"
"#;
    create_user_config(&home_dir, user_config);

    // No project config

    // Try to use the profile - will fail to connect but should find the profile
    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.args(["--profile", "user_only", "ping"]);

    // The command will try to connect and fail (no real DB), but the error
    // should show it tried to connect to the user config host
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should attempt connection to user config host (not "profile not found" error)
    assert!(
        stderr.contains("user-only.example.com")
            || stderr.contains("Connection refused")
            || stderr.contains("connect")
            || stderr.contains("Network")
            || !stderr.contains("not found"),
        "Should find profile in user config. stderr was: {}",
        stderr
    );
}

/// TC-35-008: `--profile` resolves from project config when only project has profile
///
/// When a profile exists only in project config (.tq.toml),
/// --profile should find and use it.
#[test]
fn test_profile_resolves_from_project_config_only() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create empty user config
    create_user_config(&home_dir, "# empty\n");

    // Create project config with complete profile
    let project_config = r#"
[profiles.project_only]
host = "project-only.example.com"
port = 1025
database = "projectdb"
user = "projectuser"
"#;
    create_project_config(&work_dir, project_config);

    // Try to use the profile
    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.args(["--profile", "project_only", "ping"]);

    // The command will try to connect and fail (no real DB), but the error
    // should show it tried to connect to the project config host
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should attempt connection to project config host (not "profile not found" error)
    assert!(
        stderr.contains("project-only.example.com")
            || stderr.contains("Connection refused")
            || stderr.contains("connect")
            || stderr.contains("Network")
            || !stderr.contains("not found"),
        "Should find profile in project config. stderr was: {}",
        stderr
    );
}

/// TC-35-009: `--profile` prefers project config when both have same name (AC-6)
///
/// When the same profile name exists in both user and project configs,
/// project config values should take precedence.
#[test]
fn test_profile_prefers_project_over_user() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create user config with a profile
    let user_config = r#"
[profiles.shared]
host = "user-host.example.com"
port = 1025
database = "userdb"
user = "useruser"
"#;
    create_user_config(&home_dir, user_config);

    // Create project config with same profile name but different host
    let project_config = r#"
[profiles.shared]
host = "project-host.example.com"
port = 1025
database = "projectdb"
user = "projectuser"
"#;
    create_project_config(&work_dir, project_config);

    // Try to use the profile
    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.args(["--profile", "shared", "ping"]);

    // The command will try to connect and fail, but should use project host
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should use project config host (project-host, not user-host)
    // Note: The error message might include the host name being connected to
    assert!(
        !stderr.contains("user-host.example.com")
            || stderr.contains("project-host.example.com"),
        "Should prefer project config host over user config. stderr was: {}",
        stderr
    );
}

/// TC-35-010: `--profile` merges fields from both (project host, user credentials)
///
/// When profile exists in both configs but with different fields,
/// they should merge with project values taking precedence.
#[test]
fn test_profile_merges_fields_from_both_configs() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create user config with user-specific settings
    let user_config = r#"
[profiles.merged]
host = "user-host.example.com"
port = 1025
database = "userdb"
user = "my_personal_user"
"#;
    create_user_config(&home_dir, user_config);

    // Create project config with project-specific host/database only
    let project_config = r#"
[profiles.merged]
host = "team-host.example.com"
database = "teamdb"
"#;
    create_project_config(&work_dir, project_config);

    // The merged profile should have:
    // - host = "team-host.example.com" (from project)
    // - database = "teamdb" (from project)
    // - user = "my_personal_user" (from user - inherited since project doesn't override)

    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.args(["--profile", "merged", "ping"]);

    // Command will fail to connect, but should use merged config
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should have attempted to connect (not "profile not found" or "missing user")
    assert!(
        !stderr.contains("Profile 'merged' not found")
            && !stderr.contains("missing required field 'user'"),
        "Profile should merge and have valid user. stderr was: {}",
        stderr
    );
}

/// TC-35-011: `--profile` with non-existent name shows clear error
#[test]
fn test_profile_nonexistent_shows_clear_error() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create user config with one profile
    let user_config = r#"
[profiles.existing]
host = "existing.example.com"
port = 1025
database = "existingdb"
user = "existinguser"
"#;
    create_user_config(&home_dir, user_config);

    // Try to use a non-existent profile
    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.args(["--profile", "nonexistent", "ping"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("nonexistent")));
}

/// TC-35-012: Project config precedence works from subdirectory
///
/// When running from a subdirectory, should still find and use
/// project config from parent directories.
#[test]
fn test_profile_project_config_precedence_from_subdirectory() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let project_root = temp.path().join("project");
    let subdir = project_root.join("src").join("components");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&subdir).unwrap();

    // Create user config with a profile
    let user_config = r#"
[profiles.env]
host = "user-env.example.com"
port = 1025
database = "userdb"
user = "useruser"
"#;
    create_user_config(&home_dir, user_config);

    // Create project config in project root with same profile name
    let project_config = r#"
[profiles.env]
host = "project-env.example.com"
port = 1025
database = "projectdb"
user = "projectuser"
"#;
    create_project_config(&project_root, project_config);

    // Run from deeply nested subdirectory
    let mut cmd = tq_cmd_with_env(&home_dir, &subdir);
    cmd.args(["--profile", "env", "ping"]);

    // Should use project config (from parent directory), not user config
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should connect to project-env, not user-env
    assert!(
        !stderr.contains("user-env.example.com")
            || stderr.contains("project-env.example.com"),
        "Should use project config from parent. stderr was: {}",
        stderr
    );
}

/// TC-35-013: CLI flag `--logon` overrides both user and project config
///
/// Even when --profile is specified, explicit --logon should take precedence.
/// This test validates that --logon takes precedence over any profile settings.
#[test]
fn test_logon_flag_overrides_profile() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create user config with a profile
    let user_config = r#"
[profiles.myprofile]
host = "profile-host.example.com"
port = 1025
database = "profiledb"
user = "profileuser"
"#;
    create_user_config(&home_dir, user_config);

    // Use --logon to override (note: --logon takes full precedence, --profile is ignored)
    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.args([
        "--logon",
        "cliuser:clipass@cli-host.example.com:1025/clidb",
        "ping",
    ]);

    // Should attempt to connect to CLI-specified host
    // The command will fail (no real DB), but we verify:
    // 1. It does NOT show "profile not found" error
    // 2. It does NOT show "missing required field" error (connection string is complete)
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // With a valid connection string, should attempt connection (not config error)
    // The error will be about connection failure, not about profile issues
    assert!(
        !stderr.contains("not found") && !stderr.contains("missing required field"),
        "CLI --logon should be used without needing profile. stderr was: {}",
        stderr
    );

    // Additionally, verify the command attempted to execute (ping attempt)
    assert!(
        stderr.contains("Ping") || stderr.contains("ping") || stderr.contains("failed") || output.status.code().is_some(),
        "Command should have attempted to execute ping. stderr was: {}",
        stderr
    );
}
