---
name: subagent-builder
description: Creates specialized Claude subagents. Use when users want to create a new subagent, define capabilities, or generate agent configuration files.
---

# Subagent Builder

Create specialized AI assistants with isolated context, custom prompts, controlled tools, and permission modes.

## Creation Workflow

### 1. Discovery - Understand the Need

**Ask:**
1. What task/domain should this agent handle?
2. When should Claude delegate to it?
3. What autonomy level? (read-only, read-write, command execution)
4. Any special constraints?

### 2. Design - Propose Configuration

Use the decision framework:
- **Tools**: Minimum necessary access
- **Model**: Match complexity (Haiku → Sonnet → Opus)
- **Permissions**: Safe but not restrictive
- **Hooks**: Only if validation needed

### 3. Prompt Writing

**Structure:**
```markdown
You are a [role] helping with [purpose].

When invoked:
1. [First action]
2. [Second action]

## Output Format
[How to present results]

## Constraints
[Limitations]
```

**Rules:**
- Start with role definition
- Specify immediate actions
- Use imperative language
- Keep 50-200 lines
- Define output format

### 4. File Generation

```markdown
---
name: agent-name
description: What it does. Use when [specific trigger].
tools: Tool1, Tool2
model: sonnet
permissionMode: default
---

[System prompt]
```

### 5. Validation

- Verify YAML syntax
- Test with Task tool
- Check permissions work
- Iterate based on behavior

## Detailed References

- **[Configuration](references/configuration.md)**: Tools, models, permissions, hooks
- **[Examples](references/examples.md)**: Prompt templates and patterns

## Common Patterns

| Pattern | Tools | Model | Permission |
|---------|-------|-------|------------|
| Read-only research | Read, Grep, Glob | Haiku | plan |
| Code modification | Read, Write, Edit | Sonnet | acceptEdits |
| Command executor | Bash, Read | Haiku | default |
| Validated action | Bash + hooks | Haiku | default |

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Agent not used | Make description more specific |
| Permission blocked | Check permissionMode and tools |
| Context overflow | Shorten prompt, use Haiku |
| Wrong model | Haiku=simple, Sonnet=balanced, Opus=complex |
