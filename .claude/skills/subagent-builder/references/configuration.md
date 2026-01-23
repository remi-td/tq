# Subagent Configuration Reference

## Tool Selection

**Read-only agents:**
```yaml
tools: Read, Grep, Glob, Bash
```

**Editor agents:**
```yaml
tools: Read, Write, Edit, Grep, Glob, Bash
```

**Full access:**
```yaml
tools: [omit field to inherit all]
```

**Restricted:**
```yaml
tools: Bash
disallowedTools: Write, Edit
```

## Model Selection

| Model | Use For |
|-------|---------|
| **Haiku** | Simple tasks, high-volume, cost-sensitive |
| **Sonnet** | General-purpose, balanced (default) |
| **Opus** | Complex reasoning, architectural decisions |
| **Inherit** | Use parent model for consistency |

## Permission Modes

| Mode | Behavior | Use For |
|------|----------|---------|
| `default` | Standard prompts | Most agents |
| `acceptEdits` | Auto-approve file changes | Trusted refactoring |
| `dontAsk` | Auto-deny unpermitted | Read-only enforcement |
| `bypassPermissions` | Skip all checks | Automated workflows |
| `plan` | Read-only exploration | Research agents |

## Hooks (Optional)

**Pre-tool validation:**
```yaml
hooks:
  PreToolUse:
    - matcher: "Bash"
      hooks:
        - type: command
          command: "./scripts/validate.sh"
```

**Post-tool actions:**
```yaml
hooks:
  PostToolUse:
    - matcher: "Edit|Write"
      hooks:
        - type: command
          command: "./scripts/lint.sh"
```

## Storage Locations

| Location | Use |
|----------|-----|
| `.claude/agents/` | Project-level, team-shared |
| `~/.claude/agents/` | User-level, cross-project |
