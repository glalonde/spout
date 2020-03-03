#version 450

layout(location = 0) in vec2 in_texture_coordinate;
layout(location = 0) out vec4 out_color;
// TODO bind in both terrain textures
layout(set = 0, binding = 0) uniform utexture2D in_terrain_texture_bottom;
layout(set = 0, binding = 1) uniform sampler in_terrain_sampler;

void main() {
    uint val = texture(usampler2D(in_terrain_texture_bottom, in_terrain_sampler), in_texture_coordinate).x;
    if (val <= 20) {
        discard;
    }
    out_color = vec4(.2, .2, .2, 1.0);
}