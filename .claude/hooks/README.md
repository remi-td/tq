# Claude Code Memory Protection Hooks

This directory contains pre-hook guards that prevent memory exhaustion issues in Claude Code sessions.

## Problem

Claude Code stores all bash command output in memory for the entire session, which can cause catastrophic memory usage (90GB+) and crashes when running commands with substantial output. See: https://github.com/anthropics/claude-code/issues/11155

## Solution

These pre-hooks validate tool usage before execution and block dangerous operations that could cause memory issues.

## Hooks

### 1. bash-pagination-guard.sh (CRITICAL)

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

### 2. read-memory-guard.sh (CRITICAL)

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

### 3. grep-tool-guard.sh (CRITICAL)

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

### 4. task-memory-guard.sh (MONITORING)

Monitors sub-agent launches and warns when memory usage is high.

**Purpose:**
- Tracks all Task tool calls that launch sub-agents
- Warns if parent process is using > 8GB memory before launching
- Logs all sub-agent launches for debugging

**Example:**
```
⚠️  WARNING: High memory usage detected before launching sub-agent

Current process memory: 10.24GB
Sub-agent: tq-project-manager
Model: opus

Consider:
  - Restarting the session to clear accumulated memory
  - Using model: haiku for less memory-intensive operations
  - Breaking the task into smaller chunks
```

### 5. crash-reporter.sh (DIAGNOSTIC)

SessionEnd hook that captures diagnostic information when a session ends.

**Captures:**
- Memory usage of all Claude processes
- Recent sub-agent launch history
- System memory statistics
- Recent errors from debug logs

**Output:**
- Crash reports saved to `~/.claude/crash-reports/`
- Automatically rotates old reports (keeps last 10)

## Monitoring Tools

### monitor-memory.sh

Quick utility to check current memory usage:

```bash
./.claude/hooks/monitor-memory.sh
```

Output example:
```
=== Claude Code Memory Usage ===

PID  3102:  10.24 GB  CPU: 363.4%  /path/to/claude ⚠️  HIGH
PID  5806:   0.37 GB  CPU:  97.7%  /path/to/claude

Total Claude memory: 10.61GB

=== System Memory ===
free:              2.45 GB
active:           12.31 GB
inactive:          8.92 GB
```

### Sub-Agent Launch Log

All sub-agent launches are logged to `~/.claude/subagent-launches.log`:

```bash
tail -f ~/.claude/subagent-launches.log
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
      },
      {
        "matcher": "Task",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/task-memory-guard.sh"
          }
        ]
      }
    ],
    "SessionEnd": [
      {
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/crash-reporter.sh"
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
- ✅ Monitors sub-agent launches and warns about memory usage
- ✅ Captures crash diagnostics for post-mortem analysis

## What to Do If You Experience Crashes

If Claude Code crashes with an Abort() message or becomes unresponsive:

### Immediate Actions

1. **Check crash report**:
   ```bash
   ls -lt ~/.claude/crash-reports/ | head -5
   cat ~/.claude/crash-reports/crash-<latest>.txt
   ```

2. **Check current memory usage**:
   ```bash
   ./.claude/hooks/monitor-memory.sh
   ```

3. **Review recent sub-agent launches**:
   ```bash
   tail -20 ~/.claude/subagent-launches.log
   ```

### Prevention Strategies

1. **Restart sessions proactively**: If memory exceeds 5GB, consider restarting Claude Code
2. **Use Haiku for sub-agents**: When launching agents with the Task tool, prefer `model: haiku` instead of `opus` unless complex reasoning is required
3. **Avoid parallel sub-agents**: Launching multiple sub-agents in parallel multiplies memory consumption
4. **Monitor during long sessions**: Run `monitor-memory.sh` periodically during extended work sessions
5. **Check agent configurations**: Review `.claude/agents/*.md` files - agents configured with `model: opus` consume more memory

### Known Memory-Intensive Operations

- **tq-project-manager agent**: Uses `model: opus` and can launch multiple sub-agents - VERY memory intensive
- **Large file operations**: Even with hooks, reading many large files accumulates memory
- **Long-running sessions**: Claude Code never releases memory from tool outputs during a session
- **Sub-agent cascades**: When a sub-agent launches another sub-agent, memory compounds

## Limitations

This is a **workaround**, not a fix for the root cause. Claude Code should:
- Stream and discard bash output instead of storing it all in memory
- Implement automatic pagination for large outputs
- Provide memory management controls

## References

- Issue: https://github.com/anthropics/claude-code/issues/11155
- Implementation credit: Sceat (GitHub commenter)
