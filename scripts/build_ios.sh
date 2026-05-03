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
#   ./scripts/build_ios.sh --sim --debug
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Parse flags
SIM=false
PROFILE="release"
for arg in "$@"; do
    case "$arg" in
        --sim)   SIM=true ;;
        --debug) PROFILE="debug" ;;
    esac
done

if $SIM; then
    TARGET="aarch64-apple-ios-sim"
    SDK="iphonesimulator"
    echo "==> Building for iOS Simulator (arm64, $PROFILE)..."
else
    TARGET="aarch64-apple-ios"
    SDK="iphoneos"
    echo "==> Building for iOS Device (arm64, $PROFILE)..."
fi

# Set SDKROOT so coreaudio-sys/cpal can find iOS SDK headers.
export SDKROOT
SDKROOT=$(xcrun --sdk "$SDK" --show-sdk-path)
export IPHONEOS_DEPLOYMENT_TARGET=16.0

echo "    Target: $TARGET"
echo "    SDK:    $SDKROOT"

CARGO_FLAGS=""
[ "$PROFILE" = "release" ] && CARGO_FLAGS="--release"

cargo build $CARGO_FLAGS --target "$TARGET" --manifest-path "$PROJECT_DIR/Cargo.toml"

BINARY="$PROJECT_DIR/target/$TARGET/$PROFILE/spout"
echo "==> Build complete: $BINARY"
echo "    Size: $(du -sh "$BINARY" | cut -f1)"
