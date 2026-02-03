# Table Formatting Design

This document describes the technical architecture and implementation approach for table formatting in tq, including column width calculation, terminal awareness, and alignment strategies.

## Overview

The table formatting system renders QueryResult data as ASCII/UTF-8 tables for human consumption. It integrates with two display contexts:

1. **Direct table output** (`src/format/table.rs`) - Used for one-shot queries and non-paged REPL output
2. **Pager display** (`src/commands/repl/pager.rs`) - Used for interactive result navigation in REPL

Both contexts share the same column width calculation philosophy but have independent implementations due to different rendering requirements.

## Architecture

```
QueryResult (columns, rows)
       │
       ├──► Direct Output Path (table.rs)
       │    └── select_visible_columns()
       │         └── calculate_column_width()
       │              └── render_table()
       │
       └──► Pager Path (pager.rs)
            └── TableData::from_query_result()
                 └── Per-column width calculation
                      └── Pager::render()
```

## Column Width Calculation

### Current Behavior (Pre-Sprint 32)

The `calculate_column_width()` function in `src/format/table.rs` already calculates widths based on actual content:

```rust
fn calculate_column_width(header: &str, values: &[String], max_sample: usize, max_width: Option<usize>) -> usize {
    let header_width = header.len();
    let max_value_width = values
        .iter()
        .take(max_sample)
        .map(|v| v.len())
        .max()
        .unwrap_or(0);

    let natural_width = std::cmp::max(header_width, max_value_width) + 2;

    match max_width {
        Some(max) => std::cmp::min(natural_width, max),
        None => natural_width,
    }
}
```

**Key observations:**
- Width is calculated from actual cell content, not schema type
- Header width is respected (minimum width)
- Maximum width can be capped (used by pager at 40 chars)
- Samples up to 100 rows for width calculation

### Sprint 32 Enhancement

The current implementation already uses content-based width calculation. However, the issue reported in GitHub #13 suggests the behavior may not be working as expected for certain queries like `SELECT * FROM DBC.Databases`.

**Analysis of Potential Issues:**

1. **No maximum cap in direct output**: The direct output path passes `None` for `max_width`, meaning columns can grow unbounded. A VARCHAR(64) column with actual content of 15 chars would correctly calculate width as 15, but if the header is longer, it would use header width.

2. **The real issue**: Looking at `select_visible_columns()`, the function correctly calculates content-based widths. The problem may be elsewhere in the pipeline, or user expectations differ from actual behavior.

**Hypothesis:** The column width calculation IS content-based, but there is no maximum cap applied in interactive mode. The pager caps at 40 chars (`MAX_COLUMN_WIDTH`), but direct table output has no cap.

### Solution Design

#### Phase 1: Add Maximum Column Width Cap

Add a global maximum column width constant for direct table output:

```rust
/// Maximum column width for table output (content chars, excluding padding)
const MAX_COLUMN_WIDTH: usize = 100;
```

Modify `calculate_column_width()` to always respect this cap:

```rust
fn calculate_column_width(header: &str, values: &[String], max_sample: usize, max_width: Option<usize>) -> usize {
    let header_width = header.len();
    let max_value_width = values
        .iter()
        .take(max_sample)
        .map(|v| v.len())
        .max()
        .unwrap_or(0);

    // Content width: max of header and data
    let content_width = std::cmp::max(header_width, max_value_width);

    // Apply maximum cap (default to MAX_COLUMN_WIDTH if none specified)
    let effective_max = max_width.unwrap_or(MAX_COLUMN_WIDTH);
    let capped_width = std::cmp::min(content_width, effective_max);

    // Add padding (2 chars: 1 space on each side)
    capped_width + 2
}
```

#### Phase 2: Verify Content-Based Calculation

The current implementation scans actual cell values. Key verification points:

1. **Values are converted to strings before width calculation** - This happens via `Value::display()`
2. **All rows are considered (up to max_sample)** - Currently samples 100 rows
3. **Header width is minimum floor** - Prevents columns narrower than their headers

#### Phase 3: Integration with Pager

The pager already implements content-based width with its own constants:

```rust
const MIN_COLUMN_WIDTH: usize = 8;
const MAX_COLUMN_WIDTH: usize = 40;
```

For consistency, consider aligning these constants or making them configurable.

## Terminal Width Awareness

### Detection

```rust
fn get_terminal_width() -> Option<usize> {
    if !std::io::stdout().is_terminal() {
        return None; // Batch mode: no width limit
    }
    terminal::size()
        .map(|(width, _)| width as usize)
        .ok()
        .or(Some(80)) // Fallback
}
```

### Column Visibility Selection

In `select_visible_columns()`:

1. Calculate width for each column based on content
2. Add columns left-to-right until terminal width is exhausted
3. Reserve space for truncation indicator `(+N cols)`
4. Show at least one column even if it overflows

