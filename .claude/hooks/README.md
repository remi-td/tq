# Claude Code Memory Protection Hooks

This directory contains pre-hook guards that prevent memory exhaustion issues in Claude Code sessions.

## Problem

Claude Code stores all bash command output in memory for the entire session, which can cause catastrophic memory usage (90GB+) and crashes when running commands with substantial output. See: https://github.com/anthropics/claude-code/issues/11155

## Solution

These pre-hooks validate tool usage before execution and block dangerous operations that could cause memory issues.

## Hooks

### 1. bash-pagination-guard.sh

Blocks unpaginated bash commands that generate massive output.

**Blocked patterns:**
- `grep -r` without pagination
- `find` without pagination
- `cat` large files without pagination
- `docker logs` without limits
- `kubectl logs` without limits
- `git log` without limits
- `tail -f` (follow mode)

**Example:**
```bash
# ❌ BLOCKED
grep -r "pattern" .

# ✅ ALLOWED
grep -r "pattern" . | head -100
```

### 2. read-memory-guard.sh

Prevents reading files exceeding size/line limits without pagination.

**Limits:**
- File size: 50MB
- Line count: 50,000 lines

**Example:**
```json
// ❌ BLOCKED for large files
{ "file_path": "large_file.txt" }

// ✅ ALLOWED with pagination
{ "file_path": "large_file.txt", "offset": 0, "limit": 1000 }
```

### 3. grep-tool-guard.sh

Enforces head_limit on all Grep tool operations.

**Requirements:**
- `head_limit` must be specified
- `head_limit` must be ≤ 5000

**Example:**
```json
// ❌ BLOCKED
{ "pattern": "function", "path": "src/" }

// ✅ ALLOWED
{ "pattern": "function", "path": "src/", "head_limit": 100 }
```

## Configuration

To enable these hooks in Claude Code, add the following to your settings:

### Project Settings (Recommended)

Add to `.claude/settings.local.json` in your project:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/bash-pagination-guard.sh"
          }
        ]
      },
      {
        "matcher": "Read",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/read-memory-guard.sh"
          }
        ]
      },
      {
        "matcher": "Grep",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/grep-tool-guard.sh"
          }
        ]
      }
    ]
  }
}
```

### Global Settings (Alternative)

If you want the hooks to work in any project:

1. Copy the hooks to a global location (e.g., `~/.claude/hooks/`)
2. Add to your global Claude Code settings with absolute paths:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/bash-pagination-guard.sh"
          }
        ]
      },
      {
        "matcher": "Read",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/read-memory-guard.sh"
          }
        ]
      },
      {
        "matcher": "Grep",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/grep-tool-guard.sh"
          }
        ]
      }
    ]
  }
}
```

## How It Works

1. Before each tool use, Claude Code executes the corresponding hook script
2. The hook receives the tool call parameters as JSON on stdin
3. The hook validates the parameters and returns:
   - Exit code 0: Allow the operation
   - Exit code 2: Block the operation (with helpful message)
4. If blocked, Claude sees the error message and can retry with safe parameters

## Testing

You can test the hooks manually:

```bash
# Test bash pagination guard
echo '{"parameters":{"command":"grep -r pattern ."}}' | ./.claude/hooks/bash-pagination-guard.sh
echo $?  # Should return 2 (blocked)

echo '{"parameters":{"command":"grep -r pattern . | head -100"}}' | ./.claude/hooks/bash-pagination-guard.sh
echo $?  # Should return 0 (allowed)

# Test read memory guard (with a large file)
echo '{"parameters":{"file_path":"large_file.txt"}}' | ./.claude/hooks/read-memory-guard.sh

# Test grep tool guard
echo '{"parameters":{"pattern":"test"}}' | ./.claude/hooks/grep-tool-guard.sh
echo $?  # Should return 2 (blocked)

echo '{"parameters":{"pattern":"test","head_limit":100}}' | ./.claude/hooks/grep-tool-guard.sh
echo $?  # Should return 0 (allowed)
```

## Benefits

Since implementing these hooks:
- ✅ Prevents OOM crashes from unbounded output
- ✅ Forces pagination on dangerous operations
- ✅ Provides clear guidance to Claude on safe alternatives
- ✅ No false positives in normal development workflows

## Limitations

This is a **workaround**, not a fix for the root cause. Claude Code should:
- Stream and discard bash output instead of storing it all in memory
- Implement automatic pagination for large outputs
- Provide memory management controls

## References

- Issue: https://github.com/anthropics/claude-code/issues/11155
- Implementation credit: Sceat (GitHub commenter)
