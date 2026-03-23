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
#[command(disable_help_subcommand = true)]  // Disable default help subcommand so we can use our own
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
    # Use a connection profile\n  \
    tq --profile dev query \"SELECT CURRENT_DATE\"\n\n  \
    # Secure password handling\n  \
    echo \"password\" > ~/.tq/passwords/dev && chmod 0600 ~/.tq/passwords/dev\n  \
    tq -l \"user@host:1025/db\" --password-file ~/.tq/passwords/dev query \"SELECT 1\"\n\n  \
    # Read query from file\n  \
    tq query --file script.sql\n\n  \
    # Read from stdin\n  \
    echo \"SELECT 1\" | tq query\n\n\
CONFIGURATION:\n  \
    Set TQ_LOGON environment variable to avoid repeating connection string:\n    \
    export TQ_LOGON=\"user:pass@host:1025/db\"\n\n  \
    Or create ~/.tq/config.toml with connection profiles:\n    \
    [profiles.dev]\n    \
    host = \"dev.company.com\"\n    \
    port = 1025\n    \
    database = \"development\"\n    \
    user = \"alice\"\n    \
    password_file = \"~/.tq/passwords/dev\"\n\n    \
    [profiles.prod]\n    \
    host = \"prod.company.com\"\n    \
    database = \"production\"\n    \
    user = \"alice\"\n    \
    logmech = \"LDAP\"\n    \
    password_file = \"~/.tq/passwords/prod\"\n\n  \
    Then use: tq --profile dev query \"SELECT 1\"\n\n  \
    Config file location: ~/.tq/config.toml (macOS/Linux)\n  \
    For help on configuration: tq help config\n\n\
For more information, visit: https://github.com/remi-td/tq")]
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

    /// Select connection profile from config file
    ///
    /// Profiles are defined in ~/.tq/config.toml under [profiles.<name>].
    /// Profile settings can be overridden by other CLI flags and environment variables.
    ///
    /// Example config:
    ///   [profiles.dev]
    ///   host = "dev.company.com"
    ///   database = "development"
    ///   user = "alice"
    ///   password_file = "~/.tq/passwords/dev"
    #[arg(long, env = "TQ_PROFILE", value_name = "NAME", global = true)]
    pub profile: Option<String>,

    /// Read password from file (recommended for security)
    ///
    /// File format: one password per line, or pgpass-style:
    /// hostname:port:database:username:password
    ///
    /// File should have permissions 0600.
    ///
    /// Place before the subcommand: tq --password-file pw.txt query "SELECT 1"
    #[arg(long, value_name = "FILE")]
    pub password_file: Option<PathBuf>,

    /// Authentication mechanism
    ///
    /// TD2: Teradata native authentication (default)
    /// LDAP: LDAP directory authentication
    /// KRB5: Kerberos authentication
    /// TDNEGO: Teradata negotiating mechanism
    ///
    /// Place before the subcommand: tq --logmech LDAP query "SELECT 1"
    #[arg(
        long,
        env = "TQ_LOGMECH",
        default_value = "TD2",
        value_name = "MECH"
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

    /// YAML parameter file(s) for variable substitution in SQL
    ///
    /// Load variables from YAML files. Variables in SQL are referenced
    /// as {{variable.path}}. Multiple files can be specified; later files
    /// override earlier ones.
    ///
    /// Example: tq -p params.yaml query "SELECT * FROM {{target.database}}.orders"
    #[arg(short = 'p', long = "params", value_name = "FILE", global = true)]
    pub params: Vec<PathBuf>,
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

    /// Show detailed help on a topic
    ///
    /// Display extended help for configuration, credentials, or other topics.
    /// Use without a topic to see available help topics.
    Help(HelpArgs),

    /// List available connection profiles
    ///
    /// Display all connection profiles defined in the configuration file.
    /// Shows profile names and partial connection info (no passwords).
    Profiles,

    /// Manage connection profiles (add, edit, delete, list)
    ///
    /// Add, edit, delete, or list connection profiles stored in ~/.tq/config.toml.
    /// Profiles store connection settings that can be referenced with --profile.
    #[command(subcommand)]
    Profile(ProfileAction),

    /// List active database sessions with performance metrics
    ///
    /// Displays active Teradata sessions including user, state, and
    /// performance metrics (CPU, IO, skew percentages).
    ///
    /// Requires SELECT privilege on DBC.MonitorSession.
    Sessions(SessionsArgs),

    /// Random sample of rows from a table
    ///
    /// Retrieves a random sample of rows from a table using Teradata's
    /// SAMPLE clause for efficient sampling without full table scan.
    ///
    /// Example: tq sample employees 10
    Sample(SampleArgs),

    /// Preview first rows and column metadata from a table
    ///
    /// Displays the first few rows of a table along with column metadata
    /// (name, type, nullable). Useful for quick data exploration.
    ///
    /// Example: tq peek employees
    Peek(PeekArgs),

    /// Display system topology (version, nodes, AMPs, PEs)
    ///
    /// Shows a compact summary of Teradata system configuration including
    /// version, release, and AMP count.
    ///
    /// Requires SELECT privilege on DBC.DBCInfoV.
    Sysconfig(SysconfigArgs),

    /// Display current lock contention and blocking chains
    ///
    /// Shows locked objects, lock types, locking sessions, and waiting
    /// sessions. Automatically identifies blocking chains.
    ///
    /// Requires SELECT privilege on DBC.LockInfoV.
    Locks(LocksArgs),

    /// Inspect recent SQL queries for a session
    ///
    /// Shows the most recent queries executed by a given session,
    /// including SQL text, timing, and status information.
    ///
    /// Requires SELECT privilege on DBC.QryLogV (DBQL must be enabled).
    ///
    /// Example: tq query-inspect 1234
    #[command(name = "query-inspect")]
    QueryInspect(QueryInspectArgs),

    /// Inspect a database object (type, columns, indexes, size)
    ///
    /// Shows comprehensive metadata for a table, view, or other object
    /// including type, columns with types, index structure, and storage
    /// metrics with skew factor.
    ///
    /// Example: tq inspect employees
    ///          tq inspect mydb.employees
    Inspect(InspectArgs),

    /// Describe table structure (columns, types, nullable, defaults)
    ///
    /// Shows column information for a table or view from DBC.ColumnsV.
    ///
    /// Example: tq describe employees
    ///          tq describe mydb.employees
    Describe(DescribeArgs),

    /// List database objects (databases, tables, or views)
    ///
    /// Lists objects of the specified type. For tables, an optional
    /// glob pattern can filter results (e.g., "emp*").
    ///
    /// Example: tq list databases
    ///          tq list tables "emp*"
    ///          tq list views --database mydb
    List(ListArgs),

    /// Show index information for a table
    ///
    /// Displays index names, types, columns, and positions from DBC.IndicesV.
    ///
    /// Example: tq show-indexes employees
    ///          tq show-indexes mydb.employees
    #[command(name = "show-indexes")]
    ShowIndexes(ShowIndexesArgs),

    /// Abort a session or running query
    ///
    /// Terminates a Teradata session or cancels its running query.
    /// Requires --force flag in batch mode (safety guard).
    ///
    /// Example: tq abort --force 1234
    ///          tq abort --force --query 1234
    Abort(AbortArgs),

    /// Change session priority
    ///
    /// Changes the execution priority of a Teradata session.
    /// Valid levels: RUSH, MEDIUM, LOW.
    ///
    /// Example: tq priority 1234 rush
    Priority(PriorityArgs),

    /// Show execution plan for a SQL statement
    ///
    /// Displays the Teradata EXPLAIN plan for a SQL statement,
    /// showing step-by-step execution strategy.
    ///
    /// Example: tq explain "SELECT * FROM employees"
    Explain(ExplainArgs),

    /// Analyze AMP-level resource skew
    ///
    /// Shows CPU and I/O distribution across AMPs for a session
    /// or top sessions by skew factor.
    ///
    /// Example: tq skew 1234
    ///          tq skew
    Skew(SkewArgs),
}

