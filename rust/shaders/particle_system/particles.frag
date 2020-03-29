#version 450

layout(location = 0) in vec2 in_texture_coordinate;
layout(location = 0) out vec4 out_color;
layout(set = 0, binding = 0) buffer DensityBuffer {
    uint[] density_buffer;
};
layout(set = 0, binding = 1) uniform texture1D in_color_map;
layout(set = 0, binding = 2) uniform sampler in_color_map_sampler;
layout(set = 0, binding = 3) uniform Params {
    uint width;
    uint height;
};


// TODO this should be a uniform input
int max_density_value = 10;

uint GetCell() {
  ivec2 cell = ivec2(in_texture_coordinate * vec2(width, height));
  return density_buffer[cell.y * width + cell.x];
}

// Returns the color map texture coordinate 
float ReadUnsigned() {
  uint count = GetCell();
  //texture(usampler2D(in_density_texture, in_density_sampler), in_texture_coordinate).x;
  if (count <= 0) {
    discard;
  }
  return float(count - uint(0)) / float(max_density_value);
}

void main() {
  out_color = texture(sampler1D(in_color_map, in_color_map_sampler), ReadUnsigned()).rgba;
}