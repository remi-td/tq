#!/bin/bash
# Grep Tool Guard - Enforces limits on Grep tool operations
# Exit code 2 blocks the operation and provides feedback to the agent

# Read the JSON input from stdin
input=$(cat)

# Extract head_limit parameter
head_limit=$(echo "$input" | jq -r '.parameters.head_limit // empty')

# If head_limit is not set or is 0 (unlimited), block the operation
if [ -z "$head_limit" ] || [ "$head_limit" = "null" ] || [ "$head_limit" = "0" ]; then
  echo "BLOCKED: Grep operations must specify a head_limit to prevent memory exhaustion."
  echo ""
  echo "The Grep tool can return massive amounts of output that will be stored in memory."
  echo "You MUST set the head_limit parameter to a reasonable value."
  echo ""
  echo "Examples:"
  echo "  - For quick searches: head_limit: 100"
  echo "  - For thorough searches: head_limit: 500"
  echo "  - For comprehensive searches: head_limit: 1000"
  echo ""
  echo "Add the head_limit parameter to your Grep tool call."
  exit 2
fi

# Check if head_limit is reasonable (not too large)
if [ "$head_limit" -gt 5000 ]; then
  echo "BLOCKED: head_limit of $head_limit is too large and may cause memory issues."
  echo ""
  echo "Please use a head_limit of 5000 or less."
  echo "If you need more results, consider:"
  echo "  - Refining your search pattern"
  echo "  - Using multiple targeted searches"
  echo "  - Using the offset parameter for pagination"
  exit 2
fi

# Allow the grep to proceed
exit 0
