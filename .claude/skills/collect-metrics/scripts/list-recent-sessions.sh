#!/bin/bash
# List recent Claude sessions for this project
# Usage: ./list-recent-sessions.sh [days]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"

# Default to last 7 days if not specified
DAYS=${1:-7}

# Find the Claude project directory
CLAUDE_PROJECTS_ROOT="$HOME/.claude/projects"
ENCODED_PATH=$(echo "$PROJECT_ROOT" | sed 's|/|-|g; s|\.|-|g')
PROJECT_DIR="$CLAUDE_PROJECTS_ROOT/$ENCODED_PATH"

if [ ! -d "$PROJECT_DIR" ]; then
    echo "Error: Claude project directory not found: $PROJECT_DIR" >&2
    exit 1
fi

echo "Recent sessions (last $DAYS days):"
echo ""
echo "Session ID                            | Date                | Subagents"
echo "--------------------------------------|---------------------|----------"

# Find sessions modified in the last N days
find "$PROJECT_DIR" -maxdepth 1 -name "*.jsonl" -type f -mtime -${DAYS} -print0 | \
    xargs -0 ls -lt | \
    awk '{print $NF}' | \
    while read transcript; do
        session_id=$(basename "$transcript" .jsonl)
        mod_date=$(stat -f "%Sm" -t "%Y-%m-%d %H:%M" "$transcript" 2>/dev/null || stat -c "%y" "$transcript" 2>/dev/null | cut -d' ' -f1,2 | cut -d. -f1)

        # Count subagents
        subagent_dir="$PROJECT_DIR/$session_id/subagents"
        if [ -d "$subagent_dir" ]; then
            subagent_count=$(ls -1 "$subagent_dir"/*.jsonl 2>/dev/null | wc -l | tr -d ' ')
        else
            subagent_count=0
        fi

        printf "%-38s| %-20s| %d\n" "$session_id" "$mod_date" "$subagent_count"
    done

echo ""
echo "To extract metrics for a session:"
echo "  ./extract-sprint-metrics.sh <session-id> <sprint-number>"
echo ""
echo "To combine metrics from multiple sessions:"
echo "  ./combine-sprint-metrics.sh <sprint-number> <session-id-1> <session-id-2> ..."
