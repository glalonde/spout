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
**Verified: builds and runs as of 2026-03-21.**

The code compiles and runs with the locked dependencies (`Cargo.lock`). Native desktop target confirmed working.

To build locally:
```bash
rustup show   # Check installed toolchains
cargo check   # Fast type-check without linking
cargo build   # Full build
cargo run     # Build and run
```

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

_Audited 2026-03-21 via `git log` and `git diff` against `origin/master`._

| Branch | Unique Commits vs master | Status |
|--------|--------------------------|--------|
| `master` | — | Default branch, last commit July 2023 |
| `gh-pages` | — | Auto-deployed WASM build (GitHub Actions) |
| `assessment` | 4 | Current branch — docs/planning only, no code changes |
| `blade` | 15 | **Unmerged** — major graphics backend rewrite (see below) |
| `glalonde-emitoverangle` | 1 | **Unmerged** — C++/OpenGL-era particle angle-emission experiment |
| `glalonde-fpbresen` | 1 | **Unmerged** — C++/OpenGL-era float Bresenham experiment |
| `glalonde-glslrand` | 1 | **Unmerged** — C++/OpenGL-era integer LCG RNG experiment |
| `glalonde-mixer` | 1 | Trivial README addition only (2 lines) |
| `android` | 0 | Fully merged into master |
| `legacy_wgpu` | 0 | Fully merged — tip is a direct ancestor of master |
| `glalonde-newbrese` | 0 | Fully merged into master |
| `glalonde-nonimageatomic` | 0 | Fully merged into master |
| `glalonde-wipscroll` | 0 | Fully merged into master |
| `level_manager` | 0 | Fully merged into master |
| `test_ci` | 0 | Fully merged into master |
| `update-wgpu` | 0 | Fully merged into master |

### Branch Details: `blade` (15 commits, 69 files, +3,031/−6,547 lines)

