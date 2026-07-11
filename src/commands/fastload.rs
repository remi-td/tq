use crate::cli::{FastloadArgs, SourceFormat};
use crate::db::{DatabaseClient, FastloadOptions};
use crate::error::{Result, TqError};
use std::path::Path;
use std::time::Instant;

struct SourceColumn {
    name: String,
    data_type: String,
}

/// Execute the fastload command
pub fn execute(client: &DatabaseClient, args: &FastloadArgs) -> Result<()> {
    let start = Instant::now();

    // 1. Auto-detect format if not explicitly overridden
    let format = args.source_format.unwrap_or_else(|| {
        let ext = args.source_file.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        match ext.as_deref() {
            Some("parquet") => SourceFormat::Parquet,
            Some("json") | Some("ndjson") => SourceFormat::Json,
            _ => SourceFormat::Csv, // Default fallback to CSV
        }
    });

    println!(
        "Loading {} into {} (format: {:?})...",
        args.source_file.display(),
        args.target_table,
        format
    );

    // 2. Check if the target table exists
    let check_query = format!("SELECT 1 FROM {} WHERE 1=0", args.target_table);
    let table_exists = match client.execute(&check_query) {
        Ok(_) => true,
        Err(TqError::TableNotFound { .. }) => false,
        Err(e) => {
            // Any other error (like permission denied or connection error)
            return Err(e);
        }
    };

    // 3. Lazy load auto-table creation if table does not exist
    let source_cols = if !table_exists {
        if args.no_create {
            return Err(TqError::QueryExecution(format!(
                "Target table {} does not exist and --no-create is set",
                args.target_table
            )));
        }

        println!("Target table does not exist. Inspecting source schema for auto-creation...");
        let cols = match format {
            SourceFormat::Csv => get_csv_schema(&args.source_file)?,
            SourceFormat::Parquet => get_parquet_schema(&args.source_file)?,
            SourceFormat::Json => get_json_schema(&args.source_file)?,
        };

        if cols.is_empty() {
            return Err(TqError::QueryExecution(
                "Source file has no columns or headers".to_string()
            ));
        }

        let mut create_fields = Vec::new();
        for col in &cols {
            create_fields.push(format!("\"{}\" {}", col.name, col.data_type));
        }
        let create_sql = format!(
            "CREATE TABLE {} ({}) NO PRIMARY INDEX",
            args.target_table,
            create_fields.join(", ")
        );

        println!("Executing: {}", create_sql);
        client.execute(&create_sql)?;
        println!("Created table {} successfully.", args.target_table);
        cols
    } else {
        // Table exists, query its columns to ensure correct ordering/bindings
        let describe_query = format!("SELECT * FROM {} WHERE 1=0", args.target_table);
        let res = client.execute(&describe_query)?;
        res.columns
            .into_iter()
            .map(|col| SourceColumn {
                name: col.name,
                data_type: "VARCHAR(1000)".to_string(), // type info placeholder
            })
            .collect()
    };

    let columns: Vec<String> = source_cols.iter().map(|c| c.name.clone()).collect();

    // 4. Set up temporary file if format is JSON or Parquet
    let temp_file;
    let effective_csv_path = match format {
        SourceFormat::Csv => &args.source_file,
        SourceFormat::Parquet => {
            println!("Converting Parquet to temporary CSV streamingly...");
            temp_file = tempfile::Builder::new()
                .suffix(".csv")
                .tempfile()
                .map_err(|e| TqError::QueryExecution(format!("Failed to create temp file: {}", e)))?;
            convert_parquet_to_csv(&args.source_file, temp_file.path())?;
            temp_file.path()
        }
        SourceFormat::Json => {
            println!("Converting JSON to temporary CSV streamingly...");
            temp_file = tempfile::Builder::new()
                .suffix(".csv")
                .tempfile()
                .map_err(|e| TqError::QueryExecution(format!("Failed to create temp file: {}", e)))?;
            convert_json_to_csv(&args.source_file, temp_file.path(), &columns)?;
            temp_file.path()
        }
    };

    // 5. Trigger FFI client FastLoad execution
    let error_db = args.error_table_db.as_deref();
    let options = FastloadOptions {
        sessions: args.sessions,
        error_db,
        err1_suffix: &args.error_table_1_suffix,
        err2_suffix: &args.error_table_2_suffix,
    };
    let (rows, warnings, errors) = client.fastload(
        effective_csv_path,
        &args.target_table,
        &columns,
        &options,
    )?;

    // Clean up temporary files (handled automatically by NamedTempFile drop, but explicit drop for clarity)
    // (temp_file will be deleted when it goes out of scope here)

    let duration = start.elapsed();
    println!(
        "Successfully loaded {} rows in {:.2?} ({:.2} rows/sec)",
        rows,
        duration,
        if duration.as_secs_f64() > 0.0 {
            rows as f64 / duration.as_secs_f64()
        } else {
            rows as f64
        }
    );

    if !warnings.is_empty() {
        println!("Warnings encountered during FastLoad:");
        for w in warnings {
            println!("  - {}", w);
        }
    }

    if !errors.is_empty() {
        println!("Non-fatal errors encountered during FastLoad:");
        for e in errors {
            println!("  - {}", e);
        }
    }

    Ok(())
}

