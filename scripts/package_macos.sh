#!/usr/bin/env bash
#
# Build and package Spout as a macOS .app bundle.
# Optionally creates a .dmg disk image.
#
# Usage:
#   ./scripts/package_macos.sh           # build + .app bundle
#   ./scripts/package_macos.sh --dmg     # also create .dmg
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="Spout"
BUNDLE_DIR="$PROJECT_DIR/target/release/${APP_NAME}.app"
DMG_PATH="$PROJECT_DIR/target/release/${APP_NAME}.dmg"

echo "==> Building release binary..."
cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"

echo "==> Creating ${APP_NAME}.app bundle..."
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR/Contents/MacOS"
mkdir -p "$BUNDLE_DIR/Contents/Resources"

# Copy binary
cp "$PROJECT_DIR/target/release/spout" "$BUNDLE_DIR/Contents/MacOS/spout"

# Copy Info.plist
cp "$PROJECT_DIR/macos/Info.plist" "$BUNDLE_DIR/Contents/Info.plist"

# Copy icon if it exists
if [ -f "$PROJECT_DIR/macos/AppIcon.icns" ]; then
    cp "$PROJECT_DIR/macos/AppIcon.icns" "$BUNDLE_DIR/Contents/Resources/AppIcon.icns"
fi

# Ad-hoc code sign (allows running without Gatekeeper issues locally)
echo "==> Ad-hoc signing..."
codesign --force --deep --sign - "$BUNDLE_DIR"

echo "==> Bundle created at: $BUNDLE_DIR"

# Optionally create a .dmg
if [[ "${1:-}" == "--dmg" ]]; then
    echo "==> Creating DMG..."
    rm -f "$DMG_PATH"
    hdiutil create -volname "$APP_NAME" \
        -srcfolder "$BUNDLE_DIR" \
        -ov -format UDZO \
        "$DMG_PATH"
    echo "==> DMG created at: $DMG_PATH"
fi
