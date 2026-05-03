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

## 2. Ship Collision Detection ✅

Done: Bresenham triangle rasterization against CPU-side terrain grid. Ship dies on contact, explosion particle burst, dead-state physics. See PR #56.

---

## 3. Score Display / HUD

**Depends on:** text rendering (#1).

Tasks:
- [ ] Score counter incremented by terrain destruction (particles hitting terrain)
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
- [ ] Decide internal render resolution strategy (fixed game viewport with letterboxing vs. stretch vs. dynamic)
- [ ] Handle window resize gracefully (resize swapchain, update camera projection)
- [ ] On iOS: match viewport to device screen aspect ratio (iPhone 15 Pro is ~19.5:9, game is currently 16:9 ish)
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

### 8a. Horizontal boundary death (also affects desktop)
The ship can fly off the left/right edges of the level and disappear. Need
unbreakable solid walls at x=0 and x=level_width. Options:
- Add wall cells to the terrain grid that have max health and never erode
- Clamp ship x-position and treat the boundary as a kill zone
- Easiest: in `level_manager.rs`, set the leftmost and rightmost column of each
  level chunk to max terrain health on init. Ship collision already handles this.

### 8b. Touch-to-restart (no keyboard on iOS)
Game over screen says "press R to restart" but iOS has no keyboard. Need a
touch input path:
- A tap anywhere on game-over screen should restart (same as 'R')
- In `main.rs`, check `GameMode::GameOver` + `WindowEvent::Touch` with
  `phase == TouchPhase::Started` → set `reset_requested = true`

### 8c. Music on by default on iOS
`game_config.toml` has `music_starts_on = false`. On iOS there's no easy way
to toggle music without a keyboard (M key). For now, default to on when
`cfg(target_os = "ios")`. Add `#[cfg(target_os = "ios")] { params.music_starts_on = true; }`
in `game_params::get_game_config_from_default_file()` after loading, or use a
separate embedded config for iOS.

### 8d. FPS overlay drawn outside the game viewport
The `overlay_text` renderer (display-resolution text) appears above the game
area rather than overlaid on it. On iOS the safe area insets may be pushing
the viewport down. The overlay text position needs to be relative to the actual
rendered game region, or the viewport/surface setup needs fixing for iOS safe
areas. Related to #5 (aspect ratio). Investigate `framework.rs` surface setup.

### 8e. In-game settings overlay (longer term)
See `autonomous-improvement.md` for full design. Needed so players can adjust
params without editing `game_config.toml`. Lower priority than 8a–8d.

---

## Branch Audit Remainder

The `legacy_wgpu` branch has been audited (see `legacy-port-inventory.md`). These branches have not yet been reviewed:

- [ ] `glalonde-emitoverangle` — likely emit-over-angle feature for the particle emitter
- [ ] `glalonde-fpbresen` — likely Bresenham/fixed-point line drawing
- [ ] `glalonde-glslrand` — likely GLSL random number utilities

For each: fetch the branch, identify what it adds vs master, document in `legacy-port-inventory.md` or discard.
