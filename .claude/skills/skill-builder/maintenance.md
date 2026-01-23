# Skill Maintenance

Guidelines for versioning, updating, and deprecating skills.

## Version Control

Skills should be:
- Committed to version control
- Updated based on usage feedback
- Tagged with version numbers in frontmatter
- Documented with changelogs

### Versioning in Frontmatter

```yaml
---
name: my-skill
version: 2.1.0
description: ...
---
```

Use semantic versioning:
- **Major (X.0.0)**: Breaking changes
- **Minor (0.X.0)**: New features, backward compatible
- **Patch (0.0.X)**: Bug fixes

## Updating Skills

When updating:

1. **Document what's changing and why**
   - Add a changelog entry
   - Note the reason for the change

2. **Test changes thoroughly**
   - Test with Haiku, Sonnet, and Opus
   - Verify in realistic scenarios

3. **Consider backward compatibility**
   - Will existing workflows break?
   - Do users need to update their usage?

4. **Update examples to match changes**
   - Stale examples confuse users

5. **Notify users of breaking changes**
   - Mark clearly in description if needed

## Deprecation

When retiring skills:

1. **Mark as deprecated in description**
   ```yaml
   description: "[DEPRECATED] Use new-skill instead. ..."
   ```

2. **Provide migration path**
   - Link to replacement skill
   - Document how to migrate

3. **Set sunset timeline**
   - Give users time to migrate
   - Communicate removal date

4. **Archive but don't delete**
   - Keep for reference
   - May help with edge cases

## Changelog Example

Maintain a CHANGELOG.md in the skill directory:

```markdown
# Changelog

## [2.1.0] - 2024-01-15
### Added
- New progressive disclosure pattern

### Changed
- Simplified frontmatter section

## [2.0.0] - 2024-01-01
### Changed
- BREAKING: Restructured file organization
- Moved examples to separate file

### Removed
- Deprecated "old-pattern" section
```

## Quality Review Checklist

Before releasing an update:

- [ ] Version number updated
- [ ] Changelog entry added
- [ ] Tested with all target models
- [ ] Examples still accurate
- [ ] No broken references
- [ ] Description still accurate
