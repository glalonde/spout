# iOS Target — Research & Status

**Date:** 2026-03-26
**Status:** Scaffolding only — requires Xcode with iOS SDK to build

## Toolchain

### Rust targets
```bash
rustup target add aarch64-apple-ios      # device
rustup target add aarch64-apple-ios-sim  # simulator (Apple Silicon)
```

### Build command
```bash
# Requires Xcode with iOS SDK installed
cargo build --target aarch64-apple-ios --lib
```

### Known issue: cpal cross-compilation

`cpal` (audio) depends on `coreaudio-sys`, which uses `bindgen` to generate FFI
bindings from iOS SDK headers. This requires:
1. Xcode with iOS platform support (`xcode-select --install` is not enough)
2. The `SDKROOT` environment variable or correct sysroot:
   ```bash
   export SDKROOT=$(xcrun --sdk iphoneos --show-sdk-path)
   ```

If `cpal` continues to fail cross-compilation, options:
- Use `#[cfg(not(target_os = "ios"))]` to disable native audio on iOS and use
  a Web Audio-like approach through `AVAudioEngine` via Swift bridge
- Pin a `cpal` version known to cross-compile for iOS
- Use `oboe` or another iOS-friendly audio crate

## Architecture decisions

### Window / GPU
- **winit 0.30** has iOS support via the `UIKit` backend. It handles `UIWindow`
  creation and delivers touch events. The Rust binary can be a static library
  linked into a minimal Xcode project.
- **wgpu** uses Metal on iOS — same backend as macOS. No Vulkan on iOS.
- The game already runs on Metal via wgpu on macOS, so GPU code should work
  unchanged.

### Touch input
- winit on iOS delivers `WindowEvent::Touch` events, same as on other platforms.
- The existing touch/accelerometer code is `#[cfg(target_arch = "wasm32")]` and
  won't compile on iOS. iOS touch needs its own path:
  - winit touch events work natively (no DOM/canvas needed)
  - Accelerometer needs `CoreMotion` via Swift/ObjC bridge or a Rust crate
  - For now, keyboard (Bluetooth) and winit touch are sufficient

### Audio
- `cpal` supports iOS via CoreAudio, but cross-compilation is tricky (see above).
- Alternative: use `AVAudioSession` setup in Swift, pass audio context to Rust.
- The `oxdz` tracker renderer is pure Rust and will work fine on iOS.

### Config
- `game_config.toml` is embedded via `include_str!` — works on iOS without
  filesystem access.

## Xcode project structure (planned)

```
ios/
  Spout.xcodeproj/
  Spout/
    AppDelegate.swift      — minimal app delegate
    Info.plist             — bundle ID, landscape-only, Metal required
    LaunchScreen.storyboard
  libspout.a               — Rust static library (built separately)
```

### Build flow
1. `cargo build --release --target aarch64-apple-ios --lib` → `target/aarch64-apple-ios/release/libspout.a`
2. Xcode project links against `libspout.a`
3. Swift `AppDelegate` creates window, winit takes over from there
4. `xcodebuild` builds the `.app` for device/simulator

## Next steps

1. Install Xcode with iOS platform support
2. Verify `cargo build --target aarch64-apple-ios --lib` succeeds with `SDKROOT` set
3. Create minimal Xcode project scaffold
4. Add `scripts/build_ios.sh` automation
5. Resolve `cpal` cross-compilation or add iOS audio path
6. Test on simulator, then device
