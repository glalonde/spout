# Autonomous Codebase Improvement — Agent Instructions

You are an autonomous agent tasked with continuously improving the Spout codebase
over an extended session (~8 hours) with no human input. You will work in a loop:
identify an improvement, implement it, verify it, commit it, and move on.

---

## Setup

1. Read `AGENTS.md` for build commands, architecture, and constraints.
2. Read `_context/README.md` and all files in `_context/plans/active/`.
3. Read `_context/wasm-debugging.md` before touching anything WASM/shader-related.
4. Run `cargo build && cargo clippy -- -D warnings && cargo test --verbose` to
   confirm a clean baseline. Do not proceed if the baseline is broken — fix it first.
5. Create a tracking branch: `auto-improve-YYYY-MM-DD` off `master`.

---

## The Loop

Repeat until time is exhausted:

### 1. Pick the highest-value task

Choose from the categories below, in roughly this priority order. Prefer tasks
that are self-contained and can be verified without a human in the loop.

**A. Bug fixes and correctness**
- Run `cargo clippy -- -D warnings` and fix any new warnings.
- Run `cargo test --verbose` and fix any failures.
- Grep for `unwrap()` calls without justification comments — add comments or
  replace with proper error handling.
- Grep for `todo!()`, `unimplemented!()`, `fixme`, `hack` — resolve if possible.
- Check for integer overflow, division-by-zero, or panic-on-edge-case risks.

**B. Code quality and deduplication**
- Look for duplicated logic across modules (the audio refactor was a good example).
- Look for functions over ~80 lines that could be broken up.
- Look for `pub` items that don't need to be public.
- Look for dead code (`#[allow(dead_code)]`, unused imports, unreachable branches).
- Simplify overly complex control flow.

**C. Performance**
- Profile-guided: look for per-frame allocations (`Vec::new()` in hot loops),
  unnecessary clones, or redundant GPU state changes.
- Look for `String` allocations that could be `&str`.
- Check shader code for unnecessary branches or redundant calculations.
- Look for opportunities to batch GPU operations.

**D. Test coverage**
- Add unit tests for pure functions that lack them.
- Add property-based tests where appropriate (e.g., `angle_diff` wrapping).
- Add integration tests that catch regressions (like the WASM export validation).
- Test edge cases: zero-length inputs, boundary values, wraparound.

**E. Documentation and developer experience**
- Add module-level doc comments (`//!`) to files that lack them.
- Improve error messages (replace generic "failed" with context about what failed).
- Update `_context/` files if you discover they are stale.

**F. Feature work from `near-term.md`**
Only after A–E are well-covered. Prefer features that don't require visual
verification:
- FPS estimator (pure logic, testable).
- Ship collision detection against CPU-side terrain grid (testable).
- Text rendering setup (glyphon integration — can verify it compiles and renders
  to a texture in a test, even without visual inspection).

### 2. Implement the change

- Work on one logical change at a time. Keep commits small and focused.
- Create a new branch for each logical unit if it's more than a trivial fix.
  For small fixes, batch related ones on the tracking branch.
- Follow all constraints in `AGENTS.md`:
  - No `unwrap()` without a comment.
  - `cargo fmt --all` before every commit.
  - `cargo clippy -- -D warnings` must pass.
  - `cargo test --verbose` must pass.
- Do not modify `Cargo.lock` unless adding/removing a dependency.
- Do not upgrade existing dependency versions.

### 3. Verify the change

**Every single commit must pass this gate:**

```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test --verbose
```

If any step fails, fix it before committing. Do not commit broken code.

For WASM-touching changes, also run:
```bash
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --target wasm32-unknown-unknown
```

### 4. Commit with a clear message

```
<type>: <concise summary>

<why this change matters — 1-3 sentences>

Co-Authored-By: Claude <noreply@anthropic.com>
```

Types: `fix`, `refactor`, `perf`, `test`, `docs`, `feat`

### 5. Push and open a draft PR

For each logical unit of work (a single feature, a batch of related fixes):
- Push the branch: `git push -u origin <branch-name>`
- Open a draft PR: `gh pr create --draft --title "..." --body "..."`
- This lets the owner review from anywhere. Keep PRs focused — one per topic.

### 6. Log what you did

After each commit, append a one-line summary to a running log at the top of
this file (or a separate `_context/auto-improve-log.md`). Include:
- Timestamp
- Commit hash (short)
- What changed and why
- PR number if one was opened

This log helps the human review your work efficiently when they return.

### 7. Move on

Pick the next task. Don't over-polish — if a change is good enough and passes
verification, ship it and move on. Breadth over depth.

