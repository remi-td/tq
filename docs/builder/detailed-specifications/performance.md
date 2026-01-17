# Performance Considerations

**Version:** 1.1.0
**Last Updated:** 2026-01-18
**Owner:** cli-ux-designer agent
**Status:** Active Specification

---

## Table of Contents

1. [Startup Performance](#111-startup-performance)
2. [Query Execution Performance](#112-query-execution-performance)
3. [Memory Usage](#113-memory-usage)
4. [Large Result Sets](#114-large-result-sets)
5. [Network Performance](#115-network-performance)
6. [Build Optimization](#116-build-optimization)
7. [Performance Monitoring](#117-performance-monitoring)

---

## 11.1 Startup Performance

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

## 11.2 Query Execution Performance

### 11.2.1 Connection Pooling (REPL Only)

**Batch Mode**: One-shot connections (no pooling)
**REPL Mode**: Maintain single persistent connection

**Configuration**:
```toml
[repl.connection]
idle_timeout = "5m"  # Disconnect after inactivity
ping_interval = "30s"  # Keep-alive ping
```

### 11.2.2 Result Streaming

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

### 11.2.3 Parallel Processing

Not applicable: Teradata connection is inherently sequential

## 11.3 Memory Usage

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

## 11.4 Large Result Sets

### 11.4.1 Streaming Output

```bash
# Efficiently export 10M rows
tq query --format csv "SELECT * FROM massive_table" > data.csv
```

**Implementation**: Write rows incrementally, no intermediate buffer

### 11.4.2 Client-Side Limits

```bash
# Prevent accidental large queries
tq query --limit 1000 "SELECT * FROM table"

# Override limit
tq query --limit -1 "SELECT * FROM table"  # unlimited
```

### 11.4.3 Server-Side Limits

Use Teradata's `TOP` clause:
```bash
tq query "SELECT TOP 1000 * FROM table"
```

## 11.5 Network Performance

### 11.5.1 Compression (Future)

```bash
# Enable result compression
tq --compress query "SELECT * FROM large_table"
```

### 11.5.2 Batching (Future)

For multiple queries:
```bash
tq query --file queries.sql --batch-size 10
```

## 11.6 Build Optimization

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

## 11.7 Performance Monitoring

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