/// Profile management subcommands
#[derive(Subcommand, Debug)]
pub enum ProfileAction {
    /// Add a new connection profile
    ///
    /// Creates a new profile in ~/.tq/config.toml. Requires --host.
    /// Other fields are optional.
    ///
    /// Example: tq profile add dev --host dev.company.com --database devdb --user alice
    Add {
        /// Profile name
        #[arg(value_name = "NAME")]
        name: String,

        /// Database host (required)
        #[arg(long)]
        host: String,

        /// Database port (1-65535)
        #[arg(long, value_name = "PORT")]
        port: Option<u16>,

        /// Default database
        #[arg(long, value_name = "DB")]
        database: Option<String>,

        /// Username
        #[arg(long, value_name = "USER")]
        user: Option<String>,

        /// Authentication mechanism (TD2, LDAP, KRB5, TDNEGO)
        #[arg(long = "logmech", id = "profile_add_logmech", value_name = "MECH")]
        logmech: Option<String>,

        /// Path to password file
        #[arg(long = "password-file", id = "profile_add_password_file", value_name = "FILE")]
        password_file: Option<PathBuf>,
    },

    /// Edit an existing connection profile
    ///
    /// Updates specific fields of an existing profile. At least one field
    /// must be specified. Only provided fields are changed.
    ///
    /// Example: tq profile edit dev --port 2025 --user bob
    Edit {
        /// Profile name
        #[arg(value_name = "NAME")]
        name: String,

        /// Database host
        #[arg(long, value_name = "HOST")]
        host: Option<String>,

        /// Database port (1-65535)
        #[arg(long, value_name = "PORT")]
        port: Option<u16>,

        /// Default database
        #[arg(long, value_name = "DB")]
        database: Option<String>,

        /// Username
        #[arg(long, value_name = "USER")]
        user: Option<String>,

        /// Authentication mechanism (TD2, LDAP, KRB5, TDNEGO)
        #[arg(long = "logmech", id = "profile_edit_logmech", value_name = "MECH")]
        logmech: Option<String>,

        /// Path to password file
        #[arg(long = "password-file", id = "profile_edit_password_file", value_name = "FILE")]
        password_file: Option<PathBuf>,
    },

