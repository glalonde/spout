#!/usr/bin/env bash
#
# Build, install, and launch Spout on a connected iOS device.
#
# xcodebuild handles the Rust cross-compile internally (build phase in
# ios/Spout.xcodeproj calls cargo automatically).
#
# Prerequisites:
#   - Xcode with iOS platform support installed
#   - Device trusted on this Mac (Settings > General > VPN & Device Management)
#   - rustup target add aarch64-apple-ios
#
# Usage:
#   ./scripts/run_ios.sh              # build Debug, auto-detect device
#   ./scripts/run_ios.sh --release    # Release build
#   ./scripts/run_ios.sh --device <id> # use specific device UDID
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
XCODEPROJ="$PROJECT_DIR/ios/Spout.xcodeproj"
BUNDLE_ID="com.glalonde.spout"
SCHEME="Spout"

CONFIGURATION="Debug"
DEVICE_ID=""

for arg in "$@"; do
    case "$arg" in
        --release)       CONFIGURATION="Release" ;;
        --device)        shift; DEVICE_ID="$1" ;;
        --device=*)      DEVICE_ID="${arg#--device=}" ;;
    esac
done

# Auto-detect the first connected physical device if not specified.
if [ -z "$DEVICE_ID" ]; then
    # iOS devices have two parenthesised groups: "(OS version) (UDID)"
    # Mac mini only has one. Grep for lines with two groups, extract the last one.
    DEVICE_ID=$(xcrun xctrace list devices 2>/dev/null \
        | awk '/^== Devices ==/{found=1; next} /^== Simulators ==/{found=0} \
               found && /\(.*\).*\(.*\)/{match($0,/\(([^)]+)\)$/); \
               print substr($0, RSTART+1, RLENGTH-2); exit}')
    if [ -z "$DEVICE_ID" ]; then
        echo "error: no connected device found. Plug in your iPhone and trust this Mac." >&2
        exit 1
    fi
    echo "==> Auto-detected device: $DEVICE_ID"
fi

BUILD_DIR="$PROJECT_DIR/ios/build"

echo "==> Building $SCHEME ($CONFIGURATION) for device $DEVICE_ID..."
# NSUnbufferedIO forces xcodebuild to flush immediately.
# Pipe through grep so cargo's Compiling/Linking lines show through.
NSUnbufferedIO=YES xcodebuild \
    -project "$XCODEPROJ" \
    -scheme "$SCHEME" \
    -configuration "$CONFIGURATION" \
    -destination "id=$DEVICE_ID" \
    -derivedDataPath "$BUILD_DIR" \
    -allowProvisioningUpdates \
    build 2>&1 \
    | grep --line-buffered -E "error:|warning:|==>|Compiling|Linking|Build|Installing|Signing|CodeSign|note:" \
    || true

APP_PATH=$(find "$BUILD_DIR/Build/Products/$CONFIGURATION-iphoneos" -name "Spout.app" | head -1)
if [ -z "$APP_PATH" ]; then
    echo "error: Spout.app not found under $BUILD_DIR" >&2
    exit 1
fi

echo "==> Installing $APP_PATH..."
xcrun devicectl device install app \
    --device "$DEVICE_ID" \
    "$APP_PATH"

echo "==> Launching $BUNDLE_ID (streaming logs, Ctrl-C to stop)..."
xcrun devicectl device process launch \
    --device "$DEVICE_ID" \
    --console \
    --terminate-existing \
    "$BUNDLE_ID"
