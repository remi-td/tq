# Output Formatting in Rust CLI Applications

## Basic Printing

### println! Macro

Standard output for human-readable text:

```rust
println!("Hello, World!");
println!("The answer is {}", 42);
println!("Name: {}, Age: {}", name, age);
```

### Format String Syntax

```rust
let x = 42;
let name = "Alice";

// Basic placeholder
println!("Value: {}", x);

// Multiple values
println!("x = {}, name = {}", x, name);

// Positional arguments
println!("{0} {1} {0}", "a", "b");  // Outputs: a b a

// Named arguments
println!("{name} is {age} years old", name = "Bob", age = 30);
```

## Debug vs Display Output

### Display ({}) - User-Friendly

For end-user output. Requires implementing `Display` trait:

```rust
use std::fmt;

struct Point { x: i32, y: i32 }

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn main() {
    let p = Point { x: 10, y: 20 };
    println!("Point: {}", p);  // Outputs: Point: (10, 20)
}
```

### Debug ({:?}) - Developer-Oriented

For debugging. Use `#[derive(Debug)]` for automatic implementation:

```rust
#[derive(Debug)]
struct Config {
    host: String,
    port: u16,
}

fn main() {
    let config = Config {
        host: "localhost".to_string(),
        port: 8080,
    };

    println!("Config: {:?}", config);
    // Outputs: Config { host: "localhost", port: 8080 }

    // Pretty-print with {:#?}
    println!("Config: {:#?}", config);
    /* Outputs:
    Config {
        host: "localhost",
        port: 8080,
    }
    */
}
```

### Other Format Specifiers

```rust
let x = 255;

println!("Hex: {:x}", x);           // ff
println!("Hex (upper): {:X}", x);   // FF
println!("Octal: {:o}", x);         // 377
println!("Binary: {:b}", x);        // 11111111
println!("Pointer: {:p}", &x);      // 0x7fff5fbff7ac

let pi = 3.14159;
println!("Rounded: {:.2}", pi);     // 3.14
println!("Padded: {:8.2}", pi);     // "    3.14"
```

## Standard Error vs Standard Output

### stdout - Regular Output

```rust
println!("This goes to stdout");
```

### stderr - Error Messages

```rust
eprintln!("This goes to stderr");
```

**Why separate streams?**

```bash
# Redirect output and errors separately
$ my-tool > output.txt 2> errors.txt

# Show errors on terminal, save output to file
$ my-tool > output.txt

# Discard output, keep errors
$ my-tool > /dev/null
```

### Best Practice

```rust
fn main() {
    println!("Starting process...");           // Normal messages
    eprintln!("Warning: Using default config"); // Warnings
    eprintln!("Error: File not found");        // Errors
}
```

## Performance Optimization

### Problem: Slow Printing in Loops

```rust
// SLOW: Locks and unlocks stdout on each iteration
for i in 0..100_000 {
    println!("Line {}", i);
}
```

### Solution 1: Buffer Writes

```rust
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout);

    for i in 0..100_000 {
        writeln!(handle, "Line {}", i)?;
    }

    // Buffer is flushed when handle is dropped
    Ok(())
}
```

### Solution 2: Lock stdout Once

```rust
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for i in 0..100_000 {
        writeln!(handle, "Line {}", i)?;
    }

    Ok(())
}
```

**Performance comparison:**
- Unbuffered: ~500ms for 100k lines
- With `BufWriter` or `lock()`: ~50ms (10x faster)

## Progress Indicators

### Using indicatif Crate

Add to `Cargo.toml`:
```toml
[dependencies]
indicatif = "0.17"
```

### Progress Bars

```rust
use indicatif::ProgressBar;
use std::time::Duration;
use std::thread;

fn main() {
    let pb = ProgressBar::new(100);

    for i in 0..100 {
        // Do work...
        thread::sleep(Duration::from_millis(50));

        pb.inc(1);
    }

    pb.finish_with_message("Done!");
}
```

Output:
```
████████████████████ 45/100
```

### Progress Bar with Custom Style

```rust
use indicatif::{ProgressBar, ProgressStyle};

fn main() {
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );

    for i in 0..100 {
        pb.inc(1);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    pb.finish_with_message("Completed!");
}
```

### Spinner for Indeterminate Progress

```rust
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

fn main() {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );

    spinner.set_message("Processing...");

    for _ in 0..100 {
        spinner.tick();
        std::thread::sleep(Duration::from_millis(50));
    }

    spinner.finish_with_message("Done!");
}
```

### Multiple Progress Bars

```rust
use indicatif::{MultiProgress, ProgressBar};

fn main() {
    let m = MultiProgress::new();

    let pb1 = m.add(ProgressBar::new(100));
    let pb2 = m.add(ProgressBar::new(100));

    pb1.set_message("Task 1");
    pb2.set_message("Task 2");

    // Update bars independently...
}
```

