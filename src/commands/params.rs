//! Implementation of the `tq params` subcommand
//!
//! Inspects and validates parameter YAML files used for SQL template substitution.

use crate::cli::{OutputFormat, ParamsArgs};
use crate::error::Result;
use crate::params::ParamStore;
use serde_json::json;
use std::io::Write;

/// Execute the `params` subcommand
pub fn execute<W: Write>(
    args: &ParamsArgs,
    global_params: &[std::path::PathBuf],
    writer: &mut W,
) -> Result<()> {
    let mut store = ParamStore::new();

    // Prefer explicit files passed to `tq params [files...]`, fall back to global `--params`
    let files_to_load = if !args.files.is_empty() {
        &args.files
    } else {
        global_params
    };

    for file in files_to_load {
        store.load_file(file)?;
    }

    let paths = store.list_available_paths();
    let format = if args.json {
        OutputFormat::Json
    } else {
        args.format
    };

    match format.canonical() {
        OutputFormat::Json => {
            let mut entries = Vec::new();
            for path in &paths {
                let val = store.resolve(path).unwrap_or_default();
                entries.push(json!({
                    "key": path,
                    "value": val,
                }));
            }
            let envelope = json!({
                "ok": true,
                "row_count": entries.len(),
                "data": entries,
            });
            writeln!(writer, "{}", serde_json::to_string_pretty(&envelope)?)?;
        }
        OutputFormat::Csv => {
            writeln!(writer, "key,value")?;
            for path in &paths {
                let val = store.resolve(path).unwrap_or_default();
                writeln!(writer, "{},\"{}\"", path, val.replace('"', "\"\""))?;
            }
        }
        OutputFormat::Markdown | OutputFormat::Md => {
            writeln!(writer, "| Key | Value |")?;
            writeln!(writer, "| --- | --- |")?;
            for path in &paths {
                let val = store.resolve(path).unwrap_or_default();
                writeln!(writer, "| `{}` | {} |", path, val)?;
            }
        }
        OutputFormat::Table => {
            if paths.is_empty() {
                writeln!(writer, "No parameters loaded.")?;
            } else {
                writeln!(writer, "{:<30} {:<30}", "KEY", "VALUE")?;
                writeln!(writer, "{:<30} {:<30}", "---", "---")?;
                for path in &paths {
                    let val = store.resolve(path).unwrap_or_default();
                    writeln!(writer, "{:<30} {:<30}", path, val)?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_execute_params_table_format() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("params.yaml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "db: dev_db\nuser: alice").unwrap();

        let args = ParamsArgs {
            files: vec![file_path],
            format: OutputFormat::Table,
            json: false,
        };

        let mut buf = Vec::new();
        execute(&args, &[], &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("KEY"));
        assert!(output.contains("VALUE"));
        assert!(output.contains("db"));
        assert!(output.contains("dev_db"));
        assert!(output.contains("user"));
        assert!(output.contains("alice"));
    }

    #[test]
    fn test_execute_params_json_format() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("params.yaml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "db: dev_db").unwrap();

        let args = ParamsArgs {
            files: vec![file_path],
            format: OutputFormat::Table,
            json: true,
        };

        let mut buf = Vec::new();
        execute(&args, &[], &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("\"ok\": true"));
        assert!(output.contains("\"key\": \"db\""));
        assert!(output.contains("\"value\": \"dev_db\""));
    }
}
