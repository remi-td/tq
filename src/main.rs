use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use tq::cli::{Cli, OutputFormat};
use tq::connection::ConnectionConfig;
use tq::db::{DatabaseClient, QueryResults};

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Read password from file if provided
    let password_override = if let Some(password_file) = &cli.password_file {
        let password = fs::read_to_string(password_file)
            .with_context(|| format!("Failed to read password from file: {:?}", password_file))?
            .trim()
            .to_string();

        // Validate file permissions on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(password_file)
                .with_context(|| format!("Failed to read file metadata: {:?}", password_file))?;
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;

            if mode & 0o077 != 0 {
                log::warn!(
                    "Password file {:?} has insecure permissions {:o}. Recommended: 0600",
                    password_file,
                    mode
                );
            }
        }

        Some(password)
    } else {
        None
    };

    // Parse the connection string
    let config = ConnectionConfig::parse(&cli.logon, &cli.logmech, password_override)
        .context("Failed to parse connection string")?;

    // Create database client
    let client = DatabaseClient::new(config.clone(), cli.driver_lib_dir)
        .context("Failed to create database client")?;

    // Handle the ping command
    if cli.ping {
        println!(
            "Pinging Teradata database at {}:{}...",
            config.host, config.port
        );

        let latency = client.ping().context("Ping failed")?;

        println!("Success! Database is reachable.");
        println!("  Host: {}", config.host);
        println!("  Port: {}", config.port);
        println!("  User: {}", config.user);
        println!("  Database: {}", config.database);
        println!("  Logon Mechanism: {}", config.logmech);
        println!("  Latency: {:.2}ms", latency.as_secs_f64() * 1000.0);

        return Ok(());
    }

    // Handle query execution
    if let Some(query) = cli.query {
        let results = client
            .execute_query(&query)
            .context("Query execution failed")?;

        // Format and display results
        format_results(&results, cli.format)?;
    } else {
        anyhow::bail!(
            "No command specified. Either use --ping to test connectivity or provide a SQL query."
        );
    }

    Ok(())
}

/// Format and display query results based on the specified format
fn format_results(results: &QueryResults, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Table => format_table(results),
        OutputFormat::Json => format_json(results),
        OutputFormat::Csv => format_csv(results),
    }
}

/// Format results as a simple table
fn format_table(results: &QueryResults) -> Result<()> {
    if results.is_empty() {
        println!("No results returned.");
        return Ok(());
    }

    let rows = &results.rows;

    // Find column widths
    let num_cols = rows[0].len();
    let mut widths = vec![0; num_cols];

    for row in rows {
        for (i, col) in row.iter().enumerate() {
            widths[i] = widths[i].max(col.len());
        }
    }

    // Print separator
    let separator: String = widths
        .iter()
        .map(|w| "-".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("+");
    println!("+{}+", separator);

    // Print rows
    for row in rows {
        print!("|");
        for (i, col) in row.iter().enumerate() {
            print!(" {:width$} |", col, width = widths[i]);
        }
        println!();
    }

    // Print separator
    println!("+{}+", separator);
    println!("\n{} row(s) returned.", results.row_count());

    Ok(())
}

/// Format results as JSON
fn format_json(results: &QueryResults) -> Result<()> {
    let json = serde_json::to_string_pretty(&results.rows)
        .context("Failed to serialize results to JSON")?;
    println!("{}", json);
    Ok(())
}

/// Format results as CSV
fn format_csv(results: &QueryResults) -> Result<()> {
    for row in &results.rows {
        let csv_row = row
            .iter()
            .map(|col| {
                if col.contains(',') || col.contains('"') || col.contains('\n') {
                    format!("\"{}\"", col.replace('"', "\"\""))
                } else {
                    col.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        println!("{}", csv_row);
    }
    Ok(())
}
