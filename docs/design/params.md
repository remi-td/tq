# Variable Substitution Design

This document explains how YAML-based variable substitution is implemented in tq.

**Related Specification**: `docs/specifications/batch-mode.md`

## Overview

tq supports parameterized SQL templates through YAML parameter files. Variables in SQL text are marked with `{{path.to.variable}}` syntax and resolved from loaded YAML parameter maps before the SQL is sent to Teradata.

## Module Structure

```
src/
├── params.rs           # Variable substitution engine (NEW)
├── cli.rs              # --params/-p flag on GlobalOpts
├── main.rs             # Wire params into query execution
└── commands/
    ├── query.rs        # Substitution before SQL execution
    └── repl/
        ├── metacommands.rs  # /params metacommand handler
        ├── state.rs         # ParamStore in ReplState
        └── metadata_completer.rs  # /params tab completion
```

## Core Engine: `src/params.rs`

### Data Types

```rust
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Holds merged parameter values from one or more YAML files.
///
/// The store is a single YAML mapping that results from deep-merging
/// all loaded files (later files override earlier ones at the leaf level).
pub struct ParamStore {
    /// The merged YAML tree
    values: YamlValue,
    /// Ordered list of loaded file paths (for display / diagnostics)
    loaded_files: Vec<PathBuf>,
}
```

### YAML Parsing

Parameter files are standard YAML maps. Example:

```yaml
# params/dev.yaml
target:
  database: DEV_DB
  schema: staging

batch_size: 1000
run_date: "2026-03-20"
```

Parsing uses `serde_yaml::from_reader` which produces a `serde_yaml::Value` tree. This avoids the need for a fixed struct and allows arbitrary nesting.

```rust
impl ParamStore {
    /// Create an empty store
    pub fn new() -> Self {
        Self {
            values: YamlValue::Mapping(serde_yaml::Mapping::new()),
            loaded_files: Vec::new(),
        }
    }

    /// Load and merge a YAML file into the store
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        let file = std::fs::File::open(path).map_err(|e| ParamError::FileNotFound {
            path: path.to_path_buf(),
            source: e,
        })?;

        let parsed: YamlValue = serde_yaml::from_reader(file)
            .map_err(|e| ParamError::YamlParseError {
                path: path.to_path_buf(),
                source: e,
            })?;

        // Validate that the root is a mapping
        if !parsed.is_mapping() {
            return Err(ParamError::YamlParseError {
                path: path.to_path_buf(),
                source: serde_yaml::Error::custom(
                    "Parameter file must contain a YAML mapping at the root level"
                ),
            });
        }

        // Deep merge into existing values
        self.values = deep_merge(self.values.clone(), parsed);
        self.loaded_files.push(path.to_path_buf());
        Ok(())
    }

    /// Clear all loaded parameters
    pub fn clear(&mut self) {
        self.values = YamlValue::Mapping(serde_yaml::Mapping::new());
        self.loaded_files.clear();
    }

    /// Check if any parameters are loaded
    pub fn is_empty(&self) -> bool {
        match &self.values {
            YamlValue::Mapping(m) => m.is_empty(),
            _ => true,
        }
    }

    /// Get the list of loaded file paths
    pub fn loaded_files(&self) -> &[PathBuf] {
        &self.loaded_files
    }
}
```

### Deep Merge

When multiple parameter files are loaded, later files override earlier ones at the leaf level. Mappings are recursively merged; non-mapping values are replaced.

```rust
/// Deep merge two YAML values. `overlay` takes precedence over `base`.
fn deep_merge(base: YamlValue, overlay: YamlValue) -> YamlValue {
    match (base, overlay) {
        (YamlValue::Mapping(mut base_map), YamlValue::Mapping(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                let merged = if let Some(base_val) = base_map.remove(&key) {
                    deep_merge(base_val, overlay_val)
                } else {
                    overlay_val
                };
                base_map.insert(key, merged);
            }
            YamlValue::Mapping(base_map)
        }
        // Non-mapping overlay replaces base entirely
        (_, overlay) => overlay,
    }
}
```

