struct VertexOutput {
    @builtin(position) position: vec4<f32>;
    @location(0) tex_coord: vec2<f32>;
};

struct ViewData {
    projection: mat4x4<f32>;
    view: mat4x4<f32>;
};
@group(0) @binding(0)
var<uniform> view_data: ViewData;

struct ModelData {
    pose: mat4x4<f32>;
};
@group(1) @binding(2)
var<uniform> model_data: ModelData;

@stage(vertex)
fn vs_main(@location(0) pos: vec4<f32>, @location(1) tex_coord: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = tex_coord;
    // pixel_pose_camera * camera_pose_world * world_pose_model * model_pose_vertex
    out.position = view_data.projection * view_data.view * model_data.pose * pos;
    return out;
}

@group(1) @binding(0)
var r_color: texture_2d<f32>;
@group(1) @binding(1)
var r_sampler: sampler;


@stage(fragment)
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(r_color, r_sampler, in.tex_coord);
}