# Configuration and Credential Management

**Version:** 1.1.0
**Last Updated:** 2026-01-18
**Owner:** cli-ux-designer agent
**Status:** Active Specification

---

## Table of Contents

1. [Configuration Hierarchy](#71-configuration-hierarchy)
2. [Configuration File Format](#72-configuration-file-format)
3. [Environment Variables](#73-environment-variables)
4. [Connection Profiles](#74-connection-profiles)
5. [Credential Management](#75-credential-management)
6. [SSL/TLS Configuration](#76-ssltls-configuration-future)

---

## 7.1 Configuration Hierarchy

Configuration is loaded in this order (later overrides earlier):

1. **Built-in defaults**
2. **System config** (`/etc/tq/config.toml`)
3. **User config** (`~/.config/tq/config.toml`)
4. **Project config** (`./.tq.toml`)
5. **Environment variables** (`TQ_*`)
6. **Command-line arguments**

## 7.2 Configuration File Format

### 7.2.1 User Config (`~/.config/tq/config.toml`)

```toml
# Default connection
[connection]
host = "myteradata.company.com"
port = 1025
user = "myusername"
database = "mydatabase"
logmech = "LDAP"
timeout = "30s"

# Output preferences
[output]
format = "table"
color = "auto"
pager = true
timing = false

# REPL preferences
[repl]
history_file = "~/.tq_history"
history_size = 10000
editor_mode = "emacs"
syntax_highlight = true
autocomplete = true

# Named connection profiles
[profiles.prod]
host = "prod.company.com"
port = 1025
database = "production"
logmech = "KRB5"

[profiles.dev]
host = "dev.company.com"
port = 1025
database = "development"
logmech = "TD2"

[profiles.local]
host = "localhost"
port = 1025
database = "testdb"
logmech = "TD2"
```

### 7.2.2 Project Config (`.tq.toml`)

For team-shared settings (committed to version control):

```toml
[connection]
host = "shared-dev.company.com"
port = 1025
database = "team_database"
# Note: Never commit passwords!

[output]
format = "json"  # Project prefers JSON output
```

## 7.3 Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `TQ_LOGON` | Complete connection string | `user:pass@host:1025/db` |
| `TQ_HOST` | Database hostname | `myteradata.company.com` |
| `TQ_PORT` | Database port | `1025` |
| `TQ_USER` | Database username | `myuser` |
| `TQ_PASSWORD` | Database password (discouraged) | `mypassword` |
| `TQ_DATABASE` | Database name | `mydatabase` |
| `TQ_LOGMECH` | Authentication mechanism | `LDAP` |
| `TQ_FORMAT` | Default output format | `json` |
| `TQ_TIMEOUT` | Connection timeout | `30s` |
| `TQ_PROFILE` | Configuration profile to use | `prod` |

**Usage**:
```bash
# Set for entire session
export TQ_LOGON="user:pass@host:1025/db"
tq ping
tq query "SELECT 1"

# Set for single command
TQ_FORMAT=json tq query "SELECT * FROM users"
```

## 7.4 Connection Profiles

### 7.4.1 Using Profiles

```bash
# Select profile via environment
export TQ_PROFILE=prod
tq query "SELECT COUNT(*) FROM users"

# Select profile via flag
tq --profile prod query "SELECT COUNT(*) FROM users"

# List available profiles
tq profile list
```

### 7.4.2 Managing Profiles

```bash
# Create new profile
tq profile create staging --host staging.db.com --port 1025

# Update profile
tq profile update prod --timeout 60s

# Delete profile
tq profile delete old-dev

# Show profile details
tq profile show prod
```

## 7.5 Credential Management

### 7.5.1 Security Principles

1. **Never use passwords in CLI arguments** - visible in `ps`, shell history
2. **Never log passwords** - sanitize all debug output
3. **Use file permissions** - `chmod 0600` for credential files
4. **Prefer keyring integration** - OS-native secure storage
5. **Support password prompts** - interactive secure input

### 7.5.2 Password Sources (Priority Order)

1. **Keyring** (most secure)
2. **Password file** (`--password-file`)
3. **Configuration file** (protected)
4. **Environment variable** (`TQ_PASSWORD`) - discouraged
5. **Interactive prompt** - for missing password

### 7.5.3 Password File

**Format** (similar to `.pgpass`):
```
# hostname:port:database:username:password
myhost:1025:mydb:alice:secret123
prodhost:1025:*:bob:prodpass
*:1025:*:admin:adminpass
```

**Usage**:
```bash
# Create password file
cat > ~/.tq_passwords <<EOF
myhost:1025:mydb:alice:secret123
EOF
chmod 0600 ~/.tq_passwords

# Use default location (~/.tq_passwords)
tq -l "alice@myhost:1025/mydb" query "SELECT 1"

# Use custom location
tq -l "alice@myhost:1025/mydb" --password-file ~/my-passwords query "SELECT 1"
```

### 7.5.4 Keyring Integration (Future)

```bash
# Store password in OS keyring
tq password set prod
Enter password: ****

# Use stored password
tq --profile prod query "SELECT 1"
# Automatically retrieves password from keyring

# List stored passwords
tq password list

# Delete stored password
tq password delete prod
```

### 7.5.5 Interactive Password Prompt

```bash
# Connection without password prompts for it
tq -l "user@host:1025/db" query "SELECT 1"
Password: ****  # secure input, not echoed
```

## 7.6 SSL/TLS Configuration (Future)

```toml
[connection]
host = "secure.company.com"
port = 1025
ssl = true
ssl_mode = "require"  # options: disable, allow, prefer, require, verify-ca, verify-full
ssl_ca_file = "/path/to/ca-cert.pem"
ssl_cert_file = "/path/to/client-cert.pem"
ssl_key_file = "/path/to/client-key.pem"
```

---
