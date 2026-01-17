# Automatic Library Bundling

## Overview

The `tq` tool automatically bundles the Teradata native libraries from the `teradatarustapi` crate, so **you don't need to manually download or install any drivers**.

## How It Works

When you build `tq`, a build script (`build.rs`) automatically:

1. Locates the `teradatarustapi` git checkout in your cargo cache (`~/.cargo/git/checkouts/`)
2. Identifies your platform (macOS, Linux, or Windows)
3. Copies the appropriate native library to the build target directory:
   - **macOS**: `teradatasql.dylib`
   - **Linux**: `teradatasql.so` (with architecture-specific variants)
   - **Windows**: `teradatasql.dll`
4. Embeds the library path into the binary at compile time

## No Manual Setup Required

Unlike many database clients, you don't need to:
- Download separate driver packages
- Install system-wide libraries
- Configure environment variables
- Manage library paths manually

Just build and run:

```bash
cargo build
cargo run -- --logon "user:pass@host:1025/db" --ping
```

## For Advanced Users

If you need to use a custom driver library (e.g., a FIPS-compliant version), you can override the default:

```bash
tq --logon "user:pass@host:1025/db" --ping --driver-lib-dir /path/to/custom/driver
```

## Platform-Specific Libraries

The `teradatarustapi` crate includes these platform-specific libraries:

### macOS
- `teradatasql.dylib` - Universal binary

### Linux
- `teradatasql.so` - Standard x86_64
- `teradatasql.arm.so` - ARM architecture
- `teradatasql.arm.fips.so` - ARM with FIPS compliance
- `teradatasql.fips.so` - x86_64 with FIPS compliance
- `teradatasql.power.so` - IBM Power architecture
- `teradatasql.x86.so` - x86 32-bit

### Windows
- `teradatasql.dll` - x86_64
- `teradatasql.x86.dll` - x86 32-bit

## Build Script Details

The `build.rs` script:

```rust
// Searches for teradatarustapi in cargo cache
let cargo_home = env::var("CARGO_HOME").unwrap_or_else(|_| format!("{}/.cargo", home));
let git_checkouts = PathBuf::from(cargo_home).join("git/checkouts");

// Finds the appropriate library
let lib_name = if cfg!(target_os = "macos") {
    "teradatasql.dylib"
} else if cfg!(target_os = "windows") {
    "teradatasql.dll"
} else {
    "teradatasql.so"
};

// Copies to target directory and embeds path
fs::copy(&lib_source, &lib_dest)?;
println!("cargo:rustc-env=TERADATA_LIB_DIR={}", target_dir.display());
```

## Distribution

When distributing the built binary:

1. **Development**: The library is in `target/debug/` or `target/release/`
2. **Distribution**: Package the binary together with the library:
   ```
   tq-package/
   ├── tq (executable)
   └── teradatasql.dylib (or .so/.dll)
   ```

3. **Install**: Place both files in the same directory, or use `--driver-lib-dir` to specify the library location

## Troubleshooting

### Error: "Could not load library"

If you see this error, it means the build script couldn't find the library. This can happen if:

1. **First time building**: Run `cargo clean && cargo build` to trigger the build script
2. **Missing git checkout**: The `teradatarustapi` dependency might not be fully downloaded
3. **Custom cargo directory**: Set `CARGO_HOME` if using a non-standard location

### Solution:

```bash
# Clean and rebuild to trigger library copy
cargo clean
cargo build

# Verify library was copied
ls -la target/debug/teradatasql.* # macOS/Linux
dir target\debug\teradatasql.* # Windows
```

### Manual Override

If automatic bundling fails, you can manually copy the library:

```bash
# Find the library in cargo cache
find ~/.cargo/git/checkouts -name "teradatasql.*"

# Copy to your project
cp ~/.cargo/git/checkouts/teradatarustapi-*/*/teradatasql.dylib .

# Run with explicit path
cargo run -- --logon "..." --ping --driver-lib-dir .
```

## Benefits

1. **Zero external dependencies**: Everything needed is included
2. **Cross-platform**: Works on macOS, Linux, and Windows without changes
3. **Version consistency**: Library version matches the API version
4. **Simplified deployment**: Just distribute the binary and library together
5. **No system modifications**: Doesn't require sudo or admin rights

## Technical Notes

- The library loading uses the `libloading` crate for dynamic linking
- Library path is determined at compile time using `option_env!("TERADATA_LIB_DIR")`
- Driver is loaded once per process using `once_cell::OnceCell` for thread safety
- Build script runs automatically on every clean build

## Related Files

- `build.rs` - Build script that copies the library
- `src/db.rs` - Database client that loads the library
- `Cargo.toml` - Dependency on `teradatarustapi`

## Acknowledgments

The Teradata native libraries are provided by the [teradatarustapi](https://github.com/Teradata/teradatarustapi) project, which wraps the Teradata GoSQL Driver.
