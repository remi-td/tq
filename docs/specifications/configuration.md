# Configuration and Credential Management

## Overview

### Design Principles

Configuration management follows these principles:

- **Optional by default**: Tool works without any configuration file
- **Convention over configuration**: Sensible defaults for common use cases
- **Security first**: Never encourage inline passwords, always suggest password_file
- **Clear precedence**: CLI arguments override everything
- **Fail fast**: Invalid configuration produces clear error messages with guidance

## Configuration Hierarchy

Configuration values are loaded in this order (later overrides earlier):

1. **Built-in defaults** - Hardcoded sensible defaults
2. **User config file** - `~/.tq/config.toml` (if exists)
3. **Environment variables** - `TQ_*` variables
4. **Command-line arguments** - Flags and options

### Precedence Examples

**Example: Format preference**
```bash
# Built-in default: table
# Config file: format = "json"
# Environment: TQ_FORMAT=csv
# CLI flag: --format json

# Result: json (CLI flag wins)
```

**Example: Connection details**
```bash
# Config file profile 'dev': host = "dev.company.com", database = "devdb"
# Environment: TQ_DATABASE=testdb
# CLI flag: --profile dev

# Result: host = "dev.company.com", database = "testdb"
# (Profile provides host, env var overrides database)
```

### Profile Selection Precedence

When using `--profile <name>`, the profile values are loaded as base values, then overridden by:
1. Environment variables
2. Command-line arguments

```bash
# Profile 'prod': host=prod.company.com, database=production, user=alice
# Command: tq --profile prod --database staging query "SELECT 1"

# Result: host=prod.company.com, database=staging, user=alice
```

## User Configuration File

### File Location

The user configuration file is located at:

- **macOS/Linux**: `~/.tq/config.toml`
- **Windows**: `%USERPROFILE%\.tq\config.toml`

The directory `~/.tq/` is created automatically if it doesn't exist.

### File Format

The configuration file uses TOML format for readability and type safety.

**Complete example:**

```toml
# ~/.tq/config.toml

# Default preferences (optional)
[defaults]
format = "table"              # Output format: table, json, csv
editor_mode = "emacs"         # REPL editor mode: emacs, vi
syntax_highlighting = true    # Enable SQL syntax highlighting
paging = true                 # Enable result paging in REPL
timing = false                # Show query execution time

# Connection profiles (optional)
[profiles.dev]
host = "dev.company.com"
port = 1025
database = "development"
user = "alice"
logmech = "TD2"
password_file = "~/.tq/passwords/dev"  # Path to password file

[profiles.prod]
host = "prod.company.com"
port = 1025
database = "production"
user = "alice"
logmech = "LDAP"
password_file = "~/.tq/passwords/prod"

[profiles.local]
host = "localhost"
port = 1025
database = "testdb"
user = "dbc"
logmech = "TD2"
# No password_file - will prompt interactively
```

### Defaults Section

The `[defaults]` section specifies user preferences for tool behavior.

**All fields are optional.** If not specified, built-in defaults are used.

| Field | Type | Built-in Default | Description |
|-------|------|------------------|-------------|
| `format` | string | `"table"` | Output format: `table`, `json`, `csv` |
| `editor_mode` | string | `"emacs"` | REPL editor keybindings: `emacs`, `vi` |
| `syntax_highlighting` | boolean | `true` | Enable SQL syntax highlighting in REPL |
| `paging` | boolean | `true` | Enable result paging in REPL |
| `timing` | boolean | `false` | Show query execution time |

**Minimal example:**
```toml
[defaults]
format = "json"
timing = true
```

### File Permissions

For security, `tq` checks the config file permissions:

- **Recommended**: `0600` (read/write for owner only)
- **Accepted**: `0644` (world-readable)
- **Rejected**: World-writable permissions (`0666`, `0777`, etc.)

**Warning issued if too permissive:**
```
Warning: Configuration file ~/.tq/config.toml has permissive permissions (0644)
Recommendation: chmod 0600 ~/.tq/config.toml
```

Config file permissions issue a **warning** (not an error) because config files should not contain passwords (use `password_file` instead).

For password files, permissions are **strictly enforced**.

## Connection Profiles

### Profile Structure

Each profile is defined in a `[profiles.<name>]` section.

**Required fields:**
- `host` - Database hostname (string)

