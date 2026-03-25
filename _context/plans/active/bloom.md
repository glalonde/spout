# Bloom Post-Processing Plan

## Status: Merged (bloom branch → master)

## What was done

- Added `Rgba16Float` HDR game view texture (replaces `Bgra8UnormSrgb`)
- Added `Bloom` pipeline: threshold pass → downsample blur → upsample blur → composite
- Added `VisualParams` config struct (threshold, strength, knee) in `game_params.rs`
- Wired bloom into main render loop; bloom toggleable via `game_config.toml`
- Fixed headless GPU test readback for `Rgba16Float` (8 bytes/pixel, f16→u8 conversion)
- Regenerated golden images for all three render tests

## Known issues / follow-up

### Glow pass is not smooth

The glow/bloom effect looks jagged / not smooth. Hypothesis: the blur passes are
operating at the game view resolution (e.g. 480×270 or similar low-res internal
viewport) rather than the final output resolution. Upscaling a blurry low-res
bloom onto a high-res display makes the bloom edges look blocky.

**Possible fix:** run the bloom threshold + blur in the full output (display)
resolution rather than the internal game viewport resolution. This would require
rendering the game view to a texture, then upscaling it before feeding into bloom,
or running bloom after the upscale in the `textured_model` composite pass.

See `src/bloom.rs` for current implementation.