---

## Rules of engagement

### DO
- Read code before changing it. Understand the context.
- Make the smallest change that achieves the goal.
- Run the full verification suite after every change.
- Commit frequently — one logical change per commit.
- Update `_context/` docs when you find stale information.
- Trust the existing architecture unless you find a clear bug.

### DO NOT
- Add new dependencies without strong justification.
- Refactor code that's working fine just because you'd write it differently.
- Make "drive-by" style changes in unrelated files within the same commit.
- Modify `game_config.toml` defaults (gameplay tuning is human work).
- Change visual parameters (colors, bloom strength, etc.) — these require eyes.
- Push to `master` directly. Push feature branches and open draft PRs for review.
- Delete files or remove features without understanding why they exist.
- Introduce new `unsafe` code.
- Add heavyweight abstractions (traits, generics, builders) for things that are
  currently simple and only used in one place.
- Spend more than ~30 minutes stuck on one problem. Log it, skip it, move on.

### WASM-specific caution
- Always use `web_time` instead of `std::time`.
- Never use `pollster::block_on` on WASM paths.
- `textureSample` → `textureSampleLevel` after non-uniform control flow.
- Test the WASM build compiles even if you can't run it headlessly.

---

## Suggested starting sequence

If you're unsure where to begin, this order works well:

1. **Clippy audit** — fix all warnings, tighten code.
2. **Unwrap audit** — find unjustified `unwrap()` calls, handle errors properly.
3. **Dead code removal** — find and remove unused functions, imports, modules.
4. **Test coverage** — add tests for `ship.rs` physics, `int_grid` edge cases,
   `input.rs` state machine, `game_params.rs` parsing.
5. **Module doc comments** — add `//!` headers to files missing them.
6. **Performance scan** — look for per-frame allocations in `main.rs::render()`,
   `particles.rs::run_compute()`, `level_manager.rs::sync_height()`.
7. **FPS estimator** — small self-contained feature from `near-term.md`.
8. **Collision detection** — bigger feature, but CPU-side and testable.
9. **macOS app packaging + GitHub Actions release** — see below.
10. **iOS app target** — see below.

---

## Bigger projects

These are multi-step efforts. Break them into small, independently committable
pieces. Each piece should leave the repo in a working state.

### macOS native app packaging + CI release

**Goal:** `cargo build --release` produces a proper `.app` bundle that can be
distributed, code-signed, notarized, and released via GitHub Releases.

**Steps:**

1. **App bundle structure** — Create a `macos/` directory with:
   - `Info.plist` template (bundle identifier `com.glalonde.spout`, display name,
     version from `Cargo.toml`, minimum macOS version, GPU capability keys).
   - An `AppIcon.icns` icon set (convert the existing `assets/spout_preview.png`
     or create a simple one with `iconutil`).
   - A shell script or `cargo-bundle` config that assembles
     `Spout.app/Contents/{MacOS/spout, Resources/*, Info.plist}`.

2. **cargo-bundle or manual bundling** — Evaluate `cargo-bundle` (adds
   `[package.metadata.bundle]` to `Cargo.toml`). If it works cleanly for macOS,
   use it. Otherwise, write a small `scripts/package_macos.sh` that:
   - Runs `cargo build --release`
   - Creates the `.app` directory structure
   - Copies the binary, assets, and `Info.plist`
   - Optionally creates a `.dmg` with `hdiutil`

3. **GitHub Actions workflow** — Create `.github/workflows/release-macos.yml`:
   - Trigger: push of a version tag (`v*`) or manual `workflow_dispatch`.
   - Runner: `macos-latest` (Apple Silicon).
   - Steps: checkout → install Rust toolchain → `cargo build --release` →
     run packaging script → create `.dmg` → upload as GitHub Release artifact.
   - Use `softproof/turnstyle` or similar to avoid concurrent release builds.