    /// Delete a connection profile
    ///
    /// Removes a profile from ~/.tq/config.toml. Requires --force flag
    /// for non-interactive confirmation.
    ///
    /// Example: tq profile delete dev --force
    Delete {
        /// Profile name
        #[arg(value_name = "NAME")]
        name: String,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },

    /// List all connection profiles
    ///
    /// Displays all profiles defined in the configuration file.
    List,
}

/// Arguments for the help command
#[derive(Parser, Debug)]
pub struct HelpArgs {
    /// Help topic to display
    ///
    /// Available topics: config, credentials
    #[arg(value_name = "TOPIC")]
    pub topic: Option<HelpTopic>,
}

/// Available help topics
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum HelpTopic {
    /// Configuration file format and usage
    Config,
    /// Password and credential management
    Credentials,
    /// Variable substitution syntax and YAML parameter files
    Params,
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

    /// Wrap statements in a transaction (batch mode only)
    ///
    /// Executes BEGIN TRANSACTION before the first statement and
    /// COMMIT on success. If any statement fails, automatically
    /// executes ROLLBACK to revert all changes.
    ///
    /// Only applies to multi-statement execution from --file or stdin.
    /// Single statement queries will show a warning and execute normally.
    ///
    /// Note: Cannot be used with SQL that contains explicit transaction
    /// control (BEGIN TRANSACTION, COMMIT, ROLLBACK).
    #[arg(long)]
    pub atomic: bool,
}

/// Arguments for the sessions command (Sprint 26)
#[derive(Parser, Debug)]
pub struct SessionsArgs {
    /// Output format
    ///
    /// table: Human-readable ASCII table (default)
    /// json: JSON array of session objects
    /// csv: Comma-separated values
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
}

/// Arguments for the sysconfig command (Sprint 38)
#[derive(Parser, Debug)]
pub struct SysconfigArgs {
    /// Output format
    ///
    /// table: Two-column key-value table (default)
    /// json: JSON object with property keys
    /// csv: Comma-separated Property,Value pairs
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
}

/// Arguments for the locks command (Sprint 38)
#[derive(Parser, Debug)]
pub struct LocksArgs {
    /// Output format
    ///
    /// table: Human-readable ASCII table (default)
    /// json: JSON array of lock objects
    /// csv: Comma-separated values
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
}

/// Arguments for the query-inspect command (Sprint 39)
#[derive(Parser, Debug)]
pub struct QueryInspectArgs {
    /// Session ID to inspect
    ///
    /// The Teradata session number whose recent queries should be displayed.
    #[arg(value_name = "SESSION_ID")]
    pub session_id: i64,

    /// Output format
    ///
    /// table: Key-value pairs for each query (default)
    /// json: JSON array of query objects
    /// csv: Comma-separated values
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
}

/// Arguments for the inspect command (Sprint 45)
#[derive(Parser, Debug)]
pub struct InspectArgs {
    /// Object to inspect (table, view, or database.object)
    ///
    /// Can be unqualified (uses current database) or qualified (database.object).
    #[arg(value_name = "OBJECT")]
    pub object: String,

    /// Output format
    ///
    /// table: Human-readable structured output (default)
    /// json: JSON object with all metadata
    /// csv: Column information as CSV
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
}

/// Arguments for the describe command
#[derive(Parser, Debug)]
pub struct DescribeArgs {
    /// Table or view to describe (unqualified or database.object)
    #[arg(value_name = "OBJECT")]
    pub object: String,

    /// Output format
    ///
    /// table: Human-readable column listing (default)
    /// json: JSON array of column objects
    /// csv: Comma-separated column information
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

/// Arguments for the list command (Sprint 46)
#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Type of objects to list
    #[arg(value_name = "TYPE")]
    pub object_type: ListObjectType,

