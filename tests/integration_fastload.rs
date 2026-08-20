//! Integration tests for FastLoad and FastExport against live database
//!
//! Run with: cargo test --test integration_fastload -- --ignored

mod common;

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tq::cli::{FastloadArgs, FastexportArgs, LogonMechanism};
use tq::db::{ConnectionConfig, DatabaseClient};

#[test]
#[ignore] // Requires live database connection
fn test_live_fastload_and_fastexport_csv() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let config = ConnectionConfig::from_connection_string(
            &logon,
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();
        let client = DatabaseClient::new(config, None).unwrap();

        let table_name = "demo_user.tq_fastload_test_csv";

        // Clean up table first
        let _ = client.execute(&format!("DROP TABLE {}", table_name));

        // 1. Create a dummy CSV file
        let temp_csv = tempfile::Builder::new()
            .suffix(".csv")
            .tempfile()
            .unwrap();
        
        writeln!(temp_csv.as_file(), "id,name").unwrap();
        writeln!(temp_csv.as_file(), "1,Alice").unwrap();
        writeln!(temp_csv.as_file(), "2,Bob").unwrap();
        writeln!(temp_csv.as_file(), "3,Charlie").unwrap();

        // 2. Perform FastLoad (triggers auto-creation of table)
        let args = FastloadArgs {
            source_file: temp_csv.path().to_path_buf(),
            target_table: table_name.to_string(),
            source_format: None,
            delimiter: None,
            no_create: false,
            sessions: None,
            error_table_db: None,
            error_table_1_suffix: "_FL_ERR_1".to_string(),
            error_table_2_suffix: "_FL_ERR_2".to_string(),
            json: false,
        };

        tq::commands::fastload::execute(&client, &args).unwrap();

        // 3. Query the table to verify records
        let res = client.execute(&format!("SELECT * FROM {} ORDER BY id", table_name)).unwrap();
        assert_eq!(res.rows.len(), 3);
        assert_eq!(res.rows[0][0].display(), "1");
        assert_eq!(res.rows[0][1].display(), "Alice");
        assert_eq!(res.rows[1][0].display(), "2");
        assert_eq!(res.rows[1][1].display(), "Bob");

        // 4. Perform FastExport
        let temp_export = tempfile::Builder::new()
            .suffix(".csv")
            .tempfile()
            .unwrap();

        let export_args = FastexportArgs {
            source_table: table_name.to_string(),
            target_file: temp_export.path().to_path_buf(),
            sessions: None,
            json: false,
        };

        tq::commands::fastexport::execute(&client, &export_args).unwrap();

        // Verify exported file exists and has rows
        let content = std::fs::read_to_string(temp_export.path()).unwrap();
        assert!(content.contains("id,name"));
        assert!(content.contains("1,Alice"));
        assert!(content.contains("2,Bob"));
        assert!(content.contains("3,Charlie"));

        // Clean up
        let _ = client.execute(&format!("DROP TABLE {}", table_name));
        let _ = client.execute(&format!("DROP TABLE {}_FL_ERR_1", table_name));
        let _ = client.execute(&format!("DROP TABLE {}_FL_ERR_2", table_name));
    });
}

#[test]
#[ignore] // Requires live database connection
fn test_live_fastload_parquet() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let config = ConnectionConfig::from_connection_string(
            &logon,
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();
        let client = DatabaseClient::new(config, None).unwrap();

        let table_name = "demo_user.tq_fastload_test_parquet";

        // Clean up table first
        let _ = client.execute(&format!("DROP TABLE {}", table_name));

        // 1. Create a dummy Parquet file
        let temp_parquet = tempfile::Builder::new()
            .suffix(".parquet")
            .tempfile()
            .unwrap();
        
        write_test_parquet(temp_parquet.path()).unwrap();

        // 2. Perform FastLoad (triggers auto-creation of table with mapped types)
        let args = FastloadArgs {
            source_file: temp_parquet.path().to_path_buf(),
            target_table: table_name.to_string(),
            source_format: None,
            delimiter: None,
            no_create: false,
            sessions: None,
            error_table_db: None,
            error_table_1_suffix: "_FL_ERR_1".to_string(),
            error_table_2_suffix: "_FL_ERR_2".to_string(),
            json: false,
        };

        tq::commands::fastload::execute(&client, &args).unwrap();

        // 3. Query the table to verify records and types
        let res = client.execute(&format!("SELECT * FROM {} ORDER BY id", table_name)).unwrap();
        assert_eq!(res.rows.len(), 3);
        assert_eq!(res.rows[0][0].display(), "1");
        assert_eq!(res.rows[0][1].display(), "Alice");

        // Clean up
        let _ = client.execute(&format!("DROP TABLE {}", table_name));
        let _ = client.execute(&format!("DROP TABLE {}_FL_ERR_1", table_name));
        let _ = client.execute(&format!("DROP TABLE {}_FL_ERR_2", table_name));
    });
}

