#version 430
#extension GL_ARB_compute_variable_group_size : enable
layout (binding = 0, r32ui) uniform uimage2D counter_texture;

struct Particle {
  vec2 position;
};

layout(std430, binding = 0) readonly buffer Particles {
  Particle particles[];
};

layout(local_size_variable) in;
void main() {
  uint gid = gl_GlobalInvocationID.x;
  ivec2 image_coord = ivec2(particles[gid].position);

  /*
  // Bootleg atomic add to test if CompSwap is slower. Might be getting simplified by the compiler.
  uint val;
  uint out_val;
  do {
    val = imageLoad(counter_texture, image_coord).x;
    out_val = imageAtomicCompSwap(counter_texture, image_coord, val, val + 1);
  } while (val != out_val);
  */


  imageAtomicAdd(counter_texture, image_coord, 1);
}
