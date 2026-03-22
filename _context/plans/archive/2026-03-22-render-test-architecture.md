# Render Test Architecture

Expand headless GPU test coverage to particle and ship renderers, eliminating duplicated test boilerplate in the process.

---

## Phase 1 — Shared GPU Test Utilities (Refactoring)

Create `src/gpu_test_utils.rs`, compiled only under `#[cfg(test)]`. Extract and consolidate all duplicated test helpers currently scattered across `particles.rs` and `level_manager.rs`:

- `try_create_headless_device()` — move from `particles.rs`
- `create_offscreen_target()` — struct + helper for offscreen textures
- `create_readback_buffer()` / `encode_texture_readback()` — staging buffer setup
- `readback_pixels()` — submit + poll + map
- `bgra_to_rgba()`, `images_within_tolerance()`, `save_rgba_png()`
- `compare_or_generate_golden(name, rgba, width, height)` — always saves `tests/output/{name}.png`
  for debugging; if `SPOUT_GENERATE_GOLDEN` is set, writes `tests/golden/{name}.png` instead of
  comparing; otherwise asserts pixel-wise within tolerance=5 against the existing golden
  (`SPOUT_GENERATE_GOLDEN=1 cargo test` regenerates all goldens)
- `encode_clear_texture()` — clears a texture to black; required before calling any renderer that uses `LoadOp::Load` (particles, ship) on a fresh texture

Wire up in `src/lib.rs`: `#[cfg(test)] pub(crate) mod gpu_test_utils;`

**`bgra_to_rgba` call once rule:** Call `bgra_to_rgba` exactly once per readback, then pass the result to both `save_rgba_png` (via output path) and the golden comparison. The current terrain test does the conversion twice (once in `save_bgra_as_png`, again inline before comparison) — fix this in the refactor.

**Regression gate:** Port the existing terrain test to use the shared module and verify it still passes before proceeding.

- [x] Create `src/gpu_test_utils.rs`
- [x] Refactor `level_manager.rs` terrain test to use shared helpers
- [x] Refactor `particles.rs` emitter test to import `try_create_headless_device` from `gpu_test_utils`

---

## Phase 2 — Particle Renderer Golden Image

**Location:** Inside `particles.rs` test module (retains access to private `ParticleRenderer` without visibility changes).

Bypass the full compute pipeline. Use `GameParams::default()` and manually create a `density_buffer` via `device.create_buffer_init` with `game_params.grid_width * game_params.grid_height` entries of type `u32`, using known alternating values (e.g. `[50, 0, 50, 0, ...]`). This exercises the sigmoid color mapping and color map texture without needing the emitter or particle update shaders.

Encoding sequence:
1. Create encoder A
2. `ParticleRenderer::init(&device, &game_params, &density_buffer, &mut encoder_a)` — records color map buffer→texture copy
3. Encode `clear_texture` pass on the offscreen target
4. Encode `ParticleRenderer::render` pass
5. Encode texture readback
6. Submit encoder A once, readback, `compare_or_generate_golden("particle_render", ...)`

Golden file: `tests/golden/particle_render.png`

- [x] Add `test_particle_render_headless` in `particles.rs`
- [x] Generate and commit `tests/golden/particle_render.png`

---

## Phase 3 — Ship Renderer Golden Image

**Location:** New `mod tests` block in `src/ship.rs` (`ShipRenderer` and `ShipState` are already `pub`).

Canonical state: `ShipState { position: [32.0, 32.0], orientation: 0.0, ..Default::default() }` on a 64×32 target. Ship vertices span ~±10px × ±6px — the magenta triangle appears near center.

Note: `ShipRenderer::render` has an unusual signature — it takes both `&wgpu::Device` (needed by its staging belt) and `viewport_offset: i32` (use `0` for tests) in addition to the usual encoder/view params. Call `renderer.after_queue_submission()` after `queue.submit()` (matches the game loop pattern).

Golden file: `tests/golden/ship_render.png`

- [x] Add `test_ship_render_headless` in `ship.rs`
- [x] Generate and commit `tests/golden/ship_render.png`

---

## Phase 4 (Optional) — Composite Render Test

Stack all three renderers on one 64×32 offscreen target in a single encoder:

1. `TerrainRenderer::render` — uses `LoadOp::Clear`; clears to black and draws terrain
2. `ParticleRenderer::render` — uses `LoadOp::Load`; blends particles on top
3. `ShipRenderer::render` — uses `LoadOp::Load`; blends ship on top

Golden file: `tests/golden/composite_render.png`

- [ ] Add composite render test
- [ ] Generate and commit `tests/golden/composite_render.png`

---

## Key Technical Constraints

| Issue | Solution |
|---|---|
| `LoadOp::Load` on uninitialized texture | Always prepend `encode_clear_texture` before particle/ship render passes |
| `bytes_per_row` alignment | Only **width** affects alignment: width=64 → 64×4=256 bytes/row = `COPY_BYTES_PER_ROW_ALIGNMENT` exactly. Use width=64 for all tests; height stays at 32 (matching existing terrain test) |
| `ParticleRenderer::init` needs encoder for color map upload | Create encoder before `init`, use same encoder for render + readback, submit once |
| `density_buffer` size must match `game_params` grid | Use `GameParams::default()` and size buffer as `grid_width * grid_height` u32 entries |
| No GPU on CI | Same graceful skip pattern already in use — return early if `try_create_headless_device` returns `None` |
| `staging_belt` flush order | `TerrainRenderer` and `ShipRenderer` each own their own belt; call each renderer's `after_queue_submission` after `queue.submit` |
| `ShipRenderer::render` takes `device` and `viewport_offset` | Pass the existing `&device` and `0i32` for `viewport_offset` in tests |
| Debug artifacts on test failure | `compare_or_generate_golden` always writes `tests/output/{name}.png` — inspect this file when a golden comparison fails |