**Optional fields:**
- `port` - Database port (integer, default: 1025)
- `database` - Database name (string)
- `user` - Username (string)
- `logmech` - Authentication mechanism (string: TD2, LDAP, KRB5, TDNEGO, default: TD2)
- `password_file` - Path to password file (string)
- `timeout` - Connection timeout (string, e.g., "30s", default: "30s")

**Example:**
```toml
[profiles.staging]
host = "staging.db.company.com"
port = 1025
database = "staging_db"
user = "bob"
logmech = "LDAP"
password_file = "~/.tq/passwords/staging"
timeout = "60s"
```

### Using Profiles

Profiles are selected with the `--profile <name>` flag:

```bash
# Use 'dev' profile
tq --profile dev query "SELECT CURRENT_DATE"

# Use 'prod' profile, override database
tq --profile prod --database backup_db query "SELECT COUNT(*) FROM users"

# Use 'local' profile in REPL
tq --profile local repl
```

### Profile Field Overrides

Profile fields can be overridden by:
1. Environment variables (e.g., `TQ_DATABASE`)
2. CLI flags (e.g., `--database`)

```bash
# Profile 'dev' has database = "development"
# Override with environment variable
TQ_DATABASE=testing tq --profile dev query "SELECT 1"

# Override with CLI flag (takes precedence over env var)
TQ_DATABASE=testing tq --profile dev --database production query "SELECT 1"
```

### Listing Profiles

Users can see available profiles using the `tq profiles` command:

```bash
# List all profiles
tq profiles
```

**Output format:**
```
Available profiles (from ~/.tq/config.toml):

  dev
    Host:     dev.company.com:1025
    Database: development
    User:     alice
    Logmech:  TD2

  prod
    Host:     prod.company.com:1025
    Database: production
    User:     alice
    Logmech:  LDAP
```

**Security:**
- Password fields are NEVER displayed
- Password file paths are not shown
- Only connection metadata is revealed

## Environment Variables

Environment variables provide a way to configure `tq` without a config file or CLI flags.

### Supported Variables

| Variable | Type | Description | Example |
|----------|------|-------------|---------|
| `TQ_LOGON` | string | Complete connection string | `user:pass@host:1025/db` |
| `TQ_HOST` | string | Database hostname | `myteradata.company.com` |
| `TQ_PORT` | integer | Database port | `1025` |
| `TQ_USER` | string | Database username | `alice` |
| `TQ_PASSWORD` | string | Database password (discouraged) | `mypassword` |
| `TQ_DATABASE` | string | Database name | `production` |
| `TQ_LOGMECH` | string | Authentication mechanism | `LDAP` |
| `TQ_FORMAT` | string | Default output format | `json` |
| `TQ_TIMEOUT` | string | Connection timeout | `30s` |
| `TQ_PROFILE` | string | Profile name to use | `prod` |

### Usage Examples

**Complete connection string:**
```bash
export TQ_LOGON="alice@dev.company.com:1025/development"
tq query "SELECT CURRENT_DATE"
```

**Individual connection fields:**
```bash
export TQ_HOST="dev.company.com"
export TQ_PORT="1025"
export TQ_DATABASE="development"
export TQ_USER="alice"
tq query "SELECT CURRENT_DATE"
```

**Profile selection:**
```bash
export TQ_PROFILE=dev
tq query "SELECT CURRENT_DATE"
```

**Format preference:**
```bash
TQ_FORMAT=json tq query "SELECT * FROM users"
```

### Security Considerations

**TQ_PASSWORD is discouraged:**
- Visible in process list (`ps aux`)
- Stored in shell history
- Logged by system audit tools

**Recommended alternatives:**
1. Use `password_file` in profile
2. Use password file flag: `--password-file ~/.tq/passwords/dev`
3. Allow interactive prompt (omit password entirely)

## Credential Management

### Security Principles

1. **Never use passwords in CLI arguments** - visible in `ps`, shell history
2. **Never log passwords** - sanitize all debug output
3. **Use file permissions** - `chmod 0600` for credential files
4. **Prefer password files** - secure file-based credentials
5. **Support password prompts** - interactive secure input when password missing

### Password Sources (Priority Order)

When a password is needed, `tq` checks these sources in order:

