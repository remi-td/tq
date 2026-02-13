//! Configuration management for tq
//!
//! Configuration is loaded from multiple sources in priority order:
//! 1. CLI arguments (highest priority)
//! 2. Environment variables (TQ_*)
//! 3. Project config (.tq.toml)
//! 4. User config (~/.config/tq/config.toml)
//! 5. System config (/etc/tq/config.toml)
//! 6. Built-in defaults (lowest priority)

use crate::cli::{GlobalOpts, LogonMechanism, OutputFormat};
use crate::db::{parse_duration, ConnectionConfig};
use crate::error::{Result, TqError};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Complete application configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Connection settings
    pub connection: ConnectionSettings,

    /// Output preferences
    pub output: OutputSettings,

    /// REPL settings (for future use)
    pub repl: ReplSettings,

    /// Named connection profiles
    #[serde(default)]
    pub profiles: HashMap<String, ConnectionSettings>,
}

/// Connection-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConnectionSettings {
    /// Database host
    pub host: Option<String>,

    /// Database port
    pub port: Option<u16>,

    /// Database name
    pub database: Option<String>,

    /// Username
    pub user: Option<String>,

    /// Authentication mechanism
    pub logmech: Option<String>,

    /// Connection timeout (e.g., "30s")
    pub timeout: Option<String>,

    /// Path to password file (supports ~ for home directory)
    pub password_file: Option<PathBuf>,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            host: None,
            port: Some(1025),
            database: None,
            user: None,
            logmech: Some("TD2".to_string()),
            timeout: Some("30s".to_string()),
            password_file: None,
        }
    }
}

/// Output-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputSettings {
    /// Default output format
    pub format: String,

    /// Color output control
    pub color: String,

    /// Show query timing by default
    pub timing: bool,
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            format: "table".to_string(),
            color: "auto".to_string(),
            timing: false,
        }
    }
}

/// REPL-related settings (for future use)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplSettings {
    /// History file path
    pub history_file: PathBuf,

    /// Maximum history entries
    pub history_size: usize,

    /// Editor mode (emacs/vi)
    pub editor_mode: String,

    /// Enable syntax highlighting
    pub syntax_highlight: bool,

    /// Enable autocomplete
    pub autocomplete: bool,
}

impl Default for ReplSettings {
    fn default() -> Self {
        Self {
            history_file: PathBuf::from("~/.tq_history"),
            history_size: 10000,
            editor_mode: "emacs".to_string(),
            syntax_highlight: true,
            autocomplete: true,
        }
    }
}

impl Config {
    /// Load configuration from all sources
    ///
    /// Configuration is loaded in precedence order (later sources override earlier):
    /// 1. Built-in defaults
    /// 2. System config (/etc/tq/config.toml)
    /// 3. User config (~/.tq/config.toml)
    /// 4. Project config (.tq.toml - searched in current and ancestor directories)
    /// 5. Environment variables (TQ_*)
    ///
    /// CLI arguments are applied at runtime, not in this function.
    pub fn load() -> Result<Self> {
        // Find project config by walking up the directory tree
        let project_config_path = find_project_config();

        let mut figment = Figment::new()
            // Built-in defaults
            .merge(Serialized::defaults(Config::default()))
            // System config (no .nested() - we want standard TOML section parsing)
            .merge(Toml::file("/etc/tq/config.toml"))
            // User config (no .nested() - profiles are [profiles.name] sections, not figment profiles)
            .merge(Toml::file(Self::user_config_path()));

        // Project config (if found via directory traversal)
        // This overrides user config, enabling team-shared settings
        if let Some(ref path) = project_config_path {
            log::debug!("Found project config: {}", path.display());
            figment = figment.merge(Toml::file(path));
        }

        // Environment variables (TQ_HOST, TQ_PORT, etc.)
        figment = figment.merge(Env::prefixed("TQ_").split("_").lowercase(false));

        let config: Config = figment
            .extract()
            .map_err(|e| TqError::ConfigParseError(e.to_string()))?;

        Ok(config)
    }

    /// Get the path to the project config file, if found
    ///
    /// Searches for `.tq.toml` in current directory and ancestors.
    /// Returns None if no project config exists.
    pub fn project_config_path() -> Option<PathBuf> {
        find_project_config()
    }