#[test]
#[ignore] // Requires live database connection
fn test_live_fastload_json() {
    common::with_driver(|| {
        dotenvy::dotenv().ok();
        let logon = std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let config = ConnectionConfig::from_connection_string(
            &logon,
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();
        let client = DatabaseClient::new(config, None).unwrap();

        let table_name = "demo_user.tq_fastload_test_json";

        // Clean up table first
        let _ = client.execute(&format!("DROP TABLE {}", table_name));

        // 1. Create a dummy NDJSON file
        let temp_json = tempfile::Builder::new()
            .suffix(".json")
            .tempfile()
            .unwrap();
        
        writeln!(temp_json.as_file(), r#"{{"id":1,"name":"Alice"}}"#).unwrap();
        writeln!(temp_json.as_file(), r#"{{"id":2,"name":"Bob"}}"#).unwrap();
        writeln!(temp_json.as_file(), r#"{{"id":3,"name":"Charlie"}}"#).unwrap();

        // 2. Perform FastLoad (triggers auto-creation of table)
        let args = FastloadArgs {
            source_file: temp_json.path().to_path_buf(),
            target_table: table_name.to_string(),
            source_format: None,
            delimiter: None,
            no_create: false,
            sessions: None,
            error_table_db: None,
            error_table_1_suffix: "_FL_ERR_1".to_string(),
            error_table_2_suffix: "_FL_ERR_2".to_string(),
            json: false,
        };

        tq::commands::fastload::execute(&client, &args).unwrap();

        // 3. Query the table to verify records
        let res = client.execute(&format!("SELECT * FROM {} ORDER BY id", table_name)).unwrap();
        assert_eq!(res.rows.len(), 3);
        assert_eq!(res.rows[0][0].display(), "1");
        assert_eq!(res.rows[0][1].display(), "Alice");

        // Clean up
        let _ = client.execute(&format!("DROP TABLE {}", table_name));
        let _ = client.execute(&format!("DROP TABLE {}_FL_ERR_1", table_name));
        let _ = client.execute(&format!("DROP TABLE {}_FL_ERR_2", table_name));
    });
}

fn write_test_parquet(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use arrow::array::{Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use parquet::arrow::arrow_writer::ArrowWriter;
    use std::sync::Arc;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]));

    let ids = Int64Array::from(vec![1, 2, 3]);
    let names = StringArray::from(vec!["Alice", "Bob", "Charlie"]);

    let batch = RecordBatch::try_new(schema.clone(), vec![Arc::new(ids), Arc::new(names)])?;

    let file = File::create(path)?;
    let mut writer = ArrowWriter::try_new(file, schema, None)?;

    writer.write(&batch)?;
    writer.close()?;

    Ok(())
}

#[test]
fn test_fastload_delimiter_validation_with_parquet() {
    common::with_driver(|| {
        let config = ConnectionConfig::from_connection_string(
            "dummy_user:dummy_pass@dummy_host:1025/dummy_db",
            LogonMechanism::Td2,
            Duration::from_secs(3),
            None,
        )
        .unwrap();

        // If driver loading is not supported/installed on the build machine, skip the test
        let client = match DatabaseClient::new(config, None) {
            Ok(c) => c,
            Err(_) => return, 
        };

        let args = FastloadArgs {
            source_file: std::path::PathBuf::from("data.parquet"),
            target_table: "my_table".to_string(),
            source_format: Some(tq::cli::SourceFormat::Parquet),
            delimiter: Some("|".to_string()),
            no_create: false,
            sessions: None,
            error_table_db: None,
            error_table_1_suffix: "_FL_ERR_1".to_string(),
            error_table_2_suffix: "_FL_ERR_2".to_string(),
            json: false,
        };

        let result = tq::commands::fastload::execute(&client, &args);
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("The --delimiter option can only be used with CSV/TSV source files"));
    });
}

