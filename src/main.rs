#![deny(warnings)]
//! tq - Teradata Query CLI
//!
//! Entry point for the tq command-line tool.
//! This binary provides a fast, lightweight interface to Teradata databases.

use clap::Parser;
use std::io::{self};
use std::process::ExitCode;

use tq::cli::{Cli, Command, GlobalOpts};
use tq::config::Config;
use tq::db::{parse_duration, ConnectionConfig, DatabaseClient};
use tq::error::TqError;
use tq::{commands, Result};

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

    // Build connection configuration
    let password_override = read_password_if_needed(&cli.global)?;
    let conn_config = build_connection_config(&cli.global, &config, password_override)?;

    // Create database client
    let client = DatabaseClient::new(conn_config, cli.global.driver_lib_dir.clone())?;

    // Determine output settings
    let use_color = cli.global.color.should_use_color();
    let verbose = cli.global.verbose > 0;

    // Execute command
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
    }

    Ok(())
}

/// Read password from file if --password-file is specified
fn read_password_if_needed(global: &GlobalOpts) -> Result<Option<String>> {
    let Some(ref password_file) = global.password_file else {
        return Ok(None);
    };

    // Read password from file
    let password = std::fs::read_to_string(password_file).map_err(|e| TqError::FileReadError {
        path: password_file.clone(),
        source: e,
    })?;

    // Validate file permissions on Unix
    #[cfg(unix)]
    validate_password_file_permissions(password_file)?;

    Ok(Some(password.trim().to_string()))
}

/// Validate password file has secure permissions on Unix
#[cfg(unix)]
fn validate_password_file_permissions(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path).map_err(|e| TqError::FileReadError {
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
    use tq::config::{expand_home_dir, read_password_from_file};

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
        match lm.to_uppercase().as_str() {
            "TD2" => tq::cli::LogonMechanism::Td2,
            "LDAP" => tq::cli::LogonMechanism::Ldap,
            "KRB5" => tq::cli::LogonMechanism::Krb5,
            "TDNEGO" => tq::cli::LogonMechanism::Tdnego,
            _ => return Err(TqError::InvalidLogonMechanism(lm.clone())),
        }
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
