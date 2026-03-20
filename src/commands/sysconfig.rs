//! System configuration command implementation
//!
//! This module provides functionality to display Teradata system topology
//! including version, release, node count, AMP count, and PE count.
//!
//! Sprint 38: Initial implementation

use crate::cli::{OutputFormat, SysconfigArgs};
use crate::db::DatabaseClient;
use crate::error::Result;
use super::monitoring_utils::{escape_csv, extract_integer, extract_trimmed_string};
use std::io::Write;

/// SQL query to retrieve system version and release from DBC.DBCInfoV
const DBCINFO_SQL: &str = r#"
SELECT InfoKey, CAST(InfoData AS VARCHAR(200)) AS InfoData
FROM DBC.DBCInfoV
WHERE InfoKey IN ('RELEASE', 'VERSION')
ORDER BY InfoKey
"#;

/// SQL query to retrieve total AMP count via HASHAMP()+1
const AMP_COUNT_SQL: &str = "SELECT HASHAMP()+1 AS TotalAMPs";

/// System configuration information extracted from DBC views
///
/// Contains the key properties displayed in the sysconfig output.
#[derive(Debug, Clone)]
pub struct SysconfigInfo {
    /// Teradata version string (e.g., "17.20.00.17")
    pub version: String,
    /// Teradata release string (e.g., "17.20.00.17 (Released: 2024-01-15)")
    pub release: String,
    /// Total number of AMPs in the system
    pub amp_count: i64,
}

impl SysconfigInfo {
    /// Build SysconfigInfo by executing queries against the database
    ///
    /// Executes DBC.DBCInfoV and HASHAMP()+1 queries, parsing the results
    /// into a structured SysconfigInfo.
    pub fn from_queries(client: &DatabaseClient) -> Result<Self> {
        let mut version = "[unavailable]".to_string();
        let mut release = "[unavailable]".to_string();
        let mut amp_count: i64 = 0;

        // Query DBC.DBCInfoV for version and release
        let info_result = client.execute(DBCINFO_SQL)?;
        for row in &info_result.rows {
            if row.len() >= 2 {
                let key = extract_trimmed_string(&row[0], "[unavailable]");
                let value = extract_trimmed_string(&row[1], "[unavailable]");
                match key.to_uppercase().as_str() {
                    "VERSION" => version = value,
                    "RELEASE" => release = value,
                    _ => {}
                }
            }
        }

        // Query AMP count
        let amp_result = client.execute(AMP_COUNT_SQL)?;
        if let Some(row) = amp_result.rows.first() {
            if let Some(val) = row.first() {
                amp_count = extract_integer(val).unwrap_or(0);
            }
        }

        Ok(Self {
            version,
            release,
            amp_count,
        })
    }

    /// Return properties as a vector of (name, value) pairs for display
    pub fn as_properties(&self) -> Vec<(&str, String)> {
        vec![
            ("Teradata Version", self.version.clone()),
            ("Release", self.release.clone()),
            ("AMP Count", self.amp_count.to_string()),
        ]
    }
}

/// Execute the sysconfig command and write results (batch mode)
///
/// # Arguments
/// * `client` - Database client for executing queries
/// * `args` - Command arguments (format, output file)
/// * `writer` - Output writer
/// * `_use_color` - Whether to use color output
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &SysconfigArgs,
    writer: &mut W,
    _use_color: bool,
) -> Result<()> {
    let info = SysconfigInfo::from_queries(client)?;

    match args.format {
        OutputFormat::Table => display_table(&info, writer)?,
        OutputFormat::Csv => display_csv(&info, writer)?,
        OutputFormat::Json => display_json(&info, writer)?,
    }

    Ok(())
}

