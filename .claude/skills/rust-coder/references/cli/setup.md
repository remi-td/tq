# Rust CLI Project Setup

## Prerequisites

Install Rust from https://www.rust-lang.org/tools/install

Verify installation:
```bash
rustc --version
cargo --version
```

## Creating a New CLI Project

```bash
cargo new my-cli-tool
cd my-cli-tool
```

This generates:
```
my-cli-tool/
├── Cargo.toml       # Project manifest and dependencies
└── src/
    └── main.rs      # Entry point for the binary
```

## Initial Cargo.toml Structure

```toml
[package]
name = "my-cli-tool"
version = "0.1.0"
edition = "2021"

[dependencies]
# Add dependencies here
```

## Verification

Run the generated hello world program:
```bash
cargo run
```

Expected output:
```
   Compiling my-cli-tool v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.50s
     Running `target/debug/my-cli-tool`
Hello, world!
```

## Recommended Project Structure

For CLI applications with reusable logic:

```
my-cli-tool/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs       # Core business logic (pub functions)
│   ├── main.rs      # CLI entry point
│   └── ...          # Additional modules
├── tests/
│   └── integration_test.rs
├── benches/
│   └── benchmark.rs
└── examples/
    └── usage_example.rs
```

### Key Files

**src/lib.rs** - Contains core functionality:
- Mark public with `pub fn` for external use
- Contains testable business logic
- Independent of CLI interface

**src/main.rs** - CLI-specific code:
- Argument parsing
- Output formatting
- Error reporting to users
- Calls into lib.rs for business logic

**tests/** - Integration tests that run against the compiled binary

**examples/** - Demonstration programs showing API usage

## Common Initial Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
# Argument parsing
clap = { version = "4.0", features = ["derive"] }

# Error handling
anyhow = "1.0"

# Logging
log = "0.4"
env_logger = "0.11"

[dev-dependencies]
# Testing
assert_cmd = "2.0"
predicates = "3.0"
assert_fs = "1.0"
```

## Development Workflow Commands

```bash
# Fast compilation check
cargo check

# Run with arguments
cargo run -- arg1 arg2

# Run tests
cargo test

# Build optimized release binary
cargo build --release

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Next Steps

1. Add argument parsing with clap (see [cli-args.md](cli-args.md))
2. Implement error handling (see [errors.md](errors.md))
3. Add output formatting and logging (see [output.md](output.md))
4. Write tests (see [testing.md](testing.md))
5. Prepare for distribution (see [packaging.md](packaging.md))
