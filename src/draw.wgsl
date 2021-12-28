struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] tex_coord: vec2<f32>;
};

struct Locals {
    projection: mat4x4<f32>;
    view: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> r_data: Locals;

[[stage(vertex)]]
fn vs_main([[location(0)]] pos: vec4<f32>, [[location(1)]] tex_coord: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = tex_coord;
    // pixel_pose_camera * camera_pose_model 
    out.position = r_data.projection * r_data.view * pos;
    return out;
}

[[group(0), binding(1)]]
var r_color: texture_2d<f32>;
[[group(0), binding(2)]]
var r_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(r_color, r_sampler, in.tex_coord);
}