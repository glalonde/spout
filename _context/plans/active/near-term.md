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
- [x] Audit the collision shape: `collision.wgsl` was missing the tail-notch
  vertex (-5,0) — hull was a 3-vertex triangle instead of the 4-vertex chevron
  outline rendered by `ship.wgsl`. Fixed: HULL_VERTS updated to 4 verts.
- [x] Dense edge sampling: replaced 3-vert + 3-midpoint approach with sampling
  each of the 4 edges at ~2 world-unit intervals (~32 sample points total).
  Each sample is still Bresenham-swept from prev→curr frame position.
- [ ] Test at different speeds: fast-moving ships may tunnel through 1-cell walls
  between frames; the per-point sweep handles moderate speeds but extreme
  velocities could still tunnel if the swept distance exceeds cell size.

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

## 10. App Icon ✅

Done in PR #63. Source: in-game screenshot of ship (blue pixel-art triangle +
fire thrust), cropped to 293×293.

- `macos/AppIcon.icns` — full iconset 16–1024px; picked up by `scripts/package_macos.sh`
- `ios/Assets.xcassets/AppIcon.appiconset/` — all iPhone/iPad sizes + Contents.json;
  Xcode project wired up with Resources build phase + `ASSETCATALOG_COMPILER_APPICON_NAME`
- `wasm-resources/favicon.ico` + `favicon.png`; `index.template.html` updated with `<link rel="icon">`

---

## 11. Touch Controls — Visual Feedback & Layout Options

Touch controls currently give no visual indication of where the thrust/rotate zones are or whether input is registered. The control layout should be configurable; at least two schemes are worth prototyping.

### Option A: Triangle Split (implemented, try on device)

The right half is divided by a diagonal from top-center `(W/2, 0)` to bottom-right `(W, H)`:
- **Upper-right triangle** → rotate CW (`rotate = -1.0`)
- **Lower-left triangle** → rotate CCW (`rotate = +1.0`)
- Left half → thrust (unchanged)

No dead zone. A single touch anywhere in the right half rotates; dragging across the diagonal switches direction instantly. Direction is computed from the current touch position, not where the touch started.

CW condition: `y * (W/2) < H * (x - W/2)`

Tasks:
- [x] Implement triangle zone split in `input.rs` (`TouchControlScheme::Triangle`)
- [x] Gate behind `touch_control_scheme = "triangle"` in `game_config.toml`
- [ ] Draw a faint diagonal line indicator in the HUD (makes the split visible while learning)

### Option B: Virtual Joystick

A thumb origin + knob that tracks the touch, emits thrust in the joystick direction.

Tasks:
- [ ] Simple semi-transparent zone indicators (left = rotate, right = thrust) drawn as HUD overlay
- [ ] Virtual joystick: thumb-origin + knob tracking touch position
- [ ] Haptic feedback on thrust (iOS `UIImpactFeedbackGenerator`) — requires FFI or a winit hook

### Config
Both schemes should live behind `touch_control_scheme` in `game_config.toml` so they can be swapped without recompiling.

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

## 14. Title Screen: Instructions Overlay

Brief tutorial reachable from the title screen. New players (especially on
mobile) currently have no in-app way to learn the controls or objective.

Tasks:
- [ ] Add an "INSTRUCTIONS" / "?" button to the title screen
- [ ] Overlay shows controls (touch zones + keyboard) and objective in a few lines
- [ ] Tap-to-dismiss returns to title with no game-state side effects
- [ ] Style consistent with existing `TextRenderer` + game palette

---

## 15. Title Screen: Music Toggle

Player-facing music on/off control. Today `music_starts_on` is a config-file
knob; the player should be able to flip it from the title without editing
TOML.

Tasks:
- [ ] Music icon button on the title screen (note glyph; slash overlay when muted)
- [ ] Tap toggles whether music starts in the next play session
- [ ] Persist the preference across launches (depends on §16)

---

## 16. Persistent Local State / Scoreboard

A small key-value store on each platform for player-visible state that
should survive relaunches: high scores, music preference (§15), and
similar.

Tasks:
- [ ] Decide approach: file in app documents dir (cross-platform; one
      `serde`-encoded struct) vs platform-native (`UserDefaults` on iOS/macOS,
      `localStorage` on WASM)
- [ ] `PersistentState` struct with read/write API behind a thin platform shim
- [ ] Track top-N high scores with timestamp; show on title and game-over
- [ ] Document storage location per platform in `_context/`

---

## 17. Drop-Below-Viewport Game Over

Once a timed level mechanic exists (§18), the camera will stop scrolling
with the ship in some failure modes — meaning the ship can fall out of the
visible window. When that happens, declare game over rather than letting
the player chase an offscreen ship.

Tasks:
- [x] Define drop threshold (`ship.pos.y < viewport_offset - viewport_height`)
- [x] Trigger death + game over when crossed (in `update_ship`, alongside the
      existing horizontal-edge check)
- [ ] Visual cue as the ship nears the threshold (the offscreen-ship
      indicator in `longterm-features.md` complements this)

---

## 18. Time-Attack: Per-Level Timer

Each level has a target time. Reaching the next level early awards bonus
points; running out triggers game over. Adds pressure and gives each level
a clear arc.

Tasks:
- [ ] Add `level_time_limit_seconds` to `LevelParams` (per-level override
      possible later)
