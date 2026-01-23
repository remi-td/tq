# Technical Architecture Vision

This document describes the high-level technical architecture of tq, explaining how components fit together and the design principles that guide implementation decisions.

## Design Philosophy

tq follows a **simple one-shot execution model**: one tool call → one connection → close session when done. This deliberate simplicity aligns with UNIX philosophy and makes the tool predictable, scriptable, and maintainable.

### Core Principles

1. **Library-First Design**: All business logic lives in `src/lib.rs`, with `src/main.rs` as a thin CLI wrapper
   - Enables unit testing of core logic independently from CLI parsing
   - Allows library reuse by other consumers (GUIs, web interfaces, testing harnesses)
   - Creates clean separation between interface and implementation

2. **Separation of Concerns**: Clean boundaries between layers
   - CLI layer: argument parsing and validation
   - Database layer: Teradata integration
   - Formatting layer: output rendering
   - Configuration layer: settings management

3. **Trait-Based Abstraction**: Traits define behavior contracts
   - `DatabaseClient` trait enables testing with mocks
   - Future extensibility for different connection methods
   - Compile-time polymorphism without runtime overhead

4. **Zero-Cost Abstractions**: Leverage Rust's performance guarantees
   - No runtime overhead from abstractions
   - Compile-time resolution of polymorphism
   - Stack allocation by default

5. **Fail Fast**: Validate early, provide clear error messages
   - Input validation at boundaries
   - Configuration validation before connection
   - Structured error types with context

6. **Stream-First**: Never buffer large result sets in memory
   - Streaming iterators for query results
   - Direct I/O for large exports
   - Constant memory usage regardless of result size

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                             │
│                    (CLI Entry Point)                        │
│                  - Parse arguments                          │
│                  - Dispatch commands                        │
│                  - Handle exit codes                        │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                        lib.rs                               │
│                   (Public Library API)                      │
│                  - Module exports                           │
│                  - Type re-exports                          │
│                  - Public interface                         │
└─────┬───────────────┬───────────────┬─────────────────┬─────┘
      │               │               │                 │
┌─────▼──────┐  ┌────▼─────┐  ┌──────▼──────┐  ┌──────▼──────┐
│    CLI     │  │    DB    │  │   Format    │  │   Config    │
│  (clap)    │  │ (Teradata│  │  (output)   │  │  (figment)  │
│            │  │   API)   │  │             │  │             │
│ - Argument │  │ - Connect│  │ - Table     │  │ - Load      │
│   parsing  │  │ - Query  │  │ - JSON      │  │ - Validate  │
│ - Commands │  │ - Stream │  │ - CSV       │  │ - Profiles  │
└────────────┘  └──────────┘  └─────────────┘  └─────────────┘
                      │
             ┌────────▼────────┐
             │ teradatarustapi │
             │  (C bindings)   │
             └─────────────────┘
