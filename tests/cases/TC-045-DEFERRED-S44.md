# TC-045-DEFERRED-S44: Sprint 44 Deferred Items Validation

## Metadata

| Field | Value |
|-------|-------|
| **Test ID** | TC-045-DEFERRED-S44 |
| **Title** | Sprint 44 Deferred Items — --force Help Text, Abort Message, Debug Logging, Doc Drift |
| **Category** | Unit Test + Code Inspection |
| **Priority** | Medium |
| **Feature** | Sprint 45 — Sprint 44 Deferred Items |
| **Test Type** | Unit + Manual Code Inspection |
| **DB Required** | No |
| **Created** | 2026-03-23 |
| **Covers** | TC-045-020 through TC-045-023 |

## Purpose

Validate the three Sprint 44 deferred fixes:
- AC-2: `--force` flag description reads "Skip confirmation prompt"
- AC-3: Abort message includes profile name in single quotes
- AC-4: `log::debug!` calls present at each fallback step in `resolve_driver_lib_dir`
- AC-1: `docs/design/connection-management.md` function signature matches actual code (code inspection only — no automated test)

## Acceptance Criteria Coverage

- **AC-1**: Design doc matches `resolve_driver_lib_dir` signature (code inspection)
- **AC-2**: `--force` description is "Skip confirmation prompt"
- **AC-3**: Abort message includes profile name
- **AC-4**: Debug logging present in `resolve_driver_lib_dir`

## Prerequisites

- Rust test framework available
- `src/cli.rs` updated with new `--force` help text
- `src/commands/profile.rs` updated with abort message containing profile name
- `src/db/client.rs` updated with `log::debug!` calls

## Test Procedure

### TC-045-021 and TC-045-022: Unit Tests

**Implementation (in `src/commands/profile.rs` `#[cfg(test)]` module or `src/cli.rs::tests`):**

```rust
// In src/commands/profile.rs #[cfg(test)]

#[cfg(test)]
mod tests {
    use super::*;

    // TC-045-022: Abort message includes profile name
    #[test]
    fn test_abort_message_includes_profile_name() {
        // The abort message for profile deletion must include the profile name
        // in single quotes, e.g. "Aborted. Profile 'myprofile' was not deleted."
        let profile_name = "myprofile";
        let msg = format_delete_abort_message(profile_name);
        assert!(msg.contains(profile_name),
            "Abort message must contain profile name '{}', got: {}", profile_name, msg);
        assert!(msg.contains('\''),
            "Abort message must wrap profile name in single quotes, got: {}", msg);
        // Verify it matches the expected pattern
        assert!(msg.contains(&format!("'{}'", profile_name)),
            "Abort message must contain '{}' in single quotes, got: {}", profile_name, msg);
    }

    // TC-045-022b: Abort message with a different profile name
    #[test]
    fn test_abort_message_profile_name_interpolated() {
        let profile_name = "production-db";
        let msg = format_delete_abort_message(profile_name);
        assert!(msg.contains("production-db"),
            "Abort message must contain profile name, got: {}", msg);
    }
}
```

**Alternative — test via CLI help text parsing (TC-045-021):**

```rust
// In src/cli.rs #[cfg(test)]

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    // TC-045-021: --force help text verification
    #[test]
    fn test_force_flag_help_text() {
        // Build the CLI command structure and find the --force argument
        // This test verifies the help text string in the Clap definition
        let cli = Cli::command();
        // Find the 'profile' subcommand → 'delete' subcommand → '--force' argument
        let profile_cmd = cli
            .find_subcommand("profile")
            .expect("profile subcommand must exist");
        let delete_cmd = profile_cmd
            .find_subcommand("delete")
            .expect("profile delete subcommand must exist");
        let force_arg = delete_cmd
            .get_arguments()
            .find(|a| a.get_id() == "force")
            .expect("--force argument must exist on profile delete");

        let help_text = force_arg
            .get_help()
            .map(|h| h.to_string())
            .unwrap_or_default();

        assert!(help_text.to_lowercase().contains("skip"),
            "--force help must contain 'skip', got: {}", help_text);
        assert!(help_text.to_lowercase().contains("confirmation") ||
                help_text.to_lowercase().contains("prompt"),
            "--force help must mention 'confirmation' or 'prompt', got: {}", help_text);
    }
}
```

### TC-045-023: Debug Logging Verification (RUST_LOG=debug)

This test is executed manually or via a shell command:

```bash
# Build the binary first
cargo build 2>&1

# Attempt a connection with an invalid driver path to trigger fallback logging
# The debug log lines should appear on stderr with RUST_LOG=debug
RUST_LOG=debug ./target/debug/tq --logon "dummy:pass@127.0.0.1:1025" ping 2>&1 | grep -i "debug"
```

Expected: At least one `DEBUG` log line appears from `resolve_driver_lib_dir` showing a fallback step being attempted.

Alternatively, verify via code inspection:

```bash
grep -n "log::debug!" src/db/client.rs
```

Expected: At least 3 lines match, each corresponding to a different fallback step (exe dir, env var, cwd, etc.).

### TC-045-020: Design Doc Accuracy (Manual Code Inspection)

**Procedure:**

1. Open `docs/design/connection-management.md`
2. Find the section describing `resolve_driver_lib_dir`
3. Open `src/db/client.rs`
4. Find the actual `resolve_driver_lib_dir` function signature
5. Verify: parameter names, return type, and fallback step descriptions in the doc match the actual code

**Pass criteria:** The documented function signature (parameters and return type) matches the actual Rust function. The fallback chain described in the doc matches the order of steps in the code.

**Fail criteria:** Documented signature lists parameters that do not exist, or omits parameters that do exist.

## Expected Results

- TC-045-021: `--force` help text contains "skip" and either "confirmation" or "prompt"
- TC-045-022: Abort message for "myprofile" contains `'myprofile'`
- TC-045-023: At least 3 `log::debug!` calls present in `resolve_driver_lib_dir`
- TC-045-020: Documented signature matches actual code (manual verification)

## Pass/Fail Criteria

**PASS if:**
- Unit tests compile and pass
- Manual inspection confirms `log::debug!` calls present
- Design doc signature matches code

**FAIL if:**
- `--force` help text does not contain "skip" and "confirmation/prompt"
- Abort message does not contain profile name in single quotes
- No `log::debug!` calls found in `resolve_driver_lib_dir`

## Run Command

```bash
# Unit tests
cargo test --lib -- profile::tests cli::tests 2>&1

# Debug log inspection
grep -n "log::debug!" src/db/client.rs
```

## Notes

- TC-045-020 (doc drift) is manual only — no automated test can verify prose accuracy
- TC-045-023 may alternatively be implemented as a unit test that verifies the function body contains the expected log calls via a compile-time assertion pattern, but code inspection is sufficient given the low risk
- If `format_delete_abort_message` is not a standalone function but instead an inline `writeln!` in the `profile.rs` delete handler, adapt the test to capture the handler's output buffer and assert the message
