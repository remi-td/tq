# Release & Distribution Design

This document describes the technical design for building, packaging, and distributing tq binaries via GitHub Releases, and the associated build system changes required for cross-compilation.

## 1. build.rs Cross-Compilation Fix

### Problem

The current `build.rs` uses `cfg!(target_os = "macos")` to determine which teradatasql native library to copy alongside the binary. The `cfg!()` macro evaluates against the **host** platform at compile time, not the **target** platform. When cross-compiling (e.g., building a Linux binary on macOS), the wrong library is selected.

### Solution

Replace `cfg!(target_os)` with Cargo-provided environment variables that reflect the actual target:

- `CARGO_CFG_TARGET_OS` - Target operating system (e.g., `linux`, `macos`, `windows`)
- `CARGO_CFG_TARGET_ARCH` - Target architecture (e.g., `x86_64`, `aarch64`)

These environment variables are set by Cargo during the build and always reflect the target triple, even during cross-compilation.

### Library Name Mapping Table

The teradatarustapi repository ships pre-built Go-based native libraries. The mapping from (target_os, target_arch) to library filename is:

| target_os | target_arch | Library Filename | Notes |
|-----------|-------------|------------------|-------|
| `macos` | `x86_64` | `teradatasql.dylib` | Universal binary (x86_64 + arm64) |
| `macos` | `aarch64` | `teradatasql.dylib` | Universal binary (x86_64 + arm64) |
| `windows` | `x86_64` | `teradatasql.dll` | 64-bit |
| `linux` | `x86_64` | `teradatasql.so` | Standard x86_64 |
| `linux` | `aarch64` | `teradatasql.arm.so` | ARM64 variant |

Note: `teradatasql.fips.so`, `teradatasql.arm.fips.so`, `teradatasql.power.so`, and `teradatasql.x86.so`/`teradatasql.x86.dll` (32-bit) are available in the repo but are not targeted for release builds.

### Implementation Approach

```rust
fn determine_library_name() -> &'static str {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    match (target_os.as_str(), target_arch.as_str()) {
        ("macos", _) => "teradatasql.dylib",
        ("windows", _) => "teradatasql.dll",
        ("linux", "aarch64") => "teradatasql.arm.so",
        ("linux", _) => "teradatasql.so",
        _ => "teradatasql.so", // fallback
    }
}
```

### Backward Compatibility

- `CARGO_CFG_TARGET_OS` and `CARGO_CFG_TARGET_ARCH` are set by Cargo for both native and cross builds.
- For native builds (no `--target` flag), these variables match the host platform, so behavior is identical to the current `cfg!()` approach.
- The `CARGO_HOME` / git checkout discovery logic remains unchanged.

### Runtime Compatibility

The runtime library loading in `teradatarustapi::load_driver()` uses `env::consts::OS` and `env::consts::ARCH` which are baked into the compiled binary for the **target** platform. So the runtime code already selects the correct library extension. The build.rs fix ensures the correct library file is copied to the target directory at build time, matching what the runtime will try to load.

## 2. GitHub Actions Release Workflow

### Trigger

The workflow triggers on version tag push:

```yaml
on:
  push:
    tags:
      - 'v*'
```

### Build Matrix

| Target Triple | Runner | Build Method | Library | Package Format |
|--------------|--------|-------------|---------|----------------|
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | Native `cargo build` | `teradatasql.so` | `.tar.gz` |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` | `cross build` (Docker) | `teradatasql.arm.so` | `.tar.gz` |
| `x86_64-apple-darwin` | `macos-latest` | Cross `--target x86_64-apple-darwin` | `teradatasql.dylib` | `.tar.gz` |
| `aarch64-apple-darwin` | `macos-latest` | Native `cargo build` | `teradatasql.dylib` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-latest` | Native `cargo build` | `teradatasql.dll` | `.zip` |

### Runner Selection Rationale

- **`ubuntu-latest`**: Standard for Linux builds. Also hosts `cross-rs` for aarch64 cross-compilation.
- **`macos-latest`**: GitHub's macOS runners are now ARM64 (Apple Silicon). Native build produces aarch64, and cross-compilation with `--target x86_64-apple-darwin` produces x86_64. The `teradatasql.dylib` is a universal binary, so the same `.dylib` works for both architectures.
- **`windows-latest`**: Standard for Windows MSVC builds.

