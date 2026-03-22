[![Build Status](https://github.com/glalonde/spout/workflows/CI/badge.svg)](https://github.com/glalonde/spout/actions)

# Spout Web version

![Spout preview](./assets/spout_preview.png)

Live streaming some of the programming ony my youtube channel: [https://youtu.be/QauR0n0V48M](https://youtu.be/QauR0n0V48M) 

Legacy version is in branch `legacy_wgpu`. It's actually in a more complete state, but is based on older libraries. I'm in the process of getting back to the same functionality now.

## Per-Frame Pipeline

Each frame the CPU updates game state, then enqueues a single `CommandEncoder` with compute and render passes in order:

```mermaid
flowchart TD
    subgraph CPU["CPU — update_state"]
        direction TB
        INPUT([Input / keyboard])
        SHIP[ShipState update]
        VPORT[viewport_offset]
        EMIT_ST[Emitter state\nemit_for_period]
        LVL_BG[/"LevelMaker\nbackground gen"/]
        INPUT --> SHIP --> VPORT
        SHIP --> EMIT_ST
    end

    subgraph BUFS["GPU Buffers"]
        direction TB
        T_TILES[(TerrainTiles\nloaded_tiles)]
        COMP[(CompositeTile\nterrain buffer)]
        P_BUF[(ParticleBuffer\nring buffer)]
        D_BUF[(DensityBuffer)]
        G_TEX[["GameViewTexture\nBGRA8 offscreen"]]
        SWAP[["SwapChain\nwindow surface"]]
    end

    subgraph COMPUTE["Compute Passes"]
        direction LR
        COMPOSE["compose_tiles\ncopy → composite"]
        EMIT_CS["Emitter Compute\nemitter.wgsl"]
        CLEAR_CS["Clear Density\nclear_density_buffer.wgsl"]
        UPDATE_CS["Update Particles\nparticles.wgsl\nphysics + terrain damage"]
        DECOMPOSE["decompose_tiles\ncopy back"]
    end

    subgraph RENDER["Render Passes"]
        direction LR
        T_RP["Terrain Render\nterrain.wgsl"]
        P_RP["Particle Render\nrender_particles.wgsl"]
        S_RP["Ship Render\nship.wgsl"]
        BLIT["Blit Quad\ntextured_model.wgsl\ncamera transform"]
    end

    LVL_BG -.->|uploads| T_TILES
    T_TILES -->|active tiles| COMPOSE --> COMP
    EMIT_ST -->|EmitParams| EMIT_CS --> P_BUF
    CLEAR_CS --> D_BUF
    COMP -->|read/write| UPDATE_CS
    P_BUF -->|read/write| UPDATE_CS
    D_BUF -->|write density| UPDATE_CS
    UPDATE_CS -->|terrain damage| COMP
    UPDATE_CS -->|particle density| D_BUF
    COMP --> DECOMPOSE --> T_TILES

    COMP --> T_RP --> G_TEX
    D_BUF --> P_RP --> G_TEX
    SHIP -->|position & angle| S_RP --> G_TEX
    G_TEX --> BLIT
    VPORT -->|camera uniforms| BLIT --> SWAP
```

> The DOT source for this diagram is at [`assets/pipeline.dot`](assets/pipeline.dot).

## Web Dev Notes

Try visiting the demo at [https://glalonde.github.io/spout/](https://glalonde.github.io/spout/)

Try running the wasm version! It might work:
```
./run_wasm.sh
```

Currently this only works on a few browser configs.

## Firefox Nightly
- Go to [about:config](about:config)
- Set `dom.webgpu.enabled` to true
- Set `gfx.webrender.all` to true