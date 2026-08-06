#![deny(warnings)]
//! tq - Teradata Query CLI
//!
//! Entry point for the tq command-line tool.
//! This binary provides a fast, lightweight interface to Teradata databases.

use clap::Parser;
use std::io::{self};
use std::process::ExitCode;

use tq::cli::{Cli, Command, GlobalOpts, HelpTopic, OutputFormat};
use tq::config::{parse_logmech, Config};
use tq::db::{parse_duration, ConnectionConfig, DatabaseClient};
use tq::error::TqError;
use tq::params::ParamStore;
use tq::{commands, help, Result};

fn main() -> ExitCode {
    // Load environment variables from .env file if present
    // This allows users to store connection details and other config in .env
    // Silently ignore if .env doesn't exist
    let _ = dotenvy::dotenv();

    // Initialize logger from environment
    // TQ_LOG=debug tq ping will show debug logs
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Extract format before consuming cli (for structured error output)
    let is_json_format = cli
        .command
        .format()
        .map(|f| matches!(f.canonical(), OutputFormat::Json))
        .unwrap_or(false);

    // Run the application
    match run(cli) {
        Ok(exit_code) => ExitCode::from(exit_code),
        Err(e) => {
            if is_json_format {
                // Structured JSON error to stdout (for agent consumption)
                println!("{}", e.to_json());
            } else {
                // Human-readable error to stderr
                eprintln!("{}", e.user_message());
            }

            // Return appropriate exit code
            match e.exit_code() {
                2 => ExitCode::from(2), // Usage error
                _ => ExitCode::FAILURE, // Runtime error
            }
        }
    }
}

