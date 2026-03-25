# Sprint 54 Planning: Agent-Safe Mode & Richer Introspection

**Sprint Start:** 2026-03-25
**Sprint Goals:** Implement Issue #37 parts 3-4 (agent-safe mode + richer introspection)

## Scope

### 1. Agent-Safe Execution Mode
**Flag:** `--agent-safe` on query command (also via TQ_AGENT_SAFE env var)

**Behavior when enabled:**
- Read-only by default: block DDL (CREATE, DROP, ALTER, RENAME) and DML (INSERT, UPDATE, DELETE, MERGE)
- Allow only SELECT, SHOW, EXPLAIN, HELP statements
- Enforce `--max-rows` (default 10000) — fail if result would exceed
- Single-statement-only mode (reject multi-statement input)
- Optional `--allow-dml` flag to explicitly enable write operations

**Error handling:**
- Blocked statements produce structured JSON error with code `AGENT_SAFE_BLOCKED`
- Clear message explaining what was blocked and why

**Changes:**
- `src/cli.rs`: Add `--agent-safe`, `--max-rows`, `--allow-dml` flags to QueryArgs
- `src/commands/query.rs`: Add statement classification and enforcement
- `src/error.rs`: Add `AgentSafeBlocked` error variant

### 2. Richer Introspection JSON Output
**Enhance inspect --format json to include:**
- Column-level comments (already done in Sprint 52)
- Column defaults and nullability (already present)
- Dependencies section (upstream/downstream)
- Definition section for views/macros
- Section selection: `--sections object,columns,indexes` (optional)

**Changes:**
- `src/commands/inspect.rs`: Add dependencies and definition to JSON output
- `src/cli.rs`: Add optional `--sections` flag to InspectArgs

## Acceptance Criteria
- [ ] `--agent-safe` blocks DDL/DML statements with clear error
- [ ] `--agent-safe` allows SELECT, SHOW, EXPLAIN
- [ ] `--max-rows` enforced with structured error on exceed
- [ ] `--allow-dml` overrides read-only restriction
- [ ] Single-statement enforcement in agent-safe mode
- [ ] inspect JSON includes dependencies and definition sections
- [ ] All existing tests pass
- [ ] New unit tests for statement classification and agent-safe mode
- [ ] Zero clippy warnings
