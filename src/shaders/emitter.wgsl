{% include "particle.wgsl.include" %}
{% include "grid.wgsl.include" %}

struct EmitterMotion {
    position_start: vec2<u32>;
    position_end: vec2<u32>;
    velocity: vec2<i32>;
    angle_start: f32;
    angle_end: f32;
};

struct NozzleParams {
    speed_min: f32;
    speed_max: f32;
    angle_spread: f32;
    ttl_min: f32;
    ttl_max: f32;
};

struct EmitData {
    start_index: u32;
    num_emitted: u32;
    time: f32;
    dt: f32;

    motion: EmitterMotion;
    nozzle: NozzleParams;
};

[[group(0), binding(0)]]
var<uniform> emit_data: EmitData;

struct Particles {
    particles: [[stride(24)]] array<Particle>;
};
[[group(0), binding(1)]]
var<storage, read_write> data: Particles;

[[stage(compute), workgroup_size(256)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>, [[builtin(num_workgroups)]] num_workgroups: vec3<u32>) {
    let total_particles = num_workgroups[0] * u32(512);
    let gid = global_id[0];

    data.particles[gid].position[0] = total_particles;
}