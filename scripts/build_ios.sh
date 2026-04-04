#!/usr/bin/env bash
#
# Cross-compile Spout for iOS.
#
# Prerequisites:
#   - Xcode with iOS platform support
#   - rustup target add aarch64-apple-ios aarch64-apple-ios-sim
#
# Usage:
#   ./scripts/build_ios.sh              # build for device (arm64)
#   ./scripts/build_ios.sh --sim        # build for simulator (arm64)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

if [[ "${1:-}" == "--sim" ]]; then
    TARGET="aarch64-apple-ios-sim"
    SDK="iphonesimulator"
    echo "==> Building for iOS Simulator (arm64)..."
else
    TARGET="aarch64-apple-ios"
    SDK="iphoneos"
    echo "==> Building for iOS Device (arm64)..."
fi

# Set SDK root for cpal/coreaudio-sys cross-compilation.
export SDKROOT
SDKROOT=$(xcrun --sdk "$SDK" --show-sdk-path)

echo "    Target: $TARGET"
echo "    SDK:    $SDKROOT"

cargo build --release --target "$TARGET" --manifest-path "$PROJECT_DIR/Cargo.toml"

echo "==> Build complete: target/$TARGET/release/libspout.a"
