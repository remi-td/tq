---
name: cli-designer
description: Guides the design of command-line interfaces following industry best practices from the Command Line Interface Guidelines (clig.dev). Use when designing new CLI tools, reviewing CLI architecture, planning CLI features, or evaluating CLI usability. Balances human-first design with UNIX philosophy for intuitive, composable tools.
---

# CLI Designer

Design best-in-class command-line interfaces that are intuitive for humans while maintaining UNIX composability principles.

## When to Use

- Designing a new command-line tool from scratch
- Planning the architecture and UX of CLI features
- Reviewing existing CLI designs for usability issues
- Evaluating CLI argument structures and output formats
- Creating design specifications for CLI tools
- Choosing between CLI design patterns (simple vs multi-command)

## Core Philosophy

Follow these eight foundational principles for all CLI designs:

### 1. Human-First Design
Prioritize human users over pure machine automation. Modern CLIs should be pleasant to use interactively, not just scriptable.

### 2. Simple Parts That Work Together
Maintain UNIX composability through standard mechanisms (pipes, exit codes, text output) while improving human usability.

### 3. Consistency Across Programs
Follow established conventions to reduce learning curves. Deviate intentionally only when demonstrably better for users.

### 4. Saying Just Enough
Balance output carefully—too little suggests the program is broken, too much obscures important information.

### 5. Ease of Discovery
Incorporate help, examples, and suggestions to aid learning without sacrificing efficiency for experienced users.

### 6. Conversation as the Norm
Design for trial-and-error workflows. CLI interaction is inherently conversational and multi-step.

### 7. Robustness
Ensure both objective robustness (graceful error handling) and subjective robustness (feels solid and responsive).

### 8. Empathy
Design with user success in mind. Show that you've considered their problems and want them to succeed.

## Design Process

### Step 1: Define the Tool's Purpose

**Identify core functionality:**
- What problem does this tool solve?
- Who are the primary users?
- What workflows will users follow?

**Choose the appropriate pattern:**
- **Simple single-purpose tool**: One command, focused functionality (e.g., `grep`, `jq`)
- **Git-like multi-command tool**: Multiple related subcommands under one namespace (e.g., `git`, `docker`, `cargo`)

### Step 2: Design the Command Structure

**For simple tools:**
```bash
toolname [OPTIONS] <REQUIRED_ARGS> [OPTIONAL_ARGS]
```

**For git-like tools:**
```bash
toolname <SUBCOMMAND> [OPTIONS] <ARGS>
# Or with global options:
toolname [GLOBAL_OPTIONS] <SUBCOMMAND> [OPTIONS] <ARGS>
```

**Key decisions:**
- What are the positional arguments?
- What operations need flags vs positional args?
- Should destructive operations be behind subcommands?

### Step 3: Design Help & Documentation

**Help text structure:**
1. Brief description (one line)
2. Usage syntax
3. Practical examples (2-4, from simple to complex)
4. Common flags first, grouped logically
5. Link to detailed documentation

**Documentation hierarchy:**
- `-h/--help`: Show help inline
- Web docs: Searchable, linkable, always available
- Man pages: Offline reference, version-specific
- In-tool docs: `toolname help <topic>`

### Step 4: Design Output Strategy

**Output modes:**
- **Human mode** (TTY detected): Formatted tables, colors, helpful messages
- **Machine mode** (piped): Clean parseable output, one record per line
- **JSON mode** (`--json`): Structured data for programmatic consumption
- **Quiet mode** (`-q/--quiet`): Minimal output for scripts

**Success output:**
- State what was accomplished
- Show relevant results or next steps
- Don't be silent (unless `-q` is used)

### Step 5: Design Error Handling

**Error message structure:**
```
ERROR: Clear description of what went wrong

  Context about the operation that failed

  Suggestion for how to fix it
```

**Error design principles:**
- Write for humans, not machines
- Include enough context to understand the issue
- Suggest concrete next steps
- Show relevant configuration or state
- Avoid stack traces in user-facing errors

### Step 6: Design Configuration Hierarchy

**Configuration precedence (highest to lowest):**
1. CLI flags and arguments
2. Environment variables (prefixed, e.g., `TOOL_*`)
3. Project-level config (`.toolname.toml` in current directory)
4. User-level config (`~/.config/toolname/config.toml`)
5. System-level config (`/etc/toolname/config.toml`)
6. Built-in defaults

