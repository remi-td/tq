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

The fastest way to create your first profile is with `tq profile add`:

```bash
# Create tq config directory and password file
mkdir -p ~/.tq/passwords
echo "mypassword" > ~/.tq/passwords/dev
chmod 0600 ~/.tq/passwords/dev

# Create a profile (config file is created automatically if it doesn't exist)
tq profile add dev \
  --host dev.company.com \
  --database development \
  --user alice \
  --password-file ~/.tq/passwords/dev

# Use the profile
tq --profile dev query "SELECT CURRENT_DATE"
```

See [Managing Profiles](#managing-profiles) for the full guide to adding, editing, and deleting profiles.

## Configuration Hierarchy

Settings are loaded in this order (later overrides earlier):

1. **Built-in defaults** - Hardcoded sensible defaults
2. **User config** - `~/.tq/config.toml` (your personal settings)
3. **Project config** - `.tq.toml` in project root (team-shared settings)
4. **Environment variables** - `TQ_*` variables
5. **Command-line arguments** - Highest priority (final override)

### Precedence Example

```bash
# Level 1 (Built-in):   format = "table"
# Level 2 (User):       format = "json"
# Level 3 (Project):    format = "csv"
# Level 4 (Env var):    TQ_FORMAT=yaml
# Level 5 (CLI flag):   --format json

tq --format json query "SELECT 1"
# Result uses: json (CLI flag wins)
```

**How it works:**

Each level overrides the previous one. If a setting isn't specified at a higher level, the value from a lower level is used.

**Example with partial overrides:**

```bash
# Built-in:  format = "table", timing = false
# User:      format = "json"  (timing not specified, inherits false)
# Project:   timing = true    (format not specified, inherits "json")
# Result:    format = "json", timing = true
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

## Managing Profiles

The `tq profile` subcommand lets you create, update, and delete profiles in `~/.tq/config.toml` without manually editing the TOML file. It is designed for both interactive use and scripting.

### Adding a Profile

Use `tq profile add` to create a new profile. The `--host` flag is required; all other fields are optional.

```bash
# Minimal profile (host only)
tq profile add local --host localhost

# Full profile
tq profile add dev \
  --host dev.company.com \
  --port 1025 \
  --database development \
  --user alice \
  --logmech LDAP \
  --password-file ~/.tq/passwords/dev
```

**Output:**

```
Profile 'dev' added to ~/.tq/config.toml
```

If `~/.tq/config.toml` does not exist yet, tq creates the file and directory automatically.

**If the profile already exists:**

```
Error: Profile 'dev' already exists in ~/.tq/config.toml

Use 'tq profile edit dev' to update an existing profile.
Use 'tq profile delete dev' to remove it first.
```

### Editing a Profile

Use `tq profile edit` to update individual fields of an existing profile. Only the flags you provide are changed; all other fields remain untouched.

```bash
# Update just the host
tq profile edit dev --host new-dev.company.com

# Update database and user at the same time
tq profile edit dev --database dev2 --user bob

# Switch to LDAP authentication
tq profile edit prod --logmech LDAP
```

**Output:**

```
Profile 'dev' updated in ~/.tq/config.toml
```

You must provide at least one flag. Running `tq profile edit dev` with no flags is an error:

```
Error: No fields specified to update.
Provide at least one option flag.

Usage: tq profile edit <name> [--host <host>] [--port <port>] ...
Example: tq profile edit dev --host new-dev.company.com
```

**If the profile does not exist:**

```
Error: Profile 'staging' not found in ~/.tq/config.toml

Available profiles: dev, prod, local
Use 'tq profile add staging' to create a new profile.
```

### Deleting a Profile

Use `tq profile delete` to remove a profile. Without `--force`, tq prompts for confirmation when run interactively.

```bash
# Interactive confirmation
tq profile delete prod
```

```
Delete profile 'prod' from ~/.tq/config.toml? [y/N] _
```

Press `y` to confirm, or press Enter (or any other key) to abort. Aborting is not an error (exit code `0`):

```
Aborted. Profile 'prod' was not deleted.
```

**Scripting without a confirmation prompt:**

Pass `--force` to skip the confirmation. This is the recommended approach for scripts and CI/CD pipelines:

```bash
tq profile delete old-profile --force
```

```
Profile 'old-profile' deleted from ~/.tq/config.toml
```

When stdin is not a terminal and `--force` is not provided, tq exits with an error rather than hanging:

```
Error: Interactive confirmation required but stdin is not a terminal.
Use --force to bypass confirmation:
  tq profile delete prod --force
```

### Listing Profiles

`tq profile list` is an alias for `tq profiles`. Both commands show identical output.

```bash
tq profile list
# equivalent to:
tq profiles
```

### Profile Name Rules

Profile names must contain only letters, digits, hyphens, and underscores. Spaces and other characters are rejected:

```
Error: Invalid profile name 'my profile'
Profile names may only contain letters, digits, hyphens, and underscores.

Examples of valid names: dev, staging, prod-us, my_db
```

### Flag Reference

All `add` and `edit` flags:

| Flag | Short | Required for `add` | Description |
|------|-------|--------------------|-------------|
| `--host` | - | Yes | Database hostname |
| `--port` | - | No (default: 1025) | Database port (1-65535) |
| `--database` | `-d` | No | Default database name |
| `--user` | `-u` | No | Username |
| `--logmech` | - | No (default: `TD2`) | Auth mechanism: `TD2`, `LDAP`, `KRB5`, `TDNEGO` |
| `--password-file` | - | No | Path to file containing the password |

Accepted `--logmech` values: `TD2`, `LDAP`, `KRB5`, `TDNEGO` (case-insensitive). Any other value is rejected with exit code `2`.

### Important Behaviours

- **Profile management never connects to the database.** Commands operate on the local config file only. Connection flags (`--logon`, `--profile`) are ignored.
- **Config file comments and formatting are preserved.** tq does not reformat or reorder content it does not touch.
- **Writes are atomic.** tq writes to a temporary file and renames it to prevent corruption on failure.
- **Profile management targets user config only.** `tq profile` commands read and write `~/.tq/config.toml` only. Project config (`.tq.toml`) is never modified. If a profile exists only in a project config, `edit` and `delete` will report it as not found.

---

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

Configuration paths:
  User config: /home/alice/.tq/config.toml
  Project config: /home/alice/projects/analytics/.tq.toml

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

**Header information:**

The command shows which configuration files are being used:
- **User config**: Always shown if the file exists
- **Project config**: Shown only when `.tq.toml` is found in the project tree
- If no project config exists, this line is omitted

**Indicators:**

- **[project]** - Value from project config
- **[user]** - Value from user config

**When no profiles exist:**

If you have no profiles defined, you'll see a helpful tip:

```bash
$ tq profiles

Configuration paths:
  User config: /home/alice/.tq/config.toml

No profiles defined.

Tip: Create .tq.toml in your project root for team-shared profiles
```

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

Set up a local profile from the command line:

```bash
# Create password file
mkdir -p ~/.tq/passwords
echo "mypassword" > ~/.tq/passwords/local
chmod 0600 ~/.tq/passwords/local

# Add the profile
tq profile add local \
  --host localhost \
  --port 1025 \
  --database testdb \
  --user dbc \
  --password-file ~/.tq/passwords/local

# Run queries
tq --profile local query "SELECT CURRENT_DATE"
```

You can also set global defaults by editing `~/.tq/config.toml` directly:

```toml
[defaults]
format = "table"
timing = true
```

### Workflow 2: Multiple Environments

Add all three profiles, then switch between them:

```bash
tq profile add dev \
  --host dev.company.com \
  --database development \
  --user alice \
  --password-file ~/.tq/passwords/dev

tq profile add staging \
  --host staging.company.com \
  --database staging \
  --user alice \
  --password-file ~/.tq/passwords/staging

tq profile add prod \
  --host prod.company.com \
  --database production \
  --user alice \
  --logmech LDAP \
  --password-file ~/.tq/passwords/prod
```

Switch environments easily:

```bash
tq --profile dev query "SELECT COUNT(*) FROM users"
tq --profile staging query "SELECT COUNT(*) FROM users"
tq --profile prod query "SELECT COUNT(*) FROM users"
```

### Workflow 3: Team-Shared Configuration

Repository with project config (hosts in `.tq.toml`, credentials in personal user config):

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

Each team member adds their own credentials using `tq profile add`:

```bash
# Add credential-only profiles (host comes from project config via merging)
tq profile add dev --user alice --password-file ~/.tq/passwords/dev
tq profile add prod --user alice --password-file ~/.tq/passwords/prod
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
2. **Create profiles with `tq profile add`** - Avoids manual TOML editing and validates your input
3. **Never commit passwords** - Use `password_file` and keep passwords in `~/.tq/passwords/`
4. **Use project config for teams** - Share connection metadata, keep credentials personal
5. **Set file permissions** - Always `chmod 0600` on password files
6. **Use descriptive profile names** - `dev`, `staging`, `prod` are clearer than `db1`, `db2`
7. **Test profile changes** - Use `tq --profile name query "SELECT 1"` to verify
8. **Update profiles without re-creating** - Use `tq profile edit` to change individual fields
9. **Check merged profiles** - Use `tq profiles` to see how user and project configs merge
10. **Script profile cleanup** - Use `tq profile delete --force` in automation to remove stale profiles

## Troubleshooting

### Profile Not Found

```
Error: Profile 'dev' not found

Available profiles:
  - local
  - staging
  - prod
```

**Solution:** Check profile names with `tq profiles` and verify spelling. If the profile is missing, create it with `tq profile add dev --host <host>`.

### Cannot Add a Profile That Already Exists

```
Error: Profile 'dev' already exists in ~/.tq/config.toml

Use 'tq profile edit dev' to update an existing profile.
Use 'tq profile delete dev' to remove it first.
```

**Solution:** Use `tq profile edit dev --host <new-host>` to update the profile in place, or delete and recreate it.

### Cannot Edit a Profile in Project Config

`tq profile edit` and `tq profile delete` operate on `~/.tq/config.toml` only. Profiles defined solely in a project `.tq.toml` file will be reported as not found.

**Solution:** Edit `.tq.toml` manually for project-level profiles.

### Permission Denied on Password File

```
Error: Password file has insecure permissions: ~/.tq/passwords/dev
Current permissions: 0644
Required permissions: 0600

Fix: chmod 0600 ~/.tq/passwords/dev
```

**Solution:** Run the suggested `chmod` command.

### Invalid Project Config

If your project's `.tq.toml` file has invalid TOML syntax, you'll see a warning:

```
Warning: Invalid project config at /home/alice/projects/analytics/.tq.toml: expected value at line 15

tq continues using user config only
```

**What this means:**

- Your `.tq.toml` file has a syntax error
- `tq` shows the warning to stderr but continues operating
- Only user config and environment variables are used
- Command-line arguments still work normally

**How to fix:**

1. Open the `.tq.toml` file mentioned in the warning
2. Check the line number indicated in the error
3. Fix the TOML syntax error (common issues: missing quotes, unclosed brackets, typos)
4. Test with `tq profiles` to verify it loads correctly

**Common TOML errors:**

```toml
# Bad: Missing closing quote
[profiles.dev]
host = "myhost.com

# Good: Properly quoted
[profiles.dev]
host = "myhost.com"

# Bad: Invalid value
[defaults]
timing = yes

# Good: Boolean value
[defaults]
timing = true
```

**Why it's a warning, not an error:**

`tq` continues operating when project config is invalid to prevent blocking your work. You can still use `tq` with:
- User config profiles
- Environment variables
- Command-line arguments

Once you fix the syntax error, `tq` will automatically use the project config on the next invocation.

### No Profiles Defined

If `tq profiles` shows no profiles, you have several options:

```bash
$ tq profiles

No profiles defined.

Tip: Create .tq.toml in your project root for team-shared profiles
```

**Solutions:**

1. **Use command-line arguments** (no config needed):
   ```bash
   tq -l "user@host:1025/db" query "SELECT 1"
   ```

2. **Create user config** (`~/.tq/config.toml`):
   ```bash
   mkdir -p ~/.tq
   cat > ~/.tq/config.toml <<EOF
   [profiles.dev]
   host = "myhost.com"
   database = "mydb"
   user = "alice"
   EOF
   ```

3. **Create project config** (`.tq.toml` in project root) - ideal for teams:
   ```bash
   cat > .tq.toml <<EOF
   [profiles.dev]
   host = "dev.company.com"
   database = "dev_analytics"
   EOF
   ```

### Project Config Not Found

If `tq` doesn't find `.tq.toml`, it will use only user config. Check:

1. Is `.tq.toml` in the project root?
2. Are you running `tq` from within the project directory tree?
3. Did you check for typos in the filename?

**Verify project config discovery:**

Use `tq profiles` to see which config files are detected:

```bash
$ tq profiles

Configuration paths:
  User config: /home/alice/.tq/config.toml
  Project config: /home/alice/projects/analytics/.tq.toml  # Found!
```

If "Project config" line is missing, `tq` didn't find `.tq.toml` in the directory tree.

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
