#version 450

layout(location = 0) in vec2 in_texture_coordinate;
layout(location = 0) out vec4 out_color;
// TODO bind in both terrain textures
layout(set = 0, binding = 0) uniform itexture2D in_terrain_texture_bottom;
layout(set = 0, binding = 1) uniform sampler in_terrain_sampler;

void main() {
    int val = texture(isampler2D(in_terrain_texture_bottom, in_terrain_sampler), in_texture_coordinate).x;
    if (val <= 0) {
        discard;
    } else {
        out_color = vec4(.196, .196, .196, 1.0);
    }
}