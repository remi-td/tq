# GitHub Integration Setup

This document provides instructions for setting up GitHub Issues integration for the tq project.

## Prerequisites

1. **GitHub CLI installed:**
   ```bash
   # macOS
   brew install gh

   # Linux
   # See: https://github.com/cli/cli#installation
   ```

2. **Authenticate with GitHub:**
   ```bash
   gh auth login
   ```

3. **Set your repository URL:**
   ```bash
   # Replace with your actual repository
   gh repo set-default <owner>/tq
   ```

## Initial Setup

### Step 1: Create Issue Labels

Run these commands to create the standard labels used by the sprint workflow:

```bash
# Workflow labels
gh label create "sprint-ready" \
  --color "0E8A16" \
  --description "Triaged and ready for sprint inclusion"

gh label create "needs-info" \
  --color "D93F0B" \
  --description "Needs more information from issue author"

gh label create "wont-fix" \
  --color "FFFFFF" \
  --description "Out of scope or will not be implemented"

gh label create "duplicate" \
  --color "CFD3D7" \
  --description "Duplicate of another issue"

# Type labels (may already exist)
gh label create "bug" \
  --color "D73A4A" \
  --description "Something isn't working correctly" || true

gh label create "enhancement" \
  --color "A2EEEF" \
  --description "New feature or improvement request" || true

gh label create "documentation" \
  --color "0075CA" \
  --description "Documentation updates or improvements" || true

# Priority labels
gh label create "priority-high" \
  --color "B60205" \
  --description "High priority, blocking or critical"

gh label create "priority-medium" \
  --color "FBCA04" \
  --description "Medium priority, important but not blocking"

gh label create "priority-low" \
  --color "C2E0C6" \
  --description "Low priority, nice to have"
```

**Note:** Some labels like `bug` and `enhancement` may already exist in your repository. The `|| true` allows the script to continue even if the label already exists.

### Step 2: Verify Labels

```bash
gh label list
```

You should see all the labels listed above.

### Step 3: Test Issue Creation

Create a test issue to verify templates are working:

```bash
gh issue create --web
```

This opens the GitHub web UI where you should see the issue template chooser with:
- Bug Report
- Feature Request

### Step 4: Test the `/github-issues` Skill

Create a test issue and triage it:

```bash
# In Claude Code:
/github-issues
```

The skill should:
1. Fetch untriaged issues
2. Analyze them against project scope
3. Apply appropriate labels
4. Add triage comments

## Issue Template Files

The repository includes the following issue templates:

- **bug_report.md**: Template for bug reports
  - Prompts for: Environment, reproduction steps, expected vs actual behavior
  - Includes checkboxes for OS, execution mode, configuration
  - Designed for CLI tool debugging

- **feature_request.md**: Template for feature requests
  - Prompts for: Use case, proposed solution, alternatives
  - Includes feature category selection
  - Includes priority assessment
  - Designed for structured feature planning

- **config.yml**: Template chooser configuration
  - Disables blank issues (forces template usage)
  - Provides links to documentation and discussions

## Workflow Integration

### For Sprint Coordinators

**Phase 1 (Planning):**
```bash
# Fetch sprint-ready issues
gh issue list --label "sprint-ready" --state open

# Or use the skill
/github-issues
```

**Phase 4 (Ship):**
```bash
# Close completed issue
gh issue close <number> --comment "Implementation complete. See Sprint N review."

# Or use the skill
/github-issues
```

### For Issue Triage

Run triage before each sprint:

```bash
/github-issues
```

The skill will:
1. Find all untriaged issues
2. Analyze against project specifications
3. Make accept/reject/needs-info decisions
4. Apply labels and add comments
5. Provide summary report

## Maintenance

### Review Labels Periodically

```bash
# List all labels with usage count
gh label list
```

### Archive Old Issues

```bash
# Close stale issues that haven't been updated in 90 days
gh issue list --state open --json number,updatedAt \
  --jq '.[] | select(.updatedAt < (now - 7776000)) | .number' \
  | xargs -I {} gh issue close {} --comment "Closing due to inactivity. Please reopen if still relevant."
```

## Troubleshooting

### `gh` command not found
```bash
# Install GitHub CLI
brew install gh  # macOS
# or follow instructions at: https://github.com/cli/cli#installation
```

### Authentication failed
```bash
# Re-authenticate
gh auth logout
gh auth login
```

### Can't create labels
```bash
# Check repository permissions
gh repo view --json viewerPermission
# You need "admin" or "write" permission
```

### Issue templates not appearing
1. Ensure files are in `.github/ISSUE_TEMPLATE/` (note the dot)
2. Ensure files have `.md` extension
3. Ensure YAML frontmatter is valid (test with a YAML validator)
4. Push changes to GitHub - templates won't appear until committed

## Additional Resources

- [GitHub CLI Manual](https://cli.github.com/manual/)
- [GitHub Issue Template Guide](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/configuring-issue-templates-for-your-repository)
- [Claude Code GitHub Integration](https://code.claude.com/docs/en/github-actions)
