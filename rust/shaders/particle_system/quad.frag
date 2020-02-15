#version 450

layout(location = 0) in vec2 in_texture_coordinate;
layout(location = 0) out vec4 out_color;
// Loads the texture created by the compositing stage
layout(set = 0, binding = 0) uniform texture2D in_tex;
layout(set = 0, binding = 1) uniform sampler in_sampler;

// TODO break out the glow into a separate pass that only affects the particles the "lit" materials.
// Glow effect
void main() {
  out_color = texture(sampler2D(in_tex, in_sampler), in_texture_coordinate);
}