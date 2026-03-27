// Text rendering shader: instanced glyph quads with texture atlas lookup.
// Each instance is a positioned, sized, UV-mapped, colored quad.
//
// Supports both Y-down (screen/overlay) and Y-up (game view) coordinate
// systems via screen.y_dir: +1.0 = Y-down, -1.0 = Y-up.

struct ScreenUniform {
    size: vec2<f32>,
    // +1.0 = Y-down (screen space: origin top-left)
    // -1.0 = Y-up   (game view: origin bottom-left)
    y_dir: f32,
};

@group(1) @binding(0) var<uniform> screen: ScreenUniform;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) pos: vec2<f32>,       // pixel position (top-left or bottom-left depending on y_dir)
    @location(1) size: vec2<f32>,      // width, height in pixels
    @location(2) uv: vec4<f32>,        // u_min, v_min, u_max, v_max
    @location(3) color: vec4<f32>,     // RGBA tint
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    // Triangle strip: 0=TL, 1=TR, 2=BL, 3=BR (in screen space)
    let u_idx = input.vertex_index & 1u;  // 0, 1, 0, 1
    let v_idx = input.vertex_index >> 1u; // 0, 0, 1, 1

    let pixel_pos = input.pos + vec2<f32>(f32(u_idx) * input.size.x, f32(v_idx) * input.size.y);

    // Pixel coordinates → NDC.
    // x: [0, width] → [-1, 1] (always left-to-right)
    let ndc_x = pixel_pos.x / screen.size.x * 2.0 - 1.0;

    // y_dir = +1: Y-down — pixel 0 → NDC +1, pixel height → NDC -1
    // y_dir = -1: Y-up   — pixel 0 → NDC -1, pixel height → NDC +1
    let ndc_y = screen.y_dir * (1.0 - pixel_pos.y / screen.size.y * 2.0);

    let uv = vec2<f32>(
        mix(input.uv.x, input.uv.z, f32(u_idx)),
        mix(input.uv.y, input.uv.w, f32(v_idx)),
    );

    var out: VertexOutput;
    out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.tex_coord = uv;
    out.color = input.color;
    return out;
}

@group(0) @binding(0) var atlas_tex: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSampleLevel(atlas_tex, atlas_sampler, input.tex_coord, 0.0);
    return vec4<f32>(input.color.rgb * texel.rgb, input.color.a * texel.a);
}
