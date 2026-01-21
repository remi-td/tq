# Sprint 17 Implementation Design

**Version:** 1.0.0
**Created:** 2026-01-21
**Owner:** rust-teradata-architect agent
**Status:** Design Complete

---

## Executive Summary

This document provides the detailed implementation design for Sprint 17 features. All features have been assessed as **FEASIBLE** with low-to-medium implementation complexity.

### Feasibility Assessment Summary

| Feature | Priority | Feasibility | Complexity | Risk |
|---------|----------|-------------|------------|------|
| Help Subcommands | P0 | FEASIBLE | Medium | Low |
| Security Check Ordering Fix | P0 | FEASIBLE | Low | Low |
| Password Permission Enforcement | P1 | FEASIBLE | Low | Low |
| Profile Listing Command | P1 | FEASIBLE | Medium | Low |
| Logmech Parsing Refactoring | P2 | FEASIBLE | Low | Low |

**Total Estimated Implementation Time:** 6-8 hours

---

## 1. Help Subcommands (`tq help config`, `tq help credentials`)

### 1.1 Current State

Currently, the CLI uses clap's built-in help system:
- `tq --help` shows top-level help
- `tq <command> --help` shows command-specific help
- `tq help` is not explicitly implemented as a subcommand

The help text in `src/cli.rs` references `tq help config` (line 65) but this command returns "unrecognized subcommand" because clap's default `help` subcommand only supports `tq help <subcommand>` for existing commands.

### 1.2 Implementation Approach

**Option A: Clap Subcommand Pattern (RECOMMENDED)**

Add a `Help` subcommand variant to the `Command` enum with topic-specific arguments:

```rust
// src/cli.rs
#[derive(Subcommand, Debug)]
pub enum Command {
    Ping(PingArgs),
    Query(QueryArgs),
    Repl(ReplArgs),
    /// Show detailed help on a topic
    Help(HelpArgs),
    /// List available connection profiles
    Profiles,
}

#[derive(Parser, Debug)]
pub struct HelpArgs {
    /// Help topic: config, credentials
    #[arg(value_name = "TOPIC")]
    pub topic: Option<HelpTopic>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HelpTopic {
    /// Configuration file format and usage
    Config,
    /// Password and credential management
    Credentials,
}
```

**Why Option A:**
- Follows clap's standard subcommand pattern
- Aligns with existing `Ping`, `Query`, `Repl` structure
- Provides tab completion for help topics
- Avoids custom argument parsing

### 1.3 Help Content Storage

Create a new module `src/help.rs` for help content:

```rust
// src/help.rs
//! Extended help content for tq
//!
//! This module provides detailed help text for configuration and credential topics.

/// Get help content for configuration
pub fn config_help() -> &'static str {
    include_str!("../help/config.txt")
}

/// Get help content for credentials
pub fn credentials_help() -> &'static str {
    include_str!("../help/credentials.txt")
}

/// Get general help (when no topic specified)
pub fn general_help() -> &'static str {
    "Available help topics:\n\n\
     tq help config       Configuration file format and usage\n\
     tq help credentials  Password and credential management\n\n\
     For command help, use:\n\
     tq <command> --help\n"
}
```

### 1.4 Help Content Files

Create `src/help/config.txt` and `src/help/credentials.txt` with content from `detailed-specifications/configuration.md` sections 7.8.1 and 7.8.3.

**Directory structure:**
```
src/
  help/
    config.txt
    credentials.txt
  help.rs
```

### 1.5 Command Handler

Add handler in `src/main.rs`:

```rust
fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Help(args) => {
            handle_help(args)?;
            return Ok(());
        }
        Command::Profiles => {
            handle_profiles(&config)?;
            return Ok(());
        }
        // ... existing handlers
    }
}

fn handle_help(args: HelpArgs) -> Result<()> {
    use tq::help;

    let content = match args.topic {
        Some(HelpTopic::Config) => help::config_help(),
        Some(HelpTopic::Credentials) => help::credentials_help(),
        None => help::general_help(),
    };

    println!("{}", content);
    Ok(())
}
```

### 1.6 Error Handling for Unknown Topics

Clap's `ValueEnum` derive handles unknown topics automatically:

```
$ tq help unknown
error: invalid value 'unknown' for '<TOPIC>'
  [possible values: config, credentials]
```

### 1.7 Files to Modify

