---
name: skill-builder
description: Creates effective Claude skills following best practices. Use when: creating a new skill, generating a SKILL.md file, or packaging specialized knowledge into a reusable capability.
---

# Skill Builder

Create skills that are **concise, discoverable, and effective**.

## Core Principle: Conciseness

> **The context window is a public good.**

For every piece of content, ask:
- "Does Claude really need this?"
- "Can I assume Claude already knows this?"
- "Does this justify its token cost?"

**Keep SKILL.md under 500 lines.** Split larger content into separate files.

## Skill Structure

```
skill-name/
├── SKILL.md          # Main instructions (loaded when triggered)
├── reference.md      # Detailed docs (loaded as needed)
├── examples.md       # Usage examples (loaded as needed)
└── scripts/          # Utility scripts (executed, not loaded)
    └── helper.py
```

## Detailed References

**For complete guidance, see these files:**
- **[creation-process.md](creation-process.md)**: Step-by-step skill creation workflow
- **[patterns.md](patterns.md)**: Common skill patterns with examples
- **[safety.md](safety.md)**: Markdown safety and shell-sensitive characters
- **[maintenance.md](maintenance.md)**: Versioning, updating, and deprecation

## Frontmatter Requirements

```yaml
---
name: skill-name          # Max 64 chars, lowercase-with-hyphens
description: What it does and WHEN to use it  # Max 1024 chars
---
```

**Good description:**
```yaml
description: Extracts text from PDF files. Use when working with PDFs or document extraction.
```

**Bad description:**
```yaml
description: Handles PDFs  # Too vague, doesn't say when to use
```

## Freedom Levels

Match specificity to task fragility:

| Level | When | Example |
|-------|------|---------|
| **High** (text) | Multiple valid approaches | "Analyze code structure, check for bugs" |
| **Medium** (pseudocode) | Preferred pattern exists | Template with parameters |
| **Low** (exact script) | Consistency critical | "Run exactly: `python scripts/migrate.py`" |

## Key Patterns

### 1. Progressive Disclosure
SKILL.md is the overview. Reference detailed content:

```markdown
## Advanced features
**Form filling**: See [FORMS.md](FORMS.md)
**API reference**: See [REFERENCE.md](REFERENCE.md)
```

Claude loads these only when needed.

### 2. Workflow + Checklist
For complex tasks, provide a copyable checklist:

```markdown
## Workflow
Copy this checklist:
- [ ] Step 1: Analyze input
- [ ] Step 2: Process data
- [ ] Step 3: Validate output

**Step 1: Analyze input**
[Details...]
```

### 3. Template Pattern
Provide output templates:

```markdown
## Report structure
Use this template:
# [Title]
## Summary
[Overview]
## Findings
[Details]
```

### 4. Examples Pattern
Show input/output pairs:

```markdown
## Examples
**Input:** Added user authentication
**Output:** `feat(auth): implement JWT authentication`
```

## Anti-Patterns to Avoid

| Don't | Do Instead |
|-------|------------|
| Explain what Claude knows | Assume Claude is smart |
| Offer many options | Provide a default (with escape hatch) |
| Use Windows paths `\` | Use forward slashes `/` |
| Include "just in case" content | Only what's necessary |
| Magic constants | Document why each value exists |

## Scripts: Solve, Don't Punt

Scripts should handle errors, not fail and let Claude figure it out:

```python
# Good: Handle errors explicitly
def process(path):
    try:
        return open(path).read()
    except FileNotFoundError:
        return ""  # Sensible default
```

## Quality Checklist

**Before finalizing:**
- [ ] Description says WHAT and WHEN
- [ ] Under 500 lines
- [ ] No redundant explanations
- [ ] Examples are concrete
- [ ] Tested with Haiku, Sonnet, and Opus
- [ ] Forward slashes only (no Windows paths)
