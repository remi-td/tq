#!/bin/bash
# Validates the agentic framework structure.
# Usage: bash .claude/scripts/validate-framework.sh

set -e

SKILL_DIR=".claude/skills/sprint-coordinator"
TEMPLATE_DIR=".claude/templates"
AGENT_DIR=".claude/agents"

echo "=== Agentic Framework Validation ==="

# 1. Check required process documents exist
echo "Checking process documents..."
REQUIRED_PROCESS_DOCS=(
    "sprint-workflow.md"
    "phase0-reality-check.md"
    "phase1-feature-planning.md"
    "phase2-design.md"
    "phase3-build-test.md"
    "phase4-ship.md"
    "definitions/done.md"
)
for doc in "${REQUIRED_PROCESS_DOCS[@]}"; do
    if [ ! -f "$SKILL_DIR/process/$doc" ]; then
        echo "  ERROR: Missing process doc: $SKILL_DIR/process/$doc"
        exit 1
    fi
done
echo "  OK: All process documents present."

# 2. Check required templates exist
echo "Checking templates..."
REQUIRED_TEMPLATES=(
    "quality-report-template.md"
    "test-case-template.md"
)
for tpl in "${REQUIRED_TEMPLATES[@]}"; do
    if [ ! -f "$TEMPLATE_DIR/$tpl" ]; then
        echo "  ERROR: Missing template: $TEMPLATE_DIR/$tpl"
        exit 1
    fi
done
echo "  OK: All templates present."

# 3. Check agent definitions have versioning
echo "Checking agent versioning..."
for agent_file in $AGENT_DIR/*.md; do
    if ! grep -q "^version:" "$agent_file"; then
        echo "  WARNING: No version found in $agent_file"
    fi
done
echo "  OK: Agent versioning check complete."

# 4. Check for broken internal links in process docs (basic)
echo "Checking for broken links in process docs..."
# This is a simple check; a full implementation would parse markdown links.
if grep -rn "\](process/" $SKILL_DIR/process/ | grep -v ".md)" > /dev/null 2>&1; then
    echo "  WARNING: Potential broken link in process docs."
fi
echo "  OK: Link check complete (basic)."

echo ""
echo "=== Validation Complete ==="
