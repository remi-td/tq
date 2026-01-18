//! Database connection configuration and management
//!
//! This module provides secure connection configuration with support for
//! multiple password sources and validation.

use crate::cli::LogonMechanism;
use crate::error::{Result, TqError};
use secrecy::{ExposeSecret, Secret};
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Database connection configuration with secure credential handling
#[derive(Clone)]
pub struct ConnectionConfig {
    /// Database host
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database name
    pub database: String,
    /// Username
    pub user: String,
    /// Password (securely wrapped)
    pub password: Option<Secret<String>>,
    /// Authentication mechanism
    pub logmech: LogonMechanism,
    /// Connection timeout
    pub timeout: Duration,
}

impl std::fmt::Debug for ConnectionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .field("user", &self.user)
            .field("password", &"[REDACTED]")
            .field("logmech", &self.logmech)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl std::fmt::Display for ConnectionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{}:{}/{}",
            self.user, self.host, self.port, self.database
        )
    }
}

impl ConnectionConfig {
    /// Parse connection string: user:password@host:port/database
    ///
    /// Password can be omitted if provided via password_override.
    pub fn from_connection_string(
        logon: &str,
        logmech: LogonMechanism,
        timeout: Duration,
        password_override: Option<String>,
    ) -> Result<Self> {
        // Split on @ to separate credentials from host
        let at_pos = logon.rfind('@').ok_or_else(|| {
            TqError::InvalidConnectionString(
                "Expected format: user:password@host:port/database".to_string(),
            )
        })?;

        let credentials = &logon[..at_pos];
        let host_info = &logon[at_pos + 1..];

        // Parse credentials (user:password or just user)
        let (user, password_from_string) = if let Some(colon_pos) = credentials.find(':') {
            let user = credentials[..colon_pos].to_string();
            let pass = credentials[colon_pos + 1..].to_string();
            (user, Some(pass))
        } else {
            (credentials.to_string(), None)
        };

        // Validate user
        Self::validate_identifier(&user, "Username")?;

        // Determine password source (override takes precedence)
        let password = password_override.or(password_from_string).map(Secret::new);

        // Parse host info (host:port/database)
        let slash_pos = host_info.find('/').ok_or_else(|| {
            TqError::InvalidConnectionString(
                "Host info must include database: host:port/database".to_string(),
            )
        })?;

        let host_port = &host_info[..slash_pos];
        let database = host_info[slash_pos + 1..].to_string();

        // Validate database name
        Self::validate_identifier(&database, "Database")?;

        // Parse host and port
        let colon_pos = host_port.rfind(':').ok_or_else(|| {
            TqError::InvalidConnectionString("Host must include port: host:port".to_string())
        })?;

        let host = host_port[..colon_pos].to_string();
        Self::validate_host(&host)?;

        let port = host_port[colon_pos + 1..]
            .parse::<u16>()
            .map_err(|_| TqError::InvalidConnectionString("Invalid port number".to_string()))?;

        Ok(Self {
            host,
            port,
            database,
            user,
            password,
            logmech,
            timeout,
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.host.is_empty() {
            return Err(TqError::InvalidConfig("Host cannot be empty".to_string()));
        }
        if self.port == 0 {
            return Err(TqError::InvalidConfig("Port must be non-zero".to_string()));
        }
        if self.user.is_empty() {
            return Err(TqError::InvalidConfig("User cannot be empty".to_string()));
        }
        if self.database.is_empty() {
            return Err(TqError::InvalidConfig(
                "Database cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    /// Resolve password from various sources
    ///
    /// Priority:
    /// 1. Already set password
    /// 2. Password file (specific or ~/.tq_passwords)
    /// 3. TQ_PASSWORD environment variable
    /// 4. Error (no interactive prompt in batch mode)
    pub fn resolve_password(&mut self, password_file: Option<&Path>) -> Result<()> {
        if self.password.is_some() {
            return Ok(());
        }

        // Try password file
        if let Some(path) = password_file {
            if let Some(pw) = self.read_password_file(path)? {
                self.password = Some(Secret::new(pw));
                return Ok(());
            }
        }

        // Try default password file (~/.tq_passwords)
        if let Some(home) = dirs::home_dir() {
            let default_path = home.join(".tq_passwords");
            if default_path.exists() {
                if let Some(pw) = self.read_password_file(&default_path)? {
                    self.password = Some(Secret::new(pw));
                    return Ok(());
                }
            }
        }

        // Try environment variable
        if let Ok(pw) = std::env::var("TQ_PASSWORD") {
            log::warn!("Using TQ_PASSWORD environment variable. Consider using --password-file for better security.");
            self.password = Some(Secret::new(pw));
            return Ok(());
        }

        Err(TqError::MissingPassword)
    }

    /// Read password from file
    ///
    /// Supports two formats:
    /// 1. Single password per file (entire content minus whitespace)
    /// 2. pgpass-style: hostname:port:database:username:password
    fn read_password_file(&self, path: &Path) -> Result<Option<String>> {
        // Check file permissions on Unix
        #[cfg(unix)]
        self.check_file_permissions(path)?;

        let content = fs::read_to_string(path).map_err(|e| TqError::FileReadError {
            path: path.to_path_buf(),
            source: e,
        })?;

        // Try pgpass-style format first
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Try to parse as pgpass format
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 5 {
                let (host, port, db, user, password) = (
                    parts[0],
                    parts[1],
                    parts[2],
                    parts[3],
                    parts[4..].join(":"), // Password may contain colons
                );

                // Match against connection parameters (* is wildcard)
                if (host == "*" || host == self.host)
                    && (port == "*" || port == self.port.to_string())
                    && (db == "*" || db == self.database)
                    && (user == "*" || user == self.user)
                {
                    return Ok(Some(password));
                }
            }
        }

        // Try single-password format (first non-comment line)
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                return Ok(Some(line.to_string()));
            }
        }

        Ok(None)
    }

    /// Check file permissions on Unix systems
    #[cfg(unix)]
    fn check_file_permissions(&self, path: &Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(path).map_err(|e| TqError::FileReadError {
            path: path.to_path_buf(),
            source: e,
        })?;
        let mode = metadata.permissions().mode() & 0o777;

        if mode & 0o077 != 0 {
            log::warn!(
                "Password file '{}' has insecure permissions {:o}. Recommended: 0600",
                path.display(),
                mode
            );
        }

        Ok(())
    }

    /// Convert to JSON connection string for teradatarustapi
    pub fn to_json_string(&self) -> String {
        let password = self
            .password
            .as_ref()
            .map(|p| p.expose_secret().as_str())
            .unwrap_or("");

        format!(
            r#"{{"host":"{}","user":"{}","password":"{}","dbs_port":"{}","database":"{}","logmech":"{}"}}"#,
            self.host, self.user, password, self.port, self.database, self.logmech
        )
    }

    /// Validate an identifier (username, database name)
    fn validate_identifier(s: &str, name: &str) -> Result<()> {
        if s.is_empty() {
            return Err(TqError::InvalidConnectionString(format!(
                "{} cannot be empty",
                name
            )));
        }
        if s.contains(['\0', '\n', '\r']) {
            return Err(TqError::InvalidConnectionString(format!(
                "{} contains invalid characters",
                name
            )));
        }
        Ok(())
    }

    /// Validate hostname
    fn validate_host(host: &str) -> Result<()> {
        if host.is_empty() {
            return Err(TqError::InvalidConnectionString(
                "Host cannot be empty".to_string(),
            ));
        }
        if host.contains(['\0', '\n', '\r', ' ']) {
            return Err(TqError::InvalidConnectionString(
                "Host contains invalid characters".to_string(),
            ));
        }
        Ok(())
    }
}

/// Parse duration string (e.g., "30s", "5m", "1h")
pub fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return Err(TqError::InvalidDuration(
            "Duration cannot be empty".to_string(),
        ));
    }

    // Find where the number ends and unit begins
    let num_end = s
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(s.len());

    if num_end == 0 {
        return Err(TqError::InvalidDuration(format!(
            "Invalid duration format: {}. Expected: 30s, 5m, 1h, 500ms",
            s
        )));
    }

    let num: f64 = s[..num_end]
        .parse()
        .map_err(|_| TqError::InvalidDuration(format!("Invalid number in duration: {}", s)))?;

    let unit = s[num_end..].trim().to_lowercase();

    let duration = match unit.as_str() {
        "ms" | "millis" | "milliseconds" => Duration::from_secs_f64(num / 1000.0),
        "s" | "sec" | "secs" | "second" | "seconds" | "" => Duration::from_secs_f64(num),
        "m" | "min" | "mins" | "minute" | "minutes" => Duration::from_secs_f64(num * 60.0),
        "h" | "hr" | "hrs" | "hour" | "hours" => Duration::from_secs_f64(num * 3600.0),
        _ => {
            return Err(TqError::InvalidDuration(format!(
                "Unknown duration unit: {}. Supported: ms, s, m, h",
                unit
            )))
        }
    };

    Ok(duration)
}

/// Use the directories crate for home directory
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_connection_string_full() {
        let config = ConnectionConfig::from_connection_string(
            "demo_user:demo_pass@localhost:1025/demo_db",
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();

        assert_eq!(config.user, "demo_user");
        assert_eq!(
            config.password.as_ref().unwrap().expose_secret(),
            "demo_pass"
        );
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 1025);
        assert_eq!(config.database, "demo_db");
    }