**Make configuration discoverable:**
- Document in help text
- Show current config with status commands
- Validate and report invalid configuration clearly

### Step 7: Design for Robustness

**Handle edge cases:**
- Empty input or no results
- Very large inputs (streaming, pagination)
- Network failures and timeouts
- Interrupted operations (Ctrl-C)
- File system issues (permissions, disk space)

**Design for graceful degradation:**
- Fallback to plain text when colors aren't supported
- Continue with warnings when possible
- Provide recovery mechanisms for partial failures

## Essential Guidelines

### Help Text

**MUST:**
- Display help with `-h`, `--help`, or `help` subcommand
- Show concise help when run without required arguments
- Lead with practical examples
- Display frequently-used flags first
- Use bold headings for sections
- Suggest corrections for apparent typos

**SHOULD:**
- Link to web documentation for details
- Support `toolname help subcommand` and `toolname subcommand --help`
- Exit immediately with help if stdin expected but TTY detected
- Group related flags together

**Example structure:**
```
USAGE:
  toolname [OPTIONS] <QUERY>

EXAMPLES:
  # Simple query
  toolname "SELECT * FROM users"

  # With custom output format
  toolname --format json "SELECT id, name FROM users"

  # Using connection string
  toolname -l user:pass@host:1025/db "SELECT COUNT(*) FROM orders"

OPTIONS:
  -l, --logon <STRING>     Database connection string
      --format <FORMAT>    Output format [table|json|csv] [default: table]
  -q, --quiet              Suppress non-essential output
  -h, --help               Print help
  -V, --version            Print version

See 'toolname help <command>' for more information on a specific command.
Documentation: https://docs.example.com/toolname
```

### Output & Formatting

**Human-readable output (TTY):**
- Use tables with clear alignment
- Add colors for emphasis (honor `NO_COLOR` env var)
- Include headers and separators
- Show progress for long operations
- Display summaries and statistics

**Machine-readable output (piped):**
- One record per line
- No colors or control characters
- Consistent field separators
- Stable output format across versions

**JSON output (`--json`):**
- Well-structured, documented schema
- Include metadata (timestamp, version)
- Use consistent field naming
- Validate output against schema

**Success feedback:**
- State what was accomplished: "Created 3 users", "Deployed to production"
- Show next steps: "Run 'toolname status' to verify"
- Display relevant results inline

### Errors & Robustness

**Error message best practices:**
- Start with "ERROR:" or clear indicator
- Explain what went wrong in plain language
- Provide context about the failed operation
- Suggest concrete remediation steps
- Include relevant values (sanitized)

**Exit codes:**
- **0**: Success
- **1**: General errors (runtime failures)
- **2**: Usage errors (invalid arguments)
- **130**: Terminated by Ctrl-C (SIGINT)

**Robustness checklist:**
- Validate input at boundaries
- Handle signals properly (SIGINT, SIGTERM)
- Clean up resources on exit
- Make operations idempotent where possible
- Provide dry-run mode for destructive operations
- Confirm before deleting or overwriting

### Arguments & Flags

**Standard conventions:**
- Short flags: `-v`, `-h`, `-q` (single dash, single letter)
- Long flags: `--verbose`, `--help`, `--quiet` (double dash, full word)
- Use kebab-case for multi-word flags: `--log-level`, `--output-file`
- Support `--` to separate options from positional arguments

**Universally expected flags:**
- `-h, --help`: Display help
- `-V, --version`: Display version
- `-v, --verbose`: Increase output detail
- `-q, --quiet`: Decrease output detail
- `--color <WHEN>`: Control color output [always|auto|never]
- `--no-color`: Disable color output (honor `NO_COLOR` env var)

**Flag design principles:**
- Use environment variables for frequently-set options
- Use config files for complex multi-value configuration
- Never accept passwords as CLI flags (use `--password-file` or stdin)
- Provide sensible defaults for all optional flags
- Make boolean flags work without values: `--verbose`, not `--verbose=true`

### Configuration

**Environment variable conventions:**
- Prefix with tool name: `TOOLNAME_HOST`, `TOOLNAME_PORT`
- Use uppercase with underscores
- Document all supported variables
- Show current values in `--help` or status commands

