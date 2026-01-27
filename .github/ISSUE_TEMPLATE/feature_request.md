---
name: Feature Request
about: Suggest a new feature or enhancement for tq
title: "[FEATURE] "
labels: enhancement
assignees: ''
---

## Feature Description

A clear and concise description of the feature you'd like to see in tq.

## Problem / Use Case

**What problem does this solve?**

Describe the problem you're trying to solve or the workflow you're trying to improve.

**Who would benefit from this feature?**

- [ ] DBAs managing Teradata databases
- [ ] Data Analysts running ad-hoc queries
- [ ] Developers integrating tq into scripts/pipelines
- [ ] DevOps engineers automating database tasks
- [ ] All users

## Proposed Solution

**How should this feature work?**

Describe your ideal implementation. Include:
- Command syntax (if applicable): `tq [flags] [args]`
- Configuration options (if applicable)
- Expected output or behavior

**Example Usage:**

```bash
# Show how you envision using this feature
tq --your-proposed-flag "example"
```

**Example Output:**

```
# What output would you expect to see?
```

## Alternatives Considered

**Have you considered any alternative solutions or workarounds?**

Describe any alternative approaches you've thought about, including:
- Other CLI tools that have similar features
- Current workarounds you're using
- Why those alternatives aren't ideal

## Feature Category

Which area of tq does this feature relate to?

- [ ] Core Query Execution
- [ ] REPL/Interactive Mode
- [ ] Output Formatting (table, JSON, CSV, etc.)
- [ ] Connection Management (profiles, credentials)
- [ ] Configuration (.tq/config.toml, environment variables)
- [ ] Error Handling/Messages
- [ ] Performance/Optimization
- [ ] Batch Processing
- [ ] Security/Credentials
- [ ] Tab Completion
- [ ] Help/Documentation
- [ ] Integration with other tools
- [ ] Other: ___

## Compatibility

**Would this feature require:**

- [ ] New command-line flags
- [ ] New configuration options
- [ ] Changes to existing behavior (potentially breaking)
- [ ] New dependencies or external tools
- [ ] Teradata-specific features (specific version required)

## Priority

**How important is this feature to your workflow?**

- [ ] Critical - Blocking my ability to use tq effectively
- [ ] High - Would significantly improve my productivity
- [ ] Medium - Would be nice to have
- [ ] Low - Small quality-of-life improvement

## Additional Context

**Related Features:**

Are there existing tq features that this relates to or would complement?

**Reference Implementations:**

Are there other tools (CLI or otherwise) that implement something similar? How do they handle it?

**Design Considerations:**

Any thoughts on implementation challenges, edge cases, or design trade-offs?

**Screenshots/Mockups:**

If applicable, add screenshots, mockups, or examples to help explain the feature.

## Example Scenarios

**Scenario 1:**

```bash
# Before (current workflow)
tq -q "SELECT * FROM table" > output.json
cat output.json | jq '.results'

# After (with this feature)
tq --your-feature "SELECT * FROM table"
```

**Scenario 2:**

[Additional usage scenario if applicable]

---

**Thank you for taking the time to suggest improvements to tq!**
