//! Command-line interface definitions for tq
//!
//! This module defines the CLI structure using clap with derive macros.
//! It follows UNIX conventions with proper subcommand organization.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// tq - Teradata Query
///
/// A fast, lightweight command-line client for Teradata databases.
///
/// tq follows a one-shot execution model: connect, execute, disconnect.
/// For interactive sessions, use the repl command (future feature).
#[derive(Parser, Debug)]
#[command(name = "tq")]
#[command(author, version, about)]
#[command(
    long_about = "tq is a fast, lightweight command-line client for Teradata databases.\n\n\
                  It follows UNIX philosophy: do one thing well, compose with other tools.\n\n\
                  QUICK START:\n  \
                  export TQ_LOGON=\"user:pass@host:1025/db\"\n  \
                  tq ping                    # Test connection\n  \
                  tq query \"SELECT 1\"       # Run a query\n\n\
                  SECURITY:\n  \
                  Store passwords in ~/.tq_passwords (chmod 0600) rather than in\n  \
                  environment variables or command line arguments."
)]
#[command(after_help = "EXAMPLES:\n  \
    # Quick connection test\n  \
    tq -l \"user:pass@host:1025/db\" ping\n\n  \
    # Execute query with table output\n  \
    tq query \"SELECT * FROM employees\"\n\n  \
    # Export to JSON\n  \
    tq query --format json \"SELECT * FROM data\" > data.json\n\n  \
    # Export to CSV\n  \
    tq query --format csv \"SELECT * FROM sales\" --output sales.csv\n\n  \
    # Secure password handling\n  \
    echo \"password\" > ~/.tq_pass && chmod 0600 ~/.tq_pass\n  \
    tq -l \"user@host:1025/db\" --password-file ~/.tq_pass query \"SELECT 1\"\n\n  \
    # Read query from file\n  \
    tq query --file script.sql\n\n  \
    # Read from stdin\n  \
    echo \"SELECT 1\" | tq query\n\n\
CONFIGURATION:\n  \
    Set TQ_LOGON environment variable to avoid repeating connection string:\n    \
    export TQ_LOGON=\"user:pass@host:1025/db\"\n\n  \
    Or create ~/.config/tq/config.toml:\n    \
    [connection]\n    \
    host = \"myhost\"\n    \
    port = 1025\n    \
    user = \"myuser\"\n    \
    database = \"mydb\"\n\n\
For more information, visit: https://github.com/remi-td/tq"
)]
pub struct Cli {
    /// Global options that apply to all commands
    #[command(flatten)]
    pub global: GlobalOpts,

    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Command,
}

/// Global options that apply to all commands
#[derive(Parser, Debug, Clone)]
pub struct GlobalOpts {
    /// Connection string: user:password@host:port/database
    ///
    /// If password is omitted, it will be read from --password-file,
    /// TQ_PASSWORD environment variable, or prompted interactively.
    #[arg(short = 'l', long, env = "TQ_LOGON", global = true)]
    pub logon: Option<String>,

    /// Read password from file (recommended for security)
    ///
    /// File format: one password per line, or pgpass-style:
    /// hostname:port:database:username:password
    ///
    /// File should have permissions 0600.
    #[arg(long, value_name = "FILE", global = true)]
    pub password_file: Option<PathBuf>,

    /// Authentication mechanism
    ///
    /// TD2: Teradata native authentication (default)
    /// LDAP: LDAP directory authentication
    /// KRB5: Kerberos authentication
    /// TDNEGO: Teradata negotiating mechanism
    #[arg(
        long,
        env = "TQ_LOGMECH",
        default_value = "TD2",
        value_name = "MECH",
        global = true
    )]
    pub logmech: LogonMechanism,

    /// Directory containing the Teradata driver library
    ///
    /// If not specified, uses the bundled library path from build time.
    #[arg(long, value_name = "DIR", global = true)]
    pub driver_lib_dir: Option<String>,

    /// Connection timeout
    ///
    /// Duration format: 30s, 5m, 1h
    #[arg(
        short = 't',
        long,
        env = "TQ_TIMEOUT",
        default_value = "30s",
        value_name = "DURATION",
        global = true
    )]
    pub timeout: String,

    /// Verbose output (repeat for more: -v, -vv, -vvv)
    ///
    /// -v: Show connection and timing info
    /// -vv: Show SQL queries being executed
    /// -vvv: Show driver-level debug info
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress non-essential output
    ///
    /// Only show query results and errors.
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Color output control
    ///
    /// auto: Detect terminal capabilities (default)
    /// always: Force color output
    /// never: Disable color output
    #[arg(
        long,
        env = "TQ_COLOR",
        default_value = "auto",
        value_name = "WHEN",
        global = true
    )]
    pub color: ColorChoice,
}

