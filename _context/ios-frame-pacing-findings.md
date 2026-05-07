# iOS Native vs Mobile Web Frame Pacing

**Date:** 2026-05-06  
**Status:** Investigation notes; no accepted fix yet.

## Problem

On the same physical iPhone, Spout runs smoothly through mobile web/WASM but
native iOS can become choppy after a short play session.

Observed behavior:

- Mobile web on the iPhone stays smooth.
- Native iOS starts near 60 FPS, then often drops into an uneven ~40-42 FPS
  range after roughly 20-30 seconds.
- If the native app is killed and relaunched while the device is already warm,
  the drop can happen much sooner.
- macOS native remains smooth.

This points away from a purely game-logic issue and toward an iOS-native
rendering, presentation, thermal, or sustained workload difference.

## Known Differences

- Mobile web effective viewport observed during testing: `2202x1011`.
- Native iOS drawable observed during testing: `2556x1179`.
- Native iOS therefore had about 35% more pixels in fullscreen passes.
- The game renders internally at a low fixed game resolution, but bloom and
  final post-processing can still run at surface/display resolution.
- `game_config.toml` previously used `bloom_passes = 16`, which means bloom does
  threshold plus 32 fullscreen blur passes every frame.
- Native iOS currently has native audio/music paths that may differ from mobile
  web defaults.
- Native iOS presentation is Metal/CAMetalLayer via wgpu/winit. Mobile web uses
  browser WebGPU plus `requestAnimationFrame`/browser compositor pacing.

## Instruments Findings

A Game Performance trace was captured on an iPhone while the native app was
already in the bad/choppy state.

Key observations:

- Thermal State reported `Serious thermal state` for the entire captured window
  (`00:00.000` through `00:16.512`).
- Displayed surfaces were still `Direct to Display: Yes`.
- Near the end of the capture, displayed surface durations included both
  `16.67 ms` and `33.34 ms`, consistent with missed display slots rather than a
  steady intentional lower framerate.
- CPU-to-display latency near the sampled displayed surfaces was high, around
  `33-50 ms`.
- GPU Channel Activity showed short individual GPU command durations; max
  duration observed in the summary was about `1.23 ms`.
- Average CPU-to-GPU latency was around `20.54 ms`, suggesting queueing/pacing
  pressure rather than a single long GPU command.
- GPU Performance State included time at `Minimum`, which is consistent with
  thermal or power limiting.
- Time Profiler did not show `nextDrawable` as a dominant CPU blocker in the
  sampled run.

Interpretation: the trace most strongly points to sustained thermal/power
pressure causing the system to lower GPU performance and miss display slots. It
does not look like one shader invocation or one obvious CPU function suddenly
takes 25 ms.

## Experiments Tried

- Native frame pacing was changed to honor the configured 60 FPS target instead
  of accidentally running faster on ProMotion devices.
- Xcode Debug device builds were adjusted to build optimized Rust so native iOS
  comparisons are not using Cargo's unoptimized dev profile.
- A CADisplayLink-based redraw driver was tried so native iOS redraws are
  phase-locked to the display, similar in spirit to web `requestAnimationFrame`.
- Surface acquisition was moved later so expensive offscreen work happens before
  grabbing the current drawable.
- Temporary FPS/performance overlay metrics were added to release builds.
- An iOS-only reduced-resolution bloom target was tried. It reduced the
  bandwidth target but made the image visibly fuzzier, so it was reverted.
- Bloom pass count was reduced separately during testing; this is likely a more
  visually acceptable knob than lowering bloom resolution.

## Current Working Theory

The main discrepancy is likely not "native Metal is slower than web". The more
likely issue is that native iOS was asking for more sustained work:

- more drawable pixels than mobile web,
- expensive fullscreen HDR bloom at surface resolution,
- native-only audio/presentation differences,
- and enough sustained thermal load that the device enters a lower performance
  state.

Once the device is thermally limited, the app can still render individual GPU
commands quickly but fail to consistently land every display interval.

## Suggested Next Checks

Use tight A/B tests on the same physical phone:

1. Compare native iOS with bloom disabled or very low `bloom_passes`.
2. Compare native iOS with music forced off against mobile web with music on.
3. Capture another Game Performance trace after reducing bloom passes to see
   whether Thermal State avoids `Serious` and GPU Performance State avoids
   `Minimum`.
4. Watch the overlay fields around the cliff:
   - `g`: redraw gap,
   - `a`: surface acquire time,
   - `r`: final surface render time,
   - `v`: present call time.
5. If reducing bloom passes fixes sustained FPS without changing sharpness, keep
   bloom full-resolution and tune pass count/strength rather than reducing bloom
   target resolution.

## Caution

Mobile devices are expected to allow short GPU bursts above their sustained
thermal envelope. A fix should target a stable sustained budget, not maximum
short-burst GPU use.
