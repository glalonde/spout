# wgpu + winit Upgrade

Upgrade the two major stale dependencies to unblock active development. The compute shaders (WGSL) and `int_grid` logic are preserved — this is API plumbing work, not a logic rewrite.

**Estimated effort:** 2–4 days

---

## Immediate — low effort

- [x] Delete stale merged branches: `update-wgpu`, `level_manager`, `test_ci`, `android`, `glalonde-mixer`, `glalonde-newbrese`, `glalonde-nonimageatomic`, `glalonde-wipscroll`
- [x] Fix `release.yml`: remove/fix undefined `matrix.platform` reference (workflow currently fails silently)
- [x] Update CI action versions to `actions/checkout@v4`, `actions/cache@v4`
- [x] Add autoformatting: configure `rustfmt` (format on CI) and `clippy` lints to regularize code style
- [x] Add `AGENTS.md` — project instructions for AI assistants (build commands, architecture, constraints)

## Medium effort

- [ ] Audit ALL old branches (`glalonde-emitoverangle`, `glalonde-fpbresen`, `glalonde-glslrand`, `legacy_wgpu`) — for each branch: identify what functionality is implemented, compare against master, and document what is missing from master or worth porting; then merge or close
- [x] Upgrade `rand` (0.7 → 0.9) and `toml` (0.5 → 0.8) — lower-risk upgrades
- [x] Improve CI caching: expand cargo cache to include `~/.cargo/registry/index`, `~/.cargo/git/db`, and `target/` in ci.yml and release.yml; remove stale `cargo update` from release.yml (Cargo.lock is committed)
- [x] Disable `gh-pages.yml` (WASM build broken; changed to `workflow_dispatch`-only trigger until WASM is revived)

## Headless GPU testing — prerequisite for upgrade

Build regression coverage before the wgpu/winit upgrade so breakage is caught automatically. Screenshots serve dual purpose: CI pixel-diff regression and multimodal feedback for agents iterating on rendering code.

- [x] Add Mesa `lavapipe` (software Vulkan) to CI — install `libvulkan-dev mesa-vulkan-drivers` on Ubuntu runner; wgpu picks it up automatically with no real GPU present
- [x] Write compute integration test: initialize wgpu headlessly, emit N particles, step the compute pass, copy results back to CPU, assert on particle positions/counts — validates shader correctness and buffer layout
- [x] Write render integration test: render a known terrain slice + ship state offscreen, copy framebuffer to CPU, save as PNG — validates render pipeline end-to-end
- [x] Add golden image comparison with tolerance (allow small pixel delta to avoid brittleness across drivers); check golden images into `tests/golden/`
- [x] Wire headless GPU tests into CI; gate on lavapipe being available (Linux only — macOS Metal works headlessly but keep CI simple)

**Verification:** `cargo test` produces output PNGs in `tests/output/`; agent can read these images directly to verify visual correctness after changes.

## High effort — revival path

Order of operations: Rust version → winit → wgpu → web app revival.

- [x] Pin Rust version + upgrade to edition 2021: add `rust-toolchain.toml` pinning a stable release; bump `edition = "2021"` in `Cargo.toml` and `int_grid/Cargo.toml`; run `cargo fix --edition`; documents the minimum Rust floor for wgpu 0.22 (≥ 1.76)
- [x] Upgrade `winit` to 0.30: rewritten `examples/framework.rs` using `ApplicationHandler` trait; updated `main.rs` keyboard input to `KeyEvent`/`PhysicalKey::Code`
- [x] Upgrade `wgpu` to 29.0.0: updated API throughout — `particles.rs`, `render.rs`, `level_manager.rs`, `ship.rs`, `gpu_test_utils.rs`; `StagingBelt` now owns device; renamed `TexelCopyBufferInfo`/`TexelCopyBufferLayout`; new descriptor fields throughout
- [x] Revive web app: wgpu 29 default features include `webgpu`; WASM builds and deploys via `gh-pages.yml` on push to master. WASM async init uses `wasm_bindgen_futures::spawn_local` (not `pollster::block_on`). Web Audio playback implemented for music. Fixed WGSL uniformity violation (textureSample after non-uniform early-return → textureSampleLevel) that caused black screen on Chrome/Dawn; confirmed working on desktop Chrome and mobile Safari.

## MVP tasks (post-upgrade)

- [ ] Ship collision detection
- [ ] Score display
- [ ] Game over screen
- [ ] Text rendering
- [ ] Progressive game mechanics
- [x] Music — native (cpal + oxdz) and WASM (Web Audio API) implemented
- [ ] Resolution selection / window aspect ratio
