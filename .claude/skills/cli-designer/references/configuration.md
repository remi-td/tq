# CLI Arguments, Flags & Configuration

## Standard Flag Conventions

**Short flags:** `-v`, `-h`, `-q` (single dash, single letter)
**Long flags:** `--verbose`, `--help` (double dash, full word)
**Multi-word:** Use kebab-case: `--log-level`, `--output-file`

## Universally Expected Flags

| Flag | Purpose |
|------|---------|
| `-h, --help` | Display help |
| `-V, --version` | Display version |
| `-v, --verbose` | Increase output |
| `-q, --quiet` | Decrease output |
| `--color <WHEN>` | Control color [always\|auto\|never] |
| `--no-color` | Disable color |

## Configuration Precedence

1. CLI flags and arguments (highest)
2. Environment variables (`TOOL_*`)
3. Project config (`.toolname.toml`)
4. User config (`~/.config/toolname/config.toml`)
5. System config (`/etc/toolname/config.toml`)
6. Built-in defaults (lowest)

## Environment Variables

- Prefix with tool name: `TOOLNAME_HOST`
- Use uppercase with underscores
- Document all supported variables
- Show current values in status commands

## Configuration File

TOML recommended for CLIs:
```toml
# ~/.config/toolname/config.toml
[database]
host = "localhost"
port = 1025
```

## Configuration Commands

```bash
# Show current configuration
toolname config show

# Show config file location
toolname config path

# Edit configuration
toolname config edit
```

## Security: No Passwords as CLI Flags

**Bad:** Visible in `ps` output
```bash
toolname --password supersecret
```

**Good:** Secure alternatives
```bash
# Read from file
toolname --password-file ~/.secrets/db.pwd

# Prompt if needed
toolname --ask-password

# Environment variable
export TOOLNAME_PASSWORD=...
```
