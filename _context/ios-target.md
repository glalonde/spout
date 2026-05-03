# iOS Target — Research & Status

**Date:** 2026-05-02
**Status:** RUNNING on device (iPhone 15 Pro). Several fixes needed for full playability.

## Build & Deploy

```bash
# Build for device
SDKROOT=$(xcrun --sdk iphoneos --show-sdk-path) \
  cargo build --release --target aarch64-apple-ios

# Build via Xcode (also signs + packages)
xcodebuild -project ios/Spout.xcodeproj -target Spout \
  -configuration Release -sdk iphoneos \
  -destination 'platform=iOS,id=<device-udid>' build

# Install to paired device
xcrun devicectl device install app --device <udid> \
  ios/build/Release-iphoneos/Spout.app

# Launch
xcrun devicectl device process launch --device <udid> com.glalonde.spout

# List paired devices
xcrun devicectl list devices
```

Geoffrey's iPhone 15 Pro UDID: `92609DCE-F767-5F84-B858-9BA8CF6C37D8`

## Architecture

The Rust binary IS the iOS executable — no static lib refactoring needed.

- `cargo build --target aarch64-apple-ios` produces a Mach-O arm64 binary linked
  against Metal, UIKit, AudioToolbox, CoreAudio via system dylibs.
- `ios/Spout.xcodeproj` compiles a thin ObjC placeholder (`ios/main.m`), then a
  Run Script phase builds the Rust binary and replaces the placeholder in the
  `.app` bundle before Xcode signs it.
- winit 0.30 UIKit backend handles `UIApplicationMain` internally when
  `event_loop.run_app()` is called from Rust `main()`.

### Build script (Xcode Run Script phase)
Set `SDKROOT` from `xcrun` for `coreaudio-sys`/`cpal` cross-compilation.
Uses `$PLATFORM_NAME` (iphoneos vs iphonesimulator) to pick the Cargo target.
Full script embedded in `ios/Spout.xcodeproj/project.pbxproj`.

### Signing
- Debug config: ad-hoc signing (`Sign to Run Locally`) — no developer account needed for simulator.
- Release config: Automatic signing with team `HNRULUX5AH` (Geoffrey Lalonde).
- Provisioning profile: iOS Team Provisioning Profile wildcard, auto-managed.

## Known issues / fixes needed

See `plans/active/near-term.md` §8 for full details. Summary:

| # | Issue | Where to fix |
|---|-------|-------------|
| 8a | Ship escapes left/right edges — no boundary walls | `level_manager.rs` init: set edge columns to max health |
| 8b | Game over shows "press R" — no keyboard on iOS | `main.rs`: tap anywhere on game-over → restart |
| 8c | Music off by default — no M key to toggle on iOS | `game_params.rs`: force `music_starts_on = true` on `cfg(target_os = "ios")` |
| 8d | FPS overlay draws outside game area (safe area / viewport issue) | `framework.rs` / `render.rs`: fix viewport origin on iOS |
| 8e | No in-game settings UI (long term) | See `autonomous-improvement.md` |

## Simulator

No simulator runtime is installed. Install via:
Xcode → Settings → Platforms → iOS → `+` → iOS 18

Then build/run:
```bash
xcodebuild -project ios/Spout.xcodeproj -target Spout \
  -configuration Debug -sdk iphonesimulator \
  -destination 'platform=iOS Simulator,name=iPhone 16' build
xcrun simctl install booted ios/build/Debug-iphonesimulator/Spout.app
xcrun simctl launch booted com.glalonde.spout
```

## Toolchain notes

- Rust targets installed: `aarch64-apple-ios`, `aarch64-apple-ios-sim`
- `coreaudio-sys` requires Xcode (not just CLT) for iOS SDK headers
- Must set `SDKROOT=$(xcrun --sdk iphoneos --show-sdk-path)` before `cargo build`
- `cpal` cross-compiles fine once SDKROOT is set — no workaround needed
- Audio works on device (cpal + CoreAudio)
