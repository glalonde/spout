{% include "grid.wgsl.include" %}
{% include "particle.wgsl.include" %}

// This shader module handles particle update and aggregation.
// It could possibly be combined with the emitter module... but for now it takes the particle_buffer, updates it, then aggregates particle densities into the density_buffer.

// IN:
struct UniformData {
    dt: f32,
    viewport_width: u32,
    viewport_height: u32,
    viewport_offset: i32,

    // level_width and viewport_width should be the same.
    level_width: u32,
    level_height: u32,

    terrain_buffer_offset: i32,
    terrain_buffer_height: u32,

    damage_rate: f32,
    gravity: f32,
    elasticity: f32,
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

fn global_to_terrain_buffer(cell: vec2<i32>) -> vec2<i32> {
  return vec2<i32>(cell.x, cell.y - uniforms.terrain_buffer_offset);
}

fn global_to_terrain_buffer_f32(pos: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(pos.x, pos.y - f32(uniforms.terrain_buffer_offset));
}

fn terrain_buffer_to_global(cell: vec2<i32>) -> vec2<i32> {
  return vec2<i32>(cell.x, cell.y + uniforms.terrain_buffer_offset);
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

fn copysign(in: f32) -> i32 {
  if (in >= 0.0) {
    return 1;
  } else {
    return -1;
  }
}

const PI: f32 = 3.14159265358979323846;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
  let total_particles = num_workgroups[0] * 256u;
  let gid = global_id[0];

  let particle = &(particle_buffer[gid]);
  if ((*particle).ttl <= 0.0) {
   return;
  } 

  let current_cell = vec2<i32>((*particle).position);
  var terrain_cell = global_to_terrain_buffer(current_cell);

  if (!on_terrain_buffer(terrain_cell)) {
    (*particle).ttl = 0.0;
    return;
  } 

  // To get smooth positioning between iterations, some newly emitted particles will have less than the uniform time delta.
  var dt = uniforms.dt;
  if ((*particle).local_dt >= 0.0) {
    dt = (*particle).local_dt;
    (*particle).local_dt = -1.0;
  }

  // TODO collisions 
  let signed_delta = (*particle).velocity * dt;
  let end_pos = (*particle).position + signed_delta;

  var delta = abs(signed_delta);
  var step = vec2<i32>(copysign(signed_delta.x), copysign(signed_delta.y));

  let end_cell = global_to_terrain_buffer(vec2<i32>(end_pos));
  let delta_i = end_cell - terrain_cell;

  // Starting cell remainder:
  let start_remainder = (vec2<f32>(0.5, 0.5) - (global_to_terrain_buffer_f32((*particle).position) - vec2<f32>(terrain_cell))) * vec2<f32>(step);
  var end_remainder = global_to_terrain_buffer_f32(end_pos) - vec2<f32>(end_cell);

  // 'Bresenham' Error value
  var error = delta.x * start_remainder.y - delta.y * start_remainder.x;
  var vel_out = (*particle).velocity;
  let speed = norm((*particle).velocity);

  var num_cells = abs(delta_i.x) + abs(delta_i.y);

  loop {
    if (num_cells <= 0) {
      break;
    }
    let error_horizontal = error - delta.y;
    let error_vertical = error + delta.x;
    if (error_vertical > -error_horizontal) {
      // Horizontal step
      error = error_horizontal;
      terrain_cell.x = terrain_cell.x + step.x;
      // Check cell
      let bounce = !on_terrain_buffer(terrain_cell) || try_erode(terrain_cell, speed);
      if (bounce) {
        // Bounce horizontally
        terrain_cell.x = terrain_cell.x - step.x;
        step.x = -1 * step.x;
        vel_out.x = -(vel_out.x * uniforms.elasticity);
        end_remainder.x = 1.0 - end_remainder.x;
      }
    } else {
      // Vertical step
      error = error_vertical;
      terrain_cell.y = terrain_cell.y + step.y;
      // Check cell
      let bounce = !on_terrain_buffer(terrain_cell) || try_erode(terrain_cell, speed);
      if (bounce) {
        // Bounce vertically 
        terrain_cell.y = terrain_cell.y - step.y;
        step.y = -1 * step.y;
        vel_out.y = -(vel_out.y * uniforms.elasticity);
        end_remainder.y = 1.0 - end_remainder.y;
      }
    }
    num_cells = num_cells - 1;
  }
  vel_out.y = vel_out.y + uniforms.gravity * dt;

  let global_output_pos = vec2<f32>(terrain_buffer_to_global(terrain_cell)) + end_remainder;
  (*particle).position = global_output_pos;
  (*particle).velocity = vel_out;
  (*particle).ttl = (*particle).ttl - dt;

  // Draw particle to density buffer.
  increment_cell(vec2<i32>(global_output_pos));
}