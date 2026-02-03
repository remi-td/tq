# TC-033-001: Pager Disabled by Default

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-033-001 |
| **Title** | Pager Disabled by Default |
| **Category** | Unit Test |
| **Priority** | Critical |
| **Feature** | Sprint 33 - Pager Bug Fix (AC-3) |
| **Test Type** | Unit |
| **Created** | 2026-02-03 |

## Purpose

Verify that the interactive pager is disabled by default in the REPL to protect users from broken rendering behavior reported in Issue #14.

## Acceptance Criteria Coverage

- **AC-3**: Default disabled - `pager_enabled: false` in `src/commands/repl/state.rs` regardless of fix status

## Scope

This test validates:
- ReplState::default() initializes with pager_enabled = false
- New REPL sessions start with pager disabled
- User experience is protected from known pager rendering bugs

## Prerequisites

- Rust test framework available
- Access to REPL state module in `src/commands/repl/state.rs`

## Test Procedure

### Test Implementation (in `src/commands/repl/state.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pager_disabled_by_default() {
        // Create new ReplState with default configuration
        let state = ReplState::default();

        // Assert: Pager should be disabled by default
        assert_eq!(state.pager_enabled, false,
                   "Pager must be disabled by default to protect users from Issue #14 rendering bugs");
    }
}
```

## Expected Results

- Test passes with pager_enabled = false
- Default ReplState protects users from broken pager

## Pass/Fail Criteria

**PASS if:**
- Test compiles and passes
- ReplState::default() has pager_enabled = false
- No panics or errors

**FAIL if:**
- Test fails (pager_enabled = true)
- pager_enabled field doesn't exist
- Test panics or errors

## Notes

- This is a UNIT test - no database or PTY required
- Validates AC-3 from Sprint 33 planning
- Protects users from Issue #14 pager rendering bugs
- User can still enable pager with `/pager on` if desired
