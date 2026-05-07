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

## Follow-up: Xcode Metal Frame Capture (2026-05-07)

A Metal frame capture (Xcode → Debug → Capture GPU Frame, not Instruments) on
native iOS showed the bloom horizontal/vertical blur kernels accounting for a
large share of per-frame GPU time. This rules in bloom as a meaningful
contributor but does not yet answer the structural question below.

## Open Question: structural vs throughput

The earlier Instruments trace showed short individual GPU command durations
(~1.23 ms max in the summary) alongside high CPU-to-display latency
(33-50 ms) and GPU Performance State time at `Minimum`. That is consistent
with throttling/pacing rather than the GPU being unable to keep up at full
clock. We do not yet know whether:

- mobile web runs the **same workload** faster (→ native has a fixable
  per-pass inefficiency: load/store actions, pixel format, pass merging), or
- mobile web runs a **structurally smaller workload** (fewer passes, smaller
  surface, different format) and native is genuinely doing more work.

Per-pass GPU timing on web is hard to capture: production iOS Safari and
Chrome are not attachable from Instruments (no `get-task-allow` entitlement).
Firefox Focus on iOS shows up as profilable, but only if it actually runs
the WebGPU build. WebGPU `timestamp-query` may not be enabled in iOS Safari.

## Comparison-test recipe

Goal: compare frame time vs `bloom_passes` on native iOS and mobile web on
the same physical iPhone. Slope of the line attributes per-pass cost.

### 1. Frame-time log line

`src/main.rs` emits one `log::info!` line per ~60 frames in both debug and
release builds:

```
frame avg_dt=16.74ms fps=59.7 surface=2556x1179 bloom_passes=16
```

- **Native (Xcode):** appears in the Xcode console pane when the app is
  launched via the Run scheme. `env_logger` is initialized at `Info` level
  in `examples/framework.rs`, so no `RUST_LOG` is needed.
- **Web:** appears in the JS console. View it via Mac Safari → Develop →
  [iPhone] → [tab] → Console. iOS Safari supports Web Inspector (no special
  entitlement needed); iOS Chrome does not — use Safari for the comparison
  since all iOS browsers run WebKit anyway.

### 2. Tailscale HTTPS for the web build

iOS Safari WebGPU requires a secure context. `localhost` qualifies but a
plain `http://100.x.x.x:1234` does not. `run_wasm.sh --https` runs the
static file server on `127.0.0.1:1234` and fronts it with
`tailscale serve --https=443 http://127.0.0.1:1234`, exposing
`https://<host>.<tailnet>.ts.net/` to the iPhone over Tailscale.

### 3. Sweep protocol

For each value of `visual_params.bloom_passes` in `game_config.toml`
(suggested: 0, 4, 8, 12, 16):

1. Set `bloom_passes` and rebuild.
2. Cool the device down (pause for a few minutes if it just ran).
3. Play for ~30s to reach steady state, then read `avg_dt` from the log line
   for another ~30s and average.
4. Record native and web side by side.

Interpretation:

- **Equal slope, equal intercept** — same workload, same throughput; the
  cliff is thermal/pacing, not bloom workload.
- **Native slope steeper than web** — each pass is genuinely more expensive
  on native; investigate load/store actions, pixel format, pass merging.
- **Equal slope, native intercept higher** — non-bloom work (audio, present
  loop, ship/particle render) is the actual cost gap; bloom is a red
  herring.

### Known confound

Mobile web during prior testing rendered at 2202×1011; native iOS renders at
2556×1179. The web-side test inherits whatever resolution Safari chose, so
"web has fewer pixels per pass" is baked into the comparison. The log line
prints `surface=WxH` on both sides — record this alongside `avg_dt` so you
can normalize per-pixel cost if needed.

## Resolution: dual-filter bloom (2026-05-07)

The bloom-passes sweep + frame log analysis confirmed the cliff was a sustained
GPU bandwidth/throughput ceiling, not a CPU encoding cost or a native-specific
inefficiency. Per-pass GPU cost was effectively identical per pixel between
native and web (~340-390μs). Native cliffed at `bloom_passes = 16` after ~10s
of warmup; web held until forced past its instantaneous throughput limit at
`bloom_passes = 32`. Both behaviours match a model where each platform sits
just on its respective side of the device's sustainable bandwidth threshold.

The original separable-Gaussian ping-pong bloom at full surface resolution
costs roughly 33 fullscreen passes × (read + write) per frame. At 2202×1119
RGBA16Float that is ~78 GiB/s of memory bandwidth for bloom alone, which is
the proximate cause of the thermal throttling.

Replaced with **dual-filter (mip-pyramid) bloom** (Jimenez COD AW recipe):

- New shaders: `bloom_prefilter.wgsl`, `bloom_downsample.wgsl`,
  `bloom_upsample.wgsl`. Old `bloom_threshold.wgsl` and `bloom_blur.wgsl`
  removed.
- `Bloom::render` builds a mip pyramid starting at half surface resolution.
- Prefilter combines threshold + first downsample. Downsample chain uses a
  13-tap weighted box. Upsample chain uses a 9-tap tent filter with
  **additive blending** so the destination read for the blend stays in tile
  memory on TBDR GPUs (no main-memory traffic for the read-modify-write).
- `VisualParams::bloom_passes` renamed to `bloom_mip_levels`. The old field
  name still deserialises via a serde alias for backward compat. Default 6.

Predicted bandwidth at 2202×1119 with 6 mip levels:

| stage | bandwidth (× full surface area) |
|-------|---------------------------------|
| Prefilter | 1.25 |
| Downsample chain | 0.42 |
| Upsample chain | 0.42 |
| **Total** | **~2.08** |

Compared to the old pipeline's ~66× full surface area, that is
**~32× less bandwidth**. Predicted sustained ~2.5 GiB/s for bloom on native,
well under the device thermal envelope.

The composite shader (`bloom_composite.wgsl`) is unchanged. It samples the
bloom output (now the half-resolution mip 0 of the pyramid) with bilinear
filtering, which is visually fine because bloom halos are inherently wide.

Verification needed on device: confirm 60 FPS sustained at the default mip
count, and compare visual output against the previous bloom for any
unexpected halo-shape regressions.
