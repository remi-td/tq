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
    pub fn load() -> Result<Self> {
        let figment = Figment::new()
            // Built-in defaults
            .merge(Serialized::defaults(Config::default()))
            // System config
            .merge(Toml::file("/etc/tq/config.toml").nested())
            // User config
            .merge(Toml::file(Self::user_config_path()).nested())
            // Project config
            .merge(Toml::file(".tq.toml").nested())
            // Environment variables (TQ_HOST, TQ_PORT, etc.)
            .merge(Env::prefixed("TQ_").split("_").lowercase(false));

        let config: Config = figment
            .extract()
            .map_err(|e| TqError::ConfigParseError(e.to_string()))?;

        Ok(config)
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

/// Parse logon mechanism from string
fn parse_logmech(s: &str) -> Result<LogonMechanism> {
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
}
