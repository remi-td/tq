# teradata-query Agent Skill

An [Agent Skill](https://agentskills.io/) that teaches AI coding agents how to
use `tq` -- the Teradata Query CLI.

## What This Skill Does

When equipped with this skill, an agent can:

- Install tq on Linux/macOS
- Configure database connections (profiles, password files, env vars)
- Run one-shot queries, batch SQL files, and parameterized reports
- Explore schemas (list, inspect, peek, sample)
- Monitor sessions, locks, skew, and query plans
- Export results to CSV/JSON
- Handle credentials securely (never inline passwords)

## Distribution

This directory is a **Claude Code plugin**. The canonical source lives in the
tq repository at `skills/teradata-query/` and is also published to the
`remi-td/teradata-skills` marketplace repo.

### Install as a Claude Code Plugin (recommended)

```bash
claude plugin add remi-td/teradata-skills --subdir skills/teradata-query
```

Or from the tq repo directly:

```bash
claude plugin add remi-td/tq --subdir skills/teradata-query
```

### Manual Installation

Copy the skill file into your agent's skill directory:

```bash
# Claude Code (project-level)
mkdir -p .claude/skills/teradata-query
cp <tq-repo>/skills/teradata-query/skills/teradata-query/SKILL.md .claude/skills/teradata-query/

# Claude Code (user-level, all projects)
mkdir -p ~/.claude/skills/teradata-query
cp <tq-repo>/skills/teradata-query/skills/teradata-query/SKILL.md ~/.claude/skills/teradata-query/

# Cross-client (any agent supporting the standard)
mkdir -p .agents/skills/teradata-query
cp <tq-repo>/skills/teradata-query/skills/teradata-query/SKILL.md .agents/skills/teradata-query/
```

**Download from GitHub:**

```bash
curl -sSL https://raw.githubusercontent.com/remi-td/tq/master/skills/teradata-query/skills/teradata-query/SKILL.md \
  -o SKILL.md
```

### Keeping the Skill Updated

When installed as a plugin, run `/plugin update teradata-query` to pull the
latest version.

For manual installs: re-download or re-copy the SKILL.md file from the latest
release. The skill is versioned with the tq repository.

## Maintenance

The skill is maintained as part of the tq development workflow. When new
features are added to tq:

1. Update `skills/teradata-query/SKILL.md` to document the new capability
2. Add an eval case to `evals/evals.json` if the feature introduces a new
   interaction pattern
3. The skill update ships with the same release as the feature

## Testing (Evals)

The `evals/` directory contains evaluation test cases for validating skill
effectiveness. These follow the [Agent Skills eval pattern](https://agentskills.io/).

### How Evals Work

Each eval in `evals/evals.json` defines:

- **prompt**: A realistic user request
- **context**: The starting state (what's installed, what's configured)
- **assertions**: Specific, verifiable claims about what the agent should do

### Running Evals

Evals are designed for A/B comparison:

1. **With skill**: Run each prompt with the skill loaded. Record the agent's
   tool calls, commands, and responses.
2. **Without skill**: Run the same prompt without the skill. Record the same.
3. **Grade**: For each assertion, mark PASS/FAIL with evidence (exact command
   used, output snippet, etc.).
4. **Compare**: The skill should improve pass rates, especially for:
   - Credential security (never inline passwords)
   - Using tq commands vs raw SQL (e.g., `tq inspect` vs `SELECT FROM DBC.ColumnsV`)
   - Environment confirmation before running against non-dev targets
   - Correct flag usage (`--file`, `--atomic`, `-p`)

### Eval Categories

| Category | Tests | Key Differentiator |
|----------|-------|--------------------|
| installation | Correct installer usage | `--accept-license` flag |
| connection-setup | Secure profile creation | Password files, not inline |
| simple-query | Basic tq query usage | One-shot vs REPL choice |
| file-execution | SQL file execution | `--file`, `--atomic`, env confirmation |
| schema-exploration | Schema discovery | tq commands vs raw DBC queries |
| data-export | Result export | `--format csv`, `--output` |
| monitoring | System diagnosis | Dedicated monitoring commands |
| parameterized-query | Variable substitution | `-p` params flag |
| explain-plan | Query optimization | `tq explain` command |
| credential-security | Secure handling | Password file enforcement |

### Metrics to Track

- **Assertion pass rate**: % of assertions passing (target: >90% with skill)
- **Skill delta**: Pass rate with skill minus pass rate without skill
- **Security compliance**: 100% on credential-security assertions
- **Command coverage**: Agent uses tq-native commands vs raw SQL alternatives
