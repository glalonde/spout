{% include "grid.wgsl.include" %}
{% include "particle.wgsl.include" %}

struct Params {
    dt: f32;
    viewport_width: u32;
    viewport_height: u32;
    viewport_bottom_height: i32;
};
[[group(0), binding(0)]]
var<uniform> params: Params;

struct Particles {
    particles: [[stride(24)]] array<Particle>;
};
[[group(0), binding(1)]]
var<storage, read_write> data: Particles;

struct DensityBuffer {
    data: [[stride(4)]] array<u32>;
};
[[group(0), binding(2)]]
var<storage, read_write> data: DensityBuffer;

fn IncrementCell(cell: vec2<i32>) {
  cell.y -= i32(viewport_bottom_height);
  if (cell.x < 0 || cell.x >= i32(params.viewport_width) || cell.y < 0 || cell.y >= i32(params.viewport_height)) {
    return;
  }
  atomicAdd(density_buffer[cell.y * params.viewport_width + cell.x], 1);
}

[[stage(compute), workgroup_size(256)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    let gid = global_id[0];

    let p = data.particles[gid];
    let current_cell = p.position; // GetOuterGrid(p.position);
    IncrementCell(current_cell);
}