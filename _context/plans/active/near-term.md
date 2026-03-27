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

## 2. Ship Collision Detection

**Blocker for:** lives, death events, game over screen.

Legacy branch had a GPU readback approach (`cpu_collision_detector.rs`). Better approach: test ship position against the CPU-side level grid already maintained in `level_manager.rs` — no GPU readback needed.

Tasks:
- [ ] Define collision shape for ship (point or simple radius)
- [ ] Query `LevelManager` terrain grid at ship position each frame
- [ ] Trigger a death/damage event on collision
- [ ] Decide on response: instant death, shield HP, or bounce

---

## 3. Score Display / HUD

**Depends on:** text rendering (#1), collision detection (#2 for lives).

Tasks:
- [ ] Score counter incremented by terrain destruction (particles hitting terrain)
- [ ] Render score + lives in a HUD using `TextRenderer`
- [ ] Fuel/energy gauge (visual bar or numeric)

---

## 4. Game Over Screen

**Depends on:** text rendering (#1), collision detection (#2).

Tasks:
- [ ] Game state machine: Playing → Dead → GameOver → Playing
- [ ] Render "GAME OVER" + final score text
- [ ] Input to restart

---

## 5. Resolution / Aspect Ratio

Independent — no blockers.

Currently the game renders at a fixed internal resolution with no control over aspect ratio or window size handling.

Tasks:
- [ ] Decide internal render resolution strategy (fixed game viewport with letterboxing vs. stretch)
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

## Branch Audit Remainder

The `legacy_wgpu` branch has been audited (see `legacy-port-inventory.md`). These branches have not yet been reviewed:

- [ ] `glalonde-emitoverangle` — likely emit-over-angle feature for the particle emitter
- [ ] `glalonde-fpbresen` — likely Bresenham/fixed-point line drawing
- [ ] `glalonde-glslrand` — likely GLSL random number utilities

For each: fetch the branch, identify what it adds vs master, document in `legacy-port-inventory.md` or discard.
