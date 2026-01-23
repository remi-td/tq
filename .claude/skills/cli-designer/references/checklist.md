# CLI Design Validation Checklist

## Help & Documentation
- [ ] Help displays with `-h`, `--help`, and `help` subcommand
- [ ] Help leads with practical examples
- [ ] Common operations documented first
- [ ] Link to web documentation
- [ ] Version with `-V` or `--version`

## Arguments & Flags
- [ ] Standard flags follow conventions
- [ ] Long flags use kebab-case
- [ ] Positional arguments documented
- [ ] `--` separates options from positional args
- [ ] No passwords as CLI flags
- [ ] Boolean flags work without values

## Output
- [ ] Human-readable when TTY detected
- [ ] Machine-readable when piped
- [ ] `--json` for structured output
- [ ] `--quiet` suppresses non-essential output
- [ ] Success shows what was accomplished
- [ ] Colors respect `NO_COLOR` and `--no-color`

## Errors & Robustness
- [ ] Errors go to stderr
- [ ] Error messages are clear and actionable
- [ ] Suggestions for common mistakes
- [ ] Exit codes follow convention (0/1/2)
- [ ] Signals handled gracefully
- [ ] Destructive operations require confirmation
- [ ] Dry-run mode available

## Configuration
- [ ] Configuration hierarchy documented
- [ ] Environment variables follow convention
- [ ] Config file location is standard
- [ ] Current configuration discoverable
- [ ] Invalid config produces clear errors

## Interactivity
- [ ] Prompts are clear with defaults
- [ ] `-y/--yes` skips confirmations
- [ ] Non-TTY doesn't hang on prompts
- [ ] Progress shown for long operations
- [ ] Ctrl-C cancels cleanly

## Composability
- [ ] Reads from stdin when no files
- [ ] Primary output to stdout
- [ ] Logs/errors to stderr
- [ ] Exit codes indicate success/failure
- [ ] Pipeable with UNIX tools

## Performance
- [ ] Fast startup (<100ms for simple ops)
- [ ] Responsive feedback for long ops
- [ ] Streaming for large data
- [ ] Pagination for long output
