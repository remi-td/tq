# Teradata Rust Troubleshooting

## Common Pitfalls

1. **Forgetting to load driver** - Call `load_driver()` before connections
2. **Not closing connections** - Always close to prevent leaks
3. **Incorrect JSON format** - Connection params must be valid JSON
4. **Missing native library** - Ensure build script copies the library
5. **Wrong parameter format** - Bind values must be JSON arrays
6. **Not handling connection errors** - Network issues are common

## Error Message Enhancement

```rust
fn parse_connection_error(error: &str) -> String {
    if error.contains("Connection refused") {
        format!("Connection refused. Ensure database is running.")
    } else if error.contains("timeout") {
        format!("Connection timeout. Check network connectivity.")
    } else if error.contains("Logon failed") {
        format!("Authentication failed. Verify credentials.")
    } else {
        format!("Connection failed: {}", error)
    }
}
```

## Debugging Library Loading

```bash
# Check if library is present
ls -la target/debug/teradatasql.*

# macOS: debug library loading
DYLD_PRINT_LIBRARIES=1 ./target/debug/your-app

# Linux: debug library loading
LD_DEBUG=libs ./target/debug/your-app
```

## Security Best Practices

1. **Never log connection strings** - they contain passwords
2. **Use environment variables** for credentials:
   ```rust
   let logon = env::var("TD_LOGON").expect("TD_LOGON not set");
   ```
3. **Use parameterized queries** to prevent SQL injection
4. **Close connections** to avoid resource leaks
5. **Don't commit credentials** to version control

## Performance Tips

**For CLI tools (one-shot):**
- Load driver once per process (use `once_cell`)
- Create connection, execute, close (no pooling)
- Close connections immediately after use

**For long-running apps:**
- Implement connection pooling manually
- Reuse connections for multiple queries
- Monitor connection health
