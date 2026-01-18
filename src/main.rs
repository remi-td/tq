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

    // Otherwise, try to build from config file
    config.build_connection_config(global, password_override)
}
