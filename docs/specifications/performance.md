# Performance Considerations

## Table of Contents

1. [Startup Performance](#startup-performance)
2. [Query Execution Performance](#query-execution-performance)
3. [Memory Usage](#memory-usage)
4. [Large Result Sets](#large-result-sets)
5. [Network Performance](#network-performance)
6. [Build Optimization](#build-optimization)
7. [Performance Monitoring](#performance-monitoring)

---

## Startup Performance

**Target**: < 100ms cold start

**Strategies**:
1. **Minimal dependencies**: Avoid heavy crates
2. **Lazy initialization**: Don't load config if not needed
3. **Static binary**: No dynamic library loading
4. **Optimized build**: LTO, single codegen unit

**Measurement**:
```bash
time tq --version  # Should be < 50ms
time tq --help     # Should be < 100ms
```

## Query Execution Performance

### Connection Pooling

**Batch Mode**: One-shot connections (no pooling)
**REPL Mode**: Maintain single persistent connection

### Result Streaming

**Implementation**: Use iterators/streams to avoid buffering entire result set

```rust
// Good: Stream rows as they arrive
for row in query.execute()? {
    output.write_row(row)?;
}

// Bad: Buffer all rows in memory
let rows = query.execute()?.collect::<Vec<_>>();
output.write_rows(&rows)?;
```

**Benefits**:
- Constant memory usage
- Faster time-to-first-byte
- Handles result sets larger than RAM

## Memory Usage

**Targets**:
- Idle: < 10 MB
- Small query (< 1000 rows): < 20 MB
- Large query (streaming): < 50 MB (constant)

**Strategies**:
1. **Streaming results**: Don't buffer
2. **Efficient data structures**: Avoid clones
3. **Drop early**: Release connections ASAP

**Profiling**:
```bash
# Check memory usage
/usr/bin/time -v tq query "SELECT * FROM huge_table" > /dev/null
```

## Large Result Sets

### Streaming Output

```bash
# Efficiently export 10M rows
tq query --format csv "SELECT * FROM massive_table" > data.csv
```

**Implementation**: Write rows incrementally, no intermediate buffer

### Client-Side Limits

```bash
# Prevent accidental large queries
tq query --limit 1000 "SELECT * FROM table"

# Override limit
tq query --limit -1 "SELECT * FROM table"  # unlimited
```

### Server-Side Limits

Use Teradata's `TOP` clause:
```bash
tq query "SELECT TOP 1000 * FROM table"
```

## Network Performance

Compression and batching features are planned for future releases.

## Build Optimization

**Cargo Profile** (`Cargo.toml`):
```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = "fat"            # Full LTO
codegen-units = 1      # Single codegen unit
strip = "symbols"      # Strip debug symbols
panic = "abort"        # No unwinding
```

**Target Size**:
- Linux (musl): < 5 MB
- macOS: < 4 MB
- Windows: < 5 MB

## Performance Monitoring

```bash
# Query timing
tq query --timing "SELECT COUNT(*) FROM large_table"
# Output: (Executed in 2.345s)

# Verbose timing breakdown
tq -v query "SELECT 1"
# [DEBUG] Connection: 127ms
# [DEBUG] Query: 15ms
# [DEBUG] Fetch: 3ms
# [DEBUG] Total: 145ms
```

---
