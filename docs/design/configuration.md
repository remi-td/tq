# Configuration Design

This document describes the technical implementation of tq's configuration management system.

**Related Specification**: `docs/specifications/configuration.md` (user-facing requirements)

## Overview

tq uses a layered configuration system built on the `figment` crate. Configuration is loaded from multiple sources with a clear precedence hierarchy, enabling both user-specific preferences and project-level team-shared settings.

## Architecture

```
Configuration Loading Flow:

┌─────────────────────────────────────────────────────────────────────┐
│                     Config::load()                                   │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
           ┌────────────────┼────────────────┐
           │                │                │
           ▼                ▼                ▼
    ┌────────────┐   ┌────────────┐   ┌────────────┐
    │ Built-in   │   │ File-based │   │ Environment│
    │ Defaults   │   │ Sources    │   │ Variables  │
    └─────┬──────┘   └─────┬──────┘   └─────┬──────┘
          │                │                │
          │         ┌──────┴──────┐         │
          │         │             │         │
          │    ┌────▼────┐  ┌────▼────┐    │
          │    │ System  │  │ User    │    │
          │    │ Config  │  │ Config  │    │
          │    └────┬────┘  └────┬────┘    │
          │         │            │         │
          │         │       ┌────▼────┐    │
          │         │       │ Project │    │
          │         │       │ Config  │    │
          │         │       └────┬────┘    │
          │         │            │         │
          └─────────┴────────────┴─────────┘
                            │
                    ┌───────▼───────┐
                    │ figment Merge │
                    │ (Later wins)  │
                    └───────┬───────┘
                            │
                    ┌───────▼───────┐
                    │ CLI Arguments │
                    │ (Applied at   │
                    │  runtime)     │
                    └───────┬───────┘
                            │
                    ┌───────▼───────┐
                    │ Final Config  │
                    └───────────────┘
```

## Precedence Hierarchy

Configuration sources are loaded in order (later overrides earlier):

1. **Built-in defaults** - Hardcoded sensible defaults (`Config::default()`)
2. **System config** - `/etc/tq/config.toml` (administrator settings)
3. **User config** - `~/.tq/config.toml` (personal preferences)
4. **Project config** - `.tq.toml` (team-shared settings)
5. **Environment variables** - `TQ_*` variables
6. **CLI arguments** - Command-line flags (applied at runtime, not in figment)

**Key Design Decision**: Project config overrides user config. This enables teams to share connection profiles (dev, staging, prod) that work consistently for all team members, while users can still customize behavior via CLI flags or environment variables.

## Module Structure

```
src/
├── config.rs              # Configuration types and loading
│   ├── Config             # Complete application configuration
│   ├── ConnectionSettings # Connection-related settings
│   ├── OutputSettings     # Output preferences
│   ├── ReplSettings       # REPL preferences
│   ├── Config::load()     # Load from all sources
│   ├── find_project_config() # Walk up tree for .tq.toml
│   └── read_password_from_file() # Secure credential loading
├── main.rs
│   ├── build_connection_config()      # Merge config + CLI
│   └── build_connection_from_profile() # Load named profile
└── cli.rs
    └── GlobalOpts          # CLI argument definitions
```

## Config Types

### Core Config Structure

```rust
/// Complete application configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Connection settings (defaults, not a profile)
    pub connection: ConnectionSettings,

    /// Output preferences
    pub output: OutputSettings,

    /// REPL settings
    pub repl: ReplSettings,

    /// Named connection profiles
    #[serde(default)]
    pub profiles: HashMap<String, ConnectionSettings>,
}
```