4. **Code signing** — No Apple Developer account yet. Use ad-hoc signing
   (`codesign -s -`) so the app runs locally without Gatekeeper issues.
   Document the full signing/notarization steps in a comment or `_context/`
   file so they're easy to wire up later when an account is available:
   - Repository secrets needed: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`,
     `APPLE_TEAM_ID`
   - `codesign --deep --force --verify --verbose --sign`
   - `xcrun notarytool submit` + `xcrun stapler staple`

5. **Smoke test** — Add a CI step that launches the app headlessly (or at least
   verifies the binary starts and exits cleanly with `--help` or a timeout).

**Key considerations:**
- The game uses Metal on macOS (via wgpu). Ensure `Info.plist` doesn't request
  OpenGL-only capabilities.
- `assets/` files are loaded via `include_bytes!` so they're baked into the
  binary — no need to bundle them separately in Resources unless that changes.
- Music files are also `include_bytes!` — no external assets needed.
- `game_config.toml` is currently loaded at runtime from CWD. This breaks in
  `.app` bundles and on iOS. **Prerequisite:** embed `game_config.toml` via
  `include_str!` as the default config, with an optional runtime override if a
  file exists on disk. This should be done early (before packaging work) since
  it affects `game_params.rs` and is needed by both macOS and iOS targets.
  The in-game settings UI (see text rendering section) will eventually replace
  the need for users to hand-edit the file.

### Text rendering + in-game settings UI

**Goal:** Render text on screen using the Pixel Six bitmap font. Use it to build
a debug overlay (FPS counter) and an in-game settings menu that lets the player
tweak parameters without editing `game_config.toml`.

**Font:** `pixelsix00.ttf` — a pixel/bitmap font from the legacy branch
(`assets/fonts/pixelsix00.ttf`). Copy it from the `legacy_wgpu` branch into
`assets/fonts/` on master. Embed via `include_bytes!`.

**Steps:**

1. **Copy font asset from legacy** —
   ```bash
   git checkout origin/legacy_wgpu -- assets/fonts/pixelsix00.ttf
   ```
   Commit the font file on its own so the diff is clean.

2. **Add `glyphon` dependency** — [`glyphon`](https://github.com/grovesNL/glyphon)
   is the recommended wgpu text renderer. Add it to `Cargo.toml`. Verify it
   supports wgpu 29 — check the glyphon release notes / `Cargo.toml` for
   compatible versions. If glyphon doesn't support wgpu 29 yet, evaluate
   alternatives:
   - `wgpu_text` — another wgpu text crate
   - Manual bitmap font atlas — Pixel Six is an 8px bitmap font, so a hand-rolled
     atlas (texture with glyph grid + UV lookup) is feasible and avoids the
     dependency entirely. This may actually be the better approach for a pixel
     font since glyphon is designed for vector/TTF rasterization.

3. **`TextRenderer` module** — Create `src/text.rs`:
   - `TextRenderer::init(device, queue, surface_format)` — loads the font,
     builds the glyph atlas or glyphon pipeline.
   - `TextRenderer::draw(encoder, target_view, texts: &[TextEntry])` — renders
     a list of positioned text entries in a single render pass.
   - `TextEntry { text: &str, x: f32, y: f32, color: [f32; 4], scale: f32 }`
   - The renderer should draw on top of the final composited frame (after bloom
     + CRT), so text is always crisp and not affected by post-processing.
   - Register the module in `src/lib.rs`.

4. **Debug overlay** — Create `src/debug_overlay.rs`:
   - FPS counter using a rolling window of frame times (last 60 frames).
   - Render in the top-left corner: `"FPS: 60"` in small Pixel Six text.
   - Toggle with a key (e.g., backtick `` ` `` or `F3`).
   - Add the key to `InputState` and handle in `main.rs`.

5. **In-game settings UI** — Create `src/settings_ui.rs`:
   - Toggle with `Escape` key (or a dedicated key like `F2`).
   - When open, the game pauses and a semi-transparent overlay appears.
   - Render a list of tweakable parameters with their current values:
     - **Visual:** `bloom_threshold`, `bloom_strength`, `bloom_passes`,
       `crt_strength`, `color_map`
     - **Particle:** `emission_rate`, `emission_speed`, `max_particle_life`,
       `gravity`, `elasticity`
     - **Ship:** `acceleration`, `rotation_rate`, `max_speed`
     - **Audio:** `music_starts_on` (toggle)
   - Navigation: Up/Down arrows to select a parameter, Left/Right to adjust it.
   - Each parameter needs: display name, current value, min, max, step size.
   - Changes apply immediately to `GameParams` (live preview).
   - Optional: "Save" option that writes current values back to `game_config.toml`.
   - Optional: "Reset" option that reloads defaults from the file.