### Variable Resolution

Variables use `{{path.to.variable}}` syntax. The path segments are split on `.` and used to traverse the YAML tree.

```rust
impl ParamStore {
    /// Resolve a dot-separated path to a string value.
    ///
    /// Returns the leaf value formatted as a string, or an error if the path
    /// does not exist in the loaded parameters.
    pub fn resolve(&self, path: &str) -> Result<String> {
        // Handle $ENV.* special prefix
        if let Some(env_var) = path.strip_prefix("$ENV.") {
            return std::env::var(env_var).map_err(|_| ParamError::EnvVarNotFound {
                var_name: env_var.to_string(),
            });
        }

        let segments: Vec<&str> = path.split('.').collect();
        let mut current = &self.values;

        for segment in &segments {
            match current {
                YamlValue::Mapping(map) => {
                    let key = YamlValue::String(segment.to_string());
                    current = map.get(&key).ok_or_else(|| {
                        ParamError::UndefinedVariable {
                            variable: path.to_string(),
                            available: self.list_available_paths(),
                        }
                    })?;
                }
                _ => {
                    return Err(ParamError::UndefinedVariable {
                        variable: path.to_string(),
                        available: self.list_available_paths(),
                    });
                }
            }
        }

        // Convert leaf value to string
        yaml_value_to_string(current)
    }

    /// List all available dot-separated paths for error messages
    fn list_available_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        collect_paths(&self.values, String::new(), &mut paths);
        paths.sort();
        paths
    }
}

/// Convert a YAML leaf value to its string representation for SQL substitution.
fn yaml_value_to_string(value: &YamlValue) -> Result<String> {
    match value {
        YamlValue::String(s) => Ok(s.clone()),
        YamlValue::Number(n) => Ok(n.to_string()),
        YamlValue::Bool(b) => Ok(b.to_string()),
        YamlValue::Null => Ok("NULL".to_string()),
        YamlValue::Mapping(_) | YamlValue::Sequence(_) | YamlValue::Tagged(_) => {
            Err(ParamError::NonScalarValue {
                detail: "Variable path resolves to a mapping or sequence, not a scalar value".to_string(),
            })
        }
    }
}

/// Recursively collect all leaf paths in the YAML tree.
fn collect_paths(value: &YamlValue, prefix: String, paths: &mut Vec<String>) {
    match value {
        YamlValue::Mapping(map) => {
            for (key, val) in map {
                if let YamlValue::String(key_str) = key {
                    let new_prefix = if prefix.is_empty() {
                        key_str.clone()
                    } else {
                        format!("{}.{}", prefix, key_str)
                    };
                    collect_paths(val, new_prefix, paths);
                }
            }
        }
        _ => {
            if !prefix.is_empty() {
                paths.push(prefix);
            }
        }
    }
}
```

### SQL Substitution

The `substitute` function finds all `{{...}}` markers and replaces them with resolved values.

```rust
/// Regex pattern for variable markers: {{path.to.variable}}
/// Allows alphanumeric, underscore, dot, and $ characters inside braces.
const VAR_PATTERN: &str = r"\{\{([a-zA-Z0-9_.$]+)\}\}";

impl ParamStore {
    /// Substitute all {{variable}} markers in the given SQL text.
    ///
    /// Returns the SQL with all variables replaced, or an error if any
    /// variable cannot be resolved.
    pub fn substitute(&self, sql: &str) -> Result<String> {
        let re = regex::Regex::new(VAR_PATTERN).expect("valid regex");
        let mut result = String::with_capacity(sql.len());
        let mut last_end = 0;
        let mut errors: Vec<String> = Vec::new();

        for cap in re.captures_iter(sql) {
            let full_match = cap.get(0).unwrap();
            let var_path = &cap[1];

            result.push_str(&sql[last_end..full_match.start()]);

            match self.resolve(var_path) {
                Ok(value) => result.push_str(&value),
                Err(e) => errors.push(format!("{}", e)),
            }

            last_end = full_match.end();
        }

        if !errors.is_empty() {
            return Err(ParamError::SubstitutionFailed {
                errors,
            });
        }

        result.push_str(&sql[last_end..]);
        Ok(result)
    }
}
```

