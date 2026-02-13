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
3. **Project config file** - `.tq.toml` (if exists in current directory or parent directories)
4. **Environment variables** - `TQ_*` variables
5. **Command-line arguments** - Flags and options

### Precedence Examples

**Example: Format preference**
```bash
# Built-in default: table
# User config: format = "json"
# Project config: format = "csv"
# Environment: TQ_FORMAT=yaml
# CLI flag: --format json

# Result: json (CLI flag wins)
```

**Example: Connection details with project config**
```bash
# User config profile 'dev': host = "dev.company.com", database = "devdb"
# Project config profile 'dev': database = "project_db"
# Environment: TQ_DATABASE=testdb
# CLI flag: --profile dev

# Result: host = "dev.company.com", database = "testdb"
# (User profile provides host, project config overrides database, env var overrides both)
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

## Project Configuration File

### Overview

The project configuration file (`.tq.toml`) enables **team-shared configuration** and project-specific settings. It complements the user configuration file (`~/.tq/config.toml`) by providing configuration that applies to all team members working on a project.

**Key Use Cases:**
- Team-shared connection profiles (dev, staging, prod environments)
- Project-specific default settings (format preferences, database defaults)
- Standardized configurations across the team
- Version-controlled connection metadata (without credentials)

**Design Principles:**
- **Project config overrides user config** - Team settings take precedence over personal settings
- **User config provides credentials** - Individual users store passwords in their personal config
- **Project config is version-controlled** - `.tq.toml` can be safely committed to git (never contains passwords)

### File Location and Discovery

**REQ-PROJ-001: File Location**

The project configuration file SHALL be named `.tq.toml` and located in the project directory.

**REQ-PROJ-002: Directory Walking**

When `tq` starts, it SHALL search for `.tq.toml` using directory walking:

1. **REQ-PROJ-002.1** - Start in current working directory
2. **REQ-PROJ-002.2** - Check if `.tq.toml` exists in current directory
3. **REQ-PROJ-002.3** - If not found, move up one directory level (parent directory)
4. **REQ-PROJ-002.4** - Repeat until `.tq.toml` is found OR filesystem root is reached
5. **REQ-PROJ-002.5** - If filesystem root is reached without finding `.tq.toml`, no project config is loaded
6. **REQ-PROJ-002.6** - The first `.tq.toml` found stops the search (do not continue to parent directories)

**Rationale:** Directory walking enables project config to be placed at repository root while allowing `tq` to be invoked from any subdirectory within the project.

**Example Directory Structure:**
```
/home/alice/projects/analytics/          <- .tq.toml located here
├── .tq.toml
├── scripts/
│   └── queries/
│       └── daily_report.sql         <- tq invoked from here
└── data/
```

When invoking `tq` from `/home/alice/projects/analytics/scripts/queries/`, the tool will walk up and find `.tq.toml` in `/home/alice/projects/analytics/`.

**REQ-PROJ-003: Search Path Caching**

Once `.tq.toml` is found, the resolved path SHALL be cached for the duration of the tool's execution.

**REQ-PROJ-004: Symlink Handling**

When walking directories, `tq` SHALL resolve symlinks to their real paths to prevent infinite loops.

### File Format

**REQ-PROJ-005: TOML Structure**

The `.tq.toml` file SHALL use the same TOML structure as `~/.tq/config.toml`:

```toml
# .tq.toml - Project configuration

# Project defaults (apply to all team members)
[defaults]
format = "table"
timing = true

# Shared connection profiles
[profiles.dev]
host = "dev.company.com"
port = 1025
database = "development"
# Note: No password_file here - users provide their own credentials

[profiles.staging]
host = "staging.company.com"
database = "staging"

[profiles.prod]
host = "prod.company.com"
database = "production"
logmech = "LDAP"
```

**REQ-PROJ-006: Supported Sections**

The `.tq.toml` file SHALL support the same sections as user config:
- `[defaults]` - Project-wide preference defaults
- `[profiles.<name>]` - Team-shared connection profiles

### Configuration Merging

**REQ-PROJ-007: Precedence Rules**

When both user config and project config are present, values are merged with project config taking precedence:

1. **REQ-PROJ-007.1** - Load user config (`~/.tq/config.toml`) first
2. **REQ-PROJ-007.2** - Load project config (`.tq.toml`) second
3. **REQ-PROJ-007.3** - For `[defaults]` section: Project values override user values
4. **REQ-PROJ-007.4** - For `[profiles]` section: Project profiles override user profiles with same name
5. **REQ-PROJ-007.5** - User-only profiles remain available (not removed by project config)

**Example Merging Behavior:**

**User config (`~/.tq/config.toml`):**
```toml
[defaults]
format = "json"
timing = false

