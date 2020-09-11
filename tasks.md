# MVP Tasks
## Easy/Straightforward
- [x] ship rendering
- [ ] gravity
- [x] ship position rendering 
- [x] ship motion
- [ ] ship collision detection 
- [x] text rendering
- [x] terrain collision capabilities
- [ ] resolution selection
- [ ] window aspect ratio
- [x] set fullscreen 
- [x] show score 
- [x] pausing 
- [x] music
---
## Medium
- [x] level generation
- [ ] progressive level generation
---
## Hard
- [x] scrolling behavior
- [ ] progressive game mechanics
---

# Improvement Tasks
## Easy/Straightforward
- [ ] initialization (flags, logging)
- [x] improved ship rendering
- [ ] wireframe ship rendering
- [ ] highres glow
- [ ] separated gaussian glow
- [x] CI
---
## Medium
- [ ] configs (load emitter params, etc from proto)
---
## Hard
- [x] cross platform (windows and mac os)
- [x] CI builds linux binaries
- [ ] CI builds macOS binaries
- [ ] CI builds Windows binaries
---

# Scrolling Subtasks
- [x] Create N buffers for the world space data: particle accumulation, terrain
- [x] Read the ship height in CPU and adjust the world space coordinates of the buffers to scroll
- [x] Set orthographic camera perspective
- [x] Render camera perspective and apply glow pass / visual effects


# 4/25/20 weekend goals
- [x] Fix level reset bug
- [ ] Progressive, random level generation
- [x] Get levels past level 2