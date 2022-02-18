{% include "grid.wgsl.include" %}
{% include "particle.wgsl.include" %}

// This shader module handles particle update and aggregation.
// It could possibly be combined with the emitter module... but for now it takes the particle_buffer, updates it, then aggregates particle densities into the density_buffer.

// IN:
struct UniformData {
    dt: f32;
    viewport_width: u32;
    viewport_height: u32;
    viewport_offset: i32;

    // level_width and viewport_width should be the same.
    level_width: u32;
    level_height: u32;

    terrain_buffer_offset: i32;
    terrain_buffer_height: u32;

    damage_rate: f32;
    gravity: f32;
    elasticity: f32;
};
@group(0) @binding(0)
var<uniform> uniforms: UniformData;

// IN OUT:
@group(0) @binding(1)
var<storage, read_write> particle_buffer: array<Particle>;

// IN OUT:
@group(0) @binding(2)
var<storage, read_write> terrain_buffer: array<atomic<i32>>;

// OUTPUT:
@group(0) @binding(3)
var<storage, read_write> density_buffer: array<atomic<u32>>;

// Takes cell in the global frame.
fn increment_cell(global_cell: vec2<i32>) {
  var cell = global_cell;
  cell.y = cell.y - i32(uniforms.viewport_offset);
  if (cell.x < 0 || cell.x >= i32(uniforms.viewport_width) || cell.y < 0 || cell.y >= i32(uniforms.viewport_height)) {
    return;
  }
  let index = cell.y * i32(uniforms.viewport_width) + cell.x;

  // Unfortunately, can't currently use non atomic ops here... 
  // https://github.com/gpuweb/gpuweb/issues/2377
  atomicAdd(&density_buffer[index], 1u);
}

fn to_terrain_buffer(cell: vec2<i32>) -> vec2<i32> {
  return vec2<i32>(cell.x, cell.y - uniforms.terrain_buffer_offset);
}

fn on_terrain_buffer(terrain_cell: vec2<i32>) -> bool {
  return terrain_cell.x >= i32(0) && terrain_cell.x < i32(uniforms.viewport_width) && terrain_cell.y >= 0 && terrain_cell.y < i32(uniforms.terrain_buffer_height);
}

fn get_buffer_offset(cell: vec2<i32>) -> u32 {
  return u32(cell.y) * uniforms.level_width + u32(cell.x);
}

// Returns true if bounce occurred.
fn try_erode(terrain_cell: vec2<i32>, speed: f32) -> bool {
  let dmg_amt = i32(uniforms.damage_rate * speed);
  let actual_value = atomicAdd(&terrain_buffer[get_buffer_offset(terrain_cell)], -dmg_amt);
  return actual_value > 0;
}

fn norm(vel: vec2<f32>) -> f32{
  return sqrt(vel.x*vel.x + vel.y * vel.y);
}

let PI: f32 = 3.14159265358979323846;

@stage(compute) @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
  let total_particles = num_workgroups[0] * 256u;
  let gid = global_id[0];

  let particle = &(particle_buffer[gid]);
  if ((*particle).ttl <= 0.0) {
   return;
  } 

  // TODO collisions 
  let delta_pos = (*particle).velocity * uniforms.dt;
  (*particle).position = (*particle).position + delta_pos;
  (*particle).ttl = (*particle).ttl - uniforms.dt;

  // let current_cell = center + vec2<i32>(radius * vec2<f32>(cos(p * 2.0 * PI), sin(p * 2.0 * PI)));
  let current_cell = vec2<i32>((*particle).position);

  let terrain_cell = to_terrain_buffer(current_cell);

  if (!on_terrain_buffer(terrain_cell)) {
    (*particle).ttl = 0.0;
    return;
  } 

  // let buffer_location = &terrain_buffer[get_buffer_offset(current_cell)];
   // let buffer_location = &terrain_buffer[gid];
  // terrain_buffer[get_buffer_offset(current_cell)] = 0;
  try_erode(terrain_cell, norm((*particle).velocity));

  // Draw every particle.
  increment_cell(current_cell);
}