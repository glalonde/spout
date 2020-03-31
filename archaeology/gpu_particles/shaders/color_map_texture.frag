#version 430
layout (binding = 0) uniform sampler1D in_color_map;
layout (binding = 1) uniform usampler2D unsigned_density_texture;
layout (binding = 2) uniform isampler2D signed_density_texture;

layout(location = 0) uniform int min_value;
layout(location = 1) uniform int max_value;
layout(location = 2) uniform bool signed;

in vec2 texture_coordinate;
layout (location = 0) out vec4 out_color;

// This shader maps an unsigned int texture into color fragments using the color
// map
float ReadSigned() {
  int count = texture(signed_density_texture, texture_coordinate).x;
  if (count <= 0) {
    discard;
  }
  return float(count - min_value) / float(max_value);
}

float ReadUnsigned() {
  uint count = texture(unsigned_density_texture, texture_coordinate).x;
  if (count <= 0) {
    discard;
  }
  return float(count - uint(min_value)) / float(max_value);
}

void main() {
  float color_coordinate;
  if (signed) {
    color_coordinate = ReadSigned();
  } else {
    color_coordinate = ReadUnsigned();
  }
  out_color = texture(in_color_map, color_coordinate).rgba;
}
