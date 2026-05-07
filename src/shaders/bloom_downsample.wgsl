// Dual-filter bloom: downsample pass.
//
// 13-tap weighted box filter (Jimenez "Next-gen Post-Processing in Call of Duty:
// Advanced Warfare"). Reads from one mip and writes to the next-smaller mip.

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
    // One source-texel step in UV space: (1/src_width, 1/src_height).
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

    let a = textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-2.0, -2.0)).rgb;
    let b = textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 0.0, -2.0)).rgb;
    let c = textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 2.0, -2.0)).rgb;

    let d = textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-1.0, -1.0)).rgb;
    let e = textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 1.0, -1.0)).rgb;

    let f = textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-2.0,  0.0)).rgb;
    let g = textureSample(src_tex, src_sampler, uv).rgb;
    let h = textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 2.0,  0.0)).rgb;

    let i = textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-1.0,  1.0)).rgb;
    let j = textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 1.0,  1.0)).rgb;

    let k = textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-2.0,  2.0)).rgb;
    let l = textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 0.0,  2.0)).rgb;
    let m = textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 2.0,  2.0)).rgb;

    let inner = (d + e + i + j) * 0.125;
    let edges = (b + f + h + l) * 0.0625;
    let corners = (a + c + k + m) * 0.03125;
    let result = inner + edges + corners + g * 0.125;

    return vec4<f32>(result, 1.0);
}