[profiles.dev]
host = "localhost"
database = "my_local_dev"
user = "alice"
password_file = "~/.tq/passwords/local"

[profiles.personal]
host = "my-home-server.local"
database = "sandbox"
user = "alice"
```

**Project config (`.tq.toml`):**
```toml
[defaults]
format = "table"
timing = true

[profiles.dev]
host = "dev.company.com"
database = "shared_dev"
```

**Resulting merged configuration:**
```toml
[defaults]
format = "table"        # Project config wins
timing = true           # Project config wins

[profiles.dev]
host = "dev.company.com"    # Project config overrides user config
database = "shared_dev"     # Project config overrides user config
user = "alice"              # From user config (not specified in project config)
password_file = "~/.tq/passwords/local"  # From user config

[profiles.personal]
# Remains available (user-only profile)
host = "my-home-server.local"
database = "sandbox"
user = "alice"
```

**REQ-PROJ-008: Profile Field Merging**

When a profile exists in both user and project config with the same name, fields are merged at the field level:

1. **REQ-PROJ-008.1** - Fields specified in project config override user config fields
2. **REQ-PROJ-008.2** - Fields NOT specified in project config are inherited from user config
3. **REQ-PROJ-008.3** - This enables project config to specify connection metadata (host, database) while user config provides credentials (user, password_file)

**Rationale:** Field-level merging enables separation of concerns: project config defines "where to connect", user config defines "who I am and my credentials".

### Profile Resolution

**REQ-PROJ-009: Combined Profile Listing**

The `tq profiles` command SHALL display both user and project profiles:

```bash
$ tq profiles

Available profiles:

From user config (~/.tq/config.toml):
  personal
    Host:     my-home-server.local
    Database: sandbox
    User:     alice

From project config (.tq.toml):
  dev
    Host:     dev.company.com:1025
    Database: shared_dev

  staging
    Host:     staging.company.com:1025
    Database: staging

From both (merged):
  dev
    Host:     dev.company.com:1025      [project]
    Database: shared_dev                [project]
    User:     alice                     [user]
```

**REQ-PROJ-009.1** - User-only profiles shall be listed under "From user config"
**REQ-PROJ-009.2** - Project-only profiles shall be listed under "From project config"
**REQ-PROJ-009.3** - Profiles present in both shall be listed under "From both (merged)" with source indicators
**REQ-PROJ-009.4** - Source indicators `[user]` and `[project]` shall show which config provided each field

**REQ-PROJ-010: Profile Selection**

When using `--profile <name>`, the tool SHALL:

1. **REQ-PROJ-010.1** - Check project config for profile named `<name>`
2. **REQ-PROJ-010.2** - Check user config for profile named `<name>`
3. **REQ-PROJ-010.3** - If found in both, merge fields (project overrides user)
4. **REQ-PROJ-010.4** - If found in only one, use that profile
5. **REQ-PROJ-010.5** - If not found in either, produce error message listing available profiles

### Error Handling

**REQ-PROJ-011: Invalid TOML**

If `.tq.toml` contains invalid TOML syntax, the tool SHALL:

1. **REQ-PROJ-011.1** - Display clear error message with file path and line number
2. **REQ-PROJ-011.2** - Show the TOML parse error details
3. **REQ-PROJ-011.3** - Exit with non-zero status code
4. **REQ-PROJ-011.4** - Do NOT fall back to user config only (invalid project config is an error condition)

**Example Error:**
```
Error: Failed to parse project configuration file
File: /home/alice/projects/analytics/.tq.toml
Line: 8, Column: 15

TOML parse error: expected '=', found ':'

[profiles.dev]
host: "dev.company.com"
     ^

Fix: Use '=' instead of ':' in TOML files
```

**REQ-PROJ-012: File Permission Errors**

If `.tq.toml` exists but cannot be read due to permissions, the tool SHALL:

1. **REQ-PROJ-012.1** - Display clear error message indicating permission denied
2. **REQ-PROJ-012.2** - Show the file path and current permissions
3. **REQ-PROJ-012.3** - Suggest fix (chmod command)
4. **REQ-PROJ-012.4** - Exit with non-zero status code

**Example Error:**
```
Error: Cannot read project configuration file
File: /home/alice/projects/analytics/.tq.toml
Permission denied

Current permissions: 0000 (no access)
Required: File must be readable

