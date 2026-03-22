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
- [ ] Upgrade `rand` (0.7 → 0.9) and `toml` (0.5 → 0.8) — lower-risk upgrades
- [ ] Add `wasm-bindgen-cli` caching to `gh-pages.yml` (currently installs from scratch every run)

## Headless GPU testing — prerequisite for upgrade

Build regression coverage before the wgpu/winit upgrade so breakage is caught automatically. Screenshots serve dual purpose: CI pixel-diff regression and multimodal feedback for agents iterating on rendering code.

- [ ] Add Mesa `lavapipe` (software Vulkan) to CI — install `libvulkan-dev mesa-vulkan-drivers` on Ubuntu runner; wgpu picks it up automatically with no real GPU present
- [ ] Write compute integration test: initialize wgpu headlessly, emit N particles, step the compute pass, copy results back to CPU, assert on particle positions/counts — validates shader correctness and buffer layout
- [ ] Write render integration test: render a known terrain slice + ship state offscreen, copy framebuffer to CPU, save as PNG — validates render pipeline end-to-end
- [ ] Add golden image comparison with tolerance (allow small pixel delta to avoid brittleness across drivers); check golden images into `tests/golden/`
- [ ] Wire headless GPU tests into CI; gate on lavapipe being available (Linux only — macOS Metal works headlessly but keep CI simple)

**Verification:** `cargo test` produces output PNGs in `tests/output/`; agent can read these images directly to verify visual correctness after changes.

## High effort — revival path

- [ ] Upgrade `winit` to 0.30+: breaking event loop API changes require rewriting event handling in `main.rs`
- [ ] Upgrade `wgpu` to 0.22+: breaking API changes throughout — primarily `particles.rs`, `render.rs`, pipeline setup; wgpu changelog documents migration steps
- [ ] Revive web app: evaluate WebGPU vs WebGL2 as the rendering backend — document pros/cons of each (browser support, wgpu WASM compatibility, performance) and decide before implementing

## MVP tasks (post-upgrade)

- [ ] Ship collision detection
- [ ] Score display
- [ ] Game over screen
- [ ] Text rendering
- [ ] Progressive game mechanics
- [ ] Music
- [ ] Resolution selection / window aspect ratio