| File | Changes |
|------|---------|
| `src/cli.rs` | Add `Help(HelpArgs)` and `Profiles` to `Command` enum |
| `src/main.rs` | Add `handle_help()` and `handle_profiles()` functions |
| `src/lib.rs` | Add `pub mod help;` |
| `src/help.rs` | New file - help content functions |
| `src/help/config.txt` | New file - configuration help content |
| `src/help/credentials.txt` | New file - credentials help content |

### 1.8 Testing Strategy

- Unit test: `HelpArgs` parsing with valid/invalid topics
- Unit test: Help content functions return non-empty strings
- Integration test: `tq help config` outputs expected content
- Integration test: `tq help credentials` outputs expected content
- Integration test: `tq help` shows available topics
- Integration test: `tq help invalid` shows error with valid options

---

## 2. Security Check Ordering Fix

### 2.1 Current State

In `src/main.rs`, function `read_password_if_needed()` (lines 93-109):

```rust
fn read_password_if_needed(global: &GlobalOpts) -> Result<Option<String>> {
    let Some(ref password_file) = global.password_file else {
        return Ok(None);
    };

    // BUG: File is read BEFORE permission validation
    let password = std::fs::read_to_string(password_file).map_err(|e| TqError::FileReadError {
        path: password_file.clone(),
        source: e,
    })?;

    // Permission check happens AFTER reading
    #[cfg(unix)]
    validate_password_file_permissions(password_file)?;

    Ok(Some(password.trim().to_string()))
}
```

**Security Issue:** The file content is read into memory before permissions are validated. This creates a race condition where an attacker could potentially read the password from memory even if the file has insecure permissions.

### 2.2 Correct Pattern (from config.rs)

In `src/config.rs`, function `read_password_from_file()` (lines 264-299) has the correct order:

```rust
pub fn read_password_from_file(path: &std::path::Path) -> Result<String> {
    let expanded_path = expand_home_dir(path);

    // CORRECT: Check file permissions BEFORE reading
    #[cfg(unix)]
    {
        // Permission check happens FIRST
        let metadata = std::fs::metadata(&expanded_path)?;
        let mode = metadata.permissions().mode() & 0o777;
        if mode & 0o077 != 0 {
            return Err(TqError::InvalidConfig(...));
        }
    }

    // Read AFTER permission validation
    let password = std::fs::read_to_string(&expanded_path)?;
    Ok(password.trim().to_string())
}
```

### 2.3 Implementation

Fix `read_password_if_needed()` in `src/main.rs`:

```rust
fn read_password_if_needed(global: &GlobalOpts) -> Result<Option<String>> {
    let Some(ref password_file) = global.password_file else {
        return Ok(None);
    };

    // FIXED: Validate permissions BEFORE reading file content
    #[cfg(unix)]
    validate_password_file_permissions(password_file)?;

    // Read AFTER permission validation passes
    let password = std::fs::read_to_string(password_file).map_err(|e| TqError::FileReadError {
        path: password_file.clone(),
        source: e,
    })?;

    Ok(Some(password.trim().to_string()))
}
```

### 2.4 Files to Modify

| File | Changes |
|------|---------|
| `src/main.rs` | Move `validate_password_file_permissions()` call before `read_to_string()` |

### 2.5 Testing Strategy

- Unit test: Create test that verifies permission check is called before file read
- Integration test: Verify insecure file permissions cause error before file is read
- Manual verification: Use strace/dtrace to confirm syscall order

---

## 3. Password Permission Enforcement

### 3.1 Current State

In `src/main.rs`, function `validate_password_file_permissions()` (lines 113-131):

```rust
#[cfg(unix)]
fn validate_password_file_permissions(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path)?;
    let mode = metadata.permissions().mode() & 0o777;

    if mode & 0o077 != 0 {
        // CURRENT: Warning only, does not return error
        log::warn!(
            "Password file '{}' has insecure permissions {:o}. Recommended: 0600",
            path.display(),
            mode
        );
    }

    Ok(())  // Always returns Ok
}
```

**Issue:** The function warns but allows reading insecure files. The specification requires **enforcement**, not just warning.

### 3.2 Correct Pattern (from config.rs)

In `src/config.rs`, function `read_password_from_file()` enforces permissions (lines 279-288):

```rust
// Reject if group or world readable/writable
if mode & 0o077 != 0 {
    return Err(TqError::InvalidConfig(format!(
        "Password file '{}' has insecure permissions {:04o}. Required: 0600.\n\
         Fix: chmod 0600 {}",
        expanded_path.display(),
        mode,
        expanded_path.display()
    )));
}
```

### 3.3 Implementation

