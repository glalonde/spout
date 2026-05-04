# Near-Term Features

Concrete next things to build — one step above backlog, not yet in active development.
Items are ordered roughly by dependency (text rendering unlocks most of the rest).

---

## 1. Text Rendering (prerequisite for HUD/UI) — IN PROGRESS (PR #55)

**Blocker for:** score display, game over screen, debug overlay, any menus.

Used `fontdue` (pure Rust font rasterizer) instead of `glyphon` — glyphon doesn't support wgpu 29 yet. Hand-rolled bitmap atlas approach is a better fit for Pixel Six anyway.

Tasks:
- [x] Embed Pixel Six font via `include_bytes!` (already in repo from legacy)
- [x] Implement `TextRenderer` struct (fontdue atlas + instanced glyph quads, WGSL shader)
- [x] FPS counter + score debug overlay (toggle with F3)
- [ ] In-game settings UI (see `autonomous-improvement.md` for design)

---

## 2. Ship Collision Detection — needs improvement

Bresenham triangle rasterization against CPU-side terrain grid shipped in PR #56,
but collision is not pixel-perfect enough in practice — the ship can visually
overlap terrain without dying, or die when it looks like it shouldn't.

Tasks:
- [ ] Audit the collision shape: verify the triangle vertices used match the
  rendered ship geometry exactly (check `ship.rs` fill vertices vs collision
  vertices in `collision.rs`)
- [ ] Consider testing every pixel of the ship outline rather than just triangle
  edges — rasterize the filled triangle and test all interior + border cells
- [ ] Test at different speeds: fast-moving ships may tunnel through 1-cell walls
  between frames; if so, sweep the ship path between frames

---

## 3. Score Display / HUD