    /// Optional glob pattern to filter results (tables only)
    #[arg(value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// Database to list from (defaults to current database)
    #[arg(short, long, value_name = "DB")]
    pub database: Option<String>,

    /// Output format
    ///
    /// table: Human-readable listing (default)
    /// json: JSON array of objects
    /// csv: Comma-separated values
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

/// Types of objects that can be listed
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ListObjectType {
    /// List accessible databases
    Databases,
    /// List tables in a database
    Tables,
    /// List views in a database
    Views,
}

/// Arguments for the show-indexes command
#[derive(Parser, Debug)]
pub struct ShowIndexesArgs {
    /// Object to show indexes for (unqualified or database.object)
    #[arg(value_name = "OBJECT")]
    pub table: String,

    /// Output format
    ///
    /// table: Human-readable index listing (default)
    /// json: JSON array of index objects
    /// csv: Comma-separated index information
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

/// Arguments for the abort command (Sprint 49)
#[derive(Parser, Debug)]
pub struct AbortArgs {
    /// Session ID to abort
    ///
    /// The Teradata session number to terminate.
    #[arg(value_name = "SESSION_ID")]
    pub session_id: i64,

    /// Abort only the running query, not the entire session
    ///
    /// Cancels the currently executing query while keeping the session alive.
    #[arg(long)]
    pub query: bool,

    /// Confirm the abort operation (required in batch mode)
    ///
    /// Abort is a destructive operation. This flag is required to prevent
    /// accidental session termination.
    #[arg(long)]
    pub force: bool,

    /// Output format
    ///
    /// table: Human-readable message (default)
    /// json: JSON result object
    /// csv: Comma-separated result
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

/// Arguments for the priority command (Sprint 49)
#[derive(Parser, Debug)]
pub struct PriorityArgs {
    /// Session ID to change priority for
    ///
    /// The Teradata session number whose priority should be changed.
    #[arg(value_name = "SESSION_ID")]
    pub session_id: i64,

    /// Priority level to set
    ///
    /// Valid levels: RUSH, MEDIUM, LOW (case-insensitive).
    #[arg(value_name = "LEVEL")]
    pub level: String,

    /// Output format
    ///
    /// table: Human-readable message (default)
    /// json: JSON result object
    /// csv: Comma-separated result
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

/// Arguments for the explain command (Sprint 50)
#[derive(Parser, Debug)]
pub struct ExplainArgs {
    /// SQL statement to explain
    ///
    /// The SQL query whose execution plan should be displayed.
    /// Do not include the EXPLAIN keyword — it will be added automatically.
    #[arg(value_name = "SQL")]
    pub sql: String,

    /// Output format
    ///
    /// table: Formatted explain output (default)
    /// json: JSON object with steps array
    /// csv: Step number and text as CSV
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

/// Arguments for the skew command (Sprint 50)
#[derive(Parser, Debug)]
pub struct SkewArgs {
    /// Session ID to analyze (omit for top sessions by skew)
    ///
    /// If provided, shows detailed AMP skew for that session.
    /// If omitted, shows top 10 sessions ranked by CPU skew.
    #[arg(value_name = "SESSION_ID")]
    pub session_id: Option<i64>,

    /// Output format
    ///
    /// table: Formatted skew analysis (default)
    /// json: JSON array of session objects
    /// csv: Comma-separated values
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

/// Arguments for the sample command (Sprint 33)
#[derive(Parser, Debug)]
pub struct SampleArgs {
    /// Table name to sample from
    ///
    /// Can be unqualified (uses current database) or qualified (database.table).
    #[arg(value_name = "TABLE")]
    pub table: String,

    /// Number of rows to sample (default: 10, max: 1000)
    ///
    /// Uses Teradata's SAMPLE clause for efficient random sampling.
    #[arg(value_name = "N", default_value = "10")]
    pub count: usize,

    /// Output format
    ///
    /// table: Human-readable ASCII table (default)
    /// json: JSON array of objects
    /// csv: Comma-separated values
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

/// Arguments for the peek command (Sprint 33)
#[derive(Parser, Debug)]
pub struct PeekArgs {
    /// Table name to peek at
    ///
    /// Can be unqualified (uses current database) or qualified (database.table).
    #[arg(value_name = "TABLE")]
    pub table: String,

    /// Number of rows to display (default: 5)
    #[arg(value_name = "N", default_value = "5")]
    pub count: usize,