Update `validate_password_file_permissions()` in `src/main.rs` to return an error instead of warning:

```rust
#[cfg(unix)]
fn validate_password_file_permissions(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path).map_err(|e| TqError::FileReadError {
        path: path.to_path_buf(),
        source: e,
    })?;

    let mode = metadata.permissions().mode() & 0o777;

    if mode & 0o077 != 0 {
        // FIXED: Return error instead of warning
        return Err(TqError::InvalidConfig(format!(
            "Password file '{}' has insecure permissions {:04o}. Required: 0600.\n\
             \n\
             Security risk: File is readable by other users on this system.\n\
             \n\
             Fix: chmod 0600 {}",
            path.display(),
            mode,
            path.display()
        )));
    }

    Ok(())
}
```

### 3.4 Breaking Change Notice

This is a **breaking change** for users with insecure password files:
- Previous behavior: Warning logged, file still used
- New behavior: Error returned, command fails

**Mitigation:** Error message includes clear fix command (`chmod 0600`).

### 3.5 Files to Modify

| File | Changes |
|------|---------|
| `src/main.rs` | Change `validate_password_file_permissions()` to return error instead of warning |

### 3.6 Testing Strategy

- Unit test: File with 0600 permissions succeeds
- Unit test: File with 0644 permissions returns error
- Unit test: File with 0666 permissions returns error
- Integration test: Verify error message includes fix command
- Integration test: Verify command fails (non-zero exit) with insecure file

---

## 4. Profile Listing Command (`tq profiles`)

### 4.1 Design Decision

**Option A: Separate `tq profiles` command (RECOMMENDED)**

Add `Profiles` as a standalone subcommand (as shown in section 1.2):

```rust
#[derive(Subcommand, Debug)]
pub enum Command {
    // ... existing commands
    /// List available connection profiles
    Profiles,
}
```

**Why Option A:**
- Simpler than `tq profile list` (no need for nested subcommands)
- Read-only operation, no need for create/update/delete subcommands yet
- Consistent with specification future work (profile management deferred to Sprint 18+)

### 4.2 Implementation

Add handler in `src/main.rs`:

```rust
fn handle_profiles(config: &Config) -> Result<()> {
    if config.profiles.is_empty() {
        println!("No profiles defined.\n");
        println!("To create a profile, add to {}:\n", Config::user_config_path().display());
        println!("  [profiles.myprofile]");
        println!("  host = \"myhost.example.com\"");
        println!("  port = 1025");
        println!("  database = \"mydb\"");
        println!("  user = \"myuser\"");
        println!("  password_file = \"~/.tq/passwords/myprofile\"");
        return Ok(());
    }

    println!("Available profiles:\n");

    // Sort profiles alphabetically for consistent output
    let mut profile_names: Vec<_> = config.profiles.keys().collect();
    profile_names.sort();

    for name in profile_names {
        let profile = config.profiles.get(name).unwrap();
        let host = profile.host.as_deref().unwrap_or("<not set>");
        let database = profile.database.as_deref().unwrap_or("<not set>");
        let user = profile.user.as_deref().unwrap_or("<not set>");

        println!("  {} ", name);
        println!("    Host:     {}", host);
        println!("    Database: {}", database);
        println!("    User:     {}", user);

        // Show logmech if not default
        if let Some(ref logmech) = profile.logmech {
            if logmech != "TD2" {
                println!("    Logmech:  {}", logmech);
            }
        }
        println!();
    }

    println!("Use: tq --profile <name> <command>");
    Ok(())
}
```

### 4.3 Output Format

**With profiles:**
```
Available profiles:

  dev
    Host:     dev.company.com
    Database: development
    User:     alice

  prod
    Host:     prod.company.com
    Database: production
    User:     alice
    Logmech:  LDAP

Use: tq --profile <name> <command>
```

**Without profiles:**
```
No profiles defined.

To create a profile, add to ~/.tq/config.toml:

  [profiles.myprofile]
  host = "myhost.example.com"
  port = 1025
  database = "mydb"
  user = "myuser"
  password_file = "~/.tq/passwords/myprofile"
```

### 4.4 Security Consideration

**Never display passwords or password file paths in profile listing.** This prevents accidental exposure of credential locations.

### 4.5 Files to Modify

| File | Changes |
|------|---------|
| `src/cli.rs` | Add `Profiles` variant to `Command` enum |
| `src/main.rs` | Add `handle_profiles()` function and match arm |

### 4.6 Testing Strategy

