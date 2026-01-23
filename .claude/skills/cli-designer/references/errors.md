# CLI Error Handling & Robustness

## Error Message Structure

```
ERROR: Clear description of what went wrong

  Context about the operation that failed

  Suggestion for how to fix it
```

**Example:**
```
ERROR: Deployment failed - no target environment specified

Try:
  toolname deploy --env production

Or set the default environment:
  export TOOLNAME_ENV=production
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General errors (runtime) |
| 2 | Usage errors (invalid args) |
| 130 | Terminated by Ctrl-C |

## Robustness Checklist

- [ ] Validate input at boundaries
- [ ] Handle signals properly (SIGINT, SIGTERM)
- [ ] Clean up resources on exit
- [ ] Make operations idempotent where possible
- [ ] Provide dry-run mode for destructive operations
- [ ] Confirm before deleting or overwriting

## Error Anti-Patterns

**Bad: Cryptic error**
```
Error: failed
```

**Good: Clear context and suggestion**
```
ERROR: Deployment failed - no target environment specified

Try:
  toolname deploy --env production
```

## Edge Cases to Handle

- Empty input or no results
- Very large inputs (streaming, pagination)
- Network failures and timeouts
- Interrupted operations (Ctrl-C)
- File system issues (permissions, disk space)

## Graceful Degradation

- Fallback to plain text when colors not supported
- Continue with warnings when possible
- Provide recovery mechanisms for partial failures