**Design decision**: All variable references are resolved in a single pass. If any variable fails to resolve, the entire substitution fails with a collected error listing all undefined variables. This fail-fast approach prevents partial SQL from being sent to Teradata.

### Environment Variable Integration

The `$ENV.` prefix is a special namespace that reads from process environment variables. It does not require loading any YAML file.

```rust
// In resolve():
if let Some(env_var) = path.strip_prefix("$ENV.") {
    return std::env::var(env_var).map_err(|_| ParamError::EnvVarNotFound {
        var_name: env_var.to_string(),
    });
}
```

Example usage:
```sql
SELECT * FROM {{$ENV.TARGET_DB}}.orders WHERE created_date = '{{run_date}}'
```

### Error Types

```rust
/// Errors specific to parameter substitution
#[derive(Debug, thiserror::Error)]
pub enum ParamError {
    #[error("Parameter file not found: '{path}'")]
    FileNotFound {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse YAML in '{path}': {source}")]
    YamlParseError {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("Undefined variable '{{{{{{variable}}}}}}'. Available variables:\n  {}", available.join("\n  "))]
    UndefinedVariable {
        variable: String,
        available: Vec<String>,
    },

    #[error("Environment variable '{var_name}' is not set")]
    EnvVarNotFound {
        var_name: String,
    },

    #[error("Variable resolves to a non-scalar value: {detail}")]
    NonScalarValue {
        detail: String,
    },

    #[error("Variable substitution failed:\n{}", errors.join("\n"))]
    SubstitutionFailed {
        errors: Vec<String>,
    },
}
```

**Conversion to TqError**: `ParamError` implements `From<ParamError> for TqError` to integrate with the existing error pipeline. All param errors map to exit code 2 (usage error) since they indicate problems with user-supplied parameter files.

## Integration Points

### CLI Integration

The `--params`/`-p` flag is added to `GlobalOpts` so it applies to all database commands:

```rust
// src/cli.rs - in GlobalOpts

/// YAML parameter file(s) for variable substitution
///
/// Load variables from YAML files. Variables in SQL are referenced
/// as {{variable.path}}. Multiple files can be specified; later files
/// override earlier ones.
///
/// Example: tq -p params.yaml query "SELECT * FROM {{target.database}}.orders"
#[arg(short = 'p', long = "params", value_name = "FILE", global = true)]
pub params: Vec<PathBuf>,
```

Using `Vec<PathBuf>` allows multiple `-p` flags naturally via clap's `action = Append` (the default for Vec).

### Query Execution Pipeline

Variable substitution hooks into the query pipeline between reading the SQL input and sending it to Teradata. This is the same position for both batch and single-query modes.

```
Read SQL Input -> Variable Substitution -> SQL Parsing/Splitting -> Execute
```

In `src/main.rs`, the `ParamStore` is built once from CLI args:

```rust
// In run()
let param_store = build_param_store(&cli.global.params)?;

// Pass to command execution
Command::Query(args) => {
    // param_store passed to execute functions
    commands::query::execute(&client, &args, &param_store, &mut stdout, use_color, verbose)?;
}
```

In `src/commands/query.rs`, substitution happens after reading SQL but before execution:

```rust
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &QueryArgs,
    params: &ParamStore,
    writer: &mut W,
    use_color: bool,
    verbose: bool,
) -> Result<()> {
    let source = determine_input_source(args)?;
    let sql = read_input_sql(&source)?;

    // Apply variable substitution before any SQL processing
    let sql = if !params.is_empty() {
        params.substitute(&sql)?
    } else {
        sql
    };

    // Continue with existing execution logic...
}
```

