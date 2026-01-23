# Skill Creation Process

Step-by-step guidance for creating effective skills.

## 6-Step Process

### Step 1: Understand the Need

Gather concrete examples and requirements:
- What task should the skill help with?
- What expertise or workflow should it encode?
- What are specific examples of desired behavior?
- What constraints or requirements exist?

### Step 2: Plan Contents

Decide what resources are needed:

**For SKILL.md body:**
- Core instructions and procedures
- Key decision points and guidelines
- Brief examples

**For bundled resources:**
- Scripts: Identify operations needing exact execution
- References: Determine if lengthy documentation is needed
- Assets: List templates or boilerplate to include

### Step 3: Choose Instruction Style

Match instruction detail to task variability:

| Task Type | Style | Example |
|-----------|-------|---------|
| Flexible tasks | Brief principles | "Analyze code, suggest improvements" |
| Structured workflows | Step-by-step procedures | Numbered steps with decision points |
| Precise operations | Exact scripts | Specific commands to run |

### Step 4: Write the SKILL.md

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
- Use imperative form: "Create", "Use", "Follow"
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

### Step 5: Add Bundled Resources (If Needed)

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

### Step 6: Iterate and Refine

Test the skill and improve:
1. Use the skill in real scenarios
2. Remove unnecessary content
3. Clarify ambiguous instructions
4. Add examples for common edge cases
5. Optimize token usage
6. Test with Haiku, Sonnet, and Opus models
