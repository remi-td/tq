use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Output format for query results
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table format
    Table,
    /// JSON format
    Json,
    /// CSV format
    Csv,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Csv => write!(f, "csv"),
        }
    }
}

/// A lightweight Rust command line client for Teradata databases
#[derive(Parser, Debug)]
#[command(name = "tq")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Database logon string format: user:password@host:port/database
    ///
    /// Note: If password is omitted from the connection string, you must use --password-file
    #[arg(short, long, required = true)]
    pub logon: String,

    /// Read password from file instead of connection string
    ///
    /// The file should contain only the password and have permissions set to 0600
    #[arg(long, value_name = "FILE")]
    pub password_file: Option<PathBuf>,

    /// Logon mechanism (default: TD2)
    ///
    /// Supported mechanisms: TD2, LDAP, KRB5, TDNEGO
    #[arg(long, default_value = "TD2")]
    pub logmech: String,

    /// Test database connectivity with a simple ping
    #[arg(long)]
    pub ping: bool,

    /// Output format for query results
    #[arg(long, default_value = "table")]
    pub format: OutputFormat,

    /// Directory containing the Teradata GoSQL driver library
    ///
    /// If not specified, uses the bundled library from the build
    #[arg(long)]
    pub driver_lib_dir: Option<String>,

    /// SQL query to execute
    ///
    /// Required unless --ping is specified
    #[arg(value_name = "QUERY")]
    pub query: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_with_ping() {
        let args = vec!["tq", "--logon", "user:pass@host:1025/db", "--ping"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.logon, "user:pass@host:1025/db");
        assert_eq!(cli.logmech, "TD2");
        assert!(cli.ping);
    }

    #[test]
    fn test_cli_parsing_with_logmech() {
        let args = vec![
            "tq",
            "--logon",
            "user:pass@host:1025/db",
            "--logmech",
            "LDAP",
            "--ping",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.logmech, "LDAP");
    }

    #[test]
    fn test_cli_missing_logon() {
        let args = vec!["tq", "--ping"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }
}
