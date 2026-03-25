struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

// This maps a texture onto a quad such that the 0,0 cell of the texture is at the top left corner of the quad, and when rendered, will end up back at 0,0 in the output buffer.
// So this maintains the texture coordinates.
//  1--3
//  |\ |
//  | \|
//  0--2
var<private> vertex_positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(vec2<f32>(-1.0, -1.0),
                                             vec2<f32>(-1.0, 1.0),
                                             vec2<f32>(1.0, -1.0),
                                             vec2<f32>(1.0, 1.0));

var<private> tex_coord: array<vec2<f32>, 4> = array<vec2<f32>, 4>(vec2<f32>(0.0, 1.0),
                                      vec2<f32>(0.0, 0.0),
                                      vec2<f32>(1.0, 1.0),
                                      vec2<f32>(1.0, 0.0));

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let pos2d = vertex_positions[vertex_index];

    var out: VertexOutput;
    out.tex_coord = tex_coord[vertex_index];
    out.position = vec4<f32>(pos2d.x, pos2d.y, 0.0, 1.0);
    return out;
}

struct Uniforms {
    viewport_width: u32,
    viewport_height: u32,
    viewport_offset: u32,
    terrain_buffer_offset: u32,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(0) @binding(1) var<storage, read> terrain_buffer: array<i32>;

// Reads the terrain cell at integer grid position (tx, ty).
// Returns 0 (empty) for out-of-bounds coordinates.
fn get_cell_xy(tx: i32, ty: i32) -> i32 {
    if tx < 0 || tx >= i32(uniforms.viewport_width) {
        return 0;
    }
    let absolute_height = i32(uniforms.viewport_offset) + ty;
    let row_in_terrain_buffer = absolute_height - i32(uniforms.terrain_buffer_offset);
    let cells_per_row = i32(uniforms.viewport_width);
    let cell_offset = row_in_terrain_buffer * cells_per_row + tx;
    if cell_offset < 0 || u32(cell_offset) >= arrayLength(&terrain_buffer) {
        return 0;
    }
    return terrain_buffer[cell_offset];
}

fn get_cell(tc: vec2<f32>) -> i32 {
    let cell = vec2<i32>(tc * vec2<f32>(f32(uniforms.viewport_width), f32(uniforms.viewport_height)));
    return get_cell_xy(cell.x, cell.y);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let max_val = 10000;
    let cell = vec2<i32>(in.tex_coord * vec2<f32>(f32(uniforms.viewport_width), f32(uniforms.viewport_height)));
    let val = get_cell_xy(cell.x, cell.y);

    if val <= 0 {
        return vec4<f32>(0.1, 0.1, 0.1, 0.0);
    }

    // Edge detection: any 4-connected neighbor that is empty makes this an edge cell.
    let n = get_cell_xy(cell.x,     cell.y - 1);
    let s = get_cell_xy(cell.x,     cell.y + 1);
    let e = get_cell_xy(cell.x + 1, cell.y);
    let w = get_cell_xy(cell.x - 1, cell.y);

    if n <= 0 || s <= 0 || e <= 0 || w <= 0 {
        // HDR amber glow — values > 1.0 drive the bloom pass.
        return vec4<f32>(2.0, 0.7, 0.05, 1.0);
    }

    let p = f32(val) / f32(max_val);
    return vec4<f32>(0.196, 0.196, 0.196, p);
}
