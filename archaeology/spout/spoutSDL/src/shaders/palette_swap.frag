#version 150 core

uniform sampler2D index_texture;
uniform sampler2D palette_texture;

in vec2 texture_coord;

void main(void) {
  gl_FragColor = texture(palette_texture, texture(index_texture, texture_coord).xy);
}
