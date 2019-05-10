layout(location = 0) uniform int start_index;
layout(location = 1) uniform int num_emitted;
layout(location = 3) uniform float ttl_min;
layout(location = 4) uniform float ttl_max;
layout(location = 5) uniform float random_seed;
layout(location = 6) uniform uvec2 start_position;
layout(location = 7) uniform uvec2 end_position;
layout(location = 8) uniform float emit_velocity_min;
layout(location = 9) uniform float emit_velocity_max;

layout(local_size_x = 512, local_size_y = 1, local_size_z = 1) in;

#define M_PI 3.1415926535897932384626433832795


struct Particle {
  uvec2 position;
  ivec2 velocity;
  float ttl;
  float padding;
};

layout(std430, binding = 0) buffer Particles {
  Particle particles[];
};

float rand(vec2 st) {
  return (snoise(st) + 1.0) / 2.0;
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
  float first_rand = rand(vec2(random_seed / 3.0, interp));
  particles[gid].ttl = ttl_max - (ttl_max - ttl_min) * first_rand;
  ivec2 pos_delta = ivec2(interp * vec2(end_position - start_position));
  particles[gid].position = start_position + pos_delta;

  float second_rand = rand(vec2(first_rand, interp));
  float speed =
      emit_velocity_min + second_rand * (emit_velocity_max - emit_velocity_min);

  float third_rand = rand(vec2(second_rand, interp));
  float angle = third_rand * M_PI;
  particles[gid].velocity = ivec2(speed * cos(angle), speed * sin(angle));
  particles[gid].padding = first_rand;
}