**Configuration file format:**
- Use TOML, YAML, or JSON (TOML recommended for CLIs)
- Place in standard locations (`~/.config/toolname/`)
- Support project-level overrides
- Validate and report errors clearly

**Configuration discovery:**
```bash
# Show current configuration
toolname config show

# Show configuration file location
toolname config path

# Edit configuration
toolname config edit
```

### Interactivity

**When to prompt:**
- Destructive operations (delete, overwrite)
- Operations with significant impact (deploy, publish)
- When required information is missing

**Prompt design:**
```
⚠ This will delete 42 users permanently.

Are you sure? [y/N]: _
```

**Non-interactive mode:**
- Provide `-y/--yes` to skip confirmations
- Provide `--force` for overrides
- Error out if input needed in non-TTY context

**Dry-run mode:**
```bash
toolname --dry-run deploy
# Shows what would happen without executing
```

### Subcommands (Git-like Tools)

**Structure:**
```bash
toolname <subcommand> [options] <args>
```

**Organization principles:**
- Group related operations under logical subcommands
- Keep subcommand names short and memorable
- Use verbs for actions: `create`, `delete`, `list`, `show`
- Alias common commands: `ls` for `list`, `rm` for `remove`

**Help support:**
- `toolname help`: List all subcommands
- `toolname help <subcommand>`: Show subcommand help
- `toolname <subcommand> --help`: Show subcommand help
- `toolname <subcommand>` (no args): Show subcommand help

**Example subcommand organization:**
```
Resource management:
  create      Create a new resource
  list, ls    List all resources
  show        Show details of a resource
  delete, rm  Delete a resource

Operations:
  deploy      Deploy to environment
  rollback    Rollback to previous version
  status      Show current status

Configuration:
  config      Manage configuration
  init        Initialize new project
```

## Common Design Patterns

### Pattern 1: Simple Filter/Transformer

**Use case:** Tools that read input, transform it, and write output (like `grep`, `jq`, `sed`)

**Structure:**
```bash
toolname [OPTIONS] [FILE...]
# Reads from stdin if no files provided
```

**Characteristics:**
- Reads from stdin or files
- Writes to stdout
- Errors to stderr
- Composable via pipes
- Fast and focused

**Example:** `jq '.users[] | .email' data.json`

### Pattern 2: Single-Purpose Utility

**Use case:** Tools that perform one specific operation (like `curl`, `ping`, `ssh`)

**Structure:**
```bash
toolname [OPTIONS] <TARGET> [ARGS]
```

**Characteristics:**
- One clear primary purpose
- Target/subject as positional argument
- Options modify behavior
- Clear success/failure feedback

**Example:** `curl -X POST -d '...' https://api.example.com`

### Pattern 3: Git-like Multi-Command

**Use case:** Tools that manage a domain with multiple operations (like `git`, `docker`, `npm`)

**Structure:**
```bash
toolname [GLOBAL_OPTIONS] <subcommand> [OPTIONS] <args>
```

**Characteristics:**
- Multiple related subcommands
- Shared global options
- Consistent patterns across subcommands
- Rich help system

**Example:** `docker container ls --all`

### Pattern 4: Interactive TUI

**Use case:** Tools that need rich interaction or exploration (like `htop`, `tig`, `lazygit`)

**Structure:**
```bash
toolname [OPTIONS]
# Launches full-screen interface
```

**Characteristics:**
- Full-screen terminal interface
- Keyboard-driven navigation
- Real-time updates
- Graceful fallback for non-TTY

**Example:** `htop`

### Pattern 5: REPL/Shell

**Use case:** Tools that benefit from persistent sessions (like `psql`, `redis-cli`, `python`)

**Structure:**
```bash
toolname [OPTIONS]
# Enters interactive mode with prompt
```

**Characteristics:**
- Interactive prompt loop
- Session state maintained
- Command history and completion
- Also supports one-shot mode: `toolname -c "command"`

**Example:** `redis-cli`

## Design Anti-Patterns

### ❌ Silent Success
**Bad:** Command succeeds with no output
```bash
$ toolname delete-all-users
$ _
```

**Good:** Confirm what happened
```bash
$ toolname delete-all-users
✓ Deleted 42 users
```

