#version 450

layout(location = 0) in vec2 in_texture_coordinate;
layout(location = 0) out vec4 out_color;
// Loads the texture created by the compositing stage
layout(set = 0, binding = 0) uniform texture2D in_color_map;
layout(set = 0, binding = 1) uniform sampler in_color_map_sampler;

void main() {
    out_color = texture(sampler2D(in_color_map, in_color_map_sampler), in_texture_coordinate);
}