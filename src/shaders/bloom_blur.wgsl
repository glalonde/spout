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

struct BlurUniforms {
    // One-texel step in UV space: (1/width, 0) for horizontal, (0, 1/height) for vertical.
    direction: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: BlurUniforms;

// Bilinear-optimised 9-tap Gaussian: 5 fetches instead of 9.
//
// A linear sampler blends two adjacent texels in one fetch. By placing each
// sample at a fractional offset, we recover the same weighted sum as integer
// taps at 0, ±1, ±2, ±3, ±4 while halving the non-center fetch count.
//
// Derivation from standard weights w = [0.2270, 0.1946, 0.1216, 0.0541, 0.0162]:
//   combined weight: w[i] + w[i+1]
//   combined offset: (i*w[i] + (i+1)*w[i+1]) / (w[i] + w[i+1])
//
//   tap A (replaces ±1, ±2): weight = 0.1946+0.1216 = 0.3162
//                              offset = (1*0.1946 + 2*0.1216) / 0.3162 = 1.3846
//   tap B (replaces ±3, ±4): weight = 0.0541+0.0162 = 0.0703
//                              offset = (3*0.0541 + 4*0.0162) / 0.0703 = 3.2308
//
// Total weight: 0.2270 + 2*(0.3162 + 0.0703) = 1.0  ✓
var<private> weights: array<f32, 3> = array<f32, 3>(
    0.2270270270,
    0.3162162162,
    0.0702702703,
);
var<private> offsets: array<f32, 3> = array<f32, 3>(
    0.0,
    1.3846153846,
    3.2307692308,
);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var result = textureSample(source, source_sampler, in.tex_coord).rgb * weights[0];
    for (var i: i32 = 1; i < 3; i += 1) {
        let offset = uniforms.direction * offsets[i];
        result += textureSample(source, source_sampler, in.tex_coord + offset).rgb * weights[i];
        result += textureSample(source, source_sampler, in.tex_coord - offset).rgb * weights[i];
    }
    return vec4<f32>(result, 1.0);
}
