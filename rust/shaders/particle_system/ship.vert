#version 450 core
#include "grid.h"

layout(std140, set = 0, binding = 0) uniform Params {
    uvec2 position;
    float angle;
    float time;
};

const float ship_width = 2.0;
const float ship_height = 4.0;
// Pixel coordinate ship...
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
    vec2(-ship_height/2.0, 0.0),
    vec2(-ship_height, -ship_width/2.0)
);

const vec2 scaled_ship_vertices[4] = vec2[4](
    ship_vertices[0] * kInnerGridSize,
    ship_vertices[1] * kInnerGridSize,
    ship_vertices[2] * kInnerGridSize,
    ship_vertices[3] * kInnerGridSize
);

mat2 rotate2d(float angle){
    return mat2(cos(angle),-sin(angle),
                sin(angle),cos(angle));
}

void main() {
    mat2 rotation = rotate2d(angle);
    vec2 vertex_position = rotation * scaled_ship_vertices[gl_VertexIndex] + position;
    gl_Position = vec4(vertex_position, 0.0, 1.0);
}