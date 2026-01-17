# CLI Argument Parsing in Rust

## Overview

Command-line arguments are values passed after the program name, separated by spaces:
```bash
$ my-tool pattern file.txt --verbose
```

## Basic Approach: std::env::args()

The standard library provides raw argument access:

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    // args[0] is the program name
    // args[1..] are user-provided arguments

    let pattern = &args[1];
    let path = &args[2];
}
```

Run with: `cargo run -- some-pattern some-file`

**Limitations:**
- No validation
- No help text generation
- No support for flags like `--verbose`
- Manual error handling required
- Doesn't handle `--pattern="value"` syntax

## Recommended: Clap Library

### Setup

Add to `Cargo.toml`:
```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
```

### Basic Usage with Derive Macro

Define arguments as a struct:

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "my-tool")]
#[command(about = "A brief description of what the tool does", long_about = None)]
struct Cli {
    /// The pattern to search for
    pattern: String,

    /// The path to the file to read
    path: PathBuf,
}

fn main() {
    let args = Cli::parse();
    println!("Pattern: {:?}, Path: {:?}", args.pattern, args.path);
}
```

**Key points:**
- Triple-slash comments become help text
- `PathBuf` for file paths (cross-platform)
- `Cli::parse()` exits on error (only use in main)
- Automatic `--help` generation

### Adding Optional Arguments

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// The pattern to search for
    pattern: String,

    /// The path to the file to read
    path: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Optional output file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Number of threads (default: 4)
    #[arg(short = 'j', long, default_value = "4")]
    threads: usize,
}
```

**Argument attributes:**
- `#[arg(short, long)]` - Enables `-v` and `--verbose`
- `#[arg(short = 'j')]` - Custom short flag
- `#[arg(long = "num-threads")]` - Custom long flag
- `#[arg(default_value = "...")]` - Default value
- `bool` type automatically becomes a flag
- `Option<T>` makes argument optional

### Subcommands

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "my-tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for a pattern
    Search {
        /// Pattern to search for
        pattern: String,
        /// File to search in
        path: PathBuf,
    },
    /// Count lines in a file
    Count {
        /// File to count
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search { pattern, path } => {
            println!("Searching for {} in {:?}", pattern, path);
        }
        Commands::Count { path } => {
            println!("Counting lines in {:?}", path);
        }
    }
}
```

### Validation with Value Parser

```rust
use clap::Parser;

fn parse_port(s: &str) -> Result<u16, String> {
    let port: u16 = s.parse()
        .map_err(|_| format!("Invalid port: {}", s))?;

    if port < 1024 {
        return Err(format!("Port must be >= 1024, got {}", port));
    }

    Ok(port)
}

#[derive(Parser)]
struct Cli {
    /// Port to listen on (must be >= 1024)
    #[arg(short, long, value_parser = parse_port)]
    port: u16,
}
```

### Multiple Values

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Files to process
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Exclude patterns
    #[arg(short, long)]
    exclude: Vec<String>,
}
```

Usage:
```bash
$ my-tool file1.txt file2.txt -e "*.tmp" -e "*.log"
```

### Environment Variables

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// API key
    #[arg(long, env = "MY_TOOL_API_KEY")]
    api_key: String,
}
```

Reads from `MY_TOOL_API_KEY` env var if not provided on command line.

### Conflicts and Requirements

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Use quiet mode
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,

    /// Use verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Output file (required if --format is used)
    #[arg(short, long, required_if_eq("format", "json"))]
    output: Option<PathBuf>,

    /// Output format
    #[arg(short, long)]
    format: Option<String>,
}
```

## Automatic Help Generation

Clap generates help automatically:

```bash
$ my-tool --help
```

Output:
```
A brief description of what the tool does

Usage: my-tool [OPTIONS] <PATTERN> <PATH>

Arguments:
  <PATTERN>  The pattern to search for
  <PATH>     The path to the file to read

Options:
  -v, --verbose          Enable verbose output
  -o, --output <OUTPUT>  Optional output file
  -j, --threads <NUM>    Number of threads [default: 4]
  -h, --help             Print help
  -V, --version          Print version
```

## Best Practices

1. **Document thoroughly**: Use triple-slash comments for all arguments
2. **Type safety**: Use `PathBuf` for paths, custom types for domain concepts
3. **Sensible defaults**: Provide defaults where reasonable
4. **Validation**: Use value parsers for complex validation
5. **Clear names**: Use descriptive long names, memorable short flags
6. **Only parse in main()**: `Cli::parse()` exits on error; use `try_parse()` elsewhere
7. **Group related options**: Use nested structs with `#[command(flatten)]`

## Error Handling

Clap handles errors automatically when using `parse()`:
- Missing required arguments
- Invalid values
- Type parsing failures
- Validation failures

For custom error handling, use `try_parse()`:

```rust
fn main() {
    let args = match Cli::try_parse() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Use args...
}
```

## Testing Argument Parsing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn test_parsing() {
        let args = Cli::try_parse_from(&["my-tool", "pattern", "file.txt"]).unwrap();
        assert_eq!(args.pattern, "pattern");
    }
}
```

## References

- [Clap documentation](https://docs.rs/clap/)
- [Clap derive reference](https://github.com/clap-rs/clap/blob/master/examples/derive_ref/README.md)
- [Clap examples](https://github.com/clap-rs/clap/tree/master/examples)
