# Code Health Cleanup

Pre- and post-upgrade housekeeping. Items identified during the pre-upgrade audit and during the wgpu 29 / winit 0.30 upgrade itself.

---

## Completed

- [x] Upgrade `image` 0.23 → 0.25: low-risk minor bump (also fixed `into_bgra8` → `into_rgba8` API change)
- [x] Replace broad undocumented `#[allow(dead_code)]` with documented file-level allows: root cause is `bytemuck_derive 1.4.1` generating a `check` fn for Pod impls — TODO comment left to remove when bytemuck_derive >= 1.5
- [x] Remove `bytemuck` from `[dev-dependencies]` — already in `[dependencies]`
- [x] Add `rust-version = "1.94.0"` to `Cargo.toml` — consistent with pinned toolchain

---

## Easy / mechanical (do next)

- [ ] Remove `frame_num: i64` from `Render` (`src/render.rs:18`) — incremented every frame, never read
- [ ] Remove dead demo texture machinery from `Render`: `show_demo_texture: bool`, `demo_model: Option<TexturedQuad>`, and the `load_image_to_texture` call at init — loads a texture unconditionally, allocates GPU memory, never toggled by any input; `pub show_demo_texture` is dead public API
- [ ] Remove vestigial `_device: &wgpu::Device` params from four render methods — these became unused when StagingBelt internalized the device during the wgpu 29 upgrade:
  - `Render::render()` (`src/render.rs:190`)
  - `ShipRenderer::render()` (`src/ship.rs:199`)
  - `TerrainRenderer::update_render_state()` (`src/level_manager.rs:545`)
  - `ParticleSystem::run_compute()` (`src/particles.rs:281`)
- [ ] Extract duplicated level budget constant in `main.rs` (lines 106 and 362): `Duration::from_secs_f64(1.0 / 300.0)` appears twice
- [ ] Replace `lazy_static` with `OnceLock` (stable since 1.70) — no urgency, lazy_static still maintained

---

## Moderate effort

- [ ] Upgrade `bytemuck` to >= 1.15: `bytemuck_derive >= 1.5` switches Pod derives from `check` fn to `const` assertions, allowing removal of the documented `#![allow(dead_code)]` in `particles.rs`, `ship.rs`, `level_manager.rs`, `textured_quad.rs`; then narrow remaining `#[allow(dead_code)]` suppressions and re-assess what's genuinely unused
- [ ] Convert draw pipeline to explicit pipeline layout in `render.rs` (existing TODO at line 108): currently uses `layout: None` (auto-layout via shader reflection), which is fragile against accidental shader/bind-group mismatches and blocks pipeline layout sharing
- [ ] Address "keep in sync with shader" TODOs in `particles.rs` — consider generating struct layout from shader or adding a static size assert

---

## Architectural / longer horizon

- [ ] Replace `cgmath` with `glam` — better community adoption, more active development; significant find-and-replace but cleaner to do when touching math-heavy code anyway
- [ ] Consolidate `StagingBelt` instances: currently 6 separate belts across `particles.rs` (x2), `level_manager.rs` (x2), `render.rs`, `ship.rs` — each holding a cloned `Arc<Device>`; a single shared belt at the top level would simplify ownership and allow better chunk sizing
- [ ] Decouple update logic from render frequency: `Spout::update_state()` is called inside `Spout::render()` (`main.rs:315`), coupling physics tick rate to display frame rate; a fixed-rate update loop (or at minimum a capped `dt`) would make behavior frame-rate-independent
- [ ] Add unit tests for camera math (spherical→Cartesian transforms) and buffer size calculations
- [ ] Clean up WASM dependencies — audit and prune once WASM target is revived
