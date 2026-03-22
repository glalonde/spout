# Spout Repository Assessment
_Generated: 2026-03-21_

## Overview

**Spout** is a Rust-based 2D arcade/particle game — Asteroids-style with terrain destruction via a GPU-accelerated particle spray. It targets both native desktop (wgpu) and WebAssembly (WebGPU). There is a live web demo at https://glalonde.github.io/spout/.

- **Package version:** 0.1.0
- **Rust edition:** 2018 (resolver = "2")
- **Active development:** March 2019 – July 2023 (~437 commits, single author)
- **Status: Dormant.** No commits since July 2023.

---

## Code Structure

### Source (`src/`, ~3,300 lines)

| File | Lines | Purpose |
|------|-------|---------|
| `main.rs` | 490 | Entry point, game loop, event handling (winit), state machine |
| `particles.rs` | 915 | Particle system: GPU buffers, emitters, compute pass |
| `level_manager.rs` | 706 | Level/terrain generation and progressive scrolling |
| `render.rs` | 249 | Top-level render pipeline coordination |
| `ship.rs` | 234 | Ship state, physics, input handling |
| `camera.rs` | 229 | Camera/viewport transforms |
| `game_params.rs` | 151 | Config structs (deserialized from `game_config.toml`) |
| `textured_quad.rs` | 136 | Textured quad rendering helper |
| `color_maps.rs` | 87 | Color map definitions |
| `buffer_util.rs` | 45 | GPU buffer utilities |
| `load_image.rs` | 37 | Image loading utilities |
| `shader_util.rs` | 18 | Shader utilities |
| `lib.rs` | 0 | Empty library entry point (for WASM target) |

### Shaders (`src/shaders/`, WGSL format)

Shaders are template-generated at compile time via `build.rs` (uses `tera` templating to inject `int_grid` constants):
- `particles.wgsl` — main particle compute shader
- `render_particles.wgsl`, `terrain.wgsl`, `ship.wgsl`, `clear_density_buffer.wgsl`, `textured_model.wgsl`
- Shared includes: `grid.wgsl.include`, `hash.wgsl.include`, `noise.wgsl.include`, `particle.wgsl.include`

### Local sub-crate (`int_grid/`)

A small library defining a fixed-point 2D integer grid using bit-packing (12 inner + 20 outer bits in a u32). Constants are injected into WGSL shaders at build time.

---

## Key Dependencies

| Crate | Pinned Version | Status |
|-------|---------------|--------|
| `wgpu` | 0.17 | **Stale** — current is ~0.20+, breaking changes between versions |
| `winit` | 0.28.6 | **Stale** — 0.30+ has breaking event loop API changes |
| `cgmath` | 0.18 | OK (stable) |
| `bytemuck` | 1.4 | OK |
| `rand` | 0.7.2 | **Stale** — current is 0.9 |
| `image` | 0.23 | **Stale** — current is 0.25 |
| `serde` | 1.0 | OK |
| `toml` | 0.5 | **Stale** — 0.8 has API changes |
| `scarlet` | 1 | Unmaintained (last release 2019) |
| `pollster` | 0.2 | OK |
| `lazy_static` | 1 | OK (could migrate to `std::sync::LazyLock`) |
| `wasm-bindgen`, `web-sys` | various | Need checking for WASM target |

The `Cargo.lock` pins all transitive deps as of July 2023. A `cargo update` would pull in newer compatible versions and could surface breakage.

---

## Build Status

### Rust toolchain
Rust was **not installed** in the test environment. To check the build locally:
```bash
rustup show   # Check installed toolchains
cargo check   # Fast type-check without linking
cargo build   # Full build
```

The code **should build** with the locked dependencies (`cargo build` uses `Cargo.lock` by default). However:
- `wgpu 0.17` requires a GPU or software rasterizer. Native builds need a display server / GPU.
- CI last passed when the final commit was pushed (July 2023). **There is no recent CI run confirming the build still works.**

### WASM build
```bash
# From run_wasm.sh:
RUSTFLAGS=--cfg=web_sys_unstable_apis \
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir wasm-resources/out --target web \
  target/wasm32-unknown-unknown/release/spout.wasm
# Then serve wasm-resources/ on a local HTTP server
```
Requires: `rustup target add wasm32-unknown-unknown` and `cargo install wasm-bindgen-cli`.

---

## Remote Branches (15 total)

