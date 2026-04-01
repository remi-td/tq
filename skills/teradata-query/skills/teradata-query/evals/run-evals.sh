#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# tq Skill Eval Runner
#
# Runs evaluation test cases against an agent with and without the
# teradata-query skill loaded, grades assertions via an LLM judge,
# and writes structured results.
#
# Usage:
#   ./run-evals.sh                      Run all evals
#   ./run-evals.sh --eval 3             Run eval #3 only
#   ./run-evals.sh --skill-only         Skip baseline (faster iteration)
#   ./run-evals.sh --baseline-only      Skip with-skill run
#   ./run-evals.sh --model sonnet       Use a specific model (default: sonnet)
#   ./run-evals.sh --judge-model opus   Use a specific model for grading
#   ./run-evals.sh --report             Regenerate report from existing results
#
# Requires: claude CLI, jq
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

# ── Resolve paths ────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SKILL_FILE="$SKILL_DIR/SKILL.md"
EVALS_FILE="$SCRIPT_DIR/evals.json"

# ── Defaults ─────────────────────────────────────────────────────────────────
MODEL="sonnet"
JUDGE_MODEL="sonnet"
RUN_BASELINE=true
RUN_SKILL=true
EVAL_FILTER=""
REPORT_ONLY=false
RESULTS_DIR=""

# ── Parse arguments ──────────────────────────────────────────────────────────
show_help() {
  echo "Usage: $0 [OPTIONS]"
  echo ""
  echo "Options:"
  echo "  --eval ID          Run specific eval (comma-separated IDs)"
  echo "  --model MODEL      Model for eval runs (default: sonnet)"
  echo "  --judge-model M    Model for grading (default: sonnet)"
  echo "  --skill-only       Skip baseline runs"
  echo "  --baseline-only    Skip with-skill runs"
  echo "  --report           Regenerate report from last results"
  echo "  --results-dir DIR  Use specific results directory"
  echo "  --help, -h         Show this help"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --eval)         EVAL_FILTER="$2"; shift 2 ;;
    --model)        MODEL="$2"; shift 2 ;;
    --judge-model)  JUDGE_MODEL="$2"; shift 2 ;;
    --skill-only)   RUN_BASELINE=false; shift ;;
    --baseline-only) RUN_SKILL=false; shift ;;
    --report)       REPORT_ONLY=true; shift ;;
    --results-dir)  RESULTS_DIR="$2"; shift 2 ;;
    --help|-h)      show_help; exit 0 ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

# ── Set up results directory ─────────────────────────────────────────────────
if [[ -z "$RESULTS_DIR" ]]; then
  TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
  RESULTS_DIR="$SCRIPT_DIR/results/$TIMESTAMP"
fi
mkdir -p "$RESULTS_DIR"

