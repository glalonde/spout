// Low-resolution UI rectangle renderer.
//
// Draws instanced filled rectangles with a pixel outline into a game-sized
// render target. Coordinates match TextRenderer's y_dir convention.

struct ScreenUniform {
    size: vec2<f32>,
    y_dir: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> screen: ScreenUniform;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) fill_color: vec4<f32>,
    @location(3) outline_color: vec4<f32>,
    @location(4) outline_px: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) fill_color: vec4<f32>,
    @location(3) outline_color: vec4<f32>,
    @location(4) outline_px: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let u_idx = input.vertex_index & 1u;
    let v_idx = input.vertex_index >> 1u;
    let local_pos = vec2<f32>(
        f32(u_idx) * input.size.x,
        f32(v_idx) * input.size.y,
    );
    let pixel_pos = input.pos + local_pos;

    let ndc_x = pixel_pos.x / screen.size.x * 2.0 - 1.0;
    let ndc_y = screen.y_dir * (1.0 - pixel_pos.y / screen.size.y * 2.0);

    var out: VertexOutput;
    out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.local_pos = local_pos;
    out.size = input.size;
    out.fill_color = input.fill_color;
    out.outline_color = input.outline_color;
    out.outline_px = input.outline_px;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let border = input.outline_px > 0.0 && (
        input.local_pos.x < input.outline_px ||
        input.local_pos.y < input.outline_px ||
        input.local_pos.x >= input.size.x - input.outline_px ||
        input.local_pos.y >= input.size.y - input.outline_px
    );

    if border {
        return input.outline_color;
    }
    return input.fill_color;
}
