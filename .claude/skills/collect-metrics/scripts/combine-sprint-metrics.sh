#!/bin/bash
# Combine metrics from multiple sessions into a single sprint metrics file
# Usage: ./combine-sprint-metrics.sh <sprint-number> <session-id-1> <session-id-2> [session-id-3 ...]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"

if [ $# -lt 2 ]; then
    echo "Usage: $0 <sprint-number> <session-id-1> <session-id-2> [session-id-3 ...]"
    echo ""
    echo "Combines metrics from multiple sessions for a single sprint."
    echo ""
    echo "To find recent sessions, use:"
    echo "  ./list-recent-sessions.sh [days]"
    exit 1
fi

SPRINT_NUM=$1
shift
SESSION_IDS=("$@")

echo "Collecting metrics for Sprint $SPRINT_NUM from ${#SESSION_IDS[@]} sessions..." >&2

# Extract metrics for each session to temporary files
TEMP_FILES=()
SESSION_INFO=()
for i in "${!SESSION_IDS[@]}"; do
    session_id="${SESSION_IDS[$i]}"
    temp_file=$(mktemp)
    TEMP_FILES+=("$temp_file")

    echo "  Processing session $((i+1))/${#SESSION_IDS[@]}: $session_id..." >&2

    # Extract metrics to temp file
    "$SCRIPT_DIR/extract-sprint-metrics.sh" "$session_id" "temp-$i" > "$temp_file" 2>&1

    # Extract summary metrics from temp file
    session_total=$(grep "| \*\*Grand Total\*\*" "$temp_file" | head -1 | sed 's/[^0-9,]//g' | tr -d ',')
    agent_count=$(grep -c "^### Agent:" "$temp_file" || echo "0")

    SESSION_INFO+=("$session_id|$session_total|$agent_count")
done

# Parse all metrics and sum them
total_input=0
total_output=0
total_cache_creation=0
total_cache_read=0

for temp_file in "${TEMP_FILES[@]}"; do
    # Extract individual totals from each session
    input=$(grep "^| Total Input Tokens" "$temp_file" | head -1 | sed 's/[^0-9,]//g' | tr -d ',')
    output=$(grep "^| Total Output Tokens" "$temp_file" | head -1 | sed 's/[^0-9,]//g' | tr -d ',')
    cache_create=$(grep "^| Total Cache Creation" "$temp_file" | head -1 | sed 's/[^0-9,]//g' | tr -d ',')
    cache_read=$(grep "^| Total Cache Reads" "$temp_file" | head -1 | sed 's/[^0-9,]//g' | tr -d ',')

    total_input=$((total_input + input))
    total_output=$((total_output + output))
    total_cache_creation=$((total_cache_creation + cache_create))
    total_cache_read=$((total_cache_read + cache_read))
done

# Calculate combined totals
grand_total=$((total_input + total_output + total_cache_creation + total_cache_read))
total_input_processing=$((total_input + total_cache_creation + total_cache_read))
overall_cache_rate=0
if [ $total_input_processing -gt 0 ]; then
    overall_cache_rate=$(awk "BEGIN {printf \"%.1f\", ($total_cache_read / $total_input_processing) * 100}")
fi

# Calculate costs
cost_input=$(awk "BEGIN {printf \"%.2f\", ($total_input + $total_cache_creation) * 3 / 1000000}")
cost_output=$(awk "BEGIN {printf \"%.2f\", $total_output * 15 / 1000000}")
cost_cache=$(awk "BEGIN {printf \"%.2f\", $total_cache_read * 0.30 / 1000000}")
cost_total=$(awk "BEGIN {printf \"%.2f\", $cost_input + $cost_output + $cost_cache}")

# Generate combined output
OUTPUT_FILE="$PROJECT_ROOT/docs/sprints/sprint-${SPRINT_NUM}-metrics.md"

{
echo "# Sprint $SPRINT_NUM - Token Usage Metrics (Combined)"
echo ""
echo "**Session IDs:**"
for session_id in "${SESSION_IDS[@]}"; do
    echo "- $session_id"
done
echo ""
echo "**Generated:** $(date)"
echo ""
echo "---"
echo ""
echo "## Combined Sprint Summary"
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
echo "---"
echo ""
echo "## Session Breakdown"
echo ""

# Output session breakdown
for i in "${!SESSION_IDS[@]}"; do
    session_id="${SESSION_IDS[$i]}"
    IFS='|' read -r sid stotal sagents <<< "${SESSION_INFO[$i]}"

    # Extract cost from temp file - search for the cost line
    session_cost=$(grep "| \*\*Total\*\*" "${TEMP_FILES[$i]}" | head -1 | sed 's/.*\$\([0-9.]*\).*/\1/')

    echo "### Session $((i+1)): ${session_id:0:8}"
    echo "- Agents: $sagents"
    echo "- Total Tokens: $(printf "%'d" $stotal)"
    echo "- Cost: \$$session_cost"
    echo ""
done

echo "---"
echo ""
echo "See individual session metrics in the temp files or re-run:"
for i in "${!SESSION_IDS[@]}"; do
    echo "\`./extract-sprint-metrics.sh ${SESSION_IDS[$i]} $SPRINT_NUM-session-$((i+1))\`"
done
echo ""
} | tee "$OUTPUT_FILE"

# Cleanup temp files
for temp_file in "${TEMP_FILES[@]}"; do
    rm -f "$temp_file"
done

echo "" >&2
echo "✅ Combined metrics saved to: $OUTPUT_FILE" >&2