### REPL Integration

In REPL mode, the `ParamStore` lives in `ReplState` and can be modified at runtime via the `/params` metacommand.

```rust
// src/commands/repl/state.rs
pub struct ReplState {
    // ... existing fields ...
    pub params: ParamStore,
}
```

Substitution is applied to each SQL statement before execution in the REPL executor:

```rust
// src/commands/repl/executor.rs - in execute_sql()
let sql = if !state.params.is_empty() {
    state.params.substitute(trimmed)?
} else {
    trimmed.to_string()
};
```

The `/params` metacommand provides runtime parameter management:

```rust
// src/commands/repl/metacommands.rs

// In the main match:
"params" | "p" => {
    handle_params(&args, state, writer)?;
}

fn handle_params<W: Write>(
    args: &[&str],
    state: &mut ReplState,
    writer: &mut W,
) -> Result<()> {
    match args.first().copied() {
        Some("load") => {
            let path = args.get(1).ok_or_else(|| /* usage error */)?;
            state.params.load_file(Path::new(path))?;
            writeln!(writer, "Loaded parameters from '{}'", path)?;
        }
        Some("unload") => {
            state.params.clear();
            writeln!(writer, "All parameters cleared.")?;
        }
        Some("show") | None => {
            if state.params.is_empty() {
                writeln!(writer, "No parameters loaded.")?;
            } else {
                writeln!(writer, "Loaded files:")?;
                for f in state.params.loaded_files() {
                    writeln!(writer, "  {}", f.display())?;
                }
                writeln!(writer)?;
                writeln!(writer, "Available variables:")?;
                for path in state.params.list_available_paths() {
                    writeln!(writer, "  {{{{{}}}}}", path)?;
                }
            }
        }
        Some(other) => {
            writeln!(writer, "Unknown /params subcommand: {}", other)?;
            writeln!(writer, "Usage: /params [load <file> | unload | show]")?;
        }
    }
    Ok(())
}
```

## Dependencies

| Crate | Purpose | Version |
|-------|---------|---------|
| `serde_yaml` | YAML parsing to dynamic Value tree | `0.9` |
| `regex` | Variable marker pattern matching | `1.x` |

`serde` is already a dependency. `serde_yaml` adds YAML support. `regex` is used for the `{{...}}` pattern matching; it compiles the pattern once and reuses it.

**Alternative considered**: Manual string scanning instead of regex. Rejected because regex handles edge cases (nested braces, escaped characters) more robustly and the compile cost is negligible for a one-shot tool.

## Testing Strategy

### Unit Tests in `src/params.rs`

Tests are organized by component:

1. **YAML parsing**: Load valid files, reject invalid YAML, reject non-mapping roots
2. **Deep merge**: Verify leaf override, nested merge, disjoint merge
3. **Variable resolution**: Simple paths, nested paths, missing paths, `$ENV.*` resolution
4. **SQL substitution**: Single variable, multiple variables, no variables (passthrough), undefined variable error, mixed valid/invalid
5. **Error messages**: Verify undefined variable lists available paths, YAML errors include file path
6. **Edge cases**: Empty parameter store, empty SQL, variable at start/end of SQL, adjacent variables

