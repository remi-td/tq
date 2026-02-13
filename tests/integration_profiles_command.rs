//! Integration tests for `tq profiles` command
//!
//! These tests validate the profiles command output when displaying
//! profiles from user config, project config, or both.
//!
//! Sprint 35: Project Config Support - AC-4 Integration Tests

#![allow(deprecated)] // cargo_bin is the standard way to find binary in assert_cmd

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
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
fn create_user_config(home_dir: &std::path::Path, content: &str) -> PathBuf {
    let config_dir = home_dir.join(".tq");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.toml");
    fs::write(&config_path, content).unwrap();
    config_path
}

/// Create project config file in directory
fn create_project_config(dir: &std::path::Path, content: &str) -> PathBuf {
    let config_path = dir.join(".tq.toml");
    fs::write(&config_path, content).unwrap();
    config_path
}

// =============================================================================
// Test Group 1: `tq profiles` Command (6 tests)
// =============================================================================

/// TC-35-001: `tq profiles` with only user config shows user profiles
#[test]
fn test_profiles_with_only_user_config() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create user config with a profile
    let user_config = r#"
[profiles.dev]
host = "dev.example.com"
port = 1025
database = "devdb"
user = "devuser"
"#;
    create_user_config(&home_dir, user_config);

    // No project config in work_dir

    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.arg("profiles");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available profiles:"))
        .stdout(predicate::str::contains("dev"))
        .stdout(predicate::str::contains("dev.example.com"))
        .stdout(predicate::str::contains("devdb"))
        .stdout(predicate::str::contains("devuser"))
        .stdout(predicate::str::contains("From user config"));
}

/// TC-35-002: `tq profiles` with only project config shows project profiles
#[test]
fn test_profiles_with_only_project_config() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create empty user config (no profiles)
    create_user_config(&home_dir, "# empty config\n");

    // Create project config with a profile
    let project_config = r#"
[profiles.project_db]
host = "project.example.com"
port = 1025
database = "projectdb"
user = "projectuser"
"#;
    create_project_config(&work_dir, project_config);

    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.arg("profiles");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available profiles:"))
        .stdout(predicate::str::contains("project_db"))
        .stdout(predicate::str::contains("project.example.com"))
        .stdout(predicate::str::contains("projectdb"))
        .stdout(predicate::str::contains("projectuser"))
        .stdout(predicate::str::contains("From project config"));
}

/// TC-35-003: `tq profiles` with both configs shows both with source indicators
#[test]
fn test_profiles_shows_both_user_and_project_with_sources() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create user config with a profile
    let user_config = r#"
[profiles.user_profile]
host = "user.example.com"
port = 1025
database = "userdb"
user = "useruser"
"#;
    create_user_config(&home_dir, user_config);

    // Create project config with a different profile
    let project_config = r#"
[profiles.project_profile]
host = "project.example.com"
port = 1025
database = "projectdb"
user = "projectuser"
"#;
    create_project_config(&work_dir, project_config);

    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.arg("profiles");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available profiles:"))
        // User profile
        .stdout(predicate::str::contains("user_profile"))
        .stdout(predicate::str::contains("user.example.com"))
        .stdout(predicate::str::contains("From user config"))
        // Project profile
        .stdout(predicate::str::contains("project_profile"))
        .stdout(predicate::str::contains("project.example.com"))
        .stdout(predicate::str::contains("From project config"));
}

/// TC-35-004: `tq profiles` shows merged profiles when names conflict
#[test]
fn test_profiles_shows_merged_when_names_conflict() {
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

    // Create project config with same profile name but different values
    let project_config = r#"
[profiles.shared]
host = "project-host.example.com"
database = "projectdb"
"#;
    create_project_config(&work_dir, project_config);

    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.arg("profiles");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available profiles:"))
        .stdout(predicate::str::contains("shared"))
        // Project values should take precedence
        .stdout(predicate::str::contains("project-host.example.com"))
        .stdout(predicate::str::contains("projectdb"))
        // Should indicate merged source
        .stdout(predicate::str::contains("merged"));
}

/// TC-35-005: Project config in parent directory still detected
#[test]
fn test_profiles_project_config_in_parent_directory() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let project_root = temp.path().join("project");
    let subdir = project_root.join("src").join("nested");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&subdir).unwrap();

    // Create empty user config
    create_user_config(&home_dir, "# empty\n");

    // Create project config in project root (parent of subdir)
    let project_config = r#"
[profiles.parent_profile]
host = "parent.example.com"
port = 1025
database = "parentdb"
user = "parentuser"
"#;
    create_project_config(&project_root, project_config);

    // Run from subdir - should still find parent's .tq.toml
    let mut cmd = tq_cmd_with_env(&home_dir, &subdir);
    cmd.arg("profiles");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available profiles:"))
        .stdout(predicate::str::contains("parent_profile"))
        .stdout(predicate::str::contains("parent.example.com"))
        .stdout(predicate::str::contains("From project config"));
}

/// TC-35-006: Error handling when project config has invalid TOML
///
/// When the project config has invalid TOML, the application logs a warning
/// and falls back to defaults (graceful degradation).
#[test]
fn test_profiles_with_invalid_project_config_toml() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create valid user config with a profile
    let user_config = r#"
[profiles.valid]
host = "valid.example.com"
port = 1025
database = "validdb"
user = "validuser"
"#;
    create_user_config(&home_dir, user_config);

    // Create invalid project config (malformed TOML)
    let invalid_config = r#"
[profiles.broken
host = "broken.example.com"
"#;
    create_project_config(&work_dir, invalid_config);

    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.arg("profiles");

    // The application handles invalid TOML gracefully:
    // - Logs a warning about the parse error
    // - Falls back to defaults
    // This is good UX - a corrupt project config shouldn't block all operations
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("WARN").or(predicate::str::contains("parse error")));
}