    #[test]
    fn test_parse_connection_string_with_override() {
        let config = ConnectionConfig::from_connection_string(
            "demo_user@localhost:1025/demo_db",
            LogonMechanism::Td2,
            Duration::from_secs(30),
            Some("override_pass".to_string()),
        )
        .unwrap();

        assert_eq!(
            config.password.as_ref().unwrap().expose_secret(),
            "override_pass"
        );
    }

    #[test]
    fn test_parse_connection_string_password_with_colon() {
        let config = ConnectionConfig::from_connection_string(
            "user:pass:word@host:1025/db",
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();

        assert_eq!(config.user, "user");
        assert_eq!(
            config.password.as_ref().unwrap().expose_secret(),
            "pass:word"
        );
    }

    #[test]
    fn test_parse_connection_string_invalid() {
        assert!(ConnectionConfig::from_connection_string(
            "invalid",
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .is_err());

        assert!(ConnectionConfig::from_connection_string(
            "user@host/db", // Missing port
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .is_err());
    }

    #[test]
    fn test_debug_redacts_password() {
        let config = ConnectionConfig::from_connection_string(
            "user:secret@host:1025/db",
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();

        let debug = format!("{:?}", config);
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn test_display_hides_password() {
        let config = ConnectionConfig::from_connection_string(
            "user:secret@host:1025/db",
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();

        let display = format!("{}", config);
        assert_eq!(display, "user@host:1025/db");
        assert!(!display.contains("secret"));
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(parse_duration("30").unwrap(), Duration::from_secs(30)); // Default to seconds
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("30x").is_err());
    }

    #[test]
    fn test_to_json_string() {
        let config = ConnectionConfig::from_connection_string(
            "testuser:testpass@testhost:1025/testdb",
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();

        let json = config.to_json_string();
        assert!(json.contains(r#""host":"testhost""#));
        assert!(json.contains(r#""user":"testuser""#));
        assert!(json.contains(r#""password":"testpass""#));
        assert!(json.contains(r#""dbs_port":"1025""#));
    }
}
