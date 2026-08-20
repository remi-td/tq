//! Variable substitution engine for SQL templates
//!
//! Parses YAML parameter files and substitutes `{{variable}}` markers in SQL text.
//! Supports nested YAML structures with dot-notation paths and environment variable
//! access via the `$ENV.` prefix.
//!
//! ## Design
//!
//! - `ParamStore` holds a merged YAML value tree from one or more files
//! - Deep merge: later files override earlier files at the leaf level
//! - Substitution is all-or-nothing: if any variable fails, the entire operation fails
//! - `$ENV.*` reads from the process environment (no YAML entry needed)

use regex::Regex;
use serde_yaml::Value as YamlValue;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Pre-compiled regex for `{{variable}}`, `{{ variable }}`, and `${variable}` pattern matching.
/// Compiled once on first use via `LazyLock` to avoid repeated compilation
/// on every `substitute()` call.
static VARIABLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:\{\{\s*([a-zA-Z0-9_.$]+)\s*\}\}|\$\{([a-zA-Z0-9_.$]+)\})").expect("valid regex")
});

/// Errors specific to parameter substitution
#[derive(Debug)]
pub enum ParamError {
    /// Parameter file not found or unreadable
    FileNotFound {
        path: PathBuf,
        source: std::io::Error,
    },

    /// YAML parse error
    YamlParseError {
        path: PathBuf,
        message: String,
    },

    /// Variable path not found in loaded parameters
    UndefinedVariable {
        variable: String,
        available: Vec<String>,
    },

    /// Environment variable not set
    EnvVarNotFound {
        var_name: String,
    },

    /// Variable resolves to a non-scalar (mapping or sequence)
    NonScalarValue {
        variable: String,
    },

    /// One or more substitutions failed
    SubstitutionFailed {
        errors: Vec<String>,
    },

    /// YAML root is not a mapping
    InvalidRoot {
        path: PathBuf,
    },
}

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamError::FileNotFound { path, source } => {
                write!(
                    f,
                    "Error: Parameter file not found\n\n\
                     Could not read: {}\n\
                     Reason: {}\n\n\
                     Check:\n  \
                     - File path is correct (relative paths are resolved from current directory)\n  \
                     - File exists and is readable\n  \
                     - Current directory: {}",
                    path.display(),
                    source,
                    std::env::current_dir()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| "<unknown>".to_string())
                )
            }
            ParamError::YamlParseError { path, message } => {
                write!(
                    f,
                    "Error: Invalid YAML in parameter file\n\n\
                     Could not parse: {}\n\
                     {}\n\n\
                     Fix:\n  \
                     - Verify the file is valid YAML\n  \
                     - Check for missing quotes around special characters\n  \
                     - Check for incorrect indentation\n\n\
                     Hint: Run 'tq help params' for parameter file format reference.",
                    path.display(),
                    message,
                )
            }
            ParamError::UndefinedVariable {
                variable,
                available,
            } => {
                write!(
                    f,
                    "Error: Undefined variable in template\n\n\
                     Variable '{{{{{}}}}}' is not defined.\n\n\
                     Available variables:\n  {}\n\n\
                     Fix:\n  \
                     Add '{}' to your parameter file, or\n  \
                     use '-p another-file.yaml' with the missing key defined.\n\n\
                     Hint: Run 'tq help params' for syntax reference.",
                    variable,
                    if available.is_empty() {
                        "(none - no parameters loaded)".to_string()
                    } else {
                        available.join("\n  ")
                    },
                    variable,
                )
            }
            ParamError::EnvVarNotFound { var_name } => {
                write!(
                    f,
                    "Error: Undefined environment variable in template\n\n\
                     Variable '{{{{$ENV.{}}}}}' references environment variable '{}'\n\
                     which is not set in the current environment.\n\n\
                     Fix:\n  \
                     export {}=myvalue\n  \
                     tq query -p params.yaml \"...\"",
                    var_name, var_name, var_name,
                )
            }
            ParamError::NonScalarValue { variable } => {
                write!(
                    f,
                    "Error: Variable value is not a scalar\n\n\
                     Variable '{{{{{}}}}}' resolved to a mapping or sequence, not a scalar value.\n\n\
                     Fix:\n  \
                     Use dot notation to access a specific key: {{{{{}}}.key}}}}",
                    variable, variable,
                )
            }
            ParamError::SubstitutionFailed { errors } => {
                write!(f, "{}", errors.join("\n"))
            }
            ParamError::InvalidRoot { path } => {
                write!(
                    f,
                    "Error: Invalid YAML in parameter file\n\n\
                     Could not parse: {}\n\
                     Parameter file must contain a YAML mapping at the root level\n\n\
                     Fix:\n  \
                     - Verify the file is valid YAML\n  \
                     - Ensure root is a mapping (key: value), not a list or scalar",
                    path.display(),
                )
            }
        }
    }
}

