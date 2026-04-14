# Sprint 63 Planning: Pager Exit Snapshot

**Date:** 2026-04-14
**Type:** Feature Sprint
**Version:** v1.45.0

## Reality Check Summary

- Reviewed sprints: 60, 61, 62
- Patterns detected: None — healthy velocity, 100% delivery across all three sprints
- Decision: Feature Sprint
- Rationale: No stuck issues, no accumulating debt, no framework problems

## Objectives

1. **Pager Exit Snapshot**: When the user presses `q` to exit the pager, print the last visible pager view to the normal terminal so it persists on screen.

## Problem Statement

Currently, the pager uses crossterm's `EnterAlternateScreen` / `LeaveAlternateScreen`. When the user presses `q`, the alternate screen buffer is discarded and the original screen content is restored — the query results disappear entirely. The user loses all visual context of what they were looking at.

**Expected behavior:** After exiting the pager, the terminal should show exactly what the user last saw — a static table snapshot of the visible rows and columns at exit time, plus a summary footer matching the non-paged output style (hidden column names, row count, timing).

## Acceptance Criteria

- [ ] Pressing `q` or `Esc` in the pager prints a static table to stdout before returning to the REPL
- [ ] The static table matches the last pager viewport: same rows, same columns, same column offset
- [ ] Hidden columns are reported in a footer message: "N columns hidden: col1, col2, ..."
- [ ] The "Use --format csv or --format json to see all columns" hint is shown when columns are hidden
- [ ] Row count and timing are shown: "N row(s) in set (X.XXXs)"
- [ ] The static output uses no ANSI color codes (plain text, like non-paged output)
- [ ] The static output uses the same box-drawing border characters as the pager
- [ ] No visual artifacts or terminal state issues after pager exit
- [ ] Existing pager navigation and rendering behavior unchanged
- [ ] Unit tests for the snapshot rendering logic

## Scope

### In Scope
- New `render_exit_snapshot()` method on `Pager` that writes the current viewport as static text to stdout
- Modification to `Pager::run()` to call snapshot rendering after `LeaveAlternateScreen`
- Hidden columns footer message (matching `format/table.rs` style)
- Row count and timing footer
- Unit tests for snapshot output

### Out of Scope
- Changes to non-paged table output
- Changes to pager navigation or rendering
- Color output in the snapshot (keep it plain for copy-paste friendliness)
- Batch mode changes (pager is REPL-only)

## Technical Approach

### Key Change: `pager.rs`

1. Add `render_exit_snapshot(&self, writer: &mut impl Write)` method that:
   - Calculates visible columns/rows from current `col_offset`, `row_offset`, `page_size`
   - Renders borders, header, data rows using the same box-drawing characters but writing to a `Write` trait (not stdout with crossterm commands)
   - Appends hidden columns footer
   - Appends row count + timing footer

2. Modify `Pager::run()`:
   - After `LeaveAlternateScreen` and `disable_raw_mode()`, call `render_exit_snapshot(&mut io::stdout())`

3. Modify `display_with_pager()` signature or return the pager so the caller can access snapshot state (if needed).

### Key Insight
The pager's existing `render_*` methods use crossterm `execute!()` macros with cursor positioning and color commands. The snapshot method must NOT use these — it writes plain text with `\n` line endings (not `\r\n`) to a generic `Write` implementor.

## Dependencies
- None — self-contained change in `pager.rs`

## Document History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-04-14 | 1.0 | Sprint planning | Sprint Coordinator |