### Example Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_simple() {
        let mut store = ParamStore::new();
        // Load from string for testing
        store.load_yaml_str("database: MY_DB\ntable: orders").unwrap();
        let result = store.substitute("SELECT * FROM {{database}}.{{table}}").unwrap();
        assert_eq!(result, "SELECT * FROM MY_DB.orders");
    }

    #[test]
    fn test_substitute_nested() {
        let mut store = ParamStore::new();
        store.load_yaml_str("target:\n  db: DEV\n  schema: staging").unwrap();
        let result = store.substitute("SELECT * FROM {{target.db}}.{{target.schema}}.t1").unwrap();
        assert_eq!(result, "SELECT * FROM DEV.staging.t1");
    }

    #[test]
    fn test_substitute_env_var() {
        std::env::set_var("TQ_TEST_DB", "ENV_DB");
        let store = ParamStore::new();
        let result = store.substitute("SELECT * FROM {{$ENV.TQ_TEST_DB}}.t1").unwrap();
        assert_eq!(result, "SELECT * FROM ENV_DB.t1");
        std::env::remove_var("TQ_TEST_DB");
    }

    #[test]
    fn test_undefined_variable_error() {
        let mut store = ParamStore::new();
        store.load_yaml_str("database: MY_DB").unwrap();
        let err = store.substitute("SELECT * FROM {{schema}}.t1").unwrap_err();
        assert!(err.to_string().contains("schema"));
        assert!(err.to_string().contains("database")); // lists available
    }

    #[test]
    fn test_no_variables_passthrough() {
        let store = ParamStore::new();
        let result = store.substitute("SELECT 1").unwrap();
        assert_eq!(result, "SELECT 1");
    }

    #[test]
    fn test_deep_merge_override() {
        let base = serde_yaml::from_str("a:\n  b: 1\n  c: 2").unwrap();
        let overlay = serde_yaml::from_str("a:\n  b: 99").unwrap();
        let merged = deep_merge(base, overlay);
        // a.b should be 99, a.c should be 2
    }
}
```

## Security Considerations

1. **No SQL injection risk from parameters**: Variables are substituted as literal text replacement. This is intentional -- users own both the SQL template and the parameter files. The substitution is equivalent to the user typing the value directly.

2. **Environment variables**: `$ENV.*` allows reading any environment variable the process can access. This is by design for CI/CD integration. No filtering is applied.

3. **File access**: Parameter files are read with normal file permissions. No special permission checking (unlike password files) because parameter files contain non-sensitive configuration values.

## Design Trade-offs

### Chosen: `{{var}}` Mustache-style syntax
**Pros**: Unambiguous in SQL (no SQL syntax uses `{{`), familiar from Jinja2/Mustache/Handlebars
**Cons**: More characters to type than `$var` or `:var`
**Rationale**: Zero conflict with SQL syntax is the priority. `:var` conflicts with Teradata-style bind parameters, `$var` conflicts with shell expansion.

### Chosen: YAML over TOML/JSON
**Pros**: Cleaner syntax for nested config, supports comments, widely used for config
**Cons**: Another dependency (`serde_yaml`), YAML has footguns (e.g., `no` parsed as boolean)
**Rationale**: YAML is the most natural format for parameter files that users will hand-edit.

### Chosen: Fail-all-or-nothing substitution
**Pros**: Prevents partial SQL from being sent to Teradata (which would cause confusing errors)
**Cons**: User must fix all variables at once
**Rationale**: Sending `SELECT * FROM {{db}}.orders` to Teradata would produce a syntax error that does not explain the real problem. Better to fail with a clear "undefined variable" message.

### Chosen: regex for pattern matching
**Pros**: Robust handling of the `{{...}}` pattern, well-tested library
**Cons**: Adds `regex` dependency
**Rationale**: Manual parsing is error-prone for edge cases. The regex crate is widely used in the Rust ecosystem and adds minimal overhead.

## Code Linkage

| Component | File Path | Key Types |
|-----------|-----------|-----------|
| Substitution engine | `src/params.rs` | `ParamStore`, `ParamError` |
| CLI flag | `src/cli.rs` | `GlobalOpts::params` |
| Query pipeline hook | `src/commands/query.rs` | `execute()` signature change |
| REPL state | `src/commands/repl/state.rs` | `ReplState::params` |
| REPL executor hook | `src/commands/repl/executor.rs` | `execute_sql()` |
| REPL metacommand | `src/commands/repl/metacommands.rs` | `handle_params()` |
| Tab completion | `src/commands/repl/metadata_completer.rs` | `/params` in registry |
| Help topic | `src/help.rs` | `params_help()` |
