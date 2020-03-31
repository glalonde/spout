#version 150 core
uniform sampler2D diffuse_texture;
uniform vec2 resolution;
in vec2 texture_coord;
out vec4 color_out;

int sample_range = 4;
float amount = 1.0;
// Glow effect
void main(void) {

  color_out = texture2D(diffuse_texture, texture_coord);
  vec4 sum = vec4(0);
  int j;
  int i;

  for( i= -3 ;i < 4; i++) {
    for (j = -3; j < 4; j++) {
      sum += amount*texture2D(diffuse_texture, texture_coord + vec2(j, i)/resolution) / (3.f + j*j + i*i);
    }
  }
  if (color_out.r < 0.3) {
    color_out = mix(color_out, sum*sum, 0.12);
  } else {
    if (color_out.r < 0.5) {
      color_out = mix(color_out, sum*sum, 0.09);
    } else {
      color_out = mix(color_out, sum*sum, 0.075);
    }
  }
}
