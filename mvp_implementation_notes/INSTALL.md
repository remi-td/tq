# Installation Guide for tq

## Quick Start

**Good news!** The Teradata driver libraries are automatically bundled when you build `tq`. No manual driver installation required!

### Simple Installation

```bash
# Clone and build
git clone <repository-url>
cd tq
cargo build --release

# Test the installation
cargo run -- --logon "demo_user:demo_user@mcp-vikzqtnd0db0nglk.env.clearscape.teradata.com:1025/demo_user" --ping
```

That's it! The build process automatically:
1. Locates the Teradata native libraries from the `teradatarustapi` dependency
2. Copies the appropriate library for your platform to the build directory
3. Embeds the library path into the binary

## How It Works

When you run `cargo build`, a build script (`build.rs`) automatically:
- Finds the `teradatarustapi` git checkout in your cargo cache
- Identifies your platform (macOS, Linux, or Windows)
- Copies the correct native library file to `target/debug/` or `target/release/`
- Configures the binary to find the library automatically

**Platform-specific libraries**:
- **macOS**: `teradatasql.dylib`
- **Linux**: `teradatasql.so` (with architecture variants)
- **Windows**: `teradatasql.dll`

No additional downloads or system-wide installations needed!

## Building tq

### From Source

```bash
# Clone the repository
git clone <repository-url>
cd tq

# Build the project
cargo build --release

# The executable will be in target/release/tq
```

### Using Cargo Install (once published)

```bash
cargo install tq
```

## Testing the Installation

Test your connection to the demo Teradata database:

```bash
tq --logon "demo_user:demo_user@mcp-vikzqtnd0db0nglk.env.clearscape.teradata.com:1025/demo_user" --ping
```

Expected output:
```
Pinging Teradata database at mcp-vikzqtnd0db0nglk.env.clearscape.teradata.com:1025...
Success! Database is reachable.
  Host: mcp-vikzqtnd0db0nglk.env.clearscape.teradata.com
  Port: 1025
  User: demo_user
  Database: demo_user
  Logon Mechanism: TD2
```

## Troubleshooting

### Error: "Could not load library"

This means the Teradata GoSQL driver library cannot be found. Solutions:

1. Ensure the driver library is in the current directory
2. Use `--driver-lib-dir` to specify the driver location
3. Check that you downloaded the correct version for your platform
4. Verify file permissions allow reading the library

### Error: "Connection failed"

This means the driver loaded successfully but cannot connect to the database:

1. Verify the connection string format: `user:password@host:port/database`
2. Check network connectivity to the Teradata host
3. Confirm credentials are correct
4. Ensure the Teradata database is running and accessible

### Platform-Specific Notes

#### macOS
- The driver file should be named `teradatasql.dylib`
- You may need to allow the library in System Preferences > Security & Privacy

#### Linux
- The driver file should be named `teradatasql.so`
- Ensure the library has execute permissions: `chmod +x teradatasql.so`

#### Windows
- The driver file should be named `teradatasql.dll`
- The Visual C++ Redistributable may be required

## Support

For issues related to:
- **tq tool**: Open an issue in this repository
- **Teradata GoSQL Driver**: Contact Teradata support or visit [Teradata Developer Portal](https://downloads.teradata.com/)
