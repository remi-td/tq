#!/bin/bash
# SessionStart hook to track session ID and transcript path

# Read JSON from stdin
read -r json_input

# Extract session_id and transcript_path using jq
session_id=$(echo "$json_input" | jq -r '.session_id')
transcript_path=$(echo "$json_input" | jq -r '.transcript_path')
source=$(echo "$json_input" | jq -r '.source')

# Write to tracking files
echo "$session_id" > .claude/current-session-id.txt
echo "$transcript_path" > .claude/current-transcript-path.txt

# Append to session history with timestamp
echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) $session_id $source" >> .claude/session-history.txt

exit 0
