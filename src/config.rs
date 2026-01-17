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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for Config {
    fn default() -> Self {
        Self {
            connection: ConnectionSettings::default(),
            output: OutputSettings::default(),
            repl: ReplSettings::default(),
            profiles: HashMap::new(),
        }
    }
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
    pub fn user_config_path() -> PathBuf {
        directories::ProjectDirs::from("", "", "tq")
            .map(|d| d.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("~/.config/tq/config.toml"))
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
    }
}
