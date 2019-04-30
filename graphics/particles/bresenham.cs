#version 430
#extension GL_ARB_compute_variable_group_size : enable
layout(location = 0) uniform float dt;
layout(location = 1) uniform int anchor;
layout(binding = 0, r32ui) uniform uimage2D counter_texture;

const uint kMantissaBits = 8;
const uint kCellSize = 1 << kMantissaBits;
const uint kHalfCellSize = 1 << (kMantissaBits - 1);

struct Particle {
  uvec2 position;
  ivec2 velocity;
  ivec2 debug;
};

layout(std430, binding = 0) buffer Particles {
  Particle particles[];
};

ivec2 GetCell(in uvec2 pos) {
  return ivec2(pos >> kMantissaBits) - anchor;
}

uvec2 GetRemainder(in uvec2 pos) {
  const uint kHighResMask = (1 << kMantissaBits) - 1;
  return pos & kHighResMask;
}

layout(local_size_variable) in;
void main() {
  uint gid = gl_GlobalInvocationID.x;
  Particle p = particles[gid];
  ivec2 signed_delta = ivec2(p.velocity * dt);
  uvec2 end_pos = p.position + signed_delta;

  ivec2 delta = abs(signed_delta);
  ivec2 step = ivec2(signed_delta.x > 0 ? 1 : -1, signed_delta.y > 0 ? 1 : -1);
  ivec2 current_cell = GetCell(p.position);
  ivec2 end_cell = GetCell(end_pos);
  ivec2 delta_i = end_cell - current_cell;

  // Starting cell remainder:
  ivec2 start_remainder = ivec2(kHalfCellSize, kHalfCellSize) - ivec2(GetRemainder(p.position));
  start_remainder *= step;

  // Error value
  int error = delta.x * start_remainder.y - delta.y * start_remainder.x;
  delta *= int(kCellSize);

  int num_cells = abs(delta_i.x) + abs(delta_i.y);
  while (num_cells > 0) {
    int error_horizontal = error - delta.y;
    int error_vertical = error + delta.x;
    if (error_vertical > -error_horizontal) {
      // Horizontal step
      error = error_horizontal;
      current_cell.x += step.x;
      // Check cell
    } else {
      // Vertical step
      error = error_vertical;
      current_cell.y += step.y;
      // Check cell
    }

    --num_cells;
  }

  particles[gid].position = end_pos;
  particles[gid].debug = end_cell - current_cell;

  // Draw to the density texture
  imageAtomicAdd(counter_texture, current_cell, 1);
}