# ── Preflight checks ────────────────────────────────────────────────────────
check_deps() {
  local missing=()
  command -v claude >/dev/null 2>&1 || missing+=("claude")
  command -v jq     >/dev/null 2>&1 || missing+=("jq")
  if [[ ${#missing[@]} -gt 0 ]]; then
    echo "ERROR: Missing required tools: ${missing[*]}" >&2
    exit 1
  fi
  if [[ ! -f "$EVALS_FILE" ]]; then
    echo "ERROR: Evals file not found: $EVALS_FILE" >&2
    exit 1
  fi
  if [[ ! -f "$SKILL_FILE" ]]; then
    echo "ERROR: Skill file not found: $SKILL_FILE" >&2
    exit 1
  fi
}

# ── Logging helpers ──────────────────────────────────────────────────────────
info()  { echo "[INFO] $1" >&2; }
ok()    { echo "[ OK ] $1" >&2; }
fail()  { echo "[FAIL] $1" >&2; }
skip()  { echo "[SKIP] $1" >&2; }

# ── Build the prompt for an eval case ────────────────────────────────────────
build_prompt() {
  local eval_json="$1"
  local prompt context
  prompt="$(echo "$eval_json" | jq -r '.prompt')"
  context="$(echo "$eval_json" | jq -r '.context')"

  printf '%s\n' \
    "You are an AI coding agent helping a user with Teradata database tasks." \
    "The user has the following environment context: $context" \
    "" \
    "Respond to the user request. Describe exactly what commands you would run" \
    "and in what order. Show the full command lines you would execute. Do NOT" \
    "actually execute anything -- just describe your plan step by step." \
    "" \
    "User request: $prompt"
}

# ── Run a single eval through claude ─────────────────────────────────────────
run_claude() {
  local prompt="$1"
  local mode="$2"
  local output_file="$3"

  local -a claude_args=(
    -p "$prompt"
    --output-format json
    --model "$MODEL"
    --max-turns 1
  )

  if [[ "$mode" == "with_skill" ]]; then
    claude_args+=(--append-system-prompt-file "$SKILL_FILE")
  fi

  local raw_output
  if raw_output="$(claude "${claude_args[@]}" 2>"${output_file}.stderr")"; then
    echo "$raw_output" > "$output_file"
  else
    echo "$raw_output" > "$output_file"
    echo "claude exited non-zero, see .stderr file" > "${output_file}.error"
  fi
}

# ── Grade assertions using LLM judge ────────────────────────────────────────
grade_assertions() {
  local eval_json="$1"
  local response_file="$2"
  local grading_file="$3"
  local mode="$4"

  local response
  if [[ -f "$response_file" ]]; then
    response="$(jq -r '.result // .content // "NO OUTPUT"' "$response_file" 2>/dev/null || cat "$response_file")"
  else
    response="NO OUTPUT - file not found"
  fi

  local assertions
  assertions="$(echo "$eval_json" | jq -r '.assertions[]' | nl -ba)"

  local prompt context
  prompt="$(echo "$eval_json" | jq -r '.prompt')"
  context="$(echo "$eval_json" | jq -r '.context')"

  # Build grading prompt using a temp file to avoid quoting issues
  local grading_prompt_file
  grading_prompt_file="$(mktemp)"
  trap "rm -f $grading_prompt_file" RETURN 2>/dev/null || true

  cat > "$grading_prompt_file" <<GRADING_EOF
You are an eval grader for an AI agent skill. You must judge whether each
assertion holds true for the agent response.

## Original Task
User prompt: $prompt
Environment context: $context

## Agent Response ($mode)
$response

## Assertions to Grade
$assertions

## Instructions
For EACH assertion, determine if it PASSES or FAILS based on the agent
response. An assertion passes if the agent response demonstrates the
behavior described. Be strict but fair -- the agent describes commands it
would run, so check if those commands match the assertion.

You MUST respond with ONLY a JSON object (no markdown fences, no extra text).
Use this exact format:
{"grades":[{"assertion_index":1,"assertion_text":"...","pass":true,"evidence":"..."},{"assertion_index":2,...}]}
GRADING_EOF

  local grading_prompt
  grading_prompt="$(cat "$grading_prompt_file")"
  rm -f "$grading_prompt_file"

  local grading_output
  if grading_output="$(claude -p "$grading_prompt" \
      --output-format json \
      --model "$JUDGE_MODEL" \
      --max-turns 1 2>/dev/null)"; then
    echo "$grading_output" > "$grading_file"
  else
    echo "$grading_output" > "$grading_file"
  fi
}

# ── Run one eval case ────────────────────────────────────────────────────────
run_eval() {
  local eval_json="$1"
  local eval_id eval_category
  eval_id="$(echo "$eval_json" | jq -r '.id')"
  eval_category="$(echo "$eval_json" | jq -r '.category')"

  local case_dir="$RESULTS_DIR/eval-$eval_id"
  mkdir -p "$case_dir"

  # Save the eval case for reference
  echo "$eval_json" | jq '.' > "$case_dir/eval-case.json"

  local prompt
  prompt="$(build_prompt "$eval_json")"

  # Baseline run (no skill)
  if [[ "$RUN_BASELINE" == true ]]; then
    info "Eval #$eval_id ($eval_category) - baseline..."
    run_claude "$prompt" "baseline" "$case_dir/baseline-response.json"

    info "Eval #$eval_id ($eval_category) - grading baseline..."
    grade_assertions "$eval_json" "$case_dir/baseline-response.json" \
                     "$case_dir/baseline-grading.json" "baseline"
    ok "Eval #$eval_id baseline graded"
  else
    skip "Eval #$eval_id baseline (--skill-only)"
  fi

  # With-skill run
  if [[ "$RUN_SKILL" == true ]]; then
    info "Eval #$eval_id ($eval_category) - with skill..."
    run_claude "$prompt" "with_skill" "$case_dir/skill-response.json"

    info "Eval #$eval_id ($eval_category) - grading with skill..."
    grade_assertions "$eval_json" "$case_dir/skill-response.json" \
                     "$case_dir/skill-grading.json" "with_skill"
    ok "Eval #$eval_id with-skill graded"
  else
    skip "Eval #$eval_id with-skill (--baseline-only)"
  fi
}

# ── Collect results into summary JSON ────────────────────────────────────────
collect_summary() {
  local results_dir="$1"
  local summary_json="[]"

  for case_dir in "$results_dir"/eval-*/; do
    [[ -d "$case_dir" ]] || continue

    local eval_id eval_category num_assertions
    eval_id="$(jq -r '.id' "$case_dir/eval-case.json")"
    eval_category="$(jq -r '.category' "$case_dir/eval-case.json")"
    num_assertions="$(jq '.assertions | length' "$case_dir/eval-case.json")"

    local baseline_pass="-" baseline_total="-" skill_pass="-" skill_total="-"

    if [[ -f "$case_dir/baseline-grading.json" ]]; then
      baseline_pass="$(jq '[(.result | gsub("```json\\n?";"") | gsub("```";"") | gsub("^\\s+";"") | fromjson? // {grades:[]}).grades[]? | select(.pass == true)] | length' "$case_dir/baseline-grading.json" 2>/dev/null || echo "0")"
      baseline_total="$(jq '[(.result | gsub("```json\\n?";"") | gsub("```";"") | gsub("^\\s+";"") | fromjson? // {grades:[]}).grades[]?] | length' "$case_dir/baseline-grading.json" 2>/dev/null || echo "0")"
    fi

    if [[ -f "$case_dir/skill-grading.json" ]]; then
      skill_pass="$(jq '[(.result | gsub("```json\\n?";"") | gsub("```";"") | gsub("^\\s+";"") | fromjson? // {grades:[]}).grades[]? | select(.pass == true)] | length' "$case_dir/skill-grading.json" 2>/dev/null || echo "0")"
      skill_total="$(jq '[(.result | gsub("```json\\n?";"") | gsub("```";"") | gsub("^\\s+";"") | fromjson? // {grades:[]}).grades[]?] | length' "$case_dir/skill-grading.json" 2>/dev/null || echo "0")"
    fi

    summary_json="$(echo "$summary_json" | jq \
      --argjson id "$eval_id" \
      --arg cat "$eval_category" \
      --argjson nassert "$num_assertions" \
      --arg bp "$baseline_pass" \
      --arg bt "$baseline_total" \
      --arg sp "$skill_pass" \
      --arg st "$skill_total" \
      '. + [{"id": $id, "category": $cat, "num_assertions": $nassert, "baseline_pass": $bp, "baseline_total": $bt, "skill_pass": $sp, "skill_total": $st}]'
    )"
  done

  echo "$summary_json"
}

# ── Compute aggregate rates ─────────────────────────────────────────────────
compute_aggregate() {
  local summary_json="$1"
  echo "$summary_json" | jq '
    [.[] | select(.baseline_total != "-" and .baseline_total != "0")] as $b |
    [.[] | select(.skill_total != "-" and .skill_total != "0")] as $s |
    {
      baseline_pass: ([($b[].baseline_pass | tonumber)] | add // 0),
      baseline_total: ([($b[].baseline_total | tonumber)] | add // 0),
      skill_pass: ([($s[].skill_pass | tonumber)] | add // 0),
      skill_total: ([($s[].skill_total | tonumber)] | add // 0)
    } |
    . + {
      baseline_rate: (if .baseline_total > 0 then (100 * .baseline_pass / .baseline_total | round) else 0 end),
      skill_rate: (if .skill_total > 0 then (100 * .skill_pass / .skill_total | round) else 0 end)
    } |
    . + { delta: (.skill_rate - .baseline_rate) }
  '
}

# ── Write the markdown report file ──────────────────────────────────────────
write_report() {
  local results_dir="$1"
  local summary_json="$2"
  local agg_json="$3"
  local report_file="$results_dir/report.md"

  local b_rate s_rate delta_val
  b_rate="$(echo "$agg_json" | jq -r '.baseline_rate')"
  s_rate="$(echo "$agg_json" | jq -r '.skill_rate')"
  delta_val="$(echo "$agg_json" | jq -r '.delta')"

  {
    echo "# Skill Eval Report: teradata-query"
    echo ""
    echo "**Date:** $(date '+%Y-%m-%d %H:%M:%S')"
    echo "**Model:** $MODEL | **Judge:** $JUDGE_MODEL"
    echo "**Results:** $results_dir"
    echo ""
    echo "## Results"
    echo ""
    echo "| # | Category | Assertions | Baseline | With Skill | Delta |"
    echo "|---|----------|-----------|----------|------------|-------|"

    echo "$summary_json" | jq -r '.[] | @json' | while IFS= read -r row_json; do
      local eid ecat nassert bp bt sp st
      eid="$(echo "$row_json" | jq -r '.id')"
      ecat="$(echo "$row_json" | jq -r '.category')"
      nassert="$(echo "$row_json" | jq -r '.num_assertions')"
      bp="$(echo "$row_json" | jq -r '.baseline_pass')"
      bt="$(echo "$row_json" | jq -r '.baseline_total')"
      sp="$(echo "$row_json" | jq -r '.skill_pass')"
      st="$(echo "$row_json" | jq -r '.skill_total')"

      local baseline_str="--" skill_str="--" delta_str="--"
      if [[ "$bt" != "-" && "$bt" != "0" ]]; then
        baseline_str="$bp/$bt"
      fi
      if [[ "$st" != "-" && "$st" != "0" ]]; then
        skill_str="$sp/$st"
      fi
      if [[ "$bt" != "-" && "$bt" != "0" && "$st" != "-" && "$st" != "0" ]]; then
        local d=$(( sp - bp ))
        if [[ $d -gt 0 ]]; then delta_str="+$d"
        elif [[ $d -eq 0 ]]; then delta_str="0"
        else delta_str="$d"
        fi
      fi

      echo "| $eid | $ecat | $nassert | $baseline_str | $skill_str | $delta_str |"
    done

    echo ""
    echo "## Aggregate"
    echo ""
    echo "| Metric | Value |"
    echo "|--------|-------|"
    echo "| Baseline pass rate | ${b_rate}% |"
    echo "| With-skill pass rate | ${s_rate}% |"
    echo "| **Skill delta** | **+${delta_val}pp** |"
    echo ""
    echo "## Per-Assertion Detail"
    echo ""

    for case_dir in "$results_dir"/eval-*/; do
      [[ -d "$case_dir" ]] || continue
      local eval_id eval_category
      eval_id="$(jq -r '.id' "$case_dir/eval-case.json")"
      eval_category="$(jq -r '.category' "$case_dir/eval-case.json")"

      echo "### Eval #$eval_id: $eval_category"
      echo ""

      if [[ -f "$case_dir/skill-grading.json" ]]; then
        echo "| # | Assertion | Pass | Evidence |"
        echo "|---|-----------|------|----------|"

        jq -r '
          (.result | gsub("```json\\n?";"") | gsub("```";"") | gsub("^\\s+";"") | fromjson? // {grades:[]}).grades[]? |
          "| \(.assertion_index) | \(.assertion_text) | \(if .pass then "PASS" else "FAIL" end) | \(.evidence) |"
        ' "$case_dir/skill-grading.json" 2>/dev/null || echo "| - | Grading unavailable | - | - |"

        echo ""
      fi
    done
  } > "$report_file"

  ok "Report written to $report_file"
}

# ── Print terminal summary ──────────────────────────────────────────────────
print_summary() {
  local summary_json="$1"
  local agg_json="$2"
  local results_dir="$3"

  local b_rate s_rate delta_val
  b_rate="$(echo "$agg_json" | jq -r '.baseline_rate')"
  s_rate="$(echo "$agg_json" | jq -r '.skill_rate')"
  delta_val="$(echo "$agg_json" | jq -r '.delta')"

  echo ""
  echo "================================================================"
  echo "  EVAL RESULTS: teradata-query skill"
  echo "  Model: $MODEL | Judge: $JUDGE_MODEL"
  echo "================================================================"
  echo ""

  # Print as TSV, then format with column
  {
    printf '%s\t%s\t%s\t%s\n' "#" "Category" "Baseline" "Skill"
    printf '%s\t%s\t%s\t%s\n' "-" "--------" "--------" "-----"
    echo "$summary_json" | jq -r '.[] | [
      (.id | tostring),
      .category,
      (if .baseline_total != "-" and .baseline_total != "0"
       then "\(.baseline_pass)/\(.baseline_total)" else "--" end),
      (if .skill_total != "-" and .skill_total != "0"
       then "\(.skill_pass)/\(.skill_total)" else "--" end)
    ] | join("\t")'
  } | column -t -s "	"

  echo ""
  echo "  Baseline: ${b_rate}%  |  With skill: ${s_rate}%  |  Delta: +${delta_val}pp"
  echo ""
  echo "  Full report: $results_dir/report.md"
  echo "================================================================"
}

# ── Generate summary report ──────────────────────────────────────────────────
generate_report() {
  local results_dir="$1"

  info "Generating report..."

  local summary_json
  summary_json="$(collect_summary "$results_dir")"
  echo "$summary_json" | jq '.' > "$results_dir/summary.json"

  local agg_json
  agg_json="$(compute_aggregate "$summary_json")"

  write_report "$results_dir" "$summary_json" "$agg_json"
  print_summary "$summary_json" "$agg_json" "$results_dir"
}

# ── Main ─────────────────────────────────────────────────────────────────────
main() {
  check_deps

  if [[ "$REPORT_ONLY" == true ]]; then
    local latest
    latest="$(ls -dt "$SCRIPT_DIR"/results/*/ 2>/dev/null | head -1)"
    if [[ -z "$latest" ]]; then
      echo "ERROR: No results found in $SCRIPT_DIR/results/" >&2
      exit 1
    fi
    RESULTS_DIR="${latest%/}"
    generate_report "$RESULTS_DIR"
    exit 0
  fi

  info "Starting eval run"
  info "  Skill:   $SKILL_FILE"
  info "  Evals:   $EVALS_FILE"
  info "  Model:   $MODEL"
  info "  Judge:   $JUDGE_MODEL"
  info "  Results: $RESULTS_DIR"
  echo ""

  local num_evals
  num_evals="$(jq '.evals | length' "$EVALS_FILE")"
  info "Found $num_evals eval cases"

  local eval_indices
  if [[ -n "$EVAL_FILTER" ]]; then
    eval_indices="$EVAL_FILTER"
    info "Filtering to eval(s): $eval_indices"
  else
    eval_indices="$(seq 1 "$num_evals" | tr '\n' ',')"
    eval_indices="${eval_indices%,}"
  fi

  IFS=',' read -ra ids <<< "$eval_indices"
  for id in "${ids[@]}"; do
    local eval_json
    eval_json="$(jq ".evals[] | select(.id == $id)" "$EVALS_FILE")"
    if [[ -z "$eval_json" ]]; then
      fail "Eval #$id not found in $EVALS_FILE"
      continue
    fi
    run_eval "$eval_json"
    echo ""
  done

  generate_report "$RESULTS_DIR"
}

main "$@"
