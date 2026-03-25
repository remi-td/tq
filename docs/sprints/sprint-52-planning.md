# Sprint 52 Planning: Markdown Output, Comment Fix & Format Docs

**Sprint Start:** 2026-03-25
**Sprint Goals:** Close GitHub Issues #38, #39, #40

## Scope

### 1. Add Comment Column to Inspect/Describe Output (Issue #38)
**Current:** Columns section shows Column, Type, Nullable, Default — no Comment column
**Target:** Add Comment column to the columns section in inspect table output. Show full comment text (up to 255 chars). In REPL mode with narrow terminals, comments may be truncated by terminal width but batch mode shows all.

**Changes:**
- `src/commands/inspect.rs`: Add Comment column to table, JSON, and CSV output
- Comment data already available in `ColumnInfo.comment` from `query_helpers.rs`

### 2. Document --format Argument (Issue #39)
**Current:** `--format` is only implied in examples, not documented in `--help`
**Target:** Add proper documentation to all `--format` arg definitions in CLI

**Changes:**
- `src/cli.rs`: Update help text for all `--format` arguments across all commands
- Document accepted values, default behavior, and multi-payload behavior

### 3. Add Markdown Output Format (Issue #40)
**Current:** Three output formats: table, json, csv
**Target:** Add `markdown` (alias `md`) format that renders tabular data as GitHub-Flavored Markdown tables and multi-section outputs (inspect) as structured markdown reports

**Changes:**
- `src/cli.rs`: Add `Markdown` variant to `OutputFormat` enum
- `src/format/markdown.rs`: New markdown formatter module
- `src/format/mod.rs`: Wire markdown into dispatch
- `src/commands/inspect.rs`: Add markdown output path for inspect
- `src/commands/list.rs`: Add markdown output path for list
- Other commands using custom format dispatch

## Acceptance Criteria
- [ ] `tq inspect <obj> --format table` shows Comment column
- [ ] `tq inspect <obj> --format json` includes comment field per column
- [ ] `tq inspect <obj> --format csv` includes Comment column
- [ ] `tq <cmd> --help` documents --format with accepted values
- [ ] `tq query "..." --format markdown` produces GFM table
- [ ] `tq inspect <obj> --format md` produces structured markdown report
- [ ] All existing tests pass
- [ ] New unit tests for markdown formatter
- [ ] Zero clippy warnings
