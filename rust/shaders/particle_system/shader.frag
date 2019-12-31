#version 450

layout(location = 0) in vec2 in_texture_coordinate;
layout(location = 0) out vec4 out_color;
// Loads the texture created by the compute stage
layout(set = 0, binding = 0) uniform utexture2D in_density_texture;
// Loads the texture that has been brought in from the CPU
layout(set = 0, binding = 1) uniform sampler in_density_sampler;
layout(set = 0, binding = 2) uniform utexture1D in_color_map;
layout(set = 0, binding = 3) uniform sampler in_color_map_sampler;

// Returns the color map texture coordinate 
float ReadUnsigned() {
  uint count = texture(usampler2D(in_density_texture, in_density_sampler), in_texture_coordinate).x;
  if (count <= 0) {
    discard;
  }
  return float(count - uint(0)) / float(255);
}

void main() {
    out_color =  vec4(ReadUnsigned(), 0.0, 0.0, 1.0);// texture(usampler1D(in_color_map, in_color_map_sampler), .5).rgba;
}