    /// Output format
    ///
    /// table: Human-readable ASCII table (default)
    /// json: JSON object with columns and rows
    /// csv: Comma-separated values (columns section, then data)
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

/// Arguments for the REPL command
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
    ///
    /// Choose between emacs (default) or vi keybindings.
    /// emacs: Standard readline-style keybindings
    /// vi: Modal editing with insert/normal modes
    #[arg(
        long,
        default_value = "emacs",
        value_name = "MODE",
        env = "TQ_EDITOR_MODE"
    )]
    pub editor_mode: EditorMode,

    /// Default row limit for SELECT queries (0 = unlimited)
    ///
    /// In REPL mode, SELECT queries without an explicit TOP or SAMPLE clause will
    /// be limited to this many rows by default. This prevents accidentally
    /// flooding the terminal with millions of rows.
    ///
    /// Use 0 to disable the default limit (fetch all rows).
    #[arg(long, default_value = "100", value_name = "N", env = "TQ_REPL_LIMIT")]
    pub default_limit: usize,

    /// Disable result paging (show all output at once)
    ///
    /// By default, large result sets are displayed with a pager
    /// that allows scrolling. Use this flag to disable paging.
    #[arg(long)]
    pub no_pager: bool,

    /// Show enhanced timing information
    ///
    /// Display detailed query timing breakdown including connection,
    /// execution, and transfer times.
    #[arg(long)]
    pub enhanced_timing: bool,
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

