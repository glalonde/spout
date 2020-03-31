#version 430
layout(location = 0) uniform int start_index;
layout(location = 1) uniform int num_emitted;
layout(location = 3) uniform float ttl_min;
layout(location = 4) uniform float ttl_max;
layout(location = 5) uniform uvec2 start_position;
layout(location = 6) uniform uvec2 end_position;

layout(local_size_x = 512, local_size_y = 1, local_size_z = 1) in;

struct Particle {
  uvec2 position;
  ivec2 velocity;
  float ttl;
  uint padding;
};

layout(std430, binding = 0) buffer Particles {
  Particle particles[];
};

float rand(float x) {
  return fract(sin(x)*10000.0);
}

void main() {
  int gid = int(gl_GlobalInvocationID.x);
  int total_particles = int(gl_NumWorkGroups * gl_WorkGroupSize);
  int distance = gid - start_index;
  if (distance < 0) {
    distance += total_particles;
  }
  if (distance >= num_emitted) {
    return;
  }

  float interp = float(distance) / num_emitted;
  // Start existing
  particles[gid].ttl = ttl_max - (ttl_max - ttl_min) * rand(float(gid));
  vec2 delta = mix(vec2(0, 0), vec2(end_position - start_position), interp);
  particles[gid].position = start_position + ivec2(delta);
}
