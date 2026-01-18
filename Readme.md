# tq - Teradata Query

A fast, lightweight command-line client for Teradata databases written in Rust.

## Features

- **One-shot execution model**: connect, execute, close - designed for scripts and CLI usage
- **Multiple authentication mechanisms**: TD2, LDAP, Kerberos, TDNEGO
- **Multiple output formats**: table (human-readable), JSON (scripting), CSV (data export)
- **Secure credential handling**: password files, environment variables, never in shell history
- **Configuration hierarchy**: CLI args, environment, project config, user config, defaults
- **Type preservation**: proper handling of numbers, booleans, dates, and NULL values
- **Streaming support**: efficient handling of large result sets

## License & Prerequisites

**tq source code** is licensed under the MIT License (see [LICENSE](LICENSE)).

**Important**: This tool depends on the **Teradata GoSQL Driver**, which is proprietary
software owned by Teradata Corporation with separate license terms. When you build this
project, Cargo automatically fetches the driver from Teradata's official GitHub repository
([teradatarustapi](https://github.com/Teradata/teradatarustapi)).

### Requirements to Use tq

1. **Authorized Teradata Access**: You must have a valid, authorized license to use the
   Teradata Analytics Platform. This tool is a client that requires legitimate database
   access credentials.

2. **Teradata Driver License**: By building and using this software, you agree to comply
   with Teradata Corporation's license terms for the GoSQL driver. See the
   [NOTICE](NOTICE) file for details.

3. **Rust Toolchain**: Rust 1.70 or later (for building from source)

This is similar to other database clients - the client tool itself is open source, but
you need legitimate access to the database system. See [NOTICE](NOTICE) for complete
licensing information.

## Installation

### Prerequisites

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **Teradata Access**: Valid credentials for a Teradata database
- **Teradata Driver**: Automatically fetched by Cargo from Teradata's public repository

### From source

```bash
git clone <repository-url>
cd tq
cargo install --path .
```

### Development

```bash
cargo build          # Development build
cargo build --release # Optimized release build
cargo test           # Run all tests
```

## Quick Start

```bash
# Option 1: Set connection details using environment file (recommended for development)
cp .env.example .env
# Edit .env and set TQ_LOGON=myuser:mypassword@myteradata:1025/mydatabase

# Option 2: Set connection details in your shell
export TQ_LOGON="myuser:mypassword@myteradata:1025/mydatabase"

# Test connectivity
tq ping

# Run a query
tq query "SELECT * FROM dbc.dbcinfo"

# Export to JSON
tq query --format json "SELECT * FROM employees" > employees.json

# Export to CSV
tq query --format csv "SELECT * FROM sales" --output sales.csv
```

## Usage

```
tq is a fast, lightweight command-line client for Teradata databases.

Usage: tq [OPTIONS] <COMMAND>

Commands:
  ping   Test database connectivity
  query  Execute a SQL query
  help   Print this message or the help of the given subcommand(s)

Global Options:
  -l, --logon <LOGON>        Connection string: user:password@host:port/database
      --password-file <FILE> Read password from file (recommended for security)
      --logmech <MECH>       Authentication mechanism [default: TD2]
  -t, --timeout <DURATION>   Connection timeout [default: 30s]
  -v, --verbose...           Verbose output (repeat for more: -v, -vv, -vvv)
  -q, --quiet                Suppress non-essential output
      --color <WHEN>         Color output control [default: auto]
  -h, --help                 Print help
  -V, --version              Print version
```

### Query Command

```bash
tq query [OPTIONS] [QUERY]

Arguments:
  [QUERY]  SQL query to execute (or use --file or stdin)

Options:
  -f, --format <FORMAT>   Output format: table, json, csv [default: table]
  -o, --output <FILE>     Write output to file instead of stdout
      --file <FILE>       Read SQL from file
      --no-header         Omit column headers in output
      --timing            Show query execution time
  -n, --limit <N>         Limit number of rows returned
```

### Ping Command

```bash
tq ping [OPTIONS]

Options:
  -c, --count <N>         Number of ping attempts [default: 1]
  -i, --interval <DURATION> Interval between pings [default: 1s]
```

## Configuration

Configuration follows this precedence (highest to lowest):

1. Command-line arguments
2. Environment variables
3. Project config file (`.tq.toml` in current directory)
4. User config file (`~/.config/tq/config.toml`)
5. System config file (`/etc/tq/config.toml`)
6. Built-in defaults

### Environment Variables

| Variable | Description |
|----------|-------------|
| `TQ_LOGON` | Connection string: `user:password@host:port/database` |
| `TQ_LOGMECH` | Authentication mechanism (TD2, LDAP, KRB5, TDNEGO) |
| `TQ_TIMEOUT` | Connection timeout (e.g., `30s`, `5m`) |
| `TQ_FORMAT` | Default output format (table, json, csv) |
| `TQ_COLOR` | Color output (auto, always, never) |

### Environment File (.env)

For convenience, you can store environment variables in a `.env` file in your project directory. This is especially useful for development and testing.

1. Copy the example file:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` with your connection details:
   ```bash
   # .env file
   TQ_LOGON=myuser:mypassword@myteradata:1025/mydatabase
   ```

3. Run tq commands without specifying connection details:
   ```bash
   tq ping
   tq query "SELECT 1"
   ```

**Security notes:**
- The `.env` file is automatically excluded from git (listed in `.gitignore`)
- Use `.env.example` as a template to share configuration structure without secrets
- Ensure `.env` has appropriate permissions: `chmod 0600 .env`

### Configuration File

Create `~/.config/tq/config.toml`:

```toml
[connection]
host = "myteradata.company.com"
port = 1025
user = "myuser"
database = "mydatabase"

[output]
format = "table"
color = "auto"
```

## Examples

### Basic Queries

```bash
# Set logon string once
export TQ_LOGON="user:password@host:1025/database"

# Simple SELECT
tq query "SELECT * FROM dbc.dbcinfo"

# With WHERE clause
tq query "SELECT * FROM employees WHERE department = 'IT'"

# Query from file
tq query --file report.sql

# Query from stdin
echo "SELECT COUNT(*) FROM orders" | tq query
```

### Output Formats

```bash
# Human-readable table (default)
tq query "SELECT * FROM products LIMIT 5"

# JSON for scripting
tq query --format json "SELECT product_id, name, price FROM products" | jq '.'

# CSV for data export
tq query --format csv "SELECT * FROM sales_2024" > sales_2024.csv

# With timing information
tq query --timing "SELECT COUNT(*) FROM large_table"
```

### Secure Password Handling

```bash
# Method 1: Password file (recommended)
echo "mypassword" > ~/.tq_password
chmod 0600 ~/.tq_password
tq -l "user@host:1025/db" --password-file ~/.tq_password query "SELECT 1"

# Method 2: Connection string without password + password file
export TQ_LOGON="user@host:1025/db"
tq --password-file ~/.tq_password query "SELECT 1"
```

### Different Authentication Methods

```bash
# TD2 (default - username/password)
tq --logmech TD2 query "SELECT SESSION"

# LDAP authentication
tq --logmech LDAP query "SELECT SESSION"

# Kerberos authentication
tq --logmech KRB5 query "SELECT SESSION"

# Teradata negotiation
tq --logmech TDNEGO query "SELECT SESSION"
```

### Scripting Examples

```bash
# Get user count as JSON
count=$(tq query --format json "SELECT COUNT(*) AS cnt FROM users" | jq -r '.[0].cnt')
echo "User count: $count"

# Export all tables to CSV
for table in customers orders products; do
  tq query --format csv "SELECT * FROM $table" > "${table}.csv"
done

# Conditional query based on result
if tq query "SELECT 1 FROM table WHERE condition" | grep -q .; then
  echo "Condition met"
fi
```

## Output Format Details

### Table Format (default for interactive use)

```
+---------+----------+---------+
| user_id | username | active  |
+---------+----------+---------+
|       1 | alice    | true    |
|       2 | bob      | false   |
+---------+----------+---------+
2 row(s) in set (0.045s)
```

### JSON Format (for scripting)

```json
[
  {"user_id": 1, "username": "alice", "active": true},
  {"user_id": 2, "username": "bob", "active": false}
]
```

Type preservation:
- Numbers are JSON numbers (not strings)
- Booleans are JSON booleans
- NULL values are JSON null
- Dates and timestamps are ISO 8601 strings

### CSV Format (RFC 4180 compliant)

```csv
user_id,username,active
1,alice,true
2,bob,false
```

Features:
- Proper quoting of fields containing commas, quotes, or newlines
- NULL values become empty fields
- Unix line endings for compatibility

## Security Best Practices

1. **Never use passwords in command-line arguments** - they appear in process lists and shell history
2. **Use `--password-file` or environment variables** for password handling
3. **Protect credential files** with `chmod 0600`
4. **Use project/user config files** in secure locations
5. **tq warns** if password files have insecure permissions

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | Runtime error (connection failed, query error, etc.) |
| 2 | Usage error (invalid arguments, missing required options) |

## Building from Source

```bash
# Clone repository
git clone <repository-url>
cd tq

# Development build
# Note: First build will fetch the Teradata driver from GitHub
cargo build

# Optimized release build
cargo build --release

# Run tests (101+ tests)
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- ping

# Install to ~/.cargo/bin
cargo install --path .
```

**Note**: The first build will take longer as Cargo fetches dependencies, including
the Teradata GoSQL driver from Teradata's public GitHub repository. By building this
software, you acknowledge that you have read and agree to comply with the Teradata
driver license terms (see [NOTICE](NOTICE)).

## Architecture

tq follows a library-first design:

```
src/
  lib.rs        # Public library API
  main.rs       # CLI entry point
  cli.rs        # Command-line interface (Clap)
  config.rs     # Configuration management (Figment)
  error.rs      # Error types (thiserror)
  db/           # Database connectivity
    connection.rs   # Connection management
    client.rs       # Query execution
    types.rs        # Type conversions
  format/       # Output formatters
    table.rs    # Table format (comfy-table)
    json.rs     # JSON format (serde_json)
    csv.rs      # CSV format (csv crate)
  commands/     # Command implementations
    ping.rs     # Ping command
    query.rs    # Query command
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## License

The tq source code is licensed under the MIT License - see the [LICENSE](LICENSE) file
for details.

**Important**: This software depends on the Teradata GoSQL Driver, which is proprietary
software with separate license terms. See the [NOTICE](NOTICE) file for complete
licensing information about dependencies.

## Links

- [Teradata Rust API](https://github.com/Teradata/teradatarustapi)
- [Report Issues](issues)
- [Documentation](docs)
