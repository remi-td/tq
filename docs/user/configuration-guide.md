# Configuration Guide

This guide shows you how to configure tq using configuration files, environment variables, and connection profiles.

## Overview

tq supports multiple configuration methods to match your workflow:

- **No configuration needed** - Works with command-line arguments only
- **User configuration** - Personal settings in `~/.tq/config.toml`
- **Project configuration** - Team-shared settings in `.tq.toml` (project root)
- **Environment variables** - Quick overrides with `TQ_*` variables
- **Command-line flags** - Highest priority, overrides everything

## Quick Start

### First-Time Setup

Create a basic user configuration:

```bash
# Create tq config directory
mkdir -p ~/.tq/passwords

# Create a password file
echo "mypassword" > ~/.tq/passwords/dev
chmod 0600 ~/.tq/passwords/dev

# Create config file
cat > ~/.tq/config.toml <<EOF
[profiles.dev]
host = "dev.company.com"
port = 1025
database = "development"
user = "alice"
password_file = "~/.tq/passwords/dev"
EOF

# Use the profile
tq --profile dev query "SELECT CURRENT_DATE"
```

## Configuration Hierarchy

Settings are loaded in this order (later overrides earlier):

1. **Built-in defaults** - Hardcoded sensible defaults
2. **User config** - `~/.tq/config.toml` (your personal settings)
3. **Project config** - `.tq.toml` (team-shared settings)
4. **Environment variables** - `TQ_*` variables
5. **Command-line arguments** - Highest priority

### Precedence Example

```bash
# Built-in default: format = "table"
# User config:     format = "json"
# Project config:  format = "csv"
# Environment:     TQ_FORMAT=yaml
# CLI flag:        --format json

tq --format json query "SELECT 1"
# Result uses: json (CLI flag wins)
```

## User Configuration

### File Location

User configuration is stored at:

- **macOS/Linux**: `~/.tq/config.toml`
- **Windows**: `%USERPROFILE%\.tq\config.toml`

The directory is created automatically if it doesn't exist.

### File Structure

Configuration uses TOML format for readability and type safety.

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
password_file = "~/.tq/passwords/dev"

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

The `[defaults]` section configures tool behavior. All fields are optional.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `format` | string | `"table"` | Output format: `table`, `json`, `csv` |
| `editor_mode` | string | `"emacs"` | REPL keybindings: `emacs`, `vi` |
| `syntax_highlighting` | boolean | `true` | SQL syntax highlighting in REPL |
| `paging` | boolean | `true` | Result paging in REPL |
| `timing` | boolean | `false` | Show query execution time |

**Minimal example:**

```toml
[defaults]
format = "json"
timing = true
```

### Connection Profiles

Profiles store connection settings for different databases or environments.

**Profile structure:**

```toml
[profiles.name]
host = "hostname"              # Required: Database hostname
port = 1025                    # Optional: Port (default: 1025)
database = "dbname"            # Optional: Database name
user = "username"              # Optional: Username
logmech = "TD2"                # Optional: Auth mechanism (default: TD2)
password_file = "~/.tq/passwords/name"  # Optional: Path to password file
timeout = "30s"                # Optional: Connection timeout (default: 30s)
```

**Using profiles:**

```bash
# Use profile
tq --profile dev query "SELECT CURRENT_DATE"

# Override profile settings
tq --profile dev --database staging query "SELECT 1"

# Start REPL with profile
tq --profile prod repl
```

### Security Best Practices

**Password Files:**

Never store passwords directly in config files. Use password files instead:

```bash
# Create secure password directory
mkdir -p ~/.tq/passwords
chmod 0700 ~/.tq/passwords

# Create password file
echo "mypassword" > ~/.tq/passwords/dev
chmod 0600 ~/.tq/passwords/dev
```

**File Permissions:**

tq enforces strict permissions on password files:

- **Required**: `0600` (owner read/write only)
- **Rejected**: Any permissions allowing group or world access

If permissions are too permissive, tq refuses to read the file:

```
Error: Password file has insecure permissions: ~/.tq/passwords/dev
Current permissions: 0644 (readable by group and others)
Required permissions: 0600 (owner read-write only)

Fix: chmod 0600 ~/.tq/passwords/dev
```

**Config File Permissions:**

Config file permissions are recommended but not enforced:

- **Recommended**: `0600` (owner read/write only)
- **Accepted**: `0644` (world-readable)

A warning is issued if permissions are too permissive, but tq continues to work.

## Project Configuration

### Overview

Project configuration (`.tq.toml`) enables **team-shared settings** for everyone working on a project. It complements user configuration by providing standardized connection metadata that applies to all team members.

**Key Benefits:**