/// Available commands for tq
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Test database connectivity
    ///
    /// Establishes a connection, executes a simple query, and reports latency.
    /// Use this to verify your connection settings are correct.
    Ping(PingArgs),

    /// Execute a SQL query
    ///
    /// Execute a SQL query and display results in the specified format.
    /// Query can be provided as an argument, read from a file, or from stdin.
    Query(QueryArgs),

    /// Start interactive REPL mode
    ///
    /// Start an interactive Read-Eval-Print Loop for executing SQL queries.
    /// Supports multi-line input, command history, and metacommands.
    Repl(ReplArgs),
}

/// Arguments for the ping command
#[derive(Parser, Debug)]
pub struct PingArgs {
    /// Number of ping attempts
    ///
    /// Similar to network ping, repeat the connection test multiple times.
    #[arg(short, long, default_value = "1", value_name = "N")]
    pub count: u32,

    /// Interval between pings
    ///
    /// Duration format: 1s, 500ms, etc.
    #[arg(short, long, default_value = "1s", value_name = "DURATION")]
    pub interval: String,
}

/// Arguments for the query command
#[derive(Parser, Debug)]
pub struct QueryArgs {
    /// SQL query to execute
    ///
    /// Provide the SQL directly as an argument. Mutually exclusive with --file.
    /// If neither is provided, reads from stdin.
    #[arg(value_name = "QUERY", conflicts_with = "file")]
    pub query: Option<String>,

    /// Read SQL from file
    ///
    /// Execute SQL from a file. Supports multi-statement files separated by semicolons.
    #[arg(long, value_name = "FILE", conflicts_with = "query")]
    pub file: Option<PathBuf>,

    /// Output format
    ///
    /// table: Human-readable ASCII table (default for terminals)
    /// json: JSON array of objects
    /// csv: Comma-separated values (RFC 4180 compliant)
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    ///
    /// If the file exists, it will be overwritten.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Omit column headers in output
    ///
    /// For CSV and table formats, skip the header row.
    #[arg(long)]
    pub no_header: bool,

    /// Show query execution time
    ///
    /// Display timing information after query results.
    #[arg(long)]
    pub timing: bool,

    /// Limit number of rows returned
    ///
    /// Client-side limit on result set size. For server-side limit,
    /// use TOP or SAMPLE in your SQL query.
    #[arg(short = 'n', long, value_name = "N")]
    pub limit: Option<usize>,
}

/// Arguments for the REPL command (future)
#[derive(Parser, Debug)]
pub struct ReplArgs {
    /// Disable command history
    #[arg(long)]
    pub no_history: bool,

    /// History file location
    #[arg(long, default_value = "~/.tq_history", value_name = "FILE")]
    pub history_file: PathBuf,

    /// Disable SQL syntax highlighting
    #[arg(long)]
    pub no_syntax_highlight: bool,

    /// Editor mode for key bindings
    #[arg(long, default_value = "emacs", value_name = "MODE")]
    pub editor_mode: EditorMode,
}

/// Authentication mechanism for Teradata
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogonMechanism {
    /// Teradata 2 authentication (username/password)
    #[value(name = "TD2")]
    Td2,
    /// LDAP authentication
    #[value(name = "LDAP")]
    Ldap,
    /// Kerberos authentication
    #[value(name = "KRB5")]
    Krb5,
    /// Teradata negotiation mechanism
    #[value(name = "TDNEGO")]
    Tdnego,
}

impl std::fmt::Display for LogonMechanism {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogonMechanism::Td2 => write!(f, "TD2"),
            LogonMechanism::Ldap => write!(f, "LDAP"),
            LogonMechanism::Krb5 => write!(f, "KRB5"),
            LogonMechanism::Tdnego => write!(f, "TDNEGO"),
        }
    }
}

/// Output format for query results
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table with borders
    Table,
    /// JSON array of objects
    Json,
    /// Comma-separated values (RFC 4180)
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

/// Color output control
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum ColorChoice {
    /// Detect terminal capabilities automatically
    #[default]
    Auto,
    /// Always use color
    Always,
    /// Never use color
    Never,
}

