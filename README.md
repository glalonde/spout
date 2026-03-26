[![Build Status](https://github.com/glalonde/spout/workflows/CI/badge.svg)](https://github.com/glalonde/spout/actions)

# Spout

![Spout preview](./assets/spout_preview.png)

Asteroids-style 2D arcade game with GPU-accelerated particle terrain destruction. Runs natively on desktop (via wgpu/Metal/Vulkan/DX12) and in the browser (via WebAssembly + WebGPU).

Live programming streams on YouTube: [https://youtu.be/QauR0n0V48M](https://youtu.be/QauR0n0V48M)

See [docs/pipeline.md](docs/pipeline.md) for a diagram of the per-frame compute and render pipeline.

> **Legacy version** is in branch `legacy_wgpu` — more feature-complete but based on older libraries.

## Play

**Native:**
```
cargo run
```

**Web (WebAssembly):**
```
./run_wasm.sh
```

Or try the live demo: [https://glalonde.github.io/spout/](https://glalonde.github.io/spout/)

## Controls

### Keyboard (desktop)

| Key | Action |
|-----|--------|
| W | Thrust |
| A / D | Rotate |
| T | Next track |
| Y | Toggle music |
| F | Fullscreen |
| P | Pause |

### Mobile Web (touch + accelerometer)

The screen is split into two zones in landscape orientation:

- **Left half** — touch anywhere to thrust.
- **Right half** — drag to steer. The drag direction controls where the ship's exhaust points; the ship nose faces the opposite way. Lift and re-place to reset the anchor.

**Accelerometer steering** is also active on supported devices (Android, non-iOS). Tilt the phone to steer — the control is relative, so your current holding position gradually becomes the new neutral over ~5 seconds. Tap the right half (without dragging) to instantly recalibrate the neutral orientation.

Touch steering always takes priority over the accelerometer.

## Browser Requirements

WebGPU is required. It is available in:
- **Chrome / Edge** 113+ (enabled by default)
- **Firefox Nightly** — enable `dom.webgpu.enabled` and `gfx.webrender.all` in [about:config](about:config)
- **Safari** 18+ (Technology Preview)
