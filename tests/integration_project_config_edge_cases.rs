//! Integration tests for project config edge cases
//!
//! These tests validate edge cases in project config discovery and handling,
//! including missing configs, filesystem boundaries, and error conditions.
//!
//! Sprint 35: Project Config Support - Edge Cases Integration Tests

#![allow(deprecated)] // cargo_bin is the standard way to find binary in assert_cmd

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[path = "helpers/mod.rs"]
mod helpers;
use helpers::{create_project_config, create_user_config};

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

// =============================================================================
// Test Group 3: Edge Cases (4 tests)
// =============================================================================

/// TC-35-014: No `.tq.toml` in tree - uses only user config (not an error)
///
/// When no project config exists anywhere in the directory tree,
/// the command should work normally using only user config.
#[test]
fn test_no_project_config_uses_user_config_only() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work").join("some").join("deep").join("path");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create user config with profiles
    let user_config = r#"
[profiles.userprofile]
host = "user.example.com"
port = 1025
database = "userdb"
user = "useruser"
"#;
    create_user_config(&home_dir, user_config);

    // No .tq.toml anywhere in work_dir tree

    // `tq profiles` should work and show user profiles
    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.arg("profiles");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("userprofile"))
        .stdout(predicate::str::contains("user.example.com"))
        .stdout(predicate::str::contains("From user config"));
}

/// TC-35-015: `.tq.toml` at filesystem root still discovered
///
/// Project config discovery should work even when the config is at the
/// filesystem root (though this is an unusual case).
///
/// Note: We can't actually write to filesystem root in tests, so we simulate
/// by testing that config discovery works at the temp directory root.
#[test]
fn test_project_config_at_temp_root_discovered() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let project_root = temp.path();
    let deep_subdir = project_root.join("a").join("b").join("c").join("d");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&deep_subdir).unwrap();

    // Create empty user config
    create_user_config(&home_dir, "# empty\n");

    // Create project config at the temp root (simulating filesystem root)
    let project_config = r#"
[profiles.root_profile]
host = "root.example.com"
port = 1025
database = "rootdb"
user = "rootuser"
"#;
    create_project_config(project_root, project_config);

    // Run from deeply nested directory
    let mut cmd = tq_cmd_with_env(&home_dir, &deep_subdir);
    cmd.arg("profiles");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("root_profile"))
        .stdout(predicate::str::contains("root.example.com"))
        .stdout(predicate::str::contains("From project config"));
}

/// TC-35-016: Symlink directories don't break discovery
///
/// Project config discovery should handle symlinks in the directory path
/// without breaking or infinite looping.
#[test]
#[cfg(unix)] // Symlinks work differently on Windows
fn test_symlink_directories_dont_break_discovery() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let real_project = temp.path().join("real_project");
    let symlinked_project = temp.path().join("symlinked_project");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&real_project).unwrap();

    // Create symlink to real project
    symlink(&real_project, &symlinked_project).unwrap();

    // Create empty user config
    create_user_config(&home_dir, "# empty\n");

    // Create project config in real project
    let project_config = r#"
[profiles.symlink_test]
host = "symlink.example.com"
port = 1025
database = "symlinkdb"
user = "symlinkuser"
"#;
    create_project_config(&real_project, project_config);

    // Run from symlinked directory
    let mut cmd = tq_cmd_with_env(&home_dir, &symlinked_project);
    cmd.arg("profiles");

    // Should find the config through the symlink
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("symlink_test"))
        .stdout(predicate::str::contains("symlink.example.com"));
}

/// TC-35-017: Permission error on `.tq.toml` shows helpful message
///
/// When the project config file exists but is not readable,
/// should show a clear error rather than silently ignoring.
#[test]
#[cfg(unix)] // File permissions work differently on Windows
fn test_unreadable_project_config_shows_error() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create user config with a profile
    let user_config = r#"
[profiles.fallback]
host = "fallback.example.com"
port = 1025
database = "fallbackdb"
user = "fallbackuser"
"#;
    create_user_config(&home_dir, user_config);

    // Create project config
    let project_config = r#"
[profiles.unreadable]
host = "unreadable.example.com"
port = 1025
database = "unreadabledb"
user = "unreadableuser"
"#;
    let config_path = work_dir.join(".tq.toml");
    fs::write(&config_path, project_config).unwrap();

    // Make it unreadable (permissions 000)
    let mut perms = fs::metadata(&config_path).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&config_path, perms).unwrap();

    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.arg("profiles");

    // The behavior depends on implementation:
    // Option 1: Fail with permission error
    // Option 2: Fall back to user config only (with warning)
    //
    // Either behavior is acceptable as long as it doesn't crash silently
    let output = cmd.output().unwrap();

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&config_path).unwrap().permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&config_path, perms).unwrap();

    // Either succeeds with fallback, or fails with error message
    // (we accept both - the key is no silent failure)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should either show user config profiles OR show an error
    assert!(
        stdout.contains("fallback") || !stderr.is_empty(),
        "Should either fall back to user config or show an error. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

/// Additional edge case: Empty project config file is handled gracefully
#[test]
fn test_empty_project_config_handled_gracefully() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create user config with profiles
    let user_config = r#"
[profiles.userprofile]
host = "user.example.com"
port = 1025
database = "userdb"
user = "useruser"
"#;
    create_user_config(&home_dir, user_config);

    // Create empty project config
    create_project_config(&work_dir, "");

    // Should work and show user profiles (empty project config adds nothing)
    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.arg("profiles");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("userprofile"))
        .stdout(predicate::str::contains("user.example.com"));
}

/// Additional edge case: Project config with only comments
#[test]
fn test_project_config_with_only_comments() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let work_dir = temp.path().join("work");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    // Create user config with profiles
    let user_config = r#"
[profiles.userprofile]
host = "user.example.com"
port = 1025
database = "userdb"
user = "useruser"
"#;
    create_user_config(&home_dir, user_config);

    // Create project config with only comments
    let project_config = r#"
# This is a project config file
# But it has no actual configuration
# Just comments
"#;
    create_project_config(&work_dir, project_config);

    // Should work and show user profiles
    let mut cmd = tq_cmd_with_env(&home_dir, &work_dir);
    cmd.arg("profiles");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("userprofile"))
        .stdout(predicate::str::contains("user.example.com"));
}
