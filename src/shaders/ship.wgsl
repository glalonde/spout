struct VertexOutput {
    @builtin(position) position: vec4<f32>;
};

var<private> ship_vertices: array<vec2<f32>, 4> = array<vec2<f32>, 4>(vec2<f32>(-10.0, 5.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(-10.0, 0.0),
    vec2<f32>(-10.0, -5.0));


struct Uniforms {
    position: vec2<f32>;
    orientation: f32;

    viewport_width: u32;
    viewport_height: u32;
    viewport_offset: i32;
};
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn world_to_ndc(world_coord: vec2<f32>, viewport_width: f32, viewport_height: f32, viewport_offset: f32) -> vec2<f32> {
  return 2.0 * vec2<f32>(world_coord.x, viewport_height - (world_coord.y - viewport_offset)) / vec2<f32>(viewport_width, viewport_height) - vec2<f32>(1.0, 1.0);
}

fn rotate2d(orientation: f32) -> mat2x2<f32> {
    return mat2x2<f32>(vec2<f32>(cos(orientation),sin(orientation)),
                       vec2<f32>(-sin(orientation),cos(orientation)));
}

@stage(vertex)
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let rot = rotate2d(uniforms.orientation);
    let world_pos = rot * ship_vertices[vertex_index] + uniforms.position;
    let viewport_pos = world_to_ndc(world_pos, f32(uniforms.viewport_width), f32(uniforms.viewport_height), f32(uniforms.viewport_offset));
    var out: VertexOutput;
    out.position = vec4<f32>(viewport_pos.x, viewport_pos.y, 0.0, 1.0); 
    return out;
}

@stage(fragment)
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}