**Depends on:** text rendering (#1).

Tasks:
- [ ] Score counter incremented by terrain destruction (particles hitting terrain)
- [x] **Fix: stop score incrementing once ship explodes** — gated on `!state.dead` in `main.rs` (PR #59 / 4cc9000)
- [ ] Render score + lives in a HUD using `TextRenderer`
- [ ] Fuel/energy gauge (visual bar or numeric)

---

## 4. Game Over Screen ✅

Done: title screen → playing → dead → game over → title flow implemented. "GAME OVER" text rendered, 'R' restarts. See PR #56.

---

## 5. Resolution / Aspect Ratio

Independent — no blockers. More urgent now that iOS is running on device.

Currently the game renders at a fixed internal resolution (240×135) upscaled to fill the window. On iPhone 15 Pro (2556×1179, ~19.5:9) the game viewport doesn't fill the screen correctly — letterboxed or wrong aspect.

Tasks:
- [x] Decide internal render resolution strategy → camera letterboxes game quad (261×160) into surface; bars on one axis only.
- [x] Full-screen Metal surface on iOS — see 8d above.
- [ ] Handle window resize gracefully (resize swapchain, update camera projection)
- [ ] Optional: expose resolution config in `game_config.toml`

---

## 6. Music Playlist Randomization ✅

Fixed: `rand` dep removed, `fastrand::Rng::new().shuffle()` used instead. Level terrain also randomized per-run. See `music.md` for details.

---

## 7. Music: Non-Blocking Track Render (WASM)

`render_track` runs synchronously inside `spawn_local` on WASM, blocking the main thread for ~2–4 s while a tracker file decodes. Visible as a freeze when a track loads.

Native is already correct (background thread via `std::thread::spawn`).

Tasks:
- [ ] Move `render_track` to a Web Worker on WASM (see `music.md` Phase 3 for options)
- [ ] Verify no main-thread freeze when starting / cycling tracks in the browser

---

## 8. iOS Platform Fixes

Observed on first run on iPhone 15 Pro (2026-05-02). The game launches and renders
but needs these fixes before it's properly playable on device.

### 8a. Horizontal boundary death ✅
Fixed in 4cc9000: max-health cells set in leftmost/rightmost terrain columns.
Secondary guard also kills ship on boundary exit (`c372de8`).

### 8b. Touch-to-restart ✅
Fixed in 4cc9000: tap anywhere on game over screen restarts; game over text
says "TAP TO RESTART" on iOS (cfg-gated).

### 8c. Music on by default on iOS ✅
Fixed in 4cc9000: `cfg(target_os = "ios")` override forces `music_starts_on = true`.

### 8d. Full-screen surface + letterboxing ✅
Root cause: missing `UILaunchScreen` key in `ios/Info.plist`. Without it, iOS
treats the app as a legacy app and runs it in 480×320-point compatibility mode
(1440×960 px at 3×), centered on the real screen with massive OS-level black
bars that don't pass touch events. Fixed by adding `<key>UILaunchScreen</key><dict/>`
to `ios/Info.plist`. Also added:
- `framework.rs` `init_gpu`: `window.outer_size()` on iOS (safe-area guard) + logging.
- `framework.rs` `resumed`: landscape lock, hide status bar + home indicator.

### 8e. In-game settings overlay (longer term)
See `autonomous-improvement.md` for full design. Lower priority.

---

## 9. macOS App Packaging — needs rebase + merge

Work is complete on the `macos-packaging` branch (diverged from master at
`a9fa746`, ~23 commits behind). Needs rebase onto master and a PR.

What's on that branch:
- `macos/Info.plist` — bundle metadata (com.glalonde.spout, Metal, macOS 13+)
- `scripts/package_macos.sh` — builds release binary, assembles .app, ad-hoc
  or Developer ID signs, optional --dmg flag
- `.github/workflows/release-macos.yml` — triggered on `v*` tags or manual
  dispatch; uploads .app + .dmg as release artifacts
- `_context/macos-signing.md` — signing/notarization docs (team HNRULUX5AH)

Steps:
- [ ] `git checkout macos-packaging && git rebase master`
- [ ] Resolve any conflicts (likely none — touches different files)
- [ ] Open PR and merge

---

## 10. App Icon

All three platforms need a proper icon. Currently only `assets/spout_preview.png`
exists (a gameplay screenshot, not a real icon).

Design brief: the icon should evoke the game — a ship firing particles into
terrain. Simple, bold, readable at small sizes. The particle glow/bloom
aesthetic should translate to the icon.

### macOS (.icns)
- Source: 1024×1024 PNG → `iconutil` generates the `.icns`
- Goes in `macos/` directory, referenced from `macos/Info.plist`
- `scripts/package_macos.sh` copies it into the `.app` bundle
- Sizes needed in the `.iconset`: 16, 32, 128, 256, 512 px (+ @2x variants)

### iOS (asset catalog or individual PNGs)
- Required sizes for iOS app icon: 60×60, 120×120, 180×180 px (iPhone)
  and 76×76, 152×152, 167×167 px (iPad)
- Plus 1024×1024 for App Store
- Goes in `ios/` as an `AppIcon.appiconset` (Xcode asset catalog) or listed
  in `Info.plist` under `CFBundleIcons`
- Add `ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon` to Xcode build settings

### Favicon (web / WASM)
- 32×32 and 180×180 PNG (for apple-touch-icon) + `favicon.ico`
- Goes in `web/` or wherever the gh-pages build drops static files

### Suggested workflow
1. Create master 1024×1024 artwork (Figma, Pixelmator, or procgen from game)
2. Export PNGs → `scripts/generate_icons.sh` automates resizing with `sips`
3. `iconutil --convert icns` for macOS
4. Drop icon PNGs into iOS asset catalog

---

## 11. Virtual Joystick / Touch Visual Feedback

Touch controls currently give no visual indication of where the thrust/rotate zones are or whether input is registered. A virtual joystick or translucent on-screen control overlay would make the iOS experience significantly more discoverable.

Options to explore:
- [ ] Simple semi-transparent zone indicators (left = rotate, right = thrust) drawn as HUD overlay
- [ ] Virtual joystick: a thumb origin + knob that tracks the touch, emits thrust in the joystick direction
- [ ] Haptic feedback on thrust (iOS `UIImpactFeedbackGenerator`) — requires FFI or a winit hook

---

## 12. Density-Based Fluid Dynamics for Particles

Replace the current per-particle gravity + elasticity model with a grid-based fluid sim (e.g. SPH or a simple Eulerian pressure solve on the particle density field). Would make the particle exhaust behave more like a pressurized gas — spreading laterally, building up in enclosed spaces, etc.

Notes:
- The existing density grid (used for rendering/collision) is a natural starting point for a pressure field
- A full Navier-Stokes solve is expensive; even a simple divergence-correction pass would add believable bulk flow
- Interaction between fluid pressure and terrain destruction is the interesting design space

---

## 13. Damage-Accumulation Glow on Terrain

Currently terrain cells either glow at full intensity (edge highlight) or don't. Cells should brighten progressively as they accumulate damage, giving the player clear feedback about which areas are weakened before they finally break.

Tasks:
- [ ] Pass per-cell health as a normalized value into the render shader (health / starting_health)
- [ ] Map damage fraction to an additive glow: undamaged = no glow, half-health = dim, near-zero = bright
- [ ] Ensure the glow blends with the existing bloom pipeline so heavily damaged terrain pulses visibly
- [ ] Consider a color shift (e.g. white → orange → red) as cells approach destruction

---

## Branch Audit Remainder

The `legacy_wgpu` branch has been audited (see `legacy-port-inventory.md`). These branches have not yet been reviewed:

- [ ] `glalonde-emitoverangle` — likely emit-over-angle feature for the particle emitter
- [ ] `glalonde-fpbresen` — likely Bresenham/fixed-point line drawing
- [ ] `glalonde-glslrand` — likely GLSL random number utilities

For each: fetch the branch, identify what it adds vs master, document in `legacy-port-inventory.md` or discard.
