# wgpu + winit Upgrade

Upgrade the two major stale dependencies to unblock active development. The compute shaders (WGSL) and `int_grid` logic are preserved — this is API plumbing work, not a logic rewrite.

**Estimated effort:** 2–4 days

---

## Immediate — low effort

- [ ] Fix `release.yml`: remove/fix undefined `matrix.platform` reference (workflow currently fails silently)
- [ ] Update CI action versions to `actions/checkout@v4`, `actions/cache@v4`
- [ ] Add autoformatting: configure `rustfmt` (format on CI) and `clippy` lints to regularize code style

## Medium effort

- [ ] Upgrade `rand` (0.7 → 0.9) and `toml` (0.5 → 0.8) — lower-risk upgrades
- [ ] Add `wasm-bindgen-cli` caching to `gh-pages.yml` (currently installs from scratch every run)
- [ ] Increase test coverage; ensure GPU code is tested using a software GPU backend (e.g. `wgpu` with `dx12`/`vulkan` software adapter or `wgpu`'s `Gl` backend via `lavapipe`/`llvmpipe`)

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
