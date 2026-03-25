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
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex_positions[i], 0.0, 1.0);
    out.tex_coord = tex_coords[i];
    return out;
}

@group(0) @binding(0) var hdr_tex: texture_2d<f32>;
@group(0) @binding(1) var hdr_sampler: sampler;
@group(0) @binding(2) var bloom_tex: texture_2d<f32>;
@group(0) @binding(3) var bloom_sampler: sampler;

override bloom_strength: f32 = 1.0;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let game = textureSample(hdr_tex, hdr_sampler, in.tex_coord).rgb;
    let bloom = textureSample(bloom_tex, bloom_sampler, in.tex_coord).rgb;
    let out = clamp(game + bloom * bloom_strength, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(out, 1.0);
}
