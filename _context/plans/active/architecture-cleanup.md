# Architecture Cleanup

Active follow-up from the May 2026 architecture review. Apply the principle of
progressively revealing complexity: top-level files should name game phases and
screen flow, while detailed GPU/platform/UI mechanics live behind narrower
modules.

## Recently Completed

Merged in PRs #77-#84:
- [x] Replaced boolean game-state flags with an explicit `AppState` enum and
      `Play` state.
- [x] Split the frame loop into `update_phase`, GPU-bound transitions,
      `draw_phase`, and `post_phase`.
- [x] Extracted title-screen behavior into `src/screens/title.rs`.
- [x] Added `InputFrame` edge helpers and routed audio hotkeys through input.
- [x] Grouped render resources into `src/graphics.rs`.
- [x] Replaced the title `?` affordance with `PLAY` / `MENU` buttons and exact
      game-space hit testing.
- [x] Fixed collision readback while paused so in-flight readbacks do not block
      later dispatch after unpausing.
- [x] Added `GameParams::validate()`, fail-fast embedded config parsing,
      unknown-field rejection, and semantic range tests for config knobs.
- [x] Guarded the current width invariant by validating
      `level_width == viewport_width` until terrain/view/density widths split.
- [x] Renamed the CPU `Particle` ABI field to `subframe_dt_offset`, made it
      a one-frame dt offset with `0.0` as the neutral value.
- [x] Guarded `clear_density_buffer.wgsl` against rounded-up workgroup lanes
      and added a non-workgroup-multiple density clear test.
- [x] Kept particle terrain erosion on the fast atomic decrement path and
      documented that overkill damage may drive destroyed cells negative.

## Highest Priority

- [x] Fix density-buffer clear dispatch bounds. `clear_density_buffer.wgsl`
      writes `density_buffer[gid]` without guarding rounded-up workgroup lanes;
      the checked-in 261×160 viewport dispatches 41,984 lanes for 41,760 cells.
      Add an `arrayLength` guard and cover a non-workgroup-multiple density
      size.
- [x] Add config validation and fail-fast checked-in config parsing. Today the
      loader can fall through to `GameParams::default()` after a broken embedded
      `game_config.toml`, and tests can still pass. Add `GameParams::validate()`,
      run it after every parse path, and test `include_str!("../game_config.toml")`
      directly.
- [x] Guard width invariants before wider-level work. Particle and terrain
      shaders still conflate view width, terrain row stride, and density-buffer
      width: `ParticleSystem::new` writes `level_width` into the shader
      `viewport_width` uniform while the density buffer is allocated with
      `viewport_width`, and `TerrainRenderer` has the same naming confusion.
      Validate `level_width == viewport_width` for now, then split terrain
      width, view width, and density-view width before enabling wider levels.
- [x] Fix the CPU/GPU particle ABI: Rust called the fourth `Particle` field
      `_padding`, but WGSL treats it as `subframe_dt_offset`. Rename it to
      `subframe_dt_offset`, make it a one-frame dt offset with `0.0` as the
      neutral value, and keep the Rust/WGSL layout explicitly aligned.
- [ ] Revisit collision scheduling beyond pause handling. The current
      `PendingCollisionSegment` still extends one segment to the latest ship
      pose while readback is in flight; curved/dogleg motion can collapse into
      one chord. The native readback lifecycle can occupy the in-flight slot
      across extra frame motion. Introduce a `CollisionTracker` with queued
      per-frame motion segments or a small ring of readback buffers.

## Gameplay / Simulation

- [ ] Make terrain mutation order a named GPU-frame contract. The current draw
      frame composes terrain, runs particle erosion, then dispatches ship
      collision against the mutated terrain. Keep that if desired, but encode it
      explicitly as `collide_after_erosion` or change the order deliberately.
- [ ] Add a frame-boundary terrain clamp only if future systems need
      nonnegative stored health. Today `try_erode` intentionally uses
      `atomicAdd` for speed, may drive destroyed cells negative on overkill,
      and render/collision code treats `<= 0` as empty.
- [ ] Centralize ship geometry shared by Rust and WGSL so render, collision,
      and emitter offsets cannot drift again.
- [ ] Rename or hide CPU terrain query helpers that read initial pre-erosion
      level data, so future AI/debug/gameplay code does not confuse them with
      authoritative mutable terrain. This is lower urgency until production code
      consumes those helpers.
- [ ] Decide the terrain-authority and eviction model for tile recycling.
      `decompose_tiles` persists GPU mutations only into loaded GPU tile
      buffers; CPU `level_maker.levels` remains initial terrain. Recycling old
      buffers safely needs an explicit persistence/eviction rule.