    /// Load only user configuration (for source tracking)
    ///
    /// Used by `tq profiles` to show which profiles come from user config.
    /// Returns default config if user config file doesn't exist.
    pub fn load_user_only() -> Self {
        let figment = Figment::new()
            .merge(Serialized::defaults(Config::default()))
            .merge(Toml::file(Self::user_config_path()));

        figment.extract().unwrap_or_default()
    }

    /// Load only project configuration (for source tracking)
    ///
    /// Used by `tq profiles` to show which profiles come from project config.
    /// Returns None if no project config file exists.
    pub fn load_project_only() -> Option<Self> {
        let path = find_project_config()?;

        let figment = Figment::new()
            .merge(Serialized::defaults(Config::default()))
            .merge(Toml::file(path));

        figment.extract().ok()
    }

    /// Get user config file path
    ///
    /// Returns ~/.tq/config.toml on macOS/Linux, %USERPROFILE%\.tq\config.toml on Windows
    pub fn user_config_path() -> PathBuf {
        if let Some(user_dirs) = directories::UserDirs::new() {
            user_dirs.home_dir().join(".tq").join("config.toml")
        } else {
            PathBuf::from("~/.tq/config.toml")
        }
    }

    /// Get a connection profile by name
    pub fn get_profile(&self, name: &str) -> Option<&ConnectionSettings> {
        self.profiles.get(name)
    }

    /// Build ConnectionConfig from settings and CLI overrides
    pub fn build_connection_config(
        &self,
        global: &GlobalOpts,
        password_override: Option<String>,
    ) -> Result<ConnectionConfig> {
        // If logon string is provided, parse it directly
        if let Some(ref logon) = global.logon {
            let timeout = parse_duration(&global.timeout)?;
            return ConnectionConfig::from_connection_string(
                logon,
                global.logmech,
                timeout,
                password_override,
            );
        }

        // Otherwise, build from config + CLI overrides
        let conn = &self.connection;

        let host = conn
            .host
            .clone()
            .ok_or_else(|| TqError::InvalidConfig("No host specified".to_string()))?;

        let port = conn.port.unwrap_or(1025);

        let database = conn
            .database
            .clone()
            .ok_or_else(|| TqError::InvalidConfig("No database specified".to_string()))?;

        let user = conn
            .user
            .clone()
            .ok_or_else(|| TqError::InvalidConfig("No user specified".to_string()))?;

        let logmech = conn
            .logmech
            .as_deref()
            .and_then(|s| parse_logmech(s).ok())
            .unwrap_or(global.logmech);

        let timeout_str = conn.timeout.as_deref().unwrap_or(&global.timeout);
        let timeout = parse_duration(timeout_str)?;

        let password = password_override.map(secrecy::Secret::new);

        Ok(ConnectionConfig {
            host,
            port,
            database,
            user,
            password,
            logmech,
            timeout,
        })
    }
}

/// Parse logon mechanism from string (case-insensitive)
///
/// # Examples
///
/// ```
/// use tq::config::parse_logmech;
/// use tq::cli::LogonMechanism;
///
/// assert_eq!(parse_logmech("TD2").unwrap(), LogonMechanism::Td2);
/// assert_eq!(parse_logmech("ldap").unwrap(), LogonMechanism::Ldap);
/// assert!(parse_logmech("invalid").is_err());
/// ```
pub fn parse_logmech(s: &str) -> Result<LogonMechanism> {
    match s.to_uppercase().as_str() {
        "TD2" => Ok(LogonMechanism::Td2),
        "LDAP" => Ok(LogonMechanism::Ldap),
        "KRB5" => Ok(LogonMechanism::Krb5),
        "TDNEGO" => Ok(LogonMechanism::Tdnego),
        _ => Err(TqError::InvalidLogonMechanism(s.to_string())),
    }
}

/// Expand ~ to home directory in a path
pub fn expand_home_dir(path: &std::path::Path) -> PathBuf {
    if let Some(path_str) = path.to_str() {
        if path_str.starts_with("~/") || path_str == "~" {
            if let Some(home) = directories::UserDirs::new().map(|d| d.home_dir().to_path_buf()) {
                if path_str == "~" {
                    return home;
                }
                return home.join(&path_str[2..]);
            }
        }
    }
    path.to_path_buf()
}

