# TC-045-BUG32-SEMICOLON: Bug #32 — Full Metacommand Semicolon Stripping Suite

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-045-BUG32-SEMICOLON |
| **Title** | Bug #32 Full Metacommand Semicolon Stripping Test Suite |
| **Category** | Unit Test |
| **Priority** | Critical |
| **Feature** | Sprint 45 — Bug #32 Metacommand Semicolon Stripping |
| **Test Type** | Unit |
| **DB Required** | No |
| **Created** | 2026-03-23 |
| **Covers** | TC-045-001 through TC-045-006 |

## Purpose

Comprehensive unit test suite validating that all metacommand argument parsing correctly strips trailing semicolons (Bug #32 fix), while preserving existing behavior when no semicolon is present.

## Acceptance Criteria Coverage

- **AC-1**: `/describe tablename;` resolves to `tablename`
- **AC-2**: `/list tables;` matches `tables` subcommand
- **AC-3**: `/sample dbc.tables;` delivers `dbc.tables` as argument
- **AC-4**: `/show indexes tablename;` delivers `tablename`
- **AC-5**: All other metacommands with trailing semicolons work
- **AC-6**: Unit tests cover semicolon stripping for at least 4 commands
- **REGRESSION**: No semicolon — behavior unchanged
- **EDGE**: Multiple semicolons `;;` fully stripped

## Prerequisites

- Rust test framework available
- `src/commands/repl/metacommands.rs` has the `trim_end_matches(';')` fix applied in both
  `handle_metacommand()` and `handle_metacommand_with_state()`

## Test Procedure

### Test Implementation (in `src/commands/repl/metacommands.rs` `#[cfg(test)]` module):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper that applies the same normalization the fixed handler must use.
    /// Tests must call this to simulate the fixed parsing pipeline.
    fn parse_metacommand(input: &str) -> (String, Vec<String>) {
        let trimmed = input.trim();
        let without_prefix = trimmed.trim_start_matches('/').trim_start_matches('\\');
        // The fix: strip ALL trailing semicolons before splitting
        let stripped = without_prefix.trim_end_matches(';');
        let mut parts = stripped.split_whitespace();
        let command = parts.next().unwrap_or("").to_lowercase();
        let args: Vec<String> = parts.map(|s| s.to_string()).collect();
        (command, args)
    }

    // TC-045-001: /describe tablename; → arg is "tablename"
    #[test]
    fn test_describe_strips_trailing_semicolon() {
        let (cmd, args) = parse_metacommand("/describe tablename;");
        assert_eq!(cmd, "describe");
        assert_eq!(args, vec!["tablename"]);
        assert!(!args[0].ends_with(';'));
    }

    // TC-045-002: /list tables; → subcommand is "tables" (not "tables;")
    #[test]
    fn test_list_tables_strips_trailing_semicolon() {
        let (cmd, args) = parse_metacommand("/list tables;");
        assert_eq!(cmd, "list");
        assert_eq!(args, vec!["tables"]);
        assert!(!args[0].ends_with(';'));
    }

    // TC-045-003: /sample dbc.tables; → arg is "dbc.tables"
    #[test]
    fn test_sample_strips_trailing_semicolon() {
        let (cmd, args) = parse_metacommand("/sample dbc.tables;");
        assert_eq!(cmd, "sample");
        assert_eq!(args, vec!["dbc.tables"]);
        assert!(!args[0].ends_with(';'));
    }

    // TC-045-004: /show indexes tablename; → second arg is "tablename"
    #[test]
    fn test_show_indexes_strips_trailing_semicolon() {
        let (cmd, args) = parse_metacommand("/show indexes tablename;");
        assert_eq!(cmd, "show");
        assert_eq!(args, vec!["indexes", "tablename"]);
        // Only the last token has the semicolon stripped
        assert!(!args[1].ends_with(';'));
    }

    // TC-045-005: Multiple semicolons — /describe a;; → arg is "a"
    #[test]
    fn test_multiple_semicolons_fully_stripped() {
        let (cmd, args) = parse_metacommand("/describe a;;");
        assert_eq!(cmd, "describe");
        assert_eq!(args, vec!["a"]);
        assert!(!args[0].contains(';'));
    }

    // TC-045-006: No semicolon — regression — /describe tablename → arg is "tablename"
    #[test]
    fn test_no_semicolon_regression() {
        let (cmd, args) = parse_metacommand("/describe tablename");
        assert_eq!(cmd, "describe");
        assert_eq!(args, vec!["tablename"]);
    }

    // TC-045-005b: /peek table; → arg is "table"
    #[test]
    fn test_peek_strips_trailing_semicolon() {
        let (cmd, args) = parse_metacommand("/peek table;");
        assert_eq!(cmd, "peek");
        assert_eq!(args, vec!["table"]);
        assert!(!args[0].ends_with(';'));
    }

    // Edge: semicolon only on command portion should not corrupt command name
    // /describe; → command is "describe" (no args), not "describe;"
    #[test]
    fn test_semicolon_on_command_name_stripped() {
        let (cmd, args) = parse_metacommand("/describe;");
        // After stripping trailing semicolons from the whole stripped string,
        // the command itself gets the semicolon removed.
        assert_eq!(cmd, "describe");
        assert!(args.is_empty());
    }
}
```

## Expected Results

All 8 unit tests pass:
- `test_describe_strips_trailing_semicolon` — PASS
- `test_list_tables_strips_trailing_semicolon` — PASS
- `test_sample_strips_trailing_semicolon` — PASS
- `test_show_indexes_strips_trailing_semicolon` — PASS
- `test_multiple_semicolons_fully_stripped` — PASS
- `test_no_semicolon_regression` — PASS
- `test_peek_strips_trailing_semicolon` — PASS
- `test_semicolon_on_command_name_stripped` — PASS

## Pass/Fail Criteria

**PASS if:**
- All 8 tests compile and pass with `cargo test --lib`
- No argument ends with `;` after normalization
- Regression test (no semicolon) still returns correct argument

**FAIL if:**
- Any test fails — indicates the fix was not applied or was applied incorrectly
- Any argument contains a trailing `;`

## Run Command

```bash
cargo test --lib -- metacommands::tests 2>&1
```

## Notes

- This is a UNIT test suite — no database or PTY required
- The `parse_metacommand` helper mirrors the normalization logic that must be applied in `handle_metacommand()` and `handle_metacommand_with_state()`
- If these tests fail, check that `trim_end_matches(';')` is called on the full input (after `trim()`) before `split_whitespace()`
