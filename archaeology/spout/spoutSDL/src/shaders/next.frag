#version 150 core

uniform sampler2D terrain_texture;
uniform sampler2D particle_texture;
uniform sampler2D particle_palette;
uniform sampler2D terrain_palette;

in vec2 texture_coord;

out vec4 color_out;

void main(void) {
  // Override out_Color with our texture pixel
  vec4 texel = texture(particle_texture, texture_coord);
  if (texel.r > 0.0f) {
    color_out = texture(particle_palette, texel.xy);
  } else {
    texel = texture(terrain_texture, texture_coord);
    color_out = texture(terrain_palette, texel.xy);
  }
}
