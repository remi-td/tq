#!/bin/bash
# Memory Monitor - Quick script to check current Claude Code memory usage

echo "=== Claude Code Memory Usage ==="
echo ""

# Find all Claude processes
claude_pids=$(ps aux | grep "[c]laude" | awk '{print $2}')

if [ -z "$claude_pids" ]; then
  echo "No Claude processes found"
  exit 0
fi

total_mem_kb=0
for pid in $claude_pids; do
  # Get process info
  mem_kb=$(ps -o rss= -p $pid 2>/dev/null | tr -d ' ')
  cpu=$(ps -o %cpu= -p $pid 2>/dev/null | tr -d ' ')
  cmd=$(ps -o command= -p $pid 2>/dev/null | head -c 50)

  if [ -n "$mem_kb" ]; then
    mem_gb=$(echo "scale=2; $mem_kb / 1024 / 1024" | bc 2>/dev/null)
    total_mem_kb=$((total_mem_kb + mem_kb))

    # Color code based on memory usage
    if [ "$mem_kb" -gt 10485760 ]; then  # > 10GB
      color="\033[1;31m"  # Red
      warning=" ⚠️  HIGH"
    elif [ "$mem_kb" -gt 5242880 ]; then  # > 5GB
      color="\033[1;33m"  # Yellow
      warning=" ⚠️  WARNING"
    else
      color="\033[0;32m"  # Green
      warning=""
    fi
    reset="\033[0m"

    printf "${color}PID %5d: %6.2f GB  CPU: %5s%%  %s${warning}${reset}\n" "$pid" "$mem_gb" "$cpu" "$cmd"
  fi
done

echo ""
total_mem_gb=$(echo "scale=2; $total_mem_kb / 1024 / 1024" | bc 2>/dev/null)
echo "Total Claude memory: ${total_mem_gb}GB"

# System memory info
echo ""
echo "=== System Memory ==="
vm_stat | perl -ne '/page size of (\d+)/ and $size=$1; /Pages\s+([^:]+)[^\d]+(\d+)/ and printf("%-16s % 16.2f GB\n", "$1:", $2 * $size / 1073741824);'

echo ""
echo "To reduce memory usage:"
echo "  1. Exit and restart Claude Code"
echo "  2. Use 'model: haiku' for sub-agents when possible"
echo "  3. Avoid launching multiple parallel sub-agents"
echo "  4. Check ~/.claude/subagent-launches.log for sub-agent activity"
