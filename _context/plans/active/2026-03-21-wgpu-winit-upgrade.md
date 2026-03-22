# wgpu + winit Upgrade

Upgrade the two major stale dependencies to unblock active development. The compute shaders (WGSL) and `int_grid` logic are preserved — this is API plumbing work, not a logic rewrite.

**Estimated effort:** 2–4 days

---

## Immediate — low effort

- [ ] Delete stale merged branches: `update-wgpu`, `level_manager`, `test_ci`, `glalonde-mixer`, `glalonde-newbrese`, `glalonde-nonimageatomic`, `glalonde-wipscroll`
- [ ] Fix `release.yml`: remove/fix undefined `matrix.platform` reference (workflow currently fails silently)
- [ ] Update CI action versions to `actions/checkout@v4`, `actions/cache@v4`

## Medium effort

- [ ] Audit unmerged branches: `glalonde-emitoverangle`, `glalonde-fpbresen`, `glalonde-glslrand` — merge or close
- [ ] Diff `legacy_wgpu` vs `master` — README calls it "more complete"; cherry-pick anything useful
- [ ] Upgrade `rand` (0.7 → 0.9) and `toml` (0.5 → 0.8) — lower-risk upgrades
- [ ] Add `wasm-bindgen-cli` caching to `gh-pages.yml` (currently installs from scratch every run)

## High effort — revival path

- [ ] Upgrade `winit` to 0.30+: breaking event loop API changes require rewriting event handling in `main.rs`
- [ ] Upgrade `wgpu` to 0.22+: breaking API changes throughout — primarily `particles.rs`, `render.rs`, pipeline setup; wgpu changelog documents migration steps
- [ ] iOS: wrap existing WASM build in a minimal Swift/Xcode project using `WKWebView` (iOS 17+ ships WebGPU in Safari/WKWebView)

## MVP tasks (post-upgrade)

- [ ] Ship collision detection
- [ ] Score display
- [ ] Game over screen
- [ ] Text rendering
- [ ] Progressive game mechanics
- [ ] Music
- [ ] Resolution selection / window aspect ratio
