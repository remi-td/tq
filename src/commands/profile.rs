//! Profile management commands
//!
//! Implements `tq profile add`, `tq profile edit`, `tq profile delete`, and `tq profile list`.
//! These commands manage connection profiles stored in the user config file (~/.tq/config.toml).
//!
//! Uses atomic file writes (write to temp file, then rename) to avoid corruption.
//! The `TQ_CONFIG_DIR` environment variable can override the default config directory
//! for testing purposes.

use crate::cli::ProfileAction;
use crate::config::Config;
use crate::error::{Result, TqError};
use std::path::PathBuf;

/// Valid logon mechanisms (case-insensitive matching)
const VALID_LOGMECHS: &[&str] = &["TD2", "LDAP", "KRB5", "TDNEGO"];

/// Get the config directory, respecting TQ_CONFIG_DIR env var for testing
fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("TQ_CONFIG_DIR") {
        PathBuf::from(dir)
    } else if let Some(user_dirs) = directories::UserDirs::new() {
        user_dirs.home_dir().join(".tq")
    } else {
        PathBuf::from("~/.tq")
    }
}

/// Get the config file path
fn config_file_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// Read existing config as a TOML table, or return an empty table
fn read_config_table() -> Result<toml::Table> {
    let path = config_file_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(|e| TqError::FileReadError {
            path: path.clone(),
            source: e,
        })?;
        let table: toml::Table =
            toml::from_str(&content).map_err(|e| TqError::ConfigParseError(e.to_string()))?;
        Ok(table)
    } else {
        Ok(toml::Table::new())
    }
}

/// Write a TOML table to the config file atomically
fn write_config_table(table: &toml::Table) -> Result<()> {
    let dir = config_dir();

    // Create directory if it does not exist
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| TqError::FileWriteError {
            path: dir.clone(),
            source: e,
        })?;
    }

    let content =
        toml::to_string_pretty(table).map_err(|e| TqError::ConfigParseError(e.to_string()))?;

    let config_path = config_file_path();

    // Atomic write: write to temp file in same directory, then rename
    let temp_path = dir.join(".config.toml.tmp");
    std::fs::write(&temp_path, &content).map_err(|e| TqError::FileWriteError {
        path: temp_path.clone(),
        source: e,
    })?;
    std::fs::rename(&temp_path, &config_path).map_err(|e| TqError::FileWriteError {
        path: config_path.clone(),
        source: e,
    })?;

    Ok(())
}

/// Get or create the [profiles] section of the config table
fn get_profiles_section(table: &mut toml::Table) -> &mut toml::Table {
    table
        .entry("profiles")
        .or_insert_with(|| toml::Value::Table(toml::Table::new()))
        .as_table_mut()
        .expect("profiles should be a table")
}

/// Validate logmech value (case-insensitive)
fn validate_logmech(logmech: &str) -> Result<()> {
    let upper = logmech.to_uppercase();
    if VALID_LOGMECHS.contains(&upper.as_str()) {
        Ok(())
    } else {
        Err(TqError::InvalidLogonMechanism(logmech.to_string()))
    }
}

/// Validate port value
fn validate_port(port: u16) -> Result<()> {
    if port == 0 {
        Err(TqError::InvalidConfig(
            "Port must be between 1 and 65535".to_string(),
        ))
    } else {
        Ok(())
    }
}

/// Execute a profile management action
pub fn execute(action: &ProfileAction, config: &Config) -> Result<()> {
    match action {
        ProfileAction::Add {
            name,
            host,
            port,
            database,
            user,
            logmech,
            password_file,
        } => handle_add(name, host, port, database, user, logmech, password_file),
        ProfileAction::Edit {
            name,
            host,
            port,
            database,
            user,
            logmech,
            password_file,
        } => handle_edit(name, host, port, database, user, logmech, password_file),
        ProfileAction::Delete { name, force } => handle_delete(name, *force),
        ProfileAction::List => {
            // Delegate to the existing handle_profiles logic in main
            // This is handled in main.rs by routing to the existing function
            handle_list(config)
        }
    }
}