### ❌ Cryptic Errors
**Bad:** Vague, unhelpful error
```bash
$ toolname deploy
Error: failed
```

**Good:** Clear context and suggestion
```bash
$ toolname deploy
ERROR: Deployment failed - no target environment specified

Try:
  toolname deploy --env production

Or set the default environment:
  export TOOLNAME_ENV=production
```

### ❌ Password as CLI Flag
**Bad:** Insecure, visible in `ps` output
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
toolname ...
```

### ❌ No Help Without Arguments
**Bad:** Cryptic usage line only
```bash
$ toolname
usage: toolname <query>
```

**Good:** Helpful guidance
```bash
$ toolname
Error: Missing required argument <QUERY>

USAGE:
  toolname [OPTIONS] <QUERY>

EXAMPLES:
  toolname "SELECT * FROM users"
  toolname --help
```

### ❌ Machine Output by Default
**Bad:** Unreadable for humans
```bash
$ toolname list
id,name,status
1,Alice,active
2,Bob,inactive
```

**Good:** Human-friendly when TTY detected
```bash
$ toolname list
┌────┬────────┬──────────┐
│ ID │ Name   │ Status   │
├────┼────────┼──────────┤
│ 1  │ Alice  │ active   │
│ 2  │ Bob    │ inactive │
└────┴────────┴──────────┘

# But clean output when piped:
$ toolname list | head -1
1,Alice,active
```

### ❌ Undiscoverable Configuration
**Bad:** Hidden magic behavior
```bash
# Works differently based on ~/.obscure-config-file
# Not documented anywhere
```

**Good:** Transparent configuration
```bash
$ toolname --help
...
CONFIGURATION:
  Config file: ~/.config/toolname/config.toml
  Environment: TOOLNAME_* variables

See 'toolname config --help' for details.
```

## Validation Checklist

Use this checklist to validate your CLI design:

### Help & Documentation
- [ ] Help displays with `-h`, `--help`, and `help` subcommand
- [ ] Help leads with practical examples
- [ ] Common operations are documented first
- [ ] Help includes link to web documentation
- [ ] Version displays with `-V` or `--version`
- [ ] Man pages are provided (if applicable)

### Arguments & Flags
- [ ] Standard flags follow conventions (`-h`, `-v`, `-q`, etc.)
- [ ] Long flags use kebab-case
- [ ] Positional arguments are clearly documented
- [ ] `--` separates options from positional arguments
- [ ] No passwords accepted as CLI flags
- [ ] Boolean flags work without values

### Output
- [ ] Human-readable output when TTY detected
- [ ] Machine-readable output when piped
- [ ] `--json` flag for structured output
- [ ] `--quiet` flag suppresses non-essential output
- [ ] Success operations show what was accomplished
- [ ] Colors respect `NO_COLOR` and `--no-color`
- [ ] Output is consistent and parseable

### Errors & Robustness
- [ ] Errors go to stderr
- [ ] Error messages are clear and actionable
- [ ] Suggestions provided for common mistakes
- [ ] Exit codes follow convention (0=success, 1=error, 2=usage)
- [ ] Signals handled gracefully (SIGINT, SIGTERM)
- [ ] Destructive operations require confirmation
- [ ] Dry-run mode available for complex operations

### Configuration
- [ ] Configuration hierarchy is documented
- [ ] Environment variables follow naming convention
- [ ] Config file location is standard and documented
- [ ] Current configuration is discoverable
- [ ] Invalid configuration produces clear errors

### Interactivity
- [ ] Prompts are clear and have default choices
- [ ] `-y/--yes` flag skips confirmations
- [ ] Non-TTY environments don't hang on prompts
- [ ] Progress shown for long operations
- [ ] Ctrl-C cancels operations cleanly

### Composability
- [ ] Reads from stdin when no files specified
- [ ] Writes primary output to stdout
- [ ] Writes logs/errors to stderr
- [ ] Exit codes indicate success/failure
- [ ] Pipeable with other UNIX tools
- [ ] Works in non-interactive scripts

### Performance & UX
- [ ] Fast startup time (< 100ms for simple operations)
- [ ] Responsive feedback for long operations
- [ ] Streaming for large data sets
- [ ] Pagination for long output (or defer to `less`)
- [ ] Operations feel robust and solid
- [ ] Error recovery options provided

## Examples

### Example 1: Simple Database Query Tool

**Good design:**
```bash
# Human-friendly table by default
$ dbtool -l user:pass@host/db "SELECT * FROM users LIMIT 2"
┌────┬─────────┬──────────────────┬────────────┐
│ id │ name    │ email            │ created_at │
├────┼─────────┼──────────────────┼────────────┤
│ 1  │ Alice   │ alice@example.com│ 2024-01-15 │
│ 2  │ Bob     │ bob@example.com  │ 2024-01-16 │
└────┴─────────┴──────────────────┴────────────┘

