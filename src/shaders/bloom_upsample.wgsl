// Dual-filter bloom: upsample pass.
//
// 9-tap "tent" filter (3x3 kernel with weights 1,2,1 / 2,4,2 / 1,2,1, normalised
// to 1/16). Reads from a smaller mip and outputs the filtered colour. The
// pipeline is configured with additive blending so the result is added on top
// of whatever is already in the destination mip — on a TBDR GPU the destination
// read for the blend stays in tile memory and never hits main memory.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

var<private> vertex_positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>( 1.0,  1.0),
);
var<private> tex_coords: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, 0.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex_positions[vertex_index], 0.0, 1.0);
    out.tex_coord = tex_coords[vertex_index];
    return out;
}

struct Uniforms {
    // One source-texel step in UV space (smaller mip): (1/src_width, 1/src_height).
    src_texel: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> u: Uniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.tex_coord;
    let s = u.src_texel;

    // Tent kernel weights:  1 2 1
    //                       2 4 2  / 16
    //                       1 2 1
    var sum = vec3<f32>(0.0);
    sum += textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-1.0, -1.0)).rgb;
    sum += textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 0.0, -1.0)).rgb * 2.0;
    sum += textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 1.0, -1.0)).rgb;

    sum += textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-1.0,  0.0)).rgb * 2.0;
    sum += textureSample(src_tex, src_sampler, uv).rgb * 4.0;
    sum += textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 1.0,  0.0)).rgb * 2.0;

    sum += textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-1.0,  1.0)).rgb;
    sum += textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 0.0,  1.0)).rgb * 2.0;
    sum += textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 1.0,  1.0)).rgb;

    return vec4<f32>(sum * (1.0 / 16.0), 1.0);
}
