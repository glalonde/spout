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
fn vs_main(@builtin(vertex_index) vertex_index: u32,) -> VertexOutput {
    let pos2d = vertex_positions[vertex_index];

    var out: VertexOutput;
    out.tex_coord = tex_coord[vertex_index];
    out.position = vec4<f32>(pos2d.x, pos2d.y, 0.0, 1.0); 
    return out;
}

struct ViewData {
    width: u32,
    height: u32,
    // Density-to-color mapping: count is scaled by 1/density_scale before sigmoid.
    density_scale: f32,
    // Sigmoid exponent: >1 = steeper curve, <1 = gentler.
    density_exponent: f32,
};
@group(0) @binding(0)
var<uniform> view_data: ViewData;

@group(0) @binding(1)
var<storage, read> density_buffer: array<u32>;

@group(0) @binding(2)
var color_map: texture_2d<f32>;

@group(0) @binding(3)
var color_map_sampler: sampler;

fn get_cell(tex_coord: vec2<f32>) -> u32 {
    let cell_f: vec2<f32> = tex_coord * vec2<f32>(f32(view_data.width), f32(view_data.height));
    return density_buffer[i32(cell_f.y) * i32(view_data.width) + i32(cell_f.x)];
}

// Returns the effective density (heat-weighted particle count).
// The compute shader writes heat * 256 per particle, so we divide back out.
fn read_density(tex_coord: vec2<f32>) -> f32 {
  let raw = get_cell(tex_coord);
  return f32(raw) / 256.0;
}

// Sigmoid to give an asymptotic approach to the maximum color.
// exponent > 1 = steeper transition, < 1 = gentler.
fn sigmoid(x: f32, exponent: f32) -> f32 {
  let xe = pow(x, exponent);
  return xe / sqrt(1.0 + xe * xe);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let count = read_density(in.tex_coord);
  let rescaled = sigmoid(count / view_data.density_scale, view_data.density_exponent);
  let sample = textureSample(color_map, color_map_sampler, vec2<f32>(rescaled, 0.0));
  return vec4<f32>(sample.xyz, rescaled);
}