### Connection Settings

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConnectionSettings {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub user: Option<String>,
    pub logmech: Option<String>,
    pub timeout: Option<String>,
    pub password_file: Option<PathBuf>,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            host: None,
            port: Some(1025),
            database: None,
            user: None,
            logmech: Some("TD2".to_string()),
            timeout: Some("30s".to_string()),
            password_file: None,
        }
    }
}
```

**Design decisions**:
- All fields are `Option<T>` to enable partial configuration and merging
- Defaults are specified in `Default` impl, not in field types
- `password_file` instead of inline password for security
- `serde(default)` allows partial TOML files

## Project Config Implementation

### Path Resolution Algorithm

```rust
/// Find project config by walking up the directory tree
pub fn find_project_config() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;

    let mut current = cwd.as_path();
    loop {
        let candidate = current.join(".tq.toml");
        if candidate.is_file() {
            return Some(candidate);
        }

        // Move to parent directory
        current = current.parent()?;
    }
}
```

**Algorithm characteristics**:
- Starts from current working directory
- Walks up until `.tq.toml` found or filesystem root reached
- Returns `None` if no project config found (graceful fallback)
- Does not follow symlinks for security (uses `is_file()` check)

**Edge cases handled**:
- No project config: Returns `None`, user config still applies
- Multiple nested projects: Nearest `.tq.toml` wins
- Permission errors: Silently continue walking up
- Filesystem root: Loop terminates when `parent()` returns `None`

### Config Loading Integration

```rust
impl Config {
    pub fn load() -> Result<Self> {
        let project_config_path = find_project_config();

        let mut figment = Figment::new()
            // Built-in defaults
            .merge(Serialized::defaults(Config::default()))
            // System config
            .merge(Toml::file("/etc/tq/config.toml"))
            // User config
            .merge(Toml::file(Self::user_config_path()));

        // Project config (if found)
        if let Some(path) = project_config_path {
            figment = figment.merge(Toml::file(path));
        }

        // Environment variables
        figment = figment.merge(Env::prefixed("TQ_").split("_").lowercase(false));

        let config: Config = figment
            .extract()
            .map_err(|e| TqError::ConfigParseError(e.to_string()))?;

        Ok(config)
    }
}
```

**Design decisions**:
- Project config merged after user config (project overrides user)
- Figment handles missing files gracefully (no error if file doesn't exist)
- Environment variables can still override project config
- Error message includes which file failed to parse

## Profile Resolution

### Profile Merging Strategy

When `--profile <name>` is specified:

```rust
fn build_connection_from_profile(
    global: &GlobalOpts,
    config: &Config,
    profile_name: &str,
    password_override: Option<String>,
) -> Result<ConnectionConfig> {
    // 1. Get profile (searches merged config - project profiles override user profiles)
    let profile = config.get_profile(profile_name).ok_or_else(|| {
        TqError::InvalidConfig(format!("Profile '{}' not found", profile_name))
    })?;

    // 2. Build connection config from profile
    let host = profile.host.clone().ok_or_else(|| {
        TqError::InvalidConfig(format!("Profile '{}' missing 'host'", profile_name))
    })?;

    // 3. Apply CLI overrides (--database, --user, etc.)
    // CLI arguments take precedence over profile values

    Ok(ConnectionConfig { /* ... */ })
}
```

**Profile precedence within merged config**:
- If `[profiles.dev]` exists in both user and project config, project wins
- Profiles only defined in user config remain accessible
- Profiles only defined in project config are team-shared

### Profiles Command Enhancement

The `tq profiles` command displays profiles with source indicators:

```rust
fn handle_profiles(config: &Config) -> Result<()> {
    // Group profiles by source for display
    println!("Available profiles:\n");

    for name in sorted_profile_names {
        let profile = config.profiles.get(name).unwrap();
        println!("  {}", name);
        println!("    Host:     {}", profile.host.as_deref().unwrap_or("<not set>"));
        // ... other fields
    }

    Ok(())
}
```

**Future enhancement**: Track profile source (user vs project) for display.

## Error Handling

### Config-Related Errors

```rust
#[derive(Error, Debug)]
pub enum TqError {
    /// Configuration file parse error
    #[error("Failed to parse configuration: {0}")]
    ConfigParseError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Failed to read file
    #[error("Failed to read file '{}': {}", .path.display(), .source)]
    FileReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
```

**User-friendly error messages**:

```
Error: Failed to parse configuration

File: /path/to/project/.tq.toml
Line 5: invalid value for 'port': expected integer

Troubleshooting:
  - Check TOML syntax at the indicated line
  - Verify value types match expected format
  - Use 'tq help config' for configuration reference
```

### Validation Points

1. **TOML syntax** - Figment reports parse errors with line numbers
2. **Type validation** - Serde validates field types during extraction
3. **Profile completeness** - Required fields checked when profile used
4. **Password file permissions** - Security check before reading

## Security Considerations

### Password File Permissions

```rust
pub fn read_password_from_file(path: &Path) -> Result<String> {
    let expanded_path = expand_home_dir(path);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&expanded_path)?.permissions().mode() & 0o777;

