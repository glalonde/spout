layout(location = 0) uniform int start_index;
layout(location = 1) uniform int num_emitted;
layout(location = 3) uniform float ttl_min;
layout(location = 4) uniform float ttl_max;
layout(location = 5) uniform float random_seed;
layout(location = 6) uniform uvec2 start_position;
layout(location = 7) uniform uvec2 end_position;
layout(location = 8) uniform float emit_velocity_min;
layout(location = 9) uniform float emit_velocity_max;
layout(location = 10) uniform uint random_seed_int;

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

const uint RAND_MOD = 2147483647;

uint rand_int(uint x) {
  uint a = 1664525;
  uint m = RAND_MOD;
  uint c = 1013904223;
  return (x * a + x) % m;

}

float norm_rand(uint r) {
  const float rand_mod_f = float(RAND_MOD);
  return float(r) / rand_mod_f;
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
  uint rand_seed = random_seed_int*gid;
  uint first_rand = rand_int(rand_seed);
  particles[gid].ttl = ttl_max - (ttl_max - ttl_min) * norm_rand(first_rand);
  ivec2 pos_delta = ivec2(interp * vec2(end_position - start_position));
  particles[gid].position = start_position + pos_delta;

  uint second_rand = rand_int(first_rand);
  float speed = emit_velocity_min + norm_rand(second_rand) *
                                        (emit_velocity_max - emit_velocity_min);

  uint third_rand = rand_int(second_rand);
  float angle = norm_rand(third_rand) * M_PI * 2.0 - M_PI;
  particles[gid].velocity = ivec2(vec2(cos(angle), sin(angle)) * speed);
  particles[gid].padding = speed;
}
