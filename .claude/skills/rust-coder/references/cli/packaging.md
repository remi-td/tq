# Packaging and Distributing Rust CLI Applications

## Distribution Methods Overview

Three main approaches, from simplest to most user-friendly:

1. **cargo publish** - Quick for Rust developers
2. **Binary releases** - Fast for any user
3. **Package managers** - Best user experience with auto-updates

Best practice: Support multiple methods simultaneously.

## Method 1: Cargo Publish

### Setup

1. Create account at [crates.io](https://crates.io)
2. Get API token from account settings
3. Login once per machine:
   ```bash
   cargo login <your-token>
   ```

### Required Cargo.toml Metadata

```toml
[package]
name = "my-cli-tool"
version = "0.1.0"
authors = ["Your Name <your@email.com>"]
edition = "2021"
license = "MIT OR Apache-2.0"
description = "A helpful command-line tool for doing X"
readme = "README.md"
homepage = "https://github.com/username/my-cli-tool"
repository = "https://github.com/username/my-cli-tool"
keywords = ["cli", "utility", "tool"]  # Max 5 keywords
categories = ["command-line-utilities"]  # From crates.io categories
exclude = [
    "tests/fixtures/*",
    "*.png",
    "docs/*",
]

[badges]
maintenance = { status = "actively-developed" }
```

### Publishing

```bash
# Dry run to check for issues
cargo publish --dry-run

# Publish to crates.io
cargo publish
```

### Versioning

Follow [Semantic Versioning](https://semver.org/):
- `0.1.0` → `0.1.1` - Patch: Bug fixes
- `0.1.0` → `0.2.0` - Minor: New features (backward compatible)
- `0.9.0` → `1.0.0` - Major: Breaking changes

Update version in `Cargo.toml`, then:
```bash
git tag v0.1.1
git push --tags
cargo publish
```

### User Installation

Users install with:
```bash
cargo install my-cli-tool
```

Binary is placed in `~/.cargo/bin/` (should be in PATH).

Update to latest:
```bash
cargo install my-cli-tool --force
```

### Pros and Cons

**Pros:**
- Simple to set up
- Automatic versioning
- Integrated with Rust ecosystem

**Cons:**
- Requires Rust toolchain
- Compilation takes time
- Needs all build dependencies
- Only reaches Rust developers

## Method 2: Binary Releases

### Local Building

```bash
# Build optimized binary
cargo build --release

# Binary location
ls target/release/my-cli-tool
```

### Cross-Platform Compilation

#### Using cargo-cross

Install cross:
```bash
cargo install cross
```

Build for different targets:
```bash
# Linux (x86_64)
cross build --release --target x86_64-unknown-linux-musl

# macOS (Intel)
cross build --release --target x86_64-apple-darwin

# macOS (Apple Silicon)
cross build --release --target aarch64-apple-darwin

# Windows
cross build --release --target x86_64-pc-windows-gnu
```

Common targets:
- `x86_64-unknown-linux-gnu` - Linux (glibc)
- `x86_64-unknown-linux-musl` - Linux (static, portable)
- `x86_64-apple-darwin` - macOS Intel
- `aarch64-apple-darwin` - macOS Apple Silicon
- `x86_64-pc-windows-msvc` - Windows

### Automated CI Builds

#### GitHub Actions Example

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            artifact_name: my-cli-tool
            asset_name: my-cli-tool-linux-x86_64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-musl
            artifact_name: my-cli-tool
            asset_name: my-cli-tool-linux-aarch64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact_name: my-cli-tool
            asset_name: my-cli-tool-macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact_name: my-cli-tool
            asset_name: my-cli-tool-macos-aarch64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact_name: my-cli-tool.exe
            asset_name: my-cli-tool-windows-x86_64.exe

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross (Linux only)
        if: matrix.os == 'ubuntu-latest'
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: Build
        run: |
          if [ "${{ matrix.os }}" = "ubuntu-latest" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            cargo build --release --target ${{ matrix.target }}
          fi

      - name: Rename binary
        run: |
          cp target/${{ matrix.target }}/release/${{ matrix.artifact_name }} \
             ${{ matrix.asset_name }}

      - name: Create archive
        run: |
          tar czf ${{ matrix.asset_name }}.tar.gz \
            ${{ matrix.asset_name }} \
            README.md \
            LICENSE-*

      - name: Upload Release Asset
        uses: softprops/action-gh-release@v1
        with:
          files: ${{ matrix.asset_name }}.tar.gz
```

### Creating Release Archives

Include more than just the binary:

```bash
#!/bin/bash
VERSION="0.1.0"
TARGET="x86_64-unknown-linux-musl"

mkdir -p "my-cli-tool-${VERSION}"

# Copy files
cp "target/${TARGET}/release/my-cli-tool" "my-cli-tool-${VERSION}/"
cp README.md LICENSE-MIT LICENSE-APACHE "my-cli-tool-${VERSION}/"

# Optional: Add completions, man pages
cp completions/* "my-cli-tool-${VERSION}/"
cp docs/my-cli-tool.1 "my-cli-tool-${VERSION}/"

# Create archive
tar czf "my-cli-tool-${VERSION}-${TARGET}.tar.gz" "my-cli-tool-${VERSION}"

# Cleanup
rm -rf "my-cli-tool-${VERSION}"
```

### User Installation

Users download from GitHub Releases:

```bash
# Download
curl -L -O https://github.com/user/my-cli-tool/releases/download/v0.1.0/my-cli-tool-linux-x86_64.tar.gz

# Extract
tar xzf my-cli-tool-linux-x86_64.tar.gz

# Install
sudo mv my-cli-tool /usr/local/bin/

# Or install to user directory
mv my-cli-tool ~/.local/bin/
```

Document this in your README.

### Pros and Cons

**Pros:**
- No compilation required
- Fast installation
- Works without Rust toolchain
- Reaches wider audience

**Cons:**
- Must build for each platform
- Larger distribution size
- Manual download and install
- No automatic updates

## Method 3: Package Managers

### Homebrew (macOS/Linux)

Create a tap or add to homebrew-core:

#### Creating a Formula

```ruby
class MyCliTool < Formula
  desc "A helpful command-line tool"
  homepage "https://github.com/username/my-cli-tool"
  url "https://github.com/username/my-cli-tool/archive/v0.1.0.tar.gz"
  sha256 "..."
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/my-cli-tool", "--version"
  end
end
```

Users install with:
```bash
brew install my-cli-tool
```

#### cargo-binstall Support

Add to `Cargo.toml`:
```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ target }.tar.gz"
bin-dir = "{ name }-{ version }/{ bin }{ binary-ext }"
pkg-fmt = "tgz"
```

Users install with:
```bash
cargo binstall my-cli-tool
```

### APT/Debian Packages

Use `cargo-deb`:

```bash
cargo install cargo-deb
```

Add metadata to `Cargo.toml`:
```toml
[package.metadata.deb]
maintainer = "Your Name <your@email.com>"
copyright = "2024, Your Name <your@email.com>"
license-file = ["LICENSE-MIT", "4"]
extended-description = """\
A longer description of your tool."""
depends = "$auto"
section = "utility"
priority = "optional"
assets = [
    ["target/release/my-cli-tool", "usr/bin/", "755"],
    ["README.md", "usr/share/doc/my-cli-tool/", "644"],
    ["completions/my-cli-tool.bash", "usr/share/bash-completion/completions/my-cli-tool", "644"],
]
```

Build:
```bash
cargo deb
```

Creates `.deb` file in `target/debian/`.

Users install with:
```bash
sudo dpkg -i my-cli-tool_0.1.0_amd64.deb
```

### AUR (Arch Linux)

Create a PKGBUILD:

```bash
pkgname=my-cli-tool
pkgver=0.1.0
pkgrel=1
pkgdesc="A helpful command-line tool"
arch=('x86_64')
url="https://github.com/username/my-cli-tool"
license=('MIT')
depends=()
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('...')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}
```

Submit to AUR, users install with:
```bash
yay -S my-cli-tool
```

### Chocolatey (Windows)

Create a Chocolatey package with nuspec file.

### Scoop (Windows)

Add to a Scoop bucket:

```json
{
    "version": "0.1.0",
    "description": "A helpful command-line tool",
    "homepage": "https://github.com/username/my-cli-tool",
    "license": "MIT",
    "url": "https://github.com/username/my-cli-tool/releases/download/v0.1.0/my-cli-tool-windows-x86_64.zip",
    "hash": "...",
    "bin": "my-cli-tool.exe"
}
```

### Pros and Cons

**Pros:**
- Best user experience
- Automatic updates
- Familiar installation method
- System integration

**Cons:**
- Most complex to set up
- Different process for each platform
- May require approval/review
- Maintenance overhead

## Additional Distribution Assets

### Shell Completions

Generate with clap:

```rust
use clap::{Command, CommandFactory};
use clap_complete::{generate_to, shells::*};
use std::env;
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = std::path::PathBuf::from("completions");
    std::fs::create_dir_all(&outdir)?;

    let mut cmd = Cli::command();
    generate_to(Bash, &mut cmd, "my-cli-tool", &outdir)?;
    generate_to(Zsh, &mut cmd, "my-cli-tool", &outdir)?;
    generate_to(Fish, &mut cmd, "my-cli-tool", &outdir)?;

    Ok(())
}
```

### Man Pages

Generate with clap-mangen:

```rust
use clap::CommandFactory;
use clap_mangen::Man;

include!("src/cli.rs");

fn main() -> std::io::Result<()> {
    let out_dir = std::path::PathBuf::from("man");
    std::fs::create_dir_all(&out_dir)?;

    let cmd = Cli::command();
    let man = Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer)?;

    std::fs::write(out_dir.join("my-cli-tool.1"), buffer)?;
    Ok(())
}
```

## Layered Distribution Strategy

Recommended approach using ripgrep as example:

1. **Start with cargo publish**
   - Reaches Rust developers immediately
   - Easiest to set up

2. **Add binary releases**
   - Set up CI to build for major platforms
   - Publish to GitHub Releases
   - Add installation instructions to README

3. **Integrate with package managers**
   - Start with Homebrew (popular and straightforward)
   - Add platform-specific packages as demand grows
   - Prioritize based on user requests

4. **Provide multiple installation methods**
   Document all methods in README:
   ```markdown
   ## Installation

   ### Cargo
   ```bash
   cargo install my-cli-tool
   ```

   ### Homebrew
   ```bash
   brew install my-cli-tool
   ```

   ### Binary Releases
   Download from [releases page](https://github.com/user/my-cli-tool/releases)

   ### From Source
   ```bash
   git clone https://github.com/user/my-cli-tool
   cd my-cli-tool
   cargo build --release
   ```
   ```

## Version Management

### Changelog

Maintain a CHANGELOG.md:

```markdown
# Changelog

## [Unreleased]

## [0.2.0] - 2024-01-15
### Added
- New --format option
- Support for JSON output

### Changed
- Improved error messages

### Fixed
- Bug in file parsing

## [0.1.0] - 2024-01-01
- Initial release
```

### Release Checklist

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Commit changes
4. Create and push tag: `git tag v0.2.0 && git push --tags`
5. CI builds and uploads binaries
6. Publish to crates.io: `cargo publish`
7. Update package manager formulas
8. Announce release

## Summary

**Quick start**: Use `cargo publish`
**Wider reach**: Add binary releases with CI
**Best UX**: Integrate with package managers
**Best practice**: Support multiple methods

Focus on what your users need. If targeting Rust developers, cargo is sufficient. For general audiences, prioritize binaries and package managers.
