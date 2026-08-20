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

    /// Connection timeout (TCP connection establishment only)
    ///
    /// Bounds how long tq waits to establish the connection. Distinct from
    /// --query-timeout, which bounds query execution.
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

    /// Query execution timeout (request/fetch deadline)
    ///
    /// Bounds how long a single query may run before tq cancels the request,
    /// closes the session, and returns a QUERY_TIMEOUT error. Distinct from
    /// --timeout (connection establishment). If unset, queries run without a
    /// timeout, EXCEPT in --agent-safe mode where a conservative finite default
    /// (30s) is applied automatically.
    ///
    /// Duration format: 30s, 5m, 1h
    #[arg(
        long,
        env = "TQ_QUERY_TIMEOUT",
        value_name = "DURATION",
        global = true
    )]
    pub query_timeout: Option<String>,

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

    /// Override severity levels for specific Teradata error codes
    ///
    /// Format: --errorlevel CODE [CODE...] SEVERITY
    /// Examples:
    ///   --errorlevel 3120 3802 warning
    ///   --errorlevel 3523 error --errorlevel 3802 warning
    #[arg(long, num_args = 2.., action = clap::ArgAction::Append, global = true, value_name = "ARGS")]
    pub errorlevel: Vec<String>,

    /// YAML parameter file(s) for variable substitution in SQL
    ///
    /// Load variables from YAML files. Variables in SQL are referenced
    /// as {{variable.path}}. Multiple files can be specified; later files
    /// override earlier ones.
    ///
    /// Example: tq -p params.yaml query "SELECT * FROM {{target.database}}.orders"
    #[arg(short = 'p', long = "params", value_name = "FILE", global = true)]
    pub params: Vec<PathBuf>,

    /// Pass KEY=VALUE parameters for SQL variable substitution
    ///
    /// Defines or overrides parameters directly from the command line.
    /// Overrides keys loaded from parameter files (-p/--params).
    ///
    /// Example: tq -D table=employees query "SELECT * FROM {{table}}"
    #[arg(short = 'D', long = "define", value_name = "KEY=VALUE", global = true)]
    pub define: Vec<String>,

    /// Enforce agent-safe restrictions globally
    #[arg(long, env = "TQ_AGENT_SAFE", global = true)]
    pub agent_safe: bool,

    /// Shortcut for --format json across all subcommands
    #[arg(long, global = true)]
    pub json: bool,
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
    #[command(name = "query-inspect", alias = "qi")]
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

    /// List database objects (databases, tables, or views)
    ///
    /// Lists objects of the specified type. For tables, an optional
    /// glob pattern can filter results (e.g., "emp*").
    ///
    /// Example: tq list databases
    ///          tq list tables "emp*"
    ///          tq list views --database mydb
    List(ListArgs),

    /// Search for database objects across all databases
    ///
    /// Find tables or columns by keyword across all accessible databases.
    /// Useful for discovery when exact object names are unknown.
    ///
    /// Example: tq search tables emp
    ///          tq search columns salary --database hr
    Search(SearchArgs),

    /// Show index information for a table
    ///
    /// Displays index names, types, columns, and positions from DBC.IndicesV.
    ///
    /// Example: tq show-indexes employees
    ///          tq show-indexes mydb.employees
    #[command(name = "show-indexes", alias = "di")]
    ShowIndexes(ShowIndexesArgs),

    /// Abort a session or running query
    ///
    /// Terminates a Teradata session or cancels its running query.
    /// Requires --force flag in batch mode (safety guard).
    ///
    /// Example: tq abort --force 1234
    ///          tq abort --force --query 1234
    Abort(AbortArgs),

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

    /// Analyze permanent space usage for a database or object
    ///
    /// With a bare database name, shows the database's own perm/spool/temp
    /// footprint followed by one row per contained object. With a qualified
    /// name, shows that object only.
    ///
    /// Example: tq space demo_user
    ///          tq space demo_user.orders
    Space(SpaceArgs),

    /// Analyze permanent space usage for a database only
    ///
    /// Shows database-level perm, spool and temp metrics without listing the
    /// objects it contains. Rejects a qualified `database.object` argument.
    ///
    /// Example: tq dbspace demo_user
    Dbspace(DbspaceArgs),

    /// View session logon/logoff history and trends
    ///
    /// Shows recent session activity from DBC.LogOnOffV including logon,
    /// logoff, and authentication failure events with summary statistics.
    ///
    /// Example: tq history --last 24h
    ///          tq history --last 7d --user alice
    History(HistoryArgs),

    /// Display system resource usage (CPU, I/O, memory)
    ///
    /// Shows resource metrics from Teradata ResUsage tables. Default mode
    /// shows per-VPROC metrics (virtual). Use --physical for per-node metrics.
    ///
    /// Requires SELECT privilege on DBC.ResUsageSPMA/DBC.ResUsageSVPR.
    ///
    /// Example: tq resources
    ///          tq resources --physical
    Resources(ResourcesArgs),

    /// Log off idle sessions older than a threshold
    ///
    /// Finds sessions in IDLE state whose logon time exceeds a specified
    /// duration and terminates them. Useful for cleaning up stale connections.
    ///
    /// Example: tq logoff-idle --force
    ///          tq logoff-idle --force --older-than 2h
    #[command(name = "logoff-idle")]
    LogoffIdle(LogoffIdleArgs),

    /// Bulk load data into an empty Teradata table in parallel (FastLoad)
    ///
    /// FastLoad transfers data in parallel over multiple connections.
    /// It can only load into an empty permanent table.
    ///
    /// Source file can be CSV, Parquet, or JSON.
    ///
    /// Example: tq fastload data.csv my_db.my_table
    Fastload(FastloadArgs),

    /// Bulk export data from a Teradata table in parallel (FastExport)
    ///
    /// FastExport transfers data in parallel over multiple connections.
    ///
    /// Destination file is exported in CSV format.
    ///
    /// Example: tq fastexport my_db.my_table data.csv
    Fastexport(FastexportArgs),

    /// Inspect and validate parameter files
    ///
    /// Load and display dot-notation variable mappings from YAML parameter files.
    ///
    /// Example: tq params params.yaml
    Params(ParamsArgs),

    /// Inspect error severity classification mappings
    ///
    /// Display configured or custom error level severity overrides.
    ///
    /// Example: tq errorlevel
    Errorlevel(ErrorlevelArgs),
}

