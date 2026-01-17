# Error Handling in Rust CLI Applications

## Core Concepts

Rust uses explicit error handling through the `Result<T, E>` type rather than exceptions. Functions that can fail return `Result`, which is an enum with two variants:
- `Ok(value)` - Success case containing the result
- `Err(error)` - Failure case containing error information

## Basic Error Handling Approaches

### 1. Pattern Matching

Explicitly handle both success and failure cases:

```rust
use std::fs;

fn main() {
    let result = fs::read_to_string("config.txt");

    match result {
        Ok(content) => {
            println!("File content: {}", content);
        }
        Err(error) => {
            eprintln!("Error reading file: {}", error);
            std::process::exit(1);
        }
    }
}
```

### 2. Unwrapping (For Unrecoverable Errors)

Use `.unwrap()` when failure means the program cannot continue:

```rust
fn main() {
    // Panics with error message if file doesn't exist
    let content = std::fs::read_to_string("config.txt").unwrap();
    println!("Content: {}", content);
}
```

**When to use:**
- During prototyping
- When missing resource is genuinely fatal
- In tests (use `unwrap()` liberally in test code)

**When NOT to use:**
- In library code
- When you want to provide helpful error messages
- When the caller might want to handle the error

### 3. Expect (Unwrap with Custom Message)

Like `unwrap()` but with a custom panic message:

```rust
fn main() {
    let content = std::fs::read_to_string("config.txt")
        .expect("Failed to read config.txt - make sure it exists");
}
```

## Error Propagation

### Question Mark Operator (?)

The `?` operator propagates errors up the call stack:

```rust
use std::fs;
use std::io;

fn read_config() -> Result<String, io::Error> {
    let content = fs::read_to_string("config.txt")?;
    // ? automatically returns Err if the operation failed
    // Otherwise, assigns the Ok value to content

    Ok(content)
}

fn main() -> Result<(), io::Error> {
    let config = read_config()?;
    println!("Config: {}", config);
    Ok(())
}
```

**How it works:**
- If `Result` is `Ok(value)`, extracts `value` and continues
- If `Result` is `Err(error)`, returns the error immediately
- Automatically converts error types using `From` trait

### Returning Errors from Main

Main can return `Result` to handle errors gracefully:

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let content = std::fs::read_to_string("config.txt")?;
    println!("Content: {}", content);
    Ok(())
}
```

**Exit codes:**
- Returns 0 if `main` returns `Ok(())`
- Returns non-zero if `main` returns `Err(_)`

## Adding Context to Errors

### Problem: Generic Error Messages

Default error messages lack context:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("config.txt")?;
    Ok(())
}
```

Error output:
```
Error: No such file or directory (os error 2)
```

User doesn't know which file caused the error.

### Solution: Anyhow Crate

Add to `Cargo.toml`:
```toml
[dependencies]
anyhow = "1.0"
```

Use `Context` trait to add descriptive information:

```rust
use anyhow::{Context, Result};

fn read_config() -> Result<String> {
    let content = std::fs::read_to_string("config.txt")
        .context("Failed to read config.txt")?;
    Ok(content)
}

fn main() -> Result<()> {
    let config = read_config()
        .context("Unable to load configuration")?;

    let port: u16 = config.trim().parse()
        .context("Config does not contain a valid port number")?;

    println!("Port: {}", port);
    Ok(())
}
```

Error output with context:
```
Error: Unable to load configuration

Caused by:
    0: Failed to read config.txt
    1: No such file or directory (os error 2)
```

### Context Methods

```rust
use anyhow::{Context, Result};

fn process() -> Result<()> {
    // Add static context
    let data = read_data()
        .context("Failed to read data")?;

    // Add dynamic context with with_context
    let value = parse_value(&data)
        .with_context(|| format!("Failed to parse value from: {}", data))?;

    Ok(())
}
```

**Use `context`** for static strings (more efficient)
**Use `with_context`** for dynamic strings (evaluated only on error)

## Custom Error Types

For libraries, define custom error types using the `thiserror` crate:

```toml
[dependencies]
thiserror = "1.0"
```

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found at {path}")]
    NotFound { path: String },

    #[error("Invalid config format")]
    InvalidFormat,

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
}

fn read_config(path: &str) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)
        .map_err(|_| ConfigError::NotFound { path: path.to_string() })?;

    // Parse content...

    Ok(config)
}
```

**Benefits:**
- Type-safe error handling
- Clear error variants
- Automatic `Display` implementation
- Automatic error conversions with `#[from]`

## CLI Error Reporting Best Practices

### 1. Use stderr for Errors

```rust
eprintln!("Error: {}", error);
```

This separates error output from regular output, allowing:
```bash
$ my-tool > output.txt        # Errors still shown in terminal
$ my-tool 2> errors.txt       # Errors redirected separately
```

### 2. Provide Actionable Error Messages

Bad:
```
Error: File not found
```

Good:
```
Error: Config file not found at '/etc/myapp/config.toml'
Try creating it with: myapp --init-config
```

### 3. Exit with Appropriate Exit Codes

```rust
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:?}", e);
        process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    // Main logic here
    Ok(())
}
```

Common exit codes:
- `0` - Success
- `1` - General error
- `2` - Misuse of command
- `64-78` - BSD-style exit codes (see `sysexits.h`)

### 4. Show Error Chains

Anyhow shows error chains by default with `{:?}`:

```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:?}", e);  // Shows full chain
        std::process::exit(1);
    }
}
```

For more control:

```rust
use anyhow::Result;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);

        for cause in e.chain().skip(1) {
            eprintln!("Caused by: {}", cause);
        }

        std::process::exit(1);
    }
}
```

## Error Handling Patterns

### Pattern 1: Fail Fast in Main

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;

    let data = fetch_data(&config)
        .context("Failed to fetch data")?;

    process_data(data)?;

    Ok(())
}
```

### Pattern 2: Try Multiple Approaches

```rust
use anyhow::{Context, Result};

fn find_config() -> Result<String> {
    // Try user config first
    if let Ok(content) = std::fs::read_to_string("~/.myapp/config.toml") {
        return Ok(content);
    }

    // Fall back to system config
    std::fs::read_to_string("/etc/myapp/config.toml")
        .context("Config not found in ~/.myapp/ or /etc/myapp/")
}
```

### Pattern 3: Collect Multiple Errors

```rust
use anyhow::{Context, Result};

fn validate_files(paths: &[PathBuf]) -> Result<()> {
    let errors: Vec<_> = paths
        .iter()
        .filter_map(|path| {
            std::fs::metadata(path)
                .with_context(|| format!("Cannot access: {:?}", path))
                .err()
        })
        .collect();

    if !errors.is_empty() {
        eprintln!("Found {} errors:", errors.len());
        for error in errors {
            eprintln!("  - {:?}", error);
        }
        anyhow::bail!("Validation failed");
    }

    Ok(())
}
```

## Testing Error Handling

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_file() {
        let result = read_config("nonexistent.txt");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("nonexistent.txt"));
    }

    #[test]
    #[should_panic(expected = "config.txt")]
    fn test_panic_on_missing_config() {
        std::fs::read_to_string("config.txt").unwrap();
    }
}
```

## Summary

**For applications:** Use `anyhow`
- Simple `Result<T>` with `anyhow::Result<T>`
- Add context with `.context()`
- Great for CLI tools

**For libraries:** Use `thiserror`
- Define custom error types
- Type-safe error handling
- Allows callers to match on specific errors

**General rules:**
- Use `?` to propagate errors
- Add context at every level
- Print to stderr with `eprintln!`
- Provide actionable error messages
- Return proper exit codes
