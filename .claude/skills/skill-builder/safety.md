# Markdown Safety Guide

Avoid shell-sensitive characters that can break skill loading.

## The Problem

Claude Code's skill loader performs bash command safety checks. When inline backticks surround shell-sensitive characters, the parser may interpret content as bash commands, causing skill loading to fail.

## Shell-Sensitive Characters

Watch out for these in inline backticks:
- `!` - Bash history expansion
- `$` - Variable expansion
- `&&` - Command chaining
- `||` - Command chaining
- `;` - Command separator
- `` ` `` - Command substitution (backticks in backticks)

## Common Problematic Patterns

| Problem | Error Symptom |
|---------|---------------|
| `` `//!` `` (Rust inner doc) | Bash permission error |
| `` `$VAR` `` | Variable expansion |
| `` `cmd && cmd` `` | Command chaining |

## Safe Documentation Strategies

### 1. Describe Instead of Showing

**Good:**
- "Use inner doc comments (two slashes followed by exclamation mark)"
- "Use the dollar sign for shell variables"
- "Chain commands with double ampersand operator"

**Bad:**
- `` `//!` `` in inline code
- `` `$HOME` `` in inline code

### 2. Use Code Blocks Instead of Inline Backticks

Fenced code blocks are safer:

```rust
//! This is a module doc comment
```

### 3. Use Descriptive Names

- "slash-slash-bang syntax"
- "dollar-sign variable syntax"
- "double-ampersand operator"

## Testing Requirement

After creating or modifying a skill:

1. Create/modify the skill file
2. Invoke the skill to test loading
3. If loading fails with bash permission errors, search for shell-sensitive characters
4. Rewrite using plain descriptions or code blocks
5. Test loading again to confirm

## Error Example

If you see "Bash command permission check failed" when loading a skill, look for inline backticks containing:
- Exclamation marks
- Dollar signs
- Shell operators

Rewrite those sections using plain descriptions.
