#version 430
layout (binding = 0) uniform sampler1D in_color_map;
layout (binding = 1) uniform usampler2D particle_density_texture;
layout (binding = 2) uniform usampler2D terrain_texture;
in vec2 texture_coordinate;
layout (location = 0) out vec4 out_color;
void main() {
  float max_density = 100;
  uint count = texture(particle_density_texture, texture_coordinate).x;
  uint terrain_value = texture(terrain_texture, texture_coordinate).x;
  float color_coordinate = float(count) / max_density;
  vec4 particle_color = texture(in_color_map, color_coordinate).rgba;
  vec4 terrain_color =
      (terrain_value > 0) ? vec4(0.0, 0.0, 1.0, 1.0) : vec4(0.0, 0.0, 1.0, 0.0);
  if (count <= 0) {
    // TODO better blending
    if (terrain_value > 0) {
      out_color = vec4(0.0, 0.0, 1.0, 1.0);
    } else {
      out_color.a = 0;
    }
  }
}
