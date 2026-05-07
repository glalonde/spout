// Low-resolution title UI overlay.
//
// Samples a game-resolution RGBA texture after the main bloom composite and
// alpha-blends it onto the surface. The texture is integer-scaled with nearest
// sampling so each source pixel covers a uniform display-sized block.

struct Uniforms {
    surface_size: vec2<f32>,
    game_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var overlay_tex: texture_2d<f32>;
@group(0) @binding(2) var overlay_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) screen_px: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    let x = f32(vi & 1u);
    let y = f32(vi >> 1u);
    let ndc = vec2<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0);

    var out: VertexOutput;
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.screen_px = vec2<f32>(x * u.surface_size.x, y * u.surface_size.y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let raw_scale = min(u.surface_size.x / u.game_size.x, u.surface_size.y / u.game_size.y);
    let pixel_scale = max(floor(raw_scale), 1.0);
    let draw_size = u.game_size * pixel_scale;
    let offset = floor((u.surface_size - draw_size) * 0.5);

    let local = (in.screen_px - offset) / draw_size;
    if any(local < vec2<f32>(0.0)) || any(local > vec2<f32>(1.0)) {
        return vec4<f32>(0.0);
    }

    // The main game blit flips the game texture vertically. Use the same
    // convention so UI coordinates match title hit-testing and TextRenderer.
    let uv = vec2<f32>(local.x, 1.0 - local.y);
    return textureSampleLevel(overlay_tex, overlay_sampler, uv, 0.0);
}