/// Main application logic
fn run(cli: Cli) -> Result<u8> {
    // Load configuration from files and environment
    let config = Config::load().unwrap_or_else(|e| {
        log::warn!("Failed to load config: {}. Using defaults.", e);
        Config::default()
    });

    // Threshold validation is deliberately separate from Config::load(): a load
    // failure degrades gracefully to defaults above, but a semantically invalid
    // threshold must be a hard error rather than silently reverting.
    config.monitoring.validate()?;

    // Handle commands that don't require database connection
    match &cli.command {
        Command::Help(args) => {
            handle_help(args)?;
            return Ok(0);
        }
        Command::Profiles => {
            handle_profiles(&config)?;
            return Ok(0);
        }
        Command::Profile(action) => {
            commands::profile::execute(action, &config)?;
            return Ok(0);
        }
        _ => {}
    }

    // Parse error level overrides from CLI
    let error_levels = tq::error::parse_errorlevel(&cli.global.errorlevel)?;

    // Build ParamStore from --params flag(s)
    let param_store = build_param_store(&cli.global.params)?;

    // Build connection configuration for database commands.
    //
    // Input-source selection (positional arg vs --file vs stdin) is decided
    // syntactically inside the query command itself; the only argument-level
    // conflict (query + --file) is enforced by clap at parse time, before any
    // connection is built. There is no pre-connection input-source probe.
    let password_override = read_password_if_needed(&cli.global)?;
    let mut conn_config = build_connection_config(&cli.global, &config, password_override)?;

    // Resolve the query/request timeout (distinct from the connection
    // timeout). An explicit --query-timeout always wins. In agent-safe query
    // mode, apply a conservative finite default (30s) when none is given so an
    // agent can never launch an unbounded request. Outside agent-safe mode the
    // default is "no query timeout".
    conn_config.query_timeout = resolve_query_timeout(&cli.global, &cli.command)?;

    // Create database client
    let client = DatabaseClient::new(conn_config, cli.global.driver_lib_dir.clone())?;

    // Determine output settings
    let use_color = cli.global.color.should_use_color();
    let verbose = cli.global.verbose > 0;

    // Severity thresholds and palette, resolved once for every monitoring
    // command. Structured formats never receive the styler.
    let monitoring = commands::severity::MonitoringContext::new(
        &config.monitoring.thresholds,
        &config.monitoring.colors,
        use_color,
    );

    // Watch-mode refresh interval: CLI flag > config > built-in default.
    let refresh_interval = config.monitoring.thresholds.refresh_interval;

    // Execute database commands
    let exit_code = match cli.command {
        Command::Ping(args) => {
            let mut stdout = io::stdout();
            commands::ping(&client, &args, &mut stdout, verbose)?;
            0
        }
        Command::Query(args) => {
            if args.output.is_some() {
                // Write to file
                let mut stderr = io::stderr();
                commands::query::execute_to_file(
                    &client, &args, Some(&param_store), &mut stderr, use_color, verbose, &error_levels,
                )?
            } else {
                // Write to stdout
                let mut stdout = io::stdout();
                commands::query::execute(
                    &client, &args, Some(&param_store), &mut stdout, use_color, verbose, &error_levels,
                )?
            }
        }
        Command::Repl(args) => {
            let mut stdout = io::stdout();
            // Pass parsed error levels to the REPL
            commands::repl::execute(
                client,
                &args,
                Some(param_store),
                &mut stdout,
                use_color,
                verbose,
                error_levels,
                monitoring,
            )?;
            0
        }
        // Sprint 26: Sessions command for system monitoring
        Command::Sessions(args) => {
            if args.watch {
                commands::watch::run_watch(args.interval.unwrap_or(refresh_interval), |buf| {
                    commands::sessions(&client, &args, buf, use_color)
                })?;
            } else if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::sessions(&client, &args, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::sessions(&client, &args, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 33: Sample command for random data sampling
        Command::Sample(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::sample(&client, &args, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::sample(&client, &args, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 33: Peek command for data preview with column metadata
        Command::Peek(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::peek(&client, &args, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::peek(&client, &args, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 38: Sysconfig command for system topology
        Command::Sysconfig(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::sysconfig(&client, &args, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::sysconfig(&client, &args, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 38: Locks command for lock contention analysis
        Command::Locks(args) => {
            if args.watch {
                commands::watch::run_watch(args.interval.unwrap_or(refresh_interval), |buf| {
                    commands::locks(&client, &args, buf, use_color)
                })?;
            } else if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::locks(&client, &args, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::locks(&client, &args, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 39: Query inspection for session drill-down
        Command::QueryInspect(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::query_inspect(&client, &args, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::query_inspect(&client, &args, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 45: Inspect command for object metadata
        Command::Inspect(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::inspect::execute(&client, &args.object, args.format, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::inspect::execute(&client, &args.object, args.format, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 46: List command
        Command::List(ref args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::list::execute(&client, args, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::list::execute(&client, args, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 55: Search command for cross-database discovery
        Command::Search(ref args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::search::execute(&client, args, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::search::execute(&client, args, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 46: Show-indexes command
        Command::ShowIndexes(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::show_indexes::execute(&client, &args.table, args.format, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::show_indexes::execute(&client, &args.table, args.format, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 49: Abort command for session control
        Command::Abort(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::abort(&client, &args, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::abort(&client, &args, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 50: Explain command for query analysis
        Command::Explain(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::explain(&client, &args, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::explain(&client, &args, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 51: Session history command
        Command::History(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::history(&client, &args, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::history(&client, &args, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 50: Skew command for AMP resource analysis
        Command::Skew(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::skew(&client, &args, &mut writer, &monitoring)?;
            } else {
                let mut stdout = io::stdout();
                commands::skew(&client, &args, &mut stdout, &monitoring)?;
            }
            0
        }
        // Space analysis for a database or object
        Command::Space(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::space(&client, &args, &mut writer, &monitoring)?;
            } else {
                let mut stdout = io::stdout();
                commands::space(&client, &args, &mut stdout, &monitoring)?;
            }
            0
        }
        // Database-level space analysis
        Command::Dbspace(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::dbspace(&client, &args, &mut writer, &monitoring)?;
            } else {
                let mut stdout = io::stdout();
                commands::dbspace(&client, &args, &mut stdout, &monitoring)?;
            }
            0
        }
        // Resources command for PMON resource monitoring
        Command::Resources(args) => {
            if args.watch {
                commands::watch::run_watch(args.interval.unwrap_or(refresh_interval), |buf| {
                    commands::resources(&client, &args, buf, &monitoring)
                })?;
            } else if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::resources(&client, &args, &mut writer, &monitoring)?;
            } else {
                let mut stdout = io::stdout();
                commands::resources(&client, &args, &mut stdout, &monitoring)?;
            }
            0
        }
        // Sprint 61: Logoff idle sessions
        Command::LogoffIdle(args) => {
            if let Some(ref output_path) = args.output {
                let file = std::fs::File::create(output_path)?;
                let mut writer = std::io::BufWriter::new(file);
                commands::logoff_idle(&client, &args, &mut writer, use_color)?;
            } else {
                let mut stdout = io::stdout();
                commands::logoff_idle(&client, &args, &mut stdout, use_color)?;
            }
            0
        }
        // Sprint 73: Fastload bulk loader
        Command::Fastload(args) => {
            commands::fastload::execute(&client, &args)?;
            0
        }
        // Sprint 73: Fastexport bulk exporter
        Command::Fastexport(args) => {
            commands::fastexport::execute(&client, &args)?;
            0
        }
        // Help, Profiles, and Profile already handled above
        Command::Help(_) | Command::Profiles | Command::Profile(_) => unreachable!(),
    };

    Ok(exit_code)
}

/// Handle the help command
fn handle_help(args: &tq::HelpArgs) -> Result<()> {
    let content = match args.topic {
        Some(HelpTopic::Config) => help::config_help(),
        Some(HelpTopic::Credentials) => help::credentials_help(),
        Some(HelpTopic::Params) => help::params_help(),
        None => help::general_help(),
    };

    println!("{}", content);
    Ok(())
}

/// Handle the profiles command
///
/// Displays profiles from both user config and project config, with source indicators.
/// Shows profiles grouped by source (user-only, project-only, or merged from both).
///
/// Sprint 36: Shows config file paths header and project config tip in empty state.
fn handle_profiles(config: &Config) -> Result<()> {
    // Load configs separately to track profile sources
    let user_config = Config::load_user_only();
    let project_config = Config::load_project_only();
    let project_config_path = Config::project_config_path();

    let user_profile_names: std::collections::HashSet<_> =
        user_config.profiles.keys().cloned().collect();
    let project_profile_names: std::collections::HashSet<_> = project_config
        .as_ref()
        .map(|c| c.profiles.keys().cloned().collect())
        .unwrap_or_default();

    // Collect all profile names
    let all_profile_names: std::collections::HashSet<_> = user_profile_names
        .union(&project_profile_names)
        .cloned()
        .collect();

    if all_profile_names.is_empty() {
        println!("No profiles defined.\n");
        println!(
            "To create a profile, add to {}:\n",
            Config::user_config_path().display()
        );
        println!("  [profiles.myprofile]");
        println!("  host = \"myhost.example.com\"");
        println!("  port = 1025");
        println!("  database = \"mydb\"");
        println!("  user = \"myuser\"");
        println!("  password_file = \"~/.tq/passwords/myprofile\"");
        println!();
        println!("Tip: Create .tq.toml in your project root for team-shared profiles");
        return Ok(());
    }

    // Show config file paths header
    println!("User config: {}", Config::user_config_path().display());
    if let Some(ref path) = project_config_path {
        println!("Project config: {}", path.display());
    }
    println!();

    println!("Available profiles:\n");

    // Categorize profiles by source
    let mut user_only: Vec<_> = user_profile_names
        .difference(&project_profile_names)
        .cloned()
        .collect();
    let mut project_only: Vec<_> = project_profile_names
        .difference(&user_profile_names)
        .cloned()
        .collect();
    let mut merged: Vec<_> = user_profile_names
        .intersection(&project_profile_names)
        .cloned()
        .collect();

    user_only.sort();
    project_only.sort();
    merged.sort();

    // Show user-only profiles
    if !user_only.is_empty() {
        println!(
            "From user config ({}):",
            Config::user_config_path().display()
        );
        for name in &user_only {
            if let Some(profile) = config.profiles.get(name) {
                commands::profile::display_profile(name, profile, None);
            }
        }
    }

    // Show project-only profiles
    if !project_only.is_empty() {
        if let Some(path) = Config::project_config_path() {
            println!("From project config ({}):", path.display());
            for name in &project_only {
                if let Some(profile) = config.profiles.get(name) {
                    commands::profile::display_profile(name, profile, None);
                }
            }
        }
    }

    // Show merged profiles (exist in both)
    if !merged.is_empty() {
        println!("From both (merged - project overrides user):");
        for name in &merged {
            if let Some(profile) = config.profiles.get(name) {
                // Get user and project profiles for field-level source indication
                let user_profile = user_config.profiles.get(name);
                let project_profile = project_config.as_ref().and_then(|c| c.profiles.get(name));
                print_merged_profile(name, profile, user_profile, project_profile);
            }
        }
    }

    println!("Use: tq --profile <name> <command>");
    Ok(())
}

/// Print a merged profile with source indicators for each field
fn print_merged_profile(
    name: &str,
    merged: &tq::config::ConnectionSettings,
    user_profile: Option<&tq::config::ConnectionSettings>,
    project_profile: Option<&tq::config::ConnectionSettings>,
) {
    println!("  {}", name);

    // Host
    let host = merged.host.as_deref().unwrap_or("<not set>");
    let host_source = field_source(
        project_profile.and_then(|p| p.host.as_ref()),
        user_profile.and_then(|p| p.host.as_ref()),
    );
    println!("    Host:     {}  {}", host, host_source);

    // Database
    let database = merged.database.as_deref().unwrap_or("<not set>");
    let db_source = field_source(
        project_profile.and_then(|p| p.database.as_ref()),
        user_profile.and_then(|p| p.database.as_ref()),
    );
    println!("    Database: {}  {}", database, db_source);

    // User
    let user = merged.user.as_deref().unwrap_or("<not set>");
    let user_source = field_source(
        project_profile.and_then(|p| p.user.as_ref()),
        user_profile.and_then(|p| p.user.as_ref()),
    );
    println!("    User:     {}  {}", user, user_source);

    // Show logmech if not default
    if let Some(ref logmech) = merged.logmech {
        if logmech.to_uppercase() != "TD2" {
            let logmech_source = field_source(
                project_profile.and_then(|p| p.logmech.as_ref()),
                user_profile.and_then(|p| p.logmech.as_ref()),
            );
            println!("    Logmech:  {}  {}", logmech, logmech_source);
        }
    }
    println!();
}

/// Determine source indicator for a merged field
fn field_source<T>(project_value: Option<T>, user_value: Option<T>) -> &'static str {
    match (project_value.is_some(), user_value.is_some()) {
        (true, _) => "[project]",
        (false, true) => "[user]",
        (false, false) => "[default]",
    }
}

/// Read password from file if --password-file is specified
fn read_password_if_needed(global: &GlobalOpts) -> Result<Option<String>> {
    let Some(ref password_file) = global.password_file else {
        return Ok(None);
    };

    // SECURITY: Validate file permissions BEFORE reading file content
    // This prevents loading sensitive data from insecure files
    #[cfg(unix)]
    validate_password_file_permissions(password_file)?;

    // Read password from file only after permission validation passes
    let password = std::fs::read_to_string(password_file).map_err(|e| TqError::FileReadError {
        path: password_file.clone(),
        source: e,
    })?;

    Ok(Some(password.trim().to_string()))
}

/// Validate password file has secure permissions on Unix
///
/// Returns error if file permissions allow group or world access.
/// This enforces the security requirement that password files have 0600 permissions.
#[cfg(unix)]
fn validate_password_file_permissions(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path).map_err(|e| TqError::FileReadError {
        path: path.to_path_buf(),
        source: e,
    })?;

    let mode = metadata.permissions().mode() & 0o777;

    // SECURITY: Enforce (not just warn) that password files have 0600 permissions
    // Files with group or world access are rejected to prevent credential exposure
    if mode & 0o077 != 0 {
        return Err(TqError::InvalidConfig(format!(
            "Password file '{}' has insecure permissions {:04o}. Required: 0600.\n\
             \n\
             Security risk: File is readable by other users on this system.\n\
             \n\
             Fix: chmod 0600 {}",
            path.display(),
            mode,
            path.display()
        )));
    }

    Ok(())
}

/// Resolve the effective query/request timeout for this invocation.
///
/// Precedence:
/// 1. An explicit `--query-timeout <DURATION>` (parsed) always applies.
/// 2. Otherwise, in agent-safe query mode, a conservative finite default of
///    30 seconds is applied so an agent cannot launch an unbounded request.
/// 3. Otherwise `None` (no query timeout).
fn resolve_query_timeout(
    global: &GlobalOpts,
    command: &Command,
) -> Result<Option<std::time::Duration>> {
    if let Some(ref qt) = global.query_timeout {
        return Ok(Some(parse_duration(qt)?));
    }
    if let Command::Query(args) = command {
        if args.agent_safe {
            return Ok(Some(std::time::Duration::from_secs(30)));
        }
    }
    Ok(None)
}

/// Build connection configuration from CLI args and config file
fn build_connection_config(
    global: &GlobalOpts,
    config: &Config,
    password_override: Option<String>,
) -> Result<ConnectionConfig> {
    // Try to build from CLI --logon option first
    if let Some(ref logon) = global.logon {
        let timeout = parse_duration(&global.timeout)?;
        return ConnectionConfig::from_connection_string(
            logon,
            global.logmech,
            timeout,
            password_override,
        );
    }

    // If --profile is specified, load from that profile
    if let Some(ref profile_name) = global.profile {
        return build_connection_from_profile(global, config, profile_name, password_override);
    }

    // Otherwise, try to build from config file default connection
    config.build_connection_config(global, password_override)
}

/// Build connection configuration from a named profile
fn build_connection_from_profile(
    global: &GlobalOpts,
    config: &Config,
    profile_name: &str,
    password_override: Option<String>,
) -> Result<ConnectionConfig> {
    use tq::config::expand_home_dir;
    use tq::config::read_password_from_file;

    // Get the profile
    let profile = config.get_profile(profile_name).ok_or_else(|| {
        let available: Vec<_> = config.profiles.keys().collect();
        if available.is_empty() {
            TqError::InvalidConfig(format!(
                "Profile '{}' not found. No profiles defined in config file.\n\
                 \n\
                 To create a profile, add to ~/.tq/config.toml:\n\
                 \n\
                 [profiles.{}]\n\
                 host = \"your-host.example.com\"\n\
                 database = \"your_database\"\n\
                 user = \"your_username\"\n\
                 password_file = \"~/.tq/passwords/{}\"",
                profile_name, profile_name, profile_name
            ))
        } else {
            TqError::InvalidConfig(format!(
                "Profile '{}' not found.\n\
                 \n\
                 Available profiles:\n  - {}\n\
                 \n\
                 Use --profile <name> to select one.",
                profile_name,
                available
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("\n  - ")
            ))
        }
    })?;

    // Determine password: CLI override > profile password_file > prompt
    let password = if let Some(pw) = password_override {
        Some(secrecy::Secret::new(pw))
    } else if let Some(ref pw_file) = profile.password_file {
        let expanded = expand_home_dir(pw_file);
        Some(secrecy::Secret::new(read_password_from_file(&expanded)?))
    } else {
        None // Will prompt interactively if needed
    };

    // Build connection from profile settings
    let host = profile
        .host
        .clone()
        .ok_or_else(|| TqError::InvalidConfig(format!(
            "Profile '{}' is missing required field 'host'",
            profile_name
        )))?;

    let port = profile.port.unwrap_or(1025);

    let database = profile.database.clone().ok_or_else(|| {
        TqError::InvalidConfig(format!(
            "Profile '{}' is missing required field 'database'",
            profile_name
        ))
    })?;

    let user = profile.user.clone().ok_or_else(|| {
        TqError::InvalidConfig(format!(
            "Profile '{}' is missing required field 'user'",
            profile_name
        ))
    })?;

    // Parse logmech from profile or use CLI default
    let logmech = if let Some(ref lm) = profile.logmech {
        parse_logmech(lm)?
    } else {
        global.logmech
    };

    // Parse timeout from profile or use CLI default
    let timeout_str = profile.timeout.as_deref().unwrap_or(&global.timeout);
    let timeout = parse_duration(timeout_str)?;

    Ok(ConnectionConfig {
        host,
        port,
        database,
        user,
        password,
        logmech,
        timeout,
        // Query timeout is resolved centrally in run() (explicit flag or the
        // agent-safe default), then assigned onto the built config.
        query_timeout: None,
    })
}

/// Build a ParamStore from a list of parameter file paths
fn build_param_store(paths: &[std::path::PathBuf]) -> Result<ParamStore> {
    let mut store = ParamStore::new();
    for path in paths {
        store.load_file(path)?;
    }
    Ok(store)
}