- [ ] HUD countdown timer; visual emphasis as time runs low
- [ ] On level transition: compute remaining time → bonus points
- [ ] On expiry: game over (consider an "OUT OF TIME" overlay variant)
- [ ] Tune so early levels feel generous, later ones tighten

---

## 19. HDR / EDR Output on Native Platforms

iPhone and modern Macs support Extended Dynamic Range — pixel values >1.0 in
a float framebuffer light up the panel above SDR white (up to ~1600 nits on
recent iPhones). The bloom pyramid already produces HDR values; the final
composite throws them away with a `clamp(..., 0.0, 1.0)`. Wiring up EDR
would let dense particle clusters and the ship outline genuinely "pop"
rather than topping out at SDR white.

Tasks:
- [ ] iOS / macOS surface configured for EDR. wgpu 29's `SurfaceConfiguration`
      doesn't expose this directly; reach into `wgpu::hal::api::Metal` via
      `Surface::as_hal` to set `wantsExtendedDynamicRangeContent = true` and
      pick an EDR-capable colorspace + float pixel format on the underlying
      `CAMetalLayer`. Code lives in framework's surface init; iOS-only.
- [ ] Drop the `clamp(0,1)` in `bloom_composite.wgsl` and replace with a
      tone-mapper (Reinhard or ACES) so SDR fallback stays sane and HDR
      output rolls off gracefully at panel peak.
- [ ] Tune `bloom_strength` and the threshold so that only the brightest
      stacked particle clusters cross 1.0 — the goal is "occasional
      eye-grabbing flashes," not a generally over-bright scene.
- [ ] Verify behaviour as EDR headroom collapses (Low Power Mode, bright
      ambient, thermal throttle): rendering should degrade smoothly to
      SDR-equivalent rather than going dim or banded.
- [ ] Web / mobile-Safari WebGPU EDR is not in scope here — deferred until
      browser support stabilises.

---

## 20. Ship Visual Stability (Per-Orientation Cached Sprite)

The ship's filled triangle and wireframe outline are rasterised by the GPU
every frame at the ship's current sub-pixel position. At the game's small
internal resolution (261×160) this produces visible per-pixel shimmer along
the edges as the ship translates — distracting, especially on a static
ship that's only thrusting slightly.

Idea: render the ship once per *orientation* into a small offscreen
texture, then each frame blit that texture to the game view at the ship's
pixel-snapped position. Translation no longer changes which pixels light
up; only rotation does. Rotation is rare enough that the shimmer there is
acceptable (and could be smoothed further by quantising orientation to,
say, 64 buckets and pre-rasterising once per bucket).

Tasks:
- [ ] Pick approach: live re-rasterise on orientation change (small wgpu
      render-to-texture each time `ship.orientation` moves more than ε) vs.
      pre-build a sprite atlas of N quantised orientations at startup
- [ ] Allocate a small ship sprite texture sized to the worst-case bounding
      box of the rotated ship (chevron is ~24×18 in game-view units)
- [ ] Render path: textured-quad blit to `game_view_texture` at
      `floor(ship.pos)` (pixel-snapped); orientation-dependent texture
      selected by the cache key
- [ ] Confirm the wireframe outline still bloomes correctly — the
      offscreen texture needs to be the same `Rgba16Float` HDR format as
      the game view so the bright-blue outline retains its >1.0 channel
- [ ] Verify visually that the shimmer is gone during pure translation,
      and that rotation jitter is bounded (or invisible) at the chosen
      bucket count

---

## 21. Visible Indestructible Side Walls

Today the leftmost and rightmost terrain columns hold max-health cells
(§8a) and a secondary bounds check kills the ship if it crosses the level
edge. Both are correct but visually subtle — the player may not realise
*why* they died, since the boundary looks like ordinary terrain that
could (in principle) be eroded.

The fix is to make the side walls *look* unkillable: a clear visual
distinction (e.g. a brighter colour, a hatched/striped pattern, a glowing
edge) that signals "you cannot break through here." Combined with the
existing kill behaviour, the player should immediately understand what
happens when the ship crosses that line.

Tasks:
- [ ] Decide visual treatment: distinct colour vs. pattern vs. animated
      glow. Should be readable at the small game-view resolution and not
      compete with the bloom-driven neon palette.
- [ ] Render path: probably a separate render pass / shader override on
      the leftmost+rightmost N columns, drawn after terrain so it always
      wins. Alternative: terrain shader detects max-health cells and
      colours them differently (cheaper but couples wall colour to "max
      health" semantics).
- [ ] Width: one column is enough to kill the ship but may be too thin to
      read visually; consider 2–3 columns wide so the wall is unmistakable.
- [ ] Confirm the wall reads correctly at native iPhone resolution after
      the upscale (no aliasing into invisibility, no shimmer).
- [ ] Optional: small particle/spark effect when the ship is destroyed by
      hitting the wall, to reinforce the cause of death.

---

## Branch Audit Remainder

The `legacy_wgpu` branch has been audited (see `legacy-port-inventory.md`). These branches have not yet been reviewed:

- [ ] `glalonde-emitoverangle` — likely emit-over-angle feature for the particle emitter
- [ ] `glalonde-fpbresen` — likely Bresenham/fixed-point line drawing
- [ ] `glalonde-glslrand` — likely GLSL random number utilities

For each: fetch the branch, identify what it adds vs master, document in `legacy-port-inventory.md` or discard.
