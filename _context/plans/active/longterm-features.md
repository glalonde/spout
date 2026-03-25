# Long-Term Feature Backlog

Potential features to keep in mind when making architectural decisions.
Not a roadmap — a reference to avoid boxing ourselves in.

Items marked ✦ are considered likely workable and valuable.

---

## Gameplay

- ✦ **Ship collision detection** — particles currently phase through terrain; ship does not collide with it
- ✦ **Enemies / AI** — hostile ships, turrets, patrol patterns
- **Multiple ships / co-op** — local multiplayer or networked
- **Destructible ship parts** — wings, thrusters as separate physics objects
- ✦ **Scoring / progression** — points, lives, level transitions
- ✦ **Power-ups / pickups** — weapons, shields, fuel canisters
- ✦ **Weapons beyond thruster** — bombs, forward gun, missiles
- ✦ **Upgradeable nozzle** — mutable nozzle properties (speed, spread, TTL, emitter shape) as in-game progression

## Physics / Simulation

- **Particle–particle interaction** — density-based pressure, fluid-like behavior
- **Terrain reconstruction** — terrain regrows or repairs over time
- **Rigid body debris** — chunks of terrain breaking off as physics objects
- ✦ **Wind / flow fields** — environmental forces on particles

## Falling Sand / Cellular Material

Spout's terrain buffer is already a grid of integer cells — a natural foundation for cellular automaton behavior. These features would blur the line between particle effects and world simulation (à la Noita).

- ✦ **Particle deposition** — particles that exhaust their TTL deposit material into the terrain buffer, forming accumulating piles; natural extension of the existing erosion system
- ✦ **Material types** — terrain cells have a material ID (rock, sand, liquid, fire) with distinct erosion resistance, color, and behavior
- ✦ **Gravity-driven loose material** — sand/gravel cells fall if unsupported; eroded terrain crumbles rather than simply disappearing
- **Liquid simulation** — water/lava cells spread sideways and pool; interacts with particle heat
- **Fire / heat propagation** — hot particles ignite flammable terrain cells; fire spreads to neighbors and produces smoke particles
- **Gas / pressure cells** — explosive cells release a pressure wave when ignited, pushing particles and the ship

## World / Level

- **Procedurally infinite levels** — scrolling forever with seeded generation
- **Wider levels** — horizontal as well as vertical scrolling
- **Level editor** — place terrain, enemies, pickups
- **Save/load level state** — checkpoints, persistent destruction

## Rendering / Visual

Visuals are considered high-priority — the game lives or dies on how stimulating it looks.

- ✦ **Particle color / heat** — color particles by speed, age, or type; hot = bright/white, cool = dim/colored
- ✦ **Glowing terrain edges** — SDF or edge-detect pass to give terrain a lit/glowing border where it meets space
- ✦ **Background texture / parallax** — nebula, star field, or noise-based background layer with depth
- ✦ **Terrain texture** — procedural or sampled texture overlaid on terrain rather than flat color
- ✦ **Bloom** — glow/halo on bright particles and terrain edges; relatively cheap post-process pass
- ✦ **CRT / phosphor filter** — scanlines, barrel distortion, phosphor persistence; fits the retro aesthetic
- ✦ **Vector screen effect** — simulate an oscilloscope/Asteroids-original look; particles and ship drawn as glowing vector lines rather than rasterized pixels
- ✦ **Multiple particle types** — smoke, sparks, debris with distinct visual profiles
- **Lighting** — particles as dynamic light sources illuminating nearby terrain
- **High-DPI / resolution scaling** — proper handling of device pixel ratio

## Audio

- **Procedural sound effects** — thruster pitch tied to speed/throttle
- ✦ **WASM audio** — Web Audio playback (Phase 2 of music plan)

## Platform / Infrastructure

- **WASM revival** — get the web target building and running again
- ✦ **Gamepad input** — controller support via gilrs or winit gamepad events

- **Configurable keybindings** — runtime remapping beyond game_config.toml