- Team-shared connection profiles (dev, staging, prod)
- Consistent defaults across the team
- Version-controlled connection metadata (safe to commit)
- Individual users still provide their own credentials

**Design Philosophy:**

- **Project config**: Connection metadata (host, database, defaults)
- **User config**: Personal credentials (user, password_file)
- **Project overrides user**: Team settings take precedence

### File Location and Discovery

Place `.tq.toml` at your project root (usually repository root):

```
/home/alice/projects/analytics/
├── .tq.toml                    <- Project config here
├── scripts/
│   └── queries/
│       └── report.sql          <- Can run tq from here
└── data/
```

**How tq finds `.tq.toml`:**

1. Start in current working directory
2. Check if `.tq.toml` exists
3. If not found, move up one directory (parent)
4. Repeat until `.tq.toml` found or filesystem root reached

This means you can invoke `tq` from any subdirectory within your project, and it will find the project configuration at the repository root.

### File Structure

Project config uses the same TOML structure as user config:

```toml
# .tq.toml - Project configuration (safe to commit to git)

# Project defaults (apply to all team members)
[defaults]
format = "table"
timing = true

# Shared connection profiles
[profiles.dev]
host = "dev.company.com"
port = 1025
database = "development"
# Note: No user or password_file - users provide their own

[profiles.staging]
host = "staging.company.com"
database = "staging"

[profiles.prod]
host = "prod.company.com"
database = "production"
logmech = "LDAP"
```

### Configuration Merging

When both user and project config exist, values merge with **project config taking precedence**.

**Example:**

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
format = "table"           # Project config wins
timing = true              # Project config wins

[profiles.dev]
host = "dev.company.com"   # Project config overrides user config
database = "shared_dev"    # Project config overrides user config
user = "alice"             # From user config (not in project config)
password_file = "~/.tq/passwords/local"  # From user config

[profiles.personal]
# Remains available (user-only profile)
host = "my-home-server.local"
database = "sandbox"
user = "alice"
```

**Key Points:**

- Fields in project config override user config
- Fields NOT in project config are inherited from user config
- User-only profiles remain available
- This enables separation: project defines "where", user defines "who"

### Team Workflow Example

**Step 1: Set up project config (repository maintainer)**

Create `.tq.toml` at repository root:

```toml
# .tq.toml - Safe to commit to git

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

Commit and push `.tq.toml`:

```bash
git add .tq.toml
git commit -m "Add tq project configuration for team profiles"
git push
```

**Step 2: Team members check out repository**

```bash
git clone https://github.com/company/analytics.git
cd analytics
```

**Step 3: Team members add personal credentials**

Each team member creates their user config with credentials:

```toml
# ~/.tq/config.toml - Personal, not committed

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

**Step 4: Everyone uses consistent profiles**

```bash
# Works for all team members
# Uses team-shared host + individual credentials
cd analytics/scripts
tq --profile dev query "SELECT COUNT(*) FROM users"
```

**Result:**

- All team members connect to the same hosts/databases
- Everyone uses their own credentials
- No passwords committed to git
- Consistent experience across the team

### Listing Profiles

The `tq profiles` command shows profiles from both user and project config:

```bash
$ tq profiles

Available profiles:

From user config (~/.tq/config.toml):
  personal
    Host:     my-home-server.local
    Database: sandbox
    User:     alice

From project config (.tq.toml):
  staging
    Host:     staging.company.com:1025
    Database: staging

From both (merged):
  dev
    Host:     dev.company.com:1025      [project]
    Database: shared_dev                [project]
    User:     alice                     [user]
```

**Indicators:**

- **[project]** - Value from project config
- **[user]** - Value from user config

### Security Considerations

**What to commit to `.tq.toml`:**

- Connection metadata (host, port, database)
- Authentication mechanisms (logmech)
- Default preferences (format, timing)

**What NOT to commit:**

- Usernames (if personal)
- Passwords or password_file paths
- Personal credentials

**Safe project config example:**

```toml
# .tq.toml - Safe to commit

[profiles.prod]
host = "prod.company.com"
database = "production"
logmech = "LDAP"
# No user, no password_file - each team member provides their own
```

**Corresponding user config (NOT committed):**

```toml
# ~/.tq/config.toml - Personal, private

[profiles.prod]
user = "alice"
password_file = "~/.tq/passwords/prod"
```

### User Overrides

If you need to use a different database than the team standard, create a separate user-only profile:

**Project config defines `dev` profile:**

```toml
[profiles.dev]
host = "dev.company.com"
database = "dev_analytics"
```

**You want to use your local database instead:**

```toml
# ~/.tq/config.toml

