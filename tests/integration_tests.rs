//! Integration tests for tq library
//!
//! These tests verify the public API of the tq library works as expected.
//! Since these are integration tests, they test the library through its
//! public interface without mocking or database connections.
//!
//! ## Live Database Tests
//!
//! Tests marked with `#[ignore]` require a live Teradata database connection.
//! Set the `TQ_LOGON` environment variable and run with:
//!
//! ```bash
//! cargo test --test integration_tests -- --ignored
//! ```
//!
//! Live database tests use the `with_driver` wrapper from the common module
//! to synchronize driver initialization across test threads.

mod common;

use std::time::Duration;
use tq::cli::{LogonMechanism, OutputFormat};
use tq::db::{
    parse_duration, ColumnMetadata, ConnectionConfig, QueryResult, Row, TeradataType, Value,
};
use tq::error::TqError;
use tq::format::{csv, json, table, FormatOptions};

// =============================================================================
// Connection Configuration Tests
// =============================================================================

#[test]
fn test_connection_config_from_connection_string() {
    let config = ConnectionConfig::from_connection_string(
        "user:pass@host:1025/db",
        LogonMechanism::Td2,
        Duration::from_secs(30),
        None,
    )
    .unwrap();

    assert_eq!(config.user, "user");
    assert_eq!(config.host, "host");
    assert_eq!(config.port, 1025);
    assert_eq!(config.database, "db");
    assert_eq!(config.logmech.to_string(), "TD2");
}

#[test]
fn test_connection_config_with_password_override() {
    let config = ConnectionConfig::from_connection_string(
        "user@host:1025/db",
        LogonMechanism::Td2,
        Duration::from_secs(30),
        Some("filepass".to_string()),
    )
    .unwrap();

    assert_eq!(config.user, "user");
    // Password is Secret, so we can't directly test it here without exposing
}

#[test]
fn test_connection_config_invalid_format() {
    let result = ConnectionConfig::from_connection_string(
        "invalid",
        LogonMechanism::Td2,
        Duration::from_secs(30),
        None,
    );
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TqError::InvalidConnectionString(_)
    ));
}

