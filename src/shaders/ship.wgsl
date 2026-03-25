struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

// Two triangles forming a chevron with a concave tail notch:
//   nose(12,0) → left-wing(-8,9) → notch(-5,0)
//   nose(12,0) → notch(-5,0)     → right-wing(-8,-9)
var<private> ship_vertices: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>( 12.0,  0.0),   // nose
    vec2<f32>( -8.0,  9.0),   // left wing
    vec2<f32>( -5.0,  0.0),   // tail notch
    vec2<f32>( 12.0,  0.0),   // nose (repeated)
    vec2<f32>( -5.0,  0.0),   // tail notch (repeated)
    vec2<f32>( -8.0, -9.0),   // right wing
);


struct Uniforms {
    position: vec2<f32>,
    orientation: f32,

    viewport_width: u32,
    viewport_height: u32,
    viewport_offset: i32,
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

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let rot = rotate2d(uniforms.orientation);
    let world_pos = rot * ship_vertices[vertex_index] + uniforms.position;
    let viewport_pos = world_to_ndc(world_pos, f32(uniforms.viewport_width), f32(uniforms.viewport_height), f32(uniforms.viewport_offset));
    var out: VertexOutput;
    out.position = vec4<f32>(viewport_pos.x, viewport_pos.y, 0.0, 1.0); 
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Dark steel-blue — well below the bloom threshold so it doesn't over-glow.
    return vec4<f32>(0.25, 0.42, 0.75, 0.9);
}