# Music Plan

## Current state

- `music_starts_on` flag exists in `GameParams` and `game_config.toml` (currently `true`) but nothing reads it — no audio code exists yet
- `assets/music/` has both tracker source files and pre-rendered OGGs:
  - **Tracker files** (MOD/XM/S3M): 13 files, ~800 KB total
  - **Pre-rendered OGGs** (`assets/music/output/`): ~32 MB total — dead weight now, can be deleted

## Decision: oxdz (forked, fixed)

We evaluated and eliminated the other options:
- `xmrs` — XM-only parser, no PCM output engine
- `openmpt` — C++ library, hard to build for WASM
- `playmods` — doesn't exist (hallucinated crate name)

**oxdz** (`github.com/glalonde/oxdz`) is the chosen library:
- Pure Rust, compiles to WASM — confirmed working (wasm-demo built and tested)
- Covers all our formats: MOD, XM, S3M — all 13 tracks render cleanly
- Fixed and modernized: Rust 2021, 0 warnings, all audio bugs patched
- Pull model: `play_frame()` + `buffer()` → `&[i16]` PCM per tick
- `include_bytes!` all 13 tracker files into the binary: ~800 KB vs 32 MB OGGs

### Known limitations (non-issues for our tracks)
- No Impulse Tracker (.it) — we have none
- Bidirectional sample loops unimplemented — may cause subtle timbre differences
  on some instruments; acceptable for background music
- Single maintainer, not on crates.io — using as a git dependency

## Audio output stack

```
Tracker file (include_bytes!)
    ↓
oxdz (pre-render full track at startup → Vec<f32>)
    ↓ native              ↓ wasm
  cpal stream           Web Audio API (AudioBuffer + AudioBufferSourceNode)
```

### Pre-render vs real-time streaming

**Pre-render at startup** is the chosen approach (same as the confirmed-working
wasm-demo):
- Simpler than building a real-time streaming pipeline for both native+WASM
- Render happens async / on a thread so the game loop is not blocked
- One complete loop of a typical track: ~2–4 min, ~30–60 MB of f32 PCM in
  memory — acceptable. Cap at 5 minutes to bound worst case.
- Native: render on a background thread before first frame
- WASM: render in a `spawn_local` future before starting the game loop

For future consideration: real-time streaming via cpal callback (native) or
AudioWorklet (WASM) if memory usage becomes a concern.

## Cargo dependency

```toml
[dependencies]
oxdz = { git = "https://github.com/glalonde/oxdz", branch = "master" }

# audio output
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
cpal = "0.15"
```

For WASM, audio output goes through `web-sys` (AudioContext etc.) which is
already a transitive dep.

## Implementation phases

### Phase 1 — Native playback, one track
1. Add `oxdz` + `cpal` as deps
2. Pre-render one track (`include_bytes!` a .xm) to `Vec<f32>` on a background
   thread at startup
3. Feed rendered PCM into a `cpal` output stream (i16 → f32, stereo)
4. Loop playback when the buffer ends
5. Gate on `music_starts_on` from `GameParams`

### Phase 2 — WASM playback
1. Pre-render on a `spawn_local` future (non-blocking)
2. Create `AudioContext`, fill `AudioBuffer`, play via `AudioBufferSourceNode`
3. Handle browser autoplay policy: audio context must be resumed on first user
   gesture (first keypress / click unlocks it)
4. Loop: `source.loop = true`

### Phase 3 — Track list + controls
1. Embed all 13 tracker files, build a playlist
2. Skip-track key binding (spare keys available)
3. Volume control (expose in game config or key binding)
4. Crossfade or instant switch on track change

## Cleanup tasks (can do any time)
- Delete `assets/music/output/` OGG directory (32 MB, regeneratable from source)
- Consider gitignoring the OGG output dir if kept for local reference