2 rows returned (0.23s)

# Machine-parseable when piped
$ dbtool -l ... "SELECT id, name FROM users" | head -1
1,Alice

# JSON for structured consumption
$ dbtool --format json "SELECT * FROM users LIMIT 1"
[
  {
    "id": 1,
    "name": "Alice",
    "email": "alice@example.com",
    "created_at": "2024-01-15T10:30:00Z"
  }
]

# Helpful error with suggestion
$ dbtool "SELECT * FROM users"
ERROR: Missing database connection

Specify connection with:
  -l, --logon user:pass@host:port/database

Or set environment variable:
  export DBTOOL_LOGON=user:pass@host:port/database

Run 'dbtool --help' for more options.
```

### Example 2: Git-like Deployment Tool

**Good design:**
```bash
# Helpful default output
$ deploytool
deploytool - Multi-environment deployment tool

USAGE:
  deploytool <COMMAND> [OPTIONS]

COMMON COMMANDS:
  deploy      Deploy to an environment
  rollback    Rollback to previous version
  status      Show deployment status
  logs        View deployment logs

Run 'deploytool help <command>' for more information.

# Subcommand with clear help
$ deploytool deploy
ERROR: Missing required argument <environment>

USAGE:
  deploytool deploy [OPTIONS] <environment>

EXAMPLES:
  # Deploy current branch to staging
  deploytool deploy staging

  # Deploy specific version to production
  deploytool deploy production --version v2.1.0

  # Dry-run to see what would happen
  deploytool deploy production --dry-run

OPTIONS:
  --version <VERSION>    Version to deploy [default: current]
  --dry-run              Show what would happen without deploying
  -y, --yes              Skip confirmation prompts

# Successful deployment with feedback
$ deploytool deploy staging
🚀 Deploying to staging...

Building application...        ✓ (12.3s)
Running tests...               ✓ (4.2s)
Uploading artifacts...         ✓ (2.1s)
Deploying to staging...        ✓ (8.7s)

✓ Successfully deployed v2.1.5 to staging

Application URL: https://staging.example.com
Logs: deploytool logs staging
```

## Guidelines for Breaking Rules

The CLI Guidelines acknowledge that established patterns should sometimes be violated when demonstrably better for users. When deviating from conventions:

1. **Have clear justification**: Document why the standard approach is harmful
2. **Consider user confusion**: Weigh benefits against learning curve
3. **Be consistent internally**: If you break one convention, apply that pattern throughout
4. **Document the deviation**: Explain the non-standard choice in help text
5. **Test with real users**: Validate that the deviation actually improves usability

**Example of justified deviation:**
```bash
# Standard: git-style subcommands
git commit -m "message"

# Deviation: if your tool has truly one primary operation, don't force subcommands
query-tool "SELECT ..." # Not: query-tool execute "SELECT ..."
```

## Additional Resources

- **Full Guidelines**: https://clig.dev
- **Argument Parsing Libraries**: Use established libraries like `clap` (Rust), `argparse` (Python), `commander` (Node.js)
- **Terminal Capabilities**: Use libraries for color, formatting, and TTY detection
- **Testing**: Use CLI testing frameworks (`assert_cmd` for Rust, `bats` for bash)

## Summary

Designing excellent CLIs requires balancing multiple concerns:
- **Human usability** vs **machine composability**
- **Helpful feedback** vs **quiet efficiency**
- **Discoverability** vs **expert shortcuts**
- **Consistency** vs **innovation**

Follow the core philosophy, apply the essential guidelines, validate against the checklist, and always design with empathy for your users' success.
