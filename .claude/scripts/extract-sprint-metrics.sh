#!/bin/bash
# Extract token usage metrics from subagent transcripts for a sprint
# Usage: ./extract-sprint-metrics.sh <session-id> <sprint-number>
#
# This script analyzes subagent transcripts to calculate token usage per agent

set -e

SESSION_ID=${1:-}
SPRINT_NUM=${2:-}

if [ -z "$SESSION_ID" ] || [ -z "$SPRINT_NUM" ]; then
    echo "Usage: $0 <session-id> <sprint-number>"
    echo ""
    echo "To find session IDs:"
    echo "  ls -t ~/.claude/projects/-Users-remi-turpaud-Code-genAI-tq/ | grep -E '^[0-9a-f-]+$' | head -5"
    echo ""
    echo "Example:"
    echo "  $0 f599ef4e-6741-40b9-8b70-54c6e6d7272e 8"
    exit 1
fi

PROJECT_DIR="$HOME/.claude/projects/-Users-remi-turpaud-Code-genAI-tq"
SUBAGENT_DIR="$PROJECT_DIR/$SESSION_ID/subagents"
OUTPUT_FILE="docs/builder/sprints/sprint-${SPRINT_NUM}-metrics.md"

if [ ! -d "$SUBAGENT_DIR" ]; then
    echo "Error: Subagent directory not found: $SUBAGENT_DIR"
    exit 1
fi

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
    # Try to extract from first few messages
    jq -r 'select(.message.content) | .message.content[] | select(.type == "text") | .text' "$transcript" 2>/dev/null | \
        grep -oE "rust-teradata-architect|quality-validator|cli-ux-designer|tq-project-manager|general-purpose|Explore" | \
        head -1 || echo "unknown"
}

# Initialize totals
total_input=0
total_output=0
total_cache_creation=0
total_cache_read=0

echo "## Token Usage by Agent"
echo ""

# Process each subagent transcript
for transcript in "$SUBAGENT_DIR"/agent-*.jsonl; do
    if [ ! -f "$transcript" ]; then
        continue
    fi

    agent_id=$(basename "$transcript" .jsonl | sed 's/agent-//')
    agent_name=$(get_agent_name "$transcript")

    # Sum up all token usage from this agent
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

    # Calculate totals
    total_tokens=$((input + output + cache_create + cache_read))
    effective_input=$((input + cache_create))
    cache_rate=0
    if [ $effective_input -gt 0 ]; then
        cache_rate=$(awk "BEGIN {printf \"%.1f\", ($cache_read / $effective_input) * 100}")
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

# Calculate grand totals
grand_total=$((total_input + total_output + total_cache_creation + total_cache_read))
effective_total_input=$((total_input + total_cache_creation))
overall_cache_rate=0
if [ $effective_total_input -gt 0 ]; then
    overall_cache_rate=$(awk "BEGIN {printf \"%.1f\", ($total_cache_read / $effective_total_input) * 100}")
fi

echo "---"
echo ""
echo "## Sprint Summary"
echo ""
echo "| Metric | Value |"
echo "|--------|-------|"
echo "| Total Input Tokens | $(printf "%'d" $total_input) |"
echo "| Total Output Tokens | $(printf "%'d" $total_output) |"
echo "| Total Cache Creation | $(printf "%'d" $total_cache_creation) |"
echo "| Total Cache Reads | $(printf "%'d" $total_cache_read) |"
echo "| **Grand Total** | **$(printf "%'d" $grand_total)** |"
echo "| Overall Cache Hit Rate | ${overall_cache_rate}% |"
echo ""

# Calculate estimated costs (2026 pricing)
# Sonnet 4.5: $3/1M input, $15/1M output, $0.30/1M cache read
# Opus 4.5: $15/1M input, $75/1M output, $1.50/1M cache read
# Haiku 4.5: $1/1M input, $5/1M output, $0.10/1M cache read
# Simplified: assume mix of models, use average
cost_input=$(awk "BEGIN {printf \"%.2f\", ($total_input + $total_cache_creation) * 3 / 1000000}")
cost_output=$(awk "BEGIN {printf \"%.2f\", $total_output * 15 / 1000000}")
cost_cache=$(awk "BEGIN {printf \"%.2f\", $total_cache_read * 0.30 / 1000000}")
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

# Save to file
if [ ! -d "docs/builder/sprints" ]; then
    mkdir -p docs/builder/sprints
fi

# Run again but redirect to file
{
    echo "# Sprint $SPRINT_NUM - Token Usage Metrics"
    echo ""
    echo "**Session ID:** $SESSION_ID"
    echo "**Generated:** $(date)"
    echo ""
    echo "---"
    echo ""

    # [Rest of output redirected to file]
    bash "$0" "$SESSION_ID" "$SPRINT_NUM" 2>/dev/null || true
} > "$OUTPUT_FILE"

echo "---"
echo ""
echo "**Metrics saved to:** $OUTPUT_FILE"
