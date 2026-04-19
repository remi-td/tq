#!/usr/bin/env bash
# Mirror of .github/workflows/ci.yml — run before every `git push` to master.
# Failing here is a hard stop: do NOT push.
#
# Toolchain note: CI runs `dtolnay/rust-toolchain@stable`, which tracks the
# latest stable Rust release. If your local rustc is older than CI's, new
# clippy lints from a fresh stable may still break CI even when this script
# passes locally. Keep `rustup` on `stable` (or install rustup if you're on a
# distro-packaged rustc) before trusting this gate.

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "[ci-check] rustc: $(rustc --version)"
echo "[ci-check] cargo clippy --all-targets -- -D warnings"
cargo clippy --all-targets -- -D warnings

echo "[ci-check] cargo test --lib"
cargo test --lib

echo "[ci-check] OK"
