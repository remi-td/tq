#!/bin/sh
# install.sh - Install tq (Teradata Query) CLI tool
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/remi-td/tq/master/install.sh | sh
#
# Environment variables:
#   TQ_INSTALL_DIR  - Override install directory (default: ~/.local/bin)
#   TQ_VERSION      - Install a specific version (default: latest)

set -e

REPO="remi-td/tq"
RELEASES_URL="https://github.com/${REPO}/releases"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"

# --- Helper functions ---

say() {
    printf "tq-install: %s\n" "$1"
}

err() {
    say "ERROR: $1" >&2
    exit 1
}

warn() {
    say "WARNING: $1" >&2
}

# --- Platform detection ---

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    # Detect Windows/MSYS/Cygwin environments
    case "$OS" in
        MINGW*|MSYS*|CYGWIN*)
            err "Windows is not supported by this installer.
  Download the .zip package manually from:
  ${RELEASES_URL}/latest"
            ;;
    esac

    case "$OS" in
        Linux)  OS="unknown-linux-gnu" ;;
        Darwin) OS="apple-darwin" ;;
        *)      err "Unsupported operating system: $OS. tq supports Linux and macOS." ;;
    esac

    case "$ARCH" in
        x86_64|amd64)  ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *)             err "Unsupported architecture: $ARCH. tq supports x86_64 and aarch64/arm64." ;;
    esac

    TARGET="${ARCH}-${OS}"
}

# --- Main ---

main() {
    # Check for required tools
    if ! command -v curl >/dev/null 2>&1; then
        err "curl is required but not found. Install curl and try again."
    fi

    detect_platform
    say "Detected platform: ${TARGET}"

    # Determine version
    if [ -n "${TQ_VERSION}" ]; then
        VERSION="${TQ_VERSION}"
        say "Installing version: ${VERSION}"
    else
        say "Finding latest release..."
        VERSION=$(curl -sSL "${API_URL}" | grep '"tag_name"' | sed 's/.*"tag_name": *"//;s/".*//')
        if [ -z "${VERSION}" ]; then
            err "Could not determine latest release version.
  Check your internet connection or set TQ_VERSION manually."
        fi
        say "Latest version: ${VERSION}"
    fi

    # Set up temporary directory with cleanup trap
    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT
    cd "$TMPDIR"

    # Download artifact and checksums
    ASSET="tq-${VERSION}-${TARGET}.tar.gz"
    DOWNLOAD_URL="${RELEASES_URL}/download/${VERSION}/${ASSET}"
    CHECKSUMS_URL="${RELEASES_URL}/download/${VERSION}/checksums.txt"

    say "Downloading ${ASSET}..."
    curl -fsSL -o "${ASSET}" "${DOWNLOAD_URL}" || err "Failed to download ${ASSET}.
  URL: ${DOWNLOAD_URL}
  Check that version ${VERSION} exists and includes a build for ${TARGET}."

    say "Downloading checksums..."
    curl -fsSL -o checksums.txt "${CHECKSUMS_URL}" || err "Failed to download checksums.txt."

    # Verify checksum
    EXPECTED=$(grep "${ASSET}" checksums.txt | cut -d' ' -f1)
    if [ -z "${EXPECTED}" ]; then
        warn "No checksum entry found for ${ASSET}. Skipping verification."
    else
        if command -v sha256sum >/dev/null 2>&1; then
            ACTUAL=$(sha256sum "${ASSET}" | cut -d' ' -f1)
        elif command -v shasum >/dev/null 2>&1; then
            ACTUAL=$(shasum -a 256 "${ASSET}" | cut -d' ' -f1)
        else
            warn "No sha256sum or shasum found. Skipping checksum verification."
            ACTUAL="${EXPECTED}"
        fi

        if [ "${EXPECTED}" != "${ACTUAL}" ]; then
            err "Checksum verification failed!
  Expected: ${EXPECTED}
  Actual:   ${ACTUAL}
  The downloaded file may be corrupted. Try again or download manually."
        fi
        say "Checksum verified."
    fi

    # Extract
    say "Extracting..."
    tar xzf "${ASSET}"

    # Install
    INSTALL_DIR="${TQ_INSTALL_DIR:-$HOME/.local/bin}"
    mkdir -p "${INSTALL_DIR}"

    cp "tq-${VERSION}-${TARGET}/tq" "${INSTALL_DIR}/"
    chmod +x "${INSTALL_DIR}/tq"

    # Copy the teradatasql native library alongside the binary
    for lib in "tq-${VERSION}-${TARGET}"/teradatasql.*; do
        if [ -f "$lib" ]; then
            cp "$lib" "${INSTALL_DIR}/"
        fi
    done

    say "Installed tq to ${INSTALL_DIR}/tq"

    # Check if install directory is in PATH
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*)
            ;;
        *)
            warn "${INSTALL_DIR} is not in your PATH."
            say "Add it to your shell profile:"
            say "  export PATH=\"${INSTALL_DIR}:\$PATH\""
            ;;
    esac

    say "Installation complete! Run 'tq --version' to verify."
}

main
