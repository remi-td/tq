use crate::error::{Result, TqError};
use secrecy::{ExposeSecret, Secret};

/// Logon mechanism for Teradata authentication
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogonMechanism {
    /// Teradata 2 authentication
    TD2,
    /// LDAP authentication
    LDAP,
    /// Kerberos authentication
    KRB5,
    /// Teradata negotiation
    TDNEGO,
}

impl std::fmt::Display for LogonMechanism {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogonMechanism::TD2 => write!(f, "TD2"),
            LogonMechanism::LDAP => write!(f, "LDAP"),
            LogonMechanism::KRB5 => write!(f, "KRB5"),
            LogonMechanism::TDNEGO => write!(f, "TDNEGO"),
        }
    }
}

impl std::str::FromStr for LogonMechanism {
    type Err = TqError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "TD2" => Ok(LogonMechanism::TD2),
            "LDAP" => Ok(LogonMechanism::LDAP),
            "KRB5" => Ok(LogonMechanism::KRB5),
            "TDNEGO" => Ok(LogonMechanism::TDNEGO),
            _ => Err(TqError::Config(format!(
                "Invalid logon mechanism: {}. Supported: TD2, LDAP, KRB5, TDNEGO",
                s
            ))),
        }
    }
}

/// Parsed database connection configuration
#[derive(Clone)]
pub struct ConnectionConfig {
    pub user: String,
    pub password: Secret<String>,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub logmech: LogonMechanism,
}

