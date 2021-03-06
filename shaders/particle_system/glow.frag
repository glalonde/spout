#version 450

layout(location = 0) in vec2 in_texture_coordinate;
layout(location = 0) out vec4 out_color;
// Loads the texture created by the compositing stage
layout(set = 0, binding = 0) uniform texture2D in_tex;
layout(set = 0, binding = 1) uniform sampler in_tex_sampler;

// TODO: Define as a gaussian blur and separate into horizontal and vertical passes.
float amount = 1.5;
// Glow effect
void main() {
  ivec2 grid_size = textureSize(sampler2D(in_tex, in_tex_sampler), 0);
  out_color = texture(sampler2D(in_tex, in_tex_sampler), in_texture_coordinate);
  vec4 sum = vec4(0);
  int j;
  int i;

  for( i= -3 ;i < 4; i++) {
    for (j = -3; j < 4; j++) {
      sum += amount*texture(sampler2D(in_tex, in_tex_sampler), in_texture_coordinate + vec2(j, i)/vec2(grid_size)) / (3.f + j*j + i*i);
    }
  }
  if (out_color.r < 0.3) {
    out_color = mix(out_color, sum*sum, 0.12);
  } else {
    if (out_color.r < 0.5) {
      out_color = mix(out_color, sum*sum, 0.09);
    } else {
      out_color = mix(out_color, sum*sum, 0.075);
    }
  }
}