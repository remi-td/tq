//! Ping command implementation
//!
//! Tests database connectivity with timing information.
//! Supports multiple pings with configurable interval.

use crate::cli::PingArgs;
use crate::db::{parse_duration, DatabaseClient};
use crate::error::{Result, TqError};
use std::io::Write;
use std::time::Duration;

/// Execute the ping command
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &PingArgs,
    writer: &mut W,
    verbose: bool,
) -> Result<()> {
    let interval = parse_duration(&args.interval)?;
    let count = args.count;

    if verbose {
        writeln!(
            writer,
            "Pinging {}:{} ({} time(s), interval: {:?})...",
            client.config().host,
            client.config().port,
            count,
            interval
        )?;
    }

    let mut successful = 0;
    let mut failed = 0;
    let mut total_time = Duration::ZERO;
    let mut min_time = Duration::MAX;
    let mut max_time = Duration::ZERO;

    for i in 0..count {
        // Add interval delay between pings (not before first)
        if i > 0 {
            std::thread::sleep(interval);
        }

        match client.ping() {
            Ok(latency) => {
                successful += 1;
                total_time += latency;
                min_time = min_time.min(latency);
                max_time = max_time.max(latency);

                let ms = latency.as_secs_f64() * 1000.0;
                writeln!(
                    writer,
                    "Connected to {}:{}: time={:.2}ms",
                    client.config().host,
                    client.config().port,
                    ms
                )?;
            }
            Err(e) => {
                failed += 1;
                writeln!(
                    writer,
                    "Failed to connect to {}:{}: {}",
                    client.config().host,
                    client.config().port,
                    e
                )?;
            }
        }
    }

    // Print summary for multiple pings
    if count > 1 {
        writeln!(writer)?;
        writeln!(
            writer,
            "--- {} ping statistics ---",
            client.config().host
        )?;
        writeln!(
            writer,
            "{} connections, {} successful, {} failed",
            count, successful, failed
        )?;

        if successful > 0 {
            let avg = total_time.as_secs_f64() * 1000.0 / successful as f64;
            let min_ms = min_time.as_secs_f64() * 1000.0;
            let max_ms = max_time.as_secs_f64() * 1000.0;
            writeln!(
                writer,
                "round-trip min/avg/max = {:.2}/{:.2}/{:.2} ms",
                min_ms, avg, max_ms
            )?;
        }
    }

    if failed > 0 && successful == 0 {
        return Err(TqError::PingFailed(format!(
            "All {} ping attempts failed",
            count
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Integration tests would go here but require actual database connection
    // Unit tests for statistics calculation could be added
}
