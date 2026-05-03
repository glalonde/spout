#!/usr/bin/env bash
#
# Build and package Spout as a macOS .app bundle.
# Optionally creates a .dmg disk image.
#
# Usage:
#   ./scripts/package_macos.sh           # build + .app bundle (ad-hoc sign)
#   ./scripts/package_macos.sh --dmg     # also create .dmg
#
# Code signing:
#   The script looks for a "Developer ID Application" identity in the keychain.
#   If found, it signs with that identity + hardened runtime (required for
#   notarization). Otherwise it falls back to ad-hoc signing.
#
# Notarization (after signing with Developer ID):
#   xcrun notarytool submit target/release/Spout.dmg \
#       --apple-id YOUR_APPLE_ID --password APP_SPECIFIC_PASSWORD \
#       --team-id HNRULUX5AH --wait
#   xcrun stapler staple target/release/Spout.dmg
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

# Code signing: use Developer ID if available, otherwise ad-hoc.
SIGN_IDENTITY=""
if security find-identity -v -p codesigning 2>/dev/null | grep -q "Developer ID Application"; then
    SIGN_IDENTITY=$(security find-identity -v -p codesigning 2>/dev/null \
        | grep "Developer ID Application" | head -1 | sed 's/.*"\(.*\)"/\1/')
    echo "==> Signing with: $SIGN_IDENTITY"
    codesign --force --deep --verify --verbose \
        --sign "$SIGN_IDENTITY" \
        --options runtime \
        "$BUNDLE_DIR"
else
    echo "==> No Developer ID found — ad-hoc signing"
    codesign --force --deep --sign - "$BUNDLE_DIR"
fi

echo "==> Bundle created at: $BUNDLE_DIR"

# Optionally create a .dmg
if [[ "${1:-}" == "--dmg" ]]; then
    echo "==> Creating DMG..."
    # Detach any existing mount with this volume name before creating.
    hdiutil detach "/Volumes/$APP_NAME" -force 2>/dev/null || true
    rm -f "$DMG_PATH"
    hdiutil create -volname "$APP_NAME" \
        -srcfolder "$BUNDLE_DIR" \
        -ov -format UDZO \
        "$DMG_PATH"

    # Sign the DMG too if we have a Developer ID
    if [ -n "$SIGN_IDENTITY" ]; then
        codesign --force --sign "$SIGN_IDENTITY" "$DMG_PATH"
    fi

    echo "==> DMG created at: $DMG_PATH"
fi
