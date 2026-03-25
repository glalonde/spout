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
    // Step in UV space per tap. Set to (1/width, 0) for horizontal, (0, 1/height) for vertical.
    // Multiply by step_scale for a wider blur radius.
    direction: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: BlurUniforms;

// 9-tap Gaussian weights (taps at offsets 0..4, normalized so sum = 1).
var<private> weights: array<f32, 5> = array<f32, 5>(
    0.2270270270,
    0.1945945946,
    0.1216216216,
    0.0540540541,
    0.0162162162,
);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var result = textureSample(source, source_sampler, in.tex_coord).rgb * weights[0];
    for (var i: i32 = 1; i < 5; i += 1) {
        let offset = uniforms.direction * f32(i);
        result += textureSample(source, source_sampler, in.tex_coord + offset).rgb * weights[i];
        result += textureSample(source, source_sampler, in.tex_coord - offset).rgb * weights[i];
    }
    return vec4<f32>(result, 1.0);
}