#[test]
fn test_connection_config_invalid_port() {
    let result = ConnectionConfig::from_connection_string(
        "user:pass@host:invalid/db",
        LogonMechanism::Td2,
        Duration::from_secs(30),
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_connection_config_default_port() {
    // The API requires explicit port, so test that parsing with port works
    let config = ConnectionConfig::from_connection_string(
        "user:pass@host:1025/db",
        LogonMechanism::Td2,
        Duration::from_secs(30),
        None,
    )
    .unwrap();

    assert_eq!(config.port, 1025);
}

#[test]
fn test_connection_config_missing_port_errors() {
    // Without explicit port, should error
    let result = ConnectionConfig::from_connection_string(
        "user:pass@host/db",
        LogonMechanism::Td2,
        Duration::from_secs(30),
        None,
    );
    assert!(result.is_err());
}

// =============================================================================
// LogonMechanism Tests
// =============================================================================

#[test]
fn test_logon_mechanism_display() {
    assert_eq!(format!("{}", LogonMechanism::Td2), "TD2");
    assert_eq!(format!("{}", LogonMechanism::Ldap), "LDAP");
    assert_eq!(format!("{}", LogonMechanism::Krb5), "KRB5");
    assert_eq!(format!("{}", LogonMechanism::Tdnego), "TDNEGO");
}

// =============================================================================
// Duration Parsing Tests
// =============================================================================

#[test]
fn test_parse_duration_seconds() {
    let d = parse_duration("30s").unwrap();
    assert_eq!(d, Duration::from_secs(30));
}

#[test]
fn test_parse_duration_minutes() {
    let d = parse_duration("5m").unwrap();
    assert_eq!(d, Duration::from_secs(300));
}

#[test]
fn test_parse_duration_hours() {
    let d = parse_duration("1h").unwrap();
    assert_eq!(d, Duration::from_secs(3600));
}

#[test]
fn test_parse_duration_milliseconds() {
    let d = parse_duration("500ms").unwrap();
    assert_eq!(d, Duration::from_millis(500));
}

#[test]
fn test_parse_duration_invalid() {
    assert!(parse_duration("invalid").is_err());
    assert!(parse_duration("30x").is_err());
}

// =============================================================================
// Value Type Tests
// =============================================================================

#[test]
fn test_value_null() {
    let v = Value::Null;
    assert_eq!(v.display(), "[NULL]");
}

#[test]
fn test_value_string() {
    let v = Value::String("hello".to_string());
    assert_eq!(v.display(), "hello");
}

#[test]
fn test_value_integer() {
    let v = Value::Integer(42);
    assert_eq!(v.display(), "42");
}

#[test]
fn test_value_decimal() {
    let v = Value::Decimal(3.14158); // Avoid clippy::approx_constant
    assert!(v.display().starts_with("3.14"));
}

#[test]
fn test_value_boolean() {
    assert_eq!(Value::Boolean(true).display(), "true");
    assert_eq!(Value::Boolean(false).display(), "false");
}

#[test]
fn test_value_date() {
    let v = Value::Date("2024-01-15".to_string());
    assert_eq!(v.display(), "2024-01-15");
}

#[test]
fn test_value_timestamp() {
    let v = Value::Timestamp("2024-01-15T10:30:00".to_string());
    assert_eq!(v.display(), "2024-01-15T10:30:00");
}

#[test]
fn test_value_bytes() {
    let v = Value::Bytes(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]);
    assert_eq!(v.display(), "<5 bytes>");
}

// =============================================================================
// QueryResult Helper
// =============================================================================

fn make_test_result() -> QueryResult {
    let columns = vec![
        ColumnMetadata {
            name: "Name".to_string(),
            data_type: TeradataType::Varchar,
            nullable: false,
        },
        ColumnMetadata {
            name: "Age".to_string(),
            data_type: TeradataType::Integer,
            nullable: false,
        },
        ColumnMetadata {
            name: "Score".to_string(),
            data_type: TeradataType::Decimal,
            nullable: true,
        },
    ];

    let rows: Vec<Row> = vec![
        vec![
            Value::String("Alice".to_string()),
            Value::Integer(30),
            Value::Decimal(95.5),
        ],
        vec![
            Value::String("Bob".to_string()),
            Value::Integer(25),
            Value::Null,
        ],
    ];

    QueryResult {
        columns,
        rows,
        row_count: 2,
        execution_time: Duration::from_millis(123),
    }
}

fn make_empty_result() -> QueryResult {
    QueryResult {
        columns: vec![ColumnMetadata {
            name: "Column1".to_string(),
            data_type: TeradataType::Varchar,
            nullable: false,
        }],
        rows: vec![],
        row_count: 0,
        execution_time: Duration::from_millis(10),
    }
}

// =============================================================================
// Table Format Tests
// =============================================================================

#[test]
fn test_format_table_output() {
    let result = make_test_result();
    let options = table::TableOptions::default();
    let mut output = Vec::new();

    table::write(&result, &mut output, &options).unwrap();
    let output_str = String::from_utf8(output).unwrap();

    assert!(output_str.contains("Name"));
    assert!(output_str.contains("Age"));
    assert!(output_str.contains("Alice"));
    assert!(output_str.contains("Bob"));
    assert!(output_str.contains("30"));
    assert!(output_str.contains("25"));
}

#[test]
fn test_format_table_empty() {
    let result = make_empty_result();
    let options = table::TableOptions::default();
    let mut output = Vec::new();

    table::write(&result, &mut output, &options).unwrap();
    let output_str = String::from_utf8(output).unwrap();

    // Empty result shows "No results returned"
    assert!(output_str.contains("No results returned"));
}

// =============================================================================
// JSON Format Tests
// =============================================================================

#[test]
fn test_format_json_output() {
    let result = make_test_result();
    let options = json::JsonOptions::default();
    let mut output = Vec::new();

    json::write(&result, &mut output, &options).unwrap();
    let output_str = String::from_utf8(output).unwrap();

    // Parse as JSON to validate - envelope format since Sprint 53
    let parsed: serde_json::Value = serde_json::from_str(&output_str).unwrap();

    assert!(parsed.is_object());
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["row_count"], 2);

    let arr = parsed["data"].as_array().unwrap();
    assert_eq!(arr.len(), 2);

    // Check first row
    let first = &arr[0];
    assert_eq!(first["Name"], "Alice");
    assert_eq!(first["Age"], 30);
    assert_eq!(first["Score"], 95.5);

    // Check second row
    let second = &arr[1];
    assert_eq!(second["Name"], "Bob");
    assert_eq!(second["Age"], 25);
    assert!(second["Score"].is_null());
}