### Cross-Compilation Strategy

#### Linux aarch64 (`cross-rs/cross`)

`cross` provides Docker containers with the correct cross-compilation toolchains. The container has the linker and sysroot for aarch64-linux-gnu.

```yaml
- name: Install cross
  run: cargo install cross --git https://github.com/cross-rs/cross

- name: Build
  run: cross build --release --target aarch64-unknown-linux-gnu
```

The build.rs will read `CARGO_CFG_TARGET_ARCH=aarch64` inside the cross container and select `teradatasql.arm.so`.

#### macOS x86_64

macOS supports cross-compilation between arm64 and x86_64 natively via Xcode toolchains. No Docker or special tools needed.

```yaml
- name: Add x86_64 target
  run: rustup target add x86_64-apple-darwin

- name: Build
  run: cargo build --release --target x86_64-apple-darwin
```

### Packaging Steps

Each matrix entry produces a package with this structure:

```
tq-<version>-<target>/
  tq                      (or tq.exe on Windows)
  teradatasql.<ext>       (native library for the target)
  LICENSE
```

Packaging commands:

```yaml
# Linux/macOS
- name: Package
  run: |
    mkdir -p staging/tq-${VERSION}-${TARGET}
    cp target/${TARGET}/release/tq staging/tq-${VERSION}-${TARGET}/
    cp <lib_path> staging/tq-${VERSION}-${TARGET}/
    cp LICENSE staging/tq-${VERSION}-${TARGET}/
    cd staging && tar czf tq-${VERSION}-${TARGET}.tar.gz tq-${VERSION}-${TARGET}

# Windows
- name: Package
  run: |
    mkdir staging\tq-${VERSION}-${TARGET}
    copy target\${TARGET}\release\tq.exe staging\tq-${VERSION}-${TARGET}\
    copy <lib_path> staging\tq-${VERSION}-${TARGET}\
    copy LICENSE staging\tq-${VERSION}-${TARGET}\
    cd staging && Compress-Archive -Path tq-${VERSION}-${TARGET} -DestinationPath tq-${VERSION}-${TARGET}.zip
```

### Native Library Discovery in CI

The teradatasql library files live in the cargo git checkout directory. The build.rs already handles finding and copying these. For packaging, we need to locate the correct library file from the cargo cache.

Approach: After `cargo build`, the library has been copied to the target directory by build.rs. We can find it there:

```
target/<target-triple>/release/teradatasql.<ext>
```

This works because build.rs copies the library to the target directory (the parent^3 of `OUT_DIR`).

### Checksums

A single `checksums.txt` file contains SHA256 hashes for all artifacts:

```yaml
- name: Generate checksums
  run: |
    cd staging
    shasum -a 256 tq-*.tar.gz tq-*.zip > checksums.txt
```

### Release Creation

Uses `softprops/action-gh-release` to create the GitHub Release and upload all artifacts:

```yaml
- name: Create Release
  uses: softprops/action-gh-release@v2
  with:
    files: |
      staging/tq-*.tar.gz
      staging/tq-*.zip
      staging/checksums.txt
    body: |
      ## tq ${VERSION}

      ### Installation

      **One-liner (Linux/macOS):**
      ```sh
      curl -sSL https://raw.githubusercontent.com/remi-td/tq/master/install.sh | sh
      ```

      **Manual download:** Choose your platform below.

      ### Checksums
      See `checksums.txt` for SHA256 verification.
    generate_release_notes: true
```

### Workflow Structure

The workflow uses a two-job approach:

1. **`build` job**: Matrix build across all 5 targets. Each job uploads its artifact.
2. **`release` job**: Depends on all build jobs. Downloads all artifacts, generates checksums, creates the GitHub Release.

This separation ensures that the release is only created when all builds succeed, and checksums cover all artifacts in a single file.

## 3. Install Script

### Design Principles

- POSIX `sh` compatible (no bashisms: no `[[ ]]`, no arrays, no `local` in functions on strict sh, no `source`)
- Fail fast with `set -e`
- Clear error messages for unsupported platforms
- Idempotent (safe to run multiple times)

