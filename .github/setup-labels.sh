#!/bin/bash
# Setup GitHub issue labels for tq project
# Usage: ./setup-labels.sh

set -e

echo "🏷️  Setting up GitHub issue labels for tq project..."
echo ""

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo "❌ Error: GitHub CLI (gh) is not installed."
    echo "📦 Install it with: brew install gh"
    echo "📖 Or visit: https://github.com/cli/cli#installation"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "❌ Error: Not authenticated with GitHub."
    echo "🔐 Run: gh auth login"
    exit 1
fi

echo "✅ GitHub CLI is installed and authenticated"
echo ""

# Function to create or update label
create_label() {
    local name=$1
    local color=$2
    local description=$3

    if gh label create "$name" --color "$color" --description "$description" 2>/dev/null; then
        echo "  ✅ Created: $name"
    else
        # Label might already exist, try to update it
        if gh label edit "$name" --color "$color" --description "$description" 2>/dev/null; then
            echo "  ♻️  Updated: $name"
        else
            echo "  ⚠️  Skipped: $name (already exists)"
        fi
    fi
}

echo "📋 Creating workflow labels..."
create_label "sprint-ready" "0E8A16" "Triaged and ready for sprint inclusion"
create_label "needs-info" "D93F0B" "Needs more information from issue author"
create_label "wont-fix" "FFFFFF" "Out of scope or will not be implemented"
create_label "duplicate" "CFD3D7" "Duplicate of another issue"
echo ""

echo "🏷️  Creating type labels..."
create_label "bug" "D73A4A" "Something isn't working correctly"
create_label "enhancement" "A2EEEF" "New feature or improvement request"
create_label "documentation" "0075CA" "Documentation updates or improvements"
echo ""

echo "🎯 Creating priority labels..."
create_label "priority-high" "B60205" "High priority, blocking or critical"
create_label "priority-medium" "FBCA04" "Medium priority, important but not blocking"
create_label "priority-low" "C2E0C6" "Low priority, nice to have"
echo ""

echo "✨ Label setup complete!"
echo ""
echo "📊 Current labels:"
gh label list
echo ""
echo "🎉 All done! You can now use GitHub Issues with the sprint workflow."
echo "📖 Next steps: See .github/SETUP.md for usage instructions"