Fix: chmod 0644 /home/alice/projects/analytics/.tq.toml
```

**REQ-PROJ-013: Conflicting Configuration Values**

When project config and user config provide conflicting values, the tool SHALL:

1. **REQ-PROJ-013.1** - Use project config value (no error, no warning)
2. **REQ-PROJ-013.2** - If `--verbose` flag is present, log which config source won for each setting

**Rationale:** Project config intentionally overrides user config - this is not an error condition. Verbose logging helps users understand config resolution.

### Security Considerations

**REQ-PROJ-014: Password Prohibition**

Project configuration files SHOULD NOT contain passwords or password files:

**Rationale:** Project config is intended to be version-controlled and shared among team members. Passwords are individual credentials.

**Best Practice:**
- **Project config (`.tq.toml`)**: Connection metadata only (host, port, database, logmech)
- **User config (`~/.tq/config.toml`)**: Individual credentials (user, password_file)

**Example Safe Project Config:**
```toml
# .tq.toml - Safe to commit to git
[profiles.dev]
host = "dev.company.com"
database = "development"
logmech = "LDAP"
# No user, no password_file - each team member provides their own
```

**Example User Config:**
```toml
# ~/.tq/config.toml - Personal, not committed
[profiles.dev]
user = "alice"
password_file = "~/.tq/passwords/dev"
```

**REQ-PROJ-015: File Permission Recommendations**

Project config file permissions SHOULD follow these guidelines:

- **Recommended for git-committed files**: `0644` (readable by all)
- **Accepted**: Any permissions (project config typically doesn't contain secrets)
- **Warning NOT issued**: Unlike user config, no permission warnings for project config

**Rationale:** Project config is designed to be version-controlled and shared, so restrictive permissions are not required.

### Integration with Existing Commands

**REQ-PROJ-016: `tq profiles` Integration**

The `tq profiles` command SHALL be updated to show profiles from both sources (see REQ-PROJ-009).

**REQ-PROJ-017: `--profile` Flag Behavior**

The `--profile <name>` flag SHALL work transparently with both user and project profiles (see REQ-PROJ-010).

**REQ-PROJ-018: Help Text Updates**

The tool's help text and documentation SHALL be updated to mention project config:

**Example `tq --help` excerpt:**
```
CONFIGURATION:
  Configuration is loaded from multiple sources in this order:
    1. Built-in defaults
    2. User config (~/.tq/config.toml)
    3. Project config (.tq.toml in current directory or parents)
    4. Environment variables (TQ_*)
    5. Command-line arguments

  Project config enables team-shared profiles and settings.
  See 'tq help config' for details.
```

### Common Use Cases

**Use Case 1: Team-Shared Connection Profiles**

A development team wants all members to use consistent connection settings for dev/staging/prod environments.

**Project config (`.tq.toml` - committed to git):**
```toml
[profiles.dev]
host = "dev-teradata.company.com"
database = "dev_analytics"
logmech = "LDAP"

[profiles.staging]
host = "staging-teradata.company.com"
database = "staging_analytics"
logmech = "LDAP"

[profiles.prod]
host = "prod-teradata.company.com"
database = "prod_analytics"
logmech = "LDAP"
```

**User config (`~/.tq/config.toml` - personal, not committed):**
```toml
[profiles.dev]
user = "alice"
password_file = "~/.tq/passwords/dev"

[profiles.staging]
user = "alice"
password_file = "~/.tq/passwords/staging"

[profiles.prod]
user = "alice"
password_file = "~/.tq/passwords/prod"
```

**Usage:**
```bash
# Works for all team members, uses team-shared host + individual credentials
tq --profile dev query "SELECT COUNT(*) FROM users"
```

**Use Case 2: Project-Specific Defaults**

A project wants consistent output formatting and timing for all queries.

**Project config (`.tq.toml`):**
```toml
[defaults]
format = "csv"
timing = true
```

**Effect:**
- All team members get CSV output by default
- Query timing is enabled for performance awareness
- Individual users can still override with `--format json` flag

**Use Case 3: User Override**

A user wants to use their local development database instead of the team's shared dev environment.

**Project config (`.tq.toml`):**
```toml
[profiles.dev]
host = "dev-teradata.company.com"
database = "dev_analytics"
```

**User config (`~/.tq/config.toml`):**
```toml
[profiles.dev]
host = "localhost"
database = "my_local_dev"
user = "dbc"
password_file = "~/.tq/passwords/local"
```

**Note:** This does NOT work as intended - project config wins!

**Solution:** Create a separate user-only profile:
```toml
[profiles.local]
host = "localhost"
database = "my_local_dev"
user = "dbc"
password_file = "~/.tq/passwords/local"
```

**Usage:**
```bash
tq --profile local query "SELECT 1"  # Uses personal local database
```

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
