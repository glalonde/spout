# Bloom & Visual Post-Processing Plan

## Status: Merged (all PRs #32–#37 → master)

## What was done

### Bloom pipeline (PR #32–#35)
- Added `Rgba16Float` HDR game view texture (replaces `Bgra8UnormSrgb`)
- Added `Bloom` pipeline: threshold pass → separable Gaussian blur → composite
- Added `VisualParams` config struct (`bloom_threshold`, `bloom_strength`, `bloom_passes`, `crt_strength`) in `game_params.rs`
- Moved bloom to full surface resolution (game view upscaled first, then bloom at display res — fixes blocky halos)
- Fixed headless GPU test readback for `Rgba16Float` (8 bytes/pixel, f16→u8 conversion)

### CRT & visual polish (PR #36–#37)
- Added CRT post-processing to composite pass: barrel distortion, chromatic aberration, aperture-grille phosphor mask, scanlines, vignette — all driven by a single `crt_strength` parameter
- Added HDR terrain edge glow: 4-connected neighbor edge detection in fragment shader; edges return HDR red-orange (`vec4(0.9, 0.25, 0.02, 1.0)`) to drive bloom
- Added bright blue wireframe outline on ship (second `LineStrip` pipeline in `ShipRenderer`, HDR light-blue `vec4(0.3, 0.7, 2.0, 1.0)`)
- Tuned `bloom_strength = 1.25`, `crt_strength = 0.5`
- Removed golden images and `update-goldens` CI workflow (cross-platform rasterization differences made them unmaintainable; tests still write to `tests/output/` for visual inspection)
- Fixed WGSL uniformity violation in composite shader: `textureSample` after a non-uniform early-return violated the WGSL spec; replaced with `textureSampleLevel(..., 0.0)` — this caused a black screen on desktop Chrome/Dawn while mobile Safari was lenient

## Known issues / follow-up

None currently open. The bloom resolution issue (blocky halos) was fixed in PR #35.

## Config knobs (`game_config.toml`)

```toml
[visual_params]
bloom_threshold = 0.6   # lower = more of the scene glows
bloom_strength  = 1.25  # composite intensity
bloom_passes    = 2     # H+V blur iterations (more = wider halo)
crt_strength    = 0.5   # 0.0 = off, 1.0 = full CRT
```