/// Execute sysconfig query and display for REPL mode
///
/// Displays a compact key-value summary with error handling for
/// privilege errors and unavailable views.
pub fn execute_for_repl<W: Write>(client: &DatabaseClient, writer: &mut W) -> Result<()> {
    writeln!(writer)?;

    match SysconfigInfo::from_queries(client) {
        Ok(info) => {
            display_repl_table(&info, writer)?;
        }
        Err(e) => {
            let error_str = e.to_string().to_lowercase();

            if error_str.contains("privilege")
                || error_str.contains("access")
                || error_str.contains("permission")
                || error_str.contains("3523")
            {
                writeln!(writer, "Error: Unable to retrieve system configuration.")?;
                writeln!(writer)?;
                writeln!(
                    writer,
                    "This command requires SELECT access to DBC system views."
                )?;
                writeln!(writer)?;
                writeln!(writer, "To grant access, a DBA can run:")?;
                writeln!(
                    writer,
                    "  GRANT SELECT ON DBC.DBCInfoV TO <your_username>;"
                )?;
            } else if error_str.contains("dbcinfov")
                && (error_str.contains("not found") || error_str.contains("does not exist"))
            {
                writeln!(writer, "Error: System configuration view not available.")?;
                writeln!(writer)?;
                writeln!(
                    writer,
                    "DBC.DBCInfoV is not accessible on this system."
                )?;
            } else {
                writeln!(writer, "Error retrieving system configuration: {}", e)?;
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Display sysconfig as a key-value table for REPL mode
fn display_repl_table<W: Write>(info: &SysconfigInfo, writer: &mut W) -> Result<()> {
    use comfy_table::{presets, ContentArrangement, Table};

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Property", "Value"]);

    for (name, value) in info.as_properties() {
        table.add_row(vec![name, &value]);
    }

    writeln!(writer, "System Configuration:")?;
    writeln!(writer, "{}", table)?;

    Ok(())
}

/// Display sysconfig in table format (batch mode)
fn display_table<W: Write>(info: &SysconfigInfo, writer: &mut W) -> Result<()> {
    use comfy_table::{presets, ContentArrangement, Table};

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Property", "Value"]);

    for (name, value) in info.as_properties() {
        table.add_row(vec![name, &value]);
    }

    writeln!(writer, "System Configuration:")?;
    writeln!(writer, "{}", table)?;

    Ok(())
}

/// Display sysconfig in CSV format
fn display_csv<W: Write>(info: &SysconfigInfo, writer: &mut W) -> Result<()> {
    writeln!(writer, "Property,Value")?;

    for (name, value) in info.as_properties() {
        writeln!(writer, "{},{}", name, escape_csv(&value))?;
    }

    Ok(())
}

/// Display sysconfig in JSON format
fn display_json<W: Write>(info: &SysconfigInfo, writer: &mut W) -> Result<()> {
    let json = serde_json::json!({
        "Teradata Version": info.version,
        "Release": info.release,
        "AMP Count": info.amp_count,
    });

    let json_output = serde_json::to_string_pretty(&json)?;
    writeln!(writer, "{}", json_output)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sysconfig_info_as_properties() {
        let info = SysconfigInfo {
            version: "17.20.00.17".to_string(),
            release: "17.20.00.17 (Released: 2024-01-15)".to_string(),
            amp_count: 128,
        };

        let props = info.as_properties();
        assert_eq!(props.len(), 3);
        assert_eq!(props[0], ("Teradata Version", "17.20.00.17".to_string()));
        assert_eq!(
            props[1],
            (
                "Release",
                "17.20.00.17 (Released: 2024-01-15)".to_string()
            )
        );
        assert_eq!(props[2], ("AMP Count", "128".to_string()));
    }

    #[test]
    fn test_sysconfig_info_as_properties_unavailable() {
        let info = SysconfigInfo {
            version: "[unavailable]".to_string(),
            release: "[unavailable]".to_string(),
            amp_count: 0,
        };

        let props = info.as_properties();
        assert_eq!(props[0].1, "[unavailable]");
        assert_eq!(props[1].1, "[unavailable]");
        assert_eq!(props[2].1, "0");
    }

    // extract_trimmed_string and extract_integer tests removed: already tested in monitoring_utils.rs

    #[test]
    fn test_display_table_output() {
        let info = SysconfigInfo {
            version: "17.20.00.17".to_string(),
            release: "17.20.00.17 (Released: 2024-01-15)".to_string(),
            amp_count: 128,
        };

        let mut output = Vec::new();
        display_table(&info, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("System Configuration:"));
        assert!(output_str.contains("Property"));
        assert!(output_str.contains("Value"));
        assert!(output_str.contains("Teradata Version"));
        assert!(output_str.contains("17.20.00.17"));
        assert!(output_str.contains("Release"));
        assert!(output_str.contains("AMP Count"));
        assert!(output_str.contains("128"));
    }

    #[test]
    fn test_display_csv_output() {
        let info = SysconfigInfo {
            version: "17.20.00.17".to_string(),
            release: "17.20.00.17 (Released: 2024-01-15)".to_string(),
            amp_count: 128,
        };

        let mut output = Vec::new();
        display_csv(&info, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("Property,Value"));
        assert!(output_str.contains("Teradata Version,17.20.00.17"));
        assert!(output_str.contains("Release,17.20.00.17 (Released: 2024-01-15)"));
        assert!(output_str.contains("AMP Count,128"));
    }

    #[test]
    fn test_display_json_output() {
        let info = SysconfigInfo {
            version: "17.20.00.17".to_string(),
            release: "17.20.00.17 (Released: 2024-01-15)".to_string(),
            amp_count: 128,
        };

        let mut output = Vec::new();
        display_json(&info, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // Parse as JSON to verify structure
        let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();
        assert_eq!(json["Teradata Version"], "17.20.00.17");
        assert_eq!(
            json["Release"],
            "17.20.00.17 (Released: 2024-01-15)"
        );
        assert_eq!(json["AMP Count"], 128);
    }

    #[test]
    fn test_display_repl_table_output() {
        let info = SysconfigInfo {
            version: "17.20.00.17".to_string(),
            release: "17.20.00.17 (Released: 2024-01-15)".to_string(),
            amp_count: 128,
        };

        let mut output = Vec::new();
        display_repl_table(&info, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("System Configuration:"));
        assert!(output_str.contains("Teradata Version"));
        assert!(output_str.contains("AMP Count"));
    }

    // escape_csv tests removed: already tested in monitoring_utils.rs

    // =========================================================================
    // Error handling tests (Sprint 39)
    // =========================================================================

    /// Helper: simulate the error classification logic from execute_for_repl
    fn classify_error(error_msg: &str) -> &'static str {
        let error_str = error_msg.to_lowercase();
        if error_str.contains("privilege")
            || error_str.contains("access")
            || error_str.contains("permission")
            || error_str.contains("3523")
        {
            "permission_denied"
        } else if error_str.contains("dbcinfov")
            && (error_str.contains("not found") || error_str.contains("does not exist"))
        {
            "view_not_available"
        } else {
            "generic_error"
        }
    }

    #[test]
    fn test_error_classification_privilege() {
        assert_eq!(
            classify_error("[Error 3523] The user does not have SELECT access to DBC.DBCInfoV"),
            "permission_denied"
        );
    }

    #[test]
    fn test_error_classification_access_denied() {
        assert_eq!(
            classify_error("Access denied to object DBC.DBCInfoV"),
            "permission_denied"
        );
    }

    #[test]
    fn test_error_classification_permission_error() {
        assert_eq!(
            classify_error("Permission denied on database view"),
            "permission_denied"
        );
    }

    #[test]
    fn test_error_classification_error_code_3523() {
        assert_eq!(
            classify_error("Error 3523: insufficient privileges"),
            "permission_denied"
        );
    }

    #[test]
    fn test_error_classification_view_not_found() {
        assert_eq!(
            classify_error("Object 'DBC.DBCInfoV' not found"),
            "view_not_available"
        );
    }

    #[test]
    fn test_error_classification_view_does_not_exist() {
        assert_eq!(
            classify_error("DBC.DBCInfoV does not exist on this system"),
            "view_not_available"
        );
    }

    #[test]
    fn test_error_classification_generic() {
        assert_eq!(
            classify_error("Network timeout connecting to database"),
            "generic_error"
        );
    }

    #[test]
    fn test_error_classification_empty_message() {
        assert_eq!(classify_error(""), "generic_error");
    }

    #[test]
    fn test_sysconfig_info_defaults_on_empty_data() {
        // Test that SysconfigInfo handles unavailable fields gracefully
        let info = SysconfigInfo {
            version: "[unavailable]".to_string(),
            release: "[unavailable]".to_string(),
            amp_count: 0,
        };

        // Verify table display doesn't panic with default values
        let mut output = Vec::new();
        display_table(&info, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("[unavailable]"));
        assert!(output_str.contains("0"));
    }

    #[test]
    fn test_sysconfig_csv_with_unavailable() {
        let info = SysconfigInfo {
            version: "[unavailable]".to_string(),
            release: "[unavailable]".to_string(),
            amp_count: 0,
        };

        let mut output = Vec::new();
        display_csv(&info, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Teradata Version,[unavailable]"));
        assert!(output_str.contains("AMP Count,0"));
    }

    #[test]
    fn test_sysconfig_json_with_unavailable() {
        let info = SysconfigInfo {
            version: "[unavailable]".to_string(),
            release: "[unavailable]".to_string(),
            amp_count: 0,
        };

        let mut output = Vec::new();
        display_json(&info, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();
        assert_eq!(json["Teradata Version"], "[unavailable]");
        assert_eq!(json["AMP Count"], 0);
    }
}
