use crate::cli::FastexportArgs;
use crate::db::DatabaseClient;
use crate::error::Result;
use std::time::Instant;

/// Execute the fastexport command
pub fn execute(client: &DatabaseClient, args: &FastexportArgs) -> Result<()> {
    let start = Instant::now();
    println!(
        "Exporting {} to {} in parallel...",
        args.source_table,
        args.target_file.display()
    );

    let count = client.fastexport(&args.source_table, &args.target_file, args.sessions)?;

    let duration = start.elapsed();
    println!(
        "Successfully exported {} rows in {:.2?} ({:.2} rows/sec)",
        count,
        duration,
        if duration.as_secs_f64() > 0.0 {
            count as f64 / duration.as_secs_f64()
        } else {
            count as f64
        }
    );

    Ok(())
}
