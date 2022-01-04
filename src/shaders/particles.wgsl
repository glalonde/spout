{% include "grid.wgsl.include" %}
{% include "particle.wgsl.include" %}

struct Particles {
    particles: [[stride(24)]] array<Particle>;
};
[[group(0), binding(0)]]
var<storage, read_write> data: Particles;

struct DensityBuffer {
    data: [[stride(4)]] array<u32>;
};
[[group(0), binding(1)]]
var<storage, read_write> data: DensityBuffer;


[[stage(compute), workgroup_size(256)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    let gid = global_id[0];

    let p = data.particles[gid];
    let current_cell = GetOuterGrid(p.position);
}