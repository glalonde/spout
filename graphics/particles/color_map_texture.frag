#version 430
layout (binding = 0) uniform sampler1D in_color_map;
layout (binding = 1) uniform usampler2D density_texture;

layout(location = 0) uniform int min_value;
layout(location = 1) uniform int max_value;

in vec2 texture_coordinate;
layout (location = 0) out vec4 out_color;

// This shader maps an unsigned int texture into color fragments using the color
// map

void main() {
  uint count = texture(density_texture, texture_coordinate).x;
  if (count <= 0) {
    discard;
  }
  float color_coordinate = float((int(count) - min_value)) / float(max_value);
  out_color = texture(in_color_map, color_coordinate).rgba;
}