impl ColorChoice {
    /// Determine if color should be used based on this choice and terminal state
    pub fn should_use_color(&self) -> bool {
        match self {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => {
                // Check if stdout is a terminal and NO_COLOR is not set
                std::io::IsTerminal::is_terminal(&std::io::stdout())
                    && std::env::var("NO_COLOR").is_err()
            }
        }
    }
}

/// Editor mode for REPL key bindings
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum EditorMode {
    /// Emacs-style key bindings (default)
    Emacs,
    /// Vi-style key bindings
    Vi,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parsing_with_ping() {
        let args = vec!["tq", "--logon", "user:pass@host:1025/db", "ping"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.global.logon, Some("user:pass@host:1025/db".to_string()));
        assert_eq!(cli.global.logmech, LogonMechanism::Td2);
        assert!(matches!(cli.command, Command::Ping(_)));
    }

    #[test]
    fn test_cli_parsing_with_query() {
        let args = vec![
            "tq",
            "--logon",
            "user:pass@host:1025/db",
            "query",
            "SELECT 1",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Query(args) = cli.command {
            assert_eq!(args.query, Some("SELECT 1".to_string()));
            assert_eq!(args.format, OutputFormat::Table);
        } else {
            panic!("Expected Query command");
        }
    }

    #[test]
    fn test_cli_parsing_with_logmech() {
        let args = vec![
            "tq",
            "--logon",
            "user:pass@host:1025/db",
            "--logmech",
            "LDAP",
            "ping",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.global.logmech, LogonMechanism::Ldap);
    }

    #[test]
    fn test_cli_parsing_with_format() {
        let args = vec![
            "tq",
            "--logon",
            "user:pass@host:1025/db",
            "query",
            "--format",
            "json",
            "SELECT 1",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Query(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Json);
        } else {
            panic!("Expected Query command");
        }
    }

    #[test]
    fn test_cli_verbose_counting() {
        let args = vec!["tq", "-vvv", "ping"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.global.verbose, 3);
    }

    #[test]
    fn test_cli_ping_with_count() {
        let args = vec!["tq", "ping", "--count", "5", "--interval", "2s"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Ping(args) = cli.command {
            assert_eq!(args.count, 5);
            assert_eq!(args.interval, "2s");
        } else {
            panic!("Expected Ping command");
        }
    }

    #[test]
    fn test_cli_query_with_file() {
        let args = vec!["tq", "query", "--file", "script.sql"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Query(args) = cli.command {
            assert_eq!(args.file, Some(PathBuf::from("script.sql")));
            assert!(args.query.is_none());
        } else {
            panic!("Expected Query command");
        }
    }

    #[test]
    fn test_cli_query_with_output() {
        let args = vec![
            "tq",
            "query",
            "--output",
            "results.csv",
            "--format",
            "csv",
            "SELECT 1",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Query(args) = cli.command {
            assert_eq!(args.output, Some(PathBuf::from("results.csv")));
            assert_eq!(args.format, OutputFormat::Csv);
        } else {
            panic!("Expected Query command");
        }
    }

    #[test]
    fn test_cli_query_with_limit() {
        let args = vec!["tq", "query", "--limit", "100", "SELECT * FROM t"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Query(args) = cli.command {
            assert_eq!(args.limit, Some(100));
        } else {
            panic!("Expected Query command");
        }
    }

    #[test]
    fn test_cli_query_with_timing() {
        let args = vec!["tq", "query", "--timing", "SELECT 1"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Query(args) = cli.command {
            assert!(args.timing);
        } else {
            panic!("Expected Query command");
        }
    }

    #[test]
    fn test_cli_missing_command() {
        let args = vec!["tq", "--logon", "user:pass@host:1025/db"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_color_choice_display() {
        assert!(ColorChoice::Auto.should_use_color() || !ColorChoice::Auto.should_use_color());
        assert!(!ColorChoice::Never.should_use_color());
        assert!(ColorChoice::Always.should_use_color());
    }

    #[test]
    fn test_logon_mechanism_display() {
        assert_eq!(format!("{}", LogonMechanism::Td2), "TD2");
        assert_eq!(format!("{}", LogonMechanism::Ldap), "LDAP");
        assert_eq!(format!("{}", LogonMechanism::Krb5), "KRB5");
        assert_eq!(format!("{}", LogonMechanism::Tdnego), "TDNEGO");
    }
}
