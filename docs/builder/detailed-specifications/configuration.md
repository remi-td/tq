# Configuration and Credential Management

**Version:** 2.0.0
**Last Updated:** 2026-01-21
**Owner:** cli-ux-designer agent
**Status:** Active Specification (Sprint 16)

---

## Table of Contents

1. [Overview](#71-overview)
2. [Configuration Hierarchy](#72-configuration-hierarchy)
3. [User Configuration File](#73-user-configuration-file)
4. [Connection Profiles](#74-connection-profiles)
5. [Environment Variables](#75-environment-variables)
6. [Credential Management](#76-credential-management)
7. [Error Handling](#77-error-handling)
8. [Help Text](#78-help-text)
9. [Future Features](#79-future-features)

---

## 7.1 Overview

### 7.1.1 Design Principles

Configuration management follows these principles:

- **Optional by default**: Tool works without any configuration file
- **Convention over configuration**: Sensible defaults for common use cases
- **Security first**: Never encourage inline passwords, always suggest password_file
- **Clear precedence**: CLI arguments override everything
- **Fail fast**: Invalid configuration produces clear error messages with guidance

### 7.1.2 Sprint 16 Scope

**Implemented in Sprint 16:**
- User configuration file (`~/.tq/config.toml`)
- Connection profiles in config file
- Default preferences in config file
- `--profile <name>` flag for profile selection
- Configuration file validation and error handling

**Not in Sprint 16 (Future):**
- Project-level config file (`.tq.toml`)
- System config (`/etc/tq/config.toml`)
- Keyring integration
- Profile management commands (`tq profile create/update/delete`)
- Config validation command (`tq config validate`)

---

## 7.2 Configuration Hierarchy

Configuration values are loaded in this order (later overrides earlier):

1. **Built-in defaults** - Hardcoded sensible defaults
2. **User config file** - `~/.tq/config.toml` (if exists)
3. **Environment variables** - `TQ_*` variables
4. **Command-line arguments** - Flags and options

### 7.2.1 Precedence Examples

**Example 1: Format preference**
```bash
# Built-in default: table
# Config file: format = "json"
# Environment: TQ_FORMAT=csv
# CLI flag: --format json

# Result: json (CLI flag wins)
```

**Example 2: Connection details**
```bash
# Config file profile 'dev': host = "dev.company.com", database = "devdb"
# Environment: TQ_DATABASE=testdb
# CLI flag: --profile dev

# Result: host = "dev.company.com", database = "testdb"
# (Profile provides host, env var overrides database)
```

### 7.2.2 Profile Selection Precedence

When using `--profile <name>`, the profile values are loaded as base values, then overridden by:
1. Environment variables
2. Command-line arguments

```bash
# Profile 'prod': host=prod.company.com, database=production, user=alice
# Command: tq --profile prod --database staging query "SELECT 1"

# Result: host=prod.company.com, database=staging, user=alice
```

---

## 7.3 User Configuration File

### 7.3.1 File Location

The user configuration file is located at:

- **macOS/Linux**: `~/.tq/config.toml`
- **Windows**: `%USERPROFILE%\.tq\config.toml`

The directory `~/.tq/` is created automatically if it doesn't exist.

### 7.3.2 File Format

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

### 7.3.3 Defaults Section

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

### 7.3.4 File Permissions

For security, `tq` checks the config file permissions:

- **Recommended**: `0600` (read/write for owner only)
- **Accepted**: `0644` (world-readable, since passwords should not be inline)
- **Rejected**: `0666` or similar (world-writable)

**Warning issued if too permissive:**
```
Warning: Configuration file ~/.tq/config.toml has permissive permissions (0644)
Recommendation: chmod 0600 ~/.tq/config.toml
```

---

## 7.4 Connection Profiles

### 7.4.1 Profile Structure

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

### 7.4.2 Using Profiles

Profiles are selected with the `--profile <name>` flag:

```bash
# Use 'dev' profile
tq --profile dev query "SELECT CURRENT_DATE"

# Use 'prod' profile, override database
tq --profile prod --database backup_db query "SELECT COUNT(*) FROM users"

# Use 'local' profile in REPL
tq --profile local repl
```

### 7.4.3 Profile Field Overrides

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

### 7.4.4 Listing Profiles

Users can see available profiles using the `--help` flag or by checking their config file.

**Future enhancement** (not Sprint 16):
```bash
# List all profiles (Sprint 17+)
tq profile list
```

---

## 7.5 Environment Variables

Environment variables provide a way to configure `tq` without a config file or CLI flags.

### 7.5.1 Supported Variables

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

### 7.5.2 Usage Examples

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

### 7.5.3 Security Considerations

**TQ_PASSWORD is discouraged:**
- Visible in process list (`ps aux`)
- Stored in shell history
- Logged by system audit tools

**Recommended alternatives:**
1. Use `password_file` in profile
2. Use password file flag: `--password-file ~/.tq/passwords/dev`
3. Allow interactive prompt (omit password entirely)

---

## 7.6 Credential Management

### 7.6.1 Security Principles

1. **Never use passwords in CLI arguments** - visible in `ps`, shell history
2. **Never log passwords** - sanitize all debug output
3. **Use file permissions** - `chmod 0600` for credential files
4. **Prefer password files** - secure file-based credentials
5. **Support password prompts** - interactive secure input when password missing

### 7.6.2 Password Sources (Priority Order)

When a password is needed, `tq` checks these sources in order:

1. **Command-line connection string** - `--logon user:pass@host` (discouraged, but supported)
2. **Password file from CLI** - `--password-file /path/to/file`
3. **Password file from profile** - `password_file = "~/.tq/passwords/dev"`
4. **Environment variable** - `TQ_PASSWORD` (discouraged)
5. **Interactive prompt** - Secure input if password still missing

### 7.6.3 Password File Format

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

**File permissions:**
- **Required**: Owner read-only (`0600`) or owner read-write (`0600`)
- **Rejected**: Group or world readable/writable

If permissions are too permissive, `tq` refuses to read the file:
```
Error: Password file has insecure permissions: ~/.tq/passwords/dev
Current permissions: 0644
Required permissions: 0600

Fix: chmod 0600 ~/.tq/passwords/dev
```

### 7.6.4 Interactive Password Prompt

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

### 7.6.5 Inline Passwords (Discouraged)

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

---

## 7.7 Error Handling

### 7.7.1 Config File Not Found

**Behavior**: No error, tool uses defaults

Config files are optional. If `~/.tq/config.toml` doesn't exist, `tq` proceeds with built-in defaults.

### 7.7.2 Invalid TOML Syntax

**Error:**
```
Error: Failed to parse configuration file: ~/.tq/config.toml
Line 12: Expected '=' after key

12 | host "dev.company.com"
   |      ^ Expected '='

Fix: host = "dev.company.com"

TOML syntax reference: https://toml.io/en/
```

### 7.7.3 Invalid Configuration Values

**Example: Invalid format**
```
Error: Invalid configuration in ~/.tq/config.toml
Section: [defaults]
Field: format
Value: "xml"
Valid values: "table", "json", "csv"

Fix: Change 'format = "xml"' to one of the valid values
```

**Example: Invalid editor_mode**
```
Error: Invalid configuration in ~/.tq/config.toml
Section: [defaults]
Field: editor_mode
Value: "vim"
Valid values: "emacs", "vi"

Fix: Change 'editor_mode = "vim"' to "emacs" or "vi"
```

### 7.7.4 Profile Not Found

**Error:**
```
Error: Profile 'staging' not found
Config file: ~/.tq/config.toml

Available profiles:
  - dev
  - prod
  - local

Fix: Use --profile dev or add [profiles.staging] section to config file
```

### 7.7.5 Missing Required Fields in Profile

**Error:**
```
Error: Invalid profile 'dev' in ~/.tq/config.toml
Missing required field: host

Profile must include:
  host = "hostname"

Optional fields:
  port = 1025
  database = "dbname"
  user = "username"
  logmech = "TD2"
  password_file = "~/.tq/passwords/dev"
  timeout = "30s"
```

### 7.7.6 Invalid Profile Fields

**Example: Invalid port**
```
Error: Invalid profile 'dev' in ~/.tq/config.toml
Field: port
Value: "not_a_number"
Expected: integer (e.g., 1025)

Fix: port = 1025
```

**Example: Invalid logmech**
```
Error: Invalid profile 'dev' in ~/.tq/config.toml
Field: logmech
Value: "INVALID"
Valid values: "TD2", "LDAP", "KRB5", "TDNEGO"

Fix: logmech = "TD2"
```

### 7.7.7 Password File Not Found

**Error:**
```
Error: Password file not found: ~/.tq/passwords/dev
Specified in profile: dev
Config file: ~/.tq/config.toml

Fix: Create password file:
  echo "your_password" > ~/.tq/passwords/dev
  chmod 0600 ~/.tq/passwords/dev
```

### 7.7.8 Password File Insecure Permissions

**Error:**
```
Error: Password file has insecure permissions: ~/.tq/passwords/dev
Current permissions: 0644 (readable by group and others)
Required permissions: 0600 (owner read-write only)

Security risk: Password file is readable by other users

Fix: chmod 0600 ~/.tq/passwords/dev
```

---

## 7.8 Help Text

### 7.8.1 Configuration Overview Help

```bash
$ tq help config
tq Configuration

CONFIGURATION FILE
    tq looks for a user configuration file at:
      ~/.tq/config.toml  (macOS/Linux)
      %USERPROFILE%\.tq\config.toml  (Windows)

    The configuration file is optional. If not present, built-in defaults are used.

FILE FORMAT (TOML)
    [defaults]
    format = "table"              # Output format: table, json, csv
    editor_mode = "emacs"         # REPL editor: emacs, vi
    syntax_highlighting = true    # Enable SQL syntax highlighting
    paging = true                 # Enable result paging
    timing = false                # Show query execution time

    [profiles.dev]
    host = "dev.company.com"
    port = 1025
    database = "development"
    user = "alice"
    logmech = "TD2"
    password_file = "~/.tq/passwords/dev"

PRECEDENCE ORDER
    Configuration is loaded in this order (later overrides earlier):
      1. Built-in defaults
      2. User config file (~/.tq/config.toml)
      3. Environment variables (TQ_*)
      4. Command-line arguments

EXAMPLES
    # Use a connection profile
    tq --profile dev query "SELECT CURRENT_DATE"

    # Override profile database
    tq --profile dev --database staging query "SELECT 1"

    # List available profiles
    cat ~/.tq/config.toml | grep '^\[profiles\.'

See 'tq help credentials' for password management
```

### 7.8.2 Global --profile Flag Help

Added to `tq --help`:

```
GLOBAL OPTIONS:
    ...
    --profile <NAME>         Select connection profile from config file
    ...

CONFIGURATION:
    Connection profiles can be defined in ~/.tq/config.toml
    See 'tq help config' for details
```

### 7.8.3 Credentials Help

```bash
$ tq help credentials
tq Credential Management

PASSWORD SECURITY
    NEVER use passwords in command-line arguments:
      tq -l "user:pass@host" query "SELECT 1"  # INSECURE: visible in ps, history

    ALWAYS use password files:
      tq -l "user@host" --password-file ~/.tq/passwords/dev query "SELECT 1"

    Or configure in profile:
      [profiles.dev]
      password_file = "~/.tq/passwords/dev"

PASSWORD FILES
    Format: Single line containing password
      echo "mypassword" > ~/.tq/passwords/dev
      chmod 0600 ~/.tq/passwords/dev

    Required permissions: 0600 (owner read-write only)

PASSWORD SOURCES (priority order)
    1. Connection string (discouraged): user:pass@host
    2. --password-file flag
    3. Profile password_file field
    4. TQ_PASSWORD environment variable (discouraged)
    5. Interactive prompt (secure)

INTERACTIVE PROMPT
    If no password is provided, tq prompts securely:
      $ tq -l "user@host" query "SELECT 1"
      Password: ****

See 'tq help config' for configuration details
```

---

## 7.9 Future Features

### 7.9.1 Project-Level Config (Sprint 17+)

Support for project-specific configuration in `.tq.toml`:

```toml
# .tq.toml (project root, committed to git)
[connection]
host = "shared-dev.company.com"
database = "team_database"
# Note: Never commit passwords

[defaults]
format = "json"  # Team prefers JSON
```

**Precedence:**
1. Built-in defaults
2. System config (`/etc/tq/config.toml`)
3. User config (`~/.tq/config.toml`)
4. Project config (`.tq.toml`)
5. Environment variables
6. Command-line arguments

### 7.9.2 Profile Management Commands (Sprint 17+)

```bash
# Create profile interactively
tq profile create staging

# List profiles
tq profile list

# Show profile details
tq profile show dev

# Update profile
tq profile update dev --timeout 60s

# Delete profile
tq profile delete old-dev
```

### 7.9.3 Keyring Integration (Sprint 18+)

OS-native secure credential storage:

```bash
# Store password in OS keyring
tq password set dev
Enter password: ****

# Use stored password automatically
tq --profile dev query "SELECT 1"

# List stored passwords
tq password list

# Delete stored password
tq password delete dev
```

### 7.9.4 Config Validation Command (Sprint 18+)

```bash
# Validate config file syntax and values
tq config validate

# Output:
# ✓ Config file syntax is valid
# ✓ All profiles are valid
# ✓ All password files exist and have correct permissions
#
# Available profiles:
#   - dev (host: dev.company.com)
#   - prod (host: prod.company.com)
#   - local (host: localhost)
```

### 7.9.5 SSL/TLS Configuration (Sprint 19+)

```toml
[profiles.secure]
host = "secure.company.com"
port = 1025
ssl = true
ssl_mode = "require"  # disable, allow, prefer, require, verify-ca, verify-full
ssl_ca_file = "~/.tq/certs/ca-cert.pem"
ssl_cert_file = "~/.tq/certs/client-cert.pem"
ssl_key_file = "~/.tq/certs/client-key.pem"
```

---

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

---

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-21 | 2.0.0 | Complete Sprint 16 specification: detailed configuration file format, profiles, precedence rules, error handling, help text, examples | cli-ux-designer |
| 2026-01-18 | 1.1.0 | Minor updates to existing structure | cli-ux-designer |
| 2026-01-16 | 1.0.0 | Initial configuration specification | cli-ux-designer |
