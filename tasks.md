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
- [ ] set fullscreen 
- [ ] show score 
- [x] pausing 
- [x] music
---
## Medium
- [ ] level generation
---
## Hard
- [ ] scrolling behavior
- [ ] progressive game mechanics
---

# Improvement Tasks
## Easy/Straightforward
- [ ] initialization (flags, logging)
- [ ] improved ship rendering
- [ ] wireframe ship rendering
- [ ] highres glow
- [ ] separated gaussian glow
- [x] CI
---
## Medium
- [ ] configs (load emitter params, etc from proto)
---
## Hard
- [ ] cross platform (windows and mac os)
---


# Scrolling Subtasks
- [x] Create N buffers for the world space data: particle accumulation, terrain
- [ ] Read the ship height in CPU and adjust the world space coordinates of the buffers to scroll
- [ ] Set orthographic camera perspective
- [x] Render camera perspective and apply glow pass / visual effects