fn handle_add(
    name: &str,
    host: &str,
    port: &Option<u16>,
    database: &Option<String>,
    user: &Option<String>,
    logmech: &Option<String>,
    password_file: &Option<PathBuf>,
) -> Result<()> {
    // Validate optional fields
    if let Some(p) = port {
        validate_port(*p)?;
    }
    if let Some(ref lm) = logmech {
        validate_logmech(lm)?;
    }

    let mut table = read_config_table()?;
    let profiles = get_profiles_section(&mut table);

    // Check if profile already exists
    if profiles.contains_key(name) {
        return Err(TqError::InvalidConfig(format!(
            "Profile '{}' already exists. Use 'tq profile edit {}' to modify it.",
            name, name
        )));
    }

    // Build profile table
    let mut profile = toml::Table::new();
    profile.insert("host".to_string(), toml::Value::String(host.to_string()));

    if let Some(p) = port {
        profile.insert("port".to_string(), toml::Value::Integer(i64::from(*p)));
    }
    if let Some(ref db) = database {
        profile.insert("database".to_string(), toml::Value::String(db.clone()));
    }
    if let Some(ref u) = user {
        profile.insert("user".to_string(), toml::Value::String(u.clone()));
    }
    if let Some(ref lm) = logmech {
        profile.insert(
            "logmech".to_string(),
            toml::Value::String(lm.to_uppercase()),
        );
    }
    if let Some(ref pf) = password_file {
        profile.insert(
            "password_file".to_string(),
            toml::Value::String(pf.to_string_lossy().to_string()),
        );
    }

    profiles.insert(name.to_string(), toml::Value::Table(profile));
    write_config_table(&table)?;

    println!("Profile '{}' added to {}", name, config_file_path().display());
    Ok(())
}

fn handle_edit(
    name: &str,
    host: &Option<String>,
    port: &Option<u16>,
    database: &Option<String>,
    user: &Option<String>,
    logmech: &Option<String>,
    password_file: &Option<PathBuf>,
) -> Result<()> {
    // Require at least one field to update
    if host.is_none()
        && port.is_none()
        && database.is_none()
        && user.is_none()
        && logmech.is_none()
        && password_file.is_none()
    {
        return Err(TqError::InvalidConfig(
            "At least one field must be specified to edit. Use --host, --port, --database, --user, --logmech, or --password-file.".to_string(),
        ));
    }

    // Validate optional fields
    if let Some(p) = port {
        validate_port(*p)?;
    }
    if let Some(ref lm) = logmech {
        validate_logmech(lm)?;
    }

    let mut table = read_config_table()?;
    let profiles = get_profiles_section(&mut table);

    // Check that profile exists
    let profile = profiles
        .get_mut(name)
        .and_then(|v| v.as_table_mut())
        .ok_or_else(|| {
            TqError::InvalidConfig(format!(
                "Profile '{}' does not exist. Use 'tq profile add {}' to create it.",
                name, name
            ))
        })?;

    // Update only provided fields
    if let Some(ref h) = host {
        profile.insert("host".to_string(), toml::Value::String(h.clone()));
    }
    if let Some(p) = port {
        profile.insert("port".to_string(), toml::Value::Integer(i64::from(*p)));
    }
    if let Some(ref db) = database {
        profile.insert("database".to_string(), toml::Value::String(db.clone()));
    }
    if let Some(ref u) = user {
        profile.insert("user".to_string(), toml::Value::String(u.clone()));
    }
    if let Some(ref lm) = logmech {
        profile.insert(
            "logmech".to_string(),
            toml::Value::String(lm.to_uppercase()),
        );
    }
    if let Some(ref pf) = password_file {
        profile.insert(
            "password_file".to_string(),
            toml::Value::String(pf.to_string_lossy().to_string()),
        );
    }

    write_config_table(&table)?;

    println!(
        "Profile '{}' updated in {}",
        name,
        config_file_path().display()
    );
    Ok(())
}

/// Ask the user for confirmation, reading from the provided reader.
///
/// Returns `true` if the user enters "y" or "Y", `false` otherwise.
/// Extracted for testability.
fn confirm_deletion(
    name: &str,
    is_tty: bool,
    reader: &mut dyn std::io::BufRead,
) -> Result<bool> {
    if is_tty {
        eprint!("Delete profile '{}'? [y/N] ", name);
        let mut input = String::new();
        reader
            .read_line(&mut input)
            .map_err(TqError::IoError)?;
        let trimmed = input.trim();
        Ok(trimmed == "y" || trimmed == "Y")
    } else {
        Err(TqError::InvalidConfig(format!(
            "Deleting profile '{}' requires --force in non-interactive mode.\n\
             Usage: tq profile delete {} --force",
            name, name
        )))
    }
}

