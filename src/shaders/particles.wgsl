{% include "grid.wgsl.include" %}
{% include "particle.wgsl.include" %}

struct UniformData {
    dt: f32;
    viewport_width: u32;
    viewport_height: u32;
    viewport_bottom_height: i32;
};
[[group(0), binding(0)]]
var<uniform> uniforms: UniformData;

struct Particles {
    data: [[stride(24)]] array<Particle>;
};
[[group(0), binding(1)]]
var<storage, read_write> particle_buffer: Particles;

struct DensityBuffer {
    data: [[stride(4)]] array<atomic<u32>>;
};
[[group(0), binding(2)]]
var<storage, read_write> density_buffer: DensityBuffer;

fn IncrementCell(cell_in: vec2<i32>) {
  var cell = cell_in;
  cell.y = cell.y - i32(uniforms.viewport_bottom_height);
  if (cell.x < 0 || cell.x >= i32(uniforms.viewport_width) || cell.y < 0 || cell.y >= i32(uniforms.viewport_height)) {
    return;
  }
  let index = cell.y * i32(uniforms.viewport_width) + cell.x;

  // Unfortunately, can't currently use non atomic ops here... 
  // https://github.com/gpuweb/gpuweb/issues/2377
  atomicAdd(&density_buffer.data[index], 1u);
}

[[stage(compute), workgroup_size(256)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    let gid = global_id[0];

    let p = particle_buffer.data[gid];
    let current_cell = p.position; // GetOuterGrid(p.position);
    IncrementCell(vec2<i32>(i32(current_cell.x), i32(current_cell.y)));
}