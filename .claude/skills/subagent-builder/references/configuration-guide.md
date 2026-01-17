# Subagent Configuration Reference

This reference provides detailed guidance for configuring subagents effectively.

## Complete Tool Reference

### Available Tools by Category

#### Read-Only Tools
Perfect for research, analysis, and exploration agents.

| Tool | Purpose | Common Use Cases |
|------|---------|-----------------|
| **Read** | Read files and directories | Code review, analysis, documentation |
| **Glob** | Pattern-based file finding | Searching for specific file types |
| **Grep** | Content search with regex | Finding code patterns, searching codebase |
| **Bash** | Command execution | Running read-only commands (ls, git status, etc.) |

**Typical configuration:**
```yaml
tools: Read, Grep, Glob, Bash
```

#### File Modification Tools
For agents that need to create or change code.

| Tool | Purpose | Common Use Cases |
|------|---------|-----------------|
| **Write** | Create new files | Code generation, file creation |
| **Edit** | Modify existing files | Refactoring, bug fixes, updates |

**Typical configuration:**
```yaml
tools: Read, Write, Edit, Grep, Glob, Bash
```

#### Interactive Tools
For agents requiring user interaction.

| Tool | Purpose | Common Use Cases |
|------|---------|-----------------|
| **AskUserQuestion** | Prompt user for decisions | Clarifying requirements, choosing options |

**Note**: Use sparingly in background agents (auto-denied)

#### Specialized Tools

| Tool | Purpose | Common Use Cases |
|------|---------|-----------------|
| **WebFetch** | Fetch web content | API documentation, external resources |
| **WebSearch** | Search the web | Research, finding documentation |
| **NotebookEdit** | Edit Jupyter notebooks | Data science workflows |

### Tool Access Patterns

#### Pattern 1: Read-Only Researcher
```yaml
tools: Read, Grep, Glob, Bash
disallowedTools: Write, Edit, AskUserQuestion
```
- Cannot modify files
- Cannot prompt user
- Best for: analysis, exploration, code review

#### Pattern 2: Autonomous Editor
```yaml
tools: Read, Write, Edit, Grep, Glob, Bash
permissionMode: acceptEdits
```
- Can modify files freely
- Auto-approved changes
- Best for: refactoring, code generation

#### Pattern 3: Command Runner Only
```yaml
tools: Bash, Read
disallowedTools: Write, Edit, Grep, Glob
```
- Only runs commands and reads results
- Best for: test runners, build executors, query runners

#### Pattern 4: Full Access
```yaml
# Omit tools field entirely to inherit all tools
```
- Maximum flexibility
- Best for: complex workflows, general-purpose agents

## Model Selection Deep Dive

### Model Characteristics

| Model | Speed | Cost | Capability | Best For |
|-------|-------|------|------------|----------|
| **Haiku** | Fastest | Lowest | Good | Well-defined, simple tasks |
| **Sonnet** | Fast | Medium | Excellent | Most general work |
| **Opus** | Slower | Highest | Best | Complex reasoning, critical decisions |
| **Inherit** | Varies | Varies | Matches parent | Consistency with main conversation |

### Detailed Model Selection Criteria

#### Choose Haiku When:

**Task characteristics:**
- Well-defined with clear success criteria
- Repetitive or pattern-based operations
- High volume of similar operations
- Simple search and retrieval
- Straightforward analysis

**Examples:**
- Code review for specific patterns (missing error handling)
- Running test suites and reporting results
- Searching codebase for specific patterns
- Generating simple reports from structured data
- Validating configuration files

**Cost consideration:**
- Haiku is ~20x cheaper than Opus
- For high-volume tasks, savings are significant

#### Choose Sonnet When:

**Task characteristics:**
- General-purpose development work
- Moderate complexity requiring judgment
- Balanced need for quality and speed
- Standard code generation and modification
- Most review and analysis tasks

**Examples:**
- Code reviews with nuanced feedback
- Implementing features with moderate complexity
- Refactoring existing code
- Writing tests with good coverage
- Debugging common issues

**Default recommendation:**
- Best balance for most use cases
- 90% of subagents should use Sonnet

#### Choose Opus When:

**Task characteristics:**
- Complex architectural decisions
- Nuanced reasoning required
- High stakes or critical systems
- Novel problem solving
- Multi-step planning with many variables

