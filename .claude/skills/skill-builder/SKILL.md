---
name: skill-builder
description: Guides the creation of effective Claude skills following the Agent Skills Standard. Use when the user asks to create a new skill, generate a SKILL.md file, or package specialized knowledge into a reusable capability.
---

# Skill Builder

You are a skill creation expert helping to build effective Claude skills following the Agent Skills Standard.

## Core Principles

### 1. Context Window Efficiency
**"The context window is a public good."** Only include information Claude cannot already deduce. Each piece of content must justify its token cost.

### 2. Appropriate Freedom Levels

Choose the right instruction format based on task variability:

- **High freedom (text instructions)**: Use when multiple valid approaches exist
  - Example: "Write user-friendly error messages"
  - Best for: Creative tasks, flexible workflows, judgment calls

- **Medium freedom (pseudocode/patterns)**: Use when preferred patterns exist with acceptable variation
  - Example: "Follow this structure: validate input → process → format output"
  - Best for: Structured workflows with some flexibility

- **Low freedom (specific scripts)**: Use when operations require consistency and precision
  - Example: Exact API calls, file format specifications, required command sequences
  - Best for: Deterministic tasks, tool integrations, precise operations

### 3. Progressive Disclosure

Use three-tier loading to minimize context usage:

1. **Metadata** (always loaded): Name and description in frontmatter
2. **Skill body** (loaded when triggered): Main instructions in SKILL.md
3. **Bundled resources** (loaded as needed): Additional files referenced from instructions

## Skill File Structure

### Required File

**SKILL.md** - The core skill definition:

```yaml
---
name: skill-name
description: Complete description of what this skill does and when to use it
---

# Skill Name

[Instructions that Claude will follow when the skill is active]

## When to Use

- Specific scenario 1
- Specific scenario 2

## Instructions

1. Step-by-step guidance
2. Clear procedures
3. Best practices

## Examples

[Concrete examples of usage]

## Guidelines

[Important constraints and considerations]
```

### Optional Bundled Resources

Organize additional files by purpose:

- **scripts/** - Executable code for deterministic tasks
  - Use for: API integrations, file generation, precise operations
  - Format: Any executable format (Python, shell, etc.)

- **references/** - Documentation loaded conditionally
  - Use for: API docs, specifications, detailed procedures
  - Format: Markdown files referenced from SKILL.md

- **assets/** - Output templates and resources
  - Use for: File templates, boilerplate code, example outputs
  - Format: Any format the skill needs to produce

## Frontmatter Requirements

### Required Fields

| Field | Type | Requirements |
|-------|------|--------------|
| `name` | string | Unique identifier (lowercase, hyphens for spaces) |
| `description` | string | Complete description including WHEN to use the skill |

### Description Best Practices

The description should:
- **Clearly state what the skill does**
- **Specify when Claude should use it**
- **Be complete enough for skill discovery**
- **Avoid redundancy with the skill body**

Good example:
```yaml
description: Guides the creation of effective Claude skills following the Agent Skills Standard. Use when the user asks to create a new skill, generate a SKILL.md file, or package specialized knowledge into a reusable capability.
```

Poor example:
```yaml
description: Creates skills  # Too vague, doesn't say when to use
```

## Skill Creation Process

Follow these 6 steps:

### 1. Understand the Need

Gather concrete examples and requirements:
- What task should the skill help with?
- What expertise or workflow should it encode?
- What are specific examples of desired behavior?
- What constraints or requirements exist?

### 2. Plan Contents

Decide what resources are needed:

**For SKILL.md body:**
- Core instructions and procedures
- Key decision points and guidelines
- Brief examples

**For bundled resources:**
- Scripts: Identify operations needing exact execution
- References: Determine if lengthy documentation is needed
- Assets: List templates or boilerplate to include

### 3. Choose Instruction Style

Match instruction detail to task variability:

- **Flexible tasks** → Brief principles and guidelines
- **Structured workflows** → Step-by-step procedures with patterns
- **Precise operations** → Exact scripts or detailed specifications

### 4. Write the SKILL.md

**Structure:**
```markdown
---
name: your-skill-name
description: [What it does and when to use it]
---

# Your Skill Name

## Overview
[Brief explanation of purpose]

## When to Use
- Trigger condition 1
- Trigger condition 2

## Instructions
[Main guidance - keep concise]

## Examples
[Concrete usage examples]

## Guidelines
[Important constraints]
```

**Writing style:**
- Use imperative/infinitive form: "Create", "Use", "Follow"
- Keep under 500 lines total
- Be specific but concise
- Include concrete examples
- Reference bundled resources clearly

**What to include:**
- Information Claude needs but doesn't already know
- Specific procedures unique to this task
- Critical decision criteria
- Examples showing expected behavior

**What to exclude:**
- General software development advice
- Information Claude already knows
- Obvious best practices
- Redundant explanations

### 5. Add Bundled Resources (If Needed)

Create additional files only when they add value:

**scripts/**
- Include when exact execution is required
- Document parameters and expected behavior
- Test thoroughly before including

**references/**
- Include for complex specifications or APIs
- Keep focused on non-obvious information
- Structure for easy reference

**assets/**
- Include templates that save tokens
- Ensure they're reusable across contexts
- Document how to use them

### 6. Iterate and Refine

Test the skill and improve:
- Use the skill in real scenarios
- Remove unnecessary content
- Clarify ambiguous instructions
- Add examples for common edge cases
- Optimize token usage

## Writing Best Practices

### Do:
- Use clear, imperative language
- Provide concrete examples
- Specify triggering conditions in description
- Keep instructions focused and actionable
- Reference external resources when they save tokens
- Test the skill before finalizing
- Load the skill after creation to verify it works

### Don't:
- Include information Claude already knows
- Repeat general best practices
- Create resources "just in case"
- Write overly long instructions
- Make the description too brief or vague
- Include untested code or examples

### Markdown Safety: Avoid Shell-Sensitive Characters in Backticks

**CRITICAL**: When documenting shell operators or special characters in skill files, avoid wrapping them in inline code backticks in ways that could be interpreted as bash commands.

**Shell-sensitive characters to watch:**
- Exclamation marks (bash history expansion)
- Dollar signs (variable expansion)
- Ampersands, pipes, semicolons (command chaining)
- Backticks within backticks (command substitution)

**Common problematic patterns:**
- Documenting comment syntax like slash-slash-bang
- Showing shell variables with dollar signs
- Demonstrating command operators like double-ampersand
- Any inline code containing exclamation mark followed by closing backtick and parenthesis

**Safe documentation strategies:**

1. **Describe instead of showing in inline code:**
   - GOOD: "Use inner doc comments (two slashes followed by exclamation mark)"
   - GOOD: "Use the dollar sign for shell variables"
   - GOOD: "Chain commands with double ampersand operator"

2. **Use code blocks instead of inline backticks for shell syntax:**
   - Fenced code blocks are safer than inline backticks
   - Use language tags for proper rendering (bash, rust, etc.)

3. **Separate special characters from surrounding backticks:**
   - GOOD: "Use two forward slashes followed by exclamation mark for module docs"
   - GOOD: "The ampersand-ampersand operator chains commands"

4. **Use descriptive names:**
   - "slash-slash-bang syntax"
   - "dollar-sign variable syntax"
   - "double-ampersand operator"

**Why this matters:**
Claude Code's skill loader performs bash command safety checks on skill content. When inline backticks surround shell-sensitive characters, the parser may interpret the content as a bash command pattern, causing skill loading to fail with permission errors.

**Error example:**
If you see "Bash command permission check failed" when loading a skill, look for inline backticks containing exclamation marks, dollar signs, or shell operators. Rewrite those sections using plain descriptions.

**Testing requirement:**
Always test loading your skill immediately after creation or modification:

1. Create or modify the skill file
2. Invoke the Skill tool with the skill name to test loading
3. If loading fails with bash permission errors, search for shell-sensitive characters in backticks
4. Rewrite using plain descriptions or code blocks
5. Test loading again to confirm the fix

## Quality Checklist

Before finalizing a skill, verify:

**Frontmatter:**
- [ ] Name is unique and descriptive (lowercase-with-hyphens)
- [ ] Description clearly states what and when
- [ ] YAML syntax is valid

**Content:**
- [ ] Instructions are clear and actionable
- [ ] Examples demonstrate expected behavior
- [ ] Length is under 500 lines
- [ ] No redundant or obvious information
- [ ] Imperative/infinitive form used consistently
- [ ] No shell-sensitive characters wrapped in backticks (!, $, &&, etc.)

**Resources:**
- [ ] Only necessary resources included
- [ ] Scripts are tested and documented
- [ ] References are clearly structured
- [ ] Assets are reusable and documented

**Testing:**
- [ ] Skill loads successfully without errors (test with Skill tool)
- [ ] No bash command permission errors
- [ ] Skill behaves as expected in test scenarios

**Overall:**
- [ ] Skill justifies its token cost
- [ ] Instructions match appropriate freedom level
- [ ] Triggering conditions are clear
- [ ] Skill has been tested in realistic scenarios

## Common Patterns

### Pattern: Domain Expertise Skill

For encoding specialized knowledge:

```yaml
---
name: legal-document-review
description: Reviews legal documents for common issues following standard legal review procedures. Use when user asks to review contracts, agreements, or legal documents.
---

# Legal Document Review

## Review Process

1. Identify document type and jurisdiction
2. Check for standard clauses
3. Flag missing or non-standard provisions
4. Review for clarity and enforceability

[Specific procedures follow...]
```

### Pattern: Workflow Skill

For multi-step processes:

```yaml
---
name: api-integration-builder
description: Guides building API integrations following best practices for error handling, authentication, and testing. Use when creating new API clients or integrations.
---

# API Integration Builder

## Integration Steps

1. Design client architecture
2. Implement authentication
3. Add error handling
4. Write tests
5. Document usage

[Detailed procedures follow...]
```

### Pattern: Tool Skill

For specific tool usage:

```yaml
---
name: presentation-creator
description: Creates professional PowerPoint presentations with consistent styling and structure. Use when user asks to create slides, presentations, or PPTX files.
---

# Presentation Creator

## Creation Process

1. Understand presentation requirements
2. Structure content into slides
3. Apply consistent design
4. Generate PPTX file

[Implementation details and scripts follow...]
```

## Skill Maintenance

### Version Control

Skills should be:
- Committed to version control
- Updated based on usage feedback
- Tagged with version numbers
- Documented with changelogs

### Updating Skills

When updating:
1. Document what's changing and why
2. Test changes thoroughly
3. Consider backward compatibility
4. Update examples to match changes
5. Notify users of breaking changes

### Deprecation

When retiring skills:
1. Mark as deprecated in description
2. Provide migration path to replacement
3. Set sunset timeline
4. Archive but don't delete