- [ ] Add seeded level generation for reproducible runs/tests. This is useful,
      but lower priority than tile persistence unless deterministic replay work
      starts soon.
- [ ] Remove or defer unreachable placeholder `AppState` variants
      (`Settings`, `Leaderboard`) until those screens exist; today they add
      match noise without reducing complexity.
- [ ] Avoid startup double initialization: `Spout::init` builds gameplay level
      and particle resources, submits them, then immediately rebuilds title
      resources through `transition_to_title`.

## Rendering / UI

- [ ] Move render-target formats and texture creation out of `bloom.rs` into a
      render-target/formats owner; bloom should consume the HDR format, not own
      the whole game-view contract.
- [ ] Fix and test bloom mip-level clamping. The current clamp appears to
      undercount the legal 1×1 tail level for power-of-two texture dimensions.
- [ ] Extract a behavior-preserving `FrameRenderer` facade after `Graphics`.
      `Graphics` currently groups resources, but `Spout::draw_phase` still owns
      pass order, overlays, submit/readback timing, and HUD drawing. Move the
      frame rendering API behind a narrow facade while keeping compute/collision
      scheduling explicit for now.
- [ ] Batch dynamic UI instances with reusable/ring buffers. `TextRenderer` and
      `UiRenderer` both create fresh GPU buffers per draw; use a shared dynamic
      instance upload helper for text glyphs and UI rects.
- [ ] Unify post-composite surface overlays or move them before final composite
      so non-sRGB WebGPU surfaces get one consistent color-space policy.
- [ ] Centralize game-to-surface mapping. CPU hit testing, title overlay WGSL,
      and camera letterboxing each encode related integer-scale/letterbox math.
      Fold this into the `SurfaceMetrics` work so rendering and input share one
      coordinate policy.
- [ ] Have screens emit a small UI draw model (clear color, rects, text runs,
      overlay flags) instead of taking raw GPU rendering context. This keeps
      future settings/leaderboard screens from opening render passes directly.
- [ ] Fold fullscreen-pass boilerplate cleanup into the render-target/overlay
      work; do not add a standalone abstraction unless it removes real repeated
      pass setup.
- [ ] Small cleanup: remove unused constructor parameters such as the discarded
      `queue` in `TouchZoneIndicator::new`, and store decoded background tile
      dimensions instead of hardcoding `tile_size: 65.0`.
- [ ] Align shader ABI names and constants: split particle `view_width`,
      `density_width`, and `terrain_stride`; make terrain uniform signedness
      match Rust; resolve dead emitter velocity fields; and move density heat
      scale into a shared include or build-time constant.

## Platform / Boundaries

- [ ] Add a `SurfaceMetrics` boundary owned by `app.rs`. Explicitly distinguish
      surface/backing pixels, window inner/outer sizes, scale factor, game
      viewport, and pointer coordinate mapping. Pass one metrics object through
      input, resize, rendering, debug text, and UI hit testing.
- [ ] Add `RuntimeCommand` / `RuntimeCapabilities`. `Spout` should request
      runtime actions (fullscreen, cursor visibility, quit/cancel semantics)
      and consume capabilities (touch-first UI, tap restart), while `app.rs`
      executes window/platform policy. This should also fix the current
      split-brain Escape handling where `app.rs` exits before `InputCollector`
      can expose `menu_cancel`.
- [ ] Move WASM touch attachment and native/window event adaptation out of
      gameplay-owned `Spout` over time. Split platform adapters
      (`WinitInputAdapter`, `DomTouchAdapter`) from pure logical input.
- [ ] Split logical input domains: `PlayerInput`, `UiInput`, `RuntimeAction`,
      and debug controls are currently all fields on one `InputState`.
- [ ] Finish extracting audio decoder/executor/sink seams. `audio.rs` already
      has a backend trait and playlist state; the remaining coupling is track
      decoding/job execution inside native and WASM `start_track`. Keep
      `AudioPlayer::toggle/next_track` stable so a WASM Web Worker executor can
      land without reshaping callers.

## Config / CI

- [x] Add semantic `GameParams` range tests for public knobs (`color_map`,
      density scale/exponent, particle counts/lifetimes, widths). This should
      share the validation path from the highest-priority config task.
- [ ] Consolidate tag release publishing: Linux and macOS workflows both trigger
      on `v*` tags and can race to create/update the GitHub Release. Use one
      publisher after artifacts are built, with explicit `contents: write`
      permissions.
- [ ] Make visual regression tests blocking with deterministic pixel/histogram
      assertions. Prefer this over restoring fragile goldens; the project has
      already removed unmaintainable golden files once.
- [ ] Ensure at least one GPU availability check fails in CI if the expected
      headless adapter is unavailable; individual GPU tests currently skip
      themselves when no adapter is found.