```rust
// Reserve space for truncation indicator: "| (+n cols) |" = ~15 chars
let truncation_width = 15;

for (idx, name) in column_names.iter().enumerate() {
    let col_width = calculate_column_width(name, &values, 100, None);

    let new_width = used_width + left_border + col_width + separator_width;

    if new_width + truncation_width < term_width {
        visible.push(idx);
        widths.push(col_width);
        used_width = new_width;
    } else {
        break;
    }
}
```

## Alignment Strategy

Alignment is type-driven via `TeradataType::alignment()`:

| Type | Alignment | Rationale |
|------|-----------|-----------|
| Integer, BigInt, Decimal, Float | Right | Numbers align on decimal point |
| Boolean | Center | Binary values look balanced |
| Varchar, Char, Date, Time, etc. | Left | Text flows naturally left |

Implementation in rendering:

```rust
let formatted = match col.data_type.alignment() {
    Alignment::Right => format!(" {:>width$}", value, width = width - 2),
    Alignment::Center => format!(" {:^width$}", value, width = width - 2),
    Alignment::Left => format!(" {:width$}", value, width = width - 2),
};
```

## Value Truncation

When content exceeds column width, values are truncated with ellipsis:

```rust
let max_value_len = width.saturating_sub(2); // Account for padding
let truncated_value = if value.len() > max_value_len && max_value_len > 3 {
    format!("{}...", &value[..max_value_len.saturating_sub(3)])
} else if value.len() > max_value_len {
    value[..max_value_len].to_string()
} else {
    value.clone()
};
```

## NULL Representation

NULL values display as `[NULL]` with optional styling:

```rust
if options.use_color && truncated_value == "[NULL]" {
    // ANSI escape for dim/italic
    row.push_str(&format!("\x1b[2;3m{}\x1b[0m", formatted));
} else {
    row.push_str(&formatted);
}
```

## Performance Considerations

### Width Calculation Sampling

Current implementation samples up to 100 rows for width calculation. This is a deliberate tradeoff:

- **Pro**: Accurate width for most result sets
- **Con**: May miss wider values in rows 101+
- **Alternative**: Configurable sample size or streaming calculation

For large result sets (1000+ rows), consider:

1. **Early exit optimization**: Stop scanning if all columns hit maximum width
2. **Progressive refinement**: Adjust widths as rows stream (complex, may cause visual artifacts)
3. **Defer to full scan**: For results under a threshold (e.g., 10000 rows), scan all

### Suggested Optimization

```rust
fn calculate_column_width_optimized(
    header: &str,
    values: &[String],
    max_sample: usize,
    max_width: usize
) -> usize {
    let header_width = header.len().min(max_width);

    let max_value_width = values
        .iter()
        .take(max_sample)
        .scan(header_width, |best, v| {
            let len = v.len();
            if len > *best {
                *best = len;
            }
            // Early exit if we've hit the max
            if *best >= max_width {
                None
            } else {
                Some(*best)
            }
        })
        .last()
        .unwrap_or(header_width);

    max_value_width.min(max_width) + 2
}
```

## Testing Strategy

### Unit Tests

1. **Width calculation**: Verify content-based width for various data patterns
2. **Maximum cap**: Verify width never exceeds MAX_COLUMN_WIDTH + padding
3. **Header minimum**: Verify column width >= header length
4. **NULL handling**: Verify `[NULL]` is properly sized (6 chars)
5. **Numeric alignment**: Verify right-alignment for numeric columns
6. **Empty values**: Verify empty strings don't collapse column width

### Integration Tests

1. **Wide tables**: Verify column truncation indicator appears
2. **Narrow terminals**: Verify at least one column is shown
3. **Batch mode**: Verify all columns shown without truncation

### Manual Validation (Type 4 per testing/approach.md)

Table formatting is a Type 3 feature (terminal output) requiring:

- [ ] Visual inspection at 80 char terminal
- [ ] Visual inspection at 117 char terminal
- [ ] Visual inspection at 120 char terminal
- [ ] Alignment verification for mixed type columns
- [ ] Truncation ellipsis appearance

## Constants Reference

| Constant | Location | Value | Purpose |
|----------|----------|-------|---------|
| `MAX_COLUMN_WIDTH` | table.rs | 100 | Max column width for direct output |
| `MAX_COLUMN_WIDTH` | pager.rs | 40 | Max column width in pager |
| `MIN_COLUMN_WIDTH` | pager.rs | 8 | Min column width in pager |
| `INDICATOR_WIDTH` | pager.rs | 10 | Width for `(+N cols)` indicator |

## Implementation Status

The content-based column width calculation with maximum cap has been implemented in Sprint 32:

- `MAX_COLUMN_WIDTH = 100` constant added to `src/format/table.rs`
- `calculate_column_width()` now applies the cap even when `max_width` is `None`
- Comprehensive unit tests added for edge cases (NULL, empty strings, Unicode, numeric alignment, long strings)
- All 355 tests pass including new column width tests