/// Read password from file with security checks
///
/// Verifies file permissions are secure (0600) before reading.
/// Expands ~ in the path to home directory.
pub fn read_password_from_file(path: &std::path::Path) -> Result<String> {
    let expanded_path = expand_home_dir(path);

    // Check file permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let metadata = std::fs::metadata(&expanded_path).map_err(|e| TqError::FileReadError {
            path: expanded_path.clone(),
            source: e,
        })?;

        let mode = metadata.permissions().mode() & 0o777;

        // Reject if group or world readable/writable
        if mode & 0o077 != 0 {
            return Err(TqError::InvalidConfig(format!(
                "Password file '{}' has insecure permissions {:04o}. Required: 0600.\n\
                 Fix: chmod 0600 {}",
                expanded_path.display(),
                mode,
                expanded_path.display()
            )));
        }
    }

    // Read the password
    let password = std::fs::read_to_string(&expanded_path).map_err(|e| TqError::FileReadError {
        path: expanded_path.clone(),
        source: e,
    })?;

    // Return trimmed password (remove trailing newline)
    Ok(password.trim().to_string())
}

/// Get output format from config and CLI
pub fn resolve_output_format(_config: &Config, cli_format: OutputFormat) -> OutputFormat {
    // CLI takes precedence
    // For now, just return the CLI format since it has defaults
    cli_format
}

/// Find project configuration file by walking up the directory tree
///
/// Searches for `.tq.toml` starting from the current working directory
/// and walking up to parent directories until found or filesystem root reached.
///
/// # Returns
///
/// - `Some(PathBuf)` if `.tq.toml` is found in current or ancestor directory
/// - `None` if no project config exists or current directory cannot be determined
///
/// # Algorithm
///
/// 1. Start from current working directory
/// 2. Check if `.tq.toml` exists and is a file
/// 3. If found, return the path
/// 4. Otherwise, move to parent directory and repeat
/// 5. Stop when filesystem root is reached (parent returns None)
///
/// # Security
///
/// Uses `is_file()` check which does not follow symlinks by default,
/// preventing potential symlink attacks.
///
/// # Examples
///
/// ```no_run
/// use tq::config::find_project_config;
///
/// // Returns Some if .tq.toml exists in current dir or any ancestor
/// if let Some(config_path) = find_project_config() {
///     println!("Found project config: {}", config_path.display());
/// }
/// ```
pub fn find_project_config() -> Option<PathBuf> {
    // Get current working directory
    let cwd = std::env::current_dir().ok()?;

    let mut current = cwd.as_path();
    loop {
        let candidate = current.join(".tq.toml");

        // Check if the candidate is a file (not a directory or symlink to directory)
        if candidate.is_file() {
            return Some(candidate);
        }

        // Move to parent directory
        // Returns None at filesystem root, which terminates the loop
        current = current.parent()?;
    }
}

