# CLI Design Patterns

Common patterns for CLI tool design.

## Pattern 1: Simple Filter/Transformer

**Use case:** Tools that read, transform, and output (like `grep`, `jq`, `sed`)

```bash
toolname [OPTIONS] [FILE...]
# Reads from stdin if no files provided
```

**Characteristics:**
- Reads from stdin or files
- Writes to stdout, errors to stderr
- Composable via pipes
- Fast and focused

## Pattern 2: Single-Purpose Utility

**Use case:** Tools that perform one operation (like `curl`, `ping`)

```bash
toolname [OPTIONS] <TARGET> [ARGS]
```

**Characteristics:**
- One clear primary purpose
- Target as positional argument
- Options modify behavior

## Pattern 3: Git-like Multi-Command

**Use case:** Tools managing a domain with multiple operations (like `git`, `docker`)

```bash
toolname [GLOBAL_OPTIONS] <subcommand> [OPTIONS] <args>
```

**Characteristics:**
- Multiple subcommands
- Shared global options
- Rich help system

## Pattern 4: Interactive TUI

**Use case:** Tools needing rich interaction (like `htop`, `lazygit`)

```bash
toolname [OPTIONS]
# Launches full-screen interface
```

**Characteristics:**
- Full-screen terminal interface
- Keyboard navigation
- Real-time updates

## Pattern 5: REPL/Shell

**Use case:** Tools benefiting from persistent sessions (like `psql`, `redis-cli`)

```bash
toolname [OPTIONS]
# Enters interactive prompt
```

**Characteristics:**
- Interactive prompt loop
- Session state maintained
- History and completion
- Also supports one-shot: `toolname -c "command"`
