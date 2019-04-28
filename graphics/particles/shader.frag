#version 430
layout (binding = 0) uniform usampler2D in_texture;
in vec2 texture_coordinate;
layout (location = 0) out vec4 out_color;
void main() {
  uint count = texture(in_texture, texture_coordinate).x;
  out_color = vec4(float(count) / 11.0, 0.0, 0.0, 1.0);
}
