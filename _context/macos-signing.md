# macOS Code Signing & Notarization

Currently using ad-hoc signing (`codesign -s -`). This works for local use
but unsigned apps trigger Gatekeeper warnings for other users.

## When an Apple Developer account is available

### Repository secrets needed

| Secret | Description |
|--------|-------------|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` Developer ID Application certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12` file |
| `APPLE_TEAM_ID` | 10-character Apple Developer Team ID |
| `APPLE_ID` | Apple ID email for notarization |
| `APPLE_ID_PASSWORD` | App-specific password (generate at appleid.apple.com) |

### Signing steps (replace ad-hoc in `package_macos.sh`)

```bash
# Import certificate into a temporary keychain
security create-keychain -p "" build.keychain
security import "$CERTIFICATE_PATH" -k build.keychain -P "$PASSWORD" -T /usr/bin/codesign
security set-key-partition-list -S apple-tool:,apple: -k "" build.keychain
security list-keychains -d user -s build.keychain

# Sign with Developer ID
codesign --deep --force --verify --verbose \
    --sign "Developer ID Application: Your Name ($TEAM_ID)" \
    --options runtime \
    "$BUNDLE_DIR"
```

### Notarization

```bash
# Submit for notarization
xcrun notarytool submit "$DMG_PATH" \
    --apple-id "$APPLE_ID" \
    --password "$APPLE_ID_PASSWORD" \
    --team-id "$TEAM_ID" \
    --wait

# Staple the ticket
xcrun stapler staple "$DMG_PATH"
```

### CI workflow changes

Update `.github/workflows/release-macos.yml` to:
1. Decode and import the certificate from secrets
2. Replace `codesign -s -` with the Developer ID signing command
3. Add notarization step after DMG creation
4. Clean up the temporary keychain