fn get_csv_schema(path: &Path) -> Result<Vec<SourceColumn>> {
    let mut rdr = csv::Reader::from_path(path).map_err(|e| {
        TqError::QueryExecution(format!("Failed to read CSV: {}", e))
    })?;
    let headers = rdr.headers().map_err(|e| {
        TqError::QueryExecution(format!("Failed to read CSV headers: {}", e))
    })?;
    let mut cols = Vec::new();
    for h in headers {
        if !h.trim().is_empty() {
            cols.push(SourceColumn {
                name: h.trim().to_string(),
                data_type: "VARCHAR(1000)".to_string(),
            });
        }
    }
    Ok(cols)
}

fn get_json_schema(path: &Path) -> Result<Vec<SourceColumn>> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let file = File::open(path).map_err(|e| {
        TqError::QueryExecution(format!("Failed to open JSON: {}", e))
    })?;
    let reader = BufReader::new(file);

    let mut first_obj = None;

    if let Some(Ok(line)) = reader.lines().next() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(arr) = val.as_array() {
                    if let Some(obj) = arr.first() {
                        if let Some(map) = obj.as_object() {
                            first_obj = Some(map.clone());
                        }
                    }
                }
            }
        } else if trimmed.starts_with('{') {
            if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(trimmed) {
                first_obj = Some(map);
            }
        }
    }

    let map = first_obj.ok_or_else(|| {
        TqError::QueryExecution(
            "Could not detect schema: JSON file is empty or not an array/object".to_string()
        )
    })?;

    let mut cols = Vec::new();
    for (k, v) in map {
        let t = match v {
            serde_json::Value::Bool(_) => "VARCHAR(5)".to_string(),
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    "BIGINT".to_string()
                } else {
                    "DOUBLE PRECISION".to_string()
                }
            }
            _ => "VARCHAR(1000)".to_string(),
        };
        cols.push(SourceColumn {
            name: k,
            data_type: t,
        });
    }
    Ok(cols)
}

fn get_parquet_schema(path: &Path) -> Result<Vec<SourceColumn>> {
    use parquet::file::reader::{FileReader, SerializedFileReader};
    use std::fs::File;

    let file = File::open(path).map_err(|e| {
        TqError::QueryExecution(format!("Failed to open Parquet: {}", e))
    })?;
    let reader = SerializedFileReader::new(file).map_err(|e| {
        TqError::QueryExecution(format!("Failed to parse Parquet: {}", e))
    })?;

    let metadata = reader.metadata();
    let schema_desc = metadata.file_metadata().schema_descr();

    let mut cols = Vec::new();
    for i in 0..schema_desc.num_columns() {
        let name = schema_desc.column(i).path().string();
        let physical_type = schema_desc.column(i).physical_type();

        let data_type = match physical_type {
            parquet::basic::Type::BOOLEAN => "VARCHAR(5)".to_string(),
            parquet::basic::Type::INT32 | parquet::basic::Type::INT64 => "BIGINT".to_string(),
            parquet::basic::Type::FLOAT | parquet::basic::Type::DOUBLE => "DOUBLE PRECISION".to_string(),
            _ => "VARCHAR(1000)".to_string(),
        };

        cols.push(SourceColumn {
            name,
            data_type,
        });
    }
    Ok(cols)
}

