# Sprint 53 Planning: Agent Mode - JSON Envelope & Structured Errors

**Sprint Start:** 2026-03-25
**Sprint Goals:** Implement Issue #37 parts 1-2 (stable JSON contract + structured errors)

## Scope

### 1. Stable JSON Envelope for All Commands
**Current:** `--format json` produces raw `[{...}]` arrays or inconsistent custom JSON
**Target:** All JSON output uses a consistent envelope:
```json
{
  "ok": true,
  "row_count": 10,
  "data": [...]
}
```

**Changes:**
- `src/format/json.rs`: Modify `write()` to produce `{"ok":true, "row_count": N, "data": [...]}`
- `src/format/json.rs`: Modify `write_with_metadata()` to add `"ok": true` and rename `"rows"` to `"data"`
- All custom command JSON outputs: add `"ok": true` prefix
- `src/commands/inspect.rs`, `sessions.rs`, `sysconfig.rs`, etc.

### 2. Structured Error Output
**Current:** Errors in JSON mode print plain text to stderr
**Target:** When `--format json`, errors output structured JSON to stdout:
```json
{
  "ok": false,
  "error": {
    "code": "PERMISSION_DENIED",
    "category": "authz",
    "retryable": false,
    "message": "...",
    "hint": "..."
  }
}
```

**Changes:**
- `src/error.rs`: Add `error_code()`, `error_category()`, `is_retryable()`, `hint()` methods
- `src/main.rs`: Detect JSON format in error handler, output structured JSON

## Error Code Mapping
- AUTH_FAILED → category: "auth"
- PERMISSION_DENIED → category: "authz"
- OBJECT_NOT_FOUND → category: "query"
- SQL_SYNTAX_ERROR → category: "query"
- QUERY_EXECUTION_FAILED → category: "query"
- INVALID_ARGUMENT → category: "config"
- CONNECTION_FAILED → category: "connection"
- CONNECTION_TIMEOUT → category: "connection"
- IO_ERROR → category: "io"
- INTERNAL_ERROR → category: "internal"

## Acceptance Criteria
- [ ] `tq query "SELECT 1" --format json` returns `{"ok": true, "row_count": 1, "data": [...]}`
- [ ] All commands produce consistent JSON envelope
- [ ] Errors with `--format json` produce structured JSON error
- [ ] Error codes are documented and consistent
- [ ] All existing tests pass (updated for new envelope format)
- [ ] New tests for envelope format and error classification
- [ ] Zero clippy warnings