### OS/Architecture Detection

```sh
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)  OS="unknown-linux-gnu" ;;
        Darwin) OS="apple-darwin" ;;
        *)      err "Unsupported OS: $OS. tq supports Linux and macOS." ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH="x86_64" ;;
        aarch64|arm64)   ARCH="aarch64" ;;
        *)               err "Unsupported architecture: $ARCH" ;;
    esac

    TARGET="${ARCH}-${OS}"
}
```

### Release Discovery

Uses the GitHub API to find the latest release:

```sh
RELEASE_URL="https://api.github.com/repos/remi-td/tq/releases/latest"
VERSION=$(curl -sSL "$RELEASE_URL" | grep '"tag_name"' | sed 's/.*"tag_name": *"//;s/".*//')
```

No dependency on `jq` -- uses `grep` and `sed` for JSON extraction (the `tag_name` field is simple enough for this approach).

### Download and Verify

```sh
ASSET="tq-${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/remi-td/tq/releases/download/${VERSION}/${ASSET}"
CHECKSUMS_URL="https://github.com/remi-td/tq/releases/download/${VERSION}/checksums.txt"

# Download asset and checksums
curl -sSLO "$DOWNLOAD_URL"
curl -sSLO "$CHECKSUMS_URL"

# Verify checksum
EXPECTED=$(grep "$ASSET" checksums.txt | cut -d' ' -f1)
if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL=$(sha256sum "$ASSET" | cut -d' ' -f1)
elif command -v shasum >/dev/null 2>&1; then
    ACTUAL=$(shasum -a 256 "$ASSET" | cut -d' ' -f1)
else
    warn "No sha256sum or shasum found. Skipping checksum verification."
    ACTUAL="$EXPECTED"
fi

if [ "$EXPECTED" != "$ACTUAL" ]; then
    err "Checksum verification failed!"
fi
```

### Installation

```sh
INSTALL_DIR="${TQ_INSTALL_DIR:-$HOME/.local/bin}"

mkdir -p "$INSTALL_DIR"
tar xzf "$ASSET"
cp "tq-${VERSION}-${TARGET}/tq" "$INSTALL_DIR/"
cp "tq-${VERSION}-${TARGET}/teradatasql."* "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/tq"
```

### PATH Guidance

After installation, check if the install directory is on PATH and provide guidance:

```sh
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) warn "$INSTALL_DIR is not in your PATH. Add it:"
       warn "  export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
```

### Temporary Directory Cleanup

All downloads happen in a temporary directory created with `mktemp -d`, with a trap to clean up on exit:

```sh
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT
cd "$TMPDIR"
```

### Error Handling

- Windows detection: If `uname -s` returns `MINGW*` or `MSYS*` or `CYGWIN*`, print a message directing users to download the `.zip` manually from GitHub Releases.
- Network errors: `curl` with `-f` flag to fail on HTTP errors.
- Missing tools: Check for `curl` availability at script start.

## 4. Sprint 40 Remediation Design

### 4.1 Merge execute/execute_with_params in query.rs

**Current state:** Three pairs of duplicate functions:
- `execute()` and `execute_with_params()`
- `execute_to_file()` and `execute_to_file_with_params()`

**Design:** Make `params` an `Option<&ParamStore>` parameter in the existing functions:

```rust
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &QueryArgs,
    params: Option<&ParamStore>,  // NEW: optional
    writer: &mut W,
    use_color: bool,
    verbose: bool,
) -> Result<()> {
    let source = determine_input_source(args)?;
    let sql = read_input_sql(&source)?;

    // Apply substitution if params provided and non-empty
    let sql = match params {
        Some(p) if !p.is_empty() => p.substitute(&sql)?,
        _ => sql,
    };

    // ... rest unchanged
}
```

Similarly for `execute_to_file`. This eliminates `execute_with_params` and `execute_to_file_with_params` entirely.

**Call site change in `main.rs`:**

```rust
// Before:
commands::query::execute_with_params(&client, &args, &param_store, ...);
// After:
commands::query::execute(&client, &args, Some(&param_store), ...);
```