fn convert_parquet_to_csv(parquet_path: &Path, csv_path: &Path) -> Result<()> {
    use arrow_csv::WriterBuilder;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use std::fs::File;

    let file = File::open(parquet_path).map_err(|e| {
        TqError::QueryExecution(format!("Failed to open Parquet for conversion: {}", e))
    })?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(|e| {
        TqError::QueryExecution(format!("Parquet builder error: {}", e))
    })?;

    let reader = builder.build().map_err(|e| {
        TqError::QueryExecution(format!("Parquet reader error: {}", e))
    })?;

    let out_file = File::create(csv_path).map_err(|e| {
        TqError::QueryExecution(format!("Failed to create temporary CSV: {}", e))
    })?;
    let mut writer = WriterBuilder::new().with_header(true).build(out_file);

    for maybe_batch in reader {
        let batch = maybe_batch.map_err(|e| {
            TqError::QueryExecution(format!("Parquet batch read error: {}", e))
        })?;
        writer.write(&batch).map_err(|e| {
            TqError::QueryExecution(format!("CSV write error: {}", e))
        })?;
    }
    Ok(())
}

fn convert_json_to_csv(json_path: &Path, csv_path: &Path, columns: &[String]) -> Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(json_path).map_err(|e| {
        TqError::QueryExecution(format!("Failed to open JSON for conversion: {}", e))
    })?;
    let reader = BufReader::new(file);

    let out_file = File::create(csv_path).map_err(|e| {
        TqError::QueryExecution(format!("Failed to create temporary CSV: {}", e))
    })?;
    let mut writer = csv::Writer::from_writer(out_file);

    // Write headers
    writer.write_record(columns).map_err(|e| {
        TqError::QueryExecution(format!("Failed to write CSV headers: {}", e))
    })?;

    let mut first_char = ' ';
    if let Ok(mut temp_rdr) = File::open(json_path) {
        use std::io::Read;
        let mut buf = [0; 10];
        if let Ok(n) = temp_rdr.read(&mut buf) {
            if n > 0 {
                if let Some(c) = String::from_utf8_lossy(&buf[..n]).trim().chars().next() {
                    first_char = c;
                }
            }
        }
    }

    if first_char == '[' {
        let file = File::open(json_path).map_err(|e| {
            TqError::QueryExecution(format!("Failed to open JSON: {}", e))
        })?;
        let val: serde_json::Value = serde_json::from_reader(file).map_err(|e| {
            TqError::QueryExecution(format!("Failed to parse JSON: {}", e))
        })?;
        if let Some(arr) = val.as_array() {
            for item in arr {
                if let Some(obj) = item.as_object() {
                    let mut row = Vec::new();
                    for col in columns {
                        let val_str = match obj.get(col) {
                            Some(serde_json::Value::Null) => "".to_string(),
                            Some(serde_json::Value::String(s)) => s.clone(),
                            Some(other) => other.to_string(),
                            None => "".to_string(),
                        };
                        row.push(val_str);
                    }
                    writer.write_record(&row).map_err(|e| {
                        TqError::QueryExecution(format!("Failed to write CSV row: {}", e))
                    })?;
                }
            }
        }
    } else {
        for line in reader.lines() {
            let line = line.map_err(|e| {
                TqError::QueryExecution(format!("Failed to read JSON line: {}", e))
            })?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(serde_json::Value::Object(obj)) = serde_json::from_str::<serde_json::Value>(trimmed) {
                let mut row = Vec::new();
                for col in columns {
                    let val_str = match obj.get(col) {
                        Some(serde_json::Value::Null) => "".to_string(),
                        Some(serde_json::Value::String(s)) => s.clone(),
                        Some(other) => other.to_string(),
                        None => "".to_string(),
                    };
                    row.push(val_str);
                }
                writer.write_record(&row).map_err(|e| {
                    TqError::QueryExecution(format!("Failed to write CSV row: {}", e))
                })?;
            }
        }
    }

    writer.flush().map_err(|e| {
        TqError::QueryExecution(format!("Failed to flush CSV writer: {}", e))
    })?;
    Ok(())
}
