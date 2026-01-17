#!/bin/bash
# Crash Reporter - Logs session state before potential crashes
# This is a SessionEnd hook that captures diagnostic information

# Get session ID from environment or parameter
session_id="${CLAUDE_SESSION_ID:-unknown}"
timestamp=$(date +"%Y-%m-%d_%H-%M-%S")
report_file="$HOME/.claude/crash-reports/crash-$timestamp-$session_id.txt"

# Create crash reports directory
mkdir -p "$HOME/.claude/crash-reports"

# Gather diagnostic information
{
  echo "=== Claude Code Session End Report ==="
  echo "Timestamp: $timestamp"
  echo "Session ID: $session_id"
  echo ""

  echo "=== Memory Usage ==="
  ps aux | grep "[c]laude" | head -5
  echo ""

  echo "=== Top Memory Consumers ==="
  ps aux | sort -k4 -r | head -10
  echo ""

  echo "=== Recent Sub-Agent Launches ==="
  if [ -f "$HOME/.claude/subagent-launches.log" ]; then
    tail -20 "$HOME/.claude/subagent-launches.log"
  else
    echo "No sub-agent launch log found"
  fi
  echo ""

  echo "=== System Memory ==="
  vm_stat | head -20
  echo ""

  echo "=== Recent Errors (last 50 lines of debug log) ==="
  if [ -f "$HOME/.claude/debug/latest" ]; then
    grep -i "error\|abort\|crash\|fatal" "$(readlink $HOME/.claude/debug/latest)" | tail -50
  fi

} > "$report_file"

echo "Session diagnostics saved to: $report_file"

# Rotate old crash reports (keep last 10)
cd "$HOME/.claude/crash-reports" 2>/dev/null && ls -t crash-*.txt 2>/dev/null | tail -n +11 | xargs rm -f 2>/dev/null

exit 0
