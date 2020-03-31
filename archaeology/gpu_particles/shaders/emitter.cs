layout(location = 0) uniform int start_index;
layout(location = 1) uniform int num_emitted;
layout(location = 3) uniform float ttl_min;
layout(location = 4) uniform float ttl_max;
layout(location = 5) uniform float random_seed;
layout(location = 6) uniform uvec2 start_position;
layout(location = 7) uniform uvec2 end_position;
layout(location = 8) uniform float emit_velocity_min;
layout(location = 9) uniform float emit_velocity_max;
layout(location = 10) uniform float dt;
layout(location = 11) uniform uint random_seed_int;

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

const uint RAND_MOD = 2147483647;

float rand(vec2 st) {
  return (snoise(st) + 1.0) / 2.0;
}

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
  float first_rand = rand(vec2(random_seed / 4.0, interp * .8 + .2));
  particles[gid].ttl = ttl_max - (ttl_max - ttl_min) * first_rand;


  // Compute velocity
  float second_rand = rand(vec2(first_rand, interp * .8 + .1));
  float speed =
      emit_velocity_min + second_rand * (emit_velocity_max - emit_velocity_min);

  float third_rand = rand(vec2(second_rand, interp * .8 + .2));
  float angle = third_rand - .5;
  vec2 float_velocity = vec2(cos(angle), sin(angle)) * speed;
  particles[gid].velocity = ivec2(float_velocity);

  // Emit randomly somewhere in the vector of the velocity
  // vec2 pos_rand = float_velocity * dt * norm_rand(random_seed_int * gid);
  vec2 pos_rand = vec2(0, 0);
  ivec2 pos_delta =
      ivec2(interp * vec2(end_position - start_position) + pos_rand);
  particles[gid].position = start_position + pos_delta;
  particles[gid].padding = dt;
}
