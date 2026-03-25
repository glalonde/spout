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

| Key | Action |
|-----|--------|
| W | Thrust |
| A / D | Rotate |
| T | Next track |
| Y | Toggle music |
| F | Fullscreen |
| P | Pause |

## Browser Requirements

WebGPU is required. It is available in:
- **Chrome / Edge** 113+ (enabled by default)
- **Firefox Nightly** — enable `dom.webgpu.enabled` and `gfx.webrender.all` in [about:config](about:config)
- **Safari** 18+ (Technology Preview)
