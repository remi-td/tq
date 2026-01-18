# Sprint 7 Test Execution - Issues Log

**Date**: 2026-01-18
**Commit**: 2b8320de20b610ef14bd2dc721d2e546c1d785b3

## Issue #1: Dead Code Warnings (5 warnings)

**Severity**: Minor
**Priority**: Medium
**Category**: Code Quality
**Status**: Open

### Description

The release build produces 5 dead code warnings for unused functions. These functions are present in the codebase but not currently called, indicating either:
1. Incomplete feature implementation
2. Leftover code from refactoring
3. Functions intended for future use

### Affected Functions

1. **`write_enhanced_timing`** in `src/commands/repl/executor.rs:302`
   - Purpose: Appears to be for enhanced timing display
   - Action: Either use it or remove it

2. **`cache_mut`** method in `src/commands/repl/metadata_completer.rs:46`
   - Purpose: Provides mutable access to metadata cache
   - Action: Remove if not needed, or use for cache management features

3. **`clear_cache`** method in `src/commands/repl/metadata_completer.rs:56`
   - Purpose: Clears metadata cache
   - Action: This seems like it should be used! Check if /logon properly clears cache via this method

4. **`display_with_paging`** in `src/commands/repl/pager.rs:281`
   - Purpose: Display results with paging support
   - Action: Integrate with query execution or remove

5. **`interactive_pager`** in `src/commands/repl/pager.rs:299`
   - Purpose: Interactive paging implementation
   - Action: Either complete pager feature or remove

6. **`should_page`** in `src/commands/repl/pager.rs:365`
   - Purpose: Determine if results should be paged
   - Action: Use in query executor or remove

### Build Output

```
warning: function `write_enhanced_timing` is never used
   --> src/commands/repl/executor.rs:302:8
    |
302 | pub fn write_enhanced_timing<W: Write>(
    |        ^^^^^^^^^^^^^^^^^^^^^

warning: methods `cache_mut` and `clear_cache` are never used
  --> src/commands/repl/metadata_completer.rs:46:12
   |
31 | impl CompletionState {
   | -------------------- methods in this implementation
...
46 |     pub fn cache_mut(&mut self) -> &mut MetadataCache {
   |            ^^^^^^^^^
...
56 |     pub fn clear_cache(&mut self) {
   |            ^^^^^^^^^^^

warning: function `display_with_paging` is never used
   --> src/commands/repl/pager.rs:281:8
    |
281 | pub fn display_with_paging<W: Write>(
    |        ^^^^^^^^^^^^^^^^^^^

warning: function `interactive_pager` is never used
   --> src/commands/repl/pager.rs:299:4
    |
299 | fn interactive_pager<W: Write>(mut paged: PagedOutput, writer: &mut W) -> Result<()> {
    |    ^^^^^^^^^^^^^^^^^

warning: function `should_page` is never used
   --> src/commands/repl/pager.rs:365:8
    |
365 | pub fn should_page(result: &QueryResult, config: &PagerConfig) -> bool {
    |        ^^^^^^^^^^^
```

### Impact

- **Functional**: None (functions are simply not called)
- **Code Quality**: Indicates incomplete implementation or technical debt
- **Build**: Produces warnings, violates "zero warnings" target

### Recommendations

**Option A: Complete Features**
- Integrate pager functions into query execution
- Use enhanced timing display in REPL
- Properly wire up cache management methods

**Option B: Remove Unused Code**
- Delete functions if features are deferred
- Add #[allow(dead_code)] with explanation if intentionally kept

**Estimated Effort**: 1 hour to investigate and either integrate or remove

### Resolution Plan

1. Review each function to determine intent
2. Check git history to understand why they were added
3. Either:
   - Complete the feature integration, OR
   - Remove the dead code
4. Achieve zero-warning build

---

## Issue #2: /logon Without Arguments Behavior Discrepancy

**Severity**: Minor
**Priority**: Low
**Category**: Feature Behavior
**Status**: Open - Needs Clarification

### Description

According to Sprint 7 planning document (line 80):

> "/logon with no args shows current connection info"

However, the actual implementation in `src/commands/repl/metacommands.rs:286-296` shows usage help instead:

```rust
"logon" => {
    if args.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "Usage: /logon <connection_string>")?;
        writeln!(writer)?;
        writeln!(writer, "Format: user:password@host:port/database")?;
        // ... more usage help ...
```

### Discussion

The `/session` metacommand already provides current connection information, so showing usage help for `/logon` without args may be intentional design to guide users.

However, this differs from the acceptance criteria which explicitly states showing connection info.

### Impact

- **Functional**: Low - Users can use `/session` for connection info
- **Consistency**: Differs from documented acceptance criteria
- **UX**: Showing usage help may be more helpful than duplicating `/session` output

### Recommendations

**Option A: Update Code**
- Make `/logon` without args show connection info as specified
- Keeps behavior consistent with acceptance criteria

**Option B: Update Specification**
- Update acceptance criteria to reflect current behavior (shows usage)
- Document that `/session` is the command for viewing connection info
- Rationale: Avoids duplicating functionality

**Option C: Hybrid Approach**
- Show brief connection info + usage help
- Example: "Currently connected to X. Use /logon <connection-string> to switch."

### Resolution Plan

1. Consult with product owner / user on preferred behavior
2. Update either code or specification to match decision
3. Document the rationale

**Estimated Effort**: 30 minutes discussion + 30 minutes implementation

---

## Summary

- **Total Issues**: 2
- **Critical**: 0
- **Major**: 0
- **Minor**: 2

**Blockers to Sprint Closure**: Issue #1 (dead code warnings) should be resolved to meet "zero technical debt" success criterion.

**Non-Blockers**: Issue #2 is a minor clarification that can be addressed post-sprint if needed.