impl std::error::Error for ParamError {}

impl From<ParamError> for crate::error::TqError {
    fn from(e: ParamError) -> Self {
        crate::error::TqError::InvalidConfig(e.to_string())
    }
}

/// Result type alias for param operations
type Result<T> = std::result::Result<T, ParamError>;

/// Holds merged parameter values from one or more YAML files.
///
/// The store is a single YAML mapping that results from deep-merging
/// all loaded files (later files override earlier ones at the leaf level).
pub struct ParamStore {
    /// The merged YAML tree
    values: YamlValue,
    /// Ordered list of loaded file paths (for display/diagnostics)
    loaded_files: Vec<PathBuf>,
}

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
        let content = std::fs::read_to_string(path).map_err(|e| ParamError::FileNotFound {
            path: path.to_path_buf(),
            source: e,
        })?;

        // REQ-PARAMS-013: Empty YAML file is valid (zero variables)
        let trimmed = content.trim();
        if trimmed.is_empty() {
            self.loaded_files.push(path.to_path_buf());
            return Ok(());
        }

        let parsed: YamlValue =
            serde_yaml::from_str(&content).map_err(|e| ParamError::YamlParseError {
                path: path.to_path_buf(),
                message: e.to_string(),
            })?;

        // Validate root is a mapping (or Null for empty docs like "---")
        match &parsed {
            YamlValue::Mapping(_) => {}
            YamlValue::Null => {
                // Empty YAML document (e.g., just "---\n") is valid
                self.loaded_files.push(path.to_path_buf());
                return Ok(());
            }
            _ => {
                return Err(ParamError::InvalidRoot {
                    path: path.to_path_buf(),
                });
            }
        }

        self.values = deep_merge(self.values.clone(), parsed);
        self.loaded_files.push(path.to_path_buf());
        Ok(())
    }

    /// Load parameters from a YAML string (for testing)
    #[cfg(test)]
    pub fn load_yaml_str(&mut self, yaml: &str) -> Result<()> {
        let parsed: YamlValue =
            serde_yaml::from_str(yaml).map_err(|e| ParamError::YamlParseError {
                path: PathBuf::from("<string>"),
                message: e.to_string(),
            })?;

        match &parsed {
            YamlValue::Mapping(_) => {}
            YamlValue::Null => return Ok(()),
            _ => {
                return Err(ParamError::InvalidRoot {
                    path: PathBuf::from("<string>"),
                });
            }
        }

        self.values = deep_merge(self.values.clone(), parsed);
        Ok(())
    }

    /// Parse and insert a KEY=VALUE define string into the store (e.g. from -D / --define)
    pub fn insert_define(&mut self, key_value: &str) -> Result<()> {
        let (key_path, val_str) = match key_value.find('=') {
            Some(idx) => (&key_value[..idx], &key_value[idx + 1..]),
            None => (key_value, "true"),
        };

        let key_path = key_path.trim();
        if key_path.is_empty() {
            return Ok(());
        }

        let parsed_val: YamlValue = serde_yaml::from_str(val_str)
            .unwrap_or_else(|_| YamlValue::String(val_str.to_string()));

        let segments: Vec<&str> = key_path.split('.').collect();
        let define_tree = build_yaml_tree(&segments, parsed_val);
        self.values = deep_merge(self.values.clone(), define_tree);
        Ok(())
    }

    /// Check if SQL string contains any substitution markers
    pub fn has_variables(sql: &str) -> bool {
        VARIABLE_RE.is_match(sql)
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
        let mut found = true;

        for segment in &segments {
            match current {
                YamlValue::Mapping(map) => {
                    let key = YamlValue::String(segment.to_string());
                    if let Some(next) = map.get(&key) {
                        current = next;
                    } else {
                        found = false;
                        break;
                    }
                }
                _ => {
                    found = false;
                    break;
                }
            }
        }

        if found {
            return yaml_value_to_string(current, path);
        }

        // Fallback: check environment variable if key has no dots
        if !path.contains('.') {
            if let Ok(env_val) = std::env::var(path) {
                return Ok(env_val);
            }
        }

        Err(ParamError::UndefinedVariable {
            variable: path.to_string(),
            available: self.list_available_paths(),
        })
    }

    /// Substitute all `{{variable}}`, `{{ variable }}`, and `${variable}` markers in the given SQL text.
    ///
    /// Returns the SQL with all variables replaced, or an error if any
    /// variable cannot be resolved. This is all-or-nothing: partial
    /// substitution is never returned.
    pub fn substitute(&self, sql: &str) -> std::result::Result<String, ParamError> {
        let re = &*VARIABLE_RE;

        // First pass: collect all errors
        let mut errors: Vec<String> = Vec::new();
        for cap in re.captures_iter(sql) {
            let var_path = cap.get(1).or_else(|| cap.get(2)).unwrap().as_str();
            if let Err(e) = self.resolve(var_path) {
                errors.push(format!("{}", e));
            }
        }

        if !errors.is_empty() {
            return Err(ParamError::SubstitutionFailed { errors });
        }

        // Second pass: perform substitution (all variables known to resolve)
        let mut result = String::with_capacity(sql.len());
        let mut last_end = 0;

        for cap in re.captures_iter(sql) {
            let full_match = cap.get(0).unwrap();
            let var_path = cap.get(1).or_else(|| cap.get(2)).unwrap().as_str();

            result.push_str(&sql[last_end..full_match.start()]);
            // Safe to unwrap: we verified all variables resolve in first pass
            let value = self.resolve(var_path).unwrap();
            result.push_str(&value);
            last_end = full_match.end();
        }

        result.push_str(&sql[last_end..]);
        Ok(result)
    }

    /// List all available dot-separated paths (for error messages and /params show)
    pub fn list_available_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        collect_paths(&self.values, String::new(), &mut paths);
        paths.sort();
        paths
    }

    /// List variables with their values and source file info for /params show
    pub fn list_variables(&self) -> Vec<(String, String)> {
        let paths = self.list_available_paths();
        paths
            .into_iter()
            .filter_map(|path| {
                self.resolve(&path).ok().map(|value| (path, value))
            })
            .collect()
    }
}