impl std::fmt::Debug for ConnectionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionConfig")
            .field("user", &self.user)
            .field("password", &"[REDACTED]")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .field("logmech", &self.logmech)
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
    /// Parse a connection string in the format: user:password@host:port/database
    /// or user@host:port/database (when using --password-file)
    ///
    /// # Example
    /// ```
    /// use tq::connection::ConnectionConfig;
    /// use secrecy::ExposeSecret;
    ///
    /// let config = ConnectionConfig::parse(
    ///     "demo_user:demo_user@localhost:1025/demo_user",
    ///     "TD2",
    ///     None
    /// ).unwrap();
    /// assert_eq!(config.user, "demo_user");
    /// assert_eq!(config.host, "localhost");
    /// assert_eq!(config.port, 1025);
    /// ```
    pub fn parse(logon: &str, logmech: &str, password_override: Option<String>) -> Result<Self> {
        // Split on @ to separate credentials from host
        let parts: Vec<&str> = logon.split('@').collect();
        if parts.len() != 2 {
            return Err(TqError::InvalidConnectionString(
                "Expected format: user:password@host:port/database or user@host:port/database (with --password-file)".to_string(),
            ));
        }

        let credentials = parts[0];
        let host_info = parts[1];

        // Parse credentials (user:password or just user)
        let (user, password_from_string) = if credentials.contains(':') {
            let cred_parts: Vec<&str> = credentials.splitn(2, ':').collect();
            if cred_parts.len() != 2 {
                return Err(TqError::InvalidConnectionString(
                    "Credentials must be in format user:password".to_string(),
                ));
            }
            (cred_parts[0].to_string(), Some(cred_parts[1].to_string()))
        } else {
            (credentials.to_string(), None)
        };

        // Validate user
        Self::validate_identifier(&user, "Username")?;

        // Determine password source
        let password = if let Some(pwd) = password_override {
            pwd
        } else if let Some(pwd) = password_from_string {
            pwd
        } else {
            return Err(TqError::InvalidConnectionString(
                "Password must be provided either in connection string or via --password-file"
                    .to_string(),
            ));
        };

        // Parse host info (host:port/database)
        let host_db_parts: Vec<&str> = host_info.split('/').collect();
        if host_db_parts.len() != 2 {
            return Err(TqError::InvalidConnectionString(
                "Host info must include database: host:port/database".to_string(),
            ));
        }

        let host_port = host_db_parts[0];
        let database = host_db_parts[1].to_string();

        // Validate database name
        Self::validate_identifier(&database, "Database")?;

        // Parse host and port
        let hp_parts: Vec<&str> = host_port.split(':').collect();
        if hp_parts.len() != 2 {
            return Err(TqError::InvalidConnectionString(
                "Host must include port: host:port".to_string(),
            ));
        }

        let host = hp_parts[0].to_string();
        Self::validate_host(&host)?;

        let port = hp_parts[1].parse::<u16>().map_err(|_| {
            TqError::InvalidConnectionString(format!("Invalid port number: {}", hp_parts[1]))
        })?;

        // Parse logon mechanism
        let logmech = logmech.parse()?;

        Ok(ConnectionConfig {
            user,
            password: Secret::new(password),
            host,
            port,
            database,
            logmech,
        })
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

    /// Validate a hostname
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

    /// Convert to the JSON connection string format expected by teradatarustapi
    pub fn to_json_string(&self) -> String {
        format!(
            r#"{{"host":"{}","user":"{}","password":"{}","dbs_port":"{}","database":"{}","logmech":"{}"}}"#,
            self.host,
            self.user,
            self.password.expose_secret(),
            self.port,
            self.database,
            self.logmech
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_connection_string() {
        let config =
            ConnectionConfig::parse("demo_user:demo_pass@localhost:1025/demo_db", "TD2", None)
                .unwrap();

        assert_eq!(config.user, "demo_user");
        assert_eq!(config.password.expose_secret(), "demo_pass");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 1025);
        assert_eq!(config.database, "demo_db");
        assert_eq!(config.logmech, LogonMechanism::TD2);
    }

    #[test]
    fn test_parse_with_password_file() {
        let config = ConnectionConfig::parse(
            "demo_user@localhost:1025/demo_db",
            "TD2",
            Some("file_password".to_string()),
        )
        .unwrap();

        assert_eq!(config.user, "demo_user");
        assert_eq!(config.password.expose_secret(), "file_password");
        assert_eq!(config.host, "localhost");
    }

    #[test]
    fn test_parse_invalid_no_at_sign() {
        let result = ConnectionConfig::parse("user:pass", "TD2", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_no_password() {
        let result = ConnectionConfig::parse("user@host:1025/db", "TD2", None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Password must be provided"));
    }

    #[test]
    fn test_parse_invalid_port() {
        let result = ConnectionConfig::parse("user:pass@host:invalid/db", "TD2", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_logmech() {
        let result = ConnectionConfig::parse("user:pass@host:1025/db", "INVALID", None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid logon mechanism"));
    }

    #[test]
    fn test_to_json_string() {
        let config = ConnectionConfig {
            user: "testuser".to_string(),
            password: Secret::new("testpass".to_string()),
            host: "testhost".to_string(),
            port: 1025,
            database: "testdb".to_string(),
            logmech: LogonMechanism::TD2,
        };

        let json = config.to_json_string();
        assert!(json.contains(r#""host":"testhost""#));
        assert!(json.contains(r#""user":"testuser""#));
        assert!(json.contains(r#""dbs_port":"1025""#));
        assert!(json.contains(r#""password":"testpass""#));
    }

    #[test]
    fn test_debug_redacts_password() {
        let config = ConnectionConfig {
            user: "testuser".to_string(),
            password: Secret::new("supersecret".to_string()),
            host: "testhost".to_string(),
            port: 1025,
            database: "testdb".to_string(),
            logmech: LogonMechanism::TD2,
        };

        let debug_output = format!("{:?}", config);
        assert!(debug_output.contains("[REDACTED]"));
        assert!(!debug_output.contains("supersecret"));
    }

    #[test]
    fn test_display_format() {
        let config = ConnectionConfig {
            user: "testuser".to_string(),
            password: Secret::new("testpass".to_string()),
            host: "testhost".to_string(),
            port: 1025,
            database: "testdb".to_string(),
            logmech: LogonMechanism::TD2,
        };

        let display_output = format!("{}", config);
        assert_eq!(display_output, "testuser@testhost:1025/testdb");
        assert!(!display_output.contains("testpass"));
    }
}
