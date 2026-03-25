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

WASM build:
```bash
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --target wasm32-unknown-unknown
```

## Architecture

| Path | Purpose |
|------|---------|
| `src/main.rs` | Entry point, game loop, winit event handling, state machine |
| `src/particles.rs` | Particle system: GPU buffers, emitters, compute pass |
| `src/level_manager.rs` | Level/terrain generation and progressive scrolling |
| `src/render.rs` | Top-level render pipeline coordination |
| `src/bloom.rs` | HDR bloom pipeline: threshold → blur → composite |
| `src/ship.rs` | Ship state, physics, rendering (fill + wireframe outline) |
| `src/audio.rs` | Music: oxdz tracker player, cpal (native), Web Audio (WASM) |
| `src/camera.rs` | Camera/viewport transforms |
| `src/game_params.rs` | Config structs (deserialized from `game_config.toml`) |
| `src/shaders/` | WGSL compute and render shaders |
| `examples/framework.rs` | winit app harness (shared by main and tests) |
| `int_grid/` | Local sub-crate: fixed-point 2D integer grid |
| `build.rs` | Generates shaders at compile time via `tera` templating |

## Shaders

Shaders are in `src/shaders/` as WGSL. `build.rs` uses `tera` to inject constants from `int_grid` at compile time. Do not edit generated shader output directly — edit the `.wgsl` or `.wgsl.include` source files.

After any GPU or WASM change, read `_context/wasm-debugging.md` for known pitfalls before committing.

## Key Constraints

- **wgpu 29:** Current API. Do not silently upgrade — it has breaking changes.
- **winit 0.30:** Current API (uses `ApplicationHandler` trait, `KeyEvent`/`PhysicalKey`).
- **Rust edition 2021**, pinned toolchain in `rust-toolchain.toml`.
- **Locked deps:** `Cargo.lock` is checked in. Upgrading deps is a deliberate step.
- **WASM target:** `wasm32-unknown-unknown` with `RUSTFLAGS=--cfg=web_sys_unstable_apis`.
- **No `unwrap()` in new code** without a comment explaining why it can't fail.
- **WGSL uniformity:** `textureSample` (implicit LOD) requires uniform control flow. After any conditional early-return, use `textureSampleLevel(..., 0.0)` instead. See `_context/wasm-debugging.md`.

## Code Style

- Rust edition 2021
- `rustfmt` handles formatting — run `cargo fmt --all` before committing
- Clippy is strict (`-D warnings`) — fix all warnings, don't suppress with `#[allow(...)]` without a comment

## Context & Plans

`_context/` is the persistent knowledge base. Read it before starting significant
work — it documents decisions made, known gotchas, and work already scoped.

**At the start of a session:** read `_context/README.md` and any relevant
`_context/plans/active/` files.

**At the end of a session:** update plan files to reflect what was done, add any
non-obvious gotchas to `_context/wasm-debugging.md`, and move completed plans
to `_context/plans/archive/`.

See `_context/README.md` for the full process.
