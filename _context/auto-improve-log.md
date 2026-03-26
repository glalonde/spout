# Autonomous Improvement Log — 2026-03-26

## Completed Work

### PR #53 — Code cleanup (branch: auto-improve-2026-03-26)
- `bb8d880` — **refactor**: removed dead `load_image` module, dead `ColorMap` enum, dead `stripe_level` field; added justification comments to all `unwrap()` calls
- `f2d26bd` — **test**: 16 new unit tests for ship physics (9) and angle_diff (7); test count 34 → 50
- `0c900d2` — **docs**: added `//!` module doc comments to all 14 source files that lacked them; added `.claude/worktrees/` to `.gitignore`
- `370a1f7` — **perf**: eliminated per-frame `Vec::new()` allocation in `LevelManager::work_until()` — replaced with `Option<u32>`
- `b53ea60` — **refactor**: cleaned up `shader_util` macro — removed dead commented-out hot-reload code and noisy per-shader log line
- `b8dd826` — **test**: 6 new tests for `color_maps` and `game_params` (embedded config, defaults, missing sections, invalid TOML); test count 50 → 56
- `34f3d1d` — **docs**: added auto-improve log and updated near-term plan
- `1a6b780` — **fix**: corrected misleading "Orthographic projection" comment on `CameraState.perspective` field
- `13907f8` — **feat**: config loading tries disk file first (`game_config.toml` in CWD), falls back to embedded default; supports both dev and packaged .app workflows
- `75d240d` — **refactor**: restricted `make_stripe_level` to `#[cfg(test)]` — eliminated unused function warning

### PR #54 — macOS .app packaging (branch: macos-packaging)
- `14b0606` — **feat**: `macos/Info.plist`, `scripts/package_macos.sh` (builds + bundles + ad-hoc signs), `.github/workflows/release-macos.yml` (tag-triggered release), `_context/macos-signing.md` (future signing docs)
- Tested locally: script produces a valid signed `.app` bundle

### PR #55 — Text renderer + debug overlay (branch: text-rendering)
- `5bed466` — **feat**: bitmap font text renderer using `fontdue` + Pixel Six TTF. GPU texture atlas, instanced glyph quad WGSL shader, nearest-neighbor filtering. FPS + score debug overlay toggled with F3. New dep: `fontdue = "0.9"`.

## Stats
- **Tests**: 34 → 56 (+22)
- **Dead code removed**: 3 modules/fields/enums
- **Unwrap calls justified**: all production code
- **Module docs added**: 14 files
- **New features**: text rendering, FPS overlay, macOS packaging, config disk override
- **Per-frame allocations eliminated**: 1

## Findings / Notes
- `game_config.toml` is already embedded via `include_str!` — enhanced with optional disk override for dev flexibility.
- `glyphon` does not support wgpu 29. Hand-rolled bitmap atlas with `fontdue` was the right approach.
- `AudioPlayer::disabled()` IS used (from `main.rs`), contradicting the initial dead code scan — always verify subagent findings before acting.
- The only per-frame heap allocation found was the `Vec::new()` in `work_until()` — everything else in the hot path is stack-allocated or pre-allocated.