```

## Component Integration

### CLI Layer → Database Layer

The CLI layer parses arguments and constructs configuration, then passes control to database layer:

1. **Global options** (connection, credentials) are resolved first
2. **Command-specific options** (format, output file) are bundled
3. **Database client** is instantiated with validated configuration
4. **Command execution** delegates to appropriate handler

### Database Layer → Formatting Layer

Query results flow from database to formatter without buffering:

1. **Buffered path**: Small results loaded completely for table formatting
2. **Streaming path**: Large results processed row-by-row for CSV/JSON
3. **Metadata available immediately**: Column info before first row
4. **Type-driven formatting**: Column types determine alignment and styling

### Configuration Layer → All Layers

Configuration follows strict precedence hierarchy:

```
CLI args → Env vars → Project config → User config → System config → Defaults
```

Each layer can override previous layers. The `figment` crate handles merging automatically.

## Module Organization

### Core Modules

**`src/cli.rs`**: Command-line interface definitions
- Argument structs with clap derives
- Subcommand enums
- Global options
- Value enums for format/mechanism choices

**`src/db/`**: Database operations
- `connection.rs`: Connection management, session handling
- `query.rs`: Query execution, streaming results
- `types.rs`: Type conversions, value representations
- `metadata.rs`: Schema inspection, catalog queries

**`src/format/`**: Output formatting
- `table.rs`: Table rendering with column alignment
- `json.rs`: JSON output (array and JSONL)
- `csv.rs`: CSV export with streaming

**`src/commands/`**: Command implementations
- `ping.rs`: Connectivity testing
- `query.rs`: SQL execution (batch mode)
- `repl/`: Interactive REPL mode
  - `mod.rs`: REPL loop orchestration
  - `executor.rs`: Statement execution with state
  - `completer.rs`: Tab completion logic
  - `pager.rs`: Result pagination

**`src/config.rs`**: Configuration management
- Config loading and merging
- Profile management
- Credential resolution
- Validation

**`src/error.rs`**: Error handling
- Structured error types
- User-friendly messages
- Exit code mapping

### Supporting Modules

**`src/utils/`**: Shared utilities
- Connection string parsing
- Duration parsing
- File permission checking

**`src/help.rs`**: Extended help content
- Topic-based help text
- Embedded documentation

## Data Flow Patterns

### Single Query Execution

```
User Input → Argument Parser → Config Resolver → Connection Establisher
                                                         ↓
                                                    Query Executor
                                                         ↓
                                            ┌────────────┴────────────┐
                                            ↓                         ↓
                                     Buffered Result          Streaming Result
                                            ↓                         ↓
                                      Table Formatter          CSV/JSON Writer
                                            ↓                         ↓
                                          stdout                    stdout
```

### REPL Session

```
Start REPL → Connect Database → Initialize Editor → Show Prompt
                                                         ↓
                                                    Read Input
                                                         ↓
                                        ┌────────────────┴────────────────┐
                                        ↓                                 ↓
                                  SQL Statement                    Metacommand
                                        ↓                                 ↓
                                   Execute Query                    Process Command
                                        ↓                                 ↓
                                   Format Output                    Update State
                                        ↓                                 ↓
                                   Display Results             ←─────────┘
                                        ↓
                                   Show Prompt (loop)
```

### Configuration Resolution

```
Load Defaults → System Config (/etc/tq/config.toml)
                      ↓
               User Config (~/.config/tq/config.toml)
                      ↓
               Project Config (.tq.toml)
                      ↓
               Environment Variables (TQ_*)
                      ↓
               CLI Arguments
                      ↓
               Validate → Resolved Config
```

## Design Patterns

### Connection Management

**Single-use connections**: Each command establishes a fresh connection
- Simplifies error recovery
- Avoids stale connection issues
- Clear resource lifecycle

**Exception: REPL mode** maintains persistent connection
- User expects session state
- Performance benefit for repeated queries
- Connection validated before each query

### Error Handling Strategy

**Structured errors with context**:
- `thiserror` defines error types for pattern matching
- `anyhow` adds context during propagation
- User-friendly error messages with troubleshooting tips
- Exit codes follow UNIX conventions (0=success, 1=runtime error, 2=usage error)

**Security-aware error messages**:
- Never log passwords or sensitive data
- Sanitize connection strings in error output
- Use `secrecy::Secret` to prevent accidental exposure

### Streaming Architecture

**Two-path design**:
1. **Buffered**: Load all rows for interactive table display
2. **Streaming**: Process rows incrementally for exports

**Memory guarantee**: Memory usage independent of result size
- `QueryResultStream` implements `Iterator<Item = Result<Row>>`
- Formatters write directly to output
- No intermediate buffering

### Type Safety

**Newtypes for validation**:
```rust
struct ValidatedConnectionString(String);
struct SecurePassword(Secret<String>);
```

**Type-driven behavior**:
- `TeradataType` enum determines column alignment
- `Value` enum handles NULL representation
- `OutputFormat` enum drives formatter selection

## Security Architecture

### Credential Flow

```
Password Resolution:
1. Password file (--password-file, ~/.tq_passwords)
   → Validate permissions (must be 0600)
   → Parse and extract password
2. Environment variable (TQ_PASSWORD)
3. Interactive prompt (if TTY)
4. Fail with clear error