/// Check if output should use color
pub fn should_use_color(_config: &Config, cli_color: &crate::cli::ColorChoice) -> bool {
    cli_color.should_use_color()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.connection.port, Some(1025));
        assert_eq!(config.connection.logmech, Some("TD2".to_string()));
        assert_eq!(config.output.format, "table");
        assert_eq!(config.repl.history_size, 10000);
    }

    #[test]
    fn test_parse_logmech() {
        assert_eq!(parse_logmech("TD2").unwrap(), LogonMechanism::Td2);
        assert_eq!(parse_logmech("td2").unwrap(), LogonMechanism::Td2);
        assert_eq!(parse_logmech("LDAP").unwrap(), LogonMechanism::Ldap);
        assert_eq!(parse_logmech("KRB5").unwrap(), LogonMechanism::Krb5);
        assert_eq!(parse_logmech("TDNEGO").unwrap(), LogonMechanism::Tdnego);
        assert!(parse_logmech("INVALID").is_err());
    }

    #[test]
    fn test_user_config_path() {
        let path = Config::user_config_path();
        // Should end with config.toml
        assert!(path.ends_with("config.toml"));
    }

    #[test]
    fn test_connection_settings_default() {
        let settings = ConnectionSettings::default();
        assert_eq!(settings.port, Some(1025));
        assert!(settings.host.is_none());
        assert!(settings.user.is_none());
        assert!(settings.password_file.is_none());
    }

    #[test]
    fn test_expand_home_dir_with_tilde() {
        let path = std::path::Path::new("~/test/file.txt");
        let expanded = expand_home_dir(path);

        // Should not start with ~ anymore
        let expanded_str = expanded.to_str().unwrap();
        assert!(!expanded_str.starts_with("~"));
        // Should end with test/file.txt
        assert!(expanded_str.ends_with("test/file.txt"));
    }

    #[test]
    fn test_expand_home_dir_without_tilde() {
        let path = std::path::Path::new("/absolute/path/file.txt");
        let expanded = expand_home_dir(path);
        assert_eq!(expanded, std::path::PathBuf::from("/absolute/path/file.txt"));
    }

    #[test]
    fn test_expand_home_dir_tilde_only() {
        let path = std::path::Path::new("~");
        let expanded = expand_home_dir(path);

        // Should be home directory (not "~")
        let expanded_str = expanded.to_str().unwrap();
        assert!(!expanded_str.contains("~"));
    }

    #[test]
    fn test_user_config_path_in_tq_dir() {
        let path = Config::user_config_path();
        // Should be in .tq directory
        let path_str = path.to_str().unwrap();
        assert!(path_str.contains(".tq"));
        assert!(path_str.ends_with("config.toml"));
    }

    #[test]
    fn test_config_with_profiles() {
        let mut config = Config::default();
        let profile = ConnectionSettings {
            host: Some("test.example.com".to_string()),
            database: Some("testdb".to_string()),
            user: Some("testuser".to_string()),
            password_file: Some(PathBuf::from("~/.tq/passwords/test")),
            ..Default::default()
        };

        config.profiles.insert("test".to_string(), profile);

        // Should be able to get the profile
        let retrieved = config.get_profile("test");
        assert!(retrieved.is_some());
        let p = retrieved.unwrap();
        assert_eq!(p.host.as_deref(), Some("test.example.com"));
        assert_eq!(
            p.password_file.as_ref().map(|p| p.to_str().unwrap()),
            Some("~/.tq/passwords/test")
        );

        // Non-existent profile returns None
        assert!(config.get_profile("nonexistent").is_none());
    }

    #[test]
    fn test_read_password_file_not_found() {
        let path = std::path::Path::new("/nonexistent/password/file");
        let result = read_password_from_file(path);
        assert!(result.is_err());
        // Should be a FileReadError
        if let Err(TqError::FileReadError { path: p, .. }) = result {
            assert!(p.to_str().unwrap().contains("nonexistent"));
        } else {
            panic!("Expected FileReadError");
        }
    }

    // =========================================================================
    // Project config tests (find_project_config and related functionality)
    // =========================================================================

    // Note: These tests need special handling because:
    // 1. Tests run in parallel and share the process's current directory
    // 2. On macOS, /var is a symlink to /private/var, causing path comparison issues
    //
    // We use a mutex to serialize tests that change the current directory,
    // and canonicalize paths before comparison.

    use std::sync::Mutex;

    // Global mutex to serialize tests that change current directory
    static CWD_MUTEX: Mutex<()> = Mutex::new(());

    /// Helper to run a test in a specific directory context
    /// Uses a mutex to prevent parallel tests from interfering with each other
    fn with_current_dir<F, T>(dir: &std::path::Path, test_fn: F) -> T
    where
        F: FnOnce() -> T,
    {
        let _lock = CWD_MUTEX.lock().unwrap();
        let original_dir = std::env::current_dir().expect("Failed to get current dir");
        std::env::set_current_dir(dir).expect("Failed to change dir");
        let result = test_fn();
        std::env::set_current_dir(original_dir).expect("Failed to restore dir");
        result
    }

    /// Canonicalize path for comparison (handles macOS /var -> /private/var symlink)
    fn canonical(path: &std::path::Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }

    #[test]
    fn test_find_project_config_in_current_directory() {
        // Create temp directory with .tq.toml
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(".tq.toml");
        std::fs::write(&config_path, "[profiles.test]\nhost = \"test\"").unwrap();

        // Test from the directory containing .tq.toml
        let found = with_current_dir(temp.path(), find_project_config);

        assert!(found.is_some());
        // Use canonical paths for comparison (macOS /var symlink issue)
        assert_eq!(canonical(&found.unwrap()), canonical(&config_path));
    }

    #[test]
    fn test_find_project_config_walks_up_to_parent() {
        // Create nested structure: temp/.tq.toml, temp/subdir/
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(".tq.toml");
        std::fs::write(&config_path, "[profiles.test]\nhost = \"test\"").unwrap();

        let subdir = temp.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        // Test from subdir - should find .tq.toml in parent
        let found = with_current_dir(&subdir, find_project_config);

        assert!(found.is_some());
        assert_eq!(canonical(&found.unwrap()), canonical(&config_path));
    }

    #[test]
    fn test_find_project_config_walks_up_multiple_levels() {
        // Create: temp/.tq.toml, temp/a/b/c/
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(".tq.toml");
        std::fs::write(&config_path, "[profiles.test]\nhost = \"test\"").unwrap();

        let deep_dir = temp.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&deep_dir).unwrap();

        // Test from deeply nested dir - should find .tq.toml in ancestor
        let found = with_current_dir(&deep_dir, find_project_config);

        assert!(found.is_some());
        assert_eq!(canonical(&found.unwrap()), canonical(&config_path));
    }

    #[test]
    fn test_find_project_config_stops_at_first_found() {
        // Create: temp/.tq.toml, temp/inner/.tq.toml
        let temp = tempfile::tempdir().unwrap();

        let outer_config = temp.path().join(".tq.toml");
        std::fs::write(&outer_config, "[profiles.outer]\nhost = \"outer\"").unwrap();

        let inner_dir = temp.path().join("inner");
        std::fs::create_dir(&inner_dir).unwrap();
        let inner_config = inner_dir.join(".tq.toml");
        std::fs::write(&inner_config, "[profiles.inner]\nhost = \"inner\"").unwrap();

        // Test from inner dir - should find inner/.tq.toml, not outer
        let found = with_current_dir(&inner_dir, find_project_config);

        assert!(found.is_some());
        assert_eq!(canonical(&found.unwrap()), canonical(&inner_config));
    }

    #[test]
    fn test_find_project_config_returns_none_when_not_found() {
        // Create temp dir without any .tq.toml
        let temp = tempfile::tempdir().unwrap();

        // Test from directory with no .tq.toml in any ancestor
        let found = with_current_dir(temp.path(), find_project_config);

        assert!(found.is_none());
    }

    #[test]
    fn test_find_project_config_ignores_directory_named_tq_toml() {
        // Create a directory named .tq.toml (should be ignored)
        let temp = tempfile::tempdir().unwrap();
        let dir_named_config = temp.path().join(".tq.toml");
        std::fs::create_dir(&dir_named_config).unwrap();

        // Should return None because .tq.toml is a directory, not a file
        let found = with_current_dir(temp.path(), find_project_config);

        assert!(found.is_none());
    }

    #[test]
    fn test_find_project_config_with_valid_toml_content() {
        // Test that we can actually parse the found config
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(".tq.toml");
        let toml_content = r#"
[profiles.dev]
host = "dev.example.com"
port = 1025
database = "devdb"
user = "devuser"
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let found = with_current_dir(temp.path(), find_project_config);
        assert!(found.is_some());

        // Verify we can parse it
        let content = std::fs::read_to_string(found.unwrap()).unwrap();
        let parsed: toml::Table = toml::from_str(&content).unwrap();
        assert!(parsed.contains_key("profiles"));
    }

    #[test]
    fn test_project_config_path_method() {
        // Test the Config::project_config_path() method
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(".tq.toml");
        std::fs::write(&config_path, "").unwrap();

        let found = with_current_dir(temp.path(), Config::project_config_path);
        assert!(found.is_some());
        assert_eq!(canonical(&found.unwrap()), canonical(&config_path));
    }

    #[test]
    fn test_project_config_path_returns_none_when_no_config() {
        let temp = tempfile::tempdir().unwrap();

        let found = with_current_dir(temp.path(), Config::project_config_path);
        assert!(found.is_none());
    }

    #[test]
    fn test_load_project_only_with_profiles() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(".tq.toml");
        let toml_content = r#"
[profiles.project_profile]
host = "project.example.com"
database = "projectdb"
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let config = with_current_dir(temp.path(), Config::load_project_only);

        assert!(config.is_some());
        let cfg = config.unwrap();
        assert!(cfg.profiles.contains_key("project_profile"));
        let profile = cfg.profiles.get("project_profile").unwrap();
        assert_eq!(profile.host.as_deref(), Some("project.example.com"));
        assert_eq!(profile.database.as_deref(), Some("projectdb"));
    }

    #[test]
    fn test_load_project_only_returns_none_when_no_config() {
        let temp = tempfile::tempdir().unwrap();

        let config = with_current_dir(temp.path(), Config::load_project_only);
        assert!(config.is_none());
    }
}