### 4.2 Merge execute/execute_with_params in repl/mod.rs

**Current state:** `execute()` and `execute_with_params()` differ only in that the latter initializes `state.params` from the `initial_params` argument.

**Design:** Merge into a single `execute()` that takes `Option<ParamStore>`:

```rust
pub fn execute<W: Write>(
    client: DatabaseClient,
    args: &ReplArgs,
    initial_params: Option<ParamStore>,  // NEW: optional
    writer: &mut W,
    use_color: bool,
    _verbose: bool,
) -> Result<()> {
    // ... setup ...
    let mut state = {
        let cs = completion_state.lock().unwrap();
        let mut s = ReplState::new(cs.client().config().clone());
        s.set_default_limit(args.default_limit);
        if let Some(params) = initial_params {
            s.params = params;
        }
        s
    };
    // ... rest unchanged (params banner display, editor init, repl_loop) ...
}
```

**Call site change in `main.rs`:**

```rust
// Before:
commands::repl::execute_with_params(client, &args, param_store, ...);
// After:
commands::repl::execute(client, &args, Some(param_store), ...);
```

### 4.3 LazyLock for Regex in params.rs

**Current state:** The `substitute()` method compiles a regex on every call:

```rust
let re = Regex::new(r"\{\{([a-zA-Z0-9_.$]+)\}\}").expect("valid regex");
```

This is called twice per substitution (first pass for error collection, second pass for replacement), and once per SQL statement in the REPL.

**Design:** Use `std::sync::LazyLock` (stable since Rust 1.80) to compile the regex once:

```rust
use std::sync::LazyLock;

static VARIABLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{\{([a-zA-Z0-9_.$]+)\}\}").expect("valid regex")
});
```

Then reference `&*VARIABLE_RE` or `VARIABLE_RE.captures_iter(sql)` in `substitute()`.

This is the only regex in `params.rs`. The change is straightforward and eliminates repeated compilation.

## 5. Feasibility Assessment

### Fully Feasible

All four objectives are technically feasible with no blocking risks:

1. **build.rs fix** -- Straightforward replacement of `cfg!()` with env var reads. Well-documented Cargo behavior.

2. **GitHub Actions workflow** -- Standard CI/CD pattern. The teradatarustapi library files are available for all targets in the git checkout. The `cross-rs` tool is mature and widely used.

3. **Install script** -- Standard POSIX shell scripting pattern used by many Rust projects (rustup, just, etc.).

4. **Sprint 40 remediation** -- Simple refactoring with clear mechanical changes. No behavioral changes.

### Concerns and Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| `cross-rs` container may not have `CARGO_HOME` access to teradatarustapi git checkout | Medium | The cargo git checkout is volume-mounted into the cross container by default. Verify with a test build. If not, use `CROSS_CONTAINER_OPTS` to mount it. |
| macOS universal dylib may not load on x86_64 target | Low | `teradatasql.dylib` in the repo is already a universal binary (fat binary with both x86_64 and arm64 slices). Verified by the fact that the library name is architecture-independent. |
| Windows packaging with PowerShell syntax in YAML | Low | Use `shell: pwsh` explicitly in the Windows matrix entry. Test `Compress-Archive` syntax. |
| GitHub API rate limiting for install script | Low | The `/releases/latest` endpoint has generous unauthenticated limits (60/hour). The script makes only 1 API call. For CI use, users can set `GITHUB_TOKEN`. |
| `LazyLock` requires Rust 1.80+ (MSRV concern) | Low | The project already uses Rust 2021 edition features. Current stable Rust is well past 1.80. No MSRV is declared in Cargo.toml. |

### Recommended Approach

1. **Start with build.rs fix** -- This is prerequisite for the CI workflow. It can be tested locally with `cargo build --target x86_64-apple-darwin` on the macOS dev machine.

2. **Implement remediation** -- Small, self-contained changes that clean up code. Good warm-up and reduces diff noise in later PRs.

3. **Create release workflow** -- Depends on build.rs fix. Test with a dry-run tag (e.g., `v0.0.0-test`) before the real release.

4. **Create install script** -- Independent of workflow but should be validated against actual release artifacts. Create last and test against the v0.0.0-test release.
