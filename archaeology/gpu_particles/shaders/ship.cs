#version 430

layout(location = 0) uniform float dt;
layout(location = 1) uniform vec2 acceleration;

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

struct Particle {
  uvec2 position;
  ivec2 velocity;
  float ttl;
  uint padding;
};

layout(std430, binding = 0) buffer Particles {
  Particle particles[];
};

void main() {
  int gid = int(gl_GlobalInvocationID.x);
  particles[gid].ttl = 1000.0;
  ivec2 dv = ivec2(dt * acceleration);
  particles[gid].velocity += dv;
}
