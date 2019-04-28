#version 430
layout (binding = 0) uniform usampler2D in_texture;
layout (binding = 1) uniform sampler1D in_color_map;
in vec2 texture_coordinate;
layout (location = 0) out vec4 out_color;
void main() {
  uint count = texture(in_texture, texture_coordinate).x;
  float color_coordinate = float(count) / 11.0;
  out_color = texture(in_color_map, color_coordinate).rgba;
}
