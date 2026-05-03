# SpoutDS Source Analysis

Source: [gtamp.com/DS/spoutDS_src.zip](https://gtamp.com/DS/spoutDS_src.zip)

Original game by kuni ([din.or.jp/~ku_](http://www.din.or.jp/~ku_/junk/junk.htm)),
DS port by Birslip, FAT save support by Sektor.

## What SpoutDS Does

The entire game lives in a single ~870-line C file (`spout.c`) plus a thin
platform layer (`main.c`). There is no GPU involvement — everything is
CPU software-rendered into byte buffers.

### World Representation

- **128x128 byte array** (`vbuff2`) — the full world. Each byte encodes terrain
  health and type via bit fields:
  - Bits 6-7: health (0xC0 mask) — terrain takes 3 hits before clearing
  - Bit 2: grain flag — if set, the cell contains a moving grain, not static terrain
  - Bits 0-1: color index (2-bit palette, 4 shades of grey)
- **128x88 display buffer** (`vbuff`) — what gets blitted to screen.
  The display is a sliding window into the 128x128 world using circular indexing.

### Coordinate System

All positions use **8.8 fixed-point** (multiply by 256). The ship position
`mPos` is in fixed-point; division by 256 yields pixel coordinates.

Rotation uses a precomputed **1024-entry sine table** with 12-bit precision
(range -4096..4096). Rotation angle `mR` is an index into this table,
incremented/decremented by 16 per frame (giving 64 discrete orientations).

### Ship

The ship is a **single pixel** with a 3x3 "crosshair" drawn around it:
```
XXX
X X
XXX
```
Plus a 3-pixel exhaust trail drawn in the thrust direction.

Collision is trivially: read the byte at the ship's pixel position in `vbuff2`.
If it's nonzero and not a grain (bit 2 clear), the ship is dead.

The ship also dies if it touches the left/right walls or the bottom.

### Particle System ("Grains")

- **Max 500 grains**, managed via a doubly-linked free list (no dynamic allocation)
- Each grain has:
  - Position (sub-pixel fixed-point within its cell)
  - Velocity (fixed-point)
  - Cell position (index into the 128x128 buffer)
  - Color/health byte

Grain simulation per frame:
1. Apply gravity (`v.y += 8`)
2. Move sub-pixel accumulators; when they overflow a cell boundary, step into
   the adjacent cell
3. On collision with terrain:
   - Decrement terrain health (bits 6-7 of the byte)
   - Bounce: reverse velocity component, halve it, add random lateral jitter
   - If terrain health reaches 0, clear the cell
   - Also decrement grain health; if grain dies, it deposits as static terrain
4. On collision with another grain: swap velocities (elastic-ish collision)

This is the core "erosion" mechanic — the exhaust particles physically
chip away at terrain.

### Terrain Generation

When the player scrolls upward:
1. A new row appears at the top, filled with solid terrain (color `0xd2`)
2. A rectangular vacancy is carved out at a random position
3. Vacancy size shrinks as height increases:
   ```c
   box.x = 20 - (height + 40) / 64;  // min 4
   box.y = 20 - (height + 40) / 64;  // min 4
   ```
4. Left/right walls (4 pixels each) are always solid — the playfield is
   120 pixels wide within the 128-pixel buffer

### Scrolling

The world is a vertically-wrapping circular buffer. The display reads 78 rows
starting from `dispPos` (= `upperLine`), wrapping around at row 128.
When the ship crosses the midpoint (y < 40), the world scrolls up by 1 row
and `height` increments.

### Scoring

- 1 point per row of height gained
- Every 128 rows: time bonus (remaining_seconds * 10 added to score),
  timer refills by 60 seconds (capped at 99)
- High score saved to FAT file or SRAM

### Title Screen

The title screen reuses the same loop but with the ship locked at a fixed
position, showing a scrolling demo with the game logo (encoded as a bitmap
constant `MATSUMI[]`), high scores, and credits.

### Game State Machine

```
gamePhase 0: title init / high-score save
gamePhase 1: title scrolling
gamePhase 2: game init
gamePhase 3: gameplay
gamePhase 4: paused
```

Transitions: A/B starts game from title, A/B from game-over returns to title.
Start+Select from gameplay returns to title; from title, exits.

## Comparison with Our Spout

### What's the Same

| Aspect | SpoutDS | Our Spout |
|--------|---------|-----------|
| Core mechanic | Exhaust particles erode terrain | Same |
| Goal | Fly upward as high as possible | Same |
| Scrolling direction | Upward (Y-up world) | Same |
| Terrain = destructible grid | Yes, byte per cell | Yes, int per cell |
| Particles bounce off terrain | Yes, with health decrement | Yes (atomicAdd erosion) |
| Ship has thrust + rotation | Yes | Yes |
| Gravity | Constant downward | Same |
| Ship dies on terrain contact | Yes | Yes |
| Terrain gets denser with height | Yes | Yes |
| Score = height-based | Yes, +time bonuses | Score increments |
| Timer mechanic | 60s, refills every 128 rows | Not implemented |
| Walls on left/right | 4px solid walls | Not implemented |

### What's Different

| Aspect | SpoutDS | Our Spout |
|--------|---------|-----------|
| Resolution | 128x88 (fixed) | Arbitrary GPU viewport |
| Rendering | CPU byte-buffer blit | GPU: compute shaders + render pipelines |
| Color depth | 2-bit (4 grey shades) | HDR float16 with bloom |
| Particle count | Max 500 | Configurable (thousands) |
| Particle sim | CPU linked-list, per-pixel stepping | GPU compute shader |
| Terrain storage | 128x128 byte circular buffer | GPU int buffer, row-ring |
| Terrain health | 3 hits (2-bit counter) | Configurable int health |
| Ship representation | 1 pixel + 3x3 sprite | Triangle with hull vertices |
| Ship collision | Pixel read at ship position | GPU Bresenham walk of hull vertices |
| Rotation | 64 discrete angles (1024 sine LUT) | Continuous (f32 radians) |
| Fixed-point math | Yes (8.8 integer) | No (f32 everywhere) |
| Grain-grain collision | Yes (velocity swap) | No |
| Grain deposits as terrain | Yes (dead grains become terrain) | No |
| Audio | None (DS port) | Tracker music (oxdz) |
| Title screen | Scrolling demo with logo | Not implemented |

### Ideas We Could Borrow

1. **Timer mechanic**: The countdown timer that refills every N rows adds
   urgency. Currently we have no time pressure — the player can just sit still.
   This is arguably the most important missing gameplay element.

2. **Grain-grain collisions**: SpoutDS grains swap velocities when they collide.
   This creates more organic particle behavior. Would need an additional GPU
   pass or spatial hash, but could add satisfying visual chaos.

3. **Grain → terrain deposition**: Dead grains become static terrain in SpoutDS.
   This creates a dynamic where eroded debris can re-form obstacles. Very cool
   emergent behavior.

4. **Wall boundaries**: The 4px solid walls on left and right constrain the
   playfield and prevent trivially flying around obstacles. We could add
   indestructible side walls.

5. **Title screen state**: The scrolling demo/title screen reuses the game
   loop with the ship on autopilot. Clean state machine we should implement.

6. **Terrain health visualization**: In SpoutDS the 2-bit health maps directly
   to shade (darker = more health). We already show terrain as binary but could
   tint by remaining health.

7. **Score = time bonus system**: Every 128 rows, `remaining_time * 10` is
   added to score. This rewards fast play — you're incentivized to rush
   through obstacles, not carefully clear them all.

8. **Difficulty curve**: SpoutDS makes the vacancy rectangles shrink with
   `20 - height/64` (min 4). Very simple but effective. Our curve formula
   is more complex but achieves a similar effect.

9. **Discrete rotation**: Not something to borrow (continuous feels better),
   but worth noting that the original only had 64 rotation steps.
