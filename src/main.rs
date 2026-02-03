#![deny(warnings)]
//! tq - Teradata Query CLI
//!
//! Entry point for the tq command-line tool.
//! This binary provides a fast, lightweight interface to Teradata databases.

use clap::Parser;
use std::io::{self};
use std::process::ExitCode;

use tq::cli::{Cli, Command, GlobalOpts, HelpTopic};
use tq::config::{parse_logmech, Config};
use tq::db::{parse_duration, ConnectionConfig, DatabaseClient};
use tq::error::TqError;
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

    // Run the application
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // Print user-friendly error message
            eprintln!("{}", e.user_message());

            // Return appropriate exit code
            match e.exit_code() {
                2 => ExitCode::from(2), // Usage error
                _ => ExitCode::FAILURE, // Runtime error
            }
        }
    }
}

/// Main application logic
fn run(cli: Cli) -> Result<()> {
    // Load configuration from files and environment
    let config = Config::load().unwrap_or_else(|e| {
        log::warn!("Failed to load config: {}. Using defaults.", e);
        Config::default()
    });

    // Handle commands that don't require database connection
    match &cli.command {
        Command::Help(args) => {
            return handle_help(args);
        }
        Command::Profiles => {
            return handle_profiles(&config);
        }
        _ => {}
    }

    // Build connection configuration for database commands
    let password_override = read_password_if_needed(&cli.global)?;
    let conn_config = build_connection_config(&cli.global, &config, password_override)?;

    // Create database client
    let client = DatabaseClient::new(conn_config, cli.global.driver_lib_dir.clone())?;

    // Determine output settings
    let use_color = cli.global.color.should_use_color();
    let verbose = cli.global.verbose > 0;

    // Execute database commands
    match cli.command {
        Command::Ping(args) => {
            let mut stdout = io::stdout();
            commands::ping(&client, &args, &mut stdout, verbose)?;
        }
        Command::Query(args) => {
            if args.output.is_some() {
                // Write to file
                let mut stderr = io::stderr();
                commands::query::execute_to_file(&client, &args, &mut stderr, use_color, verbose)?;
            } else {
                // Write to stdout
                let mut stdout = io::stdout();
                commands::query(&client, &args, &mut stdout, use_color, verbose)?;
            }
        }
        Command::Repl(args) => {
            let mut stdout = io::stdout();
            // Sprint 7: Pass ownership of client to REPL for /logon support
            commands::repl(client, &args, &mut stdout, use_color, verbose)?;
        }
        // Sprint 26: Sessions command for system monitoring
        Command::Sessions(args) => {
            if args.output.is_some() {
                // Write to file
                let file = std::fs::File::create(args.output.as_ref().unwrap())?;
                let mut writer = std::io::BufWriter::new(file);
                commands::sessions(&client, &args, &mut writer, use_color)?;
            } else {
                // Write to stdout
                let mut stdout = io::stdout();
                commands::sessions(&client, &args, &mut stdout, use_color)?;
            }
        }
        // Sprint 33: Sample command for random data sampling
        Command::Sample(args) => {
            if args.output.is_some() {
                // Write to file
                let file = std::fs::File::create(args.output.as_ref().unwrap())?;
                let mut writer = std::io::BufWriter::new(file);
                commands::sample(&client, &args, &mut writer, use_color)?;
            } else {
                // Write to stdout
                let mut stdout = io::stdout();
                commands::sample(&client, &args, &mut stdout, use_color)?;
            }
        }
        // Sprint 33: Peek command for data preview with column metadata
        Command::Peek(args) => {
            if args.output.is_some() {
                // Write to file
                let file = std::fs::File::create(args.output.as_ref().unwrap())?;
                let mut writer = std::io::BufWriter::new(file);
                commands::peek(&client, &args, &mut writer, use_color)?;
            } else {
                // Write to stdout
                let mut stdout = io::stdout();
                commands::peek(&client, &args, &mut stdout, use_color)?;
            }
        }
        // Help and Profiles already handled above
        Command::Help(_) | Command::Profiles => unreachable!(),
    }

    Ok(())
}

/// Handle the help command
fn handle_help(args: &tq::HelpArgs) -> Result<()> {
    let content = match args.topic {
        Some(HelpTopic::Config) => help::config_help(),
        Some(HelpTopic::Credentials) => help::credentials_help(),
        None => help::general_help(),
    };

    println!("{}", content);
    Ok(())
}

/// Handle the profiles command
fn handle_profiles(config: &Config) -> Result<()> {
    if config.profiles.is_empty() {
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
        return Ok(());
    }

    println!("Available profiles:\n");

    // Sort profiles alphabetically for consistent output
    let mut profile_names: Vec<_> = config.profiles.keys().collect();
    profile_names.sort();

    for name in profile_names {
        let profile = config.profiles.get(name).unwrap();
        let host = profile.host.as_deref().unwrap_or("<not set>");
        let database = profile.database.as_deref().unwrap_or("<not set>");
        let user = profile.user.as_deref().unwrap_or("<not set>");

        println!("  {}", name);
        println!("    Host:     {}", host);
        println!("    Database: {}", database);
        println!("    User:     {}", user);

        // Show logmech if not default
        if let Some(ref logmech) = profile.logmech {
            if logmech.to_uppercase() != "TD2" {
                println!("    Logmech:  {}", logmech);
            }
        }
        println!();
    }

    println!("Use: tq --profile <name> <command>");
    Ok(())
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
    })
}
