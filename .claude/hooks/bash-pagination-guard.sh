#!/bin/bash
# Bash Pagination Guard - Prevents unpaginated commands that could cause memory issues
# Exit code 2 blocks the operation and provides feedback to the agent

# Read the JSON input from stdin
input=$(cat)

# Extract the command from JSON
command=$(echo "$input" | jq -r '.parameters.command // empty')

if [ -z "$command" ]; then
  exit 0
fi

# Dangerous patterns that need pagination
declare -a dangerous_patterns=(
  'grep -r[^ ]*\s'
  'rg -r[^ ]*\s'
  'find[^|]*$'
  'cat[^|]*$'
  'docker logs[^|]*$'
  'kubectl logs[^|]*$'
  'journalctl[^|]*$'
  'tail -f'
  'git log[^|]*$'
  'ls -R[^|]*$'
)

# Check if command matches dangerous patterns and lacks pagination
for pattern in "${dangerous_patterns[@]}"; do
  if echo "$command" | grep -qE "$pattern"; then
    # Check if it has pagination (head, tail, less, more, grep -m)
    if ! echo "$command" | grep -qE '\|\s*(head|tail|less|more|grep -[^|]*m)'; then
      echo "BLOCKED: Command may generate massive output without pagination."
      echo ""
      echo "Dangerous pattern detected: Commands like grep -r, find, cat, docker logs, kubectl logs"
      echo "must include pagination to prevent memory exhaustion."
      echo ""
      echo "Examples of safe alternatives:"
      echo "  - grep -r 'pattern' . | head -100"
      echo "  - find . -name '*.txt' | head -50"
      echo "  - cat large_file.txt | tail -100"
      echo "  - docker logs --tail 100 container_name"
      echo "  - kubectl logs --tail 50 pod_name"
      echo "  - git log --oneline | head -20"
      echo ""
      echo "Add '| head -N' or equivalent pagination to your command."
      exit 2
    fi
  fi
done

# Allow the command to proceed
exit 0
