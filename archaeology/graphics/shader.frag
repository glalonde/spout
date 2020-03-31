#version 430
uniform sampler2D in_texture;
in vec2 texture_coordinate;
out vec4 out_color;
void main() {
  out_color = texture(in_texture, texture_coordinate);
}