**Examples:**
- Designing system architecture
- Making security-critical decisions
- Analyzing complex distributed systems
- Resolving tricky bugs requiring deep investigation
- Reviewing sensitive or high-risk code

**Cost consideration:**
- Use sparingly due to higher cost
- Reserve for tasks where quality is paramount

#### Choose Inherit When:

**Task characteristics:**
- Consistency with main conversation is important
- User has already selected optimal model
- Agent is extension of current work

**Examples:**
- Subagents that continue current task in isolation
- When user explicitly chose a model for this session
- Maintaining consistent behavior across conversation

## Permission Modes In Detail

### Mode: default (Standard)

**Behavior:**
- Normal permission checks
- User prompted for sensitive operations
- Standard safety guardrails

**When to use:**
- Most general-purpose agents
- When user oversight is desired
- Default choice for new agents

**Example:**
```yaml
permissionMode: default
```

### Mode: acceptEdits (Auto-Approve File Changes)

**Behavior:**
- File edits and writes auto-approved
- Other permissions still checked
- Reduces friction for trusted agents

**When to use:**
- Trusted code generation agents
- Refactoring agents
- When user wants uninterrupted workflow

**Example:**
```yaml
permissionMode: acceptEdits
```

**Caution:** Agent can modify any file without prompting

### Mode: dontAsk (Auto-Deny)

**Behavior:**
- Unpermitted actions automatically denied
- No user prompts
- Agent must work within constraints

**When to use:**
- Background agents
- Enforcing strict read-only behavior
- When interruptions are unwanted

**Example:**
```yaml
permissionMode: dontAsk
```

**Best paired with:** Limited tool access

### Mode: bypassPermissions (No Checks)

**Behavior:**
- All permissions automatically granted
- No safety prompts
- Maximum autonomy

**When to use:**
- Fully trusted agents
- Automated CI/CD workflows
- Internal tools with validated behavior

**Example:**
```yaml
permissionMode: bypassPermissions
```

**Caution:** Use very carefully - agent has unrestricted access

### Mode: plan (Read-Only Exploration)

**Behavior:**
- Only read operations allowed
- Cannot modify files or execute write commands
- Perfect for analysis and research

**When to use:**
- Code exploration agents
- Analysis and review agents
- Research agents

**Example:**
```yaml
permissionMode: plan
```

**Best paired with:** Read-only tools (Read, Grep, Glob)

## Hook Patterns

Hooks enable validation, automation, and lifecycle management.

### Pattern: Query Validation

**Use case:** Ensure database agent only runs safe queries

**Implementation:**
```yaml
hooks:
  PreToolUse:
    - matcher: "Bash"
      hooks:
        - type: command
          command: "./scripts/validate-db-query.sh"
```

**Validation script:**
```bash
#!/bin/bash
INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

# Block write operations
if echo "$COMMAND" | grep -iE 'INSERT|UPDATE|DELETE|DROP|CREATE|ALTER' > /dev/null; then
  echo "ERROR: Only SELECT queries allowed" >&2
  exit 2
fi

exit 0
```

### Pattern: Post-Edit Linting

**Use case:** Auto-lint files after agent modifies them

**Implementation:**
```yaml
hooks:
  PostToolUse:
    - matcher: "Edit|Write"
      hooks:
        - type: command
          command: "./scripts/auto-lint.sh"
```

**Lint script:**
```bash
#!/bin/bash
INPUT=$(cat)
FILE=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if [[ "$FILE" == *.rs ]]; then
  cargo fmt --quiet -- "$FILE"
elif [[ "$FILE" == *.py ]]; then
  black "$FILE" 2>/dev/null
fi

exit 0
```

### Pattern: Environment Setup

**Use case:** Initialize database connection before agent starts

**Project-level hook in settings.json:**
```json
{
  "hooks": {
    "SubagentStart": [
      {
        "matcher": "db-query-agent",
        "hooks": [
          {
            "type": "command",
            "command": "./scripts/setup-db-connection.sh"
          }
        ]
      }
    ],
    "SubagentStop": [
      {
        "matcher": "db-query-agent",
        "hooks": [
          {
            "type": "command",
            "command": "./scripts/cleanup-db-connection.sh"
          }
        ]
      }
    ]
  }
}
```

## Skills in Subagents

Skills provide domain expertise to subagents without duplicating content.

### When to Include Skills

**Include skills when:**
- Agent needs domain-specific knowledge (e.g., Rust guidelines)
- Reusing documented patterns and procedures
- Enforcing project-specific standards

