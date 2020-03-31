#version 150 core
in vec4 pass_position;
in vec2 pass_texture_coord;
out vec2 texture_coord;
void main() {
  gl_Position = pass_position;
  texture_coord = pass_texture_coord;
  //texture_coord = pass_position.xy;
}