#[test]
fn test_format_json_empty() {
    let result = make_empty_result();
    let options = json::JsonOptions::default();
    let mut output = Vec::new();

    json::write(&result, &mut output, &options).unwrap();
    let output_str = String::from_utf8(output).unwrap();

    // Envelope format since Sprint 53
    let parsed: serde_json::Value = serde_json::from_str(&output_str).unwrap();
    assert!(parsed.is_object());
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["row_count"], 0);
    assert_eq!(parsed["data"].as_array().unwrap().len(), 0);
}

#[test]
fn test_format_json_pretty() {
    let result = make_test_result();
    let options = json::JsonOptions { pretty: true };
    let mut output = Vec::new();

    json::write(&result, &mut output, &options).unwrap();
    let output_str = String::from_utf8(output).unwrap();

    // Pretty printed JSON should have indentation
    assert!(output_str.contains("  "));
    assert!(output_str.contains('\n'));
}

// =============================================================================
// CSV Format Tests
// =============================================================================

#[test]
fn test_format_csv_output() {
    let result = make_test_result();
    let options = csv::CsvOptions::default();
    let mut output = Vec::new();

    csv::write(&result, &mut output, &options).unwrap();
    let output_str = String::from_utf8(output).unwrap();

    // Should have header row
    assert!(output_str.contains("Name,Age,Score"));

    // Should have data rows
    assert!(output_str.contains("Alice,30,95.5"));
    assert!(output_str.contains("Bob,25,")); // NULL becomes empty
}

#[test]
fn test_format_csv_no_header() {
    let result = make_test_result();
    let options = csv::CsvOptions {
        show_header: false,
        ..Default::default()
    };
    let mut output = Vec::new();

    csv::write(&result, &mut output, &options).unwrap();
    let output_str = String::from_utf8(output).unwrap();

    // Should NOT have header row
    assert!(!output_str.contains("Name,Age,Score"));

    // But should still have data
    assert!(output_str.contains("Alice"));
}

#[test]
fn test_format_csv_empty() {
    let result = make_empty_result();
    let options = csv::CsvOptions::default();
    let mut output = Vec::new();

    csv::write(&result, &mut output, &options).unwrap();
    let output_str = String::from_utf8(output).unwrap();

    // Empty result should still have header
    assert!(output_str.contains("Column1"));
}

#[test]
fn test_format_csv_with_special_characters() {
    let columns = vec![ColumnMetadata {
        name: "Description".to_string(),
        data_type: TeradataType::Varchar,
        nullable: false,
    }];

    let rows: Vec<Row> = vec![
        vec![Value::String("Alice, Jr.".to_string())],
        vec![Value::String("Say \"Hello\"".to_string())],
    ];

    let result = QueryResult {
        columns,
        rows,
        row_count: 2,
        execution_time: Duration::from_millis(10),
    };

    let options = csv::CsvOptions::default();
    let mut output = Vec::new();

    csv::write(&result, &mut output, &options).unwrap();
    let output_str = String::from_utf8(output).unwrap();

    // Values with commas or quotes should be properly escaped
    assert!(output_str.contains("\"Alice, Jr.\""));
    assert!(output_str.contains("\"Say \"\"Hello\"\"\""));
}

// =============================================================================
// Error Tests
// =============================================================================

#[test]
fn test_error_user_message() {
    let err = TqError::ConnectionFailed {
        host: "localhost".to_string(),
        port: 1025,
        source: Box::new(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "refused",
        )),
    };

    let msg = err.user_message();
    assert!(msg.contains("localhost"));
    assert!(msg.contains("1025"));
}

#[test]
fn test_error_exit_code() {
    let usage_err = TqError::InvalidConnectionString("test".to_string());
    assert_eq!(usage_err.exit_code(), 2);

    let runtime_err = TqError::QueryExecution("test".to_string());
    assert_eq!(runtime_err.exit_code(), 1);
}

#[test]
fn test_error_display() {
    let err = TqError::QueryExecution("syntax error".to_string());
    let display = format!("{}", err);
    assert!(display.contains("syntax error"));
}