        // Reject if group or world readable/writable
        if mode & 0o077 != 0 {
            return Err(TqError::InvalidConfig(format!(
                "Password file '{}' has insecure permissions {:04o}. Required: 0600.",
                expanded_path.display(), mode
            )));
        }
    }

    Ok(std::fs::read_to_string(&expanded_path)?.trim().to_string())
}
```

### Project Config Security

**Considerations**:
- Project config may be committed to version control
- Should NOT contain passwords (use `password_file` referencing gitignored files)
- Can reference user-specific password files (`~/.tq/passwords/<profile>`)

**Recommended pattern**:
```toml
# .tq.toml (committed to repo)
[profiles.dev]
host = "dev.company.com"
database = "development"
user = "team_user"
# Password file is gitignored or in user's home
password_file = "~/.tq/passwords/dev"
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_find_project_config_in_current_dir() {
        // Create temp dir with .tq.toml
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(".tq.toml");
        std::fs::write(&config_path, "[profiles.test]\nhost = \"test\"").unwrap();

        // Change to temp dir and find config
        std::env::set_current_dir(temp.path()).unwrap();
        let found = find_project_config();
        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn test_find_project_config_walks_up() {
        // Create nested structure: /tmp/project/.tq.toml, /tmp/project/subdir/
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(".tq.toml");
        std::fs::write(&config_path, "").unwrap();

        let subdir = temp.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        std::env::set_current_dir(&subdir).unwrap();
        let found = find_project_config();
        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn test_find_project_config_returns_none() {
        // In a directory with no .tq.toml in any ancestor
        let temp = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let found = find_project_config();
        assert!(found.is_none());
    }

    #[test]
    fn test_project_config_overrides_user_config() {
        // Test that profiles in project config override user config
    }
}
```

### Integration Tests

```rust
#[test]
fn test_profiles_command_shows_project_profiles() {
    // Create temp dir with .tq.toml containing profiles
    // Run: tq profiles
    // Assert output includes project profiles
}

#[test]
fn test_profile_flag_uses_project_profile() {
    // Create temp dir with .tq.toml containing dev profile
    // Run: tq --profile dev ping
    // Assert connection uses project profile settings
}
```

## Code Linkage

| Component | File Path | Key Functions |
|-----------|-----------|---------------|
| Config types | `src/config.rs` | `Config`, `ConnectionSettings`, `OutputSettings` |
| Config loading | `src/config.rs` | `Config::load()`, `find_project_config()` |
| User config path | `src/config.rs` | `Config::user_config_path()` |
| Password reading | `src/config.rs` | `read_password_from_file()`, `expand_home_dir()` |
| Profile resolution | `src/main.rs` | `build_connection_from_profile()` |
| Profiles command | `src/main.rs` | `handle_profiles()` |
| CLI options | `src/cli.rs` | `GlobalOpts` |

## Implementation Notes

### Current State (Post-Sprint 35)

The project configuration system is fully implemented:

**Implemented features**:
- `find_project_config()` function walks up directory tree to find `.tq.toml`
- `Config::load()` uses traversal-found path for project config
- `Config::load_user_only()` and `Config::load_project_only()` for source tracking
- `Config::project_config_path()` exposes the found project config path
- `tq profiles` command shows profiles grouped by source with indicators
- Profile field-level source indicators (`[project]`, `[user]`, `[default]`)
- Comprehensive unit tests (12 tests covering all edge cases)
- `.tq.toml.example` documented example file in repository root

**Key implementation details**:
- Uses mutex for test isolation (prevents parallel tests from interfering)
- Canonicalizes paths for comparison (handles macOS /var symlink)
- Stops at filesystem root when walking up (graceful termination)
- Uses `is_file()` check (ignores directories named `.tq.toml`)

### Test Coverage

The implementation includes 12 unit tests:
1. `test_find_project_config_in_current_directory` - Basic case
2. `test_find_project_config_walks_up_to_parent` - Single level up
3. `test_find_project_config_walks_up_multiple_levels` - Deep nesting
4. `test_find_project_config_stops_at_first_found` - Nearest config wins
5. `test_find_project_config_returns_none_when_not_found` - No config case
6. `test_find_project_config_ignores_directory_named_tq_toml` - Directory vs file
7. `test_find_project_config_with_valid_toml_content` - Parseable content
8. `test_project_config_path_method` - Public API method
9. `test_project_config_path_returns_none_when_no_config` - No config case
10. `test_load_project_only_with_profiles` - Project-only loading
11. `test_load_project_only_returns_none_when_no_config` - No config case
12. Unicode identifier quoting test (in `sql/identifiers.rs`)

## Error Handling and Graceful Degradation

### Invalid Project Config

When project config exists but contains invalid TOML syntax, the tool should warn the user but continue operation using user config and other sources.

**Design Principles:**
- **Non-blocking**: Invalid project config should not prevent tool usage
- **Visible warning**: User should be notified of the problem via stderr
- **Graceful fallback**: Continue with user config, environment variables, and CLI args
- **Clear guidance**: Error message should explain the problem and how to fix it

#### Current Behavior

The current implementation fails hard on invalid project config:

```rust
// src/config.rs - Config::load()

let config: Config = figment
    .extract()
    .map_err(|e| TqError::ConfigParseError(e.to_string()))?;
    // ^ This exits with error if project config is invalid
```

#### Proposed Behavior

Wrap project config loading in error handling:

```rust
// src/config.rs - Config::load()

pub fn load() -> Result<Self> {
    // Find project config by walking up the directory tree
    let project_config_path = find_project_config();

    let mut figment = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("/etc/tq/config.toml"))
        .merge(Toml::file(Self::user_config_path()));

    // Project config: try to load, warn on error but continue
    if let Some(ref path) = project_config_path {
        log::debug!("Found project config: {}", path.display());

        // Try to load project config
        let project_figment = figment.clone().merge(Toml::file(path));

        // Test if it's valid by attempting extraction
        match project_figment.extract::<Config>() {
            Ok(_) => {
                // Valid project config, use it
                figment = project_figment;
            }
            Err(e) => {
                // Invalid project config - warn but continue
                eprintln!("Warning: Invalid project config at {}", path.display());
                eprintln!("  Error: {}", e);
                eprintln!("  Continuing without project config.");
                eprintln!();
                // figment remains unchanged (no project config merged)
            }
        }
    }

    // Environment variables (TQ_HOST, TQ_PORT, etc.)
    figment = figment.merge(Env::prefixed("TQ_").split("_").lowercase(false));

    let config: Config = figment
        .extract()
        .map_err(|e| TqError::ConfigParseError(e.to_string()))?;

    Ok(config)
}
```

#### Warning Format

The stderr warning should include:
1. File path (absolute)
2. Parse error details (line number, column, error message)
3. Guidance on how to proceed

**Example warning output:**

```
Warning: Invalid project config at /Users/alice/projects/analytics/.tq.toml
  Error: TOML parse error at line 8, column 15
  expected '=', found ':'

  [profiles.dev]
  host: "dev.company.com"
       ^

  Continuing without project config.
