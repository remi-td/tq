# tq - Teradata Query

A fast, lightweight command-line client for Teradata databases written in Rust.

## Features

- Simple one-shot query execution
- Multiple authentication mechanisms (TD2, LDAP, Kerberos)
- Multiple output formats (table, JSON, CSV)
- Secure credential handling
- Cross-platform support (Linux, macOS, Windows)
- Zero runtime dependencies (static binary)

## Installation

### From crates.io (once published)
```bash
cargo install tq
```

### From source
```bash
git clone <repository-url>
cd tq
cargo install --path .
```

### Pre-built binaries
Download from the [releases page](releases) for your platform.

## Quick Start

```bash
# Basic query with connection string
tq -l "user:password@host:1025/database" "SELECT * FROM dbc.dbcinfo"

# Using environment variables

```
export TQ_HOST=myteradata.company.com
export TQ_PORT=1025
export TQ_USER=myuser
export TQ_PASSWORD=mypassword
export TQ_DATABASE=mydatabase
```
or
```
export TQ_LOGON="myuser:mypassword@myteradata.company.com:1025/mydatabase"
```

tq "SELECT COUNT(*) FROM my_table"

# JSON output for scripting
tq --format json "SELECT user_id, username FROM users" | jq '.[] | .username'

# CSV export
tq --format csv "SELECT * FROM sales_data" > sales.csv
```

## Usage

```
Usage: tq [OPTIONS] <QUERY>

Arguments:
  <QUERY>  The SQL query to execute

Options:
  -l, --logon <STRING>
          Database logon string in the format "user:password@host:port/database"

  --logmech <MECHANISM>
          Logon mechanism to use [default: TD2]
          Supported: TD2, LDAP, KRB5, TDNEGO

  --format <FORMAT>
          Output format [default: table]
          [possible values: table, json, csv]

  --color <WHEN>
          When to use colored output
          [possible values: always, auto, never]
          [default: auto]

  -p, --ping
          Simple connection test to the database

  -h, --help
          Print help information

  -V, --version
          Print version information
```

## Configuration

`tq` supports multiple configuration methods with the following precedence (highest to lowest):

1. Command-line arguments
2. Environment variables
3. Project config file (`.tq.toml`)
4. User config file (`~/.config/tq/config.toml`)
5. Built-in defaults

### Environment Variables

- `TQ_HOST` - Database host
- `TQ_PORT` - Database port (default: 1025)
- `TQ_USER` - Database username
- `TQ_PASSWORD` - Database password
- `TQ_DATABASE` - Default database/schema
- `TQ_LOGMECH` - Logon mechanism (default: TD2)
- `TQ_FORMAT` - Default output format (table, json, csv)
- `TQ_LOGON` - Database logon string in the format "user:password@host:port/database"

### Configuration File

Create `~/.config/tq/config.toml`:

```toml
host = "myteradata.company.com"
port = 1025
user = "myuser"
database = "mydatabase"
logmech = "TD2"
format = "table"
```

## Examples

### Basic Queries

```bash
# Simple SELECT
tq "SELECT * FROM dbc.dbcinfo"

# With WHERE clause
tq "SELECT * FROM employees WHERE department = 'IT'"

# Multiple statements (use semicolons)
tq "CREATE TABLE test (id INT); INSERT INTO test VALUES (1);"
```

### Output Formats

```bash
# Human-readable table (default)
tq "SELECT * FROM products LIMIT 5"

# JSON for scripting
tq --format json "SELECT product_id, name, price FROM products" | jq '.'

# CSV for data export
tq --format csv "SELECT * FROM sales_2024" > sales_2024.csv
```

### Secure Password Handling

```bash
# Read password from file (recommended)
echo "mypassword" > ~/.tq_password
chmod 0600 ~/.tq_password
tq -l "user:@host:1025/db" --password-file ~/.tq_password "SELECT 1"

# Read password from stdin
echo "mypassword" | tq -l "user:@host:1025/db" --password-file - "SELECT 1"

# Using environment variable (less secure - visible in process list briefly)
TQ_PASSWORD=mypassword tq "SELECT 1"
```

### Connection Testing

```bash
# Test connection without running a query
tq --ping

# Test connection with specific credentials
tq -l "testuser:testpass@host:1025/testdb" --ping
```

### Different Authentication Methods

```bash
# TD2 (default - username/password)
tq --logmech TD2 "SELECT SESSION"

# LDAP authentication
tq --logmech LDAP "SELECT SESSION"

# Kerberos authentication
tq --logmech KRB5 "SELECT SESSION"
```

## Security Best Practices

1. **Never use passwords in command-line arguments** - they appear in process lists and shell history
2. **Use `--password-file` or environment variables** for password handling
3. **Protect credential files** with `chmod 0600`
4. **Use project/user config files** in secure locations
5. **Consider keyring integration** for production environments

## Output Format Details

### Table Format (default for interactive use)

```
┌─────────┬──────────┬─────────┐
│ user_id │ username │ active  │
├─────────┼──────────┼─────────┤
│ 1       │ alice    │ true    │
│ 2       │ bob      │ false   │
└─────────┴──────────┴─────────┘
```

### JSON Format (for scripting)

```json
[
  {"user_id": 1, "username": "alice", "active": true},
  {"user_id": 2, "username": "bob", "active": false}
]
```

### CSV Format (for data export)

```csv
user_id,username,active
1,alice,true
2,bob,false
```

## Exit Codes

- `0` - Success
- `1` - General error (connection failed, query error)
- `2` - Usage error (invalid arguments)

## Building from Source

```bash
# Development build
cargo build

# Optimized release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- "SELECT 1"
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## License

MIT License

## Links

- [Teradata Rust API](https://github.com/Teradata/teradatarustapi)
- [Report Issues](issues)
- [Documentation](docs)