impl Command {
    /// Extract the output format from this command's arguments, if applicable.
    ///
    /// Returns None for commands that don't have a format argument (Help, Profiles, Profile, Repl, Ping).
    pub fn format(&self) -> Option<OutputFormat> {
        match self {
            Command::Query(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Sessions(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Sample(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Peek(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Sysconfig(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Locks(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::QueryInspect(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Inspect(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::List(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Search(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::ShowIndexes(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Abort(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Explain(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Skew(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Space(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Dbspace(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::History(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Resources(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::LogoffIdle(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Params(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Errorlevel(a) => Some(if a.json { OutputFormat::Json } else { a.format }),
            Command::Fastload(_) | Command::Fastexport(_) | Command::Ping(_) | Command::Repl(_) | Command::Help(_) | Command::Profiles | Command::Profile(_) => None,
        }
    }
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
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Enable agent-safe execution mode
    ///
    /// Enforces defense-in-depth guardrails for automated/LLM-driven usage
    /// (NOT a security boundary -- use database-side least privilege for that;
    /// see `tq help` and the security guide):
    /// - Allows read-only statements: SELECT/SEL, SHOW, HELP, EXPLAIN,
    ///   including read-only WITH (CTE) and LOCKING forms.
    /// - Blocks DML (INSERT, UPDATE, DELETE, MERGE, UPSERT) unless --allow-dml.
    /// - Blocks maintenance (COLLECT STATISTICS) unless --allow-maintenance.
    /// - Always blocks DDL/DCL (CREATE, DROP, ALTER, RENAME, GRANT, REVOKE, ...).
    /// - Fails closed: statements it cannot classify are rejected, not run.
    /// - Enforces single-statement-only (rejects multi-statement input).
    /// - Applies a finite query timeout by default (see --query-timeout).
    /// - Enforces the --max-rows client fetch/output cap.
    #[arg(long, env = "TQ_AGENT_SAFE")]
    pub agent_safe: bool,

    /// Maximum rows for the client fetch/output cap in agent-safe mode (default: 10000)
    ///
    /// This is a CLIENT-side fetch/output cap, NOT a database workload limit:
    /// tq fetches at most max_rows + 1 rows and fails with AGENT_SAFE_MAX_ROWS
    /// if the extra row appears. No TOP/SAMPLE is injected into your SQL, so the
    /// database may still scan the full table. To bound server-side work, add
    /// TOP or SAMPLE to the query itself.
    #[arg(long, value_name = "N", default_value = "10000")]
    pub max_rows: usize,

    /// Allow DML operations in agent-safe mode
    ///
    /// Permits INSERT, UPDATE, DELETE, MERGE, and UPSERT statements when
    /// --agent-safe is active. DDL operations remain blocked.
    #[arg(long)]
    pub allow_dml: bool,

    /// Allow maintenance operations in agent-safe mode
    ///
    /// Permits COLLECT STATISTICS / COLLECT STATS when --agent-safe is active.
    /// DDL operations remain blocked.
    #[arg(long)]
    pub allow_maintenance: bool,

    /// Number of rows per page (enables pagination)
    ///
    /// When specified, results are split into pages of this size.
    /// Use with --page to select which page to retrieve.
    /// Mutually exclusive with --limit.
    #[arg(long, value_name = "N", conflicts_with = "limit")]
    pub page_size: Option<usize>,

    /// Page number to retrieve (1-based, default: 1)
    ///
    /// Requires --page-size. Returns the specified page of results.
    #[arg(long, value_name = "P", default_value = "1", requires = "page_size")]
    pub page: usize,

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,

    /// Output substituted SQL statement without executing query against database
    #[arg(long)]
    pub dry_run: bool,
}


/// Arguments for the sessions command (Sprint 26)
#[derive(Parser, Debug)]
pub struct SessionsArgs {
    /// Output format
    ///
    /// table: Human-readable ASCII table (default)
    /// json: JSON array of session objects
    /// csv: Comma-separated values
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Enable watch mode (auto-refresh display)
    ///
    /// Continuously refreshes the display at a fixed interval.
    /// Press q or Ctrl-C to stop. Incompatible with --output.
    #[arg(long, conflicts_with = "output")]
    pub watch: bool,

    /// Refresh interval in seconds for watch mode
    ///
    /// Defaults to `[monitoring.thresholds] refresh_interval` from the config
    /// file, or 6 seconds when that is unset.
    ///
    /// Minimum: 2 seconds. Maximum: 300 seconds.
    //
    // Deliberately `Option<u64>` rather than `u64` with a clap default: a clap
    // default makes "the user asked for 6" indistinguishable from "the user
    // said nothing", so config could never win. Precedence (flag > config >
    // built-in) is resolved at the dispatch site in `main`.
    #[arg(long, value_name = "SECONDS", requires = "watch", value_parser = clap::value_parser!(u64).range(2..=300))]
    pub interval: Option<u64>,

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the sysconfig command (Sprint 38)
#[derive(Parser, Debug)]
pub struct SysconfigArgs {
    /// Output format
    ///
    /// table: Two-column key-value table (default)
    /// json: JSON object with property keys
    /// csv: Comma-separated Property,Value pairs
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the locks command (Sprint 38)
#[derive(Parser, Debug)]
pub struct LocksArgs {
    /// Output format
    ///
    /// table: Human-readable ASCII table (default)
    /// json: JSON array of lock objects
    /// csv: Comma-separated values
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Enable watch mode (auto-refresh display)
    ///
    /// Continuously refreshes the display at a fixed interval.
    /// Press q or Ctrl-C to stop. Incompatible with --output.
    #[arg(long, conflicts_with = "output")]
    pub watch: bool,

    /// Refresh interval in seconds for watch mode
    ///
    /// Defaults to `[monitoring.thresholds] refresh_interval` from the config
    /// file, or 6 seconds when that is unset.
    ///
    /// Minimum: 2 seconds. Maximum: 300 seconds.
    //
    // Deliberately `Option<u64>` rather than `u64` with a clap default: a clap
    // default makes "the user asked for 6" indistinguishable from "the user
    // said nothing", so config could never win. Precedence (flag > config >
    // built-in) is resolved at the dispatch site in `main`.
    #[arg(long, value_name = "SECONDS", requires = "watch", value_parser = clap::value_parser!(u64).range(2..=300))]
    pub interval: Option<u64>,

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
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
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
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
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
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
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Number of rows per page (enables pagination)
    ///
    /// When specified, results are split into pages of this size.
    /// Use with --page to select which page to retrieve.
    #[arg(long, value_name = "N")]
    pub page_size: Option<usize>,

    /// Page number to retrieve (1-based, default: 1)
    ///
    /// Requires --page-size. Returns the specified page of results.
    #[arg(long, value_name = "P", default_value = "1", requires = "page_size")]
    pub page: usize,

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
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

/// Arguments for the search command
#[derive(Parser, Debug)]
pub struct SearchArgs {
    /// Type of objects to search for
    #[arg(value_name = "TYPE")]
    pub object_type: SearchObjectType,

    /// Search keyword (case-insensitive substring match)
    #[arg(value_name = "KEYWORD")]
    pub keyword: String,

    /// Restrict search to a specific database
    #[arg(short, long, value_name = "DB")]
    pub database: Option<String>,

    /// Output format
    ///
    /// table: Human-readable listing (default)
    /// json: JSON array of objects
    /// csv: Comma-separated values
    /// markdown/md: GitHub-Flavored Markdown table
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Maximum number of results (default: 100, 0 for unlimited)
    #[arg(short = 'n', long, value_name = "N", conflicts_with = "page_size")]
    pub limit: Option<usize>,

    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Number of rows per page (enables pagination)
    ///
    /// When specified, results are split into pages of this size.
    /// Use with --page to select which page to retrieve.
    /// Mutually exclusive with --limit.
    #[arg(long, value_name = "N", conflicts_with = "limit")]
    pub page_size: Option<usize>,

    /// Page number to retrieve (1-based, default: 1)
    ///
    /// Requires --page-size. Returns the specified page of results.
    #[arg(long, value_name = "P", default_value = "1", requires = "page_size")]
    pub page: usize,

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
}

/// Types of objects that can be searched
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SearchObjectType {
    /// Search for tables by name
    Tables,
    /// Search for columns by name
    Columns,
    /// Search for views by name
    Views,
    /// Search for stored procedures by name
    Procedures,
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
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the abort command (Sprint 49, Sprint 61)
///
/// Supports three modes:
/// - Single session: `tq abort --force 1234`
/// - All sessions for a user: `tq abort --force --user alice`
/// - All sessions from a host: `tq abort --force --host myserver01`
#[derive(Parser, Debug)]
pub struct AbortArgs {
    /// Session ID to abort
    ///
    /// The Teradata session number to terminate.
    /// Conflicts with --user and --host.
    #[arg(value_name = "SESSION_ID", conflicts_with_all = ["user", "host"])]
    pub session_id: Option<i64>,

    /// Abort all sessions for a specific user
    ///
    /// Queries MonitorSession to find all active sessions owned by the
    /// specified username, then aborts each one.
    #[arg(long, value_name = "USERNAME", conflicts_with_all = ["session_id", "host"])]
    pub user: Option<String>,

    /// Abort all sessions from a specific hostname
    ///
    /// Queries MonitorSession to find all active sessions whose LogonSource
    /// contains the specified hostname, then aborts each one.
    #[arg(long, value_name = "HOSTNAME", conflicts_with_all = ["session_id", "user"])]
    pub host: Option<String>,

    /// Abort only the running query, not the entire session
    ///
    /// Cancels the currently executing query while keeping the session alive.
    /// Only valid with a single session ID.
    #[arg(long, conflicts_with_all = ["user", "host"])]
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
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the logoff-idle command (Sprint 61)
///
/// Detects idle sessions older than a threshold and aborts them.
/// Useful for cleaning up stale connections that waste resources.
///
/// Example: tq logoff-idle --force
///          tq logoff-idle --force --older-than 2h
#[derive(Parser, Debug)]
pub struct LogoffIdleArgs {
    /// Minimum idle duration before a session is eligible for logoff
    ///
    /// Sessions whose logon time is older than this threshold AND are in
    /// IDLE state will be terminated. Default: 1h.
    /// Supported formats: 30m, 1h, 2h, 24h, 7d.
    #[arg(long, default_value = "1h", value_name = "DURATION")]
    pub older_than: String,

    /// Confirm the logoff operation (required in batch mode)
    ///
    /// This is a destructive operation. The --force flag is required to
    /// prevent accidental termination of idle sessions.
    #[arg(long)]
    pub force: bool,

    /// Output format
    ///
    /// table: Human-readable summary (default)
    /// json: JSON result object
    /// csv: Comma-separated results
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
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
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
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
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the space command
#[derive(Parser, Debug)]
pub struct SpaceArgs {
    /// Database or qualified object to analyze
    ///
    /// `<database>` shows the database header row plus one row per contained
    /// object; `<database>.<object>` shows that object only.
    #[arg(value_name = "TARGET")]
    pub target: String,

    /// Output format
    ///
    /// table: Human-readable table with humanized byte sizes (default)
    /// json: JSON with raw byte integers
    /// csv: Comma-separated values with raw byte integers
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the dbspace command
#[derive(Parser, Debug)]
pub struct DbspaceArgs {
    /// Database to analyze
    ///
    /// Must be a bare database or user name. A qualified `database.object`
    /// argument is rejected — use `tq space <database>.<object>` instead.
    #[arg(value_name = "DATABASE")]
    pub database: String,

    /// Output format
    ///
    /// table: Human-readable table with humanized byte sizes (default)
    /// json: JSON with raw byte integers
    /// csv: Comma-separated values with raw byte integers
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the history command (Sprint 51)
#[derive(Parser, Debug)]
pub struct HistoryArgs {
    /// Time range to query (e.g., 1h, 24h, 7d, 30m)
    ///
    /// Specifies how far back to look for session events.
    /// Default: 1h (one hour).
    #[arg(long, value_name = "DURATION")]
    pub last: Option<String>,

    /// Filter by username
    ///
    /// Show only events for a specific user.
    #[arg(long, value_name = "USERNAME")]
    pub user: Option<String>,

    /// Output format
    ///
    /// table: Summary header + event table (default)
    /// json: JSON object with summary and events
    /// csv: Comma-separated event records
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the resources command
///
/// Displays system resource usage from Teradata ResUsage tables.
/// Default mode shows per-VPROC metrics; use --physical for per-node metrics.
#[derive(Parser, Debug)]
pub struct ResourcesArgs {
    /// Show per-node (physical) metrics instead of per-VPROC (virtual)
    ///
    /// Default: virtual mode using DBC.ResUsageSVPR
    /// Physical mode uses DBC.ResUsageSPMA
    #[arg(long)]
    pub physical: bool,

    /// Output format
    ///
    /// table: Human-readable ASCII table (default)
    /// json: JSON object with resource metrics and skew
    /// csv: Comma-separated values
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Number of rows per page (enables pagination)
    ///
    /// When specified, results are split into pages of this size.
    /// Use with --page to select which page to retrieve.
    #[arg(long, value_name = "N")]
    pub page_size: Option<usize>,

    /// Page number to retrieve (1-based, default: 1)
    ///
    /// Requires --page-size. Returns the specified page of results.
    #[arg(long, value_name = "P", default_value = "1", requires = "page_size")]
    pub page: Option<usize>,

    /// Enable watch mode (auto-refresh display)
    ///
    /// Continuously refreshes the display at a fixed interval.
    /// Press q or Ctrl-C to stop. Incompatible with --output.
    #[arg(long, conflicts_with = "output")]
    pub watch: bool,

    /// Refresh interval in seconds for watch mode
    ///
    /// Defaults to `[monitoring.thresholds] refresh_interval` from the config
    /// file, or 6 seconds when that is unset.
    ///
    /// Minimum: 2 seconds. Maximum: 300 seconds.
    //
    // Deliberately `Option<u64>` rather than `u64` with a clap default: a clap
    // default makes "the user asked for 6" indistinguishable from "the user
    // said nothing", so config could never win. Precedence (flag > config >
    // built-in) is resolved at the dispatch site in `main`.
    #[arg(long, value_name = "SECONDS", requires = "watch", value_parser = clap::value_parser!(u64).range(2..=300))]
    pub interval: Option<u64>,

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
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
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
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
    /// markdown/md: GitHub-Flavored Markdown table
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

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
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
///
/// Controls how command output is rendered:
/// - `table`: Human-readable bordered table (default for terminals)
/// - `json`: JSON array of objects with type preservation
/// - `csv`: RFC 4180 compliant comma-separated values
/// - `markdown`/`md`: GitHub-Flavored Markdown table
///
/// For commands that return multiple sections (e.g. `inspect`), `csv`
/// outputs only the primary tabular section while `json` and `markdown`
/// include all sections in a structured format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table with borders (default)
    Table,
    /// JSON array of objects
    Json,
    /// Comma-separated values (RFC 4180)
    Csv,
    /// GitHub-Flavored Markdown table
    Markdown,
    /// GitHub-Flavored Markdown table (alias for markdown)
    Md,
}

impl OutputFormat {
    /// Normalize aliases to their canonical variant
    pub fn canonical(self) -> Self {
        match self {
            OutputFormat::Md => OutputFormat::Markdown,
            other => other,
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Csv => write!(f, "csv"),
            OutputFormat::Markdown | OutputFormat::Md => write!(f, "markdown"),
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

/// Arguments for the fastload command
#[derive(Parser, Debug)]
pub struct FastloadArgs {
    /// Path to the source file (CSV, Parquet, or JSON)
    #[arg(value_name = "SOURCE_FILE")]
    pub source_file: std::path::PathBuf,

    /// Target table name (e.g. database.table or table)
    #[arg(value_name = "TARGET_TABLE")]
    pub target_table: String,

    /// Force a specific source file format
    #[arg(long, value_name = "FORMAT", value_enum)]
    pub source_format: Option<SourceFormat>,

    /// Field separator for CSV/TSV source files (default: comma, or tab if file extension is .tsv)
    #[arg(long, value_name = "CHAR")]
    pub delimiter: Option<String>,

    /// Disable automatic table creation if it does not exist
    #[arg(long)]
    pub no_create: bool,

    /// Number of parallel data transfer connections (default: let database choose)
    #[arg(long, value_name = "N")]
    pub sessions: Option<usize>,

    /// Database name for FastLoad error tables
    #[arg(long, value_name = "DB")]
    pub error_table_db: Option<String>,

    /// Suffix for FastLoad Error Table 1
    #[arg(long, value_name = "SUFFIX", default_value = "_ERR_1")]
    pub error_table_1_suffix: String,

    /// Suffix for FastLoad Error Table 2
    #[arg(long, value_name = "SUFFIX", default_value = "_ERR_2")]
    pub error_table_2_suffix: String,

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
}

/// Supported source file formats for FastLoad
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SourceFormat {
    /// Comma-separated values
    Csv,
    /// Apache Parquet
    Parquet,
    /// JSON array of objects or NDJSON
    Json,
}

/// Arguments for the fastexport command
#[derive(Parser, Debug)]
pub struct FastexportArgs {
    /// Source table or view name (e.g. database.table or table)
    #[arg(value_name = "SOURCE_TABLE")]
    pub source_table: String,

    /// Path to the destination CSV file
    #[arg(value_name = "TARGET_FILE")]
    pub target_file: std::path::PathBuf,

    /// Number of parallel data transfer connections (default: let database choose)
    #[arg(long, value_name = "N")]
    pub sessions: Option<usize>,

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the params command
#[derive(Parser, Debug)]
pub struct ParamsArgs {
    /// Parameter file(s) to inspect
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Output format
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the errorlevel command
#[derive(Parser, Debug)]
pub struct ErrorlevelArgs {
    /// Output format
    #[arg(
        short,
        long,
        env = "TQ_FORMAT",
        default_value = "table",
        value_name = "FORMAT"
    )]
    pub format: OutputFormat,

    /// Shortcut for --format json
    #[arg(long)]
    pub json: bool,
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

    // Sprint 52: Markdown format tests

    #[test]
    fn test_cli_query_with_markdown_format() {
        let args = vec!["tq", "query", "--format", "markdown", "SELECT 1"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Query(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Markdown);
        } else {
            panic!("Expected Query command");
        }
    }

    #[test]
    fn test_cli_query_with_md_alias() {
        let args = vec!["tq", "query", "--format", "md", "SELECT 1"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Query(args) = cli.command {
            assert_eq!(args.format, OutputFormat::Md);
        } else {
            panic!("Expected Query command");
        }
    }

    #[test]
    fn test_output_format_canonical() {
        assert_eq!(OutputFormat::Md.canonical(), OutputFormat::Markdown);
        assert_eq!(OutputFormat::Markdown.canonical(), OutputFormat::Markdown);
        assert_eq!(OutputFormat::Table.canonical(), OutputFormat::Table);
        assert_eq!(OutputFormat::Json.canonical(), OutputFormat::Json);
        assert_eq!(OutputFormat::Csv.canonical(), OutputFormat::Csv);
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Table.to_string(), "table");
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Csv.to_string(), "csv");
        assert_eq!(OutputFormat::Markdown.to_string(), "markdown");
        assert_eq!(OutputFormat::Md.to_string(), "markdown");
    }

    #[test]
    fn test_cli_fastload_basic() {
        let args = vec!["tq", "fastload", "data.csv", "mydb.mytable"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Fastload(args) = cli.command {
            assert_eq!(args.source_file, std::path::PathBuf::from("data.csv"));
            assert_eq!(args.target_table, "mydb.mytable");
            assert_eq!(args.source_format, None);
            assert!(!args.no_create);
            assert_eq!(args.sessions, None);
        } else {
            panic!("Expected Fastload command");
        }
    }

    #[test]
    fn test_cli_fastload_with_options() {
        let args = vec![
            "tq", "fastload", "data.json", "mytable",
            "--source-format", "json",
            "--no-create",
            "--sessions", "4",
            "--error-table-db", "errdb",
            "--error-table-1-suffix", "_err1",
            "--error-table-2-suffix", "_err2"
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Fastload(args) = cli.command {
            assert_eq!(args.source_file, std::path::PathBuf::from("data.json"));
            assert_eq!(args.target_table, "mytable");
            assert_eq!(args.source_format, Some(SourceFormat::Json));
            assert!(args.no_create);
            assert_eq!(args.sessions, Some(4));
            assert_eq!(args.error_table_db, Some("errdb".to_string()));
            assert_eq!(args.error_table_1_suffix, "_err1");
            assert_eq!(args.error_table_2_suffix, "_err2");
        } else {
            panic!("Expected Fastload command");
        }
    }

    #[test]
    fn test_cli_fastexport_basic() {
        let args = vec!["tq", "fastexport", "mydb.mytable", "data.csv"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::Fastexport(args) = cli.command {
            assert_eq!(args.source_table, "mydb.mytable");
            assert_eq!(args.target_file, std::path::PathBuf::from("data.csv"));
            assert_eq!(args.sessions, None);
        } else {
            panic!("Expected Fastexport command");
        }
    }

    // Sprint 77: Tech debt & agent ergonomics tests
    #[test]
    fn test_cli_json_shortcut_flag() {
        let args = vec!["tq", "query", "--json", "SELECT 1"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.command.format(), Some(OutputFormat::Json));
    }

    #[test]
    fn test_cli_global_json_flag() {
        let args = vec!["tq", "--json", "query", "SELECT 1"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.global.json);
    }

    #[test]
    fn test_cli_global_agent_safe_flag() {
        let args = vec!["tq", "--agent-safe", "query", "SELECT 1"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.global.agent_safe);
    }

    #[test]
    fn test_cli_query_inspect_qi_alias() {
        let args = vec!["tq", "qi", "1234"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::QueryInspect(args) = cli.command {
            assert_eq!(args.session_id, 1234);
        } else {
            panic!("Expected QueryInspect command");
        }
    }

    #[test]
    fn test_cli_show_indexes_di_alias() {
        let args = vec!["tq", "di", "employees"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Command::ShowIndexes(args) = cli.command {
            assert_eq!(args.table, "employees");
        } else {
            panic!("Expected ShowIndexes command");
        }
    }
}
