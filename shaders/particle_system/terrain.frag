#version 450

layout(location = 0) in vec2 in_texture_coordinate;
layout(location = 0) out vec4 out_color;
layout(set = 0, binding = 0) buffer TerrainBufferButtom {
    int[] terrain_buffer_bottom;
};
layout(set = 0, binding = 1) buffer TerrainBufferTop {
    int[] terrain_buffer_top;
};
layout(set = 0, binding = 2) uniform Params {
    uint viewport_width;
    uint viewport_height;
    uint height_of_viewport;
    uint height_of_bottom_buffer;
    uint height_of_top_buffer;
};

int GetCell() {
  ivec2 cell = ivec2(in_texture_coordinate * vec2(viewport_width, viewport_height));
  uint absolute_height = height_of_viewport + cell.y;
  uint buffer_height = height_of_viewport + cell.y;
  bool use_bottom_buffer = (absolute_height < height_of_top_buffer);
  buffer_height -= (use_bottom_buffer) ? height_of_bottom_buffer : height_of_top_buffer;
  uint offset = buffer_height * viewport_width + cell.x;
  return (use_bottom_buffer) ? terrain_buffer_bottom[offset] : terrain_buffer_top[offset];
}

void main() {
    int val = GetCell();
    // texture(isampler2D(in_terrain_texture_bottom, in_terrain_sampler), in_texture_coordinate).x;
    if (val <= 0) {
        discard;
    } else {
        out_color = vec4(.196, .196, .196, 1.0);
    }
}