6. **Integration into the render pipeline** — In `main.rs::render()`:
   - After the final `renderer.render(view, &mut encoder)` call, add
     `text_renderer.draw(...)` for the debug overlay and/or settings UI.
   - The text pass renders directly to the surface view, not the game view
     texture (so it's at display resolution, not 240x135).

7. **Wire up parameter changes** — When the settings UI modifies a `GameParams`
   field, some subsystems need to be notified:
   - `bloom_threshold/strength/passes` → rebuild bloom pipeline or update uniforms
   - `crt_strength` → update CRT shader uniform
   - `color_map` → update particle shader uniform
   - `emission_rate/speed/life` → update particle system params
   - `ship params` → take effect immediately (they're read each frame)
   - Check which params require GPU pipeline rebuilds vs. just uniform updates.

**Key considerations:**
- Pixel Six is a bitmap font designed for small sizes (8px, 10px, 12px). Don't
  scale it to large sizes — it will look blurry. Use integer multiples (1x, 2x,
  3x) for crisp scaling.
- The game renders at 240x135 internally but displays at the window's resolution.
  Text should render at the display resolution for crispness — do not render text
  into the 240x135 game view.
- On WASM, the settings UI should also work with touch input (tap to select,
  swipe to adjust). This can be a follow-up; keyboard-only is fine initially.
- The settings UI state (open/closed, selected item) belongs in `GameState`,
  not in the renderer.
- Consider using the same text renderer for a future score display / HUD, so
  design the API to be reusable.

---

### iOS app target

**Goal:** Build and run Spout on iOS (iPhone/iPad) as a native app using the
same Rust codebase. This is a research + scaffolding task — getting it to compile
and render a frame is the milestone, not App Store submission.

**Steps:**

1. **Research and document the toolchain** — Write a `_context/ios-target.md`:
   - Rust cross-compilation: `rustup target add aarch64-apple-ios`
   - wgpu on iOS: uses Metal, should work but verify version compatibility.
   - winit on iOS: winit 0.30 has iOS support via `UIKit` backend. Document
     any required feature flags or `Cargo.toml` changes.
   - Audio: `cpal` supports iOS via CoreAudio — verify it works or identify
     the alternative.

2. **Xcode project scaffold** — Create an `ios/` directory with:
   - A minimal Xcode project that hosts the Rust binary as a static library.
   - A Swift/ObjC app delegate that creates a `UIWindow`, gets a `CAMetalLayer`,
     and hands it to the Rust code.
   - Alternatively, investigate if winit's iOS support handles window creation
     entirely, and the Xcode project just wraps the binary.
   - `Info.plist` with required device capabilities (`metal`), orientations
     (landscape), and bundle identifier.

3. **Build system integration** — Create `scripts/build_ios.sh`:
   - Cross-compile: `cargo build --release --target aarch64-apple-ios`
   - Generate the static library (or use `cargo-lipo` for fat binaries if
     simulator support is desired: `aarch64-apple-ios-sim`).
   - Copy the `.a` into the Xcode project's expected location.
   - Build the Xcode project with `xcodebuild`.

4. **Platform-specific adaptations** — Identify and fix compile errors:
   - Touch input: already implemented via `winit::event::Touch` — verify it
     works on iOS or needs `UITouch`-specific handling.
   - Accelerometer: `DeviceOrientationEvent` is web-only. iOS needs
     `CoreMotion` via a Rust FFI binding or a Swift bridge. Stub it out
     initially (accelerometer steering disabled on iOS).
   - Audio: verify `cpal` + CoreAudio works on iOS. May need
     `AVAudioSession` configuration (category, activation) before playback.
   - File paths: `game_config.toml` loading needs to work from the app bundle.
     Since it's currently loaded from CWD, either embed it or use
     `NSBundle.mainBundle.resourcePath`.

5. **CI workflow (stretch)** — `.github/workflows/build-ios.yml`:
   - Runner: `macos-latest` (has Xcode).
   - Cross-compile the Rust library for `aarch64-apple-ios`.
   - Build the Xcode project with `xcodebuild build` (no signing, just verify
     it compiles).
   - This ensures the iOS target doesn't bitrot.

**Key considerations:**
- iOS requires landscape-only orientation lock for this game. Set
  `UISupportedInterfaceOrientations` to landscape in `Info.plist`.
- Metal is mandatory on iOS (no Vulkan). wgpu handles this transparently.
- The touch input system already handles mobile touch — the main work is
  ensuring winit's iOS backend delivers the same events as the web backend.
- App Store submission is out of scope. Focus on "it builds and runs on a
  device/simulator."
- The `DeviceOrientationEvent` accelerometer code is `#[cfg(target_arch = "wasm32")]`
  so it won't interfere, but iOS accelerometer will need its own `cfg` block
  eventually.

---

## When to stop

- When you've exhausted the time budget.
- When you can't find improvements that pass the "DO NOT" rules above.
- When remaining work requires visual verification or human judgment.

Before stopping:
1. Ensure all branches have clean commits (no WIP).
2. Update `_context/plans/active/near-term.md` to reflect completed items.
3. Write a summary of everything you did in `_context/auto-improve-log.md`.
4. Run the full verification suite one final time.