fn handle_delete(name: &str, force: bool) -> Result<()> {
    use std::io::IsTerminal;

    if !force {
        let is_tty = std::io::stdin().is_terminal();
        let mut stdin = std::io::BufReader::new(std::io::stdin());
        let confirmed = confirm_deletion(name, is_tty, &mut stdin)?;
        if !confirmed {
            eprintln!("Aborted.");
            return Ok(());
        }
    }

    let mut table = read_config_table()?;
    let profiles = get_profiles_section(&mut table);

    if profiles.remove(name).is_none() {
        return Err(TqError::InvalidConfig(format!(
            "Profile '{}' does not exist.",
            name
        )));
    }

    write_config_table(&table)?;

    println!(
        "Profile '{}' deleted from {}",
        name,
        config_file_path().display()
    );
    Ok(())
}

/// Display a single profile's details to stdout.
///
/// Shared helper used by both `tq profile list` and `tq profiles`.
/// An optional `source_tag` (e.g., "[project]") is appended to each field line.
pub fn display_profile(
    name: &str,
    profile: &crate::config::ConnectionSettings,
    source_tag: Option<&str>,
) {
    let host = profile.host.as_deref().unwrap_or("<not set>");
    let database = profile.database.as_deref().unwrap_or("<not set>");
    let user = profile.user.as_deref().unwrap_or("<not set>");
    let tag = source_tag.map(|t| format!(" {}", t)).unwrap_or_default();

    println!("  {}", name);
    println!("    Host:     {}{}", host, tag);
    println!("    Database: {}{}", database, tag);
    println!("    User:     {}{}", user, tag);

    if let Some(ref lm) = profile.logmech {
        if lm.to_uppercase() != "TD2" {
            println!("    Logmech:  {}{}", lm, tag);
        }
    }
    println!();
}

