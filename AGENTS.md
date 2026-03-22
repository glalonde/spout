# Spout — Agent Instructions

Asteroids-style 2D arcade game with GPU-accelerated particle terrain destruction. Targets native desktop (wgpu/Metal) and WebAssembly (WebGPU). Written in Rust.

## Build & Check

```bash
cargo build                          # native build
cargo fmt --all -- --check           # format check (CI enforces)
cargo fmt --all                      # auto-format
cargo clippy -- -D warnings          # lint (CI enforces, warnings = errors)
cargo test --verbose                 # run tests
```

CI runs fmt → clippy → test in that order. All three must pass before merging.

System dep required on Linux: `libasound2-dev` (ALSA audio).

## Architecture

| Path | Purpose |
|------|---------|
| `src/main.rs` | Entry point, game loop, winit event handling, state machine |
| `src/particles.rs` | Particle system: GPU buffers, emitters, compute pass |
| `src/level_manager.rs` | Level/terrain generation and progressive scrolling |
| `src/render.rs` | Top-level render pipeline coordination |
| `src/ship.rs` | Ship state, physics, input handling |
| `src/camera.rs` | Camera/viewport transforms |
| `src/game_params.rs` | Config structs (deserialized from `game_config.toml`) |
| `src/shaders/` | WGSL compute and render shaders |
| `int_grid/` | Local sub-crate: fixed-point 2D integer grid (12 inner + 20 outer bits in u32) |
| `build.rs` | Generates shaders at compile time via `tera` templating — injects `int_grid` constants into WGSL |

## Shaders

Shaders are in `src/shaders/` as WGSL. `build.rs` uses `tera` to inject constants from `int_grid` at compile time. Do not edit generated shader output directly — edit the `.wgsl` or `.wgsl.include` source files.

## Key Constraints

- **Locked deps:** `Cargo.lock` is checked in and should be respected. The project builds with locked deps; upgrading is a deliberate, planned step (see `_context/plans/`).
- **wgpu 0.17 API:** Current code targets wgpu 0.17. Do not silently upgrade — it has breaking changes.
- **winit 0.28:** Same — 0.30+ has a breaking event loop rewrite.
- **WASM target:** `wasm32-unknown-unknown` with `RUSTFLAGS=--cfg=web_sys_unstable_apis`. Some deps have WASM-specific feature flags (see `Cargo.toml`).
- **No `unwrap()` in new code** without a comment explaining why it can't fail.

## Plans & Context

Active plans live in `_context/plans/active/`. Read these before starting significant work — they document decisions already made and work already scoped.

## Code Style

- Rust edition 2018
- `rustfmt` handles formatting — run `cargo fmt --all` before committing
- Clippy is strict (`-D warnings`) — fix all warnings, don't suppress with `#[allow(...)]` without a comment
