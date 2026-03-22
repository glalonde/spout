# Code Health Cleanup

Pre- and post-upgrade housekeeping identified during audit before the wgpu/winit upgrade.

---

## Do now (before wgpu/winit upgrade)

- [x] Upgrade `image` 0.23 → 0.25: captures 2 years of patches; low-risk minor bump (also fixed `into_bgra8` → `into_rgba8` API change)
- [x] Replace broad undocumented `#[allow(dead_code)]` with documented file-level allows: root cause is `bytemuck_derive 1.4.1` generating a `check` fn for Pod impls that modern Rust flags — TODO comment left to remove the allow when bytemuck_derive >= 1.5 is available
- [x] Remove `bytemuck` from `[dev-dependencies]` — already in `[dependencies]`
- [x] Add `rust-version = "1.94.0"` to `Cargo.toml` — consistent with pinned toolchain

## Defer until after wgpu/winit upgrade

- [ ] Replace `lazy_static` with `OnceLock` (stable since 1.70) — no urgency, lazy_static still maintained
- [ ] Replace `cgmath` with `glam` — better community adoption, more active development; architectural decision, cleaner to do post-upgrade when touching math-heavy code anyway
- [ ] Lock file duplicate versions (rand 0.8+0.9, png, miniz_oxide, syn 1+2, windows crates) — most are transitive from wgpu 0.17; will clean up naturally during wgpu upgrade
- [ ] Clean up WASM dependencies — audit/prune once WASM target is revived
- [ ] Add unit tests for camera math (spherical→Cartesian transforms) and buffer size calculations
- [ ] Narrow remaining `#[allow(dead_code)]` suppressions — once broad ones are removed, assess what's left
- [ ] "Keep in sync with shader" TODOs in `particles.rs` — consider generating struct layout from shader or adding an assert; address when touching particle code during wgpu upgrade
