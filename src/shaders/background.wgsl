// Tiled scrolling background.
//
// Draws a repeating tile that scrolls vertically with the viewport.
// The sampler uses AddressMode::Repeat, so we just compute UV coordinates
// that wrap naturally.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

var<private> positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>( 1.0,  1.0),
);

var<private> uvs: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, 0.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(positions[vi], 0.0, 1.0);
    out.tex_coord = uvs[vi];
    return out;
}

struct Uniforms {
    viewport_width: f32,
    viewport_height: f32,
    viewport_offset: f32,
    tile_size: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var tile_texture: texture_2d<f32>;
@group(0) @binding(2) var tile_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Screen pixel position.
    let px = in.tex_coord.x * uniforms.viewport_width;
    let py = in.tex_coord.y * uniforms.viewport_height;

    // World Y (scrolls with viewport).
    let world_y = uniforms.viewport_offset + py;

    // Tile UV — the Repeat sampler handles wrapping automatically.
    let tile_u = px / uniforms.tile_size;
    let tile_v = world_y / uniforms.tile_size;

    return textureSampleLevel(tile_texture, tile_sampler, vec2<f32>(tile_u, tile_v), 0.0);
}
