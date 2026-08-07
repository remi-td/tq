#!/bin/bash
# Extract token usage metrics from subagent transcripts for a sprint
# Usage: ./extract-sprint-metrics.sh <session-id> <sprint-number>
#
# This script analyzes subagent transcripts to calculate token usage per agent

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"

# Parse arguments: can be either (sprint-num) or (session-id sprint-num)
if [ $# -eq 1 ]; then
    # Single argument: treat as sprint number, use current session
    SESSION_ID=""
    SPRINT_NUM=$1
elif [ $# -eq 2 ]; then
    # Two arguments: session-id and sprint-num
    SESSION_ID=$1
    SPRINT_NUM=$2
else
    SESSION_ID=""
    SPRINT_NUM=""
fi

# If no session ID provided, try to use current session from hook
if [ -z "$SESSION_ID" ]; then
    SESSION_ID_FILE="$PROJECT_ROOT/.claude/current-session-id.txt"
    if [ -f "$SESSION_ID_FILE" ]; then
        SESSION_ID=$(cat "$SESSION_ID_FILE" | tr -d '\n\r ')
        echo "Using current session ID from hook: $SESSION_ID" >&2
    fi
fi

# If still no session ID or sprint number, show usage
if [ -z "$SESSION_ID" ] || [ -z "$SPRINT_NUM" ]; then
    echo "Usage: $0 [<session-id>] <sprint-number>"
    echo ""
    echo "If session-id is not provided, uses the current session (captured via hook)."
    echo ""
    echo "To find past session IDs:"
    echo "  # List recent sessions for this project"
    echo "  ls -t ~/.claude/projects/\$(pwd | sed 's|/|-|g; s|\.|-|g')/*.jsonl | head -5"
    echo ""
    echo "Example:"
    echo "  $0 f599ef4e-6741-40b9-8b70-54c6e6d7272e 18   # Use specific session"
    echo "  $0 22                                        # Use current session for sprint 22"
    exit 1
fi

# Derive output file path from sprint number
OUTPUT_FILE="$PROJECT_ROOT/docs/sprints/sprint-${SPRINT_NUM}-metrics.md"

# Find the Claude project directory
# Claude encodes paths by replacing / with - and . with -
CLAUDE_PROJECTS_ROOT="$HOME/.claude/projects"
ENCODED_PATH=$(echo "$PROJECT_ROOT" | sed 's|/|-|g; s|\.|-|g')
PROJECT_DIR="$CLAUDE_PROJECTS_ROOT/$ENCODED_PATH"

# Verify the session transcript exists
MAIN_TRANSCRIPT="$PROJECT_DIR/$SESSION_ID.jsonl"
if [ ! -f "$MAIN_TRANSCRIPT" ]; then
    # Try searching for it
    FOUND_TRANSCRIPT=$(find "$CLAUDE_PROJECTS_ROOT" -type f -name "${SESSION_ID}.jsonl" 2>/dev/null | head -1)
    if [ -n "$FOUND_TRANSCRIPT" ]; then
        MAIN_TRANSCRIPT="$FOUND_TRANSCRIPT"
        PROJECT_DIR=$(dirname "$MAIN_TRANSCRIPT")
    else
        echo "Error: Session transcript not found: $MAIN_TRANSCRIPT" >&2
        echo "" >&2
        echo "To list available sessions:" >&2
        echo "  ls -t $PROJECT_DIR/*.jsonl 2>/dev/null | head -5" >&2
        exit 1
    fi
fi

SUBAGENT_DIR="$PROJECT_DIR/$SESSION_ID/subagents"

if [ ! -d "$SUBAGENT_DIR" ]; then
    echo "Warning: Subagent directory not found: $SUBAGENT_DIR" >&2
    echo "Will only process main session transcript" >&2
fi

# Ensure output directory exists
mkdir -p "$(dirname "$OUTPUT_FILE")"

# Generate all output (will be captured by exec redirection)
generate_metrics() {
echo "# Sprint $SPRINT_NUM - Token Usage Metrics"
echo ""
echo "**Session ID:** $SESSION_ID"
echo "**Generated:** $(date)"
echo ""
echo "---"
echo ""

# Function to extract agent name from transcript
get_agent_name() {
    local transcript=$1
    # Extract from first message content (which is a string)
    local name=$(jq -r 'select(.message.content) | .message.content' "$transcript" 2>/dev/null | \
        grep -oE "rust-teradata-architect|quality-validator|cli-ux-designer|tq-project-manager|general-purpose|Explore" | \
        head -1)

    if [ -z "$name" ]; then
        echo "unknown"
    else
        echo "$name"
    fi
}

# Initialize totals
total_input=0
total_output=0
total_cache_creation=0
total_cache_read=0

echo "## Token Usage by Agent"
echo ""

# Process main session first
echo "### Agent: sprint-coordinator (Main Session)"
echo ""

set +e
read main_input main_output main_cache_create main_cache_read < <(
    jq -r 'select(.message.usage) |
           [.message.usage.input_tokens // 0,
            .message.usage.output_tokens // 0,
            .message.usage.cache_creation_input_tokens // 0,
            .message.usage.cache_read_input_tokens // 0] |
           @tsv' "$MAIN_TRANSCRIPT" | \
    awk '{
        input += $1
        output += $2
        cache_create += $3
        cache_read += $4
    } END {
        printf "%d %d %d %d", input, output, cache_create, cache_read
    }'
)
set -e

# Calculate totals for main session
main_total=$((main_input + main_output + main_cache_create + main_cache_read))
main_input_processing=$((main_input + main_cache_create + main_cache_read))
main_cache_rate=0
if [ $main_input_processing -gt 0 ]; then
    main_cache_rate=$(awk "BEGIN {printf \"%.1f\", ($main_cache_read / $main_input_processing) * 100}")
fi

# Update grand totals
total_input=$((total_input + main_input))
total_output=$((total_output + main_output))
total_cache_creation=$((total_cache_creation + main_cache_create))
total_cache_read=$((total_cache_read + main_cache_read))

echo "| Metric | Value |"
echo "|--------|-------|"
echo "| Input Tokens | $(printf "%'d" $main_input) |"
echo "| Output Tokens | $(printf "%'d" $main_output) |"
echo "| Cache Creation | $(printf "%'d" $main_cache_create) |"
echo "| Cache Reads | $(printf "%'d" $main_cache_read) |"
echo "| **Total Tokens** | **$(printf "%'d" $main_total)** |"
echo "| Cache Hit Rate | ${main_cache_rate}% |"
echo ""

# Process each subagent transcript (if directory exists)
if [ -d "$SUBAGENT_DIR" ]; then
    for transcript in "$SUBAGENT_DIR"/agent-*.jsonl; do
        if [ ! -f "$transcript" ]; then
            continue
        fi

    agent_id=$(basename "$transcript" .jsonl | sed 's/agent-//')
    agent_name=$(get_agent_name "$transcript")

    # Sum up all token usage from this agent
    # Disable exit-on-error for this read command (process substitution in subshell can be tricky)
    set +e
    read input output cache_create cache_read < <(
        jq -r 'select(.message.usage) |
               [.message.usage.input_tokens // 0,
                .message.usage.output_tokens // 0,
                .message.usage.cache_creation_input_tokens // 0,
                .message.usage.cache_read_input_tokens // 0] |
               @tsv' "$transcript" | \
        awk '{
            input += $1
            output += $2
            cache_create += $3
            cache_read += $4
        } END {
            printf "%d %d %d %d", input, output, cache_create, cache_read
        }'
    )
    set -e

    # Calculate totals
    total_tokens=$((input + output + cache_create + cache_read))
    # Cache hit rate = cache_read / (total input processing)
    total_input_processing=$((input + cache_create + cache_read))
    cache_rate=0
    if [ $total_input_processing -gt 0 ]; then
        cache_rate=$(awk "BEGIN {printf \"%.1f\", ($cache_read / $total_input_processing) * 100}")
    fi

    # Update grand totals
    total_input=$((total_input + input))
    total_output=$((total_output + output))
    total_cache_creation=$((total_cache_creation + cache_create))
    total_cache_read=$((total_cache_read + cache_read))

    echo "### Agent: $agent_name (ID: $agent_id)"
    echo ""
    echo "| Metric | Value |"
    echo "|--------|-------|"
    echo "| Input Tokens | $(printf "%'d" $input) |"
    echo "| Output Tokens | $(printf "%'d" $output) |"
    echo "| Cache Creation | $(printf "%'d" $cache_create) |"
    echo "| Cache Reads | $(printf "%'d" $cache_read) |"
    echo "| **Total Tokens** | **$(printf "%'d" $total_tokens)** |"
    echo "| Cache Hit Rate | ${cache_rate}% |"
    echo ""
    done
fi

# Calculate grand totals
grand_total=$((total_input + total_output + total_cache_creation + total_cache_read))
total_input_processing=$((total_input + total_cache_creation + total_cache_read))
overall_cache_rate=0
if [ $total_input_processing -gt 0 ]; then
    overall_cache_rate=$(awk "BEGIN {printf \"%.1f\", ($total_cache_read / $total_input_processing) * 100}")
fi

# Check for previous sprint metrics to compute per-sprint delta if in same session
PREV_SPRINT=$((SPRINT_NUM - 1))
PREV_METRICS_FILE="$PROJECT_ROOT/docs/sprints/sprint-${PREV_SPRINT}-metrics.md"
IS_DELTA=0

if [ -f "$PREV_METRICS_FILE" ]; then
    PREV_SESSION=$(grep "\*\*Session ID:\*\*" "$PREV_METRICS_FILE" | head -1 | awk '{print $NF}')
    if [ "$PREV_SESSION" = "$SESSION_ID" ]; then
        IS_DELTA=1
        prev_input=$(grep "^| Total Input Tokens" "$PREV_METRICS_FILE" | head -1 | sed 's/[^0-9,]//g' | tr -d ',')
        prev_output=$(grep "^| Total Output Tokens" "$PREV_METRICS_FILE" | head -1 | sed 's/[^0-9,]//g' | tr -d ',')
        prev_cache_create=$(grep "^| Total Cache Creation" "$PREV_METRICS_FILE" | head -1 | sed 's/[^0-9,]//g' | tr -d ',')
        prev_cache_read=$(grep "^| Total Cache Reads" "$PREV_METRICS_FILE" | head -1 | sed 's/[^0-9,]//g' | tr -d ',')

        delta_input=$((total_input - prev_input))
        delta_output=$((total_output - prev_output))
        delta_cache_create=$((total_cache_creation - prev_cache_create))
        delta_cache_read=$((total_cache_read - prev_cache_read))
        delta_grand=$((grand_total - (prev_input + prev_output + prev_cache_create + prev_cache_read)))

        delta_input_proc=$((delta_input + delta_cache_create + delta_cache_read))
        delta_cache_rate=0
        if [ $delta_input_proc -gt 0 ]; then
            delta_cache_rate=$(awk "BEGIN {printf \"%.1f\", ($delta_cache_read / $delta_input_proc) * 100}")
        fi
    fi
fi

echo "---"
echo ""
echo "## Sprint Summary"
echo ""
if [ $IS_DELTA -eq 1 ]; then
    echo "*Note: Session shared with Sprint $PREV_SPRINT. Showing per-sprint delta (and cumulative total).* "
    echo ""
    echo "| Metric | Sprint Delta | Cumulative Session Total |"
    echo "|--------|--------------|--------------------------|"
    echo "| Input Tokens | $(printf "%'d" $delta_input) | $(printf "%'d" $total_input) |"
    echo "| Output Tokens | $(printf "%'d" $delta_output) | $(printf "%'d" $total_output) |"
    echo "| Cache Creation | $(printf "%'d" $delta_cache_create) | $(printf "%'d" $total_cache_creation) |"
    echo "| Cache Reads | $(printf "%'d" $delta_cache_read) | $(printf "%'d" $total_cache_read) |"
    echo "| **Grand Total** | **$(printf "%'d" $delta_grand)** | **$(printf "%'d" $grand_total)** |"
    echo "| Cache Hit Rate | ${delta_cache_rate}% | ${overall_cache_rate}% |"
    
    cost_input_val=$delta_input
    cost_output_val=$delta_output
    cost_cache_create_val=$delta_cache_create
    cost_cache_read_val=$delta_cache_read
else
    echo "| Metric | Value |"
    echo "|--------|-------|"
    echo "| Total Input Tokens | $(printf "%'d" $total_input) |"
    echo "| Total Output Tokens | $(printf "%'d" $total_output) |"
    echo "| Total Cache Creation | $(printf "%'d" $total_cache_creation) |"
    echo "| Total Cache Reads | $(printf "%'d" $total_cache_read) |"
    echo "| **Grand Total** | **$(printf "%'d" $grand_total)** |"
    echo "| Overall Cache Hit Rate | ${overall_cache_rate}% |"
    
    cost_input_val=$total_input
    cost_output_val=$total_output
    cost_cache_create_val=$total_cache_creation
    cost_cache_read_val=$total_cache_read
fi
echo ""

# Calculate estimated costs (2026 pricing)
cost_input=$(awk "BEGIN {printf \"%.2f\", ($cost_input_val + $cost_cache_create_val) * 3 / 1000000}")
cost_output=$(awk "BEGIN {printf \"%.2f\", $cost_output_val * 15 / 1000000}")
cost_cache=$(awk "BEGIN {printf \"%.2f\", $cost_cache_read_val * 0.30 / 1000000}")
cost_total=$(awk "BEGIN {printf \"%.2f\", $cost_input + $cost_output + $cost_cache}")

echo "## Estimated Cost (Sonnet 4.5 pricing)"
echo ""
echo "| Category | Cost |"
echo "|----------|------|"
echo "| Input Tokens | \$$cost_input |"
echo "| Output Tokens | \$$cost_output |"
echo "| Cache Reads | \$$cost_cache |"
echo "| **Total** | **\$$cost_total** |"
echo ""
echo "**Note:** Actual costs may vary based on model mix (Opus/Sonnet/Haiku)."
echo ""

}

# Call function and save output to file while also displaying it
generate_metrics | tee "$OUTPUT_FILE"

echo "" >&2
echo "✅ Metrics saved to: $OUTPUT_FILE" >&2
