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
4. **Project config** - `.tq.toml` (team-shared settings, walked up from CWD)
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
├── commands/
│   └── profile.rs         # Profile management commands (add/edit/delete/list)
├── main.rs
│   ├── build_connection_config()      # Merge config + CLI
│   ├── build_connection_from_profile() # Load named profile
│   └── handle_profiles()              # tq profiles / tq profile list display
└── cli.rs
    ├── GlobalOpts          # CLI argument definitions
    └── Command::Profile    # Profile subcommand tree
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

    /// Monitoring thresholds and severity colors
    pub monitoring: MonitoringSettings,
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

### Monitoring Settings

The `[monitoring]` tree carries alert thresholds, severity colors and the watch-mode refresh
interval. Its structure, defaults, validation rules and the reason validation is invoked
explicitly from `main` rather than inside `Config::load()` are documented in
`docs/design/monitoring.md`.

Two properties are worth noting here because they follow from this module's design:

- **Partial tables merge key-by-key.** `Config::load()` seeds Figment with
  `Serialized::defaults(Config::default())`, so a user config that sets only `cpu_warning`
  inherits the remaining threshold defaults with no merge code of its own. This is the same
  mechanism that lets `[connection]` be specified partially.
- **This tree is file-only.** `Config::load()` merges `Env::prefixed("TQ_").split("_")`.
  Because the monitoring keys themselves contain underscores,
  `TQ_MONITORING_THRESHOLDS_CPU_WARNING` would split into `monitoring.thresholds.cpu.warning`
  and fail to bind. Rather than adding a second env provider with different splitting rules —
  which would make precedence harder to reason about — monitoring settings are configured
  through TOML only.
- **Validation is fatal, loading is not.** `main` keeps its existing
  `Config::load().unwrap_or_else(-> default)` fallback, so an unreadable or syntactically
  broken file still degrades gracefully. It then calls `config.monitoring.validate()?`
  explicitly. A semantically invalid threshold therefore aborts with
  `TqError::MonitoringConfigError` (exit code 2) instead of silently reverting to defaults,
  which is the behaviour the requirement asks for. Validation runs on the *merged*
  configuration, so it applies identically regardless of which file a value came from
  (REQ-MON-011).

## Profile Management Commands

### Overview

Sprint 43 adds `tq profile add/edit/delete/list` commands for managing profiles in `~/.tq/config.toml` without manual file editing. These are non-interactive, flag-based operations for scriptability.

### CLI Structure (`src/cli.rs`)

The `Profile` subcommand is added to the top-level `Command` enum:

```rust
/// Manage connection profiles
///
/// Add, edit, delete, and list connection profiles stored in ~/.tq/config.toml.
Profile(ProfileArgs),
```

`ProfileArgs` uses a nested subcommand:

```rust
#[derive(Parser, Debug)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub action: ProfileAction,
}

#[derive(Subcommand, Debug)]
pub enum ProfileAction {
    /// Add a new connection profile
    Add(ProfileAddArgs),
    /// Update an existing connection profile
    Edit(ProfileEditArgs),
    /// Remove a connection profile
    Delete(ProfileDeleteArgs),
    /// List available connection profiles
    List,
}

#[derive(Parser, Debug)]
pub struct ProfileAddArgs {
    /// Profile name
    pub name: String,
    /// Teradata host
    #[arg(long)]
    pub host: String,
    /// Database port (1-65535)
    #[arg(long)]
    pub port: Option<u16>,
    /// Default database
    #[arg(long)]
    pub database: Option<String>,
    /// Username
    #[arg(long)]
    pub user: Option<String>,
    /// Authentication mechanism (TD2, LDAP, KRB5, TDNEGO)
    #[arg(long)]
    pub logmech: Option<String>,
    /// Path to password file
    #[arg(long)]
    pub password_file: Option<PathBuf>,
    /// Overwrite if profile already exists
    #[arg(long)]
    pub force: bool,
}

#[derive(Parser, Debug)]
pub struct ProfileEditArgs {
    /// Profile name
    pub name: String,
    // Same optional fields as Add, all optional
    #[arg(long)]
    pub host: Option<String>,
    // ... etc.
}

#[derive(Parser, Debug)]
pub struct ProfileDeleteArgs {
    /// Profile name
    pub name: String,
    /// Skip confirmation prompt
    #[arg(long)]
    pub force: bool,
}
```

### Implementation Module (`src/commands/profile.rs`)

Profile management lives in a dedicated module following the pattern established by other command modules.

**Public API:**

```rust
/// Add a new profile to ~/.tq/config.toml
pub fn add_profile(args: &ProfileAddArgs) -> Result<()>

/// Edit an existing profile in ~/.tq/config.toml
pub fn edit_profile(args: &ProfileEditArgs) -> Result<()>

/// Delete a profile from ~/.tq/config.toml
pub fn delete_profile(args: &ProfileDeleteArgs) -> Result<()>
```

`tq profile list` reuses the existing `handle_profiles()` logic from `src/main.rs`.

