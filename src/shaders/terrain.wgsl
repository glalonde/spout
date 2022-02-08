struct VertexOutput {
    @builtin(position) position: vec4<f32>;
    @location(0) tex_coord: vec2<f32>;
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

@stage(vertex)
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let pos2d = vertex_positions[vertex_index];

    var out: VertexOutput;
    out.tex_coord = tex_coord[vertex_index];
    out.position = vec4<f32>(pos2d.x, pos2d.y, 0.0, 1.0); 
    return out;
}

struct Uniforms {
    viewport_width: u32;
    viewport_height: u32;
    height_of_viewport: u32;
    height_of_bottom_buffer: u32;
    height_of_top_buffer: u32;
};
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> terrain_buffer_top: array<i32>;
@group(0) @binding(2)
var<storage, read> terrain_buffer_bottom: array<i32>;

// This could probably be simplified to drawing two textured quads with a color map.
fn get_cell(tex_coord: vec2<f32>) -> i32 {
  let cell = vec2<i32>(tex_coord * vec2<f32>(f32(uniforms.viewport_width), f32(uniforms.viewport_height)));
  let absolute_height = i32(uniforms.height_of_viewport) + cell.y;

  var buffer_height = absolute_height;
  let use_bottom_buffer = (absolute_height < i32(uniforms.height_of_top_buffer));
  if (use_bottom_buffer) {
    let offset = (buffer_height - i32(uniforms.height_of_bottom_buffer)) * i32(uniforms.viewport_width) + cell.x;
    return terrain_buffer_bottom[offset];
  } else {
    let offset = (buffer_height - i32(uniforms.height_of_top_buffer)) * i32(uniforms.viewport_width) + cell.x;
    return terrain_buffer_top[offset];
  }
}

@stage(fragment)
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let val = get_cell(in.tex_coord);
    if (val <= 0) {
        return vec4<f32>(0.196, 0.196, 0.196, 0.0);
    } else {
        return vec4<f32>(0.196, 0.196, 0.196, 1.0);
    }
}