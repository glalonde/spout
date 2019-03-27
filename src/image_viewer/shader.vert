#version 430
layout (location = 0) in vec2 position;
layout (location = 1) in vec2 in_texture_coordinate;
out vec2 texture_coordinate;
void main() {
  texture_coordinate = in_texture_coordinate;
  gl_Position = vec4(position, 0.0, 1.0);
}
