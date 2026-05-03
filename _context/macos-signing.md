# macOS Code Signing & Notarization

**Team ID:** HNRULUX5AH

## Local signing

`scripts/package_macos.sh` auto-detects a "Developer ID Application" identity
in your keychain. If found, it signs with hardened runtime (required for
notarization). Otherwise it falls back to ad-hoc signing.

### Setup (one-time)

1. Open Xcode → Settings → Accounts → Manage Certificates
2. Create a "Developer ID Application" certificate
3. It will be installed in your login keychain automatically
4. Run `security find-identity -v -p codesigning` to verify it shows up

### Local notarization

After building a signed DMG:
```bash
xcrun notarytool submit target/release/Spout.dmg \
    --apple-id YOUR_APPLE_ID \
    --password YOUR_APP_SPECIFIC_PASSWORD \
    --team-id HNRULUX5AH \
    --wait

xcrun stapler staple target/release/Spout.dmg
```

Generate an app-specific password at https://appleid.apple.com/account/manage

## CI signing (GitHub Actions)

The release workflow (`.github/workflows/release-macos.yml`) supports signing
and notarization when the following repository secrets are configured:

### Required secrets

| Secret | Description |
|--------|-------------|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` Developer ID Application certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password used when exporting the `.p12` file |
| `APPLE_ID` | Apple ID email for notarization |
| `APPLE_ID_PASSWORD` | App-specific password (not your Apple ID password) |

### Exporting the certificate

```bash
# Export from Keychain Access → My Certificates → Developer ID Application
# Choose .p12 format, set a password

# Base64-encode for the GitHub secret:
base64 -i DeveloperID.p12 | pbcopy
# Paste into the APPLE_CERTIFICATE secret
```

### How it works

1. CI imports the `.p12` into a temporary keychain
2. `package_macos.sh` finds the Developer ID identity and signs with hardened runtime
3. CI notarizes the DMG via `notarytool` and staples the ticket
4. Temporary keychain is cleaned up

Without secrets, the workflow still runs — it just produces an ad-hoc signed
build (fine for testing, not distributable without Gatekeeper warnings).