**Don't include skills when:**
- Information is general knowledge
- Skill content would be redundant with prompt
- Agent needs minimal context

### Skill Inclusion Example

**Rust code reviewer with project standards:**
```yaml
name: rust-reviewer
description: Reviews Rust code following project standards
tools: Read, Grep, Glob, Bash
model: sonnet
skills: rust-coder, rust-debugger
```

**Why include rust-coder?**
- Contains project-specific Rust conventions
- Avoids duplicating guidelines in agent prompt
- Ensures consistency with main conversation standards

### Multiple Skills

Agents can load multiple relevant skills:
```yaml
skills: rust-coder, rust-debugger, teradata-rust
```

**Best practice:** Only include skills the agent will actually reference

## Storage Location Decision Matrix

### Choose Project-Level (.claude/agents/)

**When:**
- Agent is specific to this project
- Team should have access
- Want version control
- Project-specific configurations or constraints

**Examples:**
- Code reviewers using project standards
- Build/test runners for this codebase
- Database agents for project database
- Deployment agents with project-specific steps

**Benefits:**
- Shared across team
- Version controlled
- Project context included

### Choose User-Level (~/.claude/agents/)

**When:**
- Agent is useful across multiple projects
- Personal productivity tool
- User-specific preferences
- Not relevant to team

**Examples:**
- General-purpose code formatters
- Personal note-taking agents
- Cross-project search utilities
- Generic analysis tools

**Benefits:**
- Available everywhere
- Personal customization
- Not cluttering project repo

## Common Configuration Mistakes

### Mistake 1: Too Many Tools

**Problem:**
```yaml
# Unnecessary - agent only needs to read
tools: Read, Write, Edit, Grep, Glob, Bash, AskUserQuestion
```

**Solution:**
```yaml
tools: Read, Grep, Glob  # Only what's needed
```

**Principle:** Grant minimum necessary tool access

### Mistake 2: Wrong Model Selection

**Problem:**
```yaml
# Opus for simple pattern matching
model: opus
```

**Solution:**
```yaml
model: haiku  # Fast and efficient for simple tasks
```

**Principle:** Match model to task complexity

### Mistake 3: Overly Permissive

**Problem:**
```yaml
# Bypass permissions for code review agent
permissionMode: bypassPermissions
```

**Solution:**
```yaml
permissionMode: plan  # Read-only for review
```

**Principle:** Use least privileged permission mode

### Mistake 4: Vague Description

**Problem:**
```yaml
description: Helps with code tasks
```

**Solution:**
```yaml
description: Reviews Rust code for safety and performance issues. Use proactively after writing Rust code.
```

**Principle:** Be specific about what and when

### Mistake 5: Overly Complex Prompt

**Problem:**
- 500-line prompt covering every edge case
- General programming advice Claude already knows
- Redundant explanations

**Solution:**
- Focus on domain-specific guidance
- Keep under 200 lines
- Trust Claude's baseline knowledge

**Principle:** Prompt should add new information, not repeat common sense

## Testing Checklist

Before deploying a subagent, verify:

### Configuration Tests
- [ ] YAML frontmatter is valid
- [ ] Name follows naming convention (lowercase-with-hyphens)
- [ ] Description is specific and mentions triggering conditions
- [ ] Tools list is minimal but sufficient
- [ ] Model selection matches task complexity
- [ ] Permission mode is appropriate for trust level

### Functional Tests
- [ ] Agent loads without errors (use Task tool to test)
- [ ] Tools work as expected (no permission issues)
- [ ] Permission mode functions correctly
- [ ] Output quality meets requirements
- [ ] Agent completes task successfully

### Performance Tests
- [ ] Model choice is cost-effective
- [ ] Agent doesn't hit token limits
- [ ] Response time is acceptable
- [ ] Background execution works if needed

### Safety Tests
- [ ] Cannot perform unintended actions
- [ ] Hooks validate correctly (if using hooks)
- [ ] Permission mode isn't too permissive
- [ ] Tool access is properly restricted

## Summary

Effective subagent configuration requires:

1. **Minimal tool access** - Grant only what's needed
2. **Appropriate model** - Match complexity to capability
3. **Safe permissions** - Use least privileged mode
4. **Clear description** - Specify what and when
5. **Focused prompt** - Domain-specific guidance only
6. **Thorough testing** - Verify behavior before deployment

The goal is specialized agents that are efficient, safe, and predictable.
