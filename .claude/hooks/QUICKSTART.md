# Memory Protection Quick Start

## TL;DR

Your Claude Code session is protected by memory guards, but you should still monitor memory usage.

## Quick Checks

**Check current memory:**
```bash
./.claude/hooks/monitor-memory.sh
```

**Check recent sub-agent activity:**
```bash
tail -20 ~/.claude/subagent-launches.log
```

**Check if crash reports exist:**
```bash
ls -lt ~/.claude/crash-reports/ | head -5
```

## When to Restart

⚠️  Restart Claude Code if:
- Memory usage exceeds 5GB (check with monitor-memory.sh)
- Session has been running for > 2 hours with heavy sub-agent use
- You see "High memory usage" warnings when launching sub-agents
- Claude becomes slow or unresponsive

## Memory-Saving Tips

1. **Use Haiku for routine tasks**: Add `model: haiku` when launching sub-agents
2. **Avoid parallel sub-agents**: Launch one at a time instead of 3+ in parallel
3. **Watch the tq-project-manager**: This agent uses Opus and launches many sub-agents
4. **Restart proactively**: Don't wait for a crash - restart when memory is high

## After a Crash

1. Check the crash report:
   ```bash
   cat ~/.claude/crash-reports/crash-<latest>.txt
   ```

2. Look for patterns:
   - Which sub-agents were running?
   - What was the memory usage?
   - Were there parallel sub-agent launches?

3. Adjust your workflow to avoid the pattern that caused the crash

## Protection Status

Current protections installed:
- ✅ Bash pagination guard (prevents massive command output)
- ✅ Read memory guard (blocks reading huge files without pagination)
- ✅ Grep tool guard (enforces head_limit on searches)
- ✅ Task memory monitor (warns about high memory before sub-agent launches)
- ✅ Crash reporter (captures diagnostics on session end)

These are **workarounds** for a Claude Code issue (https://github.com/anthropics/claude-code/issues/11155), not permanent fixes.
