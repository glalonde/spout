#version 450

layout(location = 0) in vec2 in_texture_coordinate;
layout(location = 0) out vec4 out_color;
// TODO bind in both terrain buffers 
layout(set = 0, binding = 0) buffer TerrainBufferButtom {
    int[] terrain_buffer_bottom;
};
layout(set = 0, binding = 1) uniform Params {
    uint width;
    uint height;
};

int GetCell() {
  ivec2 cell = ivec2(in_texture_coordinate * vec2(width, height));
  return terrain_buffer_bottom[cell.y * width + cell.x];
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