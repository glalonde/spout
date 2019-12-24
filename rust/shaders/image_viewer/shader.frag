#version 450

layout(location = 0) in vec2 v_TexCoord;
layout(location = 0) out vec4 o_Target;
// Loads the texture created by the compute stage
layout(set = 0, binding = 0) uniform utexture2D t_Count;
// Loads the texture that has been brought in from the CPU
layout(set = 0, binding = 1) uniform texture2D t_Color;
layout(set = 0, binding = 2) uniform sampler s_Color;

void main() {
    // To read the unsigned int texture from the GPU:
    o_Target = texture(usampler2D(t_Count, s_Color), v_TexCoord);
    // To read the texture from the CPU:
    // o_Target = texture(sampler2D(t_Color, s_Color), v_TexCoord);
}