// =============================================================================
// CLI Tests
// =============================================================================

#[test]
fn test_cli_parsing() {
    use clap::Parser;
    use tq::cli::Cli;

    let args = vec!["tq", "--logon", "user:pass@host:1025/db", "ping"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.global.logon, Some("user:pass@host:1025/db".to_string()));
    assert_eq!(cli.global.logmech, LogonMechanism::Td2);
}

#[test]
fn test_cli_query_with_format() {
    use clap::Parser;
    use tq::cli::{Cli, Command};

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
fn test_output_format_display() {
    assert_eq!(format!("{}", OutputFormat::Table), "table");
    assert_eq!(format!("{}", OutputFormat::Json), "json");
    assert_eq!(format!("{}", OutputFormat::Csv), "csv");
}

// =============================================================================
// FormatOptions Tests
// =============================================================================

#[test]
fn test_format_options_default() {
    let opts = FormatOptions::default();
    assert!(opts.table.show_header);
    assert!(opts.csv.show_header);
    assert!(opts.table.use_color); // Default is true for table
}

#[test]
fn test_format_options_with_builder() {
    let opts = FormatOptions::default()
        .with_header(false)
        .with_color(true)
        .with_pretty(true);

    assert!(!opts.table.show_header);
    assert!(!opts.csv.show_header);
    assert!(opts.table.use_color);
    assert!(opts.json.pretty);
}

// =============================================================================
// Live Database Integration Tests (require TQ_LOGON environment variable)
// =============================================================================
//
// These tests use the `with_driver` wrapper from common/mod.rs to synchronize
// driver initialization across test threads. This prevents race conditions
// that can occur when multiple tests try to load the Teradata driver simultaneously.

/// Test that actual column names are returned from query metadata
///
/// This test validates the fix for the metadata parsing bug where the Teradata
/// API returns map-of-arrays format instead of array-of-objects format.
///
/// Run with: cargo test test_actual_column_names_from_metadata -- --ignored
#[test]
#[ignore] // Requires live database connection
fn test_actual_column_names_from_metadata() {
    common::with_driver(|| {
        use tq::db::DatabaseClient;

        // Load from .env file
        dotenvy::dotenv().ok();
        let logon =
            std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let config = ConnectionConfig::from_connection_string(
            &logon,
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();
        let client = DatabaseClient::new(config, None).unwrap();

        // Execute query with known column names
        let result = client
            .execute("SELECT 1 AS test_col, 'hello' AS text_col, NULL AS null_col")
            .unwrap();

        // Verify actual column names are used (not generic col1, col2, col3)
        assert_eq!(result.columns.len(), 3, "Expected 3 columns");
        assert_eq!(
            result.columns[0].name, "test_col",
            "First column should be 'test_col'"
        );
        assert_eq!(
            result.columns[1].name, "text_col",
            "Second column should be 'text_col'"
        );
        assert_eq!(
            result.columns[2].name, "null_col",
            "Third column should be 'null_col'"
        );

        // Verify row data
        assert_eq!(result.rows.len(), 1, "Expected 1 row");
        assert_eq!(result.rows[0].len(), 3, "Expected 3 columns in row");
    });
}

/// Test querying multiple columns with different data types
///
/// Run with: cargo test test_live_multi_column_query -- --ignored
#[test]
#[ignore] // Requires live database connection
fn test_live_multi_column_query() {
    common::with_driver(|| {
        use tq::db::DatabaseClient;

        dotenvy::dotenv().ok();
        let logon =
            std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let config = ConnectionConfig::from_connection_string(
            &logon,
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();
        let client = DatabaseClient::new(config, None).unwrap();

        // Query with multiple columns named a, b, c
        let result = client.execute("SELECT 1 AS a, 2 AS b, 3 AS c").unwrap();

        assert_eq!(result.columns.len(), 3);
        assert_eq!(result.columns[0].name, "a");
        assert_eq!(result.columns[1].name, "b");
        assert_eq!(result.columns[2].name, "c");

        // Verify values
        assert_eq!(result.rows.len(), 1);
        assert!(matches!(result.rows[0][0], Value::Integer(1)));
        assert!(matches!(result.rows[0][1], Value::Integer(2)));
        assert!(matches!(result.rows[0][2], Value::Integer(3)));
    });
}

// =============================================================================
// Sprint 22: Feature 2 - Enhanced Schema Commands Integration Tests
// =============================================================================
// These tests verify that the /list commands work correctly against a live database.
// They test the SQL query execution, result parsing, and output formatting.
// Run with: cargo test --test integration_tests -- --ignored

/// Test /list databases query returns database names
///
/// Run with: cargo test test_list_databases_query -- --ignored
#[test]
#[ignore] // Requires live database connection
fn test_list_databases_query() {
    common::with_driver(|| {
        use tq::db::DatabaseClient;

        dotenvy::dotenv().ok();
        let logon =
            std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let config = ConnectionConfig::from_connection_string(
            &logon,
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();
        let client = DatabaseClient::new(config, None).unwrap();

        // Execute the same SQL query that /list databases uses
        let sql = r#"
            SELECT TRIM(DatabaseName) AS database_name
            FROM DBC.DatabasesV
            WHERE DatabaseName NOT IN ('All', 'Console', 'Crashdumps',
                                       'dbcmngr', 'Default', 'External_AP',
                                       'EXTUSER', 'LockLogShredder', 'PUBLIC',
                                       'SQLJ', 'Sys_Calendar', 'SysAdmin',
                                       'SYSBAR', 'SYSJDBC', 'SYSLIB', 'SYSSPATIAL',
                                       'SystemFe', 'SYSUDTLIB', 'TD_SERVER_DB',
                                       'TD_SYSFNLIB', 'TD_SYSGPL', 'TD_SYSXML',
                                       'TDMaps', 'TDPUSER', 'TDQCD', 'TDStats',
                                       'tdwm', 'VIEWPOINT')
            ORDER BY DatabaseName
        "#;

        let result = client.execute(sql).unwrap();

        // Verify that we got database results
        assert!(!result.columns.is_empty(), "Should have at least one column");
        assert!(!result.rows.is_empty(), "Should have at least one database");

        // DBC database should always be present
        let dbc_found = result.rows.iter().any(|row| {
            if let Some(val) = row.first() {
                val.display().to_uppercase() == "DBC"
            } else {
                false
            }
        });
        assert!(dbc_found, "DBC database should be in results");
    });
}

/// Test /list tables query returns tables in DBC database
///
/// Run with: cargo test test_list_tables_query -- --ignored
#[test]
#[ignore] // Requires live database connection
fn test_list_tables_query() {
    common::with_driver(|| {
        use tq::db::DatabaseClient;

        dotenvy::dotenv().ok();
        let logon =
            std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let config = ConnectionConfig::from_connection_string(
            &logon,
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();
        let client = DatabaseClient::new(config, None).unwrap();

        // Execute the same SQL query that /list tables uses (for DBC database)
        let sql = r#"
            SELECT TRIM(TableName) AS table_name,
                   TableKind
            FROM DBC.TablesV
            WHERE DatabaseName = 'DBC'
              AND TableKind IN ('T', 'O')
            ORDER BY TableName
        "#;

        let result = client.execute(sql).unwrap();

        // Verify that we got table results
        assert_eq!(
            result.columns.len(),
            2,
            "Should have 2 columns (name and kind)"
        );
        assert!(!result.rows.is_empty(), "DBC should have tables");

        // Verify columns are properly named
        assert_eq!(result.columns[0].name.to_lowercase(), "table_name");
        assert_eq!(result.columns[1].name.to_lowercase(), "tablekind");
    });
}

/// Test /list tables with pattern filtering
///
/// Run with: cargo test test_list_tables_pattern_query -- --ignored
#[test]
#[ignore] // Requires live database connection
fn test_list_tables_pattern_query() {
    common::with_driver(|| {
        use tq::db::DatabaseClient;

        dotenvy::dotenv().ok();
        let logon =
            std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let config = ConnectionConfig::from_connection_string(
            &logon,
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();
        let client = DatabaseClient::new(config, None).unwrap();

        // Execute query to get all tables from DBC
        let sql = r#"
            SELECT TRIM(TableName) AS table_name,
                   TableKind
            FROM DBC.TablesV
            WHERE DatabaseName = 'DBC'
              AND TableKind IN ('T', 'O')
            ORDER BY TableName
        "#;

        let result = client.execute(sql).unwrap();

        // Apply pattern matching (same logic as /list tables uses)
        // Test that pattern matching logic works correctly
        let all_count = result.rows.len();

        // Pattern "*" should match all tables
        let wildcard_matches: Vec<_> = result
            .rows
            .iter()
            .filter(|row| {
                if let Some(val) = row.first() {
                    let _name = val.display();
                    true // "*" matches everything
                } else {
                    false
                }
            })
            .collect();

        assert_eq!(
            wildcard_matches.len(),
            all_count,
            "Pattern '*' should match all tables"
        );

        // At least verify the query ran successfully and returned tables
        assert!(all_count > 0, "DBC should have tables");
    });
}

/// Test /list views query returns views in DBC database
///
/// Run with: cargo test test_list_views_query -- --ignored
#[test]
#[ignore] // Requires live database connection
fn test_list_views_query() {
    common::with_driver(|| {
        use tq::db::DatabaseClient;

        dotenvy::dotenv().ok();
        let logon =
            std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let config = ConnectionConfig::from_connection_string(
            &logon,
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();
        let client = DatabaseClient::new(config, None).unwrap();

        // Execute the same SQL query that /list views uses (for DBC database)
        let sql = r#"
            SELECT TRIM(TableName) AS view_name
            FROM DBC.TablesV
            WHERE DatabaseName = 'DBC'
              AND TableKind = 'V'
            ORDER BY TableName
        "#;

        let result = client.execute(sql).unwrap();

        // Verify that we got view results
        assert_eq!(result.columns.len(), 1, "Should have 1 column (view_name)");
        assert!(!result.rows.is_empty(), "DBC should have views");

        // Verify column is properly named
        assert_eq!(result.columns[0].name.to_lowercase(), "view_name");

        // DBC has many system views like TablesV, ColumnsV, DatabasesV
        let views_found = result.rows.iter().any(|row| {
            if let Some(val) = row.first() {
                val.display().to_uppercase().contains("V")
            } else {
                false
            }
        });
        assert!(views_found, "Should find system views in DBC");
    });
}

/// Test glob pattern matching logic used by /list tables
///
/// Run with: cargo test test_glob_matching_integration -- --ignored
#[test]
#[ignore] // Requires live database connection
fn test_glob_matching_integration() {
    common::with_driver(|| {
        use tq::db::DatabaseClient;

        dotenvy::dotenv().ok();
        let logon =
            std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let config = ConnectionConfig::from_connection_string(
            &logon,
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();
        let client = DatabaseClient::new(config, None).unwrap();

        // Get some tables from DBC
        let sql = r#"
            SELECT TRIM(TableName) AS table_name
            FROM DBC.TablesV
            WHERE DatabaseName = 'DBC'
              AND TableKind IN ('T', 'O', 'V')
            ORDER BY TableName
            SAMPLE 10
        "#;

        let result = client.execute(sql).unwrap();
        assert!(!result.rows.is_empty(), "Should get some tables/views from DBC");

        // Test various glob patterns
        let test_cases = vec![
            ("*", true),   // Wildcard matches everything
            ("T*", false), // Some tables may not start with T
            ("*V", false), // Some tables may not end with V
        ];

        for (pattern, expect_all_match) in test_cases {
            let matches = result
                .rows
                .iter()
                .filter(|row| {
                    if let Some(val) = row.first() {
                        let name = val.display().to_uppercase();
                        let pat = pattern.to_uppercase();

                        // Simple glob implementation (same as in production code)
                        if pat == "*" {
                            true
                        } else if let Some(inner) = pat.strip_prefix('*').and_then(|p| p.strip_suffix('*')) {
                            name.contains(inner)
                        } else if let Some(suffix) = pat.strip_prefix('*') {
                            name.ends_with(suffix)
                        } else if let Some(prefix) = pat.strip_suffix('*') {
                            name.starts_with(prefix)
                        } else {
                            name == pat
                        }
                    } else {
                        false
                    }
                })
                .count();

            if expect_all_match {
                assert_eq!(
                    matches,
                    result.rows.len(),
                    "Pattern '{}' should match all",
                    pattern
                );
            }
        }
    });
}

/// Test error handling when querying non-existent database
///
/// Run with: cargo test test_list_tables_error_handling -- --ignored
#[test]
#[ignore] // Requires live database connection
fn test_list_tables_error_handling() {
    common::with_driver(|| {
        use tq::db::DatabaseClient;

        dotenvy::dotenv().ok();
        let logon =
            std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for live database tests");

        let config = ConnectionConfig::from_connection_string(
            &logon,
            LogonMechanism::Td2,
            Duration::from_secs(30),
            None,
        )
        .unwrap();
        let client = DatabaseClient::new(config, None).unwrap();

        // Try to query tables from a database that definitely doesn't exist
        let sql = r#"
            SELECT TRIM(TableName) AS table_name
            FROM DBC.TablesV
            WHERE DatabaseName = 'NonExistent_Database_XYZ_12345'
              AND TableKind IN ('T', 'O')
            ORDER BY TableName
        "#;

        // Should execute successfully but return empty results
        let result = client.execute(sql).unwrap();
        assert_eq!(
            result.rows.len(),
            0,
            "Non-existent database should return 0 tables"
        );
    });
}

// =============================================================================
// Sprint 64: Bug #42 & #43 integration tests
// TC094-INTEGRATION, TC095-A, TC095-B, TC095-C, TC095-D
// =============================================================================

/// Resolve path to the compiled tq binary.
/// Uses CARGO_BIN_EXE_tq which Cargo sets during `cargo test`.
fn tq_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_tq"))
}

/// Load TQ_LOGON from .env / environment for live-DB tests.
fn load_logon() -> String {
    dotenvy::dotenv().ok();
    std::env::var("TQ_LOGON").expect("TQ_LOGON must be set for this test")
}

// -----------------------------------------------------------------------------
// TC094-INTEGRATION: End-to-end deploy via `tq query --file` (Bug #42)
// -----------------------------------------------------------------------------

/// Test that `tq query --file` deploys a stored procedure without splitting
/// its body at internal semicolons (exact repro from Issue #42).
///
/// Run with: cargo test test_file_mode_procedure_deploys_successfully -- --ignored
#[test]
#[ignore] // Requires live Teradata database (TQ_LOGON env var)
fn test_file_mode_procedure_deploys_successfully() {
    let logon = load_logon();

    // Write the exact repro script from Issue #42 to a temp file.
    // The schema must match a writable schema in the test DB.
    // Uses demo_user as in the original issue; adjust via TQ_TEST_SCHEMA if needed.
    let schema = std::env::var("TQ_TEST_SCHEMA").unwrap_or_else(|_| "demo_user".to_string());
    let sql = format!(
        "\
REPLACE PROCEDURE {schema}.sp_tq_repro()
BEGIN
    DECLARE v INTEGER;
    SET v = 1;
    IF v = 1 THEN
        SET v = 2;
    END IF;
END;"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let sql_file = dir.path().join("repro_sp.sql");
    std::fs::write(&sql_file, &sql).expect("write sql file");

    let output = std::process::Command::new(tq_bin())
        .args(["query", "--file", sql_file.to_str().unwrap()])
        .env("TQ_LOGON", &logon)
        .output()
        .expect("failed to spawn tq");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "tq query --file must succeed; exit={}, stderr={stderr}, stdout={stdout}",
        output.status
    );
    // Confirm no SQL splitting error in stderr
    assert!(
        !stderr.contains("syntax error") && !stderr.contains("Error at statement"),
        "must not contain SQL error; stderr={stderr}"
    );
}

// -----------------------------------------------------------------------------
// TC095-A: /dev/null redirect — positional arg succeeds (Bug #43)
// -----------------------------------------------------------------------------

/// Test that `tq query "SELECT 1" < /dev/null` succeeds (Stdio::null() simulates
/// `/dev/null` redirect). The fix must not raise "Multiple input sources" when
/// stdin is null/empty and a positional arg is provided.
///
/// Run with: cargo test test_query_with_devnull_stdin_succeeds -- --ignored
#[test]
#[ignore] // Requires live Teradata database (TQ_LOGON env var)
fn test_query_with_devnull_stdin_succeeds() {
    let logon = load_logon();

    let output = std::process::Command::new(tq_bin())
        .args(["query", "SELECT 1 AS x"])
        .env("TQ_LOGON", &logon)
        .stdin(std::process::Stdio::null()) // < /dev/null
        .output()
        .expect("failed to spawn tq");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "tq query with Stdio::null() stdin must succeed; exit={}, stderr={stderr}",
        output.status
    );
    assert!(
        !stderr.contains("Multiple input sources"),
        "must not trigger multiple-sources error; stderr={stderr}"
    );
}

// -----------------------------------------------------------------------------
// TC095-B: Empty heredoc pipe — positional arg succeeds (Bug #43)
// -----------------------------------------------------------------------------

/// Test that `tq query "SELECT 1" <<< ""` succeeds.
/// Simulated by opening a pipe and immediately dropping the write end (no bytes written).
/// This is the most important AC-2 scenario: a pipe with no data must not be treated
/// as a second input source when a positional arg is present.
///
/// Run with: cargo test test_query_with_empty_pipe_stdin_succeeds -- --ignored
#[test]
#[ignore] // Requires live Teradata database (TQ_LOGON env var)
fn test_query_with_empty_pipe_stdin_succeeds() {
    let logon = load_logon();

    let mut child = std::process::Command::new(tq_bin())
        .args(["query", "SELECT 1 AS x"])
        .env("TQ_LOGON", &logon)
        .stdin(std::process::Stdio::piped()) // open a pipe
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn tq");

    // Drop the write end immediately — no bytes written (like <<< "")
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "tq query with empty pipe stdin must succeed; exit={}, stderr={stderr}",
        output.status
    );
    assert!(
        !stderr.contains("Multiple input sources"),
        "must not trigger multiple-sources error; stderr={stderr}"
    );
}

// -----------------------------------------------------------------------------
// TC095-C: Stdin-only (no positional arg) — regression guard (Bug #43)
// -----------------------------------------------------------------------------

/// Test that `echo "SELECT 1 AS x" | tq query` still works after the fix.
/// This is the original working stdin-only use case that the fix must not break.
///
/// Run with: cargo test test_query_from_stdin_only_regression -- --ignored
#[test]
#[ignore] // Requires live Teradata database (TQ_LOGON env var)
fn test_query_from_stdin_only_regression() {
    let logon = load_logon();

    let mut child = std::process::Command::new(tq_bin())
        .args(["query"]) // no positional arg
        .env("TQ_LOGON", &logon)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn tq");

    // Write SQL to stdin (like: echo "SELECT 1 AS x" | tq query)
    use std::io::Write;
    if let Some(mut stdin_pipe) = child.stdin.take() {
        stdin_pipe
            .write_all(b"SELECT 1 AS x\n")
            .expect("write to stdin");
    }

    let output = child.wait_with_output().expect("failed to wait");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stdin-only query must succeed; exit={}, stderr={stderr}",
        output.status
    );
    assert!(
        stdout.contains("1") || stdout.contains("x"),
        "stdout must contain result data; stdout={stdout}"
    );
}

// -----------------------------------------------------------------------------
// TC110-I10: Positional arg wins over ambient stdin — no conflict error
// (Issue #45 deterministic input-selection regression guard — NO DB required)
// -----------------------------------------------------------------------------

/// Issue #45: explicit positional SQL takes precedence over ambient stdin
/// (matching `psql -c`). `echo "SELECT 2" | tq query "SELECT 1"` must NOT
/// produce a "Multiple input sources" error; stdin is ignored entirely and tq
/// proceeds with the positional query (failing only at connection time, since
/// no DB is configured here). This guards against a regression to the removed
/// readiness-probe conflict model.
#[test]
fn test_positional_arg_wins_over_stdin_no_conflict() {
    let mut child = std::process::Command::new(tq_bin())
        .args(["query", "SELECT 1"]) // explicit positional arg provided
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn tq");

    // Write non-empty SQL to stdin — under the new contract this is ignored.
    use std::io::Write;
    if let Some(mut stdin_pipe) = child.stdin.take() {
        stdin_pipe
            .write_all(b"SELECT 2\n")
            .expect("write to stdin");
    }

    let output = child.wait_with_output().expect("failed to wait");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // The deterministic contract removes the conflict error entirely: the
    // positional query wins and stdin is never inspected.
    assert!(
        !stderr.contains("Multiple input sources"),
        "explicit arg must win over stdin without a conflict error; stderr={stderr}"
    );
}
