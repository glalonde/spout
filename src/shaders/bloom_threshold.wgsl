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

@group(0) @binding(0) var game_texture: texture_2d<f32>;
@group(0) @binding(1) var game_sampler: sampler;

// Pixels brighter than this contribute to bloom. Set via pipeline override constant.
override bloom_threshold: f32 = 0.6;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(game_texture, game_sampler, in.tex_coord).rgb;
    // Soft-knee: extract bright areas smoothly, avoid hard cutoff.
    let brightness = max(color.r, max(color.g, color.b));
    let contribution = max(brightness - bloom_threshold, 0.0) / max(brightness, 0.001);
    return vec4<f32>(color * contribution, 1.0);
}