## Logging

### Using log and env_logger

Add to `Cargo.toml`:
```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

### Basic Setup

```rust
use log::{info, warn, error, debug, trace};

fn main() {
    env_logger::init();

    trace!("This is trace level");
    debug!("This is debug level");
    info!("Application started");
    warn!("This is a warning");
    error!("This is an error");
}
```

### Running with Log Levels

```bash
# Set log level via environment variable
$ RUST_LOG=info cargo run
# Shows: info, warn, error

$ RUST_LOG=debug cargo run
# Shows: debug, info, warn, error

$ RUST_LOG=trace cargo run
# Shows: everything

# Filter by module
$ RUST_LOG=my_app::network=debug cargo run

# Multiple filters
$ RUST_LOG=my_app=info,my_app::network=debug cargo run
```

### Structured Logging

```rust
use log::info;

fn process_file(path: &str, size: u64) {
    info!("Processing file: path={}, size={}", path, size);
}
```

### Custom Log Format

```rust
use env_logger::Builder;
use std::io::Write;

fn main() {
    Builder::from_default_env()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}] {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();

    log::info!("Custom format");
}
```

### Adding Verbosity Flag with clap

```rust
use clap::Parser;
use log::LevelFilter;

#[derive(Parser)]
struct Cli {
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    log::info!("Starting application...");
}
```

Usage:
```bash
$ my-tool              # Only warnings and errors
$ my-tool -v           # + info messages
$ my-tool -vv          # + debug messages
$ my-tool -vvv         # + trace messages
```

## Color Output

### Using colored Crate

```toml
[dependencies]
colored = "2.0"
```

```rust
use colored::*;

fn main() {
    println!("{}", "Success!".green());
    println!("{}", "Warning!".yellow());
    println!("{}", "Error!".red());
    println!("{}", "Info".blue().bold());

    println!(
        "{} {} {}",
        "red".red(),
        "blue".blue(),
        "green".green()
    );
}
```

### Respecting NO_COLOR

```rust
use colored::*;

fn main() {
    // Automatically respects NO_COLOR environment variable
    println!("{}", "Colored text".green());
}
```

```bash
$ NO_COLOR=1 my-tool  # Disables colors
```

### Terminal Detection

```rust
use colored::*;

fn main() {
    // Only use colors if outputting to a terminal
    if atty::is(atty::Stream::Stdout) {
        println!("{}", "Colored!".green());
    } else {
        println!("Not colored");
    }
}
```

## Tables

### Using comfy-table

```toml
[dependencies]
comfy-table = "7.0"
```

```rust
use comfy_table::Table;

fn main() {
    let mut table = Table::new();

    table
        .set_header(vec!["Name", "Age", "City"])
        .add_row(vec!["Alice", "25", "NYC"])
        .add_row(vec!["Bob", "30", "SF"])
        .add_row(vec!["Charlie", "35", "LA"]);

    println!("{}", table);
}
```

Output:
```
+-------+-----+-----+
| Name  | Age | City|
+-------+-----+-----+
| Alice | 25  | NYC |
| Bob   | 30  | SF  |
| Charlie| 35 | LA  |
+-------+-----+-----+
```

## Best Practices

1. **Use appropriate streams**
   - `println!` for normal output
   - `eprintln!` for errors and warnings

2. **Optimize for performance**
   - Use `BufWriter` or `stdout().lock()` in loops
   - Batch writes when possible

3. **Provide user control**
   - Respect `NO_COLOR` environment variable
   - Offer `--quiet` and `--verbose` flags
   - Allow disabling progress bars with `--no-progress`

4. **Log strategically**
   - Use appropriate log levels
   - Include context in log messages
   - Allow users to filter by module

5. **Progress indicators**
   - Show progress for operations > 1 second
   - Provide time estimates when possible
   - Make progress bars non-intrusive

6. **Format for humans**
   - Use colors and formatting appropriately
   - Provide structured output options (JSON, CSV) with flags
   - Make output grep-friendly when colors are disabled

## Testing Output

```rust
#[cfg(test)]
mod tests {
    use std::io::Write;

    #[test]
    fn test_output() {
        let mut buffer = Vec::new();
        writeln!(buffer, "Test output").unwrap();

        assert_eq!(buffer, b"Test output\n");
    }
}
```

## Summary

- Use `println!` and `eprintln!` for basic output
- Buffer or lock stdout for high-volume output
- Use `indicatif` for progress indicators
- Implement structured logging with `log` + `env_logger`
- Add color with `colored` (respect NO_COLOR)
- Format tables with `comfy-table`
- Always separate normal output (stdout) from errors (stderr)
