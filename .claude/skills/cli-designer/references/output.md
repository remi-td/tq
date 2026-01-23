# CLI Output & Formatting Guidelines

## Output Modes

**Human mode (TTY detected):**
- Formatted tables with alignment
- Colors for emphasis (honor `NO_COLOR` env var)
- Headers and separators
- Progress indicators
- Summaries and statistics

**Machine mode (piped):**
- One record per line
- No colors or control characters
- Consistent field separators
- Stable format across versions

**JSON mode (`--json`):**
- Well-structured, documented schema
- Include metadata (timestamp, version)
- Consistent field naming

## Success Feedback

State what was accomplished:
```
✓ Created 3 users
✓ Deployed to production
```

Show next steps:
```
Run 'toolname status' to verify
```

## Help Text Structure

```
USAGE:
  toolname [OPTIONS] <QUERY>

EXAMPLES:
  # Simple query
  toolname "SELECT * FROM users"

  # With custom output
  toolname --format json "SELECT id FROM users"

OPTIONS:
  -l, --logon <STRING>     Connection string
      --format <FORMAT>    Output [table|json|csv]
  -q, --quiet              Suppress output
  -h, --help               Print help

See 'toolname help <command>' for details.
Documentation: https://docs.example.com
```

## Example Output

**Human-friendly (TTY):**
```
┌────┬────────┬──────────┐
│ ID │ Name   │ Status   │
├────┼────────┼──────────┤
│ 1  │ Alice  │ active   │
│ 2  │ Bob    │ inactive │
└────┴────────┴──────────┘
```

**Machine-readable (piped):**
```
1,Alice,active
2,Bob,inactive
```
