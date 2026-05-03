# Legacy Branch Port Inventory

**Branch examined:** `legacy_wgpu` (https://github.com/glalonde/spout/tree/legacy_wgpu)
**Date assessed:** 2026-03-25

This document inventories what existed in `legacy_wgpu` that is absent from master, and assesses what is worth porting.

---

## Already Recovered (not worth porting)

These features existed in `legacy_wgpu` and have since been re-implemented (better) in master:

- **Audio / music** (`music_player.rs`, `sound_queue.rs`) — master has a full oxdz + cpal + Web Audio implementation; the legacy used rodio + OGGs, which was inferior.
- **Bloom / glow** (`glow_pass.rs`) — master has a proper HDR bloom pipeline with CRT filter.
- **Particle system** (`particle_system.rs`, `emitter.rs`) — master has a rewritten GPU particle system.
- **int_grid** (`int_grid.rs`) — exists as a separate local crate in master.
- **Terrain rendering** (`terrain_renderer.rs`, `game_viewport.rs`, `viewport.rs`) — absorbed into master's `render.rs` / `level_manager.rs`.

---

## Worth Porting

### 1. Text Rendering / HUD / Debug Overlay

**Legacy files:** `fonts.rs`, `text_renderer.rs`, `debug_overlay.rs`

**What they did:**
- Embedded 4 TrueType fonts (Inconsolata, Pixel Six, Visitor, DejaVu Sans Mono) via `include_bytes!`
- `TextRenderer` — positioned text on screen (menus, HUD overlays)
- `DebugOverlay` — rendered an FPS counter in-game using GPU text

**Why it matters:** Master has zero text rendering capability. Any HUD (score, lives, fuel, FPS counter) requires this. It's a prerequisite for basically all game UI.

**Porting caveat:** The legacy implementation used `wgpu_glyph`, which is incompatible with wgpu 29. It cannot be directly transplanted. The recommended modern replacement is [`glyphon`](https://github.com/grovesNL/glyphon) — a text renderer designed for wgpu that works with the current API. `wgpu_text` is another option but less actively maintained.

**Suggested approach:**
1. Add `glyphon` as a dependency.
2. Re-implement a `TextRenderer` struct wrapping glyphon's `TextAtlas` + `TextRenderer`.
3. Start minimal: a `DebugOverlay` that renders an FPS counter, then expand to game HUD.
4. Embed the fonts we want to keep (Inconsolata and/or Pixel Six are good picks for a game aesthetic).

**Priority:** High — this is the biggest visible gap between legacy and current.

---

### 2. Ship Collision Detection

**Legacy file:** `cpu_collision_detector.rs`

**What it did:**
- Maintained a GPU staging buffer for reading back terrain density data to CPU.
- `check()` method tested ship position against the terrain grid for collision.
- Used to trigger ship death / damage events.

**Why it matters:** Currently the ship flies through terrain without any collision response. This is already noted in `longterm-features.md` as high priority (✦).

**Porting caveat:** The legacy approach read back a GPU buffer to CPU every frame (expensive). The modern approach should either:
- Keep the readback but throttle it (every N frames, or async with 1-frame lag).
- Or do a lightweight CPU-side check against the level's integer grid (which the CPU already maintains in `level_manager.rs`).

**Priority:** Medium — gameplay-critical but requires design thought on the collision response (what happens when the ship hits terrain?).

---

### 3. FPS Estimator

**Legacy file:** `fps_estimator.rs`

**What it did:**
- Measured frame durations with `std::time::Instant`.
- Provided a rolling average FPS estimate.
- Also had a `high_resolution_sleep_until()` for manual frame-rate capping (less relevant now that winit handles the event loop).

**Why it matters:** Needed for the debug overlay FPS counter. Master has no frame timing instrumentation.

**Porting caveat:** The sleep logic is vestigial (winit's event loop handles this now). Only the measurement part is needed.

**Priority:** Low — straightforward to add inline when implementing the debug overlay, doesn't need its own module.

---

## Not Worth Porting

- **`shader_utils.rs`** (legacy) — used `rust_embed` and SPIR-V shaders. Master uses WGSL with a build-time tera pipeline. No benefit.
- **`spout_main.rs`** (legacy game loop) — a full rewrite happened; master's architecture is cleaner.
- **`game_viewport.rs` / `viewport.rs`** — absorbed into current render pipeline.

---

## Recommended Next Steps

1. **Text / HUD (glyphon)** — highest-value gap. Opens the door to score display, lives, fuel gauge, menus, and debug info.
2. **Collision detection** — after HUD exists, death/damage events can be communicated to the player.
3. **FPS counter** — can be folded into the HUD work.
