# Six-Hour Session Plan — 2026-03-22

Code health focus. No new features. Ship collision detection deferred to next session.

---

## Phase 1 — Easy / Mechanical Cleanup (~1h) ✅

All items from the code health plan, easy tier:

- [x] Remove `frame_num: i64` from `Render` (`src/render.rs:18`) — incremented every frame, never read
- [x] Remove dead demo texture machinery from `Render`: `show_demo_texture: bool`, `demo_model: Option<TexturedQuad>`, and the `load_image_to_texture` call at init
- [x] Remove vestigial `_device: &wgpu::Device` params from four render methods:
  - `Render::render()` (`src/render.rs`)
  - `ShipRenderer::render()` (`src/ship.rs`)
  - `TerrainRenderer::update_render_state()` (`src/level_manager.rs`)
  - `ParticleSystem::run_compute()` (`src/particles.rs`) + `Emitter::run_compute()`
- [x] Extract duplicated level budget constant in `main.rs`: `LEVEL_BUDGET: Duration = from_nanos(3_333_333)` at module level

---

## Phase 2 — `bytemuck` Upgrade (~1.5h) ✅

- [x] Upgrade `bytemuck` to >= 1.15 in `Cargo.toml` (Cargo.lock already had 1.25.0)
- [x] Verify `bytemuck_derive >= 1.5` is pulled in (already in lock as part of 1.25.0)
- [x] Remove `#![allow(dead_code)]` from `particles.rs`, `ship.rs`, `level_manager.rs`, `textured_quad.rs`
- [x] Narrow any remaining `#[allow(dead_code)]` suppressions and re-assess:
  - `color_maps.rs`: `ColorMap` enum is genuinely dead (index doc), `#[allow]` kept
  - `level_manager.rs`: `stripe_level` field genuinely unused, `#[allow]` kept
- [x] Run `cargo test` — 5/5 pass

---

## Phase 3 — `lazy_static` → `OnceLock` (~1h) ✅

- [x] Audit all `lazy_static!` usages — only one in `color_maps.rs`
- [x] Replace with `std::sync::OnceLock` (COLOR_MAPS static + `color_maps()` init fn)
- [x] Remove `lazy_static` from `Cargo.toml` dependencies
- [x] Run `cargo clippy -- -D warnings` and `cargo test` — all pass

---

## Phase 4 — Explicit Pipeline Layout in `render.rs` (~1.5h) ✅

- [x] Read existing TODO at `render.rs` and understand current auto-layout usage (`layout: None`)
- [x] Create explicit `BindGroupLayout` for group 0 (camera uniform, 128 bytes, VERTEX)
- [x] Create explicit `BindGroupLayout` for group 1 (texture/sampler/model-pose, FRAGMENT+VERTEX)
- [x] Create `PipelineLayout` from both BGLs, wire into render pipeline descriptor
- [x] Confirm no bind-group/shader mismatches — clippy + tests pass
- [x] Remove the TODO comment

---

## Phase 5 — WASM Revival ✅ (partial)

- [ ] Evaluate WebGPU vs WebGL2 as the rendering backend — document pros/cons (browser support, wgpu WASM compatibility, performance) and decide
- [x] Re-enable `gh-pages.yml` — now triggers on push to master (+ workflow_dispatch retained)
- [x] Audit and update WASM-specific feature flags in `Cargo.toml` — added `getrandom = { version = "0.3", features = ["wasm_js"] }` for wgpu 29 transitive dep
- [x] Verify `wasm32-unknown-unknown` target builds: passes with `RUSTFLAGS=--cfg=web_sys_unstable_apis`
- [x] Fix compilation errors against wgpu 29 WASM API surface — getrandom 0.3 wasm_js fix
- [ ] Smoke-test in browser (or document what's still broken) — **known open issue**: `framework.rs` uses `pollster::block_on` for async wgpu init; on WASM this may fail because browser async can't be synchronously blocked. Needs `wasm-bindgen-futures::spawn_local` refactor or similar.
- [x] Clean up WASM dependencies — removed `lazy_static` (no longer needed); getrandom now explicit

---

## Success Criteria ✅

- [x] `cargo fmt --all -- --check` passes
- [x] `cargo clippy -- -D warnings` passes
- [x] `cargo test --verbose` passes (5/5 tests)
- [x] All phases 1–4 fully completed
- [x] Phase 5 WASM build passes; runtime browser smoke-test deferred (pollster async issue)