An experimental full rewrite replacing `wgpu` + `winit` with **[blade-graphics](https://github.com/kvark/blade)**, a lower-level GPU abstraction crate.

**What was added:**
- New `Cargo.toml` pulling in `blade-graphics`, `blade-macros`, `egui`, `egui-winit`, `glam`, `kira` (audio), `rust_embed`, `wasm_thread`
- `src/music.rs` — audio playback embedding `.ogg` files
- A `bunnymark` example (`examples/bunnymark/`, 457 lines) — classic GPU sprite benchmark
- `build_wasm.sh` and a `run-wasm/` helper crate
- A minimal replacement `src/shader.wgsl` (41 lines)
- `rust-toolchain.toml` pinning the Rust version

**What was deleted:**
- All core game modules: `particles.rs` (915 lines), `level_manager.rs` (706 lines), `camera.rs`, `ship.rs`, `render.rs`
- All WGSL shaders under `src/shaders/`
- `game_params.rs`, `textured_quad.rs`, `color_maps.rs`, `buffer_util.rs`, `shader_util.rs`
- `build.rs`, `game_config.toml`, `run_wasm.sh`, the `int_grid/` sub-crate, and several examples

**Verdict:** This branch gutted the game logic entirely. It is an architectural experiment/skeleton, not a continuation of the game. It does not salvage the particle system. Unless the goal is specifically to evaluate blade-graphics as a backend, this branch has little value relative to the wgpu upgrade path.

### Branch Details: `glalonde-emitoverangle`, `glalonde-fpbresen`, `glalonde-glslrand`

These three branches are **C++/OpenGL-era experiments**, not Rust/wgpu code. They predate the current Rust rewrite and operate on GLSL compute shaders and C++ test harnesses. They are not applicable to the current codebase.

- **`glalonde-emitoverangle`** (`554fa22 wip`): Reworks the GLSL particle emitter to emit particles in a fan/arc pattern with angular spread and directional velocity control. Also adds a new `ship.cs` compute shader for Bresenham-based ship physics with terrain collision. Contains a known bug (wrong buffer name in the ship shader).
- **`glalonde-fpbresen`** (`4218a98 fp bresenham with bugs?`): Replaces the fixed-point integer Bresenham particle movement with a floating-point version using `vec2` positions and `float error`. Commit message acknowledges bugs remain.
- **`glalonde-glslrand`** (`ecfb8e0 integer glsl rand`): Replaces noise-based PRNG in the GPU emitter with an integer LCG seeded from wall-clock nanoseconds. Cleaner and faster than the noise-based approach.

All three are WIP experiments from the project's early C++ phase. They contain interesting algorithmic ideas but are not directly mergeable.

### Corrections to Prior Assessment

- **`legacy_wgpu`** is **not** a separate "more complete" codebase. Its tip commit is a direct ancestor of master — it contains no code that isn't already in master. The README description is misleading.
- **`android`** is fully merged with zero unique commits. No unique code remains.

**Cleanup candidates (safe to delete):**
- Fully merged (zero unique code): `android`, `legacy_wgpu`, `update-wgpu`, `level_manager`, `test_ci`, `glalonde-newbrese`, `glalonde-nonimageatomic`, `glalonde-wipscroll`, `glalonde-mixer`
- C++/OpenGL era, not applicable: `glalonde-emitoverangle`, `glalonde-fpbresen`, `glalonde-glslrand`
- Architectural experiment, gutted game logic: `blade` (keep if blade-graphics evaluation is desired; delete otherwise)

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
| Build health | Good | Verified builds and runs as of 2026-03-21 with locked deps |
| Dependency freshness | Poor | wgpu (0.17→0.20+), winit (0.28→0.30+) both have breaking API changes |
| CI health | Poor | Old action versions; release workflow has a bug; no recent passing runs confirmed |
| Test coverage | Minimal | `cargo test` runs but likely only doctests/examples |
| Documentation | Minimal | README is sparse; no API docs |
| Branch hygiene | Poor | 15 branches; 9 are fully merged/stale; 3 are C++ era artifacts; `blade` is gutted experiment |
| Open issues/PRs | Clean | Zero open issues or PRs |
| Last active | July 2023 | ~2.5 years dormant |

---

## Salvage vs. Rewrite Analysis

**Verdict: Upgrade, don't rewrite. wgpu remains the right GPU abstraction.**

### Is the existing code worth keeping?

Yes. The ~3,300 lines break down as:
- ~1,600 lines of hard-to-rewrite domain logic (particle system, terrain generation, shaders)
- ~700 lines of winit/wgpu plumbing that is the actual upgrade target

The compute shaders (WGSL) and the `int_grid` fixed-point math are the genuinely valuable parts. A rewrite would mostly mean re-deriving the same physics and rendering decisions. The wgpu 0.17 → current and winit 0.28 → 0.30+ upgrades are disruptive but mechanical — API churn, not logic changes.

### Is wgpu still the right choice?

Requirements: compute shaders for particles, web app, native macOS, native iOS.

| Platform | wgpu backend | Status |
|----------|-------------|--------|
| Web | WebGPU | Ships via WASM; broad browser support (Chrome, Firefox, Safari) |
| macOS | Metal | First-class, mature |
| iOS | Metal | Works; winit has iOS support; App Store needs Xcode signing |

**Alternatives considered:**

- **Raw WebGPU (JS/TS):** Good for web-first, but native requires a WebView wrapper, complicates compute shaders, loses the Rust ecosystem.
- **Metal (Swift):** Excellent on Apple platforms, but no web story — would require maintaining two codebases.
- **Bevy:** Uses wgpu under the hood, but the existing particle system is custom GPU compute that doesn't map cleanly to Bevy's ECS. More work to adapt than to upgrade directly.

### The iOS constraint

iOS was never part of the original design. Two paths:

1. **wgpu + winit native iOS target** — Works via the Metal backend. App Store distribution requires Xcode. Harder to set up in CI.
2. **WASM + WKWebView** — iOS 17+ ships WebGPU support in Safari/WKWebView. The existing WASM build runs in a thin iOS wrapper app. One codebase targets WebGPU everywhere.

**Recommendation: Path 2 (WASM + WKWebView).** Simpler to maintain, reuses the existing `gh-pages.yml` WASM pipeline, and the WKWebView overhead is minimal for this type of game. Only reconsider native iOS if 60fps heavy-compute performance becomes a bottleneck.

### Recommended Upgrade Path

1. **Upgrade wgpu to current (0.22+) and winit to 0.30+** — Main work; primarily affects `main.rs` (winit event loop rewrite) and `particles.rs`/`render.rs` (wgpu API changes). The wgpu changelog documents migration steps.
2. **Keep shaders as-is** — WGSL is stable; the compute shader logic does not need to change.
3. **iOS: WASM + WKWebView wrapper** — Wrap the existing WASM build in a minimal Swift/Xcode project using `WKWebView`.
4. **Preserve the particle system** — It is the core of the game and already GPU-compute-based.

Estimated effort: 2–4 days to upgrade wgpu/winit back to a working build vs. weeks to rewrite to feature parity.

### Note on `blade` branch

The `blade` branch explored replacing wgpu with [blade-graphics](https://github.com/kvark/blade) but deleted all game logic in the process (particles, terrain, ship, shaders). It is not a viable alternative path unless starting the game from scratch. The wgpu upgrade remains the correct path.

---

## Recommended Cleanup Plan

### Immediate (low effort)
1. **Verify the build:** `cargo check` and `cargo build` to confirm it still compiles with locked deps
2. **Delete stale merged branches:** `update-wgpu`, `level_manager`, `test_ci`, `glalonde-mixer`, `glalonde-newbrese`, `glalonde-nonimageatomic`, `glalonde-wipscroll`
3. **Fix release.yml:** Remove or fix the undefined `matrix.platform` reference
4. **Update CI action versions:** `actions/checkout@v4`, `actions/cache@v4`

### Medium effort
5. **Delete C++/OpenGL-era branches:** `glalonde-emitoverangle`, `glalonde-fpbresen`, `glalonde-glslrand` are not applicable to the Rust/wgpu codebase. The algorithmic ideas (float Bresenham, angular particle emission, integer LCG RNG) could be implemented fresh in WGSL if desired, but the C++ code cannot be directly merged.
6. **Decide on `blade` branch:** Either delete it (game logic was gutted) or keep it as a reference if evaluating blade-graphics. No action needed to resume wgpu upgrade.
7. **Update `rand` and `toml`:** Relatively low-risk upgrades.
8. **Add `wasm-bindgen-cli` caching** to `gh-pages.yml`.

### High effort (revival path)
9. **Upgrade `winit` to 0.30+:** Breaking event loop API changes require rewriting the event handling in `main.rs`
10. **Upgrade `wgpu` to 0.20+:** Breaking API changes throughout; primarily affects `particles.rs`, `render.rs`, pipeline setup
11. **Complete MVP tasks:** Ship collision, score display, game over screen
12. **Add cross-platform CI builds** (macOS, Windows)
