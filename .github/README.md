# GitHub Configuration

This directory contains GitHub-specific configuration for the tq project.

## Contents

### Issue Templates (`ISSUE_TEMPLATE/`)

Professional issue templates that guide users to provide complete, actionable information:

- **`bug_report.md`**: Comprehensive bug report template
  - Environment details (OS, tq version, Teradata version)
  - Execution mode (one-shot, REPL, batch)
  - Steps to reproduce
  - Expected vs actual behavior
  - Configuration context
  - Security reminders (credential removal)

- **`feature_request.md`**: Structured feature request template
  - Problem/use case description
  - Proposed solution with examples
  - Alternative approaches considered
  - Feature category classification
  - Priority assessment
  - Compatibility considerations

- **`config.yml`**: Template chooser configuration
  - Disables blank issues (enforces template usage)
  - Links to documentation and discussions

### Setup Scripts

- **`setup-labels.sh`**: Automated label creation script
  - Creates all workflow, type, and priority labels
  - Handles existing labels gracefully
  - Provides clear progress feedback
  - Usage: `./setup-labels.sh`

### Documentation

- **`SETUP.md`**: Complete setup and usage guide
  - GitHub CLI installation and authentication
  - Label creation instructions
  - Issue template verification
  - Workflow integration for sprint coordinators
  - Troubleshooting section

## Quick Start

### For First-Time Setup

1. **Install and authenticate GitHub CLI:**
   ```bash
   brew install gh
   gh auth login
   ```

2. **Create issue labels:**
   ```bash
   cd .github
   ./setup-labels.sh
   ```

3. **Test issue creation:**
   ```bash
   gh issue create --web
   ```

### For Sprint Coordinators

**Triage new issues:**
```bash
/github-issues
```

**Fetch sprint-ready issues (Phase 1):**
```bash
gh issue list --label "sprint-ready" --state open
```

**Close completed issues (Phase 4):**
```bash
gh issue close <number> --comment "Implementation details..."
```

## Issue Labels

### Workflow Labels
- `sprint-ready` 🟢 - Triaged and ready for sprint inclusion
- `needs-info` 🔴 - Requires clarification from issue author
- `wont-fix` ⚪ - Out of scope or will not be implemented
- `duplicate` ⚪ - Duplicate of another issue

### Type Labels
- `bug` 🔴 - Something isn't working correctly
- `enhancement` 🔵 - New feature or improvement request
- `documentation` 🔵 - Documentation updates

### Priority Labels
- `priority-high` 🔴 - High priority, blocking or critical
- `priority-medium` 🟡 - Medium priority, important but not blocking
- `priority-low` 🟢 - Low priority, nice to have

## Integration with Sprint Workflow

The GitHub Issues system integrates seamlessly with the sprint-driven development workflow:

```
User submits issue
    ↓
Triage (/github-issues skill)
    ↓ (sprint-ready label)
Sprint Planning (Phase 1)
    ↓ (comment: "Included in Sprint N")
Implementation (Phases 2-3)
    ↓
Ship (Phase 4)
    ↓ (close with implementation details)
Issue closed ✅
```

## Best Practices

### For Issue Authors
- Use the provided templates (don't skip sections)
- Remove all credentials and sensitive data
- Provide minimal reproducible examples
- Search for duplicates before submitting

### For Triage
- Triage new issues promptly (within 1-2 sprints)
- Provide clear rationale in triage comments
- Reference specifications/design docs in decisions
- Be kind when rejecting requests

### For Sprint Coordinators
- Review sprint-ready issues at start of each sprint
- Comment on issues when including in sprint
- Update issues with progress during long sprints
- Close issues with complete implementation details

## Troubleshooting

**Issue templates not appearing?**
- Templates must be committed and pushed to GitHub
- Check YAML frontmatter syntax
- Ensure files are in `.github/ISSUE_TEMPLATE/` with `.md` extension

**Can't create labels?**
- Verify you have write access: `gh repo view --json viewerPermission`
- Re-authenticate: `gh auth logout && gh auth login`

**`gh` commands failing?**
- Check installation: `gh --version`
- Verify authentication: `gh auth status`
- Set default repo: `gh repo set-default <owner>/tq`

## Additional Resources

- [Setup Guide](SETUP.md) - Detailed setup and usage instructions
- [GitHub Issues Skill](../.claude/skills/github-issues/SKILL.md) - Claude skill for issue management
- [Sprint Coordinator](../.claude/skills/sprint-coordinator/SKILL.md) - Sprint workflow integration
- [GitHub CLI Manual](https://cli.github.com/manual/)
- [GitHub Issue Templates](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests)