### TOML Read/Write Strategy

**Decision: Full Serde Round-Trip (not comment-preserving)**

Profile add/edit/delete operations use the following approach:

1. Read `~/.tq/config.toml` with `std::fs::read_to_string` (if exists)
2. Parse into `toml::Table` (a `HashMap<String, toml::Value>`) using `toml::from_str`
3. Mutate the in-memory table (add, update, or remove the `[profiles.<name>]` section)
4. Serialize back to TOML string using `toml::to_string_pretty`
5. Write atomically using write-to-temp-file + `std::fs::rename`

**Rationale: comment-preserving vs. clean rewrite**

The `toml_edit` crate can preserve comments and whitespace, but it adds a dependency and significant complexity. For a `~/.tq/config.toml` that is primarily machine-managed (the profile commands are the primary write path), a clean round-trip is acceptable. The trade-off is:

- **Lose**: Comments in user-hand-edited config will be stripped on first write operation
- **Gain**: Simpler code, no extra dependency, consistent output formatting

This is documented to users in the help text: "Note: profile commands reformat config.toml; hand-written comments will be removed."

**Alternative considered**: `toml_edit` crate preserves comments but adds ~200KB to binary. Given that `~/.tq/config.toml` is small and comments are uncommon in practice, the simpler approach is chosen.

### Table Manipulation Pattern

```rust
/// Load the user config TOML table, creating it if absent
fn load_config_table() -> Result<toml::Table> {
    let path = Config::user_config_path();
    if !path.exists() {
        return Ok(toml::Table::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| TqError::FileReadError { path: path.clone(), source: e })?;
    toml::from_str(&content)
        .map_err(|e| TqError::ConfigParseError(e.to_string()))
}

/// Save the config table to ~/.tq/config.toml atomically
fn save_config_table(table: &toml::Table) -> Result<()> {
    let path = Config::user_config_path();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| TqError::FileReadError { path: parent.to_path_buf(), source: e })?;
    }

    let content = toml::to_string_pretty(table)
        .map_err(|e| TqError::ConfigParseError(e.to_string()))?;

    // Atomic write: temp file in same directory + rename
    let tmp_path = path.with_extension("toml.tmp");
    std::fs::write(&tmp_path, &content)
        .map_err(|e| TqError::FileWriteError { path: tmp_path.clone(), source: e })?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|e| TqError::FileWriteError { path: path.clone(), source: e })?;

    Ok(())
}
```

### Profile Section Access Pattern

```rust
/// Get or create the [profiles] section
fn get_profiles_mut(table: &mut toml::Table) -> &mut toml::Table {
    table
        .entry("profiles".to_string())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()))
        .as_table_mut()
        .expect("profiles key must be a table")
}
```

### Validation

Validation happens before any file mutation:

| Input | Validation |
|-------|-----------|
| `--logmech` | Must be one of TD2, LDAP, KRB5, TDNEGO (case-insensitive) via `parse_logmech()` |
| `--port` | `u16` range enforced by clap's type system (1-65535); additionally reject 0 |
| `--host` (add) | Required for `add` (enforced by clap `long` without `Option`) |
| Profile name | Must not be empty; alphanumeric + hyphens + underscores |
| Profile exists (add) | Error unless `--force` |
| Profile missing (edit/delete) | Always error; list available profiles in message |

### Command Dispatch in `main.rs`

```rust
Command::Profile(args) => {
    return handle_profile_command(args);
}

fn handle_profile_command(args: &ProfileArgs) -> Result<()> {
    match &args.action {
        ProfileAction::Add(add_args)      => commands::profile::add_profile(add_args),
        ProfileAction::Edit(edit_args)    => commands::profile::edit_profile(edit_args),
        ProfileAction::Delete(del_args)   => commands::profile::delete_profile(del_args),
        ProfileAction::List               => {
            let config = Config::load().unwrap_or_default();
            handle_profiles(&config)
        }
    }
}
```

`Profile` is handled in the early `match` block (before database connection is established), alongside `Help` and `Profiles`.

### Output Messages

| Command | Success output (stdout) |
|---------|------------------------|
| `add` | `Profile 'dev' added successfully.` |
| `edit` | `Profile 'dev' updated successfully.` |
| `delete` | `Profile 'dev' deleted.` |
| `list` | existing `handle_profiles()` output |

Error output goes to stderr via the standard `TqError` path.

### Tab Completion for Profile Names

Clap does not provide dynamic shell completion from runtime data. The tab completion noted in AC-9 is achieved at the shell level:

