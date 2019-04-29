#version 430
#extension GL_ARB_compute_variable_group_size : enable
layout(location = 0) uniform float dt;
layout(location = 1) uniform int anchor;
layout (binding = 0, r32ui) uniform uimage2D counter_texture;

struct Particle {
  uvec2 position;
  ivec2 velocity;
};

layout(std430, binding = 0) buffer Particles {
  Particle particles[];
};

void GetCell(in uvec2 pos, out ivec2 cell) {
  // High res position is stored in the 8 low order bits
  // Low res position is stored in the 24 high order bits
  cell.x = int(pos.x >> 8) - anchor;
  cell.y = int(pos.y >> 8) - anchor;
}

layout(local_size_variable) in;
void main() {
  uint gid = gl_GlobalInvocationID.x;
  Particle p = particles[gid];
  ivec2 signed_delta = ivec2(p.velocity * dt);
  uvec2 end_pos = p.position + signed_delta;

  uvec2 delta = abs(signed_delta);
  ivec2 step = sign(signed_delta);
  ivec2 cell;
  GetCell(p.position, cell);
  particles[gid].position = end_pos;
  imageAtomicAdd(counter_texture, cell, 1);
}