- Unit test: Empty profiles HashMap produces helpful message
- Unit test: Single profile displays correctly
- Unit test: Multiple profiles sorted alphabetically
- Unit test: Password/password_file fields NOT displayed
- Integration test: `tq profiles` with no config file shows setup instructions
- Integration test: `tq profiles` with profiles shows listing

---

## 5. Logmech Parsing Refactoring (P2)

### 5.1 Current State

Duplicate logmech parsing exists in:

1. `src/config.rs` (lines 235-243) - private function:
```rust
fn parse_logmech(s: &str) -> Result<LogonMechanism> {
    match s.to_uppercase().as_str() {
        "TD2" => Ok(LogonMechanism::Td2),
        "LDAP" => Ok(LogonMechanism::Ldap),
        "KRB5" => Ok(LogonMechanism::Krb5),
        "TDNEGO" => Ok(LogonMechanism::Tdnego),
        _ => Err(TqError::InvalidLogonMechanism(s.to_string())),
    }
}
```

2. `src/main.rs` (lines 238-246) - inline in `build_connection_from_profile()`:
```rust
let logmech = if let Some(ref lm) = profile.logmech {
    match lm.to_uppercase().as_str() {
        "TD2" => tq::cli::LogonMechanism::Td2,
        "LDAP" => tq::cli::LogonMechanism::Ldap,
        "KRB5" => tq::cli::LogonMechanism::Krb5,
        "TDNEGO" => tq::cli::LogonMechanism::Tdnego,
        _ => return Err(TqError::InvalidLogonMechanism(lm.clone())),
    }
} else {
    global.logmech
};
```

### 5.2 Implementation

Make `parse_logmech` public in `config.rs` and reuse in `main.rs`:

```rust
// src/config.rs
/// Parse logon mechanism from string (case-insensitive)
pub fn parse_logmech(s: &str) -> Result<LogonMechanism> {
    match s.to_uppercase().as_str() {
        "TD2" => Ok(LogonMechanism::Td2),
        "LDAP" => Ok(LogonMechanism::Ldap),
        "KRB5" => Ok(LogonMechanism::Krb5),
        "TDNEGO" => Ok(LogonMechanism::Tdnego),
        _ => Err(TqError::InvalidLogonMechanism(s.to_string())),
    }
}
```

```rust
// src/main.rs
use tq::config::parse_logmech;

// In build_connection_from_profile():
let logmech = if let Some(ref lm) = profile.logmech {
    parse_logmech(lm)?
} else {
    global.logmech
};
```

### 5.3 Files to Modify

| File | Changes |
|------|---------|
| `src/config.rs` | Change `fn parse_logmech` to `pub fn parse_logmech` |
| `src/main.rs` | Import `parse_logmech` from config, replace inline parsing |

### 5.4 Testing Strategy

- Existing unit tests in config.rs already cover `parse_logmech`
- Verify existing tests pass after making function public
- No new tests needed (behavior unchanged)

---

## 6. Implementation Order

Recommended implementation sequence:

1. **Security Check Ordering Fix** (P0, Low complexity)
   - Small, isolated change
   - No new code, just reordering
   - Unblocks permission enforcement

2. **Password Permission Enforcement** (P1, Low complexity)
   - Builds on security fix
   - Simple change from warning to error

3. **Logmech Parsing Refactoring** (P2, Low complexity)
   - Quick win for code quality
   - No behavioral changes

4. **Help Subcommands** (P0, Medium complexity)
   - Requires new files and module
   - Involves content creation

5. **Profile Listing Command** (P1, Medium complexity)
   - Depends on Config loading
   - New command handler

---

## 7. Risk Assessment

### 7.1 Low Risk Items

| Risk | Mitigation |
|------|------------|
| Help content out of sync with docs | Content sourced directly from specification |
| Breaking change for permission enforcement | Clear error message with fix command |
| Refactoring introduces bugs | Existing tests verify behavior |

### 7.2 No Blocking Risks Identified

All features can be implemented with existing codebase and dependencies.

---

## 8. Architecture Updates

### 8.1 New Module: `src/help.rs`

Purpose: Centralized help content management.

**Pattern:** Use `include_str!()` macro to embed help text at compile time from separate text files. This keeps help content maintainable and separate from code.

### 8.2 rust-architecture.md Updates Required

The following additions should be made to `rust-architecture.md`:

1. **Help Content Management Pattern** - Document `src/help/` directory structure
2. **Security Check Order** - Document requirement to validate permissions before reading files

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-21 | 1.0.0 | Initial Sprint 17 implementation design | rust-teradata-architect |
