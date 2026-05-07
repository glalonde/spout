// Dual-filter bloom: prefilter pass.
//
// Reads the HDR game view at 13 weighted taps (Jimenez "Next-gen Post-Processing
// in Call of Duty: Advanced Warfare", Siggraph 2014), applies a soft-knee bloom
// threshold to each tap, and writes the weighted sum to mip 0 of the bloom
// pyramid (which is half the surface resolution).
//
// Combining threshold + first downsample into one pass saves a fullscreen
// read+write vs. doing them separately.

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

override bloom_threshold: f32 = 0.6;

// Soft-knee threshold: smooth attenuation rather than a hard cutoff.
fn prefilter(c: vec3<f32>) -> vec3<f32> {
    let brightness = max(c.r, max(c.g, c.b));
    let contribution = max(brightness - bloom_threshold, 0.0) / max(brightness, 0.001);
    return c * contribution;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.tex_coord;
    let s = u.src_texel;

    // 13-tap pattern (Jimenez COD AW). Output = 0.5 * inner_box_avg +
    // 0.5 * (avg of 4 outer 2x2 boxes that share the center). Expanded to
    // per-tap weights:
    //   inner diamond D,E,I,J:  0.125 each (sum 0.5)
    //   center G:               0.125
    //   edges B,F,H,L:          0.0625 each (sum 0.25)
    //   corners A,C,K,M:        0.03125 each (sum 0.125)
    let a = prefilter(textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-2.0, -2.0)).rgb);
    let b = prefilter(textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 0.0, -2.0)).rgb);
    let c = prefilter(textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 2.0, -2.0)).rgb);

    let d = prefilter(textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-1.0, -1.0)).rgb);
    let e = prefilter(textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 1.0, -1.0)).rgb);

    let f = prefilter(textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-2.0,  0.0)).rgb);
    let g = prefilter(textureSample(src_tex, src_sampler, uv).rgb);
    let h = prefilter(textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 2.0,  0.0)).rgb);

    let i = prefilter(textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-1.0,  1.0)).rgb);
    let j = prefilter(textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 1.0,  1.0)).rgb);

    let k = prefilter(textureSample(src_tex, src_sampler, uv + s * vec2<f32>(-2.0,  2.0)).rgb);
    let l = prefilter(textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 0.0,  2.0)).rgb);
    let m = prefilter(textureSample(src_tex, src_sampler, uv + s * vec2<f32>( 2.0,  2.0)).rgb);

    let inner = (d + e + i + j) * 0.125;
    let edges = (b + f + h + l) * 0.0625;
    let corners = (a + c + k + m) * 0.03125;
    let result = inner + edges + corners + g * 0.125;

    return vec4<f32>(result, 1.0);
}
