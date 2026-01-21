---
name: cli-designer
description: Guides CLI design following clig.dev best practices. Use when designing new CLI tools, reviewing CLI architecture, planning CLI features, or evaluating CLI usability.
---

# CLI Designer

Design CLIs that are intuitive for humans while maintaining UNIX composability.

## Core Philosophy

### 1. Human-First Design
Prioritize human users over pure machine automation.

### 2. Simple Parts That Work Together
Standard mechanisms: pipes, exit codes, text output.

### 3. Consistency Across Programs
Follow conventions. Deviate only when demonstrably better.

### 4. Saying Just Enough
Too little = broken; too much = obscures important info.

### 5. Ease of Discovery
Help, examples, suggestions for learning.

### 6. Conversation as the Norm
Design for trial-and-error workflows.

### 7. Robustness
Graceful error handling, feels solid.

### 8. Empathy
Design for user success.

## Design Process

### Step 1: Define Purpose
- What problem does this solve?
- Who are the users?
- What workflows will they follow?

Choose pattern:
- **Simple tool**: One command (like `grep`, `jq`)
- **Git-like tool**: Multiple subcommands (like `git`, `cargo`)

### Step 2: Command Structure

**Simple:**
```bash
tool [OPTIONS] <REQUIRED> [OPTIONAL]
```

**Git-like:**
```bash
tool <SUBCOMMAND> [OPTIONS] <ARGS>
```

### Step 3: Design Help
- Brief description
- Usage syntax
- 2-4 practical examples
- Common flags first
- Link to docs

### Step 4: Design Output
- Human mode (TTY): tables, colors
- Machine mode (piped): clean, parseable
- JSON mode (`--json`)
- Quiet mode (`-q`)

### Step 5: Design Errors
```
ERROR: What went wrong
  Context
  How to fix
```

### Step 6: Configuration
CLI flags > Env vars > Project config > User config > Defaults

## Detailed References

- **[Patterns](references/patterns.md)**: Filter, utility, git-like, TUI, REPL
- **[Output](references/output.md)**: Formatting, help text, TTY detection
- **[Errors](references/errors.md)**: Error messages, exit codes, robustness
- **[Configuration](references/configuration.md)**: Flags, env vars, config files
- **[Checklist](references/checklist.md)**: Validation checklist for CLI design

## Essential Flags

| Flag | Purpose |
|------|---------|
| `-h, --help` | Display help |
| `-V, --version` | Display version |
| `-v, --verbose` | Increase output |
| `-q, --quiet` | Decrease output |
| `--json` | JSON output |
| `--no-color` | Disable colors |

## Anti-Patterns

| Don't | Do |
|-------|-----|
| Silent success | State what happened |
| Cryptic errors | Clear context + suggestion |
| Password as flag | Use env var or file |
| Machine output default | Human-friendly when TTY |

## Resources

- **Full Guidelines**: https://clig.dev
- **Libraries**: `clap` (Rust), `argparse` (Python)