| Branch | Notes |
|--------|-------|
| `master` | Default branch, last commit July 2023 |
| `legacy_wgpu` | Older wgpu implementation — README notes it as "more complete" |
| `gh-pages` | Auto-deployed WASM build (GitHub Actions) |
| `android` | Android port — had 2 merged PRs (#9, #10) |
| `blade` | Likely a graphics backend experiment |
| `update-wgpu` | Old wgpu upgrade branch (merged as PR #8) |
| `level_manager` | Level system work (merged as PR #7) |
| `test_ci` | CI testing (merged as PR #6) |
| `glalonde-emitoverangle` | Emission-over-angle feature — **unmerged, status unknown** |
| `glalonde-fpbresen` | Bresenham line variant — **unmerged, status unknown** |
| `glalonde-glslrand` | GLSL random numbers — **unmerged, status unknown** |
| `glalonde-mixer` | Mixing feature (merged as PR #2) |
| `glalonde-newbrese` | Bresenham implementation (merged as PR #1) |
| `glalonde-nonimageatomic` | Non-image atomics (merged as PR #5) |
| `glalonde-wipscroll` | WIP scrolling (merged as PR #4) |

**Branches to investigate:**
- `legacy_wgpu` — explicitly called out as "more complete" in the README; worth diffing against `master`
- `android` — may still have useful code separate from merged PRs
- `glalonde-emitoverangle`, `glalonde-fpbresen`, `glalonde-glslrand` — unmerged feature branches; unclear if abandoned or just not yet merged

**Cleanup candidates (safe to delete):** All `glalonde-*` branches that are merged, plus `update-wgpu`, `level_manager`, `test_ci`.

---

## Issues and Pull Requests

### Open Issues
**None.** Zero open (or closed) issues on the repo.

### Pull Requests (all 10 are merged)

| # | Title | Merged |
|---|-------|--------|
| 10 | Android | 2021-08-07 |
| 9 | Android works | 2020-09-26 |
| 8 | Update wgpu | 2020-04-23 |
| 7 | Level manager | 2020-04-06 |
| 6 | Test ci | 2020-03-31 |
| 5 | Glalonde nonimageatomic | 2020-03-29 |
| 4 | Glalonde wipscroll | 2020-03-03 |
| 3 | wip | 2019-12-14 |
| 2 | Fix and improve | 2019-05-11 |
| 1 | Glalonde newbrese | 2019-04-24 |

No open PRs, no draft PRs. Consistent with a solo developer using branches as scratch space and merging directly.

---

## CI/CD

Three workflows in `.github/workflows/`:

### `ci.yml` — Continuous Integration
- Triggers on all pushes/PRs
- Runs `cargo test --verbose` on `ubuntu-latest`
- Installs `libasound2-dev` (ALSA audio)
- Uses old action versions (`actions/cache@v1`, `actions/checkout@v2`) — **should upgrade**
- No matrix testing (single platform, single Rust channel)

### `gh-pages.yml` — WASM Web Deployment
- Triggers on push to `master`
- Builds `wasm32-unknown-unknown` release, generates JS bindings, deploys to `gh-pages`
- Installs `wasm-bindgen-cli` via `cargo install` on every run — **slow; no caching**
- Uses `RUSTFLAGS=--cfg=web_sys_unstable_apis`

### `release.yml` — Binary Release
- Triggers on version tags (`v*`)
- Builds Linux release binary and uploads to GitHub Releases
- **Bug:** references `matrix.platform` but no `matrix` is defined — the `chown` step condition is always false; this workflow likely fails silently

---

## Task Completion Status (`tasks.md`)

### MVP — Incomplete Items
- [ ] Ship collision detection
- [ ] Text rendering
- [ ] Resolution selection / window aspect ratio
- [ ] Show score
- [ ] Music
- [ ] Game over screen
- [ ] Progressive game mechanics

### Improvements — Incomplete Items
- [ ] Initialization flags / logging
- [ ] Improved/wireframe ship rendering
- [ ] High-res glow / separated gaussian glow / level edge glow
- [ ] Configs from proto
- [ ] Fluid dynamics for droplets
- [ ] Find compilation bottleneck
- [ ] Cross-platform builds (Windows / macOS)
- [ ] CI binary builds

---

## Summary Assessment

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Code architecture | Good | Clean GPU/compute separation; shader templating is clever |
| Code completeness | ~60% MVP | Core mechanics work; UI, score, game-over missing |
| Build health | Unknown | Not verified recently; Cargo.lock should still produce a working build |
| Dependency freshness | Poor | wgpu (0.17→0.20+), winit (0.28→0.30+) both have breaking API changes |
| CI health | Poor | Old action versions; release workflow has a bug; no recent passing runs confirmed |
| Test coverage | Minimal | `cargo test` runs but likely only doctests/examples |
| Documentation | Minimal | README is sparse; no API docs |
| Branch hygiene | Poor | 15 branches, ~8 are stale merged branches |
| Open issues/PRs | Clean | Zero open issues or PRs |
| Last active | July 2023 | ~2.5 years dormant |

---

## Recommended Cleanup Plan

### Immediate (low effort)
1. **Verify the build:** `cargo check` and `cargo build` to confirm it still compiles with locked deps
2. **Delete stale merged branches:** `update-wgpu`, `level_manager`, `test_ci`, `glalonde-mixer`, `glalonde-newbrese`, `glalonde-nonimageatomic`, `glalonde-wipscroll`
3. **Fix release.yml:** Remove or fix the undefined `matrix.platform` reference
4. **Update CI action versions:** `actions/checkout@v4`, `actions/cache@v4`

### Medium effort
5. **Audit unmerged branches:** Review `glalonde-emitoverangle`, `glalonde-fpbresen`, `glalonde-glslrand` — merge or close
6. **Diff `legacy_wgpu` vs `master`:** Understand what "more complete" means and cherry-pick useful features
7. **Update `rand` and `toml`:** Relatively low-risk upgrades
8. **Add `wasm-bindgen-cli` caching** to `gh-pages.yml`

### High effort (revival path)
9. **Upgrade `winit` to 0.30+:** Breaking event loop API changes require rewriting the event handling in `main.rs`
10. **Upgrade `wgpu` to 0.20+:** Breaking API changes throughout; primarily affects `particles.rs`, `render.rs`, pipeline setup
11. **Complete MVP tasks:** Ship collision, score display, game over screen
12. **Add cross-platform CI builds** (macOS, Windows)
