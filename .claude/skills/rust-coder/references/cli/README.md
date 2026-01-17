# Rust CLI Development References

Comprehensive guides for building command-line applications in Rust, based on the official [Rust CLI Book](https://rust-cli.github.io/book/).

## Contents

1. **[setup.md](setup.md)** - Project initialization and configuration
2. **[cli-args.md](cli-args.md)** - Parsing command-line arguments with clap
3. **[errors.md](errors.md)** - Error handling strategies for CLI applications
4. **[output.md](output.md)** - Output formatting, logging, and progress indicators
5. **[testing.md](testing.md)** - Testing strategies for CLI applications
6. **[packaging.md](packaging.md)** - Distribution methods and packaging options

## Quick Reference

### Essential Dependencies

- **clap** `{ version = "4.0", features = ["derive"] }` - Argument parsing
- **anyhow** `"1.0"` - Error handling for applications
- **env_logger** + **log** - Structured logging
- **indicatif** - Progress bars and spinners
- **assert_cmd** + **predicates** + **assert_fs** - Integration testing

### Project Structure Best Practice

```
my-cli/
├── Cargo.toml
├── src/
│   ├── lib.rs         # Core business logic (testable, reusable)
│   └── main.rs        # CLI entry point (argument parsing, output)
├── tests/              # Integration tests
├── benches/            # Benchmarks
└── examples/           # Example usage
```

### Key Principles

- **Separate concerns**: Business logic in lib.rs, CLI interface in main.rs
- **Robust errors**: Use `Result<T, E>` with `anyhow::Context` for meaningful messages
- **Testability**: Accept `impl std::io::Write` instead of printing directly
- **User experience**: Clear errors, progress indicators, helpful `--help` text
- **Distribution**: Support cargo install, binary releases, and package managers

## When to Reference

Use these guides when:
- Starting a new CLI project
- Adding argument parsing to an application
- Implementing proper error handling for user-facing tools
- Adding logging or progress indicators
- Writing tests for CLI behavior
- Preparing to distribute a CLI tool