Storage: secrecy::Secret<String>
- Zeros memory on drop
- Redacts in Debug output
- Never serialized
```

### Permission Enforcement

**Critical pattern**: Validate BEFORE reading
```rust
// ALWAYS check permissions first
validate_file_permissions(path)?;
let content = read_to_string(path)?;
```

### Input Validation

**Boundaries**: Validate at entry points
- CLI arguments: clap validators
- File paths: canonicalize and check traversal
- SQL input: pass to prepared statements (Teradata handles escaping)
- Configuration: schema validation before use

## Performance Considerations

### Optimization Targets

| Metric | Target | Strategy |
|--------|--------|----------|
| Startup time | < 100ms | Lazy initialization, minimal dependencies |
| Memory (idle) | < 10 MB | Zero-cost abstractions, stack allocation |
| Memory (query) | < 50 MB | Streaming, no result buffering |
| Binary size | < 5 MB | LTO, strip symbols, opt-level=z |
| Query overhead | < 1ms | Direct bindings, no indirection |

### Streaming Benefits

- **Constant memory**: Process 1M rows with same memory as 10 rows
- **Fast first byte**: Display begins immediately
- **Interruptible**: Ctrl-C works at any point
- **Composable**: Pipe to other UNIX tools

## Extensibility Points

### Output Formats

Add new formatters by implementing:
```rust
fn write_output(result: &QueryResult, writer: &mut dyn Write) -> Result<()>
```

Register in match statement in `commands/query.rs`.

### Metadata Commands

Add catalog browsing by querying DBC views:
- `DBC.DatabasesV`: List databases
- `DBC.TablesV`: List tables
- `DBC.ColumnsV`: Describe columns
- `DBC.IndicesV`: Show indexes

### Authentication Methods

Support additional logon mechanisms:
- Extend `LogonMechanism` enum
- Update connection string formatter
- Document in help text

### Configuration Providers

Add new config sources via `figment`:
```rust
.merge(YourProvider::new())
```

## Testing Strategy

### Unit Tests
- Inline with source (`#[cfg(test)] mod tests`)
- Test pure functions independently
- Mock database with trait objects

### Integration Tests
- `assert_cmd` for end-to-end CLI testing
- `assert_fs` for file fixtures
- Real database for smoke tests

### Property Tests
- `proptest` for input fuzzing
- Connection string parsing edge cases
- SQL statement splitting

## Dependencies Rationale

| Crate | Purpose | Rationale |
|-------|---------|-----------|
| `clap` | CLI parsing | Industry standard, derive macros |
| `teradatarustapi` | Database driver | Official Teradata bindings |
| `anyhow` | Error propagation | Ergonomic context chaining |
| `thiserror` | Error types | Derive macros for boilerplate |
| `comfy-table` | Table formatting | Lightweight, reliable |
| `serde`/`serde_json` | Serialization | Universal Rust serialization |
| `csv` | CSV export | Streaming, RFC 4180 compliant |
| `figment` | Configuration | Layered provider merging |
| `secrecy` | Credentials | Zero-on-drop, redacted debug |
| `reedline` | REPL editor | Modern line editing, Nushell-proven |
| `nu-ansi-term` | Syntax highlighting | Zero-copy styling |

Deliberately minimal: no async runtime (synchronous sufficient), no heavy parsing (simple statement splitting), no complex UI frameworks (terminal-first).

## Future Considerations

### Potential Enhancements

- **Transaction support**: `--atomic` flag for multi-statement batches
- **Variable substitution**: `--var key=value` for parameterized scripts
- **Query templates**: Named template library
- **SSL/TLS configuration**: Certificate pinning, custom CAs
- **Keyring integration**: OS-native credential storage
- **Export profiles**: Saved format configurations

### Non-Goals

- **Connection pooling**: One-shot model doesn't benefit
- **Query builder DSL**: Users write SQL directly
- **ORM features**: Not an application framework
- **GUI**: Terminal-first by design
- **Database migrations**: Use dedicated tools

## Conclusion

This architecture provides:
- **Simplicity**: Clear one-shot execution model
- **Performance**: Streaming, zero-copy, minimal overhead
- **Security**: Proper credential handling, input validation
- **Maintainability**: Library-first, trait-based, well-tested
- **Usability**: UNIX conventions, clear errors, flexible output

The design scales from simple one-off queries to complex interactive sessions while maintaining predictable behavior and resource usage.