/// Handle `tq profile list` by delegating to the existing profiles display logic
fn handle_list(config: &Config) -> Result<()> {
    if config.profiles.is_empty() {
        println!("No profiles defined.\n");
        println!(
            "To create a profile:\n  tq profile add <name> --host <host> [--port <port>] [--database <db>] [--user <user>]"
        );
        return Ok(());
    }

    println!("Available profiles:\n");

    let mut names: Vec<&String> = config.profiles.keys().collect();
    names.sort();

    for name in names {
        if let Some(profile) = config.profiles.get(name) {
            display_profile(name, profile, None);
        }
    }

    println!("Use: tq --profile <name> <command>");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mutex to serialize tests that modify environment variables and filesystem
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    /// Helper to run a test with TQ_CONFIG_DIR set to a temp directory
    fn with_test_config_dir<F>(test_fn: F)
    where
        F: FnOnce(&std::path::Path),
    {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let original = std::env::var("TQ_CONFIG_DIR").ok();

        std::env::set_var("TQ_CONFIG_DIR", temp.path());
        test_fn(temp.path());

        // Restore
        match original {
            Some(val) => std::env::set_var("TQ_CONFIG_DIR", val),
            None => std::env::remove_var("TQ_CONFIG_DIR"),
        }
    }

    /// Helper to run a test with a pre-existing config
    fn with_test_config_containing<F>(toml_content: &str, test_fn: F)
    where
        F: FnOnce(&std::path::Path),
    {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let original = std::env::var("TQ_CONFIG_DIR").ok();

        std::env::set_var("TQ_CONFIG_DIR", temp.path());

        // Write initial config
        let config_path = temp.path().join("config.toml");
        std::fs::write(&config_path, toml_content).unwrap();

        test_fn(temp.path());

        // Restore
        match original {
            Some(val) => std::env::set_var("TQ_CONFIG_DIR", val),
            None => std::env::remove_var("TQ_CONFIG_DIR"),
        }
    }

    // =========================================================================
    // Validation tests
    // =========================================================================

    #[test]
    fn test_validate_logmech_valid() {
        assert!(validate_logmech("TD2").is_ok());
        assert!(validate_logmech("td2").is_ok());
        assert!(validate_logmech("LDAP").is_ok());
        assert!(validate_logmech("ldap").is_ok());
        assert!(validate_logmech("KRB5").is_ok());
        assert!(validate_logmech("krb5").is_ok());
        assert!(validate_logmech("TDNEGO").is_ok());
        assert!(validate_logmech("tdnego").is_ok());
    }

    #[test]
    fn test_validate_logmech_invalid() {
        assert!(validate_logmech("INVALID").is_err());
        assert!(validate_logmech("").is_err());
        assert!(validate_logmech("oauth").is_err());
    }

    #[test]
    fn test_validate_port_valid() {
        assert!(validate_port(1).is_ok());
        assert!(validate_port(1025).is_ok());
        assert!(validate_port(65535).is_ok());
    }

    #[test]
    fn test_validate_port_zero() {
        assert!(validate_port(0).is_err());
    }

    // =========================================================================
    // Profile add tests
    // =========================================================================

    #[test]
    fn test_add_profile_creates_config_dir_and_file() {
        with_test_config_dir(|dir| {
            let result = handle_add(
                "dev",
                "dev.example.com",
                &None,
                &None,
                &None,
                &None,
                &None,
            );
            assert!(result.is_ok());

            // Config file should exist
            let config_path = dir.join("config.toml");
            assert!(config_path.exists());

            // Should contain the profile
            let content = std::fs::read_to_string(&config_path).unwrap();
            assert!(content.contains("[profiles.dev]"));
            assert!(content.contains("host = \"dev.example.com\""));
        });
    }

    #[test]
    fn test_add_profile_with_all_fields() {
        with_test_config_dir(|dir| {
            let result = handle_add(
                "prod",
                "prod.example.com",
                &Some(1025),
                &Some("proddb".to_string()),
                &Some("admin".to_string()),
                &Some("LDAP".to_string()),
                &Some(PathBuf::from("~/.tq/passwords/prod")),
            );
            assert!(result.is_ok());

            let content = std::fs::read_to_string(dir.join("config.toml")).unwrap();
            assert!(content.contains("host = \"prod.example.com\""));
            assert!(content.contains("port = 1025"));
            assert!(content.contains("database = \"proddb\""));
            assert!(content.contains("user = \"admin\""));
            assert!(content.contains("logmech = \"LDAP\""));
            assert!(content.contains("password_file = \"~/.tq/passwords/prod\""));
        });
    }

    #[test]
    fn test_add_profile_duplicate_fails() {
        with_test_config_containing(
            "[profiles.dev]\nhost = \"old.example.com\"\n",
            |_dir| {
                let result = handle_add(
                    "dev",
                    "new.example.com",
                    &None,
                    &None,
                    &None,
                    &None,
                    &None,
                );
                assert!(result.is_err());
                let err_msg = format!("{}", result.unwrap_err());
                assert!(err_msg.contains("already exists"));
            },
        );
    }

    #[test]
    fn test_add_profile_invalid_logmech_fails() {
        with_test_config_dir(|_dir| {
            let result = handle_add(
                "dev",
                "dev.example.com",
                &None,
                &None,
                &None,
                &Some("INVALID".to_string()),
                &None,
            );
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_add_profile_invalid_port_fails() {
        with_test_config_dir(|_dir| {
            let result = handle_add(
                "dev",
                "dev.example.com",
                &Some(0),
                &None,
                &None,
                &None,
                &None,
            );
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_add_profile_logmech_normalized_to_uppercase() {
        with_test_config_dir(|dir| {
            let result = handle_add(
                "dev",
                "dev.example.com",
                &None,
                &None,
                &None,
                &Some("ldap".to_string()),
                &None,
            );
            assert!(result.is_ok());

            let content = std::fs::read_to_string(dir.join("config.toml")).unwrap();
            assert!(content.contains("logmech = \"LDAP\""));
        });
    }

    // =========================================================================
    // Profile edit tests
    // =========================================================================

    #[test]
    fn test_edit_profile_updates_field() {
        with_test_config_containing(
            "[profiles.dev]\nhost = \"old.example.com\"\nport = 1025\n",
            |dir| {
                let result = handle_edit(
                    "dev",
                    &Some("new.example.com".to_string()),
                    &None,
                    &None,
                    &None,
                    &None,
                    &None,
                );
                assert!(result.is_ok());

                let content = std::fs::read_to_string(dir.join("config.toml")).unwrap();
                assert!(content.contains("host = \"new.example.com\""));
                // Port should be preserved
                assert!(content.contains("port = 1025"));
            },
        );
    }

    #[test]
    fn test_edit_profile_adds_new_field() {
        with_test_config_containing("[profiles.dev]\nhost = \"dev.example.com\"\n", |dir| {
            let result = handle_edit(
                "dev",
                &None,
                &None,
                &Some("mydb".to_string()),
                &None,
                &None,
                &None,
            );
            assert!(result.is_ok());

            let content = std::fs::read_to_string(dir.join("config.toml")).unwrap();
            assert!(content.contains("host = \"dev.example.com\""));
            assert!(content.contains("database = \"mydb\""));
        });
    }

    #[test]
    fn test_edit_profile_nonexistent_fails() {
        with_test_config_dir(|_dir| {
            let result = handle_edit(
                "nonexistent",
                &Some("host.example.com".to_string()),
                &None,
                &None,
                &None,
                &None,
                &None,
            );
            assert!(result.is_err());
            let err_msg = format!("{}", result.unwrap_err());
            assert!(err_msg.contains("does not exist"));
        });
    }

    #[test]
    fn test_edit_profile_no_fields_fails() {
        with_test_config_containing("[profiles.dev]\nhost = \"dev.example.com\"\n", |_dir| {
            let result = handle_edit("dev", &None, &None, &None, &None, &None, &None);
            assert!(result.is_err());
            let err_msg = format!("{}", result.unwrap_err());
            assert!(err_msg.contains("At least one field"));
        });
    }

    #[test]
    fn test_edit_profile_invalid_logmech_fails() {
        with_test_config_containing("[profiles.dev]\nhost = \"dev.example.com\"\n", |_dir| {
            let result = handle_edit(
                "dev",
                &None,
                &None,
                &None,
                &None,
                &Some("BOGUS".to_string()),
                &None,
            );
            assert!(result.is_err());
        });
    }

    // =========================================================================
    // Profile delete tests
    // =========================================================================

    #[test]
    fn test_delete_profile_with_force() {
        with_test_config_containing("[profiles.dev]\nhost = \"dev.example.com\"\n", |dir| {
            let result = handle_delete("dev", true);
            assert!(result.is_ok());

            let content = std::fs::read_to_string(dir.join("config.toml")).unwrap();
            assert!(!content.contains("[profiles.dev]"));
            assert!(!content.contains("dev.example.com"));
        });
    }

    #[test]
    fn test_delete_profile_without_force_fails() {
        with_test_config_containing("[profiles.dev]\nhost = \"dev.example.com\"\n", |_dir| {
            let result = handle_delete("dev", false);
            assert!(result.is_err());
            let err_msg = format!("{}", result.unwrap_err());
            assert!(err_msg.contains("--force"));
        });
    }

    #[test]
    fn test_delete_profile_nonexistent_fails() {
        with_test_config_dir(|_dir| {
            let result = handle_delete("nonexistent", true);
            assert!(result.is_err());
            let err_msg = format!("{}", result.unwrap_err());
            assert!(err_msg.contains("does not exist"));
        });
    }

    // =========================================================================
    // Delete confirmation tests
    // =========================================================================

    #[test]
    fn test_confirm_deletion_tty_yes() {
        let mut input = std::io::Cursor::new(b"y\n");
        let result = confirm_deletion("dev", true, &mut input);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_confirm_deletion_tty_yes_uppercase() {
        let mut input = std::io::Cursor::new(b"Y\n");
        let result = confirm_deletion("dev", true, &mut input);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_confirm_deletion_tty_no() {
        let mut input = std::io::Cursor::new(b"n\n");
        let result = confirm_deletion("dev", true, &mut input);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_confirm_deletion_tty_empty() {
        let mut input = std::io::Cursor::new(b"\n");
        let result = confirm_deletion("dev", true, &mut input);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Default is No
    }

    #[test]
    fn test_confirm_deletion_non_tty_errors() {
        let mut input = std::io::Cursor::new(b"");
        let result = confirm_deletion("dev", false, &mut input);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("--force"));
    }

    // =========================================================================
    // List tests
    // =========================================================================

    #[test]
    fn test_list_empty_profiles() {
        let config = Config::default();
        let result = handle_list(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_with_profiles() {
        let mut config = Config::default();
        config.profiles.insert(
            "dev".to_string(),
            crate::config::ConnectionSettings {
                host: Some("dev.example.com".to_string()),
                database: Some("devdb".to_string()),
                user: Some("alice".to_string()),
                ..Default::default()
            },
        );
        let result = handle_list(&config);
        assert!(result.is_ok());
    }

    // =========================================================================
    // Config dir tests
    // =========================================================================

    #[test]
    fn test_config_dir_uses_env_var() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let original = std::env::var("TQ_CONFIG_DIR").ok();

        std::env::set_var("TQ_CONFIG_DIR", "/tmp/test-tq-config");
        assert_eq!(config_dir(), PathBuf::from("/tmp/test-tq-config"));

        match original {
            Some(val) => std::env::set_var("TQ_CONFIG_DIR", val),
            None => std::env::remove_var("TQ_CONFIG_DIR"),
        }
    }

    // =========================================================================
    // Read/write roundtrip tests
    // =========================================================================

    #[test]
    fn test_add_then_edit_then_delete() {
        with_test_config_dir(|_dir| {
            // Add
            let result = handle_add(
                "test",
                "test.example.com",
                &Some(1025),
                &Some("testdb".to_string()),
                &Some("alice".to_string()),
                &None,
                &None,
            );
            assert!(result.is_ok());

            // Edit
            let result = handle_edit(
                "test",
                &None,
                &Some(2025),
                &None,
                &None,
                &None,
                &None,
            );
            assert!(result.is_ok());

            // Verify edit took effect
            let table = read_config_table().unwrap();
            let profiles = table.get("profiles").unwrap().as_table().unwrap();
            let profile = profiles.get("test").unwrap().as_table().unwrap();
            assert_eq!(
                profile.get("port").unwrap().as_integer().unwrap(),
                2025
            );
            assert_eq!(
                profile.get("host").unwrap().as_str().unwrap(),
                "test.example.com"
            );

            // Delete
            let result = handle_delete("test", true);
            assert!(result.is_ok());

            // Verify deleted
            let table = read_config_table().unwrap();
            let profiles = table.get("profiles").unwrap().as_table().unwrap();
            assert!(!profiles.contains_key("test"));
        });
    }

    #[test]
    fn test_add_multiple_profiles_preserved() {
        with_test_config_dir(|dir| {
            // Add first profile
            handle_add(
                "dev",
                "dev.example.com",
                &None,
                &None,
                &None,
                &None,
                &None,
            )
            .unwrap();

            // Add second profile
            handle_add(
                "prod",
                "prod.example.com",
                &None,
                &None,
                &None,
                &None,
                &None,
            )
            .unwrap();

            let content = std::fs::read_to_string(dir.join("config.toml")).unwrap();
            assert!(content.contains("[profiles.dev]"));
            assert!(content.contains("[profiles.prod]"));
            assert!(content.contains("dev.example.com"));
            assert!(content.contains("prod.example.com"));
        });
    }

    #[test]
    fn test_existing_non_profile_config_preserved() {
        with_test_config_containing(
            "[output]\nformat = \"json\"\n\n[profiles.existing]\nhost = \"old.example.com\"\n",
            |dir| {
                handle_add(
                    "new",
                    "new.example.com",
                    &None,
                    &None,
                    &None,
                    &None,
                    &None,
                )
                .unwrap();

                let content = std::fs::read_to_string(dir.join("config.toml")).unwrap();
                // Existing output section preserved
                assert!(content.contains("format = \"json\""));
                // Existing profile preserved
                assert!(content.contains("old.example.com"));
                // New profile added
                assert!(content.contains("new.example.com"));
            },
        );
    }
}
