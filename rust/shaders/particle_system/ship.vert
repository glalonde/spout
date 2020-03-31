#version 450 core
#include "grid.h"

out gl_PerVertex {
    vec4 gl_Position;
};

layout(std140, set = 0, binding = 0) uniform Params {
    int width;
    int height;
    uvec2 position;
    float angle;
};

const float ship_width = 10.0;
const float ship_height = 10.0;
// Pixel coordinate ship...
// TODO this probably needs to render to a framebuffer/texture/image of the same size as the main particle density buffer.
//
//     +x
//
//      1
//     /|\
//    / | \
//   /  2  \ 
//  /_ / \ _\
// 0         3
//
//+y           -y
const vec2 ship_vertices[4] = vec2[4](
    vec2(-ship_height, ship_width/2.0),
    vec2(0.0, 0.0),
    vec2(-ship_height, 0.0),
    vec2(-ship_height, -ship_width/2.0)
);

const vec2 scaled_ship_vertices[4] = vec2[4](
    ship_vertices[0] * kInnerGridSize,
    ship_vertices[1] * kInnerGridSize,
    ship_vertices[2] * kInnerGridSize,
    ship_vertices[3] * kInnerGridSize
);

const vec2 others[4] = vec2[4](
    vec2(-1.0, -1.0),
    vec2(-1.0, -.8),
    vec2(-.8, -1.0),
    vec2(-.8, -.8)
);

mat2 rotate2d(float angle){
    return mat2(/*first column=*/vec2(cos(angle),sin(angle)),
                /*second column=*/vec2(-sin(angle),cos(angle)));
}

void main() {
    mat2 rotation = rotate2d(angle);
    ivec2 vertex_position = ivec2(rotation * scaled_ship_vertices[gl_VertexIndex]);
    // Grid coordinates...
    ivec2 current_cell = vertex_position + ivec2(uvec2(position - kGridAnchor * kInnerGridSize));

    // Convert to NDC
    vec2 image_coordinates = (current_cell) / (vec2(width/2, height/2) * kInnerGridSize) + vec2(-1.0, -1.0);
    gl_Position = vec4(image_coordinates, 0.0, 1.0);
}