//! Implementation of the `tq errorlevel` subcommand
//!
//! Inspects error severity level classifications and custom overrides.

use crate::cli::{ErrorlevelArgs, OutputFormat};
use crate::error::{Severity, TqError};
use serde_json::json;
use std::collections::HashMap;
use std::io::Write;

/// Execute the `errorlevel` subcommand
pub fn execute<W: Write>(
    args: &ErrorlevelArgs,
    overrides: &HashMap<u32, Severity>,
    writer: &mut W,
) -> Result<(), TqError> {
    let format = if args.json {
        OutputFormat::Json
    } else {
        args.format
    };

    let mut sorted_entries: Vec<(u32, String)> = overrides
        .iter()
        .map(|(code, sev)| (*code, sev.to_string()))
        .collect();
    sorted_entries.sort_by_key(|k| k.0);

    match format.canonical() {
        OutputFormat::Json => {
            let data: Vec<serde_json::Value> = sorted_entries
                .iter()
                .map(|(code, sev)| {
                    json!({
                        "error_code": code,
                        "severity": sev,
                    })
                })
                .collect();
            let envelope = json!({
                "ok": true,
                "row_count": data.len(),
                "data": data,
            });
            writeln!(writer, "{}", serde_json::to_string_pretty(&envelope)?)?;
        }
        OutputFormat::Csv => {
            writeln!(writer, "error_code,severity")?;
            for (code, sev) in &sorted_entries {
                writeln!(writer, "{},{}", code, sev)?;
            }
        }
        OutputFormat::Markdown | OutputFormat::Md => {
            writeln!(writer, "| Error Code | Severity |")?;
            writeln!(writer, "| --- | --- |")?;
            if sorted_entries.is_empty() {
                writeln!(writer, "| (none) | No custom errorlevel overrides set |")?;
            } else {
                for (code, sev) in &sorted_entries {
                    writeln!(writer, "| `{}` | {} |", code, sev)?;
                }
            }
        }
        OutputFormat::Table => {
            if sorted_entries.is_empty() {
                writeln!(writer, "No custom errorlevel overrides configured.")?;
                writeln!(
                    writer,
                    "Use '--errorlevel CODE SEVERITY' (e.g. '--errorlevel 3802 warning') to set overrides."
                )?;
            } else {
                writeln!(writer, "{:<15} {:<15}", "ERROR CODE", "SEVERITY")?;
                writeln!(writer, "{:<15} {:<15}", "----------", "--------")?;
                for (code, sev) in &sorted_entries {
                    writeln!(writer, "{:<15} {:<15}", code, sev)?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_errorlevel_empty() {
        let args = ErrorlevelArgs {
            format: OutputFormat::Table,
            json: false,
        };
        let overrides = HashMap::new();
        let mut buf = Vec::new();
        execute(&args, &overrides, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("No custom errorlevel overrides configured"));
    }

    #[test]
    fn test_execute_errorlevel_json() {
        let args = ErrorlevelArgs {
            format: OutputFormat::Table,
            json: true,
        };
        let mut overrides = HashMap::new();
        overrides.insert(3802, Severity::Warning);
        let mut buf = Vec::new();
        execute(&args, &overrides, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("\"ok\": true"));
        assert!(output.contains("\"error_code\": 3802"));
        assert!(output.contains("\"severity\": \"warning\""));
    }
}
