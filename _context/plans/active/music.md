# Music Plan

## Status: Phase 1 + Phase 2 complete (merged to master)

## Current state
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

### Phase 1 — Native playback ✅
- `oxdz` + `cpal` deps added; all 13 tracker files embedded via `include_bytes!`
- Pre-renders track on background thread → `Vec<f32>` → cpal output stream
- Loops playback; gated on `music_starts_on` from `GameParams`

### Phase 2 — WASM playback ✅
- `spawn_local` future pre-renders the track (no blocking)
- `AudioContext` + `AudioBuffer` + `AudioBufferSourceNode`; `source.loop = true`
- Resumes AudioContext each frame until Running (handles browser autoplay policy)

### Phase 2b — Track list + controls ✅
- All 13 tracker files in a shuffled playlist
- `T` key = next track, `Y` key = toggle music on/off
- `music_starts_on = false` default in `game_config.toml`

### Phase 3 — Non-blocking render (WASM)

**Problem:** On WASM, `render_track` is called inside `wasm_bindgen_futures::spawn_local`. Because WASM is single-threaded, `spawn_local` does not run on a separate thread — it runs on the JS event loop. `render_track` is synchronous and CPU-heavy (decodes a full tracker file, ~2–4 s), so it blocks the main thread: no rendering, no input, visible freeze until the track is ready.

On native this is already correct — `std::thread::spawn` puts the render on a real background thread.

**Fix options:**
- **Web Worker** (proper fix): run `render_track` inside a dedicated Web Worker via `wasm-bindgen-rayon` or `gloo-worker`. Requires `SharedArrayBuffer` + COOP/COEP headers, which GitHub Pages supports.
- **Chunked/async render** (no-worker fallback): restructure `render_track` into an async generator that yields between frames, so the browser event loop can breathe. More complex to implement correctly.

**Recommended:** Web Worker approach. The worker compiles `oxdz`, renders the PCM buffer, posts it back to the main thread, and `poll()` picks it up — same as the existing native channel pattern.

- [ ] Add `wasm-bindgen-rayon` (or equivalent worker crate) to WASM deps
- [ ] Move `render_track` call into a worker; post result back via `postMessage`
- [ ] Verify no freeze on WASM when starting a new track

### Phase 4 — Future
- Volume control (game config or key binding)
- Crossfade on track change

## Cleanup tasks
- [x] Delete `assets/music/output/` OGG directory (32 MB, regeneratable from source) — removed via `git rm`
