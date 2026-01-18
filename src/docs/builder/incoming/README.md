# Incoming Feature Requests and Bug Reports

This folder collects feature requests and bug reports between sprints. The sprint coordinator checks this folder during Phase 1 (Sprint Planning).

## Usage

### Feature Requests: `FR-###.md`

```markdown
# FR-###: [Short Title]

**Status:** Pending
**Priority:** [High/Medium/Low]
**Requested By:** [User/Stakeholder]
**Date:** YYYY-MM-DD

## Description
[Detailed description]

## User Story
As a [user type], I want to [action] so that [benefit].

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
```

### Bug Reports: `BUG-###.md`

```markdown
# BUG-###: [Short Title]

**Status:** Pending
**Priority:** [Critical/High/Medium/Low]
**Date:** YYYY-MM-DD

## Description
[What's broken]

## Steps to Reproduce
1. Step 1
2. Step 2

## Expected vs Actual
Expected: [...]
Actual: [...]

## Environment
- tq version: [version]
- OS: [os]
```

## Workflow

**Phase 1 - Sprint Planning:**
1. Sprint coordinator reads all files in incoming/
2. Filters by git commit date for new items since last sprint
3. Evaluates against project vision
4. Prioritizes and includes in sprint-N-planning.md
5. Moves items to: backlog/, declined/, or completed/

Sprint coordinator uses git commit dates to track what's new.
