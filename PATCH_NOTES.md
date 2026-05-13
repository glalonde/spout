# Patch Notes

Current state at HEAD: gameplay scoring is height-only, and the time-attack
timer uses banked level-time awards.

This log tracks player-facing gameplay rule changes. Each entry includes the
commit that changed the rule, the commit date, and a short description.

## Rule Changes

- `f44b7ca` — 2026-05-13 — Replaced remaining-time score bonuses with banked
  timer awards. Reaching a new level now adds one configured level duration to
  the countdown, allowing fast play to build a time bank. Score is height-only.
- `44b1784` — 2026-05-10 — Added the per-level time-attack timer. Each level
  had a countdown; reaching the next level early awarded remaining time as score
  at 10 points per second and reset the countdown. Running out of time ended the
  run.