```

#### Alternative Approach: Hard Fail

An alternative design is to fail hard on invalid project config, requiring the user to fix it before proceeding.

**Rationale for Graceful Degradation:**
- **Non-blocking**: User can still use the tool with personal config
- **Team scenarios**: One team member's invalid project config shouldn't block others
- **Discoverability**: Warning is more user-friendly than hard failure
- **Consistency**: Mirrors behavior of missing project config (no error, just absent)

**When Hard Fail is Better:**
- **User config**: Personal config should fail hard (high confidence it's intentional)
- **CLI-specified config**: `--config file.toml` should fail hard (explicit intent)
- **Environment variables**: Bad env var values should fail hard (often scripted)

#### Help Text Updates

The `tq help config` text should mention project config warnings:

```
PROJECT CONFIGURATION
    tq looks for a project configuration file at:
      .tq.toml (in current directory or parents)

    Project config enables team-shared profiles and settings.

    If .tq.toml contains invalid TOML syntax, tq will:
      - Display a warning to stderr with error details
      - Continue operation using user config and other sources
      - Suggest fixing the syntax error

    To validate project config:
      cat .tq.toml | toml validate    # If toml CLI tool is installed
```

#### UX Improvements for `tq profiles`

When displaying profiles, show project config path (if found) in the header:

**Current behavior:**
```
Available profiles:

From user config (~/.tq/config.toml):
  personal
    Host:     my-home-server.local
    ...
```

**Proposed behavior:**
```
User config: /Users/alice/.tq/config.toml
Project config: /Users/alice/projects/analytics/.tq.toml

Available profiles:

From user config:
  personal
    Host:     my-home-server.local
    ...
```

**When project config is invalid:**
```
User config: /Users/alice/.tq/config.toml
Project config: /Users/alice/projects/analytics/.tq.toml (invalid - not loaded)

Available profiles:

From user config:
  personal
    Host:     my-home-server.local
    ...
```

#### Empty State Improvement

When no profiles are defined, mention project config as an option:

**Current behavior:**
```
No profiles defined.

To create a profile, add to ~/.tq/config.toml:

  [profiles.myprofile]
  host = "myhost.example.com"
  ...
```

**Proposed behavior:**
```
No profiles defined.

To create a profile, add to ~/.tq/config.toml:

  [profiles.myprofile]
  host = "myhost.example.com"
  port = 1025
  database = "mydb"
  user = "myuser"
  password_file = "~/.tq/passwords/myprofile"

Tip: Create .tq.toml in your project root for team-shared profiles
```

#### Code Changes Summary

**Files to modify:**
1. `src/config.rs` - `Config::load()` function (graceful degradation logic)
2. `src/main.rs` - `handle_profiles()` function (show project config path and status)
3. `src/help/config.txt` - Add project config warning section

**Testing strategy:**
1. Unit test: Invalid project config produces warning (capture stderr)
2. Unit test: Invalid project config still loads user config successfully
3. Integration test: `tq profiles` shows project config status
4. Manual test: Create invalid `.tq.toml` and verify warning message

#### Code Linkage

| Component | File Path | Key Functions |
|-----------|-----------|---------------|
| Config loading | `src/config.rs` | `Config::load()` |
| Warning output | `src/config.rs` | `eprintln!` for stderr |
| Profiles display | `src/main.rs` | `handle_profiles()` |
| Help text | `src/help/config.txt` | Project config section |

---

## Future Enhancements

- **Config validation command**: `tq config validate` to check syntax
- **Config initialization**: `tq init` to create `.tq.toml` interactively
- **Profile editing**: `tq profile add/edit/delete` commands
- **Config location override**: `--config` flag for explicit config file
- **Watch mode**: Reload config when files change (useful for long-running processes)
