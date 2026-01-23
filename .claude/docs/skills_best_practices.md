# Claude Code Skills and Agents: Best Practices Cheat Sheet

## State-of-the-Art Overview

**What is Claude Code?** Claude Code is Anthropic’s AI coding assistant that functions as a conversational developer tool. Unlike traditional code completion tools, it operates as an interactive REPL in your terminal: it can read files, execute commands, manage context, and spawn specialized sub-agents. Think of it as a collaborative, agentic pair programmer rather than an autocomplete engine.

**The Problem Skills Solve**

Large language model (LLM) agents are powerful but fragile:
- Long prompts accumulate noise and degrade reasoning quality.
- Repeated instructions waste context and tokens.
- Domain or workflow knowledge is hard to reuse consistently.

Skills address this by externalizing repeatable knowledge and procedures into modular, on-demand units. Only minimal metadata is loaded by default; full instructions are injected *only when relevant*. This keeps the active context small, focused, and reliable.

**How Skills Compare to Other Techniques**

| Technique | Strengths | Limitations |
|--------|-----------|-------------|
| Long prompts | Simple, fast to prototype | Context bloat, hard to maintain |
| System prompts | Strong control | Monolithic, not reusable |
| Fine-tuning | High consistency | Expensive, inflexible |
| Plugins / function calling | Structured I/O | Rigid schemas, external infra |
| **Skills** | Modular, auditable, reusable | Require design discipline |

Skills occupy a sweet spot: lightweight like prompts, reusable like plugins, auditable like code.

**Supporting Technical Frameworks**

- **Claude Code**: Terminal-native conversational agent with tool execution
- **Agent Skills format**: Open standard (`SKILL.md` + YAML metadata)
- **Sub-agents**: Isolated, role-specific agents with clean context
- **Model Context Protocol (MCP)**: External tool and data integration
- **Permissions system**: Tool allowlists, confirmations, and denials

---

## 1. Discoverability: Metadata Is the Router

### Principle

Claude decides *whether* to load a skill based almost entirely on its metadata—especially `description`.

### Do
- Write descriptions in **third person**
- Include *when to use it* and *trigger phrases*
- Be explicit and concrete

**Good**
```yaml
name: reviewing-prs
description: Reviews pull requests by summarizing changes, flagging risks, and suggesting improvements. Use when asked to review a PR or merge request.
```

### Don’t
- Be vague or generic
- Assume the name alone is sufficient

**Bad**
```yaml
description: Helps with code.
```

---

## 2. Context Economy: Progressive Disclosure

### Principle

Context is a shared, finite budget. Skills should minimize what is loaded eagerly and defer everything else.

### Do
- Keep `SKILL.md` concise
- Split large material into referenced files
- Add tables of contents for files >100 lines

**Example Structure**
```
my-skill/
├─ SKILL.md
├─ REFERENCE.md
├─ EXAMPLES.md
└─ scripts/
   └─ helper.sh
```

### Don’t
- Nest references multiple levels deep
- Dump long tutorials into `SKILL.md`

---

## 3. Degree of Freedom: Match the Task

### Principle

Different tasks need different constraints.

| Task Type | Freedom Level | Example |
|---------|---------------|---------|
| Exploratory | High | Architecture review |
| Procedural | Medium | Code generation |
| Fragile / risky | Low | Deployment, migrations |

### Do
- Be explicit where precision matters
- Lock down flags, order, and commands for fragile workflows

### Don’t
- Over-constrain creative tasks
- Under-specify dangerous ones

---

## 4. Determinism: Prefer Scripts for Reliability

### Principle

If correctness and repeatability matter, let code—not the model—do the work.

### Do
- Bundle scripts inside skills
- Instruct Claude to *run*, not recreate, logic

**Example**
```bash
python3 extract_text.py input.pdf
```

### Don’t
- Ask Claude to regenerate complex logic every time
- Hide assumptions in prose

---

## 5. Safety and Control: Frontmatter Matters

### Principle

YAML frontmatter is your safety harness.

### Useful Controls
- `allowed-tools`: limit execution surface
- `disable-model-invocation: true`: manual-only skills
- Sub-agent `agent:` field for isolation

**Example**
```yaml
allowed-tools: Read, Grep
disable-model-invocation: true
```

### Don’t
- Allow write or bash access unnecessarily
- Auto-trigger destructive workflows

---

## 6. Evaluation-First Development

### Principle

Build skills to fix observed failures, not hypotheticals.

### Do
- Start with real failing tasks
- Establish a baseline
- Add only the minimum instructions needed

### Don’t
- Speculate endlessly
- Optimize before observing failures

---

## 7. Sub-Agents: Divide and Conquer

### Principle

Isolate complex or specialized reasoning into clean contexts.

### Do
- Give agents narrow, explicit roles
- Restrict tools per agent
- Let them report summaries back

**Example Agent Roles**
- Code reviewer
- Security auditor
- Documentation writer
- Web researcher

### Don’t
- Create "do-everything" agents
- Assume agents share full context

---

## 8. Workflow Patterns That Work

- Plan first, execute second
- Stay in the loop—review diffs and outputs
- Use `CLAUDE.md` for project-wide memory
- Compact context periodically

---

## References

| Document | Why it matters |
|--------|----------------|
| Skill authoring best practices (Anthropic) | Canonical rules for naming, structure, and discovery |
| Extend Claude with skills | Claude Code–specific configuration and examples |
| Claude Code: Agentic coding best practices | Recommended workflows, permissions, and safety |
| Equipping agents for the real world with Skills | Conceptual foundations and evaluation-first approach |
| Writing effective tools for AI agents | Tool and evaluation design guidance |
| MCP: Code execution and efficient agents | Scaling, privacy, and token efficiency |
| anthropics/skills repository | Concrete, production-grade examples |

