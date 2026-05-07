#!/bin/bash

set -euo pipefail

REPO="fredyw/bulk-rename-rs"
BINARY_NAME="bren"
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

show_help() {
  echo "Usage: install.sh [options]"
  echo ""
  echo "Options:"
  echo "  --source     Build and install from source using 'cargo install'"
  echo "  --help       Show this help message"
}

install_from_source() {
  echo "Installing $BINARY_NAME from source..."
  if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed. Please install Rust first: https://rustup.rs/"
    exit 1
  fi
  cargo install --path .
  exit 0
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --source)
      install_from_source
      ;;
    --help)
      show_help
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      show_help
      exit 1
      ;;
  esac
  shift
done

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)
    PLATFORM="linux"
    ;;
  darwin)
    PLATFORM="macos"
    ;;
  *)
    echo "Error: Unsupported OS '$OS'. Only Linux and macOS are supported by this script."
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64)
    ARCH_NAME="x86_64"
    ;;
  arm64|aarch64)
    ARCH_NAME="arm64"
    ;;
  *)
    echo "Error: Unsupported architecture '$ARCH'."
    exit 1
    ;;
esac

if [[ "$PLATFORM" == "macos" && "$ARCH_NAME" == "arm64" ]]; then
  ARTIFACT_NAME="bren-macos-arm64"
elif [[ "$PLATFORM" == "linux" && "$ARCH_NAME" == "x86_64" ]]; then
  ARTIFACT_NAME="bren-linux-x86_64"
else
  echo "Error: No prebuilt binary found for $PLATFORM-$ARCH_NAME."
  if [[ "$PLATFORM" == "macos" && "$ARCH_NAME" == "x86_64" ]]; then
    echo "Intel-based Macs are not supported with prebuilt binaries."
  fi
  echo "Try installing from source: ./install.sh --source"
  exit 1
fi

echo "Fetching latest release version for $REPO..."
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [[ -z "$LATEST_RELEASE" ]]; then
  echo "Error: Could not find any releases for $REPO."
  echo "Make sure you have published at least one version with a tag."
  exit 1
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_RELEASE/$ARTIFACT_NAME"

echo "Downloading $BINARY_NAME $LATEST_RELEASE for $PLATFORM-$ARCH_NAME..."
TEMP_FILE=$(mktemp)
curl -SL "$DOWNLOAD_URL" -o "$TEMP_FILE"

echo "Installing to $INSTALL_DIR/$BINARY_NAME..."
mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Remove macOS quarantine flag if applicable
if [[ "$PLATFORM" == "macos" ]]; then
  xattr -d com.apple.quarantine "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null || true
fi

echo "Successfully installed $BINARY_NAME $LATEST_RELEASE!"

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  echo ""
  echo "Warning: $INSTALL_DIR is not in your PATH."
  echo "You may need to add it to your shell configuration (e.g., ~/.bashrc or ~/.zshrc):"
  echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
else
  $BINARY_NAME --version
fi
