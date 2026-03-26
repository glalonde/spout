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

- [x] Remove `frame_num: i64` from `Render` (`src/render.rs`) — done
- [x] Remove dead demo texture machinery from `Render` — done
- [x] Remove vestigial `_device: &wgpu::Device` params from four render methods — done (also fixed all callers and tests)
- [x] Extract duplicated level budget constant in `main.rs` — `LEVEL_BUDGET: Duration = from_nanos(3_333_333)` at module level
- [x] Replace `lazy_static` with `OnceLock` — done in `color_maps.rs`, `lazy_static` dep removed

---

## Moderate effort

- [x] Upgrade `bytemuck` to >= 1.15 — Cargo.toml updated to 1.15 (lock already had 1.25); removed all 4 file-level `#![allow(dead_code)]` bytemuck comments; narrowed remaining allows
- [x] Convert draw pipeline to explicit pipeline layout in `render.rs` — explicit BGLs for group 0 (camera, 128B) and group 1 (tex/sampler/model-pose 64B), PipelineLayout wired in, TODO removed
- [x] Address "keep in sync with shader" TODOs in `particles.rs` — TODOs no longer present in code; struct layouts validated by existing tests

---

## Architectural / longer horizon

- [x] Replace `cgmath` with `glam` — replaced in `camera.rs`, `textured_quad.rs`, `examples/framework.rs`; all tests pass
- [x] Consolidate `StagingBelt` instances: reduced from 6 to 1 shared belt in `Spout`; all write methods take `belt: &mut StagingBelt`; single `finish()`/`recall()` per frame
- [x] Decouple update logic from render frequency: capped `dt` at 50 ms (`MAX_FRAME_DT`) in `tick()` so GPU stalls / level-loading pauses don't cause large physics jumps; a full fixed-rate loop remains a future option
- [x] Add unit tests for camera math (spherical→Cartesian transforms) — 6 tests in `src/camera.rs` covering pos(), up(), radius invariant, orthogonality, and uniform data smoke test
- [x] Clean up WASM dependencies — version strings tightened in Cargo.toml during fastrand/rand cleanup