# Don't override 'dev' profile - project config will win
# Instead, create a new profile:
[profiles.local]
host = "localhost"
database = "my_local_dev"
user = "dbc"
password_file = "~/.tq/passwords/local"
```

**Usage:**

```bash
# Use team's dev environment
tq --profile dev query "SELECT 1"

# Use your local database
tq --profile local query "SELECT 1"
```

## Environment Variables

Environment variables provide quick configuration overrides without editing files.

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
# Will prompt for password
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

**Temporary override:**

```bash
# Override database for one command
TQ_DATABASE=testing tq --profile dev query "SELECT 1"
```

### Security Warning

**`TQ_PASSWORD` is discouraged:**

- Visible in process list (`ps aux`)
- Stored in shell history
- Logged by system audit tools

**Better alternatives:**

1. Use `password_file` in profile (best)
2. Use `--password-file` flag (good)
3. Allow interactive prompt (acceptable)

## Common Workflows

### Workflow 1: Personal Development

Simple setup for individual use:

```toml
# ~/.tq/config.toml

[defaults]
format = "table"
timing = true

[profiles.local]
host = "localhost"
port = 1025
database = "testdb"
user = "dbc"
password_file = "~/.tq/passwords/local"
```

### Workflow 2: Multiple Environments

Managing dev, staging, and production:

```toml
# ~/.tq/config.toml

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
```

Switch environments easily:

```bash
tq --profile dev query "SELECT COUNT(*) FROM users"
tq --profile staging query "SELECT COUNT(*) FROM users"
tq --profile prod query "SELECT COUNT(*) FROM users"
```

### Workflow 3: Team-Shared Configuration

Repository with project config:

```toml
# .tq.toml (committed to git)

[defaults]
format = "csv"
timing = true

[profiles.dev]
host = "dev-teradata.company.com"
database = "team_dev"

[profiles.prod]
host = "prod-teradata.company.com"
database = "team_prod"
logmech = "LDAP"
```

Each team member adds credentials:

```toml
# ~/.tq/config.toml (personal)

[profiles.dev]
user = "alice"
password_file = "~/.tq/passwords/dev"

[profiles.prod]
user = "alice"
password_file = "~/.tq/passwords/prod"
```

### Workflow 4: Quick Ad-Hoc Connection

No configuration needed:

```bash
# Connect with command-line arguments
tq -l "alice@dev.company.com:1025/mydb" query "SELECT CURRENT_DATE"
Password: ****

# Or use environment variable
export TQ_LOGON="alice@dev.company.com:1025/mydb"
tq query "SELECT CURRENT_DATE"
Password: ****
```

## Tips and Best Practices

1. **Use profiles for frequently-used connections** - Saves typing and reduces errors
2. **Never commit passwords** - Use `password_file` and keep passwords in `~/.tq/passwords/`
3. **Use project config for teams** - Share connection metadata, keep credentials personal
4. **Set file permissions** - Always `chmod 0600` on password files
5. **Use descriptive profile names** - `dev`, `staging`, `prod` are clearer than `db1`, `db2`
6. **Test profile changes** - Use `tq --profile name query "SELECT 1"` to verify
7. **Document team profiles** - Add comments to `.tq.toml` explaining each profile
8. **Use defaults wisely** - Set project defaults for consistency, allow user overrides
9. **Check merged profiles** - Use `tq profiles` to see how user and project configs merge
10. **Keep user config simple** - Let project config handle complexity, user config adds credentials

## Troubleshooting

### Profile Not Found

```
Error: Profile 'dev' not found

Available profiles:
  - local
  - staging
  - prod
```

**Solution:** Check profile names with `tq profiles` and verify spelling.

### Permission Denied on Password File

```
Error: Password file has insecure permissions: ~/.tq/passwords/dev
Current permissions: 0644
Required permissions: 0600

Fix: chmod 0600 ~/.tq/passwords/dev
```

**Solution:** Run the suggested `chmod` command.

### Project Config Not Found

If `tq` doesn't find `.tq.toml`, it will use only user config. Check:

1. Is `.tq.toml` in the project root?
2. Are you running `tq` from within the project directory tree?
3. Did you check for typos in the filename?

### Unexpected Configuration Values

Use `tq --verbose` to see configuration resolution:

```bash
tq --verbose --profile dev query "SELECT 1"
# Shows which config sources provided each setting
```

### Connection Timeout

```
Error: Connection timeout
```

**Solution:** Increase timeout in profile:

```toml
[profiles.dev]
host = "slow-server.company.com"
timeout = "60s"  # Increase from default 30s
```

## Next Steps

- **[REPL Guide](repl-guide.md)** - Learn interactive mode features
- **[Batch Mode Guide](batch-mode-guide.md)** - Run SQL scripts
- **[Specifications](../specifications/configuration.md)** - Technical configuration details