impl Default for ParamStore {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ParamStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParamStore")
            .field("loaded_files", &self.loaded_files)
            .field("is_empty", &self.is_empty())
            .finish()
    }
}

/// Recursively build a YAML tree mapping from dot-separated key segments.
fn build_yaml_tree(segments: &[&str], value: YamlValue) -> YamlValue {
    if segments.is_empty() {
        return value;
    }
    let mut map = serde_yaml::Mapping::new();
    map.insert(
        YamlValue::String(segments[0].to_string()),
        build_yaml_tree(&segments[1..], value),
    );
    YamlValue::Mapping(map)
}

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

/// Convert a YAML leaf value to its string representation for SQL substitution.
fn yaml_value_to_string(value: &YamlValue, variable_path: &str) -> Result<String> {
    match value {
        YamlValue::String(s) => Ok(s.clone()),
        YamlValue::Number(n) => Ok(n.to_string()),
        YamlValue::Bool(b) => Ok(b.to_string()),
        YamlValue::Null => Ok("NULL".to_string()),
        YamlValue::Mapping(_) | YamlValue::Sequence(_) | YamlValue::Tagged(_) => {
            Err(ParamError::NonScalarValue {
                variable: variable_path.to_string(),
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

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // -------------------------------------------------------------------------
    // ParamStore basics
    // -------------------------------------------------------------------------

    #[test]
    fn test_new_store_is_empty() {
        let store = ParamStore::new();
        assert!(store.is_empty());
        assert!(store.loaded_files().is_empty());
    }

    #[test]
    fn test_default_store_is_empty() {
        let store = ParamStore::default();
        assert!(store.is_empty());
    }

    #[test]
    fn test_load_yaml_str_simple() {
        let mut store = ParamStore::new();
        store.load_yaml_str("database: MY_DB\ntable: orders").unwrap();
        assert!(!store.is_empty());
    }

    #[test]
    fn test_load_yaml_str_empty() {
        let mut store = ParamStore::new();
        store.load_yaml_str("").unwrap();
        assert!(store.is_empty());
    }

    #[test]
    fn test_load_yaml_str_null_doc() {
        let mut store = ParamStore::new();
        store.load_yaml_str("---").unwrap();
        assert!(store.is_empty());
    }

    #[test]
    fn test_load_yaml_str_non_mapping_root() {
        let mut store = ParamStore::new();
        let result = store.load_yaml_str("- item1\n- item2");
        assert!(result.is_err());
    }

    #[test]
    fn test_clear() {
        let mut store = ParamStore::new();
        store.load_yaml_str("key: value").unwrap();
        assert!(!store.is_empty());
        store.clear();
        assert!(store.is_empty());
        assert!(store.loaded_files().is_empty());
    }

    // -------------------------------------------------------------------------
    // Variable resolution
    // -------------------------------------------------------------------------

    #[test]
    fn test_resolve_simple_key() {
        let mut store = ParamStore::new();
        store.load_yaml_str("database: MY_DB").unwrap();
        assert_eq!(store.resolve("database").unwrap(), "MY_DB");
    }

    #[test]
    fn test_resolve_nested_key() {
        let mut store = ParamStore::new();
        store
            .load_yaml_str("target:\n  db: DEV\n  schema: staging")
            .unwrap();
        assert_eq!(store.resolve("target.db").unwrap(), "DEV");
        assert_eq!(store.resolve("target.schema").unwrap(), "staging");
    }

    #[test]
    fn test_resolve_deep_nested() {
        let mut store = ParamStore::new();
        store
            .load_yaml_str("a:\n  b:\n    c: deep_value")
            .unwrap();
        assert_eq!(store.resolve("a.b.c").unwrap(), "deep_value");
    }

    #[test]
    fn test_resolve_integer_value() {
        let mut store = ParamStore::new();
        store.load_yaml_str("count: 100").unwrap();
        assert_eq!(store.resolve("count").unwrap(), "100");
    }

    #[test]
    fn test_resolve_float_value() {
        let mut store = ParamStore::new();
        store.load_yaml_str("threshold: 99.5").unwrap();
        assert_eq!(store.resolve("threshold").unwrap(), "99.5");
    }

    #[test]
    fn test_resolve_boolean_value() {
        let mut store = ParamStore::new();
        store.load_yaml_str("active: true").unwrap();
        assert_eq!(store.resolve("active").unwrap(), "true");
    }

    #[test]
    fn test_resolve_null_value() {
        let mut store = ParamStore::new();
        store.load_yaml_str("filter: ~").unwrap();
        assert_eq!(store.resolve("filter").unwrap(), "NULL");
    }

    #[test]
    fn test_resolve_undefined_variable() {
        let mut store = ParamStore::new();
        store.load_yaml_str("database: MY_DB").unwrap();
        let err = store.resolve("schema").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("schema"));
        assert!(msg.contains("database")); // lists available variables
    }

    #[test]
    fn test_resolve_non_scalar_mapping() {
        let mut store = ParamStore::new();
        store
            .load_yaml_str("target:\n  db: DEV\n  schema: staging")
            .unwrap();
        let err = store.resolve("target").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("not a scalar"));
    }

    #[test]
    fn test_resolve_non_scalar_sequence() {
        let mut store = ParamStore::new();
        store.load_yaml_str("items:\n  - a\n  - b").unwrap();
        let err = store.resolve("items").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("not a scalar"));
    }

    // -------------------------------------------------------------------------
    // Environment variable resolution
    // -------------------------------------------------------------------------

    #[test]
    fn test_resolve_env_var() {
        unsafe {
            std::env::set_var("TQ_TEST_PARAMS_DB", "ENV_DB");
        }
        let store = ParamStore::new();
        let result = store.resolve("$ENV.TQ_TEST_PARAMS_DB").unwrap();
        assert_eq!(result, "ENV_DB");
        unsafe {
            std::env::remove_var("TQ_TEST_PARAMS_DB");
        }
    }

    #[test]
    fn test_resolve_env_var_not_set() {
        let store = ParamStore::new();
        let err = store
            .resolve("$ENV.TQ_NONEXISTENT_VAR_12345")
            .unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("TQ_NONEXISTENT_VAR_12345"));
        assert!(msg.contains("not set"));
    }

    // -------------------------------------------------------------------------
    // SQL substitution
    // -------------------------------------------------------------------------

    #[test]
    fn test_substitute_simple() {
        let mut store = ParamStore::new();
        store
            .load_yaml_str("database: MY_DB\ntable: orders")
            .unwrap();
        let result = store
            .substitute("SELECT * FROM {{database}}.{{table}}")
            .unwrap();
        assert_eq!(result, "SELECT * FROM MY_DB.orders");
    }

    #[test]
    fn test_substitute_nested() {
        let mut store = ParamStore::new();
        store
            .load_yaml_str("target:\n  db: DEV\n  schema: staging")
            .unwrap();
        let result = store
            .substitute("SELECT * FROM {{target.db}}.{{target.schema}}.t1")
            .unwrap();
        assert_eq!(result, "SELECT * FROM DEV.staging.t1");
    }

    #[test]
    fn test_substitute_env_var() {
        unsafe {
            std::env::set_var("TQ_TEST_PARAMS_SUB", "ENV_DB");
        }
        let store = ParamStore::new();
        let result = store
            .substitute("SELECT * FROM {{$ENV.TQ_TEST_PARAMS_SUB}}.t1")
            .unwrap();
        assert_eq!(result, "SELECT * FROM ENV_DB.t1");
        unsafe {
            std::env::remove_var("TQ_TEST_PARAMS_SUB");
        }
    }

    #[test]
    fn test_substitute_no_variables_passthrough() {
        let store = ParamStore::new();
        let result = store.substitute("SELECT 1").unwrap();
        assert_eq!(result, "SELECT 1");
    }

    #[test]
    fn test_substitute_undefined_variable_error() {
        let mut store = ParamStore::new();
        store.load_yaml_str("database: MY_DB").unwrap();
        let err = store
            .substitute("SELECT * FROM {{schema}}.t1")
            .unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("schema"));
    }

    #[test]
    fn test_substitute_multiple_undefined_variables() {
        let mut store = ParamStore::new();
        store.load_yaml_str("database: MY_DB").unwrap();
        let err = store
            .substitute("SELECT * FROM {{schema}}.{{table}}")
            .unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("schema"));
        assert!(msg.contains("table"));
    }

    #[test]
    fn test_substitute_variable_at_start() {
        let mut store = ParamStore::new();
        store.load_yaml_str("stmt: SELECT 1").unwrap();
        let result = store.substitute("{{stmt}}").unwrap();
        assert_eq!(result, "SELECT 1");
    }

    #[test]
    fn test_substitute_variable_at_end() {
        let mut store = ParamStore::new();
        store.load_yaml_str("limit: 100").unwrap();
        let result = store
            .substitute("SELECT * FROM t SAMPLE {{limit}}")
            .unwrap();
        assert_eq!(result, "SELECT * FROM t SAMPLE 100");
    }

    #[test]
    fn test_substitute_adjacent_variables() {
        let mut store = ParamStore::new();
        store.load_yaml_str("a: hello\nb: world").unwrap();
        let result = store.substitute("{{a}}{{b}}").unwrap();
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_substitute_empty_sql() {
        let store = ParamStore::new();
        let result = store.substitute("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_substitute_preserves_non_matching_braces() {
        let store = ParamStore::new();
        // Single braces should not be matched
        let result = store.substitute("SELECT {col} FROM t").unwrap();
        assert_eq!(result, "SELECT {col} FROM t");
    }

    #[test]
    fn test_substitute_with_quoting() {
        let mut store = ParamStore::new();
        store.load_yaml_str("dept: Sales").unwrap();
        let result = store
            .substitute("SELECT * FROM emp WHERE dept = '{{dept}}'")
            .unwrap();
        assert_eq!(result, "SELECT * FROM emp WHERE dept = 'Sales'");
    }

    // -------------------------------------------------------------------------
    // Deep merge
    // -------------------------------------------------------------------------

    #[test]
    fn test_deep_merge_override() {
        let base: YamlValue = serde_yaml::from_str("a:\n  b: 1\n  c: 2").unwrap();
        let overlay: YamlValue = serde_yaml::from_str("a:\n  b: 99").unwrap();
        let merged = deep_merge(base, overlay);

        // a.b should be 99, a.c should be 2
        let map = merged.as_mapping().unwrap();
        let a = map
            .get(YamlValue::String("a".to_string()))
            .unwrap()
            .as_mapping()
            .unwrap();
        let b = a.get(YamlValue::String("b".to_string())).unwrap();
        assert_eq!(b.as_u64(), Some(99));
        let c = a.get(YamlValue::String("c".to_string())).unwrap();
        assert_eq!(c.as_u64(), Some(2));
    }

    #[test]
    fn test_deep_merge_disjoint() {
        let base: YamlValue = serde_yaml::from_str("x: 1").unwrap();
        let overlay: YamlValue = serde_yaml::from_str("y: 2").unwrap();
        let merged = deep_merge(base, overlay);

        let map = merged.as_mapping().unwrap();
        assert!(map.get(YamlValue::String("x".to_string())).is_some());
        assert!(map.get(YamlValue::String("y".to_string())).is_some());
    }

    #[test]
    fn test_deep_merge_scalar_replaces_mapping() {
        let base: YamlValue = serde_yaml::from_str("a:\n  b: 1").unwrap();
        let overlay: YamlValue = serde_yaml::from_str("a: scalar").unwrap();
        let merged = deep_merge(base, overlay);

        let map = merged.as_mapping().unwrap();
        let a = map.get(YamlValue::String("a".to_string())).unwrap();
        assert_eq!(a.as_str(), Some("scalar"));
    }

    #[test]
    fn test_deep_merge_nested_preserves_unmodified() {
        let mut store = ParamStore::new();
        store
            .load_yaml_str("database: STAGING\nschema: HR\nfilters:\n  region: GLOBAL\n  active: true")
            .unwrap();
        store
            .load_yaml_str("database: PRODUCTION\nfilters:\n  region: EMEA")
            .unwrap();

        assert_eq!(store.resolve("database").unwrap(), "PRODUCTION");
        assert_eq!(store.resolve("schema").unwrap(), "HR");
        assert_eq!(store.resolve("filters.region").unwrap(), "EMEA");
        assert_eq!(store.resolve("filters.active").unwrap(), "true");
    }

    // -------------------------------------------------------------------------
    // File loading
    // -------------------------------------------------------------------------

    #[test]
    fn test_load_file_not_found() {
        let mut store = ParamStore::new();
        let result = store.load_file(Path::new("/nonexistent/params.yaml"));
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("Parameter file not found"));
    }

    #[test]
    fn test_load_file_valid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("params.yaml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "database: TEST_DB").unwrap();
        writeln!(f, "schema: public").unwrap();

        let mut store = ParamStore::new();
        store.load_file(&path).unwrap();

        assert_eq!(store.resolve("database").unwrap(), "TEST_DB");
        assert_eq!(store.resolve("schema").unwrap(), "public");
        assert_eq!(store.loaded_files().len(), 1);
    }

    #[test]
    fn test_load_file_invalid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.yaml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "key: [invalid yaml").unwrap();

        let mut store = ParamStore::new();
        let result = store.load_file(&path);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("Invalid YAML"));
    }

    #[test]
    fn test_load_file_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.yaml");
        std::fs::File::create(&path).unwrap();

        let mut store = ParamStore::new();
        store.load_file(&path).unwrap();
        assert!(store.is_empty());
        assert_eq!(store.loaded_files().len(), 1);
    }

    #[test]
    fn test_load_multiple_files_merge() {
        let dir = tempfile::tempdir().unwrap();

        let base_path = dir.path().join("base.yaml");
        let mut f = std::fs::File::create(&base_path).unwrap();
        writeln!(f, "database: STAGING").unwrap();
        writeln!(f, "limit: 10").unwrap();

        let override_path = dir.path().join("override.yaml");
        let mut f = std::fs::File::create(&override_path).unwrap();
        writeln!(f, "database: PRODUCTION").unwrap();

        let mut store = ParamStore::new();
        store.load_file(&base_path).unwrap();
        store.load_file(&override_path).unwrap();

        assert_eq!(store.resolve("database").unwrap(), "PRODUCTION");
        assert_eq!(store.resolve("limit").unwrap(), "10");
        assert_eq!(store.loaded_files().len(), 2);
    }

    // -------------------------------------------------------------------------
    // List paths
    // -------------------------------------------------------------------------

    #[test]
    fn test_list_available_paths() {
        let mut store = ParamStore::new();
        store
            .load_yaml_str("database: MY_DB\ntarget:\n  schema: HR\n  table: employees")
            .unwrap();

        let paths = store.list_available_paths();
        assert!(paths.contains(&"database".to_string()));
        assert!(paths.contains(&"target.schema".to_string()));
        assert!(paths.contains(&"target.table".to_string()));
    }

    #[test]
    fn test_list_available_paths_empty() {
        let store = ParamStore::new();
        assert!(store.list_available_paths().is_empty());
    }

    #[test]
    fn test_list_variables() {
        let mut store = ParamStore::new();
        store.load_yaml_str("db: MY_DB\nlimit: 100").unwrap();

        let vars = store.list_variables();
        assert_eq!(vars.len(), 2);
        assert!(vars.iter().any(|(p, v)| p == "db" && v == "MY_DB"));
        assert!(vars.iter().any(|(p, v)| p == "limit" && v == "100"));
    }

    // -------------------------------------------------------------------------
    // Debug trait
    // -------------------------------------------------------------------------

    #[test]
    fn test_debug_impl() {
        let store = ParamStore::new();
        let debug = format!("{:?}", store);
        assert!(debug.contains("ParamStore"));
    }

    // -------------------------------------------------------------------------
    // Error display
    // -------------------------------------------------------------------------

    #[test]
    fn test_error_display_file_not_found() {
        let err = ParamError::FileNotFound {
            path: PathBuf::from("params.yaml"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "No such file"),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("params.yaml"));
        assert!(msg.contains("Parameter file not found"));
    }

    #[test]
    fn test_error_display_yaml_parse() {
        let err = ParamError::YamlParseError {
            path: PathBuf::from("bad.yaml"),
            message: "invalid mapping".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("bad.yaml"));
        assert!(msg.contains("Invalid YAML"));
    }

    #[test]
    fn test_error_display_env_var_not_found() {
        let err = ParamError::EnvVarNotFound {
            var_name: "MY_VAR".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("MY_VAR"));
        assert!(msg.contains("not set"));
    }

    #[test]
    fn test_error_conversion_to_tq_error() {
        let err = ParamError::EnvVarNotFound {
            var_name: "TEST".to_string(),
        };
        let tq_err: crate::error::TqError = err.into();
        assert_eq!(tq_err.exit_code(), 2); // Usage error
    }

    // -------------------------------------------------------------------------
    // Sprint 78 enhancements: Defines, Jinja2 whitespace, Shell syntax, Env fallback
    // -------------------------------------------------------------------------

    #[test]
    fn test_insert_define_scalar_and_nested() {
        let mut store = ParamStore::new();
        store.insert_define("table=employees").unwrap();
        store.insert_define("target.db=PROD").unwrap();
        store.insert_define("limit=100").unwrap();

        assert_eq!(store.resolve("table").unwrap(), "employees");
        assert_eq!(store.resolve("target.db").unwrap(), "PROD");
        assert_eq!(store.resolve("limit").unwrap(), "100");
    }

    #[test]
    fn test_insert_define_overrides_yaml() {
        let mut store = ParamStore::new();
        store.load_yaml_str("table: customers\nregion: APAC").unwrap();
        store.insert_define("table=employees").unwrap();

        assert_eq!(store.resolve("table").unwrap(), "employees");
        assert_eq!(store.resolve("region").unwrap(), "APAC");
    }

    #[test]
    fn test_substitute_whitespace_in_braces() {
        let mut store = ParamStore::new();
        store.insert_define("db=DEV").unwrap();
        store.insert_define("schema=HR").unwrap();

        let sql = "SELECT * FROM {{ db }}.{{   schema   }}.emp";
        let res = store.substitute(sql).unwrap();
        assert_eq!(res, "SELECT * FROM DEV.HR.emp");
    }

    #[test]
    fn test_substitute_shell_syntax() {
        let mut store = ParamStore::new();
        store.insert_define("db=PROD").unwrap();

        let sql = "SELECT * FROM ${db}.employees WHERE id = ${ID}";
        unsafe {
            std::env::set_var("ID", "42");
        }
        let res = store.substitute(sql).unwrap();
        assert_eq!(res, "SELECT * FROM PROD.employees WHERE id = 42");
        unsafe {
            std::env::remove_var("ID");
        }
    }

    #[test]
    fn test_implicit_env_fallback() {
        let store = ParamStore::new();
        unsafe {
            std::env::set_var("TQ_TEST_ENV_VAR", "FINANCE");
        }
        let res = store.substitute("SELECT * FROM {{TQ_TEST_ENV_VAR}}").unwrap();
        assert_eq!(res, "SELECT * FROM FINANCE");
        unsafe {
            std::env::remove_var("TQ_TEST_ENV_VAR");
        }
    }

    #[test]
    fn test_has_variables() {
        assert!(ParamStore::has_variables("SELECT * FROM {{table}}"));
        assert!(ParamStore::has_variables("SELECT * FROM {{ table }}"));
        assert!(ParamStore::has_variables("SELECT * FROM ${table}"));
        assert!(!ParamStore::has_variables("SELECT * FROM table"));
    }
}
