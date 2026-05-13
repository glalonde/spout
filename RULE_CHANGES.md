# Rule Change Patch Notes

Gameplay-rule changes only. Use this log for player-facing mechanics such as
scoring, timers, lives, level progression, win/loss conditions, resource rules,
and difficulty curves. Keep implementation notes in active plans or code
comments instead.

## Unreleased

### Time-Attack Timer

- Reaching a new level now adds one configured level duration to the timer
  instead of converting remaining time into score.
- Fast play can build a time bank across levels; the countdown no longer resets
  at each level boundary.
- Score is height-only again, with no remaining-time bonus.