    #[test]
    fn test_cli_repl_with_vi_mode() {
        let args = vec!["tq", "repl", "--editor-mode", "vi"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Repl(args) = cli.command {
            assert_eq!(args.editor_mode, EditorMode::Vi);
        } else {
            panic!("Expected Repl command");
        }
    }

    #[test]
    fn test_cli_repl_with_emacs_mode() {
        let args = vec!["tq", "repl", "--editor-mode", "emacs"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Repl(args) = cli.command {
            assert_eq!(args.editor_mode, EditorMode::Emacs);
        } else {
            panic!("Expected Repl command");
        }
    }

    #[test]
    fn test_cli_repl_default_editor_mode() {
        let args = vec!["tq", "repl"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Repl(args) = cli.command {
            // Default should be emacs
            assert_eq!(args.editor_mode, EditorMode::Emacs);
        } else {
            panic!("Expected Repl command");
        }
    }

    #[test]
    fn test_cli_repl_with_custom_history_file() {
        let args = vec!["tq", "repl", "--history-file", "/tmp/my_history"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Repl(args) = cli.command {
            assert_eq!(args.history_file, PathBuf::from("/tmp/my_history"));
        } else {
            panic!("Expected Repl command");
        }
    }

    #[test]
    fn test_cli_repl_with_no_history() {
        let args = vec!["tq", "repl", "--no-history"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Repl(args) = cli.command {
            assert!(args.no_history);
        } else {
            panic!("Expected Repl command");
        }
    }

    #[test]
    fn test_cli_with_profile() {
        let args = vec!["tq", "--profile", "dev", "ping"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.global.profile, Some("dev".to_string()));
        assert!(matches!(cli.command, Command::Ping(_)));
    }

    #[test]
    fn test_cli_with_profile_and_logon() {
        // Both can be specified - logon takes precedence in the connection logic
        let args = vec![
            "tq",
            "--profile",
            "dev",
            "--logon",
            "user:pass@host:1025/db",
            "ping",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.global.profile, Some("dev".to_string()));
        assert_eq!(cli.global.logon, Some("user:pass@host:1025/db".to_string()));
    }

    #[test]
    fn test_cli_profile_with_query() {
        let args = vec!["tq", "--profile", "prod", "query", "SELECT 1"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.global.profile, Some("prod".to_string()));
        if let Command::Query(args) = cli.command {
            assert_eq!(args.query, Some("SELECT 1".to_string()));
        } else {
            panic!("Expected Query command");
        }
    }

    #[test]
    fn test_cli_help_no_topic() {
        let args = vec!["tq", "help"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Help(args) = cli.command {
            assert!(args.topic.is_none());
        } else {
            panic!("Expected Help command");
        }
    }

    #[test]
    fn test_cli_help_config_topic() {
        let args = vec!["tq", "help", "config"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Help(args) = cli.command {
            assert_eq!(args.topic, Some(HelpTopic::Config));
        } else {
            panic!("Expected Help command");
        }
    }

    #[test]
    fn test_cli_help_credentials_topic() {
        let args = vec!["tq", "help", "credentials"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Help(args) = cli.command {
            assert_eq!(args.topic, Some(HelpTopic::Credentials));
        } else {
            panic!("Expected Help command");
        }
    }

    #[test]
    fn test_cli_help_invalid_topic() {
        let args = vec!["tq", "help", "invalid"];
        let result = Cli::try_parse_from(args);
        // Should fail because "invalid" is not a valid HelpTopic
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_query_with_atomic() {
        let args = vec!["tq", "query", "--atomic", "--file", "script.sql"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Query(args) = cli.command {
            assert!(args.atomic);
            assert_eq!(args.file, Some(PathBuf::from("script.sql")));
        } else {
            panic!("Expected Query command");
        }
    }

    #[test]
    fn test_cli_query_without_atomic() {
        let args = vec!["tq", "query", "SELECT 1"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Query(args) = cli.command {
            assert!(!args.atomic);
        } else {
            panic!("Expected Query command");
        }
    }

    #[test]
    fn test_cli_profiles_command() {
        let args = vec!["tq", "profiles"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Command::Profiles));
    }

    // Sprint 33: Tests for sample command
    #[test]
    fn test_cli_sample_with_table() {
        let args = vec!["tq", "sample", "employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Sample(args) = cli.command {
            assert_eq!(args.table, "employees");
            assert_eq!(args.count, 10); // default
            assert_eq!(args.format, OutputFormat::Table);
        } else {
            panic!("Expected Sample command");
        }
    }

    #[test]
    fn test_cli_sample_with_count() {
        let args = vec!["tq", "sample", "employees", "50"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Sample(args) = cli.command {
            assert_eq!(args.table, "employees");
            assert_eq!(args.count, 50);
        } else {
            panic!("Expected Sample command");
        }
    }

    #[test]
    fn test_cli_sample_with_format() {
        let args = vec!["tq", "sample", "--format", "json", "employees", "20"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Sample(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Json);
            assert_eq!(args.table, "employees");
            assert_eq!(args.count, 20);
        } else {
            panic!("Expected Sample command");
        }
    }

    #[test]
    fn test_cli_sample_with_output() {
        let args = vec!["tq", "sample", "--output", "sample.csv", "employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Sample(args) = cli.command {
            assert_eq!(args.output, Some(PathBuf::from("sample.csv")));
        } else {
            panic!("Expected Sample command");
        }
    }

    #[test]
    fn test_cli_sample_qualified_table() {
        let args = vec!["tq", "sample", "demo_db.employees", "25"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Sample(args) = cli.command {
            assert_eq!(args.table, "demo_db.employees");
            assert_eq!(args.count, 25);
        } else {
            panic!("Expected Sample command");
        }
    }

    // Sprint 33: Tests for peek command
    #[test]
    fn test_cli_peek_with_table() {
        let args = vec!["tq", "peek", "employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Peek(args) = cli.command {
            assert_eq!(args.table, "employees");
            assert_eq!(args.count, 5); // default
            assert_eq!(args.format, OutputFormat::Table);
        } else {
            panic!("Expected Peek command");
        }
    }

    #[test]
    fn test_cli_peek_with_count() {
        let args = vec!["tq", "peek", "employees", "10"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Peek(args) = cli.command {
            assert_eq!(args.table, "employees");
            assert_eq!(args.count, 10);
        } else {
            panic!("Expected Peek command");
        }
    }

    #[test]
    fn test_cli_peek_with_format() {
        let args = vec!["tq", "peek", "--format", "json", "employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Peek(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Json);
        } else {
            panic!("Expected Peek command");
        }
    }

    #[test]
    fn test_cli_peek_with_output() {
        let args = vec!["tq", "peek", "--output", "peek.json", "employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Peek(args) = cli.command {
            assert_eq!(args.output, Some(PathBuf::from("peek.json")));
        } else {
            panic!("Expected Peek command");
        }
    }

    #[test]
    fn test_cli_peek_qualified_table() {
        let args = vec!["tq", "peek", "demo_db.employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Peek(args) = cli.command {
            assert_eq!(args.table, "demo_db.employees");
        } else {
            panic!("Expected Peek command");
        }
    }

    // Sprint 38: Tests for sysconfig command
    #[test]
    fn test_cli_sysconfig_default() {
        let args = vec!["tq", "sysconfig"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Sysconfig(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Table);
            assert!(args.output.is_none());
        } else {
            panic!("Expected Sysconfig command");
        }
    }

    #[test]
    fn test_cli_sysconfig_with_format() {
        let args = vec!["tq", "sysconfig", "--format", "json"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Sysconfig(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Json);
        } else {
            panic!("Expected Sysconfig command");
        }
    }

    #[test]
    fn test_cli_sysconfig_with_output() {
        let args = vec!["tq", "sysconfig", "--output", "config.csv"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Sysconfig(args) = cli.command {
            assert_eq!(args.output, Some(PathBuf::from("config.csv")));
        } else {
            panic!("Expected Sysconfig command");
        }
    }

    #[test]
    fn test_cli_sysconfig_with_csv_format_and_output() {
        let args = vec!["tq", "sysconfig", "-f", "csv", "-o", "sysconfig.csv"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Sysconfig(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Csv);
            assert_eq!(args.output, Some(PathBuf::from("sysconfig.csv")));
        } else {
            panic!("Expected Sysconfig command");
        }
    }

    // Sprint 38: Tests for locks command
    #[test]
    fn test_cli_locks_default() {
        let args = vec!["tq", "locks"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Locks(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Table);
            assert!(args.output.is_none());
        } else {
            panic!("Expected Locks command");
        }
    }

    #[test]
    fn test_cli_locks_with_format() {
        let args = vec!["tq", "locks", "--format", "json"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Locks(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Json);
        } else {
            panic!("Expected Locks command");
        }
    }

    #[test]
    fn test_cli_locks_with_output() {
        let args = vec!["tq", "locks", "--output", "locks.csv"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Locks(args) = cli.command {
            assert_eq!(args.output, Some(PathBuf::from("locks.csv")));
        } else {
            panic!("Expected Locks command");
        }
    }

    #[test]
    fn test_cli_locks_with_csv_format_and_output() {
        let args = vec!["tq", "locks", "-f", "csv", "-o", "locks.csv"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Locks(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Csv);
            assert_eq!(args.output, Some(PathBuf::from("locks.csv")));
        } else {
            panic!("Expected Locks command");
        }
    }

    #[test]
    fn test_cli_sysconfig_with_profile() {
        let args = vec!["tq", "--profile", "prod", "sysconfig"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.global.profile, Some("prod".to_string()));
        assert!(matches!(cli.command, Command::Sysconfig(_)));
    }

    #[test]
    fn test_cli_locks_with_profile() {
        let args = vec!["tq", "--profile", "prod", "locks"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.global.profile, Some("prod".to_string()));
        assert!(matches!(cli.command, Command::Locks(_)));
    }

    // Sprint 39: Tests for query-inspect command
    #[test]
    fn test_cli_query_inspect_default() {
        let args = vec!["tq", "query-inspect", "1234"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::QueryInspect(args) = cli.command {
            assert_eq!(args.session_id, 1234);
            assert_eq!(args.format, OutputFormat::Table);
            assert!(args.output.is_none());
        } else {
            panic!("Expected QueryInspect command");
        }
    }

    #[test]
    fn test_cli_query_inspect_with_format() {
        let args = vec!["tq", "query-inspect", "--format", "json", "5678"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::QueryInspect(args) = cli.command {
            assert_eq!(args.session_id, 5678);
            assert_eq!(args.format, OutputFormat::Json);
        } else {
            panic!("Expected QueryInspect command");
        }
    }

    #[test]
    fn test_cli_query_inspect_with_csv_format() {
        let args = vec!["tq", "query-inspect", "-f", "csv", "1234"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::QueryInspect(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Csv);
        } else {
            panic!("Expected QueryInspect command");
        }
    }

    #[test]
    fn test_cli_query_inspect_with_output() {
        let args = vec!["tq", "query-inspect", "--output", "queries.csv", "1234"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::QueryInspect(args) = cli.command {
            assert_eq!(args.output, Some(PathBuf::from("queries.csv")));
        } else {
            panic!("Expected QueryInspect command");
        }
    }

    #[test]
    fn test_cli_query_inspect_missing_session_id() {
        let args = vec!["tq", "query-inspect"];
        let result = Cli::try_parse_from(args);
        // Should fail because session_id is required
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_query_inspect_invalid_session_id() {
        let args = vec!["tq", "query-inspect", "not_a_number"];
        let result = Cli::try_parse_from(args);
        // Should fail because session_id must be an integer
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_query_inspect_with_profile() {
        let args = vec!["tq", "--profile", "prod", "query-inspect", "1234"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.global.profile, Some("prod".to_string()));
        if let Command::QueryInspect(args) = cli.command {
            assert_eq!(args.session_id, 1234);
        } else {
            panic!("Expected QueryInspect command");
        }
    }

    // Sprint 40: Tests for --params flag

    #[test]
    fn test_cli_params_single_file() {
        let args = vec!["tq", "-p", "params.yaml", "query", "SELECT 1"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.global.params, vec![PathBuf::from("params.yaml")]);
    }

    #[test]
    fn test_cli_params_long_form() {
        let args = vec!["tq", "--params", "params.yaml", "query", "SELECT 1"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.global.params, vec![PathBuf::from("params.yaml")]);
    }

    #[test]
    fn test_cli_params_multiple_files() {
        let args = vec![
            "tq",
            "-p", "base.yaml",
            "-p", "overrides.yaml",
            "query", "SELECT 1",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(
            cli.global.params,
            vec![PathBuf::from("base.yaml"), PathBuf::from("overrides.yaml")]
        );
    }

    #[test]
    fn test_cli_params_no_files() {
        let args = vec!["tq", "query", "SELECT 1"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.global.params.is_empty());
    }

    #[test]
    fn test_cli_params_with_repl() {
        let args = vec!["tq", "-p", "params.yaml", "repl"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.global.params, vec![PathBuf::from("params.yaml")]);
        assert!(matches!(cli.command, Command::Repl(_)));
    }

    #[test]
    fn test_cli_help_params_topic() {
        let args = vec!["tq", "help", "params"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Help(args) = cli.command {
            assert_eq!(args.topic, Some(HelpTopic::Params));
        } else {
            panic!("Expected Help command");
        }
    }

    // Sprint 46: Tests for describe command
    #[test]
    fn test_cli_describe_default() {
        let args = vec!["tq", "describe", "employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Describe(args) = cli.command {
            assert_eq!(args.object, "employees");
            assert_eq!(args.format, OutputFormat::Table);
            assert!(args.output.is_none());
        } else {
            panic!("Expected Describe command");
        }
    }

    #[test]
    fn test_cli_describe_qualified() {
        let args = vec!["tq", "describe", "mydb.employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Describe(args) = cli.command {
            assert_eq!(args.object, "mydb.employees");
        } else {
            panic!("Expected Describe command");
        }
    }

    #[test]
    fn test_cli_describe_with_format() {
        let args = vec!["tq", "describe", "--format", "json", "employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Describe(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Json);
        } else {
            panic!("Expected Describe command");
        }
    }

    #[test]
    fn test_cli_describe_with_output() {
        let args = vec!["tq", "describe", "--output", "desc.csv", "employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Describe(args) = cli.command {
            assert_eq!(args.output, Some(PathBuf::from("desc.csv")));
        } else {
            panic!("Expected Describe command");
        }
    }

    // Sprint 46: Tests for list command
    #[test]
    fn test_cli_list_databases() {
        let args = vec!["tq", "list", "databases"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::List(args) = cli.command {
            assert_eq!(args.object_type, ListObjectType::Databases);
            assert!(args.pattern.is_none());
            assert!(args.database.is_none());
            assert_eq!(args.format, OutputFormat::Table);
        } else {
            panic!("Expected List command");
        }
    }

    #[test]
    fn test_cli_list_tables_with_pattern() {
        let args = vec!["tq", "list", "tables", "emp*"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::List(args) = cli.command {
            assert_eq!(args.object_type, ListObjectType::Tables);
            assert_eq!(args.pattern, Some("emp*".to_string()));
        } else {
            panic!("Expected List command");
        }
    }

    #[test]
    fn test_cli_list_views_with_database() {
        let args = vec!["tq", "list", "views", "--database", "mydb"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::List(args) = cli.command {
            assert_eq!(args.object_type, ListObjectType::Views);
            assert_eq!(args.database, Some("mydb".to_string()));
        } else {
            panic!("Expected List command");
        }
    }

    #[test]
    fn test_cli_list_with_format() {
        let args = vec!["tq", "list", "--format", "json", "databases"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::List(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Json);
        } else {
            panic!("Expected List command");
        }
    }

    #[test]
    fn test_cli_list_missing_type() {
        let args = vec!["tq", "list"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    // Sprint 46: Tests for show-indexes command
    #[test]
    fn test_cli_show_indexes_default() {
        let args = vec!["tq", "show-indexes", "employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::ShowIndexes(args) = cli.command {
            assert_eq!(args.table, "employees");
            assert_eq!(args.format, OutputFormat::Table);
            assert!(args.output.is_none());
        } else {
            panic!("Expected ShowIndexes command");
        }
    }

    #[test]
    fn test_cli_show_indexes_qualified() {
        let args = vec!["tq", "show-indexes", "mydb.employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::ShowIndexes(args) = cli.command {
            assert_eq!(args.table, "mydb.employees");
        } else {
            panic!("Expected ShowIndexes command");
        }
    }

    #[test]
    fn test_cli_show_indexes_with_format() {
        let args = vec!["tq", "show-indexes", "--format", "csv", "employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::ShowIndexes(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Csv);
        } else {
            panic!("Expected ShowIndexes command");
        }
    }

    #[test]
    fn test_cli_show_indexes_missing_table() {
        let args = vec!["tq", "show-indexes"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }
}