1. **Command-line connection string** - `--logon user:pass@host` (discouraged, but supported)
2. **Password file from CLI** - `--password-file /path/to/file`
3. **Password file from profile** - `password_file = "~/.tq/passwords/dev"`
4. **Environment variable** - `TQ_PASSWORD` (discouraged)
5. **Interactive prompt** - Secure input if password still missing

### Password File Format

Password files contain a single line with the password:

**Format:**
```
mypassword123
```

**Creating a password file:**
```bash
# Create directory
mkdir -p ~/.tq/passwords
chmod 0700 ~/.tq/passwords

# Write password
echo "mypassword123" > ~/.tq/passwords/dev
chmod 0600 ~/.tq/passwords/dev

# Use in profile
# [profiles.dev]
# password_file = "~/.tq/passwords/dev"
```

**File permissions enforcement:**

Password file permissions are **strictly enforced** for security:

- **Required**: Owner read-only or read-write (`0600`)
- **Rejected**: Any permissions allowing group or world access (`0644`, `0666`, etc.)

If permissions are too permissive, `tq` **refuses to read the file** and exits with an error:
```
Error: Password file has insecure permissions: ~/.tq/passwords/dev
Current permissions: 0644 (readable by group and others)
Required permissions: 0600 (owner read-write only)

Security risk: Password file is readable by other users

Fix: chmod 0600 ~/.tq/passwords/dev
```

### Interactive Password Prompt

If no password is provided via any source, `tq` prompts interactively:

```bash
$ tq -l "alice@dev.company.com:1025/mydb" query "SELECT 1"
Password: ****  # secure input, not echoed
```

**Prompt behavior:**
- Appears only when password is needed
- Input is not echoed to terminal
- Ctrl+C cancels connection
- Works in TTY only (fails in non-interactive environments)

### Inline Passwords (Discouraged)

While supported for convenience, inline passwords are **strongly discouraged**:

```bash
# DISCOURAGED: Password visible in shell history and process list
tq -l "alice:mypassword@host:1025/db" query "SELECT 1"

# BETTER: Use password file
tq -l "alice@host:1025/db" --password-file ~/.tq/passwords/dev query "SELECT 1"

# BEST: Use profile with password_file
tq --profile dev query "SELECT 1"
```

**Security warning issued:**
```
Warning: Password provided in connection string
Recommendation: Use --password-file or store password in profile
```

## Common Use Cases

### Use Case 1: First-Time User (No Config)

```bash
# Tool works without config file
tq -l "alice@dev.company.com:1025/mydb" query "SELECT CURRENT_DATE"
Password: ****
# Result: ...
```

### Use Case 2: Create Basic Config

```bash
# Create config directory
mkdir -p ~/.tq/passwords

# Create password file
echo "mypassword" > ~/.tq/passwords/dev
chmod 0600 ~/.tq/passwords/dev

# Create config file
cat > ~/.tq/config.toml <<EOF
[profiles.dev]
host = "dev.company.com"
port = 1025
database = "development"
user = "alice"
logmech = "TD2"
password_file = "~/.tq/passwords/dev"
EOF

# Use profile
tq --profile dev query "SELECT CURRENT_DATE"
# No password prompt needed
```

### Use Case 3: Multiple Environments

```bash
# Config with dev, staging, prod profiles
cat > ~/.tq/config.toml <<EOF
[defaults]
format = "table"
timing = true

[profiles.dev]
host = "dev.company.com"
database = "development"
user = "alice"
password_file = "~/.tq/passwords/dev"

[profiles.staging]
host = "staging.company.com"
database = "staging"
user = "alice"
password_file = "~/.tq/passwords/staging"

[profiles.prod]
host = "prod.company.com"
database = "production"
user = "alice"
logmech = "LDAP"
password_file = "~/.tq/passwords/prod"
EOF

# Switch between environments easily
tq --profile dev query "SELECT COUNT(*) FROM users"
tq --profile staging query "SELECT COUNT(*) FROM users"
tq --profile prod query "SELECT COUNT(*) FROM users"
```

### Use Case 4: Override Profile Settings

```bash
# Profile has database = "development"
# Override for one-off query against different database
tq --profile dev --database backup_db query "SELECT * FROM users"

# Use different format for one query
tq --profile dev --format json query "SELECT * FROM users"
```

### Use Case 5: REPL with Profile

```bash
# Start REPL with profile
tq --profile dev repl

# Once in REPL, use /logon to switch profiles
/logon --profile prod
```
