#!/bin/bash
# Task Memory Guard - Warns about potentially memory-intensive sub-agent launches
# Exit code 0 allows, exit code 1 warns (informational only)

# Read the JSON input from stdin
input=$(cat)

# Extract parameters
subagent_type=$(echo "$input" | jq -r '.parameters.subagent_type // empty')
model=$(echo "$input" | jq -r '.parameters.model // empty')
description=$(echo "$input" | jq -r '.parameters.description // empty')

if [ -z "$subagent_type" ]; then
  exit 0
fi

# Check current memory usage
current_pid=$$
parent_pid=$(ps -o ppid= -p $current_pid | tr -d ' ')
if [ -n "$parent_pid" ]; then
  # Get memory usage of parent Claude process (in KB)
  mem_kb=$(ps -o rss= -p $parent_pid 2>/dev/null | tr -d ' ')

  if [ -n "$mem_kb" ] && [ "$mem_kb" -gt 8388608 ]; then  # 8GB in KB
    echo "⚠️  WARNING: High memory usage detected before launching sub-agent"
    echo ""
    echo "Current process memory: $(echo "scale=2; $mem_kb / 1024 / 1024" | bc 2>/dev/null || echo "N/A")GB"
    echo "Sub-agent: $subagent_type"
    echo "Model: $model"
    echo ""
    echo "Consider:"
    echo "  - Restarting the session to clear accumulated memory"
    echo "  - Using model: haiku for less memory-intensive operations"
    echo "  - Breaking the task into smaller chunks"
    echo ""
    echo "Allowing the operation to proceed..."
  fi
fi

# Log the sub-agent launch for debugging
echo "Launching sub-agent: $subagent_type (model: $model, task: $description)" >> ~/.claude/subagent-launches.log

# Always allow (exit 0), this is just monitoring
exit 0