- Shell completion scripts (generated by `clap_complete`) can include static completions for subcommand argument values, but dynamic profile names require a custom completion approach.
- For sprint 43 scope: the `edit` and `delete` subcommands accept a positional `name` argument. Shell completion will show the argument placeholder `<NAME>` but will not dynamically enumerate profile names.
- This is acceptable for the initial implementation. A future enhancement could add a `tq profile list --names-only` flag and source it in shell completion scripts.

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

    // 2. Build connection config from profile fields
    // 3. Apply CLI overrides (--database, --user, etc.)
    Ok(ConnectionConfig { /* ... */ })
}
```

**Profile precedence within merged config**:
- If `[profiles.dev]` exists in both user and project config, project wins
- Profiles only defined in user config remain accessible
- Profiles only defined in project config are team-shared

### Profiles Command

The `tq profiles` command and `tq profile list` alias both call `handle_profiles()` in `src/main.rs`. This function:

1. Loads user config via `Config::load_user_only()`
2. Loads project config via `Config::load_project_only()`
3. Categorises profiles by source (user-only, project-only, merged)
4. Prints with source indicators

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
    FileReadError { path: PathBuf, source: std::io::Error },

    /// Failed to write file
    #[error("Failed to write file '{}': {}", .path.display(), .source)]
    FileWriteError { path: PathBuf, source: std::io::Error },
}
```

### Profile-Specific Error Messages

| Scenario | Error text |
|----------|-----------|
| `add` and profile exists | `Profile 'dev' already exists. Use --force to overwrite.` |
| `edit`/`delete` and profile missing | `Profile 'dev' not found. Available profiles: prod, staging.` |
| Invalid `--logmech` | `Invalid logon mechanism 'X'. Must be one of: TD2, LDAP, KRB5, TDNEGO.` |
| Invalid `--port` value | Handled by clap type validation before reaching command handler |
| Config file unreadable | `Failed to read file '~/.tq/config.toml': permission denied` |
| Config file invalid TOML | `Failed to parse configuration: ...` (toml parse error) |

### Validation Points

1. **TOML syntax** - `toml::from_str` reports parse errors
2. **Type validation** - Serde validates field types during extraction
3. **Profile completeness** - Required fields checked when profile used for connection
4. **Password file permissions** - Security check before reading
5. **Profile name uniqueness** - Checked during `add` before writing

## Security Considerations

### Password File Permissions

```rust
pub fn read_password_from_file(path: &Path) -> Result<String> {
    let expanded_path = expand_home_dir(path);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&expanded_path)?.permissions().mode() & 0o777;
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

### Config Write Safety

- `~/.tq/config.toml` writes use atomic temp-file + rename to prevent partial writes
- The `profile add/edit/delete` commands only write to the user config (`~/.tq/config.toml`), never to the project config (`.tq.toml`)
- Project config is read-only from the CLI perspective; teams manage it via version control

## Testing Strategy

### Unit Tests (in `src/commands/profile.rs`)

```rust
#[test]
fn test_add_profile_creates_file_when_absent() { /* ... */ }

#[test]
fn test_add_profile_preserves_other_profiles() { /* ... */ }

#[test]
fn test_add_profile_fails_if_exists_without_force() { /* ... */ }

#[test]
fn test_add_profile_overwrites_with_force() { /* ... */ }

#[test]
fn test_edit_profile_updates_only_specified_fields() { /* ... */ }

#[test]
fn test_edit_profile_fails_if_not_found() { /* ... */ }

#[test]
fn test_delete_profile_removes_only_target() { /* ... */ }

#[test]
fn test_delete_profile_fails_if_not_found() { /* ... */ }

#[test]
fn test_add_profile_validates_logmech() { /* ... */ }

#[test]
fn test_add_profile_creates_tq_directory() { /* ... */ }
```

## Code Linkage

| Component | File Path | Key Functions |
|-----------|-----------|---------------|
| Config types | `src/config.rs` | `Config`, `ConnectionSettings` |
| Monitoring settings | `src/config.rs` | `MonitoringSettings`, `MonitoringThresholds`, `MonitoringColors`, `MonitoringSettings::validate()` |
| Severity layer | `src/commands/severity.rs` | `Severity`, `Thresholds`, `SeverityStyler`, `MonitoringContext` |
| Config loading | `src/config.rs` | `Config::load()`, `find_project_config()` |
| User config path | `src/config.rs` | `Config::user_config_path()` |
| Password reading | `src/config.rs` | `read_password_from_file()`, `expand_home_dir()` |
| Profile CRUD | `src/commands/profile.rs` | `add_profile()`, `edit_profile()`, `delete_profile()` |
| Profile CLI args | `src/cli.rs` | `ProfileArgs`, `ProfileAction`, `ProfileAddArgs`, etc. |
| Profile resolution | `src/main.rs` | `build_connection_from_profile()` |
| Profiles display | `src/main.rs` | `handle_profiles()` |
| Command dispatch | `src/main.rs` | `handle_profile_command()` |

## Future Enhancements

- **Config validation command**: `tq config validate` to check syntax
- **Comment-preserving writes**: Switch to `toml_edit` crate if user demand warrants
- **Dynamic shell completion**: `tq profile list --names-only` for shell completion sourcing
- **Profile copy/rename**: `tq profile copy <src> <dst>`
- **Profile test**: `tq profile test <name>` to validate a profile connects successfully
