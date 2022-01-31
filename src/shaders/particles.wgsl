{% include "grid.wgsl.include" %}
{% include "particle.wgsl.include" %}

// This shader module handles particle update and aggregation.
// It could possibly be combined with the emitter module... but for now it takes the particle_buffer, updates it, then aggregates particle densities into the density_buffer.

// IN:
struct UniformData {
    dt: f32;
    viewport_width: u32;
    viewport_height: u32;
    viewport_bottom_height: i32;
};
[[group(0), binding(0)]]
var<uniform> uniforms: UniformData;

// IN OUT:
struct Particles {
    data: [[stride(24)]] array<Particle>;
};
[[group(0), binding(1)]]
var<storage, read_write> particle_buffer: Particles;

// IN OUT:
struct TerrainBuffer {
    data: [[stride(4)]] array<atomic<i32>>;
};
[[group(0), binding(2)]]
var<storage, read_write> terrain_buffer_top: TerrainBuffer;
[[group(0), binding(3)]]
var<storage, read_write> terrain_buffer_bottom: TerrainBuffer;

// OUTPUT:
struct DensityBuffer {
    data: [[stride(4)]] array<atomic<u32>>;
};
[[group(0), binding(4)]]
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

fn on_level_buffers(cell: vec2<i32>) -> bool {
  return cell.x >= 0 && cell.x < i32(uniforms.viewport_width) && cell.y >= 0 && cell.y < i32(uniforms.viewport_height);
}

let PI: f32 = 3.14159265358979323846;

[[stage(compute), workgroup_size(256)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>,  [[builtin(num_workgroups)]] num_workgroups: vec3<u32>) {
  let total_particles = num_workgroups[0] * 256u;
  let gid = global_id[0];

  let particle = &(particle_buffer.data[gid]);
  if ((*particle).ttl <= 0.0) {
   return;
  } 

  // TODO collisions 
  let delta_pos = (*particle).velocity * uniforms.dt;
  (*particle).position = (*particle).position + delta_pos;
  (*particle).ttl = (*particle).ttl - uniforms.dt;

  // let current_cell = center + vec2<i32>(radius * vec2<f32>(cos(p * 2.0 * PI), sin(p * 2.0 * PI)));
  let current_cell = vec2<i32>((*particle).position);

  if (!on_level_buffers(current_cell)) {
    (*particle).ttl = 0.0;
    return;
  } 

  // Draw every particle.
  IncrementCell(current_cell);
}