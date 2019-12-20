#version 450

layout(location = 0) out vec2 v_TexCoord;

const vec2 positions[3] = vec2[3](
    vec2(0.0, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5)
);

const vec2 tex_coord[3] = vec2[3](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0)
);

void main() {
    v_TexCoord = tex_coord[gl_VertexIndex];
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}